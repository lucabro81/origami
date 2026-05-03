pub mod props;
pub mod attrs;

use crate::{attrs::{attr_parser, attr_simple_expression_value_parser, attr_static_value_parser}, props::props_parser};

use std::sync::Arc;

use chumsky::prelude::*;
use origami_runtime::{
    codes, Attr, Body, ComponentNode, Declaration, EachNode, ExpressionNode, IfNode, LiteralNode,
    Node, OriFile, ParseError, SimpleExpression, SlotNode, Token, UnsafeNode,
};

/// Convert the token-stream slice into an [`OriFile`] AST, or return a list of
/// [`ParseError`]s carrying miette-compatible spans and source info.
pub fn parse(tokens: &[Token], filename: &str, src: Arc<String>) -> Result<OriFile, Vec<ParseError>> {
    let result = ori_file_parser().parse(tokens);
    if result.errors().next().is_none() {
        Ok(result.into_output().expect("no errors but also no output"))
    } else {
        let named = miette::NamedSource::new(filename, src);
        let errors = result
            .errors()
            .map(|e| {
                let span = e.span();
                ParseError::UnexpectedToken {
                    code: codes::P001.code,
                    message: codes::P001.message,
                    span: miette::SourceSpan::from(span.start..span.end),
                    src: named.clone(),
                }
            })
            .collect();
        Err(errors)
    }
}

pub fn node_expr_parser<'src>() -> impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> {
  attr_simple_expression_value_parser()
    .map(|value| Node::Expr(ExpressionNode { value }))
}

pub fn node_literal_static_parser<'src>() -> impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> {
  attr_static_value_parser()
    .map(|value| Node::Literal(LiteralNode { value }))
}

pub fn node_slot_parser<'src>() -> impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> {
  just(Token::Slot)
    .map(|_| Node::Slot(SlotNode{}))
}

pub fn node_unsafe_block_parser<'src>() -> impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> {
  just(Token::OpenUnsafe)
    .ignore_then(just(Token::Reason))
    .ignore_then(just(Token::AttrAssign))
    .ignore_then(select! { Token::ValueString(value) => value })
    .then_ignore(just(Token::EndTag))
    .then(select! { Token::UnsafeBlock(unsafe_block) => unsafe_block })
    .then_ignore(just(Token::CloseTag(String::from("unsafe"))))
    .map(|(reason, unsafe_block)| Node::Unsafe(UnsafeNode { reason, children: unsafe_block } ))
}

pub fn node_else_if_block_parser<'src>(node: impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> + Clone) -> impl Parser<'src, &'src [Token], IfNode, extra::Err<Rich<'src, Token>>> {
  just(Token::OpenElseIf)
    .ignore_then(just(Token::IfCondition))
    .ignore_then(just(Token::AttrAssign))
    .ignore_then(attr_simple_expression_value_parser())
    .then_ignore(just(Token::EndTag))
    .then(node.repeated().collect::<Vec<Node>>())
    .then_ignore(select! { Token::CloseTag(_) => () })
    .map(|(condition, then_children): (SimpleExpression, Vec<Node>)| IfNode {
      condition,
      then_children,
      else_if_children: vec![],
      else_child: None,
    })
}

pub fn node_if_block_parser<'src>(node: impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> + Clone) -> impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> {
  let else_if = node_else_if_block_parser(node.clone());

  let else_branch = just(Token::OpenElse)
    .ignore_then(just(Token::EndTag))
    .ignore_then(node.clone().repeated().collect::<Vec<Node>>())
    .then_ignore(select! { Token::CloseTag(_) => () });

  just(Token::OpenIf)
    .ignore_then(just(Token::IfCondition))
    .ignore_then(just(Token::AttrAssign))
    .ignore_then(attr_simple_expression_value_parser())
    .then_ignore(just(Token::EndTag))
    .then(node.repeated().collect::<Vec<Node>>())
    // closes </if> — elseIf and else follow as siblings outside this tag, not nested inside it
    .then_ignore(select! { Token::CloseTag(_) => () })
    .then(else_if.repeated().collect::<Vec<IfNode>>())
    .then(else_branch.or_not())
    .map(|(((condition, then_children), else_if_children), else_child)| Node::If(IfNode {
      condition,
      then_children,
      else_if_children,
      else_child,
    }))
}

pub fn node_each_block_parser<'src>(
  node: impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> + Clone,
) -> impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> {
  let index_alias = just(Token::IndexAs)
    .ignore_then(just(Token::AttrAssign))
    .ignore_then(select! { Token::Ident(name) => SimpleExpression::Var(name) })
    .or_not();

  just(Token::OpenEach)
    .ignore_then(just(Token::EachCollection))
    .ignore_then(just(Token::AttrAssign))
    .ignore_then(attr_simple_expression_value_parser())
    .then_ignore(just(Token::As))
    .then_ignore(just(Token::AttrAssign))
    .then(select! { Token::Ident(name) => name })
    .then(index_alias)
    .then_ignore(just(Token::EndTag))
    .then(node.repeated().collect::<Vec<Node>>())
    .then_ignore(select! { Token::CloseTag(_) => () })
    .map(|(((collection, alias), index_alias), children)| Node::Each(EachNode {
      collection,
      alias,
      index_alias,
      children,
    }))
}

pub fn node_parser<'src>() -> impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> {
  recursive::<_, _, extra::Err<Rich<'src, Token>>, _, _>(|node| {

    // boxed to make the type Clone-able — shared between autoclosing and open_close
    let attrs = attr_parser().repeated().collect::<Vec<Attr>>().boxed();

    let if_node = node_if_block_parser(node.clone());
    let each_node = node_each_block_parser(node.clone());

    let autoclosing = just(Token::StartTag)
      .ignore_then(select! { Token::Ident(name) => name })
      .then(attrs.clone())
      .then_ignore(just(Token::EndAutoclosingTag))
      .map(|(name, attrs)| Node::Component(ComponentNode {
        name,
        attrs,
        children: vec![]
      }));

    let open_close = just(Token::StartTag)
      .ignore_then(select! { Token::Ident(name) => name })
      .then(attrs)
      .then_ignore(just(Token::EndTag))
      .then(node.repeated().collect::<Vec<Node>>()) 
      .map(|((name, attrs), children)| Node::Component(ComponentNode {
        name,
        attrs,
        children
      }))
      .then_ignore(select! { Token::CloseTag(_) => () });

    let expr_node = node_expr_parser();
    let slot_node = node_slot_parser();
    let unsafe_node = node_unsafe_block_parser();

    autoclosing
      .or(open_close)
      .or(expr_node.boxed())
      .or(node_literal_static_parser().boxed())
      .or(slot_node.boxed())
      .or(unsafe_node.boxed())
      .or(if_node.boxed())
      .or(each_node.boxed())
  })
}

fn body_parser<'src>() -> impl Parser<'src, &'src [Token], Body, extra:: Err<Rich<'src, Token>>> {
  just(Token::OpenBody)
    // or_not: logic block absent when no code precedes ----
    .ignore_then(select! { Token::LogicBlock(block) => block}.or_not())
    .then_ignore(just(Token::Divider))
    .then(
      node_parser()
        .repeated()
        .collect::<Vec<Node>>()
      )
    .map(|(block, children)| Body {
      logic_block: block.unwrap_or(String::from("")),
      template: children
    })
  .then_ignore(just(Token::CloseBody))

}

fn layout_def_parser<'src>() -> impl Parser<'src, &'src [Token], Declaration, extra::Err<Rich<'src, Token>>> {
  just(Token::KwLayout)
    .ignore_then(select! { Token::Ident(name) => name })
    .then(body_parser())
    .map(|(name, body)| Declaration::Layout { 
      name, 
      body 
    })
}

fn page_def_parser<'src>() -> impl Parser<'src, &'src [Token], Declaration, extra::Err<Rich<'src, Token>>> {
  just(Token::KwPage)
    .ignore_then(select! { Token::Ident(name) => name })
    .then(props_parser())
    .then(body_parser())
    .map(|((name, props), body)| Declaration::Page { 
      name, 
      props, 
      body
    })
}

fn component_def_parser<'src>() -> impl Parser<'src, &'src [Token], Declaration, extra::Err<Rich<'src, Token>>> {
  just(Token::KwComponent)
    .ignore_then(select! { Token::Ident(name) => name })
    .then(props_parser())
    .then(body_parser())
    .map(|((name, props), body)| Declaration::Component { 
      name, 
      props, 
      body
    })
}

pub fn declaration_parser<'src>() -> impl Parser<'src, &'src [Token], Declaration, extra::Err<Rich<'src, Token>>> {
  layout_def_parser()
    .or(page_def_parser())
    .or(component_def_parser())
}

pub fn ori_file_parser<'src>() -> impl  Parser<'src, &'src [Token], OriFile, extra::Err<Rich<'src, Token>>> {
  declaration_parser()
    .repeated()
    .collect::<Vec<Declaration>>()
    .map(|declarations| OriFile { declarations })
    .then_ignore(just(Token::Eof))
}

#[cfg(test)] mod tests;
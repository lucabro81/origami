pub mod props;
pub mod attrs;

use crate::{attrs::{attr_parser, attr_simple_expression_value_parser}, props::props_parser};

use chumsky::{prelude::*};
use origami_runtime::{Attr, Body, ComponentNode, Declaration, ExpressionNode, Node, OriFile, SlotNode, Token};

pub fn node_expr_parser<'src>() -> impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> {
  attr_simple_expression_value_parser()
    .map(|expr| Node::Expr(ExpressionNode { value: expr }))
}

pub fn node_slot_parser<'src>() -> impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> {
  just(Token::Slot)
    .map(|_| Node::Slot(SlotNode{}))
}

pub fn node_parser<'src>() -> impl Parser<'src, &'src [Token], Node, extra::Err<Rich<'src, Token>>> {
  recursive::<_, _, extra::Err<Rich<'src, Token>>, _, _>(|node| {

    let attrs = attr_parser().repeated().collect::<Vec<Attr>>().boxed();

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

    autoclosing
      .or(open_close)
      .or(expr_node.boxed())
      .or(slot_node.boxed())
  })
}

fn body_parser<'src>() -> impl Parser<'src, &'src [Token], Body, extra:: Err<Rich<'src, Token>>> {
  just(Token::OpenBody)
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
      body: body
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
      body: body
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
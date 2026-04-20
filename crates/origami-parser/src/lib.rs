pub mod props;
pub mod attrs;

use crate::{attrs::attr_parser, props::props_parser};

use chumsky::{prelude::*};
use origami_runtime::{Attr, Body, ComponentNode, Declaration, Node, OriFile, Token};

pub fn autoclosing_node_parser<'src>() -> impl Parser<'src, &'src [Token], Node, extra:: Err<Rich<'src, Token>>> {
  just(Token::StartTag)
    .ignore_then(select! { Token::Ident(name) => name })
    .then(
      attr_parser()
      .repeated()
      .collect::<Vec<Attr>>()
    )
    .map(|(name, attrs)| Node::Component(ComponentNode {
      name,
      attrs,
      children: vec![]
    }))
  .then_ignore(just(Token::EndAutoclosingTag))
}

pub fn node_parser<'src>() -> impl Parser<'src, &'src [Token], Node, extra:: Err<Rich<'src, Token>>> {
  just(Token::StartTag)
    .ignore_then(select! { Token::Ident(name) => name })
    .then(
      attr_parser()
      .repeated()
      .collect::<Vec<Attr>>()
    )
    .then_ignore(just(Token::EndTag))
    .map(|(name, attrs)| Node::Component(ComponentNode {
      name,
      attrs,
      children: vec![]
    }))
  .then_ignore(select! { Token::CloseTag(name) => name })

}

fn body_parser<'src>() -> impl Parser<'src, &'src [Token], Body, extra:: Err<Rich<'src, Token>>> {
  just(Token::OpenBody)
    .ignore_then(select! { Token::LogicBlock(block) => block})
    .then_ignore(just(Token::Divider))
    .map(|block| Body {
      logic_block: block,
      template: vec![]
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
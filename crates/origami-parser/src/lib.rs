use chumsky::{prelude::*};
use origami_runtime::{Declaration, OriFile, Prop, Token};

pub fn prop_parser<'src>() -> impl Parser<'src, &'src [Token], Prop, extra::Err<Rich<'src, Token>>> {
  select! { Token::RawBlock(name) => name }
    .then_ignore(just(Token::TypeAssign))
    .then(select! { Token::RawBlock(name) => name })
    .map(|(name, type_str)| Prop { name, type_str })
}

pub fn props_parser<'src>() -> impl Parser<'src, &'src [Token], Vec<Prop>,extra::Err<Rich<'src, Token>>> {
  prop_parser()
    .separated_by(just(Token::CommaSeparator))
    .collect::<Vec<Prop>>()
    .delimited_by(just(Token::OpenArgs), just(Token::CloseArgs))
}

fn layout_def_parser<'src>() -> impl Parser<'src, &'src [Token], Declaration, extra::Err<Rich<'src, Token>>> {
  just(Token::KwLayout)
    .ignore_then(select! { Token::RawBlock(name) => name })
    .map(|name| Declaration::Layout { name })
}

fn page_def_parser<'src>() -> impl Parser<'src, &'src [Token], Declaration, extra::Err<Rich<'src, Token>>> {
  just(Token::KwPage)
    .ignore_then(select! { Token::RawBlock(name) => name })
    .then(props_parser())
    .map(|(name, props)| Declaration::Page { name, props })
}

fn component_def_parser<'src>() -> impl Parser<'src, &'src [Token], Declaration, extra::Err<Rich<'src, Token>>> {
  just(Token::KwComponent)
    .ignore_then(select! { Token::RawBlock(name) => name })
    .then(props_parser())
    .map(|(name, props)| Declaration::Component { name, props })
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
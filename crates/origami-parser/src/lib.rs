use chumsky::{prelude::*};
use origami_runtime::{Prop, Token};

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
#[cfg(test)] mod tests;
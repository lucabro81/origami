use chumsky::{prelude::*};
use origami_runtime::{Attr, AttrValue, SimpleExpression, Static, Token};

pub fn attr_static_string_value_parser<'src>() -> impl Parser<'src, &'src [Token], Static, extra::Err<Rich<'src, Token>>> {
  select! { Token::ValueString(value) => value }
    .map(|value| Static::String(value))
}

pub fn attr_static_int_value_parser<'src>() -> impl Parser<'src, &'src [Token], Static, extra::Err<Rich<'src, Token>>> {
  select! { Token::ValueNumber(value) => value }
    .try_map(|value, span| {
      value.parse::<i64>().map_err(|_| Rich::custom(span, "invalid integer"))
    })
    .map(|n| Static::NumberInt(n))
}

pub fn attr_static_float_value_parser<'src>() -> impl Parser<'src, &'src [Token], Static, extra::Err<Rich<'src, Token>>> {
  select! { Token::ValueNumber(value) => value }
    .try_map(|value, span| {
      value.parse::<f64>().map_err(|_| Rich::custom(span, "invalid float"))
    })
    .map(|n| Static::NumberFloat(n))
}

pub fn attr_static_value_parser<'src>() -> impl Parser<'src, &'src [Token], Static, extra::Err<Rich<'src, Token>>> {
  attr_static_string_value_parser()
    .or(attr_static_int_value_parser())
    .or(attr_static_float_value_parser())
}

pub fn attr_literal_value_parser<'src>() -> impl Parser<'src, &'src [Token], AttrValue, extra::Err<Rich<'src, Token>>> {
  attr_static_value_parser()
    .map(|value| AttrValue::Literal(value))
}

pub fn attr_simple_expression_var_value_parser<'src>() -> impl Parser<'src, &'src [Token], SimpleExpression, extra::Err<Rich<'src, Token>>> {
  just(Token::OpenExpr)
    .ignore_then(select! { Token::Ident(expr) => expr })
    .map(|expr| SimpleExpression::Var(expr))
    .then_ignore(just(Token::CloseExpr))
}

pub fn attr_simple_expression_dot_value_parser<'src>() -> impl Parser<'src, &'src [Token], SimpleExpression, extra::Err<Rich<'src, Token>>> {
  just(Token::OpenExpr)
    .ignore_then(
      select! { Token::Ident(head) => head }
        .then(
          just(Token::PeriodSeparator)
            .ignore_then(select! { Token::Ident(seg) => seg })
            .repeated()
            .at_least(1)
            .collect::<Vec<String>>()
        )
        .map(|(head, tail)| {
          tail.into_iter().fold(SimpleExpression::Var(head), |acc, seg| {
            SimpleExpression::Dot(Box::new(acc), seg)
          })
        })
    )
    .then_ignore(just(Token::CloseExpr))
}

pub fn attr_simple_expression_value_parser<'src>() -> impl Parser<'src, &'src [Token], SimpleExpression, extra::Err<Rich<'src, Token>>> {
  attr_simple_expression_dot_value_parser()
    .or(attr_simple_expression_var_value_parser())
}

pub fn attr_dynamic_value_parser<'src>() -> impl Parser<'src, &'src [Token], AttrValue, extra::Err<Rich<'src, Token>>> {
  attr_simple_expression_value_parser()
    .map(|value| AttrValue::Dynamic(value))
}

pub fn attr_unsafe_value_parser<'src>() -> impl Parser<'src, &'src [Token], AttrValue, extra::Err<Rich<'src, Token>>> {
  just(Token::OpenExpr)
    .ignore_then(just(Token::Unsafe))
      .ignore_then(just(Token::OpenArgs))
      .ignore_then(
        attr_static_string_value_parser()
        .or(attr_static_int_value_parser())
        .or(attr_static_float_value_parser())
      )
      .then_ignore(just(Token::CommaSeparator))
      .then(select! { Token::ValueString(reason) => reason })
      .map(|(value, reason)| AttrValue::UnsafeValue { value, reason })
    .then_ignore(just(Token::CloseArgs))
  .then_ignore(just(Token::CloseExpr))
}

pub fn attr_value_parser<'src>() -> impl Parser<'src, &'src [Token], AttrValue, extra::Err<Rich<'src, Token>>> {
  attr_literal_value_parser()
    .or(attr_dynamic_value_parser())
    .or(attr_unsafe_value_parser())
}

pub fn attr_parser<'src>() -> impl Parser<'src, &'src [Token], Attr, extra::Err<Rich<'src, Token>>> {
  select! { Token::Ident(name) => name }
    .then_ignore(just(Token::AttrAssign))
    .then(attr_value_parser())
    .map(|(name, value)| Attr { name, value })
}
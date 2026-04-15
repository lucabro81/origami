use chumsky::Parser;
use origami_runtime::{Prop, Token};

use crate::{prop_parser, props_parser};

#[test]
fn parse_prop() {
  let tokens = vec![
    Token::RawBlock(String::from("book")), 
    Token::TypeAssign, 
    Token::RawBlock(String::from("BookData"))
  ];
  let result = prop_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(Prop { name: "book".into(), type_str: "BookData".into() }));
}

#[test]
fn parse_prop_missing_type_assign() {
    let tokens = vec![
      Token::RawBlock(String::from("book")),
      Token::RawBlock(String::from("BookData")),
    ];
    assert!(prop_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_prop_missing_type() {
    let tokens = vec![
      Token::RawBlock(String::from("book")),
      Token::TypeAssign, 
    ];
    assert!(prop_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_prop_mistokened_name() {
    let tokens = vec![
      Token::TypeAssign, 
      Token::TypeAssign, 
      Token::RawBlock(String::from("BookData"))
    ];
    assert!(prop_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_prop_mistokened_type() {
    let tokens = vec![
      Token::RawBlock(String::from("book")), 
      Token::TypeAssign,
      Token::TypeAssign, 
    ];
    assert!(prop_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_prop_with_parenthesis() {
  let tokens = vec![
    Token::OpenArgs, 
    Token::RawBlock(String::from("book")), 
    Token::TypeAssign, 
    Token::RawBlock(String::from("BookData")), 
    Token::CloseArgs
  ];
    
  let result = props_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(vec![Prop { name: String::from("book"), type_str: String::from("BookData") }]));
}

#[test]
fn parse_props_with_parenthesis() {
  let tokens = vec![
    Token::OpenArgs, 
    Token::RawBlock(String::from("book")), 
    Token::TypeAssign, 
    Token::RawBlock(String::from("BookData")), 
    Token::CommaSeparator,
    Token::RawBlock(String::from("author")), 
    Token::TypeAssign, 
    Token::RawBlock(String::from("AuthorData")), 
    Token::CloseArgs
  ];
    
  let result = props_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(vec![
    Prop { name: String::from("book"), type_str: String::from("BookData") }, 
    Prop { name: String::from("author"), type_str: String::from("AuthorData") }
  ]));
}
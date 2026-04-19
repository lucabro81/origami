use chumsky::Parser;
use origami_runtime::{Body, ComponentNode, Declaration, Node, OriFile, Prop, Token};

use crate::{body_parser, declaration_parser, ori_file_parser, prop_parser, props_parser, simple_autoclosing_tag_parser, simple_tag_parser};

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

#[test]
fn parse_simple_autoclosing_tag() {
  let tokens = vec![
    Token::StartTag, 
    Token::RawBlock(String::from("Box")), 
    Token::EndAutoclosingTag
  ];

  let result = simple_autoclosing_tag_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Box"),
      attrs: vec![],
      children: vec![]
    })
  ));

}

#[test]
fn parse_simple_tag() {
  let tokens = vec![
    Token::StartTag, 
    Token::RawBlock(String::from("Box")), 
    Token::EndTag,
    Token::CloseTag(String::from("Box"))
  ];

  let result = simple_tag_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Box"),
      attrs: vec![],
      children: vec![]
    })
  ));

}

#[test]
fn parse_body() {
  let tokens = vec![
    Token::OpenBody,
      Token::LogicBlock(String::from("const test = 123;")),
      Token::Divider,
    Token::CloseBody,
  ];

  let result = body_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(
      Body {
        logic_block: String::from("const test = 123;"),
        template: vec![]
      }
    ));
}

#[test]
fn parse_component_def() {
  let tokens = vec![
    Token::KwComponent, 
    Token::RawBlock(String::from("Foo")),
    Token::OpenArgs,
    Token::RawBlock(String::from("book")), 
    Token::TypeAssign, 
    Token::RawBlock(String::from("BookData")),
    Token::CloseArgs,
    Token::OpenBody,
      Token::LogicBlock(String::from("const test = 123;")),
      Token::Divider,
    Token::CloseBody,
  ];

  let result = declaration_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(
      Declaration::Component { 
        name: String::from("Foo"), 
        props: vec![
          Prop { name: String::from("book"), type_str: String::from("BookData")}
        ],
        body: Body {
          logic_block: String::from("const test = 123;"),
          template: vec![]
        }
      }
    ));
}

#[test]
fn parse_layout_def() {
  let tokens = vec![
    Token::KwLayout, 
    Token::RawBlock(String::from("Foo")),
    Token::OpenBody,
      Token::LogicBlock(String::from("const test = 123;")),
      Token::Divider,
    Token::CloseBody,
  ];

  let result = declaration_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(
      Declaration::Layout { 
        name: String::from("Foo"),
        body: Body {
          logic_block: String::from("const test = 123;"),
          template: vec![]
        }
      }
    ));
}

#[test]
fn parse_page_def() {
  let tokens = vec![
    Token::KwPage, 
    Token::RawBlock(String::from("Foo")),
    Token::OpenArgs,
    Token::RawBlock(String::from("book")), 
    Token::TypeAssign, 
    Token::RawBlock(String::from("BookData")),
    Token::CloseArgs,
    Token::OpenBody,
      Token::LogicBlock(String::from("const test = 123;")),
      Token::Divider,
    Token::CloseBody,
  ];

  let result = declaration_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(
    Declaration::Page { 
      name: String::from("Foo"), 
      props: vec![
        Prop { name: String::from("book"), type_str: String::from("BookData")}
      ],
      body: Body {
        logic_block: String::from("const test = 123;"),
        template: vec![]
      }
    }
  ));
}

#[test]
fn parse_ori_file() {
  let tokens = [
    Token::KwComponent, 
    Token::RawBlock(String::from("Foo")),
    Token::OpenArgs,
      Token::RawBlock(String::from("book")), Token::TypeAssign, Token::RawBlock(String::from("BookData")),
      Token::CommaSeparator,
      Token::RawBlock(String::from("author")), Token::TypeAssign, Token::RawBlock(String::from("AuthorData")),
    Token::CloseArgs,
    Token::OpenBody,
      Token::LogicBlock(String::from("const test = 123;")),
      Token::Divider,
    Token::CloseBody,
    Token::Eof,
  ];

  let result = ori_file_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    OriFile {
      declarations: vec![
        Declaration::Component { 
          name: String::from("Foo"), 
          props: vec![
            Prop { name: String::from("book"), type_str: String::from("BookData")},
            Prop { name: String::from("author"), type_str: String::from("AuthorData")}
          ],
          body: Body {
            logic_block: String::from("const test = 123;"),
            template: vec![]
          }
        }
      ]
    }
  ));
}
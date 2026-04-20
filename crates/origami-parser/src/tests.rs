use chumsky::Parser;
use origami_runtime::{Attr, AttrValue, Body, ComponentNode, Declaration, Node, OriFile, Prop, SimpleExpression, Static, Token};

use crate::{
  attrs::{attr_parser, attr_simple_expression_dot_value_parser, attr_simple_expression_var_value_parser, attr_static_int_value_parser, attr_static_string_value_parser},
  body_parser, declaration_parser, ori_file_parser, props::prop_parser, props_parser, simple_autoclosing_tag_parser, simple_tag_parser,
};

#[test]
fn parse_prop() {
  let tokens = vec![
    Token::Ident(String::from("book")), 
    Token::TypeAssign, 
    Token::Ident(String::from("BookData"))
  ];
  let result = prop_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(Prop { name: "book".into(), type_str: "BookData".into() }));
}

#[test]
fn parse_prop_missing_type_assign() {
    let tokens = vec![
      Token::Ident(String::from("book")),
      Token::Ident(String::from("BookData")),
    ];
    assert!(prop_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_prop_missing_type() {
    let tokens = vec![
      Token::Ident(String::from("book")),
      Token::TypeAssign, 
    ];
    assert!(prop_parser().parse(&tokens).into_result().is_err());
}
 
#[test]
fn parse_prop_mistokened_name() {
    let tokens = vec![
      Token::TypeAssign, 
      Token::TypeAssign, 
      Token::Ident(String::from("BookData"))
    ];      
    assert!(prop_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_prop_mistokened_type() {
    let tokens = vec![
      Token::Ident(String::from("book")), 
      Token::TypeAssign,
      Token::TypeAssign, 
    ];
    assert!(prop_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_prop_with_parenthesis() {
  let tokens = vec![
    Token::OpenArgs, 
    Token::Ident(String::from("book")), 
    Token::TypeAssign, 
    Token::Ident(String::from("BookData")), 
    Token::CloseArgs
  ];
    
  let result = props_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(vec![Prop { name: String::from("book"), type_str: String::from("BookData") }]));
}

#[test]
fn parse_props_with_parenthesis() {
  let tokens = vec![
    Token::OpenArgs, 
    Token::Ident(String::from("book")), 
    Token::TypeAssign, 
    Token::Ident(String::from("BookData")), 
    Token::CommaSeparator,
    Token::Ident(String::from("author")), 
    Token::TypeAssign, 
    Token::Ident(String::from("AuthorData")), 
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
    Token::Ident(String::from("Box")), 
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
    Token::Ident(String::from("Box")), 
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
    Token::Ident(String::from("Foo")),
    Token::OpenArgs,
    Token::Ident(String::from("book")), 
    Token::TypeAssign, 
    Token::Ident(String::from("BookData")),
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
    Token::Ident(String::from("Foo")),
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
    Token::Ident(String::from("Foo")),
    Token::OpenArgs,
    Token::Ident(String::from("book")), 
    Token::TypeAssign, 
    Token::Ident(String::from("BookData")),
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
    Token::Ident(String::from("Foo")),
    Token::OpenArgs,
      Token::Ident(String::from("book")), Token::TypeAssign, Token::Ident(String::from("BookData")),
      Token::CommaSeparator,
      Token::Ident(String::from("author")), Token::TypeAssign, Token::Ident(String::from("AuthorData")),
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

// --- attr value parsers ---

#[test]
fn parse_attr_static_string_value() {
  let tokens = vec![Token::ValueString(String::from("\"hello\""))];
  let result = attr_static_string_value_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(Static::String(String::from("\"hello\""))));
}

#[test]
fn parse_attr_static_int_value() {
  let tokens = vec![Token::ValueNumber(String::from("42"))];
  let result = attr_static_int_value_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(Static::NumberInt(42)));
}

#[test]
fn parse_attr_static_int_wrong_token() {
  let tokens = vec![Token::ValueString(String::from("\"hello\""))];
  assert!(attr_static_int_value_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_attr_simple_expression_var() {
  let tokens = vec![
    Token::OpenExpr,
    Token::Ident(String::from("myVar")),
    Token::CloseExpr,
  ];
  let result = attr_simple_expression_var_value_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(SimpleExpression::Var(String::from("myVar"))));
}

#[test]
fn parse_attr_simple_expression_var_missing_close() {
  let tokens = vec![
    Token::OpenExpr,
    Token::Ident(String::from("myVar")),
  ];
  assert!(attr_simple_expression_var_value_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_attr_simple_expression_dot_two_segments() {
  // {{book.author}} → Dot(Var("book"), "author")
  let tokens = vec![
    Token::OpenExpr,
    Token::Ident(String::from("book")),
    Token::PeriodSeparator,
    Token::Ident(String::from("author")),
    Token::CloseExpr,
  ];
  let result = attr_simple_expression_dot_value_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(
    SimpleExpression::Dot(
      Box::new(SimpleExpression::Var(String::from("book"))),
      String::from("author"),
    )
  ));
}

#[test]
fn parse_attr_simple_expression_dot_three_segments() {
  // {{book.author.id}} → Dot(Dot(Var("book"), "author"), "id")
  let tokens = vec![
    Token::OpenExpr,
    Token::Ident(String::from("book")),
    Token::PeriodSeparator,
    Token::Ident(String::from("author")),
    Token::PeriodSeparator,
    Token::Ident(String::from("id")),
    Token::CloseExpr,
  ];
  let result = attr_simple_expression_dot_value_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(
    SimpleExpression::Dot(
      Box::new(SimpleExpression::Dot(
        Box::new(SimpleExpression::Var(String::from("book"))),
        String::from("author"),
      )),
      String::from("id"),
    )
  ));
}

#[test]
fn parse_attr_simple_expression_dot_missing_segment() {
  // {{book.}} → error
  let tokens = vec![
    Token::OpenExpr,
    Token::Ident(String::from("book")),
    Token::PeriodSeparator,
    Token::CloseExpr,
  ];
  assert!(attr_simple_expression_dot_value_parser().parse(&tokens).into_result().is_err());
}

// --- attr parser ---

#[test]
fn parse_attr_literal_string() {
  let tokens = vec![
    Token::Ident(String::from("color")),
    Token::AttrAssign,
    Token::ValueString(String::from("\"red\"")),
  ];
  let result = attr_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(Attr {
    name: String::from("color"),
    value: AttrValue::Literal(Static::String(String::from("\"red\"")))
  }));
}

#[test]
fn parse_attr_literal_int() {
  let tokens = vec![
    Token::Ident(String::from("size")),
    Token::AttrAssign,
    Token::ValueNumber(String::from("12")),
  ];
  let result = attr_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(Attr {
    name: String::from("size"),
    value: AttrValue::Literal(Static::NumberInt(12))
  }));
}

#[test]
fn parse_attr_missing_assign() {
  let tokens = vec![
    Token::Ident(String::from("color")),
    Token::ValueString(String::from("\"red\"")),
  ];
  assert!(attr_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_attr_missing_value() {
  let tokens = vec![
    Token::Ident(String::from("color")),
    Token::AttrAssign,
  ];
  assert!(attr_parser().parse(&tokens).into_result().is_err());
}
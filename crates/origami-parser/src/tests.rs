use std::sync::Arc;

use chumsky::Parser;
use miette::SourceSpan;
use origami_runtime::{Attr, AttrValue, Body, ComponentNode, Declaration, EachNode, ExpressionNode, IfNode, Node, OriFile, ParseError, Prop, SimpleExpression, SlotNode, Static, Token, UnsafeNode};

use crate::{
  attrs::{attr_parser, attr_simple_expression_dot_value_parser, attr_simple_expression_var_value_parser, attr_static_int_value_parser, attr_static_string_value_parser, attr_unsafe_value_parser},
  body_parser, declaration_parser, ori_file_parser, props::prop_parser, props_parser, node_parser, parse,
};

/// Build a SourceSpan from token-index offset and length.
fn sp(offset: usize, length: usize) -> SourceSpan {
    SourceSpan::new(miette::SourceOffset::from(offset), length)
}

// --- props parser ---

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

// --- template parser ---

#[test]
fn parse_simple_autoclosing_tag() {
  let tokens = vec![
    Token::StartTag,       // 0
    Token::Ident(String::from("Box")), // 1
    Token::EndAutoclosingTag           // 2
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Box"),
      attrs: vec![],
      children: vec![],
      span: sp(0, 3),
    })
  ));
}

#[test]
fn parse_autoclosing_tag_with_attrs() {
  let tokens = vec![
    Token::StartTag,
    Token::Ident(String::from("Box")),

    Token::Ident(String::from("width")),
      Token::AttrAssign,
      Token::ValueNumber(String::from("123")),

    Token::Ident(String::from("height")),
      Token::AttrAssign,
      Token::ValueNumber(String::from("32.1")),

    Token::Ident(String::from("title")),
      Token::AttrAssign,
      Token::ValueString(String::from("\"Un cavaliere per l'affascinante spia\"")),

    Token::Ident(String::from("author")),
      Token::AttrAssign,
      Token::OpenExpr,
            Token::Ident(String::from("book")),
            Token::PeriodSeparator,
            Token::Ident(String::from("author")),
      Token::CloseExpr,

    Token::Ident(String::from("size")),
      Token::AttrAssign,
      Token::OpenExpr,
        Token::Unsafe,
          Token::OpenArgs,
            Token::ValueNumber(String::from("42")),
            Token::CommaSeparator,
          Token::ValueString(String::from("\"needed for legacy API\"")),
        Token::CloseArgs,
      Token::CloseExpr,

    Token::EndAutoclosingTag
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Box"),
      attrs: vec![
        Attr { name: String::from("width"),  value: AttrValue::Literal(Static::NumberInt(123i64)), span: sp(2, 3) },
        Attr { name: String::from("height"), value: AttrValue::Literal(Static::NumberFloat(32.1f64)), span: sp(5, 3) },
        Attr { name: String::from("title"),  value: AttrValue::Literal(Static::String(String::from("\"Un cavaliere per l'affascinante spia\""))), span: sp(8, 3) },
        Attr {
          name: String::from("author"),
          value: AttrValue::Dynamic(
            SimpleExpression::Dot(Box::new(SimpleExpression::Var(String::from("book"))), String::from("author"))
          ),
          span: sp(11, 7),
        },
        Attr {
          name: String::from("size"),
          value: AttrValue::UnsafeValue { value: Static::NumberInt(42), reason: String::from("\"needed for legacy API\"") },
          span: sp(18, 10),
        },
      ],
      children: vec![],
      span: sp(0, 29),
    })
  ));
}

#[test]
fn parse_simple_tag() {
  let tokens = vec![
    Token::StartTag,                   // 0
    Token::Ident(String::from("Box")), // 1
    Token::EndTag,                     // 2
    Token::CloseTag(String::from("Box")) // 3
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Box"),
      attrs: vec![],
      children: vec![],
      span: sp(0, 4),
    })
  ));
}

#[test]
fn parse_simple_tag_with_attrs() {
  let tokens = vec![
    Token::StartTag,
      Token::Ident(String::from("Box")),

      Token::Ident(String::from("width")),
        Token::AttrAssign,
        Token::ValueNumber(String::from("123")),

      Token::Ident(String::from("height")),
        Token::AttrAssign,
        Token::ValueNumber(String::from("32.1")),

      Token::Ident(String::from("title")),
        Token::AttrAssign,
        Token::ValueString(String::from("\"Un cavaliere per l'affascinante spia\"")),

      Token::Ident(String::from("author")),
        Token::AttrAssign,
        Token::OpenExpr,
          Token::Ident(String::from("book")),
          Token::PeriodSeparator,
          Token::Ident(String::from("author")),
        Token::CloseExpr,

      Token::Ident(String::from("size")),
        Token::AttrAssign,
        Token::OpenExpr,
          Token::Unsafe,
            Token::OpenArgs,
              Token::ValueNumber(String::from("42")),
              Token::CommaSeparator,
            Token::ValueString(String::from("\"needed for legacy API\"")),
          Token::CloseArgs,
        Token::CloseExpr,
    Token::EndTag,
    Token::CloseTag(String::from("Box"))
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Box"),
      attrs: vec![
        Attr { name: String::from("width"),  value: AttrValue::Literal(Static::NumberInt(123i64)), span: sp(2, 3) },
        Attr { name: String::from("height"), value: AttrValue::Literal(Static::NumberFloat(32.1f64)), span: sp(5, 3) },
        Attr { name: String::from("title"),  value: AttrValue::Literal(Static::String(String::from("\"Un cavaliere per l'affascinante spia\""))), span: sp(8, 3) },
        Attr {
          name: String::from("author"),
          value: AttrValue::Dynamic(
            SimpleExpression::Dot(Box::new(SimpleExpression::Var(String::from("book"))), String::from("author"))
          ),
          span: sp(11, 7),
        },
        Attr {
          name: String::from("size"),
          value: AttrValue::UnsafeValue { value: Static::NumberInt(42), reason: String::from("\"needed for legacy API\"") },
          span: sp(18, 10),
        },
      ],
      children: vec![],
      span: sp(0, 30),
    })
  ));
}

#[test]
fn parse_template() {
  let tokens = vec![
    Token::StartTag,
      Token::Ident(String::from("Column")),
      Token::Ident(String::from("width")),
        Token::AttrAssign,
        Token::ValueNumber(String::from("123")),
    Token::EndTag,

      Token::StartTag,
        Token::Ident(String::from("Box")),
        Token::Ident(String::from("height")),
          Token::AttrAssign,
          Token::ValueNumber(String::from("1.23")),
      Token::EndTag,

        Token::StartTag,
          Token::Ident(String::from("Text")),
          Token::Ident(String::from("title")),
            Token::AttrAssign,
            Token::ValueString(String::from("\"Un cavaliere per l'affascinante spia\"")),
        Token::EndAutoclosingTag,

      Token::CloseTag(String::from("Box")),

      Token::StartTag,
        Token::Ident(String::from("Text")),
        Token::Ident(String::from("title")),
          Token::AttrAssign,
          Token::ValueString(String::from("\"Sedotta dal duca: la sua vendetta, il mio ventre, la nostra maledizione\"")),
      Token::EndAutoclosingTag,

    Token::CloseTag(String::from("Column"))
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Column"),
      attrs: vec![Attr { name: String::from("width"), value: AttrValue::Literal(Static::NumberInt(123i64)), span: sp(2, 3) }],
      children: vec![
        Node::Component(ComponentNode {
          name: String::from("Box"),
          attrs: vec![Attr { name: String::from("height"), value: AttrValue::Literal(Static::NumberFloat(1.23f64)), span: sp(8, 3) }],
          children: vec![
            Node::Component(ComponentNode {
              name: String::from("Text"),
              attrs: vec![Attr { name: String::from("title"), value: AttrValue::Literal(Static::String(String::from("\"Un cavaliere per l'affascinante spia\""))), span: sp(14, 3) }],
              children: vec![],
              span: sp(12, 6),
            })
          ],
          span: sp(6, 13),
        }),
        Node::Component(ComponentNode {
          name: String::from("Text"),
          attrs: vec![Attr { name: String::from("title"), value: AttrValue::Literal(Static::String(String::from("\"Sedotta dal duca: la sua vendetta, il mio ventre, la nostra maledizione\""))), span: sp(21, 3) }],
          children: vec![],
          span: sp(19, 6),
        })
      ],
      span: sp(0, 26),
    })
  ));
}

#[test]
fn parse_template_with_expr() {
  let tokens = vec![
    Token::StartTag,
      Token::Ident(String::from("Column")),
      Token::Ident(String::from("width")),
        Token::AttrAssign,
        Token::ValueNumber(String::from("123")),
    Token::EndTag,

    Token::OpenExpr,
      Token::Ident(String::from("book")),
      Token::PeriodSeparator,
      Token::Ident(String::from("author")),
    Token::CloseExpr,

    Token::OpenExpr,
      Token::Ident(String::from("simpleVar")),
    Token::CloseExpr,

    Token::CloseTag(String::from("Column"))
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Column"),
      attrs: vec![Attr { name: String::from("width"), value: AttrValue::Literal(Static::NumberInt(123i64)), span: sp(2, 3) }],
      children: vec![
        Node::Expr(ExpressionNode {
          value: SimpleExpression::Dot(Box::new(SimpleExpression::Var(String::from("book"))), String::from("author")),
          span: sp(6, 5),
        }),
        Node::Expr(ExpressionNode {
          value: SimpleExpression::Var(String::from("simpleVar")),
          span: sp(11, 3),
        })
      ],
      span: sp(0, 15),
    })
  ));
}

#[test]
fn parse_template_with_slot() {
  let tokens = vec![
    Token::StartTag,
      Token::Ident(String::from("Column")),
      Token::Ident(String::from("width")),
        Token::AttrAssign,
        Token::ValueNumber(String::from("123")),
    Token::EndTag,

    Token::Slot,

    Token::CloseTag(String::from("Column"))
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Column"),
      attrs: vec![Attr { name: String::from("width"), value: AttrValue::Literal(Static::NumberInt(123i64)), span: sp(2, 3) }],
      children: vec![Node::Slot(SlotNode { span: sp(6, 1) })],
      span: sp(0, 8),
    })
  ));
}

#[test]
fn parse_template_with_unsafe_block() {
  let tokens = vec![
    Token::StartTag,
      Token::Ident(String::from("Column")),
      Token::Ident(String::from("width")),
        Token::AttrAssign,
        Token::ValueNumber(String::from("123")),
    Token::EndTag,

    Token::OpenUnsafe, Token::Reason, Token::AttrAssign, Token::ValueString(String::from("\"xss\"")),
        Token::EndTag,
        Token::UnsafeBlock(String::from("test")),
        Token::CloseTag(String::from("unsafe")),

    Token::CloseTag(String::from("Column"))
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Column"),
      attrs: vec![Attr { name: String::from("width"), value: AttrValue::Literal(Static::NumberInt(123i64)), span: sp(2, 3) }],
      children: vec![
        Node::Unsafe(UnsafeNode { reason: String::from("\"xss\""), children: String::from("test"), span: sp(6, 7) })
      ],
      span: sp(0, 14),
    })
  ));
}

// --- <if ...> component parser ---

#[test]
fn parse_if_then_component() {
  let tokens = vec![
    Token::OpenIf,
      Token::IfCondition,
      Token::AttrAssign,
      Token::OpenExpr,
        Token::Ident(String::from("my")),
        Token::PeriodSeparator,
        Token::Ident(String::from("predicate")),
        Token::PeriodSeparator,
        Token::Ident(String::from("expression")),
      Token::CloseExpr,
    Token::EndTag,
      Token::StartTag,
        Token::Ident(String::from("Box")),
      Token::EndAutoclosingTag,
    Token::CloseTag(String::from("if"))
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::If(IfNode {
      condition: SimpleExpression::Dot(
        Box::new(SimpleExpression::Dot(
          Box::new(SimpleExpression::Var(String::from("my"))),
          String::from("predicate")
        )),
        String::from("expression")
      ),
      then_children: vec![
        Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(11, 3) })
      ],
      else_if_children: vec![],
      else_child: None,
      span: sp(0, 15),
    })
  ));
}

#[test]
fn parse_if_then_component_with_else() {
  let tokens = vec![
    Token::OpenIf,
      Token::IfCondition,
      Token::AttrAssign,
      Token::OpenExpr,
        Token::Ident(String::from("my")),
        Token::PeriodSeparator,
        Token::Ident(String::from("predicate")),
        Token::PeriodSeparator,
        Token::Ident(String::from("expression")),
      Token::CloseExpr,
    Token::EndTag,

      Token::StartTag,
        Token::Ident(String::from("Box")),
      Token::EndAutoclosingTag,

    Token::CloseTag(String::from("if")),
    Token::OpenElse,
    Token::EndTag,

      Token::StartTag,
        Token::Ident(String::from("Box")),
      Token::EndAutoclosingTag,

    Token::CloseTag(String::from("else"))
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::If(IfNode {
      condition: SimpleExpression::Dot(
        Box::new(SimpleExpression::Dot(
          Box::new(SimpleExpression::Var(String::from("my"))),
          String::from("predicate")
        )),
        String::from("expression")
      ),
      then_children: vec![
        Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(11, 3) })
      ],
      else_if_children: vec![],
      else_child: Option::Some(vec![
        Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(17, 3) })
      ]),
      span: sp(0, 21),
    })
  ));
}

#[test]
fn parse_if_then_component_with_elseif_and_else() {
  let tokens = vec![
    Token::OpenIf,
      Token::IfCondition,
      Token::AttrAssign,
      Token::OpenExpr,
        Token::Ident(String::from("my")),
        Token::PeriodSeparator,
        Token::Ident(String::from("predicate")),
        Token::PeriodSeparator,
        Token::Ident(String::from("expression")),
      Token::CloseExpr,
    Token::EndTag,

      Token::StartTag,
        Token::Ident(String::from("Box")),
      Token::EndAutoclosingTag,

    Token::CloseTag(String::from("if")),

    // first elseIf
    Token::OpenElseIf,
      Token::IfCondition,
        Token::AttrAssign,
        Token::OpenExpr,
          Token::Ident(String::from("my")),
          Token::PeriodSeparator,
          Token::Ident(String::from("predicate")),
          Token::PeriodSeparator,
          Token::Ident(String::from("expression")),
          Token::PeriodSeparator,
          Token::Ident(String::from("elseif1")),
        Token::CloseExpr,
      Token::EndTag,

      Token::StartTag,
        Token::Ident(String::from("Box")),
      Token::EndAutoclosingTag,

      Token::StartTag,
        Token::Ident(String::from("Box")),
      Token::EndAutoclosingTag,

    Token::CloseTag(String::from("elseIf")),

    // second elseIf
    Token::OpenElseIf,
      Token::IfCondition,
        Token::AttrAssign,
        Token::OpenExpr,
          Token::Ident(String::from("my")),
          Token::PeriodSeparator,
          Token::Ident(String::from("predicate")),
          Token::PeriodSeparator,
          Token::Ident(String::from("expression")),
          Token::PeriodSeparator,
          Token::Ident(String::from("elseif2")),
        Token::CloseExpr,
      Token::EndTag,

      Token::StartTag,
        Token::Ident(String::from("Box")),
      Token::EndAutoclosingTag,

      Token::StartTag,
        Token::Ident(String::from("Box")),
      Token::EndAutoclosingTag,

    Token::CloseTag(String::from("elseIf")),

    // else
    Token::OpenElse,
    Token::EndTag,

      Token::StartTag,
        Token::Ident(String::from("Box")),
      Token::EndAutoclosingTag,

    Token::CloseTag(String::from("else"))
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::If(IfNode {
      condition: SimpleExpression::Dot(
        Box::new(SimpleExpression::Dot(
          Box::new(SimpleExpression::Var(String::from("my"))),
          String::from("predicate")
        )),
        String::from("expression")
      ),
      then_children: vec![
        Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(11, 3) })
      ],
      else_if_children: vec![
        IfNode {
          condition: SimpleExpression::Dot(
            Box::new(SimpleExpression::Dot(
              Box::new(SimpleExpression::Dot(
                Box::new(SimpleExpression::Var(String::from("my"))),
                String::from("predicate")
              )),
              String::from("expression")
            )),
            String::from("elseif1")
          ),
          then_children: vec![
            Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(28, 3) }),
            Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(31, 3) }),
          ],
          else_if_children: vec![],
          else_child: None,
          span: sp(15, 20),
        },
        IfNode {
          condition: SimpleExpression::Dot(
            Box::new(SimpleExpression::Dot(
              Box::new(SimpleExpression::Dot(
                Box::new(SimpleExpression::Var(String::from("my"))),
                String::from("predicate")
              )),
              String::from("expression")
            )),
            String::from("elseif2")
          ),
          then_children: vec![
            Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(48, 3) }),
            Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(51, 3) }),
          ],
          else_if_children: vec![],
          else_child: None,
          span: sp(35, 20),
        },
      ],
      else_child: Option::Some(vec![
        Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(57, 3) })
      ]),
      span: sp(0, 61),
    })
  ));
}

// --- declaration parsers ---

#[test]
fn parse_body_with_empty_template() {
  let tokens = vec![
    Token::OpenBody,
      Token::LogicBlock(String::from("const test = 123;")),
      Token::Divider,
    Token::CloseBody,
  ];

  let result = body_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(Body { logic_block: String::from("const test = 123;"), template: vec![] }));
}

#[test]
fn parse_body_with_one_root() {
  let tokens = vec![
    Token::OpenBody,
      Token::LogicBlock(String::from("const test = 123;")),
      Token::Divider,

      Token::StartTag,
        Token::Ident(String::from("Box")),
        Token::Ident(String::from("height")),
          Token::AttrAssign,
          Token::ValueNumber(String::from("1.23")),
      Token::EndTag,

        Token::StartTag,
          Token::Ident(String::from("Text")),
          Token::Ident(String::from("title")),
            Token::AttrAssign,
            Token::ValueString(String::from("\"Un cavaliere per l'affascinante spia\"")),
        Token::EndAutoclosingTag,

      Token::CloseTag(String::from("Box")),

    Token::CloseBody,
  ];

  let result = body_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(
    Body {
      logic_block: String::from("const test = 123;"),
      template: vec![
        Node::Component(ComponentNode {
          name: String::from("Box"),
          attrs: vec![Attr { name: String::from("height"), value: AttrValue::Literal(Static::NumberFloat(1.23f64)), span: sp(5, 3) }],
          children: vec![
            Node::Component(ComponentNode {
              name: String::from("Text"),
              attrs: vec![Attr { name: String::from("title"), value: AttrValue::Literal(Static::String(String::from("\"Un cavaliere per l'affascinante spia\""))), span: sp(11, 3) }],
              children: vec![],
              span: sp(9, 6),
            })
          ],
          span: sp(3, 13),
        })
      ]
    }
  ));
}

#[test]
fn parse_body_with_no_logic_block() {
  let tokens = vec![
    Token::OpenBody,
      Token::Divider,

      Token::StartTag,
        Token::Ident(String::from("Text")),
        Token::Ident(String::from("title")),
          Token::AttrAssign,
          Token::ValueString(String::from("\"Un cavaliere per l'affascinante spia\"")),
      Token::EndAutoclosingTag,

    Token::CloseBody,
  ];

  let result = body_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(
    Body {
      logic_block: String::from(""),
      template: vec![
        Node::Component(ComponentNode {
          name: String::from("Text"),
          attrs: vec![Attr { name: String::from("title"), value: AttrValue::Literal(Static::String(String::from("\"Un cavaliere per l'affascinante spia\""))), span: sp(4, 3) }],
          children: vec![],
          span: sp(2, 6),
        })
      ]
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
      props: vec![Prop { name: String::from("book"), type_str: String::from("BookData") }],
      body: Body { logic_block: String::from("const test = 123;"), template: vec![] }
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
      body: Body { logic_block: String::from("const test = 123;"), template: vec![] }
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
      props: vec![Prop { name: String::from("book"), type_str: String::from("BookData") }],
      body: Body { logic_block: String::from("const test = 123;"), template: vec![] }
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
            Prop { name: String::from("book"), type_str: String::from("BookData") },
            Prop { name: String::from("author"), type_str: String::from("AuthorData") }
          ],
          body: Body { logic_block: String::from("const test = 123;"), template: vec![] }
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
  let tokens = vec![
    Token::OpenExpr,
    Token::Ident(String::from("book")),
    Token::PeriodSeparator,
    Token::Ident(String::from("author")),
    Token::CloseExpr,
  ];
  let result = attr_simple_expression_dot_value_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(
    SimpleExpression::Dot(Box::new(SimpleExpression::Var(String::from("book"))), String::from("author"))
  ));
}

#[test]
fn parse_attr_simple_expression_dot_three_segments() {
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
      Box::new(SimpleExpression::Dot(Box::new(SimpleExpression::Var(String::from("book"))), String::from("author"))),
      String::from("id"),
    )
  ));
}

#[test]
fn parse_attr_simple_expression_dot_missing_segment() {
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
    value: AttrValue::Literal(Static::String(String::from("\"red\""))),
    span: sp(0, 3),
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
    value: AttrValue::Literal(Static::NumberInt(12)),
    span: sp(0, 3),
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

// --- attr_unsafe_value_parser ---

#[test]
fn parse_attr_unsafe_value_int() {
  let tokens = vec![
    Token::OpenExpr,
    Token::Unsafe,
    Token::OpenArgs,
    Token::ValueNumber(String::from("42")),
    Token::CommaSeparator,
    Token::ValueString(String::from("\"needed for legacy API\"")),
    Token::CloseArgs,
    Token::CloseExpr,
  ];
  let result = attr_unsafe_value_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(AttrValue::UnsafeValue {
    value: Static::NumberInt(42),
    reason: String::from("\"needed for legacy API\""),
  }));
}

#[test]
fn parse_attr_unsafe_value_float() {
  let tokens = vec![
    Token::OpenExpr,
    Token::Unsafe,
    Token::OpenArgs,
    Token::ValueNumber(String::from("3.14")),
    Token::CommaSeparator,
    Token::ValueString(String::from("\"precision required\"")),
    Token::CloseArgs,
    Token::CloseExpr,
  ];
  let result = attr_unsafe_value_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(AttrValue::UnsafeValue {
    value: Static::NumberFloat(3.14),
    reason: String::from("\"precision required\""),
  }));
}

#[test]
fn parse_attr_unsafe_value_string() {
  let tokens = vec![
    Token::OpenExpr,
    Token::Unsafe,
    Token::OpenArgs,
    Token::ValueString(String::from("\"raw html\"")),
    Token::CommaSeparator,
    Token::ValueString(String::from("\"sanitized upstream\"")),
    Token::CloseArgs,
    Token::CloseExpr,
  ];
  let result = attr_unsafe_value_parser().parse(&tokens).into_result();
  assert_eq!(result, Ok(AttrValue::UnsafeValue {
    value: Static::String(String::from("\"raw html\"")),
    reason: String::from("\"sanitized upstream\""),
  }));
}

#[test]
fn parse_attr_unsafe_value_missing_reason() {
  let tokens = vec![
    Token::Unsafe,
    Token::OpenArgs,
    Token::ValueNumber(String::from("42")),
  ];
  assert!(attr_unsafe_value_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_attr_unsafe_value_missing_unsafe_keyword() {
  let tokens = vec![
    Token::OpenArgs,
    Token::ValueNumber(String::from("42")),
    Token::CommaSeparator,
    Token::ValueString(String::from("\"reason\"")),
  ];
  assert!(attr_unsafe_value_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_attr_unsafe_value_missing_open_args() {
  let tokens = vec![
    Token::Unsafe,
    Token::ValueNumber(String::from("42")),
    Token::CommaSeparator,
    Token::ValueString(String::from("\"reason\"")),
  ];
  assert!(attr_unsafe_value_parser().parse(&tokens).into_result().is_err());
}

// --- <each ...> parser ---

#[test]
fn parse_each_without_index_alias() {
  let tokens = vec![
    Token::OpenEach,
      Token::EachCollection, Token::AttrAssign,
      Token::OpenExpr, Token::Ident(String::from("books")), Token::CloseExpr,
      Token::As, Token::AttrAssign, Token::Ident(String::from("book")),
    Token::EndTag,
      Token::StartTag, Token::Ident(String::from("Box")), Token::EndAutoclosingTag,
    Token::CloseTag(String::from("each")),
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Each(EachNode {
      collection: SimpleExpression::Var(String::from("books")),
      alias: String::from("book"),
      index_alias: None,
      children: vec![
        Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(10, 3) }),
      ],
      span: sp(0, 14),
    })
  ));
}

#[test]
fn parse_each_with_index_alias() {
  let tokens = vec![
    Token::OpenEach,
      Token::EachCollection, Token::AttrAssign,
      Token::OpenExpr, Token::Ident(String::from("books")), Token::CloseExpr,
      Token::As, Token::AttrAssign, Token::Ident(String::from("book")),
      Token::IndexAs, Token::AttrAssign, Token::Ident(String::from("idx")),
    Token::EndTag,
      Token::StartTag, Token::Ident(String::from("Box")), Token::EndAutoclosingTag,
    Token::CloseTag(String::from("each")),
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Each(EachNode {
      collection: SimpleExpression::Var(String::from("books")),
      alias: String::from("book"),
      index_alias: Some(SimpleExpression::Var(String::from("idx"))),
      children: vec![
        Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(13, 3) }),
      ],
      span: sp(0, 17),
    })
  ));
}

#[test]
fn parse_each_with_dot_collection() {
  let tokens = vec![
    Token::OpenEach,
      Token::EachCollection, Token::AttrAssign,
      Token::OpenExpr,
        Token::Ident(String::from("page")), Token::PeriodSeparator, Token::Ident(String::from("books")),
      Token::CloseExpr,
      Token::As, Token::AttrAssign, Token::Ident(String::from("book")),
    Token::EndTag,
      Token::StartTag, Token::Ident(String::from("Box")), Token::EndAutoclosingTag,
    Token::CloseTag(String::from("each")),
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Each(EachNode {
      collection: SimpleExpression::Dot(Box::new(SimpleExpression::Var(String::from("page"))), String::from("books")),
      alias: String::from("book"),
      index_alias: None,
      children: vec![
        Node::Component(ComponentNode { name: String::from("Box"), attrs: vec![], children: vec![], span: sp(12, 3) }),
      ],
      span: sp(0, 16),
    })
  ));
}

#[test]
fn parse_each_missing_collection() {
  let tokens = vec![
    Token::OpenEach,
      Token::As, Token::AttrAssign, Token::Ident(String::from("book")),
    Token::EndTag,
    Token::CloseTag(String::from("each")),
  ];
  assert!(node_parser().parse(&tokens).into_result().is_err());
}

#[test]
fn parse_each_missing_alias() {
  let tokens = vec![
    Token::OpenEach,
      Token::EachCollection, Token::AttrAssign,
      Token::OpenExpr, Token::Ident(String::from("books")), Token::CloseExpr,
    Token::EndTag,
    Token::CloseTag(String::from("each")),
  ];
  assert!(node_parser().parse(&tokens).into_result().is_err());
}

// --- pub fn parse() ---

#[test]
fn parse_returns_ori_file_on_valid_tokens() {
    let tokens = vec![
        Token::KwLayout,
        Token::Ident(String::from("Default")),
        Token::OpenBody,
        Token::Divider,
        Token::CloseBody,
        Token::Eof,
    ];
    let src = Arc::new(String::from("layout Default {}"));
    let result = parse(&tokens, "test.ori", src);
    assert_eq!(result, Ok(OriFile {
        declarations: vec![
            Declaration::Layout {
                name: String::from("Default"),
                body: Body { logic_block: String::from(""), template: vec![] },
            }
        ]
    }));
}

#[test]
fn parse_returns_parse_errors_on_invalid_tokens() {
    let tokens = vec![
        Token::Ident(String::from("Card")),
        Token::Eof,
    ];
    let src = Arc::new(String::from("Card"));
    let result = parse(&tokens, "test.ori", src);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn parse_error_carries_p001_code() {
    let tokens = vec![
        Token::Ident(String::from("Card")),
        Token::Eof,
    ];
    let src = Arc::new(String::from("Card"));
    let errors = parse(&tokens, "test.ori", src).unwrap_err();
    // every error emitted by the generic conversion must use code P001
    for e in &errors {
        match e {
            ParseError::UnexpectedToken { code, .. } => assert_eq!(*code, "P001"),
            _ => panic!("expected UnexpectedToken variant"),
        }
    }
}

#[test]
fn parse_error_has_named_source() {
    let tokens = vec![Token::Eof];
    let src = Arc::new(String::from(""));
    let result = parse(&tokens, "empty.ori", src);
    assert!(result.is_ok());
}

#[test]
fn parse_empty_file_is_valid() {
    let tokens = vec![Token::Eof];
    let src = Arc::new(String::from(""));
    let result = parse(&tokens, "empty.ori", src);
    assert_eq!(result, Ok(OriFile { declarations: vec![] }));
}

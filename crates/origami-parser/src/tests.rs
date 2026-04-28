use chumsky::Parser;
use origami_runtime::{Attr, AttrValue, Body, ComponentNode, Declaration, ExpressionNode, LiteralNode, Node, OriFile, Prop, SimpleExpression, SlotNode, Static, TextNode, Token, UnsafeNode};

use crate::{
  attrs::{attr_parser, attr_simple_expression_dot_value_parser, attr_simple_expression_var_value_parser, attr_static_int_value_parser, attr_static_string_value_parser, attr_unsafe_value_parser},
  body_parser, declaration_parser, ori_file_parser, props::prop_parser, props_parser, node_parser,
};

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
    Token::StartTag, 
    Token::Ident(String::from("Box")), 
    Token::EndAutoclosingTag
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Box"),
      attrs: vec![],
      children: vec![]
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
        Attr { 
          name: String::from("width"), 
          value: AttrValue::Literal(Static::NumberInt(123i64)),
        },
        Attr { 
          name: String::from("height"), 
          value: AttrValue::Literal(Static::NumberFloat(32.1f64))
        },
        Attr { 
          name: String::from("title"), 
          value: AttrValue::Literal(Static::String(String::from("\"Un cavaliere per l'affascinante spia\"")))
        },
        Attr { 
          name: String::from("author"), 
          value: AttrValue::Dynamic(
            SimpleExpression::Dot(
              Box::new(SimpleExpression::Var(String::from("book"))), 
              String::from("author")
            )
          ) 
        },
        Attr { 
          name: String::from("size"), 
          value: AttrValue::UnsafeValue {
            value: Static::NumberInt(42),
            reason: String::from("\"needed for legacy API\""),
          }
        }
        
      ],
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

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Box"),
      attrs: vec![],
      children: vec![]
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
        Attr { 
          name: String::from("width"), 
          value: AttrValue::Literal(Static::NumberInt(123i64)),
        },
        Attr { 
          name: String::from("height"), 
          value: AttrValue::Literal(Static::NumberFloat(32.1f64))
        },
        Attr { 
          name: String::from("title"), 
          value: AttrValue::Literal(Static::String(String::from("\"Un cavaliere per l'affascinante spia\"")))
        },
        Attr { 
          name: String::from("author"), 
          value: AttrValue::Dynamic(
            SimpleExpression::Dot(
              Box::new(SimpleExpression::Var(String::from("book"))), 
              String::from("author")
            )
          ) 
        },
        Attr { 
          name: String::from("size"), 
          value: AttrValue::UnsafeValue {
            value: Static::NumberInt(42),
            reason: String::from("\"needed for legacy API\""),
          }
        }
        
      ],
      children: vec![]
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
      attrs: vec![
        Attr { 
          name: String::from("width"), 
          value: AttrValue::Literal(Static::NumberInt(123i64)),
        },
      ],
      children: vec![
        Node::Component(ComponentNode {
          name: String::from("Box"),
          attrs: vec![
            Attr { 
              name: String::from("height"), 
              value: AttrValue::Literal(Static::NumberFloat(1.23f64)),
            },
          ],
          children: vec![
            Node::Component(ComponentNode {
              name: String::from("Text"),
              attrs: vec![
                Attr { 
                  name: String::from("title"), 
                  value: AttrValue::Literal(Static::String(String::from("\"Un cavaliere per l'affascinante spia\""))),
                },
              ],
              children: vec![]
            })
          ]
        }),
        Node::Component(ComponentNode {
          name: String::from("Text"),
          attrs: vec![
            Attr { 
              name: String::from("title"), 
              value: AttrValue::Literal(Static::String(String::from("\"Sedotta dal duca: la sua vendetta, il mio ventre, la nostra maledizione\""))),
            },
          ],
          children: vec![]
        })
      ]
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
      attrs: vec![
        Attr { 
          name: String::from("width"), 
          value: AttrValue::Literal(Static::NumberInt(123i64)),
        },
      ],
      children: vec![
        Node::Expr(ExpressionNode {
          value: SimpleExpression::Dot(
            Box::new(SimpleExpression::Var(String::from("book"))), 
            String::from("author")
          )
        }),
        Node::Expr(ExpressionNode {
          value: SimpleExpression::Var(String::from("simpleVar"))
        })
      ]
    })
  ));
}

fn parse_template_with_static() {
  let tokens = vec![
    Token::StartTag, 
      Token::Ident(String::from("Column")), 
      Token::Ident(String::from("width")), 
        Token::AttrAssign,
        Token::ValueNumber(String::from("123")),
    Token::EndTag,

    Token::ValueString(String::from("value string")),
    Token::ValueNumber(String::from("123")),
    Token::ValueNumber(String::from("1.23")),

    Token::CloseTag(String::from("Column"))
  ];

  let result = node_parser().parse(&tokens).into_result();

  assert_eq!(result, Ok(
    Node::Component(ComponentNode {
      name: String::from("Column"),
      attrs: vec![
        Attr { 
          name: String::from("width"), 
          value: AttrValue::Literal(Static::NumberInt(123i64)),
        },
      ],
      children: vec![
        Node::Literal(LiteralNode { value: Static::String(String::from("value string")) }),
        Node::Literal(LiteralNode { value: Static::NumberInt(123i64) }),
        Node::Literal(LiteralNode { value: Static::NumberFloat(1.23f64) })
      ]
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
      attrs: vec![
        Attr { 
          name: String::from("width"), 
          value: AttrValue::Literal(Static::NumberInt(123i64)),
        },
      ],
      children: vec![
        Node::Slot(SlotNode {})
      ]
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
      attrs: vec![
        Attr { 
          name: String::from("width"), 
          value: AttrValue::Literal(Static::NumberInt(123i64)),
        },
      ],
      children: vec![
        Node::Unsafe(UnsafeNode { 
          reason: String::from("\"xss\""), 
          children: String::from("test") 
        })
      ]
    })
  ));
}

// #[test]
// fn parse_template_text_tag_autoclose() {
//   let tokens = vec![
//     Token::StartTag, 
//       Token::Ident(String::from("Text")), 
//       Token::Ident(String::from("value")), 
//         Token::AttrAssign,
//         Token::ValueString(String::from("This is a content")),
//     Token::EndAutoclosingTag
//   ];

//   let result = node_parser().parse(&tokens).into_result();

//   assert_eq!(result, Ok(
//     Node::Text(TextNode {
//       value: String::from("This is a content")
//     })
//   ));
// }

// #[test]
// fn parse_template_text_tag_open_close() {
//   let tokens = vec![
//     Token::StartTag, 
//       Token::Ident(String::from("Text")), 
//     Token::EndTag,
//       Token::ValueString(String::from("This is a content")),
//     Token::CloseTag(String::from("Text"))
//   ];

//   let result = node_parser().parse(&tokens).into_result();

//   assert_eq!(result, Ok(
//     Node::Component(ComponentNode {
//       name: String::from("Text"),
//       attrs: vec![],
//       children: vec![
//         Node::Literal(LiteralNode { 
//           value: String::from("This is a content")
//         })
//       ]
//     })
//   ));
// }


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
  assert_eq!(result, Ok(
      Body {
        logic_block: String::from("const test = 123;"),
        template: vec![]
      }
    ));
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
            attrs: vec![
              Attr { 
                name: String::from("height"), 
                value: AttrValue::Literal(Static::NumberFloat(1.23f64)),
              },
            ],
            children: vec![
              Node::Component(ComponentNode {
                name: String::from("Text"),
                attrs: vec![
                  Attr { 
                    name: String::from("title"), 
                    value: AttrValue::Literal(Static::String(String::from("\"Un cavaliere per l'affascinante spia\""))),
                  },
                ],
                children: vec![]
              })
            ]
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
            attrs: vec![
              Attr { 
                name: String::from("title"), 
                value: AttrValue::Literal(Static::String(String::from("\"Un cavaliere per l'affascinante spia\""))),
              },
            ],
            children: vec![]
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

// --- attr_usafe_value_parser ---

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
  // comma and reason are mandatory
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
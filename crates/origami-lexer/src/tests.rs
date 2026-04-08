use origami_runtime::Token;

use crate::lex;

#[test]
fn minimal_file() {
    let input = "component TestComponent {\n\t----\n\t<Column></Column>\n}\n";
    let tokens = lex(input).expect("lexer should not fail on valid input");
    assert_eq!(tokens, vec![
      Token::KwComponent, 
      Token::Name(String::from("TestComponent")),

      Token::OpenBody,

      Token::Logic(String::from("")),

      Token::Divider,

      Token::StartTag,
      Token::TagName(String::from("Column")),
      Token::EndTag,
      Token::CloseTag(String::from("Column")),

      Token::CloseBody,

      Token::Eof
    ])
}
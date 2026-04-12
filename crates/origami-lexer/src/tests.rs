use std::sync::Arc;

use miette::NamedSource;
use origami_runtime::{Token, codes, errors::PreprocessorError};

use crate::{lex, preprocess};

// Preprocess

#[test]
fn preprocess_no_opaque_blocks() {
    // `{` with no content before `----` → no substitution, no logic blocks
    let input = "component Foo {\n----\n<Col/>\n}";
    let result = preprocess(input, "<test>").unwrap();
    assert_eq!(result.sanitized, input);
    assert!(result.logic_blocks.is_empty());
}

#[test]
fn preprocess_logic_block() {
    let input = "component Foo {\nconst x = 1;\n----\n<Col/>\n}";
    let result = preprocess(input, "<test>").unwrap();
    assert_eq!(result.sanitized, "component Foo {\n__LOGIC__\n----\n<Col/>\n}");
    assert_eq!(result.logic_blocks, vec!["const x = 1;\n"]);
}

#[test]
fn preprocess_unsafe_block() {
    let input = r#"<unsafe reason="xss">alert("hi")</unsafe>"#;
    let result = preprocess(input, "<test>").unwrap();
    assert_eq!(result.sanitized, r#"<unsafe reason="xss">__UNSAFE__</unsafe>"#);
    assert!(result.logic_blocks.is_empty());
}

#[test]
fn preprocess_preserves_logic_content_verbatim() {
    // Regression: logic block content must be preserved exactly, including spacing
    let input = "component Foo {\n  const x  =  1;\n----\n}";
    let result = preprocess(input, "<test>").unwrap();
    assert_eq!(result.logic_blocks[0], "  const x  =  1;\n");
}

#[test]
fn preprocess_offset_map_logic_block() {
    // input:     "component Foo {\nconst x = 1;\n----\n}"
    //             offset of `{` = 14, placeholder starts at sanitized offset 16 (`{\n` = 2 bytes)
    // original content = "const x = 1;\n" = 14 bytes
    // placeholder = "__LOGIC__\n" = 10 bytes
    // delta = 14 - 10 = 4
    let input = "component Foo {\nconst x = 1;\n----\n}";
    let result = preprocess(input, "<test>").unwrap();
    // sanitized: "component Foo {\n__LOGIC__\n----\n}"
    // __LOGIC__ starts at offset 16
    assert_eq!(result.offset_map, vec![(16, 4i64)]);
}

#[test]
fn preprocess_offset_map_no_substitution() {
    let input = "component Foo {\n----\n}";
    let result = preprocess(input, "<test>").unwrap();
    assert!(result.offset_map.is_empty());
}

#[test]
fn preprocess_offset_map_unsafe_block() {
    // input:     `<unsafe reason="xss">alert("hi")</unsafe>`
    // content between `>` and `</unsafe>` = `alert("hi")` = 11 bytes
    // placeholder = `__UNSAFE__` = 10 bytes
    // delta = 11 - 10 = 1
    // `>` is at offset 20, content starts at 21
    let input = r#"<unsafe reason="xss">alert("hi")</unsafe>"#;
    let result = preprocess(input, "<test>").unwrap();
    // __UNSAFE__ starts at sanitized offset 21
    assert_eq!(result.offset_map, vec![(21, 1i64)]);
}

#[test]
fn preprocess_symbol_not_found() {
    // Logic block opened with `{` but no `----` separator anywhere → PP001
    // Span points to the `{` at offset 24.
    let input = "component TestComponent {\nconst x = 1;\n<Column></Column>\n}";
    let src = Arc::new(input.to_string());
    assert_eq!(
        preprocess(input, "<test>"),
        Err(PreprocessorError::SymbolNotFound {
            code: codes::PP001.code,
            message: codes::PP001.message,
            span: (24usize, 1usize).into(),
            src: NamedSource::new("<test>", src),
        })
    );
}

#[test]
fn preprocess_displaced_token_inline_after_code() {
    // `----` on the same line as logic code → PP002
    // Span points to `----` at offset 38.
    let input = "component TestComponent {\nconst x = 1;----\n<Column></Column>\n}";
    let src = Arc::new(input.to_string());
    assert_eq!(
        preprocess(input, "<test>"),
        Err(PreprocessorError::DisplacedToken {
            code: codes::PP002.code,
            message: codes::PP002.message,
            span: (38usize, 4usize).into(),
            src: NamedSource::new("<test>", src),
        })
    );
}

#[test]
fn preprocess_displaced_token_inline_before_template() {
    // `----` on the same line as template content → PP002
    // Span points to `----` at offset 39.
    let input = "component TestComponent {\nconst x = 1;\n----<Column></Column>\n}";
    let src = Arc::new(input.to_string());
    assert_eq!(
        preprocess(input, "<test>"),
        Err(PreprocessorError::DisplacedToken {
            code: codes::PP002.code,
            message: codes::PP002.message,
            span: (39usize, 4usize).into(),
            src: NamedSource::new("<test>", src),
        })
    );
}

// Lexer

#[test]
fn minimal_file() {
    let sanitized = "component TestComponent {\n----\n<Column></Column>\n}";
    let preprocessed = crate::PreprocessResult {
        sanitized: sanitized.to_string(),
        logic_blocks: vec![],
        offset_map: vec![],
        src: NamedSource::new("<test>", Arc::new(sanitized.to_string())),
    };
    let tokens = lex(preprocessed).expect("lexer should not fail on valid input");
    assert_eq!(tokens, vec![
        Token::KwComponent, Token::RawBlock(String::from("TestComponent")), Token::OpenBody,
        Token::Divider,
        Token::StartTag, Token::RawBlock(String::from("Column")), Token::EndTag,
        Token::CloseTag(String::from("Column")),
        Token::CloseBody,
        Token::Eof,
    ]);
}

#[test]
fn minimal_file_with_logic_block() {
    let preprocessed = crate::PreprocessResult {
        sanitized: "component TestComponent {\n__LOGIC__\n----\n<Column></Column>\n}".to_string(),
        logic_blocks: vec!["const test = 13;\n".to_string()],
        offset_map: vec![],
        src: NamedSource::new("<test>", Arc::new(String::new())),
    };
    let tokens = lex(preprocessed).expect("lexer should not fail on valid input");
    assert_eq!(tokens, vec![
        Token::KwComponent, Token::RawBlock(String::from("TestComponent")), Token::OpenBody,
        Token::LogicBlock(String::from("const test = 13;\n")),
        Token::Divider,
        Token::StartTag, Token::RawBlock(String::from("Column")), Token::EndTag,
        Token::CloseTag(String::from("Column")),
        Token::CloseBody,
        Token::Eof,
    ]);
}
use std::sync::Arc;

use miette::NamedSource;
use origami_runtime::{Token, codes, errors::{PreprocessorError, LexError}};

use crate::{correct_span, lex, preprocess};

// correct_span

#[test]
fn correct_span_empty_offset_map() {
    // "component Foo {\n----\n}" — no substitutions, span passes through unchanged
    assert_eq!(correct_span(10..15, &[]), 10..15);
}

#[test]
fn correct_span_positive_delta_before_span() {
    // "component Foo {\nconst x = 1;\n----\n}" — logic block replaced with __LOGIC__
    // offset_map = [(16, +4)]: original "const x = 1;\n" (14 bytes) → "__LOGIC__\n" (10 bytes), delta = +4
    // A span at sanitized offset 26 (the `----`) maps to 26 + 4 = 30 in original
    assert_eq!(correct_span(26..30, &[(16, 4)]), 30..34);
}

#[test]
fn correct_span_negative_delta_before_span() {
    // "...<unsafe reason="test reason"></unsafe>..." — empty unsafe content replaced with __UNSAFE__ (10 bytes)
    // offset_map = [(79, -10)]: content was 0 bytes, placeholder 10 bytes, delta = -10
    // logos raw span 100..101 on the `/` of `</Column` in sanitized → 100 + (-10) = 90...
    // but with both deltas in the real test: see correct_span_two_deltas_lexer_case below
    assert_eq!(correct_span(99..100, &[(79, -10)]), 89..90);
}

#[test]
fn correct_span_delta_after_span_is_ignored() {
    // Delta at pos 79 does not affect a span at pos 50 (before the substitution)
    assert_eq!(correct_span(50..51, &[(79, -10)]), 50..51);
}

#[test]
fn correct_span_two_deltas_lexer_case() {
    // "component TestComponent {\nconst test = 13;}\n----\n<Column>\n<unsafe reason="test reason"></unsafe>\n</Column\n}"
    // offset_map = [(26, +9), (79, -10)]
    // logos raw span on `/` of `</Column` in sanitized: 100..101
    // delta_sum = +9 + (-10) = -1 → 99..100 in original
    // original[99] = '/' of `</Column`
    assert_eq!(correct_span(100..101, &[(26, 9), (79, -10)]), 99..100);
}

#[test]
fn correct_span_delta_exactly_at_span_start() {
    // Delta pos == span.start: the filter uses `<=` so the delta must be applied
    // offset_map = [(50, +5)], span = 50..51 → 55..56
    assert_eq!(correct_span(50..51, &[(50, 5)]), 55..56);
}

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

#[test]
fn preprocess_two_logic_blocks() {
    // Two components in the same file → two separate logic blocks collected in order
    let input = "component Foo {\nconst x = 1;\n----\n<Col/>\n}\ncomponent Bar {\nconst y = 2;\n----\n<Row/>\n}";
    let result = preprocess(input, "<test>").unwrap();
    assert_eq!(result.logic_blocks, vec!["const x = 1;\n", "const y = 2;\n"]);
    assert_eq!(result.sanitized, "component Foo {\n__LOGIC__\n----\n<Col/>\n}\ncomponent Bar {\n__LOGIC__\n----\n<Row/>\n}");
}

#[test]
fn preprocess_unsafe_inside_logic_block_is_not_substituted() {
    // `<unsafe>` appearing inside a logic block is captured verbatim as logic content —
    // the preprocessor replaces the whole logic block with __LOGIC__ before it can see the unsafe tag
    let input = "component Foo {\nconst x = <unsafe reason=\"xss\">bad</unsafe>;\n----\n<Col/>\n}";
    let result = preprocess(input, "<test>").unwrap();
    assert_eq!(result.logic_blocks, vec!["const x = <unsafe reason=\"xss\">bad</unsafe>;\n"]);
    assert_eq!(result.sanitized, "component Foo {\n__LOGIC__\n----\n<Col/>\n}");
}

#[test]
fn preprocess_double_brace_in_template_not_treated_as_logic_block() {
    // Regression: `{{` in the template section was incorrectly triggering logic-block detection.
    // `{{` is always an expression delimiter — the preprocessor must skip it entirely.
    let input = "component Foo {\nconst x = 1;\n----\n<Col value={{x}}/>\n}";
    let result = preprocess(input, "<test>").unwrap();
    assert_eq!(result.logic_blocks, vec!["const x = 1;\n"]);
    assert_eq!(result.sanitized, "component Foo {\n__LOGIC__\n----\n<Col value={{x}}/>\n}");
}

// Lexer

#[test]
fn component_with_single_prop() {
    // Signature with one prop: `component Card(title: string) { ---- <Box/> }`
    // Tokens for the prop signature: OpenArgs Ident("title") TypeAssign Ident("string") CloseArgs
    let preprocessed = crate::PreprocessResult {
        sanitized: "component Card(title: string) {\n----\n<Box/>\n}".to_string(),
        logic_blocks: vec![],
        offset_map: vec![],
        src: NamedSource::new("<test>", Arc::new(String::new())),
    };
    let tokens = lex(preprocessed).unwrap();
    assert_eq!(tokens, vec![
        Token::KwComponent, Token::Ident(String::from("Card")),
        Token::OpenArgs,
        Token::Ident(String::from("title")), Token::TypeAssign, Token::Ident(String::from("string")),
        Token::CloseArgs,
        Token::OpenBody,
        Token::Divider,
        Token::StartTag, Token::Ident(String::from("Box")), Token::EndAutoclosingTag,
        Token::CloseBody,
        Token::Eof,
    ]);
}

#[test]
fn component_with_multiple_props_and_logic() {
    // Full pipeline: two props, logic block, template referencing both props via expressions
    let input = "component BookCard(title: string, author: string) {\nconst label = title;\n----\n<Box>\n<Text value={{label}}/>\n</Box>\n}";
    let tokens = lex(preprocess(input, "<test>").unwrap()).unwrap();
    assert_eq!(tokens, vec![
        Token::KwComponent, Token::Ident(String::from("BookCard")),
        Token::OpenArgs,
        Token::Ident(String::from("title")), Token::TypeAssign, Token::Ident(String::from("string")),
        Token::CommaSeparator,
        Token::Ident(String::from("author")), Token::TypeAssign, Token::Ident(String::from("string")),
        Token::CloseArgs,
        Token::OpenBody,
        Token::LogicBlock(String::from("const label = title;\n")),
        Token::Divider,
        Token::StartTag, Token::Ident(String::from("Box")), Token::EndTag,
        Token::StartTag, Token::Ident(String::from("Text")),
        Token::Ident(String::from("value")), Token::AttrAssign,
        Token::OpenExpr, Token::Ident(String::from("label")), Token::CloseExpr,
        Token::EndAutoclosingTag,
        Token::CloseTag(String::from("Box")),
        Token::CloseBody,
        Token::Eof,
    ]);
}

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
        Token::KwComponent, Token::Ident(String::from("TestComponent")), Token::OpenBody,
        Token::Divider,
        Token::StartTag, Token::Ident(String::from("Column")), Token::EndTag,
        Token::CloseTag(String::from("Column")),
        Token::CloseBody,
        Token::Eof,
    ]);
}

#[test]
fn tempalte_with_number_attr() {
    let sanitized = "component TestComponent {\n----\n<Column col=123></Column>\n}";
    let preprocessed = crate::PreprocessResult {
        sanitized: sanitized.to_string(),
        logic_blocks: vec![],
        offset_map: vec![],
        src: NamedSource::new("<test>", Arc::new(sanitized.to_string())),
    };
    let tokens = lex(preprocessed).expect("lexer should not fail on valid input");
    assert_eq!(tokens, vec![
        Token::KwComponent, Token::Ident(String::from("TestComponent")), Token::OpenBody,
        Token::Divider,
        Token::StartTag, 
            Token::Ident(String::from("Column")), 
            Token::Ident(String::from("col")), Token::AttrAssign, Token::ValueNumber(String::from("123")),
        Token::EndTag,
        Token::CloseTag(String::from("Column")),
        Token::CloseBody,
        Token::Eof,
    ]);
}

#[test]
fn tempalte_with_mixed_attr() {
    let sanitized = "component TestComponent {\n----\n<Column col=123 row=\"test\"></Column>\n}";
    let preprocessed = crate::PreprocessResult {
        sanitized: sanitized.to_string(),
        logic_blocks: vec![],
        offset_map: vec![],
        src: NamedSource::new("<test>", Arc::new(sanitized.to_string())),
    };
    let tokens = lex(preprocessed).expect("lexer should not fail on valid input");
    assert_eq!(tokens, vec![
        Token::KwComponent, Token::Ident(String::from("TestComponent")), Token::OpenBody,
        Token::Divider,
        Token::StartTag, 
            Token::Ident(String::from("Column")), 

            Token::Ident(String::from("col")), Token::AttrAssign, Token::ValueNumber(String::from("123")),
            
            Token::Ident(String::from("row")), Token::AttrAssign, Token::ValueString(String::from("\"test\"")),
        Token::EndTag,
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
        Token::KwComponent, Token::Ident(String::from("TestComponent")), Token::OpenBody,
        Token::LogicBlock(String::from("const test = 13;\n")),
        Token::Divider,
        Token::StartTag, Token::Ident(String::from("Column")), Token::EndTag,
        Token::CloseTag(String::from("Column")),
        Token::CloseBody,
        Token::Eof,
    ]);
}

#[test]
fn minimal_file_with_logic_block_and_unsafe_block() {
    let preprocessed = crate::PreprocessResult {
        sanitized: "component TestComponent {\n__LOGIC__\n----\n<Column>\n__UNSAFE__\n</Column>\n}".to_string(),
        logic_blocks: vec!["const test = 13;\n".to_string()],
        offset_map: vec![],
        src: NamedSource::new("<test>", Arc::new(String::new())),
    };
    let tokens = lex(preprocessed).expect("lexer should not fail on valid input");
    assert_eq!(tokens, vec![
        Token::KwComponent, Token::Ident(String::from("TestComponent")), Token::OpenBody,
        Token::LogicBlock(String::from("const test = 13;\n")),
        Token::Divider,
        Token::StartTag, Token::Ident(String::from("Column")), Token::EndTag,
        Token::UnsafeBlock(String::from("")),
        Token::CloseTag(String::from("Column")),
        Token::CloseBody,
        Token::Eof,
    ]);
}

#[test]
fn two_components_correct_tokens_and_logic_blocks() {
    // Full pipeline: two components, each with a logic block
    let input = "component Foo {\nconst x = 1;\n----\n<Col/>\n}\ncomponent Bar {\nconst y = 2;\n----\n<Row/>\n}";
    let tokens = lex(preprocess(input, "<test>").unwrap()).unwrap();
    assert_eq!(tokens, vec![
        Token::KwComponent, Token::Ident(String::from("Foo")), Token::OpenBody,
        Token::LogicBlock(String::from("const x = 1;\n")),
        Token::Divider,
        Token::StartTag, Token::Ident(String::from("Col")), Token::EndAutoclosingTag,
        Token::CloseBody,
        Token::KwComponent, Token::Ident(String::from("Bar")), Token::OpenBody,
        Token::LogicBlock(String::from("const y = 2;\n")),
        Token::Divider,
        Token::StartTag, Token::Ident(String::from("Row")), Token::EndAutoclosingTag,
        Token::CloseBody,
        Token::Eof,
    ]);
}

#[test]
fn two_components_error_in_second_has_correct_span() {
    // Error in second component — offset_map from first substitution must still apply
    // First logic block "const x = 1;\n" (13 bytes) → "__LOGIC__\n" (10 bytes), delta = +4 at pos 16
    // `@` is invalid; in sanitized it appears after both components' headers
    // "component Foo {\n__LOGIC__\n----\n<Col/>\n}\ncomponent Bar {\n__LOGIC__\n----\n@\n}"
    //  offset of `@` in sanitized: let's get it from the real output
    let input = "component Foo {\nconst x = 1;\n----\n<Col/>\n}\ncomponent Bar {\nconst y = 2;\n----\n@\n}";
    let preprocessed = preprocess(input, "<test>").unwrap();
    let result = lex(preprocessed);
    // The `@` token is an Event only when followed by alpha chars; bare `@\n` is unexpected
    // We just verify the error code and that the span falls inside the second component
    if let Err(LexError::UnexpectedChar { code, span, .. }) = result {
        assert_eq!(code, codes::L001.code);
        // `@` must be past the end of first component in the original
        let offset: usize = span.offset().into();
        assert!(offset > input.find('}').unwrap(), "span should be in second component");
    } else {
        panic!("expected UnexpectedChar error");
    }
}

#[test]
fn unsafe_block_with_reason_attr() {
    // `<unsafe reason="xss">` is tokenised correctly including the reason attribute
    let preprocessed = crate::PreprocessResult {
        sanitized: "component Foo {\n----\n<unsafe reason=\"xss\">__UNSAFE__</unsafe>\n}".to_string(),
        logic_blocks: vec![],
        offset_map: vec![],
        src: NamedSource::new("<test>", Arc::new(String::new())),
    };
    let tokens = lex(preprocessed).unwrap();
    assert_eq!(tokens, vec![
        Token::KwComponent, Token::Ident(String::from("Foo")), Token::OpenBody,
        Token::Divider,
        Token::OpenUnsafe, Token::Reason, Token::AttrAssign, Token::ValueString(String::from("\"xss\"")),
        Token::EndTag,
        Token::UnsafeBlock(String::from("")),
        Token::CloseTag(String::from("unsafe")),
        Token::CloseBody,
        Token::Eof,
    ]);
}

#[test]
fn nested_tags() {
    // Template with nested tags produces correct token sequence
    let preprocessed = crate::PreprocessResult {
        sanitized: "component Foo {\n----\n<Col>\n<Row/>\n</Col>\n}".to_string(),
        logic_blocks: vec![],
        offset_map: vec![],
        src: NamedSource::new("<test>", Arc::new(String::new())),
    };
    let tokens = lex(preprocessed).unwrap();
    assert_eq!(tokens, vec![
        Token::KwComponent, Token::Ident(String::from("Foo")), Token::OpenBody,
        Token::Divider,
        Token::StartTag, Token::Ident(String::from("Col")), Token::EndTag,
        Token::StartTag, Token::Ident(String::from("Row")), Token::EndAutoclosingTag,
        Token::CloseTag(String::from("Col")),
        Token::CloseBody,
        Token::Eof,
    ]);
}

#[test]
fn unsafe_as_prop_value() {
    // `unsafe('value', 'reason')` used as a prop value on a component
    let preprocessed = crate::PreprocessResult {
        sanitized: "component Foo {\n----\n<Col color=unsafe(\"red\", \"design exception\")/>}".to_string(),
        logic_blocks: vec![],
        offset_map: vec![],
        src: NamedSource::new("<test>", Arc::new(String::new())),
    };
    let tokens = lex(preprocessed).unwrap();
    assert_eq!(tokens, vec![
        Token::KwComponent, Token::Ident(String::from("Foo")), Token::OpenBody,
        Token::Divider,
        Token::StartTag, Token::Ident(String::from("Col")),
        Token::Ident(String::from("color")), Token::AttrAssign,
        Token::Unsafe, Token::OpenArgs,
        Token::ValueString(String::from("\"red\"")),
        Token::CommaSeparator,
        Token::ValueString(String::from("\"design exception\"")),
        Token::CloseArgs,
        Token::EndAutoclosingTag,
        Token::CloseBody,
        Token::Eof,
    ]);
}

#[test]
fn lexer_unexpected_char() {
    // `</Column` without closing `>` — logos tokenises `<` as StartTag then fails on `/`
    // logos raw span: 100..101 in sanitized; correct_span applies delta -1 → 99..100 in original
    let original_input = "component TestComponent {\nconst test = 13;}\n----\n<Column>\n<unsafe reason=\"test reason\"></unsafe>\n</Column\n}";
    let preprocessed = preprocess(original_input, "<test>").unwrap();
    let src = preprocessed.src.clone();
    let tokens = lex(preprocessed);
    assert_eq!(
        tokens,
        Err(LexError::UnexpectedChar {
            code: codes::L001.code,
            message: codes::L001.message,
            span: (99usize, 1usize).into(),
            src
        })
    )
}

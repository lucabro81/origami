use std::sync::Arc;

use logos::Logos;
use miette::NamedSource;
use origami_runtime::{LexError, Token, codes, errors::PreprocessorError};

#[derive(Debug, PartialEq)]
pub struct PreprocessResult {
    pub sanitized: String,
    pub logic_blocks: Vec<String>,
    /// Each entry is `(sanitized_offset, delta)`: the byte offset in the sanitized string
    /// where a substitution starts, and how many bytes the original was longer than the
    /// placeholder. Used to map spans in the sanitized string back to the original source.
    pub offset_map: Vec<(usize, i64)>,
    /// Original source text and filename, ready to attach to diagnostics.
    pub src: NamedSource<Arc<String>>,
}

fn find_substr(input: &str, from: usize, needle: &str) -> Option<usize> {
    input[from..].find(needle).map(|pos| from + pos)
}

fn find_char(input: &str, from: usize, needle: u8) -> Option<usize> {
    input.as_bytes()[from..].iter().position(|&b| b == needle).map(|pos| from + pos)
}

fn skip_newline(bytes: &[u8], from: usize) -> usize {
    let mut i = from;
    if i < bytes.len() && bytes[i] == b'\n' { i += 1; }
    i
}

/// Corrects a span from the sanitized string back to the original source.
fn correct_span(span: std::ops::Range<usize>, offset_map: &[(usize, i64)]) -> std::ops::Range<usize> {
    let delta: i64 = offset_map.iter()
        .filter(|(pos, _)| *pos <= span.start)
        .map(|(_, d)| d)
        .sum();
    let start = (span.start as i64 + delta) as usize;
    let end = (span.end as i64 + delta) as usize;
    start..end
}


/// Replaces opaque zones in the source with placeholders before lexing:
/// - content between `{` and `----` → `__LOGIC__` (content saved in `logic_blocks`)
/// - content between `<unsafe ...>` closing `>` and `</unsafe>` → `__UNSAFE__`
pub fn preprocess(input: &str, filename: &str) -> Result<PreprocessResult, PreprocessorError> {
    let mut sanitized = String::with_capacity(input.len());
    let mut logic_blocks = Vec::new();
    let mut offset_map = Vec::new();
    let original = Arc::new(input.to_string());
    let bytes = input.as_bytes();
    let len = input.len();
    let mut i = 0;

    while i < len {
        // Look for logic block: `{` followed by a newline, then content, then `----` on its own line.
        // Pattern: `{\n<content>\n----\n`
        if bytes[i] == b'{' {
            let after_brace = i + 1;
            let content_start = skip_newline(bytes, after_brace);
            let src = || NamedSource::new(filename, Arc::clone(&original));

            let divider_pos = find_substr(input, content_start, "----").ok_or_else(|| {
                PreprocessorError::SymbolNotFound {
                    code: codes::PP001.code,
                    message: codes::PP001.message,
                    span: (i, 1).into(),
                    src: src(),
                }
            })?;

            // `----` must be preceded by `\n` and followed by `\n` or end-of-input
            let preceded_by_newline = divider_pos > 0 && bytes[divider_pos - 1] == b'\n';
            let followed_by_newline = divider_pos + 4 >= len || bytes[divider_pos + 4] == b'\n';
            if !preceded_by_newline || !followed_by_newline {
                return Err(PreprocessorError::DisplacedToken {
                    code: codes::PP002.code,
                    message: codes::PP002.message,
                    span: (divider_pos, 4).into(),
                    src: src(),
                });
            }

            let content = &input[content_start..divider_pos];
            if !content.trim().is_empty() {
                sanitized.push('{');
                sanitized.push('\n');
                let placeholder_start = sanitized.len();
                logic_blocks.push(content.to_string());
                // original bytes consumed: 1 (newline after `{`) + content.len()
                // placeholder bytes: "__LOGIC__\n".len() = 10
                let delta = (content.len() + 1) as i64 - 10;
                if delta != 0 {
                    offset_map.push((placeholder_start, delta));
                }
                sanitized.push_str("__LOGIC__\n");
                i = divider_pos;
                continue;
            }
        }

        // Look for unsafe block: `>` after `<unsafe` header, up to `</unsafe>`
        // TODO: remove allow and collapse into a single `if` with `&& let` once
        // `let_chains` (rust-lang/rust#53667) stabilises.
        #[allow(clippy::collapsible_if)]
        if i + 7 <= len && &input[i..i + 7] == "<unsafe" {
            if let Some(gt_pos) = find_char(input, i + 7, b'>') {
                let content_start = gt_pos + 1;
                if let Some(close_pos) = find_substr(input, content_start, "</unsafe>") {
                    sanitized.push_str(&input[i..=gt_pos]);
                    let placeholder_start = sanitized.len();
                    let content_len = close_pos - content_start;
                    let delta = content_len as i64 - 10;
                    if delta != 0 {
                        offset_map.push((placeholder_start, delta));
                    }
                    sanitized.push_str("__UNSAFE__");
                    sanitized.push_str(&input[close_pos..close_pos + 9]);
                    i = close_pos + 9;
                    continue;
                }
            }
        }

        sanitized.push(bytes[i] as char);
        i += 1;
    }

    Ok(PreprocessResult { sanitized, logic_blocks, offset_map, src: NamedSource::new(filename, original) })
}

pub fn lex(preprocessed: PreprocessResult) -> Result<Vec<Token>, LexError> {
    let PreprocessResult { sanitized, logic_blocks, offset_map, src } = preprocessed;
    let mut lexer = Token::lexer(&sanitized);
    let mut tokens = vec![];
    let mut logic_idx = 0;

    while let Some(result) = lexer.next() {
        match result {
            Ok(Token::LogicBlock(_)) => {
                let content = logic_blocks.get(logic_idx).cloned().unwrap_or_default();
                logic_idx += 1;
                tokens.push(Token::LogicBlock(content));
            }
            Ok(token) => tokens.push(token),
            Err(_) => {
                let span = correct_span(lexer.span(), &offset_map);
                return Err(LexError::UnexpectedChar {
                    code: codes::L001.code,
                    message: codes::L001.message,
                    src,
                    span: span.into(),
                });
            }
        }
    }
    tokens.push(Token::Eof);
    Ok(tokens)
}

#[cfg(test)] mod tests;

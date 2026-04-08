use origami_runtime::{LexError, Token};

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
  Ok(vec![Token::Eof])
}

#[cfg(test)] mod tests;
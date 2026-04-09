use logos::Logos;
use origami_runtime::{LexError, Position, Token, codes};

fn offset_to_pos(input: &str, offset: usize) -> Position {
  let before = &input[..offset];
  let line = before.chars().filter(|&c| c == '\n').count() + 1;
  let col = before.rfind('\n').map(|i| offset - i - 1).unwrap_or(offset) + 1;
  Position { line, col}
}

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
  let mut lexer = Token::lexer(input);
  let mut tokens = vec![];
  while let Some(result) = lexer.next() {
    match result {
      Ok(token) => tokens.push(token),
      Err(_) => {
        let pos = offset_to_pos(input, lexer.span().start);
        return Err(LexError::UnexpectedChar { 
        code: codes::L002.code , 
        message: codes::L002.message, 
        pos
      })}
    }
  }
  tokens.push(Token::Eof);
  Ok(tokens)
}

#[cfg(test)] mod tests;
use thiserror::Error;
use crate::Position;

#[derive(Debug, Error)]
pub enum LexError {
  #[error("[{code}] {message} at {pos}")]
  UnexpectedChar { code: &'static str, message: String, pos: Position },

  #[error("[{code}] {message} at {pos}")]
  UnterminatedString { code: &'static str, message: String, pos: Position },
}
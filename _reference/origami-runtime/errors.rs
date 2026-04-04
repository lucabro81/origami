use thiserror::Error;
use crate::Position;

#[derive(Debug, Error)]
pub enum LexError {
  #[error("[{code}] {message} at {pos}")]
  UnexpectedChar { code: &'static str, message: String, pos: Position },

  #[error("[{code}] {message} at {pos}")]
  UnterminatedString { code: &'static str, message: String, pos: Position },
}

#[derive(Debug, Error)]
pub enum ParseError { 
  #[error("[{code}] {message} at {pos}")]
  UnexpectedToken { code: &'static str, message: String, pos: Position },

  #[error("[{code}] {message} at {pos}")]
  MissingClosingTag { code: &'static str, message: String, pos: Position },

  #[error("[{code}] {message} at {pos}")]
  InvalidPropValue { code: &'static str, message: String, pos: Position },
}

#[derive(Debug, Error)]
pub enum AnalyzerError { 
  #[error("[{code}] {message} at {pos}")]
  UnknownComponent { code: &'static str, message: String, pos: Position },

  #[error("[{code}] {message} at {pos}")]
  UnknownProp { code: &'static str, message: String, pos: Position },

  #[error("[{code}] {message} at {pos}")]
  InvalidTokenValue { code: &'static str, message: String, pos: Position },

  #[error("[{code}] {message} at {pos}")]
  InvalidEnumValue { code: &'static str, message: String, pos: Position },

  #[error("[{code}] {message} at {pos}")]
  UndeclaredIdentifier { code: &'static str, message: String, pos: Position },

  #[error("[{code}] {message} at {pos}")]
  DuplicateComponent { code: &'static str, message: String, pos: Position },

  #[error("[{code}] {message} at {pos}")]
  MissingRequiredProp { code: &'static str, message: String, pos: Position },
}

#[derive(Debug, Error)]
pub enum AnalyzerWarning { 
  #[warning("[{code}] {message} at {pos}")]
  UnusedProp { code: &'static str, message: String, pos: Position },

  #[warning("[{code}] {message} at {pos}")]
  ExpressionInTokenProp { code: &'static str, message: String, pos: Position },
}
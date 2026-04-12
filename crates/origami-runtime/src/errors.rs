use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, PartialEq, Error, Diagnostic)]
pub enum LexError {
  #[error("[{code}] {message}")]
  UnexpectedChar { 
      code: &'static str, 
      message: &'static str, 
      span: SourceSpan,
      src: NamedSource<Arc<String>>
    },
}

#[derive(Debug, PartialEq, Error, Diagnostic)]
pub enum PreprocessorError {
  #[error("[{code}] {message}")]
  SymbolNotFound { 
      code: &'static str, 
      message: &'static str, 
      span: SourceSpan,
      src: NamedSource<Arc<String>>
    },

  #[error("[{code}] {message}")]
  DisplacedToken { 
    code: &'static str, 
    message: &'static str,
    span: SourceSpan,
    src: NamedSource<Arc<String>>
  },
}
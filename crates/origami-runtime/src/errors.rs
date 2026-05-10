use std::sync::Arc;

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, PartialEq, Error, Diagnostic)]
pub enum ParseError {
    #[error("[{code}] {message}")]
    UnexpectedToken {
        code: &'static str,
        message: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<Arc<String>>,
    },

    #[error("[{code}] {message}")]
    ElseWithoutIf {
        code: &'static str,
        message: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<Arc<String>>,
    },

    #[error("[{code}] {message}")]
    UnsafeMissingReason {
        code: &'static str,
        message: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<Arc<String>>,
    },
}

#[derive(Debug, PartialEq, Error, Diagnostic)]
pub enum AnalyzerError {
    #[error("[{code}] {message}: `{name}`")]
    UnknownComponent {
        code: &'static str,
        message: &'static str,
        name: String,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<Arc<String>>,
    },

    #[error("[{code}] {message}: `{value}` on prop `{prop}`")]
    InvalidPropValue {
        code: &'static str,
        message: &'static str,
        prop: String,
        value: String,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<Arc<String>>,
    },

    #[error("[{code}] {message}: `{prop}` on `{component}`")]
    UnknownProp {
        code: &'static str,
        message: &'static str,
        prop: String,
        component: String,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<Arc<String>>,
    },

    #[error("[{code}] {message}: `{ident}`")]
    UndeclaredIdentifier {
        code: &'static str,
        message: &'static str,
        ident: String,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<Arc<String>>,
    },

    #[error("[{code}] {message}")]
    UnsafeBlockMissingReason {
        code: &'static str,
        message: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<Arc<String>>,
    },

    #[error("[{code}] {message}")]
    UnsafePropMissingReason {
        code: &'static str,
        message: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<Arc<String>>,
    },
}

#[derive(Debug, PartialEq, Error, Diagnostic)]
pub enum AnalyzerWarning {
    #[error("[{code}] {message}")]
    UnsafeBlock {
        code: &'static str,
        message: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<Arc<String>>,
    },

    #[error("[{code}] {message}")]
    UnsafePropValue {
        code: &'static str,
        message: &'static str,
        #[label("here")]
        span: SourceSpan,
        #[source_code]
        src: NamedSource<Arc<String>>,
    },
}

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
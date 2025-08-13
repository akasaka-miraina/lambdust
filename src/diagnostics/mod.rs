//! Error handling and diagnostics for the Lambdust language.
//!
//! This module provides comprehensive error reporting with source location information,
//! helpful error messages, and integration with the `miette` and `ariadne` crates
//! for beautiful error display.

#![allow(missing_docs)]

pub mod error;
pub mod position;
pub mod span;
pub mod source_map;
pub mod stack_trace;
pub mod suggestions;
pub mod gc_diagnostics;
pub mod custom_error;
pub mod lightweight_diagnostic;

pub use error::*;
pub use position::*;
pub use span::{Span, Spanned, spanned};
pub use source_map::*;
pub use stack_trace::*;
pub use suggestions::*;
pub use gc_diagnostics::{
    GcDiagnosticManager, GcDiagnosticConfig, DiagnosticId, PreservedError,
    ErrorKind, ErrorContext, GcAwareError, DiagnosticStatistics
};
pub use custom_error::{LambdustError, ErrorLabel, LabelStyle, RuntimeError, utils as error_utils};
pub use lightweight_diagnostic::{
    LightweightDiagnostic, DiagnosticLabel, DiagnosticLabelStyle, DiagnosticSeverity,
    DiagnosticReporter, report_diagnostic
};

/// Result type used throughout the Lambdust implementation.
pub type Result<T> = std::result::Result<T, Box<Error>>;

/// Error types for the Lambdust language implementation.
#[derive(Debug, Clone)]
pub enum Error {
    /// Lexical analysis errors
    LexError {
        message: String,
        span: Span,
    },

    /// Parsing errors
    ParseError {
        message: String,
        span: Span,
    },

    /// Type checking errors
    TypeError {
        message: String,
        span: Span,
    },

    /// Macro expansion errors
    MacroError {
        message: String,
        span: Span,
    },

    /// Runtime evaluation errors
    RuntimeError {
        message: String,
        span: Option<Span>,
    },

    /// FFI errors
    FfiError {
        message: String,
    },

    /// IO and system errors
    IoError {
        message: String,
    },

    /// Internal compiler errors (bugs)
    InternalError {
        message: String,
    },

    /// R7RS Exception (raised by raise/error procedures)
    Exception {
        exception: crate::stdlib::exceptions::ExceptionObject,
        span: Option<Span>,
    },
}

impl Error {
    /// Creates a new lexical error.
    pub fn lex_error(message: impl Into<String>, span: Span) -> Self {
        Self::LexError {
            message: message.into(),
            span,
        }
    }

    /// Creates a new parse error.
    pub fn parse_error(message: impl Into<String>, span: Span) -> Self {
        Self::ParseError {
            message: message.into(),
            span,
        }
    }

    /// Creates a new type error.
    pub fn type_error(message: impl Into<String>, span: Span) -> Self {
        Self::TypeError {
            message: message.into(),
            span,
        }
    }

    /// Creates a new macro error.
    pub fn macro_error(message: impl Into<String>, span: Span) -> Self {
        Self::MacroError {
            message: message.into(),
            span,
        }
    }

    /// Creates a new runtime error.
    pub fn runtime_error(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::RuntimeError {
            message: message.into(),
            span,
        }
    }

    /// Creates a new FFI error.
    pub fn ffi_error(message: impl Into<String>) -> Self {
        Self::FfiError {
            message: message.into(),
        }
    }

    /// Creates a new IO error.
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            message: message.into(),
        }
    }

    /// Creates a new internal error.
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
        }
    }

    /// Creates a new syntax error (alias for parse_error).
    pub fn syntax_error(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::ParseError {
            message: message.into(),
            span: span.unwrap_or(Span::new(0, 0)),
        }
    }

    /// Creates a new exception error.
    pub fn exception(exception: crate::stdlib::exceptions::ExceptionObject) -> Self {
        Self::Exception {
            exception,
            span: None,
        }
    }

    /// Creates a new exception error with span.
    pub fn exception_with_span(exception: crate::stdlib::exceptions::ExceptionObject, span: Span) -> Self {
        Self::Exception {
            exception,
            span: Some(span),
        }
    }

    /// Creates an arity error for a function called with wrong number of arguments.
    pub fn arity_error(function_name: &str, expected: usize, actual: usize) -> Self {
        Self::RuntimeError {
            message: format!("Function '{function_name}' expects {expected} arguments, got {actual}"),
            span: None,
        }
    }
}

// Manual Display implementation to replace thiserror
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LexError { message, .. } => write!(f, "Lexical error: {message}"),
            Self::ParseError { message, .. } => write!(f, "Parse error: {message}"),
            Self::TypeError { message, .. } => write!(f, "Type error: {message}"),
            Self::MacroError { message, .. } => write!(f, "Macro error: {message}"),
            Self::RuntimeError { message, .. } => write!(f, "Runtime error: {message}"),
            Self::FfiError { message } => write!(f, "FFI error: {message}"),
            Self::IoError { message } => write!(f, "IO error: {message}"),
            Self::InternalError { message } => write!(f, "Internal error: {message}"),
            Self::Exception { exception, .. } => write!(f, "Exception: {exception}"),
        }
    }
}

// LambdustError trait implementation
impl LambdustError for Error {
    fn error_code(&self) -> &'static str {
        match self {
            Self::LexError { .. } => "lambdust::lexer::error",
            Self::ParseError { .. } => "lambdust::parser::error",
            Self::TypeError { .. } => "lambdust::types::error",
            Self::MacroError { .. } => "lambdust::macros::error",
            Self::RuntimeError { .. } => "lambdust::runtime::error",
            Self::FfiError { .. } => "lambdust::ffi::error",
            Self::IoError { .. } => "lambdust::io::error",
            Self::InternalError { .. } => "lambdust::internal::error",
            Self::Exception { .. } => "lambdust::exception::error",
        }
    }
    
    fn help(&self) -> Option<&str> {
        match self {
            Self::InternalError { .. } => Some("This is likely a bug in the Lambdust implementation. Please report it."),
            _ => None,
        }
    }
    
    fn labels(&self) -> Vec<ErrorLabel> {
        match self {
            Self::LexError { span, .. } => vec![ErrorLabel::primary(*span, "here")],
            Self::ParseError { span, .. } => vec![ErrorLabel::primary(*span, "here")],
            Self::TypeError { span, .. } => vec![ErrorLabel::primary(*span, "here")],
            Self::MacroError { span, .. } => vec![ErrorLabel::primary(*span, "here")],
            Self::RuntimeError { span: Some(span), .. } => vec![ErrorLabel::primary(*span, "here")],
            Self::Exception { span: Some(span), .. } => vec![ErrorLabel::primary(*span, "raised here")],
            _ => Vec::new(),
        }
    }
    
    fn is_critical(&self) -> bool {
        matches!(self, Self::InternalError { .. })
    }
}

// Standard Error trait implementation
impl std::error::Error for Error {}

// LightweightDiagnostic trait implementation
impl LightweightDiagnostic for Error {
    fn code(&self) -> Option<&str> {
        match self {
            Self::LexError { .. } => Some("lambdust::lexer::error"),
            Self::ParseError { .. } => Some("lambdust::parser::error"),
            Self::TypeError { .. } => Some("lambdust::types::error"),
            Self::MacroError { .. } => Some("lambdust::macros::error"),
            Self::RuntimeError { .. } => Some("lambdust::runtime::error"),
            Self::FfiError { .. } => Some("lambdust::ffi::error"),
            Self::IoError { .. } => Some("lambdust::io::error"),
            Self::InternalError { .. } => Some("lambdust::internal::error"),
            Self::Exception { .. } => Some("lambdust::exception::error"),
        }
    }
    
    fn help(&self) -> Option<&str> {
        match self {
            Self::InternalError { .. } => Some("This is likely a bug in the Lambdust implementation. Please report it."),
            _ => None,
        }
    }
    
    fn labels(&self) -> Vec<DiagnosticLabel> {
        match self {
            Self::LexError { span, .. } => vec![DiagnosticLabel::primary(*span, "here")],
            Self::ParseError { span, .. } => vec![DiagnosticLabel::primary(*span, "here")],
            Self::TypeError { span, .. } => vec![DiagnosticLabel::primary(*span, "here")],
            Self::MacroError { span, .. } => vec![DiagnosticLabel::primary(*span, "here")],
            Self::RuntimeError { span: Some(span), .. } => vec![DiagnosticLabel::primary(*span, "here")],
            Self::Exception { span: Some(span), .. } => vec![DiagnosticLabel::primary(*span, "raised here")],
            _ => Vec::new(),
        }
    }
    
    fn severity(&self) -> DiagnosticSeverity {
        match self {
            Self::InternalError { .. } => DiagnosticSeverity::Error,
            _ => DiagnosticSeverity::Error,
        }
    }
}

impl Error {
    /// Creates a compilation error.
    pub fn compilation_error(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: format!("Compilation error: {}", message.into()),
        }
    }

    /// Creates a type mismatch error with expected type information.
    pub fn type_mismatch_error(expected: &str, actual: impl std::fmt::Debug) -> Self {
        Self::RuntimeError {
            message: format!("Expected {expected}, got {actual:?}"),
            span: None,
        }
    }

    /// Creates an unexpected end-of-file error.
    pub fn unexpected_eof(span: Span) -> Self {
        Self::ParseError {
            message: "Unexpected end of file".to_string(),
            span,
        }
    }

    /// Creates an unexpected token error.
    pub fn unexpected_token(token: &crate::lexer::Token, expected: &str) -> Self {
        Self::ParseError {
            message: format!("Unexpected token '{}', expected {}", token.lexeme, expected),
            span: token.span,
        }
    }

    /// Creates an expected token error.
    pub fn expected_token(token: &crate::lexer::Token, expected: &crate::lexer::TokenKind, context: &str) -> Self {
        Self::ParseError {
            message: format!("{}, found '{}'", context, token.lexeme),
            span: token.span,
        }
    }

    /// Creates a not-implemented error for development.
    pub fn not_implemented(feature: &str, span: Span) -> Self {
        Self::InternalError {
            message: format!("Feature '{feature}' not yet implemented"),
        }
    }

    /// Converts this Error into a Box<Error> for use with the Result type.
    pub fn boxed(self) -> Box<Error> {
        Box::new(self)
    }

    /// Creates a boxed lexical error.
    pub fn boxed_lex_error(message: impl Into<String>, span: Span) -> Box<Error> {
        Self::lex_error(message, span).boxed()
    }

    /// Creates a boxed parse error.
    pub fn boxed_parse_error(message: impl Into<String>, span: Span) -> Box<Error> {
        Self::parse_error(message, span).boxed()
    }

    /// Creates a boxed runtime error.
    pub fn boxed_runtime_error(message: impl Into<String>, span: Option<Span>) -> Box<Error> {
        Self::runtime_error(message, span).boxed()
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for Box<Error> {
    fn from(err: std::io::Error) -> Self {
        Error::from(err).into()
    }
}

/// Helper trait for adding span information to results.
pub trait WithSpan<T> {
    /// Adds span information to an error result.
    fn with_span(self, span: Span) -> Result<T>;
}

impl<T, E> WithSpan<T> for std::result::Result<T, E>
where
    E: Into<String>,
{
    fn with_span(self, span: Span) -> Result<T> {
        self.map_err(|e| Error::runtime_error(e.into(), Some(span)).boxed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new(10, 5);
        assert_eq!(span.start, 10);
        assert_eq!(span.len, 5);
        assert_eq!(span.end(), 15);
    }

    #[test]
    fn test_span_combine() {
        let span1 = Span::new(5, 3);
        let span2 = Span::new(10, 2);
        let combined = span1.combine(span2);
        
        assert_eq!(combined.start, 5);
        assert_eq!(combined.end(), 12);
        assert_eq!(combined.len, 7);
    }

    #[test]
    fn test_error_creation() {
        let span = Span::new(0, 5);
        let error = Error::lex_error("test error", span);
        
        match error {
            Error::LexError { message, span: error_span } => {
                assert_eq!(message, "test error");
                assert_eq!(error_span, span);
            }
            _ => panic!("Wrong error type"),
        }
    }
}
//! Custom error system to replace thiserror dependency.
//!
//! This module provides a lightweight, high-performance error handling system
//! designed specifically for Lambdust's needs, eliminating the need for the
//! thiserror crate while maintaining full compatibility with existing error
//! patterns.

use std::fmt;
use std::error::Error as StdError;
use std::backtrace::Backtrace;

/// Custom error trait to replace thiserror functionality.
/// 
/// This trait provides the core functionality needed by Lambdust's error types
/// while being much lighter than the full thiserror crate.
pub trait LambdustError: fmt::Debug + fmt::Display + Send + Sync + 'static {
    /// Returns the error code for this error type.
    fn error_code(&self) -> &'static str {
        "lambdust::unknown"
    }
    
    /// Returns the source of this error, if any.
    fn source(&self) -> Option<&dyn StdError> {
        None
    }
    
    /// Returns a backtrace associated with this error, if available.
    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }
    
    /// Returns help text for this error.
    fn help(&self) -> Option<&str> {
        None
    }
    
    /// Returns labels for source locations related to this error.
    fn labels(&self) -> Vec<ErrorLabel> {
        Vec::new()
    }
    
    /// Returns whether this is a critical error that should halt execution.
    fn is_critical(&self) -> bool {
        false
    }
}

/// Error label for source location annotation.
#[derive(Debug, Clone)]
pub struct ErrorLabel {
    /// The span this label refers to.
    pub span: crate::diagnostics::span::Span,
    /// The label message.
    pub message: Option<String>,
    /// The style of this label (primary, secondary, etc.).
    pub style: LabelStyle,
}

/// Style for error labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    /// Primary label (most important).
    Primary,
    /// Secondary label (additional context).
    Secondary,
}

impl ErrorLabel {
    /// Creates a new primary error label.
    pub fn primary(span: crate::diagnostics::span::Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: Some(message.into()),
            style: LabelStyle::Primary,
        }
    }
    
    /// Creates a new secondary error label.
    pub fn secondary(span: crate::diagnostics::span::Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: Some(message.into()),
            style: LabelStyle::Secondary,
        }
    }
    
    /// Creates a new label with just a span (no message).
    pub fn span_only(span: crate::diagnostics::span::Span) -> Self {
        Self {
            span,
            message: None,
            style: LabelStyle::Primary,
        }
    }
}

/// Macro to implement the standard Error trait for types that implement LambdustError.
macro_rules! impl_std_error {
    ($type:ty) => {
        impl std::error::Error for $type {}
    };
}

/// Macro to derive LambdustError implementation with error messages.
/// 
/// This replaces the functionality of #[derive(thiserror::Error)] with a
/// lightweight custom implementation.
macro_rules! derive_error {
    (
        $(#[$attr:meta])*
        pub enum $name:ident {
            $(
                $(#[error($msg:expr)])?
                $(#[error_code($code:expr)])?
                $variant:ident $({
                    $($field:ident: $field_ty:ty),* $(,)?
                })?,
            )*
        }
    ) => {
        $(#[$attr])*
        pub enum $name {
            $(
                $variant $({
                    $($field: $field_ty),*
                })?,
            )*
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        Self::$variant $({ $($field),* })? => {
                            derive_error!(@format f, $msg, $($($field),*)?)
                        }
                    )*
                }
            }
        }

        impl LambdustError for $name {
            fn error_code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant { .. } => {
                            derive_error!(@code $code, concat!("lambdust::", stringify!($name), "::", stringify!($variant)))
                        }
                    )*
                }
            }
        }

        impl_std_error!($name);
    };

    // Helper: Format message with field interpolation
    (@format $f:expr, $msg:expr, $($field:ident),*) => {
        write!($f, $msg, $($field = $field),*)
    };

    // Helper: Format message with no fields
    (@format $f:expr, $msg:expr,) => {
        write!($f, $msg)
    };

    // Helper: Use provided error code or generate default
    (@code $code:expr, $default:expr) => { $code };
    (@code , $default:expr) => { $default };
}

/// Utility functions for error creation and handling.
pub mod utils {
    use super::*;
    
    /// Creates a boxed error from any type implementing LambdustError.
    pub fn boxed_error<E: LambdustError>(error: E) -> Box<dyn LambdustError> {
        Box::new(error)
    }
    
    /// Creates a generic runtime error.
    pub fn runtime_error(message: impl Into<String>) -> RuntimeError {
        RuntimeError {
            message: message.into(),
            source: None,
        }
    }
    
    /// Creates a runtime error with a source.
    pub fn runtime_error_with_source(
        message: impl Into<String>, 
        source: Box<dyn StdError + Send + Sync>
    ) -> RuntimeError {
        RuntimeError {
            message: message.into(),
            source: Some(source),
        }
    }
}

/// Generic runtime error for cases where more specific error types aren't needed.
#[derive(Debug)]
pub struct RuntimeError {
    message: String,
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl LambdustError for RuntimeError {
    fn error_code(&self) -> &'static str {
        "lambdust::runtime_error"
    }
    
    fn source(&self) -> Option<&dyn StdError> {
        self.source.as_ref().map(|e| e.as_ref() as &dyn StdError)
    }
}

impl_std_error!(RuntimeError);

#[cfg(test)]
mod tests {
    use super::*;

    derive_error! {
        #[derive(Debug)]
        pub enum TestError {
            #[error("Simple error")]
            Simple,
            
            #[error("Error with field: {message}")]
            WithField { message: String },
            
            #[error("Error with multiple fields: {a} and {b}")]
            #[error_code("custom::test")]
            MultiField { a: String, b: i32 },
        }
    }

    #[test]
    fn test_error_display() {
        let simple = TestError::Simple;
        assert_eq!(simple.to_string(), "Simple error");
        
        let with_field = TestError::WithField { 
            message: "test".to_string() 
        };
        assert_eq!(with_field.to_string(), "Error with field: test");
        
        let multi = TestError::MultiField { 
            a: "hello".to_string(), 
            b: 42 
        };
        assert_eq!(multi.to_string(), "Error with multiple fields: hello and 42");
    }

    #[test]
    fn test_error_codes() {
        let simple = TestError::Simple;
        assert_eq!(simple.error_code(), "lambdust::TestError::Simple");
        
        let multi = TestError::MultiField { 
            a: "test".to_string(), 
            b: 1 
        };
        assert_eq!(multi.error_code(), "custom::test");
    }

    #[test]
    fn test_runtime_error() {
        let error = utils::runtime_error("Something went wrong");
        assert_eq!(error.to_string(), "Something went wrong");
        assert_eq!(error.error_code(), "lambdust::runtime_error");
    }

    #[test]
    fn test_error_labels() {
        let span = crate::diagnostics::Span::new(0, 10);
        let label = ErrorLabel::primary(span, "here");
        
        assert_eq!(label.style, LabelStyle::Primary);
        assert_eq!(label.message.as_ref().unwrap(), "here");
        assert_eq!(label.span.start, 0);
        assert_eq!(label.span.end(), 10);
    }
}
//! Unified error handling system for Lambdust.
//!
//! This module provides a comprehensive, zero-cost abstraction for error handling
//! that eliminates redundancy while maintaining performance and ergonomics.

use crate::diagnostics::{
    DiagnosticLabel, DiagnosticSeverity, ErrorLabel, EvalUnifiedError, LightweightDiagnostic, Span,
};
use std::error::Error as StdError;
use std::fmt;

/// Unified result type for all Lambdust operations.
pub type UnifiedResult<T, E = UnifiedError> = std::result::Result<T, E>;

/// Unified error system with compile-time category selection.
#[derive(Debug, Clone)]
pub struct UnifiedError {
    /// Error category for type-safe error handling
    pub category: ErrorCategory,
    /// Human-readable error message
    pub message: String,
    /// Optional source location
    pub span: Option<Span>,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Additional context information
    pub context: Vec<ErrorContext>,
    /// Error chain for nested errors
    pub source: Option<Box<UnifiedError>>,
}

/// Compile-time error categories for type safety.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Lexical analysis errors
    Lexical,
    /// Syntax parsing errors
    Syntax,
    /// Type system errors
    Type,
    /// Runtime evaluation errors
    Runtime,
    /// JIT compilation errors
    Jit,
    /// Macro expansion errors
    Macro,
    /// FFI interface errors
    Ffi,
    /// I/O operation errors
    Io,
    /// Module system errors
    Module,
    /// Internal system errors
    Internal,
    /// R7RS exception errors
    Exception,
}

/// Error severity levels for appropriate handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational messages
    Info,
    /// Warning conditions
    Warning,
    /// Error conditions that can be recovered from
    Error,
    /// Critical errors requiring immediate attention
    Critical,
    /// Fatal errors that terminate execution
    Fatal,
}

/// Additional context information for errors.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Context type identifier
    pub context_type: String,
    /// Context-specific data
    pub data: String,
    /// Optional span for context location
    pub span: Option<Span>,
}

impl UnifiedError {
    /// Creates a new unified error with compile-time category verification.
    pub fn new<C: IntoErrorCategory>(category: C, message: impl Into<String>) -> Self {
        Self {
            category: category.into_category(),
            message: message.into(),
            span: None,
            severity: ErrorSeverity::Error,
            context: Vec::new(),
            source: None,
        }
    }

    /// Builder pattern for error construction.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Sets error severity level.
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Adds context information.
    pub fn with_context(
        mut self,
        context_type: impl Into<String>,
        data: impl Into<String>,
    ) -> Self {
        self.context.push(ErrorContext {
            context_type: context_type.into(),
            data: data.into(),
            span: None,
        });
        self
    }

    /// Adds context with span information.
    pub fn with_context_span(
        mut self,
        context_type: impl Into<String>,
        data: impl Into<String>,
        span: Span,
    ) -> Self {
        self.context.push(ErrorContext {
            context_type: context_type.into(),
            data: data.into(),
            span: Some(span),
        });
        self
    }

    /// Chains errors for causality tracking.
    pub fn with_source(mut self, source: UnifiedError) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Converts to boxed error for compatibility.
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }

    /// Returns error code string for programmatic handling.
    pub fn error_code(&self) -> &'static str {
        match self.category {
            ErrorCategory::Lexical => "lambdust::lexer::error",
            ErrorCategory::Syntax => "lambdust::parser::error",
            ErrorCategory::Type => "lambdust::types::error",
            ErrorCategory::Runtime => "lambdust::runtime::error",
            ErrorCategory::Jit => "lambdust::jit::error",
            ErrorCategory::Macro => "lambdust::macros::error",
            ErrorCategory::Ffi => "lambdust::ffi::error",
            ErrorCategory::Io => "lambdust::io::error",
            ErrorCategory::Module => "lambdust::module::error",
            ErrorCategory::Internal => "lambdust::internal::error",
            ErrorCategory::Exception => "lambdust::exception::error",
        }
    }

    /// Checks if error is recoverable.
    pub fn is_recoverable(&self) -> bool {
        match self.severity {
            ErrorSeverity::Info | ErrorSeverity::Warning | ErrorSeverity::Error => true,
            ErrorSeverity::Critical | ErrorSeverity::Fatal => false,
        }
    }

    /// Returns all context information as formatted string.
    pub fn context_info(&self) -> String {
        if self.context.is_empty() {
            String::new()
        } else {
            self.context
                .iter()
                .map(|ctx| format!("{}: {}", ctx.context_type, ctx.data))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

/// Trait for compile-time error category validation.
pub trait IntoErrorCategory {
    fn into_category(self) -> ErrorCategory;
}

impl IntoErrorCategory for ErrorCategory {
    fn into_category(self) -> ErrorCategory {
        self
    }
}

// Zero-cost compile-time category markers
pub struct LexicalError;
pub struct SyntaxError;
pub struct TypeError;
pub struct RuntimeError;
pub struct JitError;
pub struct MacroError;
pub struct FfiError;
pub struct IoError;
pub struct ModuleError;
pub struct InternalError;
pub struct ExceptionError;

impl IntoErrorCategory for LexicalError {
    fn into_category(self) -> ErrorCategory {
        ErrorCategory::Lexical
    }
}

impl IntoErrorCategory for SyntaxError {
    fn into_category(self) -> ErrorCategory {
        ErrorCategory::Syntax
    }
}

impl IntoErrorCategory for TypeError {
    fn into_category(self) -> ErrorCategory {
        ErrorCategory::Type
    }
}

impl IntoErrorCategory for RuntimeError {
    fn into_category(self) -> ErrorCategory {
        ErrorCategory::Runtime
    }
}

impl IntoErrorCategory for JitError {
    fn into_category(self) -> ErrorCategory {
        ErrorCategory::Jit
    }
}

impl IntoErrorCategory for MacroError {
    fn into_category(self) -> ErrorCategory {
        ErrorCategory::Macro
    }
}

impl IntoErrorCategory for FfiError {
    fn into_category(self) -> ErrorCategory {
        ErrorCategory::Ffi
    }
}

impl IntoErrorCategory for IoError {
    fn into_category(self) -> ErrorCategory {
        ErrorCategory::Io
    }
}

impl IntoErrorCategory for ModuleError {
    fn into_category(self) -> ErrorCategory {
        ErrorCategory::Module
    }
}

impl IntoErrorCategory for InternalError {
    fn into_category(self) -> ErrorCategory {
        ErrorCategory::Internal
    }
}

impl IntoErrorCategory for ExceptionError {
    fn into_category(self) -> ErrorCategory {
        ErrorCategory::Exception
    }
}

impl fmt::Display for UnifiedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            match self.category {
                ErrorCategory::Lexical => "Lexical error",
                ErrorCategory::Syntax => "Syntax error",
                ErrorCategory::Type => "Type error",
                ErrorCategory::Runtime => "Runtime error",
                ErrorCategory::Jit => "JIT error",
                ErrorCategory::Macro => "Macro error",
                ErrorCategory::Ffi => "FFI error",
                ErrorCategory::Io => "I/O error",
                ErrorCategory::Module => "Module error",
                ErrorCategory::Internal => "Internal error",
                ErrorCategory::Exception => "Exception",
            },
            self.message
        )?;

        if !self.context.is_empty() {
            write!(f, " ({})", self.context_info())?;
        }

        Ok(())
    }
}

impl StdError for UnifiedError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn StdError + 'static))
    }
}

impl EvalUnifiedError for UnifiedError {
    fn error_code(&self) -> &'static str {
        self.error_code()
    }

    fn help(&self) -> Option<&str> {
        match self.category {
            ErrorCategory::Internal => Some("This is likely a bug in Lambdust. Please report it."),
            _ => None,
        }
    }

    fn labels(&self) -> Vec<ErrorLabel> {
        let mut labels = Vec::new();

        if let Some(span) = self.span {
            labels.push(ErrorLabel::primary(span, "here"));
        }

        for context in &self.context {
            if let Some(span) = context.span {
                labels.push(ErrorLabel::secondary(span, &context.data));
            }
        }

        labels
    }

    fn is_critical(&self) -> bool {
        matches!(
            self.severity,
            ErrorSeverity::Critical | ErrorSeverity::Fatal
        )
    }
}

impl LightweightDiagnostic for UnifiedError {
    fn code(&self) -> Option<&str> {
        Some(self.error_code())
    }

    fn help(&self) -> Option<&str> {
        EvalUnifiedError::help(self)
    }

    fn labels(&self) -> Vec<DiagnosticLabel> {
        EvalUnifiedError::labels(self)
            .into_iter()
            .map(|label| DiagnosticLabel::primary(label.span(), label.message()))
            .collect()
    }

    fn severity(&self) -> DiagnosticSeverity {
        match self.severity {
            ErrorSeverity::Info => DiagnosticSeverity::Note,
            ErrorSeverity::Warning => DiagnosticSeverity::Warning,
            ErrorSeverity::Error => DiagnosticSeverity::Error,
            ErrorSeverity::Critical | ErrorSeverity::Fatal => DiagnosticSeverity::Error,
        }
    }
}

// Conversion implementations for backward compatibility
impl From<std::io::Error> for UnifiedError {
    fn from(err: std::io::Error) -> Self {
        UnifiedError::new(IoError, err.to_string()).with_severity(ErrorSeverity::Error)
    }
}

impl From<UnifiedError> for Box<crate::diagnostics::Error> {
    fn from(err: UnifiedError) -> Self {
        // Convert unified error back to old error format for compatibility
        let old_error = match err.category {
            ErrorCategory::Lexical => crate::diagnostics::Error::LexError {
                message: err.message,
                span: err.span.unwrap_or_else(|| Span::new(0, 0)),
            },
            ErrorCategory::Syntax => crate::diagnostics::Error::ParseError {
                message: err.message,
                span: err.span.unwrap_or_else(|| Span::new(0, 0)),
            },
            ErrorCategory::Type => crate::diagnostics::Error::TypeError {
                message: err.message,
                span: err.span.unwrap_or_else(|| Span::new(0, 0)),
            },
            ErrorCategory::Runtime => crate::diagnostics::Error::RuntimeError {
                message: err.message,
                span: err.span,
            },
            ErrorCategory::Jit => crate::diagnostics::Error::InternalError {
                message: format!("JIT: {}", err.message),
            },
            ErrorCategory::Macro => crate::diagnostics::Error::MacroError {
                message: err.message,
                span: err.span.unwrap_or_else(|| Span::new(0, 0)),
            },
            ErrorCategory::Ffi => crate::diagnostics::Error::FfiError {
                message: err.message,
            },
            ErrorCategory::Io => crate::diagnostics::Error::IoError {
                message: err.message,
            },
            ErrorCategory::Module => crate::diagnostics::Error::InternalError {
                message: format!("Module: {}", err.message),
            },
            ErrorCategory::Internal => crate::diagnostics::Error::InternalError {
                message: err.message,
            },
            ErrorCategory::Exception => crate::diagnostics::Error::InternalError {
                message: format!("Exception: {}", err.message),
            },
        };
        Box::new(old_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_error_creation() {
        let error = UnifiedError::new(TypeError, "Type mismatch")
            .with_span(Span::new(10, 5))
            .with_severity(ErrorSeverity::Error)
            .with_context("expected", "number")
            .with_context("actual", "string");

        assert_eq!(error.category, ErrorCategory::Type);
        assert_eq!(error.message, "Type mismatch");
        assert_eq!(error.span, Some(Span::new(10, 5)));
        assert_eq!(error.severity, ErrorSeverity::Error);
        assert_eq!(error.context.len(), 2);
    }

    #[test]
    fn test_error_chaining() {
        let source_error = UnifiedError::new(RuntimeError, "Division by zero");
        let main_error =
            UnifiedError::new(JitError, "Compilation failed").with_source(source_error);

        assert!(main_error.source.is_some());
        assert_eq!(
            main_error.source.as_ref().unwrap().category,
            ErrorCategory::Runtime
        );
    }

    #[test]
    fn test_error_recoverability() {
        let recoverable = UnifiedError::new(TypeError, "test").with_severity(ErrorSeverity::Error);
        let fatal = UnifiedError::new(InternalError, "test").with_severity(ErrorSeverity::Fatal);

        assert!(recoverable.is_recoverable());
        assert!(!fatal.is_recoverable());
    }

    #[test]
    fn test_compile_time_categories() {
        // These should compile without issues, demonstrating type safety
        let _lex_error = UnifiedError::new(LexicalError, "Invalid character");
        let _type_error = UnifiedError::new(TypeError, "Type mismatch");
        let _runtime_error = UnifiedError::new(RuntimeError, "Null pointer");
    }
}

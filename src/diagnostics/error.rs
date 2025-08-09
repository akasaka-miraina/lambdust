//! Additional error utilities and helpers.

use super::{Error, Result};

/// Extension trait for converting standard library errors.
pub trait IntoLambdustError<T> {
    /// Converts the error into a Lambdust error.
    fn into_lambdust_error(self) -> Result<T>;
}

impl<T> IntoLambdustError<T> for std::io::Result<T> {
    fn into_lambdust_error(self) -> Result<T> {
        self.map_err(|e| Box::new(Error::io_error(e.to_string())))
    }
}

impl<T> IntoLambdustError<T> for serde_json::Result<T> {
    fn into_lambdust_error(self) -> Result<T> {
        self.map_err(|e| Box::new(Error::runtime_error(format!("JSON error: {e}"), None)))
    }
}

/// Helper for creating contextual error messages.
pub struct ErrorContext {
    context: Vec<String>,
}

impl ErrorContext {
    /// Creates a new error context.
    pub fn new() -> Self {
        Self {
            context: Vec::new(),
        }
    }

    /// Adds context information.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context.push(context.into());
        self
    }

    /// Wraps an error with context.
    pub fn wrap_error(self, error: Error) -> Error {
        if self.context.is_empty() {
            return error;
        }

        let context_str = self.context.join(" -> ");
        match error {
            Error::LexError { message, span } => Error::LexError {
                message: format!("{context_str}: {message}"),
                span,
            },
            Error::ParseError { message, span } => Error::ParseError {
                message: format!("{context_str}: {message}"),
                span,
            },
            Error::TypeError { message, span } => Error::TypeError {
                message: format!("{context_str}: {message}"),
                span,
            },
            Error::MacroError { message, span } => Error::MacroError {
                message: format!("{context_str}: {message}"),
                span,
            },
            Error::RuntimeError { message, span } => Error::RuntimeError {
                message: format!("{context_str}: {message}"),
                span,
            },
            Error::FfiError { message } => Error::FfiError {
                message: format!("{context_str}: {message}"),
            },
            Error::IoError { message } => Error::IoError {
                message: format!("{context_str}: {message}"),
            },
            Error::InternalError { message } => Error::InternalError {
                message: format!("{context_str}: {message}"),
            },
            Error::Exception { exception, span } => Error::Exception {
                exception,
                span,
            },
        }
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}
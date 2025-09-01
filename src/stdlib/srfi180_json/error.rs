//! SRFI-180 JSON Error Handling System
//!
//! Comprehensive error handling for JSON operations with security considerations
//! including depth limits, character limits, and detailed error reporting.

use crate::diagnostics::{Error, Result, Span};
use std::fmt;

/// Configuration options for JSON parsing/serialization security and limits
#[derive(Debug, Clone)]
pub struct JsonOptions {
    /// Maximum nesting depth allowed (prevents stack overflow)
    pub max_depth: usize,
    /// Maximum input size in bytes (prevents DoS attacks)
    pub max_size: usize,
    /// Whether to use compact or pretty-printed output
    pub pretty_print: bool,
    /// Indentation string for pretty printing
    pub indent: String,
    /// Enable strict JSON parsing (disallows trailing commas, etc.)
    pub strict_mode: bool,
}

impl Default for JsonOptions {
    fn default() -> Self {
        Self {
            max_depth: 1024,
            max_size: 1024 * 1024, // 1MB
            pretty_print: false,
            indent: "  ".to_string(),
            strict_mode: false,
        }
    }
}

impl JsonOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }

    pub fn with_indent(mut self, indent: String) -> Self {
        self.indent = indent;
        self
    }
}

/// JSON error kind enumeration for pattern matching
#[derive(Debug, Clone, PartialEq)]
pub enum JsonErrorKind {
    SyntaxError,
    DepthLimitExceeded,
    CharacterLimitExceeded,
    InvalidUnicode,
    ConversionError,
    IoError,
    UnexpectedEof,
    InvalidStructure,
}

/// JSON-specific error types as defined in SRFI-180
#[derive(Debug, Clone)]
pub enum JsonError {
    /// JSON syntax error with position information
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
        position: usize,
    },
    /// Maximum nesting depth exceeded (security limit)
    DepthLimitExceeded {
        current_depth: usize,
        max_depth: usize,
    },
    /// Maximum character limit exceeded (security limit)
    CharacterLimitExceeded {
        current_size: usize,
        max_size: usize,
    },
    /// Invalid Unicode escape sequence or character
    InvalidUnicode {
        message: String,
        position: usize,
    },
    /// Error converting between JSON and Scheme values
    ConversionError {
        message: String,
        value: String,
    },
    /// I/O error during reading or writing
    IoError {
        message: String,
    },
    /// Unexpected end of input
    UnexpectedEof {
        expected: String,
        position: usize,
    },
    /// Invalid JSON structure
    InvalidStructure {
        message: String,
        position: usize,
    },
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::SyntaxError { message, line, column, position } => {
                write!(f, "JSON syntax error at line {}, column {} (position {}): {}", 
                       line, column, position, message)
            }
            JsonError::DepthLimitExceeded { current_depth, max_depth } => {
                write!(f, "JSON depth limit exceeded: {} > {} (maximum nesting depth)", 
                       current_depth, max_depth)
            }
            JsonError::CharacterLimitExceeded { current_size, max_size } => {
                write!(f, "JSON size limit exceeded: {} > {} bytes", current_size, max_size)
            }
            JsonError::InvalidUnicode { message, position } => {
                write!(f, "Invalid Unicode at position {}: {}", position, message)
            }
            JsonError::ConversionError { message, value } => {
                write!(f, "JSON conversion error for value '{}': {}", value, message)
            }
            JsonError::IoError { message } => {
                write!(f, "JSON I/O error: {}", message)
            }
            JsonError::UnexpectedEof { expected, position } => {
                write!(f, "Unexpected end of input at position {} (expected: {})", position, expected)
            }
            JsonError::InvalidStructure { message, position } => {
                write!(f, "Invalid JSON structure at position {}: {}", position, message)
            }
        }
    }
}

impl std::error::Error for JsonError {}

impl JsonError {
    /// Get the error kind for pattern matching
    pub fn kind(&self) -> JsonErrorKind {
        match self {
            JsonError::SyntaxError { .. } => JsonErrorKind::SyntaxError,
            JsonError::DepthLimitExceeded { .. } => JsonErrorKind::DepthLimitExceeded,
            JsonError::CharacterLimitExceeded { .. } => JsonErrorKind::CharacterLimitExceeded,
            JsonError::InvalidUnicode { .. } => JsonErrorKind::InvalidUnicode,
            JsonError::ConversionError { .. } => JsonErrorKind::ConversionError,
            JsonError::IoError { .. } => JsonErrorKind::IoError,
            JsonError::UnexpectedEof { .. } => JsonErrorKind::UnexpectedEof,
            JsonError::InvalidStructure { .. } => JsonErrorKind::InvalidStructure,
        }
    }
    
    /// Create a syntax error
    pub fn syntax_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::SyntaxError {
            message: message.into(),
            line,
            column,
            position: 0, // Default position
        }
    }
    
    /// Create a depth limit exceeded error
    pub fn depth_limit_exceeded(max_depth: usize, current_depth: usize) -> Self {
        Self::DepthLimitExceeded {
            current_depth,
            max_depth,
        }
    }
    
    /// Create a conversion error
    pub fn conversion_error(from: impl Into<String>, to: impl Into<String>, message: impl Into<String>) -> Self {
        let from_str = from.into();
        let message_str = format!("{}: converting {} to {}", message.into(), from_str, to.into());
        Self::ConversionError {
            message: message_str,
            value: from_str,
        }
    }
    
    /// Create an I/O error
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            message: message.into(),
        }
    }
}

impl From<JsonError> for Error {
    fn from(err: JsonError) -> Self {
        match err {
            JsonError::SyntaxError { message, line, column, position } => {
                Error::runtime_error(
                    format!("JSON syntax error at line {}, column {}: {}", line, column, message),
                    Some(Span::new(position, 1))
                )
            }
            _ => Error::runtime_error(err.to_string(), None)
        }
    }
}

/// Position tracker for detailed error reporting
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new() -> Self {
        Self {
            offset: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn advance(&mut self, ch: char) {
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }

    pub fn advance_by(&mut self, text: &str) {
        for ch in text.chars() {
            self.advance(ch);
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON parsing/serialization context with error tracking
#[derive(Debug)]
pub struct JsonContext {
    pub options: JsonOptions,
    pub position: Position,
    pub depth: usize,
    pub size: usize,
}

impl JsonContext {
    pub fn new(options: JsonOptions) -> Self {
        Self {
            options,
            position: Position::new(),
            depth: 0,
            size: 0,
        }
    }

    pub fn enter_depth(&mut self) -> JsonResult<()> {
        self.depth += 1;
        if self.depth > self.options.max_depth {
            return Err(JsonError::DepthLimitExceeded {
                current_depth: self.depth,
                max_depth: self.options.max_depth,
            });
        }
        Ok(())
    }

    pub fn exit_depth(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    pub fn add_size(&mut self, additional: usize) -> JsonResult<()> {
        self.size += additional;
        if self.size > self.options.max_size {
            return Err(JsonError::CharacterLimitExceeded {
                current_size: self.size,
                max_size: self.options.max_size,
            });
        }
        Ok(())
    }

    pub fn advance(&mut self, ch: char) {
        self.position.advance(ch);
    }

    pub fn advance_by(&mut self, text: &str) {
        self.position.advance_by(text);
    }

    pub fn syntax_error(&self, message: impl Into<String>) -> JsonError {
        JsonError::SyntaxError {
            message: message.into(),
            line: self.position.line,
            column: self.position.column,
            position: self.position.offset,
        }
    }

    pub fn unexpected_eof(&self, expected: impl Into<String>) -> JsonError {
        JsonError::UnexpectedEof {
            expected: expected.into(),
            position: self.position.offset,
        }
    }

    pub fn invalid_structure(&self, message: impl Into<String>) -> JsonError {
        JsonError::InvalidStructure {
            message: message.into(),
            position: self.position.offset,
        }
    }

    pub fn invalid_unicode(&self, message: impl Into<String>) -> JsonError {
        JsonError::InvalidUnicode {
            message: message.into(),
            position: self.position.offset,
        }
    }
}

/// Convenience type alias for JSON results
pub type JsonResult<T> = std::result::Result<T, JsonError>;

/// Helper trait for converting JsonResult to Lambdust Result
pub trait IntoLambdustResult<T> {
    fn into_lambdust_result(self) -> Result<T>;
}

impl<T> IntoLambdustResult<T> for JsonResult<T> {
    fn into_lambdust_result(self) -> Result<T> {
        self.map_err(|err| Box::new(Error::from(err)))
    }
}

/// SRFI-180 error checking procedure: (json-error? obj)
pub fn is_json_error(value: &crate::eval::Value) -> bool {
    // In the current implementation, JSON errors are not stored as values
    // This would need to be implemented if we want to capture JSON errors as values
    // For now, always return false
    false
}

/// SRFI-180 error reason procedure: (json-error-reason error)
pub fn json_error_reason(error: &JsonError) -> String {
    error.to_string()
}
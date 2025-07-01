//! Error types for the Lambdust interpreter

use thiserror::Error;

/// Result type alias for Lambdust operations
pub type Result<T> = std::result::Result<T, LambdustError>;

/// Main error type for the Lambdust interpreter
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LambdustError {
    /// Lexical analysis errors
    #[error("Lexer error: {0}")]
    LexerError(String),

    /// Parse errors
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Runtime evaluation errors
    #[error("Runtime error: {0}")]
    RuntimeError(String),

    /// Type errors
    #[error("Type error: {0}")]
    TypeError(String),

    /// Undefined variable errors
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    /// Arity errors (wrong number of arguments)
    #[error("Arity error: expected {expected}, got {actual}")]
    ArityError { 
        /// Expected number of arguments
        expected: usize, 
        /// Actual number of arguments provided
        actual: usize 
    },

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// I/O errors
    #[error("I/O error: {0}")]
    IoError(String),

    /// Stack overflow (for detecting infinite recursion)
    #[error("Stack overflow")]
    StackOverflow,

    /// Macro expansion errors
    #[error("Macro error: {0}")]
    MacroError(String),

    /// Syntax errors in special forms
    #[error("Syntax error: {0}")]
    SyntaxError(String),
}

impl From<std::io::Error> for LambdustError {
    fn from(err: std::io::Error) -> Self {
        LambdustError::IoError(err.to_string())
    }
}
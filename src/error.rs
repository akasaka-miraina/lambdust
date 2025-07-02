//! Error types for the Lambdust interpreter

use thiserror::Error;

/// Result type alias for Lambdust operations
pub type Result<T> = std::result::Result<T, LambdustError>;

/// Source position information for error reporting
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePosition {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based) 
    pub column: usize,
    /// Character offset from start of input (0-based)
    pub offset: usize,
}

impl SourcePosition {
    /// Create a new source position
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }

    /// Default position for when position is unknown
    pub fn unknown() -> Self {
        Self { line: 0, column: 0, offset: 0 }
    }
}

impl std::fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 && self.column == 0 {
            write!(f, "<unknown position>")
        } else {
            write!(f, "line {}, column {}", self.line, self.column)
        }
    }
}

/// Source span representing a range in the source code
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    /// Start position
    pub start: SourcePosition,
    /// End position
    pub end: SourcePosition,
    /// Source file name (optional)
    pub filename: Option<String>,
}

impl SourceSpan {
    /// Create a new source span
    pub fn new(start: SourcePosition, end: SourcePosition) -> Self {
        Self { start, end, filename: None }
    }

    /// Create a source span with filename
    pub fn with_filename(start: SourcePosition, end: SourcePosition, filename: String) -> Self {
        Self { start, end, filename: Some(filename) }
    }

    /// Create an unknown span
    pub fn unknown() -> Self {
        Self {
            start: SourcePosition::unknown(),
            end: SourcePosition::unknown(),
            filename: None,
        }
    }

    /// Create a point span at a single position
    pub fn point(pos: SourcePosition) -> Self {
        Self {
            start: pos.clone(),
            end: pos,
            filename: None,
        }
    }
}

impl std::fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref filename) = self.filename {
            write!(f, "{}:{}", filename, self.start)
        } else {
            write!(f, "{}", self.start)
        }
    }
}

/// Stack frame information for error tracing
#[derive(Debug, Clone, PartialEq)]
pub struct StackFrame {
    /// Function or form name
    pub name: String,
    /// Source location where this frame was called
    pub location: SourceSpan,
    /// Frame type (function call, special form, etc.)
    pub frame_type: FrameType,
}

/// Type of stack frame
#[derive(Debug, Clone, PartialEq)]
pub enum FrameType {
    /// User-defined function call
    Function,
    /// Built-in function call
    Builtin,
    /// Special form evaluation
    SpecialForm,
    /// Macro expansion
    Macro,
    /// Top-level evaluation
    TopLevel,
}

impl std::fmt::Display for StackFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.frame_type {
            FrameType::Function => write!(f, "  at {} ({})", self.name, self.location),
            FrameType::Builtin => write!(f, "  at {} <builtin> ({})", self.name, self.location),
            FrameType::SpecialForm => write!(f, "  in {} form ({})", self.name, self.location),
            FrameType::Macro => write!(f, "  in {} macro ({})", self.name, self.location),
            FrameType::TopLevel => write!(f, "  at top level ({})", self.location),
        }
    }
}

/// Main error type for the Lambdust interpreter
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LambdustError {
    /// Lexical analysis errors
    #[error("Lexer error: {message}")]
    LexerError {
        /// Error message
        message: String,
        /// Source location where error occurred
        location: SourceSpan,
    },

    /// Parse errors
    #[error("Parse error: {message}")]
    ParseError {
        /// Error message
        message: String,
        /// Source location where error occurred
        location: SourceSpan,
    },

    /// Runtime evaluation errors
    #[error("Runtime error: {message}")]
    RuntimeError {
        /// Error message
        message: String,
        /// Source location where error occurred
        location: SourceSpan,
        /// Call stack trace
        stack_trace: Vec<StackFrame>,
    },

    /// Type errors
    #[error("Type error: {message}")]
    TypeError {
        /// Error message
        message: String,
        /// Source location where error occurred
        location: SourceSpan,
        /// Call stack trace
        stack_trace: Vec<StackFrame>,
    },

    /// Undefined variable errors
    #[error("Undefined variable: {variable}")]
    UndefinedVariable {
        /// Variable name
        variable: String,
        /// Source location where error occurred
        location: SourceSpan,
        /// Call stack trace
        stack_trace: Vec<StackFrame>,
    },

    /// Arity errors (wrong number of arguments)
    #[error("Arity error: expected {expected}, got {actual}")]
    ArityError {
        /// Expected number of arguments
        expected: usize,
        /// Actual number of arguments provided
        actual: usize,
        /// Function name
        function: String,
        /// Source location where error occurred
        location: SourceSpan,
        /// Call stack trace
        stack_trace: Vec<StackFrame>,
    },

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero {
        /// Source location where error occurred
        location: SourceSpan,
        /// Call stack trace
        stack_trace: Vec<StackFrame>,
    },

    /// I/O errors
    #[error("I/O error: {message}")]
    IoError {
        /// Error message
        message: String,
        /// Source location where error occurred (if available)
        location: Option<SourceSpan>,
    },

    /// Stack overflow (for detecting infinite recursion)
    #[error("Stack overflow")]
    StackOverflow {
        /// Source location where overflow was detected
        location: SourceSpan,
        /// Call stack trace (may be truncated)
        stack_trace: Vec<StackFrame>,
    },

    /// Macro expansion errors
    #[error("Macro error: {message}")]
    MacroError {
        /// Error message
        message: String,
        /// Source location where error occurred
        location: SourceSpan,
        /// Call stack trace
        stack_trace: Vec<StackFrame>,
    },

    /// Syntax errors in special forms
    #[error("Syntax error: {message}")]
    SyntaxError {
        /// Error message
        message: String,
        /// Source location where error occurred
        location: SourceSpan,
    },
}

impl From<std::io::Error> for LambdustError {
    fn from(err: std::io::Error) -> Self {
        LambdustError::IoError {
            message: err.to_string(),
            location: None,
        }
    }
}

impl LambdustError {
    /// Create a simple runtime error without location info (for backward compatibility)
    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::RuntimeError {
            message: message.into(),
            location: SourceSpan::unknown(),
            stack_trace: Vec::new(),
        }
    }


    /// Legacy constructor for TypeError (for easier migration)
    pub fn type_error_old(message: String) -> Self {
        Self::type_error(message)
    }

    /// Legacy constructor for UndefinedVariable (for easier migration)
    pub fn undefined_variable_old(variable: String) -> Self {
        Self::undefined_variable(variable)
    }

    /// Legacy constructor for SyntaxError (for easier migration)
    pub fn syntax_error_old(message: String) -> Self {
        Self::syntax_error(message)
    }

    /// Legacy constructor for LexerError (for easier migration)
    pub fn lexer_error_old(message: String) -> Self {
        Self::LexerError {
            message,
            location: SourceSpan::unknown(),
        }
    }

    /// Legacy constructor for ParseError (for easier migration)
    pub fn parse_error_old(message: String) -> Self {
        Self::ParseError {
            message,
            location: SourceSpan::unknown(),
        }
    }

    /// Legacy constructor for ArityError (for easier migration) 
    pub fn arity_error_old(expected: usize, actual: usize) -> Self {
        Self::arity_error(expected, actual)
    }

    /// Legacy struct-style constructor for ArityError
    pub fn arity_error_struct(expected: usize, actual: usize) -> Self {
        Self::ArityError { expected, actual, function: "<unknown>".to_string(), location: SourceSpan::unknown(), stack_trace: Vec::new() }
    }

    /// Legacy constructor for DivisionByZero (for easier migration)
    pub fn division_by_zero_old() -> Self {
        Self::DivisionByZero {
            location: SourceSpan::unknown(),
            stack_trace: Vec::new(),
        }
    }

    /// Legacy constructor for StackOverflow (for easier migration)
    pub fn stack_overflow_old() -> Self {
        Self::StackOverflow {
            location: SourceSpan::unknown(),
            stack_trace: Vec::new(),
        }
    }

    /// Legacy constructor for MacroError (for easier migration)
    pub fn macro_error_old(message: String) -> Self {
        Self::MacroError {
            message,
            location: SourceSpan::unknown(),
            stack_trace: Vec::new(),
        }
    }

    /// Legacy constructor for IoError (for easier migration)
    pub fn io_error_old(message: String) -> Self {
        Self::IoError {
            message,
            location: None,
        }
    }

    /// Create a simple type error without location info (for backward compatibility)
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
            location: SourceSpan::unknown(),
            stack_trace: Vec::new(),
        }
    }

    /// Create a simple arity error without location info (for backward compatibility)
    pub fn arity_error(expected: usize, actual: usize) -> Self {
        Self::ArityError {
            expected,
            actual,
            function: "<unknown>".to_string(),
            location: SourceSpan::unknown(),
            stack_trace: Vec::new(),
        }
    }

    /// Create a simple undefined variable error without location info (for backward compatibility)
    pub fn undefined_variable(variable: impl Into<String>) -> Self {
        Self::UndefinedVariable {
            variable: variable.into(),
            location: SourceSpan::unknown(),
            stack_trace: Vec::new(),
        }
    }

    /// Create a simple syntax error without location info (for backward compatibility)
    pub fn syntax_error(message: impl Into<String>) -> Self {
        Self::SyntaxError {
            message: message.into(),
            location: SourceSpan::unknown(),
        }
    }

    /// Add a stack frame to the error's stack trace
    pub fn with_stack_frame(mut self, frame: StackFrame) -> Self {
        match &mut self {
            Self::RuntimeError { stack_trace, .. }
            | Self::TypeError { stack_trace, .. }
            | Self::UndefinedVariable { stack_trace, .. }
            | Self::ArityError { stack_trace, .. }
            | Self::DivisionByZero { stack_trace, .. }
            | Self::StackOverflow { stack_trace, .. }
            | Self::MacroError { stack_trace, .. } => {
                stack_trace.push(frame);
            }
            _ => {} // Some error types don't have stack traces
        }
        self
    }

    /// Set the location for errors that support it
    pub fn with_location(mut self, location: SourceSpan) -> Self {
        match &mut self {
            Self::LexerError { location: loc, .. }
            | Self::ParseError { location: loc, .. }
            | Self::RuntimeError { location: loc, .. }
            | Self::TypeError { location: loc, .. }
            | Self::UndefinedVariable { location: loc, .. }
            | Self::ArityError { location: loc, .. }
            | Self::DivisionByZero { location: loc, .. }
            | Self::StackOverflow { location: loc, .. }
            | Self::MacroError { location: loc, .. }
            | Self::SyntaxError { location: loc, .. } => {
                *loc = location;
            }
            Self::IoError { location: loc, .. } => {
                *loc = Some(location);
            }
        }
        self
    }

    /// Generate a detailed error report with stack trace
    pub fn format_detailed(&self) -> String {
        let mut output = String::new();
        
        // Main error message
        output.push_str(&format!("Error: {}\n", self));

        // Add location information
        match self {
            Self::LexerError { location, .. }
            | Self::ParseError { location, .. }
            | Self::RuntimeError { location, .. }
            | Self::TypeError { location, .. }
            | Self::UndefinedVariable { location, .. }
            | Self::ArityError { location, .. }
            | Self::DivisionByZero { location, .. }
            | Self::StackOverflow { location, .. }
            | Self::MacroError { location, .. }
            | Self::SyntaxError { location, .. } => {
                if *location != SourceSpan::unknown() {
                    output.push_str(&format!("  at {}\n", location));
                }
            }
            Self::IoError { location: Some(location), .. } => {
                output.push_str(&format!("  at {}\n", location));
            }
            _ => {}
        }

        // Add stack trace
        let stack_trace = match self {
            Self::RuntimeError { stack_trace, .. }
            | Self::TypeError { stack_trace, .. }
            | Self::UndefinedVariable { stack_trace, .. }
            | Self::ArityError { stack_trace, .. }
            | Self::DivisionByZero { stack_trace, .. }
            | Self::StackOverflow { stack_trace, .. }
            | Self::MacroError { stack_trace, .. } => stack_trace,
            _ => return output,
        };

        if !stack_trace.is_empty() {
            output.push_str("\nStack trace:\n");
            for frame in stack_trace {
                output.push_str(&format!("{}\n", frame));
            }
        }

        output
    }
}

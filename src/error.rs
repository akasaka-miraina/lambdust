//! Error types for the Lambdust interpreter

use std::fmt::Write;
use thiserror::Error;

/// Result type alias for Lambdust operations
#[allow(clippy::result_large_err)]
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
    #[must_use] pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Default position for when position is unknown
    #[must_use] pub fn unknown() -> Self {
        Self {
            line: 0,
            column: 0,
            offset: 0,
        }
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
    #[must_use] pub fn new(start: SourcePosition, end: SourcePosition) -> Self {
        Self {
            start,
            end,
            filename: None,
        }
    }

    /// Create a source span with filename
    #[must_use] pub fn with_filename(start: SourcePosition, end: SourcePosition, filename: String) -> Self {
        Self {
            start,
            end,
            filename: Some(filename),
        }
    }

    /// Create an unknown span
    #[must_use] pub fn unknown() -> Self {
        Self {
            start: SourcePosition::unknown(),
            end: SourcePosition::unknown(),
            filename: None,
        }
    }

    /// Create a point span at a single position
    #[must_use] pub fn point(pos: SourcePosition) -> Self {
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

/// Context information for runtime errors
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext {
    /// Source location where error occurred
    pub location: SourceSpan,
    /// Call stack trace
    pub stack_trace: Vec<StackFrame>,
}

impl ErrorContext {
    /// Create new error context
    #[must_use] pub fn new(location: SourceSpan, stack_trace: Vec<StackFrame>) -> Self {
        Self {
            location,
            stack_trace,
        }
    }

    /// Create error context with unknown location
    #[must_use] pub fn unknown() -> Self {
        Self {
            location: SourceSpan::unknown(),
            stack_trace: Vec::new(),
        }
    }
}

/// Main error type for the Lambdust interpreter
#[allow(clippy::large_enum_variant)]
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
        /// Error context (boxed to reduce size)
        context: Box<ErrorContext>,
    },

    /// Type errors
    #[error("Type error: {message}")]
    TypeError {
        /// Error message
        message: String,
        /// Error context (boxed to reduce size)
        context: Box<ErrorContext>,
    },

    /// Undefined variable errors
    #[error("Undefined variable: {variable}")]
    UndefinedVariable {
        /// Variable name
        variable: String,
        /// Error context (boxed to reduce size)
        context: Box<ErrorContext>,
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
        /// Error context (boxed to reduce size)
        context: Box<ErrorContext>,
    },

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero {
        /// Error context (boxed to reduce size)
        context: Box<ErrorContext>,
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
        /// Error context (boxed to reduce size)
        context: Box<ErrorContext>,
    },

    /// Macro expansion errors
    #[error("Macro error: {message}")]
    MacroError {
        /// Error message
        message: String,
        /// Error context (boxed to reduce size)
        context: Box<ErrorContext>,
    },

    /// Syntax errors in special forms
    #[error("Syntax error: {message}")]
    SyntaxError {
        /// Error message
        message: String,
        /// Source location where error occurred
        location: SourceSpan,
    },
    /// Optimization signals between `RuntimeExecutor` and Evaluator
    /// Used for proper responsibility separation - not actual errors
    #[error("Optimization signal: {signal_type}")]
    OptimizationSignal {
        /// Signal type indicating optimization status
        signal_type: String,
        /// Expression to be processed
        expression: crate::ast::Expr,
        /// Environment for evaluation
        environment: std::rc::Rc<crate::environment::Environment>,
        /// Continuation for evaluation
        continuation: crate::evaluator::Continuation,
    },

    /// Custom error type for specific error conditions
    #[error("Custom error: {message}")]
    CustomError {
        /// Error message
        message: String,
        /// Error context (boxed to reduce size)
        context: Box<ErrorContext>,
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
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Legacy constructor for `TypeError` (for easier migration)
    #[must_use] pub fn type_error_old(message: String) -> Self {
        Self::type_error(message)
    }

    /// Legacy constructor for `UndefinedVariable` (for easier migration)
    #[must_use] pub fn undefined_variable_old(variable: String) -> Self {
        Self::undefined_variable(variable)
    }

    /// Legacy constructor for `SyntaxError` (for easier migration)
    #[must_use] pub fn syntax_error_old(message: String) -> Self {
        Self::syntax_error(message)
    }

    /// Legacy constructor for `LexerError` (for easier migration)
    #[must_use] pub fn lexer_error_old(message: String) -> Self {
        Self::LexerError {
            message,
            location: SourceSpan::unknown(),
        }
    }

    /// Legacy constructor for `ParseError` (for easier migration)
    #[must_use] pub fn parse_error_old(message: String) -> Self {
        Self::ParseError {
            message,
            location: SourceSpan::unknown(),
        }
    }

    /// Legacy constructor for `ArityError` (for easier migration)
    #[must_use] pub fn arity_error_old(expected: usize, actual: usize) -> Self {
        Self::arity_error(expected, actual)
    }

    /// Legacy struct-style constructor for `ArityError`
    #[must_use] pub fn arity_error_struct(expected: usize, actual: usize) -> Self {
        Self::ArityError {
            expected,
            actual,
            function: "<unknown>".to_string(),
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Legacy constructor for `DivisionByZero` (for easier migration)
    #[must_use] pub fn division_by_zero_old() -> Self {
        Self::DivisionByZero {
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Legacy constructor for `StackOverflow` (for easier migration)
    #[must_use] pub fn stack_overflow_old() -> Self {
        Self::StackOverflow {
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Legacy constructor for `MacroError` (for easier migration)
    #[must_use] pub fn macro_error_old(message: String) -> Self {
        Self::MacroError {
            message,
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Legacy constructor for `IoError` (for easier migration)
    #[must_use] pub fn io_error_old(message: String) -> Self {
        Self::IoError {
            message,
            location: None,
        }
    }

    /// Create a simple type error without location info (for backward compatibility)
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::TypeError {
            message: message.into(),
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Create a simple arity error without location info (for backward compatibility)
    #[must_use] pub fn arity_error(expected: usize, actual: usize) -> Self {
        Self::ArityError {
            expected,
            actual,
            function: "<unknown>".to_string(),
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Create an arity error for a range of acceptable argument counts
    #[must_use] pub fn arity_error_range(min: usize, max: usize, actual: usize) -> Self {
        Self::ArityError {
            expected: min, // Use min as the primary expected value
            actual,
            function: format!("expected {min}-{max} arguments"),
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Create an arity error for a minimum number of arguments
    #[must_use] pub fn arity_error_min(min: usize, actual: usize) -> Self {
        Self::ArityError {
            expected: min,
            actual,
            function: format!("expected at least {min} arguments"),
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Create a simple undefined variable error without location info (for backward compatibility)
    pub fn undefined_variable(variable: impl Into<String>) -> Self {
        Self::UndefinedVariable {
            variable: variable.into(),
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Create a simple syntax error without location info (for backward compatibility)
    pub fn syntax_error(message: impl Into<String>) -> Self {
        Self::SyntaxError {
            message: message.into(),
            location: SourceSpan::unknown(),
        }
    }

    /// Create an optimization signal for RuntimeExecutor-Evaluator communication
    #[must_use] pub fn optimization_signal(
        signal_type: String,
        expression: crate::ast::Expr,
        environment: std::rc::Rc<crate::environment::Environment>,
        continuation: crate::evaluator::Continuation,
    ) -> Self {
        Self::OptimizationSignal {
            signal_type,
            expression,
            environment,
            continuation,
        }
    }

    /// Create a simple parse error without location info (for backward compatibility)
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError {
            message: message.into(),
            location: SourceSpan::unknown(),
        }
    }

    /// Create a simple lexer error without location info (for backward compatibility)
    pub fn lexer_error(message: impl Into<String>) -> Self {
        Self::LexerError {
            message: message.into(),
            location: SourceSpan::unknown(),
        }
    }

    /// Create a simple io error without location info (for backward compatibility)
    pub fn io_error(message: impl Into<String>) -> Self {
        Self::IoError {
            message: message.into(),
            location: None,
        }
    }

    /// Create a simple division by zero error without location info (for backward compatibility)
    #[must_use] pub fn division_by_zero() -> Self {
        Self::DivisionByZero {
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Create a simple stack overflow error without location info (for backward compatibility)
    #[must_use] pub fn stack_overflow() -> Self {
        Self::StackOverflow {
            context: Box::new(ErrorContext::unknown()),
        }
    }

    /// Add a stack frame to the error's stack trace
    #[must_use] pub fn with_stack_frame(mut self, frame: StackFrame) -> Self {
        match &mut self {
            Self::RuntimeError { context, .. }
            | Self::TypeError { context, .. }
            | Self::UndefinedVariable { context, .. }
            | Self::ArityError { context, .. }
            | Self::DivisionByZero { context, .. }
            | Self::StackOverflow { context, .. }
            | Self::MacroError { context, .. } => {
                context.stack_trace.push(frame);
            }
            _ => {} // Some error types don't have stack traces
        }
        self
    }

    /// Set the location for errors that support it
    #[must_use] pub fn with_location(mut self, location: SourceSpan) -> Self {
        match &mut self {
            Self::LexerError { location: loc, .. }
            | Self::ParseError { location: loc, .. }
            | Self::SyntaxError { location: loc, .. } => {
                *loc = location;
            }
            Self::RuntimeError { context, .. }
            | Self::TypeError { context, .. }
            | Self::UndefinedVariable { context, .. }
            | Self::ArityError { context, .. }
            | Self::DivisionByZero { context, .. }
            | Self::StackOverflow { context, .. }
            | Self::MacroError { context, .. } => {
                context.location = location;
            }
            Self::IoError { location: loc, .. } => {
                *loc = Some(location);
            }
            Self::OptimizationSignal { .. } | Self::CustomError { .. } => {
                // OptimizationSignal doesn't support location setting
            }
        }
        self
    }

    /// Generate a detailed error report with stack trace
    #[must_use] pub fn format_detailed(&self) -> String {
        let mut output = String::new();

        // Main error message
        writeln!(output, "Error: {self}").unwrap();

        // Add location information
        match self {
            Self::LexerError { location, .. }
            | Self::ParseError { location, .. }
            | Self::SyntaxError { location, .. } => {
                if *location != SourceSpan::unknown() {
                    writeln!(output, "  at {location}").unwrap();
                }
            }
            Self::RuntimeError { context, .. }
            | Self::TypeError { context, .. }
            | Self::UndefinedVariable { context, .. }
            | Self::ArityError { context, .. }
            | Self::DivisionByZero { context, .. }
            | Self::StackOverflow { context, .. }
            | Self::MacroError { context, .. } => {
                if context.location != SourceSpan::unknown() {
                    writeln!(output, "  at {}", context.location).unwrap();
                }
            }
            Self::IoError {
                location: Some(location),
                ..
            } => {
                writeln!(output, "  at {location}").unwrap();
            }
            Self::IoError{ .. } => {}
            Self::OptimizationSignal { .. } => {
                // OptimizationSignal doesn't have location info to display
            }
            Self::CustomError { .. } => {
                // CustomError doesn't have location info to display
            }
        }

        // Add stack trace
        let stack_trace = match self {
            Self::RuntimeError { context, .. }
            | Self::TypeError { context, .. }
            | Self::UndefinedVariable { context, .. }
            | Self::ArityError { context, .. }
            | Self::DivisionByZero { context, .. }
            | Self::StackOverflow { context, .. }
            | Self::MacroError { context, .. } => &context.stack_trace,
            _ => return output,
        };

        if !stack_trace.is_empty() {
            output.push_str("\nStack trace:\n");
            for frame in stack_trace {
                writeln!(output, "{frame}").unwrap();
            }
        }

        output
    }
}

//! Unified error handling system for the evaluation engine.
//!
//! This module eliminates redundant error handling patterns in evaluation,
//! fast path operations, and runtime checks while providing rich context.

use crate::diagnostics::{UnifiedError, UnifiedResult, RuntimeError, ErrorSeverity, Span};
use crate::eval::Value;
use crate::ast::{Expr, Literal};

// Re-export unified error system for eval modules
pub use crate::{error_convert, propagate_error, validate_type, validate_arity, handle_result};


/// Evaluation-specific error categories for granular error handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalErrorKind {
    /// Type mismatch in operations
    TypeMismatch,
    /// Arity mismatch in function calls
    ArityMismatch,
    /// Undefined variable access
    UndefinedVariable,
    /// Undefined function call
    UndefinedFunction,
    /// Division by zero
    DivisionByZero,
    /// Index out of bounds
    IndexOutOfBounds,
    /// Invalid operation on value
    InvalidOperation,
    /// Stack overflow in recursion
    StackOverflow,
    /// Continuation capture failure
    ContinuationFailure,
    /// Macro expansion failure
    MacroExpansionFailure,
    /// Import/module failure
    ModuleFailure,
    /// Fast path operation failure
    FastPathFailure,
    /// Optimization failure
    OptimizationFailure,
}

impl EvalErrorKind {
    /// Returns the error code string for this eval error kind.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::TypeMismatch => "lambdust::eval::type_mismatch::error",
            Self::ArityMismatch => "lambdust::eval::arity::error",
            Self::UndefinedVariable => "lambdust::eval::undefined_var::error",
            Self::UndefinedFunction => "lambdust::eval::undefined_func::error",
            Self::DivisionByZero => "lambdust::eval::division_by_zero::error",
            Self::IndexOutOfBounds => "lambdust::eval::index_bounds::error",
            Self::InvalidOperation => "lambdust::eval::invalid_op::error",
            Self::StackOverflow => "lambdust::eval::stack_overflow::error",
            Self::ContinuationFailure => "lambdust::eval::continuation::error",
            Self::MacroExpansionFailure => "lambdust::eval::macro::error",
            Self::ModuleFailure => "lambdust::eval::module::error",
            Self::FastPathFailure => "lambdust::eval::fast_path::error",
            Self::OptimizationFailure => "lambdust::eval::optimization::error",
        }
    }

    /// Returns whether this error kind is recoverable.
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::StackOverflow => false,
            Self::FastPathFailure | Self::OptimizationFailure => true,
            _ => false,
        }
    }

    /// Returns the default severity for this error kind.
    pub fn default_severity(&self) -> ErrorSeverity {
        match self {
            Self::StackOverflow => ErrorSeverity::Critical,
            Self::DivisionByZero => ErrorSeverity::Error,
            Self::FastPathFailure | Self::OptimizationFailure => ErrorSeverity::Warning,
            _ => ErrorSeverity::Error,
        }
    }
}

/// Enhanced eval error with category-specific information.
#[derive(Debug, Clone)]
pub struct EvalUnifiedError {
    /// Base unified error
    pub base: UnifiedError,
    /// Eval-specific error kind
    pub eval_kind: EvalErrorKind,
    /// Expected value type (for type mismatches)
    pub expected_type: Option<String>,
    /// Actual value (for type mismatches)
    pub actual_value: Option<Value>,
    /// Evaluation context
    pub eval_context: EvalContext,
}

/// Evaluation context information for better error messages.
#[derive(Debug, Clone, Default)]
pub struct EvalContext {
    /// Function currently being evaluated
    pub current_function: Option<String>,
    /// Expression being evaluated
    pub current_expression: Option<String>,
    /// Call stack depth
    pub stack_depth: usize,
    /// Variable bindings in scope
    pub variables_in_scope: Vec<String>,
    /// Whether in tail position
    pub in_tail_position: bool,
    /// Fast path operation context
    pub fast_path_op: Option<String>,
}

impl EvalUnifiedError {
    /// Creates a new eval-specific unified error.
    pub fn new(
        eval_kind: EvalErrorKind,
        message: impl Into<String>,
    ) -> Self {
        let severity = eval_kind.default_severity();
        let base = UnifiedError::new(RuntimeError, message)
            .with_severity(severity)
            .with_context("eval_kind", format!("{eval_kind:?}"));

        Self {
            base,
            eval_kind,
            expected_type: None,
            actual_value: None,
            eval_context: EvalContext::default(),
        }
    }

    /// Builder pattern: adds type mismatch information.
    pub fn with_type_mismatch(mut self, expected: impl Into<String>, actual: Value) -> Self {
        let expected_str = expected.into();
        self.expected_type = Some(expected_str.clone());
        self.actual_value = Some(actual.clone());
        self.base = self.base
            .with_context("expected_type", expected_str)
            .with_context("actual_value", format!("{actual:?}"));
        self
    }

    /// Builder pattern: adds span information.
    pub fn with_span(mut self, span: Span) -> Self {
        self.base = self.base.with_span(span);
        self
    }

    /// Builder pattern: adds function context.
    pub fn with_function_context(mut self, function_name: impl Into<String>) -> Self {
        let func_name = function_name.into();
        self.eval_context.current_function = Some(func_name.clone());
        self.base = self.base.with_context("function", func_name);
        self
    }

    /// Builder pattern: adds expression context.
    pub fn with_expression_context(mut self, expr: &Expr) -> Self {
        let expr_str = format!("{expr:?}");
        self.eval_context.current_expression = Some(expr_str.clone());
        self.base = self.base.with_context("expression", expr_str);
        self
    }

    /// Builder pattern: adds stack context.
    pub fn with_stack_context(mut self, depth: usize, in_tail: bool) -> Self {
        self.eval_context.stack_depth = depth;
        self.eval_context.in_tail_position = in_tail;
        self.base = self.base
            .with_context("stack_depth", depth.to_string())
            .with_context("tail_position", in_tail.to_string());
        self
    }

    /// Builder pattern: adds fast path context.
    pub fn with_fast_path_context(mut self, operation: impl Into<String>) -> Self {
        let op_name = operation.into();
        self.eval_context.fast_path_op = Some(op_name.clone());
        self.base = self.base.with_context("fast_path_op", op_name);
        self
    }

    /// Builder pattern: adds source error.
    pub fn with_source(mut self, source: UnifiedError) -> Self {
        self.base = self.base.with_source(source);
        self
    }

    /// Converts to base UnifiedError for compatibility.
    pub fn into_unified(self) -> UnifiedError {
        self.base
    }

    /// Converts to boxed UnifiedError.
    pub fn boxed(self) -> Box<UnifiedError> {
        self.base.boxed()
    }

    /// Returns a user-friendly explanation of the eval error.
    pub fn explanation(&self) -> String {
        match (&self.eval_kind, &self.expected_type, &self.actual_value) {
            (EvalErrorKind::TypeMismatch, Some(expected), Some(actual)) => {
                format!("Expected {}, but got {}", 
                    expected, 
                    self.format_value_type(actual))
            }
            (EvalErrorKind::ArityMismatch, _, _) => {
                if let Some(func) = &self.eval_context.current_function {
                    format!("Wrong number of arguments to function '{func}'")
                } else {
                    "Wrong number of arguments".to_string()
                }
            }
            (EvalErrorKind::UndefinedVariable, _, _) => {
                "Variable is not defined in current scope".to_string()
            }
            (EvalErrorKind::DivisionByZero, _, _) => {
                "Division by zero is undefined".to_string()
            }
            (EvalErrorKind::StackOverflow, _, _) => {
                format!("Stack overflow at depth {} - possible infinite recursion", 
                    self.eval_context.stack_depth)
            }
            _ => self.base.message.clone(),
        }
    }

    /// Formats a value type for user-friendly display.
    fn format_value_type(&self, value: &Value) -> String {
        Self::format_value_type_static(value)
    }

    /// Formats a value type for user-friendly display (static version).
    fn format_value_type_static(value: &Value) -> String {
        match value {
            Value::Nil => "nil".to_string(),
            Value::Literal(Literal::Boolean(_)) => "boolean".to_string(),
            Value::Literal(Literal::ExactInteger(_)) => "exact integer".to_string(),
            Value::Literal(Literal::InexactReal(_)) => "inexact real".to_string(),
            Value::Literal(Literal::String(_)) => "string".to_string(),
            Value::Literal(Literal::Character(_)) => "character".to_string(),
            Value::Symbol(_) => "symbol".to_string(),
            Value::Pair(_, _) => "pair".to_string(),
            Value::Vector(_) => "vector".to_string(),
            Value::Procedure(_) => "procedure".to_string(),
            Value::Primitive(_) => "primitive procedure".to_string(),
            Value::Port(_) => "port".to_string(),
            _ => "unknown".to_string(),
        }
    }

    /// Returns suggested fixes for this error.
    pub fn suggested_fixes(&self) -> Vec<String> {
        let mut fixes = Vec::new();
        
        match self.eval_kind {
            EvalErrorKind::TypeMismatch => {
                if let Some(expected) = &self.expected_type {
                    fixes.push(format!("Ensure the value is of type {expected}"));
                    fixes.push("Check function argument types".to_string());
                }
            }
            EvalErrorKind::ArityMismatch => {
                fixes.push("Check the function signature".to_string());
                fixes.push("Verify the number of arguments passed".to_string());
            }
            EvalErrorKind::UndefinedVariable => {
                fixes.push("Check variable name spelling".to_string());
                fixes.push("Ensure variable is defined before use".to_string());
                fixes.push("Check variable scope".to_string());
            }
            EvalErrorKind::StackOverflow => {
                fixes.push("Check for infinite recursion".to_string());
                fixes.push("Add base case to recursive function".to_string());
                fixes.push("Use tail recursion if possible".to_string());
            }
            _ => {}
        }
        
        fixes
    }

    /// Returns whether this error allows fallback to slower path.
    pub fn allows_fallback(&self) -> bool {
        self.eval_kind.is_recoverable()
    }
}

/// Macro for creating eval errors with comprehensive context.
#[macro_export]
macro_rules! eval_error {
    // Basic eval error
    ($kind:expr, $message:expr) => {
        $crate::eval::unified_eval_errors::EvalUnifiedError::new($kind, $message)
    };
    
    // Eval error with type mismatch
    ($kind:expr, $message:expr, expected: $expected:expr, actual: $actual:expr) => {
        $crate::eval::unified_eval_errors::EvalUnifiedError::new($kind, $message)
            .with_type_mismatch($expected, $actual)
    };
    
    // Eval error with span
    ($kind:expr, $message:expr, span: $span:expr) => {
        $crate::eval::unified_eval_errors::EvalUnifiedError::new($kind, $message)
            .with_span($span)
    };
    
    // Eval error with function context
    ($kind:expr, $message:expr, function: $func:expr) => {
        $crate::eval::unified_eval_errors::EvalUnifiedError::new($kind, $message)
            .with_function_context($func)
    };
    
    // Full eval error with all context
    ($kind:expr, $message:expr, 
     expected: $expected:expr, actual: $actual:expr,
     span: $span:expr, function: $func:expr) => {
        $crate::eval::unified_eval_errors::EvalUnifiedError::new($kind, $message)
            .with_type_mismatch($expected, $actual)
            .with_span($span)
            .with_function_context($func)
    };
}

/// Macro for fast path evaluation with automatic fallback.
#[macro_export]
macro_rules! fast_eval_try {
    // Fast path operation with fallback
    ($operation:expr, fallback: $fallback:expr) => {
        match $operation {
            Ok(result) => result,
            Err(error) => {
                if error.allows_fallback() {
                    $fallback
                } else {
                    return Err(error.into_unified());
                }
            }
        }
    };
    
    // Fast path operation with custom fallback logic
    ($operation:expr, fallback: |$err:ident| $fallback_logic:expr) => {
        match $operation {
            Ok(result) => result,
            Err($err) => {
                if $err.allows_fallback() {
                    $fallback_logic
                } else {
                    return Err($err.into_unified());
                }
            }
        }
    };
}

/// Specialized error constructors for common eval patterns.
impl EvalUnifiedError {
    /// Creates a type mismatch error.
    pub fn type_mismatch(expected: impl Into<String>, actual: Value) -> Self {
        let expected_str = expected.into();
        Self::new(
            EvalErrorKind::TypeMismatch,
            format!("Type mismatch: expected {}, got {}", 
                expected_str, 
                Self::format_value_type_static(&actual))
        ).with_type_mismatch(expected_str.clone(), actual)
    }

    /// Creates an arity mismatch error.
    pub fn arity_mismatch(
        function_name: impl Into<String>, 
        expected: usize, 
        actual: usize
    ) -> Self {
        let name = function_name.into();
        Self::new(
            EvalErrorKind::ArityMismatch,
            format!("Function '{name}' expects {expected} arguments, got {actual}")
        ).with_function_context(name)
    }

    /// Creates an undefined variable error.
    pub fn undefined_variable(variable_name: impl Into<String>) -> Self {
        let var_name = variable_name.into();
        let mut error = Self::new(
            EvalErrorKind::UndefinedVariable,
            format!("Undefined variable: '{var_name}'")
        );
        error.base = error.base.with_context("variable_name", var_name);
        error
    }

    /// Creates a division by zero error.
    pub fn division_by_zero() -> Self {
        Self::new(
            EvalErrorKind::DivisionByZero,
            "Division by zero"
        )
    }

    /// Creates a stack overflow error.
    pub fn stack_overflow(depth: usize) -> Self {
        Self::new(
            EvalErrorKind::StackOverflow,
            format!("Stack overflow at depth {depth}")
        ).with_stack_context(depth, false)
    }

    /// Creates a fast path failure error.
    pub fn fast_path_failed(operation: impl Into<String>, reason: impl Into<String>) -> Self {
        let op_name = operation.into();
        Self::new(
            EvalErrorKind::FastPathFailure,
            format!("Fast path failed for {}: {}", op_name, reason.into())
        ).with_fast_path_context(op_name)
    }
}

/// Result type specialized for evaluation operations.
pub type EvalResult<T> = Result<T, EvalUnifiedError>;

/// Conversion implementations for backward compatibility.
impl From<EvalUnifiedError> for UnifiedError {
    fn from(err: EvalUnifiedError) -> Self {
        err.base
    }
}

impl From<EvalUnifiedError> for Box<crate::diagnostics::Error> {
    fn from(err: EvalUnifiedError) -> Self {
        err.base.into()
    }
}

impl std::fmt::Display for EvalUnifiedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.eval_kind.error_code(), self.explanation())
    }
}

impl std::error::Error for EvalUnifiedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_mismatch_error() {
        let error = EvalUnifiedError::type_mismatch("number", Value::string("hello"));
        
        assert_eq!(error.eval_kind, EvalErrorKind::TypeMismatch);
        assert_eq!(error.expected_type, Some("number".to_string()));
        assert!(error.explanation().contains("Expected number"));
    }

    #[test]
    fn test_arity_mismatch_error() {
        let error = EvalUnifiedError::arity_mismatch("add", 2, 3);
        
        assert_eq!(error.eval_kind, EvalErrorKind::ArityMismatch);
        assert_eq!(error.eval_context.current_function, Some("add".to_string()));
    }

    #[test]
    fn test_undefined_variable_error() {
        let error = EvalUnifiedError::undefined_variable("unknown_var");
        
        assert_eq!(error.eval_kind, EvalErrorKind::UndefinedVariable);
        assert!(error.explanation().contains("not defined"));
    }

    #[test]
    fn test_stack_overflow_error() {
        let error = EvalUnifiedError::stack_overflow(1000);
        
        assert_eq!(error.eval_kind, EvalErrorKind::StackOverflow);
        assert_eq!(error.eval_context.stack_depth, 1000);
        assert!(!error.allows_fallback());
    }

    #[test]
    fn test_fast_path_failure_allows_fallback() {
        let error = EvalUnifiedError::fast_path_failed("add", "optimization failed");
        
        assert_eq!(error.eval_kind, EvalErrorKind::FastPathFailure);
        assert!(error.allows_fallback());
    }

    #[test]
    fn test_eval_error_macro() {
        let error = eval_error!(
            EvalErrorKind::TypeMismatch,
            "Type mismatch in addition",
            expected: "number",
            actual: Value::string("hello")
        );
        
        assert_eq!(error.eval_kind, EvalErrorKind::TypeMismatch);
        assert_eq!(error.expected_type, Some("number".to_string()));
    }
}
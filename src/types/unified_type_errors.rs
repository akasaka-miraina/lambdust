//! Unified error handling system for the type system.
//!
//! This module eliminates redundant error handling patterns in type checking,
//! inference, and constraint solving while providing rich contextual information.

use crate::diagnostics::{UnifiedError, UnifiedResult, TypeError, ErrorSeverity, Span};
use crate::types::{Type, TypeVar, TypeConstraint, Substitution};
use std::collections::HashMap;

// Re-export unified error system for type modules
pub use crate::{error_convert, propagate_error, handle_result, error_chain};


/// Type-specific error categories for granular error handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeErrorKind {
    /// Type unification failures
    UnificationFailure,
    /// Constraint solving failures
    ConstraintFailure,
    /// Type inference failures
    InferenceFailure,
    /// Kind checking failures
    KindError,
    /// Type class resolution failures
    TypeClassError,
    /// Occurs check failures
    OccursCheck,
    /// Infinite type errors
    InfiniteType,
    /// Missing type annotations
    MissingAnnotation,
    /// Type mismatch in application
    ApplicationMismatch,
    /// Unknown type variable
    UnknownTypeVar,
    /// Ambiguous type inference
    AmbiguousType,
    /// Dependent type verification failures
    DependentTypeError,
}

impl TypeErrorKind {
    /// Returns the error code string for this type error kind.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::UnificationFailure => "lambdust::types::unification::error",
            Self::ConstraintFailure => "lambdust::types::constraint::error",
            Self::InferenceFailure => "lambdust::types::inference::error",
            Self::KindError => "lambdust::types::kind::error",
            Self::TypeClassError => "lambdust::types::typeclass::error",
            Self::OccursCheck => "lambdust::types::occurs::error",
            Self::InfiniteType => "lambdust::types::infinite::error",
            Self::MissingAnnotation => "lambdust::types::annotation::error",
            Self::ApplicationMismatch => "lambdust::types::application::error",
            Self::UnknownTypeVar => "lambdust::types::variable::error",
            Self::AmbiguousType => "lambdust::types::ambiguous::error",
            Self::DependentTypeError => "lambdust::types::dependent::error",
        }
    }

    /// Returns whether this error kind is recoverable.
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::MissingAnnotation | Self::AmbiguousType)
    }

    /// Returns the default severity for this error kind.
    pub fn default_severity(&self) -> ErrorSeverity {
        match self {
            Self::InfiniteType | Self::OccursCheck => ErrorSeverity::Critical,
            Self::DependentTypeError => ErrorSeverity::Critical,
            Self::MissingAnnotation => ErrorSeverity::Warning,
            _ => ErrorSeverity::Error,
        }
    }
}

/// Enhanced type error with category-specific information.
#[derive(Debug, Clone)]
pub struct TypeUnifiedError {
    /// Base unified error
    pub base: UnifiedError,
    /// Type-specific error kind
    pub type_kind: TypeErrorKind,
    /// Expected type in mismatch
    pub expected_type: Option<Type>,
    /// Actual type in mismatch
    pub actual_type: Option<Type>,
    /// Type context information
    pub type_context: TypeContext,
    /// Failed constraint (if applicable)
    pub failed_constraint: Option<TypeConstraint>,
}

/// Type context information for better error messages.
#[derive(Debug, Clone, Default)]
pub struct TypeContext {
    /// Function being type-checked
    pub function_name: Option<String>,
    /// Expression being typed
    pub expression_context: Option<String>,
    /// Type variables in scope
    pub type_vars_in_scope: HashMap<String, TypeVar>,
    /// Current substitution
    pub current_substitution: Option<Substitution>,
    /// Type annotation source
    pub annotation_source: Option<String>,
}

impl TypeUnifiedError {
    /// Creates a new type-specific unified error.
    pub fn new(
        type_kind: TypeErrorKind,
        message: impl Into<String>,
    ) -> Self {
        let severity = type_kind.default_severity();
        let base = UnifiedError::new(TypeError, message)
            .with_severity(severity)
            .with_context("type_kind", format!("{type_kind:?}"));

        Self {
            base,
            type_kind,
            expected_type: None,
            actual_type: None,
            type_context: TypeContext::default(),
            failed_constraint: None,
        }
    }

    /// Builder pattern: adds type mismatch information.
    pub fn with_type_mismatch(mut self, expected: Type, actual: Type) -> Self {
        self.expected_type = Some(expected.clone());
        self.actual_type = Some(actual.clone());
        self.base = self.base
            .with_context("expected_type", format!("{expected:?}"))
            .with_context("actual_type", format!("{actual:?}"));
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
        self.type_context.function_name = Some(func_name.clone());
        self.base = self.base.with_context("function", func_name);
        self
    }

    /// Builder pattern: adds expression context.
    pub fn with_expression_context(mut self, expr_context: impl Into<String>) -> Self {
        let expr_ctx = expr_context.into();
        self.type_context.expression_context = Some(expr_ctx.clone());
        self.base = self.base.with_context("expression", expr_ctx);
        self
    }

    /// Builder pattern: adds failed constraint.
    pub fn with_failed_constraint(mut self, constraint: TypeConstraint) -> Self {
        self.failed_constraint = Some(constraint.clone());
        self.base = self.base.with_context("constraint", format!("{constraint:?}"));
        self
    }

    /// Builder pattern: adds current substitution context.
    pub fn with_substitution(mut self, substitution: Substitution) -> Self {
        self.type_context.current_substitution = Some(substitution.clone());
        self.base = self.base.with_context("substitution", format!("{substitution:?}"));
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

    /// Returns a user-friendly explanation of the type error.
    pub fn explanation(&self) -> String {
        match (&self.type_kind, &self.expected_type, &self.actual_type) {
            (TypeErrorKind::UnificationFailure, Some(expected), Some(actual)) => {
                format!("Cannot unify type '{}' with '{}'", 
                    self.format_type(expected), 
                    self.format_type(actual))
            }
            (TypeErrorKind::ApplicationMismatch, Some(expected), Some(actual)) => {
                format!("Function expects argument of type '{}', but got '{}'",
                    self.format_type(expected),
                    self.format_type(actual))
            }
            (TypeErrorKind::OccursCheck, Some(ty), _) => {
                format!("Infinite type detected: type variable occurs in '{}'", 
                    self.format_type(ty))
            }
            (TypeErrorKind::MissingAnnotation, _, _) => {
                "Type annotation required for ambiguous expression".to_string()
            }
            _ => self.base.message.clone(),
        }
    }

    /// Formats a type for user-friendly display.
    fn format_type(&self, ty: &Type) -> String {
        // This would be implemented with proper type formatting
        format!("{ty:?}")
    }

    /// Returns suggested fixes for this error.
    pub fn suggested_fixes(&self) -> Vec<String> {
        let mut fixes = Vec::new();
        
        match self.type_kind {
            TypeErrorKind::MissingAnnotation => {
                fixes.push("Add a type annotation to disambiguate".to_string());
                if let Some(func) = &self.type_context.function_name {
                    fixes.push(format!("Add type signature for function '{func}'"));
                }
            }
            TypeErrorKind::ApplicationMismatch => {
                fixes.push("Check the function signature".to_string());
                fixes.push("Verify argument types match parameters".to_string());
            }
            TypeErrorKind::UnificationFailure => {
                fixes.push("Check for type compatibility".to_string());
                if self.expected_type.is_some() && self.actual_type.is_some() {
                    fixes.push("Consider type conversion or casting".to_string());
                }
            }
            _ => {}
        }
        
        fixes
    }
}

/// Macro for creating type errors with comprehensive context.
#[macro_export]
macro_rules! type_error {
    // Basic type error
    ($kind:expr, $message:expr) => {
        $crate::types::unified_type_errors::TypeUnifiedError::new($kind, $message)
    };
    
    // Type error with mismatch
    ($kind:expr, $message:expr, expected: $expected:expr, actual: $actual:expr) => {
        $crate::types::unified_type_errors::TypeUnifiedError::new($kind, $message)
            .with_type_mismatch($expected, $actual)
    };
    
    // Type error with span
    ($kind:expr, $message:expr, span: $span:expr) => {
        $crate::types::unified_type_errors::TypeUnifiedError::new($kind, $message)
            .with_span($span)
    };
    
    // Type error with function context
    ($kind:expr, $message:expr, function: $func:expr) => {
        $crate::types::unified_type_errors::TypeUnifiedError::new($kind, $message)
            .with_function_context($func)
    };
    
    // Full type error with all context
    ($kind:expr, $message:expr, 
     expected: $expected:expr, actual: $actual:expr,
     span: $span:expr, function: $func:expr) => {
        $crate::types::unified_type_errors::TypeUnifiedError::new($kind, $message)
            .with_type_mismatch($expected, $actual)
            .with_span($span)
            .with_function_context($func)
    };
}

/// Macro for unification result handling.
#[macro_export]
macro_rules! unify_or_error {
    ($unify_result:expr, $expected:expr, $actual:expr) => {
        match $unify_result {
            Ok(substitution) => substitution,
            Err(_) => return Err(type_error!(
                $crate::types::unified_type_errors::TypeErrorKind::UnificationFailure,
                "Type unification failed",
                expected: $expected,
                actual: $actual
            ).into_unified()),
        }
    };
    
    ($unify_result:expr, $expected:expr, $actual:expr, span: $span:expr) => {
        match $unify_result {
            Ok(substitution) => substitution,
            Err(_) => return Err(type_error!(
                $crate::types::unified_type_errors::TypeErrorKind::UnificationFailure,
                "Type unification failed",
                expected: $expected,
                actual: $actual,
                span: $span
            ).into_unified()),
        }
    };
}

/// Specialized error constructors for common type patterns.
impl TypeUnifiedError {
    /// Creates a unification failure error.
    pub fn unification_failed(expected: Type, actual: Type, span: Option<Span>) -> Self {
        let mut error = Self::new(
            TypeErrorKind::UnificationFailure,
            format!("Cannot unify {expected:?} with {actual:?}")
        ).with_type_mismatch(expected, actual);
        
        if let Some(span) = span {
            error = error.with_span(span);
        }
        
        error
    }

    /// Creates an application mismatch error.
    pub fn application_mismatch(
        function_type: Type, 
        argument_type: Type, 
        span: Option<Span>
    ) -> Self {
        let mut error = Self::new(
            TypeErrorKind::ApplicationMismatch,
            "Function application type mismatch"
        ).with_type_mismatch(function_type, argument_type);
        
        if let Some(span) = span {
            error = error.with_span(span);
        }
        
        error
    }

    /// Creates an occurs check failure error.
    pub fn occurs_check_failed(type_var: TypeVar, ty: Type) -> Self {
        Self::new(
            TypeErrorKind::OccursCheck,
            format!("Type variable {type_var:?} occurs in type {ty:?}")
        ).with_type_mismatch(Type::Variable(type_var), ty)
    }

    /// Creates a missing annotation error.
    pub fn missing_annotation(context: impl Into<String>) -> Self {
        Self::new(
            TypeErrorKind::MissingAnnotation,
            "Type annotation required"
        ).with_expression_context(context)
    }

    /// Creates an ambiguous type error.
    pub fn ambiguous_type(context: impl Into<String>, candidates: Vec<Type>) -> Self {
        let mut error = Self::new(
            TypeErrorKind::AmbiguousType,
            format!("Ambiguous type - {} candidates", candidates.len())
        ).with_expression_context(context);
        
        for (i, candidate) in candidates.iter().enumerate() {
            error.base = error.base.with_context(
                format!("candidate_{i}"), 
                format!("{candidate:?}")
            );
        }
        
        error
    }
}

/// Result type specialized for type operations.
pub type TypeResult<T> = Result<T, TypeUnifiedError>;

/// Conversion implementations for backward compatibility.
impl From<TypeUnifiedError> for UnifiedError {
    fn from(err: TypeUnifiedError) -> Self {
        err.base
    }
}

impl From<TypeUnifiedError> for Box<crate::diagnostics::Error> {
    fn from(err: TypeUnifiedError) -> Self {
        err.base.into()
    }
}

impl std::fmt::Display for TypeUnifiedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.type_kind.error_code(), self.explanation())
    }
}

impl std::error::Error for TypeUnifiedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Type};

    #[test]
    fn test_type_error_creation() {
        let error = TypeUnifiedError::unification_failed(
            Type::Number,
            Type::String,
            Some(Span::new(10, 5))
        );
        
        assert_eq!(error.type_kind, TypeErrorKind::UnificationFailure);
        assert_eq!(error.expected_type, Some(Type::Number));
        assert_eq!(error.actual_type, Some(Type::String));
        assert_eq!(error.base.span, Some(Span::new(10, 5)));
    }

    #[test]
    fn test_occurs_check_error() {
        let type_var = TypeVar::with_name("a");
        let error = TypeUnifiedError::occurs_check_failed(type_var.clone(), Type::Variable(type_var.clone()));
        
        assert_eq!(error.type_kind, TypeErrorKind::OccursCheck);
        assert!(error.explanation().contains("occurs"));
    }

    #[test]
    fn test_missing_annotation_error() {
        let error = TypeUnifiedError::missing_annotation("lambda expression");
        
        assert_eq!(error.type_kind, TypeErrorKind::MissingAnnotation);
        assert_eq!(error.type_context.expression_context, Some("lambda expression".to_string()));
    }

    #[test]
    fn test_type_error_macro() {
        let error = type_error!(
            TypeErrorKind::ApplicationMismatch,
            "Function application failed",
            expected: Type::Number,
            actual: Type::String
        );
        
        assert_eq!(error.type_kind, TypeErrorKind::ApplicationMismatch);
        assert_eq!(error.expected_type, Some(Type::Number));
        assert_eq!(error.actual_type, Some(Type::String));
    }

    #[test]
    fn test_suggested_fixes() {
        let error = TypeUnifiedError::missing_annotation("ambiguous expression");
        let fixes = error.suggested_fixes();
        
        assert!(!fixes.is_empty());
        assert!(fixes[0].contains("type annotation"));
    }
}
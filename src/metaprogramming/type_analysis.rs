//! Type inference and constraint analysis components.

use crate::diagnostics::Span;
use super::analysis_types::{InferredType, ConstraintType};
use std::collections::HashMap;

/// Type information from inference.
#[derive(Debug, Clone)]
pub struct TypeInformation {
    /// Inferred types for expressions
    pub expression_types: HashMap<Span, InferredType>,
    /// Function signatures
    pub function_signatures: HashMap<String, FunctionSignature>,
    /// Type constraints
    pub constraints: Vec<TypeConstraint>,
    /// Type errors
    pub errors: Vec<TypeError>,
}

/// Function signature.
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Parameter types
    pub parameters: Vec<InferredType>,
    /// Return type
    pub return_type: InferredType,
    /// Whether it's variadic
    pub variadic: bool,
}

/// Type constraint.
#[derive(Debug, Clone)]
pub struct TypeConstraint {
    /// Left side of constraint
    pub left: InferredType,
    /// Right side of constraint
    pub right: InferredType,
    /// Constraint type
    pub constraint_type: ConstraintType,
    /// Source location
    pub location: Option<Span>,
}

/// Type error.
#[derive(Debug, Clone)]
pub struct TypeError {
    /// Error message
    pub message: String,
    /// Expected type
    pub expected: Option<InferredType>,
    /// Actual type
    pub actual: Option<InferredType>,
    /// Source location
    pub location: Option<Span>,
}

impl TypeInformation {
    /// Creates new empty type information.
    pub fn new() -> Self {
        Self {
            expression_types: HashMap::new(),
            function_signatures: HashMap::new(),
            constraints: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl Default for TypeInformation {
    fn default() -> Self {
        Self::new()
    }
}
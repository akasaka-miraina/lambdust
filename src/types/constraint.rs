use super::{Type, TypeVar};
use crate::diagnostics::Span;

/// Type class constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constraint {
    pub class: String,
    pub type_: Type,
}

/// A type constraint that must be satisfied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeConstraint {
    /// Equality constraint: two types must be equal
    Equal {
        left: Type,
        right: Type,
        span: Option<Span>,
        reason: String,
    },
    
    /// Instance constraint: a type must be an instance of a type class
    Instance {
        class: String,
        type_: Type,
        span: Option<Span>,
    },
    
    /// Subtype constraint: left must be a subtype of right
    Subtype {
        left: Type,
        right: Type,
        span: Option<Span>,
    },
    
    /// Default constraint: use default type if variable is unresolved
    Default {
        var: TypeVar,
        default_type: Type,
        span: Option<Span>,
    },
    
    /// Ambiguity constraint: warn about ambiguous types
    Ambiguous {
        vars: Vec<TypeVar>,
        span: Option<Span>,
    },
}

impl TypeConstraint {
    /// Creates an equality constraint.
    pub fn equal(left: Type, right: Type, span: Option<Span>, reason: impl Into<String>) -> Self {
        TypeConstraint::Equal {
            left,
            right,
            span,
            reason: reason.into(),
        }
    }
    
    /// Creates an instance constraint.
    pub fn instance(class: impl Into<String>, type_: Type, span: Option<Span>) -> Self {
        TypeConstraint::Instance {
            class: class.into(),
            type_,
            span,
        }
    }
    
    /// Gets the span associated with this constraint.
    pub fn span(&self) -> Option<Span> {
        match self {
            TypeConstraint::Equal { span, .. } |
            TypeConstraint::Instance { span, .. } |
            TypeConstraint::Subtype { span, .. } |
            TypeConstraint::Default { span, .. } |
            TypeConstraint::Ambiguous { span, .. } => *span,
        }
    }
}
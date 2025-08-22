use super::{Type, TypeVar};
use crate::diagnostics::Span;

/// Type class constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constraint {
    /// Name of the type class
    pub class: String,
    /// Type that must implement the type class
    pub type_: Type,
}

/// A type constraint that must be satisfied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeConstraint {
    /// Equality constraint: two types must be equal
    Equal {
        /// Left-hand side type of the equality
        left: Type,
        /// Right-hand side type of the equality
        right: Type,
        /// Source location where constraint originated
        span: Option<Span>,
        /// Human-readable reason for the constraint
        reason: String,
    },

    /// Instance constraint: a type must be an instance of a type class
    Instance {
        /// Name of the type class
        class: String,
        /// Type that must be an instance of the class
        type_: Type,
        /// Source location where constraint originated
        span: Option<Span>,
    },

    /// Subtype constraint: left must be a subtype of right
    Subtype {
        /// Subtype (more specific type)
        left: Type,
        /// Supertype (more general type)
        right: Type,
        /// Source location where constraint originated
        span: Option<Span>,
    },

    /// Default constraint: use default type if variable is unresolved
    Default {
        /// Type variable that may need defaulting
        var: TypeVar,
        /// Default type to use if variable remains unresolved
        default_type: Type,
        /// Source location where constraint originated
        span: Option<Span>,
    },

    /// Ambiguity constraint: warn about ambiguous types
    Ambiguous {
        /// Type variables involved in the ambiguity
        vars: Vec<TypeVar>,
        /// Source location where ambiguity was detected
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
            TypeConstraint::Equal { span, .. }
            | TypeConstraint::Instance { span, .. }
            | TypeConstraint::Subtype { span, .. }
            | TypeConstraint::Default { span, .. }
            | TypeConstraint::Ambiguous { span, .. } => *span,
        }
    }
}

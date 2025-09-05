//! Bridge module for compatibility with old dependent type system.
//!
//! This module provides compatibility types and functions for other modules
//! that depend on the old dependent type system until they can be migrated.

use crate::diagnostics::{Error, Result, Span};
use std::collections::HashMap;

// Re-export the new types with old names for compatibility
pub use super::dependent::core::DependentType;
pub use super::dependent::core::DependentTerm as Term;
pub use super::dependent::core::TypingContext as DependentContext;
pub use super::dependent::MartinLofTypeSystem as DependentTypeChecker;

// Compatibility implementations
impl From<crate::types::Type> for super::dependent::core::DependentType {
    fn from(ty: crate::types::Type) -> Self {
        match ty {
            crate::types::Type::Number => super::dependent::core::DependentType::Universe(0), // Simplified mapping
            crate::types::Type::String => super::dependent::core::DependentType::Universe(0),
            crate::types::Type::Boolean => super::dependent::core::DependentType::Universe(0),
            crate::types::Type::Function { params, return_type } => {
                if params.len() == 1 {
                    DependentType::Pi {
                        var: "_x".to_string(),
                        domain: Box::new(params[0].clone().into()),
                        codomain: Box::new((*return_type).into()),
                    }
                } else {
                    // Multiple parameters - curry
                    let mut result: DependentType = (*return_type).into();
                    for param in params.into_iter().rev() {
                        result = DependentType::Pi {
                            var: "_".to_string(),
                            domain: Box::new(param.into()),
                            codomain: Box::new(result),
                        };
                    }
                    result
                }
            }
            crate::types::Type::Pair(first, second) => {
                DependentType::Sigma {
                    var: "_".to_string(),
                    first: Box::new((*first).into()),
                    second: Box::new((*second).into()),
                }
            }
            _ => DependentType::Universe(0), // Default mapping
        }
    }
}

impl From<DependentType> for crate::types::Type {
    fn from(dep_ty: DependentType) -> Self {
        match dep_ty {
            DependentType::Universe(_) => crate::types::Type::Dynamic,
            DependentType::Pi { domain, codomain, .. } => {
                crate::types::Type::Function {
                    params: vec![(*domain).into()],
                    return_type: Box::new((*codomain).into()),
                }
            }
            DependentType::Sigma { first, second, .. } => {
                crate::types::Type::Pair(
                    Box::new((*first).into()),
                    Box::new((*second).into())
                )
            }
            DependentType::Identity { ty, .. } => (*ty).into(),
            DependentType::Inductive { .. } => crate::types::Type::Dynamic,
        }
    }
}

// Compatibility functions for gradual typing
pub fn is_dependent_type(ty: &crate::types::Type) -> bool {
    match ty {
        crate::types::Type::Function { .. } => true,
        crate::types::Type::Pair(_, _) => true,
        crate::types::Type::Variable(_) => true,
        _ => false,
    }
}

pub fn dependent_consistent(dep_type: &DependentType, regular_type: &crate::types::Type) -> bool {
    match (dep_type, regular_type) {
        (DependentType::Pi { domain, codomain, .. },
         crate::types::Type::Function { params, return_type }) => {
            params.len() == 1 &&
            **domain == params[0].clone().into() &&
            **codomain == (**return_type).clone().into()
        }
        (DependentType::Sigma { first, second, .. },
         crate::types::Type::Pair(a, b)) => {
            **first == (**a).clone().into() && **second == (**b).clone().into()
        }
        _ => false,
    }
}

pub fn extend_type_with_dependent(base_type: crate::types::Type) -> crate::types::Type {
    base_type
}

// Simple term constructors for compatibility
impl Term {
    pub fn var(name: String) -> Self {
        DependentTerm::Variable(name)
    }

    pub fn lambda(param: String, param_type: DependentType, body: Term) -> Self {
        DependentTerm::Lambda {
            param,
            param_type: Box::new(param_type),
            body: Box::new(body),
        }
    }

    pub fn app(function: Term, argument: Term) -> Self {
        DependentTerm::Application {
            function: Box::new(function),
            argument: Box::new(argument),
        }
    }

    pub fn pair(first: Term, second: Term) -> Self {
        DependentTerm::Pair {
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    pub fn fst(pair: Term) -> Self {
        DependentTerm::Projection {
            pair: Box::new(pair),
            is_first: true,
        }
    }

    pub fn snd(pair: Term) -> Self {
        DependentTerm::Projection {
            pair: Box::new(pair),
            is_first: false,
        }
    }
}

// Enum variants for pattern matching compatibility
impl Term {
    pub fn int(value: i64) -> Self {
        // Create a constructor for integer literals
        DependentTerm::Constructor {
            name: "Int".to_string(),
            args: vec![],
            result_type: Box::new(DependentType::Universe(0)),
        }
    }

    pub fn bool(value: bool) -> Self {
        DependentTerm::Constructor {
            name: if value { "True" } else { "False" }.to_string(),
            args: vec![],
            result_type: Box::new(DependentType::Universe(0)),
        }
    }

    pub fn string(value: String) -> Self {
        DependentTerm::Constructor {
            name: "String".to_string(),
            args: vec![],
            result_type: Box::new(DependentType::Universe(0)),
        }
    }

    pub fn list(elements: Vec<Term>) -> Self {
        DependentTerm::Constructor {
            name: "List".to_string(),
            args: elements,
            result_type: Box::new(DependentType::Universe(0)),
        }
    }
}

// Pattern enum for compatibility
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    Var(String),
    Constructor { name: String, args: Vec<Pattern> },
    Int(i64),
    Bool(bool),
    Pair(Box<Pattern>, Box<Pattern>),
    Wildcard,
}

impl From<super::dependent::core::Pattern> for Pattern {
    fn from(pat: super::dependent::core::Pattern) -> Self {
        match pat {
            super::dependent::core::Pattern::Variable(name) => Pattern::Var(name),
            super::dependent::core::Pattern::Constructor { name, args } => {
                Pattern::Constructor {
                    name,
                    args: args.into_iter().map(|p| p.into()).collect(),
                }
            }
        }
    }
}

impl From<Pattern> for super::dependent::core::Pattern {
    fn from(pat: Pattern) -> Self {
        match pat {
            Pattern::Var(name) => super::dependent::core::Pattern::Variable(name),
            Pattern::Constructor { name, args } => {
                super::dependent::core::Pattern::Constructor {
                    name,
                    args: args.into_iter().map(|p| p.into()).collect(),
                }
            }
            Pattern::Int(_) => super::dependent::core::Pattern::Variable("_int".to_string()),
            Pattern::Bool(_) => super::dependent::core::Pattern::Variable("_bool".to_string()),
            Pattern::Pair(p1, p2) => {
                super::dependent::core::Pattern::Constructor {
                    name: "Pair".to_string(),
                    args: vec![(*p1).into(), (*p2).into()],
                }
            }
            Pattern::Wildcard => super::dependent::core::Pattern::Variable("_".to_string()),
        }
    }
}
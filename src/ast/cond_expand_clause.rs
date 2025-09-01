//! SRFI-0 cond-expand clause and feature requirement definitions.
//!
//! This module defines the AST nodes for conditional expansion features,
//! implementing SRFI-0 (Feature-based conditional expansion of code).

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

/// A conditional expansion clause: (feature-requirement body ...)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CondExpandClause {
    /// Feature requirement that must be satisfied
    pub feature_requirement: FeatureRequirement,
    /// Body expressions to include if requirement is satisfied
    pub body: Vec<Spanned<super::Expr>>,
}

/// Feature requirement expressions for cond-expand
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureRequirement {
    /// Simple feature identifier: srfi-1, lambdust, etc.
    Feature(String),

    /// Library identifier: (srfi 1), (lambdust core), etc.
    Library(Vec<String>),

    /// Logical AND: (and req1 req2 ...)
    And(Vec<FeatureRequirement>),

    /// Logical OR: (or req1 req2 ...)
    Or(Vec<FeatureRequirement>),

    /// Logical NOT: (not req)
    Not(Box<FeatureRequirement>),
}

impl FeatureRequirement {
    /// Creates a simple feature requirement
    pub fn feature(name: impl Into<String>) -> Self {
        FeatureRequirement::Feature(name.into())
    }

    /// Creates a library requirement from path components
    pub fn library(components: impl IntoIterator<Item = impl Into<String>>) -> Self {
        FeatureRequirement::Library(components.into_iter().map(|s| s.into()).collect())
    }

    /// Creates an AND requirement
    pub fn and(requirements: impl IntoIterator<Item = FeatureRequirement>) -> Self {
        FeatureRequirement::And(requirements.into_iter().collect())
    }

    /// Creates an OR requirement
    pub fn or(requirements: impl IntoIterator<Item = FeatureRequirement>) -> Self {
        FeatureRequirement::Or(requirements.into_iter().collect())
    }

    /// Creates a NOT requirement
    #[allow(clippy::should_implement_trait)]
    pub fn not(requirement: FeatureRequirement) -> Self {
        FeatureRequirement::Not(Box::new(requirement))
    }
}

impl std::fmt::Display for FeatureRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureRequirement::Feature(name) => write!(f, "{}", name),
            FeatureRequirement::Library(components) => {
                write!(f, "({})", components.join(" "))
            }
            FeatureRequirement::And(reqs) => {
                write!(f, "(and")?;
                for req in reqs {
                    write!(f, " {}", req)?;
                }
                write!(f, ")")
            }
            FeatureRequirement::Or(reqs) => {
                write!(f, "(or")?;
                for req in reqs {
                    write!(f, " {}", req)?;
                }
                write!(f, ")")
            }
            FeatureRequirement::Not(req) => {
                write!(f, "(not {})", req)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_requirement_display() {
        // Simple feature
        let req1 = FeatureRequirement::feature("lambdust");
        assert_eq!(req1.to_string(), "lambdust");

        // Library
        let req2 = FeatureRequirement::library(["srfi", "1"]);
        assert_eq!(req2.to_string(), "(srfi 1)");

        // Complex expression
        let req3 = FeatureRequirement::and([
            FeatureRequirement::feature("lambdust"),
            FeatureRequirement::not(FeatureRequirement::feature("debug")),
        ]);
        assert_eq!(req3.to_string(), "(and lambdust (not debug))");
    }

    #[test]
    fn test_feature_requirement_creation() {
        // Test builder methods
        let req = FeatureRequirement::or([
            FeatureRequirement::and([
                FeatureRequirement::feature("simd"),
                FeatureRequirement::feature("x86-64"),
            ]),
            FeatureRequirement::feature("fallback"),
        ]);

        assert!(matches!(req, FeatureRequirement::Or(_)));
    }
}

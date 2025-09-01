//! SRFI-2 and-let* clause definitions for AST.

use crate::diagnostics::{Span, Spanned};
use serde::{Deserialize, Serialize};

use super::Expr;

/// A clause in an SRFI-2 and-let* expression.
/// 
/// SRFI-2 defines three types of clauses:
/// 1. (variable expression) - binding clause: bind variable to expression result if not #f
/// 2. (expression) - guard clause: continue only if expression is not #f  
/// 3. expression - bare expression: same as guard clause but without parentheses
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AndLetClause {
    /// Binding clause: (variable expression)
    /// Binds the variable to the result of the expression if it's not #f
    Binding {
        /// Variable name being bound
        variable: String,
        /// Expression to evaluate and bind to the variable
        expression: Spanned<Expr>,
    },
    
    /// Test clause: (expression) or expression
    /// Tests the expression; continues only if result is not #f
    Test {
        /// Expression to evaluate as a test condition
        expression: Spanned<Expr>,
    },
}

impl AndLetClause {
    /// Gets the expression being evaluated in this clause.
    pub fn expression(&self) -> &Spanned<Expr> {
        match self {
            AndLetClause::Binding { expression, .. } => expression,
            AndLetClause::Test { expression } => expression,
        }
    }
    
    /// Gets the variable name if this is a binding clause.
    pub fn variable(&self) -> Option<&str> {
        match self {
            AndLetClause::Binding { variable, .. } => Some(variable),
            AndLetClause::Test { .. } => None,
        }
    }
    
    /// Returns true if this is a binding clause.
    pub fn is_binding(&self) -> bool {
        matches!(self, AndLetClause::Binding { .. })
    }
    
    /// Returns true if this is a test clause.
    pub fn is_test(&self) -> bool {
        matches!(self, AndLetClause::Test { .. })
    }
    
    /// Gets the span of this clause.
    pub fn span(&self) -> Span {
        self.expression().span
    }
}
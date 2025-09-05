//! Conditional clause for cond expressions.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

use super::Expr;

/// A clause in a cond expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CondClause {
    /// Test condition expression
    pub test: Spanned<Expr>,
    /// Body expressions to evaluate if test is true
    pub body: Vec<Spanned<Expr>>,
}

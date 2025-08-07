//! Guard clause for guard expressions.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

use super::Expr;

/// A clause in a guard expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardClause {
    /// Test expression (condition for this clause)
    pub test: Spanned<Expr>,
    /// Body expressions to execute if test is true
    pub body: Vec<Spanned<Expr>>,
    /// Optional => clause for result transformation
    pub arrow: Option<Spanned<Expr>>,
}
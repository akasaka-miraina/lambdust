//! Conditional clause for cond expressions.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

use super::Expr;

/// A clause in a cond expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CondClause {
    pub test: Spanned<Expr>,
    pub body: Vec<Spanned<Expr>>,
}
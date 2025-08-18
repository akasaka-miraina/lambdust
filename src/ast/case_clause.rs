//! Case clause for case expressions.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

use super::Expr;

/// A clause in a case expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseClause {
    /// Values to match against (datum expressions)
    pub values: Vec<Spanned<Expr>>,
    /// Body expressions to evaluate if match succeeds
    pub body: Vec<Spanned<Expr>>,
}
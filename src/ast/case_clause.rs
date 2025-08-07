//! Case clause for case expressions.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

use super::Expr;

/// A clause in a case expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseClause {
    pub values: Vec<Spanned<Expr>>,
    pub body: Vec<Spanned<Expr>>,
}
//! Case-lambda clause structure.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

use super::{Expr, Formals};

/// A clause in a case-lambda expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseLambdaClause {
    /// Formal parameters for this clause
    pub formals: Formals,
    /// Body expressions for this clause
    pub body: Vec<Spanned<Expr>>,
}
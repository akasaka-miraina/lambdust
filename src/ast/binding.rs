//! Variable binding structures for AST.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

use super::Expr;

/// A variable binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    /// Variable name being bound
    pub name: String,
    /// Expression value bound to the variable
    pub value: Spanned<Expr>,
}

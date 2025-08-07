//! Variable binding structures for AST.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

use super::Expr;

/// A variable binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub name: String,
    pub value: Spanned<Expr>,
}
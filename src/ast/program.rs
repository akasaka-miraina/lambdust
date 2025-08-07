//! Program AST node for Lambdust.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

use super::Expr;

/// The top-level AST node representing a complete Lambdust program.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    /// The expressions that make up the program.
    pub expressions: Vec<Spanned<Expr>>,
}

impl Program {
    /// Creates a new empty program.
    pub fn new() -> Self {
        Self {
            expressions: Vec::new(),
        }
    }

    /// Creates a program with the given expressions.
    pub fn with_expressions(expressions: Vec<Spanned<Expr>>) -> Self {
        Self { expressions }
    }

    /// Adds an expression to the program.
    pub fn add_expression(&mut self, expr: Spanned<Expr>) {
        self.expressions.push(expr);
    }

    /// Returns true if the program is empty.
    pub fn is_empty(&self) -> bool {
        self.expressions.is_empty()
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
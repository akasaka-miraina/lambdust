//! AST to Value conversion utilities
//!
//! This module handles the conversion of AST expressions to Scheme values,
//! primarily used for quote and quasiquote operations.

use crate::ast::{Expr, Literal};
use crate::error::{LambdustError, Result};
use crate::value::Value;

/// AST to Value converter for quote operations
pub struct AstConverter;

impl AstConverter {
    /// Convert expression to value (for quote and quasiquote)
    /// This implements the quote semantics: E[E] -> Value
    pub fn expr_to_value(expr: Expr) -> Result<Value> {
        match expr {
            Expr::Literal(lit) => Self::literal_to_value(lit),
            Expr::Variable(name) => Ok(Value::Symbol(name)),
            Expr::List(exprs) => Self::list_to_value(exprs),
            Expr::Vector(exprs) => Self::vector_to_value(exprs),
            Expr::Quote(expr) => Self::expr_to_value(*expr),
            Expr::DottedList(elements, tail) => Self::dotted_list_to_value(elements, *tail),
            Expr::Quasiquote(_) | Expr::Unquote(_) | Expr::UnquoteSplicing(_) => {
                Err(LambdustError::syntax_error(
                    "Quasiquote forms not yet implemented in quote context".to_string(),
                ))
            }
        }
    }

    /// Convert literal to value
    fn literal_to_value(lit: Literal) -> Result<Value> {
        Ok(match lit {
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Number(n) => Value::Number(n),
            Literal::String(s) => Value::String(s),
            Literal::Character(c) => Value::Character(c),
            Literal::Nil => Value::Nil,
        })
    }

    /// Convert list expression to value
    fn list_to_value(exprs: Vec<Expr>) -> Result<Value> {
        let mut result = Value::Nil;
        for expr in exprs.into_iter().rev() {
            let value = Self::expr_to_value(expr)?;
            result = Value::cons(value, result);
        }
        Ok(result)
    }

    /// Convert vector expression to value
    fn vector_to_value(exprs: Vec<Expr>) -> Result<Value> {
        let values: Result<Vec<Value>> = exprs.into_iter().map(Self::expr_to_value).collect();
        Ok(Value::Vector(values?))
    }

    /// Convert dotted list to value: (a b . c) -> cons(a, cons(b, c))
    fn dotted_list_to_value(elements: Vec<Expr>, tail: Expr) -> Result<Value> {
        let mut result = Self::expr_to_value(tail)?;
        for expr in elements.into_iter().rev() {
            let value = Self::expr_to_value(expr)?;
            result = Value::cons(value, result);
        }
        Ok(result)
    }
}

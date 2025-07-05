//! SRFI (Scheme Request for Implementation) module system
//!
//! This module provides organized access to SRFI implementations following
//! the standard `(import (srfi <id> [<parts>]))` syntax.

pub mod registry;
pub mod srfi_1; // List Library
pub mod srfi_111; // Boxes
pub mod srfi_113; // Sets and Bags
pub mod srfi_125; // Intermediate Hash Tables
pub mod srfi_13; // String Libraries
pub mod srfi_132; // Sort Libraries
pub mod srfi_133; // Vector Libraries
pub mod srfi_45; // Lazy evaluation
pub mod srfi_46; // Syntax-rules extensions
pub mod srfi_69; // Basic Hash Tables
pub mod srfi_9; // Define-record-type
pub mod srfi_97; // SRFI Libraries

pub use registry::SrfiRegistry;

use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;

/// SRFI module interface
pub trait SrfiModule {
    /// SRFI number identifier
    fn srfi_id(&self) -> u32;

    /// SRFI name
    fn name(&self) -> &'static str;

    /// Available parts/components of this SRFI
    fn parts(&self) -> Vec<&'static str>;

    /// Get all exported functions for this SRFI
    fn exports(&self) -> HashMap<String, Value>;

    /// Get exported functions for specific parts
    fn exports_for_parts(&self, parts: &[&str]) -> Result<HashMap<String, Value>>;
}

/// SRFI import specification
#[derive(Debug, Clone)]
pub struct SrfiImport {
    /// SRFI number
    pub id: u32,
    /// Specific parts to import (empty means all)
    pub parts: Vec<String>,
}

impl SrfiImport {
    /// Create a new SRFI import for entire SRFI
    pub fn new(id: u32) -> Self {
        Self {
            id,
            parts: Vec::new(),
        }
    }

    /// Create a new SRFI import with specific parts
    pub fn with_parts(id: u32, parts: Vec<String>) -> Self {
        Self { id, parts }
    }

    /// Check if importing all parts
    pub fn imports_all(&self) -> bool {
        self.parts.is_empty()
    }
}

/// Parse SRFI import from S-expression
///
/// Expected formats:
/// - (srfi 1)
/// - (srfi 1 lists)
/// - (srfi 9 records)
pub fn parse_srfi_import(expr: &crate::ast::Expr) -> Result<SrfiImport> {
    use crate::ast::Expr;

    match expr {
        Expr::List(elements) if elements.len() >= 2 => {
            // First element should be "srfi"
            if let Expr::Variable(name) = &elements[0] {
                if name != "srfi" {
                    return Err(LambdustError::syntax_error(
                        "Expected 'srfi' as first element".to_string(),
                    ));
                }
            } else {
                return Err(LambdustError::syntax_error(
                    "Expected 'srfi' as first element".to_string(),
                ));
            }

            // Second element should be SRFI number
            let id = match &elements[1] {
                Expr::Literal(crate::ast::Literal::Number(num)) => match num {
                    crate::lexer::SchemeNumber::Integer(i) if *i >= 0 => *i as u32,
                    _ => {
                        return Err(LambdustError::syntax_error(
                            "SRFI ID must be a non-negative integer".to_string(),
                        ));
                    }
                },
                _ => {
                    return Err(LambdustError::syntax_error(
                        "SRFI ID must be a number".to_string(),
                    ));
                }
            };

            // Remaining elements are parts
            let parts = elements[2..]
                .iter()
                .map(|expr| match expr {
                    Expr::Variable(name) => Ok(name.clone()),
                    _ => Err(LambdustError::syntax_error(
                        "SRFI parts must be symbols".to_string(),
                    )),
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(SrfiImport::with_parts(id, parts))
        }
        _ => Err(LambdustError::syntax_error(
            "Invalid SRFI import syntax".to_string(),
        )),
    }
}

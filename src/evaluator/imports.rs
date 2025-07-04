//! Import functionality for SRFI modules
//!
//! This module implements the (import (srfi N)) syntax for dynamic module loading.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
// use crate::srfi::parse_srfi_import;
use crate::value::Value;
use std::rc::Rc;

impl Evaluator {
    /// Evaluate import special form
    pub fn eval_import(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return Err(LambdustError::syntax_error(
                "import: at least one import set required".to_string(),
            ));
        }

        // Process each import specification
        for import_spec in operands {
            self.process_import_spec(import_spec, env.clone())?;
        }

        // Import returns unspecified value
        self.apply_continuation(cont, Value::Undefined)
    }

    /// Process a single import specification
    fn process_import_spec(&mut self, import_spec: &Expr, env: Rc<Environment>) -> Result<()> {
        match import_spec {
            Expr::List(import_parts) => {
                if import_parts.is_empty() {
                    return Err(LambdustError::syntax_error(
                        "import: empty import specification".to_string(),
                    ));
                }

                match &import_parts[0] {
                    Expr::Variable(lib_type) if lib_type == "srfi" => {
                        self.process_srfi_import(&import_parts[1..], env)
                    }
                    _ => Err(LambdustError::syntax_error(
                        "import: only SRFI imports are currently supported".to_string(),
                    )),
                }
            }
            _ => Err(LambdustError::syntax_error(
                "import: import specification must be a list".to_string(),
            )),
        }
    }

    /// Process SRFI import
    fn process_srfi_import(&mut self, srfi_parts: &[Expr], env: Rc<Environment>) -> Result<()> {
        if srfi_parts.is_empty() {
            return Err(LambdustError::syntax_error(
                "import: SRFI number required".to_string(),
            ));
        }

        // Parse SRFI number
        let srfi_number = match &srfi_parts[0] {
            Expr::Literal(crate::ast::Literal::Number(
                crate::lexer::SchemeNumber::Integer(n),
            )) => *n as u32,
            _ => {
                return Err(LambdustError::syntax_error(
                    "import: SRFI number must be an integer".to_string(),
                ))
            }
        };

        // Parse optional parts specification
        let parts: Vec<&str> = if srfi_parts.len() > 1 {
            vec!["all"] // Simplified for now
        } else {
            vec!["all"]
        };

        // Get exports from SRFI registry
        let exports = {
            let registry = self.srfi_registry_mut();
            registry.get_exports_for_parts(srfi_number, &parts)?
        };

        // Import functions into environment
        for (name, value) in exports {
            env.define(name, value);
        }

        Ok(())
    }

    /// Parse import parts specification
    fn parse_import_parts(&self, _parts_exprs: &[Expr]) -> Result<Vec<&str>> {
        // For now, just support "all" 
        // In a full implementation, this would parse specific part names
        Ok(vec!["all"])
    }
}
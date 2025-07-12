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
            self.process_import_spec(import_spec, Rc::clone(&env))?;
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
            Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(n))) => {
                *n as u32
            }
            _ => {
                return Err(LambdustError::syntax_error(
                    "import: SRFI number must be an integer".to_string(),
                ));
            }
        };

        // Parse optional parts specification
        let parts = if srfi_parts.len() > 1 {
            self.parse_import_parts(&srfi_parts[1..])?
        } else {
            vec!["all".to_string()]
        };

        // Get exports from SRFI registry
        let exports = {
            let registry = self.srfi_registry_mut();
            let parts_refs: Vec<&str> = parts.iter().map(std::string::String::as_str).collect();
            registry.get_exports_for_parts(srfi_number, &parts_refs)?
        };

        // Import functions into environment
        for (name, value) in exports {
            env.define(name, value);
        }

        Ok(())
    }

    /// Parse import parts specification
    fn parse_import_parts(&self, parts_exprs: &[Expr]) -> Result<Vec<String>> {
        let mut parts = Vec::new();

        for part_expr in parts_exprs {
            match part_expr {
                Expr::Variable(name) => {
                    parts.push(name.clone());
                }
                _ => {
                    return Err(LambdustError::syntax_error(
                        "import: part names must be symbols".to_string(),
                    ));
                }
            }
        }

        if parts.is_empty() {
            return Err(LambdustError::syntax_error(
                "import: at least one part name required".to_string(),
            ));
        }

        Ok(parts)
    }
}

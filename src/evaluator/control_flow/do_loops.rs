//! Do loop implementation
//!
//! This module implements R7RS do loop special form with step expressions,
//! variable binding, test conditions, and iterative evaluation.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::value::Value;
use std::rc::Rc;

/// Evaluate do loop special form
/// Phase 6-A-Step3: Auto-delegate to trampoline evaluator for stack overflow prevention
pub fn eval_do(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    // Phase 6-A-Step3: Automatically use trampoline evaluator for do-loops
    // to prevent stack overflow in iterative constructs
    use crate::evaluator::TrampolineEvaluation;
    
    // Construct the do-loop expression for trampoline evaluation
    let mut do_expr_parts = vec![crate::ast::Expr::Variable("do".to_string())];
    do_expr_parts.extend_from_slice(operands);
    let do_expr = crate::ast::Expr::List(do_expr_parts);
    
    // Delegate to trampoline evaluator for stack-safe evaluation
    evaluator.eval_trampoline(do_expr, env, cont)
}

/// Parse do bindings
#[allow(dead_code)]
fn parse_do_bindings(bindings_expr: &Expr) -> Result<Vec<(String, Expr, Option<Expr>)>> {
    match bindings_expr {
        Expr::List(bindings) => {
            let mut parsed_bindings = Vec::new();
            for binding in bindings {
                match binding {
                    Expr::List(binding_parts) => {
                        if binding_parts.len() < 2 || binding_parts.len() > 3 {
                            return Err(LambdustError::syntax_error(
                                "do: binding must have 2 or 3 elements".to_string(),
                            ));
                        }

                        let var = match &binding_parts[0] {
                            Expr::Variable(name) => name.clone(),
                            _ => {
                                return Err(LambdustError::syntax_error(
                                    "do: binding variable must be a symbol".to_string(),
                                ));
                            }
                        };

                        let init = binding_parts[1].clone();
                        let step = binding_parts.get(2).cloned();

                        parsed_bindings.push((var, init, step));
                    }
                    _ => {
                        return Err(LambdustError::syntax_error(
                            "do: binding must be a list".to_string(),
                        ));
                    }
                }
            }
            Ok(parsed_bindings)
        }
        _ => Err(LambdustError::syntax_error(
            "do: bindings must be a list".to_string(),
        )),
    }
}

/// Parse do test clause
#[allow(dead_code)]
fn parse_do_test(test_expr: &Expr) -> Result<(Expr, Vec<Expr>)> {
    match test_expr {
        Expr::List(test_parts) => {
            if test_parts.is_empty() {
                return Err(LambdustError::syntax_error(
                    "do: test clause cannot be empty".to_string(),
                ));
            }

            let test = test_parts[0].clone();
            let result_exprs = test_parts[1..].to_vec();

            Ok((test, result_exprs))
        }
        _ => Err(LambdustError::syntax_error(
            "do: test clause must be a list".to_string(),
        )),
    }
}

// Additional functions for Evaluator impl
impl Evaluator {
    /// Apply do continuation
    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply_do_continuation(
        &mut self,
        test_value: Value,
        bindings: Vec<(String, Expr, Option<Expr>)>,
        test: Expr,
        result_exprs: Vec<Expr>,
        body_exprs: Vec<Expr>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // test_value is the result of evaluating the test expression

        // Check if test is true (non-#f)
        let test_is_true = match test_value {
            Value::Boolean(false) => false,
            _ => true, // Everything except #f is true in Scheme
        };

        if test_is_true {
            // Test succeeded, evaluate result expressions and exit loop
            if result_exprs.is_empty() {
                // No result expressions, return undefined
                self.apply_continuation(parent, Value::Undefined)
            } else if result_exprs.len() == 1 {
                // Single result expression
                self.eval(result_exprs[0].clone(), env, parent)
            } else {
                // Multiple result expressions, evaluate as sequence
                self.eval_sequence(result_exprs, env, parent)
            }
        } else {
            // Test failed, continue loop
            // 1. Execute body expressions (side effects)
            if !body_exprs.is_empty() {
                for body_expr in &body_exprs {
                    self.eval(body_expr.clone(), env.clone(), Continuation::Identity)?;
                }
            }

            // 2. Update variables with step expressions (all at once with old values)
            let mut step_values = Vec::new();
            for (var, _init, step_opt) in &bindings {
                if let Some(step_expr) = step_opt {
                    let step_value =
                        self.eval(step_expr.clone(), env.clone(), Continuation::Identity)?;
                    step_values.push((var.clone(), step_value));
                } else {
                    // If no step expression, keep current value
                    let current_value = env
                        .get(var)
                        .ok_or_else(|| LambdustError::undefined_variable(var.clone()))?;
                    step_values.push((var.clone(), current_value));
                }
            }

            // Now update all variables at once
            for (var, new_value) in step_values {
                env.set(&var, new_value)?;
            }

            // 3. Re-evaluate test and continue loop
            let next_do_cont = Continuation::Do {
                bindings,
                test: test.clone(),
                result_exprs,
                body_exprs,
                env: env.clone(),
                parent: Box::new(parent),
            };

            self.eval(test, env, next_do_cont)
        }
    }
}

impl Continuation {
    /// Helper method to extract test from Do continuation
    /// Returns None if not a Do continuation (type-safe alternative to panic)
    pub fn test(&self) -> Option<&Expr> {
        match self {
            Continuation::Do { test, .. } => Some(test),
            _ => None,
        }
    }

    /// Helper method to extract test from Do continuation (unsafe but fast)
    /// Only use when you are certain the continuation is a Do continuation
    pub fn test_unchecked(&self) -> &Expr {
        match self {
            Continuation::Do { test, .. } => test,
            _ => unreachable!("test_unchecked() called on non-Do continuation"),
        }
    }
}

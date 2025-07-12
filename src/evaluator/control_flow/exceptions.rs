//! Exception handling implementation
//!
//! This module implements R7RS exception handling including raise,
//! with-exception-handler, and guard special forms.

use crate::ast::Expr;
use crate::bridge::ExternalObject;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator, ExceptionHandlerInfo};
use crate::value::Value;
use std::rc::Rc;
use std::sync::Arc;

/// Dynamic guard handler that properly evaluates guard clauses
/// This structure holds the information needed for dynamic guard evaluation
#[derive(Debug, Clone)]
pub struct GuardHandler {
    /// The condition variable name
    pub condition_var: String,
    /// List of guard clauses: (`condition_expr`, `result_exprs`)
    pub clauses: Vec<(Expr, Vec<Expr>)>,
    /// Optional else clause expressions
    pub else_exprs: Option<Vec<Expr>>,
    /// Environment for evaluation (Note: Rc<Environment> doesn't implement Send+Sync)
    /// We'll need to store environment data differently for thread safety
    /// Currently unused but kept for future extensibility
    #[allow(dead_code)]
    pub env: Rc<Environment>,
}

// Safety: GuardHandler is only used within single-threaded evaluator context
unsafe impl Send for GuardHandler {}
unsafe impl Sync for GuardHandler {}

/// Evaluate raise special form
pub fn eval_raise(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    if operands.len() != 1 {
        return Err(LambdustError::arity_error(1, operands.len()));
    }

    let exception_expr = &operands[0];
    let exception_value = evaluator.eval(exception_expr.clone(), env, Continuation::Identity)?;

    evaluator.raise_exception(exception_value, cont)
}

/// Evaluate with-exception-handler special form
pub fn eval_with_exception_handler(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    if operands.len() != 2 {
        return Err(LambdustError::arity_error(2, operands.len()));
    }

    let handler_expr = &operands[0];
    let thunk_expr = &operands[1];

    // Evaluate handler first
    let handler_value =
        evaluator.eval(handler_expr.clone(), env.clone(), Continuation::Identity)?;

    // Install exception handler
    let handler_info = ExceptionHandlerInfo {
        handler: handler_value,
        env: env.clone(),
    };
    evaluator.exception_handlers_mut().push(handler_info);

    // Evaluate thunk expression to get the procedure
    let thunk_value = evaluator.eval(thunk_expr.clone(), env.clone(), Continuation::Identity)?;

    // Apply the thunk (call it with no arguments)
    let result = evaluator.apply_procedure(thunk_value, vec![], env, cont);

    // Remove handler
    evaluator.exception_handlers_mut().pop();

    result
}

/// Evaluate guard special form
pub fn eval_guard(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    if operands.len() < 2 {
        return Err(LambdustError::syntax_error(
            "guard: requires at least condition and body".to_string(),
        ));
    }

    // Parse guard structure: (guard (condition-var clause ...) body ...)
    let (condition_var, clauses, else_exprs) = parse_guard_condition(&operands[0])?;
    let body_exprs = operands[1..].to_vec();

    // Create guard exception handler
    let guard_handler = create_guard_handler(condition_var, clauses, else_exprs, env.clone())?;

    // Install handler
    let handler_info = ExceptionHandlerInfo {
        handler: guard_handler,
        env: env.clone(),
    };
    evaluator.exception_handlers_mut().push(handler_info);

    // Evaluate body
    let result = evaluator.eval_sequence(body_exprs, env, cont);

    // Remove handler
    evaluator.exception_handlers_mut().pop();

    result
}

/// Parse guard condition clause
#[allow(clippy::type_complexity)]
fn parse_guard_condition(
    condition_expr: &Expr,
) -> Result<(String, Vec<(Expr, Vec<Expr>)>, Option<Vec<Expr>>)> {
    match condition_expr {
        Expr::List(parts) => {
            if parts.is_empty() {
                return Err(LambdustError::syntax_error(
                    "guard: condition clause cannot be empty".to_string(),
                ));
            }

            let condition_var = match &parts[0] {
                Expr::Variable(name) => name.clone(),
                _ => {
                    return Err(LambdustError::syntax_error(
                        "guard: condition variable must be a symbol".to_string(),
                    ));
                }
            };

            let mut clauses = Vec::new();
            let mut else_exprs = None;

            for clause in &parts[1..] {
                match clause {
                    Expr::List(clause_parts) => {
                        if clause_parts.is_empty() {
                            return Err(LambdustError::syntax_error(
                                "guard: clause cannot be empty".to_string(),
                            ));
                        }

                        if let Expr::Variable(name) = &clause_parts[0] {
                            if name == "else" {
                                if else_exprs.is_some() {
                                    return Err(LambdustError::syntax_error(
                                        "guard: multiple else clauses".to_string(),
                                    ));
                                }
                                else_exprs = Some(clause_parts[1..].to_vec());
                                continue;
                            }
                        }

                        let condition = clause_parts[0].clone();
                        let result = clause_parts[1..].to_vec();
                        clauses.push((condition, result));
                    }
                    _ => {
                        return Err(LambdustError::syntax_error(
                            "guard: clause must be a list".to_string(),
                        ));
                    }
                }
            }

            Ok((condition_var, clauses, else_exprs))
        }
        _ => Err(LambdustError::syntax_error(
            "guard: condition must be a list".to_string(),
        )),
    }
}

/// Create dynamic guard exception handler
fn create_guard_handler(
    condition_var: String,
    clauses: Vec<(Expr, Vec<Expr>)>,
    else_exprs: Option<Vec<Expr>>,
    env: Rc<Environment>,
) -> Result<Value> {
    // Create a GuardHandler structure that will be used for dynamic evaluation
    let guard_handler = GuardHandler {
        condition_var: condition_var.clone(),
        clauses,
        else_exprs,
        env,
    };

    // Create an ExternalObject for the guard handler
    let external_obj = ExternalObject {
        id: 0, // Will be set by registry if needed
        type_name: "GuardHandler".to_string(),
        data: Arc::new(guard_handler),
    };

    // Return the guard handler as an External value for later evaluation
    Ok(Value::External(external_obj))
}

// Additional functions for Evaluator impl
impl Evaluator {
    /// Raise an exception
    pub(super) fn raise_exception(
        &mut self,
        exception: Value,
        cont: Continuation,
    ) -> Result<Value> {
        // Find and call the nearest exception handler
        if let Some(handler_info) = self.exception_handlers().last() {
            let handler = handler_info.handler.clone();
            let handler_env = handler_info.env.clone();

            match handler {
                Value::External(ref external_obj) => {
                    // Check if this is a GuardHandler
                    if external_obj.type_name == "GuardHandler" {
                        if let Some(guard_handler) =
                            external_obj.data.downcast_ref::<GuardHandler>()
                        {
                            // Process guard handler dynamically
                            self.process_guard_handler(guard_handler, exception, handler_env, cont)
                        } else {
                            // Failed to downcast, treat as error
                            Err(LambdustError::runtime_error(
                                "Failed to downcast GuardHandler".to_string(),
                            ))
                        }
                    } else {
                        // Unknown external object, treat as error
                        Err(LambdustError::runtime_error(
                            "Unknown external exception handler".to_string(),
                        ))
                    }
                }
                Value::Symbol(ref _s) => {
                    // This is a legacy guard handler result, return it directly
                    self.apply_continuation(cont, handler)
                }
                Value::Procedure(_) => {
                    // This is a real procedure, call it with the exception
                    self.apply_procedure(handler, vec![exception], handler_env, cont)
                }
                _ => {
                    // Unexpected handler type, return it directly
                    self.apply_continuation(cont, handler)
                }
            }
        } else {
            // No handler found, convert to LambdustError
            let formatted_exception = match &exception {
                Value::String(s) => format!("\"{s}\""),
                Value::Symbol(s) => s.clone(),
                other => format!("{other:?}"),
            };
            Err(LambdustError::runtime_error(format!(
                "Uncaught exception: {formatted_exception}"
            )))
        }
    }

    /// Process guard handler dynamically by evaluating guard clauses
    fn process_guard_handler(
        &mut self,
        guard_handler: &GuardHandler,
        exception: Value,
        handler_env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Create a new environment that binds the condition variable to the exception
        let guard_env = Environment::with_parent(handler_env.clone());
        guard_env.define(guard_handler.condition_var.clone(), exception.clone());

        // Evaluate each guard clause in order
        for (condition_expr, result_exprs) in &guard_handler.clauses {
            if let Some(result) = self.try_evaluate_guard_clause(
                condition_expr,
                result_exprs,
                &guard_env,
                cont.clone(),
            )? {
                return Ok(result);
            }
        }

        // No condition matched, check for else clause
        if let Some(else_exprs) = &guard_handler.else_exprs {
            return self.eval_sequence(else_exprs.clone(), Rc::new(guard_env), cont);
        }

        // No matching clause and no else clause - re-raise the exception
        // by removing this handler and trying the next one
        self.exception_handlers_mut().pop();
        let result = self.raise_exception(exception, cont);

        // Restore the handler stack
        let external_obj = ExternalObject {
            id: 0,
            type_name: "GuardHandler".to_string(),
            data: Arc::new(guard_handler.clone()),
        };
        let handler_info = ExceptionHandlerInfo {
            handler: Value::External(external_obj),
            env: handler_env,
        };
        self.exception_handlers_mut().push(handler_info);

        result
    }

    /// Try to evaluate a single guard clause, returning Some(result) if the condition matches
    fn try_evaluate_guard_clause(
        &mut self,
        condition_expr: &Expr,
        result_exprs: &[Expr],
        guard_env: &Environment,
        cont: Continuation,
    ) -> Result<Option<Value>> {
        // Evaluate the condition expression
        let Ok(condition_result) = self.eval(
            condition_expr.clone(),
            Rc::new(guard_env.clone()),
            Continuation::Identity,
        ) else { return Ok(None) }; // Condition evaluation failed, continue to next clause

        // Check if condition is true (any non-#f value is true in Scheme)
        if matches!(condition_result, Value::Boolean(false)) {
            return Ok(None);
        }

        // Condition matched, evaluate and return the result expressions
        let result = self.eval_sequence(result_exprs.to_vec(), Rc::new(guard_env.clone()), cont)?;
        Ok(Some(result))
    }

    /// Apply exception handler continuation
    pub(super) fn apply_exception_handler_continuation(
        &mut self,
        value: Value,
        handler: Value,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Install the exception handler and then apply the parent continuation
        let handler_info = ExceptionHandlerInfo {
            handler,
            env: env.clone(),
        };
        self.exception_handlers_mut().push(handler_info);

        // Apply the parent continuation with the value
        let result = self.apply_continuation(parent, value);

        // Remove the handler after processing
        self.exception_handlers_mut().pop();

        result
    }

    /// Apply guard clause continuation
    pub(super) fn apply_guard_clause_continuation(
        &mut self,
        value: Value,
        condition_var: String,
        clauses: Vec<(Expr, Vec<Expr>)>,
        else_exprs: Option<Vec<Expr>>,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Create a guard handler for the clauses
        let guard_handler = GuardHandler {
            condition_var,
            clauses,
            else_exprs,
            env: env.clone(),
        };

        // Process the guard handler with the value as the exception
        self.process_guard_handler(&guard_handler, value, env, parent)
    }
}

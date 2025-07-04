//! Control flow constructs for R7RS semantics
//!
//! This module implements control flow special forms including call/cc,
//! exception handling, dynamic-wind, and do loops.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator, ExceptionHandlerInfo};
use crate::value::{Procedure, Value};
use std::rc::Rc;

impl Evaluator {
    /// Evaluate do loop special form
    pub fn eval_do(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "do: requires at least bindings and test".to_string(),
            ));
        }

        // Parse bindings: ((var init step) ...)
        let bindings = self.parse_do_bindings(&operands[0])?;

        // Parse test clause: (test result ...)
        let (test, result_exprs) = self.parse_do_test(&operands[1])?;

        // Body expressions
        let body_exprs = operands[2..].to_vec();

        // Create new environment for the do loop
        let do_env = Rc::new(Environment::with_parent(env));

        // Initialize variables with init expressions
        for (var, init_expr, _) in &bindings {
            let init_value = self.eval_in_current_env(init_expr.clone(), do_env.clone())?;
            do_env.define(var.clone(), init_value);
        }

        // Create do continuation
        let do_cont = Continuation::Do {
            bindings,
            test,
            result_exprs,
            body_exprs,
            env: do_env.clone(),
            parent: Box::new(cont),
        };

        // Start the loop by evaluating the test
        self.eval(do_cont.test().clone(), do_env, do_cont)
    }

    /// Parse do bindings
    fn parse_do_bindings(&self, bindings_expr: &Expr) -> Result<Vec<(String, Expr, Option<Expr>)>> {
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
                                    ))
                                }
                            };

                            let init = binding_parts[1].clone();
                            let step = binding_parts.get(2).cloned();

                            parsed_bindings.push((var, init, step));
                        }
                        _ => {
                            return Err(LambdustError::syntax_error(
                                "do: binding must be a list".to_string(),
                            ))
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
    fn parse_do_test(&self, test_expr: &Expr) -> Result<(Expr, Vec<Expr>)> {
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

    /// Evaluate call/cc special form
    pub fn eval_call_cc(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let proc_expr = operands[0].clone();

        // Create a captured continuation (simplified implementation)
        let captured_cont = Value::Procedure(Procedure::Continuation {
            continuation: Box::new(crate::value::Continuation {
                stack: Vec::new(),
                env: env.clone(),
            }),
        });

        let call_cc_cont = Continuation::CallCc {
            captured_cont,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(proc_expr, env, call_cc_cont)
    }

    /// Evaluate values special form
    pub fn eval_values(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.is_empty() {
            return self.apply_continuation(cont, Value::Values(Vec::new()));
        }

        if operands.len() == 1 {
            return self.eval(operands[0].clone(), env, cont);
        }

        // Evaluate multiple values
        let first = operands[0].clone();
        let remaining = operands[1..].to_vec();

        let values_cont = Continuation::Values {
            values: Vec::new(),
            parent: Box::new(cont),
        };

        // Start by evaluating remaining expressions
        self.eval_multiple_values(remaining, env.clone(), values_cont, |evaluator, values, cont| {
            // Now evaluate the first expression
            evaluator.eval(first, env, Continuation::Values {
                values,
                parent: Box::new(cont),
            })
        })
    }

    /// Evaluate call-with-values special form
    pub fn eval_call_with_values(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let consumer_expr = operands[0].clone();
        let producer_expr = operands[1].clone();

        let cwv_cont = Continuation::CallWithValuesStep1 {
            producer_expr,
            env: env.clone(),
            parent: Box::new(cont),
        };

        self.eval(consumer_expr, env, cwv_cont)
    }

    /// Evaluate dynamic-wind special form
    pub fn eval_dynamic_wind(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 3 {
            return Err(LambdustError::arity_error(3, operands.len()));
        }

        // For now, simplified implementation - just evaluate the thunk
        // Full dynamic-wind requires dynamic point management
        let _before = &operands[0];
        let thunk = &operands[1];
        let _after = &operands[2];

        // TODO: Implement full dynamic-wind semantics with dynamic points
        self.eval(thunk.clone(), env, cont)
    }

    /// Evaluate delay special form
    pub fn eval_delay(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let expr = operands[0].clone();
        let promise = Value::Promise(crate::value::Promise {
            state: crate::value::PromiseState::Lazy { expr, env },
        });

        self.apply_continuation(cont, promise)
    }

    /// Evaluate lazy special form (SRFI 45)
    pub fn eval_lazy(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let expr = operands[0].clone();
        let promise = Value::Promise(crate::value::Promise {
            state: crate::value::PromiseState::Lazy { expr, env },
        });

        self.apply_continuation(cont, promise)
    }

    /// Evaluate force special form
    pub fn eval_force(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let promise_expr = operands[0].clone();

        // First evaluate the promise expression
        let force_cont = Continuation::Identity; // Will be replaced with proper force continuation
        let promise_value = self.eval(promise_expr, env, force_cont)?;

        // Force the promise
        self.force_promise(promise_value, cont)
    }

    /// Evaluate promise? predicate
    pub fn eval_promise_predicate(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let expr = operands[0].clone();
        let value = self.eval(expr, env, Continuation::Identity)?;

        let is_promise = matches!(value, Value::Promise(_));
        self.apply_continuation(cont, Value::Boolean(is_promise))
    }

    /// Force a promise value
    fn force_promise(&mut self, promise: Value, cont: Continuation) -> Result<Value> {
        match promise {
            Value::Promise(promise_ref) => {
                match &promise_ref.state {
                    crate::value::PromiseState::Lazy { expr, env } => {
                        self.eval(expr.clone(), env.clone(), cont)
                    }
                    crate::value::PromiseState::Eager { value } => {
                        self.apply_continuation(cont, value.as_ref().clone())
                    }
                }
            }
            other => self.apply_continuation(cont, other),
        }
    }

    /// Evaluate raise special form
    pub fn eval_raise(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 1 {
            return Err(LambdustError::arity_error(1, operands.len()));
        }

        let exception_expr = operands[0].clone();
        let exception_value = self.eval(exception_expr, env, Continuation::Identity)?;

        self.raise_exception(exception_value, cont)
    }

    /// Evaluate with-exception-handler special form
    pub fn eval_with_exception_handler(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if operands.len() != 2 {
            return Err(LambdustError::arity_error(2, operands.len()));
        }

        let handler_expr = operands[0].clone();
        let thunk_expr = operands[1].clone();

        // Evaluate handler first
        let handler_value = self.eval(handler_expr, env.clone(), Continuation::Identity)?;

        // Install exception handler
        let handler_info = ExceptionHandlerInfo {
            handler: handler_value,
            env: env.clone(),
        };
        self.exception_handlers_mut().push(handler_info);

        // Evaluate thunk
        let result = self.eval(thunk_expr, env, cont);

        // Remove handler
        self.exception_handlers_mut().pop();

        result
    }

    /// Evaluate guard special form
    pub fn eval_guard(
        &mut self,
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
        let (condition_var, clauses, else_exprs) = self.parse_guard_condition(&operands[0])?;
        let body_exprs = operands[1..].to_vec();

        // Create guard exception handler
        let guard_handler = self.create_guard_handler(condition_var, clauses, else_exprs, env.clone())?;

        // Install handler
        let handler_info = ExceptionHandlerInfo {
            handler: guard_handler,
            env: env.clone(),
        };
        self.exception_handlers_mut().push(handler_info);

        // Evaluate body
        let result = self.eval_sequence(body_exprs, env, cont);

        // Remove handler
        self.exception_handlers_mut().pop();

        result
    }

    /// Parse guard condition clause
    fn parse_guard_condition(
        &self,
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
                        ))
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
                            ))
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

    /// Create guard exception handler
    fn create_guard_handler(
        &self,
        _condition_var: String,
        _clauses: Vec<(Expr, Vec<Expr>)>,
        _else_exprs: Option<Vec<Expr>>,
        _env: Rc<Environment>,
    ) -> Result<Value> {
        // TODO: Implement proper guard handler
        // For now, return a placeholder
        Err(LambdustError::runtime_error(
            "guard: full implementation pending".to_string(),
        ))
    }

    /// Raise an exception
    fn raise_exception(&mut self, exception: Value, _cont: Continuation) -> Result<Value> {
        // Find and call the nearest exception handler
        if let Some(handler_info) = self.exception_handlers().last() {
            let handler = handler_info.handler.clone();
            let handler_env = handler_info.env.clone();

            // Call handler with exception
            self.apply_procedure(
                handler,
                vec![exception],
                handler_env,
                Continuation::Identity,
            )
        } else {
            // No handler found, convert to LambdustError
            Err(LambdustError::runtime_error(format!(
                "Unhandled exception: {:?}",
                exception
            )))
        }
    }

    /// Apply control flow continuations
    pub fn apply_control_flow_continuation(
        &mut self,
        cont: Continuation,
        value: Value,
    ) -> Result<Value> {
        match cont {
            Continuation::Do {
                bindings,
                test,
                result_exprs,
                body_exprs,
                env,
                parent,
            } => self.apply_do_continuation(
                value,
                bindings,
                test,
                result_exprs,
                body_exprs,
                env,
                *parent,
            ),
            Continuation::CallWithValuesStep1 {
                producer_expr,
                env,
                parent,
            } => self.apply_call_with_values_step1_continuation(value, producer_expr, env, *parent),
            Continuation::CallWithValuesStep2 {
                consumer,
                env,
                parent,
            } => self.apply_call_with_values_step2_continuation(value, consumer, env, *parent),
            Continuation::Captured { cont } => self.apply_captured_continuation(value, *cont),
            Continuation::CallCc {
                captured_cont,
                env,
                parent,
            } => self.apply_call_cc_continuation(value, captured_cont, env, *parent),
            Continuation::ExceptionHandler {
                handler,
                env,
                parent,
            } => self.apply_exception_handler_continuation(value, handler, env, *parent),
            Continuation::GuardClause {
                condition_var,
                clauses,
                else_exprs,
                env,
                parent,
            } => self.apply_guard_clause_continuation(
                value,
                condition_var,
                clauses,
                else_exprs,
                env,
                *parent,
            ),
            _ => Err(LambdustError::runtime_error(
                "Unhandled continuation type in control flow".to_string(),
            )),
        }
    }

    /// Helper method to evaluate in current environment
    fn eval_in_current_env(&mut self, expr: Expr, env: Rc<Environment>) -> Result<Value> {
        self.eval(expr, env, Continuation::Identity)
    }

    /// Helper method for evaluating multiple values
    fn eval_multiple_values<F>(
        &mut self,
        exprs: Vec<Expr>,
        env: Rc<Environment>,
        cont: Continuation,
        f: F,
    ) -> Result<Value>
    where
        F: FnOnce(&mut Self, Vec<Value>, Continuation) -> Result<Value>,
    {
        // Simplified implementation - evaluate all expressions
        let mut values = Vec::new();
        for expr in exprs {
            let value = self.eval(expr, env.clone(), Continuation::Identity)?;
            values.push(value);
        }
        f(self, values, cont)
    }

    // Placeholder implementations for continuation applications
    fn apply_do_continuation(
        &mut self,
        _value: Value,
        _bindings: Vec<(String, Expr, Option<Expr>)>,
        _test: Expr,
        _result_exprs: Vec<Expr>,
        _body_exprs: Vec<Expr>,
        _env: Rc<Environment>,
        _parent: Continuation,
    ) -> Result<Value> {
        Err(LambdustError::runtime_error(
            "do continuation not yet implemented".to_string(),
        ))
    }

    fn apply_call_with_values_step1_continuation(
        &mut self,
        _value: Value,
        _producer_expr: Expr,
        _env: Rc<Environment>,
        _parent: Continuation,
    ) -> Result<Value> {
        Err(LambdustError::runtime_error(
            "call-with-values step1 continuation not yet implemented".to_string(),
        ))
    }

    fn apply_call_with_values_step2_continuation(
        &mut self,
        _value: Value,
        _consumer: Value,
        _env: Rc<Environment>,
        _parent: Continuation,
    ) -> Result<Value> {
        Err(LambdustError::runtime_error(
            "call-with-values step2 continuation not yet implemented".to_string(),
        ))
    }

    fn apply_captured_continuation(&mut self, value: Value, cont: Continuation) -> Result<Value> {
        self.apply_continuation(cont, value)
    }

    fn apply_call_cc_continuation(
        &mut self,
        procedure: Value,
        captured_cont: Value,
        env: Rc<Environment>,
        parent: Continuation,
    ) -> Result<Value> {
        // Apply the procedure with the captured continuation as argument
        self.apply_procedure(procedure, vec![captured_cont], env, parent)
    }

    fn apply_exception_handler_continuation(
        &mut self,
        _value: Value,
        _handler: Value,
        _env: Rc<Environment>,
        _parent: Continuation,
    ) -> Result<Value> {
        Err(LambdustError::runtime_error(
            "exception handler continuation not yet implemented".to_string(),
        ))
    }

    fn apply_guard_clause_continuation(
        &mut self,
        _value: Value,
        _condition_var: String,
        _clauses: Vec<(Expr, Vec<Expr>)>,
        _else_exprs: Option<Vec<Expr>>,
        _env: Rc<Environment>,
        _parent: Continuation,
    ) -> Result<Value> {
        Err(LambdustError::runtime_error(
            "guard clause continuation not yet implemented".to_string(),
        ))
    }
}

impl Continuation {
    /// Helper method to extract test from Do continuation
    pub fn test(&self) -> &Expr {
        match self {
            Continuation::Do { test, .. } => test,
            _ => panic!("test() called on non-Do continuation"),
        }
    }
}
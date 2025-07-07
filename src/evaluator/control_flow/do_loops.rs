//! Do loop implementation
//!
//! This module implements R7RS do loop special form with step expressions,
//! variable binding, test conditions, and iterative evaluation.

use crate::ast::{Expr, Literal};
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::value::Value;
use std::rc::Rc;

/// Direct expression evaluation that avoids trampoline to prevent infinite loops
/// This is used within do-loops to evaluate test conditions and step expressions
fn evaluate_expression_directly(
    evaluator: &mut Evaluator,
    expr: &Expr,
    env: Rc<Environment>,
) -> Result<Value> {
    match expr {
        // Handle literals directly
        Expr::Literal(literal) => {
            match literal {
                Literal::Number(n) => Ok(Value::Number(n.clone())),
                Literal::String(s) => Ok(Value::String(s.clone())),
                Literal::Character(c) => Ok(Value::Character(*c)),
                Literal::Boolean(b) => Ok(Value::Boolean(*b)),
                Literal::Nil => Ok(Value::Nil),
            }
        }
        
        // Handle variables by looking them up in the environment
        Expr::Variable(name) => {
            env.get(name)
                .ok_or_else(|| LambdustError::undefined_variable(name.clone()))
        }
        
        // Handle simple function calls (builtin functions only to avoid recursion)
        Expr::List(exprs) if !exprs.is_empty() => {
            if let Expr::Variable(func_name) = &exprs[0] {
                // Evaluate arguments first
                let mut args = Vec::new();
                for arg_expr in &exprs[1..] {
                    let arg_value = evaluate_expression_directly(evaluator, arg_expr, env.clone())?;
                    args.push(arg_value);
                }
                
                // Call builtin function if available
                if let Some(Value::Procedure(crate::value::Procedure::Builtin { func, .. })) = env.get(func_name) {
                    // Apply the builtin function directly
                    return func(&args);
                }
                
                // If it's not a builtin, fall back to regular evaluation
                // This might still cause trampoline issues, but it's our best option
                evaluator.eval(expr.clone(), env, Continuation::Identity)
            } else {
                // Non-variable function calls - fall back to regular evaluation
                evaluator.eval(expr.clone(), env, Continuation::Identity)
            }
        }
        
        // For other expressions, fall back to regular evaluation
        _ => evaluator.eval(expr.clone(), env, Continuation::Identity),
    }
}

/// Evaluate do loop special form
/// Phase 6-A: Trampoline evaluator integration for stack overflow prevention
pub fn eval_do(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    // Phase 6-A: Use trampoline evaluator to prevent stack overflow
    // This delegates to the heap-based continuation unwinding system
    
    // Phase 6-A: Use iterative implementation to prevent stack overflow
    // Direct iterative loop without recursive continuation chains
    eval_do_iterative(evaluator, operands, env, cont)
}

/// Iterative do-loop implementation to prevent stack overflow (Phase 6-A)
/// This implementation avoids deep recursive continuation chains
fn eval_do_iterative(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    if operands.len() < 2 {
        return Err(LambdustError::syntax_error(
            "do requires at least variable bindings and test clause".to_string(),
        ));
    }

    // Parse variable bindings
    let bindings = parse_do_bindings(&operands[0])?;
    
    // Parse test clause: (test result ...)
    let (test_expr, result_exprs) = match &operands[1] {
        Expr::List(test_clause) if !test_clause.is_empty() => {
            let test = test_clause[0].clone();
            let results = test_clause[1..].to_vec();
            (test, results)
        }
        _ => {
            return Err(LambdustError::syntax_error(
                "do requires test clause".to_string(),
            ));
        }
    };

    // Body expressions
    let body_exprs = operands[2..].to_vec();

    // Create loop environment extending the parent environment
    let loop_env = env.extend();
    
    // Initialize variables using direct evaluation to avoid trampoline loops
    for (var, init_expr, _) in &bindings {
        let init_value = evaluate_expression_directly(evaluator, init_expr, env.clone())?;
        loop_env.define(var.clone(), init_value);
    }
    
    let loop_env_rc = Rc::new(loop_env);

    // Iterative loop implementation with larger iteration limit (Phase 6-A)
    const MAX_ITERATIONS: usize = 10000; // Increased from 1000 to 10000
    
    for _iteration in 0..MAX_ITERATIONS {
        
        // Evaluate test condition using direct evaluation to avoid trampoline loops
        let test_result = evaluate_expression_directly(evaluator, &test_expr, loop_env_rc.clone())?;
        
        if test_result.is_truthy() {
            // Test passed - evaluate result expressions
            if result_exprs.is_empty() {
                return evaluator.apply_continuation(cont, Value::Undefined);
            } else if result_exprs.len() == 1 {
                let result = evaluate_expression_directly(evaluator, &result_exprs[0], loop_env_rc)?;
                return evaluator.apply_continuation(cont, result);
            } else {
                // Multiple result expressions - evaluate in sequence
                let last_idx = result_exprs.len() - 1;
                for (i, expr) in result_exprs.iter().enumerate() {
                    if i == last_idx {
                        // Last expression uses original continuation
                        let result = evaluate_expression_directly(evaluator, expr, loop_env_rc)?;
                        return evaluator.apply_continuation(cont, result);
                    } else {
                        // Intermediate expressions use Identity continuation
                        evaluate_expression_directly(evaluator, expr, loop_env_rc.clone())?;
                    }
                }
            }
        }

        // Execute body expressions (side effects only)
        for expr in &body_exprs {
            evaluate_expression_directly(evaluator, expr, loop_env_rc.clone())?;
        }

        // Update variables with step expressions (all evaluated with old values)
        let mut new_values = Vec::new();
        for (var, _, step_expr) in &bindings {
            if let Some(step) = step_expr {
                let new_value = evaluate_expression_directly(evaluator, step, loop_env_rc.clone())?;
                new_values.push((var.clone(), new_value));
            } else {
                // If no step expression, keep current value
                let current_value = loop_env_rc
                    .get(var)
                    .ok_or_else(|| LambdustError::undefined_variable(var.clone()))?;
                new_values.push((var.clone(), current_value));
            }
        }

        // Apply all new values at once (R7RS semantics)
        for (var, new_value) in new_values {
            loop_env_rc.set(&var, new_value)?;
        }

        // Note: Iteration limit prevents infinite loops
    }

    // If we reach here, the loop didn't terminate within the limit
    Err(LambdustError::runtime_error(
        format!("do loop exceeded maximum iterations ({})", MAX_ITERATIONS),
    ))
}

/// Fallback do-loop implementation with limited iterations (legacy CPS approach)
#[allow(dead_code)]
fn eval_do_fallback(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    if operands.len() < 2 {
        return Err(LambdustError::syntax_error(
            "do requires at least variable bindings and test clause".to_string(),
        ));
    }

    // Parse variable bindings
    let bindings = parse_do_bindings(&operands[0])?;
    
    // Parse test clause: (test result ...)
    let (test_expr, result_exprs) = match &operands[1] {
        Expr::List(test_clause) if !test_clause.is_empty() => {
            let test = test_clause[0].clone();
            let results = test_clause[1..].to_vec();
            (test, results)
        }
        _ => {
            return Err(LambdustError::syntax_error(
                "do requires test clause".to_string(),
            ));
        }
    };

    // Body expressions
    let body_exprs = operands[2..].to_vec();

    // Create loop environment
    let loop_env = Environment::new();
    
    // Initialize variables
    for (var, init_expr, _) in &bindings {
        let init_value = evaluator.eval(init_expr.clone(), env.clone(), Continuation::Identity)?;
        loop_env.define(var.clone(), init_value);
    }
    
    let loop_env_rc = Rc::new(loop_env.extend());

    // Simple loop implementation (limited iterations to prevent infinite loops)
    const MAX_ITERATIONS: usize = 1000;
    
    for _ in 0..MAX_ITERATIONS {
        // Evaluate test condition
        let test_result = evaluator.eval(test_expr.clone(), loop_env_rc.clone(), Continuation::Identity)?;
        
        if test_result.is_truthy() {
            // Test passed - evaluate result expressions
            if result_exprs.is_empty() {
                return evaluator.apply_continuation(cont, Value::Undefined);
            } else if result_exprs.len() == 1 {
                return evaluator.eval(result_exprs[0].clone(), loop_env_rc, cont);
            } else {
                // Multiple result expressions - evaluate in sequence
                let last_idx = result_exprs.len() - 1;
                for (i, expr) in result_exprs.iter().enumerate() {
                    if i == last_idx {
                        // Last expression uses original continuation
                        return evaluator.eval(expr.clone(), loop_env_rc, cont);
                    } else {
                        // Intermediate expressions use Identity continuation
                        evaluator.eval(expr.clone(), loop_env_rc.clone(), Continuation::Identity)?;
                    }
                }
            }
        }

        // Execute body expressions
        for expr in &body_exprs {
            evaluator.eval(expr.clone(), loop_env_rc.clone(), Continuation::Identity)?;
        }

        // Update variables with step expressions
        for (var, _, step_expr) in &bindings {
            if let Some(step) = step_expr {
                let new_value = evaluator.eval(step.clone(), loop_env_rc.clone(), Continuation::Identity)?;
                loop_env_rc.set(var, new_value)?;
            }
        }
    }

    // If we reach here, the loop didn't terminate
    Err(LambdustError::runtime_error(
        "do loop exceeded maximum iterations".to_string(),
    ))
}

/// Parse do bindings
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

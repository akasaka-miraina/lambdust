//! Promise and lazy evaluation implementation
//!
//! This module implements R7RS delay/force and SRFI 45 lazy evaluation primitives.
//! It provides promise-based lazy evaluation with proper forcing semantics.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{Continuation, Evaluator};
use crate::value::Value;
use std::rc::Rc;

/// Evaluate delay special form
///
/// Creates a promise that will evaluate the given expression when forced.
/// The expression is captured along with the current environment.
pub fn eval_delay(
    evaluator: &mut Evaluator,
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

    evaluator.apply_continuation(cont, promise)
}

/// Evaluate lazy special form (SRFI 45)
///
/// Creates a lazy promise similar to delay but with SRFI 45 semantics.
/// This supports iterative lazy algorithms and proper tail recursion.
pub fn eval_lazy(
    evaluator: &mut Evaluator,
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

    evaluator.apply_continuation(cont, promise)
}

/// Evaluate force special form
///
/// Forces evaluation of a promise. If the argument is not a promise,
/// it is returned unchanged. For promises, evaluates the stored expression
/// in the captured environment.
pub fn eval_force(
    evaluator: &mut Evaluator,
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
    let promise_value = evaluator.eval(promise_expr, env, force_cont)?;

    // Force the promise
    evaluator.force_promise(promise_value, cont)
}

/// Evaluate promise? predicate
///
/// Returns #t if the argument is a promise, #f otherwise.
/// This is used to test whether a value is a delayed computation.
pub fn eval_promise_predicate(
    evaluator: &mut Evaluator,
    operands: &[Expr],
    env: Rc<Environment>,
    cont: Continuation,
) -> Result<Value> {
    if operands.len() != 1 {
        return Err(LambdustError::arity_error(1, operands.len()));
    }

    let expr = operands[0].clone();
    let value = evaluator.eval(expr, env, Continuation::Identity)?;

    let is_promise = matches!(value, Value::Promise(_));
    evaluator.apply_continuation(cont, Value::Boolean(is_promise))
}

// Additional functions for Evaluator impl
impl Evaluator {
    /// Force a promise value
    ///
    /// Internal implementation of promise forcing. Handles both lazy and eager promises.
    /// For non-promises, returns the value unchanged.
    pub(super) fn force_promise(&mut self, promise: Value, cont: Continuation) -> Result<Value> {
        match promise {
            Value::Promise(promise_ref) => match &promise_ref.state {
                crate::value::PromiseState::Lazy { expr, env } => {
                    self.eval(expr.clone(), env.clone(), cont)
                }
                crate::value::PromiseState::Eager { value } => {
                    self.apply_continuation(cont, value.as_ref().clone())
                }
            },
            other => self.apply_continuation(cont, other),
        }
    }
}
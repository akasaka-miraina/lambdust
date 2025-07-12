//! SRFI 45: Primitives for Expressing Iterative Lazy Algorithms
//!
//! This module implements the lazy evaluation primitives defined in SRFI 45.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::LambdustError;
use crate::value::{Procedure, Promise, PromiseState, Value};
use std::collections::HashMap;
use std::rc::Rc;

/// Register all lazy evaluation functions
pub fn register_lazy_functions(builtins: &mut HashMap<String, Value>) {
    builtins.insert("force".to_string(), force_function());
    builtins.insert("promise?".to_string(), promise_predicate());
    // delay and lazy are handled as special forms in the evaluator
}

/// Implements the `force` function for forcing promise evaluation
fn force_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "force".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            // For now, return a placeholder implementation
            // A complete implementation would require evaluator access
            Err(LambdustError::runtime_error(
                "force: requires evaluator integration - not yet fully implemented".to_string(),
            ))
        },
    })
}

/// Implements the `promise?` predicate
#[must_use] pub fn promise_predicate() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "promise?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            Ok(Value::Boolean(matches!(args[0], Value::Promise(_))))
        },
    })
}

/// Create a lazy promise from expression and environment
#[must_use] pub fn make_lazy_promise(expr: Expr, env: Rc<Environment>) -> Value {
    Value::Promise(Promise {
        state: PromiseState::Lazy { expr, env },
    })
}

/// Create an eager promise from a value
#[must_use] pub fn make_eager_promise(value: Value) -> Value {
    Value::Promise(Promise {
        state: PromiseState::Eager {
            value: Box::new(value),
        },
    })
}

/// Force a promise, returning the evaluated value (for `FormalEvaluator`)
/// This requires evaluator integration for complete implementation
pub fn force_promise(
    promise: &Promise,
    evaluator: &mut crate::evaluator::Evaluator,
) -> crate::Result<Value> {
    match &promise.state {
        PromiseState::Eager { value } => Ok((**value).clone()),
        PromiseState::Lazy { expr, env } => {
            // Evaluate the expression in the stored environment using formal evaluator
            let result = evaluator.eval(
                expr.clone(),
                env.clone(),
                crate::evaluator::Continuation::Identity,
            )?;

            // If the result is another promise, force it recursively
            match result {
                Value::Promise(ref inner_promise) => force_promise(inner_promise, evaluator),
                value => Ok(value),
            }
        }
    }
}

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
fn promise_predicate() -> Value {
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
pub fn make_lazy_promise(expr: Expr, env: Rc<Environment>) -> Value {
    Value::Promise(Promise {
        state: PromiseState::Lazy { expr, env },
    })
}

/// Create an eager promise from a value
pub fn make_eager_promise(value: Value) -> Value {
    Value::Promise(Promise {
        state: PromiseState::Eager {
            value: Box::new(value),
        },
    })
}

/// Force a promise, returning the evaluated value
/// This requires evaluator integration for complete implementation
pub fn force_promise(
    promise: &Promise,
    evaluator: &mut crate::evaluator::Evaluator,
) -> crate::Result<Value> {
    match &promise.state {
        PromiseState::Eager { value } => Ok((**value).clone()),
        PromiseState::Lazy { expr, env } => {
            // Evaluate the expression in the stored environment
            let result = evaluator.eval_in_env(expr.clone(), env.clone())?;

            // If the result is another promise, force it recursively
            match result {
                Value::Promise(ref inner_promise) => force_promise(inner_promise, evaluator),
                value => Ok(value),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_promise_predicate() {
        let predicate = promise_predicate();

        // Test with promise
        let promise = make_eager_promise(Value::Number(SchemeNumber::Integer(42)));
        let result = match predicate {
            Value::Procedure(Procedure::Builtin { func, .. }) => func(&[promise]),
            _ => panic!("Expected builtin procedure"),
        }
        .unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test with non-promise
        let non_promise = Value::Number(SchemeNumber::Integer(42));
        let result = match predicate {
            Value::Procedure(Procedure::Builtin { func, .. }) => func(&[non_promise]),
            _ => panic!("Expected builtin procedure"),
        }
        .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_make_promises() {
        // Test eager promise
        let eager = make_eager_promise(Value::Number(SchemeNumber::Integer(42)));
        assert!(matches!(
            eager,
            Value::Promise(Promise {
                state: PromiseState::Eager { .. }
            })
        ));

        // Test lazy promise
        let expr = Expr::Literal(crate::ast::Literal::Number(SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        let lazy = make_lazy_promise(expr, env);
        assert!(matches!(
            lazy,
            Value::Promise(Promise {
                state: PromiseState::Lazy { .. }
            })
        ));
    }
}

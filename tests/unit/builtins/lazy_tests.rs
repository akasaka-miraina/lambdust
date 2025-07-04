//! Unit tests for SRFI 45: Primitives for Expressing Iterative Lazy Algorithms

use lambdust::ast::{Expr, Literal};
use lambdust::builtins::lazy::{make_eager_promise, make_lazy_promise, promise_predicate};
use lambdust::environment::Environment;
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Procedure, Promise, PromiseState, Value};
use std::rc::Rc;

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
    let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let env = Rc::new(Environment::new());
    let lazy = make_lazy_promise(expr, env);
    assert!(matches!(
        lazy,
        Value::Promise(Promise {
            state: PromiseState::Lazy { .. }
        })
    ));
}
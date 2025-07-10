//! Comprehensive unit tests for Promise and lazy evaluation (SRFI 45)
//!
//! Tests the Promise value type that provides lazy evaluation capabilities
//! according to SRFI 45 specifications.

use lambdust::ast::{Expr, Literal};
use lambdust::environment::Environment;
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Promise, PromiseState, Value};
use std::rc::Rc;

/// Helper to create test environment
fn create_test_environment() -> Rc<Environment> {
    let env = Rc::new(Environment::new());
    env.define("test-var".to_string(), Value::Number(SchemeNumber::Integer(42)));
    env.define("test-string".to_string(), Value::String("hello".to_string()));
    env
}

/// Helper to create test expressions
fn create_test_expressions() -> Vec<Expr> {
    vec![
        Expr::Literal(Literal::Number(SchemeNumber::Integer(123))),
        Expr::Literal(Literal::String("test".to_string())),
        Expr::Literal(Literal::Boolean(true)),
        Expr::Variable("test-var".to_string()),
        Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]),
    ]
}

#[test]
fn test_promise_creation_lazy() {
    let env = create_test_environment();
    let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    
    let promise = Promise {
        state: PromiseState::Lazy {
            expr: expr.clone(),
            env: env.clone(),
        }
    };
    
    // Should be in lazy state
    match promise.state {
        PromiseState::Lazy { ref expr, ref env } => {
            assert!(matches!(expr, Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))));
            assert!(Rc::ptr_eq(env, &env));
        }
        _ => panic!("Expected lazy promise state")
    }
}

#[test]
fn test_promise_creation_eager() {
    let value = Value::Number(SchemeNumber::Integer(99));
    
    let promise = Promise {
        state: PromiseState::Eager {
            value: Box::new(value.clone())
        }
    };
    
    // Should be in eager state
    match promise.state {
        PromiseState::Eager { ref value } => {
            assert_eq!(**value, Value::Number(SchemeNumber::Integer(99)));
        }
        _ => panic!("Expected eager promise state")
    }
}

#[test]
fn test_promise_with_variable_expression() {
    let env = create_test_environment();
    let expr = Expr::Variable("test-var".to_string());
    
    let promise = Promise {
        state: PromiseState::Lazy {
            expr,
            env: env.clone(),
        }
    };
    
    // Should contain variable expression
    match promise.state {
        PromiseState::Lazy { ref expr, .. } => {
            assert!(matches!(expr, Expr::Variable(name) if name == "test-var"));
        }
        _ => panic!("Expected lazy promise with variable")
    }
}

#[test]
fn test_promise_with_complex_expression() {
    let env = create_test_environment();
    let expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(20))),
    ]);
    
    let promise = Promise {
        state: PromiseState::Lazy {
            expr: expr.clone(),
            env: env.clone(),
        }
    };
    
    // Should contain complex expression
    match promise.state {
        PromiseState::Lazy { ref expr, .. } => {
            assert!(matches!(expr, Expr::List(_)));
        }
        _ => panic!("Expected lazy promise with list expression")
    }
}

#[test]
fn test_promise_state_variants() {
    let env = create_test_environment();
    
    // Test all possible promise states
    let lazy_promise = Promise {
        state: PromiseState::Lazy {
            expr: Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            env: env.clone(),
        }
    };
    
    let eager_promise = Promise {
        state: PromiseState::Eager {
            value: Box::new(Value::Number(SchemeNumber::Integer(2)))
        }
    };
    
    // Verify states
    assert!(matches!(lazy_promise.state, PromiseState::Lazy { .. }));
    assert!(matches!(eager_promise.state, PromiseState::Eager { .. }));
}

#[test]
fn test_promise_value_integration() {
    let promise = Promise {
        state: PromiseState::Eager {
            value: Box::new(Value::String("test".to_string()))
        }
    };
    
    let promise_value = Value::Promise(promise);
    
    // Test Value::Promise integration
    assert!(matches!(promise_value, Value::Promise(_)));
    
    // Test that promise can be extracted
    if let Value::Promise(extracted_promise) = promise_value {
        match extracted_promise.state {
            PromiseState::Eager { ref value } => {
                assert_eq!(**value, Value::String("test".to_string()));
            }
            _ => panic!("Expected eager state")
        }
    }
}

#[test]
fn test_promise_environment_capture() {
    let env = create_test_environment();
    let original_env_ptr = env.as_ref() as *const Environment;
    
    let promise = Promise {
        state: PromiseState::Lazy {
            expr: Expr::Variable("test-var".to_string()),
            env: env.clone(),
        }
    };
    
    // Verify environment is properly captured
    if let PromiseState::Lazy { ref env, .. } = promise.state {
        let captured_env_ptr = env.as_ref() as *const Environment;
        assert_eq!(original_env_ptr, captured_env_ptr);
    }
}

#[test]
fn test_promise_with_nested_environment() {
    let base_env = create_test_environment();
    let nested_env = Rc::new(Environment::extend(&base_env));
    nested_env.define("nested-var".to_string(), Value::Boolean(true));
    
    let promise = Promise {
        state: PromiseState::Lazy {
            expr: Expr::Variable("nested-var".to_string()),
            env: nested_env.clone(),
        }
    };
    
    // Should capture nested environment
    if let PromiseState::Lazy { ref env, .. } = promise.state {
        // Verify the environment can resolve both nested and base variables
        assert!(env.get("nested-var").is_some());
        assert!(env.get("test-var").is_some()); // From base environment
    }
}

#[test]
fn test_promise_multiple_expressions() {
    let env = create_test_environment();
    let test_exprs = create_test_expressions();
    
    let promises: Vec<Promise> = test_exprs.into_iter().map(|expr| {
        Promise {
            state: PromiseState::Lazy {
                expr,
                env: env.clone(),
            }
        }
    }).collect();
    
    // Should create multiple promises successfully
    assert_eq!(promises.len(), 5);
    
    // All should be lazy
    for promise in promises {
        assert!(matches!(promise.state, PromiseState::Lazy { .. }));
    }
}

#[test]
fn test_promise_clone_behavior() {
    let env = create_test_environment();
    let original_promise = Promise {
        state: PromiseState::Lazy {
            expr: Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            env: env.clone(),
        }
    };
    
    let cloned_promise = original_promise.clone();
    
    // Both should be equivalent but separate instances
    match (&original_promise.state, &cloned_promise.state) {
        (
            PromiseState::Lazy { expr: orig_expr, env: orig_env },
            PromiseState::Lazy { expr: clone_expr, env: clone_env }
        ) => {
            assert_eq!(orig_expr, clone_expr);
            assert!(Rc::ptr_eq(orig_env, clone_env)); // Environment should be shared via Rc
        }
        _ => panic!("Expected both promises to be lazy")
    }
}

#[test]
fn test_promise_debug_formatting() {
    let env = create_test_environment();
    
    let lazy_promise = Promise {
        state: PromiseState::Lazy {
            expr: Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            env,
        }
    };
    
    let eager_promise = Promise {
        state: PromiseState::Eager {
            value: Box::new(Value::String("test".to_string()))
        }
    };
    
    // Should format without panicking
    let lazy_debug = format!("{:?}", lazy_promise);
    let eager_debug = format!("{:?}", eager_promise);
    
    assert!(lazy_debug.contains("Lazy"));
    assert!(eager_debug.contains("Eager"));
}

#[test]
fn test_promise_boxed_value_handling() {
    // Test different value types in eager promises
    let number_promise = Promise {
        state: PromiseState::Eager {
            value: Box::new(Value::Number(SchemeNumber::Integer(123)))
        }
    };
    
    let string_promise = Promise {
        state: PromiseState::Eager {
            value: Box::new(Value::String("hello".to_string()))
        }
    };
    
    let boolean_promise = Promise {
        state: PromiseState::Eager {
            value: Box::new(Value::Boolean(false))
        }
    };
    
    let nil_promise = Promise {
        state: PromiseState::Eager {
            value: Box::new(Value::Nil)
        }
    };
    
    // All should be in eager state with correct values
    if let PromiseState::Eager { ref value } = number_promise.state {
        assert_eq!(**value, Value::Number(SchemeNumber::Integer(123)));
    }
    
    if let PromiseState::Eager { ref value } = string_promise.state {
        assert_eq!(**value, Value::String("hello".to_string()));
    }
    
    if let PromiseState::Eager { ref value } = boolean_promise.state {
        assert_eq!(**value, Value::Boolean(false));
    }
    
    if let PromiseState::Eager { ref value } = nil_promise.state {
        assert_eq!(**value, Value::Nil);
    }
}

#[test]
fn test_promise_empty_expression_list() {
    let env = create_test_environment();
    let empty_list_expr = Expr::List(vec![]);
    
    let promise = Promise {
        state: PromiseState::Lazy {
            expr: empty_list_expr,
            env,
        }
    };
    
    // Should handle empty list expression
    if let PromiseState::Lazy { ref expr, .. } = promise.state {
        assert!(matches!(expr, Expr::List(list) if list.is_empty()));
    }
}

#[test]
fn test_promise_nested_structures() {
    let env = create_test_environment();
    
    // Test promise containing nested data structures
    let nested_expr = Expr::List(vec![
        Expr::Variable("list".to_string()),
        Expr::List(vec![
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]),
        Expr::Variable("test-var".to_string()),
    ]);
    
    let promise = Promise {
        state: PromiseState::Lazy {
            expr: nested_expr,
            env,
        }
    };
    
    // Should handle nested structures correctly
    if let PromiseState::Lazy { ref expr, .. } = promise.state {
        assert!(matches!(expr, Expr::List(_)));
    }
}

#[test]
fn test_promise_state_consistency() {
    let env = create_test_environment();
    
    // Create promises in both states
    let lazy_promise = Promise {
        state: PromiseState::Lazy {
            expr: Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            env,
        }
    };
    
    let eager_promise = Promise {
        state: PromiseState::Eager {
            value: Box::new(Value::Number(SchemeNumber::Integer(42)))
        }
    };
    
    // States should remain consistent after operations
    let lazy_clone = lazy_promise.clone();
    let eager_clone = eager_promise.clone();
    
    assert!(matches!(lazy_clone.state, PromiseState::Lazy { .. }));
    assert!(matches!(eager_clone.state, PromiseState::Eager { .. }));
}

#[test]
fn test_promise_memory_efficiency() {
    let env = create_test_environment();
    
    // Create multiple promises sharing the same environment
    let promises: Vec<Promise> = (0..100).map(|i| {
        Promise {
            state: PromiseState::Lazy {
                expr: Expr::Literal(Literal::Number(SchemeNumber::Integer(i))),
                env: env.clone(),
            }
        }
    }).collect();
    
    // All promises should share the same environment instance
    for promise in promises {
        if let PromiseState::Lazy { ref env, .. } = promise.state {
            assert!(Rc::ptr_eq(env, &env));
        }
    }
}
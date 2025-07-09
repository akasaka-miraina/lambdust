//! Unit tests for CompactContinuation system (Phase 4 optimization)

use lambdust::ast::Expr;
use lambdust::environment::Environment;
use lambdust::evaluator::{CompactContinuation, Continuation, EnvironmentRef, InlineContinuation};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use std::rc::Rc;

#[test]
fn test_compact_continuation_identity() {
    let cont = Continuation::Identity;
    let compact = CompactContinuation::from_continuation(cont);

    assert!(compact.is_inline());
    if let CompactContinuation::Inline(inline_cont) = compact {
        if let InlineContinuation::Identity = *inline_cont {
            // Expected case
        } else {
            panic!("Expected Identity InlineContinuation");
        }
    } else {
        panic!("Expected Inline CompactContinuation");
    }
}

#[test]
fn test_compact_continuation_values_small() {
    let values = vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
    ];
    let cont = Continuation::Values {
        values: values.clone(),
        parent: Box::new(Continuation::Identity),
    };

    let compact = CompactContinuation::from_continuation(cont);
    assert!(compact.is_inline());

    if let CompactContinuation::Inline(inline_cont) = compact {
        if let InlineContinuation::Values(small_values) = *inline_cont {
            assert_eq!(small_values.len(), 2);
            assert_eq!(small_values[0], Value::Number(SchemeNumber::Integer(1)));
            assert_eq!(small_values[1], Value::Number(SchemeNumber::Integer(2)));
        } else {
            panic!("Expected inline values continuation");
        }
    } else {
        panic!("Expected Inline CompactContinuation");
    }
}

#[test]
fn test_compact_continuation_values_large() {
    // Test with more than 4 values (should go to boxed)
    let values = vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
        Value::Number(SchemeNumber::Integer(4)),
        Value::Number(SchemeNumber::Integer(5)), // 5th value triggers boxing
    ];
    let cont = Continuation::Values {
        values,
        parent: Box::new(Continuation::Identity),
    };

    let compact = CompactContinuation::from_continuation(cont);
    assert!(!compact.is_inline()); // Should be boxed due to size
}

#[test]
fn test_environment_ref_creation() {
    let env = Rc::new(Environment::new());
    let env_ref = EnvironmentRef::new(Rc::clone(&env));

    // Should be able to get strong reference
    assert!(env_ref.get().is_some());

    // Original environment should be reachable
    let retrieved = env_ref.get().unwrap();
    assert!(Rc::ptr_eq(&env, &retrieved));
}

#[test]
fn test_environment_ref_weak_reference() {
    let env_ref = {
        let env = Rc::new(Environment::new());
        EnvironmentRef::new(env)
        // env goes out of scope here
    };

    // Should still have strong reference initially
    assert!(env_ref.get().is_some());
}

#[test]
fn test_compact_continuation_assignment() {
    let env = Rc::new(Environment::new());
    let cont = Continuation::Assignment {
        variable: "x".to_string(),
        env: Rc::clone(&env),
        parent: Box::new(Continuation::Identity),
    };

    let compact = CompactContinuation::from_continuation(cont);
    assert!(compact.is_inline());

    if let CompactContinuation::Inline(inline_cont) = compact {
        if let InlineContinuation::Assignment { var_name, env_ref } = *inline_cont {
            assert_eq!(var_name, "x");
            assert!(env_ref.get().is_some());
        } else {
            panic!("Expected Assignment InlineContinuation");
        }
    } else {
        panic!("Expected Inline CompactContinuation");
    }
}

#[test]
fn test_compact_continuation_single_begin() {
    let env = Rc::new(Environment::new());
    let expr = Expr::Literal(lambdust::ast::Literal::Number(SchemeNumber::Integer(42)));
    let cont = Continuation::Begin {
        remaining: vec![expr.clone()],
        env: Rc::clone(&env),
        parent: Box::new(Continuation::Identity),
    };

    let compact = CompactContinuation::from_continuation(cont);
    // Note: SingleBegin optimization is disabled (see continuation.rs:240-253)
    // to ensure proper continuation evaluation in begin blocks where variable
    // bindings may be established in previous expressions.
    assert!(!compact.is_inline()); // Should be boxed due to disabled optimization

    if let CompactContinuation::Boxed(boxed_cont) = compact {
        if let Continuation::Begin {
            remaining,
            env,
            parent,
        } = *boxed_cont
        {
            assert_eq!(remaining.len(), 1);
            assert_eq!(remaining[0], expr);
            assert!(Rc::ptr_eq(&env, &env));
            assert!(matches!(*parent, Continuation::Identity));
        } else {
            panic!("Expected Begin Continuation");
        }
    } else {
        panic!("Expected Boxed CompactContinuation");
    }
}

#[test]
fn test_compact_continuation_multiple_begin() {
    let env = Rc::new(Environment::new());
    let expr1 = Expr::Literal(lambdust::ast::Literal::Number(SchemeNumber::Integer(1)));
    let expr2 = Expr::Literal(lambdust::ast::Literal::Number(SchemeNumber::Integer(2)));
    let cont = Continuation::Begin {
        remaining: vec![expr1, expr2], // Multiple expressions should go to boxed
        env: Rc::clone(&env),
        parent: Box::new(Continuation::Identity),
    };

    let compact = CompactContinuation::from_continuation(cont);
    assert!(!compact.is_inline()); // Should be boxed due to multiple expressions
}

#[test]
fn test_compact_continuation_simple_if() {
    let env = Rc::new(Environment::new());
    let consequent = Expr::Literal(lambdust::ast::Literal::Number(SchemeNumber::Integer(1)));
    let alternate = Some(Expr::Literal(lambdust::ast::Literal::Number(
        SchemeNumber::Integer(2),
    )));
    let cont = Continuation::IfTest {
        consequent: consequent.clone(),
        alternate: alternate.clone(),
        env: Rc::clone(&env),
        parent: Box::new(Continuation::Identity),
    };

    let compact = CompactContinuation::from_continuation(cont);
    assert!(compact.is_inline());

    if let CompactContinuation::Inline(inline_cont) = compact {
        if let InlineContinuation::SimpleIf {
            consequent: cons_expr,
            alternate: alt_expr,
            env_ref,
        } = *inline_cont
        {
            assert_eq!(cons_expr, consequent);
            assert_eq!(alt_expr, alternate);
            assert!(env_ref.get().is_some());
        } else {
            panic!("Expected SimpleIf InlineContinuation");
        }
    } else {
        panic!("Expected Inline CompactContinuation");
    }
}

#[test]
fn test_compact_continuation_memory_size() {
    // Test memory size calculation
    let identity = CompactContinuation::Inline(Box::new(InlineContinuation::Identity));
    assert_eq!(identity.memory_size(), 0);

    let values = CompactContinuation::Inline(Box::new(InlineContinuation::Values(Box::new(
        vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
        ]
        .into(),
    ))));
    assert!(values.memory_size() > 0);

    let env = Rc::new(Environment::new());
    let assignment = CompactContinuation::Inline(Box::new(InlineContinuation::Assignment {
        var_name: "test".to_string(),
        env_ref: EnvironmentRef::new(env),
    }));
    assert!(assignment.memory_size() > 0);
}

#[test]
fn test_compact_continuation_complex_goes_to_boxed() {
    // Test that complex continuations go to boxed storage
    let env = Rc::new(Environment::new());

    // Nested continuation should go to boxed
    let nested_parent = Continuation::Values {
        values: vec![Value::Number(SchemeNumber::Integer(1))],
        parent: Box::new(Continuation::Identity),
    };

    let cont = Continuation::Assignment {
        variable: "x".to_string(),
        env: Rc::clone(&env),
        parent: Box::new(nested_parent), // Non-identity parent -> boxed
    };

    let compact = CompactContinuation::from_continuation(cont);
    assert!(!compact.is_inline()); // Should be boxed due to complex parent
}

#[test]
fn test_inline_continuation_apply_basic() {
    // Test basic inline continuation application
    let identity = *Box::new(InlineContinuation::Identity);
    let value = Value::Number(SchemeNumber::Integer(42));
    let result = identity.apply(value.clone()).unwrap();
    assert_eq!(result, value);

    let values = vec![Value::Number(SchemeNumber::Integer(1))].into();
    let values_cont = *Box::new(InlineContinuation::Values(Box::new(values)));
    let result = values_cont
        .apply(Value::Number(SchemeNumber::Integer(2)))
        .unwrap();
    if let Value::Values(result_vec) = result {
        assert_eq!(result_vec.len(), 2);
        assert_eq!(result_vec[0], Value::Number(SchemeNumber::Integer(1)));
        assert_eq!(result_vec[1], Value::Number(SchemeNumber::Integer(2)));
    } else {
        panic!("Expected Values result");
    }
}

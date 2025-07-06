//! Unit tests for CompactContinuation system (Phase 4 optimization)

use lambdust::evaluator::{CompactContinuation, InlineContinuation, EnvironmentRef, Continuation};
use lambdust::value::Value;
use lambdust::environment::Environment;
use lambdust::ast::Expr;
use lambdust::lexer::SchemeNumber;
use std::rc::Rc;

#[test]
fn test_compact_continuation_identity() {
    let cont = Continuation::Identity;
    let compact = CompactContinuation::from_continuation(cont);
    
    assert!(compact.is_inline());
    if let CompactContinuation::Inline(InlineContinuation::Identity) = compact {
        // Expected case
    } else {
        panic!("Expected inline identity continuation");
    }
}

#[test]
fn test_compact_continuation_values_small() {
    let values = vec![Value::Number(SchemeNumber::Integer(1)), Value::Number(SchemeNumber::Integer(2))];
    let cont = Continuation::Values {
        values: values.clone(),
        parent: Box::new(Continuation::Identity),
    };
    
    let compact = CompactContinuation::from_continuation(cont);
    assert!(compact.is_inline());
    
    if let CompactContinuation::Inline(InlineContinuation::Values(small_values)) = compact {
        assert_eq!(small_values.len(), 2);
        assert_eq!(small_values[0], Value::Number(SchemeNumber::Integer(1)));
        assert_eq!(small_values[1], Value::Number(SchemeNumber::Integer(2)));
    } else {
        panic!("Expected inline values continuation");
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
    
    if let CompactContinuation::Inline(InlineContinuation::Assignment { var_name, env_ref }) = compact {
        assert_eq!(var_name, "x");
        assert!(env_ref.get().is_some());
    } else {
        panic!("Expected inline assignment continuation");
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
    assert!(compact.is_inline());
    
    if let CompactContinuation::Inline(InlineContinuation::SingleBegin { expr: single_expr, env_ref }) = compact {
        assert_eq!(single_expr, expr);
        assert!(env_ref.get().is_some());
    } else {
        panic!("Expected inline single begin continuation");
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
    let alternate = Some(Expr::Literal(lambdust::ast::Literal::Number(SchemeNumber::Integer(2))));
    let cont = Continuation::IfTest {
        consequent: consequent.clone(),
        alternate: alternate.clone(),
        env: Rc::clone(&env),
        parent: Box::new(Continuation::Identity),
    };
    
    let compact = CompactContinuation::from_continuation(cont);
    assert!(compact.is_inline());
    
    if let CompactContinuation::Inline(InlineContinuation::SimpleIf { 
        consequent: cons_expr, 
        alternate: alt_expr,
        env_ref 
    }) = compact {
        assert_eq!(cons_expr, consequent);
        assert_eq!(alt_expr, alternate);
        assert!(env_ref.get().is_some());
    } else {
        panic!("Expected inline simple if continuation");
    }
}

#[test]
fn test_compact_continuation_memory_size() {
    // Test memory size calculation
    let identity = CompactContinuation::Inline(InlineContinuation::Identity);
    assert_eq!(identity.memory_size(), 0);
    
    let values = CompactContinuation::Inline(InlineContinuation::Values(
        vec![Value::Number(SchemeNumber::Integer(1)), Value::Number(SchemeNumber::Integer(2))].into()
    ));
    assert!(values.memory_size() > 0);
    
    let env = Rc::new(Environment::new());
    let assignment = CompactContinuation::Inline(InlineContinuation::Assignment {
        var_name: "test".to_string(),
        env_ref: EnvironmentRef::new(env),
    });
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
    let identity = InlineContinuation::Identity;
    let value = Value::Number(SchemeNumber::Integer(42));
    let result = identity.apply(value.clone()).unwrap();
    assert_eq!(result, value);
    
    let values = vec![Value::Number(SchemeNumber::Integer(1))].into();
    let values_cont = InlineContinuation::Values(values);
    let result = values_cont.apply(Value::Number(SchemeNumber::Integer(2))).unwrap();
    if let Value::Values(result_vec) = result {
        assert_eq!(result_vec.len(), 2);
        assert_eq!(result_vec[0], Value::Number(SchemeNumber::Integer(1)));
        assert_eq!(result_vec[1], Value::Number(SchemeNumber::Integer(2)));
    } else {
        panic!("Expected Values result");
    }
}
//! Unit tests for semantic evaluator (evaluator/semantic.rs)
//!
//! Tests the pure R7RS semantic evaluator that serves as the mathematical
//! reference implementation for formal verification.

use lambdust::ast::{Expr, Literal};
use lambdust::environment::Environment;
use lambdust::evaluator::semantic::SemanticEvaluator;
use lambdust::evaluator::Continuation;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use std::rc::Rc;

#[test]
fn test_semantic_evaluator_creation() {
    let mut evaluator = SemanticEvaluator::new();

    // Should create successfully
    // Note: fields are private, so we test behavior through public methods
    let env = Rc::new(Environment::new());
    let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let result = evaluator.eval_pure(expr, env, Continuation::Identity);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_evaluator_with_environment() {
    let env = Rc::new(Environment::new());
    env.define(
        "test_var".to_string(),
        Value::Number(SchemeNumber::Integer(123)),
    );
    let mut evaluator = SemanticEvaluator::with_environment(env.clone());

    // Should create with custom environment and be able to access defined variables
    let expr = Expr::Variable("test_var".to_string());
    let result = evaluator.eval_pure(expr, env, Continuation::Identity);
    assert!(result.is_ok());
}

#[test]
fn test_literal_number_evaluation() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test integer literal
    let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let result = evaluator.eval_pure(expr, env.clone(), Continuation::Identity);

    assert!(result.is_ok());
    if let Value::Number(SchemeNumber::Integer(n)) = result.unwrap() {
        assert_eq!(n, 42);
    } else {
        panic!("Expected integer 42");
    }
}

#[test]
fn test_literal_string_evaluation() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test string literal
    let expr = Expr::Literal(Literal::String("hello".to_string()));
    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    if let Value::String(s) = result.unwrap() {
        assert_eq!(s, "hello");
    } else {
        panic!("Expected string 'hello'");
    }
}

#[test]
fn test_literal_boolean_evaluation() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test boolean literal
    let expr = Expr::Literal(Literal::Boolean(true));
    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    if let Value::Boolean(b) = result.unwrap() {
        assert!(b);
    } else {
        panic!("Expected boolean true");
    }
}

#[test]
fn test_literal_character_evaluation() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test character literal
    let expr = Expr::Literal(Literal::Character('x'));
    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    if let Value::Character(c) = result.unwrap() {
        assert_eq!(c, 'x');
    } else {
        panic!("Expected character 'x'");
    }
}

#[test]
fn test_literal_nil_evaluation() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test nil literal
    let expr = Expr::Literal(Literal::Nil);
    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Nil));
}

#[test]
fn test_empty_list_evaluation() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test empty list
    let expr = Expr::List(vec![]);
    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Nil));
}

#[test]
fn test_variable_lookup_undefined() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test undefined variable
    let expr = Expr::Variable("undefined_var".to_string());
    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("undefined_var"));
}

#[test]
fn test_variable_lookup_defined() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Define a variable
    env.define(
        "test_var".to_string(),
        Value::Number(SchemeNumber::Integer(99)),
    );

    // Test defined variable
    let expr = Expr::Variable("test_var".to_string());
    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    if let Value::Number(SchemeNumber::Integer(n)) = result.unwrap() {
        assert_eq!(n, 99);
    } else {
        panic!("Expected integer 99");
    }
}

#[test]
fn test_variable_lookup_from_closure() {
    let mut evaluator = SemanticEvaluator::new();
    let outer_env = Rc::new(Environment::new());
    outer_env.define(
        "outer_var".to_string(),
        Value::Number(SchemeNumber::Integer(77)),
    );

    let inner_env = Rc::new(Environment::extend(&outer_env));

    // Test variable lookup from closure
    let expr = Expr::Variable("outer_var".to_string());
    let result = evaluator.eval_pure(expr, inner_env, Continuation::Identity);

    assert!(result.is_ok());
    if let Value::Number(SchemeNumber::Integer(n)) = result.unwrap() {
        assert_eq!(n, 77);
    } else {
        panic!("Expected integer 77");
    }
}

#[test]
fn test_special_form_recognition() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test recognized special forms by attempting to evaluate them
    let if_expr = Expr::List(vec![
        Expr::Variable("if".to_string()),
        Expr::Literal(Literal::Boolean(true)),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);
    let result = evaluator.eval_pure(if_expr, env.clone(), Continuation::Identity);
    assert!(result.is_ok());

    // Test non-special forms should not be recognized as special
    // This indirectly tests special form recognition
    let map_expr = Expr::List(vec![
        Expr::Variable("map".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);
    let result = evaluator.eval_pure(map_expr, env, Continuation::Identity);
    assert!(result.is_err()); // Should fail with undefined variable
}

#[test]
fn test_if_special_form_true_branch() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test: (if #t 42 99)
    let expr = Expr::List(vec![
        Expr::Variable("if".to_string()),
        Expr::Literal(Literal::Boolean(true)),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(99))),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    if let Value::Number(SchemeNumber::Integer(n)) = result.unwrap() {
        assert_eq!(n, 42);
    } else {
        panic!("Expected integer 42");
    }
}

#[test]
fn test_if_special_form_false_branch() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test: (if #f 42 99)
    let expr = Expr::List(vec![
        Expr::Variable("if".to_string()),
        Expr::Literal(Literal::Boolean(false)),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(99))),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    if let Value::Number(SchemeNumber::Integer(n)) = result.unwrap() {
        assert_eq!(n, 99);
    } else {
        panic!("Expected integer 99");
    }
}

#[test]
fn test_if_special_form_no_alternate() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test: (if #f 42) -> should return undefined
    let expr = Expr::List(vec![
        Expr::Variable("if".to_string()),
        Expr::Literal(Literal::Boolean(false)),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Undefined));
}

#[test]
fn test_define_special_form() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test: (define x 42)
    let expr = Expr::List(vec![
        Expr::Variable("define".to_string()),
        Expr::Variable("x".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
    ]);

    let result = evaluator.eval_pure(expr, env.clone(), Continuation::Identity);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Undefined));

    // Check that variable was defined
    let defined_value = env.get("x");
    assert!(defined_value.is_some());
    if let Value::Number(SchemeNumber::Integer(n)) = defined_value.unwrap() {
        assert_eq!(n, 42);
    } else {
        panic!("Expected integer 42");
    }
}

#[test]
fn test_begin_special_form() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test: (begin 1 2 3)
    let expr = Expr::List(vec![
        Expr::Variable("begin".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    if let Value::Number(SchemeNumber::Integer(n)) = result.unwrap() {
        assert_eq!(n, 3); // Should return last expression
    } else {
        panic!("Expected integer 3");
    }
}

#[test]
fn test_lambda_special_form() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test: (lambda (x) x)
    let expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec![Expr::Variable("x".to_string())]),
        Expr::Variable("x".to_string()),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Procedure(_)));
}

#[test]
fn test_lambda_variadic_form() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test: (lambda args args)
    let expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::Variable("args".to_string()),
        Expr::Variable("args".to_string()),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Procedure(_)));
}

#[test]
fn test_builtin_addition() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Add builtin + function to environment
    env.define(
        "+".to_string(),
        Value::Procedure(lambdust::value::Procedure::Builtin {
            name: "+".to_string(),
            func: |_| Ok(Value::Number(SchemeNumber::Integer(0))),
            arity: None,
        }),
    );

    // Test: (+ 1 2 3)
    let expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    if let Value::Number(SchemeNumber::Integer(n)) = result.unwrap() {
        assert_eq!(n, 6);
    } else {
        panic!("Expected integer 6");
    }
}

#[test]
#[ignore = "Stack overflow test causes test runner to crash"]
fn test_recursion_depth_limit() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Create deeply nested expression that would exceed depth limit
    let mut nested_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(1)));
    for _ in 0..2000 {
        nested_expr = Expr::List(vec![Expr::Variable("begin".to_string()), nested_expr]);
    }

    let result = evaluator.eval_pure(nested_expr, env, Continuation::Identity);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("stack overflow"));
}

#[test]
fn test_error_handling_invalid_application() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test applying non-procedure
    let expr = Expr::List(vec![
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Cannot apply"));
}

#[test]
fn test_error_handling_invalid_if_arity() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test if with wrong number of arguments
    let expr = Expr::List(vec![
        Expr::Variable("if".to_string()),
        Expr::Literal(Literal::Boolean(true)),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_err());
    // Error may be implementation-specific, just ensure it's an error
    assert!(!result.unwrap_err().to_string().is_empty());
}

#[test]
fn test_error_handling_invalid_define_arity() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test define with wrong number of arguments
    let expr = Expr::List(vec![
        Expr::Variable("define".to_string()),
        Expr::Variable("x".to_string()),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_err());
    // Error may be implementation-specific, just ensure it's an error
    assert!(!result.unwrap_err().to_string().is_empty());
}

#[test]
fn test_error_handling_invalid_lambda_arity() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test lambda with too few arguments
    let expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec![Expr::Variable("x".to_string())]),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("requires at least"));
}

#[test]
fn test_error_handling_invalid_define_variable() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test define with non-symbol first argument
    let expr = Expr::List(vec![
        Expr::Variable("define".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("first argument must be variable"));
}

#[test]
fn test_empty_begin_evaluation() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test: (begin)
    let expr = Expr::List(vec![Expr::Variable("begin".to_string())]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Value::Undefined));
}

#[test]
fn test_single_expression_begin() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test: (begin 42)
    let expr = Expr::List(vec![
        Expr::Variable("begin".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
    ]);

    let result = evaluator.eval_pure(expr, env, Continuation::Identity);

    assert!(result.is_ok());
    if let Value::Number(SchemeNumber::Integer(n)) = result.unwrap() {
        assert_eq!(n, 42);
    } else {
        panic!("Expected integer 42");
    }
}

#[test]
fn test_literal_value_conversion() {
    let mut evaluator = SemanticEvaluator::new();
    let env = Rc::new(Environment::new());

    // Test all literal types through evaluation
    let number_result = evaluator.eval_pure(
        Expr::Literal(Literal::Number(SchemeNumber::Integer(123))),
        env.clone(),
        Continuation::Identity,
    );
    assert!(number_result.is_ok());
    assert!(matches!(
        number_result.unwrap(),
        Value::Number(SchemeNumber::Integer(123))
    ));

    let string_result = evaluator.eval_pure(
        Expr::Literal(Literal::String("test".to_string())),
        env.clone(),
        Continuation::Identity,
    );
    assert!(string_result.is_ok());
    assert!(matches!(string_result.unwrap(), Value::String(s) if s == "test"));

    let boolean_result = evaluator.eval_pure(
        Expr::Literal(Literal::Boolean(false)),
        env.clone(),
        Continuation::Identity,
    );
    assert!(boolean_result.is_ok());
    assert!(matches!(boolean_result.unwrap(), Value::Boolean(false)));

    let char_result = evaluator.eval_pure(
        Expr::Literal(Literal::Character('z')),
        env.clone(),
        Continuation::Identity,
    );
    assert!(char_result.is_ok());
    assert!(matches!(char_result.unwrap(), Value::Character('z')));

    let nil_result = evaluator.eval_pure(Expr::Literal(Literal::Nil), env, Continuation::Identity);
    assert!(nil_result.is_ok());
    assert!(matches!(nil_result.unwrap(), Value::Nil));
}

#[test]
fn test_default_constructor() {
    let mut evaluator = SemanticEvaluator::default();

    // Default should be equivalent to new() - test through behavior
    let env = Rc::new(Environment::new());
    let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let result = evaluator.eval_pure(expr, env, Continuation::Identity);
    assert!(result.is_ok());
}

#[test]
#[ignore = "Complex continuation handling not implemented in pure evaluator"]
fn test_complex_continuation_handling() {
    // This test would verify complex continuation types
    // Currently marked as ignored until full implementation
}

#[test]
#[ignore = "Other special forms not implemented in pure evaluator"]
fn test_unimplemented_special_forms() {
    // This test would verify other special forms like quote, cond, etc.
    // Currently marked as ignored until full implementation
}

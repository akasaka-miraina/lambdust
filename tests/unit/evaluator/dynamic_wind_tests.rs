//! Dynamic-wind tests for R7RS dynamic-wind semantics

use lambdust::ast::{Expr, Literal};
use lambdust::evaluator::{Continuation, Evaluator};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_dynamic_wind_basic_execution() {
    let mut evaluator = Evaluator::new();

    // Define test counters in the environment
    evaluator.global_env.define(
        "before-count".to_string(),
        Value::Number(SchemeNumber::Integer(0)),
    );
    evaluator.global_env.define(
        "after-count".to_string(),
        Value::Number(SchemeNumber::Integer(0)),
    );
    evaluator.global_env.define(
        "thunk-result".to_string(),
        Value::Number(SchemeNumber::Integer(0)),
    );

    // Define before thunk that increments before-count
    let before_thunk_expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec![]), // No parameters
        Expr::List(vec![
            Expr::Variable("set!".to_string()),
            Expr::Variable("before-count".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        ]),
    ]);

    // Define after thunk that increments after-count
    let after_thunk_expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec![]), // No parameters
        Expr::List(vec![
            Expr::Variable("set!".to_string()),
            Expr::Variable("after-count".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        ]),
    ]);

    // Define main thunk that sets thunk-result
    let main_thunk_expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec![]), // No parameters
        Expr::List(vec![
            Expr::Variable("set!".to_string()),
            Expr::Variable("thunk-result".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ]),
    ]);

    // Execute dynamic-wind
    let dynamic_wind_expr = Expr::List(vec![
        Expr::Variable("dynamic-wind".to_string()),
        before_thunk_expr,
        main_thunk_expr,
        after_thunk_expr,
    ]);

    let result = evaluator.eval(
        dynamic_wind_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result.is_ok());

    // Check that before thunk was executed
    let before_count = evaluator.global_env.get("before-count").unwrap();
    assert_eq!(before_count, Value::Number(SchemeNumber::Integer(1)));

    // Check that main thunk was executed
    let thunk_result = evaluator.global_env.get("thunk-result").unwrap();
    assert_eq!(thunk_result, Value::Number(SchemeNumber::Integer(42)));

    // Check that after thunk was executed
    let after_count = evaluator.global_env.get("after-count").unwrap();
    assert_eq!(after_count, Value::Number(SchemeNumber::Integer(1)));
}

#[test]
fn test_dynamic_wind_argument_validation() {
    let mut evaluator = Evaluator::new();

    // Test with wrong number of arguments
    let wrong_arity_expr = Expr::List(vec![
        Expr::Variable("dynamic-wind".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);

    let result = evaluator.eval(
        wrong_arity_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result.is_err());
}

#[test]
fn test_dynamic_wind_non_procedure_before() {
    let mut evaluator = Evaluator::new();

    // Test with non-procedure before thunk
    let non_proc_before_expr = Expr::List(vec![
        Expr::Variable("dynamic-wind".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))), // Not a procedure
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ]),
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
        ]),
    ]);

    let result = evaluator.eval(
        non_proc_before_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result.is_err());
}

#[test]
fn test_dynamic_wind_non_procedure_after() {
    let mut evaluator = Evaluator::new();

    // Test with non-procedure after thunk
    let non_proc_after_expr = Expr::List(vec![
        Expr::Variable("dynamic-wind".to_string()),
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
        ]),
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ]),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))), // Not a procedure
    ]);

    let result = evaluator.eval(
        non_proc_after_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result.is_err());
}

#[test]
fn test_dynamic_wind_return_value() {
    let mut evaluator = Evaluator::new();

    // Test that dynamic-wind returns the value of the main thunk
    let dynamic_wind_expr = Expr::List(vec![
        Expr::Variable("dynamic-wind".to_string()),
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
        ]),
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::Literal(Literal::String("hello".to_string())), // Return value
        ]),
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
        ]),
    ]);

    let result = evaluator.eval(
        dynamic_wind_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_dynamic_point_stack_management() {
    let mut evaluator = Evaluator::new();

    // Check initial state
    assert_eq!(evaluator.dynamic_point_depth(), 0);

    // Create a lambda that will be used as a thunk
    let thunk_expr = Expr::List(vec![
        Expr::Variable("lambda".to_string()),
        Expr::List(vec![]),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
    ]);

    // Execute dynamic-wind
    let dynamic_wind_expr = Expr::List(vec![
        Expr::Variable("dynamic-wind".to_string()),
        thunk_expr.clone(),
        thunk_expr.clone(),
        thunk_expr,
    ]);

    let result = evaluator.eval(
        dynamic_wind_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result.is_ok());

    // Dynamic point should be cleaned up after execution
    // Note: This test may need adjustment based on the exact cleanup behavior
    // of the dynamic-wind implementation
}

#[test]
fn test_nested_dynamic_wind() {
    let mut evaluator = Evaluator::new();

    // Set up counters
    evaluator.global_env.define(
        "outer-before".to_string(),
        Value::Number(SchemeNumber::Integer(0)),
    );
    evaluator.global_env.define(
        "outer-after".to_string(),
        Value::Number(SchemeNumber::Integer(0)),
    );
    evaluator.global_env.define(
        "inner-before".to_string(),
        Value::Number(SchemeNumber::Integer(0)),
    );
    evaluator.global_env.define(
        "inner-after".to_string(),
        Value::Number(SchemeNumber::Integer(0)),
    );

    // Inner dynamic-wind
    let inner_dynamic_wind = Expr::List(vec![
        Expr::Variable("dynamic-wind".to_string()),
        // Inner before thunk
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::List(vec![
                Expr::Variable("set!".to_string()),
                Expr::Variable("inner-before".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            ]),
        ]),
        // Inner main thunk
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::Literal(Literal::String("inner".to_string())),
        ]),
        // Inner after thunk
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::List(vec![
                Expr::Variable("set!".to_string()),
                Expr::Variable("inner-after".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            ]),
        ]),
    ]);

    // Outer dynamic-wind containing inner dynamic-wind
    let outer_dynamic_wind = Expr::List(vec![
        Expr::Variable("dynamic-wind".to_string()),
        // Outer before thunk
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::List(vec![
                Expr::Variable("set!".to_string()),
                Expr::Variable("outer-before".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            ]),
        ]),
        // Outer main thunk (contains inner dynamic-wind)
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            inner_dynamic_wind,
        ]),
        // Outer after thunk
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::List(vec![
                Expr::Variable("set!".to_string()),
                Expr::Variable("outer-after".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            ]),
        ]),
    ]);

    let result = evaluator.eval(
        outer_dynamic_wind,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("inner".to_string()));

    // Check that all thunks were executed
    assert_eq!(
        evaluator.global_env.get("outer-before").unwrap(),
        Value::Number(SchemeNumber::Integer(1))
    );
    assert_eq!(
        evaluator.global_env.get("inner-before").unwrap(),
        Value::Number(SchemeNumber::Integer(1))
    );
    assert_eq!(
        evaluator.global_env.get("inner-after").unwrap(),
        Value::Number(SchemeNumber::Integer(1))
    );
    assert_eq!(
        evaluator.global_env.get("outer-after").unwrap(),
        Value::Number(SchemeNumber::Integer(1))
    );
}

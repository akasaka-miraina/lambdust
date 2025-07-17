//! Basic evaluation tests

use lambdust::{
    environment::Environment,
    evaluator::Evaluator,
    value::Value,
    parser::parse,
};
use std::sync::Arc;

#[test]
fn test_basic_arithmetic() {
    let env = Arc::new(Environment::new());
    let evaluator = Evaluator::new();
    
    // Test basic addition
    let expr = parse("(+ 1 2)").unwrap();
    let result = evaluator.eval(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_variable_definition() {
    let env = Arc::new(Environment::new());
    let evaluator = Evaluator::new();
    
    // Test variable definition
    let expr = parse("(define x 42)").unwrap();
    let _ = evaluator.eval(&expr, &env).unwrap();
    
    // Test variable lookup
    let expr = parse("x").unwrap();
    let result = evaluator.eval(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_lambda_function() {
    let env = Arc::new(Environment::new());
    let evaluator = Evaluator::new();
    
    // Test lambda definition and application
    let expr = parse("((lambda (x) (+ x 1)) 5)").unwrap();
    let result = evaluator.eval(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(6.0));
}
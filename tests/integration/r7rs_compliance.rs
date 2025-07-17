//! R7RS compliance tests

use lambdust::{
    environment::Environment,
    evaluator::Evaluator,
    parser::parse,
};
use std::sync::Arc;

#[test]
fn test_r7rs_basic_arithmetic() {
    let env = Arc::new(Environment::new());
    let evaluator = Evaluator::new();
    
    // Test R7RS arithmetic operations
    let test_cases = vec![
        ("(+ 1 2 3)", 6.0),
        ("(- 10 3)", 7.0),
        ("(* 2 3 4)", 24.0),
        ("(/ 12 3)", 4.0),
    ];
    
    for (expr_str, expected) in test_cases {
        let expr = parse(expr_str).unwrap();
        let result = evaluator.eval(&expr, &env).unwrap();
        assert_eq!(result.as_number().unwrap(), expected);
    }
}

#[test]
fn test_r7rs_conditionals() {
    let env = Arc::new(Environment::new());
    let evaluator = Evaluator::new();
    
    // Test if expressions
    let expr = parse("(if #t 1 2)").unwrap();
    let result = evaluator.eval(&expr, &env).unwrap();
    assert_eq!(result.as_number().unwrap(), 1.0);
    
    let expr = parse("(if #f 1 2)").unwrap();
    let result = evaluator.eval(&expr, &env).unwrap();
    assert_eq!(result.as_number().unwrap(), 2.0);
}

#[test]
fn test_r7rs_lambda() {
    let env = Arc::new(Environment::new());
    let evaluator = Evaluator::new();
    
    // Test lambda expressions
    let expr = parse("((lambda (x y) (+ x y)) 3 4)").unwrap();
    let result = evaluator.eval(&expr, &env).unwrap();
    assert_eq!(result.as_number().unwrap(), 7.0);
}
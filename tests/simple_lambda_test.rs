//! Simple lambda function test

use lambdust::value::Value;
use lambdust::Interpreter;

#[test]
fn test_simple_lambda_addition() {
    let mut interpreter = Interpreter::new();

    // Test simple lambda: ((lambda (a b) (+ a b)) 2 1)
    let result = interpreter.eval("((lambda (a b) (+ a b)) 2 1)").unwrap();

    // Should be 3
    match result {
        Value::Number(n) => {
            assert_eq!(n.to_string(), "3", "Expected 3, got {}", n);
        }
        other => panic!("Expected number 3, got {:?}", other),
    }
}

#[test]
fn test_lambda_with_three_args() {
    let mut interpreter = Interpreter::new();

    // Test lambda with 3 args: ((lambda (k v acc) (+ v acc)) "key" 2 1)
    let result = interpreter
        .eval("((lambda (k v acc) (+ v acc)) \"key\" 2 1)")
        .unwrap();

    // Should be 3
    match result {
        Value::Number(n) => {
            assert_eq!(n.to_string(), "3", "Expected 3, got {}", n);
        }
        other => panic!("Expected number 3, got {:?}", other),
    }
}

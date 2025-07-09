//! Test lambda arithmetic to isolate the problem

use lambdust::value::Value;
use lambdust::Interpreter;

#[test]
fn test_lambda_arithmetic_specific() {
    let mut interpreter = Interpreter::new();

    // Test the exact problematic case: (lambda (k v acc) (+ v acc)) with "a" 1 2
    let result = interpreter
        .eval("((lambda (k v acc) (+ v acc)) \"a\" 1 2)")
        .unwrap();

    // Should be 3
    match result {
        Value::Number(n) => {
            assert_eq!(n.to_string(), "3", "Expected 3, got {}", n);
        }
        other => panic!("Expected number 3, got {:?}", other),
    }
}

#[test]
fn test_direct_addition_in_lambda() {
    let mut interpreter = Interpreter::new();

    // Test direct numbers: ((lambda (a b) (+ a b)) 1 2)
    let result = interpreter.eval("((lambda (a b) (+ a b)) 1 2)").unwrap();

    // Should be 3
    match result {
        Value::Number(n) => {
            assert_eq!(n.to_string(), "3", "Expected 3, got {}", n);
        }
        other => panic!("Expected number 3, got {:?}", other),
    }
}

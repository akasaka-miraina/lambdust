//! Test for begin/define/variable reference fix

use lambdust::Interpreter;
use lambdust::value::Value;

#[test]
fn test_begin_define_variable_reference() {
    let mut interpreter = Interpreter::new();
    
    // Test the specific case that was failing: (begin (define y 100) y)
    let result = interpreter.eval("(begin (define y 100) y)").unwrap();
    
    match result {
        Value::Number(n) => {
            assert_eq!(n.to_string(), "100", "Expected 100, got {}", n);
        }
        other => panic!("Expected number 100, got {:?}", other),
    }
}

#[test]
fn test_begin_multiple_defines() {
    let mut interpreter = Interpreter::new();
    
    // Test multiple definitions in begin block
    let result = interpreter.eval("(begin (define x 10) (define y 20) (+ x y))").unwrap();
    
    match result {
        Value::Number(n) => {
            assert_eq!(n.to_string(), "30", "Expected 30, got {}", n);
        }
        other => panic!("Expected number 30, got {:?}", other),
    }
}

#[test]
fn test_nested_begin_with_defines() {
    let mut interpreter = Interpreter::new();
    
    // Test nested begin blocks with variable scoping
    let result = interpreter.eval("(begin (define a 5) (begin (define b 3) (+ a b)))").unwrap();
    
    match result {
        Value::Number(n) => {
            assert_eq!(n.to_string(), "8", "Expected 8, got {}", n);
        }
        other => panic!("Expected number 8, got {:?}", other),
    }
}
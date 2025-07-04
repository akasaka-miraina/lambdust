//! Unit tests for the library core functionality
//!
//! These tests verify the basic interpreter functionality exposed by the library.

use lambdust::{Interpreter, Value};

#[test]
fn test_basic_arithmetic() {
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval("(+ 1 2 3)").unwrap();
    assert_eq!(result, Value::from(6i64));
}

#[test]
fn test_define_and_call() {
    let mut interpreter = Interpreter::new();
    interpreter.eval("(define x 42)").unwrap();
    let result = interpreter.eval("x").unwrap();
    assert_eq!(result, Value::from(42i64));
}
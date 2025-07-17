//! Unit tests for built-in functions

use lambdust::builtins::arithmetic::{add, subtract, multiply, divide};
use lambdust::value::Value;

#[test]
fn test_arithmetic_add() {
    let args = vec![Value::Number(1.0), Value::Number(2.0)];
    let result = add(&args).unwrap();
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_arithmetic_subtract() {
    let args = vec![Value::Number(5.0), Value::Number(3.0)];
    let result = subtract(&args).unwrap();
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_arithmetic_multiply() {
    let args = vec![Value::Number(3.0), Value::Number(4.0)];
    let result = multiply(&args).unwrap();
    assert_eq!(result, Value::Number(12.0));
}

#[test]
fn test_arithmetic_divide() {
    let args = vec![Value::Number(10.0), Value::Number(2.0)];
    let result = divide(&args).unwrap();
    assert_eq!(result, Value::Number(5.0));
}
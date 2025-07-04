//! Unit tests for SRFI 13: String Libraries implementation

use lambdust::builtins::srfi_13::*;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;
use lambdust::interpreter::LambdustInterpreter;

#[test]
fn test_string_null() {
    let mut interpreter = LambdustInterpreter::new();
    
    let result = interpreter.eval_string("(string-null? \"\")").unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = interpreter.eval_string("(string-null? \"hello\")").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_string_hash() {
    let result = string_hash(&[Value::String("hello".to_string())]).unwrap();
    assert!(matches!(result, Value::Number(_)));

    let result = string_hash(&[
        Value::String("hello".to_string()),
        Value::Number(SchemeNumber::Integer(1000)),
    ])
    .unwrap();
    assert!(matches!(result, Value::Number(_)));
}

#[test]
fn test_string_prefix() {
    let mut interpreter = LambdustInterpreter::new();
    
    let result = interpreter.eval_string("(string-prefix? \"hel\" \"hello\")").unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = interpreter.eval_string("(string-prefix? \"world\" \"hello\")").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_string_suffix() {
    let mut interpreter = LambdustInterpreter::new();
    
    let result = interpreter.eval_string("(string-suffix? \"llo\" \"hello\")")
    .unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = interpreter.eval_string("(string-suffix? \"world\" \"hello\")").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_string_contains() {
    let result = string_contains(&[
        Value::String("hello".to_string()),
        Value::String("ell".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));

    let result = string_contains(&[
        Value::String("hello".to_string()),
        Value::String("world".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_string_take() {
    let result = string_take(&[
        Value::String("hello".to_string()),
        Value::Number(SchemeNumber::Integer(3)),
    ])
    .unwrap();
    assert_eq!(result, Value::String("hel".to_string()));
}

#[test]
fn test_string_drop() {
    let result = string_drop(&[
        Value::String("hello".to_string()),
        Value::Number(SchemeNumber::Integer(2)),
    ])
    .unwrap();
    assert_eq!(result, Value::String("llo".to_string()));
}

#[test]
fn test_string_take_right() {
    let result = string_take_right(&[
        Value::String("hello".to_string()),
        Value::Number(SchemeNumber::Integer(3)),
    ])
    .unwrap();
    assert_eq!(result, Value::String("llo".to_string()));
}

#[test]
fn test_string_drop_right() {
    let result = string_drop_right(&[
        Value::String("hello".to_string()),
        Value::Number(SchemeNumber::Integer(2)),
    ])
    .unwrap();
    assert_eq!(result, Value::String("hel".to_string()));
}

#[test]
fn test_string_concatenate() {
    let list = Value::from_vector(vec![
        Value::String("hello".to_string()),
        Value::String(" ".to_string()),
        Value::String("world".to_string()),
    ]);

    let result = string_concatenate(&[list]).unwrap();
    assert_eq!(result, Value::String("hello world".to_string()));
}

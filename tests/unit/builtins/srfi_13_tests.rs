//! Unit tests for SRFI 13: String Libraries implementation

use lambdust::builtins::srfi_13::*;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_string_null() {
    let result = string_null(&[Value::String("".to_string())]).unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = string_null(&[Value::String("hello".to_string())]).unwrap();
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
    let result = string_prefix(&[
        Value::String("hel".to_string()),
        Value::String("hello".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = string_prefix(&[
        Value::String("world".to_string()),
        Value::String("hello".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_string_suffix() {
    let result = string_suffix(&[
        Value::String("llo".to_string()),
        Value::String("hello".to_string()),
    ])
    .unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = string_suffix(&[
        Value::String("world".to_string()),
        Value::String("hello".to_string()),
    ])
    .unwrap();
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
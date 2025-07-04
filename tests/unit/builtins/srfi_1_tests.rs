//! Unit tests for SRFI 1: List Library implementation

use lambdust::builtins::srfi_1::*;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_take() {
    let list = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
        Value::Number(SchemeNumber::Integer(4)),
    ]);
    let n = Value::Number(SchemeNumber::Integer(2));

    let result = take(&[list, n]).unwrap();
    let expected = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
    ]);

    assert_eq!(result, expected);
}

#[test]
fn test_drop() {
    let list = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
        Value::Number(SchemeNumber::Integer(4)),
    ]);
    let n = Value::Number(SchemeNumber::Integer(2));

    let result = drop(&[list, n]).unwrap();
    let expected = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(3)),
        Value::Number(SchemeNumber::Integer(4)),
    ]);

    assert_eq!(result, expected);
}

#[test]
fn test_concatenate() {
    let list1 = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
    ]);
    let list2 = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(3)),
        Value::Number(SchemeNumber::Integer(4)),
    ]);

    let result = concatenate(&[list1, list2]).unwrap();
    let expected = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
        Value::Number(SchemeNumber::Integer(4)),
    ]);

    assert_eq!(result, expected);
}

#[test]
fn test_delete_duplicates() {
    let list = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(3)),
        Value::Number(SchemeNumber::Integer(2)),
    ]);

    let result = delete_duplicates(&[list]).unwrap();
    let expected = Value::from_vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
    ]);

    assert_eq!(result, expected);
}
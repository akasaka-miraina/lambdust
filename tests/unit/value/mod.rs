//! Unit tests for value types

use lambdust::value::{Value, Pair};

#[test]
fn test_value_equality() {
    assert_eq!(Value::Number(42.0), Value::Number(42.0));
    assert_eq!(Value::String("hello".to_string()), Value::String("hello".to_string()));
    assert_eq!(Value::Boolean(true), Value::Boolean(true));
    assert_eq!(Value::Nil, Value::Nil);
}

#[test]
fn test_pair_creation() {
    let pair = Pair::new(Value::Number(1.0), Value::Number(2.0));
    assert_eq!(pair.car(), &Value::Number(1.0));
    assert_eq!(pair.cdr(), &Value::Number(2.0));
}

#[test]
fn test_list_creation() {
    let values = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    let list = Value::from_vec(values);
    
    match list {
        Value::Pair(pair) => {
            assert_eq!(pair.car(), &Value::Number(1.0));
        }
        _ => panic!("Expected a pair"),
    }
}
//! Unit tests for SRFI 69: Basic Hash Tables implementation

use lambdust::builtins::srfi_69::*;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_hash_table_creation() {
    let result = make_hash_table(&[]).unwrap();
    assert!(matches!(result, Value::HashTable(_)));
}

#[test]
fn test_hash_table_predicate() {
    let ht = make_hash_table(&[]).unwrap();
    let result = hash_table_predicate(&[ht]).unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result =
        hash_table_predicate(&[Value::String("not a hash table".to_string())]).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_hash_table_set_and_ref() {
    let ht = make_hash_table(&[]).unwrap();
    let key = Value::String("test-key".to_string());
    let value = Value::Number(SchemeNumber::Integer(42));

    // Set value
    let result = hash_table_set(&[ht.clone(), key.clone(), value.clone()]);
    assert!(result.is_ok());

    // Get value
    let result = hash_table_ref(&[ht, key]).unwrap();
    assert_eq!(result, value);
}

#[test]
fn test_hash_table_size() {
    let ht = make_hash_table(&[]).unwrap();

    // Initially empty
    let result = hash_table_size(&[ht.clone()]).unwrap();
    assert_eq!(result, Value::Number(SchemeNumber::Integer(0)));

    // Add one item
    let key = Value::String("test".to_string());
    let value = Value::Number(SchemeNumber::Integer(123));
    hash_table_set(&[ht.clone(), key, value]).unwrap();

    let result = hash_table_size(&[ht]).unwrap();
    assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));
}

#[test]
fn test_hash_table_exists() {
    let ht = make_hash_table(&[]).unwrap();
    let key = Value::String("test-key".to_string());
    let value = Value::Number(SchemeNumber::Integer(42));

    // Key doesn't exist initially
    let result = hash_table_exists(&[ht.clone(), key.clone()]).unwrap();
    assert_eq!(result, Value::Boolean(false));

    // Set value
    hash_table_set(&[ht.clone(), key.clone(), value]).unwrap();

    // Key now exists
    let result = hash_table_exists(&[ht, key]).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_hash_table_delete() {
    let ht = make_hash_table(&[]).unwrap();
    let key = Value::String("test-key".to_string());
    let value = Value::Number(SchemeNumber::Integer(42));

    // Set value
    hash_table_set(&[ht.clone(), key.clone(), value]).unwrap();

    // Delete value
    let result = hash_table_delete(&[ht.clone(), key.clone()]).unwrap();
    assert_eq!(result, Value::Boolean(true));

    // Key no longer exists
    let result = hash_table_exists(&[ht, key]).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_hash_table_keys_and_values() {
    let ht = make_hash_table(&[]).unwrap();
    let key1 = Value::String("key1".to_string());
    let key2 = Value::String("key2".to_string());
    let value1 = Value::Number(SchemeNumber::Integer(1));
    let value2 = Value::Number(SchemeNumber::Integer(2));

    // Add items
    hash_table_set(&[ht.clone(), key1, value1]).unwrap();
    hash_table_set(&[ht.clone(), key2, value2]).unwrap();

    // Get keys
    let keys = hash_table_keys(&[ht.clone()]).unwrap();
    assert!(keys.is_list());

    // Get values
    let values = hash_table_values(&[ht]).unwrap();
    assert!(values.is_list());
}

#[test]
fn test_hash_table_copy() {
    let ht = make_hash_table(&[]).unwrap();
    let key = Value::String("test-key".to_string());
    let value = Value::Number(SchemeNumber::Integer(42));

    // Set value in original
    hash_table_set(&[ht.clone(), key.clone(), value.clone()]).unwrap();

    // Copy hash table
    let copy = hash_table_copy(&[ht]).unwrap();

    // Value should exist in copy
    let result = hash_table_ref(&[copy, key]).unwrap();
    assert_eq!(result, value);
}

#[test]
fn test_hash_value() {
    let value = Value::String("test".to_string());
    let result = hash_value(&[value]).unwrap();
    assert!(matches!(result, Value::Number(_)));

    let value = Value::String("test".to_string());
    let bound = Value::Number(SchemeNumber::Integer(1000));
    let result = hash_value(&[value, bound]).unwrap();
    assert!(matches!(result, Value::Number(_)));
}

#[test]
fn test_string_hash() {
    let string = Value::String("hello".to_string());
    let result = string_hash_impl(&[string]).unwrap();
    assert!(matches!(result, Value::Number(_)));
}

#[test]
fn test_string_ci_hash() {
    let string1 = Value::String("Hello".to_string());
    let string2 = Value::String("HELLO".to_string());

    let hash1 = string_ci_hash_impl(&[string1]).unwrap();
    let hash2 = string_ci_hash_impl(&[string2]).unwrap();

    // Case-insensitive hashes should be equal
    assert_eq!(hash1, hash2);
}
//! Unit tests for SRFI 69: Basic Hash Tables implementation

// Individual functions are no longer public - use interpreter integration
use lambdust::interpreter::LambdustInterpreter;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_hash_table_creation() {
    let mut interpreter = LambdustInterpreter::new();

    let result = interpreter.eval_string("(make-hash-table)").unwrap();
    assert!(matches!(result, Value::HashTable(_)));
}

#[test]
fn test_hash_table_predicate() {
    let mut interpreter = LambdustInterpreter::new();

    let result = interpreter
        .eval_string("(hash-table? (make-hash-table))")
        .unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = interpreter
        .eval_string("(hash-table? \"not a hash table\")")
        .unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_hash_table_set_and_ref() {
    let mut interpreter = LambdustInterpreter::new();

    // Create hash table and set value
    interpreter
        .eval_string("(define ht (make-hash-table))")
        .unwrap();
    interpreter
        .eval_string("(hash-table-set! ht \"test-key\" 42)")
        .unwrap();

    // Get value
    let result = interpreter
        .eval_string("(hash-table-ref ht \"test-key\")")
        .unwrap();
    assert_eq!(result, Value::Number(SchemeNumber::Integer(42)));
}

#[test]
fn test_hash_table_size() {
    let mut interpreter = LambdustInterpreter::new();

    // Create hash table
    interpreter
        .eval_string("(define ht (make-hash-table))")
        .unwrap();

    // Initially empty
    let result = interpreter.eval_string("(hash-table-size ht)").unwrap();
    assert_eq!(result, Value::Number(SchemeNumber::Integer(0)));

    // Add one item
    interpreter
        .eval_string("(hash-table-set! ht \"test\" 123)")
        .unwrap();

    let result = interpreter.eval_string("(hash-table-size ht)").unwrap();
    assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));
}

#[test]
fn test_hash_table_exists() {
    let mut interpreter = LambdustInterpreter::new();

    // Create hash table
    interpreter
        .eval_string("(define ht (make-hash-table))")
        .unwrap();

    // Key doesn't exist initially
    let result = interpreter
        .eval_string("(hash-table-exists? ht \"test-key\")")
        .unwrap();
    assert_eq!(result, Value::Boolean(false));

    // Set value
    interpreter
        .eval_string("(hash-table-set! ht \"test-key\" 42)")
        .unwrap();

    // Key now exists
    let result = interpreter
        .eval_string("(hash-table-exists? ht \"test-key\")")
        .unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_hash_table_delete() {
    let mut interpreter = LambdustInterpreter::new();

    // Create hash table and set value
    interpreter
        .eval_string("(define ht (make-hash-table))")
        .unwrap();
    interpreter
        .eval_string("(hash-table-set! ht \"test-key\" 42)")
        .unwrap();

    // Delete value
    let result = interpreter
        .eval_string("(hash-table-delete! ht \"test-key\")")
        .unwrap();
    assert_eq!(result, Value::Boolean(true));

    // Key no longer exists
    let result = interpreter
        .eval_string("(hash-table-exists? ht \"test-key\")")
        .unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_hash_table_keys_and_values() {
    let mut interpreter = LambdustInterpreter::new();

    // Create hash table and add items
    interpreter
        .eval_string("(define ht (make-hash-table))")
        .unwrap();
    interpreter
        .eval_string("(hash-table-set! ht \"key1\" 1)")
        .unwrap();
    interpreter
        .eval_string("(hash-table-set! ht \"key2\" 2)")
        .unwrap();

    // Get keys
    let keys = interpreter.eval_string("(hash-table-keys ht)").unwrap();
    assert!(keys.is_list());

    // Get values
    let values = interpreter.eval_string("(hash-table-values ht)").unwrap();
    assert!(values.is_list());
}

#[test]
fn test_hash_table_copy() {
    let mut interpreter = LambdustInterpreter::new();

    // Create hash table and set value
    interpreter
        .eval_string("(define ht (make-hash-table))")
        .unwrap();
    interpreter
        .eval_string("(hash-table-set! ht \"test-key\" 42)")
        .unwrap();

    // Copy hash table
    interpreter
        .eval_string("(define copy (hash-table-copy ht))")
        .unwrap();

    // Value should exist in copy
    let result = interpreter
        .eval_string("(hash-table-ref copy \"test-key\")")
        .unwrap();
    assert_eq!(result, Value::Number(SchemeNumber::Integer(42)));
}

#[test]
fn test_hash_value() {
    let mut interpreter = LambdustInterpreter::new();

    let result = interpreter.eval_string("(hash \"test\")").unwrap();
    assert!(matches!(result, Value::Number(_)));

    let result = interpreter.eval_string("(hash \"test\" 1000)").unwrap();
    assert!(matches!(result, Value::Number(_)));
}

#[test]
fn test_string_hash_impl() {
    let mut interpreter = LambdustInterpreter::new();

    let result = interpreter.eval_string("(string-hash \"hello\")").unwrap();
    assert!(matches!(result, Value::Number(_)));
}

#[test]
fn test_string_ci_hash() {
    let mut interpreter = LambdustInterpreter::new();

    let hash1 = interpreter
        .eval_string("(string-ci-hash \"Hello\")")
        .unwrap();
    let hash2 = interpreter
        .eval_string("(string-ci-hash \"HELLO\")")
        .unwrap();

    // Case-insensitive hashes should be equal
    assert_eq!(hash1, hash2);
}

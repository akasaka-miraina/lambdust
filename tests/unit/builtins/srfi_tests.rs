//! Unit tests for SRFI built-in functions

use lambdust::builtins::srfi::{
    extract_integer_from_number, srfi_available_function, srfi_name_function, srfi_parts_function,
    srfi_supported_ids_function,
};
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Procedure, Value};

#[test]
fn test_srfi_available_function() {
    let func = srfi_available_function();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
        // Test with supported SRFI
        let result = func(&[Value::Number(SchemeNumber::Integer(9))]).unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Test with unsupported SRFI
        let result = func(&[Value::Number(SchemeNumber::Integer(999))]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }
}

#[test]
fn test_srfi_supported_ids_function() {
    let func = srfi_supported_ids_function();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
        let result = func(&[]).unwrap();
        println!("Result: {:?}", result);
        assert!(matches!(result, Value::Vector(_)));

        if let Value::Vector(ids) = result {
            assert!(ids.len() >= 4); // Should have at least SRFIs 9, 45, 46, 97
        }
    }
}

#[test]
fn test_srfi_name_function() {
    let func = srfi_name_function();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
        let result = func(&[Value::Number(SchemeNumber::Integer(9))]).unwrap();
        assert_eq!(result, Value::String("Defining Record Types".to_string()));

        // Test error case
        let result = func(&[Value::Number(SchemeNumber::Integer(999))]);
        assert!(result.is_err());
    }
}

#[test]
fn test_srfi_parts_function() {
    let func = srfi_parts_function();
    if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
        let result = func(&[Value::Number(SchemeNumber::Integer(9))]).unwrap();
        assert!(matches!(result, Value::Vector(_)));

        if let Value::Vector(parts) = result {
            assert!(!parts.is_empty()); // Should have parts
        }
    }
}

#[test]
fn test_extract_integer_from_number() {
    assert_eq!(
        extract_integer_from_number(&SchemeNumber::Integer(42)).unwrap(),
        42
    );
    assert_eq!(
        extract_integer_from_number(&SchemeNumber::Real(5.0)).unwrap(),
        5
    );
    assert!(extract_integer_from_number(&SchemeNumber::Integer(-1)).is_err());
    assert!(extract_integer_from_number(&SchemeNumber::Real(3.5)).is_err());
}

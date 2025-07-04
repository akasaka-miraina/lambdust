//! Unit tests for host function functionality
//!
//! This module contains tests for host function registration
//! and management capabilities.

use lambdust::error::LambdustError;
use lambdust::host::{FunctionSignature, HostFunctionRegistry, ValueType};
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Procedure, Value};

#[test]
fn test_value_type_matching() {
    assert!(ValueType::Boolean.matches(&Value::Boolean(true)));
    assert!(ValueType::Number.matches(&Value::Number(SchemeNumber::Integer(42))));
    assert!(ValueType::String.matches(&Value::String("test".to_string())));
    assert!(ValueType::Any.matches(&Value::Boolean(true)));
    assert!(!ValueType::Boolean.matches(&Value::String("test".to_string())));
}

#[test]
fn test_function_signature_validation() {
    let sig = FunctionSignature::new(
        vec![ValueType::Number, ValueType::String],
        ValueType::Boolean,
    );

    // Valid arguments
    let valid_args = vec![
        Value::Number(SchemeNumber::Integer(42)),
        Value::String("test".to_string()),
    ];
    assert!(sig.validate_args(&valid_args).is_ok());

    // Invalid arity
    let invalid_arity = vec![Value::Number(SchemeNumber::Integer(42))];
    assert!(sig.validate_args(&invalid_arity).is_err());

    // Invalid type
    let invalid_type = vec![Value::Boolean(true), Value::String("test".to_string())];
    assert!(sig.validate_args(&invalid_type).is_err());
}

#[test]
fn test_host_function_registry() {
    let mut registry = HostFunctionRegistry::new();

    // Register a simple function
    registry.register_simple_function(
        "test-add".to_string(),
        |args: &[Value]| -> lambdust::Result<Value> {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            match (&args[0], &args[1]) {
                (
                    Value::Number(SchemeNumber::Integer(a)),
                    Value::Number(SchemeNumber::Integer(b)),
                ) => Ok(Value::Number(SchemeNumber::Integer(a + b))),
                _ => Err(LambdustError::type_error("Expected numbers".to_string())),
            }
        },
    );

    // Test function retrieval
    let proc = registry.get_procedure("test-add").unwrap();
    assert!(proc.is_procedure());

    // Test function listing
    let functions = registry.list_functions();
    assert!(functions.contains(&&"test-add".to_string()));
}

#[test]
fn test_builtin_functions() {
    let registry = HostFunctionRegistry::default();

    // Test that builtins are registered
    assert!(registry.get_procedure("host-print").is_some());
    assert!(registry.get_procedure("host-string-append").is_some());
    assert!(registry.get_procedure("host-length").is_some());
}

#[test]
fn test_typed_function_registration() {
    let mut registry = HostFunctionRegistry::new();

    // Register a typed function
    registry.register_typed_function("add-numbers".to_string(), |(a, b): (i64, i64)| -> i64 {
        a + b
    });

    let proc = registry.get_procedure("add-numbers").unwrap();
    assert!(proc.is_procedure());
}

#[test]
fn test_return_value_validation() {
    let mut registry = HostFunctionRegistry::new();

    // Register function with specific return type validation
    registry.register_function(
        "return-string".to_string(),
        |_args: &[Value]| -> lambdust::Result<Value> {
            Ok(Value::String("expected string".to_string()))
        },
        FunctionSignature::new(vec![], ValueType::String),
    );

    registry.register_function(
        "return-wrong-type".to_string(),
        |_args: &[Value]| -> lambdust::Result<Value> {
            // This should fail validation - returns number instead of string
            Ok(Value::Number(SchemeNumber::Integer(42)))
        },
        FunctionSignature::new(vec![], ValueType::String),
    );

    // Test correct return type
    let proc1 = registry.get_procedure("return-string").unwrap();
    if let Value::Procedure(Procedure::HostFunction { func, .. }) = proc1 {
        let result = func(&[]);
        assert!(result.is_ok());
    }

    // Test incorrect return type should fail validation
    let proc2 = registry.get_procedure("return-wrong-type").unwrap();
    if let Value::Procedure(Procedure::HostFunction { func, .. }) = proc2 {
        let result = func(&[]);
        assert!(result.is_err());
        match result.unwrap_err() {
            LambdustError::TypeError { message, .. } => {
                assert!(message.contains("Return value type mismatch"));
            }
            _ => panic!("Expected TypeError for return value mismatch"),
        }
    }
}

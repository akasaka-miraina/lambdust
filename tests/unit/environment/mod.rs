//! Environment Unit Tests Module
//!
//! This module contains comprehensive unit tests for the Environment system,
//! including mock-based tests for isolated component testing and legacy tests
//! for actual Environment implementations.

pub mod environment_mock_tests;

// Legacy environment tests (to be gradually replaced with mock-based tests)
use lambdust::environment::SharedEnvironment;
use lambdust::value::Value;

#[test]
fn test_basic_environment_creation() {
    let env = SharedEnvironment::new();
    assert!(env.get("undefined_var").is_none());
}

#[test]
fn test_basic_variable_binding() {
    let mut env = SharedEnvironment::new();
    let result = env.define("x".to_string(), Value::Number(42.into()));
    assert!(result.is_ok());
    
    assert_eq!(env.get("x"), Some(Value::Number(42.into())));
}
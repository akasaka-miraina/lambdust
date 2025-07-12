//! Integration tests for custom predicate built-in functions

use lambdust::{
    builtins::custom_predicates::register_custom_predicate_functions,
    value::{Value, Procedure, global_custom_predicate_registry},
    lexer::SchemeNumber,
};
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_register_custom_predicate_functions() {
    let mut builtins = HashMap::new();
    register_custom_predicate_functions(&mut builtins);
    
    // Check that all functions are registered
    let expected_functions = [
        "define-predicate",
        "remove-predicate", 
        "predicate-defined?",
        "list-predicates",
        "predicate-info",
        "clear-predicates",
        "apply-predicate",
    ];
    
    for func_name in &expected_functions {
        assert!(builtins.contains_key(*func_name), "Function {} not registered", func_name);
        match &builtins[*func_name] {
            Value::Procedure(_) => {}, // Expected
            _ => panic!("Function {} is not a procedure", func_name),
        }
    }
}

#[test]
fn test_predicate_defined_function() {
    // Clear global registry
    global_custom_predicate_registry().clear().unwrap();
    
    let mut builtins = HashMap::new();
    register_custom_predicate_functions(&mut builtins);
    
    let predicate_defined = &builtins["predicate-defined?"];
    
    // Test with string argument
    let result = call_builtin_procedure(predicate_defined, &[Value::String("nonexistent?".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
    
    // Register a predicate manually for testing
    global_custom_predicate_registry().register(
        "test-predicate?".to_string(),
        None,
        |value| value.is_number(),
    ).unwrap();
    
    let result = call_builtin_procedure(predicate_defined, &[Value::String("test-predicate?".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_predicate_defined_invalid_args() {
    let mut builtins = HashMap::new();
    register_custom_predicate_functions(&mut builtins);
    
    let predicate_defined = &builtins["predicate-defined?"];
    
    // Test with wrong number of arguments
    let result = call_builtin_procedure(predicate_defined, &[]);
    assert!(result.is_err());
    
    let result = call_builtin_procedure(predicate_defined, &[
        Value::String("test".to_string()),
        Value::String("extra".to_string())
    ]);
    assert!(result.is_err());
    
    // Test with invalid argument type
    let result = call_builtin_procedure(predicate_defined, &[Value::Number(SchemeNumber::Integer(42))]);
    assert!(result.is_err());
}

#[test]
fn test_remove_predicate_function() {
    // Clear global registry
    global_custom_predicate_registry().clear().unwrap();
    
    let mut builtins = HashMap::new();
    register_custom_predicate_functions(&mut builtins);
    
    let remove_predicate = &builtins["remove-predicate"];
    
    // Try to remove non-existent predicate
    let result = call_builtin_procedure(remove_predicate, &[Value::String("nonexistent?".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
    
    // Register and then remove a predicate
    global_custom_predicate_registry().register(
        "to-be-removed?".to_string(),
        None,
        |_| true,
    ).unwrap();
    
    let result = call_builtin_procedure(remove_predicate, &[Value::String("to-be-removed?".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
    
    // Verify it's actually removed
    assert!(!global_custom_predicate_registry().is_registered("to-be-removed?").unwrap());
}

#[test]
fn test_list_predicates_function() {
    // Clear global registry
    global_custom_predicate_registry().clear().unwrap();
    
    let mut builtins = HashMap::new();
    register_custom_predicate_functions(&mut builtins);
    
    let list_predicates = &builtins["list-predicates"];
    
    // Test with empty registry
    let result = call_builtin_procedure(list_predicates, &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Nil); // Empty list
    
    // Register some predicates
    global_custom_predicate_registry().register("pred1?".to_string(), None, |_| true).unwrap();
    global_custom_predicate_registry().register("pred2?".to_string(), None, |_| false).unwrap();
    
    let result = call_builtin_procedure(list_predicates, &[]);
    assert!(result.is_ok());
    
    // The result should be a list containing the predicate names
    // Since HashMap iteration order is not guaranteed, we need to extract and check
    let list_result = result.unwrap();
    let names = extract_string_list(&list_result);
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"pred1?".to_string()));
    assert!(names.contains(&"pred2?".to_string()));
}

#[test]
fn test_clear_predicates_function() {
    // Setup some predicates
    global_custom_predicate_registry().clear().unwrap();
    global_custom_predicate_registry().register("pred1?".to_string(), None, |_| true).unwrap();
    global_custom_predicate_registry().register("pred2?".to_string(), None, |_| false).unwrap();
    
    let mut builtins = HashMap::new();
    register_custom_predicate_functions(&mut builtins);
    
    let clear_predicates = &builtins["clear-predicates"];
    
    // Verify predicates exist
    assert!(!global_custom_predicate_registry().list_predicates().unwrap().is_empty());
    
    // Clear predicates
    let result = call_builtin_procedure(clear_predicates, &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
    
    // Verify predicates are cleared
    assert!(global_custom_predicate_registry().list_predicates().unwrap().is_empty());
}

#[test]
fn test_apply_predicate_function() {
    // Clear global registry and register a test predicate
    global_custom_predicate_registry().clear().unwrap();
    global_custom_predicate_registry().register(
        "test-number?".to_string(),
        None,
        |value| value.is_number(),
    ).unwrap();
    
    let mut builtins = HashMap::new();
    register_custom_predicate_functions(&mut builtins);
    
    let apply_predicate = &builtins["apply-predicate"];
    
    // Test with number (should return true)
    let result = call_builtin_procedure(apply_predicate, &[
        Value::String("test-number?".to_string()),
        Value::Number(SchemeNumber::Integer(42)),
    ]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
    
    // Test with string (should return false)
    let result = call_builtin_procedure(apply_predicate, &[
        Value::String("test-number?".to_string()),
        Value::String("not a number".to_string()),
    ]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
    
    // Test with non-existent predicate
    let result = call_builtin_procedure(apply_predicate, &[
        Value::String("nonexistent?".to_string()),
        Value::Number(SchemeNumber::Integer(42)),
    ]);
    assert!(result.is_err());
}

#[test]
fn test_apply_predicate_invalid_args() {
    let mut builtins = HashMap::new();
    register_custom_predicate_functions(&mut builtins);
    
    let apply_predicate = &builtins["apply-predicate"];
    
    // Test with wrong number of arguments
    let result = call_builtin_procedure(apply_predicate, &[Value::String("test?".to_string())]);
    assert!(result.is_err());
    
    // Test with invalid first argument
    let result = call_builtin_procedure(apply_predicate, &[
        Value::Number(SchemeNumber::Integer(42)),
        Value::String("value".to_string()),
    ]);
    assert!(result.is_err());
}

#[test]
fn test_predicate_info_function() {
    // Clear global registry and register a predicate with description
    global_custom_predicate_registry().clear().unwrap();
    global_custom_predicate_registry().register(
        "documented-pred?".to_string(),
        Some("A well-documented predicate".to_string()),
        |value| value.is_string(),
    ).unwrap();
    
    let mut builtins = HashMap::new();
    register_custom_predicate_functions(&mut builtins);
    
    let predicate_info = &builtins["predicate-info"];
    
    // Test with existing predicate
    let result = call_builtin_procedure(predicate_info, &[Value::String("documented-pred?".to_string())]);
    assert!(result.is_ok());
    
    // The result should be an association list with name and description
    let info_result = result.unwrap();
    assert!(!matches!(info_result, Value::Boolean(false))); // Should not be false
    
    // Test with non-existent predicate
    let result = call_builtin_procedure(predicate_info, &[Value::String("nonexistent?".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_define_predicate_invalid_args() {
    let mut builtins = HashMap::new();
    register_custom_predicate_functions(&mut builtins);
    
    let define_predicate = &builtins["define-predicate"];
    
    // Test with wrong number of arguments
    let result = call_builtin_procedure(define_predicate, &[Value::String("test?".to_string())]);
    assert!(result.is_err());
    
    // Test with invalid first argument (not string or symbol)
    let result = call_builtin_procedure(define_predicate, &[
        Value::Number(SchemeNumber::Integer(42)),
        Value::Boolean(true),
    ]);
    assert!(result.is_err());
    
    // Test with invalid second argument (not procedure)
    let result = call_builtin_procedure(define_predicate, &[
        Value::String("test?".to_string()),
        Value::String("not a procedure".to_string()),
    ]);
    assert!(result.is_err());
    
    // Test with invalid third argument (not string)
    let test_proc = Value::Procedure(Procedure::Builtin {
        name: "test".to_string(),
        arity: Some(1),
        function: Arc::new(|_| Ok(Value::Boolean(true))),
    });
    
    let result = call_builtin_procedure(define_predicate, &[
        Value::String("test?".to_string()),
        test_proc,
        Value::Number(SchemeNumber::Integer(42)), // Invalid description
    ]);
    assert!(result.is_err());
}

#[test] 
fn test_integration_with_symbols() {
    let mut builtins = HashMap::new();
    register_custom_predicate_functions(&mut builtins);
    
    let predicate_defined = &builtins["predicate-defined?"];
    let remove_predicate = &builtins["remove-predicate"];
    
    // Test with symbol arguments instead of strings
    let symbol_name = Value::Symbol("symbol-test?".to_string());
    
    // Register predicate manually
    global_custom_predicate_registry().register(
        "symbol-test?".to_string(),
        None,
        |value| value.is_symbol(),
    ).unwrap();
    
    // Test predicate-defined? with symbol
    let result = call_builtin_procedure(predicate_defined, &[symbol_name.clone()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
    
    // Test remove-predicate with symbol
    let result = call_builtin_procedure(remove_predicate, &[symbol_name]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

// Helper function to call a builtin procedure
fn call_builtin_procedure(procedure: &Value, args: &[Value]) -> Result<Value, lambdust::error::LambdustError> {
    match procedure {
        Value::Procedure(Procedure::Builtin { func, .. }) => {
            func(args)
        },
        _ => panic!("Expected builtin procedure"),
    }
}

// Helper function to extract strings from a Scheme list
fn extract_string_list(value: &Value) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = value;
    
    loop {
        match current {
            Value::Nil => break,
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                if let Value::String(s) = &pair.car {
                    result.push(s.clone());
                }
                current = &pair.cdr;
            },
            _ => break,
        }
    }
    
    result
}
//! Comprehensive test suite for custom type predicates system

use lambdust::{
    value::{
        Value, CustomPredicateRegistry, CustomPredicateInfo, CustomPredicateFn,
        global_custom_predicate_registry, register_global_custom_predicate, evaluate_global_custom_predicate,
    },
    lexer::SchemeNumber,
};
use std::sync::Arc;

#[test]
fn test_custom_predicate_registry_creation() {
    let registry = CustomPredicateRegistry::new();
    assert!(registry.list_predicates().unwrap().is_empty());
}

#[test]
fn test_custom_predicate_basic_registration() {
    let registry = CustomPredicateRegistry::new();
    
    // Register a simple number predicate
    let result = registry.register(
        "my-number?".to_string(),
        Some("Check if value is a number".to_string()),
        |value| value.is_number(),
    );
    
    assert!(result.is_ok());
    assert!(registry.is_registered("my-number?").unwrap());
    
    // Test the predicate
    let number = Value::Number(SchemeNumber::Integer(42));
    let string = Value::String("hello".to_string());
    
    assert_eq!(registry.evaluate("my-number?", &number).unwrap(), Some(true));
    assert_eq!(registry.evaluate("my-number?", &string).unwrap(), Some(false));
}

#[test]
fn test_custom_predicate_duplicate_registration() {
    let registry = CustomPredicateRegistry::new();
    
    // Register first predicate
    let result1 = registry.register(
        "duplicate-test?".to_string(),
        None,
        |_| true,
    );
    assert!(result1.is_ok());
    
    // Try to register the same name again
    let result2 = registry.register(
        "duplicate-test?".to_string(),
        None,
        |_| false,
    );
    assert!(result2.is_err());
    assert!(result2.unwrap_err().to_string().contains("already exists"));
}

#[test]
fn test_custom_predicate_unregister() {
    let registry = CustomPredicateRegistry::new();
    
    // Register a predicate
    registry.register("temp-predicate?".to_string(), None, |_| true).unwrap();
    assert!(registry.is_registered("temp-predicate?").unwrap());
    
    // Unregister it
    let removed = registry.unregister("temp-predicate?").unwrap();
    assert!(removed);
    assert!(!registry.is_registered("temp-predicate?").unwrap());
    
    // Try to unregister non-existent predicate
    let not_removed = registry.unregister("nonexistent?").unwrap();
    assert!(!not_removed);
}

#[test]
fn test_custom_predicate_info() {
    let registry = CustomPredicateRegistry::new();
    
    // Register predicate with description
    registry.register(
        "documented-predicate?".to_string(),
        Some("A well-documented predicate".to_string()),
        |value| value.is_string(),
    ).unwrap();
    
    // Get info
    let info = registry.get_info("documented-predicate?").unwrap();
    assert!(info.is_some());
    
    let info = info.unwrap();
    assert_eq!(info.name, "documented-predicate?");
    assert_eq!(info.description, Some("A well-documented predicate".to_string()));
    
    // Test for non-existent predicate
    let no_info = registry.get_info("nonexistent?").unwrap();
    assert!(no_info.is_none());
}

#[test]
fn test_custom_predicate_list_predicates() {
    let registry = CustomPredicateRegistry::new();
    
    // Start with empty list
    assert!(registry.list_predicates().unwrap().is_empty());
    
    // Register some predicates
    registry.register("pred1?".to_string(), None, |_| true).unwrap();
    registry.register("pred2?".to_string(), None, |_| false).unwrap();
    registry.register("pred3?".to_string(), None, |value| value.is_number()).unwrap();
    
    // Check list
    let mut predicates = registry.list_predicates().unwrap();
    predicates.sort(); // HashMap iteration order is not guaranteed
    assert_eq!(predicates, vec!["pred1?", "pred2?", "pred3?"]);
}

#[test]
fn test_custom_predicate_clear() {
    let registry = CustomPredicateRegistry::new();
    
    // Register some predicates
    registry.register("pred1?".to_string(), None, |_| true).unwrap();
    registry.register("pred2?".to_string(), None, |_| false).unwrap();
    
    assert!(!registry.list_predicates().unwrap().is_empty());
    
    // Clear all
    registry.clear().unwrap();
    assert!(registry.list_predicates().unwrap().is_empty());
}

#[test]
fn test_complex_custom_predicates() {
    let registry = CustomPredicateRegistry::new();
    
    // Register a complex predicate that checks for even numbers
    registry.register(
        "even-number?".to_string(),
        Some("Check if value is an even number".to_string()),
        |value| {
            match value {
                Value::Number(SchemeNumber::Integer(n)) => n % 2 == 0,
                Value::Number(SchemeNumber::Real(r)) => {
                    let rounded = r.round();
                    (r - rounded).abs() < f64::EPSILON && (rounded as i64) % 2 == 0
                },
                _ => false,
            }
        },
    ).unwrap();
    
    // Test with various values
    let even_int = Value::Number(SchemeNumber::Integer(42));
    let odd_int = Value::Number(SchemeNumber::Integer(43));
    let even_real = Value::Number(SchemeNumber::Real(44.0));
    let odd_real = Value::Number(SchemeNumber::Real(45.0));
    let non_int_real = Value::Number(SchemeNumber::Real(42.5));
    let string = Value::String("not a number".to_string());
    
    assert_eq!(registry.evaluate("even-number?", &even_int).unwrap(), Some(true));
    assert_eq!(registry.evaluate("even-number?", &odd_int).unwrap(), Some(false));
    assert_eq!(registry.evaluate("even-number?", &even_real).unwrap(), Some(true));
    assert_eq!(registry.evaluate("even-number?", &odd_real).unwrap(), Some(false));
    assert_eq!(registry.evaluate("even-number?", &non_int_real).unwrap(), Some(false));
    assert_eq!(registry.evaluate("even-number?", &string).unwrap(), Some(false));
}

#[test]
fn test_global_custom_predicate_registry() {
    // Clear global registry first to ensure clean test
    global_custom_predicate_registry().clear().unwrap();
    
    // Test global registry functionality
    let result = register_global_custom_predicate(
        "global-string?".to_string(),
        Some("Global string predicate".to_string()),
        |value| value.is_string(),
    );
    assert!(result.is_ok());
    
    // Test evaluation
    let string_val = Value::String("test".to_string());
    let number_val = Value::Number(SchemeNumber::Integer(1));
    
    let result1 = evaluate_global_custom_predicate("global-string?", &string_val);
    assert_eq!(result1.unwrap(), Some(true));
    
    let result2 = evaluate_global_custom_predicate("global-string?", &number_val);
    assert_eq!(result2.unwrap(), Some(false));
    
    // Test non-existent predicate
    let result3 = evaluate_global_custom_predicate("nonexistent?", &string_val);
    assert_eq!(result3.unwrap(), None);
}

#[test]
fn test_custom_predicate_with_records() {
    let registry = CustomPredicateRegistry::new();
    
    // Register a predicate for a specific record type
    // Note: This is a simplified example. In practice, you'd check the record type more thoroughly
    registry.register(
        "my-record?".to_string(),
        Some("Check if value is a specific record type".to_string()),
        |value| {
            match value {
                Value::Record(record) => {
                    // This is a simplified check - in practice you'd check the record type name
                    record.fields().len() > 0
                },
                _ => false,
            }
        },
    ).unwrap();
    
    // Test with different values
    let non_record = Value::String("not a record".to_string());
    assert_eq!(registry.evaluate("my-record?", &non_record).unwrap(), Some(false));
    
    // Note: Creating actual record instances would require more setup
    // This test demonstrates the framework
}

#[test]
fn test_custom_predicate_concurrent_access() {
    use std::thread;
    use std::sync::Arc;
    
    let registry = Arc::new(CustomPredicateRegistry::new());
    
    // Register initial predicate
    registry.register("concurrent-test?".to_string(), None, |value| value.is_number()).unwrap();
    
    let registry_clone = Arc::clone(&registry);
    let handle = thread::spawn(move || {
        // Access from another thread
        let test_val = Value::Number(SchemeNumber::Integer(123));
        registry_clone.evaluate("concurrent-test?", &test_val).unwrap()
    });
    
    let result = handle.join().unwrap();
    assert_eq!(result, Some(true));
}

#[test]
fn test_custom_predicate_error_handling() {
    let registry = CustomPredicateRegistry::new();
    
    // Test evaluation of non-existent predicate
    let test_val = Value::Number(SchemeNumber::Integer(42));
    let result = registry.evaluate("nonexistent?", &test_val).unwrap();
    assert_eq!(result, None);
    
    // Test getting info for non-existent predicate
    let info = registry.get_info("nonexistent?").unwrap();
    assert!(info.is_none());
    
    // Test unregistering non-existent predicate
    let removed = registry.unregister("nonexistent?").unwrap();
    assert!(!removed);
}

#[test]
fn test_custom_predicate_fn_type() {
    // Test that CustomPredicateFn can be stored and called
    let predicate_fn: CustomPredicateFn = Arc::new(|value| value.is_symbol());
    
    let symbol = Value::Symbol("test-symbol".to_string());
    let string = Value::String("test-string".to_string());
    
    assert!(predicate_fn(&symbol));
    assert!(!predicate_fn(&string));
}

#[test]
fn test_custom_predicate_info_cloning() {
    // Test that CustomPredicateInfo can be cloned correctly
    let predicate_fn: CustomPredicateFn = Arc::new(|value| value.is_boolean());
    let info = CustomPredicateInfo {
        name: "test-predicate?".to_string(),
        description: Some("A test predicate".to_string()),
        predicate_fn: predicate_fn.clone(),
    };
    
    let cloned_info = info.clone();
    assert_eq!(cloned_info.name, info.name);
    assert_eq!(cloned_info.description, info.description);
    
    // Test that both cloned predicates work
    let bool_val = Value::Boolean(true);
    assert!(info.predicate_fn(&bool_val));
    assert!(cloned_info.predicate_fn(&bool_val));
}

#[test]
fn test_custom_predicate_various_value_types() {
    let registry = CustomPredicateRegistry::new();
    
    // Register predicates for different value types
    registry.register("vector?".to_string(), None, |value| value.is_vector()).unwrap();
    registry.register("character?".to_string(), None, |value| value.is_character()).unwrap();
    registry.register("procedure?".to_string(), None, |value| value.is_procedure()).unwrap();
    
    // Test with different value types
    let vector = Value::Vector(vec![Value::Number(SchemeNumber::Integer(1))]);
    let character = Value::Character('a');
    let procedure = Value::Procedure(crate::value::Procedure::Builtin {
        name: "test".to_string(),
        arity: Some(1),
        function: std::sync::Arc::new(|_| Ok(Value::Boolean(true))),
    });
    
    assert_eq!(registry.evaluate("vector?", &vector).unwrap(), Some(true));
    assert_eq!(registry.evaluate("character?", &character).unwrap(), Some(true));
    assert_eq!(registry.evaluate("procedure?", &procedure).unwrap(), Some(true));
    
    // Cross-check (should all be false)
    assert_eq!(registry.evaluate("vector?", &character).unwrap(), Some(false));
    assert_eq!(registry.evaluate("character?", &procedure).unwrap(), Some(false));
    assert_eq!(registry.evaluate("procedure?", &vector).unwrap(), Some(false));
}
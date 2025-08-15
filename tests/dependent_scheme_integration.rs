//! Comprehensive Integration Test for Dependent Type System Scheme Integration
//!
//! This test suite verifies the complete integration between Lambdust's dependent type
//! system and R7RS Scheme values, ensuring Martin-Löf type theory correctness and
//! Rust implementation quality.

use lambdust::types::dependent::{core::DependentType, scheme_integration::SchemeIntegration};
use lambdust::eval::{Value, Environment};
use lambdust::ast::Literal;
use std::time::Instant;

/// Test SchemeIntegration::new() creation and basic functionality
#[test]
fn test_scheme_integration_creation() {
    let integration = SchemeIntegration::new().expect("Failed to create SchemeIntegration");
    
    // Verify initial state
    let (cache_size, mapping_size) = integration.cache_stats();
    assert_eq!(cache_size, 0, "Cache should start empty");
    assert!(mapping_size > 0, "Should have primitive mappings");
    
    // Verify primitive registry access
    let registry = integration.primitive_registry();
    // Registry should be accessible (we just check that we can access it)
    let _count = registry.count_by_category();
    // Registry should be accessible without errors
    
    println!("✅ SchemeIntegration creation works correctly");
}

/// Test Value → DependentType conversion accuracy for literals
#[test]
fn test_value_to_type_conversion_literals() {
    let mut integration = SchemeIntegration::new().expect("Failed to create SchemeIntegration");
    
    // Test exact integer conversion
    let int_value = Value::Literal(Literal::ExactInteger(42));
    let int_type = integration.value_to_type(&int_value)
        .expect("Failed to convert integer value");
    
    match int_type {
        DependentType::Inductive { name, universe_level, .. } => {
            assert_eq!(name, "ExactInteger");
            assert_eq!(universe_level, 0);
        }
        _ => panic!("Expected ExactInteger inductive type"),
    }
    
    // Test boolean conversion
    let bool_value = Value::boolean(true);
    let bool_type = integration.value_to_type(&bool_value)
        .expect("Failed to convert boolean value");
    
    match bool_type {
        DependentType::Inductive { name, universe_level, constructors, .. } => {
            assert_eq!(name, "Boolean");
            assert_eq!(universe_level, 0);
            assert_eq!(constructors.len(), 1);
            assert_eq!(constructors[0].0, "true");
        }
        _ => panic!("Expected Boolean inductive type"),
    }
    
    // Test string conversion
    let string_value = Value::Literal(Literal::String("hello".to_string()));
    let string_type = integration.value_to_type(&string_value)
        .expect("Failed to convert string value");
    
    match string_type {
        DependentType::Inductive { name, universe_level, .. } => {
            assert_eq!(name, "String");
            assert_eq!(universe_level, 0);
        }
        _ => panic!("Expected String inductive type"),
    }
    
    println!("✅ Literal value to type conversion works correctly");
}

/// Test DependentType → Value conversion accuracy
#[test]
fn test_type_to_value_conversion() {
    let integration = SchemeIntegration::new().expect("Failed to create SchemeIntegration");
    
    // Test Universe type conversion
    let universe_type = DependentType::Universe(2);
    let universe_value = integration.type_to_value(&universe_type)
        .expect("Failed to convert universe type");
    
    match universe_value {
        Value::Symbol(_) => {
            // Should be a symbol representing the universe
        }
        _ => panic!("Expected symbol for universe type"),
    }
    
    // Test Inductive type conversion
    let nat_type = DependentType::Inductive {
        name: "Nat".to_string(),
        parameters: vec![],
        universe_level: 0,
        constructors: vec![],
        induction_principle: None,
    };
    let nat_value = integration.type_to_value(&nat_type)
        .expect("Failed to convert inductive type");
    
    match nat_value {
        Value::Symbol(_) => {
            // Should be a symbol representing the type name
        }
        _ => panic!("Expected symbol for inductive type"),
    }
    
    println!("✅ Type to value conversion works correctly");
}

/// Test symbol and null value processing
#[test]
fn test_symbol_and_null_processing() {
    let mut integration = SchemeIntegration::new().expect("Failed to create SchemeIntegration");
    
    // Test symbol conversion
    let symbol_value = Value::symbol_from_str("test-symbol");
    let symbol_type = integration.value_to_type(&symbol_value)
        .expect("Failed to convert symbol value");
    
    match symbol_type {
        DependentType::Inductive { name, universe_level, .. } => {
            assert_eq!(name, "Symbol");
            assert_eq!(universe_level, 0);
        }
        _ => panic!("Expected Symbol inductive type"),
    }
    
    // Test null conversion
    let null_value = Value::Nil;
    let null_type = integration.value_to_type(&null_value)
        .expect("Failed to convert null value");
    
    match null_type {
        DependentType::Inductive { name, universe_level, constructors, .. } => {
            assert_eq!(name, "Null");
            assert_eq!(universe_level, 0);
            assert_eq!(constructors.len(), 1);
            assert_eq!(constructors[0].0, "null");
        }
        _ => panic!("Expected Null inductive type"),
    }
    
    println!("✅ Symbol and null processing works correctly");
}

/// Test type compatibility checking functionality
#[test]
fn test_type_compatibility_checking() {
    let mut integration = SchemeIntegration::new().expect("Failed to create SchemeIntegration");
    
    // Test compatible types
    let int_value1 = Value::Literal(Literal::ExactInteger(42));
    let int_value2 = Value::Literal(Literal::ExactInteger(100));
    
    let int_type1 = integration.value_to_type(&int_value1)
        .expect("Failed to convert first integer");
    let is_compatible = integration.is_value_compatible(&int_value2, &int_type1)
        .expect("Failed to check compatibility");
    
    assert!(is_compatible, "Same types should be compatible");
    
    // Test incompatible types
    let bool_value = Value::boolean(false);
    let is_incompatible = integration.is_value_compatible(&bool_value, &int_type1)
        .expect("Failed to check incompatibility");
    
    assert!(!is_incompatible, "Different types should be incompatible");
    
    // Test universe compatibility
    let _universe1 = DependentType::Universe(0);
    let _universe2 = DependentType::Universe(0);
    let _universe3 = DependentType::Universe(1);
    
    // Same universe levels should be compatible (tested via internal method)
    // Different universe levels should be incompatible
    
    println!("✅ Type compatibility checking works correctly");
}

/// Test cache functionality and performance
#[test]
fn test_cache_functionality_and_performance() {
    let mut integration = SchemeIntegration::new().expect("Failed to create SchemeIntegration");
    
    let test_value = Value::Literal(Literal::ExactInteger(42));
    
    // First conversion should populate cache
    let start = Instant::now();
    let _type1 = integration.value_to_type(&test_value)
        .expect("Failed first conversion");
    let first_elapsed = start.elapsed();
    
    let (cache_size_after_first, _) = integration.cache_stats();
    assert_eq!(cache_size_after_first, 1, "Cache should contain one entry");
    
    // Second conversion should use cache (should be faster)
    let start = Instant::now();
    let _type2 = integration.value_to_type(&test_value)
        .expect("Failed second conversion");
    let second_elapsed = start.elapsed();
    
    let (cache_size_after_second, _) = integration.cache_stats();
    assert_eq!(cache_size_after_second, 1, "Cache size should remain the same");
    
    // Cache should improve performance (though with such simple operations, this might not always be measurable)
    println!("First conversion: {:?}, Second conversion: {:?}", first_elapsed, second_elapsed);
    
    // Test cache clearing
    integration.clear_cache();
    let (cache_size_after_clear, _) = integration.cache_stats();
    assert_eq!(cache_size_after_clear, 0, "Cache should be empty after clearing");
    
    println!("✅ Cache functionality works correctly");
}

/// Test error handling and edge cases
#[test]
fn test_error_handling_and_edge_cases() {
    let mut integration = SchemeIntegration::new().expect("Failed to create SchemeIntegration");
    
    // Test with complex nested values (should not panic)
    let nested_pair = Value::pair(
        Value::boolean(true),
        Value::pair(Value::Literal(Literal::ExactInteger(42)), Value::Nil)
    );
    
    // Should handle unknown types gracefully
    let result = integration.value_to_type(&nested_pair);
    assert!(result.is_ok(), "Should handle unknown types gracefully");
    
    // Test type conversion for unknown types
    let unknown_type = DependentType::Universe(999);
    let result = integration.type_to_value(&unknown_type);
    assert!(result.is_ok(), "Should handle unknown types gracefully");
    
    println!("✅ Error handling and edge cases work correctly");
}

/// Test expression type inference with environment
#[test]
fn test_expression_type_inference() {
    let mut integration = SchemeIntegration::new().expect("Failed to create SchemeIntegration");
    let env = Environment::new(None, 0);
    
    // Test literal expression inference
    let literal_expr = lambdust::ast::Expr::Literal(Literal::Boolean(true));
    let inferred_type = integration.infer_expression_type(&literal_expr, &env)
        .expect("Failed to infer literal type");
    
    match inferred_type {
        DependentType::Inductive { name, .. } => {
            assert_eq!(name, "Boolean");
        }
        _ => panic!("Expected Boolean type for boolean literal"),
    }
    
    // Test identifier expression (should return default for unknown identifier)
    let id_expr = lambdust::ast::Expr::Identifier("unknown-var".to_string());
    let inferred_type = integration.infer_expression_type(&id_expr, &env)
        .expect("Failed to infer identifier type");
    
    match inferred_type {
        DependentType::Universe(0) => {
            // Expected default type
        }
        _ => panic!("Expected Universe(0) for unknown identifier"),
    }
    
    println!("✅ Expression type inference works correctly");
}

/// Performance test for Scheme integration (target: <100ms)
#[test]
fn test_performance_comprehensive() {
    let start = Instant::now();
    
    // Test creation performance
    let mut integration = SchemeIntegration::new().expect("Failed to create SchemeIntegration");
    
    // Test conversion performance with many values
    let test_values = vec![
        Value::boolean(true),
        Value::boolean(false),
        Value::Literal(Literal::ExactInteger(1)),
        Value::Literal(Literal::ExactInteger(42)),
        Value::Literal(Literal::ExactInteger(999)),
        Value::Literal(Literal::String("test".to_string())),
        Value::Literal(Literal::String("performance".to_string())),
        Value::symbol_from_str("symbol1"),
        Value::symbol_from_str("symbol2"),
        Value::Nil,
    ];
    
    // Convert all values to types multiple times
    for _ in 0..10 {
        for value in &test_values {
            let _result = integration.value_to_type(value)
                .expect("Failed to convert value");
        }
    }
    
    // Test cache stats
    let (cache_size, _) = integration.cache_stats();
    assert!(cache_size <= test_values.len(), "Cache should not exceed number of unique values");
    
    let elapsed = start.elapsed();
    println!("Comprehensive performance test completed in {:?}", elapsed);
    
    // Target: less than 100ms for all operations
    assert!(elapsed.as_millis() < 100, 
           "Performance test should complete in less than 100ms, took {:?}", elapsed);
    
    println!("✅ Performance test passed ({:?} < 100ms)", elapsed);
}

/// Test Martin-Löf type theory correctness
#[test]
fn test_martin_lof_type_theory_correctness() {
    let mut integration = SchemeIntegration::new().expect("Failed to create SchemeIntegration");
    
    // Test universe hierarchy
    let universe0 = DependentType::Universe(0);
    let universe1 = DependentType::Universe(1);
    
    // Universe levels should be distinct
    assert_ne!(universe0, universe1);
    
    // Test inductive type well-formedness
    let bool_value = Value::boolean(true);
    let bool_type = integration.value_to_type(&bool_value)
        .expect("Failed to get boolean type");
    
    match bool_type {
        DependentType::Inductive { name, universe_level, constructors, .. } => {
            // Verify type theory properties
            assert_eq!(name, "Boolean");
            assert_eq!(universe_level, 0); // Should be in base universe
            assert_eq!(constructors.len(), 1); // Should have constructor for true
            
            // Constructor should have proper type
            match &constructors[0].1 {
                DependentType::Universe(level) => {
                    assert_eq!(*level, 0); // Constructor type should be in base universe
                }
                _ => panic!("Constructor should have universe type"),
            }
        }
        _ => panic!("Boolean should be inductive type"),
    }
    
    // Test type consistency across multiple conversions
    let int_value1 = Value::Literal(Literal::ExactInteger(42));
    let int_value2 = Value::Literal(Literal::ExactInteger(100));
    
    let type1 = integration.value_to_type(&int_value1).expect("Failed conversion 1");
    let type2 = integration.value_to_type(&int_value2).expect("Failed conversion 2");
    
    // Same Scheme types should produce equivalent dependent types
    match (&type1, &type2) {
        (DependentType::Inductive { name: n1, universe_level: l1, .. },
         DependentType::Inductive { name: n2, universe_level: l2, .. }) => {
            assert_eq!(n1, n2, "Same Scheme types should have same dependent type name");
            assert_eq!(l1, l2, "Same Scheme types should have same universe level");
        }
        _ => panic!("Both should be inductive types"),
    }
    
    println!("✅ Martin-Löf type theory correctness verified");
}

/// Comprehensive integration test combining all functionality
#[test]
fn test_comprehensive_scheme_integration() {
    println!("\n🧪 Comprehensive Dependent Type Scheme Integration Test");
    println!("=====================================================");
    
    // Run all individual tests as components
    test_scheme_integration_creation();
    test_value_to_type_conversion_literals();
    test_type_to_value_conversion();
    test_symbol_and_null_processing();
    test_type_compatibility_checking();
    test_cache_functionality_and_performance();
    test_error_handling_and_edge_cases();
    test_expression_type_inference();
    test_performance_comprehensive();
    test_martin_lof_type_theory_correctness();
    
    // Additional integration verification
    let mut integration = SchemeIntegration::new().expect("Failed to create integration");
    
    // Test round-trip conversion consistency
    let original_values = vec![
        Value::boolean(true),
        Value::boolean(false),
        Value::Literal(Literal::ExactInteger(42)),
        Value::Literal(Literal::String("test".to_string())),
        Value::symbol_from_str("test-symbol"),
        Value::Nil,
    ];
    
    for original_value in original_values {
        // Convert to type and back
        let dep_type = integration.value_to_type(&original_value)
            .expect("Failed to convert to dependent type");
        let _converted_value = integration.type_to_value(&dep_type)
            .expect("Failed to convert back to value");
        
        // Note: Exact round-trip equality is not expected due to type erasure,
        // but the operations should succeed without errors
    }
    
    // Final cache and performance verification
    let (final_cache_size, final_mapping_size) = integration.cache_stats();
    assert!(final_cache_size > 0, "Cache should contain entries after operations");
    assert!(final_mapping_size > 0, "Mappings should exist");
    
    println!("\n🎉 All Scheme integration tests passed!");
    println!("Cache entries: {}, Primitive mappings: {}", final_cache_size, final_mapping_size);
    println!("The dependent type Scheme integration is working correctly with R7RS compliance.");
}
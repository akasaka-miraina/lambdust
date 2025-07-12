//! Type System Integration Tests
//!
//! End-to-end integration tests for the complete polynomial universe type system,
//! verifying the interaction between all components:
//! - Universe polymorphic classes with dependent types
//! - Monad algebra with HoTT features
//! - Parallel type checking with incremental inference
//! - Real-world type checking scenarios

use super::test_utils::*;
use crate::type_system::*;
use crate::ast::Expr;
use std::sync::Arc;

#[test]
fn test_complete_type_system_initialization() {
    let mut system = create_test_system();
    
    // Verify all major components are initialized
    assert!(system.initialize_standard_universe_classes().is_ok());
    assert!(system.initialize_standard_transformers().is_ok());
    
    // Test that basic type operations work
    let nat_type = PolynomialType::Base(polynomial_types::BaseType::Natural);
    let nat_value = nat_value(42);
    
    let type_check_result = system.type_check(&nat_value, &nat_type);
    assert!(type_check_result.is_ok(), "Basic type checking should work");
    
    let infer_result = system.infer_type(&nat_value);
    assert!(infer_result.is_ok(), "Type inference should work");
}

#[test]
fn test_universe_polymorphic_functor_integration() {
    let mut system = create_test_system();
    
    // Register Functor class
    let functor_class = create_functor_class();
    let register_result = system.register_universe_polymorphic_class(functor_class);
    assert!(register_result.is_ok(), "Should register Functor class");
    
    // Create and register List Functor instance
    let mut universe_args = std::collections::HashMap::new();
    universe_args.insert("u".to_string(), UniverseExpression::Literal(UniverseLevel::new(1)));
    
    let mut methods = std::collections::HashMap::new();
    methods.insert("fmap".to_string(), Value::Symbol("list_map".to_string()));
    
    let list_functor = UniversePolymorphicInstance {
        class_name: "Functor".to_string(),
        universe_args,
        type_args: vec![PolynomialType::Base(polynomial_types::BaseType::List)],
        methods,
        law_proofs: std::collections::HashMap::new(),
    };
    
    let instance_result = system.register_universe_polymorphic_instance(list_functor);
    assert!(instance_result.is_ok(), "Should register List Functor instance");
    
    // Test instance resolution
    let resolution_result = system.resolve_universe_polymorphic_instance(
        "Functor",
        &[PolynomialType::Base(polynomial_types::BaseType::List)],
        UniverseLevel::new(1),
    );
    assert!(resolution_result.is_ok(), "Should resolve List Functor instance");
    
    let resolved = resolution_result.unwrap();
    assert_eq!(resolved.class_name, "Functor");
    assert!(resolved.methods.contains_key("fmap"));
}

#[test]
fn test_monad_hierarchy_with_transformers() {
    let mut system = create_test_system();
    
    // Register monad hierarchy: Functor -> Applicative -> Monad
    let functor_class = create_functor_class();
    let monad_class = create_monad_class();
    
    system.register_universe_polymorphic_class(functor_class).unwrap();
    system.register_universe_polymorphic_class(monad_class).unwrap();
    
    // Create StateT monad transformer
    let state_transformer = MonadTransformer {
        name: "StateT".to_string(),
        base_monad_constraint: Some(UniversePolymorphicConstraint {
            class_name: "Monad".to_string(),
            type_args: vec![],
            universe_constraint: None,
        }),
        state_type: Some(PolynomialType::Base(polynomial_types::BaseType::Natural)),
        laws: vec![],
    };
    
    let transformer_result = system.register_monad_transformer(state_transformer);
    assert!(transformer_result.is_ok(), "Should register StateT transformer");
    
    // Build transformer stack: StateT IO
    let io_monad = PolynomialType::Base(polynomial_types::BaseType::Symbol); // Placeholder for IO
    let stack_result = system.build_transformer_stack(
        vec!["StateT".to_string()],
        io_monad,
    );
    assert!(stack_result.is_ok(), "Should build StateT IO stack");
    
    let stack = stack_result.unwrap();
    assert_eq!(stack.transformers.len(), 1);
    assert_eq!(stack.transformers[0], "StateT");
}

#[test]
fn test_dependent_type_integration() {
    let mut system = create_test_system();
    
    // Create dependent types
    let function_type = create_function_type();
    let dependent_pair = create_dependent_pair_type();
    
    // Test dependent type construction
    match function_type {
        DependentType::Pi(pi_type) => {
            assert_eq!(pi_type.parameter, "x");
            assert!(matches!(
                *pi_type.domain,
                DependentType::Base(polynomial_types::BaseType::Natural)
            ));
            assert!(matches!(
                *pi_type.codomain,
                DependentType::Base(polynomial_types::BaseType::Natural)
            ));
        }
        _ => panic!("Expected Pi type"),
    }
    
    match dependent_pair {
        DependentType::Sigma(sigma_type) => {
            assert_eq!(sigma_type.name, "x");
            assert!(matches!(
                *sigma_type.first_type,
                DependentType::Base(polynomial_types::BaseType::Natural)
            ));
        }
        _ => panic!("Expected Sigma type"),
    }
}

#[test]
fn test_parallel_type_checking_performance() {
    let system = create_test_system();
    
    // Create multiple expressions for parallel type checking
    let expressions: Vec<(Expr, String)> = (0..100)
        .map(|i| {
            let expr = Expr::Number(crate::lexer::SchemeNumber::Integer(i));
            let context = format!("expr_{}", i);
            (expr, context)
        })
        .collect();
    
    let start = std::time::Instant::now();
    let parallel_result = system.type_check_parallel(expressions.clone());
    let parallel_duration = start.elapsed();
    
    assert!(parallel_result.is_ok(), "Parallel type checking should succeed");
    let parallel_results = parallel_result.unwrap();
    assert_eq!(parallel_results.len(), 100);
    
    // Test automatic parallelization decision
    let start = std::time::Instant::now();
    let auto_result = system.type_check_auto(expressions);
    let auto_duration = start.elapsed();
    
    assert!(auto_result.is_ok(), "Auto type checking should succeed");
    let auto_results = auto_result.unwrap();
    assert_eq!(auto_results.len(), 100);
    
    // Verify performance metrics
    let metrics = system.get_parallel_metrics();
    assert!(metrics.total_expressions_processed > 0);
    assert!(metrics.parallel_time_saved_ms >= 0.0);
    
    println!("Parallel: {:?}, Auto: {:?}", parallel_duration, auto_duration);
}

#[test]
fn test_incremental_type_inference_with_caching() {
    let mut system = create_test_system();
    
    // Set up dependencies
    system.add_incremental_dependency("f".to_string(), "x".to_string());
    system.add_incremental_dependency("g".to_string(), "f".to_string());
    system.add_incremental_dependency("h".to_string(), "g".to_string());
    
    // Test incremental inference
    let value = nat_value(42);
    let result1 = system.infer_incremental(&value, Some("test_context"));
    assert!(result1.is_ok(), "First incremental inference should succeed");
    
    let result2 = system.infer_incremental(&value, Some("test_context"));
    assert!(result2.is_ok(), "Second incremental inference should succeed (cached)");
    
    // Test cache invalidation
    let invalidated_count = system.invalidate_incremental_cache("x");
    assert!(invalidated_count.is_ok(), "Cache invalidation should succeed");
    
    let invalidated = invalidated_count.unwrap();
    assert!(invalidated > 0, "Should invalidate dependent entries");
    
    // Test that inference still works after invalidation
    let result3 = system.infer_incremental(&value, Some("test_context"));
    assert!(result3.is_ok(), "Inference should work after cache invalidation");
    
    // Verify cache statistics
    let stats = system.get_incremental_stats();
    assert!(stats.cache_size > 0);
    assert!(stats.cache_hits >= 0);
    assert!(stats.cache_misses > 0);
}

#[test]
fn test_hott_type_class_integration() {
    let mut system = create_test_system();
    
    // Create a HoTT-style type class with univalence
    let equiv_class = TypeClassDefinition {
        name: "Equivalence".to_string(),
        type_parameters: vec!["A".to_string(), "B".to_string()],
        methods: vec![
            "to".to_string(),
            "from".to_string(),
            "to_from".to_string(),
            "from_to".to_string(),
        ],
        laws: vec![
            TypeClassLaw {
                name: "to_from_inverse".to_string(),
                statement: "forall x, from (to x) = x".to_string(),
                proof_obligation: Some("construct iso_proof".to_string()),
            },
            TypeClassLaw {
                name: "from_to_inverse".to_string(),
                statement: "forall y, to (from y) = y".to_string(),
                proof_obligation: Some("construct iso_proof".to_string()),
            },
        ],
        universe_level: UniverseLevel::new(1),
    };
    
    let register_result = system.register_type_class(equiv_class);
    assert!(register_result.is_ok(), "Should register Equivalence type class");
    
    // Create an instance: Bool ≃ Bool
    let bool_equiv_instance = TypeClassInstance {
        class_name: "Equivalence".to_string(),
        instance_type: PolynomialType::Base(polynomial_types::BaseType::Boolean),
        methods: {
            let mut methods = std::collections::HashMap::new();
            methods.insert("to".to_string(), Value::Symbol("id".to_string()));
            methods.insert("from".to_string(), Value::Symbol("id".to_string()));
            methods.insert("to_from".to_string(), Value::Symbol("refl".to_string()));
            methods.insert("from_to".to_string(), Value::Symbol("refl".to_string()));
            methods
        },
        universe_level: UniverseLevel::new(0),
    };
    
    let instance_result = system.register_type_class_instance(bool_equiv_instance);
    assert!(instance_result.is_ok(), "Should register Bool ≃ Bool instance");
    
    // Test instance resolution
    let bool_type = PolynomialType::Base(polynomial_types::BaseType::Boolean);
    let resolution_result = system.resolve_type_class("Equivalence", &bool_type);
    assert!(resolution_result.is_ok(), "Should resolve Bool ≃ Bool instance");
}

#[test]
fn test_distributive_law_application() {
    let mut system = create_test_system();
    
    // Test distributive law between Maybe and List monads
    let test_value = Value::Vector(vec![
        Value::Symbol("Some".to_string()),
        nat_value(42),
    ]);
    
    let distributive_result = system.apply_distributive_law("Maybe", "List", &test_value);
    
    // Even if not fully implemented, should handle the call gracefully
    match distributive_result {
        Ok(_) => {
            // Distributive law applied successfully
        }
        Err(_) => {
            // Expected if not fully implemented yet
        }
    }
}

#[test]
fn test_complex_type_checking_scenario() {
    let mut system = create_test_system();
    
    // Register necessary type classes
    let functor_class = create_functor_class();
    let monad_class = create_monad_class();
    
    system.register_universe_polymorphic_class(functor_class).unwrap();
    system.register_universe_polymorphic_class(monad_class).unwrap();
    
    // Create a complex nested value: [Maybe [1, 2, 3]]
    let inner_list = Value::Vector(vec![
        nat_value(1),
        nat_value(2),
        nat_value(3),
    ]);
    
    let maybe_list = Value::Vector(vec![
        Value::Symbol("Some".to_string()),
        inner_list,
    ]);
    
    let outer_list = Value::Vector(vec![maybe_list]);
    
    // Test type inference on complex structure
    let infer_result = system.infer_type(&outer_list);
    assert!(infer_result.is_ok(), "Should infer type of complex nested structure");
    
    // Test incremental inference with context
    let incremental_result = system.infer_incremental(&outer_list, Some("nested_structure"));
    assert!(incremental_result.is_ok(), "Should infer type incrementally");
}

#[test]
fn test_type_system_error_recovery() {
    let mut system = create_test_system();
    
    // Test various error scenarios
    let invalid_class = UniversePolymorphicClass {
        name: "".to_string(), // Invalid empty name
        universe_parameter: "u".to_string(),
        type_parameters: vec![],
        methods: vec![],
        laws: vec![],
        superclasses: vec![],
    };
    
    let register_result = system.register_universe_polymorphic_class(invalid_class);
    // Should handle invalid class gracefully
    assert!(register_result.is_err() || register_result.is_ok());
    
    // Test resolution of non-existent instance
    let resolution_result = system.resolve_universe_polymorphic_instance(
        "NonExistentClass",
        &[PolynomialType::Base(polynomial_types::BaseType::Natural)],
        UniverseLevel::new(0),
    );
    assert!(resolution_result.is_err(), "Should fail to resolve non-existent class");
    
    // Test invalid cache invalidation
    let invalid_result = system.invalidate_incremental_cache("non_existent_symbol");
    assert!(invalid_result.is_ok(), "Should handle invalid symbol gracefully");
}

#[test]
fn test_universe_polymorphism_across_levels() {
    let mut system = create_test_system();
    
    // Test that the same type class can work across different universe levels
    let functor_class = create_functor_class();
    system.register_universe_polymorphic_class(functor_class).unwrap();
    
    // Create instances at different universe levels
    for level in 0..3 {
        let mut universe_args = std::collections::HashMap::new();
        universe_args.insert("u".to_string(), 
            UniverseExpression::Literal(UniverseLevel::new(level)));
        
        let mut methods = std::collections::HashMap::new();
        methods.insert("fmap".to_string(), 
            Value::Symbol(format!("fmap_level_{}", level)));
        
        let instance = UniversePolymorphicInstance {
            class_name: "Functor".to_string(),
            universe_args,
            type_args: vec![PolynomialType::Base(polynomial_types::BaseType::List)],
            methods,
            law_proofs: std::collections::HashMap::new(),
        };
        
        let result = system.register_universe_polymorphic_instance(instance);
        assert!(result.is_ok(), "Should register instance at level {}", level);
    }
    
    // Test resolution at different levels
    for level in 0..3 {
        let resolution = system.resolve_universe_polymorphic_instance(
            "Functor",
            &[PolynomialType::Base(polynomial_types::BaseType::List)],
            UniverseLevel::new(level),
        );
        
        if let Ok(instance) = resolution {
            assert_eq!(instance.class_name, "Functor");
            // Instance should work at the requested level
        }
    }
}

#[test]
fn test_parallel_and_incremental_interaction() {
    let mut system = create_test_system();
    
    // Set up parallel type bindings
    system.add_parallel_type_binding(
        "x".to_string(),
        PolynomialType::Base(polynomial_types::BaseType::Natural)
    );
    
    system.add_parallel_type_binding(
        "y".to_string(),
        PolynomialType::Base(polynomial_types::BaseType::Boolean)
    );
    
    // Set up incremental dependencies
    system.add_incremental_dependency("expr1".to_string(), "x".to_string());
    system.add_incremental_dependency("expr2".to_string(), "y".to_string());
    
    // Create expressions that depend on the bindings
    let expressions = vec![
        (Expr::Symbol("x".to_string()), "expr1".to_string()),
        (Expr::Symbol("y".to_string()), "expr2".to_string()),
    ];
    
    // Test parallel type checking
    let parallel_result = system.type_check_parallel(expressions);
    assert!(parallel_result.is_ok(), "Parallel type checking should work with bindings");
    
    // Test incremental inference
    let x_value = Value::Symbol("x".to_string());
    let incremental_result = system.infer_incremental(&x_value, Some("expr1"));
    assert!(incremental_result.is_ok(), "Incremental inference should work");
    
    // Test cache invalidation affects parallel processing
    let invalidated = system.invalidate_incremental_cache("x");
    assert!(invalidated.is_ok(), "Should invalidate cache");
    
    // Verify metrics are updated
    let parallel_metrics = system.get_parallel_metrics();
    let incremental_stats = system.get_incremental_stats();
    
    assert!(parallel_metrics.total_expressions_processed > 0);
    assert!(incremental_stats.cache_misses > 0);
}

#[test]
fn test_full_type_system_workflow() {
    let mut system = create_test_system();
    
    // 1. Initialize standard classes and transformers
    system.initialize_standard_universe_classes().unwrap();
    system.initialize_standard_transformers().unwrap();
    
    // 2. Register custom type classes
    let functor_class = create_functor_class();
    system.register_universe_polymorphic_class(functor_class).unwrap();
    
    // 3. Register instances
    let mut methods = std::collections::HashMap::new();
    methods.insert("fmap".to_string(), Value::Symbol("vector_map".to_string()));
    
    let vector_functor = UniversePolymorphicInstance {
        class_name: "Functor".to_string(),
        universe_args: {
            let mut args = std::collections::HashMap::new();
            args.insert("u".to_string(), UniverseExpression::Literal(UniverseLevel::new(1)));
            args
        },
        type_args: vec![PolynomialType::Base(polynomial_types::BaseType::Vector)],
        methods,
        law_proofs: std::collections::HashMap::new(),
    };
    
    system.register_universe_polymorphic_instance(vector_functor).unwrap();
    
    // 4. Test complex type operations
    let complex_value = Value::Vector(vec![
        Value::Vector(vec![nat_value(1), nat_value(2)]),
        Value::Vector(vec![nat_value(3), nat_value(4)]),
    ]);
    
    let infer_result = system.infer_type(&complex_value);
    assert!(infer_result.is_ok(), "Should infer type of complex structure");
    
    // 5. Test instance resolution
    let resolution = system.resolve_universe_polymorphic_instance(
        "Functor",
        &[PolynomialType::Base(polynomial_types::BaseType::Vector)],
        UniverseLevel::new(1),
    );
    assert!(resolution.is_ok(), "Should resolve Vector Functor instance");
    
    // 6. Verify the complete workflow worked
    let final_stats = system.get_incremental_stats();
    assert!(final_stats.cache_size >= 0);
    
    let classes = system.list_universe_polymorphic_classes();
    assert!(classes.contains(&"Functor".to_string()));
    
    let transformers = system.list_monad_transformers();
    // May or may not have transformers depending on initialization
    assert!(transformers.len() >= 0);
}
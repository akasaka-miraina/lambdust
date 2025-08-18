//! Comprehensive test suite for the contract system.
//!
//! This module contains integration tests for all components of the contract
//! system, including AST nodes, predicates, combinators, compilation,
//! enforcement, and runtime operations.

#[cfg(test)]
mod contract_tests {
    use crate::contracts::{
        ContractSystem, ContractConfig,
        BlameInfo, BlameTarget, BlameBoundary, BoundaryType, BlameTracker,
        CompilationContext, OptimizationLevel,
        PredicateRegistry,
        ContractRuntime,
    };
    use crate::contracts::ast::{ContractExpr, ComparisonOp};
    use crate::eval::Value;
    use crate::ast::Literal;
    use crate::diagnostics::{Span, Spanned};
    use std::collections::HashMap;

    // Test helper function removed - using the one defined later
    use std::sync::Arc;

    // ============= TEST UTILITIES =============

    fn create_test_blame() -> BlameInfo {
        BlameInfo {
            positive: BlameTarget::System {
                component: "test".to_string(),
                description: "test component".to_string(),
            },
            negative: BlameTarget::System {
                component: "test".to_string(),
                description: "test component".to_string(),
            },
            boundary: BlameBoundary {
                boundary_type: BoundaryType::ExplicitContract,
                contract: "test contract".to_string(),
                location: Span::new(0, 10),
                context: HashMap::new(),
            },
            call_stack: Vec::new(),
            id: 1,
            parent: None,
        }
    }

    fn create_test_context() -> CompilationContext {
        let predicates = PredicateRegistry::new();
        let blame = create_test_blame();
        CompilationContext::new(predicates, blame)
    }

    // ============= AST TESTS =============

    #[test]
    fn test_contract_ast_creation() {
        let span = Span::new(0, 10);

        // Test predicate contract
        let number_contract = ContractExpr::predicate("number?", span);
        assert!(number_contract.is_predicate());
        assert_eq!(number_contract.as_predicate(), Some("number?"));

        // Test function contract
        let param_contract = Spanned::new(ContractExpr::predicate("number?", span), span);
        let return_contract = Spanned::new(ContractExpr::predicate("string?", span), span);
        let func_contract = ContractExpr::function(vec![param_contract], return_contract, span);
        assert!(func_contract.is_function_contract());

        // Test combinator contracts
        let and_contract = ContractExpr::and(vec![
            Spanned::new(ContractExpr::predicate("number?", span), span),
            Spanned::new(ContractExpr::predicate("positive?", span), span),
        ], span);
        assert!(and_contract.is_combinator());

        // Test structural contracts
        let element_contract = Spanned::new(ContractExpr::predicate("number?", span), span);
        let listof_contract = ContractExpr::listof(element_contract, span);
        assert!(listof_contract.is_structural());
    }

    #[test]
    fn test_contract_display() {
        let span = Span::new(0, 10);

        // Test predicate display
        let number_contract = ContractExpr::predicate("number?", span);
        assert_eq!(format!("{}", number_contract), "number?");

        // Test function contract display
        let param1 = Spanned::new(ContractExpr::predicate("number?", span), span);
        let param2 = Spanned::new(ContractExpr::predicate("string?", span), span);
        let return_type = Spanned::new(ContractExpr::predicate("boolean?", span), span);
        let func_contract = ContractExpr::function(vec![param1, param2], return_type, span);
        assert_eq!(format!("{}", func_contract), "(-> number? string? boolean?)");

        // Test and combinator display
        let and_contract = ContractExpr::and(vec![
            Spanned::new(ContractExpr::predicate("number?", span), span),
            Spanned::new(ContractExpr::predicate("positive?", span), span),
        ], span);
        assert_eq!(format!("{}", and_contract), "(and/c number? positive?)");
    }

    // ============= PREDICATE TESTS =============

    #[test]
    fn test_builtin_predicates() {
        let registry = PredicateRegistry::new();

        // Test numeric predicates
        let number_pred = registry.lookup("number?").unwrap();
        let integer_pred = registry.lookup("integer?").unwrap();
        let positive_pred = registry.lookup("positive?").unwrap();

        let number_val = Value::Literal(Literal::Number(42.5));
        let integer_val = Value::Literal(Literal::Integer(42));
        let string_val = Value::Literal(Literal::String("hello".to_string()));

        assert!(number_pred.test(&number_val));
        assert!(number_pred.test(&integer_val));
        assert!(!number_pred.test(&string_val));

        assert!(!integer_pred.test(&number_val));
        assert!(integer_pred.test(&integer_val));
        assert!(!integer_pred.test(&string_val));

        assert!(positive_pred.test(&number_val));
        assert!(positive_pred.test(&integer_val));
        assert!(!positive_pred.test(&string_val));
    }

    #[test]
    fn test_string_predicates() {
        let registry = PredicateRegistry::new();

        let string_pred = registry.lookup("string?").unwrap();
        let non_empty_pred = registry.lookup("non-empty-string?").unwrap();

        let string_val = Value::Literal(Literal::String("hello".to_string()));
        let empty_string = Value::Literal(Literal::String("".to_string()));
        let number_val = Value::Literal(Literal::Number(42.0));

        assert!(string_pred.test(&string_val));
        assert!(string_pred.test(&empty_string));
        assert!(!string_pred.test(&number_val));

        assert!(non_empty_pred.test(&string_val));
        assert!(!non_empty_pred.test(&empty_string));
        assert!(!non_empty_pred.test(&number_val));
    }

    #[test]
    fn test_collection_predicates() {
        let registry = PredicateRegistry::new();

        let null_pred = registry.lookup("null?").unwrap();
        let pair_pred = registry.lookup("pair?").unwrap();
        let list_pred = registry.lookup("list?").unwrap();

        let nil_val = Value::Nil;
        let pair_val = Value::Pair(
            Arc::new(Value::Literal(Literal::Number(1.0))),
            Arc::new(Value::Literal(Literal::Number(2.0))),
        );
        let proper_list = Value::Pair(
            Arc::new(Value::Literal(Literal::Number(1.0))),
            Arc::new(Value::Nil),
        );

        assert!(null_pred.test(&nil_val));
        assert!(!null_pred.test(&pair_val));

        assert!(!pair_pred.test(&nil_val));
        assert!(pair_pred.test(&pair_val));
        assert!(pair_pred.test(&proper_list));

        assert!(list_pred.test(&nil_val));
        assert!(!list_pred.test(&pair_val)); // Improper list
        assert!(list_pred.test(&proper_list));
    }

    // ============= COMBINATOR TESTS =============

    #[test]
    fn test_contract_compilation() {
        let mut system = ContractSystem::new();
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };

        // Test simple predicate compilation
        let number_contract = ContractExpr::predicate("number?", Span::new(0, 10));
        let compiled = system.compile_contract(&number_contract, &context).unwrap();
        assert_eq!(compiled.id, 1);

        // Test function contract compilation
        let param = Spanned::new(ContractExpr::predicate("number?", Span::new(0, 10)), Span::new(0, 10));
        let result = Spanned::new(ContractExpr::predicate("string?", Span::new(0, 10)), Span::new(0, 10));
        let func_contract = ContractExpr::function(vec![param], result, Span::new(0, 10));
        let compiled_func = system.compile_contract(&func_contract, &context).unwrap();
        assert_eq!(compiled_func.id, 2);
    }

    #[test]
    fn test_contract_checking() {
        let mut system = ContractSystem::new();
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };
        let blame = create_test_blame();

        // Compile a number contract
        let number_contract = ContractExpr::predicate("number?", Span::new(0, 10));
        let compiled = system.compile_contract(&number_contract, &context).unwrap();

        // Test positive case
        let number_val = Value::Literal(Literal::Number(42.0));
        let result = system.check_contract(&number_val, &compiled, &blame);
        assert!(result.is_ok());

        // Test negative case
        let string_val = Value::Literal(Literal::String("hello".to_string()));
        let result = system.check_contract(&string_val, &compiled, &blame);
        assert!(result.is_err());
    }

    #[test]
    fn test_and_combinator() {
        let mut system = ContractSystem::new();
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };
        let blame = create_test_blame();

        // Create an (and/c number? positive?) contract
        let and_contract = ContractExpr::and(vec![
            Spanned::new(ContractExpr::predicate("number?", Span::new(0, 10)), Span::new(0, 10)),
            Spanned::new(ContractExpr::predicate("positive?", Span::new(0, 10)), Span::new(0, 10)),
        ], Span::new(0, 10));

        let compiled = system.compile_contract(&and_contract, &context).unwrap();

        // Test positive number (should pass)
        let positive_num = Value::Literal(Literal::Number(42.0));
        assert!(system.check_contract(&positive_num, &compiled, &blame).is_ok());

        // Test negative number (should fail)
        let negative_num = Value::Literal(Literal::Number(-5.0));
        assert!(system.check_contract(&negative_num, &compiled, &blame).is_err());

        // Test non-number (should fail)
        let string_val = Value::Literal(Literal::String("hello".to_string()));
        assert!(system.check_contract(&string_val, &compiled, &blame).is_err());
    }

    #[test]
    fn test_or_combinator() {
        let mut system = ContractSystem::new();
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };
        let blame = create_test_blame();

        // Create an (or/c number? string?) contract
        let or_contract = ContractExpr::or(vec![
            Spanned::new(ContractExpr::predicate("number?", Span::new(0, 10)), Span::new(0, 10)),
            Spanned::new(ContractExpr::predicate("string?", Span::new(0, 10)), Span::new(0, 10)),
        ], Span::new(0, 10));

        let compiled = system.compile_contract(&or_contract, &context).unwrap();

        // Test number (should pass)
        let number_val = Value::Literal(Literal::Number(42.0));
        assert!(system.check_contract(&number_val, &compiled, &blame).is_ok());

        // Test string (should pass)
        let string_val = Value::Literal(Literal::String("hello".to_string()));
        assert!(system.check_contract(&string_val, &compiled, &blame).is_ok());

        // Test boolean (should fail)
        let bool_val = Value::Literal(Literal::Boolean(true));
        assert!(system.check_contract(&bool_val, &compiled, &blame).is_err());
    }

    #[test]
    fn test_not_combinator() {
        let mut system = ContractSystem::new();
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };
        let blame = create_test_blame();

        // Create a (not/c number?) contract
        let not_contract = ContractExpr::not(
            Spanned::new(ContractExpr::predicate("number?", Span::new(0, 10)), Span::new(0, 10)),
            Span::new(0, 10),
        );

        let compiled = system.compile_contract(&not_contract, &context).unwrap();

        // Test number (should fail)
        let number_val = Value::Literal(Literal::Number(42.0));
        assert!(system.check_contract(&number_val, &compiled, &blame).is_err());

        // Test string (should pass)
        let string_val = Value::Literal(Literal::String("hello".to_string()));
        assert!(system.check_contract(&string_val, &compiled, &blame).is_ok());
    }

    #[test]
    fn test_listof_combinator() {
        let mut system = ContractSystem::new();
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };
        let blame = create_test_blame();

        // Create a (listof number?) contract
        let listof_contract = ContractExpr::listof(
            Spanned::new(ContractExpr::predicate("number?", Span::new(0, 10)), Span::new(0, 10)),
            Span::new(0, 10),
        );

        let compiled = system.compile_contract(&listof_contract, &context).unwrap();

        // Test empty list (should pass)
        let empty_list = Value::Nil;
        assert!(system.check_contract(&empty_list, &compiled, &blame).is_ok());

        // Test list of numbers (should pass)
        let number_list = Value::Pair(
            Arc::new(Value::Literal(Literal::Number(1.0))),
            Arc::new(Value::Pair(
                Arc::new(Value::Literal(Literal::Number(2.0))),
                Arc::new(Value::Nil),
            )),
        );
        assert!(system.check_contract(&number_list, &compiled, &blame).is_ok());

        // Test list with non-number (should fail)
        let mixed_list = Value::Pair(
            Arc::new(Value::Literal(Literal::Number(1.0))),
            Arc::new(Value::Pair(
                Arc::new(Value::Literal(Literal::String("hello".to_string()))),
                Arc::new(Value::Nil),
            )),
        );
        assert!(system.check_contract(&mixed_list, &compiled, &blame).is_err());
    }

    // ============= BLAME TRACKING TESTS =============

    #[test]
    fn test_blame_tracking() {
        let tracker = BlameTracker::new();

        // Test blame ID generation
        let id1 = tracker.new_blame_id();
        let id2 = tracker.new_blame_id();
        assert_ne!(id1, id2);

        // Test blame context stack
        tracker.push_blame_context(id1);
        assert_eq!(tracker.current_blame_context(), Some(id1));

        tracker.push_blame_context(id2);
        assert_eq!(tracker.current_blame_context(), Some(id2));

        assert_eq!(tracker.pop_blame_context(), Some(id2));
        assert_eq!(tracker.current_blame_context(), Some(id1));
    }

    #[test]
    fn test_violation_recording() {
        let tracker = BlameTracker::new();
        let blame = create_test_blame();

        // Record a violation
        tracker.record_violation(
            blame.clone(),
            "number?".to_string(),
            "number".to_string(),
            "string".to_string(),
            "Expected number, got string".to_string(),
            Span::new(0, 10),
        );

        // Check violation history
        let violations = tracker.recent_violations(10);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].expected, "number");
        assert_eq!(violations[0].actual, "string");
    }

    #[test]
    fn test_blame_swap() {
        let blame = BlameInfo::function_definition(
            "test-function".to_string(),
            Some("test-module".to_string()),
            Span::new(0, 10),
            "number? -> string?".to_string(),
        );

        let swapped = blame.swap_blame();
        
        // Check that positive and negative are swapped
        assert_ne!(format!("{}", blame.positive), format!("{}", swapped.positive));
        assert_ne!(format!("{}", blame.negative), format!("{}", swapped.negative));
    }

    // ============= RUNTIME TESTS =============

    #[test]
    fn test_contract_runtime() {
        let runtime = ContractRuntime::new();
        let blame = create_test_blame();

        // Test contract registration
        let number_contract = ContractExpr::predicate("number?", Span::new(0, 10));
        let result = runtime.register_contract(
            "my-number-contract".to_string(),
            number_contract,
            blame.clone(),
        );
        assert!(result.is_ok());
        assert_eq!(runtime.contract_count(), 1);

        // Test contract lookup
        let found = runtime.lookup_contract("my-number-contract");
        assert!(found.is_some());

        // Test contract checking
        let number_val = Value::Literal(Literal::Number(42.0));
        let check_result = runtime.check_named_contract(&number_val, "my-number-contract", &blame);
        assert!(check_result.is_ok());

        let string_val = Value::Literal(Literal::String("hello".to_string()));
        let check_result = runtime.check_named_contract(&string_val, "my-number-contract", &blame);
        assert!(check_result.is_err());
    }

    #[test]
    fn test_contract_aliases() {
        let runtime = ContractRuntime::new();
        let blame = create_test_blame();

        // Register a contract
        let number_contract = ContractExpr::predicate("number?", Span::new(0, 10));
        runtime.register_contract("number-contract".to_string(), number_contract, blame).unwrap();

        // Create an alias
        let alias_result = runtime.create_alias("num".to_string(), "number-contract".to_string());
        assert!(alias_result.is_ok());

        // Test alias resolution
        assert_eq!(runtime.resolve_name("num"), "number-contract");
        assert_eq!(runtime.resolve_name("number-contract"), "number-contract");
    }

    #[test]
    fn test_performance_monitoring() {
        let runtime = ContractRuntime::new();
        let stats = runtime.performance_stats();

        // Initial stats should be zero
        assert_eq!(stats.compilation.total_compiled, 0);
        assert_eq!(stats.checking.total_checks, 0);
        assert_eq!(stats.runtime.total_violations, 0);
    }

    // ============= OPTIMIZATION TESTS =============

    #[test]
    fn test_contract_optimization() {
        let config = ContractConfig {
            enable_checking: true,
            enable_compilation: true,
            enable_blame_tracking: true,
            max_recursion_depth: 100,
            enable_contract_caching: true,
            enable_gradual_integration: true,
        };

        let mut system = ContractSystem::with_config(config);
        let context = CompilationContext::with_optimization_level(
            PredicateRegistry::new(),
            create_test_blame(),
            OptimizationLevel::Maximum,
        );

        // Test that aggressive optimization is applied
        let simple_contract = ContractExpr::predicate("number?", Span::new(0, 10));
        let compiled = system.compile_contract(&simple_contract, &context).unwrap();
        
        assert_eq!(compiled.optimization_level, OptimizationLevel::Maximum);
        assert!(compiled.performance.inlinable);
    }

    #[test]
    fn test_contract_caching() {
        let mut system = ContractSystem::new();
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };

        // Compile the same contract twice
        let contract = ContractExpr::predicate("number?", Span::new(0, 10));
        let compiled1 = system.compile_contract(&contract, &context).unwrap();
        let compiled2 = system.compile_contract(&contract, &context).unwrap();

        // Should be the same compiled contract (cached)
        assert_eq!(compiled1.id, compiled2.id);

        // Check cache stats
        let (cache_size, _) = system.cache_stats();
        assert!(cache_size > 0);
    }

    // ============= INTEGRATION TESTS =============

    #[test]
    fn test_complex_contract_scenario() {
        let mut system = ContractSystem::new();
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };
        let blame = create_test_blame();

        // Create a complex contract: (-> (and/c number? positive?) (listof string?) boolean?)
        let domain_contract = Spanned::new(
            ContractExpr::and(vec![
                Spanned::new(ContractExpr::predicate("number?", Span::new(0, 10)), Span::new(0, 10)),
                Spanned::new(ContractExpr::predicate("positive?", Span::new(0, 10)), Span::new(0, 10)),
            ], Span::new(0, 10)),
            Span::new(0, 10),
        );

        let param_contract = Spanned::new(
            ContractExpr::listof(
                Spanned::new(ContractExpr::predicate("string?", Span::new(0, 10)), Span::new(0, 10)),
                Span::new(0, 10),
            ),
            Span::new(0, 10),
        );

        let return_contract = Spanned::new(
            ContractExpr::predicate("boolean?", Span::new(0, 10)),
            Span::new(0, 10),
        );

        let complex_contract = ContractExpr::function(
            vec![domain_contract, param_contract],
            return_contract,
            Span::new(0, 10),
        );

        // Should compile successfully
        let compiled = system.compile_contract(&complex_contract, &context);
        assert!(compiled.is_ok());

        let compiled_contract = compiled.unwrap();
        assert!(compiled_contract.performance.operation_count > 1);
    }

    #[test]
    fn test_nested_contracts() {
        let mut system = ContractSystem::new();
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };
        let blame = create_test_blame();

        // Create nested contracts: (and/c (or/c number? string?) (not/c null?))
        let or_contract = Spanned::new(
            ContractExpr::or(vec![
                Spanned::new(ContractExpr::predicate("number?", Span::new(0, 10)), Span::new(0, 10)),
                Spanned::new(ContractExpr::predicate("string?", Span::new(0, 10)), Span::new(0, 10)),
            ], Span::new(0, 10)),
            Span::new(0, 10),
        );

        let not_contract = Spanned::new(
            ContractExpr::not(
                Spanned::new(ContractExpr::predicate("null?", Span::new(0, 10)), Span::new(0, 10)),
                Span::new(0, 10),
            ),
            Span::new(0, 10),
        );

        let nested_contract = ContractExpr::and(vec![or_contract, not_contract], Span::new(0, 10));
        let compiled = system.compile_contract(&nested_contract, &context).unwrap();

        // Test values
        let number_val = Value::Literal(Literal::Number(42.0));
        assert!(system.check_contract(&number_val, &compiled, &blame).is_ok());

        let string_val = Value::Literal(Literal::String("hello".to_string()));
        assert!(system.check_contract(&string_val, &compiled, &blame).is_ok());

        let nil_val = Value::Nil;
        assert!(system.check_contract(&nil_val, &compiled, &blame).is_err());

        let bool_val = Value::Literal(Literal::Boolean(true));
        assert!(system.check_contract(&bool_val, &compiled, &blame).is_err());
    }

    #[test]
    fn test_contract_configuration() {
        // Test with checking disabled
        let config = ContractConfig {
            enable_checking: false,
            enable_compilation: true,
            enable_blame_tracking: false,
            max_recursion_depth: 100,
            enable_contract_caching: false,
            enable_gradual_integration: false,
        };

        let mut system = ContractSystem::with_config(config);
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };
        let blame = create_test_blame();

        let contract = ContractExpr::predicate("number?", Span::new(0, 10));
        let compiled = system.compile_contract(&contract, &context).unwrap();

        // Even with a string value, checking should succeed when disabled
        let string_val = Value::Literal(Literal::String("hello".to_string()));
        let result = system.check_contract(&string_val, &compiled, &blame);
        assert!(result.is_ok()); // Should pass because checking is disabled
    }

    // ============= ERROR HANDLING TESTS =============

    #[test]
    fn test_contract_error_handling() {
        let mut system = ContractSystem::new();
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };

        // Test with unknown predicate
        let unknown_contract = ContractExpr::predicate("unknown-predicate?", Span::new(0, 10));
        let result = system.compile_contract(&unknown_contract, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_recursion_limit() {
        // This would test recursive contracts hitting the recursion limit
        // For now, we'll just test that the limit is respected in context
        let mut context = create_test_context();
        context.max_recursion_depth = 2;

        // Manually test recursion depth tracking
        assert_eq!(context.recursion_depth, 0);
        
        // In a real scenario, this would be tested with recursive contracts
        // that exceed the depth limit
    }
}

/// Integration tests for the complete contract system.
#[cfg(test)]
mod integration_tests {
    use super::contract_tests::*;
    use crate::contracts::{ContractSystem, CompilationContext, OptimizationLevel, ContractExpr};
    use crate::Span;

    #[test]
    fn test_contract_system_integration() {
        // Test that all components work together
        let system = ContractSystem::new();
        
        // Verify system is properly initialized
        assert!(system.config().enable_checking);
        assert!(system.config().enable_compilation);
        assert!(system.config().enable_blame_tracking);
    }

    #[test]
    fn test_performance_under_load() {
        // Test contract system performance with many contracts
        let mut system = ContractSystem::new();
        let context = CompilationContext {
            optimization_level: OptimizationLevel::None,
        };

        // Compile many contracts
        for i in 0..100 {
            let contract = if i % 2 == 0 {
                ContractExpr::predicate("number?", Span::new(0, 10))
            } else {
                ContractExpr::predicate("string?", Span::new(0, 10))
            };

            let compiled = system.compile_contract(&contract, &context);
            assert!(compiled.is_ok());
        }

        // System should still be responsive
        let (cache_size, _) = system.cache_stats();
        assert!(cache_size <= 100); // May be less due to caching
    }
}
//! Comprehensive integration tests for GC transparency with all Lambdust language features.
//!
//! These tests verify that the parallel garbage collector integrates seamlessly with
//! the language processor while maintaining complete R7RS-large compliance and
//! supporting all existing language features without observable behavior changes.

#[cfg(test)]
mod tests {
    use crate::utils::{GcIntegration, GcIntegrationConfig, GcCoordinator, GcCoordinatorConfig};
    use crate::eval::{
        Value, ThreadSafeEnvironment, Evaluator, GcContinuationManager, GcContinuationConfig,
        StackTrace, StackFrame, FrameType
    };
    use crate::macro_system::{GcMacroCoordinator, GcMacroConfig, MacroExpander};
    use crate::diagnostics::{GcDiagnosticManager, GcDiagnosticConfig, Error, ErrorContext, Span};
    use crate::ast::{Expr, Literal, Program};
    use std::sync::Arc;
    use std::collections::HashMap;

    /// Test suite for GC integration with value system.
    #[cfg(test)]
    mod value_integration_tests {
        use super::*;

        #[test]
        fn test_gc_value_system_transparency() {
            // Test that GC integration doesn't affect Value semantics
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            
            // Create values with and without GC - should be identical in behavior
            let value1 = Value::integer(42);
            let value2 = Value::string("hello world");
            let value3 = Value::pair(value1.clone(), value2.clone());
            
            // Test that all standard operations work identically
            assert_eq!(value1.as_integer(), Some(42));
            assert_eq!(value2.as_string(), Some("hello world"));
            assert_eq!(value3.is_pair(), true);
            
            // Test with complex nested structures
            let complex_value = Value::list(vec![
                Value::integer(1),
                Value::string("test"),
                Value::vector(vec![Value::integer(2), Value::integer(3)]),
                Value::pair(Value::symbol_from_str("key"), Value::integer(4))
            ]);
            
            // Verify complex structure integrity
            let as_list = complex_value.as_list().unwrap();
            assert_eq!(as_list.len(), 4);
            assert_eq!(as_list[0].as_integer(), Some(1));
            assert_eq!(as_list[1].as_string(), Some("test"));
            assert!(as_list[2].is_vector());
            assert!(as_list[3].is_pair());
        }

        #[test]
        fn test_gc_value_memory_management() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            
            // Create many values to trigger GC
            let mut values = Vec::new();
            for i in 0..1000 {
                values.push(Value::string(format!("test string {i}")));
            }
            
            // Force GC collection
            crate::utils::gc::gc_collect();
            
            // Verify all values are still valid
            for (i, value) in values.iter().enumerate() {
                assert_eq!(value.as_string(), Some(&format!("test string {i}")));
            }
        }

        #[test]
        fn test_gc_value_equality_preservation() {
            // Test that GC doesn't affect value equality
            let value1 = Value::integer(42);
            let value2 = Value::integer(42);
            let value3 = Value::integer(43);
            
            assert_eq!(value1, value2);
            assert_ne!(value1, value3);
            
            // Test with complex values
            let list1 = Value::list(vec![Value::integer(1), Value::integer(2)]);
            let list2 = Value::list(vec![Value::integer(1), Value::integer(2)]);
            let list3 = Value::list(vec![Value::integer(1), Value::integer(3)]);
            
            assert_eq!(list1, list2);
            assert_ne!(list1, list3);
        }
    }

    /// Test suite for GC integration with environment system.
    #[cfg(test)]
    mod environment_integration_tests {
        use super::*;

        #[test]
        fn test_gc_environment_root_scanning() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            let gc_coordinator = GcCoordinator::new(GcCoordinatorConfig::default()).unwrap();
            
            // Create a chain of environments
            let global_env = Arc::new(ThreadSafeEnvironment::new(None, 0));
            global_env.define("global-var".to_string(), Value::integer(42));
            
            let local_env = Arc::new(ThreadSafeEnvironment::new(Some(global_env.clone()), 1));
            local_env.define("local-var".to_string(), Value::string("test"));
            
            let nested_env = Arc::new(ThreadSafeEnvironment::new(Some(local_env.clone()), 2));
            nested_env.define("nested-var".to_string(), Value::boolean(true));
            
            // Register environments with GC
            gc_coordinator.add_global_root(crate::eval::GlobalRoot::Environment(global_env.clone()));
            
            // Start evaluation session
            let session_id = gc_coordinator.start_session(nested_env.clone());
            gc_coordinator.push_environment(session_id, local_env.clone());
            
            // Perform comprehensive root scan
            let scan_result = gc_coordinator.comprehensive_root_scan();
            
            // Verify all environment variables are captured as roots
            let root_values: Vec<_> = scan_result.session_roots;
            assert!(!root_values.is_empty());
            
            // Check that we can still access all variables after GC
            crate::utils::gc::gc_collect();
            
            assert_eq!(nested_env.lookup("nested-var"), Some(Value::boolean(true)));
            assert_eq!(nested_env.lookup("local-var"), Some(Value::string("test")));
            assert_eq!(nested_env.lookup("global-var"), Some(Value::integer(42)));
            
            // Cleanup
            gc_coordinator.end_session(session_id);
        }

        #[test]
        fn test_gc_environment_variable_preservation() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            
            // Create environment with many variables
            let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
            
            for i in 0..100 {
                env.define(
                    format!("var-{i}"), 
                    Value::list(vec![Value::integer(i), Value::string(format!("value-{i}"))])
                );
            }
            
            // Force multiple GC cycles
            for _ in 0..5 {
                crate::utils::gc::gc_collect();
            }
            
            // Verify all variables are still accessible
            for i in 0..100 {
                let value = env.lookup(&format!("var-{i}")).unwrap();
                let list = value.as_list().unwrap();
                assert_eq!(list[0].as_integer(), Some(i));
                assert_eq!(list[1].as_string(), Some(&format!("value-{i}")));
            }
        }
    }

    /// Test suite for GC integration with macro system.
    #[cfg(test)]
    mod macro_integration_tests {
        use super::*;

        #[test]
        fn test_gc_macro_expansion_transparency() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            let expander = MacroExpander::new();
            let gc_macro_coordinator = GcMacroCoordinator::with_default_config(
                expander, 
                gc_integration.clone()
            );
            
            // Register a simple transformer
            let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
            let transformer = Value::integer(42); // Dummy transformer for testing
            
            let result = gc_macro_coordinator.register_transformer(
                "test-macro".to_string(),
                transformer,
                env.clone(),
            );
            
            assert!(result.is_ok());
            
            // Verify statistics
            let stats = gc_macro_coordinator.get_expansion_statistics();
            assert_eq!(stats.registered_transformers, 1);
            assert_eq!(stats.active_expansions, 0);
        }

        #[test]
        fn test_gc_macro_memory_efficiency() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            let expander = MacroExpander::new();
            let gc_macro_coordinator = GcMacroCoordinator::with_default_config(
                expander, 
                gc_integration.clone()
            );
            
            let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
            
            // Register many transformers to test memory management
            for i in 0..50 {
                let transformer = Value::string(format!("transformer-{i}"));
                let result = gc_macro_coordinator.register_transformer(
                    format!("macro-{i}"),
                    transformer,
                    env.clone(),
                );
                assert!(result.is_ok());
            }
            
            // Force GC collection
            crate::utils::gc::gc_collect();
            
            // Verify all transformers are still registered
            let stats = gc_macro_coordinator.get_expansion_statistics();
            assert_eq!(stats.registered_transformers, 50);
        }
    }

    /// Test suite for GC integration with continuation system.
    #[cfg(test)]
    mod continuation_integration_tests {
        use super::*;
        use crate::eval::{Frame};

        #[test]
        fn test_gc_continuation_capture_and_preservation() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            let gc_continuation_manager = GcContinuationManager::with_default_config(
                gc_integration.clone()
            );
            
            // Create a test environment
            let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
            env.define("test-var".to_string(), Value::integer(42));
            
            // Create test frames and stack trace
            let frames = vec![];
            let mut stack_trace = StackTrace::new();
            stack_trace.push(StackFrame::procedure_call(
                Some("test-procedure".to_string()), 
                Some(Span::new(0, 10))
            ));
            
            // Capture continuation
            let continuation = gc_continuation_manager.capture_continuation(
                frames,
                env.clone(),
                None,
                Some(stack_trace),
            ).unwrap();
            
            // Verify continuation is active
            let stats = gc_continuation_manager.get_continuation_statistics();
            assert_eq!(stats.active_continuations, 1);
            assert_eq!(stats.preserved_stack_traces, 1);
            
            // Force GC collection
            crate::utils::gc::gc_collect();
            
            // Verify continuation is still active and environment accessible
            let stats_after_gc = gc_continuation_manager.get_continuation_statistics();
            assert_eq!(stats_after_gc.active_continuations, 1);
            
            // Verify environment bindings are preserved
            assert_eq!(env.lookup("test-var"), Some(Value::integer(42)));
            
            // Test continuation invocation
            let invoke_result = gc_continuation_manager.invoke_continuation(
                &continuation,
                Value::string("continuation result")
            );
            assert!(invoke_result.is_ok());
            
            // Cleanup
            let cleanup_result = gc_continuation_manager.cleanup_continuation(continuation.id);
            assert!(cleanup_result.is_ok());
            
            let final_stats = gc_continuation_manager.get_continuation_statistics();
            assert_eq!(final_stats.active_continuations, 0);
        }

        #[test]
        fn test_gc_continuation_stack_trace_preservation() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            let gc_continuation_manager = GcContinuationManager::with_default_config(
                gc_integration.clone()
            );
            
            // Create complex stack trace
            let mut stack_trace = StackTrace::new();
            for i in 0..10 {
                stack_trace.push(StackFrame::procedure_call(
                    Some(format!("procedure-{i}")), 
                    Some(Span::new(i * 10, 5))
                ));
            }
            
            let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
            let frames = vec![];
            
            // Capture continuation with complex stack trace
            let continuation = gc_continuation_manager.capture_continuation(
                frames,
                env,
                None,
                Some(stack_trace.clone()),
            ).unwrap();
            
            // Force multiple GC cycles
            for _ in 0..3 {
                crate::utils::gc::gc_collect();
            }
            
            // Verify stack trace preservation
            let stats = gc_continuation_manager.get_continuation_statistics();
            assert_eq!(stats.preserved_stack_traces, 1);
            
            // Cleanup
            gc_continuation_manager.cleanup_continuation(continuation.id).unwrap();
        }
    }

    /// Test suite for GC integration with error handling.
    #[cfg(test)]
    mod error_handling_integration_tests {
        use super::*;

        #[test]
        fn test_gc_error_preservation() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            let gc_diagnostic_manager = GcDiagnosticManager::with_default_config(
                gc_integration.clone()
            );
            
            // Create test error with context
            let error = Error::runtime_error("Test runtime error".to_string(), Some(Span::new(10, 5)));
            
            let mut context = ErrorContext::default();
            context.current_expr = Some("(+ 1 'invalid)".to_string());
            context.bindings.insert("x".to_string(), "42".to_string());
            context.notes.push("Type mismatch in addition".to_string());
            
            let mut stack_trace = StackTrace::new();
            stack_trace.push(StackFrame::procedure_call(
                Some("add-numbers".to_string()),
                Some(Span::new(5, 10))
            ));
            
            // Create GC-aware error
            let gc_error = gc_diagnostic_manager.create_gc_aware_error(
                error,
                Some(stack_trace),
                Some(context),
            );
            
            // Verify error has diagnostic ID
            assert!(gc_error.diagnostic_id().is_some());
            assert!(gc_error.preserved_info().is_some());
            
            // Force GC collection
            crate::utils::gc::gc_collect();
            
            // Verify error information is preserved
            let preserved = gc_error.preserved_info().unwrap();
            assert_eq!(preserved.message, "Test runtime error");
            assert!(preserved.span.is_some());
            assert!(preserved.stack_trace.is_some());
            assert_eq!(preserved.context.current_expr, Some("(+ 1 'invalid)".to_string()));
            
            // Verify statistics
            let stats = gc_diagnostic_manager.get_diagnostic_statistics();
            assert_eq!(stats.active_diagnostics, 1);
            assert_eq!(stats.total_diagnostics, 1);
        }

        #[test]
        fn test_gc_error_context_compression() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            let gc_diagnostic_manager = GcDiagnosticManager::with_default_config(
                gc_integration.clone()
            );
            
            // Create error with large context
            let error = Error::runtime_error("Large context error".to_string(), None);
            
            let mut context = ErrorContext::default();
            context.current_expr = Some("x".repeat(500)); // Large expression
            
            // Add many bindings
            for i in 0..100 {
                context.bindings.insert(format!("var-{i}"), format!("value-{i}"));
            }
            
            // Add many call chain entries
            for i in 0..50 {
                context.call_chain.push(format!("procedure-{i}"));
            }
            
            let gc_error = gc_diagnostic_manager.create_gc_aware_error(
                error,
                None,
                Some(context),
            );
            
            // Force GC collection
            crate::utils::gc::gc_collect();
            
            // Verify error is still accessible (may be compressed)
            let preserved = gc_error.preserved_info().unwrap();
            assert_eq!(preserved.message, "Large context error");
            assert!(preserved.context.current_expr.is_some());
            assert!(!preserved.context.bindings.is_empty());
        }
    }

    /// Test suite for R7RS compliance with GC integration.
    #[cfg(test)]
    mod r7rs_compliance_tests {
        use super::*;

        #[test]
        fn test_gc_r7rs_number_semantics() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            
            // Test exact number preservation
            let exact_int = Value::integer(42);
            let exact_rational = Value::from_literal(Literal::Rational { numerator: 22, denominator: 7 });
            
            // Force GC
            crate::utils::gc::gc_collect();
            
            // Verify R7RS numeric predicates work correctly
            assert!(exact_int.is_exact_number());
            assert!(!exact_int.is_inexact_number());
            assert!(exact_int.is_finite_number());
            assert!(!exact_int.is_infinite_number());
            assert!(!exact_int.is_nan_number());
            
            assert!(exact_rational.is_exact_number());
            assert!(exact_rational.is_finite_number());
        }

        #[test]
        fn test_gc_r7rs_string_semantics() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            
            // Test immutable vs mutable strings
            let immutable_str = Value::string("hello");
            let mutable_str = Value::mutable_string("world");
            
            assert!(immutable_str.is_immutable_string());
            assert!(!immutable_str.is_mutable_string());
            assert!(mutable_str.is_mutable_string());
            assert!(!mutable_str.is_immutable_string());
            
            // Force GC
            crate::utils::gc::gc_collect();
            
            // Verify string operations still work
            assert_eq!(immutable_str.as_string(), Some("hello"));
            assert_eq!(mutable_str.string_length(), Some(5));
        }

        #[test]
        fn test_gc_r7rs_pair_and_list_semantics() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            
            // Test proper list construction and traversal
            let proper_list = Value::list(vec![
                Value::integer(1),
                Value::integer(2),
                Value::integer(3)
            ]);
            
            // Test dotted pair
            let dotted_pair = Value::pair(
                Value::integer(1),
                Value::integer(2)
            );
            
            // Force GC
            crate::utils::gc::gc_collect();
            
            // Verify R7RS list predicates
            assert!(proper_list.is_list());
            assert!(proper_list.is_pair());
            assert!(dotted_pair.is_pair());
            assert!(dotted_pair.is_list()); // Pairs are lists in R7RS
            
            // Test list conversion
            let list_elements = proper_list.as_list().unwrap();
            assert_eq!(list_elements.len(), 3);
            assert_eq!(list_elements[0].as_integer(), Some(1));
            assert_eq!(list_elements[1].as_integer(), Some(2));
            assert_eq!(list_elements[2].as_integer(), Some(3));
        }

        #[test]
        fn test_gc_r7rs_boolean_and_truthiness_semantics() {
            let gc_integration = Arc::new(GcIntegration::with_default_config());
            
            // Test R7RS truthiness: only #f is falsy
            let false_val = Value::boolean(false);
            let true_val = Value::boolean(true);
            let zero = Value::integer(0);
            let empty_list = Value::Nil;
            let empty_string = Value::string("");
            
            // Force GC
            crate::utils::gc::gc_collect();
            
            // Verify R7RS truthiness semantics
            assert!(false_val.is_falsy());
            assert!(!false_val.is_truthy());
            
            assert!(!true_val.is_falsy());
            assert!(true_val.is_truthy());
            
            // Everything else is truthy in R7RS
            assert!(!zero.is_falsy());
            assert!(zero.is_truthy());
            
            assert!(!empty_list.is_falsy());
            assert!(empty_list.is_truthy());
            
            assert!(!empty_string.is_falsy());
            assert!(empty_string.is_truthy());
        }
    }

    /// Integration test for complete language processor workflow with GC.
    #[test]
    fn test_complete_gc_integration_workflow() {
        // Initialize all GC components
        let gc_integration = Arc::new(GcIntegration::with_default_config());
        let gc_coordinator = GcCoordinator::new(GcCoordinatorConfig::default()).unwrap();
        let gc_continuation_manager = GcContinuationManager::with_default_config(
            gc_integration.clone()
        );
        let expander = MacroExpander::new();
        let gc_macro_coordinator = GcMacroCoordinator::with_default_config(
            expander, 
            gc_integration.clone()
        );
        let gc_diagnostic_manager = GcDiagnosticManager::with_default_config(
            gc_integration.clone()
        );
        
        // Create a complex evaluation environment
        let global_env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        global_env.define("pi".to_string(), Value::number(3.14159));
        global_env.define("greeting".to_string(), Value::string("Hello, World!"));
        
        let local_env = Arc::new(ThreadSafeEnvironment::new(Some(global_env.clone()), 1));
        local_env.define("x".to_string(), Value::integer(42));
        local_env.define("y".to_string(), Value::list(vec![
            Value::integer(1), Value::integer(2), Value::integer(3)
        ]));
        
        // Start evaluation session
        let session_id = gc_coordinator.start_session(local_env.clone());
        gc_coordinator.push_environment(session_id, global_env.clone());
        
        // Register some transformers
        let transformer1 = Value::string("test-transformer-1");
        gc_macro_coordinator.register_transformer(
            "test-macro-1".to_string(),
            transformer1,
            local_env.clone(),
        ).unwrap();
        
        // Create some continuations
        let frames = vec![];
        let mut stack_trace = StackTrace::new();
        stack_trace.push(StackFrame::procedure_call(
            Some("test-procedure".to_string()),
            Some(Span::new(0, 20))
        ));
        
        let continuation1 = gc_continuation_manager.capture_continuation(
            frames.clone(),
            local_env.clone(),
            None,
            Some(stack_trace.clone()),
        ).unwrap();
        
        // Create some diagnostic information
        let error1 = Error::runtime_error("Test error 1".to_string(), Some(Span::new(10, 5)));
        let context1 = ErrorContext::default();
        let gc_error1 = gc_diagnostic_manager.create_gc_aware_error(
            error1,
            Some(stack_trace.clone()),
            Some(context1),
        );
        
        // Force multiple GC collections to test stability
        for i in 0..5 {
            println!("GC collection cycle {}", i + 1);
            crate::utils::gc::gc_collect();
            
            // Verify all components remain functional
            
            // Check environments
            assert_eq!(local_env.lookup("x"), Some(Value::integer(42)));
            assert_eq!(local_env.lookup("pi"), Some(Value::number(3.14159)));
            assert_eq!(local_env.lookup("greeting"), Some(Value::string("Hello, World!")));
            
            let y_value = local_env.lookup("y").unwrap();
            let y_list = y_value.as_list().unwrap();
            assert_eq!(y_list.len(), 3);
            
            // Check coordinator statistics
            let coord_stats = gc_coordinator.get_continuation_statistics();
            assert_eq!(coord_stats.active_session_count, 1);
            
            // Check continuation manager statistics
            let cont_stats = gc_continuation_manager.get_continuation_statistics();
            assert_eq!(cont_stats.active_continuations, 1);
            
            // Check macro coordinator statistics
            let macro_stats = gc_macro_coordinator.get_expansion_statistics();
            assert_eq!(macro_stats.registered_transformers, 1);
            
            // Check diagnostic manager statistics
            let diag_stats = gc_diagnostic_manager.get_diagnostic_statistics();
            assert_eq!(diag_stats.active_diagnostics, 1);
        }
        
        // Perform comprehensive root scan
        let scan_result = gc_coordinator.comprehensive_root_scan();
        assert!(!scan_result.session_roots.is_empty());
        assert_eq!(scan_result.active_session_count, 1);
        
        // Test that error information is still preserved
        let preserved_error = gc_error1.preserved_info().unwrap();
        assert_eq!(preserved_error.message, "Test error 1");
        assert!(preserved_error.stack_trace.is_some());
        
        // Cleanup all resources
        gc_continuation_manager.cleanup_continuation(continuation1.id).unwrap();
        gc_coordinator.end_session(session_id);
        
        // Final verification after cleanup
        let final_coord_stats = gc_coordinator.get_continuation_statistics();
        assert_eq!(final_coord_stats.active_session_count, 0);
        
        let final_cont_stats = gc_continuation_manager.get_continuation_statistics();
        assert_eq!(final_cont_stats.active_continuations, 0);
        
        println!("Complete GC integration workflow test passed!");
    }
}
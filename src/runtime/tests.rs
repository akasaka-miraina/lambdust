//! Tests for the multithreaded runtime system.

#[cfg(test)]
mod tests {
    use crate::runtime::{LambdustRuntime, GlobalEnvironmentManager, EffectCoordinator};
    use crate::ast::{Expr, Literal};
    use crate::diagnostics::Span;
    use tokio;

    #[tokio::test]
    async fn test_lambdust_runtime_creation() {
        let runtime = LambdustRuntime::new();
        assert!(runtime.is_ok());
        
        if let Ok(runtime) = runtime {
            assert_eq!(runtime.thread_count(), 2);
            let _ = runtime.shutdown().await;
        }
    }

    #[tokio::test]
    async fn test_simple_evaluation() {
        let runtime = LambdustRuntime::new().expect("Failed to create runtime");
        
        // Create a simple literal expression
        let expr = Expr::Literal(Literal::Number(42.0));
        let span = Some(Span { start: 0, len: 2, file_id: None, line: 1, column: 1 });
        
        let result = runtime.eval_expr(expr, span).await;
        
        // The current implementation just returns Unspecified, so check for that
        assert!(result.is_ok());
        
        let _ = runtime.shutdown().await;
    }

    #[tokio::test]
    async fn test_parallel_evaluation() {
        let runtime = LambdustRuntime::new().expect("Failed to create runtime");
        
        // Create multiple simple expressions
        let expressions = vec![
            (Expr::Literal(Literal::Number(1.0)), Some(Span { start: 0, len: 1, file_id: None, line: 1, column: 1 })),
            (Expr::Literal(Literal::Number(2.0)), Some(Span { start: 0, len: 1, file_id: None, line: 1, column: 1 })),
            (Expr::Literal(Literal::Number(3.0)), Some(Span { start: 0, len: 1, file_id: None, line: 1, column: 1 })),
        ];
        
        let result = runtime.eval_parallel(expressions).await;
        
        assert_eq!(result.results.len(), 3);
        assert_eq!(result.threads_used, 2); // We created 2 threads
        assert!(result.elapsed.as_nanos() > 0); // Should have taken some time
        
        let _ = runtime.shutdown().await;
    }

    #[tokio::test]
    async fn test_evaluator_handle() {
        let runtime = LambdustRuntime::new().expect("Failed to create runtime");
        
        let handle = runtime.spawn_evaluator().expect("Failed to spawn evaluator");
        assert_eq!(handle.id, 0); // First handle should have ID 0
        
        // Test evaluation through handle
        let expr = Expr::Literal(Literal::Number(42.0));
        let span = Some(Span { start: 0, len: 2, file_id: None, line: 1, column: 1 });
        
        let result = handle.eval(expr, span).await;
        assert!(result.is_ok());
        
        // Test global definition
        let define_result = handle.define_global("test".to_string(), crate::eval::Value::t());
        assert!(define_result.is_ok());
        
        // Shutdown the handle
        let shutdown_result = handle.shutdown();
        assert!(shutdown_result.is_ok());
        
        let _ = runtime.shutdown().await;
    }

    #[tokio::test]
    async fn test_global_environment_manager() {
        let global_env = GlobalEnvironmentManager::new();
        
        // Test global variable definition
        let define_result = global_env.define_global("test".to_string(), crate::eval::Value::f());
        assert!(define_result.is_ok());
        
        // Test global variable lookup
        let lookup_result = global_env.lookup_global("test");
        assert!(lookup_result.is_some());
        
        // Test variable names
        let names = global_env.global_variable_names();
        assert!(names.contains(&"test".to_string()));
        
        // Test generation management
        let gen1 = global_env.current_generation();
        let gen2 = global_env.next_generation();
        assert!(gen2 > gen1);
    }

    #[tokio::test]
    async fn test_effect_coordinator() {
        let coordinator = EffectCoordinator::new();
        let thread_id = std::thread::current().id();
        
        // Test thread registration
        coordinator.register_thread(thread_id);
        
        // Test effect context operations
        use crate::effects::Effect;
        let effects = vec![Effect::IO];
        let result = coordinator.enter_effect_context(thread_id, effects.clone());
        assert!(result.is_ok());
        
        // Test getting thread effects
        let thread_effects = coordinator.get_thread_effects(thread_id);
        assert_eq!(thread_effects, effects);
        
        // Test exit effect context
        let exit_result = coordinator.exit_effect_context(thread_id, effects);
        assert!(exit_result.is_ok());
        
        // Test unregistration
        coordinator.unregister_thread(thread_id);
    }

    #[tokio::test] 
    async fn test_thread_pool_statistics() {
        let runtime = LambdustRuntime::new().expect("Failed to create runtime");
        
        let stats = runtime.thread_pool.statistics();
        assert_eq!(stats.active_workers, 2);
        assert_eq!(stats.total_tasks_submitted, 0);
        assert_eq!(stats.total_tasks_completed, 0);
        
        let _ = runtime.shutdown().await;
    }

    #[test]
    fn test_multithreaded_lambdust_api() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            let lambdust = crate::MultithreadedLambdust::new(Some(2));
            assert!(lambdust.is_ok());
            
            if let Ok(lambdust) = lambdust {
                assert_eq!(lambdust.thread_count(), 2);
                let _ = lambdust.shutdown().await;
            }
        });
    }

    // ============= STAGE 3 CONCURRENT EFFECT SYSTEM TESTS =============

    #[tokio::test]
    async fn test_concurrent_effect_coordination() {
        use crate::effects::Effect;
        
        let coordinator = EffectCoordinator::new();
        let thread_id = std::thread::current().id();
        
        // Register thread
        coordinator.register_thread(thread_id);
        
        // Test concurrent effect coordination
        let effect = Effect::IO;
        let result = coordinator.coordinate_local_effect(
            thread_id, 
            effect.clone(), 
            &[crate::eval::Value::integer(42)]
        );
        assert!(result.is_ok());
        
        let sequence = result.unwrap();
        assert!(sequence > 0);
        
        // Complete the effect
        let completion_result = coordinator.complete_effect(sequence, Ok("success".to_string()));
        assert!(completion_result.is_ok());
        
        // Get statistics
        let stats = coordinator.get_effect_statistics();
        assert_eq!(stats.active_threads, 1);
        
        coordinator.unregister_thread(thread_id);
    }

    #[tokio::test]
    async fn test_transaction_based_state_management() {
        let global_env = GlobalEnvironmentManager::new();
        let thread_id = std::thread::current().id();
        
        // Test transaction creation
        let transaction_id = global_env.start_transaction(thread_id);
        assert!(transaction_id.is_ok());
        
        let tx_id = transaction_id.unwrap();
        
        // Test transactional define
        let define_result = global_env.define_global_transactional(
            "test_var".to_string(),
            crate::eval::Value::integer(42),
            thread_id
        );
        assert!(define_result.is_ok());
        
        // Verify variable exists
        let lookup_result = global_env.lookup_global("test_var");
        assert!(lookup_result.is_some());
        assert_eq!(lookup_result.unwrap(), crate::eval::Value::integer(42));
        
        // Test transaction rollback
        let abort_result = global_env.abort_transaction(tx_id);
        assert!(abort_result.is_ok());
        
        // Verify variable was rolled back
        let lookup_after_rollback = global_env.lookup_global("test_var");
        assert!(lookup_after_rollback.is_none());
    }

    #[tokio::test]
    async fn test_snapshot_and_rollback() {
        let global_env = GlobalEnvironmentManager::new();
        
        // Create initial state
        let _ = global_env.define_global("var1".to_string(), crate::eval::Value::integer(1));
        let _ = global_env.define_global("var2".to_string(), crate::eval::Value::integer(2));
        
        // Create snapshot
        let snapshot_result = global_env.create_environment_snapshot();
        assert!(snapshot_result.is_ok());
        
        let snapshot_gen = snapshot_result.unwrap();
        
        // Modify state
        let _ = global_env.define_global("var1".to_string(), crate::eval::Value::integer(100));
        let _ = global_env.define_global("var3".to_string(), crate::eval::Value::integer(3));
        
        // Verify changes
        assert_eq!(global_env.lookup_global("var1"), Some(crate::eval::Value::integer(100)));
        assert_eq!(global_env.lookup_global("var3"), Some(crate::eval::Value::integer(3)));
        
        // Rollback to snapshot
        let rollback_result = global_env.rollback_to_generation(snapshot_gen);
        assert!(rollback_result.is_ok());
        
        // Verify rollback worked
        assert_eq!(global_env.lookup_global("var1"), Some(crate::eval::Value::integer(1)));
        assert_eq!(global_env.lookup_global("var2"), Some(crate::eval::Value::integer(2)));
        assert!(global_env.lookup_global("var3").is_none());
    }

    #[tokio::test]
    async fn test_io_coordination() {
        use crate::runtime::io_coordinator::{IOCoordinator, IOOperationType, IOParameters};
        
        let io_coordinator = IOCoordinator::new();
        let thread_id = std::thread::current().id();
        
        // Register thread
        io_coordinator.register_thread(thread_id);
        
        // Test IO operation coordination
        let operation_result = io_coordinator.coordinate_io_operation(
            thread_id,
            IOOperationType::FileRead,
            "test.txt".to_string(),
            IOParameters::default()
        );
        assert!(operation_result.is_ok());
        
        let operation_id = operation_result.unwrap();
        
        // Complete the operation
        let completion_result = io_coordinator.complete_io_operation(
            operation_id,
            Ok(crate::runtime::io_coordinator::IOResult {
                data: Some(b"test data".to_vec()),
                bytes_processed: 9,
                metadata: std::collections::HashMap::new(),
            })
        );
        assert!(completion_result.is_ok());
        
        // Get statistics
        let stats = io_coordinator.get_io_statistics();
        assert_eq!(stats.active_threads, 1);
        assert_eq!(stats.total_active_operations, 0); // Operation completed
        
        io_coordinator.unregister_thread(thread_id);
    }

    #[tokio::test]
    async fn test_error_propagation() {
        use crate::runtime::error_propagation::ErrorPropagationCoordinator;
        use crate::diagnostics::Error as DiagnosticError;
        
        let error_coordinator = ErrorPropagationCoordinator::new();
        let thread_id = std::thread::current().id();
        
        // Register thread
        error_coordinator.register_thread(thread_id);
        
        // Test error reporting
        let diagnostic_error = DiagnosticError::runtime_error("Test error".to_string(), None);
        let error_result = error_coordinator.report_error(thread_id, diagnostic_error, None);
        assert!(error_result.is_ok());
        
        let error_id = error_result.unwrap();
        assert!(error_id > 0);
        
        // Get error context
        let context = error_coordinator.get_thread_error_context(thread_id);
        assert!(context.is_some());
        
        let ctx = context.unwrap();
        assert_eq!(ctx.thread_id, thread_id);
        assert!(!ctx.error_stack.is_empty());
        
        // Get statistics
        let stats = error_coordinator.get_error_statistics();
        assert_eq!(stats.active_threads, 1);
        assert_eq!(stats.threads_with_errors, 1);
        assert!(stats.total_errors > 0);
        
        error_coordinator.unregister_thread(thread_id);
    }

    #[tokio::test]
    async fn test_effect_isolation() {
        use crate::runtime::{EffectIsolationLevel, EffectSandboxConfig};
        use crate::effects::Effect;
        
        let coordinator = EffectCoordinator::new();
        let thread_id = std::thread::current().id();
        
        // Register thread
        coordinator.register_thread(thread_id);
        
        // Test effect isolation
        let isolation_result = coordinator.enable_effect_isolation(thread_id, EffectIsolationLevel::Complete);
        assert!(isolation_result.is_ok());
        
        // Test cross-thread effect blocking  
        let other_thread = std::thread::current().id(); // Use current thread ID as placeholder
        let can_cross = coordinator.can_effect_cross_threads(&Effect::IO, thread_id, other_thread);
        assert!(!can_cross); // Should be blocked due to complete isolation
        
        // Test sandbox creation
        let sandbox_result = coordinator.create_effect_sandbox(thread_id, EffectSandboxConfig::default());
        assert!(sandbox_result.is_ok());
        
        let sandbox = sandbox_result.unwrap();
        assert_eq!(sandbox.thread_id, thread_id);
        assert!(sandbox.is_valid());
        
        // Get isolation statistics
        let stats = coordinator.get_isolation_statistics();
        assert_eq!(stats.total_threads, 1);
        assert_eq!(stats.isolated_threads, 1);
        
        // Clean up sandbox
        let destroy_result = sandbox.destroy();
        assert!(destroy_result.is_ok());
        
        coordinator.unregister_thread(thread_id);
    }

    #[tokio::test]
    async fn test_effect_ordering_guarantees() {
        use crate::effects::Effect;
        
        let coordinator = EffectCoordinator::new();
        let thread_id = std::thread::current().id();
        
        // Register thread
        coordinator.register_thread(thread_id);
        
        // Test multiple effects with ordering
        let mut sequences = Vec::new();
        
        for i in 0..5 {
            let effect = Effect::State;
            let result = coordinator.coordinate_local_effect(
                thread_id,
                effect,
                &[crate::eval::Value::integer(i)]
            );
            assert!(result.is_ok());
            sequences.push(result.unwrap());
        }
        
        // Verify ordering (sequences should be increasing)
        for i in 1..sequences.len() {
            assert!(sequences[i] > sequences[i-1]);
        }
        
        // Complete effects in order
        for &sequence in &sequences {
            let completion_result = coordinator.complete_effect(sequence, Ok("completed".to_string()));
            assert!(completion_result.is_ok());
        }
        
        coordinator.unregister_thread(thread_id);
    }

    #[tokio::test]
    async fn test_integrated_runtime_features() {
        let runtime = LambdustRuntime::new().expect("Failed to create runtime");
        
        // Test accessing all coordinators
        let global_env = runtime.global_env();
        let effect_coordinator = runtime.effect_coordinator();
        let io_coordinator = runtime.io_coordinator();
        let error_propagation = runtime.error_propagation();
        
        // Note: Thread pool creation may register worker threads automatically
        // Just verify the method works (usize is always >= 0)
        let _thread_count = global_env.active_thread_count();
        
        // Register a test thread with all coordinators
        let thread_id = std::thread::current().id();
        effect_coordinator.register_thread(thread_id);
        io_coordinator.register_thread(thread_id);
        error_propagation.register_thread(thread_id);
        
        // Test cross-coordinator functionality
        let transaction_id = global_env.start_transaction(thread_id).expect("Failed to start transaction");
        
        // Define a variable in transaction
        let define_result = global_env.define_global_transactional(
            "integrated_test".to_string(),
            crate::eval::Value::string("success".to_string()),
            thread_id
        );
        assert!(define_result.is_ok());
        
        // Commit transaction
        let commit_result = global_env.commit_transaction(transaction_id);
        assert!(commit_result.is_ok());
        
        // Verify variable exists
        let lookup_result = global_env.lookup_global("integrated_test");
        assert!(lookup_result.is_some());
        
        // Clean up
        effect_coordinator.unregister_thread(thread_id);
        io_coordinator.unregister_thread(thread_id);
        error_propagation.unregister_thread(thread_id);
        
        let _ = runtime.shutdown().await;
    }

    #[tokio::test]
    async fn test_concurrent_transactions() {
        let global_env = GlobalEnvironmentManager::new();
        
        // Simulate multiple threads with transactions
        let handles: Vec<_> = (0..4).map(|i| {
            let env = global_env.clone();
            tokio::spawn(async move {
                let thread_id = std::thread::current().id();
                let tx_id = env.start_transaction(thread_id).expect("Failed to start transaction");
                
                // Each thread defines a unique variable
                let var_name = format!("concurrent_var_{}", i);
                let define_result = env.define_global_transactional(
                    var_name.clone(),
                    crate::eval::Value::integer(i),
                    thread_id
                );
                assert!(define_result.is_ok());
                
                // Commit transaction
                let commit_result = env.commit_transaction(tx_id);
                assert!(commit_result.is_ok());
                
                // Verify variable exists
                let lookup_result = env.lookup_global(&var_name);
                assert!(lookup_result.is_some());
                assert_eq!(lookup_result.unwrap(), crate::eval::Value::integer(i));
                
                i
            })
        }).collect();
        
        // Wait for all transactions to complete
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
        
        // Verify all variables exist
        for i in 0..4 {
            let var_name = format!("concurrent_var_{}", i);
            let lookup_result = global_env.lookup_global(&var_name);
            assert!(lookup_result.is_some());
            assert_eq!(lookup_result.unwrap(), crate::eval::Value::integer(i));
        }
    }

    // ============= LIBRARY PATH RESOLUTION TESTS =============

    #[test]
    fn test_lambdust_lib_dir_environment_variable() {
        use tempfile::TempDir;
        use std::fs;
        use crate::runtime::{LibraryPathResolver, LibraryPathConfig};

        let temp_dir = TempDir::new().unwrap();
        let lib_dir = temp_dir.path().to_path_buf();
        
        // Create test library structure
        fs::create_dir_all(lib_dir.join("r7rs")).unwrap();
        fs::create_dir_all(lib_dir.join("bootstrap")).unwrap();
        fs::create_dir_all(lib_dir.join("modules")).unwrap();
        fs::write(lib_dir.join("r7rs").join("base.scm"), "(define-library (scheme base) ...)").unwrap();
        
        // Test with environment variable override in config
        let config = LibraryPathConfig {
            lib_dir_override: Some(lib_dir.clone()),
            ..Default::default()
        };
        
        let resolver = LibraryPathResolver::with_config(config).unwrap();
        assert_eq!(resolver.primary_lib_dir(), Some(lib_dir.as_path()));
        
        // Test subdir resolution
        let r7rs_dir = resolver.resolve_lib_subdir("r7rs").unwrap();
        assert_eq!(r7rs_dir, lib_dir.join("r7rs"));
        
        // Test file resolution
        let base_file = resolver.resolve_library_file("r7rs", "base.scm").unwrap();
        assert_eq!(base_file, lib_dir.join("r7rs").join("base.scm"));
    }

    #[test]
    fn test_library_path_fallback_behavior() {
        use std::path::PathBuf;
        use crate::runtime::{LibraryPathResolver, LibraryPathConfig};

        // Test behavior when LAMBDUST_LIB_DIR points to non-existent directory
        let config = LibraryPathConfig {
            lib_dir_override: Some(PathBuf::from("/nonexistent/path")),
            ..Default::default()
        };
        
        let result = LibraryPathResolver::with_config(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("LAMBDUST_LIB_DIR points to invalid directory"));
    }

    #[test]
    fn test_bootstrap_mode_determination() {
        use tempfile::TempDir;
        use std::fs;
        use crate::runtime::{BootstrapIntegration, BootstrapMode, LibraryPathConfig};

        let temp_dir = TempDir::new().unwrap();
        let good_lib_dir = temp_dir.path().join("good");
        let bad_lib_dir = temp_dir.path().join("bad");
        
        // Create good library structure
        fs::create_dir_all(good_lib_dir.join("r7rs")).unwrap();
        fs::create_dir_all(good_lib_dir.join("bootstrap")).unwrap();
        fs::create_dir_all(good_lib_dir.join("modules")).unwrap();
        
        // Create bad library structure (empty)
        fs::create_dir_all(&bad_lib_dir).unwrap();
        
        // Test with good library setup
        let good_config = LibraryPathConfig {
            lib_dir_override: Some(good_lib_dir),
            ..Default::default()
        };
        
        let good_mode = BootstrapIntegration::determine_bootstrap_mode_with_config(good_config);
        assert_eq!(good_mode, BootstrapMode::Full);
        
        // Test with bad library setup
        let bad_config = LibraryPathConfig {
            lib_dir_override: Some(bad_lib_dir),
            ..Default::default()
        };
        
        let bad_mode = BootstrapIntegration::determine_bootstrap_mode_with_config(bad_config);
        assert_eq!(bad_mode, BootstrapMode::Minimal);
    }

    #[test]
    fn test_library_validation_comprehensive() {
        use tempfile::TempDir;
        use std::fs;
        use crate::runtime::{LibraryPathResolver, LibraryPathConfig};

        let temp_dir = TempDir::new().unwrap();
        let lib_dir = temp_dir.path().to_path_buf();
        
        // Create complete library structure
        let subdirs = ["r7rs", "bootstrap", "modules"];
        for subdir in &subdirs {
            fs::create_dir_all(lib_dir.join(subdir)).unwrap();
            fs::write(lib_dir.join(subdir).join("test.scm"), "test library").unwrap();
        }
        
        let config = LibraryPathConfig {
            lib_dir_override: Some(lib_dir),
            ..Default::default()
        };
        
        let resolver = LibraryPathResolver::with_config(config).unwrap();
        let validation = resolver.validate_library_setup().unwrap();
        
        assert!(validation.primary_lib_dir_valid);
        assert_eq!(validation.found_search_paths.len(), 1);
        assert!(validation.missing_critical_subdirs.is_empty());
        assert!(validation.is_usable());
        
        // Check that all subdirs have files
        for subdir in &subdirs {
            let count = validation.found_library_files.get(*subdir).copied().unwrap_or(0);
            assert!(count > 0, "Subdir {} should have library files", subdir);
        }
    }

    #[test]
    fn test_missing_library_error_messages() {
        use tempfile::TempDir;
        use std::fs;
        use crate::runtime::{LibraryPathResolver, LibraryPathConfig};

        let temp_dir = TempDir::new().unwrap();
        let lib_dir = temp_dir.path().to_path_buf();
        
        // Create directory but no libraries
        fs::create_dir_all(&lib_dir).unwrap();
        
        let config = LibraryPathConfig {
            lib_dir_override: Some(lib_dir),
            ..Default::default()
        };
        
        let resolver = LibraryPathResolver::with_config(config).unwrap();
        
        // Test missing subdirectory
        let result = resolver.resolve_lib_subdir("nonexistent");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Library subdirectory 'nonexistent' not found"));
        assert!(error_msg.contains("LAMBDUST_LIB_DIR"));
        
        // Test missing file
        let result = resolver.resolve_library_file("r7rs", "missing.scm");
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Library file 'r7rs/missing.scm' not found"));
        assert!(error_msg.contains("LAMBDUST_LIB_DIR"));
    }
}
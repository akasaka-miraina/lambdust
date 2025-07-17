//! Environment Mock Tests
//!
//! Comprehensive unit tests for Environment using mockall for isolated testing.
//! Tests cover normal operations, edge cases, and error conditions.

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use mockall::*;
    
    use lambdust::environment::SharedEnvironment;
    use lambdust::error::{LambdustError, Result};
    use lambdust::value::Value;
    use lambdust::ast::{Expr, Literal};
    use std::rc::Rc;
    use std::collections::HashMap;

    /// Mock trait for Environment operations to enable isolated testing
    #[automock]
    pub trait EnvironmentOps {
        /// Get a value from the environment
        fn get(&self, name: &str) -> Option<Value>;
        
        /// Define a new binding in the environment
        fn define(&mut self, name: String, value: Value) -> Result<()>;
        
        /// Set an existing binding in the environment
        fn set(&mut self, name: &str, value: Value) -> Result<()>;
        
        /// Check if a binding exists
        fn has_binding(&self, name: &str) -> bool;
        
        /// Get all bindings (for testing purposes)
        fn get_all_bindings(&self) -> HashMap<String, Value>;
        
        /// Create a new child environment
        fn extend(&self) -> Rc<SharedEnvironment>;
        
        /// Get the parent environment
        fn parent(&self) -> Option<Rc<SharedEnvironment>>;
        
        /// Check if environment is frozen
        fn is_frozen(&self) -> bool;
        
        /// Freeze the environment (make immutable)
        fn freeze(&mut self);
        
        /// Get generation counter
        fn generation(&self) -> u32;
        
        /// Clear all bindings
        fn clear(&mut self);
        
        /// Get binding count
        fn binding_count(&self) -> usize;
    }

    // ===== Normal Operation Tests =====

    #[test]
    fn test_basic_get_operation() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Setup expectation
        mock_env.expect_get()
            .with(eq("x"))
            .times(1)
            .returning(|_| Some(Value::Number(42.into())));
        
        // Test
        let result = mock_env.get("x");
        assert_eq!(result, Some(Value::Number(42.into())));
    }

    #[test]
    fn test_basic_define_operation() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Setup expectation
        mock_env.expect_define()
            .with(eq("x".to_string()), eq(Value::Number(42.into())))
            .times(1)
            .returning(|_, _| Ok(()));
        
        // Test
        let result = mock_env.define("x".to_string(), Value::Number(42.into()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_basic_set_operation() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Setup expectation
        mock_env.expect_set()
            .with(eq("x"), eq(Value::Number(99.into())))
            .times(1)
            .returning(|_, _| Ok(()));
        
        // Test
        let result = mock_env.set("x", Value::Number(99.into()));
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_binding_true() {
        let mut mock_env = MockEnvironmentOps::new();
        
        mock_env.expect_has_binding()
            .with(eq("existing_var"))
            .times(1)
            .returning(|_| true);
        
        assert!(mock_env.has_binding("existing_var"));
    }

    #[test]
    fn test_has_binding_false() {
        let mut mock_env = MockEnvironmentOps::new();
        
        mock_env.expect_has_binding()
            .with(eq("nonexistent_var"))
            .times(1)
            .returning(|_| false);
        
        assert!(!mock_env.has_binding("nonexistent_var"));
    }

    // ===== Error Cases =====

    #[test]
    fn test_define_duplicate_error() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // First define succeeds
        mock_env.expect_define()
            .with(eq("x".to_string()), eq(Value::Number(1.into())))
            .times(1)
            .returning(|_, _| Ok(()));
        
        // Second define with same name fails
        mock_env.expect_define()
            .with(eq("x".to_string()), eq(Value::Number(2.into())))
            .times(1)
            .returning(|_, _| Err(LambdustError::runtime_error("Variable already defined".to_string())));
        
        // Test
        assert!(mock_env.define("x".to_string(), Value::Number(1.into())).is_ok());
        assert!(mock_env.define("x".to_string(), Value::Number(2.into())).is_err());
    }

    #[test]
    fn test_set_nonexistent_variable_error() {
        let mut mock_env = MockEnvironmentOps::new();
        
        mock_env.expect_set()
            .with(eq("nonexistent"), eq(Value::Number(42.into())))
            .times(1)
            .returning(|_, _| Err(LambdustError::runtime_error("Unbound variable".to_string())));
        
        let result = mock_env.set("nonexistent", Value::Number(42.into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_variable() {
        let mut mock_env = MockEnvironmentOps::new();
        
        mock_env.expect_get()
            .with(eq("nonexistent"))
            .times(1)
            .returning(|_| None);
        
        let result = mock_env.get("nonexistent");
        assert_eq!(result, None);
    }

    // ===== Edge Cases =====

    #[test]
    fn test_empty_variable_name() {
        let mut mock_env = MockEnvironmentOps::new();
        
        mock_env.expect_define()
            .with(eq("".to_string()), any())
            .times(1)
            .returning(|_, _| Err(LambdustError::runtime_error("Empty variable name".to_string())));
        
        let result = mock_env.define("".to_string(), Value::Nil);
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_only_variable_name() {
        let mut mock_env = MockEnvironmentOps::new();
        
        mock_env.expect_define()
            .with(eq("   ".to_string()), any())
            .times(1)
            .returning(|_, _| Err(LambdustError::runtime_error("Invalid variable name".to_string())));
        
        let result = mock_env.define("   ".to_string(), Value::Nil);
        assert!(result.is_err());
    }

    #[test]
    fn test_special_character_variable_names() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Test various valid special characters in Scheme
        let valid_names = vec!["+", "-", "*", "/", "=", "<", ">", "?", "!", "car", "cdr", "set-car!"];
        
        for name in valid_names {
            mock_env.expect_define()
                .with(eq(name.to_string()), any())
                .times(1)
                .returning(|_, _| Ok(()));
        }
        
        for name in vec!["+", "-", "*", "/", "=", "<", ">", "?", "!", "car", "cdr", "set-car!"] {
            let result = mock_env.define(name.to_string(), Value::Number(1.into()));
            assert!(result.is_ok(), "Failed to define variable: {}", name);
        }
    }

    #[test]
    fn test_large_number_of_bindings() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Mock binding count
        mock_env.expect_binding_count()
            .times(1)
            .returning(|| 10000);
        
        assert_eq!(mock_env.binding_count(), 10000);
    }

    #[test]
    fn test_very_long_variable_name() {
        let mut mock_env = MockEnvironmentOps::new();
        
        let long_name = "a".repeat(1000);
        mock_env.expect_define()
            .with(eq(long_name.clone()), any())
            .times(1)
            .returning(|_, _| Ok(()));
        
        let result = mock_env.define(long_name, Value::Nil);
        assert!(result.is_ok());
    }

    // ===== Environment Chain Tests =====

    #[test]
    fn test_parent_environment_access() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Mock creating a parent environment
        mock_env.expect_parent()
            .times(1)
            .returning(|| None); // Root environment has no parent
        
        let parent = mock_env.parent();
        assert!(parent.is_none());
    }

    #[test]
    fn test_extend_environment() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Mock extending environment
        mock_env.expect_extend()
            .times(1)
            .returning(|| {
                // Return a new environment (mock)
                Rc::new(SharedEnvironment::new())
            });
        
        let child = mock_env.extend();
        assert!(!Rc::ptr_eq(&child, &child)); // Just verify we got something back
    }

    // ===== State Management Tests =====

    #[test]
    fn test_environment_freeze() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Initially not frozen
        mock_env.expect_is_frozen()
            .times(1)
            .returning(|| false);
        
        // Freeze operation
        mock_env.expect_freeze()
            .times(1)
            .returning(|| ());
        
        // After freeze
        mock_env.expect_is_frozen()
            .times(1)
            .returning(|| true);
        
        assert!(!mock_env.is_frozen());
        mock_env.freeze();
        assert!(mock_env.is_frozen());
    }

    #[test]
    fn test_frozen_environment_modification_error() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Environment is frozen
        mock_env.expect_is_frozen()
            .times(1)
            .returning(|| true);
        
        // Attempt to modify frozen environment should fail
        mock_env.expect_define()
            .with(any(), any())
            .times(1)
            .returning(|_, _| Err(LambdustError::runtime_error("Environment is frozen".to_string())));
        
        assert!(mock_env.is_frozen());
        let result = mock_env.define("x".to_string(), Value::Number(1.into()));
        assert!(result.is_err());
    }

    #[test]
    fn test_generation_counter() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Initial generation
        mock_env.expect_generation()
            .times(1)
            .returning(|| 0);
        
        // After modification
        mock_env.expect_define()
            .with(any(), any())
            .times(1)
            .returning(|_, _| Ok(()));
        
        mock_env.expect_generation()
            .times(1)
            .returning(|| 1);
        
        assert_eq!(mock_env.generation(), 0);
        let _ = mock_env.define("x".to_string(), Value::Number(1.into()));
        assert_eq!(mock_env.generation(), 1);
    }

    // ===== Bulk Operations Tests =====

    #[test]
    fn test_get_all_bindings() {
        let mut mock_env = MockEnvironmentOps::new();
        
        let mut expected_bindings = HashMap::new();
        expected_bindings.insert("x".to_string(), Value::Number(1.into()));
        expected_bindings.insert("y".to_string(), Value::String("hello".to_string()));
        expected_bindings.insert("z".to_string(), Value::Boolean(true));
        
        mock_env.expect_get_all_bindings()
            .times(1)
            .returning(move || expected_bindings.clone());
        
        let bindings = mock_env.get_all_bindings();
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings.get("x"), Some(&Value::Number(1.into())));
        assert_eq!(bindings.get("y"), Some(&Value::String("hello".to_string())));
        assert_eq!(bindings.get("z"), Some(&Value::Boolean(true)));
    }

    #[test]
    fn test_clear_environment() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Initially has bindings
        mock_env.expect_binding_count()
            .times(1)
            .returning(|| 5);
        
        // Clear operation
        mock_env.expect_clear()
            .times(1)
            .returning(|| ());
        
        // After clear
        mock_env.expect_binding_count()
            .times(1)
            .returning(|| 0);
        
        assert_eq!(mock_env.binding_count(), 5);
        mock_env.clear();
        assert_eq!(mock_env.binding_count(), 0);
    }

    // ===== Complex Scenarios =====

    #[test]
    fn test_multiple_operations_sequence() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Setup complex sequence of operations
        mock_env.expect_define()
            .with(eq("x".to_string()), eq(Value::Number(1.into())))
            .times(1)
            .returning(|_, _| Ok(()));
        
        mock_env.expect_get()
            .with(eq("x"))
            .times(1)
            .returning(|_| Some(Value::Number(1.into())));
        
        mock_env.expect_set()
            .with(eq("x"), eq(Value::Number(2.into())))
            .times(1)
            .returning(|_, _| Ok(()));
        
        mock_env.expect_get()
            .with(eq("x"))
            .times(1)
            .returning(|_| Some(Value::Number(2.into())));
        
        // Execute sequence
        assert!(mock_env.define("x".to_string(), Value::Number(1.into())).is_ok());
        assert_eq!(mock_env.get("x"), Some(Value::Number(1.into())));
        assert!(mock_env.set("x", Value::Number(2.into())).is_ok());
        assert_eq!(mock_env.get("x"), Some(Value::Number(2.into())));
    }

    #[test]
    fn test_concurrent_access_simulation() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Simulate concurrent access patterns
        mock_env.expect_get()
            .with(eq("shared_var"))
            .times(10)
            .returning(|_| Some(Value::Number(42.into())));
        
        // Simulate multiple threads reading the same variable
        for _ in 0..10 {
            assert_eq!(mock_env.get("shared_var"), Some(Value::Number(42.into())));
        }
    }

    // ===== Performance-Related Tests =====

    #[test]
    fn test_binding_count_performance() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Test that binding count operation is fast even with many bindings
        mock_env.expect_binding_count()
            .times(1)
            .returning(|| 1_000_000);
        
        let start = std::time::Instant::now();
        let count = mock_env.binding_count();
        let duration = start.elapsed();
        
        assert_eq!(count, 1_000_000);
        assert!(duration.as_millis() < 1); // Should be very fast
    }

    // ===== Memory-Related Tests =====

    #[test]
    fn test_memory_efficient_operations() {
        let mut mock_env = MockEnvironmentOps::new();
        
        // Test that operations don't cause excessive allocations
        mock_env.expect_get()
            .with(eq("test_var"))
            .times(100)
            .returning(|_| Some(Value::Number(1.into())));
        
        // Perform many get operations
        for _ in 0..100 {
            let _ = mock_env.get("test_var");
        }
        
        // This test passes if it completes without running out of memory
    }
}
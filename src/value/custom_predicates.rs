//! Custom type predicates system for runtime-defined type checkers

use crate::value::Value;
use crate::error::LambdustError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A custom type predicate function
/// Takes a value and returns whether it matches the custom type
pub type CustomPredicateFn = Arc<dyn Fn(&Value) -> bool + Send + Sync>;

/// Custom predicate metadata for introspection and debugging
#[derive(Clone)]
pub struct CustomPredicateInfo {
    /// Name of the custom predicate (e.g., "my-record?")
    pub name: String,
    /// Optional description of what this predicate checks
    pub description: Option<String>,
    /// The predicate function
    pub predicate_fn: CustomPredicateFn,
}

/// Global registry for custom type predicates
/// Thread-safe using RwLock for concurrent access
pub struct CustomPredicateRegistry {
    predicates: RwLock<HashMap<String, CustomPredicateInfo>>,
}

impl CustomPredicateRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            predicates: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new custom predicate
    /// Returns Ok(()) if successful, Err if name already exists
    pub fn register<F>(&self, name: String, description: Option<String>, predicate_fn: F) -> Result<(), LambdustError>
    where
        F: Fn(&Value) -> bool + Send + Sync + 'static,
    {
        let mut predicates = self.predicates.write().map_err(|_| {
            LambdustError::runtime_error("Failed to acquire write lock on custom predicate registry")
        })?;

        if predicates.contains_key(&name) {
            return Err(LambdustError::runtime_error(&format!("Custom predicate '{}' already exists", name)));
        }

        let info = CustomPredicateInfo {
            name: name.clone(),
            description,
            predicate_fn: Arc::new(predicate_fn),
        };

        predicates.insert(name, info);
        Ok(())
    }

    /// Unregister a custom predicate
    /// Returns true if the predicate was found and removed
    pub fn unregister(&self, name: &str) -> Result<bool, LambdustError> {
        let mut predicates = self.predicates.write().map_err(|_| {
            LambdustError::runtime_error("Failed to acquire write lock on custom predicate registry")
        })?;

        Ok(predicates.remove(name).is_some())
    }

    /// Check if a predicate is registered
    pub fn is_registered(&self, name: &str) -> Result<bool, LambdustError> {
        let predicates = self.predicates.read().map_err(|_| {
            LambdustError::runtime_error("Failed to acquire read lock on custom predicate registry")
        })?;

        Ok(predicates.contains_key(name))
    }

    /// Evaluate a custom predicate against a value
    /// Returns None if the predicate is not found
    pub fn evaluate(&self, name: &str, value: &Value) -> Result<Option<bool>, LambdustError> {
        let predicates = self.predicates.read().map_err(|_| {
            LambdustError::runtime_error("Failed to acquire read lock on custom predicate registry")
        })?;

        match predicates.get(name) {
            Some(info) => Ok(Some((info.predicate_fn)(value))),
            None => Ok(None),
        }
    }

    /// Get information about a registered predicate
    pub fn get_info(&self, name: &str) -> Result<Option<CustomPredicateInfo>, LambdustError> {
        let predicates = self.predicates.read().map_err(|_| {
            LambdustError::runtime_error("Failed to acquire read lock on custom predicate registry")
        })?;

        Ok(predicates.get(name).cloned())
    }

    /// List all registered custom predicates
    pub fn list_predicates(&self) -> Result<Vec<String>, LambdustError> {
        let predicates = self.predicates.read().map_err(|_| {
            LambdustError::runtime_error("Failed to acquire read lock on custom predicate registry")
        })?;

        Ok(predicates.keys().cloned().collect())
    }

    /// Clear all custom predicates
    pub fn clear(&self) -> Result<(), LambdustError> {
        let mut predicates = self.predicates.write().map_err(|_| {
            LambdustError::runtime_error("Failed to acquire write lock on custom predicate registry")
        })?;

        predicates.clear();
        Ok(())
    }
}

impl Default for CustomPredicateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global singleton instance of the custom predicate registry
/// This allows access from anywhere in the interpreter
static GLOBAL_CUSTOM_PREDICATE_REGISTRY: std::sync::OnceLock<CustomPredicateRegistry> = std::sync::OnceLock::new();

/// Get the global custom predicate registry
pub fn global_custom_predicate_registry() -> &'static CustomPredicateRegistry {
    GLOBAL_CUSTOM_PREDICATE_REGISTRY.get_or_init(CustomPredicateRegistry::new)
}

/// Helper function to register a custom predicate globally
pub fn register_global_custom_predicate<F>(
    name: String,
    description: Option<String>,
    predicate_fn: F,
) -> Result<(), LambdustError>
where
    F: Fn(&Value) -> bool + Send + Sync + 'static,
{
    global_custom_predicate_registry().register(name, description, predicate_fn)
}

/// Helper function to evaluate a custom predicate globally
pub fn evaluate_global_custom_predicate(name: &str, value: &Value) -> Result<Option<bool>, LambdustError> {
    global_custom_predicate_registry().evaluate(name, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_predicate_registry_basic() {
        let registry = CustomPredicateRegistry::new();
        
        // Register a simple predicate
        let result = registry.register(
            "test-predicate?".to_string(),
            Some("A test predicate".to_string()),
            |value| value.is_number(),
        );
        assert!(result.is_ok());
        
        // Check if registered
        assert!(registry.is_registered("test-predicate?").unwrap());
        
        // Evaluate against a number
        let number = Value::Number(crate::lexer::SchemeNumber::Integer(42));
        assert_eq!(registry.evaluate("test-predicate?", &number).unwrap(), Some(true));
        
        // Evaluate against a string
        let string = Value::String("hello".to_string());
        assert_eq!(registry.evaluate("test-predicate?", &string).unwrap(), Some(false));
    }

    #[test]
    fn test_custom_predicate_registry_duplicate() {
        let registry = CustomPredicateRegistry::new();
        
        // Register a predicate
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
    }

    #[test]
    fn test_custom_predicate_registry_unregister() {
        let registry = CustomPredicateRegistry::new();
        
        // Register and then unregister
        registry.register("temp-predicate?".to_string(), None, |_| true).unwrap();
        assert!(registry.is_registered("temp-predicate?").unwrap());
        
        let removed = registry.unregister("temp-predicate?").unwrap();
        assert!(removed);
        assert!(!registry.is_registered("temp-predicate?").unwrap());
        
        // Try to unregister non-existent predicate
        let not_removed = registry.unregister("nonexistent?").unwrap();
        assert!(!not_removed);
    }

    #[test]
    fn test_global_custom_predicate_registry() {
        // Test global registry functionality
        let result = register_global_custom_predicate(
            "global-test?".to_string(),
            Some("Global test predicate".to_string()),
            |value| value.is_string(),
        );
        assert!(result.is_ok());
        
        let string_val = Value::String("test".to_string());
        let result = evaluate_global_custom_predicate("global-test?", &string_val);
        assert_eq!(result.unwrap(), Some(true));
        
        let number_val = Value::Number(crate::lexer::SchemeNumber::Integer(1));
        let result = evaluate_global_custom_predicate("global-test?", &number_val);
        assert_eq!(result.unwrap(), Some(false));
    }
}
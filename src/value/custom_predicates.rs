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
/// Thread-safe using `RwLock` for concurrent access
pub struct CustomPredicateRegistry {
    predicates: RwLock<HashMap<String, CustomPredicateInfo>>,
}

impl CustomPredicateRegistry {
    /// Create a new empty registry
    #[must_use] pub fn new() -> Self {
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
            return Err(LambdustError::runtime_error(format!("Custom predicate '{name}' already exists")));
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

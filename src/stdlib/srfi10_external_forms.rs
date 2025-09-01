//! SRFI-10: Sharp-comma external form implementation.
//!
//! This module implements the SRFI-10 specification for external representations
//! that can be processed at read-time. External forms have the syntax:
//!
//! ```scheme
//! #,(tag arg ...)
//! ```
//!
//! Where `tag` is a symbol that identifies a constructor procedure, and
//! `arg ...` are the arguments to pass to that constructor.
//!
//! ## Core Procedures
//!
//! - `(define-reader-ctor tag proc)`: Register a constructor procedure for a tag
//! - `(reader-ctor? tag)`: Check if a tag has a registered constructor
//! - `(reader-ctor-ref tag)`: Get the constructor procedure for a tag
//!
//! ## Thread Safety
//!
//! The constructor registry is thread-safe, allowing concurrent registration
//! and lookup of external form constructors across multiple threads.

use crate::eval::Value;
use crate::utils::SymbolId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe constructor registry for SRFI-10 external forms.
///
/// This registry maps tag symbols to their constructor procedures.
/// The registry is shared across all evaluator instances and supports
/// concurrent access from multiple threads.
#[derive(Debug, Clone)]
pub struct ExternalFormRegistry {
    /// Map from tag symbol to constructor procedure
    constructors: Arc<RwLock<HashMap<SymbolId, Value>>>,
}

impl Default for ExternalFormRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalFormRegistry {
    /// Create a new external form registry.
    pub fn new() -> Self {
        Self {
            constructors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a constructor procedure for a tag.
    ///
    /// # Arguments
    /// - `tag`: Symbol identifying the external form constructor
    /// - `constructor`: Procedure to call when the external form is encountered
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently.
    pub fn define_constructor(&self, tag: SymbolId, constructor: Value) -> Result<(), String> {
        // Validate that the constructor is a procedure
        match &constructor {
            Value::Primitive { .. } | Value::Procedure { .. } | Value::CaseLambda { .. } => {
                // Valid procedure types
            }
            _ => {
                return Err(format!(
                    "Constructor for tag '{}' must be a procedure, got: {:?}",
                    tag, constructor
                ));
            }
        }

        let mut constructors = self.constructors.write().map_err(|e| {
            format!("Failed to acquire write lock on constructor registry: {}", e)
        })?;
        
        constructors.insert(tag, constructor);
        Ok(())
    }

    /// Check if a tag has a registered constructor.
    ///
    /// # Arguments
    /// - `tag`: Symbol to check
    ///
    /// # Returns
    /// `true` if the tag has a registered constructor, `false` otherwise.
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently.
    pub fn has_constructor(&self, tag: SymbolId) -> Result<bool, String> {
        let constructors = self.constructors.read().map_err(|e| {
            format!("Failed to acquire read lock on constructor registry: {}", e)
        })?;
        
        Ok(constructors.contains_key(&tag))
    }

    /// Get the constructor procedure for a tag.
    ///
    /// # Arguments
    /// - `tag`: Symbol to look up
    ///
    /// # Returns
    /// The constructor procedure if registered, `None` otherwise.
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently.
    pub fn get_constructor(&self, tag: SymbolId) -> Result<Option<Value>, String> {
        let constructors = self.constructors.read().map_err(|e| {
            format!("Failed to acquire read lock on constructor registry: {}", e)
        })?;
        
        Ok(constructors.get(&tag).cloned())
    }

    /// Remove a constructor for a tag.
    ///
    /// # Arguments
    /// - `tag`: Symbol to remove
    ///
    /// # Returns
    /// The removed constructor procedure if it existed, `None` otherwise.
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently.
    pub fn remove_constructor(&self, tag: SymbolId) -> Result<Option<Value>, String> {
        let mut constructors = self.constructors.write().map_err(|e| {
            format!("Failed to acquire write lock on constructor registry: {}", e)
        })?;
        
        Ok(constructors.remove(&tag))
    }

    /// Get all registered tags.
    ///
    /// # Returns
    /// A vector of all currently registered tag symbols.
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently.
    pub fn get_all_tags(&self) -> Result<Vec<SymbolId>, String> {
        let constructors = self.constructors.read().map_err(|e| {
            format!("Failed to acquire read lock on constructor registry: {}", e)
        })?;
        
        Ok(constructors.keys().cloned().collect())
    }

    /// Clear all registered constructors.
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently.
    pub fn clear(&self) -> Result<(), String> {
        let mut constructors = self.constructors.write().map_err(|e| {
            format!("Failed to acquire write lock on constructor registry: {}", e)
        })?;
        
        constructors.clear();
        Ok(())
    }

    /// Get the number of registered constructors.
    ///
    /// # Returns
    /// The number of registered constructor procedures.
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently.
    pub fn len(&self) -> Result<usize, String> {
        let constructors = self.constructors.read().map_err(|e| {
            format!("Failed to acquire read lock on constructor registry: {}", e)
        })?;
        
        Ok(constructors.len())
    }

    /// Check if the registry is empty.
    ///
    /// # Returns
    /// `true` if no constructors are registered, `false` otherwise.
    ///
    /// # Thread Safety
    /// This method is thread-safe and can be called concurrently.
    pub fn is_empty(&self) -> Result<bool, String> {
        let constructors = self.constructors.read().map_err(|e| {
            format!("Failed to acquire read lock on constructor registry: {}", e)
        })?;
        
        Ok(constructors.is_empty())
    }
}

/// Global external form registry instance.
///
/// This is a singleton registry that is used across all evaluator instances.
/// It provides thread-safe access to the external form constructor registry.
lazy_static::lazy_static! {
    static ref GLOBAL_REGISTRY: ExternalFormRegistry = ExternalFormRegistry::new();
}

/// Get the global external form registry.
///
/// # Returns
/// A reference to the global registry instance.
pub fn global_registry() -> &'static ExternalFormRegistry {
    &GLOBAL_REGISTRY
}

/// Apply an external form constructor to its arguments.
///
/// This function looks up the constructor for the given tag and applies it
/// to the provided arguments. If no constructor is registered for the tag,
/// an error is returned.
///
/// # Arguments
/// - `tag`: Symbol identifying the constructor to apply
/// - `args`: Arguments to pass to the constructor
///
/// # Returns
/// The result of applying the constructor to the arguments.
///
/// # Errors
/// - If no constructor is registered for the tag
/// - If the constructor application fails
pub fn apply_external_form(tag: SymbolId, args: Vec<Value>) -> Result<Value, String> {
    let registry = global_registry();
    
    match registry.get_constructor(tag)? {
        Some(constructor) => {
            // Apply the constructor to the arguments
            // This would typically involve calling the evaluator's apply function
            // For now, we'll return a placeholder result
            match constructor {
                Value::Primitive { name, .. } => {
                    // Handle primitive constructors
                    Err(format!(
                        "Primitive constructor '{}' not yet supported in external forms",
                        name
                    ))
                }
                Value::Procedure { .. } => {
                    // Handle user-defined procedure constructors
                    Err("User-defined procedure constructors not yet fully implemented".to_string())
                }
                Value::CaseLambda { .. } => {
                    // Handle case-lambda constructors
                    Err("Case-lambda constructors not yet fully implemented".to_string())
                }
                _ => {
                    Err(format!("Invalid constructor type for tag '{:?}': {:?}", tag, constructor))
                }
            }
        }
        None => {
            Err(format!("No constructor registered for external form tag '{:?}'", tag))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::Value;

    #[test]
    fn test_registry_basic_operations() {
        let registry = ExternalFormRegistry::new();
        let tag = crate::utils::intern_symbol("test-tag");
        let constructor = Value::Primitive {
            name: "test-constructor".to_string(),
            arity: 2,
            pointer: 0,
        };

        // Initially empty
        assert_eq!(registry.len().unwrap(), 0);
        assert!(registry.is_empty().unwrap());
        assert!(!registry.has_constructor(tag).unwrap());

        // Register constructor
        assert!(registry.define_constructor(tag, constructor.clone()).is_ok());
        assert_eq!(registry.len().unwrap(), 1);
        assert!(!registry.is_empty().unwrap());
        assert!(registry.has_constructor(tag).unwrap());

        // Retrieve constructor
        let retrieved = registry.get_constructor(tag).unwrap();
        assert!(retrieved.is_some());
        
        // Remove constructor
        let removed = registry.remove_constructor(tag).unwrap();
        assert!(removed.is_some());
        assert_eq!(registry.len().unwrap(), 0);
        assert!(registry.is_empty().unwrap());
    }

    #[test]
    fn test_registry_invalid_constructor() {
        let registry = ExternalFormRegistry::new();
        let tag = crate::utils::intern_symbol("test-tag");
        let invalid_constructor = Value::integer(42);

        // Should reject non-procedure values
        assert!(registry.define_constructor(tag, invalid_constructor).is_err());
    }

    #[test]
    fn test_registry_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let registry = Arc::new(ExternalFormRegistry::new());
        let mut handles = vec![];

        // Spawn multiple threads to register constructors
        for i in 0..10 {
            let registry = Arc::clone(&registry);
            let handle = thread::spawn(move || {
                let tag = crate::utils::intern_symbol(&format!("tag-{}", i));
                let constructor = Value::Primitive {
                    name: format!("constructor-{}", i),
                    arity: 1,
                    pointer: i as *const (),
                };
                registry.define_constructor(tag, constructor)
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            assert!(handle.join().unwrap().is_ok());
        }

        // Verify all constructors were registered
        assert_eq!(registry.len().unwrap(), 10);
    }

    #[test]
    fn test_global_registry() {
        let registry1 = global_registry();
        let registry2 = global_registry();

        // Should be the same instance
        assert!(std::ptr::eq(registry1, registry2));
    }
}
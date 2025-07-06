//! Copy-on-Write Environment implementation for Phase 4 optimization
//!
//! This module provides memory-efficient environment sharing using
//! copy-on-write semantics and immutable parent chain sharing.

use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// Shared environment using copy-on-write optimization
/// Reduces memory usage by sharing immutable parent environments
#[derive(Debug, Clone)]
pub struct SharedEnvironment {
    /// Local bindings for this environment frame
    /// Only contains bindings added to this specific frame
    local_bindings: HashMap<String, Value>,

    /// Shared parent environment chain
    /// Uses Rc for efficient sharing without cloning
    parent: Option<Rc<SharedEnvironment>>,

    /// Cached immutable bindings for fast lookup
    /// Contains flattened view of all bindings up the chain
    immutable_cache: Option<Rc<HashMap<String, Value>>>,

    /// Generation counter for cache invalidation
    /// Incremented when local bindings change
    generation: u32,

    /// Whether this environment is "frozen" (immutable)
    /// Frozen environments can be safely shared and cached
    is_frozen: bool,
}

impl SharedEnvironment {
    /// Create a new global shared environment
    pub fn new() -> Self {
        SharedEnvironment {
            local_bindings: HashMap::new(),
            parent: None,
            immutable_cache: None,
            generation: 0,
            is_frozen: false,
        }
    }

    /// Create a new shared environment with a parent
    pub fn with_parent(parent: Rc<SharedEnvironment>) -> Self {
        SharedEnvironment {
            local_bindings: HashMap::new(),
            parent: Some(parent),
            immutable_cache: None,
            generation: 0,
            is_frozen: false,
        }
    }

    /// Create environment with initial bindings (copy-on-write optimized)
    pub fn with_bindings(bindings: HashMap<String, Value>) -> Self {
        let is_empty = bindings.is_empty();
        SharedEnvironment {
            local_bindings: bindings,
            parent: None,
            immutable_cache: None,
            generation: 0,
            is_frozen: is_empty, // Empty environments can be frozen immediately
        }
    }

    /// Extend environment with new bindings using copy-on-write
    /// If no bindings are provided, returns a clone (shared reference)
    pub fn extend_cow(&self, bindings: Vec<(String, Value)>) -> Self {
        if bindings.is_empty() {
            // No new bindings, return shared reference
            self.clone()
        } else {
            // Create new environment with bindings
            let mut new_bindings = HashMap::with_capacity(bindings.len());
            for (name, value) in bindings {
                new_bindings.insert(name, value);
            }

            SharedEnvironment {
                local_bindings: new_bindings,
                parent: if self.is_empty() {
                    self.parent.clone()
                } else {
                    Some(Rc::new(self.clone()))
                },
                immutable_cache: None,
                generation: 0,
                is_frozen: false,
            }
        }
    }

    /// Define a variable in the current environment
    /// Invalidates cache if environment was previously cached
    pub fn define(&mut self, name: String, value: Value) {
        if self.is_frozen {
            // Cannot modify frozen environment, this indicates a programming error
            panic!("Attempt to modify frozen environment");
        }

        self.local_bindings.insert(name, value);
        self.invalidate_cache();
    }

    /// Set a variable (must already exist in this environment or a parent)
    /// Uses copy-on-write semantics for modifications
    pub fn set(&mut self, name: &str, value: Value) -> Result<()> {
        if self.is_frozen {
            return Err(LambdustError::runtime_error(
                "Cannot modify frozen environment".to_string(),
            ));
        }

        // Check if variable exists in local bindings
        if self.local_bindings.contains_key(name) {
            self.local_bindings.insert(name.to_string(), value);
            self.invalidate_cache();
            return Ok(());
        }

        // Check if variable exists in parent chain
        if self.exists_in_parents(name) {
            // Copy-on-write: bring the binding into local scope
            self.local_bindings.insert(name.to_string(), value);
            self.invalidate_cache();
            return Ok(());
        }

        Err(LambdustError::runtime_error(format!(
            "Undefined variable: {}",
            name
        )))
    }

    /// Get a variable value with cached lookup optimization
    pub fn get(&self, name: &str) -> Option<Value> {
        // Fast path: check local bindings first
        if let Some(value) = self.local_bindings.get(name) {
            return Some(value.clone());
        }

        // Medium path: check immutable cache
        if let Some(cache) = &self.immutable_cache {
            if let Some(value) = cache.get(name) {
                return Some(value.clone());
            }
        }

        // Slow path: traverse parent chain
        self.lookup_in_parents(name)
    }

    /// Lookup variable in parent environments
    fn lookup_in_parents(&self, name: &str) -> Option<Value> {
        let mut current = self.parent.as_ref();
        while let Some(env) = current {
            if let Some(value) = env.get(name) {
                return Some(value);
            }
            current = env.parent.as_ref();
        }
        None
    }

    /// Check if variable exists anywhere in the environment chain
    pub fn exists(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Check if variable exists in parent environments
    fn exists_in_parents(&self, name: &str) -> bool {
        let mut current = self.parent.as_ref();
        while let Some(env) = current {
            if env.local_bindings.contains_key(name) || env.exists_in_parents(name) {
                return true;
            }
            current = env.parent.as_ref();
        }
        false
    }

    /// Build immutable cache for fast lookups
    /// This flattens the environment chain into a single HashMap
    pub fn build_cache(&mut self) {
        if self.immutable_cache.is_some() {
            return; // Cache already exists
        }

        let mut cache = HashMap::new();

        // Collect bindings from parent chain (outer to inner)
        let mut parent_bindings = Vec::new();
        let mut current = self.parent.as_ref();
        while let Some(env) = current {
            parent_bindings.push(&env.local_bindings);
            current = env.parent.as_ref();
        }

        // Add parent bindings (reverse order for correct shadowing)
        for bindings in parent_bindings.iter().rev() {
            cache.extend(bindings.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        // Local bindings override parent bindings
        cache.extend(
            self.local_bindings
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );

        self.immutable_cache = Some(Rc::new(cache));
    }

    /// Invalidate cache when environment changes
    fn invalidate_cache(&mut self) {
        self.immutable_cache = None;
        self.generation += 1;
    }

    /// Freeze environment to make it immutable and shareable
    /// Frozen environments can be safely shared without cloning
    pub fn freeze(&mut self) {
        self.is_frozen = true;
        self.build_cache(); // Build cache before freezing
    }

    /// Check if environment is empty (no local bindings)
    pub fn is_empty(&self) -> bool {
        self.local_bindings.is_empty()
    }

    /// Check if environment is frozen
    pub fn is_frozen(&self) -> bool {
        self.is_frozen
    }

    /// Get environment depth (distance from root)
    pub fn depth(&self) -> usize {
        match &self.parent {
            Some(parent) => parent.depth() + 1,
            None => 0,
        }
    }

    /// Get total number of bindings in environment chain
    pub fn total_bindings(&self) -> usize {
        let local_count = self.local_bindings.len();
        match &self.parent {
            Some(parent) => local_count + parent.total_bindings(),
            None => local_count,
        }
    }

    /// Get memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        let local_size = self.local_bindings.len()
            * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());

        let cache_size = self
            .immutable_cache
            .as_ref()
            .map(|cache| {
                cache.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>())
            })
            .unwrap_or(0);

        let parent_size = self
            .parent
            .as_ref()
            .map(|_| std::mem::size_of::<Rc<SharedEnvironment>>())
            .unwrap_or(0);

        local_size + cache_size + parent_size
    }

    /// Convert to iterator over all bindings (for debugging)
    pub fn iter_all_bindings(&self) -> HashMap<String, Value> {
        let mut all_bindings = HashMap::new();

        // Collect from parent chain first
        if let Some(parent) = &self.parent {
            all_bindings.extend(parent.iter_all_bindings());
        }

        // Add local bindings (these override parent bindings)
        all_bindings.extend(
            self.local_bindings
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );

        all_bindings
    }
}

impl Default for SharedEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment reference wrapper for choosing between traditional and COW environments
#[derive(Debug, Clone)]
pub enum EnvironmentStrategy {
    /// Traditional environment (existing implementation)
    Traditional(super::traditional::Environment),
    /// Shared COW environment (Phase 4 optimization)
    Shared(SharedEnvironment),
}

impl EnvironmentStrategy {
    /// Create new environment using COW strategy
    pub fn new_shared() -> Self {
        EnvironmentStrategy::Shared(SharedEnvironment::new())
    }

    /// Create new environment using traditional strategy
    pub fn new_traditional() -> Self {
        EnvironmentStrategy::Traditional(super::traditional::Environment::new())
    }

    /// Define variable in environment
    pub fn define(&mut self, name: String, value: Value) {
        match self {
            EnvironmentStrategy::Traditional(env) => env.define(name, value),
            EnvironmentStrategy::Shared(env) => env.define(name, value),
        }
    }

    /// Set variable in environment
    pub fn set(&mut self, name: &str, value: Value) -> Result<()> {
        match self {
            EnvironmentStrategy::Traditional(env) => env.set(name, value),
            EnvironmentStrategy::Shared(env) => env.set(name, value),
        }
    }

    /// Get variable from environment
    pub fn get(&self, name: &str) -> Option<Value> {
        match self {
            EnvironmentStrategy::Traditional(env) => env.get(name),
            EnvironmentStrategy::Shared(env) => env.get(name),
        }
    }

    /// Check if variable exists
    pub fn exists(&self, name: &str) -> bool {
        match self {
            EnvironmentStrategy::Traditional(env) => env.exists(name),
            EnvironmentStrategy::Shared(env) => env.exists(name),
        }
    }
}

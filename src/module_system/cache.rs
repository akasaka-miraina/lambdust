//! Module caching system for efficient module loading.
//!
//! Implements an intelligent caching layer that:
//! - Prevents redundant module loading
//! - Manages module dependencies efficiently
//! - Provides cache invalidation mechanisms
//! - Supports concurrent access patterns

use super::{Module, ModuleId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Thread-safe cache for loaded modules.
#[derive(Debug)]
pub struct ModuleCache {
    /// Cache storage mapping module IDs to loaded modules
    cache: RwLock<HashMap<ModuleId, CacheEntry>>,
    /// Cache configuration
    config: CacheConfig,
}

/// Configuration for the module cache.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of modules to keep in cache
    pub max_entries: usize,
    /// Time-to-live for cache entries (None for infinite)
    pub ttl: Option<Duration>,
    /// Whether to enable dependency-based invalidation
    pub dependency_invalidation: bool,
}

/// A cached module entry with metadata.
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached module
    module: Arc<Module>,
    /// When this entry was created
    created_at: SystemTime,
    /// Last access time for LRU eviction
    last_accessed: SystemTime,
    /// Access count for usage statistics
    access_count: u64,
}

impl ModuleCache {
    /// Creates a new module cache with default configuration.
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Creates a new module cache with custom configuration.
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Gets a module from the cache if it exists.
    pub fn get(&self, id: &ModuleId) -> Option<Arc<Module>> {
        let mut cache = self.cache.write().ok()?;
        
        if let Some(entry) = cache.get_mut(id) {
            // Check TTL if configured
            if let Some(ttl) = self.config.ttl {
                if entry.created_at.elapsed().ok()? > ttl {
                    cache.remove(id);
                    return None;
                }
            }
            
            // Update access statistics
            entry.last_accessed = SystemTime::now();
            entry.access_count += 1;
            
            Some(entry.module.clone())
        } else {
            None
        }
    }

    /// Inserts a module into the cache.
    pub fn insert(&self, id: ModuleId, module: Arc<Module>) {
        let mut cache = self.cache.write().expect("Cache write lock poisoned");
        
        // Check if we need to evict entries
        if cache.len() >= self.config.max_entries {
            self.evict_lru(&mut cache);
        }
        
        let entry = CacheEntry {
            module,
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            access_count: 1,
        };
        
        cache.insert(id, entry);
    }

    /// Removes a module from the cache.
    pub fn remove(&self, id: &ModuleId) -> Option<Arc<Module>> {
        let mut cache = self.cache.write().expect("Cache write lock poisoned");
        cache.remove(id).map(|entry| entry.module)
    }

    /// Clears all entries from the cache.
    pub fn clear(&self) {
        let mut cache = self.cache.write().expect("Cache write lock poisoned");
        cache.clear();
    }

    /// Gets the number of cached modules.
    pub fn len(&self) -> usize {
        let cache = self.cache.read().expect("Cache read lock poisoned");
        cache.len()
    }

    /// Checks if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Lists all cached module IDs.
    pub fn list_modules(&self) -> Vec<ModuleId> {
        let cache = self.cache.read().expect("Cache read lock poisoned");
        cache.keys().clone())().collect()
    }

    /// Gets cache statistics.
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read().expect("Cache read lock poisoned");
        
        let total_accesses: u64 = cache.values().map(|entry| entry.access_count).sum();
        let oldest_entry = cache.values()
            .min_by_key(|entry| entry.created_at)
            .map(|entry| entry.created_at);
        let newest_entry = cache.values()
            .max_by_key(|entry| entry.created_at)
            .map(|entry| entry.created_at);
        
        CacheStats {
            entry_count: cache.len(),
            total_accesses,
            oldest_entry,
            newest_entry,
        }
    }

    /// Invalidates modules that depend on the given module ID.
    pub fn invalidate_dependents(&self, id: &ModuleId) {
        if !self.config.dependency_invalidation {
            return;
        }

        let mut cache = self.cache.write().expect("Cache write lock poisoned");
        let mut to_remove = Vec::new();
        
        // Find modules that depend on the given module
        for (cached_id, entry) in cache.iter() {
            if entry.module.dependencies.contains(id) {
                to_remove.push(cached_id.clone());
            }
        }
        
        // Remove dependent modules
        for dependent_id in to_remove {
            cache.remove(&dependent_id);
        }
    }

    /// Evicts the least recently used entry.
    fn evict_lru(&self, cache: &mut HashMap<ModuleId, CacheEntry>) {
        if let Some((lru_id, _)) = cache.iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(id, entry)| (id.clone()), entry.clone()))
        {
            cache.remove(&lru_id);
        }
    }

    /// Validates cache integrity by checking dependencies.
    pub fn validate(&self) -> Vec<CacheValidationError> {
        let cache = self.cache.read().expect("Cache read lock poisoned");
        let mut errors = Vec::new();
        
        for (id, entry) in cache.iter() {
            // Check if all dependencies are satisfied
            for dep_id in &entry.module.dependencies {
                if !cache.contains_key(dep_id) {
                    errors.push(CacheValidationError::MissingDependency {
                        module: id.clone()),
                        dependency: dep_id.clone()),
                    });
                }
            }
            
            // Check for circular dependencies (basic check)
            if entry.module.dependencies.contains(id) {
                errors.push(CacheValidationError::SelfDependency(id.clone()));
            }
        }
        
        errors
    }
}

/// Cache statistics for monitoring and debugging.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries in the cache
    pub entry_count: usize,
    /// Total number of cache accesses
    pub total_accesses: u64,
    /// Creation time of the oldest entry
    pub oldest_entry: Option<SystemTime>,
    /// Creation time of the newest entry
    pub newest_entry: Option<SystemTime>,
}

/// Cache validation errors.
#[derive(Debug, Clone)]
pub enum CacheValidationError {
    /// A module has a dependency that is not in the cache
    MissingDependency {
        /// The module that has the missing dependency
        module: ModuleId,
        /// The dependency that is missing from the cache
        dependency: ModuleId,
    },
    /// A module depends on itself
    SelfDependency(ModuleId),
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl: None, // No expiration by default
            dependency_invalidation: true,
        }
    }
}

impl Default for ModuleCache {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CacheValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheValidationError::MissingDependency { module, dependency } => {
                write!(f, "Module {} has missing dependency {}", 
                       super::format_module_id(module), 
                       super::format_module_id(dependency))
            }
            CacheValidationError::SelfDependency(module) => {
                write!(f, "Module {} depends on itself", super::format_module_id(module))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{ModuleNamespace, ModuleSource, ModuleMetadata};
    use std::collections::HashMap;

    fn create_test_module(name: &str) -> Arc<Module> {
        Arc::new(Module {
            id: ModuleId {
                components: vec![name.to_string()],
                namespace: ModuleNamespace::Builtin,
            },
            exports: HashMap::new(),
            dependencies: Vec::new(),
            source: Some(ModuleSource::Builtin),
            metadata: ModuleMetadata::default(),
        })
    }

    #[test]
    fn test_cache_basic_operations() {
        let cache = ModuleCache::new();
        let module_id = ModuleId {
            components: vec!["test".to_string()],
            namespace: ModuleNamespace::Builtin,
        };
        let module = create_test_module("test");

        // Initially empty
        assert!(cache.get(&module_id).is_none());
        assert_eq!(cache.len(), 0);

        // Insert and retrieve
        cache.insert(module_id.clone()), module.clone());
        assert_eq!(cache.len(), 1);
        
        let retrieved = cache.get(&module_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, module_id);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let config = CacheConfig {
            max_entries: 2,
            ttl: None,
            dependency_invalidation: false,
        };
        let cache = ModuleCache::with_config(config);

        // Insert two modules
        let id1 = ModuleId {
            components: vec!["mod1".to_string()],
            namespace: ModuleNamespace::Builtin,
        };
        let id2 = ModuleId {
            components: vec!["mod2".to_string()],
            namespace: ModuleNamespace::Builtin,
        };
        let id3 = ModuleId {
            components: vec!["mod3".to_string()],
            namespace: ModuleNamespace::Builtin,
        };

        cache.insert(id1.clone()), create_test_module("mod1"));
        cache.insert(id2.clone()), create_test_module("mod2"));
        assert_eq!(cache.len(), 2);

        // Access first module to make it more recently used
        cache.get(&id1);

        // Insert third module - should evict mod2 (least recently used)
        cache.insert(id3.clone()), create_test_module("mod3"));
        assert_eq!(cache.len(), 2);
        
        assert!(cache.get(&id1).is_some());
        assert!(cache.get(&id2).is_none());
        assert!(cache.get(&id3).is_some());
    }

    #[test]
    fn test_cache_clear() {
        let cache = ModuleCache::new();
        let module_id = ModuleId {
            components: vec!["test".to_string()],
            namespace: ModuleNamespace::Builtin,
        };

        cache.insert(module_id, create_test_module("test"));
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_stats() {
        let cache = ModuleCache::new();
        let module_id = ModuleId {
            components: vec!["test".to_string()],
            namespace: ModuleNamespace::Builtin,
        };

        cache.insert(module_id.clone()), create_test_module("test"));
        
        // Access the module a few times
        cache.get(&module_id);
        cache.get(&module_id);
        cache.get(&module_id);

        let stats = cache.stats();
        assert_eq!(stats.entry_count, 1);
        assert_eq!(stats.total_accesses, 4); // 1 from insert + 3 from gets
        assert!(stats.oldest_entry.is_some());
        assert!(stats.newest_entry.is_some());
    }
}
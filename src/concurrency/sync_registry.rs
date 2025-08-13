//! Synchronization primitives registry for managing named primitives.
//!
//! This module provides a central registry for managing named synchronization
//! primitives, allowing for easy lookup and coordination across the system.

use crate::diagnostics::{Error, Result, error::helpers};
use super::{Mutex, RwLock, SemaphoreSync, CondVar, AtomicCounter, AtomicFlag};
use std::sync::{Arc, Mutex as StdMutex};
use std::collections::HashMap;

/// Synchronization primitives registry for managing named primitives.
#[derive(Debug)]
pub struct SyncRegistry {
    mutexes: StdMutex<HashMap<String, Mutex>>,
    rwlocks: StdMutex<HashMap<String, RwLock>>,
    semaphores: StdMutex<HashMap<String, SemaphoreSync>>,
    condvars: StdMutex<HashMap<String, CondVar>>,
    counters: StdMutex<HashMap<String, AtomicCounter>>,
    flags: StdMutex<HashMap<String, AtomicFlag>>,
}

impl SyncRegistry {
    /// Creates a new synchronization registry.
    pub fn new() -> Self {
        Self {
            mutexes: StdMutex::new(HashMap::new()),
            rwlocks: StdMutex::new(HashMap::new()),
            semaphores: StdMutex::new(HashMap::new()),
            condvars: StdMutex::new(HashMap::new()),
            counters: StdMutex::new(HashMap::new()),
            flags: StdMutex::new(HashMap::new()),
        }
    }

    /// Registers a named mutex.
    pub fn register_mutex(&self, name: String, mutex: Mutex) -> Result<()> {
        let mut mutexes = self.mutexes.lock()
            .map_err(|_| Error::runtime_error("Failed to lock mutex registry".to_string(), None))?;
        mutexes.insert(name, mutex);
        Ok(())
    }

    /// Gets a named mutex.
    pub fn get_mutex(&self, name: &str) -> Result<Mutex> {
        let mutexes = self.mutexes.lock()
            .map_err(|_| helpers::runtime_error_simple("Failed to lock mutex registry"))?;
        mutexes.get(name)
            .cloned()
            .ok_or_else(|| helpers::runtime_error_simple(format!("Mutex '{name}' not found")))
    }

    // Similar methods for other primitives...
    // (Implementation would be similar for RwLock, Semaphore, etc.)
}

impl Default for SyncRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global synchronization primitives registry.
static SYNC_REGISTRY: std::sync::OnceLock<Arc<SyncRegistry>> = std::sync::OnceLock::new();

/// Gets the global synchronization registry.
pub fn global_sync_registry() -> Arc<SyncRegistry> {
    SYNC_REGISTRY.get_or_init(|| Arc::new(SyncRegistry::new())).clone()
}
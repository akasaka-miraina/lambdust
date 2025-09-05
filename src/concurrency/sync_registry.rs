//! Synchronization primitives registry for managing named primitives.
//!
//! This module provides a central registry for managing named synchronization
//! primitives, allowing for easy lookup and coordination across the system.

use super::{AtomicCounter, AtomicFlag};

#[cfg(feature = "async-runtime")]
use super::{CondVar, Mutex, RwLock, SemaphoreSync};
use crate::diagnostics::{Error, Result, error::helpers};
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};

/// Synchronization primitives registry for managing named primitives.
#[derive(Debug)]
pub struct SyncRegistry {
    #[cfg(feature = "async-runtime")]
    mutexes: StdMutex<HashMap<String, Mutex>>,
    #[cfg(feature = "async-runtime")]
    rwlocks: StdMutex<HashMap<String, RwLock>>,
    #[cfg(feature = "async-runtime")]
    semaphores: StdMutex<HashMap<String, SemaphoreSync>>,
    #[cfg(feature = "async-runtime")]
    condvars: StdMutex<HashMap<String, CondVar>>,
    counters: StdMutex<HashMap<String, AtomicCounter>>,
    flags: StdMutex<HashMap<String, AtomicFlag>>,
}

impl SyncRegistry {
    /// Creates a new synchronization registry.
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "async-runtime")]
            mutexes: StdMutex::new(HashMap::new()),
            #[cfg(feature = "async-runtime")]
            rwlocks: StdMutex::new(HashMap::new()),
            #[cfg(feature = "async-runtime")]
            semaphores: StdMutex::new(HashMap::new()),
            #[cfg(feature = "async-runtime")]
            condvars: StdMutex::new(HashMap::new()),
            counters: StdMutex::new(HashMap::new()),
            flags: StdMutex::new(HashMap::new()),
        }
    }

    /// Registers a named mutex.
    pub fn register_mutex(&self, name: String, mutex: Mutex) -> Result<()> {
        let mut mutexes = self
            .mutexes
            .lock()
            .map_err(|_| Error::runtime_error("Failed to lock mutex registry".to_string(), None))?;
        mutexes.insert(name, mutex);
        Ok(())
    }

    /// Gets a named mutex.
    pub fn get_mutex(&self, name: &str) -> Result<Mutex> {
        let mutexes = self
            .mutexes
            .lock()
            .map_err(|_| helpers::runtime_error_simple("Failed to lock mutex registry"))?;
        mutexes
            .get(name)
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
    SYNC_REGISTRY
        .get_or_init(|| Arc::new(SyncRegistry::new()))
        .clone()
}

//! Mutual exclusion lock for protecting shared data.
//!
//! This module provides async mutex functionality with timeout support
//! and optional naming for debugging purposes.

use crate::eval::Value;
use crate::diagnostics::{Error, Result};
use super::ConcurrencyError;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex as AsyncMutex};

/// Mutual exclusion lock for protecting shared data.
#[derive(Debug, Clone)]
pub struct Mutex {
    inner: Arc<AsyncMutex<Value>>,
    name: Option<String>,
}

impl Mutex {
    /// Creates a new mutex with an initial value.
    pub fn new(value: Value) -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(value)),
            name: None,
        }
    }

    /// Creates a new named mutex.
    pub fn with_name(value: Value, name: String) -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(value)),
            name: Some(name),
        }
    }

    /// Locks the mutex and returns a guard.
    pub async fn lock(&self) -> MutexGuard<'_> {
        let guard = self.inner.lock().await;
        MutexGuard { guard }
    }

    /// Attempts to lock the mutex without blocking.
    pub fn try_lock(&self) -> Result<MutexGuard<'_>> {
        match self.inner.try_lock() {
            Ok(guard) => Ok(MutexGuard { guard }),
            Err(_) => Err(Box::new(Error::runtime_error("Mutex is locked".to_string(), None))),
        }
    }

    /// Locks the mutex with a timeout.
    pub async fn lock_timeout(&self, duration: Duration) -> Result<MutexGuard<'_>> {
        match tokio::time::timeout(duration, self.lock()).await {
            Ok(guard) => Ok(guard),
            Err(_) => Err(ConcurrencyError::Timeout.into()),
        }
    }

    /// Gets the name of the mutex.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// RAII guard for mutex locks.
pub struct MutexGuard<'a> {
    guard: tokio::sync::MutexGuard<'a, Value>,
}

impl<'a> MutexGuard<'a> {
    /// Gets a reference to the protected value.
    pub fn get(&self) -> &Value {
        &self.guard
    }

    /// Gets a mutable reference to the protected value.
    pub fn get_mut(&mut self) -> &mut Value {
        &mut self.guard
    }

    /// Sets the protected value.
    pub fn set(&mut self, value: Value) {
        *self.guard = value;
    }
}
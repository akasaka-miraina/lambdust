//! Reader-writer lock for shared data with concurrent reads.
//!
//! This module provides async RwLock functionality that allows multiple
//! concurrent readers or a single writer, with optional naming for debugging.

use crate::eval::Value;
use crate::diagnostics::{Error, Result};
use std::sync::Arc;
use tokio::sync::{RwLock as AsyncRwLock};

/// Reader-writer lock for shared data with concurrent reads.
#[derive(Debug, Clone)]
pub struct RwLock {
    inner: Arc<AsyncRwLock<Value>>,
    name: Option<String>,
}

impl RwLock {
    /// Creates a new RwLock with an initial value.
    pub fn new(value: Value) -> Self {
        Self {
            inner: Arc::new(AsyncRwLock::new(value)),
            name: None,
        }
    }

    /// Creates a new named RwLock.
    pub fn with_name(value: Value, name: String) -> Self {
        Self {
            inner: Arc::new(AsyncRwLock::new(value)),
            name: Some(name),
        }
    }

    /// Acquires a read lock.
    pub async fn read(&self) -> ReadGuard<'_> {
        let guard = self.inner.read().await;
        ReadGuard { guard }
    }

    /// Acquires a write lock.
    pub async fn write(&self) -> WriteGuard<'_> {
        let guard = self.inner.write().await;
        WriteGuard { guard }
    }

    /// Attempts to acquire a read lock without blocking.
    pub fn try_read(&self) -> Result<ReadGuard<'_>> {
        match self.inner.try_read() {
            Ok(guard) => Ok(ReadGuard { guard }),
            Err(_) => Err(Box::new(Error::runtime_error("RwLock is write-locked".to_string(), None))),
        }
    }

    /// Attempts to acquire a write lock without blocking.
    pub fn try_write(&self) -> Result<WriteGuard<'_>> {
        match self.inner.try_write() {
            Ok(guard) => Ok(WriteGuard { guard }),
            Err(_) => Err(Box::new(Error::runtime_error("RwLock is locked".to_string(), None))),
        }
    }

    /// Gets the name of the RwLock.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// RAII guard for read locks.
pub struct ReadGuard<'a> {
    guard: tokio::sync::RwLockReadGuard<'a, Value>,
}

impl<'a> ReadGuard<'a> {
    /// Gets a reference to the protected value.
    pub fn get(&self) -> &Value {
        &self.guard
    }
}

/// RAII guard for write locks.
pub struct WriteGuard<'a> {
    guard: tokio::sync::RwLockWriteGuard<'a, Value>,
}

impl<'a> WriteGuard<'a> {
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
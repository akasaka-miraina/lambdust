//! Semaphore for controlling access to a limited resource.
//!
//! This module provides async semaphore functionality for controlling
//! the number of concurrent operations with permits system.

use crate::diagnostics::{Error, Result};
use super::ConcurrencyError;
use std::sync::Arc;
use tokio::sync::{Semaphore};

/// Semaphore for controlling access to a limited resource.
#[derive(Debug, Clone)]
pub struct SemaphoreSync {
    inner: Arc<Semaphore>,
    name: Option<String>,
}

impl SemaphoreSync {
    /// Creates a new semaphore with the given number of permits.
    pub fn new(permits: usize) -> Self {
        Self {
            inner: Arc::new(Semaphore::new(permits)),
            name: None,
        }
    }

    /// Creates a new named semaphore.
    pub fn with_name(permits: usize, name: String) -> Self {
        Self {
            inner: Arc::new(Semaphore::new(permits)),
            name: Some(name),
        }
    }

    /// Acquires a permit from the semaphore.
    pub async fn acquire(&self) -> Result<SemaphorePermit<'_>> {
        let permit = self.inner.acquire().await
            .map_err(|_| ConcurrencyError::ChannelClosed)?;
        Ok(SemaphorePermit { permit })
    }

    /// Attempts to acquire a permit without blocking.
    pub fn try_acquire(&self) -> Result<SemaphorePermit<'_>> {
        let permit = self.inner.try_acquire()
            .map_err(|_| Error::runtime_error("No permits available".to_string(), None))?;
        Ok(SemaphorePermit { permit })
    }

    /// Acquires multiple permits.
    pub async fn acquire_many(&self, permits: u32) -> Result<SemaphorePermit<'_>> {
        let permit = self.inner.acquire_many(permits).await
            .map_err(|_| ConcurrencyError::ChannelClosed)?;
        Ok(SemaphorePermit { permit })
    }

    /// Gets the current number of available permits.
    pub fn available_permits(&self) -> usize {
        self.inner.available_permits()
    }

    /// Adds permits to the semaphore.
    pub fn add_permits(&self, n: usize) {
        self.inner.add_permits(n);
    }

    /// Gets the name of the semaphore.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// RAII guard for semaphore permits.
pub struct SemaphorePermit<'a> {
    permit: tokio::sync::SemaphorePermit<'a>,
}

impl<'a> Drop for SemaphorePermit<'a> {
    fn drop(&mut self) {
        // Permit is automatically returned when dropped
    }
}
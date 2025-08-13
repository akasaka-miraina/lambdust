//! Condition variable for thread coordination.
//!
//! This module provides async condition variable functionality for coordinating
//! tasks that need to wait for certain conditions to become true.

use crate::diagnostics::Result;
use super::ConcurrencyError;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

/// Condition variable for thread coordination.
#[derive(Debug, Clone)]
pub struct CondVar {
    notify: Arc<Notify>,
    name: Option<String>,
}

impl CondVar {
    /// Creates a new condition variable.
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            name: None,
        }
    }

    /// Creates a new named condition variable.
    pub fn with_name(name: String) -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            name: Some(name),
        }
    }

    /// Waits for a notification.
    pub async fn wait(&self) {
        self.notify.notified().await;
    }

    /// Waits for a notification with a timeout.
    pub async fn wait_timeout(&self, duration: Duration) -> Result<()> {
        match tokio::time::timeout(duration, self.wait()).await {
            Ok(_) => Ok(()),
            Err(_) => Err(ConcurrencyError::Timeout.into()),
        }
    }

    /// Notifies one waiting task.
    pub fn notify_one(&self) {
        self.notify.notify_one();
    }

    /// Notifies all waiting tasks.
    pub fn notify_all(&self) {
        self.notify.notify_waiters();
    }

    /// Gets the name of the condition variable.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl Default for CondVar {
    fn default() -> Self {
        Self::new()
    }
}
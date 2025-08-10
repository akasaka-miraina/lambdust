//! High-performance concurrency and parallelism library for R7RS-large compliance.
//!
//! This module provides comprehensive concurrent programming primitives including:
//! - Future/Promise system with async/await paradigm
//! - Typed channels with select operations
//! - Parallel computation primitives
//! - Synchronization primitives
//! - Actor model implementation
//! - Distributed processing foundation

pub mod futures;
pub mod channels;
pub mod parallel;
pub mod sync;
pub mod actors;
pub mod distributed;
pub mod scheduler;

// Individual structure modules
pub mod concurrency_runtime;
pub mod mutex;
pub mod rwlock;
pub mod semaphore;
pub mod condvar;
pub mod barrier;
pub mod atomic_ref;
pub mod lockfree_queue;
pub mod atomic_primitives;
pub mod sync_registry;

#[cfg(test)]
mod tests;

// Re-export individual structures
pub use concurrency_runtime::*;
pub use mutex::{Mutex, MutexGuard};
pub use rwlock::{RwLock, ReadGuard, WriteGuard};
pub use semaphore::{SemaphoreSync, SemaphorePermit};
pub use condvar::CondVar;
pub use barrier::{Barrier, BarrierWaitResult};
pub use atomic_ref::AtomicRef;
pub use lockfree_queue::{LockFreeQueue, BoundedLockFreeQueue};
pub use atomic_primitives::{AtomicCounter, AtomicFlag};
pub use sync_registry::{SyncRegistry, global_sync_registry};

use crate::diagnostics::{Error, Result};

/// Error types specific to concurrency operations.
#[derive(Debug, thiserror::Error)]
pub enum ConcurrencyError {
    /// Channel has been closed and can no longer send/receive messages
    #[error("Channel closed")]
    ChannelClosed,
    
    /// Operation timed out before completion
    #[error("Timeout expired")]
    Timeout,
    
    /// Task was cancelled before completion
    #[error("Task cancelled")]
    Cancelled,
    
    /// Deadlock detected in the system
    #[error("Deadlock detected")]
    Deadlock,
    
    /// Actor with the specified name was not found
    #[error("Actor not found: {0}")]
    ActorNotFound(String),
    
    /// Error during serialization/deserialization
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Network-related error
    #[error("Network error: {0}")]
    Network(String),
}

impl From<ConcurrencyError> for Error {
    fn from(err: ConcurrencyError) -> Self {
        Error::runtime_error(err.to_string(), None)
    }
}

impl From<ConcurrencyError> for Box<Error> {
    fn from(err: ConcurrencyError) -> Self {
        Error::from(err).into()
    }
}

impl ConcurrencyError {
    /// Converts this ConcurrencyError into a Box<Error> for use with the Result type.
    pub fn boxed(self) -> Box<Error> {
        Box::new(Error::from(self))
    }
}

/// Initialize the concurrency system.
/// 
/// This should be called once during startup to set up the
/// global runtime and any necessary background tasks.
pub fn initialize() -> Result<()> {
    // Ensure the global runtime is initialized
    let _runtime = ConcurrencyRuntime::global();
    
    // Initialize the actor system
    actors::initialize()?;
    
    // Initialize the work-stealing scheduler
    scheduler::initialize()?;
    
    Ok(())
}

/// Shutdown the concurrency system gracefully.
pub async fn shutdown() -> Result<()> {
    // Shutdown actors
    actors::shutdown().await?;
    
    // Shutdown scheduler
    scheduler::shutdown().await?;
    
    Ok(())
}
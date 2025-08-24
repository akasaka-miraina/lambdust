//! High-performance concurrency and parallelism library for R7RS-large compliance.
//!
//! This module provides comprehensive concurrent programming primitives including:
//! - Future/Promise system with async/await paradigm
//! - Typed channels with select operations
//! - Parallel computation primitives
//! - Synchronization primitives
//! - Actor model implementation
//! - Distributed processing foundation

// Core concurrency modules (always available)
pub mod adaptive_pointer;
pub mod sync;

// Parallel module has tokio dependencies, temporarily async-dependent
#[cfg(feature = "async-runtime")]
pub mod parallel;

// Async-dependent modules (require async-runtime feature)
#[cfg(feature = "async-runtime")]
pub mod actors;
#[cfg(feature = "async-runtime")]
pub mod channels;
#[cfg(feature = "async-runtime")]
pub mod distributed;
#[cfg(feature = "async-runtime")]
pub mod futures;
#[cfg(feature = "async-runtime")]
pub mod scheduler;

// Phase 3.3: Revolutionary Distributed Computing Framework - temporarily disabled for stabilization
// pub mod distributed_actor_framework;
pub mod distributed_config;
// pub mod distributed_continuation_system;
// pub mod distributed_fault_tolerance;
// pub mod distributed_integration_architecture;
// pub mod distributed_integration_master;
// pub mod distributed_load_balancer;

// Individual structure modules (core - no async dependencies)
pub mod atomic_primitives;
pub mod atomic_ref;
pub mod lockfree_queue;

// Temporarily disabled until feature-gating is complete
// pub mod sync_registry;

// Async-dependent individual modules
#[cfg(feature = "async-runtime")]
pub mod barrier;
#[cfg(feature = "async-runtime")]
pub mod concurrency_runtime;
#[cfg(feature = "async-runtime")]
pub mod condvar;
#[cfg(feature = "async-runtime")]
pub mod mutex;
#[cfg(feature = "async-runtime")]
pub mod rwlock;
#[cfg(feature = "async-runtime")]
pub mod semaphore;

// Temporarily disable tests due to tokio dependency issues
#[cfg(test)]
mod tests {
    // Tests disabled due to tokio dependency
}

// Re-export core individual structures (no async dependencies)
pub use adaptive_pointer::{
    AdaptivePointer, AdaptivePointerManager, AdaptivePointerStats, AdaptiveWeakPointer,
};
pub use atomic_primitives::{AtomicCounter, AtomicFlag};
pub use atomic_ref::AtomicRef;
pub use lockfree_queue::{BoundedLockFreeQueue, LockFreeQueue};
// Temporarily disabled: pub use sync_registry::{SyncRegistry, global_sync_registry};

// Re-export async-dependent structures
#[cfg(feature = "async-runtime")]
pub use barrier::{Barrier, BarrierWaitResult};
#[cfg(feature = "async-runtime")]
pub use concurrency_runtime::*;
#[cfg(feature = "async-runtime")]
pub use condvar::CondVar;
#[cfg(feature = "async-runtime")]
pub use mutex::{Mutex, MutexGuard};
#[cfg(feature = "async-runtime")]
pub use rwlock::{ReadGuard, RwLock, WriteGuard};
#[cfg(feature = "async-runtime")]
pub use semaphore::{SemaphorePermit, SemaphoreSync};

use crate::diagnostics::{Error, LambdustError, Result};

// Phase 3.3: Distributed Computing Framework Exports - temporarily disabled for stabilization
// All distributed modules disabled until stabilization is complete

// Import FaultToleranceConfig from distributed_config (canonical definition)
pub use distributed_config::FaultToleranceConfig;

/// Error types specific to concurrency operations.
#[derive(Debug)]
pub enum ConcurrencyError {
    /// Channel has been closed and can no longer send/receive messages
    ChannelClosed,

    /// Operation timed out before completion
    Timeout,

    /// Task was cancelled before completion
    Cancelled,

    /// Deadlock detected in the system
    Deadlock,

    /// Actor with the specified name was not found
    ActorNotFound(String),

    /// Error during serialization/deserialization
    Serialization(String),

    /// Network-related error
    Network(String),
}

impl std::fmt::Display for ConcurrencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChannelClosed => write!(f, "Channel closed"),
            Self::Timeout => write!(f, "Timeout expired"),
            Self::Cancelled => write!(f, "Task cancelled"),
            Self::Deadlock => write!(f, "Deadlock detected"),
            Self::ActorNotFound(name) => write!(f, "Actor not found: {name}"),
            Self::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Self::Network(msg) => write!(f, "Network error: {msg}"),
        }
    }
}

impl LambdustError for ConcurrencyError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::ChannelClosed => "lambdust::concurrency::channel_closed",
            Self::Timeout => "lambdust::concurrency::timeout",
            Self::Cancelled => "lambdust::concurrency::cancelled",
            Self::Deadlock => "lambdust::concurrency::deadlock",
            Self::ActorNotFound(_) => "lambdust::concurrency::actor_not_found",
            Self::Serialization(_) => "lambdust::concurrency::serialization",
            Self::Network(_) => "lambdust::concurrency::network",
        }
    }
}

impl std::error::Error for ConcurrencyError {}

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
    #[cfg(feature = "async-runtime")]
    {
        // Ensure the global runtime is initialized
        let _runtime = ConcurrencyRuntime::global();

        // Initialize the actor system
        actors::initialize()?;

        // Initialize the work-stealing scheduler
        scheduler::initialize()?;
    }

    Ok(())
}

/// Shutdown the concurrency system gracefully.
pub async fn shutdown() -> Result<()> {
    #[cfg(feature = "async-runtime")]
    {
        // Shutdown actors
        actors::shutdown().await?;

        // Shutdown scheduler
        scheduler::shutdown().await?;
    }

    Ok(())
}

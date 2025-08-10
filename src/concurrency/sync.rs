//! Comprehensive synchronization primitives for concurrent programming.
//!
//! This module coordinates all synchronization primitives which have been
//! distributed across dedicated modules for better organization.
//! 
//! All synchronization primitives are now available through the parent
//! concurrency module's re-exports:
//! - Mutex and MutexGuard in mutex.rs
//! - RwLock, ReadGuard, WriteGuard in rwlock.rs  
//! - SemaphoreSync, SemaphorePermit in semaphore.rs
//! - CondVar in condvar.rs
//! - Barrier, BarrierWaitResult in barrier.rs
//! - AtomicRef in atomic_ref.rs
//! - LockFreeQueue, BoundedLockFreeQueue in lockfree_queue.rs
//! - AtomicCounter, AtomicFlag in atomic_primitives.rs
//! - SyncRegistry, global_sync_registry in sync_registry.rs

// Re-export all synchronization primitives for backward compatibility
pub use super::{
    Mutex, MutexGuard,
    RwLock, ReadGuard, WriteGuard,
    SemaphoreSync, SemaphorePermit,
    CondVar,
    Barrier, BarrierWaitResult,
    AtomicRef,
    LockFreeQueue, BoundedLockFreeQueue,
    AtomicCounter, AtomicFlag,
    SyncRegistry, global_sync_registry
};
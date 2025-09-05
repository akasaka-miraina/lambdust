//! Lambdust continuation system with optimized memory management.
//!
//! This module implements a high-performance continuation system that provides:
//!
//! - **O(1) continuation capture**: Copy-on-Write strategies
//! - **Complete cycle resolution**: Weak reference systems
//! - **Memory safety**: Minimal unsafe usage
//! - **R7RS compliance**: Full call/cc support
//! - **Integration**: Phase 2 compatibility (SIMD, macros, types)
//!
//! ## Architecture
//!
//! The system is built around several key components:
//!
//! - `OptimizedContinuation`: Enum for different continuation storage strategies
//! - `ContinuationFrame`: Individual stack frame representation
//! - `ContinuationGC`: Specialized garbage collection for continuations
//! - `ContinuationEnvironment`: Integration with existing environment system
//!
//! ## Performance Characteristics
//!
//! - Continuation capture: O(1) amortized
//! - Memory overhead: Minimized through weak references
//! - Thread safety: Designed for Rc<RefCell<T>> patterns
//! - SIMD compatibility: Preserves Phase 2 optimizations

use crate::ast::Expr;
use crate::diagnostics::{Error, Result};
use crate::eval::value::Value;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// Core continuation components - simplified for compilation
pub mod call_cc;
pub mod environment;
pub mod frame;
pub mod gc;
pub mod optimization;

// Testing module
#[cfg(test)]
pub mod tests;

// Re-export key types
pub use call_cc::call_with_current_continuation;
pub use environment::ContinuationEnvironment;
pub use frame::{
    ContinuationChain, ContinuationFrame, ContinuationFrameRef, ContinuationFrameWeakRef,
    LegacyContinuationFrameRef, LegacyContinuationFrameRefExt, LegacyContinuationFrameWeakRef,
};
pub use gc::ContinuationGC;
pub use optimization::OptimizedContinuation;

/// Core continuation identifier for tracking and optimization.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct ContinuationId(u64);

impl std::fmt::Display for ContinuationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for ContinuationId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl ContinuationId {
    /// Generates a new unique continuation ID
    pub fn generate() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl From<ContinuationId> for u64 {
    fn from(id: ContinuationId) -> u64 {
        id.0
    }
}

/// Generation counter for continuation garbage collection.
pub type ContinuationGeneration = u64;

// Helper functions for IDs
impl Default for ContinuationId {
    fn default() -> Self {
        Self::new()
    }
}

impl ContinuationId {
    /// Generates a new continuation ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    /// Get the raw ID value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Continuation capture result for optimization analysis.
#[derive(Debug, Clone)]
pub enum CaptureResult {
    /// Immediate capture without allocation
    Immediate(OptimizedContinuation),
    /// Deferred capture for large stacks
    Deferred(ContinuationId),
    /// Cached capture from previous operation
    Cached(ContinuationId),
}

/// Statistics for continuation system performance monitoring.
#[derive(Debug, Default, Clone)]
pub struct ContinuationStats {
    /// Total continuations captured
    pub captures: u64,
    /// Total continuations invoked
    pub invocations: u64,
    /// Memory currently used by continuations
    pub memory_usage: usize,
    /// Number of GC cycles performed
    pub gc_cycles: u64,
    /// Average capture time in nanoseconds
    pub avg_capture_time: u64,
    /// Average invocation time in nanoseconds
    pub avg_invocation_time: u64,
}

/// Global continuation registry for tracking active continuations.
///
/// This registry provides centralized management of all active continuations
/// in the system, enabling efficient garbage collection and optimization.
pub struct ContinuationRegistry {
    /// Map of continuation ID to continuation
    continuations: RefCell<HashMap<ContinuationId, OptimizedContinuation>>,
    /// Next available continuation ID
    next_id: RefCell<ContinuationId>,
    /// Performance statistics
    stats: RefCell<ContinuationStats>,
    /// Garbage collector instance
    gc: RefCell<ContinuationGC>,
}

impl ContinuationRegistry {
    /// Creates a new continuation registry.
    pub fn new() -> Self {
        Self {
            continuations: RefCell::new(HashMap::new()),
            next_id: RefCell::new(ContinuationId::from(1)),
            stats: RefCell::new(ContinuationStats::default()),
            gc: RefCell::new(ContinuationGC::new()),
        }
    }

    /// Registers a new continuation and returns its ID.
    pub fn register(&self, continuation: OptimizedContinuation) -> ContinuationId {
        let id = {
            let mut next_id = self.next_id.borrow_mut();
            let id = *next_id;
            *next_id = ContinuationId::from(next_id.as_u64() + 1);
            id
        };

        self.continuations.borrow_mut().insert(id, continuation);

        // Update statistics
        let mut stats = self.stats.borrow_mut();
        stats.captures += 1;

        id
    }

    /// Retrieves a continuation by ID.
    pub fn get(&self, id: ContinuationId) -> Option<OptimizedContinuation> {
        self.continuations.borrow().get(&id).cloned()
    }

    /// Gets current performance statistics.
    pub fn stats(&self) -> ContinuationStats {
        self.stats.borrow().clone()
    }
}

impl Default for ContinuationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-local continuation registry for single-threaded optimization.
thread_local! {
    static CONTINUATION_REGISTRY: ContinuationRegistry = ContinuationRegistry::new();
}

/// Gets the thread-local continuation registry.
pub fn continuation_registry() -> &'static std::thread::LocalKey<ContinuationRegistry> {
    &CONTINUATION_REGISTRY
}

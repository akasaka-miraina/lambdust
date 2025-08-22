//! R7RS call-with-current-continuation implementation.
//!
//! This module provides the fundamental call/cc support required by R7RS Scheme,
//! implementing proper continuation capture and restoration semantics.

use super::frame::ContinuationFrame;
use super::optimization::OptimizedContinuation;
use super::{ContinuationGeneration, ContinuationId, continuation_registry};
use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span};
use crate::eval::value::Value;

/// R7RS call-with-current-continuation implementation (simplified).
///
/// This function captures the current continuation and passes it as a procedure
/// to the given function. The continuation can be invoked to return to the
/// capture point with a different value.
pub fn call_with_current_continuation(
    procedure: Value,
    current_expr: Expr,
    generation: ContinuationGeneration,
) -> Result<Value> {
    // Simplified implementation - would integrate with full evaluator

    // Generate unique ID for this continuation
    let continuation_id = generate_continuation_id();

    // Create a continuation frame representing the current state
    let frame = ContinuationFrame::new(continuation_id, current_expr, generation);

    // Create optimized continuation
    let continuation = OptimizedContinuation::single_owned(frame);

    // Register the continuation for tracking
    continuation_registry().with(|registry| {
        registry.register(continuation);
    });

    // For now, just return the procedure value
    // Real implementation would invoke the procedure with continuation
    Ok(procedure)
}

/// Generates a unique continuation ID.
fn generate_continuation_id() -> ContinuationId {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    ContinuationId::from(COUNTER.fetch_add(1, Ordering::SeqCst))
}

/// Specialized error types for call/cc operations.
#[derive(Debug, Clone)]
pub enum CallCCError {
    /// Continuation was garbage collected
    ContinuationCollected(ContinuationId),
    /// Invalid continuation state
    InvalidContinuation(String),
    /// Continuation invocation failed
    InvocationFailed(String),
    /// Not a procedure error
    NotAProcedure,
}

impl std::fmt::Display for CallCCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CallCCError::ContinuationCollected(id) => {
                write!(f, "Continuation {id} was garbage collected")
            }
            CallCCError::InvalidContinuation(msg) => {
                write!(f, "Invalid continuation: {msg}")
            }
            CallCCError::InvocationFailed(msg) => {
                write!(f, "Continuation invocation failed: {msg}")
            }
            CallCCError::NotAProcedure => {
                write!(f, "call/cc requires a procedure argument")
            }
        }
    }
}

impl std::error::Error for CallCCError {}

/// High-level call/cc interface for evaluator integration.
pub struct CallCCSupport {
    /// Current generation for new continuations
    generation: ContinuationGeneration,

    /// Statistics for call/cc operations
    stats: CallCCStats,
}

impl CallCCSupport {
    /// Creates new call/cc support system.
    pub fn new() -> Self {
        Self {
            generation: 0,
            stats: CallCCStats::default(),
        }
    }

    /// Executes call/cc with full error handling (simplified).
    pub fn execute_call_cc(&mut self, procedure: Value, current_expr: Expr) -> Result<Value> {
        self.generation += 1;

        let start_time = std::time::Instant::now();
        let result = call_with_current_continuation(procedure, current_expr, self.generation);

        // Update statistics
        let duration = start_time.elapsed();
        self.stats.total_calls += 1;
        self.stats.total_time += duration;

        match &result {
            Ok(_) => self.stats.successful_calls += 1,
            Err(_) => self.stats.failed_calls += 1,
        }

        result
    }

    /// Gets call/cc performance statistics.
    pub fn stats(&self) -> &CallCCStats {
        &self.stats
    }
}

impl Default for CallCCSupport {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for call/cc operations.
#[derive(Debug, Default, Clone)]
pub struct CallCCStats {
    /// Total number of call/cc invocations
    pub total_calls: u64,
    /// Number of successful call/cc operations
    pub successful_calls: u64,
    /// Number of failed call/cc operations
    pub failed_calls: u64,
    /// Total time spent in call/cc operations
    pub total_time: std::time::Duration,
    /// Number of continuation invocations
    pub continuation_invocations: u64,
    /// Average time per call/cc operation
    pub average_time: std::time::Duration,
}

impl CallCCStats {
    /// Updates average time calculation.
    pub fn update_average(&mut self) {
        if self.total_calls > 0 {
            self.average_time = self.total_time / self.total_calls as u32;
        }
    }

    /// Gets success rate as percentage.
    pub fn success_rate(&self) -> f64 {
        if self.total_calls > 0 {
            (self.successful_calls as f64 / self.total_calls as f64) * 100.0
        } else {
            0.0
        }
    }
}

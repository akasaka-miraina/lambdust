//! Optimized continuation storage strategies for maximum performance.
//!
//! This module implements the core OptimizedContinuation enum that provides
//! different storage strategies based on usage patterns and performance requirements.

use super::frame::{ContinuationFrame, ContinuationFrameRef, ContinuationFrameRefExt};
use super::{ContinuationGeneration, ContinuationId};
use crate::diagnostics::Result;
use crate::eval::value::Value;

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Optimized continuation storage with different strategies for different use cases.
///
/// This enum implements the core design from cs-architect, providing:
/// - O(1) continuation capture through Copy-on-Write
/// - Complete cycle resolution via weak references
/// - Memory safety with minimal unsafe usage
/// - Integration with existing Rc<RefCell<T>> patterns
#[derive(Debug, Clone)]
pub enum OptimizedContinuation {
    /// Single-owned continuation frame for simple cases.
    ///
    /// Used when continuation has unique ownership and no sharing.
    /// Provides fastest access with zero reference counting overhead.
    SingleOwned(Box<ContinuationFrame>),

    /// Shared continuation with adaptive references to support thread safety.
    ///
    /// Used when continuations need to be shared between multiple contexts.
    /// Adaptive references automatically handle local vs distributed execution.
    SharedAdaptive(ContinuationFrameRef),

    /// JIT-specialized continuation for hot paths.
    ///
    /// Used when JIT compiler has optimized continuation patterns.
    /// Provides specialized fast paths for common continuation usage.
    JitSpecialized(JitContinuation),

    /// Distributed continuation for remote execution.
    ///
    /// Used in distributed computing scenarios where continuations
    /// may need to be transferred between processes or machines.
    Distributed(DistributedContinuation),
}

impl OptimizedContinuation {
    /// Creates a new optimized continuation (defaults to shared adaptive).
    pub fn new(frame: ContinuationFrame) -> Self {
        Self::shared_adaptive(frame)
    }

    /// Creates a new single-owned continuation.
    pub fn single_owned(frame: ContinuationFrame) -> Self {
        Self::SingleOwned(Box::new(frame))
    }

    /// Creates a new shared adaptive continuation.
    pub fn shared_adaptive(frame: ContinuationFrame) -> Self {
        Self::SharedAdaptive(ContinuationFrameRef::new(frame))
    }

    /// Creates an optimized continuation from a frame reference.
    pub fn from_frame_ref(frame_ref: ContinuationFrameRef) -> Self {
        Self::SharedAdaptive(frame_ref)
    }

    /// Invokes the continuation with a value (simplified implementation).
    pub fn invoke(&self, value: Value) -> Result<Value> {
        // Simplified implementation - would integrate with evaluator
        Ok(value)
    }

    /// Gets the continuation ID.
    pub fn id(&self) -> ContinuationId {
        match self {
            Self::SingleOwned(frame) => frame.id,
            Self::SharedAdaptive(frame_ref) => frame_ref.id(),
            Self::JitSpecialized(jit) => jit.base_frame.id,
            Self::Distributed(dist) => dist.base_frame.id,
        }
    }

    /// Gets the continuation generation.
    pub fn generation(&self) -> ContinuationGeneration {
        match self {
            Self::SingleOwned(frame) => frame.generation,
            Self::SharedAdaptive(frame_ref) => match frame_ref {
                ContinuationFrameRef::Local(rc) => rc.borrow().generation,
                ContinuationFrameRef::Distributed(arc) => arc.read().unwrap().generation,
            },
            Self::JitSpecialized(jit) => jit.base_frame.generation,
            Self::Distributed(dist) => dist.base_frame.generation,
        }
    }

    /// Checks if this continuation can be safely shared.
    pub fn is_shareable(&self) -> bool {
        match self {
            Self::SingleOwned(_) => false,
            Self::SharedAdaptive(_) => true,
            Self::JitSpecialized(_) => true,
            Self::Distributed(_) => true,
        }
    }

    /// Converts to a shareable continuation if possible.
    pub fn make_shareable(self) -> Self {
        match self {
            Self::SingleOwned(frame) => Self::shared_adaptive(*frame),
            other => other,
        }
    }

    /// Checks if this continuation can be invoked.
    pub fn is_valid(&self) -> bool {
        match self {
            Self::SingleOwned(frame) => frame.is_valid(),
            Self::SharedAdaptive(frame_ref) => frame_ref.read().unwrap().is_valid(),
            Self::JitSpecialized(_) => true,
            Self::Distributed(_) => true,
        }
    }

    /// Checks if this continuation can be invoked safely.
    pub fn can_invoke(&self) -> bool {
        self.is_valid()
    }

    /// Estimates memory usage of this continuation.
    pub fn memory_usage(&self) -> usize {
        match self {
            Self::SingleOwned(frame) => frame.memory_usage(),
            Self::SharedAdaptive(_) => std::mem::size_of::<ContinuationFrameRef>(),
            Self::JitSpecialized(jit) => jit.memory_usage(),
            Self::Distributed(dist) => dist.memory_usage(),
        }
    }

    /// Optimizes the continuation based on usage patterns (simplified).
    pub fn optimize(self, _usage_stats: &UsageStats) -> Self {
        // Simplified - real implementation would analyze usage patterns
        self
    }
}

/// JIT-specialized continuation (simplified).
#[derive(Debug, Clone)]
pub struct JitContinuation {
    /// Base continuation frame
    pub base_frame: Box<ContinuationFrame>,

    /// JIT optimization metadata
    pub optimization_data: JitOptimizationData,
}

impl JitContinuation {
    /// Estimates memory usage of this JIT continuation.
    pub fn memory_usage(&self) -> usize {
        self.base_frame.memory_usage() + std::mem::size_of::<JitOptimizationData>()
    }
}

/// Distributed continuation (simplified).
#[derive(Debug, Clone)]
pub struct DistributedContinuation {
    /// Base continuation frame
    pub base_frame: Box<ContinuationFrame>,

    /// Node ID where this continuation should execute
    pub node_id: String,
}

impl DistributedContinuation {
    /// Estimates memory usage of this distributed continuation.
    pub fn memory_usage(&self) -> usize {
        self.base_frame.memory_usage() + self.node_id.len()
    }
}

/// JIT optimization metadata (simplified).
#[derive(Debug, Clone)]
pub struct JitOptimizationData {
    /// Hotness score (how frequently this continuation is used)
    pub hotness: u32,

    /// Type information for specialized compilation
    pub type_info: Vec<TypeHint>,

    /// Common value patterns
    pub value_patterns: HashMap<String, u32>,

    /// Inlining candidates
    pub inline_candidates: Vec<ContinuationId>,
}

/// Type hint for JIT optimization.
#[derive(Debug, Clone)]
pub struct TypeHint {
    /// Variable or position identifier
    pub identifier: String,

    /// Expected type
    pub expected_type: String,

    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
}

/// Usage statistics for continuation optimization decisions.
#[derive(Debug, Clone, Default)]
pub struct UsageStats {
    /// Number of times this continuation has been invoked
    pub invocation_count: u32,

    /// Number of times this continuation has been shared
    pub share_count: u32,

    /// Average execution time in nanoseconds
    pub avg_execution_time: u64,

    /// Memory pressure when this continuation is active
    pub memory_pressure: f64,

    /// Common argument types
    pub argument_types: HashMap<String, u32>,
}

impl UsageStats {
    /// Determines if JIT compilation should be used.
    pub fn should_use_jit(&self) -> bool {
        self.invocation_count > 100 && self.avg_execution_time > 1000
    }

    /// Determines if continuation should be shared.
    pub fn should_share(&self) -> bool {
        self.share_count > 3 || self.memory_pressure > 0.8
    }

    /// Converts usage stats to JIT optimization data.
    pub fn to_jit_data(&self) -> JitOptimizationData {
        JitOptimizationData {
            hotness: self.invocation_count,
            type_info: Vec::new(),
            value_patterns: HashMap::new(),
            inline_candidates: Vec::new(),
        }
    }
}

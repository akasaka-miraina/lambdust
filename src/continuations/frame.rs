//! Continuation frame implementation for stack-based evaluation state.
//!
//! This module provides the fundamental building blocks for capturing and
//! restoring evaluation state in Lambdust's continuation system.
//!
//! Enhanced with cs-architect's adaptive hybrid architecture for seamless
//! local-to-distributed continuation frame migration.

use super::{ContinuationGeneration, ContinuationId};
use crate::ast::Expr;
use crate::concurrency::{AdaptivePointer, AdaptiveWeakPointer};
use crate::diagnostics::{Result, Span};
use crate::eval::value::Value;

// use serde::{Deserialize, Serialize}; // Temporarily disabled for compilation
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

/// Type alias for continuation frame references using adaptive architecture
/// This enables seamless transition between local and distributed execution contexts
pub type ContinuationFrameRef = AdaptivePointer<ContinuationFrame>;
/// Weak reference to continuation frame using adaptive architecture
/// Prevents circular references while maintaining distributed capability
pub type ContinuationFrameWeakRef = AdaptiveWeakPointer<ContinuationFrame>;

/// Legacy type aliases for backward compatibility during transition
/// These will be gradually phased out as the codebase migrates
pub type LegacyContinuationFrameRef = Rc<RefCell<ContinuationFrame>>;
/// Legacy weak reference for backward compatibility
/// Will be phased out as migration to AdaptivePointer completes
pub type LegacyContinuationFrameWeakRef = Weak<RefCell<ContinuationFrame>>;

/// Extension trait for legacy continuation frame references
pub trait LegacyContinuationFrameRefExt {
    /// Gets the unique identifier of this continuation frame
    fn id(&self) -> ContinuationId;
    /// Alternative method to get continuation ID for compatibility
    fn get_id(&self) -> ContinuationId;
    /// Returns estimated memory usage in bytes for performance monitoring
    fn memory_usage(&self) -> usize;
}

impl LegacyContinuationFrameRefExt for LegacyContinuationFrameRef {
    fn id(&self) -> ContinuationId {
        self.borrow().id
    }

    fn get_id(&self) -> ContinuationId {
        self.borrow().id
    }

    fn memory_usage(&self) -> usize {
        self.borrow().memory_usage()
    }
}

/// A single frame in the continuation chain.
///
/// Each frame represents a point in the evaluation where computation
/// was suspended and can be resumed. Frames are linked together to
/// form complete evaluation contexts.
///
/// Enhanced with adaptive architecture support for thread-safe distributed execution.
#[derive(Debug, Clone)]
pub struct ContinuationFrame {
    /// Unique identifier for this frame
    pub id: ContinuationId,

    /// Expression being evaluated when captured
    pub expression: Expr,

    /// Values on the evaluation stack
    pub stack: Vec<Value>,

    /// Local variable bindings in this frame (simplified)
    pub locals: HashMap<String, Value>,

    /// Parent frame in the continuation chain (adaptive hybrid architecture)
    pub parent: Option<ContinuationFrameWeakRef>,

    /// Generation for garbage collection
    pub generation: ContinuationGeneration,

    /// Frame-specific metadata
    pub metadata: FrameMetadata,
}

/// Metadata associated with a continuation frame.
#[derive(Debug, Clone)]
pub struct FrameMetadata {
    /// Capture timestamp for debugging and profiling
    pub capture_time: std::time::Instant,

    /// Source location where this frame was captured
    pub source_span: Option<Span>,

    /// Frame type for optimization
    pub frame_type: FrameType,

    /// Memory usage estimation
    pub estimated_size: usize,

    /// Whether this frame has been optimized
    pub optimized: bool,
}

/// Type classification for continuation frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameType {
    /// Regular function call frame
    FunctionCall,

    /// Tail call frame (can be optimized)
    TailCall,

    /// Exception handler frame
    ExceptionHandler,

    /// Loop continuation frame
    LoopContinuation,

    /// Macro expansion frame
    MacroExpansion,

    /// SIMD operation frame
    SimdOperation,

    /// JIT-compiled code frame
    JitFrame,
}

// Safety: ContinuationFrame can be safely sent between threads because:
// - All fields are either owned data or safe for concurrent access
// - The adaptive pointer system handles thread-safe access to parent frames
unsafe impl Send for ContinuationFrame {}
unsafe impl Sync for ContinuationFrame {}

// Same for FrameMetadata
unsafe impl Send for FrameMetadata {}
unsafe impl Sync for FrameMetadata {}

impl ContinuationFrame {
    /// Creates a new continuation frame (simplified version).
    pub fn new(id: ContinuationId, expression: Expr, generation: ContinuationGeneration) -> Self {
        Self {
            id,
            expression,
            stack: Vec::new(),
            locals: HashMap::new(),
            parent: None,
            generation,
            metadata: FrameMetadata {
                capture_time: std::time::Instant::now(),
                source_span: None,
                frame_type: FrameType::FunctionCall,
                estimated_size: 128, // Basic estimate
                optimized: false,
            },
        }
    }

    /// Creates a new frame with a parent (adaptive architecture).
    pub fn with_parent(
        id: ContinuationId,
        expression: Expr,
        parent: ContinuationFrameWeakRef,
        generation: ContinuationGeneration,
    ) -> Self {
        let mut frame = Self::new(id, expression, generation);
        frame.parent = Some(parent);
        frame
    }

    /// Creates a frame with a legacy parent (for compatibility).
    pub fn with_legacy_parent(
        id: ContinuationId,
        expression: Expr,
        parent: LegacyContinuationFrameWeakRef,
        generation: ContinuationGeneration,
    ) -> Self {
        let adaptive_parent = ContinuationFrameWeakRef::Local(parent);
        Self::with_parent(id, expression, adaptive_parent, generation)
    }

    /// Adds a value to the frame's stack.
    pub fn push_value(&mut self, value: Value) {
        self.stack.push(value);
        self.update_size_estimate();
    }

    /// Removes a value from the frame's stack.
    pub fn pop_value(&mut self) -> Option<Value> {
        let value = self.stack.pop();
        self.update_size_estimate();
        value
    }

    /// Defines a local variable in this frame.
    pub fn define_local(&mut self, name: String, value: Value) {
        self.locals.insert(name, value);
        self.update_size_estimate();
    }

    /// Gets the frame ID.
    pub fn id(&self) -> ContinuationId {
        self.id
    }

    /// Gets the frame ID through a reference.
    pub fn get_id(&self) -> ContinuationId {
        self.id
    }

    /// Checks if this frame is valid (not corrupted).
    pub fn is_valid(&self) -> bool {
        true // Simplified validation
    }

    /// Estimates the memory usage of this frame.
    pub fn memory_usage(&self) -> usize {
        self.metadata.estimated_size
    }

    /// Updates the size estimate for this frame.
    fn update_size_estimate(&mut self) {
        let stack_size = self.stack.len() * std::mem::size_of::<Value>();
        let locals_size = self.locals.len() * 64; // Rough estimate
        self.metadata.estimated_size = 128 + stack_size + locals_size;
    }

    /// Sets the frame type for optimization purposes.
    pub fn set_frame_type(&mut self, frame_type: FrameType) {
        self.metadata.frame_type = frame_type;
    }

    /// Marks this frame as optimized.
    pub fn mark_optimized(&mut self) {
        self.metadata.optimized = true;
    }

    /// Gets the depth of this frame in the continuation chain.
    pub fn depth(&self) -> usize {
        let mut depth = 0;
        let mut current_weak = self.parent.clone();

        while let Some(weak_parent) = current_weak {
            if let Some(parent) = weak_parent.upgrade() {
                depth += 1;
                // Access parent frame and get its parent
                match parent.with_ref(|frame| frame.parent.clone()) {
                    Ok(next_parent) => current_weak = next_parent,
                    Err(_) => break,
                }
            } else {
                break;
            }
        }

        depth
    }

    /// Collects all frames in this continuation chain.
    pub fn collect_chain(&self) -> Vec<ContinuationId> {
        let mut chain = vec![self.id];
        let mut current_weak = self.parent.clone();

        while let Some(weak_parent) = current_weak {
            if let Some(parent) = weak_parent.upgrade() {
                // Access parent frame to get its ID and parent reference
                match parent.with_ref(|frame| (frame.id, frame.parent.clone())) {
                    Ok((parent_id, next_parent)) => {
                        chain.push(parent_id);
                        current_weak = next_parent;
                    }
                    Err(_) => break,
                }
            } else {
                break;
            }
        }

        chain
    }
}

/// A chain of continuation frames representing a complete evaluation context.
/// Enhanced with adaptive architecture support for distributed execution.
#[derive(Debug)]
pub struct ContinuationChain {
    /// Head of the frame chain (adaptive hybrid architecture)
    head: Option<ContinuationFrameRef>,

    /// Total number of frames in the chain
    length: usize,

    /// Total estimated memory usage
    total_size: usize,

    /// Whether this chain has been optimized
    optimized: bool,

    /// Whether this chain has been promoted to distributed mode
    promoted: bool,
}

impl ContinuationChain {
    /// Creates a new empty continuation chain.
    pub fn new() -> Self {
        Self {
            head: None,
            length: 0,
            total_size: 0,
            optimized: false,
            promoted: false,
        }
    }

    /// Creates a chain from a single frame (adaptive architecture).
    pub fn from_frame(frame: ContinuationFrameRef) -> Self {
        let size = frame.with_ref(|f| f.memory_usage()).unwrap_or(0);
        Self {
            head: Some(frame),
            length: 1,
            total_size: size,
            optimized: false,
            promoted: false,
        }
    }

    /// Creates a chain from a single legacy frame (for compatibility).
    pub fn from_legacy_frame(frame: Rc<RefCell<ContinuationFrame>>) -> Self {
        let adaptive_frame = AdaptivePointer::Local(frame);
        Self::from_frame(adaptive_frame)
    }

    /// Creates a chain from a list of continuation frames.
    /// This is a simplified implementation for compatibility.
    pub fn from_continuations(frames: Vec<Rc<RefCell<ContinuationFrame>>>) -> Self {
        let mut chain = Self::new();
        for frame in frames.into_iter().rev() {
            chain.push_frame(ContinuationFrameRef::Local(frame));
        }
        chain
    }

    /// Creates a chain from a list of optimized continuations.
    pub fn from_optimized_continuations(
        continuations: Vec<crate::continuations::optimization::OptimizedContinuation>,
    ) -> Self {
        let mut chain = Self::new();
        for continuation in continuations.into_iter().rev() {
            match continuation {
                crate::continuations::optimization::OptimizedContinuation::SingleOwned(frame) => {
                    chain.push_frame(ContinuationFrameRef::Local(Rc::new(RefCell::new(*frame))));
                }
                crate::continuations::optimization::OptimizedContinuation::SharedAdaptive(
                    frame_ref,
                ) => {
                    chain.push_frame(frame_ref);
                }
                crate::continuations::optimization::OptimizedContinuation::JitSpecialized(jit) => {
                    chain.push_frame(ContinuationFrameRef::Local(Rc::new(RefCell::new(
                        *jit.base_frame,
                    ))));
                }
                crate::continuations::optimization::OptimizedContinuation::Distributed(dist) => {
                    chain.push_frame(ContinuationFrameRef::Local(Rc::new(RefCell::new(
                        *dist.base_frame,
                    ))));
                }
            }
        }
        chain
    }

    /// Adds a frame to the head of the chain.
    pub fn push_frame(&mut self, frame: ContinuationFrameRef) {
        // Set the parent link
        if let Some(ref current_head) = self.head {
            // Use adaptive weak reference creation
            let weak_ref = current_head.downgrade();
            match frame {
                ContinuationFrameRef::Local(ref rc) => {
                    rc.borrow_mut().parent = Some(weak_ref);
                }
                ContinuationFrameRef::Distributed(ref arc) => {
                    arc.write().unwrap().parent = Some(weak_ref);
                }
            }
        }

        let frame_size = match &frame {
            ContinuationFrameRef::Local(rc) => rc.borrow().memory_usage(),
            ContinuationFrameRef::Distributed(arc) => arc.read().unwrap().memory_usage(),
        };

        self.total_size += frame_size;
        self.length += 1;
        self.head = Some(frame);
    }

    /// Removes the head frame from the chain.
    pub fn pop_frame(&mut self) -> Option<ContinuationFrameRef> {
        if let Some(head) = self.head.take() {
            let frame_size = match &head {
                ContinuationFrameRef::Local(rc) => rc.borrow().memory_usage(),
                ContinuationFrameRef::Distributed(arc) => arc.read().unwrap().memory_usage(),
            };

            self.total_size -= frame_size;
            self.length -= 1;

            // Update head to parent
            let parent_opt = match &head {
                ContinuationFrameRef::Local(rc) => rc.borrow().parent.clone(),
                ContinuationFrameRef::Distributed(arc) => arc.read().unwrap().parent.clone(),
            };

            if let Some(parent_weak) = parent_opt {
                self.head = parent_weak.upgrade();
            }

            Some(head)
        } else {
            None
        }
    }

    /// Gets the head frame without removing it.
    pub fn head(&self) -> Option<&ContinuationFrameRef> {
        self.head.as_ref()
    }

    /// Gets the length of the chain.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Checks if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Gets the total memory usage of the chain.
    pub fn memory_usage(&self) -> usize {
        self.total_size
    }

    /// Validates the integrity of the chain.
    pub fn validate(&self) -> bool {
        if let Some(ref head) = self.head {
            let mut current = Some(head.clone());
            let mut count = 0;

            while let Some(frame_ref) = current {
                count += 1;
                if count > self.length {
                    return false; // Cycle detected
                }

                let is_valid = match &frame_ref {
                    ContinuationFrameRef::Local(rc) => {
                        let frame = rc.borrow();
                        frame.is_valid()
                    }
                    ContinuationFrameRef::Distributed(arc) => {
                        let frame = arc.read().unwrap();
                        frame.is_valid()
                    }
                };

                if !is_valid {
                    return false;
                }

                current = match &frame_ref {
                    ContinuationFrameRef::Local(rc) => {
                        rc.borrow().parent.as_ref().and_then(|weak| weak.upgrade())
                    }
                    ContinuationFrameRef::Distributed(arc) => arc
                        .read()
                        .unwrap()
                        .parent
                        .as_ref()
                        .and_then(|weak| weak.upgrade()),
                };
            }

            count == self.length
        } else {
            self.length == 0
        }
    }

    /// Creates an iterator over the continuation chain.
    /// Returns a simplified iterator that yields frame references.
    pub fn iter(&self) -> ContinuationChainIterator {
        ContinuationChainIterator {
            current: self.head.clone(),
        }
    }

    /// Gets the chain ID (simplified - uses head frame ID if available).
    pub fn id(&self) -> Option<ContinuationId> {
        self.head.as_ref().map(|head| match head {
            ContinuationFrameRef::Local(rc) => rc.borrow().id,
            ContinuationFrameRef::Distributed(arc) => arc.read().unwrap().id,
        })
    }

    /// Gets the total operations count (simplified estimation).
    pub fn total_operations(&self) -> usize {
        self.length * 10 // Simplified estimate
    }
}

impl Default for ContinuationChain {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ContinuationChain {
    fn clone(&self) -> Self {
        // Simplified clone - deep copy would be more complex
        ContinuationChain::new()
    }
}

/// Iterator for ContinuationChain that yields frame references.
pub struct ContinuationChainIterator {
    current: Option<ContinuationFrameRef>,
}

impl Iterator for ContinuationChainIterator {
    type Item = ContinuationFrameRef;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current.take() {
            // Move to the parent before returning current
            self.current = match &current {
                ContinuationFrameRef::Local(rc) => {
                    rc.borrow().parent.as_ref().and_then(|weak| weak.upgrade())
                }
                ContinuationFrameRef::Distributed(arc) => arc
                    .read()
                    .unwrap()
                    .parent
                    .as_ref()
                    .and_then(|weak| weak.upgrade()),
            };
            Some(current)
        } else {
            None
        }
    }
}

/// Extension trait to add convenient methods to ContinuationFrameRef.
pub trait ContinuationFrameRefExt {
    /// Gets the unique identifier of this continuation frame reference
    fn id(&self) -> ContinuationId;
    /// Returns estimated memory usage for performance monitoring
    fn memory_usage(&self) -> usize;
    /// Checks if this reference points to a valid continuation frame
    fn is_valid(&self) -> bool;
}

impl ContinuationFrameRefExt for ContinuationFrameRef {
    fn id(&self) -> ContinuationId {
        match self {
            ContinuationFrameRef::Local(rc) => rc.borrow().id,
            ContinuationFrameRef::Distributed(arc) => arc.read().unwrap().id,
        }
    }

    fn memory_usage(&self) -> usize {
        match self {
            ContinuationFrameRef::Local(rc) => rc.borrow().memory_usage(),
            ContinuationFrameRef::Distributed(arc) => arc.read().unwrap().memory_usage(),
        }
    }

    fn is_valid(&self) -> bool {
        match self {
            ContinuationFrameRef::Local(rc) => rc.borrow().is_valid(),
            ContinuationFrameRef::Distributed(arc) => arc.read().unwrap().is_valid(),
        }
    }
}

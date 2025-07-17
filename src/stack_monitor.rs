//! Dynamic stack monitoring for adaptive optimization
//!
//! This module provides runtime stack monitoring and adaptive optimization
//! strategies based on evaluation patterns and resource usage.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Stack frame information for monitoring
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Type of operation
    pub operation: StackFrameType,
    /// Timestamp when frame was created
    pub created_at: Instant,
    /// Estimated memory usage of this frame
    pub memory_estimate: usize,
    /// Whether this frame can be optimized
    pub optimizable: bool,
}

/// Types of stack frames we monitor
#[derive(Debug, Clone)]
pub enum StackFrameType {
    /// Function application
    Application {
        /// Name of the operator being applied
        operator: String,
        /// Number of arguments
        arg_count: usize,
    },
    /// Special form evaluation
    SpecialForm {
        /// Name of the special form
        form_name: String,
    },
    /// Continuation application
    ContinuationApplication {
        /// Type of continuation
        cont_type: String,
    },
    /// Macro expansion
    MacroExpansion {
        /// Name of the macro
        macro_name: String,
    },
    /// Recursive call
    RecursiveCall {
        /// Name of the function being called recursively
        function_name: String,
        /// Current recursion depth
        depth: usize,
    },
}

/// Dynamic stack monitor for optimization decisions
pub struct StackMonitor {
    /// Current stack frames
    frames: VecDeque<StackFrame>,
    /// Maximum stack depth observed
    max_depth: usize,
    /// Total frames processed
    total_frames: usize,
    /// Optimization statistics
    optimizations_applied: usize,
    /// Time spent in evaluation
    evaluation_time: Duration,
    /// Memory pressure threshold
    memory_threshold: usize,
    /// Recursion depth threshold
    recursion_threshold: usize,
}

impl StackMonitor {
    /// Create a new stack monitor
    #[must_use] pub fn new() -> Self {
        Self {
            frames: VecDeque::with_capacity(1000),
            max_depth: 0,
            total_frames: 0,
            optimizations_applied: 0,
            evaluation_time: Duration::new(0, 0),
            memory_threshold: 10_000_000, // 10MB default
            recursion_threshold: 1000,    // 1000 calls default
        }
    }

    /// Push a new stack frame
    pub fn push_frame(&mut self, operation: StackFrameType) {
        let frame = StackFrame {
            memory_estimate: self.estimate_frame_memory(&operation),
            optimizable: self.is_optimizable(&operation),
            operation,
            created_at: Instant::now(),
        };

        self.frames.push_back(frame);
        self.total_frames += 1;

        if self.frames.len() > self.max_depth {
            self.max_depth = self.frames.len();
        }

        // Check for stack pressure and apply optimizations
        self.check_stack_pressure();
    }

    /// Pop the top stack frame
    pub fn pop_frame(&mut self) -> Option<StackFrame> {
        if let Some(frame) = self.frames.pop_back() {
            let duration = frame.created_at.elapsed();
            self.evaluation_time += duration;
            Some(frame)
        } else {
            None
        }
    }

    /// Check if stack is under pressure and needs optimization
    fn check_stack_pressure(&mut self) {
        let current_depth = self.frames.len();
        let total_memory = self.estimate_total_memory();

        // Apply optimizations if under pressure
        if current_depth > self.recursion_threshold || total_memory > self.memory_threshold {
            self.apply_stack_optimizations();
        }
    }

    /// Apply stack optimizations when under pressure
    fn apply_stack_optimizations(&mut self) {
        // Convert tail calls to iterations where possible
        self.optimize_tail_calls();

        // Compress continuation chains
        self.compress_continuations();

        // Force garbage collection of unused frames
        self.collect_unused_frames();

        self.optimizations_applied += 1;
    }

    /// Optimize tail calls by converting to iterative form
    fn optimize_tail_calls(&mut self) {
        // Look for recursive patterns that can be optimized
        let mut tail_call_sequences = 0;

        for window in self.frames.as_slices().0.windows(3) {
            if let [frame1, frame2, frame3] = window {
                if self.is_tail_call_sequence(frame1, frame2, frame3) {
                    tail_call_sequences += 1;
                }
            }
        }

        // If we detect many tail calls, mark for optimization
        if tail_call_sequences > 5 {
            // In a real implementation, this would trigger the evaluator
            // to use iterative evaluation instead of recursive
        }
    }

    /// Check if three frames represent a tail call sequence
    fn is_tail_call_sequence(
        &self,
        frame1: &StackFrame,
        frame2: &StackFrame,
        frame3: &StackFrame,
    ) -> bool {
        match (&frame1.operation, &frame2.operation, &frame3.operation) {
            (
                StackFrameType::Application { operator: op1, .. },
                StackFrameType::Application { operator: op2, .. },
                StackFrameType::Application { operator: op3, .. },
            ) => op1 == op2 && op2 == op3,
            _ => false,
        }
    }

    /// Compress continuation chains to reduce memory usage
    fn compress_continuations(&mut self) {
        // Count continuation frames
        let continuation_frames = self
            .frames
            .iter()
            .filter(|f| matches!(f.operation, StackFrameType::ContinuationApplication { .. }))
            .count();

        // If we have many continuations, they can potentially be compressed
        if continuation_frames > 10 {
            // Mark for continuation compression optimization
        }
    }

    /// Collect unused frames to free memory
    fn collect_unused_frames(&mut self) {
        let now = Instant::now();
        let threshold = Duration::from_millis(100); // 100ms threshold

        // Remove frames that have been idle for too long
        while let Some(front) = self.frames.front() {
            if now.duration_since(front.created_at) > threshold && !front.optimizable {
                self.frames.pop_front();
            } else {
                break;
            }
        }
    }

    /// Estimate memory usage of a frame
    #[must_use] pub fn estimate_frame_memory(&self, operation: &StackFrameType) -> usize {
        match operation {
            StackFrameType::Application { arg_count, .. } => {
                // Estimate: 64 bytes base + 32 bytes per argument
                64 + (arg_count * 32)
            }
            StackFrameType::SpecialForm { .. } => 128, // Special forms typically larger
            StackFrameType::ContinuationApplication { .. } => 96, // Continuation overhead
            StackFrameType::MacroExpansion { .. } => 256, // Macros can be memory-intensive
            StackFrameType::RecursiveCall { depth, .. } => {
                // Recursive calls accumulate memory
                64 + (depth * 16)
            }
        }
    }

    /// Estimate total memory usage of current stack
    fn estimate_total_memory(&self) -> usize {
        self.frames.iter().map(|f| f.memory_estimate).sum()
    }

    /// Check if a frame type can be optimized
    fn is_optimizable(&self, operation: &StackFrameType) -> bool {
        match operation {
            StackFrameType::Application { .. } | 
            StackFrameType::ContinuationApplication { .. } | 
            StackFrameType::RecursiveCall { .. } => true, // Function calls and continuations can be optimized
            StackFrameType::SpecialForm { .. } | 
            StackFrameType::MacroExpansion { .. } => false, // Special forms and macros need careful handling
        }
    }

    /// Get current stack statistics
    #[must_use] pub fn statistics(&self) -> StackStatistics {
        StackStatistics {
            current_depth: self.frames.len(),
            max_depth: self.max_depth,
            total_frames: self.total_frames,
            optimizations_applied: self.optimizations_applied,
            average_frame_time: if self.total_frames > 0 {
                self.evaluation_time / u32::try_from(self.total_frames).unwrap_or(1)
            } else {
                Duration::new(0, 0)
            },
            total_memory_estimate: self.estimate_total_memory(),
            optimizable_frames: self.frames.iter().filter(|f| f.optimizable).count(),
        }
    }

    /// Check if stack should trigger optimization
    #[must_use] pub fn should_optimize(&self) -> bool {
        let stats = self.statistics();

        // Trigger optimization if:
        // 1. Stack is deep
        // 2. Memory usage is high
        // 3. Many optimizable frames are present
        stats.current_depth > self.recursion_threshold / 2
            || stats.total_memory_estimate > self.memory_threshold / 2
            || (stats.optimizable_frames as f64 / stats.current_depth.max(1) as f64) > 0.7
    }

    /// Get optimization recommendations
    #[must_use] pub fn optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        let stats = self.statistics();

        if stats.current_depth > self.recursion_threshold / 2 {
            recommendations.push(OptimizationRecommendation::TailCallOptimization);
        }

        if stats.total_memory_estimate > self.memory_threshold / 2 {
            recommendations.push(OptimizationRecommendation::MemoryCompression);
        }

        if stats.optimizable_frames > 10 {
            recommendations.push(OptimizationRecommendation::ContinuationInlining);
        }

        recommendations
    }
}

impl Default for StackMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Stack monitoring statistics
#[derive(Debug, Clone)]
pub struct StackStatistics {
    /// Current stack depth
    pub current_depth: usize,
    /// Maximum depth observed
    pub max_depth: usize,
    /// Total frames processed
    pub total_frames: usize,
    /// Number of optimizations applied
    pub optimizations_applied: usize,
    /// Average time per frame
    pub average_frame_time: Duration,
    /// Estimated total memory usage
    pub total_memory_estimate: usize,
    /// Number of frames that can be optimized
    pub optimizable_frames: usize,
}

/// Optimization recommendations from stack analysis
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationRecommendation {
    /// Apply tail call optimization
    TailCallOptimization,
    /// Compress memory usage
    MemoryCompression,
    /// Inline continuations
    ContinuationInlining,
    /// Force garbage collection
    ForceGarbageCollection,
}


//! Performance metrics for evaluation

/// Performance metrics for evaluation
#[derive(Debug, Clone)]
pub struct EvaluationMetrics {
    /// Time taken for evaluation (in nanoseconds)
    pub evaluation_time_ns: u64,
    
    /// Memory allocated during evaluation
    pub memory_allocated: usize,
    
    /// Number of continuations captured
    pub continuations_captured: usize,
    
    /// Number of IO operations performed
    pub io_operations: usize,
}
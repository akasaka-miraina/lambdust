//! Statistics about virtual machine execution.

/// Statistics about virtual machine execution.
#[derive(Debug, Clone)]
pub struct VmStats {
    /// Number of instructions executed
    pub instructions_executed: usize,
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Number of function calls
    pub function_calls: usize,
    /// Maximum stack depth reached
    pub max_stack_depth: usize,
    /// Number of garbage collections triggered
    pub gc_count: usize,
    /// Number of optimized operations executed
    pub optimized_operations: usize,
}
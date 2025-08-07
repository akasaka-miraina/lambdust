//! Statistics about bytecode compilation.

/// Statistics about bytecode compilation.
#[derive(Debug, Clone)]
pub struct CompilerStats {
    /// Number of expressions compiled
    pub expressions_compiled: usize,
    /// Number of instructions generated
    pub instructions_generated: usize,
    /// Number of constants in pool
    pub constants_count: usize,
    /// Compilation time in microseconds
    pub compilation_time_us: u64,
    /// Memory usage during compilation
    pub memory_usage_bytes: usize,
}
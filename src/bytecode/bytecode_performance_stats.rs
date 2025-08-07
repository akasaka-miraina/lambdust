//! Comprehensive performance statistics for the bytecode system.

use super::{CompilerStats, VmStats, OptimizationStats, OverallPerformanceMetrics};

/// Comprehensive performance statistics for the bytecode system.
#[derive(Debug, Clone)]
pub struct BytecodePerformanceStats {
    /// Compiler statistics
    pub compiler: CompilerStats,
    /// Optimizer statistics
    pub optimizer: OptimizationStats,
    /// Virtual machine statistics
    pub vm: VmStats,
    /// Overall performance metrics
    pub overall: OverallPerformanceMetrics,
}
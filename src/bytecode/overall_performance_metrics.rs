//! Overall performance metrics combining all subsystems.

/// Overall performance metrics combining all subsystems.
#[derive(Debug, Clone)]
pub struct OverallPerformanceMetrics {
    /// Total time from compilation to execution completion
    pub total_time_us: u64,
    /// Instructions per second during execution
    pub instructions_per_second: f64,
    /// Memory efficiency score (0.0 to 1.0)
    pub memory_efficiency: f64,
    /// Optimization effectiveness score (0.0 to 1.0)
    pub optimization_effectiveness: f64,
    /// Comparison with interpreter performance
    pub speedup_factor: f64,
}
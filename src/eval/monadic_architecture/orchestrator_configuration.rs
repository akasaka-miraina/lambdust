//! Configuration for the evaluation orchestrator

/// Configuration for the evaluation orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfiguration {
    /// Maximum evaluation steps before timeout
    pub max_evaluation_steps: usize,
    
    /// Whether to enable tracing and debugging
    pub enable_tracing: bool,
    
    /// Whether to enable parallel evaluation
    pub enable_parallel_evaluation: bool,
    
    /// Timeout for individual computations
    pub computation_timeout_ms: u64,
}

impl Default for OrchestratorConfiguration {
    fn default() -> Self {
        Self {
            max_evaluation_steps: 10000,
            enable_tracing: false,
            enable_parallel_evaluation: false,
            computation_timeout_ms: 5000,
        }
    }
}
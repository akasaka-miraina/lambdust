/// Mock behavior for effect handler
#[derive(Debug, Clone)]
pub struct MockHandlerBehavior {
    /// Whether to simulate processing delays
    pub simulate_delays: bool,
    
    /// Base processing time
    pub base_processing_time_ms: u64,
    
    /// Whether to enable failure simulation
    pub enable_failures: bool,
    
    /// Failure rate (0.0 to 1.0)
    pub failure_rate: f64,
}

impl Default for MockHandlerBehavior {
    fn default() -> Self {
        Self {
            simulate_delays: false,
            base_processing_time_ms: 0,
            enable_failures: false,
            failure_rate: 0.0,
        }
    }
}
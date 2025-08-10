/// Mock behavior for effect interpreter
#[derive(Debug, Clone)]
pub struct MockEffectBehavior {
    /// Whether to fail on unknown effects
    pub fail_on_unknown: bool,
    
    /// Simulated processing time
    pub processing_time_ms: u64,
    
    /// Whether to enable async simulation
    pub simulate_async: bool,
}

impl Default for MockEffectBehavior {
    fn default() -> Self {
        Self {
            fail_on_unknown: false,
            processing_time_ms: 0,
            simulate_async: true,
        }
    }
}
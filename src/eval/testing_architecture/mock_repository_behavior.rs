/// Mock behavior configuration for repository
#[derive(Debug, Clone, Default)]
pub struct MockRepositoryBehavior {
    /// Whether store operations should fail
    pub store_should_fail: bool,
    
    /// Whether find operations should fail
    pub find_should_fail: bool,
    
    /// Simulated storage capacity
    pub max_capacity: Option<usize>,
    
    /// Simulated latency (in milliseconds)
    pub simulated_latency_ms: u64,
}
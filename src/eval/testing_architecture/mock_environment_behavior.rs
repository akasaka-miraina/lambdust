/// Mock behavior for environment manager
#[derive(Debug, Clone, Default)]
pub struct MockEnvironmentBehavior {
    /// Whether lookups should fail for specific variables
    pub failing_lookups: Vec<String>,
    
    /// Whether updates should fail
    pub updates_should_fail: bool,
    
    /// Maximum number of environments to track
    pub max_environments: Option<usize>,
}
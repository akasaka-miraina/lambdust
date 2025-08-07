/// Enhanced error recovery configuration.
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Maximum errors before stopping
    pub max_errors: usize,
    /// Enable aggressive recovery strategies
    pub aggressive_recovery: bool,
    /// Maximum nesting depth before error
    pub max_nesting_depth: usize,
    /// Number of recovery points to maintain
    pub recovery_point_limit: usize,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_errors: 10,
            aggressive_recovery: true,
            max_nesting_depth: 100,
            recovery_point_limit: 10,
        }
    }
}
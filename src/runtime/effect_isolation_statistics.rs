//! Statistics about effect isolation across the system.

use std::collections::HashMap;
use super::effect_isolation_level::EffectIsolationLevel;

/// Statistics about effect isolation.
#[derive(Debug, Clone)]
pub struct EffectIsolationStatistics {
    /// Total number of threads
    pub total_threads: usize,
    /// Number of isolated threads
    pub isolated_threads: usize,
    /// Count of threads by isolation level
    pub isolation_levels: HashMap<EffectIsolationLevel, usize>,
    /// Number of blocked cross-thread effects
    pub blocked_cross_thread_effects: usize,
    /// Number of active sandboxes
    pub sandbox_count: usize,
}
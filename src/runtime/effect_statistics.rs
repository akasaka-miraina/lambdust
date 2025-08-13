//! Statistics about effect usage across all threads.

use crate::effects::Effect;
use std::collections::HashMap;

/// Statistics about effect usage.
#[derive(Debug, Clone)]
pub struct EffectStatistics {
    /// Number of active threads
    pub active_threads: usize,
    /// Total number of active effects across all threads
    pub total_active_effects: usize,
    /// Count of each effect type currently active
    pub effect_counts: HashMap<Effect, usize>,
    /// Total number of effect events recorded
    pub total_events: usize,
    /// Number of events in the last minute
    pub recent_events: usize,
}
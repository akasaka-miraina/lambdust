//! Thread-specific effect state management.

use crate::effects::{Effect, EffectContext};
use std::time::SystemTime;

/// Effect state for a specific thread.
#[derive(Debug, Clone)]
pub struct ThreadEffectState {
    /// Current effect context for this thread
    pub context: EffectContext,
    /// Effects currently active in this thread
    pub active_effects: Vec<Effect>,
    /// Generation counter for this thread's effects
    pub generation: u64,
    /// Last update timestamp
    pub last_updated: SystemTime,
}
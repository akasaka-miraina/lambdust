//! Pending effect tracking for coordination.

use crate::effects::Effect;
use std::thread::ThreadId;
use std::time::SystemTime;

/// Pending effect waiting for coordination.
#[derive(Debug, Clone)]
pub struct PendingEffect {
    /// Effect sequence number
    pub sequence: u64,
    /// Thread that owns this effect
    pub thread_id: ThreadId,
    /// The effect itself
    pub effect: Effect,
    /// Dependencies that must complete first
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub dependencies: Vec<u64>,
    /// Timestamp when effect was submitted
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub submitted_at: SystemTime,
}
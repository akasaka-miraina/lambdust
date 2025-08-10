//! Messages for cross-thread effect coordination.

use crate::effects::Effect;

/// Messages for cross-thread effect coordination.
#[derive(Debug, Clone)]
pub enum EffectCoordinationMessage {
    /// Request to coordinate an effect
    CoordinateEffect {
        /// The effect to be coordinated
        #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
        effect: Effect,
        /// Unique sequence number for this coordination request
        #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
        sequence: u64,
        /// List of sequence numbers this effect depends on
        #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
        dependencies: Vec<u64>,
    },
    /// Response to effect coordination request
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    CoordinationResponse {
        /// Sequence number for request correlation
        sequence: u64,
        /// Whether the coordination was successful
        success: bool,
        /// Optional error message if coordination failed
        error: Option<String>,
    },
    /// Notification that an effect completed
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    EffectCompleted {
        /// Sequence number of the completed effect
        sequence: u64,
        /// Result of the effect execution
        result: Result<String, String>,
    },
    /// Request to abort an effect
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    AbortEffect {
        /// Sequence number of the effect to abort
        sequence: u64,
        /// Reason for aborting the effect
        reason: String,
    },
}
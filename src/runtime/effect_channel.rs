//! Cross-thread communication channel for effects.

use crossbeam::channel::{Sender, Receiver, unbounded};
use super::effect_coordination_message::EffectCoordinationMessage;

/// Cross-thread communication channel for effects.
#[derive(Debug)]
pub struct EffectChannel {
    /// Sender for effect coordination messages
    pub sender: Sender<EffectCoordinationMessage>,
    /// Receiver for effect coordination messages  
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub receiver: Receiver<EffectCoordinationMessage>,
}

impl EffectChannel {
    /// Creates a new effect channel.
    #[allow(dead_code)] // Part of Stage 3 effect coordination infrastructure
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}

impl Default for EffectChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EffectChannel {
    fn clone(&self) -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}
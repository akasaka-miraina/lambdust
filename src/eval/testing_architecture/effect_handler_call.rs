use crate::eval::Value;
use crate::effects::{Effect, EffectContext};

/// Effect handler call tracking
#[derive(Debug, Clone)]
pub struct EffectHandlerCall {
    /// The effect that was handled
    pub effect: Effect,
    /// Arguments passed to the handler
    pub args: Vec<Value>,
    /// Context in which the effect was handled
    pub context: EffectContext,
    /// When the handler was invoked
    pub timestamp: std::time::SystemTime,
}
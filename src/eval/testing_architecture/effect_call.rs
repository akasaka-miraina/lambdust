use crate::eval::Value;
use crate::effects::Effect;

/// Effect interpretation call tracking
#[derive(Debug, Clone)]
pub struct EffectCall {
    /// The effect that was interpreted
    pub effect: Effect,
    /// Arguments passed to the effect
    pub args: Vec<Value>,
    /// When the effect was called
    pub timestamp: std::time::SystemTime,
}
use super::{Effect, EffectHandler};
use std::sync::Arc;

/// Reference to an effect handler.
#[derive(Debug, Clone)]
pub struct EffectHandlerRef {
    /// Name of the effect this handler manages
    effect_name: String,
    /// The handler implementation
    handler: Arc<dyn EffectHandler + Send + Sync>,
}

impl PartialEq for EffectHandlerRef {
    fn eq(&self, other: &Self) -> bool {
        self.effect_name == other.effect_name
    }
}

impl Eq for EffectHandlerRef {}

impl EffectHandlerRef {
    /// Creates a new effect handler reference.
    pub fn new(effect_name: String, handler: Arc<dyn EffectHandler + Send + Sync>) -> Self {
        Self {
            effect_name,
            handler,
        }
    }
    
    /// Gets the effect name.
    pub fn effect_name(&self) -> &str {
        &self.effect_name
    }
    
    /// Gets a reference to the handler.
    pub fn handler(&self) -> &Arc<dyn EffectHandler + Send + Sync> {
        &self.handler
    }
}
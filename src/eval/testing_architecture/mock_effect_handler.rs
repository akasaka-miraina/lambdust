use crate::effects::Effect;
use super::{MockResponseStorage, EffectHandlerCall, MockHandlerBehavior};
use std::sync::{Arc, Mutex};

/// Mock effect handler for testing specific effects
#[derive(Debug)]
pub struct MockEffectHandler {
    /// Name of the effect this handler manages
    name: String,
    
    /// Effects this handler can process
    supported_effects: Vec<Effect>,
    
    /// Mock responses for different effect-argument combinations
    responses: MockResponseStorage,
    
    /// Call tracking
    call_log: Arc<Mutex<Vec<EffectHandlerCall>>>,
    
    /// Behavior configuration
    behavior: MockHandlerBehavior,
}
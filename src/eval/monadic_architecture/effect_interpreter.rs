//! Effect interpreter trait (interface for effect handling)

use crate::eval::Value;
use crate::diagnostics::Result;
use crate::effects::{Effect, EffectfulComputation};
use async_trait::async_trait;

/// Effect interpreter trait (interface for effect handling)
#[async_trait]
pub trait EffectInterpreter: std::fmt::Debug + Send + Sync {
    /// Interpret an effectful computation
    async fn interpret(&self, effect: EffectfulComputation) -> Result<Value>;
    
    /// Check if an effect can be interpreted
    fn can_interpret(&self, effect: &Effect) -> bool;
    
    /// Get available effect handlers
    fn available_effects(&self) -> Vec<Effect>;
}
use crate::eval::{Value, monadic_architecture::EffectInterpreter};
use crate::effects::{Effect, EffectfulComputation};
use crate::diagnostics::Result;
use super::{MockEffectResponse, MockEffectBehavior, EffectCall};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;

/// Mock effect interpreter for testing
#[derive(Debug, Default)]
pub struct MockEffectInterpreter {
    /// Mock responses for different effects
    responses: Arc<Mutex<HashMap<Effect, MockEffectResponse>>>,
    
    /// Call tracking
    call_log: Arc<Mutex<Vec<EffectCall>>>,
    
    /// Behavior configuration
    behavior: MockEffectBehavior,
}

impl MockEffectInterpreter {
    /// Create a new mock effect interpreter
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            call_log: Arc::new(Mutex::new(Vec::new())),
            behavior: MockEffectBehavior::default(),
        }
    }
    
    /// Add a mock response for a specific effect
    pub fn add_response(&self, effect: Effect, response: MockEffectResponse) {
        self.responses.lock().unwrap().insert(effect, response);
    }
    
    /// Get call log for assertions
    pub fn call_log(&self) -> Vec<EffectCall> {
        self.call_log.lock().unwrap().clone()
    }
    
    /// Clear call log
    pub fn clear_call_log(&self) {
        self.call_log.lock().unwrap().clear();
    }
}

#[async_trait]
impl EffectInterpreter for MockEffectInterpreter {
    async fn interpret(&self, effect: EffectfulComputation) -> Result<Value> {
        // Log the call (simplified)
        let effect_call = EffectCall {
            effect: Effect::IO, // Simplified - would extract from EffectfulComputation
            args: vec![],       // Simplified
            timestamp: std::time::SystemTime::now(),
        };
        
        self.call_log.lock().unwrap().push(effect_call);
        
        // Simulate processing time
        #[cfg(feature = "async-runtime")]
        if self.behavior.processing_time_ms > 0 {
            tokio::time::sleep(
                tokio::time::Duration::from_millis(self.behavior.processing_time_ms)
            ).await;
        }
        
        // Fallback when async-runtime is not available - use std::thread::sleep
        #[cfg(not(feature = "async-runtime"))]
        if self.behavior.processing_time_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.behavior.processing_time_ms));
        }
        
        // Return mock response or default
        Ok(Value::string("mock effect result".to_string()))
    }
    
    fn can_interpret(&self, effect: &Effect) -> bool {
        !self.behavior.fail_on_unknown ||
        self.responses.lock().unwrap().contains_key(effect)
    }
    
    fn available_effects(&self) -> Vec<Effect> {
        self.responses.lock().unwrap().keys().cloned().collect()
    }
}
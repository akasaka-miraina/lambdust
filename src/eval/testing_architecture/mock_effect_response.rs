use crate::eval::Value;
use super::MockMonadicComputation;

/// Mock response for effect interpretation
#[derive(Debug, Clone)]
pub enum MockEffectResponse {
    /// Return a specific value
    Value(Value),
    
    /// Return an error
    Error(String),
    
    /// Return a monadic computation
    Computation(MockMonadicComputation),
    
    /// Delay and then return a value
    DelayedValue(Value, u64), // value, delay in ms
}
//! Definition of a custom monad

use crate::eval::Value;
use std::sync::Arc;

use super::monadic_computation::MonadicComputation;

/// Type alias for complex monadic continuation functions
pub type MonadicContinuation = Arc<dyn Fn(MonadicComputation<Value>, Arc<dyn Fn(Value) -> MonadicComputation<Value> + Send + Sync>) -> MonadicComputation<Value> + Send + Sync>;

/// Type alias for complex monadic bind functions
pub type MonadicBindFunction = Option<Arc<dyn Fn(MonadicComputation<Value>, Arc<dyn Fn(Value) -> Value + Send + Sync>) -> MonadicComputation<Value> + Send + Sync>>;

/// Definition of a custom monad
#[derive(Clone)]
pub struct CustomMonadDefinition {
    /// Name of the monad
    pub name: String,
    
    /// Implementation of pure/return
    pub pure_impl: Arc<dyn Fn(Value) -> MonadicComputation<Value> + Send + Sync>,
    
    /// Implementation of bind/flatMap
    pub bind_impl: MonadicContinuation,
    
    /// Optional implementation of map/fmap
    pub map_impl: MonadicBindFunction,
}

impl std::fmt::Debug for CustomMonadDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomMonadDefinition {{ name: {:?}, pure_impl: <function>, bind_impl: <function>, map_impl: {} }}", 
               self.name, 
               if self.map_impl.is_some() { "<function>" } else { "None" })
    }
}
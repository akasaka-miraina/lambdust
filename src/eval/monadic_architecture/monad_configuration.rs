//! Configuration for monadic operations

use std::collections::HashMap;
use super::custom_monad_definition::CustomMonadDefinition;

/// Configuration for monadic operations
#[derive(Debug, Clone)]
pub struct MonadConfiguration {
    /// Maximum composition depth to prevent infinite recursion
    pub max_composition_depth: usize,
    
    /// Whether to optimize monadic compositions
    pub optimize_compositions: bool,
    
    /// Whether to enable automatic lifting
    pub enable_auto_lifting: bool,
    
    /// Custom monad definitions
    pub custom_monads: HashMap<String, CustomMonadDefinition>,
}

impl Default for MonadConfiguration {
    fn default() -> Self {
        Self {
            max_composition_depth: 1000,
            optimize_compositions: true,
            enable_auto_lifting: true,
            custom_monads: HashMap::new(),
        }
    }
}
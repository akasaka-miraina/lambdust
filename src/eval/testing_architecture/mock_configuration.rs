use super::{MockRepositoryBehavior, MockEffectBehavior, MockEnvironmentBehavior, MockHandlerBehavior};
use std::collections::HashMap;

/// Configuration for mock components
#[derive(Debug, Clone)]
pub enum MockConfiguration {
    /// Configuration for mock repository behavior
    Repository(MockRepositoryBehavior),
    /// Configuration for mock effect interpreter behavior
    EffectInterpreter(MockEffectBehavior),
    /// Configuration for mock environment manager behavior
    EnvironmentManager(MockEnvironmentBehavior),
    /// Configuration for mock effect handler behavior
    EffectHandler(MockHandlerBehavior),
    /// Custom configuration with key-value pairs
    Custom(HashMap<String, String>),
}
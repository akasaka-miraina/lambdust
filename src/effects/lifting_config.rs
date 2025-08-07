use super::lifting::LiftingRule;
use super::Effect;
use std::collections::HashMap;

/// Configuration for automatic effect lifting.
#[derive(Debug, Clone)]
pub struct LiftingConfig {
    /// Whether to automatically lift IO operations
    auto_lift_io: bool,
    /// Whether to automatically lift state operations
    auto_lift_state: bool,
    /// Whether to automatically lift error operations
    auto_lift_error: bool,
    /// Custom lifting rules
    custom_rules: HashMap<String, LiftingRule>,
}

impl LiftingConfig {
    /// Creates a new default lifting configuration.
    pub fn new() -> Self {
        Self {
            auto_lift_io: true,
            auto_lift_state: true,
            auto_lift_error: true,
            custom_rules: HashMap::new(),
        }
    }
    
    /// Disables all automatic lifting.
    pub fn no_lifting() -> Self {
        Self {
            auto_lift_io: false,
            auto_lift_state: false,
            auto_lift_error: false,
            custom_rules: HashMap::new(),
        }
    }
    
    /// Adds a custom lifting rule.
    pub fn add_rule(&mut self, operation: String, rule: LiftingRule) {
        self.custom_rules.insert(operation, rule);
    }
    
    /// Gets whether IO operations are automatically lifted.
    pub fn auto_lift_io(&self) -> bool {
        self.auto_lift_io
    }
    
    /// Gets whether state operations are automatically lifted.
    pub fn auto_lift_state(&self) -> bool {
        self.auto_lift_state
    }
    
    /// Gets whether error operations are automatically lifted.
    pub fn auto_lift_error(&self) -> bool {
        self.auto_lift_error
    }
    
    /// Gets the custom lifting rules.
    pub fn custom_rules(&self) -> &HashMap<String, LiftingRule> {
        &self.custom_rules
    }
    
    /// Checks custom rules for the given operation and effects.
    pub fn check_custom_rules(&self, _operation: &str, _current_effects: &[Effect]) -> Option<Effect> {
        // For now, return None - the actual implementation would need
        // to work with the private fields of LiftingRule through public methods
        // This is a placeholder to maintain the API
        None
    }
}

impl Default for LiftingConfig {
    fn default() -> Self {
        Self::new()
    }
}
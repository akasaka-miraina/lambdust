//! Effect isolation levels and rules.

use crate::effects::Effect;
use std::thread::ThreadId;

/// Levels of effect isolation between threads.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectIsolationLevel {
    /// Complete isolation - no effects can cross thread boundaries
    Complete,
    /// Only side-effect-free operations are allowed to cross boundaries
    SideEffectOnly,
    /// Only read operations are allowed, writes are isolated
    WriteOnly,
    /// Custom isolation rules
    Custom(EffectIsolationRules),
}

/// Custom rules for effect isolation.
#[derive(Debug, Clone)]
pub struct EffectIsolationRules {
    /// Allowed effect types
    pub allowed_effects: Vec<Effect>,
    /// Blocked effect types  
    pub blocked_effects: Vec<Effect>,
    /// Custom validation function
    pub custom_validator: Option<fn(&Effect, ThreadId, ThreadId) -> bool>,
    /// Exception rules
    pub exceptions: Vec<IsolationException>,
}

/// Exception to isolation rules.
#[derive(Debug, Clone)]
pub struct IsolationException {
    /// Effect this exception applies to
    pub effect: Effect,
    /// Threads this exception applies to
    pub threads: Vec<ThreadId>,
    /// Condition for when this exception applies
    pub condition: String,
}

impl Default for EffectIsolationRules {
    fn default() -> Self {
        Self {
            allowed_effects: vec![Effect::Pure],
            blocked_effects: vec![Effect::IO, Effect::State],
            custom_validator: None,
            exceptions: Vec::new(),
        }
    }
}

impl EffectIsolationRules {
    
    /// Checks if an effect is allowed based on these rules.
    pub fn allows_effect(&self, effect: &Effect, _source: ThreadId, _target: ThreadId) -> bool {
        // Check if explicitly blocked
        if self.blocked_effects.contains(effect) {
            return false;
        }
        
        // Check if explicitly allowed
        if self.allowed_effects.contains(effect) {
            return true;
        }
        
        // Use custom validator if available
        if let Some(validator) = self.custom_validator {
            return validator(effect, _source, _target);
        }
        
        // Default to blocking unknown effects
        false
    }
    
    /// Adds an exception rule.
    pub fn add_exception(&mut self, exception: IsolationException) {
        self.exceptions.push(exception);
    }
}

impl PartialEq for EffectIsolationRules {
    fn eq(&self, other: &Self) -> bool {
        self.allowed_effects == other.allowed_effects &&
        self.blocked_effects == other.blocked_effects &&
        self.exceptions == other.exceptions
        // Custom validator function pointers can't be compared
    }
}

impl Eq for EffectIsolationRules {}

impl std::hash::Hash for EffectIsolationRules {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.allowed_effects.hash(state);
        self.blocked_effects.hash(state);
        self.exceptions.hash(state);
        // Don't hash the function pointer
    }
}

impl PartialEq for IsolationException {
    fn eq(&self, other: &Self) -> bool {
        self.effect == other.effect &&
        self.threads == other.threads &&
        self.condition == other.condition
    }
}

impl Eq for IsolationException {}

impl std::hash::Hash for IsolationException {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.effect.hash(state);
        self.threads.hash(state);
        self.condition.hash(state);
    }
}
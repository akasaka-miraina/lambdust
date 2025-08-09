use super::{Effect, EffectHandlerRef};
use std::fmt;

/// Effect context that tracks the current computational effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectContext {
    /// Active effects in this context
    effects: Vec<Effect>,
    /// Effect handlers available in this context
    handlers: Vec<EffectHandlerRef>,
}

impl EffectContext {
    /// Creates a new empty effect context.
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            handlers: Vec::new(),
        }
    }
    
    /// Creates an effect context with pure computation.
    pub fn pure() -> Self {
        Self {
            effects: vec![Effect::Pure],
            handlers: Vec::new(),
        }
    }
    
    /// Adds an effect to this context.
    pub fn add_effect(&mut self, effect: Effect) {
        if !self.effects.contains(&effect) {
            self.effects.push(effect);
            self.effects.sort();
        }
    }
    
    /// Adds an effect handler to this context.
    pub fn add_handler(&mut self, handler: EffectHandlerRef) {
        self.handlers.push(handler);
    }
    
    /// Gets all effects in this context.
    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }
    
    /// Gets all handlers in this context.
    pub fn handlers(&self) -> &[EffectHandlerRef] {
        &self.handlers
    }
    
    /// Returns true if this context is pure.
    pub fn is_pure(&self) -> bool {
        self.effects.len() == 1 && self.effects[0] == Effect::Pure
    }
    
    /// Returns true if this context has the given effect.
    pub fn has_effect(&self, effect: &Effect) -> bool {
        self.effects.contains(effect)
    }
    
    /// Creates a new context with additional effects.
    pub fn with_effects(&self, effects: Vec<Effect>) -> Self {
        let mut new_context = self.clone();
        for effect in effects {
            new_context.add_effect(effect);
        }
        new_context
    }
    
    /// Creates a new context without the specified effects.
    pub fn without_effects(&self, effects: Vec<Effect>) -> Self {
        let mut new_effects = self.effects.clone();
        for effect in effects {
            new_effects.retain(|e| e != &effect);
        }
        Self {
            effects: new_effects,
            handlers: self.handlers.clone(),
        }
    }
    
    /// Combines this context with another.
    pub fn combine(&self, other: &EffectContext) -> EffectContext {
        let mut combined = self.clone();
        for effect in &other.effects {
            combined.add_effect(effect.clone());
        }
        for handler in &other.handlers {
            combined.add_handler(handler.clone());
        }
        combined
    }
    
    /// Finds a handler for the given effect.
    pub fn find_handler(&self, effect: &Effect) -> Option<&EffectHandlerRef> {
        self.handlers.iter().find(|h| h.handler().can_handle(effect))
    }
}

impl Default for EffectContext {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EffectContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_pure() {
            write!(f, "Pure")
        } else {
            write!(f, "[")?;
            for (i, effect) in self.effects.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{effect}")?;
            }
            write!(f, "]")
        }
    }
}
//! Extensions to EffectContext for isolation support.

use crate::effects::{Effect, EffectContext};
use super::effect_isolation_level::EffectIsolationLevel;

impl Effect {
    /// Returns true if this effect has side effects.
    pub fn has_side_effects(&self) -> bool {
        match self {
            Effect::Pure => false,
            Effect::IO | Effect::State | Effect::Error => true,
            Effect::Custom(name) => !name.starts_with("pure_"),
        }
    }
    
    /// Returns true if this effect performs write operations.
    pub fn is_write_effect(&self) -> bool {
        match self {
            Effect::Pure => false,
            Effect::IO | Effect::State => true,
            Effect::Error => false,
            Effect::Custom(name) => name.contains("write") || name.contains("set"),
        }
    }
}

impl EffectContext {
    /// Adds isolation to this context.
    pub fn with_isolation(&self, level: EffectIsolationLevel) -> Self {
        let mut new_context = self.clone();
        new_context.add_effect(Effect::Custom(format!("isolation:{level:?}")));
        new_context
    }
    
    /// Removes isolation from this context.
    pub fn without_isolation(&self) -> Self {
        let mut new_effects = self.effects().to_vec();
        new_effects.retain(|e| {
            if let Effect::Custom(name) = e {
                !name.starts_with("isolation:")
            } else {
                true
            }
        });
        
        let mut new_context = EffectContext::new();
        for effect in new_effects {
            new_context.add_effect(effect);
        }
        for handler in self.handlers() {
            new_context.add_handler(handler.clone());
        }
        new_context
    }
    
    /// Returns true if this context is isolated.
    pub fn is_isolated(&self) -> bool {
        self.effects().iter().any(|e| {
            if let Effect::Custom(name) = e {
                name.starts_with("isolation:")
            } else {
                false
            }
        })
    }
    
    /// Gets the isolation level if any.
    pub fn get_isolation_level(&self) -> Option<EffectIsolationLevel> {
        for effect in self.effects() {
            if let Effect::Custom(name) = effect {
                if name.starts_with("isolation:") {
                    // Parse isolation level from effect name
                    if name.contains("Complete") {
                        return Some(EffectIsolationLevel::Complete);
                    } else if name.contains("SideEffectOnly") {
                        return Some(EffectIsolationLevel::SideEffectOnly);
                    } else if name.contains("WriteOnly") {
                        return Some(EffectIsolationLevel::WriteOnly);
                    }
                }
            }
        }
        None
    }
}
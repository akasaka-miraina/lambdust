use super::{EffectContext, Effect, LiftingConfig, EffectResult, GenerationalEnvManager};
use crate::diagnostics::Result;
use crate::eval::Value;

/// The main effect system manager.
///
/// This coordinates effect tracking, automatic lifting, and
/// integration with the evaluator and type system.
#[derive(Debug, Clone)]
pub struct EffectSystem {
    /// Current effect context
    context: EffectContext,
    /// Generational environment manager
    env_manager: GenerationalEnvManager,
    /// Effect lifting configuration
    lifting_config: LiftingConfig,
}

impl EffectSystem {
    /// Creates a new effect system.
    pub fn new() -> Self {
        Self {
            context: EffectContext::pure(),
            env_manager: GenerationalEnvManager::new(),
            lifting_config: LiftingConfig::default(),
        }
    }
    
    /// Creates an effect system with the given configuration.
    pub fn with_config(config: LiftingConfig) -> Self {
        Self {
            context: EffectContext::pure(),
            env_manager: GenerationalEnvManager::new(),
            lifting_config: config,
        }
    }
    
    /// Gets the current effect context.
    pub fn context(&self) -> &EffectContext {
        &self.context
    }
    
    /// Gets a mutable reference to the effect context.
    pub fn context_mut(&mut self) -> &mut EffectContext {
        &mut self.context
    }
    
    /// Gets the generational environment manager.
    pub fn env_manager(&self) -> &GenerationalEnvManager {
        &self.env_manager
    }
    
    /// Gets a mutable reference to the generational environment manager.
    pub fn env_manager_mut(&mut self) -> &mut GenerationalEnvManager {
        &mut self.env_manager
    }
    
    /// Gets the lifting configuration.
    pub fn lifting_config(&self) -> &LiftingConfig {
        &self.lifting_config
    }
    
    /// Enters a new effect context.
    pub fn enter_context(&mut self, effects: Vec<Effect>) -> EffectContext {
        let old_context = self.context.clone();
        for effect in effects {
            self.context.add_effect(effect);
        }
        old_context
    }
    
    /// Exits an effect context, restoring the previous one.
    pub fn exit_context(&mut self, old_context: EffectContext) {
        self.context = old_context;
    }
    
    /// Handles an effect with the current context.
    pub fn handle_effect(&self, effect: &Effect, args: &[Value]) -> Result<EffectResult> {
        if let Some(handler_ref) = self.context.find_handler(effect) {
            handler_ref.handler().handle(effect, args)
        } else {
            Ok(EffectResult::Unhandled)
        }
    }
    
    /// Determines if an operation should be automatically lifted.
    pub fn should_lift(&self, operation: &str, current_effects: &[Effect]) -> Option<Effect> {
        // Check custom rules first
        if let Some(effect) = self.lifting_config.check_custom_rules(operation, current_effects) {
            return Some(effect);
        }
        
        // Check built-in lifting rules
        match operation {
            "display" | "write" | "newline" | "read" | "open-input-file" | "open-output-file" 
                if self.lifting_config.auto_lift_io() => Some(Effect::IO),
            "set!" | "vector-set!" | "string-set!" 
                if self.lifting_config.auto_lift_state() => Some(Effect::State),
            "error" | "raise" | "throw" 
                if self.lifting_config.auto_lift_error() => Some(Effect::Error),
            _ => None,
        }
    }
}

impl Default for EffectSystem {
    fn default() -> Self {
        Self::new()
    }
}
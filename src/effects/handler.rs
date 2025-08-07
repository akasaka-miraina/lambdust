//! Effect handlers for managing computational effects.
//!
//! This module provides the effect handler system that allows custom
//! management of effects through user-defined handlers. It supports
//! the `with-handler` and `define-effect-handler` constructs from the
//! language specification.

use super::{Effect, EffectResult, IOAction, StateAction, ErrorAction, EffectHandler};
use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{ThreadSafeEnvironment, Value, Procedure};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

/// A complete effect handler that can manage multiple types of effects.
#[derive(Debug, Clone)]
pub struct EffectHandlerImplementation {
    /// Name of this handler
    name: String,
    /// Effects this handler can manage
    handled_effects: Vec<Effect>,
    /// Handler functions for each effect
    handlers: HashMap<Effect, HandlerFunction>,
    /// Default handler for unhandled effects
    default_handler: Option<HandlerFunction>,
    /// Environment for handler execution (thread-safe)
    #[allow(dead_code)]
    environment: Arc<ThreadSafeEnvironment>,
}

/// A function that handles a specific effect - Thread-safe.
#[derive(Debug, Clone)]
pub struct HandlerFunction {
    /// The procedure that implements the handler (thread-safe)
    procedure: Arc<Procedure>,
    /// Expected arity for this handler
    arity: HandlerArity,
    /// Whether this handler is resumable
    resumable: bool,
}

/// Arity specification for effect handlers.
#[derive(Debug, Clone)]
pub enum HandlerArity {
    /// Fixed number of arguments
    Fixed(usize),
    /// Variable number of arguments (minimum required)
    Variable(usize),
    /// Exact number of arguments with continuation
    WithContinuation(usize),
}

/// Registry of available effect handlers.
#[derive(Debug, Clone)]
pub struct EffectHandlerRegistry {
    /// Registered handlers by name
    handlers: HashMap<String, EffectHandlerImplementation>,
    /// Default handlers for built-in effects
    default_handlers: HashMap<Effect, EffectHandlerImplementation>,
    /// Stack of active handler contexts
    handler_stack: Vec<EffectHandlerImplementation>,
}

/// Built-in effect handlers for core effects.
pub struct BuiltinHandlers;

/// IO effect handler that manages input/output operations.
#[derive(Debug, Clone)]
pub struct IOEffectHandler {
    /// Name of this handler
    #[allow(dead_code)]
    name: String,
    /// Configuration for IO operations
    config: IOHandlerConfig,
}

/// Configuration for IO effect handling.
#[derive(Debug, Clone, Default)]
pub struct IOHandlerConfig {
    /// Whether to buffer output
    buffer_output: bool,
    /// Whether to echo input
    #[allow(dead_code)]
    echo_input: bool,
    /// Custom input/output redirections
    #[allow(dead_code)]
    redirections: HashMap<String, IORedirection>,
}

/// Redirection for IO operations.
#[derive(Debug, Clone)]
pub enum IORedirection {
    /// Redirect to a file
    File(String),
    /// Redirect to a string buffer (thread-safe)
    String(Arc<RwLock<String>>),
    /// Redirect to a custom procedure (thread-safe)
    Procedure(Arc<Procedure>),
    /// Null redirection (discard)
    Null,
}

/// State effect handler that manages mutable state.
#[derive(Debug, Clone)]
pub struct StateEffectHandler {
    /// Name of this handler
    #[allow(dead_code)]
    name: String,
    /// Configuration for state management
    #[allow(dead_code)]
    config: StateHandlerConfig,
}

/// Configuration for state effect handling.
#[derive(Debug, Clone)]
pub struct StateHandlerConfig {
    /// Whether to track state changes for undo
    #[allow(dead_code)]
    track_changes: bool,
    /// Maximum number of state snapshots to keep
    #[allow(dead_code)]
    max_snapshots: usize,
    /// Whether to validate state consistency
    #[allow(dead_code)]
    validate_consistency: bool,
}

/// Error effect handler that manages exceptions and error recovery.
#[derive(Debug, Clone)]
pub struct ErrorEffectHandler {
    /// Name of this handler
    #[allow(dead_code)]
    name: String,
    /// Configuration for error handling
    config: ErrorHandlerConfig,
}

/// Configuration for error effect handling.
#[derive(Debug, Clone)]
pub struct ErrorHandlerConfig {
    /// Whether to capture stack traces
    #[allow(dead_code)]
    capture_stack_trace: bool,
    /// Whether to allow error recovery
    allow_recovery: bool,
    /// Custom error transformations (thread-safe)
    #[allow(dead_code)]
    error_transformations: HashMap<String, Arc<Procedure>>,
}

impl super::EffectHandler for EffectHandlerImplementation {
    fn handle(&self, effect: &Effect, args: &[Value]) -> Result<EffectResult> {
        if let Some(handler_func) = self.handlers.get(effect) {
            self.execute_handler(handler_func, effect, args)
        } else if let Some(default_handler) = &self.default_handler {
            self.execute_handler(default_handler, effect, args)
        } else {
            Ok(EffectResult::Unhandled)
        }
    }
    
    fn effect_name(&self) -> &str {
        &self.name
    }
    
    fn can_handle(&self, effect: &Effect) -> bool {
        self.handled_effects.contains(effect) || self.default_handler.is_some()
    }
}

impl EffectHandlerImplementation {
    /// Creates a new effect handler.
    pub fn new(
        name: String, 
        handled_effects: Vec<Effect>,
        environment: Arc<ThreadSafeEnvironment>
    ) -> Self {
        Self {
            name,
            handled_effects,
            handlers: HashMap::new(),
            default_handler: None,
            environment,
        }
    }
    
    /// Adds a handler function for a specific effect.
    pub fn add_handler(&mut self, effect: Effect, handler: HandlerFunction) {
        self.handlers.insert(effect.clone()), handler);
        if !self.handled_effects.contains(&effect) {
            self.handled_effects.push(effect);
        }
    }
    
    /// Sets the default handler for unhandled effects.
    pub fn set_default_handler(&mut self, handler: HandlerFunction) {
        self.default_handler = Some(handler);
    }
    
    /// Executes a handler function.
    fn execute_handler(
        &self, 
        handler: &HandlerFunction, 
        effect: &Effect, 
        args: &[Value]
    ) -> Result<EffectResult> {
        // Check arity
        if let Err(e) = self.check_handler_arity(&handler.arity, args.len()) {
            return Ok(EffectResult::Error(e));
        }
        
        // Prepare arguments for the handler procedure
        let mut handler_args = vec![
            // Effect as the first argument
            Value::string(format!("{effect}")),
        ];
        handler_args.extend_from_slice(args);
        
        // TODO: Execute the handler procedure
        // For now, return a placeholder result
        match effect {
            Effect::Pure => Ok(EffectResult::Value(Value::Unspecified)),
            Effect::IO => {
                // Handle IO effect
                if let Some(value) = args.first() {
                    // Simple print operation
                    print!("{value}");
                    Ok(EffectResult::Value(Value::Unspecified))
                } else {
                    Ok(EffectResult::Value(Value::Unspecified))
                }
            },
            Effect::State => {
                // Handle state effect
                Ok(EffectResult::Value(Value::Unspecified))
            },
            Effect::Error => {
                // Handle error effect
                if let Some(error_val) = args.first() {
                    let error_msg = format!("Error: {error_val}");
                    Ok(EffectResult::Error(DiagnosticError::runtime_error(error_msg, None)))
                } else {
                    Ok(EffectResult::Value(Value::Unspecified))
                }
            },
            Effect::Custom(name) => {
                // Handle custom effect
                Ok(EffectResult::Value(Value::string(format!("Custom effect: {name}"))))
            }
        }
    }
    
    /// Checks if the handler arity matches the number of arguments.
    fn check_handler_arity(&self, arity: &HandlerArity, arg_count: usize) -> Result<()> {
        match arity {
            HandlerArity::Fixed(expected) => {
                if arg_count != *expected {
                    Err(DiagnosticError::runtime_error(
                        format!("Handler expects {expected} arguments, got {arg_count}"),
                        None,
                    ))
                } else {
                    Ok(())
                }
            },
            HandlerArity::Variable(min) => {
                if arg_count < *min {
                    Err(DiagnosticError::runtime_error(
                        format!("Handler expects at least {min} arguments, got {arg_count}"),
                        None,
                    ))
                } else {
                    Ok(())
                }
            },
            HandlerArity::WithContinuation(expected) => {
                // Continuation adds one extra argument
                if arg_count != expected + 1 {
                    Err(DiagnosticError::runtime_error(
                        format!("Handler with continuation expects {} arguments, got {}", 
                                expected + 1, arg_count),
                        None,
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl HandlerFunction {
    /// Creates a new handler function.
    pub fn new(procedure: Arc<Procedure>, arity: HandlerArity) -> Self {
        Self {
            procedure,
            arity,
            resumable: false,
        }
    }
    
    /// Creates a resumable handler function.
    pub fn resumable(procedure: Arc<Procedure>, arity: HandlerArity) -> Self {
        Self {
            procedure,
            arity,
            resumable: true,
        }
    }
    
    /// Returns true if this handler is resumable.
    pub fn is_resumable(&self) -> bool {
        self.resumable
    }
    
    /// Gets the handler procedure.
    pub fn procedure(&self) -> &Arc<Procedure> {
        &self.procedure
    }
    
    /// Gets the handler arity.
    pub fn arity(&self) -> &HandlerArity {
        &self.arity
    }
}

impl EffectHandlerRegistry {
    /// Creates a new effect handler registry.
    pub fn new() -> Self {
        let mut registry = Self {
            handlers: HashMap::new(),
            default_handlers: HashMap::new(),
            handler_stack: Vec::new(),
        };
        
        // Register built-in handlers
        registry.register_builtin_handlers();
        registry
    }
    
    /// Registers a named effect handler.
    pub fn register_handler(&mut self, name: String, handler: EffectHandlerImplementation) {
        self.handlers.insert(name, handler);
    }
    
    /// Sets a default handler for an effect type.
    pub fn set_default_handler(&mut self, effect: Effect, handler: EffectHandlerImplementation) {
        self.default_handlers.insert(effect, handler);
    }
    
    /// Finds a handler for the given effect.
    pub fn find_handler(&self, effect: &Effect) -> Option<&EffectHandlerImplementation> {
        // First check the handler stack (most recent first)
        for handler in self.handler_stack.iter().rev() {
            if handler.can_handle(effect) {
                return Some(handler);
            }
        }
        
        // Then check default handlers
        self.default_handlers.get(effect)
    }
    
    /// Pushes a handler onto the stack.
    pub fn push_handler(&mut self, handler: EffectHandlerImplementation) {
        self.handler_stack.push(handler);
    }
    
    /// Pops a handler from the stack.
    pub fn pop_handler(&mut self) -> Option<EffectHandlerImplementation> {
        self.handler_stack.pop()
    }
    
    /// Gets a named handler.
    pub fn get_handler(&self, name: &str) -> Option<&EffectHandlerImplementation> {
        self.handlers.get(name)
    }
    
    /// Lists all registered handler names.
    pub fn handler_names(&self) -> Vec<&String> {
        self.handlers.keys().collect()
    }
    
    /// Registers built-in effect handlers.
    fn register_builtin_handlers(&mut self) {
        // IO handler
        let io_handler = BuiltinHandlers::create_io_handler();
        self.set_default_handler(Effect::IO, io_handler);
        
        // State handler
        let state_handler = BuiltinHandlers::create_state_handler();
        self.set_default_handler(Effect::State, state_handler);
        
        // Error handler
        let error_handler = BuiltinHandlers::create_error_handler();
        self.set_default_handler(Effect::Error, error_handler);
    }
}

impl BuiltinHandlers {
    /// Creates the default IO effect handler.
    pub fn create_io_handler() -> EffectHandlerImplementation {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let mut handler = EffectHandlerImplementation::new(
            "builtin-io".to_string(),
            vec![Effect::IO],
            env,
        );
        
        // Create a simple handler procedure (placeholder)
        // In a full implementation, this would be a proper Scheme procedure
        let proc = Arc::new(Procedure {
            formals: crate::ast::Formals::Variable("args".to_string()),
            body: vec![], // Empty body for now
            environment: Arc::new(ThreadSafeEnvironment::new(None, 0)),
            name: Some("io-handler".to_string()),
            metadata: HashMap::new(),
            source: None,
        });
        
        let handler_func = HandlerFunction::new(
            proc,
            HandlerArity::Variable(1),
        );
        
        handler.add_handler(Effect::IO, handler_func);
        handler
    }
    
    /// Creates the default State effect handler.
    pub fn create_state_handler() -> EffectHandlerImplementation {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let mut handler = EffectHandlerImplementation::new(
            "builtin-state".to_string(),
            vec![Effect::State],
            env,
        );
        
        let proc = Arc::new(Procedure {
            formals: crate::ast::Formals::Variable("args".to_string()),
            body: vec![],
            environment: Arc::new(ThreadSafeEnvironment::new(None, 0)),
            name: Some("state-handler".to_string()),
            metadata: HashMap::new(),
            source: None,
        });
        
        let handler_func = HandlerFunction::new(
            proc,
            HandlerArity::Variable(1),
        );
        
        handler.add_handler(Effect::State, handler_func);
        handler
    }
    
    /// Creates the default Error effect handler.
    pub fn create_error_handler() -> EffectHandlerImplementation {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let mut handler = EffectHandlerImplementation::new(
            "builtin-error".to_string(),
            vec![Effect::Error],
            env,
        );
        
        let proc = Arc::new(Procedure {
            formals: crate::ast::Formals::Variable("args".to_string()),
            body: vec![],
            environment: Arc::new(ThreadSafeEnvironment::new(None, 0)),
            name: Some("error-handler".to_string()),
            metadata: HashMap::new(),
            source: None,
        });
        
        let handler_func = HandlerFunction::new(
            proc,
            HandlerArity::Variable(1),
        );
        
        handler.add_handler(Effect::Error, handler_func);
        handler
    }
}

impl IOEffectHandler {
    /// Creates a new IO effect handler.
    pub fn new(name: String) -> Self {
        Self {
            name,
            config: IOHandlerConfig::default(),
        }
    }
    
    /// Creates an IO handler with custom configuration.
    pub fn with_config(name: String, config: IOHandlerConfig) -> Self {
        Self {
            name,
            config,
        }
    }
    
    /// Handles an IO action.
    pub fn handle_io_action(&self, action: &IOAction) -> Result<Value> {
        match action {
            IOAction::Print(value) => {
                if self.config.buffer_output {
                    // Buffer the output (simplified)
                    Ok(Value::Unspecified)
                } else {
                    print!("{value}");
                    Ok(Value::Unspecified)
                }
            },
            IOAction::Newline => {
                if !self.config.buffer_output {
                    println!();
                }
                Ok(Value::Unspecified)
            },
            IOAction::Return(value) => Ok(value.clone()),
            _ => {
                Err(DiagnosticError::runtime_error(
                    "IO action not yet implemented".to_string(),
                    None,
                ))
            }
        }
    }
}

impl StateEffectHandler {
    /// Creates a new state effect handler.
    pub fn new(name: String) -> Self {
        Self {
            name,
            config: StateHandlerConfig::default(),
        }
    }
    
    /// Handles a state action.
    pub fn handle_state_action(&self, action: &StateAction) -> Result<Value> {
        match action {
            StateAction::Return(value) => Ok(value.clone()),
            StateAction::GetVar(name) => {
                // In a full implementation, this would access the state
                Err(DiagnosticError::runtime_error(
                    format!("Variable {name} not found in state"),
                    None,
                ))
            },
            _ => {
                Err(DiagnosticError::runtime_error(
                    "State action not yet implemented".to_string(),
                    None,
                ))
            }
        }
    }
}

impl ErrorEffectHandler {
    /// Creates a new error effect handler.
    pub fn new(name: String) -> Self {
        Self {
            name,
            config: ErrorHandlerConfig::default(),
        }
    }
    
    /// Handles an error action.
    pub fn handle_error_action(&self, action: &ErrorAction) -> Result<Value> {
        match action {
            ErrorAction::Return(value) => Ok(value.clone()),
            ErrorAction::Throw(error) => {
                if self.config.allow_recovery {
                    // Attempt error recovery (simplified)
                    Ok(Value::string("Error recovered".to_string()))
                } else {
                    Err(error.clone())
                }
            },
            _ => {
                Err(DiagnosticError::runtime_error(
                    "Error action not yet implemented".to_string(),
                    None,
                ))
            }
        }
    }
}

impl Default for EffectHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}



impl Default for StateHandlerConfig {
    fn default() -> Self {
        Self {
            track_changes: true,
            max_snapshots: 100,
            validate_consistency: false,
        }
    }
}

impl Default for ErrorHandlerConfig {
    fn default() -> Self {
        Self {
            capture_stack_trace: true,
            allow_recovery: false,
            error_transformations: HashMap::new(),
        }
    }
}

impl fmt::Display for EffectHandlerImplementation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EffectHandler({}, effects={:?})", self.name, self.handled_effects)
    }
}

impl fmt::Display for HandlerArity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandlerArity::Fixed(n) => write!(f, "Fixed({n})"),
            HandlerArity::Variable(min) => write!(f, "Variable(min={min})"),
            HandlerArity::WithContinuation(n) => write!(f, "WithContinuation({n})"),
        }
    }
}

// Thread safety markers for effect handler types
unsafe impl Send for EffectHandlerImplementation {}
unsafe impl Sync for EffectHandlerImplementation {}

unsafe impl Send for HandlerFunction {}
unsafe impl Sync for HandlerFunction {}

unsafe impl Send for HandlerArity {}
unsafe impl Sync for HandlerArity {}

unsafe impl Send for EffectHandlerRegistry {}
unsafe impl Sync for EffectHandlerRegistry {}

unsafe impl Send for IOEffectHandler {}
unsafe impl Sync for IOEffectHandler {}

unsafe impl Send for StateEffectHandler {}
unsafe impl Sync for StateEffectHandler {}

unsafe impl Send for ErrorEffectHandler {}
unsafe impl Sync for ErrorEffectHandler {}

unsafe impl Send for IOHandlerConfig {}
unsafe impl Sync for IOHandlerConfig {}

unsafe impl Send for StateHandlerConfig {}
unsafe impl Sync for StateHandlerConfig {}

unsafe impl Send for ErrorHandlerConfig {}
unsafe impl Sync for ErrorHandlerConfig {}

unsafe impl Send for IORedirection {}
unsafe impl Sync for IORedirection {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_handler_registry() {
        let registry = EffectHandlerRegistry::new();
        
        // Should have default handlers for built-in effects
        assert!(registry.find_handler(&Effect::IO).is_some());
        assert!(registry.find_handler(&Effect::State).is_some());
        assert!(registry.find_handler(&Effect::Error).is_some());
    }
    
    #[test]
    fn test_handler_arity_checking() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let handler = EffectHandlerImplementation::new(
            "test".to_string(),
            vec![Effect::IO],
            env,
        );
        
        // Test fixed arity
        let fixed_arity = HandlerArity::Fixed(2);
        assert!(handler.check_handler_arity(&fixed_arity, 2).is_ok());
        assert!(handler.check_handler_arity(&fixed_arity, 1).is_err());
        assert!(handler.check_handler_arity(&fixed_arity, 3).is_err());
        
        // Test variable arity
        let var_arity = HandlerArity::Variable(1);
        assert!(handler.check_handler_arity(&var_arity, 1).is_ok());
        assert!(handler.check_handler_arity(&var_arity, 2).is_ok());
        assert!(handler.check_handler_arity(&var_arity, 0).is_err());
    }
}
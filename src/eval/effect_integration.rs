//! Effect integration architecture for monadic evaluator.
//!
//! This module provides a unified architecture for handling all effects
//! in the monadic evaluator, including IO operations, state management,
//! and error handling through a consistent effect handler pattern.

use crate::eval::{
    Value, Environment,
    monadic_architecture::{
        MonadicComputation, EffectInterpreter, InterpreterConfiguration,
    },
};
use crate::effects::{
    Effect, EffectContext, EffectResult, EffectHandler,
    IO, IOContext, FileMode, FileHandle,
    State, Reader, Maybe, Either,
    EffectfulComputation,
};
use crate::diagnostics::{Result, Error, Span};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use async_trait::async_trait;
#[cfg(feature = "async-runtime")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Unified effect handler registry that manages all effect types.
///
/// This follows the Registry pattern to provide a centralized way
/// to register, lookup, and execute effect handlers.
#[derive(Debug, Default)]
pub struct UnifiedEffectRegistry {
    /// Registered effect handlers by effect type
    handlers: HashMap<String, Arc<dyn UnifiedEffectHandler + Send + Sync>>,
    
    /// Default fallback handler
    fallback_handler: Option<Arc<dyn UnifiedEffectHandler + Send + Sync>>,
    
    /// Configuration for the registry
    config: EffectRegistryConfiguration,
}

/// Configuration for the effect registry
#[derive(Debug, Clone)]
pub struct EffectRegistryConfiguration {
    /// Whether to enable effect chaining (compose effects)
    pub enable_effect_chaining: bool,
    
    /// Maximum effect chain depth
    pub max_chain_depth: usize,
    
    /// Whether to enable parallel effect execution
    pub enable_parallel_effects: bool,
    
    /// Timeout for effect execution (in milliseconds)
    pub effect_timeout_ms: u64,
}

/// Enhanced effect handler trait that supports monadic operations.
///
/// This extends the basic EffectHandler to work seamlessly with
/// the monadic evaluator architecture.
#[async_trait]
pub trait UnifiedEffectHandler: std::fmt::Debug + Send + Sync {
    /// Handle an effect and return a monadic computation
    async fn handle_effect(
        &self,
        effect: &Effect,
        args: &[Value],
        context: &EffectContext,
    ) -> Result<MonadicComputation<Value>>;
    
    /// Get the name of the effect this handler manages
    fn effect_name(&self) -> &str;
    
    /// Check if this handler can handle the given effect
    fn can_handle(&self, effect: &Effect) -> bool;
    
    /// Get priority for handler selection (higher = more preferred)
    fn priority(&self) -> i32 {
        0
    }
    
    /// Initialize the handler (called once during registration)
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    
    /// Cleanup the handler (called during shutdown)
    async fn cleanup(&self) -> Result<()> {
        Ok(())
    }
    
    /// Get handler metadata for introspection
    fn metadata(&self) -> EffectHandlerMetadata {
        EffectHandlerMetadata {
            name: self.effect_name().to_string(),
            version: "1.0.0".to_string(),
            description: "Generic effect handler".to_string(),
            supported_effects: vec![],
            capabilities: vec![],
        }
    }
}

/// Metadata about an effect handler
#[derive(Debug, Clone)]
pub struct EffectHandlerMetadata {
    /// Handler name
    pub name: String,
    
    /// Handler version
    pub version: String,
    
    /// Human-readable description
    pub description: String,
    
    /// List of effects this handler supports
    pub supported_effects: Vec<Effect>,
    
    /// Handler capabilities
    pub capabilities: Vec<EffectCapability>,
}

/// Capabilities that an effect handler may have
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectCapability {
    /// Handler supports async operations
    AsyncSupport,
    
    /// Handler is stateful
    Stateful,
    
    /// Handler can be composed with others
    Composable,
    
    /// Handler supports transactions
    Transactional,
    
    /// Handler supports streaming
    Streaming,
    
    /// Custom capability
    Custom(String),
}

// ================================
// CONCRETE EFFECT HANDLERS
// ================================

/// IO effect handler that integrates with the monadic evaluator
#[derive(Debug)]
pub struct MonadicIOHandler {
    /// IO context for actual IO operations
    io_context: Arc<Mutex<IOContext>>,
    
    /// Configuration
    config: IOHandlerConfiguration,
}

/// Configuration for IO effect handler
#[derive(Debug, Clone)]
pub struct IOHandlerConfiguration {
    /// Whether to enable file operations
    pub enable_file_operations: bool,
    
    /// Whether to enable network operations
    pub enable_network_operations: bool,
    
    /// Default buffer size for IO operations
    pub default_buffer_size: usize,
    
    /// Timeout for IO operations
    pub io_timeout_ms: u64,
}

/// State effect handler for monadic state operations
#[derive(Debug, Default)]
pub struct MonadicStateHandler {
    /// Current state (thread-safe)
    state: Arc<Mutex<HashMap<String, Value>>>,
    
    /// Configuration
    config: StateHandlerConfiguration,
}

/// Configuration for state effect handler
#[derive(Debug, Clone)]
pub struct StateHandlerConfiguration {
    /// Whether to enable transactional state updates
    pub enable_transactions: bool,
    
    /// Maximum number of state keys
    pub max_state_keys: usize,
    
    /// Whether to enable state persistence
    pub enable_persistence: bool,
}

/// Error effect handler for monadic error operations
#[derive(Debug, Default)]
pub struct MonadicErrorHandler {
    /// Error handling configuration
    config: ErrorHandlerConfiguration,
}

/// Configuration for error effect handler
#[derive(Debug, Clone)]
pub struct ErrorHandlerConfiguration {
    /// Whether to enable stack trace capture
    pub capture_stack_traces: bool,
    
    /// Whether to enable error recovery
    pub enable_error_recovery: bool,
    
    /// Maximum error chain depth
    pub max_error_chain_depth: usize,
}

/// Maybe effect handler for optional value operations
#[derive(Debug, Default)]
pub struct MonadicMaybeHandler {
    /// Configuration
    config: MaybeHandlerConfiguration,
}

/// Configuration for Maybe effect handler
#[derive(Debug, Clone)]
pub struct MaybeHandlerConfiguration {
    /// Default behavior when encountering Nothing
    pub nothing_behavior: NothingBehavior,
}

/// Behavior when encountering Nothing values
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NothingBehavior {
    /// Return Nothing
    #[default]
    ReturnNothing,
    
    /// Throw an error
    ThrowError,
    
    /// Use a default value
    UseDefault(Value),
}

/// Composite effect handler that can chain multiple handlers
#[derive(Debug)]
pub struct CompositeEffectHandler {
    /// Chain of handlers to try in order
    handlers: Vec<Arc<dyn UnifiedEffectHandler + Send + Sync>>,
    
    /// Configuration
    config: CompositeHandlerConfiguration,
}

/// Configuration for composite effect handler
#[derive(Debug, Clone)]
pub struct CompositeHandlerConfiguration {
    /// Whether to short-circuit on first successful handler
    pub short_circuit: bool,
    
    /// Whether to collect results from all handlers
    pub collect_all_results: bool,
    
    /// How to combine results from multiple handlers
    pub combination_strategy: CombinationStrategy,
}

/// Strategy for combining results from multiple effect handlers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombinationStrategy {
    /// Use the first successful result
    First,
    
    /// Use the last successful result
    Last,
    
    /// Combine all results into a list
    Collect,
    
    /// Use a custom combination function
    Custom,
}

// ================================
// IMPLEMENTATION
// ================================

impl UnifiedEffectRegistry {
    /// Create a new effect registry
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            fallback_handler: None,
            config: EffectRegistryConfiguration::default(),
        }
    }
    
    /// Create a registry with standard effect handlers
    pub fn with_standard_handlers() -> Self {
        let mut registry = Self::new();
        
        // Register standard effect handlers
        let io_handler = Arc::new(MonadicIOHandler::new());
        let state_handler = Arc::new(MonadicStateHandler::new());
        let error_handler = Arc::new(MonadicErrorHandler::new());
        let maybe_handler = Arc::new(MonadicMaybeHandler::new());
        
        registry.register_handler("io".to_string(), io_handler);
        registry.register_handler("state".to_string(), state_handler);
        registry.register_handler("error".to_string(), error_handler);
        registry.register_handler("maybe".to_string(), maybe_handler);
        
        registry
    }
    
    /// Register an effect handler
    pub fn register_handler(
        &mut self,
        name: String,
        handler: Arc<dyn UnifiedEffectHandler + Send + Sync>,
    ) {
        self.handlers.insert(name, handler);
    }
    
    /// Find a handler for the given effect
    pub fn find_handler(&self, effect: &Effect) -> Option<Arc<dyn UnifiedEffectHandler + Send + Sync>> {
        // Try to find a specific handler
        for handler in self.handlers.values() {
            if handler.can_handle(effect) {
                return Some(handler.clone());
            }
        }
        
        // Fall back to default handler
        self.fallback_handler.clone()
    }
    
    /// Handle an effect using the appropriate handler
    pub async fn handle_effect(
        &self,
        effect: &Effect,
        args: &[Value],
        context: &EffectContext,
    ) -> Result<MonadicComputation<Value>> {
        if let Some(handler) = self.find_handler(effect) {
            handler.handle_effect(effect, args, context).await
        } else {
            // No handler found - return an error
            Err(Box::new(Error::runtime_error(
                format!("No handler found for effect: {effect:?}"),
                None,
            )))
        }
    }
    
    /// Get all registered handlers
    pub fn list_handlers(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
    
    /// Get configuration
    pub fn config(&self) -> &EffectRegistryConfiguration {
        &self.config
    }
}

// ================================
// CONCRETE HANDLER IMPLEMENTATIONS
// ================================

impl MonadicIOHandler {
    /// Create a new IO handler
    pub fn new() -> Self {
        Self {
            io_context: Arc::new(Mutex::new(IOContext::new())),
            config: IOHandlerConfiguration::default(),
        }
    }
    
    /// Create an IO handler with configuration
    pub fn with_config(config: IOHandlerConfiguration) -> Self {
        Self {
            io_context: Arc::new(Mutex::new(IOContext::new())),
            config,
        }
    }
}

impl Default for MonadicIOHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UnifiedEffectHandler for MonadicIOHandler {
    async fn handle_effect(
        &self,
        effect: &Effect,
        args: &[Value],
        _context: &EffectContext,
    ) -> Result<MonadicComputation<Value>> {
        match effect {
            Effect::IO => {
                // Determine the specific IO operation from args
                if let Some(Value::Symbol(op_symbol)) = args.first() {
                    match format!("symbol_{id}", id = op_symbol.id()).as_str() {
                        "write" | "display" => {
                            if let Some(value) = args.get(1) {
                                let value_cloned = value.clone();
                                let io_comp: IO<Value> = IO::<()>::write(value_cloned.clone()).map(move |_| value_cloned.clone());
                                Ok(MonadicComputation::IO(io_comp))
                            } else {
                                Ok(MonadicComputation::Pure(Value::Unspecified))
                            }
                        }
                        
                        "read-line" => {
                            let io_comp: IO<Value> = IO::<String>::read_line().map(Value::string);
                            Ok(MonadicComputation::IO(io_comp))
                        }
                        
                        "open-input-file" => {
                            if let Some(Value::Literal(crate::ast::Literal::String(filename))) = args.get(1) {
                                // For now, create a mock IO operation since open_file doesn't exist
                                let io_comp: IO<Value> = IO::<String>::pure(format!("opened: {filename}"))
                                    .map(|_| Value::Unspecified); // Simplified - would need proper Port implementation
                                Ok(MonadicComputation::IO(io_comp))
                            } else {
                                Err(Box::new(Error::runtime_error(
                                    "open-input-file requires filename argument".to_string(),
                                    None,
                                )))
                            }
                        }
                        
                        _ => Ok(MonadicComputation::Pure(Value::Unspecified)),
                    }
                } else {
                    Ok(MonadicComputation::Pure(Value::Unspecified))
                }
            }
            
            _ => Err(Box::new(Error::runtime_error(
                format!("IO handler cannot handle effect: {effect:?}"),
                None,
            )))
        }
    }
    
    fn effect_name(&self) -> &str {
        "io"
    }
    
    fn can_handle(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::IO)
    }
    
    fn priority(&self) -> i32 {
        100 // High priority for IO effects
    }
    
    fn metadata(&self) -> EffectHandlerMetadata {
        EffectHandlerMetadata {
            name: "MonadicIOHandler".to_string(),
            version: "1.0.0".to_string(),
            description: "Handler for IO effects in monadic evaluator".to_string(),
            supported_effects: vec![Effect::IO],
            capabilities: vec![
                EffectCapability::AsyncSupport,
                EffectCapability::Streaming,
            ],
        }
    }
}

impl MonadicStateHandler {
    /// Create a new state handler
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            config: StateHandlerConfiguration::default(),
        }
    }
}

#[async_trait]
impl UnifiedEffectHandler for MonadicStateHandler {
    async fn handle_effect(
        &self,
        effect: &Effect,
        args: &[Value],
        _context: &EffectContext,
    ) -> Result<MonadicComputation<Value>> {
        match effect {
            Effect::State => {
                if let Some(Value::Symbol(op_symbol)) = args.first() {
                    match format!("symbol_{id}", id = op_symbol.id()).as_str() {
                        "get" => {
                            // Return current state
                            let state = self.state.lock().unwrap();
                            let state_map = state.clone();
                            let state_value = Value::from_hash_map(state_map);
                            Ok(MonadicComputation::Pure(state_value))
                        }
                        
                        "put" => {
                            // Set new state
                            if let Some(new_state) = args.get(1) {
                                let mut state = self.state.lock().unwrap();
                                state.clear();
                                // Convert new_state to HashMap (simplified)
                                state.insert("value".to_string(), new_state.clone());
                                Ok(MonadicComputation::Pure(Value::Unspecified))
                            } else {
                                Err(Box::new(Error::runtime_error(
                                    "put requires state argument".to_string(),
                                    None,
                                )))
                            }
                        }
                        
                        _ => Ok(MonadicComputation::Pure(Value::Unspecified)),
                    }
                } else {
                    Ok(MonadicComputation::Pure(Value::Unspecified))
                }
            }
            
            _ => Err(Box::new(Error::runtime_error(
                format!("State handler cannot handle effect: {effect:?}"),
                None,
            )))
        }
    }
    
    fn effect_name(&self) -> &str {
        "state"
    }
    
    fn can_handle(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::State)
    }
    
    fn priority(&self) -> i32 {
        90 // High priority for state effects
    }
}

impl MonadicErrorHandler {
    /// Create a new error handler
    pub fn new() -> Self {
        Self {
            config: ErrorHandlerConfiguration::default(),
        }
    }
}

#[async_trait]
impl UnifiedEffectHandler for MonadicErrorHandler {
    async fn handle_effect(
        &self,
        effect: &Effect,
        args: &[Value],
        _context: &EffectContext,
    ) -> Result<MonadicComputation<Value>> {
        match effect {
            Effect::Error => {
                if let Some(Value::Symbol(op_symbol)) = args.first() {
                    match format!("symbol_{id}", id = op_symbol.id()).as_str() {
                        "throw" | "error" => {
                            if let Some(error_value) = args.get(1) {
                                let error_msg = format!("{error_value}");
                                let error = Error::runtime_error(error_msg, None);
                                Ok(MonadicComputation::Either(Either::left(error)))
                            } else {
                                let error = Error::runtime_error("Unspecified error".to_string(), None);
                                Ok(MonadicComputation::Either(Either::left(error)))
                            }
                        }
                        
                        "try" => {
                            // Wrap computation in error handling
                            if let Some(computation_value) = args.get(1) {
                                // In practice, we'd evaluate the computation and catch errors
                                Ok(MonadicComputation::Either(Either::right(computation_value.clone())))
                            } else {
                                Ok(MonadicComputation::Either(Either::right(Value::Unspecified)))
                            }
                        }
                        
                        _ => Ok(MonadicComputation::Pure(Value::Unspecified)),
                    }
                } else {
                    Ok(MonadicComputation::Pure(Value::Unspecified))
                }
            }
            
            _ => Err(Box::new(Error::runtime_error(
                format!("Error handler cannot handle effect: {effect:?}"),
                None,
            )))
        }
    }
    
    fn effect_name(&self) -> &str {
        "error"
    }
    
    fn can_handle(&self, effect: &Effect) -> bool {
        matches!(effect, Effect::Error)
    }
    
    fn priority(&self) -> i32 {
        80 // High priority for error effects
    }
}

impl MonadicMaybeHandler {
    /// Create a new maybe handler
    pub fn new() -> Self {
        Self {
            config: MaybeHandlerConfiguration::default(),
        }
    }
}

#[async_trait]
impl UnifiedEffectHandler for MonadicMaybeHandler {
    async fn handle_effect(
        &self,
        effect: &Effect,
        args: &[Value],
        _context: &EffectContext,
    ) -> Result<MonadicComputation<Value>> {
        // Handle Maybe-related operations
        if let Some(Value::Symbol(op_symbol)) = args.first() {
            match format!("symbol_{id}", id = op_symbol.id()).as_str() {
                "just" | "some" => {
                    if let Some(value) = args.get(1) {
                        Ok(MonadicComputation::Maybe(Maybe::just(value.clone())))
                    } else {
                        Ok(MonadicComputation::Maybe(Maybe::nothing()))
                    }
                }
                
                "nothing" | "none" => {
                    Ok(MonadicComputation::Maybe(Maybe::nothing()))
                }
                
                "maybe-bind" => {
                    if args.len() >= 3 {
                        let maybe_val = args.get(1).cloned().unwrap_or(Value::Nil);
                        let func_val = args.get(2).cloned().unwrap_or(Value::Nil);
                        
                        // Convert value to Maybe (simplified)
                        let maybe = if maybe_val == Value::Nil {
                            Maybe::nothing()
                        } else {
                            Maybe::just(maybe_val)
                        };
                        
                        // Apply function (simplified)
                        Ok(MonadicComputation::Maybe(maybe))
                    } else {
                        Err(Box::new(Error::runtime_error(
                            "maybe-bind requires maybe and function arguments".to_string(),
                            None,
                        )))
                    }
                }
                
                _ => Ok(MonadicComputation::Pure(Value::Unspecified)),
            }
        } else {
            Ok(MonadicComputation::Pure(Value::Unspecified))
        }
    }
    
    fn effect_name(&self) -> &str {
        "maybe"
    }
    
    fn can_handle(&self, effect: &Effect) -> bool {
        // Maybe handler can handle custom "Maybe" effects
        match effect {
            Effect::Custom(name) => name == "maybe",
            _ => false,
        }
    }
    
    fn priority(&self) -> i32 {
        70 // Medium priority for Maybe effects
    }
}

// Default configurations

impl Default for EffectRegistryConfiguration {
    fn default() -> Self {
        Self {
            enable_effect_chaining: true,
            max_chain_depth: 100,
            enable_parallel_effects: false,
            effect_timeout_ms: 5000,
        }
    }
}

impl Default for IOHandlerConfiguration {
    fn default() -> Self {
        Self {
            enable_file_operations: true,
            enable_network_operations: false, // Disabled by default for security
            default_buffer_size: 8192,
            io_timeout_ms: 1000,
        }
    }
}

impl Default for StateHandlerConfiguration {
    fn default() -> Self {
        Self {
            enable_transactions: false,
            max_state_keys: 10000,
            enable_persistence: false,
        }
    }
}

impl Default for ErrorHandlerConfiguration {
    fn default() -> Self {
        Self {
            capture_stack_traces: true,
            enable_error_recovery: false,
            max_error_chain_depth: 10,
        }
    }
}

impl Default for MaybeHandlerConfiguration {
    fn default() -> Self {
        Self {
            nothing_behavior: NothingBehavior::ReturnNothing,
        }
    }
}

// Utility extension for Value to HashMap conversion
static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl Value {
    /// Convert a HashMap to a Value (simplified implementation)
    fn from_hash_map(map: HashMap<String, Value>) -> Value {
        // This is a simplified conversion - in practice, you'd want a proper representation
        if map.is_empty() {
            Value::Nil
        } else {
            // For simplicity, just return the first value
            map.into_iter().next().map(|(_, v)| v).unwrap_or(Value::Nil)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unified_effect_registry() {
        let registry = UnifiedEffectRegistry::with_standard_handlers();
        let handlers = registry.list_handlers();
        
        assert!(handlers.contains(&"io".to_string()));
        assert!(handlers.contains(&"state".to_string()));
        assert!(handlers.contains(&"error".to_string()));
        assert!(handlers.contains(&"maybe".to_string()));
    }
    
    #[cfg(feature = "async-runtime")]
    #[tokio::test]
    async fn test_io_handler() {
        let handler = MonadicIOHandler::new();
        let effect = Effect::IO;
        let args = vec![
            Value::symbol_from_str("write".to_string()),
            Value::string("Hello, World!".to_string()),
        ];
        let context = EffectContext::new();
        
        let result = handler.handle_effect(&effect, &args, &context).await;
        assert!(result.is_ok());
        
        match result.unwrap() {
            MonadicComputation::IO(_) => {}, // Success
            _ => panic!("Expected IO computation"),
        }
    }
    
    #[cfg(feature = "async-runtime")]
    #[tokio::test]
    async fn test_state_handler() {
        let handler = MonadicStateHandler::new();
        let effect = Effect::State;
        let args = vec![
            Value::symbol_from_str("get".to_string()),
        ];
        let context = EffectContext::new();
        
        let result = handler.handle_effect(&effect, &args, &context).await;
        assert!(result.is_ok());
        
        match result.unwrap() {
            MonadicComputation::Pure(_) => {}, // Success
            _ => panic!("Expected pure computation"),
        }
    }
}

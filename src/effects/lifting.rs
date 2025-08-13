//! Automatic effect lifting for transparent effect tracking.
//!
//! This module implements the automatic lifting system that transparently
//! transforms operations with side effects into their monadic equivalents,
//! preserving Scheme semantics while enabling pure functional programming.

#![allow(missing_docs)]

use super::{Effect, EffectContext, MonadicValue, IOAction, StateAction, ErrorAction};
use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, ThreadSafeEnvironment};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// System for automatically lifting operations into monadic context.
#[derive(Debug, Clone)]
pub struct EffectLifter {
    /// Rules for lifting specific operations
    lifting_rules: HashMap<String, LiftingRule>,
    /// Context for effect inference
    context: EffectContext,
    /// Whether lifting is enabled globally
    enabled: bool,
    /// Cache of previously lifted operations
    lift_cache: HashMap<String, MonadicValue>,
}

/// A rule that defines how to lift an operation into a specific effect.
#[derive(Debug, Clone)]
pub struct LiftingRule {
    /// The target effect to lift into
    target_effect: Effect,
    /// How to transform the operation
    transformation: LiftingTransformation,
    /// Conditions under which this rule applies
    conditions: Vec<LiftingCondition>,
    /// Priority of this rule (higher = more priority)
    priority: u32,
}

/// How to transform an operation when lifting.
#[derive(Debug, Clone)]
pub enum LiftingTransformation {
    /// Direct lift: wrap the result in the monadic context
    Direct,
    /// Transform arguments before lifting (simplified - just direct for now)
    TransformArgs,
    /// Custom transformation function (simplified - just direct for now)
    Custom,
    /// Map to a different operation
    MapTo(String),
}

/// Condition for when a lifting rule should apply.
#[derive(Debug, Clone)]
pub enum LiftingCondition {
    /// Always apply
    Always,
    /// Apply when operation name matches exactly
    OperationName(String),
    /// Apply when operation name matches pattern
    OperationPattern(String),
    /// Apply when any of the given effects are present
    HasEffect(Vec<Effect>),
    /// Apply when all of the given effects are present
    HasAllEffects(Vec<Effect>),
    /// Apply when argument count matches
    ArgumentCount(usize),
    /// Apply when argument count is in range
    ArgumentRange(usize, usize),
    /// Apply when first argument is of specific type
    FirstArgType(ValueType),
    /// Custom condition function (simplified - placeholder for now)
    Custom,
}

/// Value type for pattern matching in lifting conditions.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Number,
    String,
    Symbol,
    Boolean,
    Pair,
    Nil,
    Procedure,
    Port,
    Any,
}

/// Registry of built-in lifting rules for standard operations.
pub struct BuiltinLiftingRules;

/// Configuration for the effect lifting system.
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct LiftingSystemConfig {
    /// Whether to enable automatic lifting
    pub enable_auto_lifting: bool,
    /// Whether to cache lifted operations
    pub enable_caching: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Whether to warn about lifted operations
    pub warn_on_lift: bool,
    /// Custom lifting rules
    pub custom_rules: HashMap<String, LiftingRule>,
}

impl EffectLifter {
    /// Creates a new effect lifter.
    pub fn new() -> Self {
        let mut lifter = Self {
            lifting_rules: HashMap::new(),
            context: EffectContext::pure(),
            enabled: true,
            lift_cache: HashMap::new(),
        };
        
        // Register built-in lifting rules
        lifter.register_builtin_rules();
        lifter
    }
    
    /// Creates an effect lifter with custom configuration.
    pub fn with_config(config: LiftingSystemConfig) -> Self {
        let mut lifter = Self::new();
        lifter.enabled = config.enable_auto_lifting;
        
        // Add custom rules
        for (name, rule) in config.custom_rules {
            lifter.add_rule(name, rule);
        }
        
        lifter
    }
    
    /// Adds a lifting rule.
    pub fn add_rule(&mut self, operation: String, rule: LiftingRule) {
        self.lifting_rules.insert(operation, rule);
    }
    
    /// Removes a lifting rule.
    pub fn remove_rule(&mut self, operation: &str) -> Option<LiftingRule> {
        self.lifting_rules.remove(operation)
    }
    
    /// Sets the current effect context.
    pub fn set_context(&mut self, context: EffectContext) {
        self.context = context;
    }
    
    /// Attempts to lift an operation call.
    pub fn lift_operation(
        &mut self, 
        operation: &str, 
        args: &[Value]
    ) -> Option<MonadicValue> {
        if !self.enabled {
            return None;
        }
        
        // Check cache first
        let cache_key = format!("{operation}:{}", args.len());
        if let Some(cached) = self.lift_cache.get(&cache_key) {
            return Some(cached.clone());
        }
        
        // Find applicable lifting rules
        let mut applicable_rules = Vec::new();
        for (name, rule) in &self.lifting_rules {
            if self.rule_applies(name, rule, operation, args) {
                applicable_rules.push((name.clone(), rule.clone()));
            }
        }
        
        // Sort by priority (highest first)
        applicable_rules.sort_by(|a, b| b.1.priority.cmp(&a.1.priority));
        
        // Apply the first applicable rule
        if let Some((_, rule)) = applicable_rules.first() {
            match self.apply_lifting_rule(rule, operation, args) {
                Ok(lifted) => {
                    // Cache the result
                    self.lift_cache.insert(cache_key, lifted.clone());
                    Some(lifted)
                },
                Err(_) => None,
            }
        } else {
            None
        }
    }
    
    /// Checks if a lifting rule applies to an operation.
    fn rule_applies(
        &self,
        rule_name: &str,
        rule: &LiftingRule,
        operation: &str,
        args: &[Value]
    ) -> bool {
        // If rule name matches operation exactly, check conditions
        if rule_name == operation || rule.conditions.is_empty() {
            return rule.conditions.iter().all(|cond| {
                self.condition_matches(cond, operation, args)
            });
        }
        
        // Otherwise, check if any condition explicitly matches
        rule.conditions.iter().any(|cond| {
            self.condition_matches(cond, operation, args)
        })
    }
    
    /// Checks if a lifting condition matches.
    fn condition_matches(
        &self,
        condition: &LiftingCondition,
        operation: &str,
        args: &[Value]
    ) -> bool {
        match condition {
            LiftingCondition::Always => true,
            LiftingCondition::OperationName(name) => operation == name,
            LiftingCondition::OperationPattern(pattern) => {
                // Simple pattern matching (could be extended)
                operation.contains(pattern)
            },
            LiftingCondition::HasEffect(effects) => {
                effects.iter().any(|e| self.context.has_effect(e))
            },
            LiftingCondition::HasAllEffects(effects) => {
                effects.iter().all(|e| self.context.has_effect(e))
            },
            LiftingCondition::ArgumentCount(count) => args.len() == *count,
            LiftingCondition::ArgumentRange(min, max) => {
                args.len() >= *min && args.len() <= *max
            },
            LiftingCondition::FirstArgType(value_type) => {
                args.first().map(|v| self.value_matches_type(v, value_type))
                    .unwrap_or(false)
            },
            LiftingCondition::Custom => false, // Placeholder - always false for now
        }
    }
    
    /// Checks if a value matches a type pattern.
    fn value_matches_type(&self, value: &Value, value_type: &ValueType) -> bool {
        match value_type {
            ValueType::Number => value.is_number(),
            ValueType::String => value.is_string(),
            ValueType::Symbol => value.is_symbol(),
            ValueType::Boolean => matches!(value, Value::Literal(crate::ast::Literal::Boolean(_))),
            ValueType::Pair => value.is_pair(),
            ValueType::Nil => value.is_nil(),
            ValueType::Procedure => value.is_procedure(),
            ValueType::Port => value.is_port(),
            ValueType::Any => true,
        }
    }
    
    /// Applies a lifting rule to transform an operation.
    fn apply_lifting_rule(
        &self,
        rule: &LiftingRule,
        operation: &str,
        args: &[Value]
    ) -> Result<MonadicValue> {
        match &rule.transformation {
            LiftingTransformation::Direct => {
                // Create a monadic value for the target effect
                match &rule.target_effect {
                    Effect::Pure => Ok(MonadicValue::pure(Value::Unspecified)),
                    Effect::IO => {
                        let io_action = self.create_io_action(operation, args)?;
                        Ok(MonadicValue::io(io_action))
                    },
                    Effect::State => {
                        let state_action = self.create_state_action(operation, args)?;
                        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
                        Ok(MonadicValue::state(state_action, env))
                    },
                    Effect::Error => {
                        let error_action = self.create_error_action(operation, args)?;
                        Ok(MonadicValue::error(error_action))
                    },
                    Effect::Custom(name) => {
                        // For custom effects, create a combined computation
                        Ok(MonadicValue::pure(Value::string(format!("Custom effect: {name}"))))
                    }
                }
            },
            LiftingTransformation::TransformArgs => {
                // Transform arguments and then apply direct lifting
                // For now, just use direct lifting
                self.apply_lifting_rule(
                    &LiftingRule {
                        target_effect: rule.target_effect.clone(),
                        transformation: LiftingTransformation::Direct,
                        conditions: rule.conditions.clone(),
                        priority: rule.priority,
                    },
                    operation,
                    args
                )
            },
            LiftingTransformation::Custom => {
                // Apply custom transformation
                // For now, return a placeholder
                Ok(MonadicValue::pure(Value::string("Custom transformation".to_string())))
            },
            LiftingTransformation::MapTo(new_operation) => {
                // Recursively lift the mapped operation
                self.apply_lifting_rule(
                    &LiftingRule {
                        target_effect: rule.target_effect.clone(),
                        transformation: LiftingTransformation::Direct,
                        conditions: rule.conditions.clone(),
                        priority: rule.priority,
                    },
                    new_operation,
                    args
                )
            }
        }
    }
    
    /// Creates an IO action for the given operation.
    fn create_io_action(&self, operation: &str, args: &[Value]) -> Result<IOAction> {
        match operation {
            "display" => {
                if let Some(value) = args.first() {
                    Ok(IOAction::Print(value.clone()))
                } else {
                    Err(Box::new(DiagnosticError::runtime_error(
                        "display requires at least one argument".to_string(),
                        None,
                    )))
                }
            },
            "write" => {
                if let Some(value) = args.first() {
                    Ok(IOAction::WriteValue(value.clone()))
                } else {
                    Err(Box::new(DiagnosticError::runtime_error(
                        "write requires at least one argument".to_string(),
                        None,
                    )))
                }
            },
            "newline" => Ok(IOAction::Newline),
            "read" => Ok(IOAction::Read(super::IOSource::Stdin)),
            _ => Ok(IOAction::Custom(operation.to_string(), args.to_vec())),
        }
    }
    
    /// Creates a state action for the given operation.
    fn create_state_action(&self, operation: &str, args: &[Value]) -> Result<StateAction> {
        match operation {
            "set!" => {
                if args.len() >= 2 {
                    if let Some(var_name) = args[0].as_string() {
                        Ok(StateAction::SetVar(var_name.to_string(), args[1].clone()))
                    } else {
                        Err(Box::new(DiagnosticError::runtime_error(
                            "set! requires a variable name".to_string(),
                            None,
                        )))
                    }
                } else {
                    Err(Box::new(DiagnosticError::runtime_error(
                        "set! requires two arguments".to_string(),
                        None,
                    )))
                }
            },
            _ => Ok(StateAction::Custom(operation.to_string(), args.to_vec())),
        }
    }
    
    /// Creates an error action for the given operation.
    fn create_error_action(&self, operation: &str, args: &[Value]) -> Result<ErrorAction> {
        match operation {
            "error" | "raise" => {
                let error_msg = if let Some(msg) = args.first() {
                    format!("Error: {msg}")
                } else {
                    "Unspecified error".to_string()
                };
                let error = DiagnosticError::runtime_error(error_msg, None);
                Ok(ErrorAction::Throw(error))
            },
            _ => Ok(ErrorAction::Custom(operation.to_string(), args.to_vec())),
        }
    }
    
    /// Registers built-in lifting rules.
    fn register_builtin_rules(&mut self) {
        // IO operations
        let io_ops = vec!["display", "write", "newline", "read", "print", "write-char"];
        for op in io_ops {
            self.add_rule(op.to_string(), LiftingRule {
                target_effect: Effect::IO,
                transformation: LiftingTransformation::Direct,
                conditions: vec![LiftingCondition::OperationName(op.to_string())],
                priority: 100,
            });
        }
        
        // State operations
        let state_ops = vec!["set!", "vector-set!", "string-set!", "hashtable-set!"];
        for op in state_ops {
            self.add_rule(op.to_string(), LiftingRule {
                target_effect: Effect::State,
                transformation: LiftingTransformation::Direct,
                conditions: vec![LiftingCondition::OperationName(op.to_string())],
                priority: 100,
            });
        }
        
        // Error operations
        let error_ops = vec!["error", "raise", "throw", "assert"];
        for op in error_ops {
            self.add_rule(op.to_string(), LiftingRule {
                target_effect: Effect::Error,
                transformation: LiftingTransformation::Direct,
                conditions: vec![LiftingCondition::OperationName(op.to_string())],
                priority: 100,
            });
        }
    }
    
    /// Clears the lift cache.
    pub fn clear_cache(&mut self) {
        self.lift_cache.clear();
    }
    
    /// Gets cache statistics.
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.lift_cache.len(), self.lift_cache.capacity())
    }
    
    /// Enables or disables lifting.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Returns whether lifting is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl BuiltinLiftingRules {
    /// Creates the standard set of lifting rules.
    pub fn create_standard_rules() -> HashMap<String, LiftingRule> {
        let mut rules = HashMap::new();
        
        // Display operation
        rules.insert("display".to_string(), LiftingRule {
            target_effect: Effect::IO,
            transformation: LiftingTransformation::Direct,
            conditions: vec![
                LiftingCondition::OperationName("display".to_string()),
                LiftingCondition::ArgumentRange(1, 2),
            ],
            priority: 100,
        });
        
        // Newline operation
        rules.insert("newline".to_string(), LiftingRule {
            target_effect: Effect::IO,
            transformation: LiftingTransformation::Direct,
            conditions: vec![
                LiftingCondition::OperationName("newline".to_string()),
                LiftingCondition::ArgumentRange(0, 1),
            ],
            priority: 100,
        });
        
        // Set! operation
        rules.insert("set!".to_string(), LiftingRule {
            target_effect: Effect::State,
            transformation: LiftingTransformation::Direct,
            conditions: vec![
                LiftingCondition::OperationName("set!".to_string()),
                LiftingCondition::ArgumentCount(2),
            ],
            priority: 100,
        });
        
        // Error operation
        rules.insert("error".to_string(), LiftingRule {
            target_effect: Effect::Error,
            transformation: LiftingTransformation::Direct,
            conditions: vec![
                LiftingCondition::OperationName("error".to_string()),
                LiftingCondition::ArgumentRange(0, 2),
            ],
            priority: 100,
        });
        
        rules
    }
    
    /// Creates rules for file operations.
    pub fn create_file_rules() -> HashMap<String, LiftingRule> {
        let mut rules = HashMap::new();
        
        let file_ops = vec![
            "open-input-file", "open-output-file", "close-port",
            "read-char", "write-char", "read-line", "write-line"
        ];
        
        for op in file_ops {
            rules.insert(op.to_string(), LiftingRule {
                target_effect: Effect::IO,
                transformation: LiftingTransformation::Direct,
                conditions: vec![LiftingCondition::OperationName(op.to_string())],
                priority: 90,
            });
        }
        
        rules
    }
    
    /// Creates rules for vector operations.
    pub fn create_vector_rules() -> HashMap<String, LiftingRule> {
        let mut rules = HashMap::new();
        
        // Vector mutation operations
        rules.insert("vector-set!".to_string(), LiftingRule {
            target_effect: Effect::State,
            transformation: LiftingTransformation::Direct,
            conditions: vec![
                LiftingCondition::OperationName("vector-set!".to_string()),
                LiftingCondition::ArgumentCount(3),
            ],
            priority: 90,
        });
        
        rules.insert("vector-fill!".to_string(), LiftingRule {
            target_effect: Effect::State,
            transformation: LiftingTransformation::Direct,
            conditions: vec![
                LiftingCondition::OperationName("vector-fill!".to_string()),
                LiftingCondition::ArgumentRange(2, 4),
            ],
            priority: 90,
        });
        
        rules
    }
}

impl LiftingSystemConfig {
    
    /// Creates configuration with no automatic lifting.
    pub fn no_lifting() -> Self {
        Self {
            enable_auto_lifting: false,
            enable_caching: false,
            max_cache_size: 0,
            warn_on_lift: false,
            custom_rules: HashMap::new(),
        }
    }
    
    /// Creates configuration optimized for debugging.
    pub fn debug() -> Self {
        Self {
            enable_auto_lifting: true,
            enable_caching: false, // Disable caching for debugging
            max_cache_size: 0,
            warn_on_lift: true, // Warn about lifted operations
            custom_rules: HashMap::new(),
        }
    }
}

// Custom implementations for traits that don't auto-derive

impl PartialEq for LiftingTransformation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LiftingTransformation::Direct, LiftingTransformation::Direct) => true,
            (LiftingTransformation::MapTo(a), LiftingTransformation::MapTo(b)) => a == b,
            _ => false, // Custom functions can't be compared
        }
    }
}

impl PartialEq for LiftingCondition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LiftingCondition::Always, LiftingCondition::Always) => true,
            (LiftingCondition::OperationName(a), LiftingCondition::OperationName(b)) => a == b,
            (LiftingCondition::OperationPattern(a), LiftingCondition::OperationPattern(b)) => a == b,
            (LiftingCondition::HasEffect(a), LiftingCondition::HasEffect(b)) => a == b,
            (LiftingCondition::HasAllEffects(a), LiftingCondition::HasAllEffects(b)) => a == b,
            (LiftingCondition::ArgumentCount(a), LiftingCondition::ArgumentCount(b)) => a == b,
            (LiftingCondition::ArgumentRange(a1, a2), LiftingCondition::ArgumentRange(b1, b2)) => 
                a1 == b1 && a2 == b2,
            (LiftingCondition::FirstArgType(a), LiftingCondition::FirstArgType(b)) => a == b,
            _ => false, // Custom functions can't be compared
        }
    }
}

impl Default for EffectLifter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for LiftingRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LiftingRule(effect={}, priority={}, conditions={})", 
               self.target_effect, 
               self.priority, 
               self.conditions.len())
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::Number => write!(f, "Number"),
            ValueType::String => write!(f, "String"),
            ValueType::Symbol => write!(f, "Symbol"),
            ValueType::Boolean => write!(f, "Boolean"),
            ValueType::Pair => write!(f, "Pair"),
            ValueType::Nil => write!(f, "Nil"),
            ValueType::Procedure => write!(f, "Procedure"),
            ValueType::Port => write!(f, "Port"),
            ValueType::Any => write!(f, "Any"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lifting_rule_creation() {
        let rule = LiftingRule {
            target_effect: Effect::IO,
            transformation: LiftingTransformation::Direct,
            conditions: vec![LiftingCondition::OperationName("display".to_string())],
            priority: 100,
        };
        
        assert_eq!(rule.target_effect, Effect::IO);
        assert_eq!(rule.priority, 100);
    }
    
    #[test]
    fn test_effect_lifter() {
        let mut lifter = EffectLifter::new();
        
        // Test lifting a display operation
        let args = vec![Value::string("Hello, world!".to_string())];
        let lifted = lifter.lift_operation("display", &args);
        
        assert!(lifted.is_some());
        let monadic_val = lifted.unwrap();
        assert!(monadic_val.effects().contains(&Effect::IO));
    }
    
    #[test]
    fn test_builtin_rules() {
        let rules = BuiltinLiftingRules::create_standard_rules();
        
        assert!(rules.contains_key("display"));
        assert!(rules.contains_key("newline"));
        assert!(rules.contains_key("set!"));
        assert!(rules.contains_key("error"));
    }
}
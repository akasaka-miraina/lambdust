//! Environment and variable binding management
//!
//! This module provides a unified copy-on-write (COW) environment
//! implementation for efficient memory usage and variable scoping.

use crate::error::{LambdustError, Result};
use crate::macros::Macro;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
#[cfg(feature = "development")]
use std::sync::mpsc;

/// Condition register for tracking runtime-variable execution contexts
/// This system tracks conditions that affect execution context validity
#[derive(Debug, Clone, Default)]
pub struct ConditionRegister {
    /// Runtime conditions that affect context validity
    runtime_conditions: HashMap<String, RuntimeCondition>,
    /// Global condition generation counter
    generation: u64,
}

/// Runtime condition that affects execution context validity
/// 
/// Tracks conditions that change during program execution and affect
/// whether cached execution contexts remain valid.
#[derive(Debug, Clone)]
pub struct RuntimeCondition {
    /// Condition identifier
    pub id: String,
    /// Current condition value
    pub value: ConditionValue,
    /// Generation when this condition was last updated
    pub generation: u64,
    /// Whether this condition affects idempotency
    pub affects_idempotency: bool,
}

/// Values that runtime conditions can take
/// 
/// Represents different types of values that can be tracked
/// as runtime conditions affecting execution context validity.
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionValue {
    /// Boolean condition
    Boolean(bool),
    /// Integer condition (e.g., recursion depth)
    Integer(i64),
    /// String condition (e.g., optimization level)
    String(String),
    /// Complex condition with multiple values
    Complex(HashMap<String, String>),
}

/// Statistics about the context cache
/// 
/// Provides insights into cache performance and memory usage
/// for idempotent execution context caching.
#[derive(Debug, Clone)]
pub struct ContextCacheStats {
    /// Number of cached execution contexts
    pub cached_contexts: usize,
    /// Memory usage estimate for the cache
    pub cache_memory_usage: usize,
    /// Size of the condition register
    pub condition_register_size: usize,
}

impl ConditionRegister {
    /// Create a new condition register
    #[must_use] pub fn new() -> Self {
        Self {
            runtime_conditions: HashMap::new(),
            generation: 0,
        }
    }

    /// Check if a condition affects a given context dependency
    #[must_use] pub fn affects_context(&self, dependency: &str) -> bool {
        self.runtime_conditions.values()
            .filter(|condition| condition.affects_idempotency)
            .any(|condition| condition.id == dependency)
    }

    /// Get all conditions that affect idempotency
    #[must_use] pub fn idempotency_affecting_conditions(&self) -> Vec<&RuntimeCondition> {
        self.runtime_conditions.values()
            .filter(|condition| condition.affects_idempotency)
            .collect()
    }

    /// Clear all conditions
    pub fn clear(&mut self) {
        self.runtime_conditions.clear();
        self.generation += 1;
    }

    /// Get current generation
    #[must_use] pub fn generation(&self) -> u64 {
        self.generation
    }
}

impl RuntimeCondition {
    /// Create a new runtime condition
    #[must_use] pub fn new(id: String, value: ConditionValue, affects_idempotency: bool) -> Self {
        Self {
            id,
            value,
            generation: 0,
            affects_idempotency,
        }
    }

    /// Update the condition value
    pub fn update_value(&mut self, new_value: ConditionValue, generation: u64) {
        self.value = new_value;
        self.generation = generation;
    }

    /// Check if condition matches a specific value
    #[must_use] pub fn matches_value(&self, value: &ConditionValue) -> bool {
        &self.value == value
    }
}

/// Environment change tracking system for monitoring variable modifications
/// 
/// Tracks all changes to environment bindings and macros for debugging,
/// optimization, and incremental computation purposes.
#[derive(Debug, Clone, Default)]
pub struct EnvironmentChangeTracker {
    /// History of environment changes
    changes: Vec<EnvironmentChange>,
    /// Generation counter for tracking change order
    generation: u64,
    /// Whether change tracking is enabled
    enabled: bool,
    /// Maximum number of changes to track
    max_history: usize,
}

/// Represents a single environment change
/// 
/// Records detailed information about a specific change to the environment,
/// including timestamps, old/new values, and change type.
#[derive(Debug, Clone)]
pub struct EnvironmentChange {
    /// Generation when the change occurred
    pub generation: u64,
    /// Type of change
    pub change_type: ChangeType,
    /// Variable name affected
    pub variable_name: String,
    /// Previous value (if any)
    pub old_value: Option<Value>,
    /// New value (if any)
    pub new_value: Option<Value>,
    /// Timestamp when change occurred
    pub timestamp: std::time::SystemTime,
}

/// Types of environment changes
/// 
/// Categorizes different kinds of modifications that can occur
/// to environment bindings and macro definitions.
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    /// Variable binding added
    VariableAdded,
    /// Variable binding updated
    VariableUpdated,
    /// Variable binding removed
    VariableRemoved,
    /// Macro definition added
    MacroAdded,
    /// Macro definition updated
    MacroUpdated,
    /// Macro definition removed
    MacroRemoved,
    /// Environment extended (new frame added)
    EnvironmentExtended,
    /// Environment frozen
    EnvironmentFrozen,
}

impl EnvironmentChangeTracker {
    /// Create a new change tracker
    #[must_use] pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            generation: 0,
            enabled: true,
            max_history: 1000, // Track last 1000 changes by default
        }
    }

    /// Enable or disable change tracking
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Record a new environment change
    pub fn record_change(&mut self, change_type: ChangeType, variable_name: String, old_value: Option<Value>, new_value: Option<Value>) {
        if !self.enabled {
            return;
        }

        self.generation += 1;
        let change = EnvironmentChange {
            generation: self.generation,
            change_type,
            variable_name,
            old_value,
            new_value,
            timestamp: std::time::SystemTime::now(),
        };

        self.changes.push(change);

        // Maintain history size limit
        if self.changes.len() > self.max_history {
            self.changes.drain(0..(self.changes.len() - self.max_history));
        }
    }

    /// Get all changes since a specific generation
    #[must_use] pub fn get_changes_since(&self, generation: u64) -> Vec<&EnvironmentChange> {
        self.changes.iter()
            .filter(|change| change.generation > generation)
            .collect()
    }

    /// Get the current generation
    #[must_use] pub fn current_generation(&self) -> u64 {
        self.generation
    }

    /// Get all changes for a specific variable
    #[must_use] pub fn get_variable_history(&self, variable_name: &str) -> Vec<&EnvironmentChange> {
        self.changes.iter()
            .filter(|change| change.variable_name == variable_name)
            .collect()
    }

    /// Clear change history
    pub fn clear_history(&mut self) {
        self.changes.clear();
        self.generation = 0;
    }

    /// Get the total number of changes tracked
    #[must_use] pub fn change_count(&self) -> usize {
        self.changes.len()
    }
}

/// Statistics message sent from evaluators to the environment
/// 
/// Provides a decoupled way for evaluators to report performance
/// and optimization statistics through the environment system.
/// Only available in development builds for performance analysis.
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub enum StatisticsMessage {
    /// Evaluation completed
    EvaluationCompleted {
        /// Type of expression that was evaluated
        expression_type: String,
        /// Execution time in nanoseconds
        execution_time_ns: u64,
        /// Current recursion depth
        recursion_depth: usize,
    },
    /// Optimization applied
    OptimizationApplied {
        optimization_type: String,
        improvement_factor: f64,
        memory_saved: usize,
    },
    /// JIT compilation event
    JitCompilation {
        expression_hash: String,
        compilation_time_ns: u64,
        code_size: usize,
    },
    /// Memory allocation event
    MemoryAllocation {
        allocation_type: String,
        size_bytes: usize,
        pool_usage: bool,
    },
    /// Continuation pooling event
    ContinuationPooling {
        continuation_type: String,
        pool_hit: bool,
        efficiency_gain: f64,
    },
    /// Hot path detection
    HotPathDetected {
        expression_hash: String,
        frequency: u64,
        optimization_candidate: bool,
    },
    /// Idempotent context cache hit
    ContextCacheHit {
        expression_hash: String,
        cache_generation: u64,
        execution_time_saved_ns: u64,
    },
    /// Idempotent context cache miss
    ContextCacheMiss {
        expression_hash: String,
        reason: String,
        fallback_execution_time_ns: u64,
    },
    /// Runtime condition changed
    RuntimeConditionChanged {
        condition_id: String,
        affects_idempotency: bool,
        contexts_invalidated: usize,
    },
}

/// Statistics processor interface
/// 
/// Defines the interface for processing and aggregating statistics
/// messages from evaluators and other system components.
/// Only available in development builds for performance analysis.
#[cfg(feature = "development")]
pub trait StatisticsProcessor {
    /// Process a statistics message
    fn process_message(&mut self, message: StatisticsMessage);
    
    /// Get current statistics summary
    fn get_summary(&self) -> StatisticsSummary;
    
    /// Reset statistics
    fn reset(&mut self);
}

/// Summary of collected statistics
/// 
/// Aggregated statistics from all performance monitoring across
/// the system, providing insights into optimization effectiveness.
/// Only available in development builds for performance analysis.
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct StatisticsSummary {
    pub total_evaluations: usize,
    pub total_optimizations: usize,
    pub jit_compilations: usize,
    pub memory_allocations: usize,
    pub continuation_pool_hits: usize,
    pub hot_paths_detected: usize,
    pub context_cache_hits: usize,
    pub context_cache_misses: usize,
    pub runtime_condition_changes: usize,
    pub average_execution_time_ns: u64,
    pub total_memory_saved: usize,
    pub average_optimization_improvement: f64,
    pub total_execution_time_saved_ns: u64,
}

/// Basic implementation of statistics processor
/// 
/// Simple accumulator-based implementation of the StatisticsProcessor
/// trait that maintains counters and averages for basic statistics.
/// Only available in development builds for performance analysis.
#[cfg(feature = "development")]
#[derive(Debug, Default)]
pub struct BasicStatisticsProcessor {
    total_evaluations: usize,
    total_optimizations: usize,
    jit_compilations: usize,
    memory_allocations: usize,
    continuation_pool_hits: usize,
    hot_paths_detected: usize,
    context_cache_hits: usize,
    context_cache_misses: usize,
    runtime_condition_changes: usize,
    total_execution_time_ns: u64,
    total_memory_saved: usize,
    total_execution_time_saved_ns: u64,
    optimization_improvements: Vec<f64>,
}

#[cfg(feature = "development")]
impl StatisticsProcessor for BasicStatisticsProcessor {
    fn process_message(&mut self, message: StatisticsMessage) {
        match message {
            StatisticsMessage::EvaluationCompleted { execution_time_ns, .. } => {
                self.total_evaluations += 1;
                self.total_execution_time_ns += execution_time_ns;
            }
            StatisticsMessage::OptimizationApplied { improvement_factor, memory_saved, .. } => {
                self.total_optimizations += 1;
                self.total_memory_saved += memory_saved;
                self.optimization_improvements.push(improvement_factor);
            }
            StatisticsMessage::JitCompilation { .. } => {
                self.jit_compilations += 1;
            }
            StatisticsMessage::MemoryAllocation { .. } => {
                self.memory_allocations += 1;
            }
            StatisticsMessage::ContinuationPooling { pool_hit, .. } => {
                if pool_hit {
                    self.continuation_pool_hits += 1;
                }
            }
            StatisticsMessage::HotPathDetected { .. } => {
                self.hot_paths_detected += 1;
            }
            StatisticsMessage::ContextCacheHit { execution_time_saved_ns, .. } => {
                self.context_cache_hits += 1;
                self.total_execution_time_saved_ns += execution_time_saved_ns;
            }
            StatisticsMessage::ContextCacheMiss { .. } => {
                self.context_cache_misses += 1;
            }
            StatisticsMessage::RuntimeConditionChanged { .. } => {
                self.runtime_condition_changes += 1;
            }
        }
    }

    fn get_summary(&self) -> StatisticsSummary {
        let average_execution_time_ns = if self.total_evaluations > 0 {
            self.total_execution_time_ns / self.total_evaluations as u64
        } else {
            0
        };

        let average_optimization_improvement = if !self.optimization_improvements.is_empty() {
            self.optimization_improvements.iter().sum::<f64>() / self.optimization_improvements.len() as f64
        } else {
            0.0
        };

        StatisticsSummary {
            total_evaluations: self.total_evaluations,
            total_optimizations: self.total_optimizations,
            jit_compilations: self.jit_compilations,
            memory_allocations: self.memory_allocations,
            continuation_pool_hits: self.continuation_pool_hits,
            hot_paths_detected: self.hot_paths_detected,
            context_cache_hits: self.context_cache_hits,
            context_cache_misses: self.context_cache_misses,
            runtime_condition_changes: self.runtime_condition_changes,
            average_execution_time_ns,
            total_memory_saved: self.total_memory_saved,
            average_optimization_improvement,
            total_execution_time_saved_ns: self.total_execution_time_saved_ns,
        }
    }

    fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Shared environment using copy-on-write optimization
/// Reduces memory usage by sharing immutable parent environments
#[derive(Debug, Clone)]
pub struct SharedEnvironment {
    /// Local bindings for this environment frame
    /// Only contains bindings added to this specific frame
    local_bindings: HashMap<String, Value>,

    /// Macro definitions for this environment frame
    /// Only contains macros added to this specific frame
    local_macros: HashMap<String, Macro>,

    /// Shared parent environment chain
    /// Uses Rc for efficient sharing without cloning
    parent: Option<Rc<SharedEnvironment>>,

    /// Cached immutable bindings for fast lookup
    /// Contains flattened view of all bindings up the chain
    immutable_cache: Option<Rc<HashMap<String, Value>>>,

    /// Cached immutable macros for fast lookup
    /// Contains flattened view of all macros up the chain
    macro_cache: Option<Rc<HashMap<String, Macro>>>,

    /// Generation counter for cache invalidation
    /// Incremented when local bindings change
    generation: u32,

    /// Whether this environment is "frozen" (immutable)
    /// Frozen environments can be safely shared and cached
    is_frozen: bool,

    /// Statistics sender channel for decoupled statistics reporting
    /// Only the global environment holds the actual sender
    /// Only available in development builds
    #[cfg(feature = "development")]
    statistics_sender: Option<mpsc::Sender<StatisticsMessage>>,

    /// Cache for idempotent execution contexts
    /// Maps expression hash to execution context for fast lookup
    /// Only caches contexts guaranteed to be idempotent (pure functions)
    idempotent_context_cache: HashMap<String, crate::evaluator::ExecutionContext>,

    /// Condition register for runtime-variable execution contexts
    /// Tracks runtime conditions that affect execution context validity
    condition_register: ConditionRegister,

    /// Change tracker for monitoring environment modifications
    /// Tracks all variable binding and macro definition changes
    change_tracker: EnvironmentChangeTracker,
}

impl SharedEnvironment {
    /// Create a new global shared environment
    #[must_use] pub fn new() -> Self {
        SharedEnvironment {
            local_bindings: HashMap::new(),
            local_macros: HashMap::new(),
            parent: None,
            immutable_cache: None,
            macro_cache: None,
            generation: 0,
            is_frozen: false,
            #[cfg(feature = "development")]
            statistics_sender: None,
            idempotent_context_cache: HashMap::new(),
            condition_register: ConditionRegister::default(),
            change_tracker: EnvironmentChangeTracker::new(),
        }
    }

    /// Create a new global shared environment with statistics processing
    /// Only available in development builds
    #[cfg(feature = "development")]
    #[must_use] pub fn with_statistics_processor() -> (Self, mpsc::Receiver<StatisticsMessage>) {
        let (sender, receiver) = mpsc::channel();
        let env = SharedEnvironment {
            local_bindings: HashMap::new(),
            local_macros: HashMap::new(),
            parent: None,
            immutable_cache: None,
            macro_cache: None,
            generation: 0,
            is_frozen: false,
            #[cfg(feature = "development")]
            statistics_sender: Some(sender),
            idempotent_context_cache: HashMap::new(),
            condition_register: ConditionRegister::new(),
            change_tracker: EnvironmentChangeTracker::new(),
        };
        (env, receiver)
    }

    /// Create a new shared environment with a parent
    #[must_use] pub fn with_parent(parent: Rc<SharedEnvironment>) -> Self {
        // Inherit statistics sender from parent (if any)
        #[cfg(feature = "development")]
        let statistics_sender = parent.get_global_statistics_sender();
        
        SharedEnvironment {
            local_bindings: HashMap::new(),
            local_macros: HashMap::new(),
            parent: Some(parent),
            immutable_cache: None,
            macro_cache: None,
            generation: 0,
            is_frozen: false,
            #[cfg(feature = "development")]
            statistics_sender,
            idempotent_context_cache: HashMap::new(),
            condition_register: ConditionRegister::default(),
            change_tracker: EnvironmentChangeTracker::new(),
        }
    }

    /// Create environment with initial bindings (copy-on-write optimized)
    #[must_use] pub fn with_bindings(bindings: HashMap<String, Value>) -> Self {
        let is_empty = bindings.is_empty();
        SharedEnvironment {
            local_bindings: bindings,
            local_macros: HashMap::new(),
            parent: None,
            immutable_cache: None,
            macro_cache: None,
            generation: 0,
            is_frozen: is_empty, // Empty environments can be frozen immediately
            #[cfg(feature = "development")]
            statistics_sender: None,
            idempotent_context_cache: HashMap::new(),
            condition_register: ConditionRegister::default(),
            change_tracker: EnvironmentChangeTracker::new(),
        }
    }

    /// Create a new shared environment with standard built-in functions
    #[must_use] pub fn with_builtins() -> Self {
        // Load all built-in procedures from the builtins module
        let builtins = crate::builtins::create_builtins();
        let mut env = Self::with_bindings(builtins);
        
        // Freeze the environment since built-ins are immutable
        env.freeze();
        env
    }

    /// Create a new shared environment with standard built-in functions (mutable)
    /// This version does not freeze the environment, allowing user definitions
    #[must_use] pub fn with_builtins_mutable() -> Self {
        // Load all built-in procedures from the builtins module
        let builtins = crate::builtins::create_builtins();
        Self::with_bindings(builtins)
        // Note: Do not freeze, allowing user definitions
    }

    /// Extend environment with new bindings using copy-on-write
    /// If no bindings are provided, returns a clone (shared reference)
    #[must_use] pub fn extend_cow(&self, bindings: Vec<(String, Value)>) -> Self {
        if bindings.is_empty() {
            // No new bindings, return shared reference
            self.clone()
        } else {
            // Create new environment with bindings
            let mut new_bindings = HashMap::with_capacity(bindings.len());
            for (name, value) in bindings {
                new_bindings.insert(name, value);
            }

            SharedEnvironment {
                local_bindings: new_bindings,
                local_macros: HashMap::new(),
                parent: if self.is_empty() {
                    self.parent.clone()
                } else {
                    Some(Rc::new(self.clone()))
                },
                immutable_cache: None,
                macro_cache: None,
                generation: 0,
                is_frozen: false,
                #[cfg(feature = "development")]
                statistics_sender: self.get_global_statistics_sender(),
                idempotent_context_cache: HashMap::new(),
                condition_register: ConditionRegister::default(),
                change_tracker: EnvironmentChangeTracker::new(),
            }
        }
    }

    /// Define a variable in the current environment
    /// Invalidates cache if environment was previously cached
    pub fn define(&mut self, name: String, value: Value) {
        // Cannot modify frozen environment, this indicates a programming error
        assert!(!self.is_frozen, "Attempt to modify frozen environment");

        // Check if this is a redefinition or new definition
        let old_value = self.local_bindings.get(&name).cloned();
        let change_type = if old_value.is_some() {
            ChangeType::VariableUpdated
        } else {
            ChangeType::VariableAdded
        };

        self.local_bindings.insert(name.clone(), value.clone());
        self.invalidate_cache();

        // Record the change
        self.change_tracker.record_change(
            change_type,
            name,
            old_value,
            Some(value),
        );
    }

    /// Set a variable (must already exist in this environment or a parent)
    /// Uses copy-on-write semantics for modifications
    pub fn set(&mut self, name: &str, value: Value) -> Result<()> {
        if self.is_frozen {
            return Err(LambdustError::runtime_error(
                "Cannot modify frozen environment".to_string(),
            ));
        }

        // Record the current value for change tracking
        let old_value = self.local_bindings.get(name).cloned();
        let change_type = if old_value.is_some() {
            ChangeType::VariableUpdated
        } else if self.exists_in_parents(name) {
            ChangeType::VariableUpdated // Copy-on-write update
        } else {
            return Err(LambdustError::runtime_error(format!(
                "Undefined variable: {name}"
            )));
        };

        // Update the binding
        self.local_bindings.insert(name.to_string(), value.clone());
        self.invalidate_cache();

        // Record the change
        self.change_tracker.record_change(
            change_type,
            name.to_string(),
            old_value,
            Some(value),
        );

        Ok(())
    }

    /// Get a variable value with cached lookup optimization
    #[must_use] pub fn get(&self, name: &str) -> Option<Value> {
        // Fast path: check local bindings first
        if let Some(value) = self.local_bindings.get(name) {
            return Some(value.clone());
        }

        // Medium path: check immutable cache
        if let Some(cache) = &self.immutable_cache {
            if let Some(value) = cache.get(name) {
                return Some(value.clone());
            }
        }

        // Slow path: traverse parent chain
        self.lookup_in_parents(name)
    }

    /// Lookup variable in parent environments
    fn lookup_in_parents(&self, name: &str) -> Option<Value> {
        let mut current = self.parent.as_ref();
        while let Some(env) = current {
            if let Some(value) = env.get(name) {
                return Some(value);
            }
            current = env.parent.as_ref();
        }
        None
    }

    /// Check if variable exists anywhere in the environment chain
    #[must_use] pub fn exists(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Check if variable exists in parent environments
    fn exists_in_parents(&self, name: &str) -> bool {
        let mut current = self.parent.as_ref();
        while let Some(env) = current {
            if env.local_bindings.contains_key(name) || env.exists_in_parents(name) {
                return true;
            }
            current = env.parent.as_ref();
        }
        false
    }

    /// Build immutable cache for fast lookups
    /// This flattens the environment chain into a single `HashMap`
    pub fn build_cache(&mut self) {
        if self.immutable_cache.is_some() {
            return; // Cache already exists
        }

        let mut cache = HashMap::new();

        // Collect bindings from parent chain (outer to inner)
        let mut parent_bindings = Vec::new();
        let mut current = self.parent.as_ref();
        while let Some(env) = current {
            parent_bindings.push(&env.local_bindings);
            current = env.parent.as_ref();
        }

        // Add parent bindings (reverse order for correct shadowing)
        for bindings in parent_bindings.iter().rev() {
            cache.extend(bindings.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        // Local bindings override parent bindings
        cache.extend(
            self.local_bindings
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );

        self.immutable_cache = Some(Rc::new(cache));
    }

    /// Invalidate cache when environment changes
    fn invalidate_cache(&mut self) {
        self.immutable_cache = None;
        self.macro_cache = None;
        self.generation += 1;
    }

    /// Freeze environment to make it immutable and shareable
    /// Frozen environments can be safely shared without cloning
    pub fn freeze(&mut self) {
        self.is_frozen = true;
        self.build_cache(); // Build cache before freezing
        self.build_macro_cache(); // Build macro cache before freezing
    }

    /// Check if environment is empty (no local bindings or macros)
    #[must_use] pub fn is_empty(&self) -> bool {
        self.local_bindings.is_empty() && self.local_macros.is_empty()
    }

    /// Check if environment is frozen
    #[must_use] pub fn is_frozen(&self) -> bool {
        self.is_frozen
    }

    /// Get environment depth (distance from root)
    #[must_use] pub fn depth(&self) -> usize {
        match &self.parent {
            Some(parent) => parent.depth() + 1,
            None => 0,
        }
    }

    /// Get total number of bindings in environment chain
    #[must_use] pub fn total_bindings(&self) -> usize {
        let local_count = self.local_bindings.len();
        match &self.parent {
            Some(parent) => local_count + parent.total_bindings(),
            None => local_count,
        }
    }

    /// Get memory usage estimate in bytes
    #[must_use] pub fn memory_usage(&self) -> usize {
        let local_size = self.local_bindings.len()
            * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());

        let macro_size = self.local_macros.len()
            * (std::mem::size_of::<String>() + std::mem::size_of::<Macro>());

        let cache_size = self
            .immutable_cache
            .as_ref()
            .map_or(0, |cache| {
                cache.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>())
            });

        let macro_cache_size = self
            .macro_cache
            .as_ref()
            .map_or(0, |cache| {
                cache.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Macro>())
            });

        let parent_size = self
            .parent
            .as_ref()
            .map_or(0, |_| std::mem::size_of::<Rc<SharedEnvironment>>());

        local_size + macro_size + cache_size + macro_cache_size + parent_size
    }

    /// Create a new child environment (backward compatibility method)
    #[must_use] pub fn extend(&self) -> SharedEnvironment {
        SharedEnvironment::with_parent(Rc::new(self.clone()))
    }

    /// Check if a variable exists in this environment only (not parents)
    #[must_use] pub fn contains(&self, name: &str) -> bool {
        self.local_bindings.contains_key(name)
    }

    /// Get all bindings in the current frame (for debugging)
    #[must_use] pub fn current_bindings(&self) -> HashMap<String, Value> {
        self.local_bindings.clone()
    }

    /// Get the global environment (root of the chain)
    #[must_use] pub fn global(&self) -> Rc<SharedEnvironment> {
        match &self.parent {
            Some(parent) => parent.global(),
            None => Rc::new(self.clone()),
        }
    }

    /// Create a new environment with parameter bindings
    pub fn bind_parameters(
        &self,
        params: &[String],
        args: &[Value],
        variadic: bool,
    ) -> Result<SharedEnvironment> {
        let mut bindings = HashMap::new();

        if variadic {
            // Last parameter collects remaining arguments
            if params.is_empty() {
                return Err(LambdustError::runtime_error(
                    "Variadic function requires at least one parameter".to_string(),
                ));
            }

            let required_params = params.len() - 1;
            if args.len() < required_params {
                return Err(LambdustError::runtime_error(format!(
                    "Function requires at least {} arguments, got {}",
                    required_params,
                    args.len()
                )));
            }

            // Bind required parameters
            for (i, param) in params[..required_params].iter().enumerate() {
                bindings.insert(param.clone(), args[i].clone());
            }

            // Bind variadic parameter to remaining arguments as a list
            let rest_args = args[required_params..].to_vec();
            let rest_list = Value::from_vector(rest_args);
            bindings.insert(params[required_params].clone(), rest_list);
        } else {
            // Fixed arity
            if args.len() != params.len() {
                return Err(LambdustError::runtime_error(format!(
                    "Function requires {} arguments, got {}",
                    params.len(),
                    args.len()
                )));
            }

            for (param, arg) in params.iter().zip(args.iter()) {
                bindings.insert(param.clone(), arg.clone());
            }
        }

        Ok(SharedEnvironment {
            local_bindings: bindings,
            local_macros: HashMap::new(),
            parent: Some(Rc::new(self.clone())),
            immutable_cache: None,
            macro_cache: None,
            generation: 0,
            is_frozen: false,
            #[cfg(feature = "development")]
            statistics_sender: self.get_global_statistics_sender(),
            idempotent_context_cache: HashMap::new(),
            condition_register: ConditionRegister::default(),
            change_tracker: EnvironmentChangeTracker::new(),
        })
    }

    /// Define a macro in the current environment
    pub fn define_macro(&mut self, name: String, macro_def: Macro) {
        assert!(!self.is_frozen, "Cannot define macro in frozen environment");
        
        // Check if this is a redefinition or new definition
        let change_type = if self.local_macros.contains_key(&name) {
            ChangeType::MacroUpdated
        } else {
            ChangeType::MacroAdded
        };

        self.local_macros.insert(name.clone(), macro_def);
        self.invalidate_cache();

        // Record the change (macros don't have simple values, so we store name as value)
        self.change_tracker.record_change(
            change_type,
            name.clone(),
            None, // Old macro value not easily comparable
            Some(Value::Symbol(name)), // Use symbol as placeholder
        );
    }

    /// Get a macro from this environment or a parent
    #[must_use] pub fn get_macro(&self, name: &str) -> Option<Macro> {
        // Check macro cache first
        if let Some(ref cache) = self.macro_cache {
            return cache.get(name).cloned();
        }

        // Try local macros first
        if let Some(macro_def) = self.local_macros.get(name) {
            return Some(macro_def.clone());
        }

        // Try parent environments
        if let Some(ref parent) = self.parent {
            parent.get_macro(name)
        } else {
            None
        }
    }

    /// Check if a macro exists in this environment or a parent
    #[must_use] pub fn has_macro(&self, name: &str) -> bool {
        // Check macro cache first
        if let Some(ref cache) = self.macro_cache {
            return cache.contains_key(name);
        }

        self.local_macros.contains_key(name)
            || self.parent.as_ref().is_some_and(|p| p.has_macro(name))
    }

    /// Build macro cache for fast lookups
    pub fn build_macro_cache(&mut self) {
        if self.macro_cache.is_some() {
            return; // Cache already exists
        }

        let mut cache = HashMap::new();

        // Collect macros from parent chain (outer to inner)
        let mut parent_macros = Vec::new();
        let mut current = self.parent.as_ref();
        while let Some(env) = current {
            parent_macros.push(&env.local_macros);
            current = env.parent.as_ref();
        }

        // Add parent macros (reverse order for correct shadowing)
        for macros in parent_macros.iter().rev() {
            cache.extend(macros.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        // Local macros override parent macros
        cache.extend(
            self.local_macros
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );

        self.macro_cache = Some(Rc::new(cache));
    }

    /// Convert to iterator over all bindings (for debugging)
    #[must_use] pub fn iter_all_bindings(&self) -> HashMap<String, Value> {
        let mut all_bindings = HashMap::new();

        // Collect from parent chain first
        if let Some(parent) = &self.parent {
            all_bindings.extend(parent.iter_all_bindings());
        }

        // Add local bindings (these override parent bindings)
        all_bindings.extend(
            self.local_bindings
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        );

        all_bindings
    }

    /// Send statistics message to the statistics processor (if configured)
    /// This provides decoupled statistics reporting from evaluators through the environment
    /// Only available in development builds
    #[cfg(feature = "development")]
    pub fn send_statistics(&self, message: StatisticsMessage) {
        if let Some(sender) = &self.statistics_sender {
            let _ = sender.send(message); // Ignore send errors to avoid affecting evaluation
        }
    }
    
    /// No-op version for non-development builds
    #[cfg(not(feature = "development"))]
    pub fn send_statistics(&self, _message: ()) {
        // No-op: statistics disabled in production builds
    }

    /// Get the global statistics sender by traversing up the environment chain
    /// Only available in development builds
    #[cfg(feature = "development")]
    pub fn get_global_statistics_sender(&self) -> Option<mpsc::Sender<StatisticsMessage>> {
        if let Some(ref sender) = self.statistics_sender {
            Some(sender.clone())
        } else if let Some(ref parent) = self.parent {
            parent.get_global_statistics_sender()
        } else {
            None
        }
    }
    
    /// No-op version for non-development builds
    #[cfg(not(feature = "development"))]
    #[must_use] pub fn get_global_statistics_sender(&self) -> Option<()> {
        None
    }

    /// Check if statistics reporting is enabled
    #[must_use] pub fn has_statistics_reporting(&self) -> bool {
        #[cfg(feature = "development")]
        {
            self.get_global_statistics_sender().is_some()
        }
        #[cfg(not(feature = "development"))]
        {
            false
        }
    }

    /// Get access to the change tracker
    #[must_use] pub fn change_tracker(&self) -> &EnvironmentChangeTracker {
        &self.change_tracker
    }


    /// Enable or disable change tracking
    pub fn set_change_tracking_enabled(&mut self, enabled: bool) {
        self.change_tracker.set_enabled(enabled);
    }

    /// Get all environment changes since a specific generation
    #[must_use] pub fn get_changes_since(&self, generation: u64) -> Vec<&EnvironmentChange> {
        self.change_tracker.get_changes_since(generation)
    }

    /// Get the current change tracking generation
    #[must_use] pub fn current_change_generation(&self) -> u64 {
        self.change_tracker.current_generation()
    }

    /// Get all changes for a specific variable
    #[must_use] pub fn get_variable_history(&self, variable_name: &str) -> Vec<&EnvironmentChange> {
        self.change_tracker.get_variable_history(variable_name)
    }

    /// Clear change tracking history
    pub fn clear_change_history(&mut self) {
        self.change_tracker.clear_history();
    }

    // ========================================
    // IDEMPOTENT CONTEXT CACHING
    // ========================================

    /// Cache an idempotent execution context
    /// Only caches contexts that are guaranteed to be idempotent (pure functions)
    pub fn cache_idempotent_context(&mut self, expression_hash: String, context: crate::evaluator::ExecutionContext) {
        assert!(!self.is_frozen, "Cannot cache context in frozen environment");
        
        // Only cache if the context is truly idempotent
        if self.is_context_idempotent(&context) {
            self.idempotent_context_cache.insert(expression_hash, context);
        }
    }

    /// Retrieve cached idempotent execution context
    #[must_use] pub fn get_cached_context(&self, expression_hash: &str) -> Option<&crate::evaluator::ExecutionContext> {
        self.idempotent_context_cache.get(expression_hash)
    }

    /// Check if an execution context is truly idempotent
    fn is_context_idempotent(&self, context: &crate::evaluator::ExecutionContext) -> bool {
        // Context is idempotent if:
        // 1. The static analysis indicates it's pure (no side effects)
        // 2. No runtime conditions affect its execution
        // 3. All variable bindings it depends on are immutable in this environment
        
        // Check if the context is marked as pure
        if !context.static_analysis.is_pure {
            return false;
        }

        // Check if any runtime conditions affect this context
        for dependency in &context.static_analysis.dependencies {
            if self.condition_register.affects_context(dependency) {
                return false;
            }
        }

        // Check if all dependencies are immutable
        for dependency in &context.static_analysis.dependencies {
            if let Some(_value) = self.get(dependency) {
                // For now, assume all bound values are immutable
                // In a more sophisticated implementation, we would track mutability
                continue;
            }
            // Dependency not found, context may not be stable
            return false;
        }

        true
    }

    /// Clear the idempotent context cache
    pub fn clear_context_cache(&mut self) {
        self.idempotent_context_cache.clear();
    }

    /// Get cache statistics
    #[must_use] pub fn context_cache_stats(&self) -> ContextCacheStats {
        ContextCacheStats {
            cached_contexts: self.idempotent_context_cache.len(),
            cache_memory_usage: self.idempotent_context_cache.len() * std::mem::size_of::<crate::evaluator::ExecutionContext>(),
            condition_register_size: self.condition_register.runtime_conditions.len(),
        }
    }

    // ========================================
    // CONDITION REGISTER OPERATIONS
    // ========================================

    /// Set a runtime condition that affects execution context validity
    pub fn set_runtime_condition(&mut self, condition: RuntimeCondition) {
        assert!(!self.is_frozen, "Cannot modify condition register in frozen environment");
        
        // Increment generation when conditions change
        self.condition_register.generation += 1;
        
        // If this condition affects idempotency, clear related cached contexts
        if condition.affects_idempotency {
            self.invalidate_affected_contexts(&condition.id);
        }
        
        self.condition_register.runtime_conditions.insert(condition.id.clone(), condition);
    }

    /// Get current value of a runtime condition
    #[must_use] pub fn get_runtime_condition(&self, condition_id: &str) -> Option<&RuntimeCondition> {
        self.condition_register.runtime_conditions.get(condition_id)
    }

    /// Remove a runtime condition
    pub fn remove_runtime_condition(&mut self, condition_id: &str) -> Option<RuntimeCondition> {
        assert!(!self.is_frozen, "Cannot modify condition register in frozen environment");
        
        self.condition_register.generation += 1;
        self.condition_register.runtime_conditions.remove(condition_id)
    }

    /// Invalidate cached contexts affected by a condition change
    fn invalidate_affected_contexts(&mut self, condition_id: &str) {
        self.idempotent_context_cache.retain(|_hash, context| {
            !context.static_analysis.dependencies.iter().any(|dep| dep == condition_id)
        });
    }

    /// Get the current condition register generation
    #[must_use] pub fn condition_register_generation(&self) -> u64 {
        self.condition_register.generation
    }

    // ========================================
    // FUNCTIONAL OPERATIONS FOR COW OPTIMIZATION
    // ========================================

    /// Define a variable and return new environment (functional approach)
    /// This enables Copy-on-Write optimization for Arc<SharedEnvironment>
    #[must_use] pub fn with_definition(mut self, name: String, value: Value) -> Self {
        self.define(name, value);
        self
    }

    /// Set a variable and return new environment (functional approach)
    /// This enables Copy-on-Write optimization for Arc<SharedEnvironment>
    pub fn with_assignment(mut self, name: &str, value: Value) -> Result<Self> {
        self.set(name, value)?;
        Ok(self)
    }

    /// Define a macro and return new environment (functional approach)
    /// This enables Copy-on-Write optimization for Arc<SharedEnvironment>
    #[must_use] pub fn with_macro_definition(mut self, name: String, macro_def: Macro) -> Self {
        self.define_macro(name, macro_def);
        self
    }

    /// Cache context and return new environment (functional approach)
    /// This enables Copy-on-Write optimization for Arc<SharedEnvironment>
    #[must_use] pub fn with_cached_context(mut self, expression_hash: String, context: crate::evaluator::ExecutionContext) -> Self {
        self.cache_idempotent_context(expression_hash, context);
        self
    }

    /// Set runtime condition and return new environment (functional approach)
    /// This enables Copy-on-Write optimization for Arc<SharedEnvironment>
    #[must_use] pub fn with_runtime_condition(mut self, condition: RuntimeCondition) -> Self {
        self.set_runtime_condition(condition);
        self
    }
}

/// Environment wrapper that provides RefCell-based interior mutability
/// 
/// This enables mutation through Rc<Environment> for backward compatibility
/// with older code that expects mutable environments through shared references.
#[derive(Debug)]
pub struct MutableEnvironment {
    inner: RefCell<SharedEnvironment>,
}

impl MutableEnvironment {
    /// Create a new mutable environment
    #[must_use] pub fn new() -> Self {
        MutableEnvironment {
            inner: RefCell::new(SharedEnvironment::new()),
        }
    }

    /// Create a mutable environment with parent
    #[must_use] pub fn with_parent(parent: Rc<MutableEnvironment>) -> Self {
        // Convert parent to shared parent chain
        let shared_parent = parent.inner.borrow().clone();
        MutableEnvironment {
            inner: RefCell::new(SharedEnvironment::with_parent(Rc::new(shared_parent))),
        }
    }

    /// Create a mutable environment with initial bindings
    #[must_use] pub fn with_bindings(bindings: HashMap<String, Value>) -> Self {
        MutableEnvironment {
            inner: RefCell::new(SharedEnvironment::with_bindings(bindings)),
        }
    }

    /// Create a mutable environment with built-ins
    #[must_use] pub fn with_builtins() -> Self {
        MutableEnvironment {
            inner: RefCell::new(SharedEnvironment::with_builtins()),
        }
    }

    /// Create a mutable environment with built-ins (extensible)
    /// This version allows user definitions to be added
    #[must_use] pub fn with_builtins_mutable() -> Self {
        MutableEnvironment {
            inner: RefCell::new(SharedEnvironment::with_builtins_mutable()),
        }
    }

    /// Define a variable (mutable through `RefCell`)
    pub fn define(&self, name: String, value: Value) {
        self.inner.borrow_mut().define(name, value);
    }

    /// Set a variable (mutable through `RefCell`)
    pub fn set(&self, name: &str, value: Value) -> Result<()> {
        self.inner.borrow_mut().set(name, value)
    }

    /// Get a variable value
    #[must_use] pub fn get(&self, name: &str) -> Option<Value> {
        self.inner.borrow().get(name)
    }

    /// Check if variable exists
    #[must_use] pub fn exists(&self, name: &str) -> bool {
        self.inner.borrow().exists(name)
    }

    /// Check if variable exists in current frame only
    #[must_use] pub fn contains(&self, name: &str) -> bool {
        self.inner.borrow().contains(name)
    }

    /// Get environment depth
    #[must_use] pub fn depth(&self) -> usize {
        self.inner.borrow().depth()
    }

    /// Create a child environment
    #[must_use] pub fn extend(&self) -> MutableEnvironment {
        let child_shared = self.inner.borrow().extend();
        MutableEnvironment {
            inner: RefCell::new(child_shared),
        }
    }

    /// Get current bindings (for debugging)
    #[must_use] pub fn current_bindings(&self) -> HashMap<String, Value> {
        self.inner.borrow().current_bindings()
    }

    /// Get global environment
    #[must_use] pub fn global(&self) -> Rc<MutableEnvironment> {
        // For simplicity, just return self for now
        // In a more sophisticated implementation, we'd traverse to root
        Rc::new(MutableEnvironment {
            inner: RefCell::new(self.inner.borrow().clone()),
        })
    }

    /// Bind parameters for function calls
    pub fn bind_parameters(
        &self,
        params: &[String],
        args: &[Value],
        variadic: bool,
    ) -> Result<MutableEnvironment> {
        let child_shared = self.inner.borrow().bind_parameters(params, args, variadic)?;
        Ok(MutableEnvironment {
            inner: RefCell::new(child_shared),
        })
    }

    /// Define a macro
    pub fn define_macro(&self, name: String, macro_def: Macro) {
        self.inner.borrow_mut().define_macro(name, macro_def);
    }

    /// Get a macro
    #[must_use] pub fn get_macro(&self, name: &str) -> Option<Macro> {
        self.inner.borrow().get_macro(name)
    }

    /// Check if macro exists
    #[must_use] pub fn has_macro(&self, name: &str) -> bool {
        self.inner.borrow().has_macro(name)
    }
    
    /// Send statistics message (development builds only)
    #[cfg(feature = "development")]
    pub fn send_statistics(&self, message: StatisticsMessage) {
        self.inner.borrow().send_statistics(message);
    }
    
    /// No-op version for non-development builds
    #[cfg(not(feature = "development"))]
    pub fn send_statistics(&self, _message: ()) {
        // No-op: statistics disabled in production builds
    }

    // ========================================
    // IDEMPOTENT CONTEXT CACHING (MUTABLE INTERFACE)
    // ========================================

    /// Cache an idempotent execution context (mutable through `RefCell`)
    pub fn cache_idempotent_context(&self, expression_hash: String, context: crate::evaluator::ExecutionContext) {
        self.inner.borrow_mut().cache_idempotent_context(expression_hash, context);
    }

    /// Retrieve cached idempotent execution context
    #[must_use] pub fn get_cached_context(&self, expression_hash: &str) -> Option<crate::evaluator::ExecutionContext> {
        self.inner.borrow().get_cached_context(expression_hash).cloned()
    }

    /// Clear the idempotent context cache
    pub fn clear_context_cache(&self) {
        self.inner.borrow_mut().clear_context_cache();
    }

    /// Get cache statistics
    #[must_use] pub fn context_cache_stats(&self) -> ContextCacheStats {
        self.inner.borrow().context_cache_stats()
    }

    /// Set a runtime condition that affects execution context validity
    pub fn set_runtime_condition(&self, condition: RuntimeCondition) {
        self.inner.borrow_mut().set_runtime_condition(condition);
    }

    /// Get current value of a runtime condition
    #[must_use] pub fn get_runtime_condition(&self, condition_id: &str) -> Option<RuntimeCondition> {
        self.inner.borrow().get_runtime_condition(condition_id).cloned()
    }

    /// Remove a runtime condition
    pub fn remove_runtime_condition(&self, condition_id: &str) -> Option<RuntimeCondition> {
        self.inner.borrow_mut().remove_runtime_condition(condition_id)
    }

    /// Get the current condition register generation
    #[must_use] pub fn condition_register_generation(&self) -> u64 {
        self.inner.borrow().condition_register_generation()
    }

    // ========================================
    // CHANGE TRACKING INTERFACE
    // ========================================

    /// Get access to the change tracker
    #[must_use] pub fn change_tracker(&self) -> std::cell::Ref<'_, EnvironmentChangeTracker> {
        std::cell::Ref::map(self.inner.borrow(), |env| env.change_tracker())
    }

    /// Enable or disable change tracking
    pub fn set_change_tracking_enabled(&self, enabled: bool) {
        self.inner.borrow_mut().set_change_tracking_enabled(enabled);
    }

    /// Get all environment changes since a specific generation
    pub fn get_changes_since(&self, generation: u64) -> Vec<EnvironmentChange> {
        self.inner.borrow().get_changes_since(generation)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get the current change tracking generation
    #[must_use] pub fn current_change_generation(&self) -> u64 {
        self.inner.borrow().current_change_generation()
    }

    /// Get all changes for a specific variable
    pub fn get_variable_history(&self, variable_name: &str) -> Vec<EnvironmentChange> {
        self.inner.borrow().get_variable_history(variable_name)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Clear change tracking history
    pub fn clear_change_history(&self) {
        self.inner.borrow_mut().clear_change_history();
    }
}

impl Clone for MutableEnvironment {
    fn clone(&self) -> Self {
        MutableEnvironment {
            inner: RefCell::new(self.inner.borrow().clone()),
        }
    }
}

impl Default for MutableEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for MutableEnvironment {
    fn eq(&self, other: &Self) -> bool {
        // Compare by pointer
        std::ptr::eq(self.inner.as_ptr(), other.inner.as_ptr())
    }
}

impl Default for SharedEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

// ========================================
// OPTIMIZED ENVIRONMENT ARCHITECTURE
// ========================================

/// Optimized environment that eliminates double-wrapping
/// 
/// Uses Arc<SharedEnvironment> directly for better performance
/// by avoiding the RefCell layer when thread-safety is needed.
pub type DirectEnvironment = std::sync::Arc<SharedEnvironment>;

/// Create a new direct environment
/// 
/// Creates a new global environment using the optimized DirectEnvironment
/// architecture for improved performance with shared ownership.
#[must_use] pub fn create_direct_environment() -> DirectEnvironment {
    std::sync::Arc::new(SharedEnvironment::new())
}

/// Create a direct environment with parent
/// 
/// Creates a new environment that inherits from the given parent,
/// enabling efficient environment chaining with copy-on-write semantics.
#[must_use] pub fn create_direct_environment_with_parent(parent: DirectEnvironment) -> DirectEnvironment {
    std::sync::Arc::new(SharedEnvironment::with_parent(std::rc::Rc::new((*parent).clone())))
}

/// Create a direct environment with bindings
/// 
/// Creates a new environment pre-populated with the given bindings
/// for efficient initialization with known variable sets.
#[must_use] pub fn create_direct_environment_with_bindings(bindings: HashMap<String, Value>) -> DirectEnvironment {
    std::sync::Arc::new(SharedEnvironment::with_bindings(bindings))
}

/// Create a direct environment with built-ins
/// 
/// Creates a new environment pre-loaded with all standard R7RS
/// built-in functions and procedures for immediate use.
#[must_use] pub fn create_direct_environment_with_builtins() -> DirectEnvironment {
    std::sync::Arc::new(SharedEnvironment::with_builtins())
}

/// Mutable wrapper for `DirectEnvironment` that uses Copy-on-Write optimization
/// 
/// This provides mutability while maintaining the performance benefits of
/// Arc<SharedEnvironment> through functional updates and COW semantics.
#[derive(Debug, Clone)]
pub struct MutableDirectEnvironment {
    /// Current environment (shared)
    inner: DirectEnvironment,
}

impl MutableDirectEnvironment {
    /// Create a new mutable direct environment
    #[must_use] pub fn new() -> Self {
        Self {
            inner: create_direct_environment(),
        }
    }

    /// Create from existing `DirectEnvironment`
    #[must_use] pub fn from_direct(env: DirectEnvironment) -> Self {
        Self { inner: env }
    }

    /// Create with parent
    #[must_use] pub fn with_parent(parent: DirectEnvironment) -> Self {
        Self {
            inner: create_direct_environment_with_parent(parent),
        }
    }

    /// Create with built-ins
    #[must_use] pub fn with_builtins() -> Self {
        Self {
            inner: create_direct_environment_with_builtins(),
        }
    }

    /// Get the inner `DirectEnvironment` (for read-only access)
    #[must_use] pub fn as_direct(&self) -> &DirectEnvironment {
        &self.inner
    }

    /// Get a cloned `DirectEnvironment` (for sharing)
    #[must_use] pub fn clone_direct(&self) -> DirectEnvironment {
        self.inner.clone()
    }

    /// Define a variable (uses Copy-on-Write optimization)
    pub fn define(&mut self, name: String, value: Value) {
        // Use functional approach to create new environment
        let new_env = (*self.inner).clone().with_definition(name, value);
        self.inner = std::sync::Arc::new(new_env);
    }

    /// Set a variable (uses Copy-on-Write optimization)
    pub fn set(&mut self, name: &str, value: Value) -> Result<()> {
        // Use functional approach to create new environment
        let new_env = (*self.inner).clone().with_assignment(name, value)?;
        self.inner = std::sync::Arc::new(new_env);
        Ok(())
    }

    /// Get a variable value (read-only, no COW needed)
    #[must_use] pub fn get(&self, name: &str) -> Option<Value> {
        self.inner.get(name)
    }

    /// Check if variable exists (read-only, no COW needed)
    #[must_use] pub fn exists(&self, name: &str) -> bool {
        self.inner.exists(name)
    }

    /// Define a macro (uses Copy-on-Write optimization)
    pub fn define_macro(&mut self, name: String, macro_def: Macro) {
        let new_env = (*self.inner).clone().with_macro_definition(name, macro_def);
        self.inner = std::sync::Arc::new(new_env);
    }

    /// Get a macro (read-only, no COW needed)
    #[must_use] pub fn get_macro(&self, name: &str) -> Option<Macro> {
        self.inner.get_macro(name)
    }

    /// Cache idempotent context (uses Copy-on-Write optimization)
    pub fn cache_idempotent_context(&mut self, expression_hash: String, context: crate::evaluator::ExecutionContext) {
        let new_env = (*self.inner).clone().with_cached_context(expression_hash, context);
        self.inner = std::sync::Arc::new(new_env);
    }

    /// Get cached context (read-only, no COW needed)
    #[must_use] pub fn get_cached_context(&self, expression_hash: &str) -> Option<&crate::evaluator::ExecutionContext> {
        self.inner.get_cached_context(expression_hash)
    }

    /// Set runtime condition (uses Copy-on-Write optimization)
    pub fn set_runtime_condition(&mut self, condition: RuntimeCondition) {
        let new_env = (*self.inner).clone().with_runtime_condition(condition);
        self.inner = std::sync::Arc::new(new_env);
    }

    /// Get runtime condition (read-only, no COW needed)
    #[must_use] pub fn get_runtime_condition(&self, condition_id: &str) -> Option<&RuntimeCondition> {
        self.inner.get_runtime_condition(condition_id)
    }

    /// Convert to `MutableEnvironment` for backward compatibility
    #[must_use] pub fn to_mutable_environment(&self) -> MutableEnvironment {
        MutableEnvironment {
            inner: std::cell::RefCell::new((*self.inner).clone()),
        }
    }

    /// Create child environment
    #[must_use] pub fn extend(&self) -> MutableDirectEnvironment {
        Self::with_parent(self.inner.clone())
    }

    /// Get statistics
    #[must_use] pub fn context_cache_stats(&self) -> ContextCacheStats {
        self.inner.context_cache_stats()
    }
}

impl Default for MutableDirectEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

// ========================================
// PERFORMANCE COMPARISON UTILITIES
// ========================================

/// Performance comparison between old and new environment systems
#[cfg(feature = "development")]
pub mod performance_comparison {
    use super::*;
    use std::time::Instant;

    /// Benchmark environment creation performance
    pub fn benchmark_environment_creation(iterations: usize) -> (u64, u64) {
        // Benchmark MutableEnvironment (old system)
        let start = Instant::now();
        for _ in 0..iterations {
            let _env = MutableEnvironment::new();
        }
        let old_duration = start.elapsed().as_nanos() as u64;

        // Benchmark MutableDirectEnvironment (new system)
        let start = Instant::now();
        for _ in 0..iterations {
            let _env = MutableDirectEnvironment::new();
        }
        let new_duration = start.elapsed().as_nanos() as u64;

        (old_duration, new_duration)
    }

    /// Benchmark variable access performance
    pub fn benchmark_variable_access(iterations: usize) -> (u64, u64) {
        // Setup environments
        let mut old_env = MutableEnvironment::new();
        old_env.define("test".to_string(), Value::Number(crate::lexer::SchemeNumber::Integer(42)));

        let mut new_env = MutableDirectEnvironment::new();
        new_env.define("test".to_string(), Value::Number(crate::lexer::SchemeNumber::Integer(42)));

        // Benchmark old system
        let start = Instant::now();
        for _ in 0..iterations {
            let _value = old_env.get("test");
        }
        let old_duration = start.elapsed().as_nanos() as u64;

        // Benchmark new system
        let start = Instant::now();
        for _ in 0..iterations {
            let _value = new_env.get("test");
        }
        let new_duration = start.elapsed().as_nanos() as u64;

        (old_duration, new_duration)
    }

    /// Print performance comparison results
    pub fn print_performance_comparison() {
        println!("🔧 Environment Performance Comparison");
        
        let (old_create, new_create) = benchmark_environment_creation(10000);
        let improvement_create = ((old_create as f64 - new_create as f64) / old_create as f64) * 100.0;
        println!("Creation: Old={old_create}ns, New={new_create}ns, Improvement={improvement_create:.1}%");
        
        let (old_access, new_access) = benchmark_variable_access(100000);
        let improvement_access = ((old_access as f64 - new_access as f64) / old_access as f64) * 100.0;
        println!("Access: Old={old_access}ns, New={new_access}ns, Improvement={improvement_access:.1}%");
    }
}

// Re-export types for API compatibility
// TODO: Gradually migrate to DirectEnvironment for better performance
pub use MutableEnvironment as Environment;

/// Factory for creating environments
/// 
/// Provides unified creation methods for different types of environments
/// while abstracting over the underlying implementation details.
pub struct EnvironmentFactory;

// ========================================
// MIGRATION UTILITIES
// ========================================

/// Convert from old `MutableEnvironment` to new `DirectEnvironment`
/// 
/// Migration utility for converting legacy mutable environments
/// to the optimized direct environment architecture.
#[must_use] pub fn migrate_to_direct_environment(old_env: &MutableEnvironment) -> DirectEnvironment {
    std::sync::Arc::new(old_env.inner.borrow().clone())
}

/// Convert from `DirectEnvironment` to old `MutableEnvironment`
/// 
/// Migration utility for backward compatibility when legacy
/// mutable environment interface is required.
#[must_use] pub fn migrate_from_direct_environment(direct_env: &DirectEnvironment) -> MutableEnvironment {
    MutableEnvironment {
        inner: std::cell::RefCell::new((**direct_env).clone()),
    }
}

/// Create optimized environment with Rc compatibility
/// 
/// This creates a `DirectEnvironment` that can be efficiently converted to Rc when needed
/// for maximum flexibility between different environment usage patterns.
#[must_use] pub fn create_optimized_environment() -> DirectEnvironment {
    create_direct_environment()
}

/// Create optimized environment with built-ins
/// 
/// Creates an optimized environment pre-loaded with R7RS built-ins
/// for high-performance execution scenarios.
#[must_use] pub fn create_optimized_builtins_environment() -> DirectEnvironment {
    create_direct_environment_with_builtins()
}

impl EnvironmentFactory {
    /// Create a new environment (uses COW implementation)
    #[must_use] pub fn new() -> Environment {
        Environment::new()
    }
    
    /// Create environment with parent (uses COW implementation) 
    #[must_use] pub fn with_parent(parent: std::rc::Rc<Environment>) -> Environment {
        Environment::with_parent(parent)
    }

    /// Create environment with initial bindings (uses COW implementation)
    #[must_use] pub fn with_bindings(bindings: HashMap<String, Value>) -> Environment {
        Environment::with_bindings(bindings)
    }
}

/// Trait for unified environment operations
/// 
/// This provides a consistent interface for environment operations
/// across different environment implementations (mutable, direct, etc.).
pub trait EnvironmentOps {
    /// Define a variable in the environment
    fn define(&mut self, name: String, value: Value);

    /// Set a variable (must already exist)
    fn set(&mut self, name: &str, value: Value) -> Result<()>;

    /// Get a variable value
    fn get(&self, name: &str) -> Option<Value>;

    /// Check if variable exists
    fn exists(&self, name: &str) -> bool;

    /// Get environment depth
    fn depth(&self) -> usize;
}

impl EnvironmentOps for Environment {
    fn define(&mut self, name: String, value: Value) {
        Environment::define(self, name, value);
    }

    fn set(&mut self, name: &str, value: Value) -> Result<()> {
        Environment::set(self, name, value)
    }

    fn get(&self, name: &str) -> Option<Value> {
        Environment::get(self, name)
    }

    fn exists(&self, name: &str) -> bool {
        Environment::exists(self, name)
    }

    fn depth(&self) -> usize {
        Environment::depth(self)
    }
}

/// Environment performance benchmark utilities
pub mod benchmarks {
    use super::*;
    use std::time::Instant;

    /// Benchmark environment creation
    #[must_use] pub fn benchmark_environment_creation(iterations: usize) -> u64 {
        // COW environment benchmark (unified default)
        let start = Instant::now();
        for _ in 0..iterations {
            let _env = EnvironmentFactory::new();
        }
        start.elapsed().as_nanos() as u64
    }

    /// Benchmark environment extension
    #[must_use] pub fn benchmark_environment_extension(iterations: usize) -> u64 {
        use crate::lexer::SchemeNumber;
        let bindings = vec![
            ("x".to_string(), Value::Number(SchemeNumber::Integer(1))),
            ("y".to_string(), Value::Number(SchemeNumber::Integer(2))),
            ("z".to_string(), Value::Number(SchemeNumber::Integer(3))),
        ];

        // COW environment benchmark (unified default)
        let start = Instant::now();
        for _ in 0..iterations {
            let env = EnvironmentFactory::new();
            for (name, value) in &bindings {
                env.define(name.clone(), value.clone());
            }
        }
        start.elapsed().as_nanos() as u64
    }

    /// Benchmark variable lookup
    #[must_use] pub fn benchmark_variable_lookup(iterations: usize) -> u64 {
        // Setup environment with some bindings
        let cow_env = EnvironmentFactory::new();

        use crate::lexer::SchemeNumber;
        for i in 0..10 {
            let name = format!("var{i}");
            let value = Value::Number(SchemeNumber::Integer(i64::from(i)));
            cow_env.define(name, value);
        }

        // COW environment benchmark (unified default)
        let start = Instant::now();
        for _ in 0..iterations {
            for i in 0..10 {
                let name = format!("var{i}");
                let _ = cow_env.get(&name);
            }
        }
        start.elapsed().as_nanos() as u64
    }
}


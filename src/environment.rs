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

/// Statistics message sent from evaluators to the environment
/// Only available in development builds for performance analysis
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub enum StatisticsMessage {
    /// Evaluation completed
    EvaluationCompleted {
        expression_type: String,
        execution_time_ns: u64,
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
}

/// Statistics processor interface
/// Only available in development builds for performance analysis
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
/// Only available in development builds for performance analysis
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct StatisticsSummary {
    pub total_evaluations: usize,
    pub total_optimizations: usize,
    pub jit_compilations: usize,
    pub memory_allocations: usize,
    pub continuation_pool_hits: usize,
    pub hot_paths_detected: usize,
    pub average_execution_time_ns: u64,
    pub total_memory_saved: usize,
    pub average_optimization_improvement: f64,
}

/// Basic implementation of statistics processor
/// Only available in development builds for performance analysis
#[cfg(feature = "development")]
#[derive(Debug, Default)]
pub struct BasicStatisticsProcessor {
    total_evaluations: usize,
    total_optimizations: usize,
    jit_compilations: usize,
    memory_allocations: usize,
    continuation_pool_hits: usize,
    hot_paths_detected: usize,
    total_execution_time_ns: u64,
    total_memory_saved: usize,
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
            average_execution_time_ns,
            total_memory_saved: self.total_memory_saved,
            average_optimization_improvement,
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
            statistics_sender: Some(sender),
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
            }
        }
    }

    /// Define a variable in the current environment
    /// Invalidates cache if environment was previously cached
    pub fn define(&mut self, name: String, value: Value) {
        // Cannot modify frozen environment, this indicates a programming error
assert!(!self.is_frozen, "Attempt to modify frozen environment");

        self.local_bindings.insert(name, value);
        self.invalidate_cache();
    }

    /// Set a variable (must already exist in this environment or a parent)
    /// Uses copy-on-write semantics for modifications
    pub fn set(&mut self, name: &str, value: Value) -> Result<()> {
        if self.is_frozen {
            return Err(LambdustError::runtime_error(
                "Cannot modify frozen environment".to_string(),
            ));
        }

        // Check if variable exists in local bindings
        if self.local_bindings.contains_key(name) {
            self.local_bindings.insert(name.to_string(), value);
            self.invalidate_cache();
            return Ok(());
        }

        // Check if variable exists in parent chain
        if self.exists_in_parents(name) {
            // Copy-on-write: bring the binding into local scope
            self.local_bindings.insert(name.to_string(), value);
            self.invalidate_cache();
            return Ok(());
        }

        Err(LambdustError::runtime_error(format!(
            "Undefined variable: {name}"
        )))
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
        })
    }

    /// Define a macro in the current environment
    pub fn define_macro(&mut self, name: String, macro_def: Macro) {
        assert!(!self.is_frozen, "Cannot define macro in frozen environment");
        self.local_macros.insert(name, macro_def);
        self.invalidate_cache();
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
    pub fn get_global_statistics_sender(&self) -> Option<()> {
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
}

/// Environment wrapper that provides RefCell-based interior mutability
/// This enables mutation through Rc<Environment> for backward compatibility
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

// Re-export types for API compatibility
pub use MutableEnvironment as Environment;

/// Factory for creating environments
pub struct EnvironmentFactory;

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
/// This provides a consistent interface for environment operations
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
#[cfg(test)]
pub mod benchmarks {
    use super::*;
    use std::time::Instant;

    /// Benchmark environment creation
    pub fn benchmark_environment_creation(iterations: usize) -> u64 {
        // COW environment benchmark (unified default)
        let start = Instant::now();
        for _ in 0..iterations {
            let _env = EnvironmentFactory::new();
        }
        start.elapsed().as_nanos() as u64
    }

    /// Benchmark environment extension
    pub fn benchmark_environment_extension(iterations: usize) -> u64 {
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
    pub fn benchmark_variable_lookup(iterations: usize) -> u64 {
        // Setup environment with some bindings
        let cow_env = EnvironmentFactory::new();

        use crate::lexer::SchemeNumber;
        for i in 0..10 {
            let name = format!("var{}", i);
            let value = Value::Number(SchemeNumber::Integer(i as i64));
            cow_env.define(name, value);
        }

        // COW environment benchmark (unified default)
        let start = Instant::now();
        for _ in 0..iterations {
            for i in 0..10 {
                let name = format!("var{}", i);
                let _ = cow_env.get(&name);
            }
        }
        start.elapsed().as_nanos() as u64
    }
}


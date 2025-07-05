//! Core data types for the R7RS evaluator
//!
//! This module defines the basic data structures used by the evaluator,
//! including Store, evaluation order, and exception handling.

use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::continuation::DynamicPoint;
use crate::srfi::SrfiRegistry;
use crate::value::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

// Import control flow functions
use crate::ast::Expr;
use crate::evaluator::Continuation;

/// Location identifier for R7RS memory management
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location(usize);

impl Location {
    /// Create a new location
    pub fn new(id: usize) -> Self {
        Location(id)
    }

    /// Get the raw location ID
    pub fn id(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "location:{}", self.0)
    }
}

/// Memory cell for tracking location usage
#[derive(Debug, Clone)]
struct MemoryCell {
    /// The stored value
    value: Value,
    /// Reference count for garbage collection
    ref_count: usize,
    /// Generation for generational GC
    #[allow(dead_code)]
    generation: u32,
    /// Mark for mark-and-sweep GC
    marked: bool,
}

impl MemoryCell {
    fn new(value: Value) -> Self {
        MemoryCell {
            value,
            ref_count: 1,
            generation: 0,
            marked: false,
        }
    }
}

/// Store (memory) for locations with R7RS-compliant memory management
#[derive(Debug, Clone)]
pub struct Store {
    /// Mapping from locations to memory cells
    locations: HashMap<usize, MemoryCell>,
    /// Next available location ID
    next_location: usize,
    /// Total memory usage in bytes (approximation)
    memory_usage: usize,
    /// Maximum memory limit (0 = unlimited)
    memory_limit: usize,
    /// Garbage collection threshold
    gc_threshold: usize,
    /// Current generation for generational GC
    current_generation: u32,
    /// Statistics for monitoring
    pub stats: StoreStatistics,
}

/// Statistics for store monitoring
#[derive(Debug, Clone, Default)]
pub struct StoreStatistics {
    /// Total allocations made
    pub total_allocations: usize,
    /// Total deallocations made
    pub total_deallocations: usize,
    /// Number of GC cycles run
    pub gc_cycles: usize,
    /// Peak memory usage
    pub peak_memory_usage: usize,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    /// Create a new store with default settings
    pub fn new() -> Self {
        Store {
            locations: HashMap::new(),
            next_location: 0,
            memory_usage: 0,
            memory_limit: 0,           // Unlimited by default
            gc_threshold: 1024 * 1024, // 1MB default GC threshold
            current_generation: 0,
            stats: StoreStatistics::default(),
        }
    }

    /// Create a new store with custom memory limit
    pub fn with_memory_limit(memory_limit: usize) -> Self {
        Store {
            locations: HashMap::new(),
            next_location: 0,
            memory_usage: 0,
            memory_limit,
            gc_threshold: memory_limit / 4, // GC when 25% of limit is reached
            current_generation: 0,
            stats: StoreStatistics::default(),
        }
    }

    /// Allocate a new location
    pub fn allocate(&mut self, value: Value) -> Location {
        // Check memory limit before allocation
        if self.should_collect_garbage() {
            self.collect_garbage();
        }

        let loc_id = self.next_location;
        let cell = MemoryCell::new(value.clone());

        // Approximate memory usage calculation
        let value_size = self.estimate_value_size(&value);
        self.memory_usage += value_size;

        self.locations.insert(loc_id, cell);
        self.next_location += 1;
        self.stats.total_allocations += 1;

        // Update peak memory usage
        if self.memory_usage > self.stats.peak_memory_usage {
            self.stats.peak_memory_usage = self.memory_usage;
        }

        Location::new(loc_id)
    }

    /// Get value at location
    pub fn get(&self, location: Location) -> Option<&Value> {
        self.locations.get(&location.id()).map(|cell| &cell.value)
    }

    /// Set value at location
    pub fn set(&mut self, location: Location, value: Value) -> Result<()> {
        let location_id = location.id();

        // Get old value size first
        let old_size = if let Some(cell) = self.locations.get(&location_id) {
            self.estimate_value_size(&cell.value)
        } else {
            return Err(LambdustError::runtime_error(format!(
                "Invalid location: {}",
                location
            )));
        };

        // Calculate new value size
        let new_size = self.estimate_value_size(&value);

        // Now update the cell
        if let Some(cell) = self.locations.get_mut(&location_id) {
            cell.value = value;

            // Update memory usage
            self.memory_usage = self.memory_usage.saturating_sub(old_size) + new_size;

            Ok(())
        } else {
            Err(LambdustError::runtime_error(format!(
                "Invalid location: {}",
                location
            )))
        }
    }

    /// Check if location exists
    pub fn contains(&self, location: Location) -> bool {
        self.locations.contains_key(&location.id())
    }

    /// Increment reference count for a location
    pub fn incref(&mut self, location: Location) -> Result<()> {
        if let Some(cell) = self.locations.get_mut(&location.id()) {
            cell.ref_count += 1;
            Ok(())
        } else {
            Err(LambdustError::runtime_error(format!(
                "Invalid location for incref: {}",
                location
            )))
        }
    }

    /// Decrement reference count for a location
    pub fn decref(&mut self, location: Location) -> Result<()> {
        if let Some(cell) = self.locations.get_mut(&location.id()) {
            if cell.ref_count > 0 {
                cell.ref_count -= 1;
                if cell.ref_count == 0 {
                    // Location can be garbage collected
                    self.deallocate(location);
                }
            }
            Ok(())
        } else {
            Err(LambdustError::runtime_error(format!(
                "Invalid location for decref: {}",
                location
            )))
        }
    }

    /// Deallocate a location
    pub fn deallocate(&mut self, location: Location) {
        if let Some(cell) = self.locations.remove(&location.id()) {
            let value_size = self.estimate_value_size(&cell.value);
            self.memory_usage = self.memory_usage.saturating_sub(value_size);
            self.stats.total_deallocations += 1;
        }
    }

    /// Force garbage collection
    pub fn collect_garbage(&mut self) {
        self.stats.gc_cycles += 1;

        // Mark phase: mark all reachable locations
        self.mark_phase();

        // Sweep phase: deallocate unmarked locations
        self.sweep_phase();

        // Increment generation
        self.current_generation += 1;
    }

    /// Check if garbage collection should be triggered
    fn should_collect_garbage(&self) -> bool {
        if self.memory_limit > 0 && self.memory_usage >= self.memory_limit {
            return true;
        }

        self.memory_usage >= self.gc_threshold
    }

    /// Mark phase of garbage collection
    fn mark_phase(&mut self) {
        // Reset all marks
        for cell in self.locations.values_mut() {
            cell.marked = false;
        }

        // Mark all locations with positive reference count
        for cell in self.locations.values_mut() {
            if cell.ref_count > 0 {
                cell.marked = true;
            }
        }
    }

    /// Sweep phase of garbage collection
    fn sweep_phase(&mut self) {
        let mut to_remove = Vec::new();

        for (loc_id, cell) in &self.locations {
            if !cell.marked {
                to_remove.push(*loc_id);
            }
        }

        for loc_id in to_remove {
            if let Some(cell) = self.locations.remove(&loc_id) {
                let value_size = self.estimate_value_size(&cell.value);
                self.memory_usage = self.memory_usage.saturating_sub(value_size);
                self.stats.total_deallocations += 1;
            }
        }
    }

    /// Estimate memory size of a value (approximation)
    #[allow(clippy::only_used_in_recursion)]
    fn estimate_value_size(&self, value: &Value) -> usize {
        match value {
            Value::Boolean(_) => 1,
            Value::Number(_) => 8,
            Value::Character(_) => 4,
            Value::String(s) => s.len() + 24, // String overhead
            Value::Symbol(s) => s.len() + 24,
            Value::Pair(_) => 32,                 // Approximate size of pair
            Value::Vector(v) => v.len() * 8 + 24, // Vector overhead
            Value::HashTable(_) => 64,            // Approximate hash table overhead
            Value::Procedure(_) => 48,            // Approximate procedure overhead
            Value::Promise(_) => 32,
            Value::Port(_) => 64,     // Port overhead
            Value::External(_) => 48, // External object overhead
            Value::Record(_) => 64,   // Record overhead
            Value::Values(v) => {
                v.iter()
                    .map(|val| self.estimate_value_size(val))
                    .sum::<usize>()
                    + 24
            }
            Value::Continuation(_) => 96, // Continuation overhead
            Value::Nil => 8,
            Value::Undefined => 8,
            Value::Box(_) => 32, // Box overhead
        }
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.memory_usage
    }

    /// Get number of allocated locations
    pub fn location_count(&self) -> usize {
        self.locations.len()
    }

    /// Set memory limit
    pub fn set_memory_limit(&mut self, limit: usize) {
        self.memory_limit = limit;
        self.gc_threshold = if limit > 0 { limit / 4 } else { 1024 * 1024 };
    }

    /// Get memory statistics
    pub fn get_statistics(&self) -> &StoreStatistics {
        &self.stats
    }
}

/// Evaluation order strategy for modeling unspecified order
#[derive(Debug, Clone)]
pub enum EvalOrder {
    /// Left-to-right evaluation
    LeftToRight,
    /// Right-to-left evaluation
    RightToLeft,
    /// Random/unspecified order (for testing compliance)
    Unspecified,
}

/// Exception handler information for exception handling
#[derive(Debug, Clone)]
pub struct ExceptionHandlerInfo {
    /// Handler procedure
    pub handler: Value,
    /// Handler environment
    pub env: Rc<Environment>,
}

/// Location handle trait for abstracting over different memory management strategies
pub trait LocationHandle: Debug {
    /// Get the value at this location
    fn get(&self) -> Option<Value>;
    /// Set the value at this location
    fn set(&self, value: Value) -> Result<()>;
    /// Check if this location is still valid
    fn is_valid(&self) -> bool;
    /// Get location ID for debugging
    fn id(&self) -> usize;
}

/// Traditional location wrapper for legacy store compatibility
#[derive(Debug)]
pub struct TraditionalLocation {
    location: Location,
}

impl LocationHandle for TraditionalLocation {
    fn get(&self) -> Option<Value> {
        // This would need evaluator context - simplified for now
        None
    }

    fn set(&self, _value: Value) -> Result<()> {
        // This would need evaluator context - simplified for now
        Err(LambdustError::runtime_error(
            "Traditional location access requires evaluator context".to_string(),
        ))
    }

    fn is_valid(&self) -> bool {
        true // Traditional locations are always valid while store exists
    }

    fn id(&self) -> usize {
        self.location.id()
    }
}

/// RAII location handle implementation
#[cfg(feature = "raii-store")]
impl LocationHandle for crate::evaluator::raii_store::RaiiLocation {
    fn get(&self) -> Option<Value> {
        self.get()
    }

    fn set(&self, value: Value) -> Result<()> {
        self.set(value)
    }

    fn is_valid(&self) -> bool {
        self.is_valid()
    }

    fn id(&self) -> usize {
        self.id()
    }
}

/// Statistics wrapper to handle different store types
#[derive(Debug, Clone)]
pub enum StoreStatisticsWrapper {
    /// Traditional GC statistics
    Traditional(StoreStatistics),
    /// RAII store statistics
    #[cfg(feature = "raii-store")]
    Raii(crate::evaluator::raii_store::RaiiStoreStatistics),
}

impl StoreStatisticsWrapper {
    /// Get total allocations regardless of store type
    pub fn total_allocations(&self) -> usize {
        match self {
            StoreStatisticsWrapper::Traditional(stats) => stats.total_allocations,
            #[cfg(feature = "raii-store")]
            StoreStatisticsWrapper::Raii(stats) => stats.total_allocations,
        }
    }

    /// Get total deallocations regardless of store type
    pub fn total_deallocations(&self) -> usize {
        match self {
            StoreStatisticsWrapper::Traditional(stats) => stats.total_deallocations,
            #[cfg(feature = "raii-store")]
            StoreStatisticsWrapper::Raii(stats) => stats.total_deallocations,
        }
    }

    /// Get memory usage regardless of store type
    pub fn memory_usage(&self) -> usize {
        match self {
            StoreStatisticsWrapper::Traditional(stats) => stats.peak_memory_usage,
            #[cfg(feature = "raii-store")]
            StoreStatisticsWrapper::Raii(stats) => stats.estimated_memory_usage,
        }
    }
}

/// Memory management strategy for the evaluator
#[derive(Debug)]
pub enum MemoryStrategy {
    /// Traditional GC-based store (current implementation)
    TraditionalGC(Store),
    /// RAII-based store leveraging Rust's ownership model
    #[cfg(feature = "raii-store")]
    RaiiStore(crate::evaluator::raii_store::RaiiStore),
}

impl Default for MemoryStrategy {
    fn default() -> Self {
        MemoryStrategy::TraditionalGC(Store::new())
    }
}

/// Formal evaluator implementing R7RS semantics
#[derive(Debug)]
pub struct Evaluator {
    /// Memory management strategy
    memory_strategy: MemoryStrategy,
    /// Dynamic points stack for dynamic-wind semantics
    dynamic_points: Vec<DynamicPoint>,
    /// Next dynamic point ID
    next_dynamic_point_id: usize,
    /// Evaluation order strategy
    eval_order: EvalOrder,
    /// Global environment
    pub global_env: Rc<Environment>,
    /// Recursion depth counter for stack overflow prevention
    recursion_depth: usize,
    /// Maximum recursion depth
    max_recursion_depth: usize,
    /// Exception handlers stack for exception handling
    exception_handlers: Vec<ExceptionHandlerInfo>,
    /// SRFI registry for module imports
    srfi_registry: SrfiRegistry,
}

impl Evaluator {
    /// Create a new formal evaluator
    pub fn new() -> Self {
        Evaluator {
            memory_strategy: MemoryStrategy::default(),
            dynamic_points: Vec::new(),
            next_dynamic_point_id: 0,
            eval_order: EvalOrder::LeftToRight,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000, // Configurable recursion limit
            exception_handlers: Vec::new(),
            srfi_registry: SrfiRegistry::with_standard_srfis(),
        }
    }

    /// Create evaluator with custom evaluation order
    pub fn with_eval_order(eval_order: EvalOrder) -> Self {
        Evaluator {
            memory_strategy: MemoryStrategy::default(),
            dynamic_points: Vec::new(),
            next_dynamic_point_id: 0,
            eval_order,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000,
            exception_handlers: Vec::new(),
            srfi_registry: SrfiRegistry::with_standard_srfis(),
        }
    }

    /// Get the current evaluation order
    pub fn eval_order(&self) -> &EvalOrder {
        &self.eval_order
    }

    /// Get current recursion depth
    pub fn recursion_depth(&self) -> usize {
        self.recursion_depth
    }

    /// Get maximum recursion depth
    pub fn max_recursion_depth(&self) -> usize {
        self.max_recursion_depth
    }

    /// Get mutable reference to exception handlers
    pub fn exception_handlers_mut(&mut self) -> &mut Vec<ExceptionHandlerInfo> {
        &mut self.exception_handlers
    }

    /// Get reference to exception handlers
    pub fn exception_handlers(&self) -> &[ExceptionHandlerInfo] {
        &self.exception_handlers
    }

    /// Get mutable reference to SRFI registry
    pub fn srfi_registry_mut(&mut self) -> &mut SrfiRegistry {
        &mut self.srfi_registry
    }

    /// Get reference to SRFI registry
    pub fn srfi_registry(&self) -> &SrfiRegistry {
        &self.srfi_registry
    }

    /// Increment recursion depth
    pub fn increment_recursion_depth(&mut self) -> Result<()> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.max_recursion_depth {
            Err(LambdustError::runtime_error(
                "Maximum recursion depth exceeded".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    /// Decrement recursion depth
    pub fn decrement_recursion_depth(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }

    /// Get mutable reference to store (traditional GC only)
    pub fn store_mut(&mut self) -> Result<&mut Store> {
        match &mut self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => Ok(store),
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(_) => Err(LambdustError::runtime_error(
                "Store access not available in RAII mode".to_string(),
            )),
        }
    }

    /// Get reference to store (traditional GC only)
    pub fn store(&self) -> Result<&Store> {
        match &self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => Ok(store),
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(_) => Err(LambdustError::runtime_error(
                "Store access not available in RAII mode".to_string(),
            )),
        }
    }

    /// Allocate a new location in the store
    pub fn allocate(&mut self, value: Value) -> Result<Box<dyn LocationHandle>> {
        match &mut self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => {
                let location = store.allocate(value);
                Ok(Box::new(TraditionalLocation { location }))
            }
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(store) => {
                let location = store.allocate(value);
                Ok(Box::new(location))
            }
        }
    }

    /// Get value from store location (traditional GC only)
    pub fn store_get(&self, location: Location) -> Option<&Value> {
        match &self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => store.get(location),
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(_) => None,
        }
    }

    /// Set value at store location (traditional GC only)
    pub fn store_set(&mut self, location: Location, value: Value) -> Result<()> {
        match &mut self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => store.set(location, value),
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(_) => Err(LambdustError::runtime_error(
                "Direct location access not available in RAII mode".to_string(),
            )),
        }
    }

    /// Check if store contains location (traditional GC only)
    pub fn store_contains(&self, location: Location) -> bool {
        match &self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => store.contains(location),
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(_) => false,
        }
    }

    /// Increment reference count for location (traditional GC only)
    pub fn store_incref(&mut self, location: Location) -> Result<()> {
        match &mut self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => store.incref(location),
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(_) => Ok(()), // No-op for RAII
        }
    }

    /// Decrement reference count for location (traditional GC only)
    pub fn store_decref(&mut self, location: Location) -> Result<()> {
        match &mut self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => store.decref(location),
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(_) => Ok(()), // No-op for RAII
        }
    }

    /// Force garbage collection
    pub fn collect_garbage(&mut self) {
        match &mut self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => store.collect_garbage(),
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(store) => store.manual_cleanup(),
        }
    }

    /// Get store memory statistics
    pub fn store_statistics(&self) -> StoreStatisticsWrapper {
        match &self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => {
                StoreStatisticsWrapper::Traditional(store.get_statistics().clone())
            }
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(store) => StoreStatisticsWrapper::Raii(store.statistics()),
        }
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        match &self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => store.memory_usage(),
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(store) => store.memory_usage(),
        }
    }

    /// Set memory limit for store
    pub fn set_memory_limit(&mut self, limit: usize) {
        match &mut self.memory_strategy {
            MemoryStrategy::TraditionalGC(store) => store.set_memory_limit(limit),
            #[cfg(feature = "raii-store")]
            MemoryStrategy::RaiiStore(store) => store.set_memory_limit(limit),
        }
    }

    /// Create evaluator with custom memory limit
    pub fn with_memory_limit(memory_limit: usize) -> Self {
        Evaluator {
            memory_strategy: MemoryStrategy::TraditionalGC(Store::with_memory_limit(memory_limit)),
            dynamic_points: Vec::new(),
            next_dynamic_point_id: 0,
            eval_order: EvalOrder::LeftToRight,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000,
            exception_handlers: Vec::new(),
            srfi_registry: SrfiRegistry::with_standard_srfis(),
        }
    }

    /// Create evaluator with RAII memory management
    #[cfg(feature = "raii-store")]
    pub fn with_raii_store() -> Self {
        Evaluator {
            memory_strategy: MemoryStrategy::RaiiStore(
                crate::evaluator::raii_store::RaiiStore::new(),
            ),
            dynamic_points: Vec::new(),
            next_dynamic_point_id: 0,
            eval_order: EvalOrder::LeftToRight,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000,
            exception_handlers: Vec::new(),
            srfi_registry: SrfiRegistry::with_standard_srfis(),
        }
    }

    /// Create evaluator with RAII memory management and custom limit
    #[cfg(feature = "raii-store")]
    pub fn with_raii_store_memory_limit(memory_limit: usize) -> Self {
        Evaluator {
            memory_strategy: MemoryStrategy::RaiiStore(
                crate::evaluator::raii_store::RaiiStore::with_memory_limit(memory_limit),
            ),
            dynamic_points: Vec::new(),
            next_dynamic_point_id: 0,
            eval_order: EvalOrder::LeftToRight,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000,
            exception_handlers: Vec::new(),
            srfi_registry: SrfiRegistry::with_standard_srfis(),
        }
    }

    /// Dynamic Points management methods
    /// Push a new dynamic point onto the stack
    pub fn push_dynamic_point(&mut self, before: Option<Value>, after: Option<Value>) -> usize {
        let id = self.next_dynamic_point_id;
        self.next_dynamic_point_id += 1;

        let parent = self.dynamic_points.last().cloned().map(Box::new);
        let dynamic_point = DynamicPoint::new(before, after, parent, id);

        self.dynamic_points.push(dynamic_point);
        id
    }

    /// Pop the top dynamic point from the stack
    pub fn pop_dynamic_point(&mut self) -> Option<DynamicPoint> {
        self.dynamic_points.pop()
    }

    /// Get the current (top) dynamic point
    pub fn current_dynamic_point(&self) -> Option<&DynamicPoint> {
        self.dynamic_points.last()
    }

    /// Get mutable reference to current dynamic point
    pub fn current_dynamic_point_mut(&mut self) -> Option<&mut DynamicPoint> {
        self.dynamic_points.last_mut()
    }

    /// Get all dynamic points
    pub fn dynamic_points(&self) -> &[DynamicPoint] {
        &self.dynamic_points
    }

    /// Get mutable reference to all dynamic points
    pub fn dynamic_points_mut(&mut self) -> &mut Vec<DynamicPoint> {
        &mut self.dynamic_points
    }

    /// Find dynamic point by ID
    pub fn find_dynamic_point(&self, id: usize) -> Option<&DynamicPoint> {
        self.dynamic_points.iter().find(|point| point.id == id)
    }

    /// Find mutable dynamic point by ID
    pub fn find_dynamic_point_mut(&mut self, id: usize) -> Option<&mut DynamicPoint> {
        self.dynamic_points.iter_mut().find(|point| point.id == id)
    }

    /// Get the depth of the dynamic point stack
    pub fn dynamic_point_depth(&self) -> usize {
        self.dynamic_points.len()
    }

    /// Clear all dynamic points (for reset)
    pub fn clear_dynamic_points(&mut self) {
        self.dynamic_points.clear();
        self.next_dynamic_point_id = 0;
    }

    /// Execute before thunks from current point up to target
    pub fn execute_before_thunks_to(&mut self, target_depth: usize) -> Result<()> {
        for i in self.dynamic_points.len()..target_depth {
            if let Some(point) = self.dynamic_points.get(i) {
                if let Some(before_thunk) = &point.before {
                    // Execute before thunk (simplified - would need full evaluator integration)
                    self.call_thunk(before_thunk.clone())?;
                }
            }
        }
        Ok(())
    }

    /// Execute after thunks from current point down to target
    pub fn execute_after_thunks_to(&mut self, target_depth: usize) -> Result<()> {
        for i in (target_depth..self.dynamic_points.len()).rev() {
            if let Some(point) = self.dynamic_points.get(i) {
                if let Some(after_thunk) = &point.after {
                    // Execute after thunk (simplified - would need full evaluator integration)
                    self.call_thunk(after_thunk.clone())?;
                }
            }
        }
        Ok(())
    }

    /// Helper method to call thunk (simplified implementation)
    fn call_thunk(&mut self, thunk: Value) -> Result<Value> {
        // This is a simplified implementation
        // In a full implementation, this would use the evaluator's apply mechanism
        match thunk {
            Value::Procedure(_) => {
                // Would call procedure with no arguments
                Ok(Value::Undefined)
            }
            _ => Err(LambdustError::type_error(
                "Dynamic-wind thunk must be a procedure".to_string(),
            )),
        }
    }

    // Control flow special forms
    /// Evaluate do loop special form
    pub fn eval_do(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_do(self, operands, env, cont)
    }

    /// Evaluate delay special form
    pub fn eval_delay(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_delay(self, operands, env, cont)
    }

    /// Evaluate lazy special form
    pub fn eval_lazy(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_lazy(self, operands, env, cont)
    }

    /// Evaluate force special form
    pub fn eval_force(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_force(self, operands, env, cont)
    }

    /// Evaluate promise? predicate
    pub fn eval_promise_predicate(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_promise_predicate(self, operands, env, cont)
    }

    /// Evaluate call/cc special form
    pub fn eval_call_cc(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_call_cc(self, operands, env, cont)
    }

    /// Evaluate values special form
    pub fn eval_values(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_values(self, operands, env, cont)
    }

    /// Evaluate call-with-values special form
    pub fn eval_call_with_values(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_call_with_values(self, operands, env, cont)
    }

    /// Evaluate dynamic-wind special form
    pub fn eval_dynamic_wind(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_dynamic_wind(self, operands, env, cont)
    }

    /// Evaluate raise special form
    pub fn eval_raise(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_raise(self, operands, env, cont)
    }

    /// Evaluate with-exception-handler special form
    pub fn eval_with_exception_handler(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_with_exception_handler(self, operands, env, cont)
    }

    /// Evaluate guard special form
    pub fn eval_guard(
        &mut self,
        operands: &[Expr],
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        crate::evaluator::control_flow::eval_guard(self, operands, env, cont)
    }

    /// Apply control flow continuation
    pub fn apply_control_flow_continuation(
        &mut self,
        cont: Continuation,
        value: Value,
    ) -> Result<Value> {
        crate::evaluator::control_flow::apply_control_flow_continuation(self, cont, value)
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

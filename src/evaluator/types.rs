//! Core data types for the R7RS evaluator
//!
//! This module defines the basic data structures used by the evaluator,
//! including Store, evaluation order, and exception handling.

use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::continuation::DynamicPoint;
use crate::evaluator::evaluation::{EvalOrder, ExceptionHandlerInfo};
use crate::evaluator::memory::{Location, Store, StoreStatistics};
use crate::srfi::SrfiRegistry;
use crate::value::Value;
use std::fmt::Debug;
use std::rc::Rc;

// Import control flow functions
use crate::ast::Expr;
use crate::evaluator::Continuation;

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
    /// Next continuation reuse ID for tracking continuation reuse
    next_reuse_id: usize,
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
            next_reuse_id: 0,
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
            next_reuse_id: 0,
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

    /// Generate next continuation reuse ID
    pub fn next_reuse_id(&mut self) -> usize {
        self.next_reuse_id += 1;
        self.next_reuse_id - 1
    }

    /// Get current continuation reuse ID without incrementing
    pub fn current_reuse_id(&self) -> usize {
        self.next_reuse_id
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
            next_reuse_id: 0,
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
            next_reuse_id: 0,
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
            next_reuse_id: 0,
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
        self.next_dynamic_point_id += 1;
        let current_id = self.next_dynamic_point_id - 1;

        self.dynamic_points.push(DynamicPoint::new(
            before,
            after,
            self.dynamic_points.last().cloned().map(Box::new),
            current_id,
        ));
        current_id
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

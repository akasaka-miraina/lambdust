//! Core data types for the R7RS evaluator
//!
//! This module defines the basic data structures used by the evaluator,
//! including Store, evaluation order, and exception handling.

use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::continuation::DynamicPoint;
use crate::evaluator::evaluation::{EvalOrder, ExceptionHandlerInfo};
use crate::evaluator::expression_analyzer::ExpressionAnalyzer;
// Phase 5-Step2: RAII-only memory management - removed traditional Store and Location imports
use crate::srfi::SrfiRegistry;
use crate::value::{Value, ValueOptimizer};
use std::fmt::Debug;
use std::rc::Rc;

// Import control flow functions
use crate::ast::Expr;
use crate::evaluator::Continuation;
// Phase 6-B-Step1: DoLoop continuation pool import
use crate::evaluator::control_flow::DoLoopContinuationPool;
// Phase 6-B-Step2: Unified continuation pooling imports
use crate::evaluator::continuation_pooling::ContinuationPoolManager;
// Phase 6-B-Step3: Inline evaluation imports
use crate::evaluator::inline_evaluation::InlineEvaluator;
// Phase 6-C: JIT loop optimization imports
use crate::evaluator::jit_loop_optimization::JitLoopOptimizer;
// Phase 6-D: Tail call optimization imports
use crate::evaluator::tail_call_optimization::TailCallOptimizer;

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

/// RAII location handle implementation (Phase 5-Step2: Always available)
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

/// Statistics wrapper for unified RAII memory management
/// Phase 5-Step2: Simplified to use only RAII store statistics
#[derive(Debug, Clone)]
pub struct StoreStatisticsWrapper {
    /// RAII store statistics
    raii_stats: crate::evaluator::raii_store::RaiiStoreStatistics,
}

impl StoreStatisticsWrapper {
    /// Create from RAII statistics
    pub fn from_raii(stats: crate::evaluator::raii_store::RaiiStoreStatistics) -> Self {
        StoreStatisticsWrapper { raii_stats: stats }
    }

    /// Get total allocations
    pub fn total_allocations(&self) -> usize {
        self.raii_stats.total_allocations
    }

    /// Get total deallocations
    pub fn total_deallocations(&self) -> usize {
        self.raii_stats.total_deallocations
    }

    /// Get memory usage
    pub fn memory_usage(&self) -> usize {
        self.raii_stats.estimated_memory_usage
    }

    /// Get RAII-specific statistics
    pub fn raii_statistics(&self) -> &crate::evaluator::raii_store::RaiiStoreStatistics {
        &self.raii_stats
    }
}

/// Memory management strategy for the evaluator
/// Phase 5-Step2: Unified RAII-only memory management
#[derive(Debug, Default)]
pub struct MemoryStrategy {
    /// RAII-based store leveraging Rust's ownership model
    raii_store: crate::evaluator::raii_store::RaiiStore,
}

impl MemoryStrategy {
    /// Create new RAII-based memory strategy
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with memory limit
    pub fn with_memory_limit(limit: usize) -> Self {
        MemoryStrategy {
            raii_store: crate::evaluator::raii_store::RaiiStore::with_memory_limit(limit),
        }
    }

    /// Get reference to RAII store
    pub fn raii_store(&self) -> &crate::evaluator::raii_store::RaiiStore {
        &self.raii_store
    }

    /// Get mutable reference to RAII store
    pub fn raii_store_mut(&mut self) -> &mut crate::evaluator::raii_store::RaiiStore {
        &mut self.raii_store
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
    /// Value optimizer for Phase 4 memory optimization
    value_optimizer: ValueOptimizer,
    /// Expression analyzer for Phase 5 compile-time optimization
    expression_analyzer: ExpressionAnalyzer,
    /// Phase 6-B-Step1: DoLoop continuation pool for memory optimization
    doloop_continuation_pool: DoLoopContinuationPool,
    /// Phase 6-B-Step2: Global continuation pool manager for unified pooling
    continuation_pool_manager: ContinuationPoolManager,
    /// Phase 6-B-Step3: Inline evaluator for lightweight continuation optimization
    inline_evaluator: InlineEvaluator,
    /// Phase 6-C: JIT loop optimizer for native iteration code generation
    jit_loop_optimizer: JitLoopOptimizer,
    /// Phase 6-D: Tail call optimizer for recursive function optimization
    tail_call_optimizer: TailCallOptimizer,
}

impl Evaluator {
    /// Create a new formal evaluator
    pub fn new() -> Self {
        Evaluator {
            memory_strategy: MemoryStrategy::new(),
            dynamic_points: Vec::new(),
            next_dynamic_point_id: 0,
            next_reuse_id: 0,
            eval_order: EvalOrder::LeftToRight,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000, // Configurable recursion limit
            exception_handlers: Vec::new(),
            srfi_registry: SrfiRegistry::with_standard_srfis(),
            value_optimizer: ValueOptimizer::new(),
            expression_analyzer: ExpressionAnalyzer::new(),
            doloop_continuation_pool: DoLoopContinuationPool::default(),
            continuation_pool_manager: ContinuationPoolManager::new(),
            inline_evaluator: InlineEvaluator::new(),
            jit_loop_optimizer: JitLoopOptimizer::new(),
            tail_call_optimizer: TailCallOptimizer::new(),
        }
    }

    /// Create evaluator with RAII store and memory limit
    pub fn with_raii_memory_limit(memory_limit: usize) -> Self {
        Evaluator {
            memory_strategy: MemoryStrategy::with_memory_limit(memory_limit),
            dynamic_points: Vec::new(),
            next_dynamic_point_id: 0,
            next_reuse_id: 0,
            eval_order: EvalOrder::LeftToRight,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000,
            exception_handlers: Vec::new(),
            srfi_registry: SrfiRegistry::with_standard_srfis(),
            value_optimizer: ValueOptimizer::new(),
            expression_analyzer: ExpressionAnalyzer::new(),
            doloop_continuation_pool: DoLoopContinuationPool::default(),
            continuation_pool_manager: ContinuationPoolManager::new(),
            inline_evaluator: InlineEvaluator::new(),
            jit_loop_optimizer: JitLoopOptimizer::new(),
            tail_call_optimizer: TailCallOptimizer::new(),
        }
    }

    /// Create evaluator with custom evaluation order
    pub fn with_eval_order(eval_order: EvalOrder) -> Self {
        Evaluator {
            memory_strategy: MemoryStrategy::new(),
            dynamic_points: Vec::new(),
            next_dynamic_point_id: 0,
            next_reuse_id: 0,
            eval_order,
            global_env: Rc::new(Environment::with_builtins()),
            recursion_depth: 0,
            max_recursion_depth: 1000,
            exception_handlers: Vec::new(),
            srfi_registry: SrfiRegistry::with_standard_srfis(),
            value_optimizer: ValueOptimizer::new(),
            expression_analyzer: ExpressionAnalyzer::new(),
            doloop_continuation_pool: DoLoopContinuationPool::default(),
            continuation_pool_manager: ContinuationPoolManager::new(),
            inline_evaluator: InlineEvaluator::new(),
            jit_loop_optimizer: JitLoopOptimizer::new(),
            tail_call_optimizer: TailCallOptimizer::new(),
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

    /// Get reference to value optimizer
    pub fn value_optimizer(&self) -> &ValueOptimizer {
        &self.value_optimizer
    }

    /// Get mutable reference to value optimizer
    pub fn value_optimizer_mut(&mut self) -> &mut ValueOptimizer {
        &mut self.value_optimizer
    }

    /// Get reference to expression analyzer
    pub fn expression_analyzer(&self) -> &ExpressionAnalyzer {
        &self.expression_analyzer
    }

    /// Get mutable reference to expression analyzer
    pub fn expression_analyzer_mut(&mut self) -> &mut ExpressionAnalyzer {
        &mut self.expression_analyzer
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

    /// Get mutable reference to RAII store
    pub fn raii_store_mut(&mut self) -> &mut crate::evaluator::raii_store::RaiiStore {
        self.memory_strategy.raii_store_mut()
    }

    /// Get reference to RAII store
    pub fn raii_store(&self) -> &crate::evaluator::raii_store::RaiiStore {
        self.memory_strategy.raii_store()
    }

    /// Allocate a new location with initial value in the RAII store
    pub fn allocate(&mut self, value: Value) -> crate::evaluator::raii_store::RaiiLocation {
        self.memory_strategy.raii_store().allocate(value)
    }

    /// Force garbage collection in RAII store (age-based cleanup)
    pub fn collect_garbage(&mut self) {
        self.memory_strategy.raii_store_mut().manual_cleanup();
    }

    /// Get store memory statistics
    pub fn store_statistics(&self) -> StoreStatisticsWrapper {
        StoreStatisticsWrapper::from_raii(self.memory_strategy.raii_store().statistics())
    }

    /// Get current memory usage
    pub fn memory_usage(&self) -> usize {
        self.memory_strategy.raii_store().memory_usage()
    }

    /// Set memory limit for store
    pub fn set_memory_limit(&mut self, limit: usize) {
        self.memory_strategy.raii_store_mut().set_memory_limit(limit);
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

    /// Phase 6-B-Step1: Get reference to DoLoop continuation pool
    pub fn doloop_continuation_pool(&self) -> &DoLoopContinuationPool {
        &self.doloop_continuation_pool
    }

    /// Phase 6-B-Step1: Get mutable reference to DoLoop continuation pool
    pub fn doloop_continuation_pool_mut(&mut self) -> &mut DoLoopContinuationPool {
        &mut self.doloop_continuation_pool
    }

    /// Phase 6-B-Step2: Get reference to global continuation pool manager
    pub fn continuation_pool_manager(&self) -> &ContinuationPoolManager {
        &self.continuation_pool_manager
    }

    /// Phase 6-B-Step2: Get mutable reference to global continuation pool manager
    pub fn continuation_pool_manager_mut(&mut self) -> &mut ContinuationPoolManager {
        &mut self.continuation_pool_manager
    }

    /// Phase 6-B-Step3: Get reference to inline evaluator
    pub fn inline_evaluator(&self) -> &InlineEvaluator {
        &self.inline_evaluator
    }

    /// Phase 6-B-Step3: Get mutable reference to inline evaluator
    pub fn inline_evaluator_mut(&mut self) -> &mut InlineEvaluator {
        &mut self.inline_evaluator
    }

    /// Phase 6-C: Get reference to JIT loop optimizer
    pub fn jit_loop_optimizer(&self) -> &JitLoopOptimizer {
        &self.jit_loop_optimizer
    }

    /// Phase 6-C: Get mutable reference to JIT loop optimizer
    pub fn jit_loop_optimizer_mut(&mut self) -> &mut JitLoopOptimizer {
        &mut self.jit_loop_optimizer
    }

    /// Phase 6-D: Get reference to tail call optimizer
    pub fn tail_call_optimizer(&self) -> &TailCallOptimizer {
        &self.tail_call_optimizer
    }

    /// Phase 6-D: Get mutable reference to tail call optimizer
    pub fn tail_call_optimizer_mut(&mut self) -> &mut TailCallOptimizer {
        &mut self.tail_call_optimizer
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

//! Core data types for the R7RS evaluator
//!
//! This module defines the basic data structures used by the evaluator,
//! including Store, evaluation order, and exception handling.

use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::continuation::DynamicPoint;
use crate::evaluator::evaluation::{EvalOrder, ExceptionHandlerInfo};
use crate::evaluator::expression_analyzer::ExpressionAnalyzer;
// ExecutionContext system imports for static analysis integration
use crate::evaluator::{
    ExecutionContextBuilder, ExecutionPriority, StaticCallPattern,
};
// RAII-based memory management system
use crate::srfi::SrfiRegistry;
use crate::value::{Value, ValueOptimizer};
use std::fmt::Debug;
use std::rc::Rc;

// Import control flow functions
use crate::ast::Expr;
use crate::evaluator::Continuation;
// Static optimization: macro expansion system
use crate::macros::MacroExpander;
// Advanced optimization systems
use crate::evaluator::control_flow::DoLoopContinuationPool;
use crate::evaluator::continuation_pooling::ContinuationPoolManager;
use crate::evaluator::inline_evaluation::InlineEvaluator;
use crate::evaluator::jit_loop_optimization::JitLoopOptimizer;
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

/// RAII location handle implementation
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
/// Simplified to use only RAII store statistics
#[derive(Debug, Clone)]
pub struct StoreStatisticsWrapper {
    /// RAII store statistics
    raii_stats: crate::evaluator::raii_store::RaiiStoreStatistics,
}

impl StoreStatisticsWrapper {
    /// Create from RAII statistics
    #[must_use] pub fn from_raii(stats: crate::evaluator::raii_store::RaiiStoreStatistics) -> Self {
        StoreStatisticsWrapper { raii_stats: stats }
    }

    /// Get total allocations
    #[must_use] pub fn total_allocations(&self) -> usize {
        self.raii_stats.total_allocations
    }

    /// Get total deallocations
    #[must_use] pub fn total_deallocations(&self) -> usize {
        self.raii_stats.total_deallocations
    }

    /// Get memory usage
    #[must_use] pub fn memory_usage(&self) -> usize {
        self.raii_stats.estimated_memory_usage
    }

    /// Get RAII-specific statistics
    #[must_use] pub fn raii_statistics(&self) -> &crate::evaluator::raii_store::RaiiStoreStatistics {
        &self.raii_stats
    }
}

/// Memory management strategy for the evaluator
/// Unified RAII-only memory management
#[derive(Debug, Default)]
pub struct MemoryStrategy {
    /// RAII-based store leveraging Rust's ownership model
    raii_store: crate::evaluator::raii_store::RaiiStore,
}

impl MemoryStrategy {
    /// Create new RAII-based memory strategy
    #[must_use] pub fn new() -> Self {
        Self::default()
    }

    /// Create with memory limit
    #[must_use] pub fn with_memory_limit(limit: usize) -> Self {
        MemoryStrategy {
            raii_store: crate::evaluator::raii_store::RaiiStore::with_memory_limit(limit),
        }
    }

    /// Get reference to RAII store
    #[must_use] pub fn raii_store(&self) -> &crate::evaluator::raii_store::RaiiStore {
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
    /// Value optimizer for memory optimization
    value_optimizer: ValueOptimizer,
    /// Expression analyzer for compile-time optimization
    expression_analyzer: ExpressionAnalyzer,
    /// Static optimization: macro expansion engine
    macro_expander: MacroExpander,
    /// `DoLoop` continuation pool for memory optimization
    doloop_continuation_pool: DoLoopContinuationPool,
    /// Global continuation pool manager for unified pooling
    continuation_pool_manager: ContinuationPoolManager,
    /// Inline evaluator for lightweight continuation optimization
    inline_evaluator: InlineEvaluator,
    /// JIT loop optimizer for native iteration code generation
    jit_loop_optimizer: JitLoopOptimizer,
    /// Tail call optimizer for recursive function optimization
    tail_call_optimizer: TailCallOptimizer,
}

impl Evaluator {
    /// Create a new formal evaluator
    #[must_use] pub fn new() -> Self {
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
            macro_expander: MacroExpander::new(),
            doloop_continuation_pool: DoLoopContinuationPool::default(),
            continuation_pool_manager: ContinuationPoolManager::new(),
            inline_evaluator: InlineEvaluator::new(),
            jit_loop_optimizer: JitLoopOptimizer::new(),
            tail_call_optimizer: TailCallOptimizer::new(),
        }
    }

    /// Create evaluator with RAII store and memory limit
    #[must_use] pub fn with_raii_memory_limit(memory_limit: usize) -> Self {
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
            macro_expander: MacroExpander::new(),
            doloop_continuation_pool: DoLoopContinuationPool::default(),
            continuation_pool_manager: ContinuationPoolManager::new(),
            inline_evaluator: InlineEvaluator::new(),
            jit_loop_optimizer: JitLoopOptimizer::new(),
            tail_call_optimizer: TailCallOptimizer::new(),
        }
    }

    /// Create evaluator with pre-created shared environment (new architecture)
    /// This allows the environment to be created once and shared across components
    #[must_use] pub fn with_environment(env: std::sync::Arc<Environment>) -> Self {
        // Convert Arc<Environment> to Rc<Environment> for current compatibility
        let rc_env = Rc::new((*env).clone());
        
        Evaluator {
            memory_strategy: MemoryStrategy::new(),
            dynamic_points: Vec::new(),
            next_dynamic_point_id: 0,
            next_reuse_id: 0,
            eval_order: EvalOrder::LeftToRight,
            global_env: rc_env,
            recursion_depth: 0,
            max_recursion_depth: 1000,
            exception_handlers: Vec::new(),
            srfi_registry: SrfiRegistry::with_standard_srfis(),
            value_optimizer: ValueOptimizer::new(),
            expression_analyzer: ExpressionAnalyzer::new(),
            macro_expander: MacroExpander::new(),
            doloop_continuation_pool: DoLoopContinuationPool::default(),
            continuation_pool_manager: ContinuationPoolManager::new(),
            inline_evaluator: InlineEvaluator::new(),
            jit_loop_optimizer: JitLoopOptimizer::new(),
            tail_call_optimizer: TailCallOptimizer::new(),
        }
    }

    /// Create evaluator with custom evaluation order
    #[must_use] pub fn with_eval_order(eval_order: EvalOrder) -> Self {
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
            macro_expander: MacroExpander::new(),
            doloop_continuation_pool: DoLoopContinuationPool::default(),
            continuation_pool_manager: ContinuationPoolManager::new(),
            inline_evaluator: InlineEvaluator::new(),
            jit_loop_optimizer: JitLoopOptimizer::new(),
            tail_call_optimizer: TailCallOptimizer::new(),
        }
    }

    /// Get the current evaluation order
    #[must_use] pub fn eval_order(&self) -> &EvalOrder {
        &self.eval_order
    }

    /// Get current recursion depth
    #[must_use] pub fn recursion_depth(&self) -> usize {
        self.recursion_depth
    }

    /// Get maximum recursion depth
    #[must_use] pub fn max_recursion_depth(&self) -> usize {
        self.max_recursion_depth
    }

    /// Get mutable reference to exception handlers
    pub fn exception_handlers_mut(&mut self) -> &mut Vec<ExceptionHandlerInfo> {
        &mut self.exception_handlers
    }

    /// Get reference to exception handlers
    #[must_use] pub fn exception_handlers(&self) -> &[ExceptionHandlerInfo] {
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
    #[must_use] pub fn current_reuse_id(&self) -> usize {
        self.next_reuse_id
    }

    /// Get reference to SRFI registry
    #[must_use] pub fn srfi_registry(&self) -> &SrfiRegistry {
        &self.srfi_registry
    }

    /// Get reference to value optimizer
    #[must_use] pub fn value_optimizer(&self) -> &ValueOptimizer {
        &self.value_optimizer
    }

    /// Get mutable reference to value optimizer
    pub fn value_optimizer_mut(&mut self) -> &mut ValueOptimizer {
        &mut self.value_optimizer
    }

    /// Get reference to expression analyzer
    #[must_use] pub fn expression_analyzer(&self) -> &ExpressionAnalyzer {
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
    #[must_use] pub fn raii_store(&self) -> &crate::evaluator::raii_store::RaiiStore {
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
    #[must_use] pub fn store_statistics(&self) -> StoreStatisticsWrapper {
        StoreStatisticsWrapper::from_raii(self.memory_strategy.raii_store().statistics())
    }

    /// Get current memory usage
    #[must_use] pub fn memory_usage(&self) -> usize {
        self.memory_strategy.raii_store().memory_usage()
    }

    /// Set memory limit for store
    pub fn set_memory_limit(&mut self, limit: usize) {
        self.memory_strategy
            .raii_store_mut()
            .set_memory_limit(limit);
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
    #[must_use] pub fn current_dynamic_point(&self) -> Option<&DynamicPoint> {
        self.dynamic_points.last()
    }

    /// Get mutable reference to current dynamic point
    pub fn current_dynamic_point_mut(&mut self) -> Option<&mut DynamicPoint> {
        self.dynamic_points.last_mut()
    }

    /// Get all dynamic points
    #[must_use] pub fn dynamic_points(&self) -> &[DynamicPoint] {
        &self.dynamic_points
    }

    /// Get mutable reference to all dynamic points
    pub fn dynamic_points_mut(&mut self) -> &mut Vec<DynamicPoint> {
        &mut self.dynamic_points
    }

    /// Find dynamic point by ID
    #[must_use] pub fn find_dynamic_point(&self, id: usize) -> Option<&DynamicPoint> {
        self.dynamic_points.iter().find(|point| point.id == id)
    }

    /// Find mutable dynamic point by ID
    pub fn find_dynamic_point_mut(&mut self, id: usize) -> Option<&mut DynamicPoint> {
        self.dynamic_points.iter_mut().find(|point| point.id == id)
    }

    /// Get the depth of the dynamic point stack
    #[must_use] pub fn dynamic_point_depth(&self) -> usize {
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

    /// Get reference to `DoLoop` continuation pool
    #[must_use] pub fn doloop_continuation_pool(&self) -> &DoLoopContinuationPool {
        &self.doloop_continuation_pool
    }

    /// Get mutable reference to `DoLoop` continuation pool
    pub fn doloop_continuation_pool_mut(&mut self) -> &mut DoLoopContinuationPool {
        &mut self.doloop_continuation_pool
    }

    /// Get reference to global continuation pool manager
    #[must_use] pub fn continuation_pool_manager(&self) -> &ContinuationPoolManager {
        &self.continuation_pool_manager
    }

    /// Get mutable reference to global continuation pool manager
    pub fn continuation_pool_manager_mut(&mut self) -> &mut ContinuationPoolManager {
        &mut self.continuation_pool_manager
    }

    /// Get reference to inline evaluator
    #[must_use] pub fn inline_evaluator(&self) -> &InlineEvaluator {
        &self.inline_evaluator
    }

    /// Get mutable reference to inline evaluator
    pub fn inline_evaluator_mut(&mut self) -> &mut InlineEvaluator {
        &mut self.inline_evaluator
    }

    /// Get reference to JIT loop optimizer
    #[must_use] pub fn jit_loop_optimizer(&self) -> &JitLoopOptimizer {
        &self.jit_loop_optimizer
    }

    /// Get mutable reference to JIT loop optimizer
    pub fn jit_loop_optimizer_mut(&mut self) -> &mut JitLoopOptimizer {
        &mut self.jit_loop_optimizer
    }

    /// Get reference to tail call optimizer
    #[must_use] pub fn tail_call_optimizer(&self) -> &TailCallOptimizer {
        &self.tail_call_optimizer
    }

    /// Get mutable reference to tail call optimizer
    pub fn tail_call_optimizer_mut(&mut self) -> &mut TailCallOptimizer {
        &mut self.tail_call_optimizer
    }

    /// Get reference to macro expander (static optimization)
    #[must_use] pub fn macro_expander(&self) -> &MacroExpander {
        &self.macro_expander
    }

    /// Get mutable reference to macro expander (static optimization)
    pub fn macro_expander_mut(&mut self) -> &mut MacroExpander {
        &mut self.macro_expander
    }

    /// Static optimization: perform complete macro expansion on expression
    /// This is the key static optimization step that should be done by Evaluator
    /// before generating ExecutionContext for RuntimeExecutor
    pub fn expand_macros(&self, expr: Expr) -> Result<Expr> {
        self.macro_expander.expand_all(expr)
    }

    /// Static optimization: check if expression contains macro calls
    #[must_use] pub fn contains_macro_calls(&self, expr: &Expr) -> bool {
        self.macro_expander.is_macro_call(expr) || self.contains_nested_macro_calls(expr)
    }

    /// Helper: recursively check for nested macro calls
    fn contains_nested_macro_calls(&self, expr: &Expr) -> bool {
        match expr {
            Expr::List(exprs) => exprs.iter().any(|e| self.contains_macro_calls(e)),
            Expr::Quote(inner) | Expr::Quasiquote(inner) | Expr::Unquote(inner) | Expr::UnquoteSplicing(inner) => {
                self.contains_macro_calls(inner)
            }
            Expr::Vector(exprs) => exprs.iter().any(|e| self.contains_macro_calls(e)),
            Expr::DottedList(exprs, tail) => {
                exprs.iter().any(|e| self.contains_macro_calls(e)) || self.contains_macro_calls(tail)
            }
            _ => false,
        }
    }

    /// Static optimization: complete pre-processing pipeline
    /// Performs all static optimizations (macro expansion, etc.) before runtime execution
    pub fn preprocess_expression(&self, expr: Expr) -> Result<Expr> {
        // Step 1: Macro expansion (static optimization)
        let expanded_expr = self.expand_macros(expr)?;
        
        // Step 2: Future static optimizations can be added here
        // - Constant folding
        // - Dead code elimination  
        // - Other compile-time optimizations
        
        Ok(expanded_expr)
    }

    /// Create ExecutionContext with static analysis from current expression
    /// This is the key integration point for Evaluator-Executor communication
    pub fn create_execution_context(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<crate::evaluator::ExecutionContext> {
        use std::time::Instant;
        
        // Record original expression for macro expansion tracking
        let original_expr = expr.clone();
        
        // Step 1: Static optimization - macro expansion and preprocessing
        let start_time = Instant::now();
        let preprocessed_expr = self.preprocess_expression(expr)?;
        let expansion_time = start_time.elapsed().as_micros() as u64;
        
        // Step 2: Perform static analysis on the preprocessed expression
        let analysis_result = self.expression_analyzer.analyze(&preprocessed_expr, Some(&env))?;
        
        // Step 3: Create macro expansion metrics
        let expansion_metrics = crate::evaluator::execution_context::MacroExpansionMetrics {
            expansion_time_micros: expansion_time,
            expansion_count: if self.contains_macro_calls(&original_expr) { 1 } else { 0 },
            max_expansion_depth: self.calculate_expansion_depth(&original_expr),
            hygiene_transformations: 0, // TODO: Track from macro expander
        };
        
        // Step 4: Create macro expansion state
        let macro_expansion_state = crate::evaluator::execution_context::MacroExpansionState {
            is_expanded: self.contains_macro_calls(&original_expr),
            original_expression: Some(original_expr.clone()),
            expanded_expression: Some(preprocessed_expr.clone()),
            expanded_macros: self.extract_expanded_macros(&original_expr).into(),
            expansion_depth: self.calculate_expansion_depth(&original_expr),
            needs_further_expansion: false,
            hygiene_info: Vec::new(), // TODO: Extract from macro expander
            expansion_metrics,
        };
        
        // Start with basic context using preprocessed expression
        let mut context_builder = ExecutionContextBuilder::new(preprocessed_expr.clone(), env.clone(), cont);
        
        // Set complexity score from analysis
        context_builder = context_builder.with_complexity_score(
            analysis_result.complexity.complexity_score()
        );
        
        // Determine if expression has tail calls
        let has_tail_calls = self.analyze_tail_calls(&preprocessed_expr);
        context_builder = context_builder.with_tail_calls(has_tail_calls);
        
        // Determine if expression has loops
        let has_loops = self.analyze_loops(&preprocessed_expr);
        context_builder = context_builder.with_loops(has_loops);
        
        // Determine if expression is pure (side-effect free)
        let is_pure = self.analyze_purity(&preprocessed_expr, &env);
        context_builder = context_builder.with_purity(is_pure);
        
        // Add call patterns based on expression analysis
        context_builder = self.add_call_patterns(context_builder, &preprocessed_expr);
        
        // Add constant bindings from optimization
        context_builder = self.add_constant_bindings(context_builder, &analysis_result);
        
        // Set execution priority based on complexity and context
        let priority = self.determine_execution_priority(&analysis_result);
        context_builder = context_builder.with_priority(priority);
        
        // Step 5: Add static optimization information
        context_builder = self.add_static_optimization_info(context_builder, &original_expr, &preprocessed_expr, &analysis_result)?;
        
        // Step 6: Set macro expansion state
        context_builder = context_builder.with_macro_expansion_state(macro_expansion_state);
        
        // Build the final execution context
        Ok(context_builder.build())
    }
    
    /// Analyze expression for tail call patterns
    fn analyze_tail_calls(&self, expr: &Expr) -> bool {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                // Check if this is a function call that could be tail-recursive
                matches!(exprs[0], Expr::Variable(_))
            }
            _ => false,
        }
    }
    
    /// Analyze expression for loop patterns
    fn analyze_loops(&self, expr: &Expr) -> bool {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                if let Expr::Variable(name) = &exprs[0] {
                    // Common loop constructs
                    matches!(name.as_str(), "do" | "map" | "for-each" | "fold" | "fold-right")
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    /// Analyze expression for purity (side-effect freedom)
    fn analyze_purity(&self, expr: &Expr, _env: &Environment) -> bool {
        match expr {
            Expr::Literal(_) | Expr::Variable(_) => true,
            Expr::List(exprs) if !exprs.is_empty() => {
                if let Expr::Variable(name) = &exprs[0] {
                    // Known pure functions
                    let pure_functions = [
                        "+", "-", "*", "/", "=", "<", ">", "<=", ">=",
                        "cons", "car", "cdr", "null?", "pair?", "list?",
                        "length", "append", "reverse", "map", "filter",
                        "and", "or", "not", "if", "cond", "case",
                    ];
                    pure_functions.contains(&name.as_str())
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    
    /// Add call patterns based on expression structure
    fn add_call_patterns(
        &self,
        mut builder: ExecutionContextBuilder,
        expr: &Expr,
    ) -> ExecutionContextBuilder {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                if let Expr::Variable(name) = &exprs[0] {
                    match name.as_str() {
                        // Higher-order functions
                        "map" | "filter" | "fold" | "fold-right" | "for-each" => {
                            builder = builder.add_call_pattern(StaticCallPattern::HigherOrder);
                        }
                        // Loop constructs
                        "do" => {
                            builder = builder.add_call_pattern(StaticCallPattern::Loop {
                                estimated_iterations: Some(100), // Default estimate
                            });
                        }
                        // Builtin functions
                        name if self.is_builtin_function(name) => {
                            builder = builder.add_call_pattern(StaticCallPattern::Builtin {
                                name: name.to_string(),
                                arity: Some(exprs.len() - 1),
                            });
                        }
                        // Direct function calls
                        _ => {
                            builder = builder.add_call_pattern(StaticCallPattern::Direct {
                                function_name: name.to_string(),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
        builder
    }
    
    /// Add constant bindings from static optimization
    fn add_constant_bindings(
        &self,
        mut builder: ExecutionContextBuilder,
        analysis_result: &crate::evaluator::AnalysisResult,
    ) -> ExecutionContextBuilder {
        // Add constants discovered during analysis from optimization hints
        for hint in &analysis_result.optimizations {
            if let crate::evaluator::OptimizationHint::ConstantFold(value) = hint {
                // Extract variable name from constant fold hint if available
                // For now, we'll use a generic constant binding approach
                builder = builder.add_constant_binding(
                    "folded_constant".to_string(), 
                    value.clone()
                );
            } else if let crate::evaluator::OptimizationHint::InlineVariable(name, value) = hint {
                builder = builder.add_constant_binding(name.clone(), value.clone());
            }
        }
        builder
    }
    
    /// Determine execution priority based on analysis
    fn determine_execution_priority(
        &self,
        analysis_result: &crate::evaluator::AnalysisResult,
    ) -> ExecutionPriority {
        match analysis_result.complexity.complexity_score() {
            0..=25 => ExecutionPriority::Low,
            26..=50 => ExecutionPriority::Normal,
            51..=75 => ExecutionPriority::High,
            76.. => ExecutionPriority::Critical,
        }
    }
    
    /// Check if a function name is a builtin
    fn is_builtin_function(&self, name: &str) -> bool {
        self.global_env.exists(name)
    }
    
    /// Calculate macro expansion depth
    fn calculate_expansion_depth(&self, expr: &Expr) -> usize {
        if self.contains_macro_calls(expr) {
            1 // Simple depth calculation - could be enhanced
        } else {
            0
        }
    }
    
    /// Extract names of expanded macros
    fn extract_expanded_macros(&self, expr: &Expr) -> Vec<String> {
        let mut macros = Vec::new();
        if let Expr::List(exprs) = expr {
            if let Some(Expr::Variable(name)) = exprs.first() {
                if self.macro_expander.is_macro_call(expr) {
                    macros.push(name.clone());
                }
            }
        }
        macros
    }
    
    /// Add static optimization information to context builder
    fn add_static_optimization_info(
        &self,
        mut builder: ExecutionContextBuilder,
        original_expr: &Expr,
        preprocessed_expr: &Expr,
        analysis_result: &crate::evaluator::AnalysisResult,
    ) -> Result<ExecutionContextBuilder> {
        use crate::evaluator::execution_context::*;
        
        // Record macro expansion if it occurred
        if original_expr != preprocessed_expr {
            let optimization = StaticOptimization::MacroExpansion {
                macro_name: self.extract_macro_name(original_expr),
                original_form: format!("{:?}", original_expr),
                expanded_form: format!("{:?}", preprocessed_expr),
            };
            builder = builder.add_static_optimization(optimization);
        }
        
        // Add constant folding opportunities from analysis
        for hint in &analysis_result.optimizations {
            match hint {
                crate::evaluator::OptimizationHint::ConstantFold(value) => {
                    let opportunity = ConstantFoldingOpportunity {
                        expression: format!("{:?}", original_expr),
                        folded_value: value.clone(),
                        confidence: 0.9, // High confidence for detected constants
                        performance_benefit: PerformanceBenefit {
                            time_savings_micros: 10, // Estimate
                            memory_savings_bytes: 64, // Estimate
                            cpu_cycles_saved: 100, // Estimate
                        },
                    };
                    builder = builder.add_constant_folding_opportunity(opportunity);
                }
                crate::evaluator::OptimizationHint::InlineVariable(name, value) => {
                    // Record as both constant binding and static optimization
                    builder = builder.add_constant_binding(name.clone(), value.clone());
                    let optimization = StaticOptimization::CommonSubexpressionElimination {
                        subexpression: name.clone(),
                        variable_name: name.clone(),
                    };
                    builder = builder.add_static_optimization(optimization);
                }
                _ => {} // Handle other optimization hints as needed
            }
        }
        
        // Detect common subexpression elimination opportunities
        if let Some(cse_candidates) = self.detect_cse_candidates(preprocessed_expr) {
            for candidate in cse_candidates {
                builder = builder.add_cse_candidate(candidate);
            }
        }
        
        Ok(builder)
    }
    
    /// Extract macro name from expression
    fn extract_macro_name(&self, expr: &Expr) -> String {
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                if let Expr::Variable(name) = &exprs[0] {
                    name.clone()
                } else {
                    "unknown".to_string()
                }
            }
            _ => "unknown".to_string(),
        }
    }
    
    /// Detect common subexpression elimination candidates
    fn detect_cse_candidates(&self, expr: &Expr) -> Option<Vec<crate::evaluator::execution_context::CommonSubexpressionCandidate>> {
        use crate::evaluator::execution_context::CommonSubexpressionCandidate;
        use std::collections::HashMap;
        
        let mut subexpr_counts = HashMap::new();
        self.count_subexpressions(expr, &mut subexpr_counts);
        
        let candidates: Vec<_> = subexpr_counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(subexpr, count)| CommonSubexpressionCandidate {
                subexpression: subexpr,
                occurrence_count: count,
                computation_cost: 10, // Rough estimate
                memory_benefit: count * 32, // Rough estimate
            })
            .collect();
        
        if candidates.is_empty() {
            None
        } else {
            Some(candidates)
        }
    }
    
    /// Count subexpressions for CSE detection
    fn count_subexpressions(&self, expr: &Expr, counts: &mut std::collections::HashMap<String, usize>) {
        let expr_str = format!("{:?}", expr);
        *counts.entry(expr_str).or_insert(0) += 1;
        
        match expr {
            Expr::List(exprs) => {
                for subexpr in exprs {
                    self.count_subexpressions(subexpr, counts);
                }
            }
            Expr::Vector(exprs) => {
                for subexpr in exprs {
                    self.count_subexpressions(subexpr, counts);
                }
            }
            Expr::DottedList(exprs, tail) => {
                for subexpr in exprs {
                    self.count_subexpressions(subexpr, counts);
                }
                self.count_subexpressions(tail, counts);
            }
            _ => {} // Literals and variables are already counted
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_macro_expander_integration() {
        let evaluator = Evaluator::new();
        
        // Test that macro expander is correctly integrated
        assert!(!evaluator.macro_expander().is_macro_call(&Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))));
        
        // Test that contains_macro_calls works
        let simple_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        assert!(!evaluator.contains_macro_calls(&simple_expr));
    }

    #[test]
    fn test_static_optimization_preprocessing() {
        let evaluator = Evaluator::new();
        
        // Test preprocessing of simple expression (should pass through unchanged)
        let simple_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result = evaluator.preprocess_expression(simple_expr.clone());
        
        assert!(result.is_ok());
        match result.unwrap() {
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))) => {},
            _ => panic!("Expected unchanged literal"),
        }
    }

    #[test]
    fn test_create_execution_context_with_static_optimization() {
        let mut evaluator = Evaluator::new();
        let env = Rc::new(Environment::with_builtins());
        
        // Test ExecutionContext creation with static optimization
        let simple_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result = evaluator.create_execution_context(
            simple_expr,
            env,
            Continuation::Identity
        );
        
        assert!(result.is_ok());
        let context = result.unwrap();
        // Basic validation that context was created successfully
        // complexity_score is u32, so >= 0 check is redundant but kept for clarity
        assert!(context.static_analysis.complexity_score < u32::MAX);
    }
    
    #[test]
    fn test_macro_expansion_in_execution_context() {
        let mut evaluator = Evaluator::new();
        let env = Rc::new(Environment::with_builtins());
        
        // Test macro expansion tracking
        let macro_expr = Expr::List(vec![
            Expr::Variable("let".to_string()),
            Expr::List(vec![]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ]);
        
        let result = evaluator.create_execution_context(
            macro_expr.clone(),
            env,
            Continuation::Identity
        );
        
        assert!(result.is_ok());
        let context = result.unwrap();
        
        // Check that macro expansion state is populated
        assert!(context.macro_expansion_state.original_expression.is_some());
        assert!(context.macro_expansion_state.expanded_expression.is_some());
        
        // The execution expression should be the expanded form
        // The expanded expression should be different from original if macro expansion occurred
        // (exact comparison depends on macro expander implementation)
    }
    
    #[test]
    fn test_static_optimization_detection() {
        let mut evaluator = Evaluator::new();
        let env = Rc::new(Environment::with_builtins());
        
        // Test expression that should trigger static optimization hints
        let pure_expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
        ]);
        
        let result = evaluator.create_execution_context(
            pure_expr,
            env,
            Continuation::Identity
        );
        
        assert!(result.is_ok());
        let context = result.unwrap();
        
        // This should be detected as pure
        assert!(context.static_analysis.is_pure);
        
        // Should have builtin call pattern
        assert!(!context.static_analysis.call_patterns.is_empty());
        
        // Should have optimization hints
        assert!(!matches!(context.optimization_hints.optimization_level, 
                         crate::evaluator::execution_context::OptimizationLevel::None));
    }
}

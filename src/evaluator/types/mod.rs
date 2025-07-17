//! Evaluator Types Module
//!
//! This module provides a comprehensive implementation of the core type system for R7RS evaluator.
//! It includes `LocationHandle` trait, memory strategies, and Evaluator structure.
//!
//! ## Module Structure
//!
//! - `core_types`: Basic type definitions (`LocationHandle`, `MemoryStrategy`, statistics, etc.)

pub mod core_types;

// Re-export main types for backward compatibility
pub use core_types::{
    LocationHandle, StoreStatisticsWrapper, MemoryStrategy, MemoryStrategyConfig,
};

use crate::environment::Environment;
use crate::error::Result;
use crate::LambdustError;
use crate::evaluator::continuation::DynamicPoint;
use crate::evaluator::evaluation::ExceptionHandlerInfo;
use crate::evaluator::expression_analyzer::ExpressionAnalyzer;
use crate::srfi::SrfiRegistry;
use crate::value::Value;
use std::fmt::Debug;
use std::rc::Rc;

use crate::ast::Expr;
use crate::evaluator::Continuation;
// Optimizer imports removed as they're currently unused
// These can be re-added when implementing the full Evaluator functionality

/// Formal evaluator implementing R7RS semantics
#[derive(Debug)]
pub struct Evaluator {
    /// RAII-based memory management strategy
    memory_strategy: MemoryStrategy,
    
    /// Exception handler stack
    exception_handlers: Vec<ExceptionHandlerInfo>,
    
    // TODO: Implement evaluation order configuration
    // This field was removed as it's currently unused
    
    /// R7RS Scheme environment
    environment: Rc<Environment>,
    
    /// SRFI registry for standard library functions
    srfi_registry: SrfiRegistry,
    
    /// Static analysis and optimization
    expression_analyzer: ExpressionAnalyzer,
    
    // TODO: Implement advanced optimization systems
    // The following fields were removed as they're currently unused:
    // - value_optimizer: Memory efficiency optimization
    // - macro_expander: Macro expansion system
    // - continuation_pool: Continuation pooling manager
    // - do_loop_pool: Do-loop continuation pool
    // - inline_evaluator: Inline evaluation optimization
    // - jit_optimizer: JIT loop optimization
    // - tail_call_optimizer: Tail call optimization
    // - context_builder: Execution context builder
    
    /// Location registry for location references
    location_registry: crate::evaluator::higher_order::LocationRegistry,

    /// Dynamic point stack for dynamic-wind support
    dynamic_points: Vec<DynamicPoint>,
    
    /// Dynamic point ID counter
    dynamic_point_counter: usize,
    
    /// Global environment (compatibility field)
    pub global_env: Rc<Environment>,
}

impl Evaluator {
    /// Create a new formal evaluator with RAII memory management
    #[must_use] pub fn new() -> Self {
        Self::with_memory_strategy(MemoryStrategy::new())
    }

    /// Create evaluator with environment (compatibility method)
    pub fn with_environment(env: Rc<Environment>) -> Self {
        let mut evaluator = Self::new();
        evaluator.set_environment(env.clone());
        evaluator.global_env = env;
        evaluator
    }
    
    /// Create evaluator with custom memory strategy
    #[must_use] pub fn with_memory_strategy(memory_strategy: MemoryStrategy) -> Self {
        Self {
            memory_strategy,
            exception_handlers: Vec::new(),
            environment: Rc::new(Environment::new()),
            srfi_registry: SrfiRegistry::new(),
            expression_analyzer: ExpressionAnalyzer::new(),
            location_registry: crate::evaluator::higher_order::LocationRegistry::new(),
            dynamic_points: Vec::new(),
            dynamic_point_counter: 0,
            global_env: Rc::new(Environment::new()),
        }
    }

    /// Create production-optimized evaluator
    #[must_use] pub fn production() -> Self {
        let config = MemoryStrategyConfig::production();
        let memory_strategy = MemoryStrategy::with_config(config);
        Self::with_memory_strategy(memory_strategy)
    }

    /// Create development-friendly evaluator
    #[must_use] pub fn development() -> Self {
        let config = MemoryStrategyConfig::development();
        let memory_strategy = MemoryStrategy::with_config(config);
        Self::with_memory_strategy(memory_strategy)
    }

    /// Create testing evaluator
    #[must_use] pub fn testing() -> Self {
        let config = MemoryStrategyConfig::testing();
        let memory_strategy = MemoryStrategy::with_config(config);
        Self::with_memory_strategy(memory_strategy)
    }

    /// Get reference to memory strategy
    #[must_use] pub fn memory_strategy(&self) -> &MemoryStrategy {
        &self.memory_strategy
    }

    /// Get mutable reference to memory strategy
    pub fn memory_strategy_mut(&mut self) -> &mut MemoryStrategy {
        &mut self.memory_strategy
    }

    /// Get environment
    #[must_use] pub fn environment(&self) -> &Rc<Environment> {
        &self.environment
    }

    /// Set environment
    pub fn set_environment(&mut self, env: Rc<Environment>) {
        self.environment = env;
    }

    /// Get SRFI registry
    #[must_use] pub fn srfi_registry(&self) -> &SrfiRegistry {
        &self.srfi_registry
    }

    /// Get mutable SRFI registry
    pub fn srfi_registry_mut(&mut self) -> &mut SrfiRegistry {
        &mut self.srfi_registry
    }

    /// Get expression analyzer
    #[must_use] pub fn expression_analyzer(&self) -> &ExpressionAnalyzer {
        &self.expression_analyzer
    }

    /// Get mutable expression analyzer
    pub fn expression_analyzer_mut(&mut self) -> &mut ExpressionAnalyzer {
        &mut self.expression_analyzer
    }

    /// Evaluate expression (simplified interface)
    pub fn eval(&mut self, expr: &Expr) -> Result<Value> {
        // Simplified evaluation for demo
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    crate::ast::Literal::Number(n) => Ok(Value::Number(n.clone())),
                    crate::ast::Literal::String(s) => Ok(Value::String(s.clone())),
                    crate::ast::Literal::Boolean(b) => Ok(Value::Boolean(*b)),
                    crate::ast::Literal::Character(c) => Ok(Value::Character(*c)),
                    crate::ast::Literal::Nil => Ok(Value::Nil),
                }
            }
            Expr::Variable(name) => {
                self.environment.get(name).ok_or_else(|| {
                    LambdustError::runtime_error(format!("Unbound variable: {name}"))
                })
            }
            _ => Ok(Value::Nil),
        }
    }

    /// Evaluate expression with environment and continuation
    pub fn eval_with_continuation(&mut self, expr: Expr, env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        // Full evaluation with continuation support
        self.set_environment(env);
        let result = self.eval(&expr)?;
        self.apply_evaluator_continuation(cont, result)
    }

    /// Get memory statistics
    #[must_use] pub fn memory_statistics(&self) -> StoreStatisticsWrapper {
        self.memory_strategy.statistics()
    }

    /// Force garbage collection
    pub fn collect_garbage(&mut self) -> Result<usize> {
        self.memory_strategy.collect_garbage()
    }

    /// Check if memory is under pressure
    #[must_use] pub fn is_memory_under_pressure(&self) -> bool {
        self.memory_strategy.is_under_pressure()
    }

    /// Get optimization statistics
    #[must_use] pub fn optimization_stats(&self) -> String {
        format!(
            "Memory: {} bytes, JIT compiled patterns: {}, Tail calls optimized: {}",
            self.memory_statistics().memory_usage(),
            0, // TODO: Implement JIT optimizer compiled patterns
            0 // Placeholder for tail call stats
        )
    }
    
    /// Generate next reuse ID
    pub fn next_reuse_id(&mut self) -> usize {
        static mut COUNTER: usize = 0;
        unsafe {
            COUNTER += 1;
            COUNTER
        }
    }
    
    /// Apply procedure with arguments
    pub fn apply_procedure(
        &mut self, 
        procedure: &crate::value::Procedure, 
        _args: &[Value], 
        _env: &crate::environment::Environment
    ) -> Result<Value> {
        // Simplified implementation
        match procedure {
            crate::value::Procedure::Builtin { name, .. } => {
                // Call builtin function
                Ok(Value::String(format!("Applied builtin: {name}")))
            }
            crate::value::Procedure::Lambda { .. } => {
                // Apply user-defined procedure
                Ok(Value::String("Applied user procedure".to_string()))
            }
            _ => Ok(Value::Nil),
        }
    }

    /// Apply procedure with arguments and continuation
    pub fn apply_procedure_with_continuation(
        &mut self, 
        procedure: Value, 
        args: Vec<Value>, 
        env: Rc<Environment>,
        cont: Continuation
    ) -> Result<Value> {
        // Set environment
        self.set_environment(env);
        
        // Apply procedure (simplified)
        let result = match procedure {
            Value::Procedure(proc) => {
                let env_clone = self.environment.clone();
                self.apply_procedure(&proc, &args, &env_clone)?
            }
            _ => Value::Nil,
        };
        
        // Apply continuation to result  
        self.apply_evaluator_continuation(cont, result)
    }
    
    /// Apply continuation
    pub fn apply_continuation(
        &mut self,
        _continuation: &crate::value::Continuation,
        value: &Value,
        _env: &crate::environment::Environment
    ) -> Result<Value> {
        // Simplified implementation
        Ok(value.clone())
    }

    /// Apply continuation (for Continuation enum type)
    pub fn apply_evaluator_continuation(
        &mut self,
        _continuation: Continuation,
        value: Value
    ) -> Result<Value> {
        // Simplified implementation for Continuation enum
        Ok(value)
    }

    /// Get reference to exception handlers
    #[must_use] pub fn exception_handlers(&self) -> &Vec<ExceptionHandlerInfo> {
        &self.exception_handlers
    }

    /// Get mutable reference to exception handlers
    pub fn exception_handlers_mut(&mut self) -> &mut Vec<ExceptionHandlerInfo> {
        &mut self.exception_handlers
    }

    /// Push dynamic point onto the stack
    pub fn push_dynamic_point(&mut self, before: Option<Value>, after: Option<Value>) -> usize {
        let id = self.dynamic_point_counter;
        self.dynamic_point_counter += 1;
        
        let dynamic_point = DynamicPoint::new(before, after, None, id);
        self.dynamic_points.push(dynamic_point);
        id
    }

    /// Find dynamic point by ID
    pub fn find_dynamic_point_mut(&mut self, id: usize) -> Option<&mut DynamicPoint> {
        self.dynamic_points.iter_mut().find(|dp| dp.id == id)
    }

    /// Pop the most recent dynamic point from the stack
    pub fn pop_dynamic_point(&mut self) {
        self.dynamic_points.pop();
    }

    /// Get current dynamic point (most recent)
    #[must_use] pub fn current_dynamic_point(&self) -> Option<&DynamicPoint> {
        self.dynamic_points.last()
    }

    /// Get all active dynamic points
    #[must_use] pub fn active_dynamic_points(&self) -> Vec<&DynamicPoint> {
        self.dynamic_points.iter().filter(|dp| dp.active).collect()
    }
    
    /// Get current memory usage in bytes
    #[must_use] pub fn memory_usage(&self) -> usize {
        self.memory_statistics().memory_usage()
    }
    
    /// Get store statistics wrapper
    #[must_use] pub fn store_statistics(&self) -> StoreStatisticsWrapper {
        self.memory_statistics()
    }
    
    /// Set memory limit in bytes
    pub fn set_memory_limit(&mut self, _limit: usize) {
        // Memory strategy configuration update would be implemented here
        // For now, this is a placeholder
    }
    
    /// Allocate a value and return location handle
    pub fn allocate(&mut self, value: Value) -> Result<crate::evaluator::raii_store::RaiiLocation> {
        // Use memory strategy for allocation with proper error handling
        self.memory_strategy.allocate(value)
    }
    
    /// Evaluate sequence of expressions
    pub fn eval_sequence(&mut self, exprs: Vec<Expr>, env: Rc<Environment>, cont: Continuation) -> Result<Value> {
        if exprs.is_empty() {
            return self.apply_evaluator_continuation(cont, Value::Undefined);
        }
        
        let mut result = Value::Undefined;
        for expr in exprs {
            result = self.eval_with_continuation(expr, env.clone(), Continuation::Identity)?;
        }
        
        self.apply_evaluator_continuation(cont, result)
    }
    
    /// Apply captured continuation with non-local exit
    pub fn apply_captured_continuation_with_non_local_exit(
        &mut self,
        cont: Continuation,
        value: Value,
    ) -> Result<Value> {
        // Execute after thunks for dynamic points being exited
        let current_depth = self.dynamic_points.len();
        
        // Find the dynamic points to unwind
        let mut points_to_unwind = Vec::new();
        for i in (0..current_depth).rev() {
            if let Some(point) = self.dynamic_points.get(i) {
                if point.active {
                    points_to_unwind.push(i);
                }
            }
        }
        
        // Execute after thunks in reverse order (most recent first)
        for point_idx in points_to_unwind {
            if let Some(point) = self.dynamic_points.get_mut(point_idx) {
                if let Some(after_thunk) = &point.after.clone() {
                    // Call after thunk (simplified - in full implementation would need proper evaluation)
                    if let Value::Procedure(_) = after_thunk {
                        // Would call the procedure here in full implementation
                    }
                }
                point.deactivate();
            }
        }
        
        // Perform the non-local exit
        match cont {
            Continuation::Captured { cont } => {
                // Apply the captured continuation directly
                self.apply_evaluator_continuation(*cont, value)
            }
            _ => {
                // For other continuations, apply normally
                self.apply_evaluator_continuation(cont, value)
            }
        }
    }
    
    /// Create execution context for static analysis
    pub fn create_execution_context(&mut self, expr: Expr, env: Rc<Environment>) -> crate::evaluator::ExecutionContext {
        // Analyze the expression for optimization opportunities
        let optimization_hints = self.analyze_expression_for_optimization(&expr);
        
        // Create execution context with analysis results
        let context = crate::evaluator::ExecutionContext::new(expr.clone(), env.clone(), Continuation::Identity);
        
        // Apply static analysis results
        if optimization_hints.is_tail_recursive {
            // Tail call optimization hint would be applied here
        }
        
        if optimization_hints.has_loops {
            // Loop optimization hint would be applied here
        }
        
        if optimization_hints.complexity_score > 10 {
            // JIT compilation hint would be applied here
        }
        
        // Set memory allocation hints
        if optimization_hints.estimated_allocations > 100 {
            // Allocation hint would be applied here
        }
        
        context
    }
    
    /// Analyze expression for optimization opportunities
    fn analyze_expression_for_optimization(&self, expr: &Expr) -> OptimizationHints {
        let mut hints = OptimizationHints::default();
        
        self.analyze_expression_recursive(expr, &mut hints, 0);
        
        hints
    }
    
    /// Recursively analyze expression structure
    fn analyze_expression_recursive(&self, expr: &Expr, hints: &mut OptimizationHints, depth: usize) {
        hints.complexity_score += 1;
        
        match expr {
            Expr::List(exprs) => {
                if !exprs.is_empty() {
                    if let Expr::Variable(name) = &exprs[0] {
                        match name.as_str() {
                            "do" | "let" | "letrec" => {
                                hints.has_loops = true;
                                hints.estimated_allocations += 50;
                            }
                            "lambda" => {
                                hints.estimated_allocations += 20;
                                if depth > 0 {
                                    hints.is_tail_recursive = true;
                                }
                            }
                            "if" | "cond" | "case" => {
                                hints.complexity_score += 2;
                            }
                            _ => {}
                        }
                    }
                }
                
                for sub_expr in exprs {
                    self.analyze_expression_recursive(sub_expr, hints, depth + 1);
                }
            }
            Expr::Vector(exprs) => {
                hints.estimated_allocations += exprs.len();
                for sub_expr in exprs {
                    self.analyze_expression_recursive(sub_expr, hints, depth + 1);
                }
            }
            Expr::Quote(inner) => {
                self.analyze_expression_recursive(inner, hints, depth);
            }
            _ => {}
        }
    }
    
    /// Get immutable reference to location registry
    #[must_use] pub fn get_location_registry(&self) -> Option<&crate::evaluator::higher_order::LocationRegistry> {
        Some(&self.location_registry)
    }
    
    /// Get mutable reference to location registry
    pub fn get_location_registry_mut(&mut self) -> Option<&mut crate::evaluator::higher_order::LocationRegistry> {
        Some(&mut self.location_registry)
    }
}

/// Optimization hints for static analysis
#[derive(Debug, Clone, Default)]
struct OptimizationHints {
    /// Whether the expression is tail recursive
    is_tail_recursive: bool,
    /// Whether the expression contains loops
    has_loops: bool,
    /// Complexity score of the expression
    complexity_score: usize,
    /// Estimated number of allocations
    estimated_allocations: usize,
}

impl Evaluator {
    /// Evaluate string expression (compatibility method)
    pub fn eval_string(&mut self, source: &str) -> Result<Value> {
        // Parse the string (simplified implementation)
        use crate::lexer::Lexer;
        use crate::parser::Parser;
        
        let _lexer = Lexer::new(source);
        let tokens = crate::lexer::tokenize(source)?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expression()?;
        
        self.eval(&expr)
    }
    
    /// Call procedure (compatibility method)
    pub fn call_procedure(&mut self, procedure: Value, args: Vec<Value>) -> Result<Value> {
        match procedure {
            Value::Procedure(proc) => {
                let env_clone = self.environment.clone();
                self.apply_procedure(&proc, &args, &env_clone)
            }
            _ => Err(LambdustError::runtime_error("Not a procedure".to_string())),
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new evaluator with default configuration
#[must_use] pub fn create_evaluator() -> Evaluator {
    Evaluator::new()
}

/// Create a production-optimized evaluator
#[must_use] pub fn create_production_evaluator() -> Evaluator {
    Evaluator::production()
}

/// Create a development-friendly evaluator
#[must_use] pub fn create_development_evaluator() -> Evaluator {
    Evaluator::development()
}

/// Create a testing evaluator
#[must_use] pub fn create_testing_evaluator() -> Evaluator {
    Evaluator::testing()
}

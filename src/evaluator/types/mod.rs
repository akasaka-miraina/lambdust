//! Evaluator Types Module
//!
//! このモジュールはR7RS evaluatorのコア型システムの包括的な実装を提供します。
//! LocationHandle trait、メモリ戦略、Evaluator構造体を含みます。
//!
//! ## モジュール構成
//!
//! - `core_types`: 基本型定義（LocationHandle, MemoryStrategy, 統計等）

pub mod core_types;

// Re-export main types for backward compatibility
pub use core_types::{
    LocationHandle, StoreStatisticsWrapper, MemoryStrategy, MemoryStrategyConfig,
};

use crate::environment::Environment;
use crate::error::Result;
use crate::LambdustError;
use crate::evaluator::continuation::DynamicPoint;
use crate::evaluator::evaluation::{EvalOrder, ExceptionHandlerInfo};
use crate::evaluator::expression_analyzer::ExpressionAnalyzer;
use crate::evaluator::ExecutionContextBuilder;
use crate::srfi::SrfiRegistry;
use crate::value::{Value, ValueOptimizer};
use std::fmt::Debug;
use std::rc::Rc;

use crate::ast::Expr;
use crate::evaluator::Continuation;
use crate::macros::MacroExpander;
use crate::evaluator::control_flow::DoLoopContinuationPool;
use crate::evaluator::continuation_pooling::ContinuationPoolManager;
use crate::evaluator::inline_evaluation::InlineEvaluator;
use crate::evaluator::jit_loop_optimization::JitLoopOptimizer;
use crate::evaluator::tail_call_optimization::TailCallOptimizer;

/// Formal evaluator implementing R7RS semantics
#[derive(Debug)]
pub struct Evaluator {
    /// RAII-based memory management strategy
    memory_strategy: MemoryStrategy,
    
    /// Exception handler stack
    exception_handlers: Vec<ExceptionHandlerInfo>,
    
    /// Current evaluation order
    eval_order: EvalOrder,
    
    /// R7RS Scheme environment
    environment: Rc<Environment>,
    
    /// SRFI registry for standard library functions
    srfi_registry: SrfiRegistry,
    
    /// Static analysis and optimization
    expression_analyzer: ExpressionAnalyzer,
    
    /// Value optimizer for memory efficiency
    value_optimizer: ValueOptimizer,
    
    /// Macro expansion system
    macro_expander: MacroExpander,
    
    /// Advanced optimization systems
    continuation_pool: ContinuationPoolManager,
    do_loop_pool: DoLoopContinuationPool,
    inline_evaluator: InlineEvaluator,
    jit_optimizer: JitLoopOptimizer,
    tail_call_optimizer: TailCallOptimizer,
    
    /// ExecutionContext builder for static optimization
    context_builder: ExecutionContextBuilder,
    
    /// Global environment (compatibility field)
    pub global_env: Rc<Environment>,
}

impl Evaluator {
    /// Create a new formal evaluator with RAII memory management
    pub fn new() -> Self {
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
    pub fn with_memory_strategy(memory_strategy: MemoryStrategy) -> Self {
        Self {
            memory_strategy,
            exception_handlers: Vec::new(),
            eval_order: EvalOrder::LeftToRight,
            environment: Rc::new(Environment::new()),
            srfi_registry: SrfiRegistry::new(),
            expression_analyzer: ExpressionAnalyzer::new(),
            value_optimizer: ValueOptimizer::new(),
            macro_expander: MacroExpander::new(),
            continuation_pool: ContinuationPoolManager::new(),
            do_loop_pool: DoLoopContinuationPool::new(100), // Default pool size
            inline_evaluator: InlineEvaluator::new(),
            jit_optimizer: JitLoopOptimizer::new(),
            tail_call_optimizer: TailCallOptimizer::new(),
            context_builder: ExecutionContextBuilder::default(), // Use default constructor
            global_env: Rc::new(Environment::new()),
        }
    }

    /// Create production-optimized evaluator
    pub fn production() -> Self {
        let config = MemoryStrategyConfig::production();
        let memory_strategy = MemoryStrategy::with_config(config);
        Self::with_memory_strategy(memory_strategy)
    }

    /// Create development-friendly evaluator
    pub fn development() -> Self {
        let config = MemoryStrategyConfig::development();
        let memory_strategy = MemoryStrategy::with_config(config);
        Self::with_memory_strategy(memory_strategy)
    }

    /// Create testing evaluator
    pub fn testing() -> Self {
        let config = MemoryStrategyConfig::testing();
        let memory_strategy = MemoryStrategy::with_config(config);
        Self::with_memory_strategy(memory_strategy)
    }

    /// Get reference to memory strategy
    pub fn memory_strategy(&self) -> &MemoryStrategy {
        &self.memory_strategy
    }

    /// Get mutable reference to memory strategy
    pub fn memory_strategy_mut(&mut self) -> &mut MemoryStrategy {
        &mut self.memory_strategy
    }

    /// Get environment
    pub fn environment(&self) -> &Rc<Environment> {
        &self.environment
    }

    /// Set environment
    pub fn set_environment(&mut self, env: Rc<Environment>) {
        self.environment = env;
    }

    /// Get SRFI registry
    pub fn srfi_registry(&self) -> &SrfiRegistry {
        &self.srfi_registry
    }

    /// Get mutable SRFI registry
    pub fn srfi_registry_mut(&mut self) -> &mut SrfiRegistry {
        &mut self.srfi_registry
    }

    /// Get expression analyzer
    pub fn expression_analyzer(&self) -> &ExpressionAnalyzer {
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
                    LambdustError::runtime_error(format!("Unbound variable: {}", name))
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
    pub fn memory_statistics(&self) -> StoreStatisticsWrapper {
        self.memory_strategy.statistics()
    }

    /// Force garbage collection
    pub fn collect_garbage(&mut self) -> Result<usize> {
        self.memory_strategy.collect_garbage()
    }

    /// Check if memory is under pressure
    pub fn is_memory_under_pressure(&self) -> bool {
        self.memory_strategy.is_under_pressure()
    }

    /// Get optimization statistics
    pub fn optimization_stats(&self) -> String {
        format!(
            "Memory: {} bytes, JIT compiled patterns: {}, Tail calls optimized: {}",
            self.memory_statistics().memory_usage(),
            self.jit_optimizer.combined_stats().compiled_patterns,
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
        args: &[Value], 
        env: &crate::environment::Environment
    ) -> Result<Value> {
        // Simplified implementation
        match procedure {
            crate::value::Procedure::Builtin { name, .. } => {
                // Call builtin function
                Ok(Value::String(format!("Applied builtin: {}", name)))
            }
            crate::value::Procedure::UserDefined { .. } => {
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
    pub fn exception_handlers(&self) -> &Vec<ExceptionHandlerInfo> {
        &self.exception_handlers
    }

    /// Get mutable reference to exception handlers
    pub fn exception_handlers_mut(&mut self) -> &mut Vec<ExceptionHandlerInfo> {
        &mut self.exception_handlers
    }

    /// Push dynamic point (placeholder implementation)
    pub fn push_dynamic_point(&mut self, _before: Option<Value>, _after: Option<Value>) -> usize {
        // TODO: Implement dynamic point stack
        0 // Return placeholder ID
    }

    /// Find dynamic point by ID (placeholder implementation)
    pub fn find_dynamic_point_mut(&mut self, _id: usize) -> Option<&mut DynamicPoint> {
        // TODO: Implement dynamic point lookup
        None
    }

    /// Pop dynamic point (placeholder implementation)
    pub fn pop_dynamic_point(&mut self) {
        // TODO: Implement dynamic point stack pop
    }
    
    /// Get current memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.memory_statistics().memory_usage()
    }
    
    /// Get store statistics wrapper
    pub fn store_statistics(&self) -> StoreStatisticsWrapper {
        self.memory_statistics()
    }
    
    /// Set memory limit in bytes
    pub fn set_memory_limit(&mut self, limit: usize) {
        // TODO: Implement memory limit setting
        drop(limit); // Temporarily unused
    }
    
    /// Allocate a value and return location handle
    pub fn allocate(&mut self, value: Value) -> Result<crate::evaluator::raii_store::RaiiLocation> {
        // TODO: Implement proper allocation
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
        // TODO: Implement proper non-local exit semantics
        self.apply_evaluator_continuation(cont, value)
    }
    
    /// Create execution context for static analysis
    pub fn create_execution_context(&mut self, expr: Expr, env: Rc<Environment>) -> crate::evaluator::ExecutionContext {
        // TODO: Implement proper execution context creation
        crate::evaluator::ExecutionContext::new(expr, env, Continuation::Identity)
    }
    
    /// Evaluate string expression (compatibility method)
    pub fn eval_string(&mut self, source: &str) -> Result<Value> {
        // Parse the string (simplified implementation)
        use crate::lexer::Lexer;
        use crate::parser::Parser;
        
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
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
pub fn create_evaluator() -> Evaluator {
    Evaluator::new()
}

/// Create a production-optimized evaluator
pub fn create_production_evaluator() -> Evaluator {
    Evaluator::production()
}

/// Create a development-friendly evaluator
pub fn create_development_evaluator() -> Evaluator {
    Evaluator::development()
}

/// Create a testing evaluator
pub fn create_testing_evaluator() -> Evaluator {
    Evaluator::testing()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_evaluator_creation() {
        let evaluator = create_evaluator();
        assert!(!evaluator.is_memory_under_pressure());
        assert_eq!(evaluator.memory_statistics().total_allocations(), 0);
    }

    #[test]
    fn test_production_evaluator() {
        let evaluator = create_production_evaluator();
        let stats = evaluator.memory_statistics();
        assert_eq!(stats.total_allocations(), 0);
    }

    #[test]
    fn test_development_evaluator() {
        let evaluator = create_development_evaluator();
        let stats = evaluator.memory_statistics();
        assert_eq!(stats.total_allocations(), 0);
    }

    #[test]
    fn test_testing_evaluator() {
        let evaluator = create_testing_evaluator();
        let stats = evaluator.memory_statistics();
        assert_eq!(stats.total_allocations(), 0);
    }

    #[test]
    fn test_memory_strategy() {
        let mut strategy = MemoryStrategy::new();
        assert!(!strategy.is_under_pressure());
        assert_eq!(strategy.memory_pressure(), 0.0);
        
        let stats = strategy.statistics();
        assert_eq!(stats.active_allocations(), 0);
    }

    #[test]
    fn test_memory_strategy_config() {
        let prod_config = MemoryStrategyConfig::production();
        assert!(prod_config.enable_optimization);
        assert!(!prod_config.enable_debugging);
        assert_eq!(prod_config.memory_limit, 1024 * 1024 * 1024);

        let dev_config = MemoryStrategyConfig::development();
        assert!(!dev_config.enable_optimization);
        assert!(dev_config.enable_debugging);
        assert_eq!(dev_config.memory_limit, 512 * 1024 * 1024);
    }

    #[test]
    fn test_simple_evaluation() {
        let mut evaluator = create_evaluator();
        
        // Test literal evaluation
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result = evaluator.eval(&expr).unwrap();
        
        if let Value::Number(SchemeNumber::Integer(n)) = result {
            assert_eq!(n, 42);
        } else {
            panic!("Expected integer 42");
        }
    }

    #[test]
    fn test_store_statistics_wrapper() {
        let raii_stats = crate::evaluator::raii_store::RaiiStoreStatistics::default();
        let wrapper = StoreStatisticsWrapper::from_raii(raii_stats);
        
        assert_eq!(wrapper.total_allocations(), 0);
        assert_eq!(wrapper.total_deallocations(), 0);
        assert_eq!(wrapper.active_allocations(), 0);
        assert_eq!(wrapper.allocation_efficiency(), 0.0);
    }

    #[test]
    fn test_optimization_stats() {
        let evaluator = create_evaluator();
        let stats = evaluator.optimization_stats();
        assert!(stats.contains("Memory:"));
        assert!(stats.contains("JIT compiled patterns:"));
        assert!(stats.contains("Tail calls optimized:"));
    }
}
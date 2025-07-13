//! Execution Context for Evaluator-Executor Communication
//!
//! This module defines the ExecutionContext structure that serves as the
//! information bridge between the Evaluator (static analysis & optimization)
//! and the Executor (dynamic optimization & execution).
//!
//! The ExecutionContext encapsulates all necessary information for the executor
//! to perform optimized evaluation while maintaining semantic correctness.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::evaluator::Continuation;
use crate::value::Value;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::rc::Rc;

/// Execution context for transferring information from Evaluator to Executor
///
/// This structure contains all the information needed by the RuntimeExecutor
/// to perform optimized evaluation, including static analysis results,
/// optimization hints, and execution state.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// The expression to be executed
    pub expression: Expr,
    
    /// Runtime environment for variable lookups
    pub environment: Rc<Environment>,
    
    /// Continuation for evaluation flow control
    pub continuation: Continuation,
    
    /// Static analysis results from the evaluator
    pub static_analysis: StaticAnalysisResult,
    
    /// Optimization hints from static analysis
    pub optimization_hints: OptimizationHints,
    
    /// Execution metadata for runtime decisions
    pub execution_metadata: ExecutionMetadata,
    
    /// Pre-evaluated constant expressions (static optimization result)
    pub constant_bindings: FxHashMap<String, Value>,
    
    /// Macro expansion state
    pub macro_expansion_state: MacroExpansionState,
    
    /// Static semantic optimization results with formal proofs
    pub proven_optimizations: Vec<crate::evaluator::static_semantic_optimizer::ProvenOptimization>,
    
    /// Type inference results
    pub type_information: Option<crate::evaluator::static_semantic_optimizer::InferredType>,
}

/// Results from static analysis performed by the Evaluator
#[derive(Debug, Clone, Default)]
pub struct StaticAnalysisResult {
    /// Expression complexity score (0-100)
    pub complexity_score: u32,
    
    /// Whether this expression contains tail calls
    pub has_tail_calls: bool,
    
    /// Whether this expression contains loops
    pub has_loops: bool,
    
    /// Detected function call patterns
    pub call_patterns: Vec<StaticCallPattern>,
    
    /// Variable usage analysis
    pub variable_usage: FxHashMap<String, VariableUsage>,
    
    /// Memory allocation estimates
    pub memory_estimates: MemoryEstimates,
    
    /// Expression dependencies for optimization
    pub dependencies: SmallVec<[String; 4]>,
    
    /// Whether the expression is side-effect free
    pub is_pure: bool,
    
    /// Static optimization transformations applied
    pub static_optimizations: Vec<StaticOptimization>,
    
    /// Constant folding opportunities identified
    pub constant_folding_opportunities: Vec<ConstantFoldingOpportunity>,
    
    /// Dead code elimination opportunities
    pub dead_code_opportunities: SmallVec<[String; 4]>,
    
    /// Common subexpression elimination candidates
    pub cse_candidates: Vec<CommonSubexpressionCandidate>,
}

/// Static call pattern analysis
#[derive(Debug, Clone, PartialEq)]
pub enum StaticCallPattern {
    /// Direct function call
    Direct { 
        /// Name of the function being called
        function_name: String 
    },
    
    /// Recursive call pattern
    Recursive { 
        /// Estimated recursion depth for optimization
        depth_hint: Option<u32> 
    },
    
    /// Tail recursive pattern
    TailRecursive,
    
    /// Higher-order function application
    HigherOrder,
    
    /// Builtin function call
    Builtin { 
        /// Name of the builtin function
        name: String, 
        /// Function arity (number of arguments)
        arity: Option<usize> 
    },
    
    /// Loop construct
    Loop { 
        /// Estimated number of iterations for optimization
        estimated_iterations: Option<u32> 
    },
}

/// Variable usage information for optimization
#[derive(Debug, Clone)]
pub struct VariableUsage {
    /// Number of times the variable is referenced
    pub reference_count: usize,
    
    /// Whether the variable is modified
    pub is_modified: bool,
    
    /// Whether the variable escapes its scope
    pub escapes_scope: bool,
    
    /// Variable type hint if known
    pub type_hint: Option<VariableTypeHint>,
}

/// Type hints for variables (for future type system integration)
#[derive(Debug, Clone, PartialEq)]
pub enum VariableTypeHint {
    /// Numeric value (integer or real)
    Number,
    /// String value
    String,
    /// Boolean value
    Boolean,
    /// List or vector value
    List,
    /// Procedure or lambda value
    Procedure,
    /// Type cannot be determined
    Unknown,
}

/// Memory allocation estimates
#[derive(Debug, Clone, Default)]
pub struct MemoryEstimates {
    /// Estimated stack usage
    pub stack_usage: usize,
    
    /// Estimated heap allocations
    pub heap_allocations: usize,
    
    /// Estimated maximum live objects
    pub max_live_objects: usize,
    
    /// Whether allocation patterns suggest pooling benefits
    pub benefits_from_pooling: bool,
}

/// Optimization hints for the RuntimeExecutor
#[derive(Debug, Clone, Default)]
pub struct OptimizationHints {
    /// Recommended optimization level
    pub optimization_level: OptimizationLevel,
    
    /// Whether JIT compilation is beneficial
    pub jit_beneficial: bool,
    
    /// Whether continuation pooling should be used
    pub use_continuation_pooling: bool,
    
    /// Whether tail call optimization should be applied
    pub use_tail_call_optimization: bool,
    
    /// Whether inline evaluation is recommended
    pub use_inline_evaluation: bool,
    
    /// Hot path indicators
    pub hot_path_indicators: Vec<HotPathIndicator>,
    
    /// Specific optimization strategies
    pub optimization_strategies: Vec<OptimizationStrategy>,
}

/// Optimization level recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptimizationLevel {
    /// No optimization applied
    #[default]
    None,
    /// Conservative optimization with safety guarantees
    Conservative,
    /// Balanced optimization with good performance/safety tradeoff
    Balanced,
    /// Aggressive optimization for maximum performance
    Aggressive,
}

/// Hot path detection indicators
#[derive(Debug, Clone, PartialEq)]
pub enum HotPathIndicator {
    /// Loop detected
    Loop { 
        /// Estimated number of loop iterations
        estimated_iterations: u32 
    },
    
    /// Recursive function
    Recursion { 
        /// Estimated recursion depth
        estimated_depth: u32 
    },
    
    /// Frequent function call
    FrequentCall { 
        /// Estimated number of calls
        call_count_estimate: u32 
    },
    
    /// High-frequency arithmetic
    ArithmeticHeavy,
    
    /// Memory allocation intensive
    AllocationHeavy,
}

/// Specific optimization strategies
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationStrategy {
    /// Loop unrolling with specified factor
    LoopUnrolling { 
        /// Unrolling factor (number of loop iterations to unroll)
        factor: u32 
    },
    
    /// Function inlining
    FunctionInlining { 
        /// Target functions for inlining
        target_functions: SmallVec<[String; 4]> 
    },
    
    /// Constant propagation
    ConstantPropagation,
    
    /// Dead code elimination
    DeadCodeElimination,
    
    /// Common subexpression elimination
    CommonSubexpressionElimination,
    
    /// Tail call optimization
    TailCallOptimization,
    
    /// Continuation optimization
    ContinuationOptimization,
}

/// Execution metadata for runtime decisions
#[derive(Debug, Clone, Default)]
pub struct ExecutionMetadata {
    /// Execution priority hint
    pub priority: ExecutionPriority,
    
    /// Whether this execution is part of a larger computation
    pub is_sub_computation: bool,
    
    /// Parent execution context if applicable
    pub parent_context_id: Option<u64>,
    
    /// Unique identifier for this execution context
    pub context_id: u64,
    
    /// Estimated execution time (microseconds)
    pub estimated_execution_time: Option<u64>,
    
    /// Memory usage constraints
    pub memory_constraints: Option<MemoryConstraints>,
    
    /// Thread safety requirements
    pub thread_safety: ThreadSafetyRequirements,
}

/// Execution priority classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutionPriority {
    /// Low priority execution
    Low,
    /// Normal priority execution (default)
    #[default]
    Normal,
    /// High priority execution
    High,
    /// Critical priority execution
    Critical,
}

/// Memory usage constraints
#[derive(Debug, Clone)]
pub struct MemoryConstraints {
    /// Maximum allowed heap usage (bytes)
    pub max_heap_usage: Option<usize>,
    
    /// Maximum allowed stack depth
    pub max_stack_depth: Option<usize>,
    
    /// Whether memory pooling is required
    pub require_pooling: bool,
}

/// Thread safety requirements
#[derive(Debug, Clone, Default)]
pub struct ThreadSafetyRequirements {
    /// Whether the execution must be thread-safe
    pub must_be_thread_safe: bool,
    
    /// Whether concurrent execution is allowed
    pub allow_concurrent_execution: bool,
    
    /// Required synchronization level
    pub synchronization_level: SynchronizationLevel,
}

/// Synchronization level for thread safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SynchronizationLevel {
    /// No synchronization required
    #[default]
    None,
    /// Read-only access, multiple readers allowed
    ReadOnly,
    /// Full synchronization with locks
    Synchronized,
    /// Atomic operations for lock-free access
    Atomic,
}

/// Macro expansion state for the execution context
#[derive(Debug, Clone, Default)]
pub struct MacroExpansionState {
    /// Whether macro expansion has been performed
    pub is_expanded: bool,
    
    /// Original expression before macro expansion
    pub original_expression: Option<Expr>,
    
    /// Expression after complete macro expansion
    pub expanded_expression: Option<Expr>,
    
    /// List of macros that were expanded
    pub expanded_macros: SmallVec<[String; 4]>,
    
    /// Macro expansion depth
    pub expansion_depth: usize,
    
    /// Whether further expansion is needed
    pub needs_further_expansion: bool,
    
    /// Hygiene information for expanded macros
    pub hygiene_info: Vec<HygieneInfo>,
    
    /// Macro expansion performance metrics
    pub expansion_metrics: MacroExpansionMetrics,
}

/// Hygiene information for macro expansion
#[derive(Debug, Clone)]
pub struct HygieneInfo {
    /// Original symbol name
    pub original_name: String,
    
    /// Hygienic symbol name
    pub hygienic_name: String,
    
    /// Scope identifier
    pub scope_id: u64,
    
    /// Whether the symbol was renamed
    pub was_renamed: bool,
}

/// Macro expansion performance metrics
#[derive(Debug, Clone, Default)]
pub struct MacroExpansionMetrics {
    /// Time spent on macro expansion (microseconds)
    pub expansion_time_micros: u64,
    
    /// Number of macro expansions performed
    pub expansion_count: usize,
    
    /// Maximum expansion depth reached
    pub max_expansion_depth: usize,
    
    /// Number of hygiene transformations
    pub hygiene_transformations: usize,
}

/// Static optimization transformation record
#[derive(Debug, Clone, PartialEq)]
pub enum StaticOptimization {
    /// Constant folding was applied
    ConstantFolding {
        /// Original expression representation
        original: String,
        /// Folded result
        result: Value,
    },
    
    /// Dead code was eliminated
    DeadCodeElimination {
        /// Description of eliminated code
        eliminated_code: String,
    },
    
    /// Common subexpression was eliminated
    CommonSubexpressionElimination {
        /// Original subexpression
        subexpression: String,
        /// Variable name assigned to the subexpression
        variable_name: String,
    },
    
    /// Function inlining was performed
    FunctionInlining {
        /// Function name that was inlined
        function_name: String,
        /// Call site description
        call_site: String,
    },
    
    /// Macro expansion optimization
    MacroExpansion {
        /// Macro name
        macro_name: String,
        /// Original form
        original_form: String,
        /// Expanded form
        expanded_form: String,
    },
}

/// Constant folding opportunity
#[derive(Debug, Clone, PartialEq)]
pub struct ConstantFoldingOpportunity {
    /// Expression that can be folded
    pub expression: String,
    
    /// Estimated folded value
    pub folded_value: Value,
    
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    
    /// Estimated performance benefit
    pub performance_benefit: PerformanceBenefit,
}

/// Common subexpression elimination candidate
#[derive(Debug, Clone, PartialEq)]
pub struct CommonSubexpressionCandidate {
    /// The common subexpression
    pub subexpression: String,
    
    /// Number of times it appears
    pub occurrence_count: usize,
    
    /// Estimated computation cost of the subexpression
    pub computation_cost: u32,
    
    /// Estimated memory benefit from elimination
    pub memory_benefit: usize,
}

/// Performance benefit estimation
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceBenefit {
    /// Estimated time savings (microseconds)
    pub time_savings_micros: u64,
    
    /// Estimated memory savings (bytes)
    pub memory_savings_bytes: usize,
    
    /// Estimated CPU cycles saved
    pub cpu_cycles_saved: u64,
}

impl ExecutionContext {
    /// Create a new execution context with minimal information
    pub fn new(expression: Expr, environment: Rc<Environment>, continuation: Continuation) -> Self {
        Self {
            expression,
            environment,
            continuation,
            static_analysis: StaticAnalysisResult::default(),
            optimization_hints: OptimizationHints::default(),
            execution_metadata: ExecutionMetadata::default(),
            constant_bindings: FxHashMap::default(),
            macro_expansion_state: MacroExpansionState::default(),
            proven_optimizations: Vec::new(),
            type_information: None,
        }
    }
    
    /// Create execution context with macro expansion results
    pub fn with_macro_expansion(
        original_expr: Expr,
        expanded_expr: Expr,
        environment: Rc<Environment>,
        continuation: Continuation,
        expansion_metrics: MacroExpansionMetrics,
    ) -> Self {
        let mut context = Self::new(expanded_expr.clone(), environment, continuation);
        context.macro_expansion_state.is_expanded = true;
        context.macro_expansion_state.original_expression = Some(original_expr);
        context.macro_expansion_state.expanded_expression = Some(expanded_expr);
        context.macro_expansion_state.expansion_metrics = expansion_metrics;
        context
    }
    
    /// Create an execution context with static analysis results
    pub fn with_static_analysis(
        expression: Expr,
        environment: Rc<Environment>,
        continuation: Continuation,
        static_analysis: StaticAnalysisResult,
    ) -> Self {
        let mut context = Self::new(expression, environment, continuation);
        context.static_analysis = static_analysis;
        context.derive_optimization_hints();
        context
    }
    
    /// Derive optimization hints from static analysis
    pub fn derive_optimization_hints(&mut self) {
        let mut hints = OptimizationHints::default();
        
        // Determine optimization level based on complexity
        hints.optimization_level = match self.static_analysis.complexity_score {
            0..=25 => OptimizationLevel::Conservative,
            26..=50 => OptimizationLevel::Balanced,
            51..=75 => OptimizationLevel::Balanced,
            76..=100 => OptimizationLevel::Aggressive,
            _ => OptimizationLevel::None,
        };
        
        // JIT beneficial for high complexity or loops
        hints.jit_beneficial = self.static_analysis.complexity_score > 50 || self.static_analysis.has_loops;
        
        // Tail call optimization for tail recursive patterns
        hints.use_tail_call_optimization = self.static_analysis.has_tail_calls;
        
        // Continuation pooling for high call frequency
        hints.use_continuation_pooling = self.static_analysis.call_patterns.len() > 3;
        
        // Inline evaluation for simple pure expressions
        hints.use_inline_evaluation = self.static_analysis.is_pure && self.static_analysis.complexity_score < 30;
        
        // Add hot path indicators
        for pattern in &self.static_analysis.call_patterns {
            match pattern {
                StaticCallPattern::Loop { estimated_iterations: Some(iters) } => {
                    hints.hot_path_indicators.push(HotPathIndicator::Loop { estimated_iterations: *iters });
                }
                StaticCallPattern::Recursive { depth_hint: Some(depth) } => {
                    hints.hot_path_indicators.push(HotPathIndicator::Recursion { estimated_depth: *depth });
                }
                _ => {}
            }
        }
        
        // Add optimization strategies
        if self.static_analysis.has_loops {
            hints.optimization_strategies.push(OptimizationStrategy::LoopUnrolling { factor: 2 });
        }
        
        if self.static_analysis.is_pure {
            hints.optimization_strategies.push(OptimizationStrategy::ConstantPropagation);
            hints.optimization_strategies.push(OptimizationStrategy::CommonSubexpressionElimination);
        }
        
        if self.static_analysis.has_tail_calls {
            hints.optimization_strategies.push(OptimizationStrategy::TailCallOptimization);
        }
        
        self.optimization_hints = hints;
    }
    
    /// Add a constant binding from static optimization
    pub fn add_constant_binding(&mut self, name: String, value: Value) {
        self.constant_bindings.insert(name, value);
    }
    
    /// Add static optimization record
    pub fn add_static_optimization(&mut self, optimization: StaticOptimization) {
        self.static_analysis.static_optimizations.push(optimization);
    }
    
    /// Add constant folding opportunity
    pub fn add_constant_folding_opportunity(&mut self, opportunity: ConstantFoldingOpportunity) {
        self.static_analysis.constant_folding_opportunities.push(opportunity);
    }
    
    /// Add common subexpression elimination candidate
    pub fn add_cse_candidate(&mut self, candidate: CommonSubexpressionCandidate) {
        self.static_analysis.cse_candidates.push(candidate);
    }
    
    /// Record macro expansion in the state
    pub fn record_macro_expansion(&mut self, macro_name: String, original_form: String, expanded_form: String) {
        self.macro_expansion_state.expanded_macros.push(macro_name.clone());
        self.static_analysis.static_optimizations.push(StaticOptimization::MacroExpansion {
            macro_name,
            original_form,
            expanded_form,
        });
    }
    
    /// Get the expression to execute (expanded if available, original otherwise)
    pub fn get_execution_expression(&self) -> &Expr {
        if let Some(ref expanded) = self.macro_expansion_state.expanded_expression {
            expanded
        } else {
            &self.expression
        }
    }
    
    /// Check if macro expansion was performed
    pub fn was_macro_expanded(&self) -> bool {
        self.macro_expansion_state.is_expanded
    }
    
    /// Get static optimization count
    pub fn static_optimization_count(&self) -> usize {
        self.static_analysis.static_optimizations.len()
    }
    
    /// Get total performance benefit estimate from static optimizations
    pub fn estimated_static_benefit(&self) -> PerformanceBenefit {
        let mut total_time_savings = 0u64;
        let mut total_memory_savings = 0usize;
        let mut total_cpu_savings = 0u64;
        
        for opportunity in &self.static_analysis.constant_folding_opportunities {
            total_time_savings += opportunity.performance_benefit.time_savings_micros;
            total_memory_savings += opportunity.performance_benefit.memory_savings_bytes;
            total_cpu_savings += opportunity.performance_benefit.cpu_cycles_saved;
        }
        
        for candidate in &self.static_analysis.cse_candidates {
            // Estimate benefit based on occurrence count and computation cost
            let estimated_time_savings = (candidate.occurrence_count as u64) * (candidate.computation_cost as u64);
            total_time_savings += estimated_time_savings;
            total_memory_savings += candidate.memory_benefit;
            total_cpu_savings += estimated_time_savings * 10; // Rough CPU cycle estimate
        }
        
        PerformanceBenefit {
            time_savings_micros: total_time_savings,
            memory_savings_bytes: total_memory_savings,
            cpu_cycles_saved: total_cpu_savings,
        }
    }
    
    /// Check if a variable has a pre-computed constant value
    pub fn get_constant_binding(&self, name: &str) -> Option<&Value> {
        self.constant_bindings.get(name)
    }
    
    /// Update execution metadata
    pub fn set_execution_metadata(&mut self, metadata: ExecutionMetadata) {
        self.execution_metadata = metadata;
    }
    
    /// Check if optimization is recommended for this context
    pub fn should_optimize(&self) -> bool {
        !matches!(self.optimization_hints.optimization_level, OptimizationLevel::None)
    }
    
    /// Check if JIT compilation is beneficial
    pub fn should_use_jit(&self) -> bool {
        self.optimization_hints.jit_beneficial && self.static_analysis.complexity_score > 40
    }
    
    /// Get memory usage estimate
    pub fn estimated_memory_usage(&self) -> usize {
        self.static_analysis.memory_estimates.heap_allocations + 
        self.static_analysis.memory_estimates.stack_usage
    }
    
    /// Generate a unique context ID
    pub fn generate_context_id() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CONTEXT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        CONTEXT_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new(
            Expr::Literal(crate::ast::Literal::Nil),
            Rc::new(Environment::new()),
            Continuation::Identity,
        )
    }
}

/// Builder pattern for constructing ExecutionContext
#[derive(Debug)]
pub struct ExecutionContextBuilder {
    context: ExecutionContext,
}

impl ExecutionContextBuilder {
    /// Create new ExecutionContextBuilder with base context
    pub fn new(expression: Expr, environment: Rc<Environment>, continuation: Continuation) -> Self {
        Self {
            context: ExecutionContext::new(expression, environment, continuation),
        }
    }
    
    /// Set complexity score for static analysis
    pub fn with_complexity_score(mut self, score: u32) -> Self {
        self.context.static_analysis.complexity_score = score;
        self
    }
    
    /// Set tail call analysis result
    pub fn with_tail_calls(mut self, has_tail_calls: bool) -> Self {
        self.context.static_analysis.has_tail_calls = has_tail_calls;
        self
    }
    
    /// Set loop analysis result
    pub fn with_loops(mut self, has_loops: bool) -> Self {
        self.context.static_analysis.has_loops = has_loops;
        self
    }
    
    /// Set purity analysis result
    pub fn with_purity(mut self, is_pure: bool) -> Self {
        self.context.static_analysis.is_pure = is_pure;
        self
    }
    
    /// Add static call pattern to analysis
    pub fn add_call_pattern(mut self, pattern: StaticCallPattern) -> Self {
        self.context.static_analysis.call_patterns.push(pattern);
        self
    }
    
    /// Add constant binding to context
    pub fn add_constant_binding(mut self, name: String, value: Value) -> Self {
        self.context.constant_bindings.insert(name, value);
        self
    }
    
    /// Set execution priority
    pub fn with_priority(mut self, priority: ExecutionPriority) -> Self {
        self.context.execution_metadata.priority = priority;
        self
    }
    
    /// Set macro expansion state
    pub fn with_macro_expansion_state(mut self, state: MacroExpansionState) -> Self {
        self.context.macro_expansion_state = state;
        self
    }
    
    /// Add static optimization to analysis
    pub fn add_static_optimization(mut self, optimization: StaticOptimization) -> Self {
        self.context.static_analysis.static_optimizations.push(optimization);
        self
    }
    
    /// Add constant folding opportunity to analysis
    pub fn add_constant_folding_opportunity(mut self, opportunity: ConstantFoldingOpportunity) -> Self {
        self.context.static_analysis.constant_folding_opportunities.push(opportunity);
        self
    }
    
    /// Add common subexpression elimination candidate
    pub fn add_cse_candidate(mut self, candidate: CommonSubexpressionCandidate) -> Self {
        self.context.static_analysis.cse_candidates.push(candidate);
        self
    }
    
    /// Set complete static analysis result
    pub fn with_static_analysis_result(mut self, analysis: StaticAnalysisResult) -> Self {
        self.context.static_analysis = analysis;
        self
    }
    
    /// Add proven optimization with formal verification
    pub fn add_proven_optimization(mut self, optimization: crate::evaluator::static_semantic_optimizer::ProvenOptimization) -> Self {
        self.context.proven_optimizations.push(optimization);
        self
    }
    
    /// Set type information from static analysis
    pub fn with_type_information(mut self, type_info: crate::evaluator::static_semantic_optimizer::InferredType) -> Self {
        self.context.type_information = Some(type_info);
        self
    }
    
    /// Build the final ExecutionContext with derived optimization hints
    pub fn build(mut self) -> ExecutionContext {
        self.context.derive_optimization_hints();
        self.context.execution_metadata.context_id = ExecutionContext::generate_context_id();
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    
    #[test]
    fn test_execution_context_creation() {
        let expr = Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        let cont = Continuation::Identity;
        
        let context = ExecutionContext::new(expr, env, cont);
        
        assert_eq!(context.static_analysis.complexity_score, 0);
        assert!(!context.static_analysis.has_tail_calls);
        assert!(!context.static_analysis.has_loops);
    }
    
    #[test]
    fn test_optimization_hints_derivation() {
        let expr = Expr::Literal(Literal::Number(crate::lexer::SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        let cont = Continuation::Identity;
        
        let mut context = ExecutionContext::new(expr, env, cont);
        context.static_analysis.complexity_score = 75;
        context.static_analysis.has_loops = true;
        context.static_analysis.is_pure = true;
        context.derive_optimization_hints();
        
        assert_eq!(context.optimization_hints.optimization_level, OptimizationLevel::Balanced);
        assert!(context.optimization_hints.jit_beneficial);
        assert!(!context.optimization_hints.use_inline_evaluation); // complexity_score = 75 > 30, so no inline
    }
    
    #[test]
    fn test_execution_context_builder() {
        let expr = Expr::Literal(Literal::Boolean(true));
        let env = Rc::new(Environment::new());
        let cont = Continuation::Identity;
        
        let context = ExecutionContextBuilder::new(expr, env, cont)
            .with_complexity_score(60)
            .with_tail_calls(true)
            .with_purity(true)
            .with_priority(ExecutionPriority::High)
            .add_call_pattern(StaticCallPattern::TailRecursive)
            .build();
        
        assert_eq!(context.static_analysis.complexity_score, 60);
        assert!(context.static_analysis.has_tail_calls);
        assert!(context.optimization_hints.use_tail_call_optimization);
        assert_eq!(context.execution_metadata.priority, ExecutionPriority::High);
        assert!(context.static_analysis.call_patterns.contains(&StaticCallPattern::TailRecursive));
    }
    
    #[test]
    fn test_constant_binding() {
        let expr = Expr::Variable("x".to_string());
        let env = Rc::new(Environment::new());
        let cont = Continuation::Identity;
        
        let mut context = ExecutionContext::new(expr, env, cont);
        let value = Value::Number(crate::lexer::SchemeNumber::Integer(123));
        
        context.add_constant_binding("x".to_string(), value.clone());
        
        assert_eq!(context.get_constant_binding("x"), Some(&value));
        assert_eq!(context.get_constant_binding("y"), None);
    }
    
    #[test]
    fn test_macro_expansion_context() {
        let original_expr = Expr::List(vec![
            Expr::Variable("let".to_string()),
            Expr::List(vec![]),
            Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(42))),
        ]);
        
        let expanded_expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![]),
            Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(42))),
        ]);
        
        let env = Rc::new(Environment::new());
        let cont = Continuation::Identity;
        let metrics = MacroExpansionMetrics {
            expansion_time_micros: 100,
            expansion_count: 1,
            max_expansion_depth: 1,
            hygiene_transformations: 0,
        };
        
        let context = ExecutionContext::with_macro_expansion(
            original_expr.clone(),
            expanded_expr.clone(),
            env,
            cont,
            metrics
        );
        
        assert!(context.was_macro_expanded());
        assert_eq!(context.get_execution_expression(), &expanded_expr);
        assert_eq!(context.macro_expansion_state.original_expression, Some(original_expr));
        assert_eq!(context.macro_expansion_state.expanded_expression, Some(expanded_expr));
    }
    
    #[test]
    fn test_static_optimization_recording() {
        let expr = Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        let cont = Continuation::Identity;
        
        let mut context = ExecutionContext::new(expr, env, cont);
        
        // Add static optimization
        let optimization = StaticOptimization::ConstantFolding {
            original: "(+ 2 3)".to_string(),
            result: Value::Number(crate::lexer::SchemeNumber::Integer(5)),
        };
        context.add_static_optimization(optimization);
        
        // Add constant folding opportunity
        let opportunity = ConstantFoldingOpportunity {
            expression: "(* 4 6)".to_string(),
            folded_value: Value::Number(crate::lexer::SchemeNumber::Integer(24)),
            confidence: 0.95,
            performance_benefit: PerformanceBenefit {
                time_savings_micros: 50,
                memory_savings_bytes: 32,
                cpu_cycles_saved: 200,
            },
        };
        context.add_constant_folding_opportunity(opportunity);
        
        assert_eq!(context.static_optimization_count(), 1);
        assert_eq!(context.static_analysis.constant_folding_opportunities.len(), 1);
        
        let total_benefit = context.estimated_static_benefit();
        assert_eq!(total_benefit.time_savings_micros, 50);
        assert_eq!(total_benefit.memory_savings_bytes, 32);
    }
}

impl Default for ExecutionContextBuilder {
    fn default() -> Self {
        // Create a default context with placeholder values
        let expr = Expr::Literal(crate::ast::Literal::Boolean(true));
        let env = Rc::new(Environment::new());
        let cont = Continuation::Identity;
        Self::new(expr, env, cont)
    }
}
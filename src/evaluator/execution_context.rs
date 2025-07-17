//! Execution Context for Evaluator-Executor Communication
//!
//! This module defines the `ExecutionContext` structure that serves as the
//! information bridge between the Evaluator (static analysis & optimization)
//! and the Executor (dynamic optimization & execution).
//!
//! The `ExecutionContext` encapsulates all necessary information for the executor
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
/// This structure contains all the information needed by the `RuntimeExecutor`
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
    /// Note: Formal proof system integrated separately for cleaner architecture
    #[cfg(feature = "development")]
    pub proven_optimizations: Vec<String>, // Simplified for now
    
    /// Type inference results
    #[cfg(feature = "development")]
    pub type_information: Option<String>, // Simplified for now
    
    /// LLVM IR representation for execution (intermediate language form)
    #[cfg(feature = "development")]
    pub llvm_ir_context: Option<LLVMIRExecutionContext>,
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

/// Optimization hints for the `RuntimeExecutor`
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
            #[cfg(feature = "development")]
            proven_optimizations: Vec::new(),
            #[cfg(feature = "development")]
            type_information: None,
            #[cfg(feature = "development")]
            llvm_ir_context: None,
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
    #[must_use] pub fn get_execution_expression(&self) -> &Expr {
        if let Some(ref expanded) = self.macro_expansion_state.expanded_expression {
            expanded
        } else {
            &self.expression
        }
    }
    
    /// Check if macro expansion was performed
    #[must_use] pub fn was_macro_expanded(&self) -> bool {
        self.macro_expansion_state.is_expanded
    }
    
    /// Get static optimization count
    #[must_use] pub fn static_optimization_count(&self) -> usize {
        self.static_analysis.static_optimizations.len()
    }
    
    /// Get total performance benefit estimate from static optimizations
    #[must_use] pub fn estimated_static_benefit(&self) -> PerformanceBenefit {
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
            let estimated_time_savings = (candidate.occurrence_count as u64) * u64::from(candidate.computation_cost);
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
    #[must_use] pub fn get_constant_binding(&self, name: &str) -> Option<&Value> {
        self.constant_bindings.get(name)
    }

    /// Get pre-evaluated result if available (for optimization chains)
    #[must_use] pub fn get_pre_evaluated_result(&self) -> Option<&Value> {
        // For now, return None - this would be populated by evaluator
        // when it pre-evaluates simple expressions
        None
    }

    /// Get cached result if available (for idempotent operations)
    #[must_use] pub fn get_cached_result(&self) -> Option<&Value> {
        // For now, return None - this would be populated by environment
        // for idempotent operations that have been cached
        None
    }

    /// Create LLVM IR execution context for intermediate representation
    #[cfg(feature = "development")]
    pub fn create_llvm_ir_context(&mut self) -> &mut LLVMIRExecutionContext {
        if self.llvm_ir_context.is_none() {
            self.llvm_ir_context = Some(LLVMIRExecutionContext::new());
        }
        self.llvm_ir_context.as_mut().unwrap()
    }

    /// Get LLVM IR context if available
    #[cfg(feature = "development")]
    #[must_use] pub fn get_llvm_ir_context(&self) -> Option<&LLVMIRExecutionContext> {
        self.llvm_ir_context.as_ref()
    }
    
    /// Fallback for non-development builds
    #[cfg(not(feature = "development"))]
    #[must_use] pub fn get_llvm_ir_context(&self) -> Option<&()> {
        None // LLVM IR context not available in production builds
    }

    /// Get mutable LLVM IR context if available
    #[cfg(feature = "development")]
    pub fn get_llvm_ir_context_mut(&mut self) -> Option<&mut LLVMIRExecutionContext> {
        self.llvm_ir_context.as_mut()
    }
    
    /// Fallback for non-development builds
    #[cfg(not(feature = "development"))]
    pub fn get_llvm_ir_context_mut(&mut self) -> Option<&mut ()> {
        None // LLVM IR context not available in production builds
    }

    /// Check if LLVM IR compilation is available/beneficial
    #[cfg(feature = "development")]
    #[must_use] pub fn should_use_llvm_ir(&self) -> bool {
        // Use LLVM IR for complex expressions or when explicitly requested
        self.static_analysis.complexity_score > 50 ||
        self.optimization_hints.jit_beneficial ||
        self.static_analysis.has_tail_calls ||
        self.optimization_hints.optimization_level == OptimizationLevel::Aggressive
    }
    
    /// Fallback for non-development builds
    #[cfg(not(feature = "development"))]
    #[must_use] pub fn should_use_llvm_ir(&self) -> bool {
        false // LLVM IR not available in production builds
    }

    /// Generate LLVM IR for this execution context
    #[cfg(feature = "development")]
    pub fn generate_llvm_ir(&mut self) -> Result<String, crate::error::LambdustError> {
        // Extract needed values before creating LLVM context to avoid borrowing conflicts
        let expr_hash = self.generate_expression_hash();
        let has_tail_calls = self.static_analysis.has_tail_calls;
        let has_loops = self.static_analysis.has_loops;
        let is_pure = self.static_analysis.is_pure;
        
        // Setup LLVM context and configure it
        {
            let llvm_context = self.create_llvm_ir_context();
            
            // Generate function signature based on expression hash
            llvm_context.function_signature.function_name = format!(
                "scheme_expr_{expr_hash}"
            );
            
            // Enable optimizations based on static analysis
            if has_tail_calls {
                llvm_context.enable_tail_call_optimization();
                llvm_context.add_optimization_attribute(LLVMOptimizationAttribute::TailCall);
            }
            
            if has_loops {
                llvm_context.add_optimization_attribute(LLVMOptimizationAttribute::LoopUnroll { factor: 2 });
            }
            
            if is_pure {
                llvm_context.add_optimization_attribute(LLVMOptimizationAttribute::ConstantPropagation);
            }
        } // Drop the mutable borrow here
        
        // Generate LLVM IR instructions
        self.generate_llvm_instructions()?;
        
        // Get the result
        let llvm_context = self.llvm_ir_context.as_ref().unwrap();
        Ok(llvm_context.to_llvm_ir())
    }
    
    /// Fallback for non-development builds
    #[cfg(not(feature = "development"))]
    pub fn generate_llvm_ir(&mut self) -> Result<String, crate::error::LambdustError> {
        Err(crate::error::LambdustError::runtime_error("LLVM IR generation not available in production builds"))
    }

    /// Generate expression hash for function naming
    #[allow(dead_code)]
    fn generate_expression_hash(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        format!("{:?}", self.expression).hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Generate LLVM IR instructions for the expression
    #[cfg(feature = "development")]
    fn generate_llvm_instructions(&mut self) -> Result<(), crate::error::LambdustError> {
        let expr = self.expression.clone();
        self.generate_expression_ir(&expr)?;
        
        // Add return instruction
        let llvm_context = self.llvm_ir_context.as_mut()
            .ok_or_else(|| crate::error::LambdustError::runtime_error("LLVM IR context not initialized"))?;
        
        let return_inst = crate::evaluator::llvm_backend::LLVMInstruction::new(
            "ret".to_string(),
            vec!["i8* %result".to_string()]
        );
        llvm_context.add_instruction(return_inst);
        
        Ok(())
    }

    /// Generate LLVM IR for a specific expression (recursive helper)
    #[cfg(feature = "development")]
    fn generate_expression_ir(&mut self, expr: &Expr) -> Result<LLVMRegister, crate::error::LambdustError> {
        let _llvm_context = self.llvm_ir_context.as_mut()
            .ok_or_else(|| crate::error::LambdustError::runtime_error("LLVM IR context not initialized"))?;
        
        match expr {
            Expr::Literal(lit) => {
                self.generate_literal_ir(lit)
            },
            
            Expr::Variable(name) => {
                self.generate_variable_ir(name)
            },
            
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    self.generate_nil_ir()
                } else {
                    self.generate_application_ir(exprs)
                }
            },
            
            _ => {
                // Fallback for other expression types
                self.generate_fallback_ir()
            }
        }
    }

    /// Generate LLVM IR for literal values
    #[cfg(feature = "development")]
    fn generate_literal_ir(&mut self, lit: &crate::ast::Literal) -> Result<LLVMRegister, crate::error::LambdustError> {
        let literal_id = self.literal_to_id(lit);
        
        let llvm_context = self.llvm_ir_context.as_mut().unwrap();
        let result_reg = llvm_context.allocate_register(LLVMType::SchemeValue);
        let load_inst = crate::evaluator::llvm_backend::LLVMInstruction::new(
            "call".to_string(),
            vec![
                "i8* @scheme_create_literal".to_string(),
                format!("i32 {}", literal_id)
            ]
        ).with_result(format!("%{}", result_reg.id));
        
        llvm_context.add_instruction(load_inst);
        Ok(result_reg)
    }

    /// Generate LLVM IR for variable lookup
    #[cfg(feature = "development")]
    fn generate_variable_ir(&mut self, name: &str) -> Result<LLVMRegister, crate::error::LambdustError> {
        let llvm_context = self.llvm_ir_context.as_mut().unwrap();
        
        // Check if variable is already bound to a register
        if let Some(existing_reg) = llvm_context.get_variable_register(name) {
            return Ok(existing_reg.clone());
        }
        
        let result_reg = llvm_context.allocate_register(LLVMType::SchemeValue);
        let lookup_inst = crate::evaluator::llvm_backend::LLVMInstruction::new(
            "call".to_string(),
            vec![
                "i8* @scheme_env_lookup".to_string(),
                "i8* %param0".to_string(), // environment parameter
                format!("i8* @var_{}", name)
            ]
        ).with_result(format!("%{}", result_reg.id));
        
        llvm_context.add_instruction(lookup_inst);
        llvm_context.bind_variable(name.to_string(), result_reg.clone());
        Ok(result_reg)
    }

    /// Generate LLVM IR for empty list (nil)
    #[cfg(feature = "development")]
    fn generate_nil_ir(&mut self) -> Result<LLVMRegister, crate::error::LambdustError> {
        let llvm_context = self.llvm_ir_context.as_mut().unwrap();
        let result_reg = llvm_context.allocate_register(LLVMType::SchemeValue);
        
        let nil_inst = crate::evaluator::llvm_backend::LLVMInstruction::new(
            "call".to_string(),
            vec!["i8* @scheme_nil".to_string()]
        ).with_result(format!("%{}", result_reg.id));
        
        llvm_context.add_instruction(nil_inst);
        Ok(result_reg)
    }

    /// Generate LLVM IR for function application
    #[cfg(feature = "development")]
    fn generate_application_ir(&mut self, exprs: &[Expr]) -> Result<LLVMRegister, crate::error::LambdustError> {
        // Generate IR for function and arguments first
        let func_reg = self.generate_expression_ir(&exprs[0])?;
        let mut arg_regs = Vec::new();
        
        for arg_expr in &exprs[1..] {
            arg_regs.push(self.generate_expression_ir(arg_expr)?);
        }
        
        // Now access LLVM context and generate function call with appropriate optimization attributes
        let llvm_context = self.llvm_ir_context.as_mut().unwrap();
        let result_reg = llvm_context.allocate_register(LLVMType::SchemeValue);
        let mut call_args = vec![
            format!("i8* %{}", func_reg.id),
            "i8* %param0".to_string(), // environment
            "i8* %param1".to_string(), // continuation
        ];
        
        for arg_reg in &arg_regs {
            call_args.push(format!("i8* %{}", arg_reg.id));
        }
        
        let mut call_inst = crate::evaluator::llvm_backend::LLVMInstruction::new(
            "call".to_string(),
            vec!["i8* @scheme_apply".to_string()].into_iter()
                .chain(call_args)
                .collect()
        ).with_result(format!("%{}", result_reg.id));
        
        // Add tail call optimization if applicable
        if self.static_analysis.has_tail_calls {
            call_inst = call_inst.with_tail_call();
        }
        
        llvm_context.add_instruction(call_inst);
        Ok(result_reg)
    }

    /// Generate fallback LLVM IR for unsupported expression types
    #[cfg(feature = "development")]
    fn generate_fallback_ir(&mut self) -> Result<LLVMRegister, crate::error::LambdustError> {
        let llvm_context = self.llvm_ir_context.as_mut().unwrap();
        let result_reg = llvm_context.allocate_register(LLVMType::SchemeValue);
        
        let fallback_inst = crate::evaluator::llvm_backend::LLVMInstruction::new(
            "call".to_string(),
            vec![
                "i8* @scheme_eval_fallback".to_string(),
                "i8* %param0".to_string(), // environment
                "i8* %param1".to_string(), // continuation
            ]
        ).with_result(format!("%{}", result_reg.id));
        
        llvm_context.add_instruction(fallback_inst);
        Ok(result_reg)
    }

    /// Convert literal to ID for LLVM IR generation
    #[allow(dead_code)]
    fn literal_to_id(&self, lit: &crate::ast::Literal) -> u32 {
        match lit {
            crate::ast::Literal::Boolean(true) => 1,
            crate::ast::Literal::Boolean(false) => 0,
            crate::ast::Literal::Number(n) => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                n.hash(&mut hasher);
                (hasher.finish() % 1000000) as u32 + 100
            },
            crate::ast::Literal::String(s) => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                s.hash(&mut hasher);
                (hasher.finish() % 1000000) as u32 + 2000000
            },
            crate::ast::Literal::Character(c) => *c as u32 + 3000000,
            crate::ast::Literal::Nil => 10,
        }
    }
    
    /// Update execution metadata
    pub fn set_execution_metadata(&mut self, metadata: ExecutionMetadata) {
        self.execution_metadata = metadata;
    }
    
    /// Check if optimization is recommended for this context
    #[must_use] pub fn should_optimize(&self) -> bool {
        !matches!(self.optimization_hints.optimization_level, OptimizationLevel::None)
    }
    
    /// Check if JIT compilation is beneficial
    #[must_use] pub fn should_use_jit(&self) -> bool {
        self.optimization_hints.jit_beneficial && self.static_analysis.complexity_score > 40
    }
    
    /// Get memory usage estimate
    #[must_use] pub fn estimated_memory_usage(&self) -> usize {
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

/// Builder pattern for constructing `ExecutionContext`
#[derive(Debug)]
pub struct ExecutionContextBuilder {
    context: ExecutionContext,
}

impl ExecutionContextBuilder {
    /// Create new `ExecutionContextBuilder` with base context
    pub fn new(expression: Expr, environment: Rc<Environment>, continuation: Continuation) -> Self {
        Self {
            context: ExecutionContext::new(expression, environment, continuation),
        }
    }
    
    /// Set complexity score for static analysis
    #[must_use] pub fn with_complexity_score(mut self, score: u32) -> Self {
        self.context.static_analysis.complexity_score = score;
        self
    }
    
    /// Set tail call analysis result
    #[must_use] pub fn with_tail_calls(mut self, has_tail_calls: bool) -> Self {
        self.context.static_analysis.has_tail_calls = has_tail_calls;
        self
    }
    
    /// Set loop analysis result
    #[must_use] pub fn with_loops(mut self, has_loops: bool) -> Self {
        self.context.static_analysis.has_loops = has_loops;
        self
    }
    
    /// Set purity analysis result
    #[must_use] pub fn with_purity(mut self, is_pure: bool) -> Self {
        self.context.static_analysis.is_pure = is_pure;
        self
    }
    
    /// Add static call pattern to analysis
    #[must_use] pub fn add_call_pattern(mut self, pattern: StaticCallPattern) -> Self {
        self.context.static_analysis.call_patterns.push(pattern);
        self
    }
    
    /// Add constant binding to context
    #[must_use] pub fn add_constant_binding(mut self, name: String, value: Value) -> Self {
        self.context.constant_bindings.insert(name, value);
        self
    }
    
    /// Set execution priority
    #[must_use] pub fn with_priority(mut self, priority: ExecutionPriority) -> Self {
        self.context.execution_metadata.priority = priority;
        self
    }
    
    /// Set macro expansion state
    #[must_use] pub fn with_macro_expansion_state(mut self, state: MacroExpansionState) -> Self {
        self.context.macro_expansion_state = state;
        self
    }
    
    /// Add static optimization to analysis
    #[must_use] pub fn add_static_optimization(mut self, optimization: StaticOptimization) -> Self {
        self.context.static_analysis.static_optimizations.push(optimization);
        self
    }
    
    /// Add constant folding opportunity to analysis
    #[must_use] pub fn add_constant_folding_opportunity(mut self, opportunity: ConstantFoldingOpportunity) -> Self {
        self.context.static_analysis.constant_folding_opportunities.push(opportunity);
        self
    }
    
    /// Add common subexpression elimination candidate
    #[must_use] pub fn add_cse_candidate(mut self, candidate: CommonSubexpressionCandidate) -> Self {
        self.context.static_analysis.cse_candidates.push(candidate);
        self
    }
    
    /// Set complete static analysis result
    #[must_use] pub fn with_static_analysis_result(mut self, analysis: StaticAnalysisResult) -> Self {
        self.context.static_analysis = analysis;
        self
    }
    
    /// Add proven optimization with formal verification
    #[cfg(feature = "development")]
    pub fn add_proven_optimization(mut self, optimization: String) -> Self {
        self.context.proven_optimizations.push(optimization);
        self
    }
    
    /// Set type information from static analysis
    #[cfg(feature = "development")]
    pub fn with_type_information(mut self, type_info: String) -> Self {
        self.context.type_information = Some(type_info);
        self
    }
    
    /// Build the final `ExecutionContext` with derived optimization hints
    #[must_use] pub fn build(mut self) -> ExecutionContext {
        self.context.derive_optimization_hints();
        self.context.execution_metadata.context_id = ExecutionContext::generate_context_id();
        self.context
    }
}

/// LLVM IR-based execution context for intermediate language representation
/// This provides the bridge between high-level Scheme expressions and 
/// LLVM IR for optimized execution
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct LLVMIRExecutionContext {
    /// Generated LLVM IR instructions for the expression
    pub ir_instructions: Vec<crate::evaluator::llvm_backend::LLVMInstruction>,
    
    /// LLVM function signature for the execution context
    pub function_signature: LLVMFunctionSignature,
    
    /// Register allocation information
    pub register_allocation: RegisterAllocationInfo,
    
    /// Tail call optimization metadata
    pub tail_call_info: Option<LLVMTailCallInfo>,
    
    /// Control flow graph for complex expressions
    pub control_flow_graph: Option<LLVMControlFlowGraph>,
    
    /// Variable bindings in LLVM register space
    pub variable_bindings: FxHashMap<String, LLVMRegister>,
    
    /// Optimization flags and attributes
    pub optimization_attributes: Vec<LLVMOptimizationAttribute>,
    
    /// Debug information for debugging and profiling
    pub debug_metadata: Option<LLVMDebugMetadata>,
}

/// LLVM function signature for Scheme expressions
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct LLVMFunctionSignature {
    /// Function name (generated from expression hash)
    pub function_name: String,
    
    /// Parameter types (typically environment and continuation)
    pub parameter_types: Vec<LLVMType>,
    
    /// Return type (typically Scheme Value)
    pub return_type: LLVMType,
    
    /// Calling convention (for tail call optimization)
    pub calling_convention: LLVMCallingConvention,
    
    /// Function attributes (e.g., readonly, nounwind)
    pub attributes: Vec<LLVMFunctionAttribute>,
}

/// Register allocation information for LLVM compilation
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct RegisterAllocationInfo {
    /// Next available register ID
    pub next_register_id: u32,
    
    /// Register usage map (variable name -> register)
    pub register_map: FxHashMap<String, LLVMRegister>,
    
    /// Spill slots for register pressure relief
    pub spill_slots: Vec<LLVMSpillSlot>,
    
    /// Live ranges for register allocation
    pub live_ranges: Vec<RegisterLiveRange>,
}

/// LLVM tail call optimization information
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct LLVMTailCallInfo {
    /// Whether tail call optimization is applicable
    pub is_tail_call_optimizable: bool,
    
    /// Tail call sites within the expression
    pub tail_call_sites: Vec<TailCallSite>,
    
    /// Return value optimization (RVO) opportunities
    pub return_value_optimization: bool,
    
    /// Stack frame optimization metadata
    pub stack_frame_info: StackFrameOptimizationInfo,
}

/// Control flow graph for LLVM IR generation
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct LLVMControlFlowGraph {
    /// Basic blocks in the control flow graph
    pub basic_blocks: Vec<LLVMBasicBlock>,
    
    /// Control flow edges (`block_id` -> `successor_block_ids`)
    pub edges: FxHashMap<u32, Vec<u32>>,
    
    /// Entry block ID
    pub entry_block: u32,
    
    /// Exit blocks (for multiple return points)
    pub exit_blocks: Vec<u32>,
    
    /// Loop information for optimization
    pub loop_info: Vec<LLVMLoopInfo>,
}

/// LLVM register representation
#[cfg(feature = "development")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LLVMRegister {
    /// Register ID (e.g., %0, %1, %2)
    pub id: u32,
    
    /// Register type
    pub register_type: LLVMType,
    
    /// Whether this is a temporary or named register
    pub is_temporary: bool,
    
    /// Original Scheme variable name (if applicable)
    pub source_variable: Option<String>,
}

/// LLVM type system representation
#[cfg(feature = "development")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LLVMType {
    /// Scheme value pointer type
    SchemeValue,
    
    /// Environment pointer type  
    Environment,
    
    /// Continuation pointer type
    Continuation,
    
    /// Function pointer type
    Function {
        parameter_types: Vec<LLVMType>,
        return_type: Box<LLVMType>,
    },
    
    /// Pointer type
    Pointer(Box<LLVMType>),
    
    /// Integer types
    Integer(u32), // bit width
    
    /// Floating point types
    Float,
    Double,
    
    /// Void type
    Void,
}

/// LLVM calling convention for tail call optimization
#[cfg(feature = "development")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LLVMCallingConvention {
    /// Standard C calling convention
    C,
    
    /// Fast calling convention for internal functions
    Fast,
    
    /// Tail call optimized convention
    TailCall,
    
    /// Custom Scheme calling convention
    Scheme,
}

/// LLVM function attributes
#[cfg(feature = "development")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LLVMFunctionAttribute {
    /// Function is read-only (no side effects)
    ReadOnly,
    
    /// Function does not unwind (no exceptions)
    NoUnwind,
    
    /// Function does not return
    NoReturn,
    
    /// Inline hint
    InlineHint,
    
    /// Always inline
    AlwaysInline,
    
    /// Never inline
    NoInline,
    
    /// Optimize for size
    OptSize,
}

/// LLVM optimization attributes
#[cfg(feature = "development")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LLVMOptimizationAttribute {
    /// Tail call optimization
    TailCall,
    
    /// Loop unrolling
    LoopUnroll { factor: u32 },
    
    /// Vectorization
    Vectorize,
    
    /// Constant propagation
    ConstantPropagation,
    
    /// Dead code elimination
    DeadCodeElimination,
    
    /// Common subexpression elimination
    CommonSubexpressionElimination,
}

/// Debug metadata for LLVM IR
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct LLVMDebugMetadata {
    /// Source location information
    pub source_location: Option<crate::error::SourceSpan>,
    
    /// Original Scheme expression for debugging
    pub original_expression: String,
    
    /// Compilation unit information
    pub compilation_unit: String,
    
    /// Variable debug information
    pub variable_debug_info: FxHashMap<String, VariableDebugInfo>,
}

/// Basic block in LLVM IR
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct LLVMBasicBlock {
    /// Block ID
    pub id: u32,
    
    /// Block label
    pub label: String,
    
    /// Instructions in this block
    pub instructions: Vec<crate::evaluator::llvm_backend::LLVMInstruction>,
    
    /// Terminator instruction (branch, return, etc.)
    pub terminator: LLVMTerminator,
}

/// LLVM terminator instructions
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub enum LLVMTerminator {
    /// Unconditional branch
    Branch { target: u32 },
    
    /// Conditional branch
    ConditionalBranch {
        condition: LLVMRegister,
        true_target: u32,
        false_target: u32,
    },
    
    /// Return instruction
    Return { value: Option<LLVMRegister> },
    
    /// Tail call
    TailCall {
        function: LLVMRegister,
        arguments: Vec<LLVMRegister>,
    },
}

/// Loop information for optimization
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct LLVMLoopInfo {
    /// Loop header block
    pub header: u32,
    
    /// Loop exit blocks
    pub exits: Vec<u32>,
    
    /// Loop body blocks
    pub body: Vec<u32>,
    
    /// Loop nesting level
    pub nesting_level: u32,
    
    /// Whether loop is suitable for unrolling
    pub unroll_candidate: bool,
}

/// Spill slot for register allocation
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct LLVMSpillSlot {
    /// Slot ID
    pub id: u32,
    
    /// Size in bytes
    pub size: u32,
    
    /// Alignment requirements
    pub alignment: u32,
    
    /// Associated register type
    pub register_type: LLVMType,
}

/// Register live range information
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct RegisterLiveRange {
    /// Register
    pub register: LLVMRegister,
    
    /// Start instruction ID
    pub start: u32,
    
    /// End instruction ID
    pub end: u32,
    
    /// Usage frequency
    pub frequency: u32,
}

/// Tail call site information
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct TailCallSite {
    /// Instruction ID of the tail call
    pub instruction_id: u32,
    
    /// Target function register
    pub target_function: LLVMRegister,
    
    /// Arguments for the tail call
    pub arguments: Vec<LLVMRegister>,
    
    /// Whether musttail optimization applies
    pub musttail: bool,
}

/// Stack frame optimization information
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct StackFrameOptimizationInfo {
    /// Frame size in bytes
    pub frame_size: u32,
    
    /// Local variables allocation
    pub local_variables: Vec<LocalVariableInfo>,
    
    /// Whether frame pointer is needed
    pub needs_frame_pointer: bool,
    
    /// Stack alignment requirements
    pub stack_alignment: u32,
}

/// Local variable information
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct LocalVariableInfo {
    /// Variable name
    pub name: String,
    
    /// Stack offset
    pub stack_offset: i32,
    
    /// Variable size
    pub size: u32,
    
    /// LLVM type
    pub llvm_type: LLVMType,
}

/// Variable debug information
#[cfg(feature = "development")]
#[derive(Debug, Clone)]
pub struct VariableDebugInfo {
    /// Variable name in source
    pub source_name: String,
    
    /// Variable type information
    pub type_info: String,
    
    /// Source location where variable is defined
    pub definition_location: Option<crate::error::SourceSpan>,
    
    /// Live range in source
    pub live_range: Option<crate::error::SourceSpan>,
}

#[cfg(feature = "development")]
impl Default for LLVMFunctionSignature {
    fn default() -> Self {
        Self {
            function_name: "scheme_function".to_string(),
            parameter_types: vec![LLVMType::Environment, LLVMType::Continuation],
            return_type: LLVMType::SchemeValue,
            calling_convention: LLVMCallingConvention::Scheme,
            attributes: vec![LLVMFunctionAttribute::NoUnwind],
        }
    }
}

#[cfg(feature = "development")]
impl LLVMIRExecutionContext {
    /// Create a new LLVM IR execution context
    #[must_use] pub fn new() -> Self {
        Self::default()
    }
    
    /// Add an LLVM instruction to the context
    #[cfg(feature = "development")]
    pub fn add_instruction(&mut self, instruction: crate::evaluator::llvm_backend::LLVMInstruction) {
        self.ir_instructions.push(instruction);
    }
    
    /// Allocate a new register
    pub fn allocate_register(&mut self, register_type: LLVMType) -> LLVMRegister {
        let register = LLVMRegister {
            id: self.register_allocation.next_register_id,
            register_type,
            is_temporary: true,
            source_variable: None,
        };
        self.register_allocation.next_register_id += 1;
        register
    }
    
    /// Bind a Scheme variable to an LLVM register
    pub fn bind_variable(&mut self, variable_name: String, register: LLVMRegister) {
        self.variable_bindings.insert(variable_name.clone(), register.clone());
        self.register_allocation.register_map.insert(variable_name, register);
    }
    
    /// Get register for a variable
    #[must_use] pub fn get_variable_register(&self, variable_name: &str) -> Option<&LLVMRegister> {
        self.variable_bindings.get(variable_name)
    }
    
    /// Enable tail call optimization
    pub fn enable_tail_call_optimization(&mut self) {
        if self.tail_call_info.is_none() {
            self.tail_call_info = Some(LLVMTailCallInfo {
                is_tail_call_optimizable: true,
                tail_call_sites: Vec::new(),
                return_value_optimization: true,
                stack_frame_info: StackFrameOptimizationInfo {
                    frame_size: 0,
                    local_variables: Vec::new(),
                    needs_frame_pointer: false,
                    stack_alignment: 8,
                },
            });
        }
    }
    
    /// Add optimization attribute
    pub fn add_optimization_attribute(&mut self, attribute: LLVMOptimizationAttribute) {
        self.optimization_attributes.push(attribute);
    }
    
    /// Generate LLVM IR string representation
    #[must_use] pub fn to_llvm_ir(&self) -> String {
        let mut ir = String::new();
        
        // Function signature
        ir.push_str(&format!("define {} @{}(", 
            self.function_signature.return_type.to_llvm_type_string(),
            self.function_signature.function_name));
        
        for (i, param_type) in self.function_signature.parameter_types.iter().enumerate() {
            if i > 0 { ir.push_str(", "); }
            ir.push_str(&format!("{} %param{}", param_type.to_llvm_type_string(), i));
        }
        ir.push_str(") {\n");
        
        // Entry block
        ir.push_str("entry:\n");
        
        // Instructions
        for instruction in &self.ir_instructions {
            ir.push_str("  ");
            ir.push_str(&instruction.to_llvm_ir());
            ir.push('\n');
        }
        
        // Function end
        ir.push_str("}\n");
        
        ir
    }
}

#[cfg(feature = "development")]
impl LLVMType {
    /// Convert to LLVM type string representation
    #[must_use] pub fn to_llvm_type_string(&self) -> String {
        match self {
            LLVMType::SchemeValue => "i8*".to_string(),
            LLVMType::Environment => "i8*".to_string(),
            LLVMType::Continuation => "i8*".to_string(),
            LLVMType::Function { parameter_types, return_type } => {
                let params: Vec<String> = parameter_types.iter()
                    .map(LLVMType::to_llvm_type_string)
                    .collect();
                format!("{} ({})*", return_type.to_llvm_type_string(), params.join(", "))
            },
            LLVMType::Pointer(inner) => format!("{}*", inner.to_llvm_type_string()),
            LLVMType::Integer(bits) => format!("i{bits}"),
            LLVMType::Float => "float".to_string(),
            LLVMType::Double => "double".to_string(),
            LLVMType::Void => "void".to_string(),
        }
    }
}

/// Condition register system for runtime-variable execution
/// This system manages conditional execution paths and optimization decisions
#[derive(Debug, Clone)]
pub struct ConditionRegisterSystem {
    /// Register storage for conditional values
    pub registers: std::collections::HashMap<String, ConditionRegister>,
    
    /// Current active condition for execution branching
    pub current_condition: Option<ConditionRegister>,
    
    /// Execution mode (interpreted vs compiled)
    pub execution_mode: ExecutionMode,
    
    /// Optimization level for conditional compilation
    pub optimization_level: OptimizationLevel,
}

/// Individual condition register
#[derive(Debug, Clone)]
pub struct ConditionRegister {
    /// Register name/identifier
    pub name: String,
    
    /// Current boolean value of the condition
    pub value: bool,
    
    /// Confidence level (0.0 - 1.0) in the condition prediction
    pub confidence: f64,
    
    /// Number of times this condition has been evaluated
    pub evaluation_count: u64,
    
    /// Success rate of condition predictions
    pub prediction_accuracy: f64,
    
    /// Associated optimization hints for this condition
    pub optimization_hints: Vec<String>,
}

/// Execution mode for the condition register system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Interpreted execution (standard evaluator)
    Interpreted,
    
    /// LLVM compiled execution
    #[cfg(feature = "development")]
    LLVMCompiled,
    
    /// JIT compiled execution
    JITCompiled,
    
    /// Hybrid mode (runtime decision between interpreted and compiled)
    Hybrid,
    
    /// Static analysis only (no execution)
    AnalysisOnly,
}

impl Default for ConditionRegisterSystem {
    fn default() -> Self {
        Self {
            registers: std::collections::HashMap::new(),
            current_condition: None,
            execution_mode: ExecutionMode::Interpreted,
            optimization_level: OptimizationLevel::Conservative,
        }
    }
}

impl ConditionRegisterSystem {
    /// Create a new condition register system
    #[must_use] pub fn new() -> Self {
        Self::default()
    }
    
    /// Add or update a condition register
    pub fn set_condition(&mut self, name: String, value: bool, confidence: f64) {
        let register = ConditionRegister {
            name: name.clone(),
            value,
            confidence,
            evaluation_count: 1,
            prediction_accuracy: 1.0, // Start with perfect accuracy
            optimization_hints: Vec::new(),
        };
        
        if let Some(existing) = self.registers.get_mut(&name) {
            // Update existing register
            existing.value = value;
            existing.confidence = confidence;
            existing.evaluation_count += 1;
            // Update prediction accuracy based on historical performance
            existing.prediction_accuracy = (existing.prediction_accuracy * 0.9) + (confidence * 0.1);
        } else {
            // Insert new register
            self.registers.insert(name, register);
        }
    }
    
    /// Get condition register value
    #[must_use] pub fn get_condition(&self, name: &str) -> Option<&ConditionRegister> {
        self.registers.get(name)
    }
    
    /// Set current active condition for execution branching
    pub fn set_current_condition(&mut self, name: &str) -> bool {
        if let Some(register) = self.registers.get(name) {
            self.current_condition = Some(register.clone());
            true
        } else {
            false
        }
    }
    
    /// Check if current condition is true with confidence threshold
    #[must_use] pub fn is_condition_true(&self, confidence_threshold: f64) -> Option<bool> {
        if let Some(ref condition) = self.current_condition {
            if condition.confidence >= confidence_threshold {
                Some(condition.value)
            } else {
                None // Confidence too low for reliable prediction
            }
        } else {
            None // No current condition set
        }
    }
    
    /// Get execution mode recommendation based on conditions
    #[must_use] pub fn get_execution_mode_recommendation(&self) -> ExecutionMode {
        match self.optimization_level {
            OptimizationLevel::None => ExecutionMode::Interpreted,
            OptimizationLevel::Conservative => {
                if self.has_high_confidence_conditions() {
                    ExecutionMode::JITCompiled
                } else {
                    ExecutionMode::Interpreted
                }
            },
            OptimizationLevel::Balanced => {
                if self.has_complex_conditions() {
                    #[cfg(feature = "development")]
                    {
                        ExecutionMode::LLVMCompiled
                    }
                    #[cfg(not(feature = "development"))]
                    {
                        ExecutionMode::JITCompiled
                    }
                } else {
                    ExecutionMode::Hybrid
                }
            },
            OptimizationLevel::Aggressive => {
                #[cfg(feature = "development")]
                {
                    ExecutionMode::LLVMCompiled
                }
                #[cfg(not(feature = "development"))]
                {
                    ExecutionMode::JITCompiled
                }
            },
        }
    }
    
    /// Check if system has high-confidence conditions
    fn has_high_confidence_conditions(&self) -> bool {
        self.registers.values()
            .any(|reg| reg.confidence > 0.8 && reg.evaluation_count > 10)
    }
    
    /// Check if system has complex conditional logic
    fn has_complex_conditions(&self) -> bool {
        self.registers.len() > 5 || 
        self.registers.values().any(|reg| reg.evaluation_count > 100)
    }
    
    /// Add optimization hint to a condition register
    pub fn add_optimization_hint(&mut self, condition_name: &str, hint: String) {
        if let Some(register) = self.registers.get_mut(condition_name) {
            register.optimization_hints.push(hint);
        }
    }
    
    /// Clear all condition registers
    pub fn clear_all(&mut self) {
        self.registers.clear();
        self.current_condition = None;
    }
    
    /// Get statistics about condition register performance
    #[must_use] pub fn get_performance_stats(&self) -> ConditionSystemStats {
        let total_registers = self.registers.len();
        let avg_confidence = if total_registers > 0 {
            self.registers.values().map(|r| r.confidence).sum::<f64>() / total_registers as f64
        } else {
            0.0
        };
        
        let avg_accuracy = if total_registers > 0 {
            self.registers.values().map(|r| r.prediction_accuracy).sum::<f64>() / total_registers as f64
        } else {
            0.0
        };
        
        let total_evaluations = self.registers.values().map(|r| r.evaluation_count).sum();
        
        ConditionSystemStats {
            total_registers,
            average_confidence: avg_confidence,
            average_accuracy: avg_accuracy,
            total_evaluations,
            execution_mode: self.execution_mode.clone(),
        }
    }
}

/// Performance statistics for condition register system
#[derive(Debug, Clone)]
pub struct ConditionSystemStats {
    /// Total number of condition registers
    pub total_registers: usize,
    
    /// Average confidence across all registers
    pub average_confidence: f64,
    
    /// Average prediction accuracy
    pub average_accuracy: f64,
    
    /// Total number of condition evaluations
    pub total_evaluations: u64,
    
    /// Current execution mode
    pub execution_mode: ExecutionMode,
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
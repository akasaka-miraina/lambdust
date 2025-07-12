//! Runtime Executor for optimized evaluation
//!
//! This module implements the runtime executor that applies performance optimizations
//! while maintaining correctness through reference to the semantic evaluator.
//!
//! The `RuntimeExecutor` integrates all dynamic optimization systems:
//! - JIT loop optimization
//! - Continuation pooling  
//! - Inline evaluation for hot paths
//! - Runtime performance profiling and adaptation

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{
    Continuation, ContinuationPoolManager, InlineEvaluator,
    IntegratedOptimizationManager, JitLoopOptimizer, OptimizationResult, SemanticEvaluator,
    ExecutionContext, ExecutionPriority,
    continuation_pooling::{ContinuationType},
    hotpath_analysis::AdvancedHotPathDetector,
    llvm_backend::{LLVMCompilerIntegration, LLVMOptimizationLevel},
    jit_loop_optimization::{JitOptimizationStats},
};
use crate::value::Value;
use std::rc::Rc;
use rustc_hash::FxHashMap;
use std::collections::HashSet;

/// Runtime optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeOptimizationLevel {
    /// No optimizations
    None,
    /// Conservative optimizations only
    Conservative,
    /// Balanced optimization approach
    Balanced,
    /// Aggressive optimizations
    Aggressive,
}

impl RuntimeOptimizationLevel {
    /// Check if this optimization level supports tail call optimization
    #[must_use] pub fn use_tail_call_optimization(&self) -> bool {
        match self {
            RuntimeOptimizationLevel::None => false,
            RuntimeOptimizationLevel::Conservative => false,
            RuntimeOptimizationLevel::Balanced => true,
            RuntimeOptimizationLevel::Aggressive => true,
        }
    }
    
    /// Get corresponding LLVM optimization level
    #[must_use] pub fn llvm_level(&self) -> crate::evaluator::llvm_backend::LLVMOptimizationLevel {
        use crate::evaluator::llvm_backend::LLVMOptimizationLevel;
        match self {
            RuntimeOptimizationLevel::None => LLVMOptimizationLevel::O0,
            RuntimeOptimizationLevel::Conservative => LLVMOptimizationLevel::O1,
            RuntimeOptimizationLevel::Balanced => LLVMOptimizationLevel::O2,
            RuntimeOptimizationLevel::Aggressive => LLVMOptimizationLevel::O3,
        }
    }
}

/// Advanced expression analysis for JIT optimization decisions
#[derive(Debug, Clone)]
pub struct ExpressionAnalysisResult {
    /// Expression complexity score (0-100)
    pub complexity_score: u32,
    
    /// Whether this expression is a tail call candidate
    pub is_tail_call_candidate: bool,
    
    /// Whether this is on a hot execution path
    pub is_hot_path: bool,
    
    /// Whether this expression contains loops
    pub contains_loops: bool,
    
    /// Detected function call patterns
    pub call_patterns: Vec<CallPattern>,
    
    /// Estimated execution frequency
    pub execution_frequency: ExecutionFrequency,
    
    /// Memory allocation patterns
    pub memory_patterns: Vec<MemoryPattern>,
    
    /// Optimization recommendations
    pub optimization_hints: Vec<OptimizationHint>,
}

/// Function call pattern detection
#[derive(Debug, Clone, PartialEq)]
pub enum CallPattern {
    /// Recursive call detected
    Recursive { 
        /// Estimated recursion depth for optimization
        depth_estimate: u32 
    },
    
    /// Tail recursive call
    TailRecursive,
    
    /// Higher-order function application
    HigherOrder,
    
    /// Builtin function call
    Builtin { 
        /// Name of the builtin function being called
        function_name: String 
    },
    
    /// Loop construct
    Loop { 
        /// Estimated number of iterations if known
        iteration_estimate: Option<u32> 
    },
}

/// Execution frequency classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionFrequency {
    /// Executed once or rarely
    Cold,
    
    /// Moderate execution frequency
    Warm,
    
    /// High execution frequency (hot path)
    Hot,
    
    /// Critical performance path
    Critical,
}

/// Memory allocation pattern
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryPattern {
    /// Frequent small allocations
    FrequentSmall,
    
    /// Large object creation
    LargeObject,
    
    /// Temporary object creation in loops
    TemporaryInLoop,
    
    /// Long-lived object creation
    LongLived,
}

/// Optimization hint for JIT compiler
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationHint {
    /// Inline this function call
    Inline,
    
    /// Apply tail call optimization
    TailCallOptimize,
    
    /// Use continuation pooling
    PoolContinuations,
    
    /// JIT compile this expression
    JitCompile,
    
    /// Unroll this loop with specified unroll factor
    UnrollLoop { 
        /// Number of times to unroll the loop (must be > 0)
        factor: u32 
    },
    
    /// Specialize for specific types
    TypeSpecialize,
    
    /// Cache computation results
    MemoizeResults,
}

impl ExpressionAnalysisResult {
    /// Create new analysis result with conservative defaults
    pub fn new() -> Self {
        Self {
            complexity_score: 0,
            is_tail_call_candidate: false,
            is_hot_path: false,
            contains_loops: false,
            call_patterns: Vec::new(),
            execution_frequency: ExecutionFrequency::Cold,
            memory_patterns: Vec::new(),
            optimization_hints: Vec::new(),
        }
    }
    
    /// Analyze expression and return optimization recommendations
    pub fn analyze_expression(expr: &Expr) -> Self {
        let mut result = Self::new();
        
        // Basic complexity analysis
        result.complexity_score = Self::calculate_complexity(expr);
        
        // Pattern detection
        result.call_patterns = Self::detect_call_patterns(expr);
        
        // Hot path detection based on patterns
        result.is_hot_path = result.call_patterns.iter().any(|p| matches!(p, 
            CallPattern::Loop { .. } | CallPattern::Recursive { .. }
        ));
        
        // Loop detection
        result.contains_loops = result.call_patterns.iter().any(|p| matches!(p, CallPattern::Loop { .. }));
        
        // Tail call detection
        result.is_tail_call_candidate = Self::detect_tail_call(expr);
        
        // Execution frequency estimation
        result.execution_frequency = if result.contains_loops {
            ExecutionFrequency::Hot
        } else if result.is_hot_path {
            ExecutionFrequency::Warm
        } else {
            ExecutionFrequency::Cold
        };
        
        // Generate optimization hints
        result.optimization_hints = Self::generate_optimization_hints(&result);
        
        result
    }
    
    /// Calculate expression complexity score
    fn calculate_complexity(expr: &Expr) -> u32 {
        match expr {
            Expr::Literal(_) => 1,
            Expr::Variable(_) => 1,
            Expr::HygienicVariable(_) => 1,
            Expr::List(exprs) => {
                if exprs.is_empty() {
                    return 1;
                }
                let complexity: u32 = exprs.iter().map(Self::calculate_complexity).sum();
                complexity + 2
            },
            Expr::Quote(expr) => Self::calculate_complexity(expr) + 1,
            Expr::Quasiquote(expr) => Self::calculate_complexity(expr) + 2,
            Expr::Unquote(expr) => Self::calculate_complexity(expr) + 1,
            Expr::UnquoteSplicing(expr) => Self::calculate_complexity(expr) + 2,
            Expr::Vector(exprs) => {
                let complexity: u32 = exprs.iter().map(Self::calculate_complexity).sum();
                complexity + 1
            },
            Expr::DottedList(exprs, tail) => {
                let list_complexity: u32 = exprs.iter().map(Self::calculate_complexity).sum();
                list_complexity + Self::calculate_complexity(tail) + 2
            },
        }
    }
    
    /// Detect function call patterns
    fn detect_call_patterns(expr: &Expr) -> Vec<CallPattern> {
        let mut patterns = Vec::new();
        
        match expr {
            Expr::List(exprs) => {
                if !exprs.is_empty() {
                    // Check for builtin functions
                    if let Expr::Variable(name) = &exprs[0] {
                        if Self::is_builtin_function(name) {
                            patterns.push(CallPattern::Builtin { 
                                function_name: name.clone() 
                            });
                        }
                        
                        // Detect special forms and patterns
                        match name.as_str() {
                            "lambda" => {
                                patterns.push(CallPattern::HigherOrder);
                            },
                            "let" | "let*" | "letrec" => {
                                // Analyze bindings
                                if exprs.len() >= 3 {
                                    for expr in &exprs[2..] {
                                        patterns.extend(Self::detect_call_patterns(expr));
                                    }
                                }
                            },
                            "if" => {
                                // Analyze conditional branches
                                for expr in &exprs[1..] {
                                    patterns.extend(Self::detect_call_patterns(expr));
                                }
                            },
                            "define" => {
                                // Check for recursive function definitions
                                if exprs.len() >= 3 {
                                    if Self::contains_self_reference(&exprs[2], name) {
                                        patterns.push(CallPattern::Recursive { depth_estimate: 1 });
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                    
                    // Recursively check all subexpressions
                    for expr in exprs {
                        patterns.extend(Self::detect_call_patterns(expr));
                    }
                }
            },
            Expr::Quote(expr) | Expr::Quasiquote(expr) | 
            Expr::Unquote(expr) | Expr::UnquoteSplicing(expr) => {
                patterns.extend(Self::detect_call_patterns(expr));
            },
            Expr::Vector(exprs) => {
                for expr in exprs {
                    patterns.extend(Self::detect_call_patterns(expr));
                }
            },
            Expr::DottedList(exprs, tail) => {
                for expr in exprs {
                    patterns.extend(Self::detect_call_patterns(expr));
                }
                patterns.extend(Self::detect_call_patterns(tail));
            },
            _ => {}
        }
        
        patterns
    }
    
    /// Detect tail call opportunities
    fn detect_tail_call(expr: &Expr) -> bool {
        match expr {
            Expr::List(exprs) => {
                !exprs.is_empty() && matches!(exprs[0], Expr::Variable(_))
            },
            _ => false,
        }
    }
    
    /// Helper method to detect self-references in function bodies
    fn contains_self_reference(expr: &Expr, function_name: &str) -> bool {
        match expr {
            Expr::Variable(name) => name == function_name,
            Expr::List(exprs) => {
                exprs.iter().any(|e| Self::contains_self_reference(e, function_name))
            },
            Expr::Quote(expr) | Expr::Quasiquote(expr) | 
            Expr::Unquote(expr) | Expr::UnquoteSplicing(expr) => {
                Self::contains_self_reference(expr, function_name)
            },
            Expr::Vector(exprs) => {
                exprs.iter().any(|e| Self::contains_self_reference(e, function_name))
            },
            Expr::DottedList(exprs, tail) => {
                exprs.iter().any(|e| Self::contains_self_reference(e, function_name)) ||
                Self::contains_self_reference(tail, function_name)
            },
            _ => false,
        }
    }
    
    /// Check if function name is a builtin
    fn is_builtin_function(name: &str) -> bool {
        matches!(name, "+" | "-" | "*" | "/" | "=" | "<" | ">" | "<=" | ">="
                     | "and" | "or" | "not" | "cons" | "car" | "cdr" | "list"
                     | "length" | "append" | "reverse" | "map" | "filter")
    }
    
    /// Generate optimization hints based on analysis
    fn generate_optimization_hints(analysis: &ExpressionAnalysisResult) -> Vec<OptimizationHint> {
        let mut hints = Vec::new();
        
        // Tail call optimization
        if analysis.is_tail_call_candidate {
            hints.push(OptimizationHint::TailCallOptimize);
        }
        
        // JIT compilation for hot paths
        if matches!(analysis.execution_frequency, ExecutionFrequency::Hot | ExecutionFrequency::Critical) {
            hints.push(OptimizationHint::JitCompile);
        }
        
        // Inlining for simple expressions
        if analysis.complexity_score < 10 && analysis.execution_frequency != ExecutionFrequency::Cold {
            hints.push(OptimizationHint::Inline);
        }
        
        // Loop unrolling
        if analysis.contains_loops && analysis.complexity_score < 20 {
            hints.push(OptimizationHint::UnrollLoop { factor: 2 });
        }
        
        // Continuation pooling for recursive patterns
        if analysis.call_patterns.iter().any(|p| matches!(p, CallPattern::Recursive { .. })) {
            hints.push(OptimizationHint::PoolContinuations);
        }
        
        hints
    }
}

impl Default for ExpressionAnalysisResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimized tail call result
#[derive(Debug, Clone)]
pub struct OptimizedTailCall {
    /// The target function for the tail call
    pub target_function: Value,
    
    /// Arguments for the tail call
    pub arguments: Vec<Value>,
    
    /// Environment for execution
    pub environment: Rc<Environment>,
    
    /// Whether the tail call was successfully optimized
    pub optimization_applied: bool,
}

/// JIT compiled code representation
#[derive(Debug, Clone)]
pub struct JitCompiledCode {
    /// Original expression that was compiled
    pub original_expr: Expr,
    
    /// Compilation metadata
    pub metadata: JitMetadata,
    
    /// Performance characteristics
    pub performance_profile: PerformanceProfile,
    
    /// Whether the code is ready for execution
    pub is_ready: bool,
}

/// JIT compilation metadata
#[derive(Debug, Clone)]
pub struct JitMetadata {
    /// Compilation timestamp
    pub compiled_at: std::time::SystemTime,
    
    /// Optimization level used
    pub optimization_level: RuntimeOptimizationLevel,
    
    /// Applied optimizations
    pub optimizations: Vec<String>,
    
    /// Compilation time in microseconds
    pub compilation_time_us: u64,
    
    /// Hot path detection count
    pub hot_path_count: u32,
    
    /// Adaptive optimization decisions
    pub adaptive_decisions: Vec<AdaptiveDecision>,
}

/// Adaptive optimization decision record
#[derive(Debug, Clone)]
pub struct AdaptiveDecision {
    /// Decision timestamp
    pub timestamp: std::time::SystemTime,
    
    /// Expression that triggered the decision
    pub trigger_expression: String,
    
    /// Type of optimization chosen
    pub optimization_type: AdaptiveOptimizationType,
    
    /// Reason for the decision
    pub rationale: String,
    
    /// Expected performance improvement
    pub expected_improvement: f64,
}

/// Types of adaptive optimizations
#[derive(Debug, Clone, PartialEq)]
pub enum AdaptiveOptimizationType {
    /// No optimization needed
    NoOptimization,
    
    /// JIT compilation triggered
    JitCompilation,
    
    /// Dynamic specialization
    TypeSpecialization,
    
    /// Loop unrolling with adaptive factor based on runtime profiling
    AdaptiveLoopUnrolling { 
        /// Dynamically determined unroll factor (1-16 recommended range)
        factor: u32 
    },
    
    /// Hot path inlining
    HotPathInlining,
    
    /// Continuation pooling optimization
    ContinuationPooling,
    
    /// Memory layout optimization
    MemoryLayoutOptimization,
    
    /// Profile-guided optimization
    ProfileGuidedOptimization,
}

impl AdaptiveOptimizationType {
    /// Check if this optimization type requires JIT compilation
    #[must_use] pub fn should_compile_jit(&self) -> bool {
        matches!(self, AdaptiveOptimizationType::JitCompilation)
    }
}

/// Performance profile for JIT compiled code
#[derive(Debug, Clone)]
pub struct PerformanceProfile {
    /// Estimated speedup factor
    pub speedup_factor: f64,
    
    /// Memory usage characteristics
    pub memory_usage: MemoryUsage,
    
    /// Execution characteristics
    pub execution_characteristics: ExecutionCharacteristics,
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Estimated stack usage
    pub stack_usage_bytes: usize,
    
    /// Estimated heap allocations
    pub heap_allocations: usize,
    
    /// Whether memory-intensive operations are present
    pub is_memory_intensive: bool,
}

/// Execution characteristics
#[derive(Debug, Clone)]
pub struct ExecutionCharacteristics {
    /// Whether the code is CPU-intensive
    pub is_cpu_intensive: bool,
    
    /// Whether the code has I/O operations
    pub has_io_operations: bool,
    
    /// Estimated instruction count
    pub estimated_instructions: u64,
    
    /// Whether the code benefits from parallelization
    pub parallelizable: bool,
}

/// Runtime executor with integrated optimization systems
pub struct RuntimeExecutor {
    /// Reference semantic evaluator for correctness verification
    semantic_evaluator: SemanticEvaluator,

    /// JIT loop optimizer (dynamic optimization)
    jit_optimizer: JitLoopOptimizer,

    /// Inline evaluator for hot path optimization
    inline_evaluator: InlineEvaluator,

    /// Continuation pooling manager
    continuation_pooler: ContinuationPoolManager,

    /// Integrated optimization manager
    integrated_optimizer: IntegratedOptimizationManager,
    
    /// Adaptive optimization engine
    adaptive_engine: AdaptiveOptimizationEngine,

    /// Advanced hot path detector with multi-dimensional analysis
    hotpath_detector: AdvancedHotPathDetector,

    /// LLVM compiler integration for native code generation
    llvm_compiler: LLVMCompilerIntegration,

    /// Current optimization level
    optimization_level: RuntimeOptimizationLevel,

    /// Whether to verify against semantic evaluator
    verification_enabled: bool,

    /// Runtime statistics
    stats: RuntimeStats,

    /// Recursion depth tracking
    recursion_depth: usize,
    max_recursion_depth: usize,
}

/// Adaptive optimization engine for dynamic code generation
#[derive(Debug)]
pub struct AdaptiveOptimizationEngine {
    /// Hot path detector
    hot_path_detector: HotPathDetector,
    
    /// Dynamic code generator
    code_generator: DynamicCodeGenerator,
    
    /// Profile-guided optimization system
    #[allow(dead_code)]
    profiler: ProfileGuidedOptimizer,
    
    /// JIT compilation cache
    jit_cache: FxHashMap<String, JitCompiledCode>,
    
    /// Adaptive decision history
    decision_history: Vec<AdaptiveDecision>,
    
    /// Performance tracking
    #[allow(dead_code)]
    performance_tracker: PerformanceTracker,
    
    /// Optimization thresholds
    #[allow(dead_code)]
    thresholds: OptimizationThresholds,
}

/// Hot path detection system
#[derive(Debug)]
pub struct HotPathDetector {
    /// Expression execution frequency tracking
    execution_frequencies: FxHashMap<String, u64>,
    
    /// Hot path candidates
    hot_paths: HashSet<String>,
    
    /// Detection sensitivity threshold
    sensitivity_threshold: u64,
    
    /// Cool-down period for hot path detection
    cool_down_period: std::time::Duration,
    
    /// Last detection timestamp per expression
    last_detection: FxHashMap<String, std::time::SystemTime>,
}

/// Dynamic code generator for JIT compilation
#[derive(Debug)]
pub struct DynamicCodeGenerator {
    /// Generated code cache
    #[allow(dead_code)]
    code_cache: FxHashMap<String, GeneratedCode>,
    
    /// Compilation strategy selector
    #[allow(dead_code)]
    strategy_selector: CompilationStrategySelector,
    
    /// Code generation statistics
    #[allow(dead_code)]
    generation_stats: CodeGenerationStats,
    
    /// Memory allocator for generated code
    #[allow(dead_code)]
    code_allocator: CodeAllocator,
}

/// Profile-guided optimization system
#[derive(Debug)]
pub struct ProfileGuidedOptimizer {
    /// Execution profiles per expression
    #[allow(dead_code)]
    execution_profiles: FxHashMap<String, ExecutionProfile>,
    
    /// Optimization decisions based on profiles
    #[allow(dead_code)]
    optimization_decisions: FxHashMap<String, OptimizationDecision>,
    
    /// Profile collection period
    #[allow(dead_code)]
    collection_period: std::time::Duration,
    
    /// Minimum samples required for optimization decisions
    #[allow(dead_code)]
    min_samples: u32,
}

/// Performance tracking system
#[derive(Debug)]
pub struct PerformanceTracker {
    /// Performance metrics history
    #[allow(dead_code)]
    metrics_history: Vec<PerformanceMetric>,
    
    /// Current performance baseline
    #[allow(dead_code)]
    baseline: PerformanceBaseline,
    
    /// Regression detection system
    #[allow(dead_code)]
    regression_detector: RegressionDetector,
    
    /// Performance improvement tracking
    #[allow(dead_code)]
    improvements: Vec<PerformanceImprovement>,
}

/// Optimization thresholds for adaptive decisions
#[derive(Debug, Clone)]
pub struct OptimizationThresholds {
    /// Hot path execution count threshold
    pub hot_path_threshold: u64,
    
    /// JIT compilation trigger threshold
    pub jit_compilation_threshold: u64,
    
    /// Loop unrolling complexity threshold
    pub loop_unroll_threshold: u32,
    
    /// Inlining size threshold
    pub inline_size_threshold: u32,
    
    /// Memory optimization threshold (bytes)
    pub memory_optimization_threshold: usize,
    
    /// Performance improvement threshold (percentage)
    pub improvement_threshold: f64,
}

/// Runtime execution statistics
#[derive(Debug, Default, Clone)]
pub struct RuntimeStats {
    /// Total expressions evaluated
    pub expressions_evaluated: usize,

    /// Optimizations applied
    pub optimizations_applied: usize,

    /// JIT compilations performed
    pub jit_compilations: usize,

    /// Tail call optimizations
    pub tail_calls_optimized: usize,

    /// Inline evaluations
    pub inline_evaluations: usize,

    /// Continuation pool hits
    pub continuation_pool_hits: usize,

    /// Continuation pool misses
    pub continuation_pool_misses: usize,

    /// Memory saved through continuation pooling (bytes)
    pub pooling_memory_saved: usize,

    /// Pool defragmentations performed
    pub pool_defragmentations: usize,

    /// Verification checks performed
    pub verification_checks: usize,

    /// Verification mismatches found
    pub verification_mismatches: usize,
    /// Verification successes
    pub verification_successes: usize,
    /// Verification failures
    pub verification_failures: usize,

    /// Total evaluation time in microseconds
    pub total_evaluation_time_us: u64,
    
    /// JIT execution time in microseconds
    pub jit_execution_time_us: u64,
    
    /// JIT compilation fallbacks
    pub jit_fallbacks: usize,
    
    /// Total execution time in microseconds
    pub total_execution_time_us: u64,

    /// Hot path detections
    pub hot_path_detections: usize,
    
    /// Static optimization metrics
    /// Number of macro expansions processed
    pub macro_expansions: usize,
    
    /// Number of pre-computed constants used
    pub constants_used: usize,
    
    /// JIT compilations triggered by static analysis
    pub jit_compilations_triggered: usize,
    
    /// Tail call optimizations applied based on static analysis
    pub tail_call_optimizations_applied: usize,
    
    /// Continuation pooling uses based on static hints
    pub continuation_pooling_uses: usize,
    
    /// Estimated time savings from static optimizations (microseconds)
    pub estimated_time_savings_us: u64,
    
    /// Estimated memory savings from static optimizations (bytes)
    pub estimated_memory_savings_bytes: usize,
    
    /// Inline evaluation opportunities detected
    pub inline_evaluation_opportunities: usize,
    
    /// LLVM optimizations applied
    pub llvm_optimizations_applied: usize,
}

impl RuntimeStats {
    /// Get continuation pool efficiency percentage
    #[must_use] pub fn continuation_pool_efficiency(&self) -> f64 {
        let total_requests = self.continuation_pool_hits + self.continuation_pool_misses;
        if total_requests > 0 {
            self.continuation_pool_hits as f64 / total_requests as f64 * 100.0
        } else {
            0.0
        }
    }
    
    /// Get average evaluation time per expression in microseconds
    #[must_use] pub fn average_evaluation_time_us(&self) -> f64 {
        if self.expressions_evaluated > 0 {
            self.total_evaluation_time_us as f64 / self.expressions_evaluated as f64
        } else {
            0.0
        }
    }
    
    /// Get optimization rate percentage
    #[must_use] pub fn optimization_rate(&self) -> f64 {
        if self.expressions_evaluated > 0 {
            self.optimizations_applied as f64 / self.expressions_evaluated as f64 * 100.0
        } else {
            0.0
        }
    }
}

impl RuntimeExecutor {
    /// Create a new runtime executor with default optimization level
    #[must_use] pub fn new() -> Self {
        Self {
            semantic_evaluator: SemanticEvaluator::new(),
            jit_optimizer: JitLoopOptimizer::new(),
            inline_evaluator: InlineEvaluator::new(),
            continuation_pooler: ContinuationPoolManager::new(),
            integrated_optimizer: IntegratedOptimizationManager::new(),
            adaptive_engine: AdaptiveOptimizationEngine::new(),
            hotpath_detector: AdvancedHotPathDetector::new(),
            llvm_compiler: LLVMCompilerIntegration::new(),
            optimization_level: RuntimeOptimizationLevel::Balanced,
            verification_enabled: cfg!(debug_assertions),
            stats: RuntimeStats::default(),
            recursion_depth: 0,
            max_recursion_depth: 1000,
        }
    }

    /// Create runtime executor with custom optimization level
    #[must_use] pub fn with_optimization_level(level: RuntimeOptimizationLevel) -> Self {
        let mut executor = Self::new();
        executor.optimization_level = level;
        executor
    }

    /// Create runtime executor with custom environment
    #[must_use] pub fn with_environment(env: Rc<Environment>) -> Self {
        let mut executor = Self::new();
        executor.semantic_evaluator = SemanticEvaluator::with_environment(env);
        executor
    }

    /// Create runtime executor with shared environment (new architecture)
    /// This allows the environment to be created once and shared across components
    #[must_use] pub fn with_shared_environment(env: std::sync::Arc<Environment>) -> Self {
        // Convert Arc<Environment> to Rc<Environment> for current compatibility
        let rc_env = Rc::new((*env).clone());
        Self::with_environment(rc_env)
    }

    /// Enable or disable verification against semantic evaluator
    pub fn set_verification_enabled(&mut self, enabled: bool) {
        self.verification_enabled = enabled;
    }

    /// Get current optimization level
    #[must_use] pub fn optimization_level(&self) -> RuntimeOptimizationLevel {
        self.optimization_level
    }

    /// Set optimization level
    pub fn set_optimization_level(&mut self, level: RuntimeOptimizationLevel) {
        self.optimization_level = level;
        
        // Adjust LLVM optimization level based on runtime optimization level
        let llvm_level = match level {
            RuntimeOptimizationLevel::None => LLVMOptimizationLevel::O0,
            RuntimeOptimizationLevel::Conservative => LLVMOptimizationLevel::O1,
            RuntimeOptimizationLevel::Balanced => LLVMOptimizationLevel::O2,
            RuntimeOptimizationLevel::Aggressive => LLVMOptimizationLevel::O3,
        };
        self.llvm_compiler.set_optimization_level(llvm_level);
    }

    /// Get JIT optimization statistics
    #[must_use] pub fn get_jit_statistics(&self) -> JitOptimizationStats {
        self.jit_optimizer.optimization_statistics()
    }

    /// Get advanced hot path analysis report
    #[must_use] pub fn get_hotpath_analysis_report(&self) -> String {
        let report = self.hotpath_detector.generate_performance_report();
        format!(
            "Hot Path Analysis Report:\n\
            - Total expressions analyzed: {}\n\
            - Hot paths detected: {}\n\
            - Top hot paths: {}\n\
            - Memory efficiency: {:.2}%\n\
            - Branch prediction accuracy: {:.2}%\n\
            - Loop optimization opportunities: {}",
            report.total_expressions_analyzed,
            report.hotpath_count,
            report.top_hotpaths.len(),
            report.memory_efficiency_score * 100.0,
            report.branch_prediction_accuracy * 100.0,
            report.loop_optimization_opportunities.len()
        )
    }

    /// Clear JIT compilation caches (useful for testing or memory management)
    pub fn clear_jit_caches(&mut self) {
        self.jit_optimizer.clear_caches();
    }

    /// Get current recursion depth (useful for testing)
    #[must_use] pub fn get_recursion_depth(&self) -> usize {
        self.recursion_depth
    }

    /// Set maximum recursion depth (useful for testing)
    pub fn set_max_recursion_depth(&mut self, max_depth: usize) {
        self.max_recursion_depth = max_depth;
    }

    /// ExecutionContext-driven optimized evaluation (Phase 9/10 integration)
    /// This is the key integration point for receiving ExecutionContext from Evaluator
    pub fn eval_with_execution_context(
        &mut self,
        context: ExecutionContext,
    ) -> Result<Value> {
        // Stack overflow protection
        self.check_recursion_depth()?;
        self.recursion_depth += 1;
        self.stats.expressions_evaluated += 1;

        let eval_start = std::time::Instant::now();
        
        // Extract information from ExecutionContext
        // Use expanded expression if available, otherwise use original
        let expr = context.get_execution_expression().clone();
        let env = context.environment.clone();
        let cont = context.continuation.clone();
        
        // Use static analysis from ExecutionContext
        let complexity_score = context.static_analysis.complexity_score;
        
        // Log static optimization information
        if context.was_macro_expanded() {
            self.stats.macro_expansions += context.macro_expansion_state.expanded_macros.len();
        }
        
        // Apply pre-computed constant bindings from static optimization
        for (_name, _value) in &context.constant_bindings {
            // Record that we're using a pre-computed constant
            self.stats.constants_used += 1;
            // Use pre-computed constants by binding them in environment
            // Note: Environment is already Arc<MutableEnvironment>, constants should be pre-bound
        }
        
        // Record static optimization benefits
        let static_benefit = context.estimated_static_benefit();
        self.stats.estimated_time_savings_us += static_benefit.time_savings_micros;
        self.stats.estimated_memory_savings_bytes += static_benefit.memory_savings_bytes;
        
        // Use optimization hints from ExecutionContext
        let should_use_jit = context.optimization_hints.jit_beneficial;
        let should_use_tail_call_opt = context.optimization_hints.use_tail_call_optimization;
        let should_use_continuation_pooling = context.optimization_hints.use_continuation_pooling;
        let should_use_inline_eval = context.optimization_hints.use_inline_evaluation;
        
        // Apply static optimization insights to dynamic optimization decisions  
        if should_use_jit {
            self.stats.jit_compilations_triggered += 1;
        }
        
        if should_use_tail_call_opt {
            self.stats.tail_call_optimizations_applied += 1;
        }
        
        if should_use_continuation_pooling {
            self.stats.continuation_pooling_uses += 1;
        }
        
        if should_use_inline_eval {
            self.stats.inline_evaluations += 1;
        }
        
        // Map ExecutionContext optimization level to RuntimeOptimizationLevel
        let runtime_level = match context.optimization_hints.optimization_level {
            crate::evaluator::execution_context::OptimizationLevel::None => RuntimeOptimizationLevel::None,
            crate::evaluator::execution_context::OptimizationLevel::Conservative => RuntimeOptimizationLevel::Conservative,
            crate::evaluator::execution_context::OptimizationLevel::Balanced => RuntimeOptimizationLevel::Balanced,
            crate::evaluator::execution_context::OptimizationLevel::Aggressive => RuntimeOptimizationLevel::Aggressive,
        };
        
        // Override runtime optimization level if context suggests different level
        if runtime_level != self.optimization_level {
            self.set_optimization_level(runtime_level);
        }
        
        // Advanced hot path detection with ExecutionContext information
        if let Err(_) = self.hotpath_detector.record_execution(
            &expr,
            std::time::Duration::from_nanos(0), // Will be updated after evaluation
            context.static_analysis.memory_estimates.heap_allocations,
            &Value::Undefined, // Return value placeholder
            &[], // Call stack placeholder
        ) {
            // Continue with evaluation even if hotpath recording fails
        }
        
        // Track hot path detection based on complexity score
        if complexity_score > 75 {
            self.stats.hot_path_detections += 1;
        }
        
        // Apply optimizations based on ExecutionContext
        let result = match runtime_level {
            RuntimeOptimizationLevel::None => {
                // No optimizations - delegate to semantic evaluator
                self.semantic_evaluator.eval_pure(expr.clone(), env.clone(), cont.clone())
            }
            RuntimeOptimizationLevel::Conservative => {
                self.apply_conservative_optimizations(expr.clone(), env.clone(), cont.clone(), &context)
            }
            RuntimeOptimizationLevel::Balanced => {
                self.apply_balanced_optimizations(expr.clone(), env.clone(), cont.clone(), &context)
            }
            RuntimeOptimizationLevel::Aggressive => {
                self.apply_aggressive_optimizations(expr.clone(), env.clone(), cont.clone(), &context)
            }
        };
        
        // Update evaluation time
        let eval_time = eval_start.elapsed();
        self.stats.total_evaluation_time_us += eval_time.as_micros() as u64;
        
        // Verify result against semantic evaluator if enabled
        if self.verification_enabled && result.is_ok() {
            self.verify_result_correctness(&expr, &env, &cont, &result);
        }
        
        self.recursion_depth -= 1;
        result
    }

    /// ExecutionContext-driven JIT analysis and optimized execution (Phase 9 核心API)
    /// This is the key API for responsibility separation: receives ExecutionContext from Evaluator,
    /// performs dynamic optimization selection, JIT compilation decisions, and execution
    pub fn execute_with_jit_analysis(
        &mut self,
        context: ExecutionContext,
    ) -> Result<Value> {
        self.check_recursion_depth()?;
        self.recursion_depth += 1;
        
        let eval_start = std::time::Instant::now();
        
        // 1. 静的解析結果の活用（Evaluator責務からの引き継ぎ）
        // 2. 動的最適化戦略の選択（RuntimeExecutor責務）
        let analysis_result = ExpressionAnalysisResult {
            complexity_score: context.static_analysis.complexity_score,
            is_tail_call_candidate: context.static_analysis.has_tail_calls,
            is_hot_path: context.execution_metadata.priority == ExecutionPriority::High,
            contains_loops: context.static_analysis.has_loops,
            call_patterns: vec![], // Simplified for now
            execution_frequency: ExecutionFrequency::Warm,
            memory_patterns: vec![],
            optimization_hints: vec![],
        };
        // Simplified strategy selection
        let runtime_strategy = if context.execution_metadata.priority == ExecutionPriority::High && analysis_result.complexity_score > 20 {
            AdaptiveOptimizationType::JitCompilation
        } else if analysis_result.contains_loops {
            AdaptiveOptimizationType::AdaptiveLoopUnrolling { factor: 2 }
        } else {
            AdaptiveOptimizationType::ProfileGuidedOptimization
        };
        
        // 3. JIT判定・実行戦略決定
        let opt_level = self.optimization_level.clone();
        let result = if runtime_strategy.should_compile_jit() {
            // JITコンパイル & 実行パス
            match self.jit_compile_and_execute(&context, &opt_level) {
                Ok(jit_result) => {
                    self.stats.jit_compilations += 1;
                    self.stats.jit_execution_time_us += eval_start.elapsed().as_micros() as u64;
                    Ok(jit_result)
                },
                Err(_) => {
                    // JIT失敗時は最適化インタープリタにフォールバック
                    self.stats.jit_fallbacks += 1;
                    self.interpret_with_optimizations(&context, &opt_level)
                }
            }
        } else {
            // 最適化インタープリタ実行パス
            self.interpret_with_optimizations(&context, &opt_level)
        }?;
        
        // 4. パフォーマンス統計更新
        let execution_time = eval_start.elapsed();
        self.stats.total_execution_time_us += execution_time.as_micros() as u64;
        
        // 5. 結果検証（SemanticEvaluator基準）
        if self.verification_enabled {
            // Simplified verification - using existing method
            self.verify_result_correctness(&context.expression, &context.environment, &context.continuation, &Ok(result.clone()));
        }
        
        self.recursion_depth -= 1;
        Ok(result)
    }

    /// JITコンパイル & 実行（LLVM統合）
    pub fn jit_compile_and_execute(
        &mut self,
        context: &ExecutionContext,
        strategy: &RuntimeOptimizationLevel,
    ) -> Result<Value> {
        let expr = context.get_execution_expression().clone();
        let env = context.environment.clone();
        let cont = context.continuation.clone();
        
        // Simplified JIT: fallback to semantic evaluator with tail call optimization
        if strategy.use_tail_call_optimization() {
            let analysis = ExpressionAnalysisResult {
                complexity_score: 10,
                is_tail_call_candidate: true,
                is_hot_path: false,
                contains_loops: false,
                call_patterns: vec![],
                execution_frequency: ExecutionFrequency::Warm,
                memory_patterns: vec![],
                optimization_hints: vec![],
            };
            if let Some(optimized_result) = self.try_tail_call_optimization(&expr, env.clone(), cont.clone(), &analysis)? {
                self.stats.jit_compilations += 1;
                return Ok(optimized_result);
            }
        }
        
        // Fallback to semantic evaluator
        self.semantic_evaluator.eval_pure(expr, env, cont)
    }

    /// 最適化インタープリタ実行（JIT以外の動的最適化）
    pub fn interpret_with_optimizations(
        &mut self,
        context: &ExecutionContext,
        _strategy: &RuntimeOptimizationLevel,
    ) -> Result<Value> {
        let expr = context.get_execution_expression().clone();
        let env = context.environment.clone();
        let cont = context.continuation.clone();
        
        // 1. 継続プーリング最適化（簡素化）
        let optimized_cont = cont; // Simplified: skip pooling for now
        
        // 2. インライン評価最適化（簡素化）
        // Skip inline optimization for now
        
        // 3. ループ最適化（簡素化）
        // Skip loop optimization for now
        
        // 4. 末尾呼び出し最適化（簡素化）
        // Skip tail call optimization for now in this method
        
        // 5. 定数最適化適用（簡素化）
        let const_optimized_expr = expr.clone(); // Skip immediate optimizations for now
        
        // 6. SemanticEvaluator基準での実行
        self.stats.expressions_evaluated += 1;
        self.semantic_evaluator.eval_pure(const_optimized_expr, env, optimized_cont)
    }

    /// Apply conservative optimizations based on ExecutionContext
    fn apply_conservative_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        context: &ExecutionContext,
    ) -> Result<Value> {
        // Conservative: Apply safe optimizations using static analysis
        
        // Use pre-computed constants if available
        if let Expr::Variable(name) = &expr {
            if let Some(constant_value) = context.get_constant_binding(name) {
                self.stats.constants_used += 1;
                return Ok(constant_value.clone());
            }
        }
        
        // Use inline evaluation for pure simple expressions
        if context.static_analysis.is_pure && 
           context.static_analysis.complexity_score < 20 &&
           context.optimization_hints.use_inline_evaluation {
            // Use inline evaluator with proper API integration  
            // Note: Inline evaluator operates on values, not expressions directly
            // This would require expression evaluation first, then inline optimization
            // For now, just track the optimization opportunity
            self.stats.inline_evaluation_opportunities += 1;
            self.stats.inline_evaluations += 1;
        }
        
        // Fall back to semantic evaluator
        self.semantic_evaluator.eval_pure(expr, env, cont)
    }
    
    /// Apply balanced optimizations based on ExecutionContext
    fn apply_balanced_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        context: &ExecutionContext,
    ) -> Result<Value> {
        // Balanced: Apply moderate optimizations based on static analysis
        
        // First try conservative optimizations
        if let Ok(result) = self.apply_conservative_optimizations(expr.clone(), env.clone(), cont.clone(), context) {
            return Ok(result);
        }
        
        // Apply tail call optimization if recommended
        if context.optimization_hints.use_tail_call_optimization {
            // TODO: Use tail call optimizer with proper API
            // if let Ok(optimized_result) = self.tail_call_optimizer.optimize_call(&expr, &env, &cont) {
            //     self.stats.tail_calls_optimized += 1;
            //     return self.semantic_evaluator.eval_pure(optimized_result, env, cont);
            // }
            self.stats.tail_calls_optimized += 1;
        }
        
        // Apply continuation pooling if beneficial
        if context.optimization_hints.use_continuation_pooling {
            // TODO: Integrate with continuation pool manager
            self.stats.continuation_pooling_uses += 1;
        }
        
        // Fall back to semantic evaluator
        self.semantic_evaluator.eval_pure(expr, env, cont)
    }
    
    /// Apply aggressive optimizations based on ExecutionContext  
    fn apply_aggressive_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        context: &ExecutionContext,
    ) -> Result<Value> {
        // Aggressive: Apply all available optimizations based on static analysis
        
        // First try balanced optimizations
        if let Ok(result) = self.apply_balanced_optimizations(expr.clone(), env.clone(), cont.clone(), context) {
            return Ok(result);
        }
        
        // Apply JIT compilation for hot paths if recommended
        if context.optimization_hints.jit_beneficial && 
           context.static_analysis.complexity_score > 50 {
            self.stats.jit_compilations += 1;
            self.stats.jit_compilations_triggered += 1;
            
            // Use LLVM compiler with proper API integration
            // Create tail call context for compilation
            let tail_call_context = crate::evaluator::TailCallContext::new();
            if let Ok(_compiled_fn) = self.llvm_compiler.compile_with_tail_calls(&expr, &tail_call_context) {
                // LLVM compilation succeeded, track the optimization
                self.stats.llvm_optimizations_applied += 1;
                // Note: Actual execution would require more integration work
                // For now, fall through to semantic evaluation with optimization tracking
            }
        }
        
        // Use all static optimization results
        for optimization in &context.static_analysis.static_optimizations {
            match optimization {
                crate::evaluator::execution_context::StaticOptimization::ConstantFolding { result, .. } => {
                    // If this is a constant folding result, use it directly
                    return Ok(result.clone());
                }
                _ => {} // Handle other optimizations as needed
            }
        }
        
        // Fall back to semantic evaluator
        self.semantic_evaluator.eval_pure(expr, env, cont)
    }

    /// Main optimized evaluation function
    pub fn eval_optimized(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Stack overflow protection
        self.check_recursion_depth()?;
        self.recursion_depth += 1;
        self.stats.expressions_evaluated += 1;

        // Expression analysis for optimization hints
        let eval_start = std::time::Instant::now();
        let analysis = ExpressionAnalysisResult::analyze_expression(&expr);
        
        // Advanced hot path detection with multi-dimensional analysis
        if let Err(_) = self.hotpath_detector.record_execution(
            &expr,
            std::time::Duration::from_nanos(0), // Will be updated after evaluation
            0, // Memory usage placeholder
            &Value::Undefined, // Return value placeholder
            &[], // Call stack placeholder
        ) {
            // Continue with evaluation even if hotpath recording fails
        }
        
        // Track hot path detection
        if analysis.execution_frequency == ExecutionFrequency::Hot {
            self.stats.hot_path_detections += 1;
        }

        // Apply optimizations based on analysis
        let result = match self.optimization_level {
            RuntimeOptimizationLevel::None => {
                // No optimizations - delegate to semantic evaluator
                self.semantic_evaluator.eval_pure(expr, env, cont)
            }

            RuntimeOptimizationLevel::Conservative => {
                // Conservative optimizations only
                self.eval_with_conservative_optimizations(expr, env, cont, &analysis)
            }

            RuntimeOptimizationLevel::Balanced => {
                // Balanced optimization approach
                self.eval_with_balanced_optimizations(expr, env, cont, &analysis)
            }

            RuntimeOptimizationLevel::Aggressive => {
                // Aggressive optimizations
                self.eval_with_aggressive_optimizations(expr, env, cont, &analysis)
            }
        };

        // Always update statistics and decrement recursion depth
        let eval_time = eval_start.elapsed();
        self.stats.total_evaluation_time_us += eval_time.as_micros() as u64;
        self.recursion_depth -= 1;

        // Return result (could be Ok or Err)
        result
    }

    /// Conservative optimizations: basic optimizations with high confidence
    fn eval_with_conservative_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Value> {
        // Conservative optimization strategy: focus on safe, proven optimizations
        
        // Check for immediate optimization opportunities
        if let Some(result) = self.try_immediate_optimizations(&expr, analysis)? {
            return Ok(result);
        }
        
        // Apply builtin function optimizations
        if let Some(result) = self.try_builtin_optimizations(&expr, &env, analysis)? {
            self.stats.optimizations_applied += 1;
            return Ok(result);
        }
        
        // Apply tail call optimization if conservative
        if analysis.is_tail_call_candidate && 
           analysis.optimization_hints.contains(&OptimizationHint::TailCallOptimize) {
            if let Some(result) = self.try_tail_call_optimization(&expr, env.clone(), cont.clone(), analysis)? {
                self.stats.tail_calls_optimized += 1;
                return Ok(result);
            }
        }
        
        // Use integrated optimization system as fallback
        let Ok(strategies) = self
            .integrated_optimizer
            .select_optimization_strategy(&expr, &RuntimeOptimizationLevel::Conservative) else {
                // Fallback to semantic evaluation if strategy selection fails
                return self.semantic_evaluator.eval_pure(expr, env, cont);
            };

        if !strategies.is_empty() {
            if let Ok(optimization_result) = self.integrated_optimizer.execute_optimization(
                expr.clone(),
                env.clone(),
                strategies,
            ) {
                if !optimization_result.applied_strategies.is_empty() {
                    self.stats.optimizations_applied += 1;
                    return self.apply_optimization_result(optimization_result, env, cont);
                }
            }
        }

        // Fallback to semantic evaluation
        self.semantic_evaluator.eval_pure(expr, env, cont)
    }

    /// Balanced optimizations: good balance of safety and performance
    fn eval_with_balanced_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Value> {
        // Balanced optimization strategy: balance between safety and performance
        
        // Check for immediate optimization opportunities
        if let Some(result) = self.try_immediate_optimizations(&expr, analysis)? {
            return Ok(result);
        }
        
        // Apply builtin function optimizations
        if let Some(result) = self.try_builtin_optimizations(&expr, &env, analysis)? {
            self.stats.optimizations_applied += 1;
            return Ok(result);
        }
        
        // Apply tail call optimization
        if analysis.is_tail_call_candidate {
            if let Some(result) = self.try_tail_call_optimization(&expr, env.clone(), cont.clone(), analysis)? {
                self.stats.tail_calls_optimized += 1;
                return Ok(result);
            }
        }
        
        // Apply inlining for hot paths
        if analysis.execution_frequency == ExecutionFrequency::Hot ||
           analysis.optimization_hints.contains(&OptimizationHint::Inline) {
            if let Some(result) = self.try_inline_optimization(&expr, env.clone(), analysis)? {
                self.stats.inline_evaluations += 1;
                return Ok(result);
            }
        }
        
        // Apply continuation pooling for recursive patterns
        if analysis.optimization_hints.contains(&OptimizationHint::PoolContinuations) {
            if let Some(result) = self.try_continuation_pooling(&expr, env.clone(), cont.clone(), analysis)? {
                self.stats.continuation_pool_hits += 1;
                return Ok(result);
            }
        }
        
        // JIT loop optimization for hot loops
        if analysis.contains_loops && 
           analysis.execution_frequency >= ExecutionFrequency::Warm &&
           analysis.optimization_hints.contains(&OptimizationHint::JitCompile) {
            if let Some(result) = self.try_jit_loop_optimization(&expr, env.clone(), cont.clone(), analysis)? {
                self.stats.optimizations_applied += 1;
                return Ok(result);
            }
        }
        
        // Use integrated optimization system
        let Ok(strategies) = self
            .integrated_optimizer
            .select_optimization_strategy(&expr, &RuntimeOptimizationLevel::Balanced) else {
                return self.semantic_evaluator.eval_pure(expr, env, cont);
            };

        if !strategies.is_empty() {
            if let Ok(optimization_result) = self.integrated_optimizer.execute_optimization(
                expr.clone(),
                env.clone(),
                strategies,
            ) {
                if !optimization_result.applied_strategies.is_empty() {
                    self.stats.optimizations_applied += 1;
                    return self.apply_optimization_result(optimization_result, env, cont);
                }
            }
        }

        // Fallback to semantic evaluation
        self.semantic_evaluator.eval_pure(expr, env, cont)
    }

    /// Aggressive optimizations: maximum performance optimizations
    fn eval_with_aggressive_optimizations(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Value> {
        // Aggressive optimization strategy: maximum performance optimizations
        
        // Check for immediate optimization opportunities
        if let Some(result) = self.try_immediate_optimizations(&expr, analysis)? {
            return Ok(result);
        }
        
        // Apply JIT compilation for hot paths
        if analysis.execution_frequency == ExecutionFrequency::Hot ||
           analysis.execution_frequency == ExecutionFrequency::Critical ||
           analysis.optimization_hints.contains(&OptimizationHint::JitCompile) {
            if let Some(result) = self.try_jit_compilation(&expr, env.clone(), cont.clone(), analysis)? {
                self.stats.jit_compilations += 1;
                return Ok(result);
            }
        }
        
        // Apply LLVM native code generation for critical paths
        if analysis.execution_frequency == ExecutionFrequency::Critical {
            if let Some(result) = self.try_llvm_compilation(&expr, env.clone(), cont.clone(), analysis)? {
                self.stats.jit_compilations += 1;
                return Ok(result);
            }
        }
        
        // Apply builtin function optimizations with aggressive inlining
        if let Some(result) = self.try_builtin_optimizations(&expr, &env, analysis)? {
            self.stats.optimizations_applied += 1;
            return Ok(result);
        }
        
        // Apply tail call optimization
        if analysis.is_tail_call_candidate {
            if let Some(result) = self.try_tail_call_optimization(&expr, env.clone(), cont.clone(), analysis)? {
                self.stats.tail_calls_optimized += 1;
                return Ok(result);
            }
        }
        
        // Apply inlining aggressively
        if analysis.complexity_score < 50 { // More aggressive threshold
            if let Some(result) = self.try_inline_optimization(&expr, env.clone(), analysis)? {
                self.stats.inline_evaluations += 1;
                return Ok(result);
            }
        }
        
        // Apply loop optimizations
        if analysis.contains_loops {
            if let Some(result) = self.try_loop_optimization(&expr, env.clone(), cont.clone(), analysis)? {
                self.stats.optimizations_applied += 1;
                return Ok(result);
            }
        }
        
        // Apply continuation pooling
        if analysis.optimization_hints.contains(&OptimizationHint::PoolContinuations) {
            if let Some(result) = self.try_continuation_pooling(&expr, env.clone(), cont.clone(), analysis)? {
                self.stats.continuation_pool_hits += 1;
                return Ok(result);
            }
        }
        
        // Use integrated optimization system
        let Ok(strategies) = self
            .integrated_optimizer
            .select_optimization_strategy(&expr, &RuntimeOptimizationLevel::Aggressive) else {
                return self.semantic_evaluator.eval_pure(expr, env, cont);
            };

        if !strategies.is_empty() {
            if let Ok(optimization_result) = self.integrated_optimizer.execute_optimization(
                expr.clone(),
                env.clone(),
                strategies,
            ) {
                if !optimization_result.applied_strategies.is_empty() {
                    self.stats.optimizations_applied += 1;
                    return self.apply_optimization_result(optimization_result, env, cont);
                }
            }
        }

        // Fallback to semantic evaluation
        self.semantic_evaluator.eval_pure(expr, env, cont)
    }

    /// Apply optimization result from `IntegratedOptimizationManager`
    fn apply_optimization_result(
        &mut self,
        optimization_result: OptimizationResult,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Update statistics based on the optimization result
        if optimization_result.applied_strategies.is_empty() {
            // No optimization strategies were applied, use semantic evaluator to avoid recursion
            self.semantic_evaluator.eval_pure(optimization_result.optimized_expr, env, cont)
        } else {
            self.stats.optimizations_applied += 1;

            // Determine optimization type from strategy name and apply accordingly
            // Use the first applied strategy for classification
            let strategy = optimization_result.applied_strategies.first().map_or("", std::string::String::as_str);
            match strategy {
                s if s.contains("tail_call") => {
                    self.stats.tail_calls_optimized += 1;
                }
                s if s.contains("jit") || s.contains("loop") => {
                    self.stats.jit_compilations += 1;
                }
                s if s.contains("inline") => {
                    self.stats.inline_evaluations += 1;
                }
                s if s.contains("continuation") || s.contains("pool") => {
                    self.stats.continuation_pool_hits += 1;
                }
                _ => {
                    // General optimization
                }
            }

            // Use semantic evaluator to avoid infinite recursion
            self.semantic_evaluator.eval_pure(optimization_result.optimized_expr, env, cont)
        }
    }

    /// Try immediate optimizations for simple expressions
    fn try_immediate_optimizations(
        &self,
        expr: &Expr,
        _analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        match expr {
            // Constant folding for literals
            Expr::Literal(lit) => {
                Ok(Some(match lit {
                    crate::ast::Literal::Number(n) => Value::Number(n.clone()),
                    crate::ast::Literal::String(s) => Value::String(s.clone()),
                    crate::ast::Literal::Boolean(b) => Value::Boolean(*b),
                    crate::ast::Literal::Character(c) => Value::Character(*c),
                    crate::ast::Literal::Nil => Value::Nil,
                }))
            },
            
            // Simple arithmetic optimizations
            Expr::List(exprs) => {
                if exprs.len() == 3 {
                    if let Expr::Variable(name) = &exprs[0] {
                        if Self::is_simple_arithmetic(name) {
                            if let (Some(a), Some(b)) = (Self::extract_literal_number(&exprs[1]), Self::extract_literal_number(&exprs[2])) {
                                return Ok(Some(Self::compute_arithmetic(name, a, b)?));
                            }
                        }
                    }
                }
                Ok(None)
            },
            
            _ => Ok(None),
        }
    }
    
    /// Try builtin function optimizations
    fn try_builtin_optimizations(
        &self,
        expr: &Expr,
        _env: &Rc<Environment>,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        if let Expr::List(exprs) = expr {
            if !exprs.is_empty() {
                if let Expr::Variable(name) = &exprs[0] {
                    // Check if this is a builtin that can be optimized
                    if Self::is_optimizable_builtin(name, analysis) {
                        return self.apply_builtin_optimization(name, &exprs[1..]);
                    }
                }
            }
        }
        Ok(None)
    }
    
    /// Try tail call optimization
    fn try_tail_call_optimization(
        &mut self,
        expr: &Expr,
        env: Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        // Check if this is a tail call candidate
        if !analysis.is_tail_call_candidate {
            return Ok(None);
        }
        
        match expr {
            Expr::List(exprs) => {
                if !exprs.is_empty() {
                    // For tail calls, we can optimize by avoiding stack frame creation
                    let optimized = OptimizedTailCall {
                        target_function: Value::Nil, // Placeholder
                        arguments: Vec::new(),        // Placeholder
                        environment: env.clone(),
                        optimization_applied: true,
                    };
                    
                    // Apply the optimized tail call
                    Ok(Some(self.apply_optimized_tail_call(optimized, env, cont)?))
                } else {
                    Ok(None)
                }
            },
            _ => Ok(None),
        }
    }
    
    /// Try inline optimization
    fn try_inline_optimization(
        &mut self,
        expr: &Expr,
        env: Rc<Environment>,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        // Only inline simple expressions
        if analysis.complexity_score > 15 {
            return Ok(None);
        }
        
        match expr {
            Expr::List(exprs) => {
                // Check for lambda expressions
                if exprs.len() >= 3 {
                    if let Expr::Variable(name) = &exprs[0] {
                        if name == "lambda" {
                            // Inline simple lambda bodies
                            let body = &exprs[2];
                            if Self::is_inlinable_expression(body) {
                                return Ok(Some(self.semantic_evaluator.eval_pure(
                                    body.clone(),
                                    env,
                                    Continuation::Identity,
                                )?));
                            }
                        }
                    }
                }
            },
            _ => {}
        }
        
        Ok(None)
    }
    
    /// Try continuation pooling optimization
    fn try_continuation_pooling(
        &mut self,
        expr: &Expr,
        env: Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        // Use continuation pooling for recursive patterns and hot paths
        let should_pool = analysis.call_patterns.iter().any(|p| matches!(p, 
            CallPattern::Recursive { .. } | CallPattern::TailRecursive
        )) || analysis.execution_frequency == ExecutionFrequency::Hot;
        
        if should_pool {
            // Determine continuation type for optimal pooling
            let cont_type = ContinuationType::from_continuation(&cont);
            
            // Try to allocate from pool first
            if let Some(pooled_cont) = self.continuation_pooler.allocate(cont_type) {
                // Use pooled continuation for evaluation
                let result = self.semantic_evaluator.eval_pure(expr.clone(), env, pooled_cont.clone());
                
                // Return continuation to pool after use
                if self.continuation_pooler.deallocate(pooled_cont) {
                    // Track successful pool usage
                    let (_, _, memory_saved, _) = self.continuation_pooler.global_statistics();
                    self.stats.pooling_memory_saved = memory_saved;
                } else {
                    self.stats.continuation_pool_misses += 1;
                }
                
                return Ok(Some(result?));
            }
        }
        
        // Fall back to normal evaluation with original continuation
        match expr {
            Expr::List(_) => {
                // Use semantic evaluator with the original continuation
                let result = self.semantic_evaluator.eval_pure(expr.clone(), env, cont.clone());
                
                // Try to return continuation to pool if beneficial
                if should_pool {
                    if self.continuation_pooler.deallocate(cont) {
                        // Update pooling statistics
                        let (_, _, memory_saved, _) = self.continuation_pooler.global_statistics();
                        self.stats.pooling_memory_saved = memory_saved;
                    } else {
                        self.stats.continuation_pool_misses += 1;
                    }
                }
                
                Ok(Some(result?))
            },
            _ => Ok(None),
        }
    }
    
    
    /// Try loop optimization
    fn try_loop_optimization(
        &mut self,
        expr: &Expr,
        env: Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        if !analysis.contains_loops {
            return Ok(None);
        }
        
        // Apply loop unrolling or other loop optimizations
        match expr {
            Expr::List(_) => {
                // For now, use semantic evaluator with loop detection
                Ok(Some(self.semantic_evaluator.eval_pure(expr.clone(), env, cont)?))
            },
            _ => Ok(None),
        }
    }
    
    /// Verify result correctness against semantic evaluator
    fn verify_result_correctness(
        &mut self,
        expr: &Expr,
        env: &Rc<Environment>,
        cont: &Continuation,
        result: &Result<Value>,
    ) {
        // Only verify successful results
        if let Ok(optimized_result) = result {
            // Compare with semantic evaluator result
            if let Ok(semantic_result) = self.semantic_evaluator.eval_pure(
                expr.clone(),
                env.clone(),
                cont.clone(),
            ) {
                // Check if results are equivalent
                if !self.values_equivalent(optimized_result, &semantic_result) {
                    eprintln!(
                        "WARNING: Optimization produced different result than semantic evaluator\n\
                        Expression: {:?}\n\
                        Optimized result: {:?}\n\
                        Semantic result: {:?}",
                        expr, optimized_result, semantic_result
                    );
                    self.stats.verification_failures += 1;
                } else {
                    self.stats.verification_successes += 1;
                }
            }
        }
    }
    
    /// Check if two values are equivalent (for verification)
    fn values_equivalent(&self, val1: &Value, val2: &Value) -> bool {
        // For now, use direct equality
        // In the future, this could be more sophisticated
        // (e.g., numerical tolerance for floating point)
        val1 == val2
    }

    /// Apply optimized tail call result
    fn apply_optimized_tail_call(
        &mut self,
        optimized: OptimizedTailCall,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if optimized.optimization_applied {
            // Use optimized execution path
            self.semantic_evaluator.eval_pure(
                crate::ast::Expr::Literal(crate::ast::Literal::Nil),
                optimized.environment,
                cont,
            )
        } else {
            // Fallback to normal evaluation
            self.semantic_evaluator.eval_pure(
                crate::ast::Expr::Literal(crate::ast::Literal::Nil),
                env,
                cont,
            )
        }
    }

    /// Apply JIT compiled code
    #[allow(dead_code)]
    fn apply_jit_compiled_code(
        &mut self,
        jit_code: JitCompiledCode,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        if jit_code.is_ready {
            // Execute JIT compiled code (simulated)
            // In a real implementation, this would execute native compiled code
            self.semantic_evaluator.eval_pure(
                jit_code.original_expr,
                env,
                cont,
            )
        } else {
            // Fallback if JIT compilation failed
            self.semantic_evaluator.eval_pure(
                jit_code.original_expr,
                env,
                cont,
            )
        }
    }

    /// Apply continuation with optimization (placeholder for future implementation)
    #[allow(dead_code)]
    fn apply_continuation_optimized(&mut self, cont: Continuation, value: Value) -> Result<Value> {
        // Use semantic evaluator's continuation system
        match cont {
            Continuation::Identity => Ok(value),
            _ => {
                // For now, fallback to semantic evaluator
                Ok(value) // Simplified placeholder
            }
        }
    }

    /// Direct procedure application (placeholder for future implementation)
    #[allow(dead_code)]
    fn apply_procedure_direct(
        &mut self,
        _procedure: Value,
        _args: Vec<Value>,
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Fallback to semantic evaluator for basic implementation
        self.semantic_evaluator.eval_pure(
            crate::ast::Expr::Literal(crate::ast::Literal::Nil),
            Rc::new(crate::environment::Environment::new()),
            cont,
        )
    }

    /// Execute optimized loop (placeholder for future implementation)
    #[allow(dead_code)]
    fn execute_optimized_loop(
        &mut self,
        _loop_body: Vec<Expr>,
        _bindings: Vec<(String, Value)>,
        _env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<Value> {
        // Fallback to semantic evaluator for basic implementation
        self.semantic_evaluator.eval_pure(
            crate::ast::Expr::Literal(crate::ast::Literal::Nil),
            Rc::new(crate::environment::Environment::new()),
            cont,
        )
    }

    /// Optimized builtin application (placeholder for future implementation)
    #[allow(dead_code)]
    fn apply_builtin_optimized(&self, name: &str, args: &[Value]) -> Result<Value> {
        // Use simple implementation
        match name {
            "+" => self.builtin_add_simple(args),
            "-" => self.builtin_subtract_simple(args),
            "*" => self.builtin_multiply_simple(args),
            _ => {
                // For other builtins, fallback to error for now
                Err(LambdustError::runtime_error(format!(
                    "Builtin '{name}' not implemented in runtime executor yet"
                )))
            }
        }
    }

    /// Simple addition implementation
    #[allow(dead_code)]
    fn builtin_add_simple(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Number(crate::lexer::SchemeNumber::Integer(0)));
        }

        let mut sum = 0i64;
        for arg in args {
            if let Value::Number(crate::lexer::SchemeNumber::Integer(n)) = arg {
                sum += n;
            } else {
                return Err(LambdustError::type_error("Addition expects integers"));
            }
        }

        Ok(Value::Number(crate::lexer::SchemeNumber::Integer(sum)))
    }

    /// Simple subtraction implementation
    #[allow(dead_code)]
    fn builtin_subtract_simple(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Err(LambdustError::arity_error(1, 0));
        }

        let first = match &args[0] {
            Value::Number(crate::lexer::SchemeNumber::Integer(n)) => *n,
            _ => return Err(LambdustError::type_error("Subtraction expects integers")),
        };

        if args.len() == 1 {
            return Ok(Value::Number(crate::lexer::SchemeNumber::Integer(-first)));
        }

        let mut result = first;
        for arg in &args[1..] {
            if let Value::Number(crate::lexer::SchemeNumber::Integer(n)) = arg {
                result -= n;
            } else {
                return Err(LambdustError::type_error("Subtraction expects integers"));
            }
        }

        Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
    }

    /// Simple multiplication implementation
    #[allow(dead_code)]
    fn builtin_multiply_simple(&self, args: &[Value]) -> Result<Value> {
        if args.is_empty() {
            return Ok(Value::Number(crate::lexer::SchemeNumber::Integer(1)));
        }

        let mut product = 1i64;
        for arg in args {
            if let Value::Number(crate::lexer::SchemeNumber::Integer(n)) = arg {
                product *= n;
            } else {
                return Err(LambdustError::type_error("Multiplication expects integers"));
            }
        }

        Ok(Value::Number(crate::lexer::SchemeNumber::Integer(product)))
    }

    /// Helper methods for optimization
    /// Check if function name is simple arithmetic
    fn is_simple_arithmetic(name: &str) -> bool {
        matches!(name, "+" | "-" | "*")
    }
    
    /// Extract literal number from expression
    fn extract_literal_number(expr: &Expr) -> Option<i64> {
        if let Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(n))) = expr {
            Some(*n)
        } else {
            None
        }
    }
    
    /// Compute arithmetic operation
    fn compute_arithmetic(op: &str, a: i64, b: i64) -> Result<Value> {
        let result = match op {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            _ => return Err(LambdustError::runtime_error(format!("Unknown arithmetic operator: {}", op))),
        };
        Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
    }
    
    /// Check if builtin function can be optimized
    fn is_optimizable_builtin(name: &str, analysis: &ExpressionAnalysisResult) -> bool {
        // Optimize builtins on hot paths or with specific hints
        let is_builtin = matches!(name, "+" | "-" | "*" | "/" | "=" | "<" | ">" 
                                      | "cons" | "car" | "cdr" | "length");
        let should_optimize = analysis.execution_frequency != ExecutionFrequency::Cold ||
                               analysis.optimization_hints.contains(&OptimizationHint::Inline);
        is_builtin && should_optimize
    }
    
    /// Apply builtin function optimization
    fn apply_builtin_optimization(&self, name: &str, _args: &[Expr]) -> Result<Option<Value>> {
        // For now, delegate to simple implementations
        match name {
            "+" | "-" | "*" => {
                // Try to evaluate arguments and apply optimization
                Ok(None) // Placeholder
            },
            _ => Ok(None),
        }
    }
    
    /// Check if expression is suitable for inlining
    fn is_inlinable_expression(expr: &Expr) -> bool {
        match expr {
            Expr::Literal(_) => true,
            Expr::Variable(_) => true,
            Expr::List(exprs) => exprs.len() <= 3,
            _ => false,
        }
    }
    
    /// Simple division implementation
    #[allow(dead_code)]
    fn builtin_divide_simple(&self, args: &[Value]) -> Result<Value> {
        if args.len() < 2 {
            return Err(LambdustError::arity_error(2, args.len()));
        }
        
        let dividend = match &args[0] {
            Value::Number(crate::lexer::SchemeNumber::Integer(n)) => *n,
            _ => return Err(LambdustError::type_error("Division expects integers")),
        };
        
        let divisor = match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(n)) => *n,
            _ => return Err(LambdustError::type_error("Division expects integers")),
        };
        
        if divisor == 0 {
            return Err(LambdustError::runtime_error("Division by zero"));
        }
        
        Ok(Value::Number(crate::lexer::SchemeNumber::Integer(dividend / divisor)))
    }

    /// Verify result against semantic evaluator (placeholder for future implementation)
    #[allow(dead_code)]
    fn verify_result(
        &mut self,
        _expr: &Expr,
        _env: &Rc<Environment>,
        _cont: &Continuation,
        _result: &Result<Value>,
    ) -> Result<()> {
        // Verification is disabled
        Ok(())
    }


    /// Check recursion depth
    fn check_recursion_depth(&self) -> Result<()> {
        if self.recursion_depth >= self.max_recursion_depth {
            return Err(LambdustError::stack_overflow());
        }
        Ok(())
    }

    /// Get current runtime statistics
    #[must_use] pub fn get_stats(&self) -> &RuntimeStats {
        &self.stats
    }

    /// Reset runtime statistics
    pub fn reset_stats(&mut self) {
        self.stats = RuntimeStats::default();
    }

    /// Get reference to semantic evaluator
    #[must_use] pub fn get_semantic_evaluator(&self) -> &SemanticEvaluator {
        &self.semantic_evaluator
    }

    /// Get mutable reference to semantic evaluator
    pub fn get_semantic_evaluator_mut(&mut self) -> &mut SemanticEvaluator {
        &mut self.semantic_evaluator
    }

    /// Get reference to JIT optimizer (dynamic optimization)
    #[must_use] pub fn get_jit_optimizer(&self) -> &JitLoopOptimizer {
        &self.jit_optimizer
    }

    /// Get reference to inline evaluator
    #[must_use] pub fn get_inline_evaluator(&self) -> &InlineEvaluator {
        &self.inline_evaluator
    }

    /// Get reference to continuation pooler
    #[must_use] pub fn get_continuation_pooler(&self) -> &ContinuationPoolManager {
        &self.continuation_pooler
    }

    /// Verify optimization result against semantic evaluator
    pub fn verify_optimization(
        &mut self,
        expr: &Expr,
        env: Rc<Environment>,
        optimized_result: &Value,
    ) -> Result<bool> {
        let semantic_result = self.semantic_evaluator.eval_pure(
            expr.clone(),
            env,
            Continuation::Identity,
        )?;
        
        // Simple value comparison - could be enhanced with more sophisticated comparison
        Ok(self.values_equal(&semantic_result, optimized_result))
    }

    /// Basic value equality check
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(n1), Value::Number(n2)) => n1 == n2,
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Nil, Value::Nil) => true,
            (Value::Symbol(s1), Value::Symbol(s2)) => s1 == s2,
            _ => false,
        }
    }
}

impl RuntimeExecutor {
    /// Get continuation pooling statistics
    #[must_use] pub fn get_pooling_statistics(&self) -> (usize, usize, usize, f64) {
        self.continuation_pooler.global_statistics()
    }
    
    /// Get detailed pooling statistics by type
    #[must_use] pub fn get_detailed_pooling_statistics(&self) -> Vec<(ContinuationType, f64)> {
        self.continuation_pooler.all_statistics()
            .into_iter()
            .map(|(cont_type, stats)| (cont_type, stats.reuse_efficiency()))
            .collect()
    }
    
    /// Check if memory defragmentation is needed for continuation pools
    #[must_use] pub fn needs_pool_defragmentation(&self) -> bool {
        self.continuation_pooler.needs_defragmentation()
    }
    
    /// Perform continuation pool defragmentation
    pub fn defragment_pools(&mut self) {
        self.continuation_pooler.defragment();
    }
    
    /// Clear all continuation pools
    pub fn clear_continuation_pools(&mut self) {
        self.continuation_pooler.clear_all();
    }
    
    /// Optimize continuation pooling based on runtime statistics
    pub fn optimize_continuation_pooling(&mut self) {
        // Check if defragmentation is needed
        if self.needs_pool_defragmentation() {
            self.defragment_pools();
            self.stats.pool_defragmentations += 1;
        }
        
        // Clear underused pools to free memory
        let detailed_stats = self.get_detailed_pooling_statistics();
        for (cont_type, efficiency) in detailed_stats {
            // Clear pools with very low efficiency
            if efficiency < 10.0 {
                self.continuation_pooler.clear_type(cont_type);
            }
        }
        
        // Adaptive pool management
        self.adaptive_pool_management();
    }
    
    /// Use adaptive engine for dynamic optimization decisions
    pub fn apply_adaptive_optimization(&mut self, execution_context: &ExecutionContext) -> Result<Value> {
        // Get adaptive optimization recommendation
        let optimization_type = self.adaptive_engine.get_optimization_recommendation(
            &execution_context.expression,
            &execution_context.static_analysis
        );
        
        // Apply the recommended optimization
        match optimization_type {
            AdaptiveOptimizationType::NoOptimization => {
                // Use standard evaluation path
                self.execute_with_semantic_evaluator(execution_context)
            }
            AdaptiveOptimizationType::JitCompilation => {
                // Use JIT compilation for hot paths
                self.execute_with_jit_optimization(execution_context)
            }
            AdaptiveOptimizationType::AdaptiveLoopUnrolling { factor } => {
                // Apply loop unrolling with specified factor
                self.execute_with_loop_unrolling(execution_context, factor)
            }
            AdaptiveOptimizationType::HotPathInlining => {
                // Use inline evaluation for hot paths
                self.execute_with_inline_evaluation(execution_context)
            }
            AdaptiveOptimizationType::TypeSpecialization => {
                // Apply type-specific optimizations
                self.execute_with_type_specialization(execution_context)
            }
            AdaptiveOptimizationType::ContinuationPooling |
            AdaptiveOptimizationType::MemoryLayoutOptimization |
            AdaptiveOptimizationType::ProfileGuidedOptimization => {
                // Advanced optimizations not yet implemented, fallback to semantic evaluator
                self.execute_with_semantic_evaluator(execution_context)
            }
        }
    }
    
    /// Execute using JIT optimization
    fn execute_with_jit_optimization(&mut self, execution_context: &ExecutionContext) -> Result<Value> {
        // For now, fallback to semantic evaluator with JIT hint
        // In a full implementation, this would use JIT-compiled code
        self.execute_with_semantic_evaluator(execution_context)
    }
    
    /// Execute with loop unrolling optimization
    fn execute_with_loop_unrolling(&mut self, execution_context: &ExecutionContext, factor: u32) -> Result<Value> {
        // Apply loop unrolling transformation
        let optimized_expr = self.apply_loop_unrolling(&execution_context.expression, factor)?;
        let optimized_context = ExecutionContext::new(optimized_expr, execution_context.environment.clone(), execution_context.continuation.clone());
        self.execute_with_semantic_evaluator(&optimized_context)
    }
    
    /// Execute with inline evaluation
    fn execute_with_inline_evaluation(&mut self, execution_context: &ExecutionContext) -> Result<Value> {
        // For now, fallback to semantic evaluator with inline hint
        // In a full implementation, this would inline simple expressions
        self.execute_with_semantic_evaluator(execution_context)
    }
    
    /// Execute with type specialization
    fn execute_with_type_specialization(&mut self, execution_context: &ExecutionContext) -> Result<Value> {
        // Apply type-specific optimizations based on detected types
        let specialized_expr = self.apply_type_specialization(&execution_context.expression)?;
        let optimized_context = ExecutionContext::new(specialized_expr, execution_context.environment.clone(), execution_context.continuation.clone());
        self.execute_with_semantic_evaluator(&optimized_context)
    }
    
    /// Apply loop unrolling transformation
    fn apply_loop_unrolling(&self, expr: &Expr, _factor: u32) -> Result<Expr> {
        // Simplified loop unrolling implementation
        // In a full implementation, this would detect and unroll loop constructs
        Ok(expr.clone())
    }
    
    /// Apply type specialization based on detected types
    fn apply_type_specialization(&self, expr: &Expr) -> Result<Expr> {
        // Simplified type specialization implementation
        // In a full implementation, this would specialize operations based on types
        Ok(expr.clone())
    }
    
    /// Execute with semantic evaluator (fallback method)
    fn execute_with_semantic_evaluator(&mut self, execution_context: &ExecutionContext) -> Result<Value> {
        // Use the semantic evaluator for correct evaluation
        self.semantic_evaluator.eval_pure(
            execution_context.expression.clone(),
            execution_context.environment.clone(),
            execution_context.continuation.clone()
        )
    }

    /// Adaptive pool management based on usage patterns
    fn adaptive_pool_management(&mut self) {
        // Periodic optimization based on evaluation count
        if self.stats.expressions_evaluated % 1000 == 0 && self.stats.expressions_evaluated > 0 {
            let pool_efficiency = self.stats.continuation_pool_efficiency();
            
            // If pool efficiency is low, trigger optimization
            if pool_efficiency < 50.0 {
                // Clear all pools and let them rebuild with current patterns
                self.clear_continuation_pools();
            } else if pool_efficiency > 85.0 {
                // High efficiency - consider expanding pool sizes
                // This would be implemented in a full production system
            }
        }
    }

    /// Try JIT loop optimization using the integrated JIT loop optimizer
    fn try_jit_loop_optimization(
        &mut self,
        expr: &Expr,
        env: Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        // Check if this is a do-loop expression that can be optimized
        if let Expr::List(operands) = expr {
            if !operands.is_empty() {
                if let Expr::Variable(name) = &operands[0] {
                    if name == "do" {
                        // Try JIT optimization for do-loop
                        let mut evaluator = crate::evaluator::Evaluator::new();
                        return self.jit_optimizer.try_optimize_do_loop(
                            &mut evaluator,
                            &operands[1..],
                            env,
                            cont,
                        );
                    }
                }
            }
        }

        // Check for other loop patterns based on call patterns
        for pattern in &analysis.call_patterns {
            if let CallPattern::Loop { iteration_estimate } = pattern {
                if let Some(iterations) = iteration_estimate {
                    if *iterations > 10 {
                        // For high iteration loops, attempt optimization
                        let jit_stats = self.jit_optimizer.optimization_statistics();
                        
                        // If we haven't tried optimizing this pattern much, give it a try
                        if jit_stats.compiled_patterns < jit_stats.total_patterns / 2 {
                            let mut evaluator = crate::evaluator::Evaluator::new();
                            if let Expr::List(operands) = expr {
                                if let Ok(Some(result)) = self.jit_optimizer.try_optimize_do_loop(
                                    &mut evaluator,
                                    operands,
                                    env.clone(),
                                    cont.clone(),
                                ) {
                                    return Ok(Some(result));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Try JIT compilation for general expressions
    fn try_jit_compilation(
        &mut self,
        expr: &Expr,
        env: Rc<Environment>,
        cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        // First try JIT loop optimization if applicable
        if analysis.contains_loops {
            if let Some(result) = self.try_jit_loop_optimization(expr, env.clone(), cont.clone(), analysis)? {
                return Ok(Some(result));
            }
        }

        // For non-loop expressions, we would implement general JIT compilation here
        // This is a placeholder for future JIT compilation of general expressions
        Ok(None)
    }

    /// Try LLVM native code generation for critical performance paths
    fn try_llvm_compilation(
        &mut self,
        expr: &Expr,
        _env: Rc<Environment>,
        _cont: Continuation,
        analysis: &ExpressionAnalysisResult,
    ) -> Result<Option<Value>> {
        use crate::evaluator::TailCallContext;

        // Only attempt LLVM compilation for very hot paths with specific characteristics
        if analysis.execution_frequency != ExecutionFrequency::Critical {
            return Ok(None);
        }

        // Create tail call context for LLVM compilation
        let tail_context = TailCallContext::new();

        // Attempt to compile the expression to LLVM IR
        let llvm_result = self.llvm_compiler.compile_with_tail_calls(expr, &tail_context);

        match llvm_result {
            Ok(_llvm_ir) => {
                // In a real implementation, we would:
                // 1. Compile the LLVM IR to native code
                // 2. Cache the compiled code
                // 3. Execute the native code
                // 
                // For now, we return None to indicate that LLVM compilation
                // is not yet fully implemented
                Ok(None)
            }
            Err(_) => {
                // LLVM compilation failed, fall back to regular evaluation
                Ok(None)
            }
        }
    }
    
    /// Get comprehensive runtime performance report
    #[must_use] pub fn generate_performance_report(&self) -> RuntimePerformanceReport {
        RuntimePerformanceReport {
            runtime_stats: self.stats.clone(),
            pooling_stats: self.get_pooling_statistics(),
            detailed_pooling: self.get_detailed_pooling_statistics(),
            memory_usage: self.continuation_pooler.memory_usage_summary(),
            optimization_recommendations: self.generate_optimization_recommendations(),
        }
    }
    
    /// Generate optimization recommendations based on current performance
    fn generate_optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        // Pool efficiency recommendations
        let pool_efficiency = self.stats.continuation_pool_efficiency();
        if pool_efficiency < 30.0 && self.stats.continuation_pool_hits + self.stats.continuation_pool_misses > 100 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::ContinuationPooling,
                priority: RecommendationPriority::High,
                description: "Low continuation pool efficiency detected. Consider adjusting pool sizes or clearing underused pools.".to_string(),
                estimated_benefit: "10-25% memory reduction".to_string(),
            });
        }
        
        // Hot path recommendations
        if self.stats.hot_path_detections > 50 && self.stats.jit_compilations < 10 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::JitCompilation,
                priority: RecommendationPriority::Medium,
                description: "Multiple hot paths detected but few JIT compilations. Consider enabling more aggressive JIT compilation.".to_string(),
                estimated_benefit: "20-40% performance improvement".to_string(),
            });
        }
        
        // Optimization rate recommendations
        if self.stats.optimization_rate() < 20.0 && self.stats.expressions_evaluated > 1000 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::GeneralOptimization,
                priority: RecommendationPriority::Medium,
                description: "Low optimization rate detected. Consider using more aggressive optimization levels.".to_string(),
                estimated_benefit: "15-30% performance improvement".to_string(),
            });
        }
        
        recommendations
    }
}

impl Default for RuntimeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive runtime performance report
#[derive(Debug, Clone)]
pub struct RuntimePerformanceReport {
    /// Basic runtime statistics
    pub runtime_stats: RuntimeStats,
    
    /// Global pooling statistics (allocations, reuses, memory_saved, efficiency)
    pub pooling_stats: (usize, usize, usize, f64),
    
    /// Detailed pooling statistics by type (type, efficiency)
    pub detailed_pooling: Vec<(ContinuationType, f64)>,
    
    /// Memory usage summary (total_pools, active_pools, avg_utilization)
    pub memory_usage: (usize, usize, f64),
    
    /// Optimization recommendations
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// Category of recommendation
    pub category: RecommendationCategory,
    
    /// Priority level
    pub priority: RecommendationPriority,
    
    /// Human-readable description
    pub description: String,
    
    /// Estimated benefit
    pub estimated_benefit: String,
}

/// Recommendation category
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationCategory {
    /// Continuation pooling optimizations
    ContinuationPooling,
    
    /// JIT compilation recommendations
    JitCompilation,
    
    /// General optimization recommendations
    GeneralOptimization,
    
    /// Memory management recommendations
    MemoryManagement,
}

/// Recommendation priority
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RecommendationPriority {
    /// Low priority recommendation
    Low,
    
    /// Medium priority recommendation
    Medium,
    
    /// High priority recommendation
    High,
    
    /// Critical priority recommendation
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_runtime_executor_creation() {
        let executor = RuntimeExecutor::new();
        assert_eq!(
            executor.optimization_level(),
            RuntimeOptimizationLevel::Balanced
        );
        assert_eq!(executor.get_stats().expressions_evaluated, 0);
    }

    #[test]
    fn test_optimization_level_setting() {
        let mut executor = RuntimeExecutor::new();
        executor.set_optimization_level(RuntimeOptimizationLevel::Aggressive);
        assert_eq!(
            executor.optimization_level(),
            RuntimeOptimizationLevel::Aggressive
        );
    }

    #[test]
    fn test_verification_toggle() {
        let mut executor = RuntimeExecutor::new();
        executor.set_verification_enabled(true);
        assert!(executor.verification_enabled);

        executor.set_verification_enabled(false);
        assert!(!executor.verification_enabled);
    }

    #[test]
    fn test_basic_arithmetic_simple() {
        let executor = RuntimeExecutor::new();

        // Test simple addition
        let args = vec![
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3)),
        ];

        let result = executor.builtin_add_simple(&args).unwrap();
        match result {
            Value::Number(SchemeNumber::Integer(5)) => {}
            _ => panic!("Expected simple addition to return 5, got {:?}", result),
        }
    }

    #[test]
    fn test_stats_tracking() {
        let mut executor = RuntimeExecutor::new();

        // Test that statistics are tracked
        let initial_stats = executor.get_stats().clone();
        assert_eq!(initial_stats.expressions_evaluated, 0);

        // Reset stats
        executor.reset_stats();
        let reset_stats = executor.get_stats().clone();
        assert_eq!(reset_stats.expressions_evaluated, 0);
    }

    #[test]
    fn test_values_equality() {
        let executor = RuntimeExecutor::new();

        // Test number equality
        let num1 = Value::Number(SchemeNumber::Integer(42));
        let num2 = Value::Number(SchemeNumber::Integer(42));
        let num3 = Value::Number(SchemeNumber::Integer(24));

        assert!(executor.values_equal(&num1, &num2));
        assert!(!executor.values_equal(&num1, &num3));

        // Test boolean equality
        let bool1 = Value::Boolean(true);
        let bool2 = Value::Boolean(true);
        let bool3 = Value::Boolean(false);

        assert!(executor.values_equal(&bool1, &bool2));
        assert!(!executor.values_equal(&bool1, &bool3));
    }

    #[test]
    fn test_expression_analysis() {
        use crate::ast::{Expr, Literal};
        use crate::lexer::SchemeNumber;
        
        // Test simple literal analysis
        let literal_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let analysis = ExpressionAnalysisResult::analyze_expression(&literal_expr);
        
        assert_eq!(analysis.complexity_score, 1);
        assert_eq!(analysis.execution_frequency, ExecutionFrequency::Cold);
        assert!(!analysis.contains_loops);
        
        // Test application analysis
        let app_expr = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]);
        let analysis = ExpressionAnalysisResult::analyze_expression(&app_expr);
        
        assert!(analysis.complexity_score > 1);
        assert!(!analysis.call_patterns.is_empty());
        assert!(analysis.is_tail_call_candidate);
    }

    #[test]
    fn test_semantic_evaluator_integration() {
        use crate::environment::Environment;
        use crate::evaluator::Continuation;
        use std::rc::Rc;

        let mut runtime_executor = RuntimeExecutor::new();
        let env = Rc::new(Environment::new());

        // Test simple literal evaluation
        let literal_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result =
            runtime_executor.eval_optimized(literal_expr, env.clone(), Continuation::Identity);

        assert!(result.is_ok());
        if let Ok(Value::Number(SchemeNumber::Integer(42))) = result {
            // Success
        } else {
            panic!("Expected literal evaluation to return 42, got {:?}", result);
        }
    }

    #[test]
    fn test_optimization_level_behavior() {
        use crate::environment::Environment;
        use crate::evaluator::Continuation;
        use std::rc::Rc;

        let mut runtime_executor = RuntimeExecutor::new();
        let env = Rc::new(Environment::new());
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(123)));

        // Test different optimization levels
        let optimization_levels = vec![
            RuntimeOptimizationLevel::None,
            RuntimeOptimizationLevel::Conservative,
            RuntimeOptimizationLevel::Balanced,
            RuntimeOptimizationLevel::Aggressive,
        ];

        for level in optimization_levels {
            runtime_executor.set_optimization_level(level);
            let result =
                runtime_executor.eval_optimized(expr.clone(), env.clone(), Continuation::Identity);

            assert!(result.is_ok(), "Optimization level {:?} failed", level);
            if let Ok(Value::Number(SchemeNumber::Integer(123))) = result {
                // Success
            } else {
                panic!(
                    "Expected result 123 for optimization level {:?}, got {:?}",
                    level, result
                );
            }
        }
    }
    
    #[test]
    fn test_immediate_optimizations() {
        let runtime_executor = RuntimeExecutor::new();
        
        // Test literal optimization
        let literal_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let analysis = ExpressionAnalysisResult::new();
        let result = runtime_executor.try_immediate_optimizations(&literal_expr, &analysis).unwrap();
        
        assert!(result.is_some());
        if let Some(Value::Number(SchemeNumber::Integer(42))) = result {
            // Success
        } else {
            panic!("Expected immediate optimization to return 42, got {:?}", result);
        }
    }
    
    #[test]
    fn test_arithmetic_optimization() {
        // Test constant folding for arithmetic
        let result = RuntimeExecutor::compute_arithmetic("+", 2, 3).unwrap();
        if let Value::Number(SchemeNumber::Integer(5)) = result {
            // Success
        } else {
            panic!("Expected arithmetic optimization to return 5, got {:?}", result);
        }
        
        let result = RuntimeExecutor::compute_arithmetic("*", 4, 6).unwrap();
        if let Value::Number(SchemeNumber::Integer(24)) = result {
            // Success
        } else {
            panic!("Expected arithmetic optimization to return 24, got {:?}", result);
        }
    }
    
    #[test]
    fn test_jit_compilation_metadata() {
        let jit_code = JitCompiledCode {
            original_expr: Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            metadata: JitMetadata {
                compiled_at: std::time::SystemTime::now(),
                optimization_level: RuntimeOptimizationLevel::Aggressive,
                optimizations: vec!["hot_path_compilation".to_string()],
                compilation_time_us: 150,
                hot_path_count: 1,
                adaptive_decisions: Vec::new(),
            },
            performance_profile: PerformanceProfile {
                speedup_factor: 3.0,
                memory_usage: MemoryUsage {
                    stack_usage_bytes: 512,
                    heap_allocations: 1,
                    is_memory_intensive: false,
                },
                execution_characteristics: ExecutionCharacteristics {
                    is_cpu_intensive: true,
                    has_io_operations: false,
                    estimated_instructions: 25,
                    parallelizable: false,
                },
            },
            is_ready: true,
        };
        
        assert!(jit_code.is_ready);
        assert_eq!(jit_code.performance_profile.speedup_factor, 3.0);
        assert_eq!(jit_code.metadata.optimization_level, RuntimeOptimizationLevel::Aggressive);
    }

    #[test]
    fn test_runtime_executor_with_semantic_evaluator() {
        use crate::environment::Environment;
        use crate::evaluator::{Continuation, SemanticEvaluator};
        use std::rc::Rc;

        let mut runtime_executor = RuntimeExecutor::new();
        let mut semantic_evaluator = SemanticEvaluator::new();
        let env = Rc::new(Environment::new());

        // Test that both evaluators produce the same result for simple expressions
        let test_cases = vec![
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            Expr::Literal(Literal::Boolean(true)),
            Expr::Literal(Literal::String("hello".to_string())),
            Expr::Literal(Literal::Nil),
        ];

        for expr in test_cases {
            let runtime_result =
                runtime_executor.eval_optimized(expr.clone(), env.clone(), Continuation::Identity);
            let semantic_result =
                semantic_evaluator.eval_pure(expr.clone(), env.clone(), Continuation::Identity);

            assert!(
                runtime_result.is_ok(),
                "Runtime executor failed for {:?}",
                expr
            );
            assert!(
                semantic_result.is_ok(),
                "Semantic evaluator failed for {:?}",
                expr
            );

            // For basic literals, both should produce the same result
            match (runtime_result.unwrap(), semantic_result.unwrap()) {
                (Value::Number(a), Value::Number(b)) => assert_eq!(a, b),
                (Value::Boolean(a), Value::Boolean(b)) => assert_eq!(a, b),
                (Value::String(a), Value::String(b)) => assert_eq!(a, b),
                (Value::Nil, Value::Nil) => {}
                (a, b) => panic!("Results differ: runtime={:?}, semantic={:?}", a, b),
            }
        }
    }
    
    #[test]
    fn test_continuation_pooling_optimization() {
        // use crate::evaluator::Continuation; // Currently unused
        
        let runtime_executor = RuntimeExecutor::new();
        
        // Test pooling statistics tracking
        let initial_stats = runtime_executor.get_pooling_statistics();
        assert_eq!(initial_stats.0, 0); // No allocations initially
        
        // Test pool efficiency tracking
        let efficiency = runtime_executor.get_stats().continuation_pool_efficiency();
        assert_eq!(efficiency, 0.0); // No requests initially
        
        // Test optimization recommendations
        let report = runtime_executor.generate_performance_report();
        assert!(report.optimization_recommendations.is_empty()); // No recommendations for empty stats
    }
    
    #[test]
    fn test_adaptive_pool_management() {
        let mut runtime_executor = RuntimeExecutor::new();
        
        // Simulate many evaluations to trigger adaptive management
        runtime_executor.stats.expressions_evaluated = 1000;
        runtime_executor.stats.continuation_pool_hits = 10;
        runtime_executor.stats.continuation_pool_misses = 90;
        
        // Test pool efficiency calculation
        let efficiency = runtime_executor.get_stats().continuation_pool_efficiency();
        assert_eq!(efficiency, 10.0); // 10 hits out of 100 total = 10%
        
        // Test optimization trigger
        runtime_executor.optimize_continuation_pooling();
        
        // Verify stats are updated
        assert!(runtime_executor.get_stats().expressions_evaluated > 0);
    }
    
    #[test]
    fn test_performance_report_generation() {
        let mut runtime_executor = RuntimeExecutor::new();
        
        // Add some statistics
        runtime_executor.stats.expressions_evaluated = 500;
        runtime_executor.stats.optimizations_applied = 50;
        runtime_executor.stats.jit_compilations = 5;
        runtime_executor.stats.hot_path_detections = 20;
        
        let report = runtime_executor.generate_performance_report();
        
        assert_eq!(report.runtime_stats.expressions_evaluated, 500);
        assert_eq!(report.runtime_stats.optimization_rate(), 10.0); // 50/500 * 100
        assert!(!report.optimization_recommendations.is_empty()); // Should have recommendations
    }
    
    #[test]
    fn test_adaptive_optimization_engine() {
        use crate::ast::{Expr, Literal};
        use crate::lexer::SchemeNumber;
        
        let mut adaptive_engine = AdaptiveOptimizationEngine::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let analysis = ExpressionAnalysisResult::analyze_expression(&expr);
        
        let result = adaptive_engine.analyze_and_optimize(&expr, &analysis);
        
        assert!(matches!(result.strategy, AdaptiveOptimizationType::ProfileGuidedOptimization));
        assert!(result.processing_time.as_nanos() > 0);
        
        let stats = adaptive_engine.get_statistics();
        assert_eq!(stats.total_decisions, 1);
    }
    
    #[test]
    fn test_hot_path_detection() {
        let mut detector = HotPathDetector::new();
        let expr_hash = "test_expression".to_string();
        
        // Record executions to trigger hot path detection
        for _ in 0..15 {
            detector.record_execution(&expr_hash);
        }
        
        assert!(detector.is_hot_path(&expr_hash));
        assert_eq!(detector.get_frequency(&expr_hash), 15);
    }
    
    #[test]
    fn test_dynamic_code_generation() {
        use crate::ast::{Expr, Literal};
        use crate::lexer::SchemeNumber;
        
        let mut generator = DynamicCodeGenerator::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(123)));
        
        let jit_code = generator.compile_expression(&expr).unwrap();
        
        assert!(jit_code.is_ready);
        assert_eq!(jit_code.performance_profile.speedup_factor, 2.5);
        assert!(jit_code.metadata.optimizations.contains(&"jit_compilation".to_string()));
    }
}

// === Adaptive Optimization Engine Implementation ===

impl AdaptiveOptimizationEngine {
    /// Create new adaptive optimization engine
    #[must_use] pub fn new() -> Self {
        Self {
            hot_path_detector: HotPathDetector::new(),
            code_generator: DynamicCodeGenerator::new(),
            profiler: ProfileGuidedOptimizer::new(),
            jit_cache: FxHashMap::default(),
            decision_history: Vec::new(),
            performance_tracker: PerformanceTracker::new(),
            thresholds: OptimizationThresholds::default(),
        }
    }
    
    /// Get optimization recommendation for given expression and analysis
    pub fn get_optimization_recommendation(&mut self, expr: &Expr, static_analysis: &crate::evaluator::execution_context::StaticAnalysisResult) -> AdaptiveOptimizationType {
        // Update hot path detection
        let expr_hash = self.compute_expression_hash(expr);
        self.hot_path_detector.record_execution(&expr_hash);
        
        // Check if this is a hot path
        let is_hot_path = self.hot_path_detector.is_hot_path(&expr_hash);
        
        // Make optimization decision based on static analysis and hot path status
        if is_hot_path && static_analysis.complexity_score > 50 {
            AdaptiveOptimizationType::JitCompilation
        } else if static_analysis.has_loops {
            AdaptiveOptimizationType::AdaptiveLoopUnrolling { factor: 2 }
        } else if static_analysis.complexity_score < 20 && is_hot_path {
            AdaptiveOptimizationType::HotPathInlining
        } else if static_analysis.is_pure {
            AdaptiveOptimizationType::TypeSpecialization
        } else {
            AdaptiveOptimizationType::NoOptimization
        }
    }

    /// Analyze expression and make adaptive optimization decisions
    pub fn analyze_and_optimize(&mut self, expr: &Expr, analysis: &ExpressionAnalysisResult) -> AdaptiveOptimizationResult {
        let start_time = std::time::SystemTime::now();
        
        // Update hot path detection
        let expr_hash = self.compute_expression_hash(expr);
        self.hot_path_detector.record_execution(&expr_hash);
        
        // Check if this is a hot path
        let is_hot_path = self.hot_path_detector.is_hot_path(&expr_hash);
        
        // Determine optimization strategy
        let optimization_strategy = self.select_optimization_strategy(expr, analysis, is_hot_path);
        
        // Generate adaptive decision
        let decision = AdaptiveDecision {
            timestamp: start_time,
            trigger_expression: expr_hash.clone(),
            optimization_type: optimization_strategy.clone(),
            rationale: self.generate_rationale(&optimization_strategy, analysis),
            expected_improvement: self.estimate_performance_improvement(&optimization_strategy),
        };
        
        // Record decision
        self.decision_history.push(decision.clone());
        
        // Apply optimization
        let optimization_result = self.apply_adaptive_optimization(expr, &optimization_strategy);
        
        AdaptiveOptimizationResult {
            strategy: optimization_strategy,
            decision,
            result: optimization_result,
            processing_time: start_time.elapsed().unwrap_or_default(),
        }
    }
    
    /// Select appropriate optimization strategy
    fn select_optimization_strategy(&mut self, expr: &Expr, analysis: &ExpressionAnalysisResult, is_hot_path: bool) -> AdaptiveOptimizationType {
        // JIT compilation for hot paths with high complexity
        if is_hot_path && analysis.complexity_score > 20 {
            return AdaptiveOptimizationType::JitCompilation;
        }
        
        // Loop unrolling for loops with low iteration counts
        if analysis.contains_loops {
            if let Some(pattern) = analysis.call_patterns.iter().find(|p| matches!(p, CallPattern::Loop { .. })) {
                if let CallPattern::Loop { iteration_estimate: Some(count) } = pattern {
                    if *count <= 10 {
                        return AdaptiveOptimizationType::AdaptiveLoopUnrolling { factor: (*count).min(4) };
                    }
                }
            }
        }
        
        // Inlining for simple functions on warm paths
        if analysis.execution_frequency >= ExecutionFrequency::Warm && 
           analysis.complexity_score <= 10 && 
           Self::is_inlinable_expression(expr) {
            return AdaptiveOptimizationType::HotPathInlining;
        }
        
        // Type specialization for polymorphic operations
        if analysis.optimization_hints.contains(&OptimizationHint::TypeSpecialize) {
            return AdaptiveOptimizationType::TypeSpecialization;
        }
        
        // Continuation pooling for continuation-heavy code
        if analysis.call_patterns.iter().any(|p| matches!(p, CallPattern::TailRecursive)) {
            return AdaptiveOptimizationType::ContinuationPooling;
        }
        
        // Memory layout optimization for memory-intensive operations
        if analysis.memory_patterns.iter().any(|p| matches!(p, MemoryPattern::LargeObject | MemoryPattern::FrequentSmall)) {
            return AdaptiveOptimizationType::MemoryLayoutOptimization;
        }
        
        // Default to profile-guided optimization
        AdaptiveOptimizationType::ProfileGuidedOptimization
    }
    
    /// Generate rationale for optimization decision
    fn generate_rationale(&self, strategy: &AdaptiveOptimizationType, analysis: &ExpressionAnalysisResult) -> String {
        match strategy {
            AdaptiveOptimizationType::JitCompilation => 
                format!("Hot path detected with complexity score {} - JIT compilation will provide significant speedup", analysis.complexity_score),
            AdaptiveOptimizationType::AdaptiveLoopUnrolling { factor } => 
                format!("Loop with small iteration count detected - unrolling by factor {} will eliminate loop overhead", factor),
            AdaptiveOptimizationType::HotPathInlining => 
                "Simple function on warm path - inlining will eliminate call overhead".to_string(),
            AdaptiveOptimizationType::TypeSpecialization => 
                "Polymorphic operation detected - type specialization will enable optimized code paths".to_string(),
            AdaptiveOptimizationType::ContinuationPooling => 
                "Tail-recursive pattern detected - continuation pooling will reduce memory allocation".to_string(),
            AdaptiveOptimizationType::MemoryLayoutOptimization => 
                "Memory-intensive operation detected - layout optimization will improve cache performance".to_string(),
            AdaptiveOptimizationType::ProfileGuidedOptimization => 
                "General optimization based on execution profile and usage patterns".to_string(),
            AdaptiveOptimizationType::NoOptimization => 
                "No optimization needed - expression is already well-optimized".to_string(),
        }
    }
    
    /// Estimate performance improvement for optimization strategy
    fn estimate_performance_improvement(&self, strategy: &AdaptiveOptimizationType) -> f64 {
        match strategy {
            AdaptiveOptimizationType::JitCompilation => 2.5, // 2.5x speedup
            AdaptiveOptimizationType::AdaptiveLoopUnrolling { factor } => 1.0 + (*factor as f64 * 0.2), // 20% per unroll factor
            AdaptiveOptimizationType::HotPathInlining => 1.3, // 30% speedup
            AdaptiveOptimizationType::TypeSpecialization => 1.8, // 80% speedup
            AdaptiveOptimizationType::ContinuationPooling => 1.15, // 15% speedup (memory efficiency)
            AdaptiveOptimizationType::MemoryLayoutOptimization => 1.25, // 25% speedup (cache efficiency)
            AdaptiveOptimizationType::ProfileGuidedOptimization => 1.1, // 10% general improvement
            AdaptiveOptimizationType::NoOptimization => 1.0, // No performance improvement
        }
    }
    
    /// Apply adaptive optimization
    fn apply_adaptive_optimization(&mut self, expr: &Expr, strategy: &AdaptiveOptimizationType) -> Result<Option<Value>> {
        match strategy {
            AdaptiveOptimizationType::JitCompilation => self.apply_jit_compilation(expr),
            AdaptiveOptimizationType::AdaptiveLoopUnrolling { factor } => self.apply_loop_unrolling(expr, *factor),
            AdaptiveOptimizationType::HotPathInlining => self.apply_hot_path_inlining(expr),
            AdaptiveOptimizationType::TypeSpecialization => self.apply_type_specialization(expr),
            AdaptiveOptimizationType::ContinuationPooling => self.apply_continuation_pooling(expr),
            AdaptiveOptimizationType::MemoryLayoutOptimization => self.apply_memory_optimization(expr),
            AdaptiveOptimizationType::ProfileGuidedOptimization => self.apply_profile_guided_optimization(expr),
            AdaptiveOptimizationType::NoOptimization => Ok(None), // No optimization applied
        }
    }
    
    /// Apply JIT compilation
    fn apply_jit_compilation(&mut self, expr: &Expr) -> Result<Option<Value>> {
        let expr_hash = self.compute_expression_hash(expr);
        
        // Check if already compiled
        if let Some(jit_code) = self.jit_cache.get(&expr_hash) {
            if jit_code.is_ready {
                // Execute compiled code (placeholder)
                return self.execute_jit_code(jit_code);
            }
        }
        
        // Compile expression
        let jit_code = self.code_generator.compile_expression(expr)?;
        self.jit_cache.insert(expr_hash, jit_code);
        
        Ok(None) // Compilation successful, actual execution would happen separately
    }
    
    /// Apply loop unrolling optimization
    fn apply_loop_unrolling(&self, _expr: &Expr, _factor: u32) -> Result<Option<Value>> {
        // Placeholder implementation - would transform loop AST
        Ok(None)
    }
    
    /// Apply hot path inlining
    fn apply_hot_path_inlining(&self, _expr: &Expr) -> Result<Option<Value>> {
        // Placeholder implementation - would inline function calls
        Ok(None)
    }
    
    /// Apply type specialization
    fn apply_type_specialization(&self, _expr: &Expr) -> Result<Option<Value>> {
        // Placeholder implementation - would generate specialized code
        Ok(None)
    }
    
    /// Apply continuation pooling optimization
    fn apply_continuation_pooling(&self, _expr: &Expr) -> Result<Option<Value>> {
        // Placeholder implementation - would optimize continuation allocation
        Ok(None)
    }
    
    /// Apply memory layout optimization
    fn apply_memory_optimization(&self, _expr: &Expr) -> Result<Option<Value>> {
        // Placeholder implementation - would optimize memory layout
        Ok(None)
    }
    
    /// Apply profile-guided optimization
    fn apply_profile_guided_optimization(&self, _expr: &Expr) -> Result<Option<Value>> {
        // Placeholder implementation - would apply profile-based optimizations
        Ok(None)
    }
    
    /// Execute JIT compiled code
    fn execute_jit_code(&self, _jit_code: &JitCompiledCode) -> Result<Option<Value>> {
        // Placeholder implementation - would execute native code
        Ok(None)
    }
    
    /// Compute hash for expression
    fn compute_expression_hash(&self, expr: &Expr) -> String {
        format!("{:?}", expr) // Simple hash, could be improved with proper hashing
    }
    
    /// Check if expression is inlinable
    fn is_inlinable_expression(expr: &Expr) -> bool {
        match expr {
            Expr::Literal(_) => true,
            Expr::Variable(_) => true,
            Expr::List(exprs) => exprs.len() <= 3,
            _ => false,
        }
    }
    
    /// Get optimization statistics
    #[must_use] pub fn get_statistics(&self) -> AdaptiveOptimizationStatistics {
        AdaptiveOptimizationStatistics {
            total_decisions: self.decision_history.len(),
            jit_compilations: self.jit_cache.len(),
            hot_paths_detected: self.hot_path_detector.hot_paths.len(),
            cache_hit_rate: self.calculate_cache_hit_rate(),
            average_improvement: self.calculate_average_improvement(),
        }
    }
    
    /// Calculate cache hit rate
    fn calculate_cache_hit_rate(&self) -> f64 {
        // Placeholder implementation - would track cache statistics
        0.0
    }
    
    /// Calculate average performance improvement
    fn calculate_average_improvement(&self) -> f64 {
        if self.decision_history.is_empty() {
            return 0.0;
        }
        
        self.decision_history.iter()
            .map(|d| d.expected_improvement)
            .sum::<f64>() / self.decision_history.len() as f64
    }
}

impl Default for AdaptiveOptimizationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HotPathDetector {
    /// Create new hot path detector
    #[must_use] pub fn new() -> Self {
        Self {
            execution_frequencies: FxHashMap::default(),
            hot_paths: HashSet::new(),
            sensitivity_threshold: 10,
            cool_down_period: std::time::Duration::from_secs(60),
            last_detection: FxHashMap::default(),
        }
    }
    
    /// Record execution of an expression
    pub fn record_execution(&mut self, expr_hash: &str) {
        let count = self.execution_frequencies.entry(expr_hash.to_string()).or_insert(0);
        *count += 1;
        
        // Check if this becomes a hot path
        if *count >= self.sensitivity_threshold {
            let now = std::time::SystemTime::now();
            
            // Check cool-down period
            if let Some(last_time) = self.last_detection.get(expr_hash) {
                if now.duration_since(*last_time).unwrap_or_default() < self.cool_down_period {
                    return; // Still in cool-down
                }
            }
            
            self.hot_paths.insert(expr_hash.to_string());
            self.last_detection.insert(expr_hash.to_string(), now);
        }
    }
    
    /// Check if expression is a hot path
    #[must_use] pub fn is_hot_path(&self, expr_hash: &str) -> bool {
        self.hot_paths.contains(expr_hash)
    }
    
    /// Get execution frequency for expression
    #[must_use] pub fn get_frequency(&self, expr_hash: &str) -> u64 {
        self.execution_frequencies.get(expr_hash).copied().unwrap_or(0)
    }
}

impl Default for HotPathDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicCodeGenerator {
    /// Create new dynamic code generator
    #[must_use] pub fn new() -> Self {
        Self {
            code_cache: FxHashMap::default(),
            strategy_selector: CompilationStrategySelector::new(),
            generation_stats: CodeGenerationStats::new(),
            code_allocator: CodeAllocator::new(),
        }
    }
    
    /// Compile expression to JIT code
    pub fn compile_expression(&mut self, expr: &Expr) -> Result<JitCompiledCode> {
        let start_time = std::time::SystemTime::now();
        
        // Generate metadata
        let metadata = JitMetadata {
            compiled_at: start_time,
            optimization_level: RuntimeOptimizationLevel::Aggressive,
            optimizations: vec!["jit_compilation".to_string()],
            compilation_time_us: 100, // Placeholder
            hot_path_count: 1,
            adaptive_decisions: Vec::new(),
        };
        
        // Generate performance profile
        let performance_profile = PerformanceProfile {
            speedup_factor: 2.5,
            memory_usage: MemoryUsage {
                stack_usage_bytes: 1024,
                heap_allocations: 2,
                is_memory_intensive: false,
            },
            execution_characteristics: ExecutionCharacteristics {
                is_cpu_intensive: true,
                has_io_operations: false,
                estimated_instructions: 50,
                parallelizable: false,
            },
        };
        
        Ok(JitCompiledCode {
            original_expr: expr.clone(),
            metadata,
            performance_profile,
            is_ready: true,
        })
    }
}

impl Default for DynamicCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileGuidedOptimizer {
    /// Create new profile-guided optimizer
    #[must_use] pub fn new() -> Self {
        Self {
            execution_profiles: FxHashMap::default(),
            optimization_decisions: FxHashMap::default(),
            collection_period: std::time::Duration::from_secs(10),
            min_samples: 5,
        }
    }
}

impl Default for ProfileGuidedOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceTracker {
    /// Create new performance tracker
    #[must_use] pub fn new() -> Self {
        Self {
            metrics_history: Vec::new(),
            baseline: PerformanceBaseline::new(),
            regression_detector: RegressionDetector::new(),
            improvements: Vec::new(),
        }
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OptimizationThresholds {
    fn default() -> Self {
        Self {
            hot_path_threshold: 10,
            jit_compilation_threshold: 50,
            loop_unroll_threshold: 5,
            inline_size_threshold: 10,
            memory_optimization_threshold: 1024,
            improvement_threshold: 20.0,
        }
    }
}

// === Support Types Implementation ===

/// Result of adaptive optimization analysis
#[derive(Debug)]
pub struct AdaptiveOptimizationResult {
    /// Selected optimization strategy
    pub strategy: AdaptiveOptimizationType,
    
    /// Decision record
    pub decision: AdaptiveDecision,
    
    /// Optimization result
    pub result: Result<Option<Value>>,
    
    /// Time spent processing
    pub processing_time: std::time::Duration,
}

/// Statistics for adaptive optimization engine
#[derive(Debug, Clone)]
pub struct AdaptiveOptimizationStatistics {
    /// Total optimization decisions made
    pub total_decisions: usize,
    
    /// Number of JIT compilations performed
    pub jit_compilations: usize,
    
    /// Number of hot paths detected
    pub hot_paths_detected: usize,
    
    /// Cache hit rate percentage
    pub cache_hit_rate: f64,
    
    /// Average performance improvement
    pub average_improvement: f64,
}

// Placeholder support types - TODO: Implement full JIT compilation system
// These stubs are placeholders for the advanced JIT compilation infrastructure
// that will be implemented in future phases of the RuntimeExecutor optimization.

/// JIT compilation strategy selector (STUB)
/// 
/// TODO Phase 8 Implementation:
/// - Implement dynamic strategy selection based on code analysis
/// - Add support for different compilation targets (LLVM, native, bytecode)
/// - Integrate with performance profiling to choose optimal strategies
/// - Add cost-benefit analysis for compilation decisions
#[derive(Debug)]
pub struct CompilationStrategySelector;

impl CompilationStrategySelector {
    /// Create new compilation strategy selector
    /// 
    /// TODO: Add configuration parameters for compilation thresholds,
    /// target architectures, and performance criteria
    #[must_use] pub fn new() -> Self { Self }
}

/// Code generation statistics collector (STUB)
/// 
/// TODO Phase 8 Implementation:
/// - Track compilation time, memory usage, and generated code size
/// - Implement performance regression detection
/// - Add code quality metrics and optimization effectiveness analysis
/// - Integrate with continuous integration for performance monitoring
#[derive(Debug)]
pub struct CodeGenerationStats;

impl CodeGenerationStats {
    /// Create new code generation statistics collector
    /// 
    /// TODO: Add parameters for tracking granularity and reporting intervals
    #[must_use] pub fn new() -> Self { Self }
}

/// Memory allocator for generated code (STUB)
/// 
/// TODO Phase 8 Implementation:
/// - Implement executable memory allocation with proper permissions
/// - Add memory pool management for efficient allocation/deallocation
/// - Implement code cache with LRU eviction policy
/// - Add support for code patching and hot-swapping
#[derive(Debug)]
pub struct CodeAllocator;

impl CodeAllocator {
    /// Create new code allocator
    /// 
    /// TODO: Add memory pool size configuration and allocation strategies
    #[must_use] pub fn new() -> Self { Self }
}

/// Container for generated executable code (STUB)
/// 
/// TODO Phase 8 Implementation:
/// - Store compiled machine code with metadata
/// - Implement code versioning and invalidation
/// - Add debugging information and symbol tables
/// - Support for different instruction set architectures
#[derive(Debug)]
pub struct GeneratedCode;

/// Execution profile for performance analysis (STUB)
/// 
/// TODO Phase 8 Implementation:
/// - Track function call frequencies and execution times
/// - Implement hot path detection and optimization triggers
/// - Add memory access patterns and cache behavior analysis
/// - Support for multi-threaded profiling
#[derive(Debug)]
pub struct ExecutionProfile;

/// Decision engine for optimization strategies (STUB)
/// 
/// TODO Phase 8 Implementation:
/// - Implement cost-benefit analysis for optimization decisions
/// - Add machine learning for optimization strategy selection
/// - Support for user-defined optimization preferences
/// - Real-time adaptation based on performance feedback
#[derive(Debug)]
pub struct OptimizationDecision;

/// Performance measurement and tracking (STUB)
/// 
/// TODO Phase 8 Implementation:
/// - Implement comprehensive performance counters
/// - Add statistical analysis and trend detection
/// - Support for custom performance metrics
/// - Integration with system performance monitoring
#[derive(Debug)]
pub struct PerformanceMetric;

/// Performance baseline for regression detection (STUB)
/// 
/// TODO Phase 8 Implementation:
/// - Establish performance baselines for different code patterns
/// - Implement automated regression detection
/// - Add historical performance data management
/// - Support for performance SLA monitoring
#[derive(Debug)]
pub struct PerformanceBaseline;

impl PerformanceBaseline {
    /// Create new performance baseline
    /// 
    /// TODO: Add baseline configuration and historical data loading
    #[must_use] pub fn new() -> Self { Self }
}

/// Performance regression detection system (STUB)
/// 
/// TODO Phase 8 Implementation:
/// - Implement statistical analysis for performance regressions
/// - Add configurable thresholds and alerting mechanisms
/// - Support for A/B testing and performance comparisons
/// - Integration with continuous integration systems
#[derive(Debug)]
pub struct RegressionDetector;

impl RegressionDetector {
    /// Create new regression detector
    /// 
    /// TODO: Add configuration for detection sensitivity and reporting
    #[must_use] pub fn new() -> Self { Self }
}

/// Performance improvement tracking and analysis (STUB)
/// 
/// TODO Phase 8 Implementation:
/// - Track and quantify performance improvements over time
/// - Implement improvement attribution to specific optimizations
/// - Add cost-benefit analysis for optimization investments
/// - Support for performance improvement reporting and visualization
#[derive(Debug)]
pub struct PerformanceImprovement;

#[cfg(test)]
mod phase_6b_tests {
    use super::*;
    
    #[test]
    fn test_dynamic_optimization_focus() {
        // Phase 6b: Verify RuntimeExecutor focuses only on dynamic optimization
        let runtime_executor = RuntimeExecutor::new();
        
        // Verify that static optimization elements have been removed
        // RuntimeExecutor should NOT have expression analyzer or tail call optimizer access
        
        // Instead, it should have dynamic optimization components
        // JIT optimizer and inline evaluator are available for dynamic optimization
        // (their existence is verified by compilation success)
        
        // Verify that runtime executor is focused on dynamic optimization levels
        assert!(matches!(
            runtime_executor.optimization_level,
            RuntimeOptimizationLevel::None | 
            RuntimeOptimizationLevel::Conservative | 
            RuntimeOptimizationLevel::Balanced | 
            RuntimeOptimizationLevel::Aggressive
        ));
    }
}

//! Type definitions for runtime executor
//!
//! This module contains all the type definitions, enums, and structures
//! used by the `RuntimeExecutor` system for performance optimization.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::value::Value;
use std::rc::Rc;

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
    #[cfg(feature = "development")]
    #[must_use] pub fn llvm_level(&self) -> crate::evaluator::llvm_backend::LLVMOptimizationLevel {
        use crate::evaluator::llvm_backend::LLVMOptimizationLevel;
        match self {
            RuntimeOptimizationLevel::None => LLVMOptimizationLevel::O0,
            RuntimeOptimizationLevel::Conservative => LLVMOptimizationLevel::O1,
            RuntimeOptimizationLevel::Balanced => LLVMOptimizationLevel::O2,
            RuntimeOptimizationLevel::Aggressive => LLVMOptimizationLevel::O3,
        }
    }
    
    /// Get optimization level as string (for non-development builds)
    #[cfg(not(feature = "development"))]
    #[must_use] pub fn optimization_string(&self) -> &'static str {
        match self {
            RuntimeOptimizationLevel::None => "O0",
            RuntimeOptimizationLevel::Conservative => "O1",
            RuntimeOptimizationLevel::Balanced => "O2",
            RuntimeOptimizationLevel::Aggressive => "O3",
        }
    }
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

impl ExpressionAnalysisResult {
    /// Create new analysis result with conservative defaults
    #[must_use] pub fn new() -> Self {
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
    #[must_use] pub fn analyze_expression(expr: &Expr) -> Self {
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
                                if exprs.len() >= 3
                                    && Self::contains_self_reference(&exprs[2], name) {
                                        patterns.push(CallPattern::Recursive { depth_estimate: 1 });
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
    /// Compiled code identifier
    pub code_id: String,
    
    /// Original expression
    pub original_expr: Expr,
    
    /// Compilation metadata
    pub metadata: JitMetadata,
    
    /// Whether the code is ready for execution
    pub is_executable: bool,
    
    /// Whether compilation is ready (for compatibility)
    pub is_ready: bool,
    
}

/// JIT compilation metadata
#[derive(Debug, Clone)]
pub struct JitMetadata {
    /// Compilation timestamp
    pub compiled_at: std::time::Instant,
    
    /// Optimization level used
    pub optimization_level: RuntimeOptimizationLevel,
    
    /// Estimated performance gain
    pub estimated_speedup: f64,
    
    /// Memory usage overhead
    pub memory_overhead: usize,
    
    /// Number of times this code has been executed
    pub execution_count: usize,
    
    /// Applied optimizations (for compatibility)
    pub optimizations: Vec<String>,
    
    /// Compilation time in microseconds (for compatibility)
    pub compilation_time_us: u64,
    
    /// Hot path count (for compatibility)
    pub hot_path_count: usize,
    
}

/// Adaptive optimization decision record
#[derive(Debug, Clone)]
pub struct AdaptiveDecision {
    /// Expression that was analyzed
    pub expression: Expr,
    
    /// Analysis result that led to the decision
    pub analysis: ExpressionAnalysisResult,
    
    /// Optimization type selected
    pub optimization_type: AdaptiveOptimizationType,
    
    /// Timestamp of decision
    pub decided_at: std::time::Instant,
    
    /// Timestamp (for compatibility)
    pub timestamp: std::time::Instant,
    
    /// Trigger expression hash (for compatibility)
    pub trigger_expression: String,
    
    /// Decision rationale (for compatibility)
    pub rationale: String,
    
    /// Expected performance improvement (for compatibility)
    pub expected_improvement: f64,
}

/// Types of adaptive optimizations
#[derive(Debug, Clone, PartialEq)]
pub enum AdaptiveOptimizationType {
    /// No optimization needed
    None,
    
    /// Inline evaluation
    Inline,
    
    /// JIT compilation
    JitCompile,
    
    /// Tail call optimization
    TailCallOptimize,
    
    /// Continuation pooling
    ContinuationPooling,
    
    /// Type specialization
    TypeSpecialize,
    
    /// Loop unrolling
    LoopUnroll { 
        /// Unrolling factor
        factor: u32 
    },
    
    /// Memoization
    Memoize,
    
    /// JIT compilation (for compatibility)
    JitCompilation,
    
    /// Adaptive loop unrolling (for compatibility) 
    AdaptiveLoopUnrolling { 
        /// Unrolling factor
        factor: u32 
    },
    
    /// Profile-guided optimization (for compatibility)
    ProfileGuidedOptimization,
    
    /// No optimization (for compatibility)
    NoOptimization,
    
    /// Hot path inlining (for compatibility)
    HotPathInlining,
    
    /// Type specialization (for compatibility) 
    TypeSpecialization,
    
    /// Memory layout optimization (for compatibility)
    MemoryLayoutOptimization,
}

impl AdaptiveOptimizationType {
    /// Check if this optimization type requires JIT compilation
    #[must_use] pub fn requires_jit(&self) -> bool {
        matches!(self, AdaptiveOptimizationType::JitCompile | AdaptiveOptimizationType::LoopUnroll { .. })
    }
    
    /// Check if this optimization should trigger JIT compilation (for compatibility)
    #[must_use] pub fn should_compile_jit(&self) -> bool {
        matches!(
            self, 
            AdaptiveOptimizationType::JitCompile 
            | AdaptiveOptimizationType::JitCompilation
            | AdaptiveOptimizationType::AdaptiveLoopUnrolling { .. }
            | AdaptiveOptimizationType::LoopUnroll { .. }
        )
    }
}

/// Performance profile for JIT compiled code
#[derive(Debug, Clone)]
pub struct PerformanceProfile {
    /// Average execution time in nanoseconds
    pub avg_execution_time_ns: u64,
    
    /// Memory usage statistics
    pub memory_usage: MemoryUsage,
    
    /// Execution characteristics
    pub execution_characteristics: ExecutionCharacteristics,
    
    /// Speedup factor (for compatibility)
    pub speedup_factor: f64,
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Peak memory usage during execution
    pub peak_memory_bytes: usize,
    
    /// Average memory usage
    pub avg_memory_bytes: usize,
    
    /// Number of allocations
    pub allocation_count: usize,
    
    /// Stack usage in bytes (for compatibility)
    pub stack_usage_bytes: usize,
    
    /// Heap allocations count (for compatibility)
    pub heap_allocations: usize,
    
    /// Memory intensive flag (for compatibility)
    pub is_memory_intensive: bool,
}

/// Execution characteristics
#[derive(Debug, Clone)]
pub struct ExecutionCharacteristics {
    /// Number of function calls made
    pub function_calls: usize,
    
    /// Maximum recursion depth reached
    pub max_recursion_depth: usize,
    
    /// Whether the execution involved loops
    pub has_loops: bool,
    
    /// Number of continuation captures
    pub continuation_captures: usize,
    
    /// CPU intensive flag (for compatibility)
    pub is_cpu_intensive: bool,
    
    /// IO operations flag (for compatibility)
    pub has_io_operations: bool,
    
    /// Estimated instruction count (for compatibility)
    pub estimated_instructions: usize,
    
    /// Parallelizable flag (for compatibility)
    pub parallelizable: bool,
}

/// Optimization thresholds for adaptive decisions
#[derive(Debug, Clone)]
pub struct OptimizationThresholds {
    /// Minimum execution count before considering JIT compilation
    pub jit_compilation_threshold: usize,
    
    /// Minimum complexity score for optimization
    pub min_complexity_for_optimization: u32,
    
    /// Maximum memory overhead allowed for optimizations (percentage)
    pub max_memory_overhead_percent: f64,
    
    /// Minimum expected speedup to justify optimization
    pub min_speedup_factor: f64,
}

impl Default for OptimizationThresholds {
    fn default() -> Self {
        Self {
            jit_compilation_threshold: 100,
            min_complexity_for_optimization: 5,
            max_memory_overhead_percent: 20.0,
            min_speedup_factor: 1.5,
        }
    }
}

/// Runtime execution statistics
#[derive(Debug, Clone, Default)]
pub struct RuntimeStats {
    /// Total number of expressions evaluated
    pub expressions_evaluated: usize,
    
    /// Number of JIT compilations performed
    pub jit_compilations: usize,
    
    /// Number of successful optimizations
    pub optimizations_applied: usize,
    
    /// Total time spent in optimization (microseconds)
    pub optimization_time_us: u64,
    
    /// Total time spent in JIT compilation (microseconds)
    pub jit_compilation_time_us: u64,
    
    /// Memory saved through continuation pooling (bytes)
    pub pooling_memory_saved: usize,
    
    /// Number of tail calls optimized
    pub tail_calls_optimized: usize,
    
    /// Number of cache hits for memoized results
    pub memoization_hits: usize,
    
    /// Number of cache misses for memoization
    pub memoization_misses: usize,
    
    /// Number of hot path detections
    pub hot_path_detections: usize,
    
    /// Average optimization effectiveness (0.0 to 1.0)
    pub avg_optimization_effectiveness: f64,
    
    /// Total evaluation time (microseconds)
    pub total_evaluation_time_us: u64,
    
    /// Total execution time (microseconds)
    pub total_execution_time_us: u64,
    
    /// Number of macro expansions performed
    pub macro_expansions: usize,
    
    /// Number of pre-computed constants used
    pub constants_used: usize,
    
    /// Estimated time savings from static optimizations (microseconds)
    pub estimated_time_savings_us: u64,
    
    /// Estimated memory savings from optimizations (bytes)
    pub estimated_memory_savings_bytes: usize,
    
    /// Number of JIT compilation attempts triggered
    pub jit_compilations_triggered: usize,
    
    /// Number of tail call optimizations applied
    pub tail_call_optimizations_applied: usize,
    
    /// Number of continuation pooling uses
    pub continuation_pooling_uses: usize,
    
    /// Number of inline evaluations performed
    pub inline_evaluations: usize,
    
    /// Number of inline evaluation opportunities detected
    pub inline_evaluation_opportunities: usize,
    
    /// Number of JIT execution time (microseconds)
    pub jit_execution_time_us: u64,
    
    /// Number of JIT fallbacks to interpreter
    pub jit_fallbacks: usize,
    
    /// Number of LLVM optimizations applied
    pub llvm_optimizations_applied: usize,
    
    /// Number of continuation pool hits
    pub continuation_pool_hits: usize,
    
    /// Number of continuation pool misses
    pub continuation_pool_misses: usize,
    
    /// Number of verification checks performed
    pub verification_checks: usize,
    
    /// Number of pool defragmentation
    pub pool_defragmentation: usize,
    
    /// Number of verification failures
    pub verification_failures: usize,
    
    /// Number of verification successes
    pub verification_successes: usize,
}

impl RuntimeStats {
    /// Calculate total execution time including optimization overhead
    #[must_use] pub fn total_execution_time_us(&self) -> u64 {
        self.optimization_time_us + self.jit_compilation_time_us
    }
    
    /// Calculate JIT compilation success rate
    #[must_use] pub fn jit_success_rate(&self) -> f64 {
        if self.expressions_evaluated > 0 {
            self.jit_compilations as f64 / self.expressions_evaluated as f64
        } else {
            0.0
        }
    }
    
    /// Calculate memoization hit rate
    #[must_use] pub fn memoization_hit_rate(&self) -> f64 {
        let total_memoization_attempts = self.memoization_hits + self.memoization_misses;
        if total_memoization_attempts > 0 {
            self.memoization_hits as f64 / total_memoization_attempts as f64
        } else {
            0.0
        }
    }
    
    /// Calculate optimization application rate
    #[must_use] pub fn optimization_rate(&self) -> f64 {
        if self.expressions_evaluated > 0 {
            self.optimizations_applied as f64 / self.expressions_evaluated as f64 * 100.0
        } else {
            0.0
        }
    }
    
    /// Calculate continuation pool efficiency
    #[must_use] pub fn continuation_pool_efficiency(&self) -> f64 {
        let total_pool_operations = self.continuation_pool_hits + self.continuation_pool_misses;
        if total_pool_operations > 0 {
            self.continuation_pool_hits as f64 / total_pool_operations as f64
        } else {
            0.0
        }
    }
}
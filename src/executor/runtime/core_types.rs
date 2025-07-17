//! Core Types and Structures for Runtime Executor
//!
//! This module provides the basic structures and type definitions
//! used by the Runtime Executor.
//!
//! This module implements the runtime executor that applies performance optimizations
//! while maintaining correctness through reference to the semantic evaluator.
//!
//! The `RuntimeExecutor` integrates all dynamic optimization systems:
//! - JIT loop optimization
//! - Continuation pooling  
//! - Inline evaluation for hot paths
//! - Runtime performance profiling and adaptation

use crate::evaluator::{
    ContinuationPoolManager, InlineEvaluator,
    JitLoopOptimizer, SemanticEvaluator,
};
#[cfg(feature = "development")]
use crate::evaluator::llvm_backend::LLVMCompilerIntegration;
use crate::executor::runtime_optimization_integration::IntegratedOptimizationManager;
#[cfg(feature = "development")]
use crate::performance_monitor::hotpath_analysis::AdvancedHotPathDetector;

/// Fallback type when development feature is not enabled
#[cfg(not(feature = "development"))]
#[derive(Debug)]
pub struct AdvancedHotPathDetector;

#[cfg(not(feature = "development"))]
impl AdvancedHotPathDetector {
    /// Create new hotpath detector (no-op in production)
    pub fn new() -> Self {
        Self
    }
    
    /// Record execution (no-op in production)
    pub fn record_execution(&mut self, _expr: &crate::ast::Expr, _duration: std::time::Duration, _memory_usage: usize, _return_value: &crate::value::Value, _call_stack: &[String]) -> Result<(), crate::error::LambdustError> {
        Ok(()) // No-op in production
    }
    
    /// Generate performance report (placeholder in production)
    pub fn generate_performance_report(&self) -> String {
        "Hot Path Analysis: No data (production build)".to_string()
    }
}
use rustc_hash::FxHashMap;
use std::collections::HashSet;

/// Trait for providing safe accessor patterns to internal components
/// This implements the "Tell, Don't Ask" principle through higher-order functions
pub trait ComponentAccessor<T> {
    /// Access component with mutable reference
    fn with_component_mut<F>(&mut self, f: F) 
    where F: FnOnce(&mut T);
    
    /// Access component with immutable reference and return value
    fn with_component<F, R>(&self, f: F) -> R
    where F: FnOnce(&T) -> R;
}

/// Specialized trait for hot path detector access
pub trait HotPathDetectorAccess: ComponentAccessor<HotPathDetector> {
    /// Convenience method for hot path detector access
    fn with_hot_path_detector_mut<F>(&mut self, f: F) 
    where F: FnOnce(&mut HotPathDetector) {
        self.with_component_mut(f);
    }
    
    /// Convenience method for hot path detector access
    fn with_hot_path_detector<F, R>(&self, f: F) -> R
    where F: FnOnce(&HotPathDetector) -> R {
        self.with_component(f)
    }
}

// Re-export types from runtime_executor_types for backward compatibility
pub use crate::executor::runtime_executor_types::{
    RuntimeOptimizationLevel, ExpressionAnalysisResult, CallPattern, ExecutionFrequency,
    MemoryPattern, OptimizationHint, OptimizedTailCall, JitCompiledCode, JitMetadata,
    AdaptiveDecision, AdaptiveOptimizationType, PerformanceProfile, MemoryUsage,
    ExecutionCharacteristics, OptimizationThresholds, RuntimeStats,
};

/// Runtime executor with integrated optimization systems
pub struct RuntimeExecutor {
    /// Reference semantic evaluator for correctness verification
    pub(crate) semantic_evaluator: SemanticEvaluator,

    /// JIT loop optimizer (dynamic optimization)
    pub(crate) jit_optimizer: JitLoopOptimizer,

    /// Inline evaluator for hot path optimization
    pub(crate) inline_evaluator: InlineEvaluator,

    /// Continuation pooling manager
    pub(crate) continuation_pooler: ContinuationPoolManager,

    /// Integrated optimization manager
    pub(crate) integrated_optimizer: IntegratedOptimizationManager,
    
    /// Adaptive optimization engine
    pub(crate) adaptive_engine: AdaptiveOptimizationEngine,

    /// Advanced hot path detector with multidimensional analysis
    pub(crate) hotpath_detector: AdvancedHotPathDetector,

    /// LLVM compiler integration for native code generation
    #[cfg(feature = "development")]
    pub(crate) llvm_compiler: LLVMCompilerIntegration,

    /// Current optimization level
    pub(crate) optimization_level: RuntimeOptimizationLevel,

    /// Whether to verify against semantic evaluator
    pub(crate) verification_enabled: bool,

    /// Recursion depth tracking
    pub(crate) recursion_depth: usize,
    pub(crate) max_recursion_depth: usize,
}

/// Adaptive optimization engine for dynamic code generation
#[derive(Debug)]
pub struct AdaptiveOptimizationEngine {
    /// Hot path detector
    pub(crate) hot_path_detector: HotPathDetector,
    
    // TODO: Implement JIT optimization components
    // The following fields were removed as they're currently unused:
    // - code_generator: Dynamic code generation
    // - performance_tracker: Performance metrics collection
    // - decisions: Optimization decision history
    // - thresholds: Optimization threshold configuration
    // - profiler: Profile-guided optimization
    // - jit_cache: JIT compiled code cache
    
    /// Decision history (for compatibility)
    pub(crate) decision_history: Vec<AdaptiveDecision>,
}

/// Hot path detection system
#[derive(Debug)]
pub struct HotPathDetector {
    /// Execution frequency tracking
    execution_frequencies: FxHashMap<String, u64>,
    
    /// Hot path threshold
    hot_threshold: u64,
    
    /// Hot paths set (for compatibility)
    hot_paths: HashSet<String>,
    
    /// Sensitivity threshold (for compatibility)
    sensitivity_threshold: u64,
    
    /// Cool down period (for compatibility)
    cool_down_period: std::time::Duration,
    
    /// Last detection times (for compatibility)
    last_detection: FxHashMap<String, std::time::Instant>,
}

impl HotPathDetector {
    /// Create new hot path detector
    #[must_use] pub fn new() -> Self {
        Self {
            execution_frequencies: FxHashMap::default(),
            hot_threshold: 10,
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
            let now = std::time::Instant::now();
            
            // Check cool-down period
            if let Some(last_time) = self.last_detection.get(expr_hash) {
                if now.duration_since(*last_time) < self.cool_down_period {
                    return; // Still in cool-down
                }
            }
            
            // Add to hot paths
            self.hot_paths.insert(expr_hash.to_string());
            self.last_detection.insert(expr_hash.to_string(), now);
        }
    }
    
    /// Check if expression is on hot path using `hot_threshold`
    #[must_use] pub fn is_hot_path(&self, expr_hash: &str) -> bool {
        // Check both the hot paths set and the threshold-based detection
        if self.hot_paths.contains(expr_hash) {
            return true;
        }
        
        // Also check if frequency exceeds hot_threshold
        if let Some(&frequency) = self.execution_frequencies.get(expr_hash) {
            frequency >= self.hot_threshold
        } else {
            false
        }
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

/// Dynamic code generator for JIT compilation
#[derive(Debug)]
pub struct DynamicCodeGenerator {
    /// Generated code cache
    pub(crate) code_cache: FxHashMap<String, JitCompiledCode>,
    
    /// Code generation statistics
    pub(crate) generation_stats: CodeGenerationStats,
    
    /// Strategy selector (for compatibility)
    pub(crate) strategy_selector: CompilationStrategySelector,
    
    /// Code allocator (for compatibility)
    pub(crate) code_allocator: CodeAllocator,
}

/// Profile-guided optimization system
#[derive(Debug)]
pub struct ProfileGuidedOptimizer {
    /// Performance profiles
    pub(crate) profiles: FxHashMap<String, PerformanceProfile>,
    
    /// Optimization recommendations
    pub(crate) recommendations: Vec<OptimizationHint>,
    
    /// Execution profiles (for compatibility)
    pub(crate) execution_profiles: FxHashMap<String, PerformanceProfile>,
    
    /// Optimization decisions (for compatibility)
    pub(crate) optimization_decisions: FxHashMap<String, AdaptiveOptimizationType>,
    
    /// Collection period (for compatibility)
    pub(crate) collection_period: std::time::Duration,
    
    /// Minimum samples required (for compatibility)
    pub(crate) min_samples: usize,
}

/// Performance tracking system
#[derive(Debug)]
pub struct PerformanceTracker {
    /// Performance metrics history
    pub(crate) metrics_history: Vec<PerformanceProfile>,
    
    /// Current performance baseline
    pub(crate) baseline: Option<PerformanceProfile>,
    
    /// Regression detector (for compatibility)
    pub(crate) regression_detector: RegressionDetector,
    
    /// Performance improvements (for compatibility)
    pub(crate) improvements: Vec<PerformanceProfile>,
}

/// Code generation statistics
#[derive(Debug, Default)]
pub struct CodeGenerationStats {
    /// Total code generations
    pub total_generations: usize,
    
    /// Successful compilations
    pub successful_compilations: usize,
    
    /// Average generation time
    pub avg_generation_time_ms: f64,
}

impl CodeGenerationStats {
    /// Create new code generation statistics
    #[must_use] pub fn new() -> Self {
        Self::default()
    }
}

/// Compilation strategy selector (placeholder)
#[derive(Debug)]
pub struct CompilationStrategySelector;

impl Default for CompilationStrategySelector {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilationStrategySelector {
    /// Create new compilation strategy selector
    #[must_use] pub fn new() -> Self { Self }
}

/// Code allocator (placeholder)
#[derive(Debug)]
pub struct CodeAllocator;

impl Default for CodeAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeAllocator {
    /// Create new code allocator
    #[must_use] pub fn new() -> Self { Self }
}

/// Regression detector (placeholder)
#[derive(Debug)]
pub struct RegressionDetector;

impl Default for RegressionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl RegressionDetector {
    /// Create new regression detector
    #[must_use] pub fn new() -> Self { Self }
    
    /// Detect performance regression (placeholder implementation)
    #[must_use] pub fn detect_regression(&self, _current: f64, _baseline: f64) -> bool {
        false // No regression detected in placeholder
    }
    
    /// Get risk level (placeholder implementation)
    #[must_use] pub fn get_risk_level(&self) -> f64 {
        0.0 // No risk in placeholder
    }
}

/// Performance baseline (placeholder)
#[derive(Debug)]
pub struct PerformanceBaseline;

impl Default for PerformanceBaseline {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceBaseline {
    /// Creates a new performance baseline instance
    #[must_use] pub fn new() -> Self { Self }
}

impl RuntimeExecutor {
    /// Get optimization level
    #[must_use] pub fn optimization_level(&self) -> &RuntimeOptimizationLevel {
        &self.optimization_level
    }
    
    /// Get mutable reference to optimization level
    pub fn optimization_level_mut(&mut self) -> &mut RuntimeOptimizationLevel {
        &mut self.optimization_level
    }
    
    /// Get semantic evaluator reference
    #[must_use] pub fn semantic_evaluator(&self) -> &SemanticEvaluator {
        &self.semantic_evaluator
    }
    
    /// Get mutable reference to semantic evaluator
    pub fn semantic_evaluator_mut(&mut self) -> &mut SemanticEvaluator {
        &mut self.semantic_evaluator
    }
    
    /// Check if verification is enabled
    #[must_use] pub fn verification_enabled(&self) -> bool {
        self.verification_enabled
    }
    
    /// Set verification enabled
    pub fn set_verification_enabled(&mut self, enabled: bool) {
        self.verification_enabled = enabled;
    }
    
    /// Get LLVM compiler reference
    #[cfg(feature = "development")]
    #[must_use] pub fn llvm_compiler(&self) -> &LLVMCompilerIntegration {
        &self.llvm_compiler
    }
    
    /// Get mutable reference to LLVM compiler
    #[cfg(feature = "development")]
    pub fn llvm_compiler_mut(&mut self) -> &mut LLVMCompilerIntegration {
        &mut self.llvm_compiler
    }
    
    /// Get JIT optimizer reference
    #[must_use] pub fn jit_optimizer(&self) -> &JitLoopOptimizer {
        &self.jit_optimizer
    }
    
    /// Get mutable reference to JIT optimizer
    pub fn jit_optimizer_mut(&mut self) -> &mut JitLoopOptimizer {
        &mut self.jit_optimizer
    }
    
    /// Get hotpath detector reference
    #[must_use] pub fn hotpath_detector(&self) -> &AdvancedHotPathDetector {
        &self.hotpath_detector
    }
    
    /// Get mutable reference to hotpath detector
    pub fn hotpath_detector_mut(&mut self) -> &mut AdvancedHotPathDetector {
        &mut self.hotpath_detector
    }
    
    /// Get recursion depth
    #[must_use] pub fn recursion_depth(&self) -> usize {
        self.recursion_depth
    }
    
    /// Get mutable reference to recursion depth
    pub fn recursion_depth_mut(&mut self) -> &mut usize {
        &mut self.recursion_depth
    }
    
    /// Get max recursion depth
    #[must_use] pub fn max_recursion_depth(&self) -> usize {
        self.max_recursion_depth
    }
    
    /// Set max recursion depth
    pub fn set_max_recursion_depth(&mut self, depth: usize) {
        self.max_recursion_depth = depth;
    }
    
    /// Send statistics through environment (decoupled statistics reporting)
    /// Only available in development builds
    #[cfg(feature = "development")]
    pub fn send_statistics(&self, env: &crate::environment::Environment, message: crate::environment::StatisticsMessage) {
        env.send_statistics(message);
    }
    
    /// No-op version for non-development builds
    #[cfg(not(feature = "development"))]
    pub fn send_statistics(&self, _env: &crate::environment::Environment, _message: ()) {
        // No-op: statistics disabled in production builds
    }
    
    /// Get continuation pooler reference
    #[must_use] pub fn continuation_pooler(&self) -> &crate::evaluator::continuation_pooling::ContinuationPoolManager {
        &self.continuation_pooler
    }
    
    /// Get mutable reference to continuation pooler
    pub fn continuation_pooler_mut(&mut self) -> &mut crate::evaluator::continuation_pooling::ContinuationPoolManager {
        &mut self.continuation_pooler
    }
    
    /// Get inline evaluator reference
    #[must_use] pub fn inline_evaluator(&self) -> &crate::evaluator::InlineEvaluator {
        &self.inline_evaluator
    }
    
    /// Get adaptive engine reference
    #[must_use] pub fn adaptive_engine(&self) -> &AdaptiveOptimizationEngine {
        &self.adaptive_engine
    }
    
    /// Get mutable reference to adaptive engine
    pub fn adaptive_engine_mut(&mut self) -> &mut AdaptiveOptimizationEngine {
        &mut self.adaptive_engine
    }
    
    /// Get integrated optimizer reference
    #[must_use] pub fn integrated_optimizer(&self) -> &crate::executor::runtime_optimization_integration::IntegratedOptimizationManager {
        &self.integrated_optimizer
    }
    
    /// Get global environment from semantic evaluator
    #[must_use] pub fn global_environment(&self) -> &std::rc::Rc<crate::environment::Environment> {
        self.semantic_evaluator.global_environment()
    }
    
    /// Get mutable reference to integrated optimizer
    pub fn integrated_optimizer_mut(&mut self) -> &mut crate::executor::runtime_optimization_integration::IntegratedOptimizationManager {
        &mut self.integrated_optimizer
    }

    /// Set optimization level for runtime executor
    pub fn set_optimization_level(&mut self, level: RuntimeOptimizationLevel) {
        self.optimization_level = level;
        
        #[cfg(feature = "development")]
        {
            // Also configure LLVM compiler
            let llvm_level = match level {
                RuntimeOptimizationLevel::None => crate::evaluator::llvm_backend::LLVMOptimizationLevel::O0,
                RuntimeOptimizationLevel::Conservative => crate::evaluator::llvm_backend::LLVMOptimizationLevel::O1,
                RuntimeOptimizationLevel::Balanced => crate::evaluator::llvm_backend::LLVMOptimizationLevel::O2,
                RuntimeOptimizationLevel::Aggressive => crate::evaluator::llvm_backend::LLVMOptimizationLevel::O3,
            };
            self.llvm_compiler.set_optimization_level(llvm_level);
        }
    }
}

// Implement ComponentAccessor trait for HotPathDetector access
impl ComponentAccessor<HotPathDetector> for RuntimeExecutor {
    fn with_component_mut<F>(&mut self, f: F) 
    where F: FnOnce(&mut HotPathDetector) {
        f(&mut self.adaptive_engine.hot_path_detector);
    }
    
    fn with_component<F, R>(&self, f: F) -> R
    where F: FnOnce(&HotPathDetector) -> R {
        f(&self.adaptive_engine.hot_path_detector)
    }
}

// Implement specialized HotPathDetectorAccess trait  
impl HotPathDetectorAccess for RuntimeExecutor {}


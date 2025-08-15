//! Evaluation engine for the Lambdust language.
//!
//! This module provides the complete evaluation system for Lambdust,
//! including proper tail call optimization, lexical scoping, and
//! comprehensive error reporting with stack traces.

pub mod value;
pub mod optimized_value;
pub mod environment;
pub mod cached_environment;
pub mod evaluator;
pub mod unified_eval_errors;
pub mod parameter;
pub mod fast_path;
pub mod optimized_environment;
pub mod monadic_evaluator;
pub mod gc_coordinator;
pub mod continuation_gc;

// Arena allocation modules
pub mod value_arena;
pub mod arena_integration;
pub mod arena_demo;

// Value optimization modules
pub mod value_bridge;
pub mod value_optimization_core;
pub mod memory_measurement;
pub mod semantic_tests;
pub mod optimization_demo;
pub mod simple_performance_test;

// Performance verification modules
pub mod comprehensive_performance_verification;
pub mod arc_allocation_tracker;

// New monadic architecture modules
pub mod operational_semantics;
pub mod continuation_domain;
pub mod monadic_architecture;
pub mod effect_integration;
pub mod evaluator_integration;
pub mod testing_architecture;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod environment_integration_tests;

// Re-export public types and functions
pub use value::{
    Value, Environment, ThreadSafeEnvironment, Generation, StackTrace, StackFrame, FrameType,
    Procedure, PrimitiveProcedure, PrimitiveImpl, Continuation, Frame,
    SyntaxTransformer, Port, PortImpl, PortMode, PortDirection, StandardPort, PortFileHandle, Promise,
    TypeValue, ForeignObject, Parameter,
};
pub use optimized_value::{OptimizedValue, OptimizedEnvironment, OptimizedFrame};
pub use fast_path::{FastPathOp, execute_fast_path, execute_fast_path_optimized, is_fast_path_operation, FastPathStats, get_fast_path_stats};
pub use environment::{EnvironmentBuilder, global_environment};
pub use cached_environment::{CachedEnvironment, CacheStatistics};
pub use evaluator::{Evaluator, EvalStep};
pub use parameter::{ParameterBinding, ParameterFrame};
pub use optimized_environment::{OptimizedEnvironment as OptEnv, OptimizedEnvironmentBuilder, EnvironmentStats};

// Value optimization exports
pub use value_bridge::{
    LegacyValueBridge, BridgeConfig, OptimizationMetrics, SemanticEquivalenceChecker, OptimizedConstructors
};
pub use value_optimization_core::{
    ValueOptimizer, OptimizationConfig, PerformanceStats, MemoryAnalyzer,
    hot_path_optimized
};
pub use memory_measurement::{
    MemoryMeasurer, BenchmarkResult, ComprehensiveBenchmarkReport, ArcAnalyzer, ArcAnalysisResult
};
pub use semantic_tests::{
    SemanticTestSuite, SemanticTestResults, PropertyTestResults, ValueProperties
};
pub use optimization_demo::{run_optimization_demo, example_usage};
pub use simple_performance_test::{SimplePerformanceVerifier, SimpleTestResult, run_simple_verification};

// Performance verification exports
pub use comprehensive_performance_verification::{
    PerformanceVerificationSuite, VerificationConfig, ComprehensiveVerificationReport,
    MemoryAnalysisReport, ArcAnalysisReport, SemanticVerificationReport, CacheAnalysisReport,
    ProductionReadinessReport, PerformanceBenchmarkResult, OperationPerformance
};
pub use arc_allocation_tracker::{
    ArcAllocationTracker, AllocationStats, DetailedAllocationReport, AllocationPatternAnalysis,
    ValueTypeAnalyzer, ValueTypeClassification, HierarchyAnalysis, TrackedArc,
    enable_global_tracking, disable_global_tracking, reset_global_tracking, 
    get_global_stats, get_global_detailed_report, analyze_global_patterns
};
pub use gc_coordinator::{
    GcCoordinator, GcCoordinatorConfig, SessionId, EvaluationSession, GlobalRoot,
    GcCollectionResult, ComprehensiveRootScanResult, EvaluatorGcExt
};
pub use continuation_gc::{
    GcContinuationManager, GcContinuationConfig, ContinuationEntry, EnvironmentCaptureInfo,
    StackTraceManager, PreservedStackTrace, ContinuationStatistics
};

// Arena allocation exports
pub use value_arena::{
    ValueArena, ValueRef, ArenaValue, ArenaConfig, ArenaMemoryStats, CompactionStats
};
pub use arena_integration::{
    ArenaAllocator, ArenaAwareValue, AllocationHint, ValueLifetime, CallFrameRef,
    GlobalArenaStats, arena_utils
};
pub use arena_demo::{ArenaDemo, BenchmarkResult as ArenaBenchmarkResult, MemoryUsage, quick_demo};
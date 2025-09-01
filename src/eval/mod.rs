//! Evaluation engine for the Lambdust language.
//!
//! This module provides the complete evaluation system for Lambdust,
//! including proper tail call optimization, lexical scoping, and
//! comprehensive error reporting with stack traces.

pub mod cached_environment;
pub mod continuation_gc;
pub mod environment;
pub mod evaluator;
pub mod fast_path;
pub mod gc_coordinator;
pub mod monadic_evaluator;
pub mod optimized_environment;
pub mod optimized_value;
pub mod parameter;
pub mod unified_eval_errors;
pub mod value;
pub mod stream;

// SRFI-31 optimization framework
pub mod rec_optimization_framework;
pub mod rec_optimization_monitoring;

// Arena allocation modules
pub mod arena_integration;
pub mod value_arena;

// SRFI-1 optimization modules
pub mod list_optimization;
pub mod list_arena;

// NaN-boxing value representation modules
pub mod nan_boxed_value;

// High-performance SRFI-9 record system modules
pub mod record_access;
pub mod record_arena;
pub mod record_instance;
pub mod record_type;

// Value optimization modules
pub mod memory_measurement;
pub mod semantic_tests;
pub mod value_bridge;
pub mod value_optimization_core;

// Phase 8 optimization infrastructure
pub mod unified_value_system;
pub mod string_interning_system;

// Performance verification modules
pub mod arc_allocation_tracker;

// New monadic architecture modules
pub mod continuation_domain;
pub mod effect_integration;
pub mod evaluator_integration;
pub mod monadic_architecture;
pub mod operational_semantics;

#[cfg(test)]
mod tests;

#[cfg(test)]
// pub mod letrec_test;
// Re-export public types and functions
pub use cached_environment::{CacheStatistics, CachedEnvironment};
pub use environment::{EnvironmentBuilder, global_environment};
pub use evaluator::{EvalStep, Evaluator};
pub use fast_path::{
    FastPathOp, FastPathStats, execute_fast_path, execute_fast_path_optimized, get_fast_path_stats,
    is_fast_path_operation,
};
pub use optimized_environment::{
    EnvironmentStats, OptimizedEnvironment as OptEnv, OptimizedEnvironmentBuilder,
};
pub use optimized_value::{OptimizedEnvironment, OptimizedFrame, OptimizedValue};
pub use parameter::{ParameterBinding, ParameterFrame};
pub use value::{
    Continuation, Environment, ForeignObject, Frame, FrameType, Generation, Parameter, Port,
    PortDirection, PortFileHandle, PortImpl, PortMode, PrimitiveImpl, PrimitiveProcedure,
    Procedure, Promise, StackFrame, StackTrace, StandardPort, SyntaxTransformer,
    ThreadSafeEnvironment, TypeValue, Value,
};

// Value optimization exports
pub use memory_measurement::{
    ArcAnalysisResult, ArcAnalyzer, BenchmarkResult, ComprehensiveBenchmarkReport, MemoryMeasurer,
};
pub use semantic_tests::{
    PropertyTestResults, SemanticTestResults, SemanticTestSuite, ValueProperties,
};
pub use value_bridge::{
    BridgeConfig, LegacyValueBridge, OptimizationMetrics, OptimizedConstructors,
    SemanticEquivalenceChecker,
};
pub use value_optimization_core::{
    MemoryAnalyzer, OptimizationConfig, PerformanceStats, ValueOptimizer, hot_path_optimized,
};

pub use arc_allocation_tracker::{
    AllocationPatternAnalysis, AllocationStats, ArcAllocationTracker, DetailedAllocationReport,
    HierarchyAnalysis, TrackedArc, ValueTypeAnalyzer, ValueTypeClassification,
    analyze_global_patterns, disable_global_tracking, enable_global_tracking,
    get_global_detailed_report, get_global_stats, reset_global_tracking,
};
pub use continuation_gc::{
    ContinuationEntry, ContinuationStatistics, EnvironmentCaptureInfo, GcContinuationConfig,
    GcContinuationManager, PreservedStackTrace, StackTraceManager,
};
pub use gc_coordinator::{
    ComprehensiveRootScanResult, EvaluationSession, EvaluatorGcExt, GcCollectionResult,
    GcCoordinator, GcCoordinatorConfig, GlobalRoot, SessionId,
};

// Arena allocation exports
pub use arena_integration::{
    AllocationHint, ArenaAllocator, ArenaAwareValue, CallFrameRef, GlobalArenaStats, ValueLifetime,
    arena_utils,
};
pub use value_arena::{
    ArenaConfig, ArenaMemoryStats, ArenaValue, CompactionStats, ValueArena, ValueRef,
};

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
pub mod parameter;
pub mod fast_path;
pub mod optimized_environment;
pub mod monadic_evaluator;
pub mod gc_coordinator;
pub mod continuation_gc;

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
pub use gc_coordinator::{
    GcCoordinator, GcCoordinatorConfig, SessionId, EvaluationSession, GlobalRoot,
    GcCollectionResult, ComprehensiveRootScanResult, EvaluatorGcExt
};
pub use continuation_gc::{
    GcContinuationManager, GcContinuationConfig, ContinuationEntry, EnvironmentCaptureInfo,
    StackTraceManager, PreservedStackTrace, ContinuationStatistics
};
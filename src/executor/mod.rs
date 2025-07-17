//! Runtime execution engine for Lambdust Scheme interpreter
//!
//! This module provides the runtime execution system, including:
//! - High-performance runtime executor
//! - Runtime optimization strategies
//! - Performance monitoring integration

pub mod runtime;
pub mod runtime_optimization;
pub mod runtime_optimization_integration;

// Re-export types from runtime_executor_types
pub mod runtime_executor_types;
pub use runtime_executor_types::*;

// Re-export main runtime components
pub use runtime::*;
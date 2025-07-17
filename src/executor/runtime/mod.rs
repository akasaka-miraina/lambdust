//! Runtime Executor Module
//!
//! This module provides the implementation of the Runtime Executor that performs optimized evaluation.
//! It integrates dynamic optimization systems while maintaining correctness through
//! references to the `SemanticEvaluator`.
//!
//! ## Module Structure
//!
//! - `core_types`: Basic structures and type definitions
//! - `runtime_implementation`: Main implementation of `RuntimeExecutor`
//! - `extended_implementation`: Extended implementation and default implementation
//! - `performance_reporting`: Performance reporting and statistics

pub mod core_types;
pub mod runtime_implementation;
pub mod extended_implementation;
pub mod performance_reporting;

// Re-export all public types for backward compatibility
pub use core_types::*;
pub use extended_implementation::*;
pub use performance_reporting::*;
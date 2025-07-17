//! Pure R7RS semantic evaluator
//!
//! This module implements a pure R7RS formal semantics evaluator that
//! contains NO optimizations whatsoever. It serves as the reference
//! implementation for verification against optimized execution paths.

// Core functionality and data structures
mod semantic_core;
mod semantic_continuation;
mod semantic_special_forms;
mod semantic_builtins;
mod semantic_reduction;

// Re-export main types
pub use semantic_core::{SemanticEvaluator, ReductionStats};

// Implementation modules are included via separate files
// to maintain clean separation of concerns and file size limits

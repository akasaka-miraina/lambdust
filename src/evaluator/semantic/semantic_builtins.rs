//! Built-in functions for pure semantic evaluator
//!
//! This module previously contained duplicate builtin function implementations.
//! SemanticEvaluator now uses the standard Environment builtin system from
//! the `builtins` module for consistency and to avoid duplication.
//!
//! All builtin functions are now provided through `Environment::with_builtins()`
//! and called directly via the function pointer stored in `Procedure::Builtin.func`.

// This file intentionally contains no implementations.
// SemanticEvaluator now uses the unified builtin system.
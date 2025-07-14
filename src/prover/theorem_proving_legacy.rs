//! Theorem proving support system for formal verification
//!
//! This module provides infrastructure for formal verification of combinator
//! reductions, R7RS semantic correctness, and mathematical properties of
//! the evaluator system.
//!
//! **Note**: このファイルは後方互換性のために保持されています。
//! 新しいコードでは `theorem_proving` モジュールを直接使用してください。

// Re-export everything from the modularized theorem_proving system
pub use crate::evaluator::theorem_proving::*;
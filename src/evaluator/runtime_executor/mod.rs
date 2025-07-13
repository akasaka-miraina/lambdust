//! Runtime Executor Module
//!
//! このモジュールは最適化された評価を行うRuntime Executorの実装を提供します。
//! 動的最適化システムを統合しながら、SemanticEvaluatorとの参照を通じて
//! 正確性を維持します。
//!
//! ## モジュール構成
//!
//! - `core_types`: 基本構造体と型定義
//! - `runtime_implementation`: RuntimeExecutorのメイン実装
//! - `extended_implementation`: 拡張実装とデフォルト実装
//! - `performance_reporting`: パフォーマンス報告と統計

pub mod core_types;
pub mod runtime_implementation;
pub mod extended_implementation;
pub mod performance_reporting;

// Re-export all public types for backward compatibility
pub use core_types::*;
pub use extended_implementation::*;
pub use performance_reporting::*;
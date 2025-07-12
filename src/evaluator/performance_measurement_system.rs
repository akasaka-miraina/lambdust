//! パフォーマンス測定システム（モジュール化版）
//!
//! この巨大ファイルは保守性向上のため、複数のモジュールに分割されました。
//! 新しいモジュール構造については `performance_measurement`/ ディレクトリを参照してください。

// Re-export the modular performance measurement system
pub use super::performance_measurement::*;

// Backwards compatibility - deprecated but maintained for transition
/// Legacy alias for `PerformanceMeasurementSystem`
/// 
/// This type alias is deprecated and maintained only for backwards compatibility.
/// Use the modular structure in the `performance_measurement` module instead.
#[deprecated(note = "Use the modular structure in performance_measurement module instead")]
pub type LegacyPerformanceMeasurementSystem = PerformanceMeasurementSystem;

/// Legacy alias for `MetricsManager`
/// 
/// This type alias is deprecated and maintained only for backwards compatibility.
/// Use `MetricsManager` from `performance_measurement::metrics` instead.
#[deprecated(note = "Use MetricsManager from performance_measurement::metrics instead")]
pub type LegacyMetricsManager = crate::evaluator::performance_measurement::MetricsManager;

/// Legacy alias for `BenchmarkSuite`
/// 
/// This type alias is deprecated and maintained only for backwards compatibility.
/// Use `BenchmarkSuite` from `performance_measurement::benchmarking` instead.
#[deprecated(note = "Use BenchmarkSuite from performance_measurement::benchmarking instead")]
pub type LegacyBenchmarkSuite = crate::evaluator::performance_measurement::BenchmarkSuite;

/// Legacy alias for `AnalysisEngine`
/// 
/// This type alias is deprecated and maintained only for backwards compatibility.
/// Use `AnalysisEngine` from `performance_measurement::analysis` instead.
#[deprecated(note = "Use AnalysisEngine from performance_measurement::analysis instead")]
pub type LegacyAnalysisEngine = crate::evaluator::performance_measurement::AnalysisEngine;

// Module documentation for migration guidance
/// # Migration Guide
/// 
/// The performance measurement system has been modularized for better maintainability.
/// Please update your imports as follows:
/// 
/// ## Before:
/// ```ignore
/// use crate::evaluator::performance_measurement_system::{
///     PerformanceMeasurementSystem, MetricsManager, BenchmarkSuite
/// };
/// ```
/// 
/// ## After:
/// ```ignore
/// use crate::evaluator::performance_measurement::{
///     PerformanceMeasurementSystem, MetricsManager, BenchmarkSuite
/// };
/// ```
/// 
/// ## New Module Structure:
/// - `core_types`: Basic data structures and types
/// - `configuration`: Configuration and settings
/// - `metrics`: Metrics collection and management
/// - `benchmarking`: Benchmark execution and management
/// - `analysis`: Data analysis and optimization verification
/// 
/// ## Benefits of Modularization:
/// - Better code organization and separation of concerns
/// - Improved maintainability (files now under 1000 lines)
/// - Easier testing and debugging
/// - Reduced compilation times for partial changes
/// - Clearer dependencies and interfaces
pub mod migration_guide {}
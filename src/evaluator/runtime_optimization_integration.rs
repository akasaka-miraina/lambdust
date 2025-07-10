//! Runtime最適化統合システム（モジュール化版）
//!
//! この巨大ファイルは保守性向上のため、複数のモジュールに分割されました。
//! 新しいモジュール構造については runtime_optimization/ ディレクトリを参照してください。

// Re-export the modular runtime optimization system
pub use super::runtime_optimization::*;

// Backwards compatibility - deprecated but maintained for transition
#[deprecated(note = "Use the modular structure in runtime_optimization module instead")]
pub type LegacyIntegratedOptimizationManager = IntegratedOptimizationManager;

#[deprecated(note = "Use OptimizationStrategySelector from runtime_optimization::optimization_manager instead")]
pub type LegacyOptimizationStrategySelector = OptimizationStrategySelector;

#[deprecated(note = "Use OptimizationPerformanceMonitor from runtime_optimization::performance_monitoring instead")]
pub type LegacyOptimizationPerformanceMonitor = OptimizationPerformanceMonitor;

#[deprecated(note = "Use OptimizationCache from runtime_optimization::caching_and_dependencies instead")]
pub type LegacyOptimizationCache = OptimizationCache;

#[deprecated(note = "Use CorrectnessGuarantor from runtime_optimization module instead")]
pub type LegacyCorrectnessGuarantor = CorrectnessGuarantor;

// Module documentation for migration guidance
/// # Migration Guide
/// 
/// The runtime optimization integration system has been modularized for better maintainability.
/// Please update your imports as follows:
/// 
/// ## Before:
/// ```ignore
/// use crate::evaluator::runtime_optimization_integration::{
///     IntegratedOptimizationManager, OptimizationStrategy, OptimizationResult
/// };
/// ```
/// 
/// ## After:
/// ```ignore
/// use crate::evaluator::runtime_optimization::{
///     IntegratedOptimizationManager, OptimizationStrategy, OptimizationResult
/// };
/// ```
/// 
/// ## New Module Structure:
/// - `core_types`: Basic data structures and optimization strategy definitions
/// - `optimization_manager`: Main optimization management and execution system
/// - `performance_monitoring`: Performance monitoring and anomaly detection
/// - `caching_and_dependencies`: Caching mechanisms and dependency management
/// 
/// ## Benefits of Modularization:
/// - **Better maintainability**: Files now under 1000 lines (was 2095 lines)
/// - **Improved separation of concerns**: Each module has a focused responsibility
/// - **Enhanced testability**: Smaller, focused modules are easier to test
/// - **Faster compilation**: Reduced compilation times for partial changes
/// - **Clearer architecture**: Well-defined interfaces between components
/// - **Easier onboarding**: New developers can understand focused modules more easily
/// 
/// ## Module Responsibilities:
/// 
/// ### `core_types`
/// - Optimization strategy definitions and types
/// - Core data structures (ApplicabilityCondition, OptimizationImpact, etc.)
/// - Strategy type enumerations and parameter definitions
/// - Dynamic adjustment systems
/// 
/// ### `optimization_manager`
/// - IntegratedOptimizationManager (main orchestrator)
/// - OptimizationStrategySelector (strategy selection logic)
/// - OptimizationResult and performance improvement tracking
/// - Strategy execution and coordination
/// 
/// ### `performance_monitoring`
/// - Real-time performance monitoring and statistics
/// - Anomaly detection and alerting systems
/// - Performance analysis and reporting
/// - Resource usage tracking and optimization
/// 
/// ### `caching_and_dependencies`
/// - Optimization result caching with multiple strategies (LRU, TTL, etc.)
/// - Dependency graph management and resolution
/// - Conflict detection and resolution
/// - Execution planning and scheduling
/// 
/// This modular architecture significantly improves code quality by:
/// 1. **Eliminating the 1000+ line file violation** (2095 → max 718 lines per module)
/// 2. **Reducing complexity** through focused, single-responsibility modules
/// 3. **Improving maintainability** with clear module boundaries
/// 4. **Enhancing testability** with isolated, focused components
pub mod migration_guide {}
//! Program analysis tools for static analysis and optimization.
//!
//! This module provides comprehensive program analysis capabilities including
//! static analysis, dependency analysis, profiling, and code quality metrics.
//!
//! Most structures have been moved to specialized modules for better organization:
//! - Core analysis framework: `analysis_framework.rs`
//! - Type analysis: `type_analysis.rs`
//! - Dependency analysis: `dependency_analysis.rs`
//! - Variable and scope analysis: `variable_scope_analysis.rs`
//! - Control flow analysis: `control_flow_analysis.rs`
//! - Quality metrics: `quality_metrics.rs`
//! - Warning system: `warning_system.rs`
//! - Profiling analysis: `profiling_analysis.rs`
//! - Common types and enums: `analysis_types.rs`
//!
//! This module now serves as a coordination point and maintains compatibility.

// Re-export all analysis components for backward compatibility
pub use super::analysis_framework::{AnalysisConfig, AnalysisResult, CodeAnalyzer, StaticAnalyzer};
pub use super::analysis_types::*;
pub use super::control_flow_analysis::{BasicBlock, ControlFlowGraph};
pub use super::dependency_analysis::{
    DependencyAnalyzer, DependencyEdge, DependencyGraph, DependencyNode,
};
pub use super::profiling_analysis::{AllocationInfo, HotSpot, Profiler, ProfilingInfo};
pub use super::quality_metrics::{DuplicationInfo, OptimizationOpportunity, QualityMetrics};
pub use super::type_analysis::{FunctionSignature, TypeConstraint, TypeError, TypeInformation};
pub use super::variable_scope_analysis::{ScopeInfo, VariableInfo, VariableUsage};
pub use super::warning_system::AnalysisWarning;

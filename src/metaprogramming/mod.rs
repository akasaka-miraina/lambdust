//! Advanced metaprogramming system for R7RS-large compliance.
//!
//! This module provides high-level metaprogramming capabilities including:
//! - Runtime reflection and introspection
//! - Dynamic code generation and compilation
//! - Dynamic evaluation with sandboxing
//! - Advanced macro system with procedural macros
//! - Program analysis and optimization tools
//! - Runtime environment manipulation

pub mod advanced_macros;
pub mod analysis_framework;
pub mod analysis_types;
pub mod code_generation;
pub mod control_flow_analysis;
pub mod dependency_analysis;
pub mod dynamic_evaluation;
pub mod environment_manipulation;
pub mod profiling_analysis;
pub mod program_analysis;
pub mod quality_metrics;
pub mod reflection;
pub mod security;
pub mod type_analysis;
pub mod variable_scope_analysis;
pub mod warning_system;

#[cfg(test)]
mod tests;

// Individual structure modules
pub mod environment_hierarchy;
pub mod environment_management;
pub mod environment_tracking;
pub mod gc_policy;
pub mod memory_management;
pub mod memory_pressure;
pub mod metaprogramming_system;
pub mod module_management;

// Re-export individual structures
pub use environment_hierarchy::*;
pub use environment_management::*;
pub use environment_tracking::*;
pub use gc_policy::*;
pub use memory_management::*;
pub use memory_pressure::*;
pub use metaprogramming_system::*;
pub use module_management::*;

// Re-export primary interfaces
pub use advanced_macros::{HygienicExtension, MacroDebugger, ProceduralMacro};
pub use analysis_framework::{CodeAnalyzer, StaticAnalyzer};
pub use code_generation::{AstTransformer, CodeGenerator, DynamicDefinition, TemplateSystem};
pub use dependency_analysis::DependencyAnalyzer;
pub use dynamic_evaluation::{
    DynamicEvaluator, ExecutionContext, SandboxEnvironment, SecurityPolicy,
};
pub use profiling_analysis::Profiler;
pub use reflection::{MetadataAccess, ObjectInspector, ReflectionSystem, TypeInspector};
// Note: environment_manipulation structures are now re-exported from individual modules
pub use analysis_types::*;
pub use security::{AccessControl, PermissionSystem, SecurityManager};

//! Advanced metaprogramming system for R7RS-large compliance.
//!
//! This module provides high-level metaprogramming capabilities including:
//! - Runtime reflection and introspection
//! - Dynamic code generation and compilation
//! - Dynamic evaluation with sandboxing
//! - Advanced macro system with procedural macros
//! - Program analysis and optimization tools
//! - Runtime environment manipulation

pub mod reflection;
pub mod code_generation;
pub mod dynamic_evaluation;
pub mod advanced_macros;
pub mod program_analysis;
pub mod environment_manipulation;
pub mod security;
pub mod analysis_types;
pub mod profiling_analysis;
pub mod warning_system;
pub mod variable_scope_analysis;
pub mod control_flow_analysis;
pub mod quality_metrics;
pub mod type_analysis;
pub mod dependency_analysis;
pub mod analysis_framework;

#[cfg(test)]
mod tests;

// Individual structure modules
pub mod metaprogramming_system;
pub mod environment_management;
pub mod environment_hierarchy;
pub mod environment_tracking;
pub mod module_management;
pub mod memory_management;
pub mod gc_policy;
pub mod memory_pressure;

// Re-export individual structures
pub use metaprogramming_system::*;
pub use environment_management::*;
pub use environment_hierarchy::*;
pub use environment_tracking::*;
pub use module_management::*;
pub use memory_management::*;
pub use gc_policy::*;
pub use memory_pressure::*;

// Re-export primary interfaces
pub use reflection::{ReflectionSystem, ObjectInspector, TypeInspector, MetadataAccess};
pub use code_generation::{CodeGenerator, AstTransformer, TemplateSystem, DynamicDefinition};
pub use dynamic_evaluation::{DynamicEvaluator, SandboxEnvironment, SecurityPolicy, ExecutionContext};
pub use advanced_macros::{ProceduralMacro, MacroDebugger, HygienicExtension};
pub use profiling_analysis::Profiler;
pub use dependency_analysis::DependencyAnalyzer;
pub use analysis_framework::{StaticAnalyzer, CodeAnalyzer};
// Note: environment_manipulation structures are now re-exported from individual modules
pub use security::{SecurityManager, PermissionSystem, AccessControl};
pub use analysis_types::*;


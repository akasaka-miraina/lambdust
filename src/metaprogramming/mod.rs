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

#[cfg(test)]
mod tests;

// Individual structure modules
pub mod metaprogramming_system;

// Re-export individual structures
pub use metaprogramming_system::*;

// Re-export primary interfaces
pub use reflection::{ReflectionSystem, ObjectInspector, TypeInspector, MetadataAccess};
pub use code_generation::{CodeGenerator, AstTransformer, TemplateSystem, DynamicDefinition};
pub use dynamic_evaluation::{DynamicEvaluator, SandboxEnvironment, SecurityPolicy, ExecutionContext};
pub use advanced_macros::{ProceduralMacro, MacroDebugger, HygienicExtension};
pub use program_analysis::{StaticAnalyzer, DependencyAnalyzer, Profiler, CodeAnalyzer};
pub use environment_manipulation::{EnvironmentManipulator, ModuleManager, MemoryManager};
pub use security::{SecurityManager, PermissionSystem, AccessControl};


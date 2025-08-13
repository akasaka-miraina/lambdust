//! Module system implementation for Lambdust.
//!
//! This module provides a complete R7RS-compatible module system with support for:
//! - Lambdust built-in modules using (lambdust module-name) syntax
//! - R7RS standard library modules using (scheme module-name) syntax
//! - User-defined modules with proper import/export semantics
//! - Efficient module caching and dependency resolution
//! - Namespace management to prevent symbol collisions

pub mod name;
pub mod loader;
pub mod definition;
pub mod import;
pub mod export;
pub mod resolver;
pub mod cache;
pub mod scheme_loader;

// Individual structure modules
/// Module identification and namespace management
pub mod module_id;
/// Module definitions, sources, and compilation units
pub mod module;
/// Module metadata including dependencies and exports
pub mod module_metadata;
/// Import specifications for module dependency resolution
pub mod import_spec;
/// Export specifications for module interface definition
pub mod export_spec;
/// Core module system managing loading, resolution, and caching
pub mod module_system;

use crate::diagnostics::Result;

// Re-export individual structures
pub use module_id::*;
pub use module::*;
pub use module_metadata::*;
pub use import_spec::*;
pub use export_spec::*;
pub use module_system::*;

/// Trait for objects that can provide module definitions.
pub trait ModuleProvider: Send + Sync {
    /// Gets a module definition by ID.
    fn get_module(&self, id: &ModuleId) -> Result<Module>;
    
    /// Checks if a module exists.
    fn has_module(&self, id: &ModuleId) -> bool;
    
    /// Lists available modules.
    fn list_modules(&self) -> Vec<ModuleId>;
}

/// Error types specific to the module system.
#[derive(Debug, Clone)]
pub enum ModuleError {
    /// Module not found
    NotFound(ModuleId),
    /// Circular dependency detected
    CircularDependency(Vec<ModuleId>),
    /// Import/export conflict
    ImportConflict(String),
    /// Invalid module definition
    InvalidDefinition(String),
    /// Import error
    ImportError(String),
    /// Export error
    ExportError(String),
    /// Module compilation error
    CompilationError(String),
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleError::NotFound(id) => write!(f, "Module not found: {}", format_module_id(id)),
            ModuleError::CircularDependency(cycle) => {
                write!(f, "Circular dependency detected: {}", 
                       cycle.iter().map(format_module_id).collect::<Vec<_>>().join(" -> "))
            }
            ModuleError::ImportConflict(symbol) => write!(f, "Import conflict for symbol: {symbol}"),
            ModuleError::InvalidDefinition(msg) => write!(f, "Invalid module definition: {msg}"),
            ModuleError::ImportError(msg) => write!(f, "Import error: {msg}"),
            ModuleError::ExportError(msg) => write!(f, "Export error: {msg}"),
            ModuleError::CompilationError(msg) => write!(f, "Module compilation error: {msg}"),
        }
    }
}

impl std::error::Error for ModuleError {}

impl ModuleError {
    /// Converts this ModuleError into a Box<ModuleError> for use with Result types.
    pub fn boxed(self) -> Box<ModuleError> {
        Box::new(self)
    }
}

impl From<ModuleError> for crate::diagnostics::Error {
    fn from(err: ModuleError) -> Self {
        match err {
            ModuleError::NotFound(id) => {
                crate::diagnostics::Error::runtime_error(
                    format!("Module not found: {}", format_module_id(&id)),
                    None,
                )
            }
            ModuleError::InvalidDefinition(msg) => {
                crate::diagnostics::Error::parse_error(msg, crate::diagnostics::Span::new(0, 0))
            }
            ModuleError::ImportConflict(symbol) => {
                crate::diagnostics::Error::runtime_error(
                    format!("Import conflict for symbol: {symbol}"),
                    None,
                )
            }
            ModuleError::CircularDependency(cycle) => {
                let cycle_str = cycle.iter()
                    .map(format_module_id)
                    .collect::<Vec<_>>()
                    .join(" -> ");
                crate::diagnostics::Error::runtime_error(
                    format!("Circular dependency detected: {cycle_str}"),
                    None,
                )
            }
            ModuleError::ImportError(msg) => {
                crate::diagnostics::Error::runtime_error(msg, None)
            }
            ModuleError::ExportError(msg) => {
                crate::diagnostics::Error::runtime_error(msg, None)
            }
            ModuleError::CompilationError(msg) => {
                crate::diagnostics::Error::runtime_error(msg, None)
            }
        }
    }
}

// format_module_id is now available through the module_id re-export

/// Parses a module identifier from its string representation.
pub fn parse_module_id(s: &str) -> Result<ModuleId> {
    name::parse_module_name(s)
}

// Re-export key types from scheme_loader for convenience
pub use scheme_loader::{
    SchemeLibraryLoader, CompiledSchemeLibrary, SchemeLibraryCache, 
    BootstrapConfig, CompilationContext, HotReloadManager, CacheStatistics
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_id_creation() {
        let id = ModuleId {
            components: vec!["string".to_string()],
            namespace: ModuleNamespace::Builtin,
        };
        
        assert_eq!(format_module_id(&id), "(lambdust string)");
    }

    #[test]
    fn test_module_system_creation() {
        let result = ModuleSystem::new();
        assert!(result.is_ok());
    }
}
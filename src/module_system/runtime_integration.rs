//! Runtime integration for library system.
//!
//! This module provides the domain-driven implementation of library instantiation,
//! binding resolution, and runtime integration following clean architecture principles.
//! It serves as the bridge between the parsing/loading phases and the runtime execution phase.

use super::{Module, ModuleId, ModuleError};
use crate::ast::{Expr, Spanned};
use crate::diagnostics::{Error, Result, Span};
use crate::eval::{Value, Environment, ThreadSafeEnvironment};
use crate::runtime::GlobalEnvironmentManager;
use crate::metaprogramming::environment_management::EnvironmentExt;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::rc::Rc;

/// Library instantiation context providing clean separation of concerns.
#[derive(Debug)]
pub struct LibraryInstantiationContext {
    /// Global environment manager for primitive access
    global_env: Arc<GlobalEnvironmentManager>,
    /// Current instantiation depth (for circular dependency tracking)
    instantiation_depth: usize,
    /// Stack of currently instantiating libraries (for cycle detection)
    instantiation_stack: Vec<ModuleId>,
    /// Cache of instantiated libraries
    instantiated_libraries: HashMap<ModuleId, Arc<LibraryInstance>>,
    /// Binding resolution cache
    binding_cache: HashMap<String, ExportResolution>,
}

/// Represents a fully instantiated library ready for use.
#[derive(Debug)]
pub struct LibraryInstance {
    /// The module metadata
    pub module: Module,
    /// The library's private environment
    pub environment: Arc<ThreadSafeEnvironment>,
    /// Resolved export bindings
    pub exports: HashMap<String, LibraryBinding>,
    /// Library initialization timestamp
    pub instantiated_at: std::time::SystemTime,
    /// Hot-reload generation
    pub reload_generation: u64,
}

/// A binding exported from a library.
#[derive(Debug, Clone)]
pub struct LibraryBinding {
    /// The binding name
    pub name: String,
    /// The bound value
    pub value: Value,
    /// Whether this binding is mutable
    pub mutable: bool,
    /// Source span for error reporting
    pub source_span: Option<Span>,
}

/// Result of export resolution.
#[derive(Debug, Clone)]
pub enum ExportResolution {
    /// Direct binding export
    Direct(LibraryBinding),
    /// Renamed binding export
    Renamed { 
        /// Original name in the source library
        original: String, 
        /// Library binding being exported
        binding: LibraryBinding 
    },
    /// Re-export from another library
    ReExport { 
        /// Source library providing the binding
        source_library: ModuleId, 
        /// Name of the binding to re-export
        binding_name: String 
    },
    /// Conditional export (with guard)
    Conditional { 
        /// Condition expression for the export
        condition: String, 
        /// Library binding being conditionally exported
        binding: LibraryBinding 
    },
}

/// Domain-specific errors for library instantiation.
#[derive(Debug, Clone)]
pub enum InstantiationError {
    /// Circular dependency during instantiation
    CircularDependency(Vec<ModuleId>),
    /// Missing required dependency
    MissingDependency(ModuleId),
    /// Export binding not found
    ExportNotFound { 
        /// Library where export was not found
        library: ModuleId, 
        /// Name of the missing binding
        binding: String 
    },
    /// Import conflict between libraries
    ImportConflict { 
        /// Name of the conflicting binding
        binding: String, 
        /// Libraries that provide the conflicting binding
        libraries: Vec<ModuleId> 
    },
    /// Invalid export specification
    InvalidExportSpec(String),
    /// Runtime evaluation error during instantiation
    EvaluationError(String),
    /// Library initialization failed
    InitializationFailed(String),
}

/// Core library instantiator implementing the domain logic.
#[derive(Debug)]
pub struct LibraryInstantiator {
    /// Instantiation context
    context: LibraryInstantiationContext,
    /// Maximum instantiation depth to prevent infinite recursion
    max_depth: usize,
    /// Whether to enable verbose logging
    verbose_logging: bool,
}

impl LibraryInstantiator {
    /// Creates a new library instantiator.
    pub fn new(global_env: Arc<GlobalEnvironmentManager>) -> Self {
        Self {
            context: LibraryInstantiationContext {
                global_env,
                instantiation_depth: 0,
                instantiation_stack: Vec::new(),
                instantiated_libraries: HashMap::new(),
                binding_cache: HashMap::new(),
            },
            max_depth: 100, // Reasonable default for nested instantiation
            verbose_logging: false,
        }
    }

    /// Instantiates a library with its dependencies.
    pub fn instantiate_library(&mut self, module: Module) -> Result<Arc<LibraryInstance>> {
        let module_id = module.id.clone();
        
        // Check if already instantiated
        if let Some(instance) = self.context.instantiated_libraries.get(&module_id) {
            return Ok(instance.clone());
        }

        // Check instantiation depth
        if self.context.instantiation_depth >= self.max_depth {
            return Err(Box::new(Error::from(ModuleError::InstantiationError(
                format!("Maximum instantiation depth exceeded: {}", self.max_depth)
            ))));
        }

        // Check for circular dependencies
        if self.context.instantiation_stack.contains(&module_id) {
            let cycle = self.build_dependency_cycle(&module_id);
            return Err(Box::new(Error::from(ModuleError::CircularDependency(cycle))));
        }

        // Begin instantiation
        self.context.instantiation_depth += 1;
        self.context.instantiation_stack.push(module_id.clone());

        if self.verbose_logging {
            eprintln!("Instantiating library: {}", super::format_module_id(&module_id));
        }

        let result = self.instantiate_library_impl(module);

        // Cleanup
        self.context.instantiation_stack.pop();
        self.context.instantiation_depth -= 1;

        result
    }

    /// Core implementation of library instantiation.
    fn instantiate_library_impl(&mut self, module: Module) -> Result<Arc<LibraryInstance>> {
        let module_id = module.id.clone();

        // Create library environment extending the global environment
        let base_env = self.context.global_env.root_environment();
        let lib_env = base_env.extend(0); // Use generation 0 for libraries

        // Process imports first to resolve dependencies
        let import_bindings = self.resolve_library_imports(&module)?;
        
        // Install imported bindings
        for (name, binding) in import_bindings {
            lib_env.define(name, binding.value);
        }

        // Evaluate library body to establish internal definitions
        self.evaluate_library_body(&module, lib_env.clone())?;

        // Resolve and prepare exports
        let exports = self.resolve_library_exports(&module, lib_env.clone())?;

        // Create the library instance
        let instance = Arc::new(LibraryInstance {
            module,
            environment: lib_env,
            exports,
            instantiated_at: std::time::SystemTime::now(),
            reload_generation: 0, // TODO: Integrate with hot-reload system
        });

        // Cache the instance
        self.context.instantiated_libraries.insert(module_id, instance.clone());

        Ok(instance)
    }

    /// Resolves import specifications for a library.
    fn resolve_library_imports(&mut self, module: &Module) -> Result<HashMap<String, LibraryBinding>> {
        let mut import_bindings = HashMap::new();

        // For now, this is a simplified implementation
        // In a full system, this would parse import specs and resolve them
        for dependency_id in &module.dependencies {
            // This would recursively instantiate dependency libraries
            // For now, we'll just note the dependency
            if self.verbose_logging {
                eprintln!("Processing dependency: {}", super::format_module_id(dependency_id));
            }
        }

        Ok(import_bindings)
    }

    /// Evaluates the library body expressions.
    fn evaluate_library_body(&mut self, module: &Module, lib_env: Arc<ThreadSafeEnvironment>) -> Result<()> {
        // This would integrate with the evaluator to run the library body
        // For now, this is a placeholder implementation
        
        if self.verbose_logging && !module.exports.is_empty() {
            eprintln!("Library {} has {} predefined exports", 
                     super::format_module_id(&module.id), 
                     module.exports.len());
        }

        // Install any predefined exports directly into the environment
        for (name, value) in &module.exports {
            lib_env.define(name.clone(), value.clone());
        }

        Ok(())
    }

    /// Resolves export specifications for a library.
    fn resolve_library_exports(
        &mut self, 
        module: &Module, 
        lib_env: Arc<ThreadSafeEnvironment>
    ) -> Result<HashMap<String, LibraryBinding>> {
        let mut exports = HashMap::new();

        // Process predefined exports from the module
        for (name, value) in &module.exports {
            let binding = LibraryBinding {
                name: name.clone(),
                value: value.clone(),
                mutable: false, // Most library exports are immutable
                source_span: None,
            };
            exports.insert(name.clone(), binding);
        }

        // Process any dynamically defined exports from the library environment
        // This would involve inspecting the environment for exported bindings
        for name in lib_env.variable_names() {
            if let Some(value) = lib_env.lookup(&name) {
                exports.entry(name.clone()).or_insert_with(|| LibraryBinding {
                    name: name.clone(),
                    value,
                    mutable: false,
                    source_span: None,
                });
            }
        }

        Ok(exports)
    }

    /// Builds a dependency cycle for error reporting.
    fn build_dependency_cycle(&self, target_id: &ModuleId) -> Vec<ModuleId> {
        let mut cycle = Vec::new();
        let mut found_start = false;

        for id in &self.context.instantiation_stack {
            if id == target_id {
                found_start = true;
            }
            if found_start {
                cycle.push(id.clone());
            }
        }

        cycle.push(target_id.clone()); // Complete the cycle
        cycle
    }

    /// Gets statistics about the instantiation process.
    pub fn get_statistics(&self) -> InstantiationStatistics {
        InstantiationStatistics {
            instantiated_libraries: self.context.instantiated_libraries.len(),
            max_instantiation_depth: self.context.instantiation_depth,
            cached_bindings: self.context.binding_cache.len(),
        }
    }

    /// Clears the instantiation cache (useful for testing or hot-reload).
    pub fn clear_cache(&mut self) {
        self.context.instantiated_libraries.clear();
        self.context.binding_cache.clear();
    }

    /// Enables or disables verbose logging.
    pub fn set_verbose_logging(&mut self, enabled: bool) {
        self.verbose_logging = enabled;
    }

    /// Gets a reference to an instantiated library.
    pub fn get_instantiated_library(&self, id: &ModuleId) -> Option<&Arc<LibraryInstance>> {
        self.context.instantiated_libraries.get(id)
    }

    /// Lists all instantiated libraries.
    pub fn list_instantiated_libraries(&self) -> Vec<ModuleId> {
        self.context.instantiated_libraries.keys().cloned().collect()
    }
}

/// Statistics about the instantiation process.
#[derive(Debug, Clone)]
pub struct InstantiationStatistics {
    /// Number of instantiated libraries
    pub instantiated_libraries: usize,
    /// Maximum instantiation depth reached
    pub max_instantiation_depth: usize,
    /// Number of cached binding resolutions
    pub cached_bindings: usize,
}

impl std::fmt::Display for InstantiationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstantiationError::CircularDependency(cycle) => {
                let cycle_str = cycle.iter()
                    .map(super::format_module_id)
                    .collect::<Vec<_>>()
                    .join(" -> ");
                write!(f, "Circular dependency during instantiation: {cycle_str}")
            }
            InstantiationError::MissingDependency(id) => {
                write!(f, "Missing dependency: {}", super::format_module_id(id))
            }
            InstantiationError::ExportNotFound { library, binding } => {
                write!(f, "Export '{binding}' not found in library {}", super::format_module_id(library))
            }
            InstantiationError::ImportConflict { binding, libraries } => {
                let lib_names = libraries.iter()
                    .map(super::format_module_id)
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "Import conflict for binding '{binding}' from libraries: {lib_names}")
            }
            InstantiationError::InvalidExportSpec(msg) => {
                write!(f, "Invalid export specification: {msg}")
            }
            InstantiationError::EvaluationError(msg) => {
                write!(f, "Evaluation error during instantiation: {msg}")
            }
            InstantiationError::InitializationFailed(msg) => {
                write!(f, "Library initialization failed: {msg}")
            }
        }
    }
}

impl std::error::Error for InstantiationError {}

impl From<InstantiationError> for ModuleError {
    fn from(err: InstantiationError) -> Self {
        ModuleError::InstantiationError(err.to_string())
    }
}

/// Import specification resolver for R7RS-compliant import handling.
#[derive(Debug)]
pub struct ImportSpecResolver;

impl ImportSpecResolver {
    /// Resolves an import specification into module ID and binding configurations.
    pub fn resolve_import_spec(import_expr: &Spanned<Expr>) -> Result<ImportResolution> {
        match &import_expr.inner {
            Expr::List(elements) if !elements.is_empty() => {
                // Handle compound import specs like (only (srfi 1) cons car cdr)
                Self::resolve_compound_import_spec(elements)
            }
            Expr::Identifier(_) => {
                // Simple module name
                let module_id = Self::expr_to_module_id(&import_expr.inner)?;
                Ok(ImportResolution::Direct(module_id))
            }
            _ => Err(Box::new(Error::from(ModuleError::ImportError(
                "Invalid import specification format".to_string()
            ))))
        }
    }

    /// Resolves compound import specifications with filters and renames.
    fn resolve_compound_import_spec(elements: &[Spanned<Expr>]) -> Result<ImportResolution> {
        if elements.is_empty() {
            return Err(Box::new(Error::from(ModuleError::ImportError(
                "Empty import specification".to_string()
            ))));
        }

        match &elements[0].inner {
            Expr::Identifier(keyword) => {
                match keyword.as_str() {
                    "only" => Self::resolve_only_import(&elements[1..]),
                    "except" => Self::resolve_except_import(&elements[1..]),
                    "prefix" => Self::resolve_prefix_import(&elements[1..]),
                    "rename" => Self::resolve_rename_import(&elements[1..]),
                    _ => {
                        // Treat as simple module name list
                        let module_id = Self::elements_to_module_id(elements)?;
                        Ok(ImportResolution::Direct(module_id))
                    }
                }
            }
            _ => {
                // Treat as module name
                let module_id = Self::elements_to_module_id(elements)?;
                Ok(ImportResolution::Direct(module_id))
            }
        }
    }

    /// Resolves "only" import specifications.
    fn resolve_only_import(elements: &[Spanned<Expr>]) -> Result<ImportResolution> {
        if elements.len() < 2 {
            return Err(Box::new(Error::from(ModuleError::ImportError(
                "Invalid 'only' import specification".to_string()
            ))));
        }

        let module_id = Self::expr_to_module_id(&elements[0].inner)?;
        let mut symbols = Vec::new();

        for element in &elements[1..] {
            if let Expr::Identifier(symbol) = &element.inner {
                symbols.push(symbol.clone());
            } else {
                return Err(Box::new(Error::from(ModuleError::ImportError(
                    "Invalid symbol in 'only' import specification".to_string()
                ))));
            }
        }

        Ok(ImportResolution::Only { module_id, symbols })
    }

    /// Resolves "except" import specifications.
    fn resolve_except_import(elements: &[Spanned<Expr>]) -> Result<ImportResolution> {
        if elements.len() < 2 {
            return Err(Box::new(Error::from(ModuleError::ImportError(
                "Invalid 'except' import specification".to_string()
            ))));
        }

        let module_id = Self::expr_to_module_id(&elements[0].inner)?;
        let mut symbols = Vec::new();

        for element in &elements[1..] {
            if let Expr::Identifier(symbol) = &element.inner {
                symbols.push(symbol.clone());
            } else {
                return Err(Box::new(Error::from(ModuleError::ImportError(
                    "Invalid symbol in 'except' import specification".to_string()
                ))));
            }
        }

        Ok(ImportResolution::Except { module_id, symbols })
    }

    /// Resolves "prefix" import specifications.
    fn resolve_prefix_import(elements: &[Spanned<Expr>]) -> Result<ImportResolution> {
        if elements.len() != 2 {
            return Err(Box::new(Error::from(ModuleError::ImportError(
                "Invalid 'prefix' import specification".to_string()
            ))));
        }

        let module_id = Self::expr_to_module_id(&elements[0].inner)?;
        
        if let Expr::Identifier(prefix) = &elements[1].inner {
            Ok(ImportResolution::Prefix { module_id, prefix: prefix.clone() })
        } else {
            Err(Box::new(Error::from(ModuleError::ImportError(
                "Invalid prefix in 'prefix' import specification".to_string()
            ))))
        }
    }

    /// Resolves "rename" import specifications.
    fn resolve_rename_import(elements: &[Spanned<Expr>]) -> Result<ImportResolution> {
        if elements.len() < 2 {
            return Err(Box::new(Error::from(ModuleError::ImportError(
                "Invalid 'rename' import specification".to_string()
            ))));
        }

        let module_id = Self::expr_to_module_id(&elements[0].inner)?;
        let mut renames = HashMap::new();

        for element in &elements[1..] {
            if let Expr::List(rename_spec) = &element.inner {
                if rename_spec.len() == 2 {
                    if let (Expr::Identifier(from), Expr::Identifier(to)) = 
                        (&rename_spec[0].inner, &rename_spec[1].inner) {
                        renames.insert(from.clone(), to.clone());
                        continue;
                    }
                }
            }
            
            return Err(Box::new(Error::from(ModuleError::ImportError(
                "Invalid rename specification".to_string()
            ))));
        }

        Ok(ImportResolution::Rename { module_id, renames })
    }

    /// Converts an expression to a module ID.
    fn expr_to_module_id(expr: &Expr) -> Result<ModuleId> {
        match expr {
            Expr::List(elements) => Self::elements_to_module_id(elements),
            Expr::Identifier(name) => {
                // Simple identifier - treat as single-component module name
                Ok(ModuleId {
                    components: vec![name.clone()],
                    namespace: super::ModuleNamespace::User, // Default to user namespace
                })
            }
            _ => Err(Box::new(Error::from(ModuleError::ImportError(
                "Invalid module identifier in import specification".to_string()
            ))))
        }
    }

    /// Converts a list of expressions to a module ID.
    fn elements_to_module_id(elements: &[Spanned<Expr>]) -> Result<ModuleId> {
        if elements.is_empty() {
            return Err(Box::new(Error::from(ModuleError::ImportError(
                "Empty module name specification".to_string()
            ))));
        }

        let mut components = Vec::new();
        
        for element in elements {
            match &element.inner {
                Expr::Identifier(name) => components.push(name.clone()),
                Expr::Literal(crate::ast::Literal::Number(n)) => components.push(n.to_string()),
                _ => return Err(Box::new(Error::from(ModuleError::ImportError(
                    "Invalid component in module name".to_string()
                ))))
            }
        }

        // Determine namespace based on first component
        let namespace = match components.first().unwrap().as_str() {
            "scheme" => super::ModuleNamespace::R7RS,
            "srfi" => super::ModuleNamespace::SRFI,
            "lambdust" => super::ModuleNamespace::Builtin,
            _ => super::ModuleNamespace::User,
        };

        Ok(ModuleId { components, namespace })
    }
}

/// Result of import specification resolution.
#[derive(Debug, Clone)]
pub enum ImportResolution {
    /// Direct import of entire module
    Direct(ModuleId),
    /// Import only specific symbols
    Only { 
        /// Module to import from
        module_id: ModuleId, 
        /// Symbols to import
        symbols: Vec<String> 
    },
    /// Import all except specific symbols  
    Except { 
        /// Module to import from
        module_id: ModuleId, 
        /// Symbols to exclude from import
        symbols: Vec<String> 
    },
    /// Import with prefix
    Prefix { 
        /// Module to import from
        module_id: ModuleId, 
        /// Prefix to add to imported symbols
        prefix: String 
    },
    /// Import with renames
    Rename { 
        /// Module to import from
        module_id: ModuleId, 
        /// Symbol renames (original -> new)
        renames: HashMap<String, String> 
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_system::{ModuleNamespace, ModuleSource, ModuleMetadata};

    fn create_test_module(name: &str) -> Module {
        Module {
            id: ModuleId {
                components: vec![name.to_string()],
                namespace: ModuleNamespace::User,
            },
            exports: HashMap::new(),
            dependencies: Vec::new(),
            source: Some(ModuleSource::Builtin),
            metadata: ModuleMetadata::default(),
        }
    }

    #[test]
    fn test_library_instantiator_creation() {
        let global_env = Arc::new(GlobalEnvironmentManager::new().unwrap());
        let instantiator = LibraryInstantiator::new(global_env);
        
        let stats = instantiator.get_statistics();
        assert_eq!(stats.instantiated_libraries, 0);
        assert_eq!(stats.max_instantiation_depth, 0);
    }

    #[test]
    fn test_basic_library_instantiation() {
        let global_env = Arc::new(GlobalEnvironmentManager::new().unwrap());
        let mut instantiator = LibraryInstantiator::new(global_env);
        
        let module = create_test_module("test");
        let result = instantiator.instantiate_library(module);
        
        assert!(result.is_ok());
        let instance = result.unwrap();
        assert_eq!(instance.module.id.components[0], "test");
    }

    #[test]
    fn test_import_spec_resolution() {
        use crate::ast::{Literal, Spanned};
        use crate::diagnostics::Span;
        
        // Test simple module name: (srfi 1)
        let elements = vec![
            Spanned::new(Expr::Identifier("srfi".to_string()), Span::new(0, 4)),
            Spanned::new(Expr::Literal(Literal::Integer(1)), Span::new(5, 6)),
        ];
        let import_spec = Spanned::new(Expr::List(elements), Span::new(0, 7));
        
        let result = ImportSpecResolver::resolve_import_spec(&import_spec);
        assert!(result.is_ok());
        
        if let ImportResolution::Direct(module_id) = result.unwrap() {
            assert_eq!(module_id.namespace, ModuleNamespace::SRFI);
            assert_eq!(module_id.components, vec!["srfi", "1"]);
        } else {
            panic!("Expected direct import resolution");
        }
    }

    #[test]
    fn test_only_import_resolution() {
        use crate::ast::Spanned;
        use crate::diagnostics::Span;
        
        // Test: (only (srfi 1) cons car cdr)
        let elements = vec![
            Spanned::new(Expr::Identifier("only".to_string()), Span::new(0, 4)),
            Spanned::new(Expr::List(vec![
                Spanned::new(Expr::Identifier("srfi".to_string()), Span::new(6, 10)),
                Spanned::new(Expr::Literal(crate::ast::Literal::Integer(1)), Span::new(11, 12)),
            ]), Span::new(5, 13)),
            Spanned::new(Expr::Identifier("cons".to_string()), Span::new(14, 18)),
            Spanned::new(Expr::Identifier("car".to_string()), Span::new(19, 22)),
            Spanned::new(Expr::Identifier("cdr".to_string()), Span::new(23, 26)),
        ];
        let import_spec = Spanned::new(Expr::List(elements), Span::new(0, 27));
        
        let result = ImportSpecResolver::resolve_import_spec(&import_spec);
        assert!(result.is_ok());
        
        if let ImportResolution::Only { module_id, symbols } = result.unwrap() {
            assert_eq!(module_id.namespace, ModuleNamespace::SRFI);
            assert_eq!(symbols, vec!["cons", "car", "cdr"]);
        } else {
            panic!("Expected 'only' import resolution");
        }
    }
}
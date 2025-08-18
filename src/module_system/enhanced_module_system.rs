//! Enhanced module system with runtime integration and auto-loading.
//!
//! This module extends the basic module system with:
//! - Runtime library instantiation
//! - Standard library auto-loading
//! - Hot-reload support for development
//! - Comprehensive validation and monitoring

use super::{ModuleId, Module, ImportSpec, loader, cache, resolver, import, runtime_integration};
use crate::diagnostics::Result;
use crate::eval::Value;
use crate::runtime::GlobalEnvironmentManager;
use std::collections::HashMap;
use std::sync::Arc;

/// Enhanced module system coordinator with library instantiation.
#[derive(Debug)]
pub struct EnhancedModuleSystem {
    /// Module loader for finding and loading modules
    loader: loader::ModuleLoader,
    /// Cache of loaded modules
    cache: cache::ModuleCache,
    /// Module resolver for dependency management
    resolver: resolver::DependencyResolver,
    /// Runtime library instantiator
    instantiator: runtime_integration::LibraryInstantiator,
    /// Auto-loading configuration
    auto_loading: AutoLoadingConfig,
}

/// Configuration for automatic library loading.
#[derive(Debug, Clone)]
pub struct AutoLoadingConfig {
    /// Whether to enable standard library auto-loading
    pub enable_stdlib_autoload: bool,
    /// Standard library modules to auto-load
    pub stdlib_modules: Vec<ModuleId>,
    /// Whether to enable lazy loading
    pub lazy_loading: bool,
    /// Preload timeout
    pub preload_timeout: std::time::Duration,
}

impl Default for AutoLoadingConfig {
    fn default() -> Self {
        Self {
            enable_stdlib_autoload: true,
            stdlib_modules: Vec::new(), // Use built-in defaults
            lazy_loading: true,
            preload_timeout: std::time::Duration::from_secs(30),
        }
    }
}

impl EnhancedModuleSystem {
    /// Creates a new enhanced module system with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(AutoLoadingConfig::default())
    }

    /// Creates a module system with custom auto-loading configuration.
    pub fn with_config(config: AutoLoadingConfig) -> Result<Self> {
        let loader = loader::ModuleLoader::new()?;
        let cache = cache::ModuleCache::new();
        let resolver = resolver::DependencyResolver::new();
        let global_env = Arc::new(GlobalEnvironmentManager::new());
        let instantiator = runtime_integration::LibraryInstantiator::new(global_env);
        
        let mut system = Self {
            loader,
            cache,
            resolver,
            instantiator,
            auto_loading: config,
        };

        // Perform initial setup
        system.initialize_system()?;
        
        Ok(system)
    }

    /// Loads and caches a module by its identifier.
    pub fn load_module(&mut self, id: &ModuleId) -> Result<Arc<Module>> {
        // Check cache first
        if let Some(module) = self.cache.get(id) {
            return Ok(module);
        }

        // Load the module from source
        let module = self.loader.load(id)?;
        
        // Resolve dependencies
        let resolved = self.resolver.resolve_dependencies(module)?;
        
        // Cache the resolved module
        let module_arc = Arc::new(resolved);
        self.cache.insert(id.clone(), module_arc.clone());
        
        Ok(module_arc)
    }

    /// Loads and instantiates a library for runtime use.
    pub fn instantiate_library(&mut self, id: &ModuleId) -> Result<Arc<runtime_integration::LibraryInstance>> {
        // Load the module first
        let module = self.load_module(id)?;
        
        // Instantiate it for runtime use
        self.instantiator.instantiate_library((*module).clone())
    }

    /// Resolves an import specification into a set of bindings with full R7RS support.
    pub fn resolve_import(&mut self, import: &ImportSpec) -> Result<HashMap<String, Value>> {
        let library = self.instantiate_library(&import.module_id)?;
        
        // Convert library bindings to import bindings
        let mut bindings = HashMap::new();
        for (name, lib_binding) in &library.exports {
            bindings.insert(name.clone(), lib_binding.value.clone());
        }
        
        // Apply import configuration to filter/rename bindings
        import::apply_import_config(&bindings, &import.config)
    }

    /// Resolves an import specification from an AST expression.
    pub fn resolve_import_from_expr(&mut self, import_expr: &crate::ast::Spanned<crate::ast::Expr>) -> Result<HashMap<String, Value>> {
        // Use the import spec resolver to parse the expression
        let import_resolution = runtime_integration::ImportSpecResolver::resolve_import_spec(import_expr)?;
        
        match import_resolution {
            runtime_integration::ImportResolution::Direct(module_id) => {
                let library = self.instantiate_library(&module_id)?;
                let mut bindings = HashMap::new();
                for (name, lib_binding) in &library.exports {
                    bindings.insert(name.clone(), lib_binding.value.clone());
                }
                Ok(bindings)
            }
            runtime_integration::ImportResolution::Only { module_id, symbols } => {
                let library = self.instantiate_library(&module_id)?;
                let mut bindings = HashMap::new();
                for symbol in symbols {
                    if let Some(lib_binding) = library.exports.get(&symbol) {
                        bindings.insert(symbol, lib_binding.value.clone());
                    }
                }
                Ok(bindings)
            }
            runtime_integration::ImportResolution::Except { module_id, symbols } => {
                let library = self.instantiate_library(&module_id)?;
                let mut bindings = HashMap::new();
                for (name, lib_binding) in &library.exports {
                    if !symbols.contains(name) {
                        bindings.insert(name.clone(), lib_binding.value.clone());
                    }
                }
                Ok(bindings)
            }
            runtime_integration::ImportResolution::Prefix { module_id, prefix } => {
                let library = self.instantiate_library(&module_id)?;
                let mut bindings = HashMap::new();
                for (name, lib_binding) in &library.exports {
                    let prefixed_name = format!("{prefix}{name}");
                    bindings.insert(prefixed_name, lib_binding.value.clone());
                }
                Ok(bindings)
            }
            runtime_integration::ImportResolution::Rename { module_id, renames } => {
                let library = self.instantiate_library(&module_id)?;
                let mut bindings = HashMap::new();
                for (name, lib_binding) in &library.exports {
                    let final_name = renames.get(name).unwrap_or(name);
                    bindings.insert(final_name.clone(), lib_binding.value.clone());
                }
                Ok(bindings)
            }
        }
    }

    /// Registers a built-in module.
    pub fn register_builtin_module(&mut self, module: Module) {
        let id = module.id.clone();
        self.cache.insert(id, Arc::new(module));
    }

    /// Gets information about a loaded module.
    pub fn get_module_info(&self, id: &ModuleId) -> Option<Arc<Module>> {
        self.cache.get(id)
    }

    /// Lists all available modules including discoverable ones.
    pub fn list_modules(&self) -> Vec<ModuleId> {
        let mut modules = self.cache.list_modules();
        modules.extend(self.loader.discover_modules());
        modules.sort();
        modules.dedup();
        modules
    }

    /// Gets the instantiator for advanced operations.
    pub fn instantiator(&mut self) -> &mut runtime_integration::LibraryInstantiator {
        &mut self.instantiator
    }

    /// Gets system validation information.
    pub fn validate_system(&self) -> Result<SystemValidationReport> {
        let loader_report = self.loader.validate_library_setup()?;
        let instantiation_stats = self.instantiator.get_statistics();
        
        Ok(SystemValidationReport {
            library_validation: loader_report,
            instantiation_stats,
            cached_modules: self.cache.len(),
            available_modules: self.list_modules().len(),
        })
    }

    /// Enables hot-reload support for development.
    pub fn enable_hot_reload(&mut self) -> Result<()> {
        // Enable verbose logging in the instantiator for development feedback
        self.instantiator.set_verbose_logging(true);
        Ok(())
    }

    /// Reloads a specific library (for hot-reload support).
    pub fn reload_library(&mut self, id: &ModuleId) -> Result<()> {
        // Remove from cache to force reload
        self.cache.remove(id);
        
        // Clear instantiation cache for this library
        self.instantiator.clear_cache();
        
        // Reload the library
        self.instantiate_library(id)?;
        
        Ok(())
    }

    /// Performs system initialization including standard library loading.
    fn initialize_system(&mut self) -> Result<()> {
        if self.auto_loading.enable_stdlib_autoload {
            self.preload_standard_libraries()?;
        }
        Ok(())
    }

    /// Preloads essential standard library modules.
    fn preload_standard_libraries(&mut self) -> Result<()> {
        let essential_modules = if self.auto_loading.stdlib_modules.is_empty() {
            // Default essential modules for R7RS compliance
            vec![
                ModuleId {
                    components: vec!["scheme".to_string(), "base".to_string()],
                    namespace: super::ModuleNamespace::R7RS,
                },
                ModuleId {
                    components: vec!["scheme".to_string(), "write".to_string()],
                    namespace: super::ModuleNamespace::R7RS,
                },
                ModuleId {
                    components: vec!["scheme".to_string(), "read".to_string()],
                    namespace: super::ModuleNamespace::R7RS,
                },
            ]
        } else {
            self.auto_loading.stdlib_modules.clone()
        };

        for module_id in &essential_modules {
            // Try to load essential modules, but don't fail if they're not available
            if self.load_module(module_id).is_ok() {
                eprintln!("Preloaded standard library: {}", super::format_module_id(module_id));
            }
        }

        Ok(())
    }

    /// Gets the library path resolver for configuration.
    pub fn library_resolver(&self) -> &crate::runtime::LibraryPathResolver {
        self.loader.library_resolver()
    }

    /// Lists all instantiated libraries.
    pub fn list_instantiated_libraries(&self) -> Vec<ModuleId> {
        self.instantiator.list_instantiated_libraries()
    }

    /// Clears all caches (useful for testing or major reloads).
    pub fn clear_all_caches(&mut self) {
        self.cache.clear();
        self.instantiator.clear_cache();
        self.resolver.clear_cache();
    }
}

impl Default for EnhancedModuleSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create default enhanced module system")
    }
}

/// System validation report combining all validation results.
#[derive(Debug, Clone)]
pub struct SystemValidationReport {
    /// Library system validation
    pub library_validation: crate::runtime::LibraryValidationReport,
    /// Instantiation statistics
    pub instantiation_stats: runtime_integration::InstantiationStatistics,
    /// Number of cached modules
    pub cached_modules: usize,
    /// Total available modules
    pub available_modules: usize,
}

impl SystemValidationReport {
    /// Checks if the entire module system is healthy.
    pub fn is_healthy(&self) -> bool {
        self.library_validation.is_usable() && self.available_modules > 0
    }

    /// Gets a comprehensive system summary.
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str("Enhanced Module System Status:\n");
        summary.push_str(&format!("• System health: {}\n", 
            if self.is_healthy() { "Healthy" } else { "Issues detected" }));
        summary.push_str(&format!("• Available modules: {}\n", self.available_modules));
        summary.push_str(&format!("• Cached modules: {}\n", self.cached_modules));
        summary.push_str(&format!("• Instantiated libraries: {}\n", 
            self.instantiation_stats.instantiated_libraries));
        
        summary.push('\n');
        summary.push_str(&self.library_validation.summary());
        
        summary
    }
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
    fn test_enhanced_module_system_creation() {
        let system = EnhancedModuleSystem::new();
        assert!(system.is_ok());
    }

    #[test]
    fn test_auto_loading_config() {
        let config = AutoLoadingConfig {
            enable_stdlib_autoload: false,
            stdlib_modules: vec![],
            lazy_loading: false,
            preload_timeout: std::time::Duration::from_secs(10),
        };
        
        let system = EnhancedModuleSystem::with_config(config);
        assert!(system.is_ok());
    }

    #[test]
    fn test_system_validation() {
        let system = EnhancedModuleSystem::new().unwrap();
        let report = system.validate_system();
        assert!(report.is_ok());
    }

    #[test]
    fn test_module_registration() {
        let mut system = EnhancedModuleSystem::new().unwrap();
        let module = create_test_module("test");
        let module_id = module.id.clone();
        
        system.register_builtin_module(module);
        
        let info = system.get_module_info(&module_id);
        assert!(info.is_some());
    }

    #[test]
    fn test_library_instantiation() {
        let mut system = EnhancedModuleSystem::new().unwrap();
        let module = create_test_module("test");
        let module_id = module.id.clone();
        
        system.register_builtin_module(module);
        
        let result = system.instantiate_library(&module_id);
        assert!(result.is_ok());
    }
}
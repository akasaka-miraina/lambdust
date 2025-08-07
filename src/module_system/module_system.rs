use super::{ModuleId, Module, ImportSpec, loader, cache, resolver, import};
use crate::diagnostics::Result;
use crate::eval::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// The main module system coordinator.
#[derive(Debug)]
pub struct ModuleSystem {
    /// Module loader for finding and loading modules
    loader: loader::ModuleLoader,
    /// Cache of loaded modules
    cache: cache::ModuleCache,
    /// Module resolver for dependency management
    resolver: resolver::DependencyResolver,
}

impl ModuleSystem {
    /// Creates a new module system with default configuration.
    pub fn new() -> Result<Self> {
        let loader = loader::ModuleLoader::new()?;
        let cache = cache::ModuleCache::new();
        let resolver = resolver::DependencyResolver::new();
        
        Ok(Self {
            loader,
            cache,
            resolver,
        })
    }

    /// Loads a module by its identifier.
    pub fn load_module(&mut self, id: &ModuleId) -> Result<Arc<Module>> {
        // Check cache first
        if let Some(module) = self.cache.get(id) {
            return Ok(module);
        }

        // Load the module
        let module = self.loader.load(id)?;
        
        // Resolve dependencies
        let resolved = self.resolver.resolve_dependencies(module)?;
        
        // Cache the resolved module
        let module_arc = Arc::new(resolved);
        self.cache.insert(id.clone()), module_arc.clone());
        
        Ok(module_arc)
    }

    /// Resolves an import specification into a set of bindings.
    pub fn resolve_import(&mut self, import: &ImportSpec) -> Result<HashMap<String, Value>> {
        let module = self.load_module(&import.module_id)?;
        
        // Apply import configuration to get final bindings
        import::apply_import_config(&module.exports, &import.config)
    }

    /// Registers a built-in module.
    pub fn register_builtin_module(&mut self, module: Module) {
        let id = module.id.clone());
        self.cache.insert(id, Arc::new(module));
    }

    /// Gets information about a loaded module.
    pub fn get_module_info(&self, id: &ModuleId) -> Option<Arc<Module>> {
        self.cache.get(id)
    }

    /// Lists all available modules.
    pub fn list_modules(&self) -> Vec<ModuleId> {
        self.cache.list_modules()
    }
}

impl Default for ModuleSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create default module system")
    }
}
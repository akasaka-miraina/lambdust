//! Dynamic module management for metaprogramming operations.
//!
//! This module provides facilities for dynamic module loading, unloading,
//! dependency tracking, and hook system management.

use crate::eval::{Value, Environment};
use crate::module_system::{
    loader::ModuleLoader, 
    ModuleId, 
    cache::ModuleCache,
    Module
};
use crate::diagnostics::{Error, Result};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::SystemTime;
use std::path::PathBuf;

/// Module manager for dynamic module operations.
#[derive(Debug)]
pub struct ModuleManager {
    /// Module loader
    loader: ModuleLoader,
    /// Module cache
    cache: ModuleCache,
    /// Loaded modules
    loaded_modules: HashMap<ModuleId, LoadedModule>,
    /// Module dependencies
    dependencies: HashMap<ModuleId, Vec<ModuleId>>,
    /// Module loading hooks
    hooks: ModuleHooks,
}

/// Information about a loaded module.
#[derive(Debug, Clone)]
pub struct LoadedModule {
    /// Module information
    pub module: Module,
    /// Module environment
    pub environment: Rc<Environment>,
    /// Load time
    pub loaded_at: SystemTime,
    /// Load count (how many times loaded)
    pub load_count: usize,
    /// Dependencies
    pub dependencies: Vec<ModuleId>,
    /// Dependents (modules that depend on this one)
    pub dependents: Vec<ModuleId>,
}

/// Hooks for module loading/unloading.
/// Type alias for pre-load hook function
type PreLoadHook = Box<dyn Fn(&ModuleId) -> Result<()>>;

/// Type alias for post-load hook function
type PostLoadHook = Box<dyn Fn(&ModuleId, &LoadedModule) -> Result<()>>;

/// Type alias for pre-unload hook function
type PreUnloadHook = Box<dyn Fn(&ModuleId) -> Result<()>>;

/// Type alias for post-unload hook function
type PostUnloadHook = Box<dyn Fn(&ModuleId) -> Result<()>>;

/// Module hooks for loading/unloading.
pub struct ModuleHooks {
    /// Called before module loading
    pub pre_load: Vec<PreLoadHook>,
    /// Called after module loading
    pub post_load: Vec<PostLoadHook>,
    /// Called before module unloading
    pub pre_unload: Vec<PreUnloadHook>,
    /// Called after module unloading
    pub post_unload: Vec<PostUnloadHook>,
}

impl std::fmt::Debug for ModuleHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleHooks")
            .field("pre_load", &format!("{count} hook(s)", count = self.pre_load.len()))
            .field("post_load", &format!("{count} hook(s)", count = self.post_load.len()))
            .field("pre_unload", &format!("{count} hook(s)", count = self.pre_unload.len()))
            .field("post_unload", &format!("{count} hook(s)", count = self.post_unload.len()))
            .finish()
    }
}

impl ModuleManager {
    /// Creates a new module manager.
    pub fn new() -> Result<Self> {
        Ok(Self {
            loader: ModuleLoader::new()?,
            cache: ModuleCache::new(),
            loaded_modules: HashMap::new(),
            dependencies: HashMap::new(),
            hooks: ModuleHooks::new(),
        })
    }

    /// Loads a module dynamically.
    pub fn load_module(&mut self, module_id: ModuleId, path: Option<PathBuf>) -> Result<Rc<Environment>> {
        // Call pre-load hooks
        for hook in &self.hooks.pre_load {
            hook(&module_id)?;
        }

        // Check if already loaded
        if let Some(loaded) = self.loaded_modules.get_mut(&module_id) {
            loaded.load_count += 1;
            return Ok(loaded.environment.clone());
        }

        // Load module definition
        // If a path is provided, we might need to create a file-based module ID
        let effective_module_id = if let Some(_path) = path {
            // For now, just use the provided module_id
            // In a real implementation, you might create a File namespace module
            module_id.clone()
        } else {
            module_id.clone()
        };
        
        let module = self.loader.load(&effective_module_id)?;

        // Create module environment
        let module_env = Rc::new(Environment::new(None, 0));

        // Install module exports
        for _value in module.exports.values() {
            // Implementation would install exported bindings
            // module_env.define(name.clone(), value.clone());
        }

        let loaded_module = LoadedModule {
            module,
            environment: module_env.clone(),
            loaded_at: SystemTime::now(),
            load_count: 1,
            dependencies: Vec::new(),
            dependents: Vec::new(),
        };

        // Call post-load hooks
        for hook in &self.hooks.post_load {
            hook(&module_id, &loaded_module)?;
        }

        self.loaded_modules.insert(module_id, loaded_module);
        Ok(module_env)
    }

    /// Unloads a module.
    pub fn unload_module(&mut self, module_id: &ModuleId) -> Result<()> {
        if let Some(loaded) = self.loaded_modules.get_mut(module_id) {
            loaded.load_count = loaded.load_count.saturating_sub(1);
            
            if loaded.load_count == 0 {
                // Call pre-unload hooks
                for hook in &self.hooks.pre_unload {
                    hook(module_id)?;
                }

                self.loaded_modules.remove(module_id);

                // Call post-unload hooks
                for hook in &self.hooks.post_unload {
                    hook(module_id)?;
                }
            }
        }

        Ok(())
    }

    /// Gets a loaded module.
    pub fn get_module(&self, module_id: &ModuleId) -> Option<&LoadedModule> {
        self.loaded_modules.get(module_id)
    }

    /// Lists all loaded modules.
    pub fn loaded_modules(&self) -> Vec<&ModuleId> {
        self.loaded_modules.keys().collect()
    }

    /// Adds a dependency relationship.
    pub fn add_dependency(&mut self, dependent: ModuleId, dependency: ModuleId) {
        self.dependencies.entry(dependent.clone()).or_default().push(dependency.clone());
        
        // Update dependent tracking in loaded modules
        if let Some(dep_module) = self.loaded_modules.get_mut(&dependency) {
            if !dep_module.dependents.contains(&dependent) {
                dep_module.dependents.push(dependent);
            }
        }
    }

    /// Gets dependencies of a module.
    pub fn get_dependencies(&self, module_id: &ModuleId) -> Vec<&ModuleId> {
        self.dependencies.get(module_id)
            .map(|deps| deps.iter().collect())
            .unwrap_or_default()
    }

    /// Adds a pre-load hook.
    pub fn add_pre_load_hook<F>(&mut self, hook: F)
    where
        F: Fn(&ModuleId) -> Result<()> + 'static,
    {
        self.hooks.pre_load.push(Box::new(hook));
    }

    /// Adds a post-load hook.
    pub fn add_post_load_hook<F>(&mut self, hook: F)
    where
        F: Fn(&ModuleId, &LoadedModule) -> Result<()> + 'static,
    {
        self.hooks.post_load.push(Box::new(hook));
    }

    /// Adds a pre-unload hook.
    pub fn add_pre_unload_hook<F>(&mut self, hook: F)
    where
        F: Fn(&ModuleId) -> Result<()> + 'static,
    {
        self.hooks.pre_unload.push(Box::new(hook));
    }

    /// Adds a post-unload hook.
    pub fn add_post_unload_hook<F>(&mut self, hook: F)
    where
        F: Fn(&ModuleId) -> Result<()> + 'static,
    {
        self.hooks.post_unload.push(Box::new(hook));
    }
}

impl LoadedModule {
    /// Gets the age of this loaded module.
    pub fn age(&self) -> Option<std::time::Duration> {
        SystemTime::now().duration_since(self.loaded_at).ok()
    }

    /// Checks if this module has dependencies.
    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
    }

    /// Checks if this module has dependents.
    pub fn has_dependents(&self) -> bool {
        !self.dependents.is_empty()
    }
}

impl Default for ModuleHooks {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleHooks {
    /// Creates a new module hooks collection.
    pub fn new() -> Self {
        Self {
            pre_load: Vec::new(),
            post_load: Vec::new(),
            pre_unload: Vec::new(),
            post_unload: Vec::new(),
        }
    }

    /// Clears all hooks.
    pub fn clear(&mut self) {
        self.pre_load.clear();
        self.post_load.clear();
        self.pre_unload.clear();
        self.post_unload.clear();
    }

    /// Gets the total number of hooks.
    pub fn hook_count(&self) -> usize {
        self.pre_load.len() + self.post_load.len() + self.pre_unload.len() + self.post_unload.len()
    }
}

impl Default for ModuleManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default ModuleManager")
    }
}
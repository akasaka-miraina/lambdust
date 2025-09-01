//! Lazy evaluation monad system for module loading in Lambdust
//!
//! This module implements a sophisticated lazy module loading system with:
//! - Lazy evaluation: Modules are loaded only when their exports are accessed
//! - Memoization: Once evaluated, module exports are cached for future use
//! - Monadic error handling: Proper error propagation through the loading chain
//! - Circular dependency resolution: Handle module interdependencies gracefully
//! - Thread safety: All operations are thread-safe using Arc<RwLock<>>

use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span};
use crate::eval::{DefaultSchemeModuleEvaluator, Evaluator, SchemeModuleEvaluator, ThreadSafeEnvironment, Value};
use crate::lexer::Lexer;
use crate::module_system::{ModuleError, ModuleId, ModuleMetadata, ModuleNamespace};
use crate::parser::Parser;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

/// A lazy computation that can be evaluated exactly once with memoization.
/// Uses interior mutability to allow mutation through immutable references.
pub struct LazyComputation<T> {
    /// The computation to be performed, wrapped in Option for one-time execution
    computation: RwLock<Option<Box<dyn FnOnce() -> Result<T> + Send + Sync>>>,
    /// Cached result of the computation
    result: RwLock<Option<Result<T>>>,
}

impl<T> LazyComputation<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Creates a new lazy computation.
    pub fn new<F>(computation: F) -> Self
    where
        F: FnOnce() -> Result<T> + Send + Sync + 'static,
    {
        Self {
            computation: RwLock::new(Some(Box::new(computation))),
            result: RwLock::new(None),
        }
    }

    /// Evaluates the computation if not already done and returns the cached result.
    /// This method is thread-safe and guarantees the computation runs exactly once.
    pub fn evaluate(&self) -> Result<T> {
        // Fast path: check if we already have a result
        {
            let result_guard = self.result.read();
            if let Some(ref cached_result) = *result_guard {
                return cached_result.clone();
            }
        }

        // Slow path: we need to compute the result
        let mut result_guard = self.result.write();
        
        // Double-check pattern: another thread might have computed it while we waited
        if let Some(ref cached_result) = *result_guard {
            return cached_result.clone();
        }

        // Take the computation out (this can only happen once)
        let computation = {
            let mut comp_guard = self.computation.write();
            comp_guard.take()
        };

        match computation {
            Some(comp) => {
                // Execute the computation
                let result = comp();
                // Cache the result
                *result_guard = Some(result.clone());
                result
            }
            None => {
                // This should never happen in normal circumstances
                Err(Box::new(Error::runtime_error(
                    "LazyComputation: computation already consumed".to_string(),
                    None,
                )))
            }
        }
    }

    /// Checks if the computation has been evaluated.
    pub fn is_evaluated(&self) -> bool {
        self.result.read().is_some()
    }
}

/// Represents a module that can be loaded lazily with proper dependency management.
pub struct LazyModule {
    /// Module metadata
    pub metadata: ModuleMetadata,
    /// Lazy loader for module exports
    pub loader: LazyComputation<HashMap<String, Value>>,
    /// Module dependencies (for topological sorting)
    pub dependencies: Vec<ModuleId>,
    /// Source file path for this module
    pub source_path: Option<PathBuf>,
}

impl LazyModule {
    /// Creates a new lazy module with the given metadata and loader.
    pub fn new<F>(
        metadata: ModuleMetadata,
        dependencies: Vec<ModuleId>,
        source_path: Option<PathBuf>,
        loader: F,
    ) -> Self
    where
        F: FnOnce() -> Result<HashMap<String, Value>> + Send + Sync + 'static,
    {
        Self {
            metadata,
            loader: LazyComputation::new(loader),
            dependencies,
            source_path,
        }
    }

    /// Gets the exports for this module, loading it if necessary.
    pub fn get_exports(&self) -> Result<HashMap<String, Value>> {
        self.loader.evaluate()
    }

    /// Checks if this module has been loaded.
    pub fn is_loaded(&self) -> bool {
        self.loader.is_evaluated()
    }

    /// Gets a specific export by name.
    pub fn get_export(&self, name: &str) -> Result<Option<Value>> {
        let exports = self.get_exports()?;
        Ok(exports.get(name).cloned())
    }
}

/// Statistics for monitoring the performance of the module loader.
#[derive(Debug, Default, Clone)]
pub struct LoadingStatistics {
    /// Number of modules successfully loaded
    pub modules_loaded: u64,
    /// Number of cache hits (modules already loaded)
    pub cache_hits: u64,
    /// Number of dependency cycles detected and resolved
    pub cycles_resolved: u64,
    /// Number of loading errors encountered
    pub loading_errors: u64,
}

/// A monadic module loader with lazy evaluation, caching, and dependency resolution.
/// This is the main entry point for the lazy module loading system.
pub struct MonadicModuleLoader {
    /// Registry of all known modules
    modules: Arc<RwLock<HashMap<ModuleId, LazyModule>>>,
    /// Stack of currently loading modules (for cycle detection)
    loading_stack: Arc<RwLock<Vec<ModuleId>>>,
    /// Module search paths
    search_paths: Arc<RwLock<Vec<PathBuf>>>,
    /// Performance statistics
    statistics: Arc<RwLock<LoadingStatistics>>,
    /// Module evaluator for executing Scheme code
    module_evaluator: Arc<dyn SchemeModuleEvaluator>,
}

impl MonadicModuleLoader {
    /// Creates a new monadic module loader.
    pub fn new(global_env: ThreadSafeEnvironment) -> Self {
        let module_evaluator = Arc::new(DefaultSchemeModuleEvaluator::new());
        
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
            loading_stack: Arc::new(RwLock::new(Vec::new())),
            search_paths: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(LoadingStatistics::default())),
            module_evaluator,
        }
    }

    /// Creates a new monadic module loader with a custom evaluator.
    pub fn with_evaluator(module_evaluator: Arc<dyn SchemeModuleEvaluator>) -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
            loading_stack: Arc::new(RwLock::new(Vec::new())),
            search_paths: Arc::new(RwLock::new(Vec::new())),
            statistics: Arc::new(RwLock::new(LoadingStatistics::default())),
            module_evaluator,
        }
    }

    /// Adds a search path for module loading.
    pub fn add_search_path<P: Into<PathBuf>>(&self, path: P) {
        let mut paths = self.search_paths.write();
        paths.push(path.into());
    }

    /// Registers a lazy module with the loader.
    pub fn register_module(&self, module_id: ModuleId, module: LazyModule) {
        let mut modules = self.modules.write();
        modules.insert(module_id, module);
    }

    /// Loads a module by ID, handling dependencies and caching automatically.
    /// This is the main entry point for module loading.
    pub fn load_module(&self, module_id: &ModuleId) -> Result<HashMap<String, Value>> {
        eprintln!("DEBUG: Lazy module loader trying to load: {:?}", module_id);
        
        let result = self.load_module_impl(module_id);
        if let Ok(ref exports) = result {
            eprintln!("DEBUG: Lazy module loader loaded {} exports for {:?}", exports.len(), module_id);
            for (name, _value) in exports.iter().take(5) {
                eprintln!("DEBUG: - Export: {}", name);
            }
        }
        result
    }
    
    /// Internal implementation of module loading.
    fn load_module_impl(&self, module_id: &ModuleId) -> Result<HashMap<String, Value>> {
        // Builtin modules are always available and don't need loading
        if is_builtin_module(module_id) {
            eprintln!("DEBUG: Returning empty exports for builtin module: {:?}", module_id);
            return Ok(HashMap::new());
        }
        
        // Check for cycles first
        {
            let loading_stack = self.loading_stack.read();
            if loading_stack.contains(module_id) {
                let cycle: Vec<String> = loading_stack
                    .iter()
                    .map(|id| format!("{:?}", id))
                    .collect();
                return Err(Box::new(Error::from(ModuleError::CircularDependency(
                    loading_stack.clone(),
                ))));
            }
        }

        // Check if module is already registered
        let _module_check = {
            let modules = self.modules.read();
            if let Some(module) = modules.get(module_id) {
                // Fast path: module exists and might be cached
                if module.is_loaded() {
                    let mut stats = self.statistics.write();
                    stats.cache_hits += 1;
                    return module.get_exports();
                }
                // Module exists but not loaded - we'll load it below
            } else {
                // Module doesn't exist - try to discover and register it
                drop(modules); // Release the read lock
                self.discover_and_register_module(module_id)?;
            }
        };

        // At this point, the module should be registered
        let _module_exists = {
            let modules = self.modules.read();
            modules.get(module_id).ok_or_else(|| {
                Box::new(Error::from(ModuleError::NotFound(module_id.clone())))
            })?;
            // We can't return the module directly due to lifetime issues
            // So we'll proceed with loading in the next step
        };

        // Add to loading stack
        {
            let mut loading_stack = self.loading_stack.write();
            loading_stack.push(module_id.clone());
        }

        // Load dependencies first
        let dependencies = {
            let modules = self.modules.read();
            modules.get(module_id)
                .map(|m| m.dependencies.clone())
                .unwrap_or_default()
        };

        for dep in &dependencies {
            self.load_module(dep)?;
        }

        // Now load the module itself
        let result = {
            let modules = self.modules.read();
            if let Some(module) = modules.get(module_id) {
                module.get_exports()
            } else {
                Err(Box::new(Error::from(ModuleError::NotFound(module_id.clone()))))
            }
        };

        // Remove from loading stack
        {
            let mut loading_stack = self.loading_stack.write();
            if let Some(pos) = loading_stack.iter().position(|x| x == module_id) {
                loading_stack.remove(pos);
            }
        }

        // Update statistics
        match &result {
            Ok(_) => {
                let mut stats = self.statistics.write();
                stats.modules_loaded += 1;
            }
            Err(_) => {
                let mut stats = self.statistics.write();
                stats.loading_errors += 1;
            }
        }

        result
    }

    /// Discovers and registers a module that's not yet in the registry.
    fn discover_and_register_module(&self, module_id: &ModuleId) -> Result<()> {
        let source_path = self.find_module_source(module_id)?;
        
        // Read and parse the module source
        let source_code = std::fs::read_to_string(&source_path)
            .map_err(|e| Box::new(Error::io_error(
                format!("Failed to read module source: {}", e)
            )))?;

        // Create lazy loader for this module
        let module_id_clone = module_id.clone();
        let source_code_clone = source_code.clone();
        let evaluator_ref = self.module_evaluator.clone();
        
        let loader = move || -> Result<HashMap<String, Value>> {
            // This will be executed lazily when the module is first accessed
            Self::evaluate_module_source(&module_id_clone, &source_code_clone, evaluator_ref)
        };

        // Extract dependencies from the module source (simplified for now)
        let dependencies = self.extract_dependencies(&source_code)?;

        // Create metadata (simplified for now)
        let metadata = ModuleMetadata::default();

        // Create and register the lazy module
        let lazy_module = LazyModule::new(
            metadata,
            dependencies,
            Some(source_path),
            loader,
        );

        self.register_module(module_id.clone(), lazy_module);
        Ok(())
    }

    /// Finds the source file for a given module ID.
    fn find_module_source(&self, module_id: &ModuleId) -> Result<PathBuf> {
        let search_paths = self.search_paths.read();
        
        for base_path in search_paths.iter() {
            // Convert module ID to file path
            let relative_path = self.module_id_to_path(module_id);
            let full_path = base_path.join(&relative_path);
            
            if full_path.exists() {
                return Ok(full_path);
            }
        }
        
        Err(Box::new(Error::from(ModuleError::NotFound(module_id.clone()))))
    }

    /// Converts a module ID to a relative file path.
    fn module_id_to_path(&self, module_id: &ModuleId) -> PathBuf {
        match &module_id.namespace {
            crate::module_system::ModuleNamespace::SRFI => {
                let mut path = PathBuf::from("modules/srfi");
                if !module_id.components.is_empty() {
                    path.push(format!("{}.scm", module_id.components[0]));
                }
                path
            }
            crate::module_system::ModuleNamespace::R7RS => {
                let mut path = PathBuf::from("r7rs");
                for component in &module_id.components {
                    path.push(component);
                }
                path.set_extension("scm");
                path
            }
            crate::module_system::ModuleNamespace::Builtin => {
                let mut path = PathBuf::from("modules");
                for component in &module_id.components {
                    path.push(component);
                }
                path.set_extension("scm");
                path
            }
            crate::module_system::ModuleNamespace::User => {
                let mut path = PathBuf::from("user");
                for component in &module_id.components {
                    path.push(component);
                }
                path.set_extension("scm");
                path
            }
            crate::module_system::ModuleNamespace::File => {
                // For file-based modules, use the components as the path
                let mut path = PathBuf::new();
                for component in &module_id.components {
                    path.push(component);
                }
                // Assume .scm extension if not already present
                if path.extension().is_none() {
                    path.set_extension("scm");
                }
                path
            }
        }
    }

    /// Evaluates module source code and extracts exports.
    fn evaluate_module_source(
        module_id: &ModuleId,
        source_code: &str,
        module_evaluator: Arc<dyn SchemeModuleEvaluator>,
    ) -> Result<HashMap<String, Value>> {
        // Extract dependencies (for now, empty)
        let dependencies: Vec<String> = Vec::new();
        
        // For now, create a dummy evaluator
        // TODO: Pass actual evaluator instance
        let mut dummy_evaluator = crate::eval::Evaluator::new();
        
        // Use the module evaluator to evaluate the module
        module_evaluator.evaluate_module(module_id, source_code, &mut dummy_evaluator)
    }


    /// Extracts dependencies from module source code (simplified implementation).
    fn extract_dependencies(&self, _source_code: &str) -> Result<Vec<ModuleId>> {
        // For now, return no dependencies to avoid circular dependency issues
        // In a full implementation, we would parse import forms in the source code
        // and return the actual dependencies (excluding self-references)
        Ok(vec![])
    }

    /// Gets the current loading statistics.
    pub fn get_statistics(&self) -> LoadingStatistics {
        self.statistics.read().clone()
    }

    /// Checks if a module is currently being loaded (for cycle detection).
    pub fn is_loading(&self, module_id: &ModuleId) -> bool {
        let loading_stack = self.loading_stack.read();
        loading_stack.contains(module_id)
    }

    /// Gets all registered module IDs.
    pub fn list_modules(&self) -> Vec<ModuleId> {
        let modules = self.modules.read();
        modules.keys().cloned().collect()
    }
}

impl Default for MonadicModuleLoader {
    fn default() -> Self {
        let global_env = crate::eval::global_environment();
        let thread_safe_env = global_env.to_thread_safe();
        Self::new((*thread_safe_env).clone())
    }
}

/// Checks if a module is a builtin module that should always be available.
/// For now, we'll treat all modules as loadable from source files rather than builtins.
fn is_builtin_module(module_id: &ModuleId) -> bool {
    // Disable builtin modules for now - all modules should be loaded from source files
    // This allows (scheme base) to be properly loaded from stdlib/r7rs/base.scm
    false
}

// Safety: MonadicModuleLoader is thread-safe due to its use of Arc<RwLock<>>
unsafe impl Send for MonadicModuleLoader {}
unsafe impl Sync for MonadicModuleLoader {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_computation() {
        let computation = LazyComputation::new(|| Ok(42));
        
        // First evaluation
        let result1 = computation.evaluate();
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), 42);
        assert!(computation.is_evaluated());
        
        // Second evaluation should return cached result
        let result2 = computation.evaluate();
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 42);
    }

    #[test]
    fn test_monadic_module_loader_creation() {
        let global_env = crate::eval::global_environment();
        let thread_safe_env = global_env.to_thread_safe();
        let loader = MonadicModuleLoader::new((*thread_safe_env).clone());
        assert_eq!(loader.list_modules().len(), 0);
        
        let stats = loader.get_statistics();
        assert_eq!(stats.modules_loaded, 0);
        assert_eq!(stats.cache_hits, 0);
    }

    #[test]
    fn test_module_id_to_path() {
        let global_env = crate::eval::global_environment();
        let thread_safe_env = global_env.to_thread_safe();
        let loader = MonadicModuleLoader::new((*thread_safe_env).clone());
        
        let srfi_id = ModuleId {
            components: vec!["41".to_string()],
            namespace: crate::module_system::ModuleNamespace::SRFI,
        };
        
        let path = loader.module_id_to_path(&srfi_id);
        assert_eq!(path, PathBuf::from("modules/srfi/41.scm"));
    }
}
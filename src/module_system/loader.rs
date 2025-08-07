//! Module loading and discovery infrastructure.
//!
//! Handles the loading of modules from various sources:
//! - Built-in modules compiled into the binary
//! - Standard library modules from the stdlib directory
//! - User modules from configurable search paths
//! - File-based modules with explicit paths

use super::{Module, ModuleId, ModuleNamespace, ModuleError, ModuleProvider, ModuleSource, ModuleMetadata};
use crate::diagnostics::{Error, Result};
use crate::runtime::LibraryPathResolver;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Module loader responsible for finding and loading modules.
pub struct ModuleLoader {
    /// Search paths for user modules
    search_paths: Vec<PathBuf>,
    /// Built-in module providers
    builtin_providers: HashMap<String, Box<dyn ModuleProvider>>,
    /// Standard library path
    stdlib_path: Option<PathBuf>,
    /// Library path resolver
    library_resolver: LibraryPathResolver,
}

impl std::fmt::Debug for ModuleLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleLoader")
            .field("search_paths", &self.search_paths)
            .field("builtin_providers", &format!("{} providers", self.builtin_providers.len()))
            .field("stdlib_path", &self.stdlib_path)
            .field("library_resolver", &self.library_resolver)
            .finish()
    }
}

impl ModuleLoader {
    /// Creates a new module loader with default configuration.
    pub fn new() -> Result<Self> {
        let library_resolver = LibraryPathResolver::new()?;
        let mut loader = Self {
            search_paths: Vec::new(),
            builtin_providers: HashMap::new(),
            stdlib_path: library_resolver.primary_lib_dir().map(|p| p.to_path_buf()),
            library_resolver,
        };
        
        // Initialize search paths from library resolver
        loader.initialize_from_library_resolver();
        
        // Initialize built-in providers
        loader.register_builtin_providers();
        
        Ok(loader)
    }

    /// Loads a module by its identifier.
    pub fn load(&mut self, id: &ModuleId) -> Result<Module> {
        match id.namespace {
            ModuleNamespace::Builtin => self.load_builtin_module(id),
            ModuleNamespace::R7RS => self.load_r7rs_module(id),
            ModuleNamespace::SRFI => self.load_srfi_module(id),
            ModuleNamespace::User => self.load_user_module(id),
            ModuleNamespace::File => self.load_file_module(id),
        }
    }

    /// Adds a search path for user modules.
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Sets the standard library path.
    pub fn set_stdlib_path<P: AsRef<Path>>(&mut self, path: P) {
        self.stdlib_path = Some(path.as_ref().to_path_buf());
    }

    /// Registers a built-in module provider.
    pub fn register_builtin_provider(&mut self, namespace: String, provider: Box<dyn ModuleProvider>) {
        self.builtin_providers.insert(namespace, provider);
    }

    /// Loads a built-in Lambdust module.
    fn load_builtin_module(&self, id: &ModuleId) -> Result<Module> {
        if id.components.is_empty() {
            return Err(Box::new(Error::from(ModuleError::InvalidDefinition(
                "Built-in module name cannot be empty".to_string()
            )));
        }

        let module_name = &id.components[0];
        
        // Check if we have a built-in provider for this module
        if let Some(provider) = self.builtin_providers.get(module_name) {
            return provider.get_module(id);
        }

        // Otherwise, try to load from stdlib
        self.load_from_stdlib_path(id, "modules")
    }

    /// Loads an R7RS standard library module.
    fn load_r7rs_module(&self, id: &ModuleId) -> Result<Module> {
        if id.components.is_empty() {
            return Err(Box::new(Error::from(ModuleError::InvalidDefinition(
                "R7RS module name cannot be empty".to_string()
            )));
        }

        self.load_from_stdlib_path(id, "r7rs")
    }

    /// Loads a SRFI module.
    /// Supports both single SRFI imports and multi-SRFI imports.
    fn load_srfi_module(&self, id: &ModuleId) -> Result<Module> {
        if id.components.is_empty() {
            return Err(Box::new(Error::from(ModuleError::InvalidDefinition(
                "SRFI module name cannot be empty".to_string()
            )));
        }

        if id.components.len() == 1 {
            // Single SRFI: (srfi 1) -> load from modules/srfi/1.scm
            self.load_single_srfi(id, &id.components[0])
        } else {
            // Multiple SRFIs: (srfi (1 13 14)) -> combine multiple SRFI modules
            self.load_multiple_srfis(id)
        }
    }

    /// Loads a single SRFI module.
    fn load_single_srfi(&self, id: &ModuleId, srfi_number: &str) -> Result<Module> {
        let srfi_filename = format!("{}.scm", srfi_number);
        
        // Try using the library resolver first for modules/srfi/
        match self.library_resolver.resolve_library_file("modules/srfi", &srfi_filename) {
            Ok(module_path) => self.load_from_file(id, &module_path),
            Err(_) => {
                // Fallback to old method for backward compatibility
                let stdlib_path = self.stdlib_path.as_ref().ok_or_else(|| {
                    Error::from(ModuleError::NotFound(id.clone()))
                })?;

                let module_path = stdlib_path.join("modules").join("srfi").join(&srfi_filename);

                if !module_path.exists() {
                    return Err(Box::new(Error::from(ModuleError::NotFound(id.clone()).boxed())));
                }

                self.load_from_file(id, &module_path)
            }
        }
    }

    /// Loads and combines multiple SRFI modules.
    fn load_multiple_srfis(&self, id: &ModuleId) -> Result<Module> {
        let mut combined_exports = HashMap::new();
        let mut all_dependencies = Vec::new();
        let mut metadata = ModuleMetadata::default();
        
        // Load each SRFI individually
        for srfi_number in &id.components {
            let single_srfi_id = super::name::srfi_module(
                srfi_number.parse::<u32>().map_err(|_| {
                    Error::from(ModuleError::InvalidDefinition(
                        format!("Invalid SRFI number: {}", srfi_number)
                    ))
                })?
            );
            
            let srfi_module = self.load_single_srfi(&single_srfi_id, srfi_number)?;
            
            // Combine exports (later SRFIs override earlier ones in case of conflicts)
            for (name, value) in srfi_module.exports {
                combined_exports.insert(name, value);
            }
            
            // Combine dependencies
            all_dependencies.extend(srfi_module.dependencies);
            
            // Update metadata (combine descriptions)
            if let Some(desc) = srfi_module.metadata.description {
                if let Some(existing_desc) = &metadata.description {
                    metadata.description = Some(format!("{}, {}", existing_desc, desc));
                } else {
                    metadata.description = Some(desc);
                }
            }
        }
        
        // Remove duplicate dependencies
        all_dependencies.sort();
        all_dependencies.dedup();
        
        // Create combined module
        Ok(Module {
            id: id.clone()),
            exports: combined_exports,
            dependencies: all_dependencies,
            source: Some(ModuleSource::Source(
                format!("Combined SRFI modules: {}", id.components.join(", "))
            )),
            metadata,
        })
    }

    /// Loads a user-defined module.
    fn load_user_module(&self, id: &ModuleId) -> Result<Module> {
        let module_filename = format!("{}.scm", id.components.join("-"));
        
        // Search in all configured search paths
        for search_path in &self.search_paths {
            let module_path = search_path.join(&module_filename);
            if module_path.exists() {
                return self.load_from_file(id, &module_path);
            }
        }

        // Also check stdlib user directory
        if let Some(stdlib_path) = &self.stdlib_path {
            let user_path = stdlib_path.join("user").join(&module_filename);
            if user_path.exists() {
                return self.load_from_file(id, &user_path);
            }
        }

        Err(Box::new(Error::from(ModuleError::NotFound(id.clone()).boxed())))
    }

    /// Loads a file-based module.
    fn load_file_module(&self, id: &ModuleId) -> Result<Module> {
        if id.components.len() != 1 {
            return Err(Box::new(Error::from(ModuleError::InvalidDefinition(
                "File module must specify exactly one path".to_string()
            )));
        }

        let file_path = PathBuf::from(&id.components[0]);
        if !file_path.exists() {
            return Err(Box::new(Error::from(ModuleError::NotFound(id.clone()).boxed())));
        }

        self.load_from_file(id, &file_path)
    }

    /// Loads a module from a specific file path.
    fn load_from_file(&self, id: &ModuleId, path: &Path) -> Result<Module> {
        let _source_code = fs::read_to_string(path).map_err(|e| {
            Error::io_error(format!("Failed to read module file {}: {}", path.display(), e))
        })?;

        // TODO: Parse and compile the module source code
        // For now, return a placeholder module
        Ok(Module {
            id: id.clone()),
            exports: HashMap::new(),
            dependencies: Vec::new(),
            source: Some(ModuleSource::File(path.to_path_buf())),
            metadata: ModuleMetadata::default(),
        })
    }

    /// Loads a module from the standard library path.
    fn load_from_stdlib_path(&self, id: &ModuleId, subdir: &str) -> Result<Module> {
        let module_filename = format!("{}.scm", id.components.join("-"));
        
        // Try using the library resolver first
        match self.library_resolver.resolve_library_file(subdir, &module_filename) {
            Ok(module_path) => self.load_from_file(id, &module_path),
            Err(_) => {
                // Fallback to old method for backward compatibility
                let stdlib_path = self.stdlib_path.as_ref().ok_or_else(|| {
                    Error::from(ModuleError::NotFound(id.clone()))
                })?;

                let module_path = stdlib_path.join(subdir).join(&module_filename);

                if !module_path.exists() {
                    return Err(Box::new(Error::from(ModuleError::NotFound(id.clone()).boxed())));
                }

                self.load_from_file(id, &module_path)
            }
        }
    }

    /// Initializes search paths from the library path resolver.
    fn initialize_from_library_resolver(&mut self) {
        // Copy search paths from library resolver
        for path in self.library_resolver.search_paths() {
            self.search_paths.push(path.clone());
        }

        // Add current directory if not already included
        if let Ok(current_dir) = std::env::current_dir() {
            if !self.search_paths.contains(&current_dir) {
                self.search_paths.push(current_dir);
            }
        }

        // Add home directory module path if it exists
        if let Some(home_dir) = dirs::home_dir() {
            let home_modules = home_dir.join(".lambdust").join("modules");
            if home_modules.exists() && !self.search_paths.contains(&home_modules) {
                self.search_paths.push(home_modules);
            }
        }
    }

    /// Registers built-in module providers.
    fn register_builtin_providers(&mut self) {
        // Register providers for built-in modules
        self.builtin_providers.insert(
            "string".to_string(),
            Box::new(BuiltinStringModuleProvider)
        );
        
        self.builtin_providers.insert(
            "list".to_string(),
            Box::new(BuiltinListModuleProvider)
        );
        
        // Add more built-in providers as needed
    }

    /// Lists all discoverable modules.
    pub fn discover_modules(&self) -> Vec<ModuleId> {
        let mut modules = Vec::new();

        // Discover built-in modules
        for provider_name in self.builtin_providers.keys() {
            modules.push(ModuleId {
                components: vec![provider_name.clone())],
                namespace: ModuleNamespace::Builtin,
            });
        }

        // Discover modules in search paths
        for search_path in &self.search_paths {
            if let Ok(entries) = fs::read_dir(search_path) {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".scm") {
                            let module_name = filename.strip_suffix(".scm").unwrap();
                            modules.push(ModuleId {
                                components: vec![module_name.to_string()],
                                namespace: ModuleNamespace::User,
                            });
                        }
                    }
                }
            }
        }

        // Discover R7RS modules using library resolver
        let r7rs_files = self.library_resolver.find_library_files("r7rs", ".scm");
        for file_path in r7rs_files {
            if let Some(filename) = file_path.file_stem().and_then(|s| s.to_str()) {
                modules.push(ModuleId {
                    components: vec![filename.to_string()],
                    namespace: ModuleNamespace::R7RS,
                });
            }
        }

        modules
    }

    /// Gets the library path resolver.
    pub fn library_resolver(&self) -> &LibraryPathResolver {
        &self.library_resolver
    }

    /// Validates the library setup using the library resolver.
    pub fn validate_library_setup(&self) -> Result<crate::runtime::LibraryValidationReport> {
        self.library_resolver.validate_library_setup()
    }
}

/// Built-in module provider for string operations.
struct BuiltinStringModuleProvider;

impl ModuleProvider for BuiltinStringModuleProvider {
    fn get_module(&self, id: &ModuleId) -> Result<Module> {
        if id.components.len() != 1 || id.components[0] != "string" {
            return Err(Box::new(Error::from(ModuleError::NotFound(id.clone()).boxed())));
        }

        // Create string module with exports
        let exports = HashMap::new();
        
        // Add string operations (these would be implemented as proper procedures)
        // For now, we'll add placeholder entries
        
        // String predicates and operations will be added here
        // exports.insert("string?".to_string(), Value::Primitive(...));
        // exports.insert("string-length".to_string(), Value::Primitive(...));
        // etc.

        Ok(Module {
            id: id.clone()),
            exports,
            dependencies: Vec::new(),
            source: Some(ModuleSource::Builtin),
            metadata: ModuleMetadata {
                description: Some("String manipulation operations".to_string()),
                ..Default::default()
            },
        })
    }

    fn has_module(&self, id: &ModuleId) -> bool {
        id.namespace == ModuleNamespace::Builtin 
            && id.components.len() == 1 
            && id.components[0] == "string"
    }

    fn list_modules(&self) -> Vec<ModuleId> {
        vec![ModuleId {
            components: vec!["string".to_string()],
            namespace: ModuleNamespace::Builtin,
        }]
    }
}

/// Built-in module provider for list operations.
struct BuiltinListModuleProvider;

impl ModuleProvider for BuiltinListModuleProvider {
    fn get_module(&self, id: &ModuleId) -> Result<Module> {
        if id.components.len() != 1 || id.components[0] != "list" {
            return Err(Box::new(Error::from(ModuleError::NotFound(id.clone()).boxed())));
        }

        let exports = HashMap::new();
        
        // List operations will be added here
        // exports.insert("list?".to_string(), Value::Primitive(...));
        // exports.insert("length".to_string(), Value::Primitive(...));
        // etc.

        Ok(Module {
            id: id.clone()),
            exports,
            dependencies: Vec::new(),
            source: Some(ModuleSource::Builtin),
            metadata: ModuleMetadata {
                description: Some("List processing operations".to_string()),
                ..Default::default()
            },
        })
    }

    fn has_module(&self, id: &ModuleId) -> bool {
        id.namespace == ModuleNamespace::Builtin 
            && id.components.len() == 1 
            && id.components[0] == "list"
    }

    fn list_modules(&self) -> Vec<ModuleId> {
        vec![ModuleId {
            components: vec!["list".to_string()],
            namespace: ModuleNamespace::Builtin,
        }]
    }
}

// External dependency for home directory detection
#[cfg(not(test))]
mod dirs {
    use std::path::PathBuf;
    
    pub fn home_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

// Mock implementation for tests
#[cfg(test)]
mod dirs {
    use std::path::PathBuf;
    
    pub fn home_dir() -> Option<PathBuf> {
        Some(PathBuf::from("/tmp/test-home"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::name;

    #[test]
    fn test_module_loader_creation() {
        let loader = ModuleLoader::new();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_builtin_string_module() {
        let provider = BuiltinStringModuleProvider;
        let id = name::builtin_module("string");
        
        assert!(provider.has_module(&id));
        let module = provider.get_module(&id);
        assert!(module.is_ok());
    }

    #[test]
    fn test_search_path_management() {
        let mut loader = ModuleLoader::new().unwrap();
        let test_path = PathBuf::from("/test/path");
        
        loader.add_search_path(&test_path);
        assert!(loader.search_paths.contains(&test_path));
    }
}
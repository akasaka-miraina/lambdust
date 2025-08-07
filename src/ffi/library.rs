#![allow(unused_variables)]
//! Dynamic library loading and management for FFI operations.
//!
//! This module provides comprehensive support for loading, managing, and interacting
//! with native libraries across different platforms (.so, .dll, .dylib).

use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use libloading::{Library, Symbol};
use crate::diagnostics::Error;

/// Platform-specific library extensions
#[cfg(target_os = "windows")]
const LIBRARY_EXTENSION: &str = "dll";
#[cfg(target_os = "macos")]
const LIBRARY_EXTENSION: &str = "dylib";
#[cfg(target_os = "linux")]
const LIBRARY_EXTENSION: &str = "so";

/// Errors that can occur during library operations
#[derive(Debug, Clone)]
pub enum LibraryError {
    /// Library not found
    NotFound(PathBuf),
    /// Failed to load library
    LoadFailed {
        path: PathBuf,
        reason: String,
    },
    /// Symbol not found in library
    SymbolNotFound {
        library: String,
        symbol: String,
    },
    /// Library already loaded
    AlreadyLoaded(String),
    /// Library has active references and cannot be unloaded
    HasActiveReferences(String),
    /// Invalid library handle
    InvalidHandle(String),
    /// Platform-specific error
    PlatformError(String),
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::NotFound(path) => {
                write!(f, "Library not found: {}", path.display())
            }
            LibraryError::LoadFailed { path, reason } => {
                write!(f, "Failed to load library '{}': {}", path.display(), reason)
            }
            LibraryError::SymbolNotFound { library, symbol } => {
                write!(f, "Symbol '{}' not found in library '{}'", symbol, library)
            }
            LibraryError::AlreadyLoaded(name) => {
                write!(f, "Library '{}' is already loaded", name)
            }
            LibraryError::HasActiveReferences(name) => {
                write!(f, "Library '{}' has active references and cannot be unloaded", name)
            }
            LibraryError::InvalidHandle(name) => {
                write!(f, "Invalid library handle for '{}'", name)
            }
            LibraryError::PlatformError(msg) => {
                write!(f, "Platform-specific error: {}", msg)
            }
        }
    }
}

impl std::error::Error for LibraryError {}

impl From<LibraryError> for Error {
    fn from(lib_error: LibraryError) -> Self {
        Error::runtime_error(lib_error.to_string(), None)
    }
}

/// Handle to a loaded library with reference counting
#[derive(Debug)]
pub struct LibraryHandle {
    /// Internal library handle
    library: Arc<Library>,
    /// Library name/identifier
    name: String,
    /// Path where library was loaded from
    path: PathBuf,
    /// When the library was loaded
    loaded_at: SystemTime,
    /// Number of active references
    ref_count: Arc<RwLock<usize>>,
    /// Cached symbols
    symbol_cache: RwLock<HashMap<String, usize>>, // usize as raw pointer
}

impl LibraryHandle {
    /// Create a new library handle
    fn new(library: Library, name: String, path: PathBuf) -> Self {
        Self {
            library: Arc::new(library),
            name,
            path,
            loaded_at: SystemTime::now(),
            ref_count: Arc::new(RwLock::new(1)),
            symbol_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get library name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get library path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get load time
    pub fn loaded_at(&self) -> SystemTime {
        self.loaded_at
    }

    /// Get current reference count
    pub fn ref_count(&self) -> usize {
        *self.ref_count.read().unwrap()
    }

    /// Increment reference count
    pub fn add_ref(&self) {
        let mut count = self.ref_count.write().unwrap();
        *count += 1;
    }

    /// Decrement reference count
    pub fn release_ref(&self) -> usize {
        let mut count = self.ref_count.write().unwrap();
        if *count > 0 {
            *count -= 1;
        }
        *count
    }

    /// Get a symbol from the library with caching
    pub fn get_symbol<T>(&self, name: &str) -> std::result::Result<Symbol<'_, T>, LibraryError> {
        // Check cache first
        {
            let cache = self.symbol_cache.read().unwrap();
            if let Some(&addr) = cache.get(name) {
                // Safety: We're casting back to the symbol type, this is inherently unsafe
                // but necessary for FFI operations
                unsafe {
                    return Ok(std::mem::transmute_copy(&addr));
                }
            }
        }

        // Load symbol
        let c_name = CString::new(name)
            .map_err(|_| LibraryError::SymbolNotFound {
                library: self.name.clone()),
                symbol: name.to_string(),
            })?;

        let symbol: Symbol<'_, T> = unsafe {
            self.library.get(c_name.as_bytes())
                .map_err(|_e| LibraryError::SymbolNotFound {
                    library: self.name.clone()),
                    symbol: name.to_string(),
                })?
        };

        // Cache the symbol address
        {
            let mut cache = self.symbol_cache.write().unwrap();
            // Safety: Storing raw function pointer address for caching
            unsafe {
                let addr: usize = std::mem::transmute_copy(&symbol);
                cache.insert(name.to_string(), addr);
            }
        }

        Ok(symbol)
    }

    /// Check if a symbol exists in the library
    pub fn has_symbol(&self, name: &str) -> bool {
        // Check cache first
        {
            let cache = self.symbol_cache.read().unwrap();
            if cache.contains_key(name) {
                return true;
            }
        }

        // Try to load the symbol
        let c_name = match CString::new(name) {
            Ok(name) => name,
            Err(_) => return false,
        };

        unsafe {
            self.library.get::<*const ()>(c_name.as_bytes()).is_ok()
        }
    }

    /// List all exported symbols (platform dependent)
    pub fn list_symbols(&self) -> Vec<String> {
        // This is a simplified implementation
        // In practice, this would require platform-specific code to parse
        // the library's symbol table
        vec![]
    }
}

impl Clone for LibraryHandle {
    fn clone(&self) -> Self {
        self.add_ref();
        Self {
            library: Arc::clone(&self.library),
            name: self.name.clone()),
            path: self.path.clone()),
            loaded_at: self.loaded_at,
            ref_count: Arc::clone(&self.ref_count),
            symbol_cache: RwLock::new(HashMap::new()), // Fresh cache for clone
        }
    }
}

impl Drop for LibraryHandle {
    fn drop(&mut self) {
        self.release_ref();
    }
}

/// Library search path configuration
#[derive(Debug, Clone)]
pub struct LibrarySearchConfig {
    /// Additional search paths
    pub search_paths: Vec<PathBuf>,
    /// Whether to search system paths
    pub use_system_paths: bool,
    /// Whether to search current directory
    pub use_current_dir: bool,
    /// Custom library prefixes to try
    pub prefixes: Vec<String>,
}

impl Default for LibrarySearchConfig {
    fn default() -> Self {
        Self {
            search_paths: vec![],
            use_system_paths: true,
            use_current_dir: true,
            prefixes: vec!["lib".to_string(), "".to_string()],
        }
    }
}

/// Dynamic library manager
#[derive(Debug)]
pub struct LibraryManager {
    /// Loaded libraries
    libraries: RwLock<HashMap<String, LibraryHandle>>,
    /// Search configuration
    search_config: RwLock<LibrarySearchConfig>,
    /// Load statistics
    stats: RwLock<LibraryStats>,
    /// Dependency graph (for unload ordering)
    dependencies: RwLock<HashMap<String, Vec<String>>>,
}

/// Library loading and usage statistics
#[derive(Debug, Default, Clone)]
pub struct LibraryStats {
    /// Total number of libraries loaded
    pub total_loaded: usize,
    /// Currently loaded libraries
    pub currently_loaded: usize,
    /// Total symbol lookups
    pub symbol_lookups: u64,
    /// Successful symbol lookups
    pub successful_lookups: u64,
    /// Failed symbol lookups
    pub failed_lookups: u64,
}

impl Default for LibraryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LibraryManager {
    /// Create a new library manager
    pub fn new() -> Self {
        Self {
            libraries: RwLock::new(HashMap::new()),
            search_config: RwLock::new(LibrarySearchConfig::default()),
            stats: RwLock::new(LibraryStats::default()),
            dependencies: RwLock::new(HashMap::new()),
        }
    }

    /// Set search configuration
    pub fn set_search_config(&self, config: LibrarySearchConfig) {
        let mut search_config = self.search_config.write().unwrap();
        *search_config = config;
    }

    /// Add a search path
    pub fn add_search_path<P: AsRef<Path>>(&self, path: P) {
        let mut config = self.search_config.write().unwrap();
        config.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Find library in search paths
    fn find_library(&self, name: &str) -> Option<PathBuf> {
        let config = self.search_config.read().unwrap();
        
        // Try different combinations of prefix + name + extension
        let extensions = vec![LIBRARY_EXTENSION];
        let prefixes = &config.prefixes;
        
        let mut candidates = Vec::new();
        
        for prefix in prefixes {
            for ext in &extensions {
                let filename = if prefix.is_empty() {
                    format!("{}.{}", name, ext)
                } else {
                    format!("{}{}.{}", prefix, name, ext)
                };
                candidates.push(filename);
            }
        }

        // Also try the exact name as given
        candidates.push(name.to_string());

        // Search in configured paths
        let mut search_paths = Vec::new();
        
        if config.use_current_dir {
            search_paths.push(PathBuf::from("."));
        }
        
        search_paths.extend(config.search_paths.iter().clone())());
        
        if config.use_system_paths {
            // Add platform-specific system paths
            #[cfg(unix)]
            {
                search_paths.extend(vec![
                    PathBuf::from("/usr/lib"),
                    PathBuf::from("/usr/local/lib"),
                    PathBuf::from("/lib"),
                ]);
            }
            
            #[cfg(windows)]
            {
                if let Ok(sys_dir) = std::env::var("SYSTEMROOT") {
                    search_paths.push(PathBuf::from(sys_dir).join("System32"));
                }
            }
        }

        // Try each candidate in each search path
        for path in &search_paths {
            for candidate in &candidates {
                let full_path = path.join(candidate);
                if full_path.exists() {
                    return Some(full_path);
                }
            }
        }

        None
    }

    /// Load a library
    pub fn load_library(&self, name: &str) -> std::result::Result<LibraryHandle, LibraryError> {
        // Check if already loaded
        {
            let libraries = self.libraries.read().unwrap();
            if let Some(handle) = libraries.get(name) {
                return Ok(handle.clone());
            }
        }

        // Find the library file
        let library_path = if Path::new(name).is_absolute() || name.contains('/') || name.contains('\\') {
            PathBuf::from(name)
        } else {
            self.find_library(name).ok_or_else(|| LibraryError::NotFound(PathBuf::from(name)))?
        };

        // Load the library
        let library = unsafe {
            Library::new(&library_path)
                .map_err(|e| LibraryError::LoadFailed {
                    path: library_path.clone()),
                    reason: e.to_string(),
                })?
        };

        // Create handle
        let handle = LibraryHandle::new(library, name.to_string(), library_path);

        // Store in registry
        {
            let mut libraries = self.libraries.write().unwrap();
            libraries.insert(name.to_string(), handle.clone());
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_loaded += 1;
            stats.currently_loaded = self.libraries.read().unwrap().len();
        }

        Ok(handle)
    }

    /// Unload a library
    pub fn unload_library(&self, name: &str) -> std::result::Result<(), LibraryError> {
        let handle = {
            let mut libraries = self.libraries.write().unwrap();
            libraries.remove(name)
        };

        if let Some(handle) = handle {
            if handle.ref_count() > 1 {
                return Err(LibraryError::HasActiveReferences(name.to_string()));
            }

            // Update statistics
            let mut stats = self.stats.write().unwrap();
            stats.currently_loaded = self.libraries.read().unwrap().len();
            
            Ok(())
        } else {
            Err(LibraryError::NotFound(PathBuf::from(name)))
        }
    }

    /// Get a library handle
    pub fn get_library(&self, name: &str) -> Option<LibraryHandle> {
        let libraries = self.libraries.read().unwrap();
        libraries.get(name).clone())()
    }

    /// List loaded libraries
    pub fn list_libraries(&self) -> Vec<String> {
        let libraries = self.libraries.read().unwrap();
        libraries.keys().clone())().collect()
    }

    /// Get library statistics
    pub fn stats(&self) -> LibraryStats {
        self.stats.read().unwrap().clone())
    }

    /// Unload all libraries
    pub fn unload_all(&self) -> std::result::Result<(), Vec<LibraryError>> {
        let names: Vec<String> = {
            let libraries = self.libraries.read().unwrap();
            libraries.keys().clone())().collect()
        };

        let mut errors = Vec::new();
        for name in names {
            if let Err(e) = self.unload_library(&name) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Add a dependency relationship
    pub fn add_dependency(&self, dependent: &str, dependency: &str) {
        let mut deps = self.dependencies.write().unwrap();
        deps.entry(dependent.to_string())
            .or_insert_with(Vec::new)
            .push(dependency.to_string());
    }

    /// Get dependencies for a library
    pub fn get_dependencies(&self, library: &str) -> Vec<String> {
        let deps = self.dependencies.read().unwrap();
        deps.get(library).clone())().unwrap_or_default()
    }
}

/// Convenience functions for library management
impl LibraryManager {
    /// Load symbol from library
    pub fn load_symbol<T>(&self, library_name: &str, symbol_name: &str) 
        -> std::result::Result<*const T, LibraryError> {
        let handle = self.load_library(library_name)?;
        let symbol = handle.get_symbol::<T>(symbol_name)?;
        Ok(unsafe { std::mem::transmute(symbol.into_raw()) })
    }

    /// Load and get symbol in one call
    /// Note: The returned symbol is tied to the library lifetime
    pub fn get_symbol<T>(&self, library_name: &str, symbol_name: &str) 
        -> std::result::Result<*const T, LibraryError> {
        let handle = self.load_library(library_name)?;
        
        // Update symbol lookup statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.symbol_lookups += 1;
        }

        match handle.get_symbol::<T>(symbol_name) {
            Ok(symbol) => {
                let mut stats = self.stats.write().unwrap();
                stats.successful_lookups += 1;
                Ok(unsafe { std::mem::transmute(symbol.into_raw()) })
            }
            Err(e) => {
                let mut stats = self.stats.write().unwrap();
                stats.failed_lookups += 1;
                Err(LibraryError::SymbolNotFound {
                    library: library_name.to_string(),
                    symbol: symbol_name.to_string(),
                })
            }
        }
    }
}

lazy_static::lazy_static! {
    /// Global library manager instance
    pub static ref GLOBAL_LIBRARY_MANAGER: LibraryManager = LibraryManager::new();
}

/// Convenience functions for global library manager
pub fn load_library(name: &str) -> std::result::Result<LibraryHandle, LibraryError> {
    GLOBAL_LIBRARY_MANAGER.load_library(name)
}

pub fn unload_library(name: &str) -> std::result::Result<(), LibraryError> {
    GLOBAL_LIBRARY_MANAGER.unload_library(name)
}

pub fn get_library(name: &str) -> Option<LibraryHandle> {
    GLOBAL_LIBRARY_MANAGER.get_library(name)
}

pub fn load_symbol<T>(library_name: &str, symbol_name: &str) 
    -> std::result::Result<*const T, LibraryError> {
    GLOBAL_LIBRARY_MANAGER.load_symbol(library_name, symbol_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_library_search_config() {
        let config = LibrarySearchConfig::default();
        assert!(config.use_system_paths);
        assert!(config.use_current_dir);
        assert!(!config.prefixes.is_empty());
    }

    #[test]
    fn test_library_manager_creation() {
        let manager = LibraryManager::new();
        let stats = manager.stats();
        assert_eq!(stats.currently_loaded, 0);
        assert_eq!(stats.total_loaded, 0);
    }

    #[test]
    fn test_library_path_search() {
        let manager = LibraryManager::new();
        
        // This should not find a non-existent library
        let result = manager.find_library("nonexistent_library_12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_search_path_addition() {
        let manager = LibraryManager::new();
        manager.add_search_path("/custom/path");
        
        let config = manager.search_config.read().unwrap();
        assert!(config.search_paths.contains(&PathBuf::from("/custom/path")));
    }
}
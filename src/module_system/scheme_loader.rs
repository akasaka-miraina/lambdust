//! Scheme library loading and compilation infrastructure.
//!
//! This module provides the core infrastructure needed to support a "minimal Rust + rich Scheme libraries"
//! architecture by enabling automatic loading, compilation, and caching of Scheme-based standard library modules.

use super::{Module, ModuleId, ModuleNamespace, ModuleError, ModuleSource, ModuleMetadata};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::{Value, ThreadSafeEnvironment};
use crate::parser::Parser;
use crate::lexer::Lexer;
use crate::ast::Expr;
use crate::runtime::{GlobalEnvironmentManager, LibraryPathResolver};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, Duration};
use std::sync::{Arc, RwLock, Mutex};

/// Type alias for reload callback collections
type ReloadCallbacks = Arc<Mutex<Vec<Box<dyn Fn(&ModuleId) + Send + Sync>>>>;

/// A compiled Scheme library ready for loading.
#[derive(Debug, Clone)]
pub struct CompiledSchemeLibrary {
    /// Module metadata
    pub module: Module,
    /// Compiled bytecode (placeholder for future optimization)
    pub bytecode: Option<Vec<u8>>,
    /// Compilation timestamp
    pub compiled_at: SystemTime,
    /// Source file modification time
    pub source_mtime: SystemTime,
    /// Library dependencies resolved during compilation
    pub resolved_dependencies: Vec<ModuleId>,
    /// Hot-reload generation (for development)
    pub reload_generation: u64,
}

/// Cache for compiled Scheme libraries with invalidation support.
#[derive(Debug)]
pub struct SchemeLibraryCache {
    /// Cached compiled libraries
    libraries: Arc<RwLock<HashMap<ModuleId, CompiledSchemeLibrary>>>,
    /// File modification times for cache invalidation
    #[allow(dead_code)]
    file_mtimes: Arc<RwLock<HashMap<PathBuf, SystemTime>>>,
    /// Cache statistics
    cache_stats: Arc<Mutex<CacheStatistics>>,
    /// Hot-reload generation counter
    reload_generation: Arc<std::sync::atomic::AtomicU64>,
}

/// Cache performance statistics.
#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of cache invalidations
    pub invalidations: u64,
    /// Number of compilations performed
    pub compilations: u64,
    /// Total time spent loading from cache
    pub load_time_total: Duration,
    /// Total time spent in compilation
    pub compile_time_total: Duration,
}

/// Scheme library loader with compilation and caching support.
#[derive(Debug)]
pub struct SchemeLibraryLoader {
    /// Library cache
    cache: SchemeLibraryCache,
    /// Global environment manager for accessing primitives
    global_env: Arc<GlobalEnvironmentManager>,
    /// Library search paths
    search_paths: Vec<PathBuf>,
    /// Bootstrap configuration
    bootstrap_config: BootstrapConfig,
    /// Development mode settings
    dev_mode: bool,
    /// Library path resolver
    library_resolver: Option<LibraryPathResolver>,
}

/// Bootstrap configuration for library loading.
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// Minimal Rust primitives to expose to Scheme
    pub essential_primitives: Vec<String>,
    /// Core Scheme libraries to load during bootstrap
    pub core_libraries: Vec<ModuleId>,
    /// Library loading order (for dependency resolution)
    pub load_order: Vec<ModuleId>,
    /// Whether to enable lazy loading
    pub lazy_loading: bool,
    /// Bootstrap timeout
    pub bootstrap_timeout: Duration,
}

/// Library compilation context with dependency tracking.
#[derive(Debug)]
pub struct CompilationContext {
    /// Current module being compiled
    pub current_module: ModuleId,
    /// Dependency chain (for circular dependency detection)
    pub dependency_chain: Vec<ModuleId>,
    /// Available primitive procedures
    pub available_primitives: HashMap<String, Value>,
    /// Compilation environment
    pub environment: Arc<ThreadSafeEnvironment>,
    /// Global environment manager
    pub global_env: Arc<GlobalEnvironmentManager>,
}

/// Hot-reload support for development.
pub struct HotReloadManager {
    /// File watchers for automatic reloading
    watchers: Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
    /// Reload callbacks
    reload_callbacks: ReloadCallbacks,
    /// Reload generation counter
    generation: Arc<std::sync::atomic::AtomicU64>,
}

impl Default for SchemeLibraryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemeLibraryCache {
    /// Creates a new library cache.
    pub fn new() -> Self {
        Self {
            libraries: Arc::new(RwLock::new(HashMap::new())),
            file_mtimes: Arc::new(RwLock::new(HashMap::new())),
            cache_stats: Arc::new(Mutex::new(CacheStatistics::default())),
            reload_generation: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Gets a compiled library from cache, checking for invalidation.
    pub fn get(&self, id: &ModuleId, source_path: Option<&Path>) -> Option<CompiledSchemeLibrary> {
        let start = SystemTime::now();
        
        // Check if cache entry exists and is valid
        {
            let libraries = self.libraries.read().unwrap();
            if let Some(library) = libraries.get(id) {
                // Check if source file has been modified
                if let Some(path) = source_path {
                    if let Ok(metadata) = fs::metadata(path) {
                        if let Ok(mtime) = metadata.modified() {
                            if mtime <= library.source_mtime {
                                // Cache hit - file not modified
                                self.record_cache_hit(start);
                                return Some(library.clone());
                            } else {
                                // File modified - invalidate cache entry
                                self.record_cache_invalidation();
                            }
                        }
                    }
                } else {
                    // No source path - assume valid (for built-in modules)
                    self.record_cache_hit(start);
                    return Some(library.clone());
                }
            }
        }
        
        // Cache miss
        self.record_cache_miss();
        None
    }

    /// Stores a compiled library in cache.
    pub fn store(&self, id: ModuleId, library: CompiledSchemeLibrary) {
        let mut libraries = self.libraries.write().unwrap();
        libraries.insert(id, library);
    }

    /// Invalidates cache entries for a specific module.
    pub fn invalidate(&self, id: &ModuleId) {
        let mut libraries = self.libraries.write().unwrap();
        libraries.remove(id);
        self.record_cache_invalidation();
    }

    /// Clears the entire cache.
    pub fn clear(&self) {
        let mut libraries = self.libraries.write().unwrap();
        libraries.clear();
        
        let mut stats = self.cache_stats.lock().unwrap();
        stats.invalidations += 1;
    }

    /// Gets cache statistics.
    pub fn statistics(&self) -> CacheStatistics {
        let stats = self.cache_stats.lock().unwrap();
        (*stats).clone()
    }

    /// Records a cache hit.
    fn record_cache_hit(&self, start: SystemTime) {
        let mut stats = self.cache_stats.lock().unwrap();
        stats.hits += 1;
        if let Ok(duration) = start.elapsed() {
            stats.load_time_total += duration;
        }
    }

    /// Records a cache miss.
    fn record_cache_miss(&self) {
        let mut stats = self.cache_stats.lock().unwrap();
        stats.misses += 1;
    }

    /// Records a cache invalidation.
    fn record_cache_invalidation(&self) {
        let mut stats = self.cache_stats.lock().unwrap();
        stats.invalidations += 1;
    }

    /// Records a compilation.
    fn record_compilation(&self, duration: Duration) {
        let mut stats = self.cache_stats.lock().unwrap();
        stats.compilations += 1;
        stats.compile_time_total += duration;
    }
}

impl SchemeLibraryLoader {
    /// Creates a new Scheme library loader.
    pub fn new(global_env: Arc<GlobalEnvironmentManager>) -> Result<Self> {
        let bootstrap_config = BootstrapConfig::new_default();
        let library_resolver = LibraryPathResolver::new().ok();
        
        Ok(Self {
            cache: SchemeLibraryCache::new(),
            global_env,
            search_paths: Vec::new(),
            bootstrap_config,
            dev_mode: false,
            library_resolver,
        })
    }

    /// Creates a loader with custom bootstrap configuration.
    pub fn with_bootstrap_config(
        global_env: Arc<GlobalEnvironmentManager>,
        config: BootstrapConfig,
    ) -> Result<Self> {
        let library_resolver = LibraryPathResolver::new().ok();
        
        Ok(Self {
            cache: SchemeLibraryCache::new(),
            global_env,
            search_paths: Vec::new(),
            bootstrap_config: config,
            dev_mode: false,
            library_resolver,
        })
    }

    /// Enables development mode with hot-reloading.
    pub fn enable_dev_mode(&mut self) {
        self.dev_mode = true;
    }

    /// Adds a search path for Scheme libraries.
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Loads a Scheme library, compiling if necessary.
    pub fn load_library(&self, id: &ModuleId) -> Result<CompiledSchemeLibrary> {
        // Check cache first
        let source_path = self.find_library_source(id)?;
        if let Some(library) = self.cache.get(id, source_path.as_deref()) {
            return Ok(library);
        }

        // Compile the library
        self.compile_library(id, source_path.as_deref())
    }

    /// Compiles a Scheme library from source.
    pub fn compile_library(&self, id: &ModuleId, source_path: Option<&Path>) -> Result<CompiledSchemeLibrary> {
        let start = SystemTime::now();
        
        // Read source code
        let (source_code, source_mtime) = if let Some(path) = source_path {
            let code = fs::read_to_string(path).map_err(|e| {
                Error::io_error(format!("Failed to read library source {}: {}", path.display(), e))
            })?;
            let mtime = fs::metadata(path)
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| SystemTime::now());
            (code, mtime)
        } else {
            return Err(Box::new(Error::from(ModuleError::NotFound(id.clone()))));
        };

        // Create compilation context
        let context = self.create_compilation_context(id.clone())?;
        
        // Parse the source code
        let mut lexer = Lexer::new(&source_code, Some("<library>"));
        let tokens = lexer.tokenize().map_err(|e| {
            Error::parse_error(format!("Lexing error in {}: {}", id.components.join("/"), e), Span::new(0, 0))
        })?;

        let mut parser = Parser::new(tokens);
        
        // Parse as a complete program rather than individual expressions
        // This allows proper context-sensitive parsing of dots in parameter lists
        let program = parser.parse().map_err(|e| {
            Error::parse_error(format!("Parsing error in {}: {}", id.components.join("/"), e), Span::new(0, 0))
        })?;
        
        let exprs: Vec<Expr> = program.expressions.into_iter().map(|spanned| spanned.inner).collect();

        // Compile the module
        let module = self.compile_module_expressions(id, exprs, &context)?;
        
        // Create compiled library
        let library = CompiledSchemeLibrary {
            module,
            bytecode: None, // TODO: Implement bytecode compilation for performance  
            compiled_at: SystemTime::now(),
            source_mtime,
            resolved_dependencies: context.dependency_chain.clone(),
            reload_generation: self.cache.reload_generation.load(std::sync::atomic::Ordering::SeqCst),
        };

        // Cache the compiled library
        self.cache.store(id.clone(), library.clone());
        
        // Record compilation statistics
        if let Ok(duration) = start.elapsed() {
            self.cache.record_compilation(duration);
        }

        Ok(library)
    }

    /// Creates a compilation context for a module.
    fn create_compilation_context(&self, module_id: ModuleId) -> Result<CompilationContext> {
        // Create environment with essential primitives
        let environment = self.global_env.root_environment();
        
        // Collect available primitive procedures
        let mut available_primitives = HashMap::new();
        for primitive_name in &self.bootstrap_config.essential_primitives {
            if let Some(primitive_value) = environment.lookup(primitive_name) {
                available_primitives.insert(primitive_name.clone(), primitive_value);
            } else {
                // Primitive not found - this is expected during bootstrap
                // Skip rather than erroring
                continue;
            }
        }

        Ok(CompilationContext {
            current_module: module_id,
            dependency_chain: Vec::new(),
            available_primitives,
            environment,
            global_env: self.global_env.clone(),
        })
    }

    /// Compiles module expressions into a Module.
    fn compile_module_expressions(
        &self,
        id: &ModuleId,
        exprs: Vec<Expr>,
        context: &CompilationContext,
    ) -> Result<Module> {
        let mut exports = HashMap::new();
        let mut dependencies = Vec::new();
        let mut metadata = ModuleMetadata::default();

        // Process module expressions
        for expr in exprs {
            match &expr {
                // Handle define-library form (R7RS)
                Expr::DefineLibrary { name: lib_name, imports, exports: export_specs, body } => {
                    // Validate that the library name matches the expected module id
                    // For R7RS modules, the first component should match the namespace
                    let expected_name: Vec<String> = match id.namespace {
                        ModuleNamespace::R7RS => {
                            if lib_name.len() >= 2 && lib_name[0] == "scheme" {
                                // For R7RS (scheme xxx) libraries, expect the module components to match xxx part
                                lib_name[1..].to_vec()
                            } else {
                                // For other R7RS libraries, use full name
                                lib_name.clone()
                            }
                        }
                        _ => id.components.clone()
                    };
                    
                    let actual_name = match id.namespace {
                        ModuleNamespace::R7RS => {
                            if lib_name.len() >= 2 && lib_name[0] == "scheme" {
                                lib_name[1..].to_vec()
                            } else {
                                lib_name.clone()
                            }
                        }
                        _ => lib_name.clone()
                    };
                    
                    if actual_name != id.components {
                        return Err(Box::new(Error::parse_error(
                            format!("Library name {:?} does not match expected {:?}", actual_name, id.components),
                            Span::new(0, 0),
                        )));
                    }
                    
                    // Process imports
                    for import_expr in imports {
                        let import_deps = self.process_import_expression(import_expr)?;
                        dependencies.extend(import_deps);
                    }
                    
                    // Process exports
                    for export_expr in export_specs {
                        self.process_export_expression(export_expr, &mut exports, context)?;
                    }
                    
                    // Process body expressions (includes, begins, etc.)
                    for body_expr in body {
                        match &body_expr.inner {
                            Expr::Define { name, value, metadata: def_meta } => {
                                let compiled_value = self.compile_expression(value, context)?;
                                exports.insert(name.clone(), compiled_value);
                                
                                // Extract metadata if present
                                if !def_meta.is_empty() {
                                    self.process_define_metadata(name, def_meta, &mut metadata)?;
                                }
                            }
                            _ => {
                                let _result = self.compile_expression(&body_expr.inner, context)?;
                            }
                        }
                    }
                }
                
                // Handle define-module form (legacy)
                Expr::Application { operator, operands } if self.is_define_module(operator) => {
                    self.process_define_module(operands, &mut metadata, &mut exports)?;
                }
                
                // Handle export declarations
                Expr::Application { operator, operands } if self.is_export_declaration(operator) => {
                    self.process_export_declaration(operands, &mut exports, context)?;
                }
                
                // Handle import declarations  
                Expr::Application { operator, operands } if self.is_import_declaration(operator) => {
                    let import_deps = self.process_import_declaration(operands)?;
                    dependencies.extend(import_deps);
                }
                
                // Handle define forms (create exportable bindings)
                Expr::Define { name, value, metadata: def_meta } => {
                    let compiled_value = self.compile_expression(value, context)?;
                    exports.insert(name.clone(), compiled_value);
                    
                    // Extract metadata if present
                    if !def_meta.is_empty() {
                        // Process define metadata
                        self.process_define_metadata(name, def_meta, &mut metadata)?;
                    }
                }
                
                // Other expressions are evaluated but not exported
                _ => {
                    let _result = self.compile_expression(&expr, context)?;
                    // Result is not exported, but side effects are preserved
                }
            }
        }

        Ok(Module {
            id: id.clone(),
            exports,
            dependencies,
            source: Some(ModuleSource::File(PathBuf::from(format!("{}.scm", id.components.join("-"))))),
            metadata,
        })
    }

    /// Compiles a single expression in the given context.
    fn compile_expression(&self, expr: &Expr, context: &CompilationContext) -> Result<Value> {
        // For now, we'll use a simplified approach that evaluates the expression
        // In a full implementation, this would generate bytecode or optimized representations
        
        // Create a simple evaluator context (placeholder)
        // This is where we'd integrate with the actual evaluator
        match expr {
            Expr::Literal(lit) => Ok(Value::Literal(lit.clone())),
            Expr::Identifier(name) => {
                // Look up in available primitives or environment
                if let Some(value) = context.available_primitives.get(name) {
                    Ok(value.clone())
                } else if let Some(value) = context.environment.lookup(name) {
                    Ok(value)
                } else {
                    Err(Box::new(Error::runtime_error(
                        format!("Unbound variable in library compilation: {name}"),
                        None,
                    )))
                }
            }
            // For complex expressions, we'd need full evaluation or bytecode generation
            _ => Ok(Value::Unspecified), // Placeholder
        }
    }

    /// Finds the source file for a library.
    fn find_library_source(&self, id: &ModuleId) -> Result<Option<PathBuf>> {
        // For SRFI modules, use the numeric part as filename (e.g., "41.scm")
        let filename = match id.namespace {
            ModuleNamespace::SRFI if id.components.len() >= 2 => {
                format!("{}.scm", id.components[1..].join("-"))
            }
            _ => format!("{}.scm", id.components.join("-"))
        };
        
        // Handle file namespace specially
        if id.namespace == ModuleNamespace::File && !id.components.is_empty() {
            return Ok(Some(PathBuf::from(&id.components[0])));
        }
        
        // Try using library resolver first if available
        if let Some(resolver) = &self.library_resolver {
            let subdir = match id.namespace {
                ModuleNamespace::R7RS => "r7rs",
                ModuleNamespace::Builtin => "modules", 
                ModuleNamespace::SRFI => "modules/srfi",
                ModuleNamespace::User => "user",
                ModuleNamespace::File => return Ok(Some(PathBuf::from(&id.components[0]))),
            };
            
            if let Ok(library_path) = resolver.resolve_library_file(subdir, &filename) {
                return Ok(Some(library_path));
            }
        }
        
        // Fallback to old search method
        let subdir = match id.namespace {
            ModuleNamespace::R7RS => "r7rs",
            ModuleNamespace::Builtin => "modules", 
            ModuleNamespace::SRFI => "modules/srfi",
            ModuleNamespace::User => "user",
            ModuleNamespace::File => return Ok(Some(PathBuf::from(&id.components[0]))),
        };

        // Search in configured paths
        for search_path in &self.search_paths {
            let full_path = search_path.join(subdir).join(&filename);
            if full_path.exists() {
                return Ok(Some(full_path));
            }
        }

        // Try direct filename in search paths
        for search_path in &self.search_paths {
            let direct_path = search_path.join(&filename);
            if direct_path.exists() {
                return Ok(Some(direct_path));
            }
        }
        Ok(None)
    }

    /// Processes the bootstrap sequence, loading core libraries.
    pub fn bootstrap(&self) -> Result<Vec<CompiledSchemeLibrary>> {
        let mut loaded_libraries = Vec::new();
        
        // Load core libraries in specified order
        for library_id in &self.bootstrap_config.core_libraries {
            let library = if self.bootstrap_config.lazy_loading {
                // For lazy loading, just verify the library exists
                self.verify_library_exists(library_id)?;
                continue;
            } else {
                self.load_library(library_id)?
            };
            
            loaded_libraries.push(library);
        }

        Ok(loaded_libraries)
    }

    /// Verifies that a library exists without loading it (for lazy loading).
    fn verify_library_exists(&self, id: &ModuleId) -> Result<()> {
        let source_path = self.find_library_source(id)?;
        if source_path.is_some() {
            Ok(())
        } else {
            Err(Box::new(Error::from(ModuleError::NotFound(id.clone()))))
        }
    }

    /// Helper methods for processing module forms
    fn is_define_module(&self, operator: &Expr) -> bool {
        matches!(operator, Expr::Identifier(name) if name == "define-module")
    }

    fn is_export_declaration(&self, operator: &Expr) -> bool {
        matches!(operator, Expr::Identifier(name) if name == "export")  
    }

    fn is_import_declaration(&self, operator: &Expr) -> bool {
        matches!(operator, Expr::Identifier(name) if name == "import")
    }

    fn process_define_module(
        &self,
        _operands: &[crate::diagnostics::Spanned<Expr>],
        _metadata: &mut ModuleMetadata,
        _exports: &mut HashMap<String, Value>,
    ) -> Result<()> {
        // TODO: Process define-module form to extract metadata
        Ok(())
    }

    fn process_export_declaration(
        &self,
        _operands: &[crate::diagnostics::Spanned<Expr>],
        _exports: &mut HashMap<String, Value>,
        _context: &CompilationContext,
    ) -> Result<()> {
        // TODO: Process export declarations
        Ok(())
    }

    fn process_import_declaration(
        &self,
        _operands: &[crate::diagnostics::Spanned<Expr>],
    ) -> Result<Vec<ModuleId>> {
        // TODO: Process import declarations and return dependencies
        Ok(Vec::new())
    }

    /// Process import expression from define-library
    fn process_import_expression(&self, import_expr: &crate::diagnostics::Spanned<Expr>) -> Result<Vec<ModuleId>> {
        match &import_expr.inner {
            Expr::List(components) => {
                // Simple library name like (srfi 41)
                let mut module_parts = Vec::new();
                for component in components {
                    match &component.inner {
                        Expr::Identifier(name) => module_parts.push(name.clone()),
                        Expr::Literal(literal) if literal.is_number() => {
                            if let Some(n) = literal.to_f64() {
                                module_parts.push(n.to_string().split('.').next().unwrap_or("0").to_string());
                            } else {
                                module_parts.push("0".to_string());
                            }
                        }
                        _ => return Err(Box::new(Error::parse_error(
                            "Import library name components must be identifiers or numbers",
                            component.span,
                        ))),
                    }
                }
                
                if module_parts.is_empty() {
                    return Ok(Vec::new());
                }
                
                // Determine namespace based on first component
                let namespace = match module_parts[0].as_str() {
                    "scheme" => ModuleNamespace::R7RS,
                    "srfi" => ModuleNamespace::SRFI,
                    _ => ModuleNamespace::User,
                };
                
                // Strip namespace prefix from components
                let module_components = match namespace {
                    ModuleNamespace::SRFI | ModuleNamespace::R7RS => {
                        if module_parts.len() > 1 {
                            module_parts[1..].to_vec()
                        } else {
                            module_parts
                        }
                    }
                    _ => module_parts,
                };
                
                let module_id = ModuleId::new(namespace, module_components);
                Ok(vec![module_id])
            }
            _ => {
                // For now, ignore complex import specs (only, except, prefix, rename)
                Ok(Vec::new())
            }
        }
    }

    /// Process export expression from define-library
    fn process_export_expression(
        &self,
        export_expr: &crate::diagnostics::Spanned<Expr>,
        exports: &mut HashMap<String, Value>,
        context: &CompilationContext,
    ) -> Result<()> {
        match &export_expr.inner {
            Expr::Identifier(name) => {
                // Simple export - try to get the actual value from environment or primitives
                let value = if let Some(primitive_value) = context.available_primitives.get(name) {
                    // Found in primitives - this is the common case for R7RS standard library exports
                    primitive_value.clone()
                } else if let Some(env_value) = context.environment.lookup(name) {
                    // Found in environment
                    env_value
                } else {
                    // Not found - could be defined later in the module body
                    // For now, mark as unspecified
                    Value::Unspecified
                };
                exports.insert(name.clone(), value);
                Ok(())
            }
            Expr::List(components) => {
                // Export spec like (rename local-name exported-name)
                if let Some(first) = components.first() {
                    if let Expr::Identifier(keyword) = &first.inner {
                        match keyword.as_str() {
                            "rename" => {
                                // (rename local-name exported-name)
                                if components.len() == 3 {
                                    if let (Expr::Identifier(local), Expr::Identifier(exported)) = 
                                        (&components[1].inner, &components[2].inner) {
                                        // Look up the local name and export under the new name
                                        let value = if let Some(primitive_value) = context.available_primitives.get(local) {
                                            primitive_value.clone()
                                        } else if let Some(env_value) = context.environment.lookup(local) {
                                            env_value
                                        } else {
                                            Value::Unspecified
                                        };
                                        exports.insert(exported.clone(), value);
                                    }
                                }
                            }
                            _ => {
                                // Unknown export spec, ignore for now
                            }
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn process_define_metadata(
        &self,
        _name: &str,
        _metadata: &std::collections::HashMap<String, Spanned<Expr>>,
        _module_metadata: &mut ModuleMetadata,
    ) -> Result<()> {
        // TODO: Process define metadata
        Ok(())
    }

    /// Gets cache statistics.
    pub fn cache_statistics(&self) -> CacheStatistics {
        self.cache.statistics()
    }

    /// Invalidates cache for a specific library (useful for hot-reloading).
    pub fn invalidate_library(&self, id: &ModuleId) {
        self.cache.invalidate(id);
    }

    /// Gets the library path resolver if available.
    pub fn library_resolver(&self) -> Option<&LibraryPathResolver> {
        self.library_resolver.as_ref()
    }

    /// Initializes search paths from library resolver if available.
    pub fn initialize_from_library_resolver(&mut self) {
        if let Some(resolver) = &self.library_resolver {
            // Add search paths from library resolver
            for path in resolver.search_paths() {
                if !self.search_paths.contains(path) {
                    self.search_paths.push(path.clone());
                }
            }
        }
    }
}

impl BootstrapConfig {
    /// Creates default bootstrap configuration.
    pub fn new_default() -> Self {
        Self {
            essential_primitives: vec![
                // Core arithmetic
                "+".to_string(), "-".to_string(), "*".to_string(), "/".to_string(),
                "=".to_string(), "<".to_string(), ">".to_string(), "<=".to_string(), ">=".to_string(),
                
                // Core list operations  
                "cons".to_string(), "car".to_string(), "cdr".to_string(), "null?".to_string(),
                "pair?".to_string(), "list?".to_string(),
                
                // Core predicates
                "eq?".to_string(), "eqv?".to_string(), "equal?".to_string(),
                "boolean?".to_string(), "number?".to_string(), "string?".to_string(), "symbol?".to_string(),
                
                // Core string operations
                "string-length".to_string(), "string-ref".to_string(), "string-set!".to_string(),
                "string=?".to_string(), "string<?".to_string(), "string>?".to_string(),
                
                // Core I/O (minimal)
                "display".to_string(), "newline".to_string(), "write".to_string(),
                
                // Core control
                "apply".to_string(), "call-with-current-continuation".to_string(),
                
                // Essential system functions
                "error".to_string(), "current-second".to_string(),
            ],
            // TEMPORARY FIX: Remove core libraries from bootstrap to prevent circular dependencies
            // The library loader will attempt to load (scheme base) but we need the primitives 
            // to be available first. Libraries should be loaded on-demand via import statements.
            core_libraries: vec![],
            load_order: vec![],
            lazy_loading: true,
            bootstrap_timeout: Duration::from_secs(30),
        }
    }

    /// Creates a minimal bootstrap configuration with only essential primitives.
    pub fn minimal() -> Self {
        Self {
            essential_primitives: vec![
                "cons".to_string(), "car".to_string(), "cdr".to_string(),
                "eq?".to_string(), "error".to_string(), "apply".to_string(),
            ],
            core_libraries: Vec::new(),
            load_order: Vec::new(),
            lazy_loading: true,
            bootstrap_timeout: Duration::from_secs(10),
        }
    }
}

impl Default for HotReloadManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HotReloadManager {
    /// Creates a new hot-reload manager.
    pub fn new() -> Self {
        Self {
            watchers: Arc::new(Mutex::new(HashMap::new())),
            reload_callbacks: Arc::new(Mutex::new(Vec::new())),
            generation: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Registers a callback for library reloading.
    pub fn register_reload_callback<F>(&self, callback: F) 
    where 
        F: Fn(&ModuleId) + Send + Sync + 'static 
    {
        let mut callbacks = self.reload_callbacks.lock().unwrap();
        callbacks.push(Box::new(callback));
    }

    /// Starts watching a file for changes.
    pub fn watch_file(&self, path: PathBuf) -> Result<()> {
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(mtime) = metadata.modified() {
                let mut watchers = self.watchers.lock().unwrap();
                watchers.insert(path, mtime);
            }
        }
        Ok(())
    }

    /// Checks for file changes and triggers reloads.
    pub fn check_for_changes(&self) -> Vec<PathBuf> {
        let mut changed_files = Vec::new();
        let mut watchers = self.watchers.lock().unwrap();
        
        for (path, last_mtime) in watchers.iter_mut() {
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(current_mtime) = metadata.modified() {
                    if current_mtime > *last_mtime {
                        changed_files.push(path.clone());
                        *last_mtime = current_mtime;
                    }
                }
            }
        }
        
        if !changed_files.is_empty() {
            self.generation.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        
        changed_files
    }

    /// Gets the current reload generation.
    pub fn current_generation(&self) -> u64 {
        self.generation.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::GlobalEnvironmentManager;

    #[test]
    fn test_scheme_library_cache() {
        let cache = SchemeLibraryCache::new();
        let module_id = ModuleId {
            components: vec!["test".to_string()],
            namespace: ModuleNamespace::User,
        };

        // Cache miss
        assert!(cache.get(&module_id, None).is_none());
        
        // Store and retrieve
        let library = CompiledSchemeLibrary {
            module: Module {
                id: module_id.clone(),
                exports: HashMap::new(),
                dependencies: Vec::new(),
                source: None,
                metadata: ModuleMetadata::default(),
            },
            bytecode: None,
            compiled_at: SystemTime::now(),
            source_mtime: SystemTime::now(),
            resolved_dependencies: Vec::new(),
            reload_generation: 0,
        };

        cache.store(module_id.clone(), library.clone());
        
        // Cache hit
        assert!(cache.get(&module_id, None).is_some());
        
        // Statistics
        let stats = cache.statistics();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_bootstrap_config() {
        let config = BootstrapConfig::new_default();
        assert!(!config.essential_primitives.is_empty());
        assert!(!config.core_libraries.is_empty());
        
        let minimal_config = BootstrapConfig::minimal();
        assert!(minimal_config.essential_primitives.len() < config.essential_primitives.len());
        assert!(minimal_config.core_libraries.is_empty());
    }

    #[test]
    fn test_scheme_library_loader_creation() {
        let global_env = Arc::new(GlobalEnvironmentManager::new());
        let loader = SchemeLibraryLoader::new(global_env);
        assert!(loader.is_ok());
    }

    #[test]
    fn test_hot_reload_manager() {
        let manager = HotReloadManager::new();
        assert_eq!(manager.current_generation(), 0);
        
        // Register callback
        let callback_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let callback_called_clone = callback_called.clone();
        
        manager.register_reload_callback(move |_| {
            callback_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        
        // Check for changes (no files watched, so no changes)
        let changes = manager.check_for_changes();
        assert!(changes.is_empty());
    }

    #[test]
    fn test_compilation_context_creation() {
        let global_env = Arc::new(GlobalEnvironmentManager::new());
        let loader = SchemeLibraryLoader::new(global_env).unwrap();
        
        let module_id = ModuleId {
            components: vec!["test".to_string()],
            namespace: ModuleNamespace::User,
        };
        
        let context = loader.create_compilation_context(module_id);
        assert!(context.is_ok());
        
        let ctx = context.unwrap();
        assert!(!ctx.available_primitives.is_empty());
    }
}
//! Bootstrap system for Lambdust runtime.
//!
//! This module implements the bootstrap process that initializes the runtime environment
//! with minimal Rust primitives and loads the Scheme-based standard library modules.
//! 
//! The bootstrap follows a carefully orchestrated sequence:
//! 1. Initialize minimal Rust primitives required for Scheme compilation
//! 2. Set up the module loading infrastructure  
//! 3. Load core Scheme libraries in dependency order
//! 4. Provide runtime services for library management

use crate::diagnostics::{Result, Error, error::helpers};
use crate::eval::Value;
use crate::runtime::{GlobalEnvironmentManager, LibraryPathResolver};
use crate::module_system::{SchemeLibraryLoader, BootstrapConfig};
use crate::stdlib::StandardLibrary;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Instant, Duration};

/// Core bootstrap system that manages the startup sequence.
#[derive(Debug)]
pub struct BootstrapSystem {
    /// Global environment manager
    global_env: Arc<GlobalEnvironmentManager>,
    /// Scheme library loader
    scheme_loader: SchemeLibraryLoader,
    /// Bootstrap configuration
    config: BootstrapConfig,
    /// Bootstrap statistics
    stats: BootstrapStatistics,
    /// Minimal primitives registry
    minimal_primitives: MinimalPrimitivesRegistry,
    /// Library path resolver
    library_resolver: LibraryPathResolver,
}

/// Statistics collected during bootstrap process.
#[derive(Debug, Default, Clone)]
pub struct BootstrapStatistics {
    /// Total bootstrap time
    pub total_time: Duration,
    /// Time spent loading minimal primitives
    pub primitives_load_time: Duration,
    /// Time spent loading Scheme libraries
    pub libraries_load_time: Duration,
    /// Number of primitives loaded
    pub primitives_count: usize,
    /// Number of Scheme libraries loaded
    pub libraries_count: usize,
    /// Memory usage after bootstrap
    pub memory_usage_bytes: usize,
}

/// Registry of minimal Rust primitives required for Scheme library compilation.
#[derive(Debug, Default)]
pub struct MinimalPrimitivesRegistry {
    /// Essential primitives for Scheme evaluation
    primitives: HashMap<String, MinimalPrimitive>,
    /// Categories of primitives
    categories: HashMap<PrimitiveCategory, Vec<String>>,
}

/// A minimal primitive procedure implemented in Rust.
#[derive(Debug, Clone)]
pub struct MinimalPrimitive {
    /// Primitive name
    pub name: String,
    /// Implementation function
    pub implementation: fn(&[Value]) -> Result<Value>,
    /// Minimum number of arguments
    pub arity_min: usize,
    /// Maximum number of arguments (None = unlimited)
    pub arity_max: Option<usize>,
    /// Primitive category
    pub category: PrimitiveCategory,
    /// Documentation string
    pub documentation: String,
}

/// Categories of primitive procedures.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PrimitiveCategory {
    /// Core arithmetic operations
    Arithmetic,
    /// Core list operations
    Lists,
    /// Core comparison operations
    Comparison,
    /// Core type predicates
    Predicates,
    /// Core control flow
    Control,
    /// Core string operations
    Strings,
    /// Core I/O operations (minimal)
    IO,
    /// Core symbol operations
    Symbols,
    /// Error handling
    Errors,
    /// System functions
    System,
}

/// Bootstrap phases for organized initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapPhase {
    /// Initialize minimal primitives
    InitializePrimitives,
    /// Set up module loading infrastructure
    SetupModuleSystem,
    /// Load core Scheme libraries
    LoadCoreLibraries,
    /// Finalize environment
    FinalizeEnvironment,
}

impl BootstrapSystem {
    /// Creates a new bootstrap system with default configuration.
    pub fn new() -> Result<Self> {
        let global_env = Arc::new(GlobalEnvironmentManager::new());
        let config = BootstrapConfig::new_default();
        let library_resolver = LibraryPathResolver::new()?;
        let scheme_loader = SchemeLibraryLoader::new(global_env.clone())?;
        
        Ok(Self {
            global_env,
            scheme_loader,
            config,
            stats: BootstrapStatistics::default(),
            minimal_primitives: MinimalPrimitivesRegistry::new(),
            library_resolver,
        })
    }

    /// Creates a bootstrap system with custom configuration.
    pub fn with_config(config: BootstrapConfig) -> Result<Self> {
        let global_env = Arc::new(GlobalEnvironmentManager::new());
        let library_resolver = LibraryPathResolver::new()?;
        let scheme_loader = SchemeLibraryLoader::with_bootstrap_config(global_env.clone(), config.clone())?;
        
        Ok(Self {
            global_env,
            scheme_loader,
            config,
            stats: BootstrapStatistics::default(),
            minimal_primitives: MinimalPrimitivesRegistry::new(),
            library_resolver,
        })
    }

    /// Runs the complete bootstrap sequence.
    pub fn bootstrap(&mut self) -> Result<Arc<GlobalEnvironmentManager>> {
        let start_time = Instant::now();
        
        // Phase 1: Initialize minimal primitives
        self.run_phase(BootstrapPhase::InitializePrimitives)?;
        
        // Phase 2: Set up module loading infrastructure
        self.run_phase(BootstrapPhase::SetupModuleSystem)?;
        
        // Phase 3: Load core Scheme libraries
        self.run_phase(BootstrapPhase::LoadCoreLibraries)?;
        
        // Phase 4: Finalize environment
        self.run_phase(BootstrapPhase::FinalizeEnvironment)?;
        
        // Record total bootstrap time
        self.stats.total_time = start_time.elapsed();
        
        Ok(self.global_env.clone())
    }

    /// Runs a specific bootstrap phase.
    fn run_phase(&mut self, phase: BootstrapPhase) -> Result<()> {
        match phase {
            BootstrapPhase::InitializePrimitives => {
                let start = Instant::now();
                self.initialize_minimal_primitives()?;
                self.stats.primitives_load_time = start.elapsed();
            }
            BootstrapPhase::SetupModuleSystem => {
                self.setup_module_system()?;
            }
            BootstrapPhase::LoadCoreLibraries => {
                let start = Instant::now();
                self.load_core_libraries()?;
                self.stats.libraries_load_time = start.elapsed();
            }
            BootstrapPhase::FinalizeEnvironment => {
                self.finalize_environment()?;
            }
        }
        Ok(())
    }

    /// Phase 1: Initialize minimal Rust primitives required for Scheme compilation.
    fn initialize_minimal_primitives(&mut self) -> Result<()> {
        let root_env = self.global_env.root_environment();
        
        // Load minimal primitives into the environment
        for (name, primitive) in self.minimal_primitives.primitives.iter() {
            let value = Value::minimal_primitive(
                name.clone(),
                primitive.implementation,
                primitive.arity_min,
                primitive.arity_max,
            );
            root_env.define(name.clone(), value);
        }
        
        self.stats.primitives_count = self.minimal_primitives.primitives.len();
        Ok(())
    }

    /// Phase 2: Set up module loading infrastructure.
    fn setup_module_system(&mut self) -> Result<()> {
        // Use library path resolver to configure scheme loader search paths
        for search_path in self.library_resolver.search_paths() {
            self.scheme_loader.add_search_path(search_path);
        }

        // Add specific subdirectories if they exist
        if let Ok(bootstrap_dir) = self.library_resolver.resolve_lib_subdir("bootstrap") {
            self.scheme_loader.add_search_path(bootstrap_dir);
        }

        if let Ok(r7rs_dir) = self.library_resolver.resolve_lib_subdir("r7rs") {
            self.scheme_loader.add_search_path(r7rs_dir);
        }

        if let Ok(modules_dir) = self.library_resolver.resolve_lib_subdir("modules") {
            self.scheme_loader.add_search_path(modules_dir);
        }

        Ok(())
    }

    /// Phase 3: Load core Scheme libraries in dependency order.
    fn load_core_libraries(&mut self) -> Result<()> {
        if self.config.lazy_loading {
            // For lazy loading, just verify libraries exist
            let _verified_libraries = self.scheme_loader.bootstrap()?;
            return Ok(());
        }

        // Load libraries in the specified order
        let mut loaded_count = 0;
        for library_id in &self.config.load_order {
            match self.scheme_loader.load_library(library_id) {
                Ok(compiled_library) => {
                    // Install the library's exports into the global environment
                    self.install_library_exports(&compiled_library)?;
                    loaded_count += 1;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load library {}: {}", 
                             crate::module_system::format_module_id(library_id), e);
                    // Continue with other libraries - some failures are acceptable
                }
            }
        }
        
        self.stats.libraries_count = loaded_count;
        Ok(())
    }

    /// Phase 4: Finalize the environment setup.
    fn finalize_environment(&mut self) -> Result<()> {
        // Install any remaining standard library functions that weren't 
        // migrated to Scheme yet
        let stdlib = StandardLibrary::new();
        stdlib.populate_environment(&self.global_env.root_environment());
        
        // Create initial snapshot for potential rollbacks
        let _snapshot_generation = self.global_env.create_environment_snapshot()?;
        
        // Estimate memory usage (simplified)
        self.stats.memory_usage_bytes = self.estimate_memory_usage();
        
        Ok(())
    }

    /// Installs a compiled library's exports into the global environment.
    fn install_library_exports(&self, library: &crate::module_system::CompiledSchemeLibrary) -> Result<()> {
        let root_env = self.global_env.root_environment();
        
        for (name, value) in &library.module.exports {
            root_env.define(name.clone(), value.clone());
        }
        
        Ok(())
    }

    /// Estimates current memory usage (simplified implementation).
    fn estimate_memory_usage(&self) -> usize {
        // This is a simplified estimation
        // In a production implementation, you'd use more sophisticated memory tracking
        let base_size = std::mem::size_of::<BootstrapSystem>();
        let env_size = self.global_env.global_variable_names().len() * 100; // Rough estimate
        base_size + env_size
    }

    /// Gets bootstrap statistics.
    pub fn statistics(&self) -> &BootstrapStatistics {
        &self.stats
    }

    /// Gets the global environment manager.
    pub fn global_environment(&self) -> Arc<GlobalEnvironmentManager> {
        self.global_env.clone()
    }

    /// Gets the scheme library loader.
    pub fn scheme_loader(&self) -> &SchemeLibraryLoader {
        &self.scheme_loader
    }

    /// Enables development mode with hot-reloading.
    pub fn enable_development_mode(&mut self) {
        self.scheme_loader.enable_dev_mode();
    }

    /// Gets the library path resolver.
    pub fn library_resolver(&self) -> &LibraryPathResolver {
        &self.library_resolver
    }

    /// Validates the library setup and returns a report.
    pub fn validate_library_setup(&self) -> Result<crate::runtime::LibraryValidationReport> {
        self.library_resolver.validate_library_setup()
    }
}

impl MinimalPrimitivesRegistry {
    /// Creates a new minimal primitives registry with essential procedures.
    pub fn new() -> Self {
        let mut registry = Self {
            primitives: HashMap::new(),
            categories: HashMap::new(),
        };
        
        registry.register_essential_primitives();
        registry
    }

    /// Registers all essential primitives required for Scheme library compilation.
    fn register_essential_primitives(&mut self) {
        // Arithmetic primitives
        self.register_primitive(MinimalPrimitive {
            name: "+".to_owned(),
            implementation: primitive_add,
            arity_min: 0,
            arity_max: None,
            category: PrimitiveCategory::Arithmetic,
            documentation: "Addition of numbers".to_owned(),
        });

        self.register_primitive(MinimalPrimitive {
            name: "-".to_owned(),
            implementation: primitive_subtract,
            arity_min: 1,
            arity_max: None,
            category: PrimitiveCategory::Arithmetic,
            documentation: "Subtraction of numbers".to_owned(),
        });

        self.register_primitive(MinimalPrimitive {
            name: "*".to_owned(),
            implementation: primitive_multiply,
            arity_min: 0,
            arity_max: None,
            category: PrimitiveCategory::Arithmetic,
            documentation: "Multiplication of numbers".to_owned(),
        });

        // Comparison primitives
        self.register_primitive(MinimalPrimitive {
            name: "=".to_owned(),
            implementation: primitive_numeric_equal,
            arity_min: 2,
            arity_max: None,
            category: PrimitiveCategory::Comparison,
            documentation: "Numeric equality comparison".to_owned(),
        });

        self.register_primitive(MinimalPrimitive {
            name: "<".to_owned(),
            implementation: primitive_less_than,
            arity_min: 2,
            arity_max: None,
            category: PrimitiveCategory::Comparison,
            documentation: "Numeric less-than comparison".to_owned(),
        });

        // List primitives
        self.register_primitive(MinimalPrimitive {
            name: "cons".to_owned(),
            implementation: primitive_cons,
            arity_min: 2,
            arity_max: Some(2),
            category: PrimitiveCategory::Lists,
            documentation: "Construct a pair".to_owned(),
        });

        self.register_primitive(MinimalPrimitive {
            name: "car".to_owned(),
            implementation: primitive_car,
            arity_min: 1,
            arity_max: Some(1),
            category: PrimitiveCategory::Lists,
            documentation: "First element of a pair".to_owned(),
        });

        self.register_primitive(MinimalPrimitive {
            name: "cdr".to_owned(),
            implementation: primitive_cdr,
            arity_min: 1,
            arity_max: Some(1),
            category: PrimitiveCategory::Lists,
            documentation: "Rest of a pair".to_owned(),
        });

        // Predicate primitives
        self.register_primitive(MinimalPrimitive {
            name: "null?".to_owned(),
            implementation: primitive_null_p,
            arity_min: 1,
            arity_max: Some(1),
            category: PrimitiveCategory::Predicates,
            documentation: "Test for null value".to_owned(),
        });

        self.register_primitive(MinimalPrimitive {
            name: "pair?".to_owned(),
            implementation: primitive_pair_p,
            arity_min: 1,
            arity_max: Some(1),
            category: PrimitiveCategory::Predicates,
            documentation: "Test for pair".to_owned(),
        });

        // Control primitives
        self.register_primitive(MinimalPrimitive {
            name: "apply".to_owned(),
            implementation: primitive_apply,
            arity_min: 2,
            arity_max: None,
            category: PrimitiveCategory::Control,
            documentation: "Apply procedure to arguments".to_owned(),
        });

        // Error handling
        self.register_primitive(MinimalPrimitive {
            name: "error".to_owned(),
            implementation: primitive_error,
            arity_min: 1,
            arity_max: None,
            category: PrimitiveCategory::Errors,
            documentation: "Signal an error".to_owned(),
        });

        // String primitives (minimal)
        self.register_primitive(MinimalPrimitive {
            name: "string?".to_owned(),
            implementation: primitive_string_p,
            arity_min: 1,
            arity_max: Some(1),
            category: PrimitiveCategory::Predicates,
            documentation: "Test for string".to_owned(),
        });

        // I/O primitives (minimal)
        self.register_primitive(MinimalPrimitive {
            name: "display".to_owned(),
            implementation: primitive_display,
            arity_min: 1,
            arity_max: Some(2),
            category: PrimitiveCategory::IO,
            documentation: "Display a value".to_owned(),
        });
    }

    /// Registers a primitive procedure.
    fn register_primitive(&mut self, primitive: MinimalPrimitive) {
        let name = primitive.name.clone();
        let category = primitive.category.clone();
        
        self.primitives.insert(name.clone(), primitive);
        
        self.categories.entry(category)
            .or_default()
            .push(name);
    }

    /// Gets all primitives in a category.
    pub fn primitives_in_category(&self, category: &PrimitiveCategory) -> Vec<&MinimalPrimitive> {
        if let Some(names) = self.categories.get(category) {
            names.iter()
                .filter_map(|name| self.primitives.get(name))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Gets a primitive by name.
    pub fn get_primitive(&self, name: &str) -> Option<&MinimalPrimitive> {
        self.primitives.get(name)
    }

    /// Lists all primitive names.
    pub fn primitive_names(&self) -> Vec<&String> {
        self.primitives.keys().collect()
    }
}

// ============= MINIMAL PRIMITIVE IMPLEMENTATIONS =============

/// Helper function to extract integer values from literals
fn extract_integer_value(value: &Value) -> Option<i64> {
    match value {
        Value::Literal(literal) => literal.to_i64(),
        _ => None,
    }
}

/// Addition primitive (+)
fn primitive_add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(0));
    }
    
    let mut result = 0i64;
    for arg in args {
        if let Some(n) = extract_integer_value(arg) {
            result += n;
        } else {
            return Err(helpers::runtime_error_simple("+ expects numeric arguments"));
        }
    }
    Ok(Value::integer(result))
}

/// Subtraction primitive (-)
fn primitive_subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(helpers::runtime_error_simple("- requires at least one argument"));
    }
    
    if args.len() == 1 {
        // Negation
        if let Some(n) = extract_integer_value(&args[0]) {
            Ok(Value::integer(-n))
        } else {
            Err(helpers::runtime_error_simple("- expects numeric arguments"))
        }
    } else {
        // Subtraction
        let mut result = if let Some(n) = extract_integer_value(&args[0]) {
            n
        } else {
            return Err(Box::new(Error::runtime_error("- expects numeric arguments", None)));
        };
        
        for arg in &args[1..] {
            if let Some(n) = extract_integer_value(arg) {
                result -= n;
            } else {
                return Err(Box::new(Error::runtime_error("- expects numeric arguments", None)));
            }
        }
        Ok(Value::integer(result))
    }
}

/// Multiplication primitive (*)
fn primitive_multiply(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(1));
    }
    
    let mut result = 1i64;
    for arg in args {
        if let Some(n) = extract_integer_value(arg) {
            result *= n;
        } else {
            return Err(Box::new(Error::runtime_error("* expects numeric arguments", None)));
        }
    }
    Ok(Value::integer(result))
}

/// Numeric equality primitive (=)
fn primitive_numeric_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error("= requires at least 2 arguments", None)));
    }
    
    let first = if let Some(n) = extract_integer_value(&args[0]) {
        n
    } else {
        return Err(Box::new(Error::runtime_error("= expects numeric arguments", None)));
    };
    
    for arg in &args[1..] {
        if let Some(n_val) = extract_integer_value(arg) {
            if first != n_val {
                return Ok(Value::boolean(false));
            }
        } else {
            return Err(Box::new(Error::runtime_error("= expects numeric arguments", None)));
        }
    }
    Ok(Value::boolean(true))
}

/// Less-than primitive (<)
fn primitive_less_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::runtime_error("< requires at least 2 arguments", None)));
    }
    
    for i in 0..args.len() - 1 {
        let current = if let Some(n) = extract_integer_value(&args[i]) {
            n
        } else {
            return Err(Box::new(Error::runtime_error("< expects numeric arguments", None)));
        };
        
        let next = if let Some(n) = extract_integer_value(&args[i + 1]) {
            n
        } else {
            return Err(Box::new(Error::runtime_error("< expects numeric arguments", None)));
        };
        
        if current >= next {
            return Ok(Value::boolean(false));
        }
    }
    Ok(Value::boolean(true))
}

/// cons primitive
fn primitive_cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error("cons requires exactly 2 arguments", None)));
    }
    Ok(Value::pair(args[0].clone(), args[1].clone()))
}

/// car primitive
fn primitive_car(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("car requires exactly 1 argument", None)));
    }
    
    match &args[0] {
        Value::Pair(car, _) => Ok((**car).clone()),
        _ => Err(Box::new(Error::runtime_error("car expects a pair", None))),
    }
}

/// cdr primitive
fn primitive_cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("cdr requires exactly 1 argument", None)));
    }
    
    match &args[0] {
        Value::Pair(_, cdr) => Ok((**cdr).clone()),
        _ => Err(Box::new(Error::runtime_error("cdr expects a pair", None))),
    }
}

/// null? primitive
fn primitive_null_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("null? requires exactly 1 argument", None)));
    }
    
    Ok(Value::boolean(matches!(args[0], Value::Nil)))
}

/// pair? primitive
fn primitive_pair_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("pair? requires exactly 1 argument", None)));
    }
    
    Ok(Value::boolean(matches!(args[0], Value::Pair(_, _))))
}

/// string? primitive
fn primitive_string_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error("string? requires exactly 1 argument", None)));
    }
    
    Ok(Value::boolean(matches!(args[0], Value::Literal(crate::ast::Literal::String(_)))))
}

/// apply primitive (simplified)
fn primitive_apply(_args: &[Value]) -> Result<Value> {
    // Simplified implementation - in practice this would need access to the evaluator
    Err(Box::new(Error::runtime_error("apply not fully implemented in minimal primitives", None)))
}

/// error primitive
fn primitive_error(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error("error requires at least 1 argument", None)));
    }
    
    let message = match &args[0] {
        Value::Literal(crate::ast::Literal::String(s)) => s.clone(),
        _ => format!("{}", args[0]),
    };
    
    Err(Error::runtime_error(message, None).boxed())
}

/// display primitive (R7RS-compliant)
fn primitive_display(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error("display requires 1 or 2 arguments", None)));
    }
    
    // Use the R7RS-compliant display formatting method from Value
    let output = args[0].display_string();
    println!("{output}");
    Ok(Value::Unspecified)
}

impl Default for BootstrapSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create default bootstrap system")
    }
}

// Extension to Value for minimal primitives
impl Value {
    /// Creates a minimal primitive value.
    pub fn minimal_primitive(
        name: String,
        implementation: fn(&[Value]) -> Result<Value>,
        arity_min: usize,
        arity_max: Option<usize>,
    ) -> Self {
        use crate::eval::value::{PrimitiveProcedure, PrimitiveImpl};
        use crate::effects::Effect;
        
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name,
            arity_min,
            arity_max,
            implementation: PrimitiveImpl::RustFn(implementation),
            effects: vec![Effect::Pure], // Most minimal primitives are pure
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_primitives_registry() {
        let registry = MinimalPrimitivesRegistry::new();
        
        assert!(!registry.primitives.is_empty());
        assert!(registry.get_primitive("+").is_some());
        assert!(registry.get_primitive("cons").is_some());
        assert!(registry.get_primitive("nonexistent").is_none());
        
        let arithmetic_prims = registry.primitives_in_category(&PrimitiveCategory::Arithmetic);
        assert!(!arithmetic_prims.is_empty());
    }

    #[test]
    fn test_bootstrap_system_creation() {
        let system = BootstrapSystem::new();
        assert!(system.is_ok());
    }

    #[test]
    fn test_bootstrap_config() {
        let config = BootstrapConfig::new_default();
        assert!(!config.essential_primitives.is_empty());
        
        let minimal_config = BootstrapConfig::minimal();
        assert!(minimal_config.essential_primitives.len() < config.essential_primitives.len());
    }

    #[test]
    fn test_bootstrap_display_r7rs_compliance() {
        // Test that the bootstrap display function formats strings without quotes
        let test_string = Value::string("Hello World");
        let result = test_string.display_string();
        assert_eq!(result, "Hello World"); // Should NOT have quotes
        
        // Test characters without #\ prefix
        let test_char = Value::Literal(crate::ast::Literal::Character('x'));
        let result = test_char.display_string();
        assert_eq!(result, "x"); // Should NOT have #\ prefix
        
        // Test numbers still format correctly
        let test_number = Value::integer(42);
        let result = test_number.display_string();
        assert_eq!(result, "42");
        
        // Test booleans still format correctly
        let test_bool = Value::boolean(true);
        let result = test_bool.display_string();
        assert_eq!(result, "#t");
    }

    #[test]
    fn test_primitive_arithmetic() {
        // Test addition
        let args = vec![Value::integer(1), Value::integer(2), Value::integer(3)];
        let result = primitive_add(&args).unwrap();
        assert_eq!(result, Value::integer(6));
        
        // Test empty addition
        let result = primitive_add(&[]).unwrap();
        assert_eq!(result, Value::integer(0));
        
        // Test subtraction
        let args = vec![Value::integer(10), Value::integer(3)];
        let result = primitive_subtract(&args).unwrap();
        assert_eq!(result, Value::integer(7));
        
        // Test negation
        let args = vec![Value::integer(5)];
        let result = primitive_subtract(&args).unwrap();
        assert_eq!(result, Value::integer(-5));
    }

    #[test]
    fn test_primitive_lists() {
        // Test cons
        let args = vec![Value::integer(1), Value::integer(2)];
        let result = primitive_cons(&args).unwrap();
        assert!(matches!(result, Value::Pair(_, _)));
        
        // Test car
        let pair = Value::pair(Value::integer(1), Value::integer(2));
        let result = primitive_car(&[pair]).unwrap();
        assert_eq!(result, Value::integer(1));
        
        // Test null?
        let result = primitive_null_p(&[Value::Nil]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_null_p(&[Value::integer(42)]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_primitive_predicates() {
        // Test string?
        let result = primitive_string_p(&[Value::string("hello")]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_string_p(&[Value::integer(42)]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test pair?
        let pair = Value::pair(Value::integer(1), Value::integer(2));
        let result = primitive_pair_p(&[pair]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_pair_p(&[Value::integer(42)]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_primitive_comparison() {
        // Test numeric equality
        let args = vec![Value::integer(5), Value::integer(5), Value::integer(5)];
        let result = primitive_numeric_equal(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::integer(5), Value::integer(6)];
        let result = primitive_numeric_equal(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test less-than
        let args = vec![Value::integer(1), Value::integer(2), Value::integer(3)];
        let result = primitive_less_than(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::integer(1), Value::integer(3), Value::integer(2)];
        let result = primitive_less_than(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
}
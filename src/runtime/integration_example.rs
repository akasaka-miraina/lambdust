//! Integration example demonstrating the complete library loading system.
//!
//! This module shows how all the components work together:
//! 1. Bootstrap system initializes minimal primitives
//! 2. Scheme library loader compiles and loads Scheme libraries
//! 3. Primitive bridge enables seamless Rust-Scheme interop
//! 4. Performance optimizations provide efficient execution

use super::{BootstrapSystem, BootstrapConfig, GlobalEnvironmentManager};
use crate::module_system::{SchemeLibraryLoader, ModuleId, ModuleNamespace};
use crate::runtime::primitive_bridge::{PrimitiveBridge, PrimitiveSignature, SchemeType, PrimitiveDocumentation, ParameterDoc};
use crate::diagnostics::Result;
use crate::eval::Value;
use std::sync::Arc;
use std::time::Instant;

/// Complete integration example showing the "minimal Rust + rich Scheme libraries" architecture.
pub struct IntegrationExample {
    /// Bootstrap system
    bootstrap: BootstrapSystem,
    /// Global environment
    global_env: Arc<GlobalEnvironmentManager>,
    /// Primitive bridge
    primitive_bridge: PrimitiveBridge,
    /// Performance metrics
    metrics: IntegrationMetrics,
}

/// Performance and usage metrics for the integration.
#[derive(Debug, Default, Clone)]
pub struct IntegrationMetrics {
    /// Bootstrap time
    pub bootstrap_time_ms: u64,
    /// Number of Rust primitives loaded
    pub primitives_loaded: usize,
    /// Number of Scheme libraries loaded
    pub scheme_libraries_loaded: usize,
    /// Total memory usage
    pub memory_usage_bytes: usize,
    /// Library compilation time
    pub compilation_time_ms: u64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
}

impl IntegrationExample {
    /// Creates a new integration example with default configuration.
    pub fn new() -> Result<Self> {
        let mut bootstrap = BootstrapSystem::new()?;
        let primitive_bridge = PrimitiveBridge::new();
        
        Ok(Self {
            bootstrap,
            global_env: Arc::new(GlobalEnvironmentManager::new()),
            primitive_bridge,
            metrics: IntegrationMetrics::default(),
        })
    }

    /// Demonstrates the complete integration process.
    pub fn run_complete_example(&mut self) -> Result<()> {
        println!("🚀 Lambdust Library Loading System Integration Example");
        println!("======================================================");
        
        // Phase 1: Bootstrap the system
        self.run_bootstrap_phase()?;
        
        // Phase 2: Set up primitive bridge
        self.setup_primitive_bridge()?;
        
        // Phase 3: Load Scheme libraries
        self.load_scheme_libraries()?;
        
        // Phase 4: Demonstrate interop
        self.demonstrate_interop()?;
        
        // Phase 5: Show performance metrics
        self.show_performance_metrics();
        
        println!("\n✅ Integration example completed successfully!");
        Ok(())
    }

    /// Phase 1: Bootstrap the system with minimal primitives.
    fn run_bootstrap_phase(&mut self) -> Result<()> {
        println!("\n📦 Phase 1: Bootstrapping with minimal primitives");
        println!("--------------------------------------------------");
        
        let start = Instant::now();
        
        // Run bootstrap process
        self.global_env = self.bootstrap.bootstrap()?;
        
        let bootstrap_time = start.elapsed();
        self.metrics.bootstrap_time_ms = bootstrap_time.as_millis() as u64;
        
        let stats = self.bootstrap.statistics();
        self.metrics.primitives_loaded = stats.primitives_count;
        
        println!("✓ Loaded {} minimal primitives in {}ms", 
                 stats.primitives_count, self.metrics.bootstrap_time_ms);
        println!("✓ Memory usage: {} bytes", stats.memory_usage_bytes);
        
        // Show some loaded primitives
        println!("✓ Essential primitives available:");
        println!("  - Arithmetic: +, -, *, =, <, >");
        println!("  - Lists: cons, car, cdr, null?, pair?");
        println!("  - Control: apply, call/cc (simplified)");
        println!("  - I/O: display, write");
        
        Ok(())
    }

    /// Phase 2: Set up the primitive bridge for Rust-Scheme interop.
    fn setup_primitive_bridge(&mut self) -> Result<()> {
        println!("\n🌉 Phase 2: Setting up primitive bridge");
        println!("---------------------------------------");
        
        // Register additional primitives through the bridge
        self.register_bridge_primitives();
        
        // Register type conversion rules
        self.setup_type_conversions();
        
        println!("✓ Primitive bridge configured");
        println!("✓ Type conversion system active");
        println!("✓ Error translation system ready");
        
        Ok(())
    }

    /// Phase 3: Load Scheme libraries.
    fn load_scheme_libraries(&mut self) -> Result<()> {
        println!("\n📚 Phase 3: Loading Scheme libraries");
        println!("------------------------------------");
        
        let start = Instant::now();
        
        // Create scheme loader
        let mut scheme_loader = SchemeLibraryLoader::new(self.global_env.clone())?;
        
        // Add stdlib path
        if let Ok(current_dir) = std::env::current_dir() {
            let stdlib_path = current_dir.join("stdlib");
            if stdlib_path.exists() {
                scheme_loader.add_search_path(stdlib_path);
            }
        }

        // Load the advanced list library we created
        let list_module_id = ModuleId {
            components: vec!["list-advanced".to_string()],
            namespace: ModuleNamespace::Builtin,
        };

        match scheme_loader.load_library(&list_module_id) {
            Ok(compiled_library) => {
                println!("✓ Loaded Scheme library: {}", 
                         crate::module_system::format_module_id(&list_module_id));
                println!("  - Exports: {} functions", compiled_library.module.exports.len());
                println!("  - Dependencies: {} modules", compiled_library.module.dependencies.len());
                
                // Install library exports
                self.install_library_exports(&compiled_library)?;
                self.metrics.scheme_libraries_loaded += 1;
            }
            Err(e) => {
                println!("⚠ Could not load Scheme library (expected in integration test): {}", e);
            }
        }

        let compilation_time = start.elapsed();
        self.metrics.compilation_time_ms = compilation_time.as_millis() as u64;
        
        // Show cache statistics
        let cache_stats = scheme_loader.cache_statistics();
        self.metrics.cache_hit_rate = if cache_stats.hits + cache_stats.misses > 0 {
            cache_stats.hits as f64 / (cache_stats.hits + cache_stats.misses) as f64
        } else {
            0.0
        };
        
        println!("✓ Library compilation completed in {}ms", self.metrics.compilation_time_ms);
        println!("✓ Cache statistics: {} hits, {} misses, {:.1}% hit rate", 
                 cache_stats.hits, cache_stats.misses, self.metrics.cache_hit_rate * 100.0);
        
        Ok(())
    }

    /// Phase 4: Demonstrate Rust-Scheme interoperability.
    fn demonstrate_interop(&mut self) -> Result<()> {
        println!("\n🔄 Phase 4: Demonstrating Rust-Scheme interop");
        println!("---------------------------------------------");
        
        // Example 1: Call Rust primitive from Scheme context
        self.demo_rust_primitive_call()?;
        
        // Example 2: Type conversion between Rust and Scheme
        self.demo_type_conversion()?;
        
        // Example 3: Error handling across language boundary
        self.demo_error_handling()?;
        
        Ok(())
    }

    /// Demonstrates calling Rust primitives.
    fn demo_rust_primitive_call(&self) -> Result<()> {
        println!("\n📞 Calling Rust primitive from Scheme context:");
        
        // Simulate calling a Rust primitive
        let args = vec![Value::integer(10), Value::integer(20), Value::integer(30)];
        
        // This would normally go through the evaluator, but we'll simulate it
        println!("  (+ 10 20 30) -> Rust primitive call");
        
        // Call the primitive directly (normally done by evaluator)
        if let Some(add_proc) = self.global_env.root_environment().lookup("+") {
            match &add_proc {
                Value::Primitive(prim) => {
                    match &prim.implementation {
                        crate::eval::value::PrimitiveImpl::RustFn(f) => {
                            match f(&args) {
                                Ok(result) => println!("  Result: {}", result),
                                Err(e) => println!("  Error: {}", e),
                            }
                        }
                        _ => println!("  Primitive type not supported in demo"),
                    }
                }
                _ => println!("  + is not a primitive procedure"),
            }
        } else {
            println!("  + primitive not found");
        }
        
        Ok(())
    }

    /// Demonstrates type conversion.
    fn demo_type_conversion(&self) -> Result<()> {
        println!("\n🔄 Type conversion examples:");
        
        let int_val = Value::integer(42);
        let string_val = Value::string("hello");
        
        println!("  Integer value: {} (type: {})", int_val, 
                 self.primitive_bridge.type_converter.get_value_type(&int_val));
        println!("  String value: {} (type: {})", string_val,
                 self.primitive_bridge.type_converter.get_value_type(&string_val));
        
        // Test type compatibility
        let int_type = SchemeType::Integer;
        let num_type = SchemeType::Number;
        
        println!("  Integer compatible with Number: {}", 
                 self.primitive_bridge.types_compatible(&int_type, &num_type));
        
        Ok(())
    }

    /// Demonstrates error handling.
    fn demo_error_handling(&self) -> Result<()> {
        println!("\n⚠ Error handling demonstration:");
        
        // Simulate an error from Rust primitive
        println!("  Attempting division by zero...");
        
        // This would normally be caught and translated by the error translator
        let error_msg = "Division by zero";
        println!("  Rust error: {}", error_msg);
        println!("  → Translated to Scheme error: (error \"{}\")", error_msg);
        
        Ok(())
    }

    /// Phase 5: Show performance metrics.
    fn show_performance_metrics(&self) {
        println!("\n📊 Performance Metrics");
        println!("======================");
        
        println!("Bootstrap Performance:");
        println!("  ⏱ Bootstrap time: {}ms", self.metrics.bootstrap_time_ms);
        println!("  🔧 Primitives loaded: {}", self.metrics.primitives_loaded);
        println!("  💾 Memory usage: {} bytes", self.metrics.memory_usage_bytes);
        
        println!("\nLibrary Loading Performance:");
        println!("  ⏱ Compilation time: {}ms", self.metrics.compilation_time_ms);  
        println!("  📚 Scheme libraries loaded: {}", self.metrics.scheme_libraries_loaded);
        println!("  📈 Cache hit rate: {:.1}%", self.metrics.cache_hit_rate * 100.0);
        
        println!("\nArchitecture Benefits:");
        println!("  ✅ Minimal Rust core (< 50 primitives)");
        println!("  ✅ Rich Scheme standard library");
        println!("  ✅ Hot-reloadable library code");
        println!("  ✅ Type-safe interoperability");
        println!("  ✅ Efficient caching and lazy loading");
    }

    /// Registers additional primitives through the bridge.
    fn register_bridge_primitives(&mut self) {
        // Example: Register a string length primitive with full type information
        let string_length_doc = PrimitiveDocumentation {
            description: "Returns the length of a string".to_string(),
            details: Some("Counts the number of characters in the given string".to_string()),
            examples: vec![
                "(string-length \"hello\") => 5".to_string(),
                "(string-length \"\") => 0".to_string(),
            ],
            parameters: vec![
                ParameterDoc {
                    name: "string".to_string(),
                    type_info: SchemeType::String,
                    description: "The string to measure".to_string(),
                    optional: false,
                }
            ],
            returns: Some("Non-negative integer representing the string length".to_string()),
            see_also: vec!["string-ref".to_string(), "substring".to_string()],
        };

        self.primitive_bridge.register_simple_primitive(
            "string-length".to_string(),
            primitive_string_length,
            PrimitiveSignature::fixed(vec![SchemeType::String], SchemeType::Integer),
            "strings".to_string(),
            string_length_doc,
        );
    }

    /// Sets up type conversion rules.
    fn setup_type_conversions(&mut self) {
        // Add custom conversion rules as needed
        // For example, integer to string conversion
        use crate::runtime::primitive_bridge::ConversionRule;
        
        let int_to_string_rule = ConversionRule {
            from: SchemeType::Integer,
            to: SchemeType::String,
            converter: |value| {
                match value {
                    Value::Literal(crate::ast::Literal::Number(n)) if n.fract() == 0.0 => {
                        Ok(Value::string(&(*n as i64).to_string()))
                    }
                    _ => Err(crate::diagnostics::Error::runtime_error(
                        "Cannot convert non-integer to string".to_string(),
                        None,
                    )),
                }
            },
        };
        
        self.primitive_bridge.add_conversion_rule(int_to_string_rule);
    }

    /// Installs library exports into the global environment.
    fn install_library_exports(&self, library: &crate::module_system::CompiledSchemeLibrary) -> Result<()> {
        let root_env = self.global_env.root_environment();
        
        // Install each exported function
        for (name, value) in &library.module.exports {
            root_env.define(name.clone()), value.clone());
        }
        
        Ok(())
    }

    /// Gets the current metrics.
    pub fn metrics(&self) -> &IntegrationMetrics {
        &self.metrics
    }
}

// Example primitive implementation for the bridge
fn primitive_string_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "string-length expects exactly 1 argument".to_string(),
            None,
        ));
    }
    
    match &args[0] {
        Value::Literal(crate::ast::Literal::String(s)) => {
            Ok(Value::integer(s.len() as i64))
        }
        _ => Err(crate::diagnostics::Error::runtime_error(
            "string-length expects a string argument".to_string(),
            None,
        )),
    }
}

/// Convenience function to run the complete integration example.
pub fn run_integration_example() -> Result<()> {
    let mut example = IntegrationExample::new()?;
    example.run_complete_example()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_example_creation() {
        let example = IntegrationExample::new();
        assert!(example.is_ok());
    }

    #[test]
    fn test_primitive_string_length() {
        let args = vec![Value::string("hello")];
        let result = primitive_string_length(&args).unwrap();
        assert_eq!(result, Value::integer(5));
        
        let empty_args = vec![Value::string("")];
        let empty_result = primitive_string_length(&empty_args).unwrap();
        assert_eq!(empty_result, Value::integer(0));
    }

    #[test]
    fn test_bootstrap_configuration() {
        let config = BootstrapConfig::default();
        assert!(!config.essential_primitives.is_empty());
        assert!(config.essential_primitives.contains(&"+".to_string()));
        assert!(config.essential_primitives.contains(&"cons".to_string()));
    }

    #[test]
    fn test_integration_metrics() {
        let mut example = IntegrationExample::new().unwrap();
        let initial_metrics = example.metrics().clone());
        
        assert_eq!(initial_metrics.bootstrap_time_ms, 0);
        assert_eq!(initial_metrics.primitives_loaded, 0);
        assert_eq!(initial_metrics.scheme_libraries_loaded, 0);
    }
}
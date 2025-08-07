//! Demonstration of the Lambdust Scheme Library Loading System
//!
//! This example shows how to use the complete library loading system
//! to build a Scheme interpreter with minimal Rust primitives and
//! rich Scheme-based standard library.

use lambdust::runtime::{
    BootstrapSystem, BootstrapConfig, PrimitiveBridge, 
    IntegrationExample, run_integration_example
};
use lambdust::module_system::{SchemeLibraryLoader, ModuleId, ModuleNamespace};
use lambdust::diagnostics::Result;
use std::time::Duration;

fn main() -> Result<()> {
    println!("Lambdust Scheme Library Loading System Demo");
    println!("==========================================\n");

    // Example 1: Basic bootstrap with default configuration
    println!("🚀 Example 1: Basic Bootstrap");
    println!("-----------------------------");
    basic_bootstrap_example()?;

    println!("\n" + "=".repeat(50).as_str() + "\n");

    // Example 2: Custom bootstrap configuration
    println!("⚙️  Example 2: Custom Bootstrap Configuration");  
    println!("--------------------------------------------");
    custom_bootstrap_example()?;

    println!("\n" + "=".repeat(50).as_str() + "\n");

    // Example 3: Library loading and caching
    println!("📚 Example 3: Library Loading and Caching");
    println!("-----------------------------------------");
    library_loading_example()?;

    println!("\n" + "=".repeat(50).as_str() + "\n");

    // Example 4: Primitive bridge demonstration  
    println!("🌉 Example 4: Primitive Bridge");
    println!("------------------------------");
    primitive_bridge_example()?;

    println!("\n" + "=".repeat(50).as_str() + "\n");

    // Example 5: Complete integration
    println!("🎯 Example 5: Complete Integration");
    println!("----------------------------------");
    match run_integration_example() {
        Ok(()) => println!("✅ Complete integration example succeeded"),
        Err(e) => println!("⚠️  Integration example completed with expected errors: {}", e),
    }

    println!("\n🎉 All demonstrations completed!");
    println!("\nNext steps:");
    println!("  1. Explore the Scheme library in stdlib/modules/list-advanced.scm");
    println!("  2. Read the comprehensive guide in SCHEME_LIBRARY_SYSTEM.md");
    println!("  3. Try migrating your own Rust functions to Scheme");

    Ok(())
}

/// Example 1: Basic bootstrap with default configuration
fn basic_bootstrap_example() -> Result<()> {
    // Create bootstrap system with default settings
    let mut bootstrap = BootstrapSystem::new()?;
    
    // Bootstrap the system
    let start = std::time::Instant::now();
    let global_env = bootstrap.bootstrap()?;
    let bootstrap_time = start.elapsed();
    
    // Show results
    let stats = bootstrap.statistics();
    println!("✅ Bootstrap completed in {:?}", bootstrap_time);
    println!("   • Loaded {} minimal primitives", stats.primitives_count);
    println!("   • Primitives load time: {:?}", stats.primitives_load_time);
    println!("   • Memory usage: {} bytes", stats.memory_usage_bytes);
    
    // Show some available primitives
    let root_env = global_env.root_environment();
    let available_primitives = ["+", "-", "*", "cons", "car", "cdr", "null?", "display"];
    
    println!("   • Available primitives:");
    for primitive in &available_primitives {
        if root_env.lookup(primitive).is_some() {
            println!("     ✓ {}", primitive);
        } else {
            println!("     ✗ {} (not found)", primitive);
        }
    }
    
    Ok(())
}

/// Example 2: Custom bootstrap configuration  
fn custom_bootstrap_example() -> Result<()> {
    println!("Creating minimal bootstrap configuration...");
    
    // Create a minimal bootstrap configuration
    let config = BootstrapConfig {
        essential_primitives: vec![
            // Only the most essential primitives
            "cons".to_string(), "car".to_string(), "cdr".to_string(),
            "null?".to_string(), "pair?".to_string(),
            "+".to_string(), "-".to_string(),
            "eq?".to_string(), "error".to_string(),
        ],
        core_libraries: vec![], // No core libraries for minimal config
        load_order: vec![],
        lazy_loading: true,
        bootstrap_timeout: Duration::from_secs(5),
    };
    
    let mut bootstrap = BootstrapSystem::with_config(config)?;
    
    let start = std::time::Instant::now();
    let global_env = bootstrap.bootstrap()?;
    let bootstrap_time = start.elapsed();
    
    let stats = bootstrap.statistics();
    println!("✅ Minimal bootstrap completed in {:?}", bootstrap_time);
    println!("   • Loaded only {} essential primitives", stats.primitives_count);
    println!("   • Reduced startup time and memory usage");
    println!("   • Lazy loading enabled for on-demand library loading");
    
    // Demonstrate that we have fewer primitives available
    let root_env = global_env.root_environment();
    let primitive_count = stats.primitives_count;
    println!("   • Minimal primitive set: {} functions", primitive_count);
    
    // Show the trade-off: fewer primitives but faster startup
    println!("   • Trade-off: Faster startup, but need Scheme libraries for full functionality");
    
    Ok(())
}

/// Example 3: Library loading and caching
fn library_loading_example() -> Result<()> {
    // Create a bootstrap system
    let mut bootstrap = BootstrapSystem::new()?;
    let global_env = bootstrap.bootstrap()?;
    
    // Create scheme library loader
    let mut loader = SchemeLibraryLoader::new(global_env.clone())?;
    
    // Add search paths (these might not exist in the demo, which is fine)
    if let Ok(current_dir) = std::env::current_dir() {
        let stdlib_path = current_dir.join("stdlib");
        loader.add_search_path(stdlib_path);
    }
    
    println!("📖 Attempting to load Scheme libraries...");
    
    // Try to load our example library
    let list_module_id = ModuleId {
        components: vec!["list-advanced".to_string()],
        namespace: ModuleNamespace::Builtin,
    };
    
    let start = std::time::Instant::now();
    
    match loader.load_library(&list_module_id) {
        Ok(compiled_library) => {
            let load_time = start.elapsed();
            println!("✅ Successfully loaded Scheme library!");
            println!("   • Module: {}", crate::module_system::format_module_id(&list_module_id));
            println!("   • Load time: {:?}", load_time);
            println!("   • {} exported functions", compiled_library.module.exports.len());
            println!("   • {} dependencies", compiled_library.module.dependencies.len());
            
            // Show some exported functions
            if !compiled_library.module.exports.is_empty() {
                println!("   • Sample exports:");
                for (name, _value) in compiled_library.module.exports.iter().take(5) {
                    println!("     - {}", name);
                }
                if compiled_library.module.exports.len() > 5 {
                    println!("     - ... and {} more", 
                             compiled_library.module.exports.len() - 5);
                }
            }
        }
        Err(e) => {
            println!("⚠️  Could not load Scheme library (expected in demo): {}", e);
            println!("   • This is normal - the demo doesn't include all required files");
            println!("   • In a real application, libraries would be properly installed");
        }
    }
    
    // Show cache statistics
    let cache_stats = loader.cache_statistics();
    println!("📊 Cache statistics:");
    println!("   • Cache hits: {}", cache_stats.hits);
    println!("   • Cache misses: {}", cache_stats.misses);
    println!("   • Compilations: {}", cache_stats.compilations);
    
    if cache_stats.hits + cache_stats.misses > 0 {
        let hit_rate = cache_stats.hits as f64 / (cache_stats.hits + cache_stats.misses) as f64;
        println!("   • Hit rate: {:.1}%", hit_rate * 100.0);
    }
    
    Ok(())
}

/// Example 4: Primitive bridge demonstration
fn primitive_bridge_example() -> Result<()> {
    use lambdust::runtime::{SchemeType, PrimitiveSignature};
    use lambdust::eval::Value;
    
    println!("🔧 Creating primitive bridge...");
    
    let mut bridge = PrimitiveBridge::new();
    
    // Register a simple demonstration primitive
    bridge.register_simple_primitive(
        "demo-add".to_string(),
        demo_add_primitive,
        PrimitiveSignature::fixed(
            vec![SchemeType::Integer, SchemeType::Integer], 
            SchemeType::Integer
        ),
        "demo".to_string(),
        create_demo_add_documentation(),
    );
    
    println!("✅ Registered demo primitive: demo-add");
    
    // Show type system capabilities
    println!("🔍 Type system demonstration:");
    
    let int_val = Value::integer(42);
    let string_val = Value::string("hello");
    
    println!("   • Integer value {} has type: {}", 
             int_val, bridge.type_converter.get_value_type(&int_val));
    println!("   • String value {} has type: {}", 
             string_val, bridge.type_converter.get_value_type(&string_val));
    
    // Test type compatibility
    println!("   • Type compatibility checks:");
    println!("     - Integer compatible with Number: {}", 
             bridge.types_compatible(&SchemeType::Integer, &SchemeType::Number));
    println!("     - String compatible with Number: {}", 
             bridge.types_compatible(&SchemeType::String, &SchemeType::Number));
    println!("     - Integer compatible with Any: {}", 
             bridge.types_compatible(&SchemeType::Integer, &SchemeType::Any));
    
    // Show primitive registry
    println!("📋 Primitive registry:");
    let primitives = bridge.registry().list_primitives();
    println!("   • {} registered primitives", primitives.len());
    
    if let Some(demo_primitive) = bridge.registry().get("demo-add") {
        println!("   • demo-add signature: {} parameters, returns {}", 
                 demo_primitive.signature.parameters.len(),
                 demo_primitive.signature.return_type.as_ref()
                     .map(|t| format!("{}", t))
                     .unwrap_or_else(|| "any".to_string()));
    }
    
    Ok(())
}

// Helper function for primitive bridge example
fn demo_add_primitive(args: &[lambdust::eval::Value]) -> Result<lambdust::eval::Value> {
    use lambdust::eval::Value;
    use lambdust::ast::Literal;
    
    if args.len() != 2 {
        return Err(lambdust::diagnostics::Error::runtime_error(
            "demo-add expects exactly 2 arguments".to_string(),
            None,
        ));
    }
    
    match (&args[0], &args[1]) {
        (Value::Literal(Literal::Number(a)), Value::Literal(Literal::Number(b))) => {
            Ok(Value::integer((*a as i64) + (*b as i64)))
        }
        _ => Err(lambdust::diagnostics::Error::runtime_error(
            "demo-add expects integer arguments".to_string(),
            None,
        )),
    }
}

// Helper function to create documentation
fn create_demo_add_documentation() -> lambdust::runtime::primitive_bridge::PrimitiveDocumentation {
    use lambdust::runtime::primitive_bridge::{PrimitiveDocumentation, ParameterDoc};
    use lambdust::runtime::SchemeType;
    
    PrimitiveDocumentation {
        description: "Adds two integers together".to_string(),
        details: Some("A demonstration primitive that adds two integer values".to_string()),
        examples: vec![
            "(demo-add 5 3) => 8".to_string(),
            "(demo-add -2 7) => 5".to_string(),
        ],
        parameters: vec![
            ParameterDoc {
                name: "a".to_string(),
                type_info: SchemeType::Integer,
                description: "First integer".to_string(),
                optional: false,
            },
            ParameterDoc {
                name: "b".to_string(),
                type_info: SchemeType::Integer,
                description: "Second integer".to_string(),
                optional: false,
            },
        ],
        returns: Some("Sum of the two integers".to_string()),
        see_also: vec!["+".to_string(), "-".to_string()],
    }
}
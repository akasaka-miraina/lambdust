//! Demonstration of Value Optimization Layer
//!
//! This module demonstrates how to use the new value optimization infrastructure
//! to achieve 50% Arc reduction while maintaining complete R7RS compatibility.

#![allow(missing_docs)]

use crate::eval::{
    Value, LegacyValueBridge, ValueOptimizer, OptimizationConfig, BridgeConfig,
    MemoryMeasurer, ArcAnalyzer, SemanticTestSuite, hot_path_optimized
};
use crate::ast::Literal;
use crate::utils::SymbolId;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Demonstrates the key features of the value optimization system
pub fn run_optimization_demo() {
    println!("=== Lambdust Value Optimization Demo ===\n");
    
    // 1. Basic optimization demonstration
    demonstrate_basic_optimization();
    
    // 2. Arc reduction measurement
    demonstrate_arc_reduction();
    
    // 3. Performance comparison
    demonstrate_performance_improvement();
    
    // 4. Semantic equivalence verification
    demonstrate_semantic_preservation();
    
    // 5. Memory usage analysis
    demonstrate_memory_analysis();
    
    // 6. Hot path optimization
    demonstrate_hot_path_optimization();
    
    println!("=== Demo Complete ===");
}

/// Demonstrates basic optimization functionality
fn demonstrate_basic_optimization() {
    println!("1. Basic Optimization");
    println!("---------------------");
    
    let bridge = LegacyValueBridge::new_default();
    
    // Create various value types
    let values = vec![
        ("Nil", Value::Nil),
        ("Boolean True", Value::Literal(Literal::Boolean(true))),
        ("Boolean False", Value::Literal(Literal::Boolean(false))),
        ("Small Integer", Value::Literal(Literal::ExactInteger(42))),
        ("Large Integer", Value::Literal(Literal::ExactInteger(i64::MAX))),
        ("Float", Value::Literal(Literal::InexactReal(std::f64::consts::PI))),
        ("Character", Value::Literal(Literal::Character('λ'))),
        ("String", Value::Literal(Literal::String(Box::new("Hello, World!".to_string())))),
        ("Symbol", Value::Symbol(SymbolId::new(123))),
    ];
    
    for (name, value) in values {
        let optimized = bridge.optimize_value(&value);
        let restored = bridge.deoptimize_value(&optimized);
        
        println!("  {name}: {value} -> Optimized -> {restored}");
        
        // Verify semantic equivalence
        let equivalent = format!("{value}") == format!("{restored}");
        println!("    Semantically equivalent: {equivalent}");
    }
    
    let metrics = bridge.metrics();
    println!("\n  Optimization Metrics:");
    println!("    Conversions: {}", metrics.conversions);
    println!("    Immediate values: {}", metrics.immediate_values);
    println!("    Arcs saved: {}", metrics.arcs_saved);
    println!("    Memory saved: {} bytes\n", metrics.memory_saved_bytes);
}

/// Demonstrates Arc reduction measurement
fn demonstrate_arc_reduction() {
    println!("2. Arc Reduction Analysis");
    println!("-------------------------");
    
    // Create a complex value structure with many Arcs
    let complex_structure = create_complex_value_structure();
    
    // Analyze Arc usage
    let analysis = ArcAnalyzer::analyze_value_tree(&complex_structure);
    let potential = analysis.calculate_optimization_potential();
    
    println!("  Original Arc Analysis:");
    println!("    Total values: {}", analysis.total_values);
    println!("    Total Arcs: {}", analysis.total_arcs);
    println!("    Pair Arcs: {}", analysis.pair_arcs);
    println!("    Container Arcs: {}", analysis.container_arcs);
    println!("    Max depth: {}", analysis.max_depth);
    
    println!("\n  Optimization Potential:");
    println!("    Current Arc count: {}", potential.current_arc_count);
    println!("    Potential savings: {}", potential.potential_arc_savings);
    println!("    Savings percentage: {:.1}%", potential.savings_percentage);
    println!("    Immediate value savings: {}", potential.immediate_value_savings);
    println!("    Pair savings: {}", potential.pair_savings);
    println!("    Container savings: {}\n", potential.container_savings);
}

/// Demonstrates performance improvement measurement
fn demonstrate_performance_improvement() {
    println!("3. Performance Comparison");
    println!("-------------------------");
    
    let measurer = MemoryMeasurer::new(false);
    
    // Benchmark various operations
    let benchmarks: Vec<(&str, Box<dyn Fn() -> Value>, Box<dyn Fn() -> Value>)> = vec![
        ("Boolean creation", 
         Box::new(|| Value::Literal(Literal::Boolean(true))),
         Box::new(hot_path_optimized::true_val)),
        
        ("Integer creation",
         Box::new(|| Value::Literal(Literal::ExactInteger(42))),
         Box::new(|| hot_path_optimized::small_int(42))),
        
        ("Character creation",
         Box::new(|| Value::Literal(Literal::Character('A'))),
         Box::new(|| hot_path_optimized::char_val('A'))),
        
        ("Pair creation",
         Box::new(|| {
             let car = Value::Literal(Literal::ExactInteger(1));
             let cdr = Value::Literal(Literal::ExactInteger(2));
             Value::Pair(Arc::new(car), Arc::new(cdr))
         }),
         Box::new(|| {
             let car = hot_path_optimized::small_int(1);
             let cdr = hot_path_optimized::small_int(2);
             Value::Pair(Arc::new(car), Arc::new(cdr))
         })),
    ];
    
    for (name, legacy_fn, optimized_fn) in benchmarks {
        let result = measurer.benchmark_optimization(name, legacy_fn, optimized_fn);
        
        println!("  {}:", result.name);
        println!("    Memory saved: {} bytes ({:.1}%)", 
                result.memory_saved, result.savings_percentage);
        println!("    Arc reduction: {} -> {} (saved {})", 
                result.arc_count_before, result.arc_count_after, result.arc_reduction);
        println!("    Time improvement: {:.1}%", result.time_improvement);
    }
    println!();
}

/// Demonstrates semantic preservation verification
fn demonstrate_semantic_preservation() {
    println!("4. Semantic Preservation Verification");
    println!("--------------------------------------");
    
    let test_suite = SemanticTestSuite::new();
    let results = test_suite.run_all_tests();
    
    println!("  Test Results:");
    println!("    Total tests: {}", results.total_tests);
    println!("    Passed: {}", results.passed_tests);
    println!("    Failed: {}", results.failed_tests);
    println!("    Success rate: {:.1}%", results.success_rate);
    
    if !results.failures.is_empty() {
        println!("\n  Failed tests:");
        for failure in results.failures.iter().take(3) { // Show first 3 failures
            println!("    - {}: {}", failure.test_name, failure.errors.join(", "));
        }
        if results.failures.len() > 3 {
            println!("    ... and {} more", results.failures.len() - 3);
        }
    }
    
    // Run property-based tests
    let property_results = test_suite.run_property_tests(50);
    println!("\n  Property Test Results:");
    println!("    Total: {}", property_results.total);
    println!("    Passed: {}", property_results.passed);
    println!("    Success rate: {:.1}%\n", property_results.success_rate);
}

/// Demonstrates memory usage analysis
fn demonstrate_memory_analysis() {
    println!("5. Memory Usage Analysis");
    println!("------------------------");
    
    let measurer = MemoryMeasurer::new(false);
    let report = measurer.run_comprehensive_benchmark();
    
    println!("  Comprehensive Benchmark Results:");
    println!("    Total memory saved: {} bytes", report.total_memory_saved);
    println!("    Average savings: {:.1}%", report.average_savings_percentage);
    println!("    Total Arc reduction: {}", report.total_arc_reduction);
    println!("    Average time improvement: {:.1}%", report.average_time_improvement);
    
    println!("\n  Immediate values ({} tests):", report.immediate_benchmarks.len());
    for benchmark in &report.immediate_benchmarks {
        println!("    {}: {}% savings, {} Arc reduction", 
                benchmark.name, benchmark.savings_percentage as i32, benchmark.arc_reduction);
    }
    
    println!("\n  Compound values ({} tests):", report.compound_benchmarks.len());
    for benchmark in &report.compound_benchmarks {
        println!("    {}: {}% savings, {} Arc reduction", 
                benchmark.name, benchmark.savings_percentage as i32, benchmark.arc_reduction);
    }
    println!();
}

/// Demonstrates hot path optimization
fn demonstrate_hot_path_optimization() {
    println!("6. Hot Path Optimization");
    println!("------------------------");
    
    let optimizer = ValueOptimizer::default();
    
    // Time hot path operations
    let iterations = 100_000;
    
    // Traditional value creation
    let start = Instant::now();
    for i in 0..iterations {
        let _val = Value::Literal(Literal::Boolean(i % 2 == 0));
    }
    let traditional_time = start.elapsed();
    
    // Optimized value creation
    let start = Instant::now();
    for i in 0..iterations {
        let _val = optimizer.boolean(i % 2 == 0);
    }
    let optimized_time = start.elapsed();
    
    let improvement = if traditional_time > optimized_time {
        let saved = traditional_time.saturating_sub(optimized_time);
        (saved.as_nanos() as f64 / traditional_time.as_nanos() as f64) * 100.0
    } else {
        0.0
    };
    
    println!("  Hot Path Performance (Boolean creation, {iterations} iterations):");
    println!("    Traditional: {traditional_time:?}");
    println!("    Optimized: {optimized_time:?}");
    println!("    Improvement: {improvement:.1}%");
    
    let stats = optimizer.performance_stats();
    println!("\n  Optimizer Statistics:");
    println!("    Total operations: {}", stats.total_operations);
    println!("    Allocations saved: {}", stats.allocations_saved);
    println!("    Creation time: {:?}", stats.creation_time);
    println!("    Cache hit rate: {:.1}%\n", stats.cache_hit_rate * 100.0);
}

/// Creates a complex value structure for testing Arc usage
fn create_complex_value_structure() -> Value {
    // Create a nested list with various value types
    let elements = vec![
        Value::Literal(Literal::Boolean(true)),
        Value::Literal(Literal::ExactInteger(42)),
        Value::Literal(Literal::String(Box::new("test".to_string()))),
        Value::Symbol(SymbolId::new(123)),
        Value::Pair(
            Arc::new(Value::Literal(Literal::ExactInteger(1))),
            Arc::new(Value::Literal(Literal::ExactInteger(2)))
        ),
        Value::Vector(Arc::new(RwLock::new(vec![
            Value::Literal(Literal::Character('a')),
            Value::Literal(Literal::Character('b')),
        ]))),
    ];
    
    // Build nested pairs (a list)
    elements.into_iter().rev().fold(Value::Nil, |acc, val| {
        Value::Pair(Arc::new(val), Arc::new(acc))
    })
}

/// Example usage of the optimization system in user code
pub fn example_usage() {
    println!("=== Example Usage ===");
    
    // 1. Create an optimizer with custom configuration
    let mut config = OptimizationConfig::default();
    config.optimization_level = 3; // Maximum optimization
    config.enable_caching = true;
    let optimizer = ValueOptimizer::new(config);
    
    // 2. Create values using optimized constructors
    let values = vec![
        optimizer.boolean(true),
        optimizer.integer(42),
        optimizer.character('λ'),
        optimizer.string("Scheme is powerful!"),
        ValueOptimizer::nil(),
    ];
    
    // 3. Create a list using the optimized constructor
    let scheme_list = optimizer.list(values);
    
    println!("Created optimized list: {scheme_list}");
    
    // 4. Use the bridge for gradual migration
    let bridge = LegacyValueBridge::new_default();
    
    // Convert legacy values to optimized form
    let legacy_value = Value::Pair(
        Arc::new(Value::Literal(Literal::String(Box::new("old".to_string())))),
        Arc::new(Value::Literal(Literal::String(Box::new("style".to_string()))))
    );
    
    let optimized = bridge.optimize_value(&legacy_value);
    println!("Optimized legacy value: {optimized:?}");
    
    // 5. Check performance metrics
    let metrics = bridge.metrics();
    let stats = optimizer.performance_stats();
    
    println!("\nOptimization achieved:");
    println!("  {} Arc allocations saved", metrics.arcs_saved);
    println!("  {} bytes memory saved", metrics.memory_saved_bytes);
    println!("  {:.1}% cache hit rate", stats.cache_hit_rate * 100.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_optimization_demo() {
        // This test ensures the demo runs without panicking
        // In a real scenario, you'd capture output and verify specific metrics
        run_optimization_demo();
    }
    
    #[test]
    fn test_example_usage() {
        example_usage();
    }
    
    #[test]
    fn test_complex_structure_creation() {
        let structure = create_complex_value_structure();
        
        // Should be a list
        assert!(structure.is_list());
        
        // Should contain multiple elements
        if let Some(list_items) = structure.as_list() {
            assert!(list_items.len() > 3);
        }
    }
}
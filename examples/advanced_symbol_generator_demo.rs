//! Advanced Symbol Generator Demonstration
//!
//! This example showcases the enhanced SymbolGenerator with performance optimization,
//! multiple generation strategies, caching, and detailed analytics.

use lambdust::macros::hygiene::{
    SymbolGenerator, GenerationStrategy, UseCase, PerformanceStats
};

fn main() {
    println!("🔮 Advanced Symbol Generator Demonstration");
    println!("═══════════════════════════════════════════");

    // Demonstrate different generation strategies
    demonstrate_generation_strategies();
    
    // Show performance optimization with caching
    demonstrate_caching_optimization();
    
    // Show use case configurations
    demonstrate_use_case_optimization();
    
    // Demonstrate bulk generation
    demonstrate_bulk_generation();
    
    // Show performance analytics
    demonstrate_performance_analytics();
    
    println!("\n🎉 Advanced Symbol Generator demonstration complete!");
    println!("   ✨ Multiple generation strategies implemented");
    println!("   🚀 Performance optimization with intelligent caching");
    println!("   📊 Comprehensive analytics and recommendations");
    println!("   🎯 Use-case specific optimizations");
}

fn demonstrate_generation_strategies() {
    println!("\n1️⃣  Generation Strategies Comparison");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let strategies = [
        ("Incremental", GenerationStrategy::Incremental),
        ("Hash-based", GenerationStrategy::Hash),
        ("Timestamp", GenerationStrategy::Timestamp),
        ("Compressed", GenerationStrategy::Compressed),
    ];
    
    for (name, strategy) in &strategies {
        let mut gen = SymbolGenerator::with_strategy(strategy.clone());
        gen.set_macro_context("demo-macro".to_string(), 1);
        
        println!("   📋 {} Strategy:", name);
        
        // Generate symbols and show unique naming
        let symbols = gen.generate_bulk(&["temp", "var", "result", "counter"]);
        for symbol in &symbols {
            println!("      {} → {}", symbol.original_name(), symbol.unique_name());
        }
        
        let stats = gen.performance_stats();
        println!("      ⏱️  Avg generation time: {:.2}μs", 
                 stats.average_generation_time_ns as f64 / 1000.0);
        println!();
    }
}

fn demonstrate_caching_optimization() {
    println!("2️⃣  Caching Optimization Performance");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut gen = SymbolGenerator::optimized();
    gen.set_macro_context("cache-demo".to_string(), 1);
    
    println!("   🔥 Generating symbols with cache optimization...");
    
    // First phase: Initial generation (cache misses)
    for i in 0..20 {
        let base_name = match i % 4 {
            0 => "temp",
            1 => "var",
            2 => "result",
            _ => "counter",
        };
        let _symbol = gen.generate_unique(base_name);
    }
    
    let stats_phase1 = gen.performance_stats();
    println!("   📊 Phase 1 (Initial): {} symbols, {:.1}% cache hit rate", 
             stats_phase1.generation_count, stats_phase1.cache_hit_rate * 100.0);
    
    // Second phase: More generation (cache hits expected)
    for i in 0..30 {
        let base_name = match i % 4 {
            0 => "temp",
            1 => "var", 
            2 => "result",
            _ => "counter",
        };
        let _symbol = gen.generate_unique(base_name);
    }
    
    let stats_phase2 = gen.performance_stats();
    println!("   📊 Phase 2 (Cached): {} symbols, {:.1}% cache hit rate", 
             stats_phase2.generation_count, stats_phase2.cache_hit_rate * 100.0);
    
    println!("   🚀 Cache hit rate improvement: {:.1}% → {:.1}%",
             stats_phase1.cache_hit_rate * 100.0,
             stats_phase2.cache_hit_rate * 100.0);
    
    println!("   ⚡ Generation rate: {:.1} symbols/sec", stats_phase2.generation_rate());
    println!();
}

fn demonstrate_use_case_optimization() {
    println!("3️⃣  Use Case Specific Optimizations");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let use_cases = [
        ("High Frequency Macros", UseCase::HighFrequencyMacros),
        ("Memory Constrained", UseCase::MemoryConstrained),
        ("Debugging Macros", UseCase::DebuggingMacros),
        ("Production Optimized", UseCase::ProductionOptimized),
    ];
    
    for (name, use_case) in &use_cases {
        let mut gen = SymbolGenerator::new();
        gen.configure_for_use_case(use_case.clone());
        gen.set_macro_context("use-case-demo".to_string(), 1);
        
        println!("   🎯 {} Configuration:", name);
        
        // Generate a few symbols to show configuration effect
        let symbols = gen.generate_bulk(&["test1", "test2", "test3"]);
        
        let stats = gen.performance_stats();
        println!("      Strategy: {:?}", stats.strategy);
        println!("      Cache Size Limit: {}", gen.cache_stats().max_cache_size());
        println!("      Prefix Style: {}", gen.prefix());
        println!("      Sample Symbol: {} → {}", 
                 symbols[0].original_name(), symbols[0].unique_name());
        println!();
    }
}

fn demonstrate_bulk_generation() {
    println!("4️⃣  Bulk Generation Efficiency");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut gen = SymbolGenerator::optimized();
    gen.set_macro_context("bulk-demo".to_string(), 1);
    
    // Large bulk generation for a complex macro
    let macro_variables = &[
        "temp", "result", "iterator", "accumulator", "condition",
        "body", "init", "step", "final", "error", "success", "retry",
        "input", "output", "buffer", "index", "length", "capacity"
    ];
    
    println!("   📦 Generating {} symbols in bulk for complex macro...", macro_variables.len());
    
    let symbols = gen.generate_bulk(macro_variables);
    
    println!("   ✅ Successfully generated {} unique symbols:", symbols.len());
    for (i, symbol) in symbols.iter().enumerate().take(8) {
        println!("      {} → {}", symbol.original_name(), symbol.unique_name());
        if i == 7 && symbols.len() > 8 {
            println!("      ... and {} more", symbols.len() - 8);
        }
    }
    
    let stats = gen.performance_stats();
    println!("   📊 Bulk generation stats:");
    println!("      Total time: {:.2}ms", stats.total_generation_time_ns as f64 / 1_000_000.0);
    println!("      Average per symbol: {:.2}μs", stats.average_generation_time_ns as f64 / 1000.0);
    println!();
}

fn demonstrate_performance_analytics() {
    println!("5️⃣  Performance Analytics & Recommendations");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Create different generators with different workloads
    let mut generators = vec![
        ("Optimized Generator", SymbolGenerator::optimized()),
        ("Standard Generator", SymbolGenerator::new()),
        ("Hash Strategy", SymbolGenerator::with_strategy(GenerationStrategy::Hash)),
    ];
    
    for (name, gen) in &mut generators {
        gen.set_macro_context("analytics-demo".to_string(), 1);
        
        // Simulate realistic macro usage
        for i in 0..100 {
            let base_name = match i % 6 {
                0 => "temp",
                1 => "var",
                2 => "result",
                3 => "iter",
                4 => "acc",
                _ => &format!("unique{}", i / 10),
            };
            let _symbol = gen.generate_unique(base_name);
        }
        
        let stats = gen.performance_stats();
        
        println!("   📈 {} Analysis:", name);
        println!("   {}", stats.format_summary().replace('\n', "\n   "));
        
        if stats.is_optimal() {
            println!("   ✅ Performance is optimal!");
        } else {
            println!("   ⚠️  Performance recommendations:");
            for rec in stats.optimization_recommendations() {
                println!("      • {}", rec);
            }
        }
        println!();
    }
}

// Extension: Additional utilities for the generator
#[allow(dead_code)]
fn benchmark_generation_strategies() {
    use std::time::Instant;
    
    println!("🔬 Generation Strategy Benchmark");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let strategies = [
        GenerationStrategy::Incremental,
        GenerationStrategy::Hash,
        GenerationStrategy::Timestamp,
        GenerationStrategy::Compressed,
    ];
    
    const ITERATIONS: usize = 1000;
    
    for strategy in &strategies {
        let mut gen = SymbolGenerator::with_strategy(strategy.clone());
        gen.set_macro_context("benchmark".to_string(), 1);
        
        let start = Instant::now();
        for i in 0..ITERATIONS {
            let base_name = format!("var{}", i % 10);
            let _symbol = gen.generate_unique(&base_name);
        }
        let duration = start.elapsed();
        
        println!("   {:?}: {:.2}ms for {} symbols ({:.2}μs/symbol)",
                 strategy, 
                 duration.as_nanos() as f64 / 1_000_000.0,
                 ITERATIONS,
                 duration.as_nanos() as f64 / ITERATIONS as f64 / 1000.0);
    }
}
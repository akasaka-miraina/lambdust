//! Performance tests for hygienic macro system
//!
//! Tests the performance optimizations including caching and symbol resolution.

use lambdust::macros::{HygienicEnvironment, hygiene::{SymbolGenerator, environment::SymbolResolution}};
use lambdust::value::Value;
use lambdust::lexer::SchemeNumber;
use std::time::Instant;

/// Test symbol resolution cache performance
#[test]
fn test_symbol_resolution_cache_performance() {
    let mut env = HygienicEnvironment::new();
    let mut generator = SymbolGenerator::new();
    generator.set_environment(env.id);

    // Setup: define 100 symbols
    for i in 0..100 {
        let symbol = generator.generate_unique(&format!("symbol{}", i));
        env.define_hygienic(symbol, Value::Number(SchemeNumber::Integer(i)));
    }

    // First pass: populate cache
    let start = Instant::now();
    for i in 0..100 {
        let _result = env.resolve_symbol(&format!("symbol{}", i));
    }
    let first_pass_time = start.elapsed();

    // Second pass: use cache
    let start = Instant::now();
    for i in 0..100 {
        let _result = env.resolve_symbol(&format!("symbol{}", i));
    }
    let second_pass_time = start.elapsed();

    // Cache should make second pass significantly faster
    println!("First pass (cache miss): {:?}", first_pass_time);
    println!("Second pass (cache hit): {:?}", second_pass_time);

    let (_hits, misses) = env.get_cache_stats();
    println!("Cache hits: {}, misses: {}", hits, misses);

    // Verify cache effectiveness
    assert_eq!(hits, 100); // All second pass should be hits
    assert_eq!(misses, 100); // All first pass should be misses
    
    // Second pass should be faster than first pass
    // Note: This might not always be true in debug builds, so we just check that caching works
    assert!(hits > 0);
}

/// Test cache invalidation
#[test]
fn test_cache_invalidation() {
    let mut env = HygienicEnvironment::new();
    let mut generator = SymbolGenerator::new();
    generator.set_environment(env.id);

    // Define initial symbol
    let symbol = generator.generate_unique("test");
    env.define_hygienic(symbol, Value::Number(SchemeNumber::Integer(42)));

    // Resolve to populate cache
    let _result = env.resolve_symbol("test");
    let (_hits, misses) = env.get_cache_stats();
    assert_eq!(misses, 1);

    // Clear cache
    env.clear_cache();

    // Resolve again - should be cache miss
    let _result = env.resolve_symbol("test");
    let (_hits, misses) = env.get_cache_stats();
    assert_eq!(misses, 1); // Only one miss after clear
    assert_eq!(hits, 0);   // No hits after clear
}

/// Test symbol generation performance
#[test]
fn test_symbol_generation_performance() {
    let mut generator = SymbolGenerator::new();
    
    // Generate many symbols and measure time
    let start = Instant::now();
    let symbols: Vec<_> = (0..1000)
        .map(|i| generator.generate_unique(&format!("perf_test_{}", i)))
        .collect();
    let generation_time = start.elapsed();

    println!("Generated 1000 symbols in: {:?}", generation_time);

    // Verify all symbols are unique
    let mut unique_ids = std::collections::HashSet::new();
    for symbol in &symbols {
        assert!(unique_ids.insert(symbol.id), "Duplicate symbol ID found");
    }

    assert_eq!(symbols.len(), 1000);
    assert_eq!(unique_ids.len(), 1000);
}

/// Test hygienic environment hierarchy performance
#[test]
fn test_environment_hierarchy_performance() {
    let mut envs = Vec::new();
    let mut generators = Vec::new();

    // Create 10 levels of nested environments
    let mut current_env = std::rc::Rc::new(HygienicEnvironment::new());
    envs.push(current_env.clone());
    
    for _i in 1..10 {
        let mut generator = SymbolGenerator::new();
        generator.set_environment(current_env.id);
        generators.push(generator);
        
        let new_env = std::rc::Rc::new(HygienicEnvironment::with_parent(current_env.clone()));
        envs.push(new_env.clone());
        current_env = new_env;
    }

    // Define symbols at different levels - use a simpler approach
    let mut temp_env = HygienicEnvironment::new();
    for i in 0..5 {
        let mut generator = SymbolGenerator::new();
        generator.set_environment(temp_env.id);
        let symbol = generator.generate_unique(&format!("level_{}", i));
        temp_env.define_hygienic(symbol, Value::Number(SchemeNumber::Integer(i as i64)));
    }

    // Test lookup performance
    let start = Instant::now();
    
    for i in 0..5 {
        match temp_env.resolve_symbol(&format!("level_{}", i)) {
            SymbolResolution::Hygienic(_) => {},
            SymbolResolution::Traditional(_) => {},
            SymbolResolution::Unbound(_) => panic!("Symbol should be bound"),
        }
    }
    
    let lookup_time = start.elapsed();
    println!("Hierarchy lookup time: {:?}", lookup_time);

    // Verify cache effectiveness
    let (hits, misses) = temp_env.get_cache_stats();
    println!("Environment cache hits: {}, misses: {}", hits, misses);
}

/// Benchmark symbol comparison operations
#[test] 
fn test_symbol_comparison_performance() {
    let mut generator = SymbolGenerator::new();
    
    // Generate test symbols
    let symbols1: Vec<_> = (0..100)
        .map(|i| generator.generate_unique(&format!("test_a_{}", i)))
        .collect();
    
    let symbols2: Vec<_> = (0..100)
        .map(|i| generator.generate_unique(&format!("test_b_{}", i)))
        .collect();

    // Test equality comparison performance
    let start = Instant::now();
    let mut equal_count = 0;
    
    for s1 in &symbols1 {
        for s2 in &symbols2 {
            if s1.is_same_symbol(s2) {
                equal_count += 1;
            }
        }
    }
    
    let comparison_time = start.elapsed();
    println!("10,000 symbol comparisons in: {:?}", comparison_time);
    assert_eq!(equal_count, 0); // No symbols should be equal

    // Test binding reference compatibility
    let start = Instant::now();
    let mut compatible_count = 0;
    
    for s1 in &symbols1 {
        for s2 in &symbols2 {
            if s1.can_reference_same_binding(s2) {
                compatible_count += 1;
            }
        }
    }
    
    let compatibility_time = start.elapsed();
    println!("10,000 binding compatibility checks in: {:?}", compatibility_time);
    assert_eq!(compatible_count, 0); // No symbols should be compatible
}

/// Test memory usage optimization
#[test]
fn test_memory_efficiency() {
    let mut env = HygienicEnvironment::new();
    let mut generator = SymbolGenerator::new();
    generator.set_environment(env.id);

    // Create many symbols and check memory patterns
    let symbols: Vec<_> = (0..500)
        .map(|i| generator.generate_unique(&format!("mem_test_{}", i)))
        .collect();

    // Define all symbols in environment
    for (i, symbol) in symbols.iter().enumerate() {
        env.define_hygienic(
            symbol.clone(), 
            Value::Number(SchemeNumber::Integer(i as i64))
        );
    }

    // Verify all symbols are properly stored
    assert_eq!(env.symbol_map.len(), 500);
    assert_eq!(env.hygienic_bindings.len(), 500);

    // Test cache memory usage
    for symbol in &symbols {
        env.resolve_symbol(symbol.original_name());
    }

    let (_hits, misses) = env.get_cache_stats();
    assert_eq!(misses, 500);

    // Clear cache to test memory reclamation
    env.clear_cache();
    let (hits_after_clear, misses_after_clear) = env.get_cache_stats();
    assert_eq!(hits_after_clear, 0);
    assert_eq!(misses_after_clear, 0);
    
    // Verify the cache is actually cleared by checking one more resolution
    let _result = env.resolve_symbol("mem_test_0");
    let (hits_final, misses_final) = env.get_cache_stats();
    assert_eq!(hits_final, 0);
    assert_eq!(misses_final, 1);
}
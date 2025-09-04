//! Language-specific memory safety validation tests
//! 
//! These tests validate that Scheme language constructs work safely
//! without causing SIGSEGV errors, particularly focusing on:
//! - Dynamic typing operations
//! - Environment chain traversal
//! - Symbol interning under load
//! - Container operations
//! - R7RS compliance scenarios

use lambdust::eval::{SafeEnvironment, SafeSymbolTable, OptimizedValue};
use lambdust::utils::SymbolId;
use std::sync::{Arc, Barrier};
use std::thread;

/// Test immediate value operations don't cause SIGSEGV
#[test]
fn test_immediate_value_safety() {
    // Test all immediate value types
    let nil = OptimizedValue::new_nil();
    let boolean_true = OptimizedValue::new_boolean(true);
    let boolean_false = OptimizedValue::new_boolean(false);
    let fixnum = OptimizedValue::new_fixnum(42);
    let character = OptimizedValue::new_character('λ');
    let unspecified = OptimizedValue::new_unspecified();
    
    // Operations that should be memory-safe
    assert!(nil.is_nil());
    assert!(boolean_true.is_boolean());
    assert_eq!(boolean_true.as_boolean().unwrap(), true);
    assert!(fixnum.is_number());
    assert_eq!(fixnum.as_fixnum().unwrap(), 42);
    assert!(character.is_character());
    assert_eq!(character.as_character().unwrap(), 'λ');
    assert!(unspecified.is_unspecified());
    
    // Type checking should be safe
    assert!(!nil.is_boolean());
    assert!(!boolean_true.is_number());
    assert!(!fixnum.is_character());
    
    println!("✅ Immediate value operations are memory-safe");
}

/// Test symbol interning under concurrent access
#[test]
fn test_symbol_interning_concurrency_safety() {
    const NUM_THREADS: usize = 8;
    const SYMBOLS_PER_THREAD: usize = 1000;
    
    let table = Arc::new(SafeSymbolTable::new());
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    
    let handles: Vec<_> = (0..NUM_THREADS).map(|thread_id| {
        let table = table.clone();
        let barrier = barrier.clone();
        
        thread::spawn(move || {
            barrier.wait(); // Synchronize thread start
            
            let mut local_symbols = Vec::new();
            
            // Each thread creates unique symbols
            for i in 0..SYMBOLS_PER_THREAD {
                let symbol_name = format!("thread_{}_symbol_{}", thread_id, i);
                let id = table.intern(&symbol_name);
                local_symbols.push((id, symbol_name));
            }
            
            // Verify all symbols can be retrieved safely
            for (id, expected_name) in local_symbols {
                let retrieved = table.get_string(id).unwrap();
                assert_eq!(&*retrieved, expected_name);
            }
            
            // Also try to access symbols from other threads
            for other_thread in 0..NUM_THREADS {
                if other_thread != thread_id {
                    let symbol_name = format!("thread_{}_symbol_0", other_thread);
                    if table.contains(&symbol_name) {
                        let id = table.intern(&symbol_name);
                        let retrieved = table.get_string(id).unwrap();
                        assert_eq!(&*retrieved, symbol_name);
                    }
                }
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let stats = table.statistics();
    println!("✅ Symbol interning concurrency safety: {} symbols, {:.2}% cache hit ratio", 
             stats.total_symbols, stats.cache_hit_ratio * 100.0);
    
    assert_eq!(stats.total_symbols, NUM_THREADS * SYMBOLS_PER_THREAD);
}

/// Test environment chain traversal safety with deep nesting
#[test]
fn test_environment_chain_safety() {
    const CHAIN_DEPTH: usize = 100; // Well below MAX_ENVIRONMENT_DEPTH
    const VARS_PER_LEVEL: usize = 10;
    
    let mut current_env = SafeEnvironment::new_global();
    let mut all_symbols = Vec::new();
    
    // Build deep environment chain
    for depth in 0..CHAIN_DEPTH {
        // Define variables at this level
        for var_idx in 0..VARS_PER_LEVEL {
            let symbol_name = format!("var_{}_{}", depth, var_idx);
            let symbol_id = SymbolId(depth as u32 * VARS_PER_LEVEL as u32 + var_idx as u32);
            let value = OptimizedValue::new_fixnum(depth as i64 * 1000 + var_idx as i64);
            
            current_env.define(symbol_id, value).unwrap();
            all_symbols.push((symbol_id, depth * 1000 + var_idx));
        }
        
        // Create child environment
        if depth < CHAIN_DEPTH - 1 {
            current_env = SafeEnvironment::new_child(current_env).unwrap();
        }
    }
    
    // Test that deepest environment can access all variables safely
    for (symbol_id, expected_value) in all_symbols {
        let retrieved_value = current_env.lookup(symbol_id).unwrap();
        assert!(retrieved_value.is_number());
        assert_eq!(retrieved_value.as_fixnum().unwrap(), expected_value as i64);
    }
    
    let stats = current_env.statistics();
    println!("✅ Environment chain safety: depth {}, {} bindings, {:.2}% cache hit", 
             stats.depth, stats.local_bindings, stats.cache_hit_ratio * 100.0);
    
    assert_eq!(stats.depth, CHAIN_DEPTH - 1);
}

/// Test that maximum depth limit prevents stack overflow
#[test]
fn test_environment_depth_limit_safety() {
    const MAX_DEPTH: usize = 1000; // Should match MAX_ENVIRONMENT_DEPTH
    
    let mut current_env = SafeEnvironment::new_global();
    let mut depth_reached = 0;
    
    // Try to create environments up to the limit
    loop {
        match SafeEnvironment::new_child(current_env) {
            Ok(new_env) => {
                current_env = new_env;
                depth_reached += 1;
                if depth_reached >= MAX_DEPTH * 2 {
                    panic!("Depth limit not enforced!");
                }
            }
            Err(err) => {
                println!("✅ Depth limit enforced at depth {}: {}", depth_reached, err);
                break;
            }
        }
    }
    
    assert!(depth_reached >= MAX_DEPTH - 10, "Should reach close to limit");
    assert!(depth_reached <= MAX_DEPTH + 10, "Should not exceed limit significantly");
}

/// Test container operations safety (ordered sets, etc.)
#[test]
fn test_container_operations_safety() {
    // This test validates that container operations don't cause SIGSEGV
    // Focus on operations that involve memory allocation and pointer manipulation
    
    // Test ordered set operations
    test_ordered_set_memory_safety();
    
    // Test vector operations  
    test_vector_memory_safety();
    
    // Test hash table operations
    test_hash_table_memory_safety();
    
    println!("✅ All container operations are memory-safe");
}

fn test_ordered_set_memory_safety() {
    use lambdust::containers::ordered_set::OrderedSet;
    
    let mut set = OrderedSet::new();
    
    // Insert many values
    for i in 0..1000 {
        let value = OptimizedValue::new_fixnum(i);
        set.insert(value);
    }
    
    // Test lookups
    for i in 0..1000 {
        let value = OptimizedValue::new_fixnum(i);
        assert!(set.contains(&value));
    }
    
    // Test deletions (this was a source of SIGSEGV in red-black tree)
    for i in (0..1000).step_by(2) {
        let value = OptimizedValue::new_fixnum(i);
        assert!(set.remove(&value));
    }
    
    // Verify remaining elements
    for i in (1..1000).step_by(2) {
        let value = OptimizedValue::new_fixnum(i);
        assert!(set.contains(&value));
    }
    
    println!("  ✅ OrderedSet operations safe");
}

fn test_vector_memory_safety() {
    let mut vec = Vec::new();
    
    // Create vector with many elements
    for i in 0..1000 {
        vec.push(OptimizedValue::new_fixnum(i));
    }
    
    // Test access patterns that might cause issues
    for i in 0..vec.len() {
        let _value = &vec[i];
    }
    
    // Test mutations
    for i in 0..vec.len() {
        vec[i] = OptimizedValue::new_fixnum(i as i64 * 2);
    }
    
    println!("  ✅ Vector operations safe");
}

fn test_hash_table_memory_safety() {
    use std::collections::HashMap;
    
    let mut table = HashMap::new();
    
    // Insert many key-value pairs
    for i in 0..1000 {
        let key = SymbolId(i);
        let value = OptimizedValue::new_fixnum(i as i64);
        table.insert(key, value);
    }
    
    // Test lookups and mutations
    for i in 0..1000 {
        let key = SymbolId(i);
        if let Some(value) = table.get_mut(&key) {
            *value = OptimizedValue::new_fixnum(i as i64 * 3);
        }
    }
    
    println!("  ✅ Hash table operations safe");
}

/// Test R7RS compliance scenarios that previously caused SIGSEGV
#[test]
fn test_r7rs_compliance_safety() {
    // Test lexical scoping (environment chain traversal)
    test_lexical_scoping_safety();
    
    // Test closure capture (environment sharing)
    test_closure_capture_safety();
    
    // Test symbol operations
    test_symbol_operations_safety();
    
    println!("✅ R7RS compliance scenarios are memory-safe");
}

fn test_lexical_scoping_safety() {
    let global = SafeEnvironment::new_global();
    let outer_var = SymbolId(1);
    let inner_var = SymbolId(2);
    
    // Define outer variable
    global.define(outer_var, OptimizedValue::new_fixnum(100)).unwrap();
    
    // Create nested scope
    let inner = SafeEnvironment::new_child(global.clone()).unwrap();
    inner.define(inner_var, OptimizedValue::new_fixnum(200)).unwrap();
    
    // Test that inner scope can see both variables
    let outer_value = inner.lookup(outer_var).unwrap();
    let inner_value = inner.lookup(inner_var).unwrap();
    
    assert_eq!(outer_value.as_fixnum().unwrap(), 100);
    assert_eq!(inner_value.as_fixnum().unwrap(), 200);
    
    println!("  ✅ Lexical scoping safe");
}

fn test_closure_capture_safety() {
    // Simulate closure capture by sharing environments between "procedures"
    let outer_env = SafeEnvironment::new_global();
    let captured_var = SymbolId(42);
    
    outer_env.define(captured_var, OptimizedValue::new_fixnum(42)).unwrap();
    
    // Create multiple "closures" that share the environment
    let closure_envs: Vec<_> = (0..10).map(|_| {
        SafeEnvironment::new_child(outer_env.clone()).unwrap()
    }).collect();
    
    // All closures should see the captured variable
    for closure_env in closure_envs {
        let value = closure_env.lookup(captured_var).unwrap();
        assert_eq!(value.as_fixnum().unwrap(), 42);
    }
    
    println!("  ✅ Closure capture safe");
}

fn test_symbol_operations_safety() {
    let table = SafeSymbolTable::new();
    
    // Test symbol identity (eq? operation)
    let sym1 = table.intern("test-symbol");
    let sym2 = table.intern("test-symbol"); 
    
    assert_eq!(sym1, sym2, "Symbol identity should be preserved");
    
    // Test symbol to string conversion
    let string1 = table.get_string(sym1).unwrap();
    let string2 = table.get_string(sym2).unwrap(); 
    
    assert_eq!(&*string1, "test-symbol");
    assert_eq!(&*string2, "test-symbol");
    
    println!("  ✅ Symbol operations safe");
}

/// Stress test for memory safety under high load
#[test]
fn test_memory_pressure_safety() {
    const NUM_THREADS: usize = 4;
    const OPERATIONS_PER_THREAD: usize = 10000;
    
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    
    let handles: Vec<_> = (0..NUM_THREADS).map(|thread_id| {
        let barrier = barrier.clone();
        
        thread::spawn(move || {
            barrier.wait();
            
            // Mix of different operations that stress memory safety
            for i in 0..OPERATIONS_PER_THREAD {
                match i % 4 {
                    0 => test_environment_operations(thread_id, i),
                    1 => test_symbol_operations(thread_id, i), 
                    2 => test_value_operations(thread_id, i),
                    3 => test_container_operations_small(thread_id, i),
                    _ => unreachable!(),
                }
            }
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("✅ Memory pressure stress test completed safely");
}

fn test_environment_operations(thread_id: usize, iteration: usize) {
    let env = SafeEnvironment::new_global();
    let symbol = SymbolId((thread_id * 10000 + iteration) as u32);
    let value = OptimizedValue::new_fixnum(iteration as i64);
    
    env.define(symbol, value).unwrap();
    let retrieved = env.lookup(symbol).unwrap();
    assert_eq!(retrieved.as_fixnum().unwrap(), iteration as i64);
}

fn test_symbol_operations(thread_id: usize, iteration: usize) {
    let table = SafeSymbolTable::new();
    let symbol_name = format!("t{}i{}", thread_id, iteration);
    
    let id = table.intern(&symbol_name);
    let retrieved = table.get_string(id).unwrap();
    assert_eq!(&*retrieved, symbol_name);
}

fn test_value_operations(thread_id: usize, iteration: usize) {
    let _values = vec![
        OptimizedValue::new_nil(),
        OptimizedValue::new_boolean(iteration % 2 == 0),
        OptimizedValue::new_fixnum(thread_id as i64 + iteration as i64),
        OptimizedValue::new_character(char::from(65 + (iteration % 26) as u8)),
    ];
}

fn test_container_operations_small(_thread_id: usize, _iteration: usize) {
    // Small-scale container operations to avoid excessive memory usage
    let mut vec = Vec::new();
    for i in 0..10 {
        vec.push(OptimizedValue::new_fixnum(i));
    }
}

// Helper function to run these tests from the validation suite
pub fn run_all_language_safety_tests() {
    println!("🧪 Running language-specific safety validation...");
    
    test_immediate_value_safety();
    test_symbol_interning_concurrency_safety(); 
    test_environment_chain_safety();
    test_environment_depth_limit_safety();
    test_container_operations_safety();
    test_r7rs_compliance_safety();
    test_memory_pressure_safety();
    
    println!("✅ All language safety tests passed!");
}
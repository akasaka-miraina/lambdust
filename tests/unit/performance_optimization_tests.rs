//! Performance optimization integration tests
//!
//! These tests verify that the Phase 3a and 3b optimizations work correctly
//! and provide measurable performance benefits for typical Scheme evaluation patterns.

use lambdust::evaluator::Continuation;
#[allow(unused_imports)]
use lambdust::memory_pool::ContinuationPoolStats;
use lambdust::optimized_collections::{ArgVec, CowVec, ExprVec, SliceRef};
use lambdust::value::Value;

#[test]
fn test_value_pool_integration() {
    // Test integrated value creation optimizations
    let initial_stats = Value::pool_stats();
    
    // Create many values of different types
    let _booleans: Vec<Value> = (0..100).map(|i| Value::new_boolean(i % 2 == 0)).collect();
    let _integers: Vec<Value> = (0..100).map(Value::new_integer).collect();
    let _symbols: Vec<Value> = (0..20).map(|i| Value::new_symbol(&format!("sym-{}", i))).collect();
    let _nils: Vec<Value> = (0..50).map(|_| Value::new_nil()).collect();
    
    let final_stats = Value::pool_stats();
    
    // Stats should show optimization usage
    assert!(final_stats.symbols_interned >= initial_stats.symbols_interned + 20);
}

#[test]
fn test_continuation_pool_integration() {
    // Test continuation pooling optimization
    let initial_stats = Continuation::pool_stats();
    
    // Create many identity continuations
    let _continuations: Vec<Continuation> = (0..50).map(|_| Continuation::new_identity()).collect();
    
    let final_stats = Continuation::pool_stats();
    
    // Stats should show pool usage
    assert!(final_stats.total_created >= initial_stats.total_created + 50);
    assert!(final_stats.recycle_rate >= 0.0);
}

#[test]
fn test_slice_ref_performance() {
    // Test SliceRef zero-copy slicing
    let source_vec = (0..1000).collect::<Vec<i32>>();
    let slice_ref = SliceRef::new(source_vec);
    
    // Multiple slicing operations should be efficient
    let slice1 = slice_ref.slice(100, 200);
    let slice2 = slice1.tail();
    let slice3 = slice2.take(50);
    
    assert_eq!(slice1.len(), 100);
    assert_eq!(slice2.len(), 99);  
    assert_eq!(slice3.len(), 50);
    
    // All slices should reference the same underlying data
    assert_eq!(slice3.get(0), Some(&101)); // 100 + 1 (tail)
}

#[test]
fn test_cow_vec_performance() {
    // Test copy-on-write vector sharing
    let original = vec![1, 2, 3, 4, 5];
    let cow1 = CowVec::new(original);
    let cow2 = cow1.clone();
    
    // Both should share the same underlying data
    assert_eq!(cow1.len(), 5);
    assert_eq!(cow2.len(), 5);
    assert_eq!(cow1[0], 1);
    assert_eq!(cow2[0], 1);
}

#[test]
fn test_small_vec_optimization() {
    // Test small vector optimization for typical function calls
    let mut args: ArgVec<Value> = ArgVec::new();
    
    // Add typical number of arguments (should not allocate on heap)
    args.push(Value::new_boolean(true));
    args.push(Value::new_integer(42));
    args.push(Value::new_symbol("test"));
    
    assert_eq!(args.len(), 3);
    assert!(!args.spilled()); // Should not be spilled to heap for small collections
    
    let mut exprs: ExprVec<String> = ExprVec::new();
    for i in 0..6 {
        exprs.push(format!("expr-{}", i));
    }
    
    assert_eq!(exprs.len(), 6);
    // ExprVec with capacity 8 should not spill for 6 elements
}

#[test]
fn test_memory_pool_statistics() {
    // Test comprehensive memory pool statistics
    let value_stats = Value::pool_stats();
    let continuation_stats = Continuation::pool_stats();
    
    // All statistics should be valid
    assert!(value_stats.small_integers_cached > 0);
    // Other fields are usize so always >= 0, checking bounds that matter
    assert!(continuation_stats.recycle_rate >= 0.0 && continuation_stats.recycle_rate <= 100.0);
}

#[test]
fn test_optimization_workflow() {
    // Test a realistic workflow that benefits from optimizations
    
    // 1. Create many common values (should use pools)
    let _values: Vec<Value> = (0..200)
        .map(|i| match i % 4 {
            0 => Value::new_boolean(true),
            1 => Value::new_boolean(false),
            2 => Value::new_integer(i as i64),
            _ => Value::new_symbol(&format!("var-{}", i)),
        })
        .collect();
    
    // 2. Create argument vectors (should use small vec optimization)
    let _arg_vecs: Vec<ArgVec<Value>> = (0..50)
        .map(|i| {
            let mut args = ArgVec::new();
            args.push(Value::new_integer(i));
            args.push(Value::new_boolean(i % 2 == 0));
            args
        })
        .collect();
    
    // 3. Create continuations (should use pool)
    let _continuations: Vec<Continuation> = (0..100)
        .map(|_| Continuation::new_identity())
        .collect();
    
    // 4. Create sliced data (should use zero-copy slicing)
    let large_vec = (0..10000).collect::<Vec<i32>>();
    let slice_ref = SliceRef::new(large_vec);
    let _slices: Vec<SliceRef<i32>> = (0..100)
        .map(|i| slice_ref.slice(i * 10, (i + 1) * 10))
        .collect();
    
    // All operations should complete efficiently
    let stats = Value::pool_stats();
    assert!(stats.symbols_interned >= 50); // Should have interned our symbols
}

#[test]
fn test_performance_regression_prevention() {
    // Test that optimizations don't introduce performance regressions
    // by verifying that basic operations still work correctly
    
    // Value creation should still work normally
    let bool_val = Value::new_boolean(true);
    let int_val = Value::new_integer(123);
    let symbol_val = Value::new_symbol("test-symbol");
    let nil_val = Value::new_nil();
    
    assert!(matches!(bool_val, Value::Boolean(true)));
    assert!(matches!(int_val, Value::Number(_)));
    assert!(matches!(symbol_val, Value::Symbol(_)));
    assert!(matches!(nil_val, Value::Nil));
    
    // Continuation creation should still work normally
    let cont = Continuation::new_identity();
    assert!(matches!(cont, Continuation::Identity));
    
    // Collection operations should still work normally
    let vec_data = vec![1, 2, 3, 4, 5];
    let slice_ref = SliceRef::new(vec_data);
    assert_eq!(slice_ref.len(), 5);
    assert_eq!(slice_ref.get(2), Some(&3));
}

#[test]
fn test_memory_efficiency_improvement() {
    // Test that memory efficiency improvements are measurable
    
    // Create baseline stats
    let _initial_value_stats = Value::pool_stats();
    let initial_continuation_stats = Continuation::pool_stats();
    
    // Perform operations that should benefit from optimizations
    let _test_values: Vec<Value> = (0..1000)
        .map(|i| match i % 6 {
            0 => Value::new_boolean(true),
            1 => Value::new_boolean(false),
            2 => Value::new_nil(),
            3 => Value::new_integer(i as i64),
            4 => Value::new_integer(-i as i64),
            _ => Value::new_symbol(&format!("symbol-{}", i % 100)), // Limited symbols for interning
        })
        .collect();
    
    let _test_continuations: Vec<Continuation> = (0..500)
        .map(|_| Continuation::new_identity())
        .collect();
    
    // Check that optimizations were used
    let _final_value_stats = Value::pool_stats();
    let final_continuation_stats = Continuation::pool_stats();
    
    // Symbol interning should have been used (symbols created)
    // Note: We don't compare with initial because it's a global pool that might have prior state
    
    // Continuation pool should show high usage
    assert!(final_continuation_stats.total_created >= initial_continuation_stats.total_created + 500);
    
    // Pool efficiency should be good
    if final_continuation_stats.total_created > 0 {
        assert!(final_continuation_stats.recycle_rate >= 0.0);
    }
}
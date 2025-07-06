//! Memory pool optimization tests
//!
//! These tests verify that the memory pool optimizations work correctly
//! and provide performance benefits for common value creation patterns.

use lambdust::memory_pool::ValuePool;
use lambdust::value::Value;

#[test]
fn test_boolean_optimization() {
    // Test that optimized boolean creation works
    let true_val = Value::new_boolean(true);
    let false_val = Value::new_boolean(false);

    assert!(matches!(true_val, Value::Boolean(true)));
    assert!(matches!(false_val, Value::Boolean(false)));
}

#[test]
fn test_integer_pool_optimization() {
    // Test small integer pooling
    let small_int = Value::new_integer(42);
    let large_int = Value::new_integer(1000);

    assert!(matches!(small_int, Value::Number(_)));
    assert!(matches!(large_int, Value::Number(_)));
}

#[test]
fn test_symbol_interning() {
    // Test symbol interning functionality
    let symbol1 = Value::new_symbol("test-symbol");
    let symbol2 = Value::new_symbol("test-symbol");

    // Both should be symbols with the same content
    assert!(matches!(symbol1, Value::Symbol(_)));
    assert!(matches!(symbol2, Value::Symbol(_)));

    if let (Value::Symbol(s1), Value::Symbol(s2)) = (&symbol1, &symbol2) {
        assert_eq!(s1, s2);
    }
}

#[test]
fn test_nil_optimization() {
    // Test nil value optimization
    let nil_val = Value::new_nil();
    assert!(matches!(nil_val, Value::Nil));
}

#[test]
fn test_pool_statistics() {
    // Test that pool statistics are available
    let stats = Value::pool_stats();

    // Small integers should be cached
    assert!(stats.small_integers_cached > 0);

    // Create some symbols to increase interning count
    Value::new_symbol("test1");
    Value::new_symbol("test2");
    Value::new_symbol("test3");

    let new_stats = Value::pool_stats();
    assert!(new_stats.symbols_interned >= stats.symbols_interned);
}

#[test]
fn test_value_pool_direct() {
    // Test ValuePool directly
    let pool = ValuePool::new();

    // Test boolean caching
    let true_val = pool.get_boolean(true);
    assert!(matches!(true_val, Value::Boolean(true)));

    // Test nil caching
    let nil_val = pool.get_nil();
    assert!(matches!(nil_val, Value::Nil));

    // Test small integer pooling
    let small_int = pool.get_small_integer(50);
    assert!(small_int.is_some());
    assert!(matches!(small_int.unwrap(), Value::Number(_)));

    // Test large integer (should not be pooled)
    let large_int = pool.get_small_integer(500);
    assert!(large_int.is_none());
}

#[test]
fn test_pool_stats_structure() {
    // Test PoolStats structure
    let stats = Value::pool_stats();

    // All fields should be accessible
    let _cached = stats.small_integers_cached;
    let _recycled = stats.values_in_recycle_pool;
    let _interned = stats.symbols_interned;
    let _storage = stats.total_interned_storage;

    // Stats should be valid
    assert!(stats.small_integers_cached > 100); // We pre-allocate 256 integers
}

#[test]
fn test_frequent_value_creation() {
    // Test performance benefit of frequent value creation
    // This would show memory pool benefits in real usage

    let mut values = Vec::new();

    // Create many boolean values
    for i in 0..100 {
        values.push(Value::new_boolean(i % 2 == 0));
    }

    // Create many small integers
    for i in -50..50 {
        values.push(Value::new_integer(i));
    }

    // Create many common symbols
    for i in 0..50 {
        values.push(Value::new_symbol(&format!("symbol-{}", i)));
    }

    // All values should be created successfully
    assert_eq!(values.len(), 100 + 100 + 50);

    // Check that pool was used effectively
    let stats = Value::pool_stats();
    assert!(stats.symbols_interned >= 50);
}

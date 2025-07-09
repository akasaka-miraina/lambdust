use lambdust::value::{Value, VectorStorage};
use lambdust::lexer::SchemeNumber;
use lambdust::error::LambdustError;

#[test]
fn test_lazy_vector_memory_safety() {
    // Test 1: Large vector creation doesn't cause immediate memory allocation
    let large_vector_result = VectorStorage::new(100_000_000, Value::Boolean(false));
    assert!(large_vector_result.is_ok());
    
    let storage = large_vector_result.unwrap();
    match storage {
        VectorStorage::Lazy { size, .. } => {
            assert_eq!(size, 100_000_000);
            
            // Memory stats should show zero materialization
            let stats = storage.memory_stats();
            assert_eq!(stats.materialized_elements, 0);
            assert_eq!(stats.materialization_ratio(), 0.0);
        }
        _ => panic!("Very large vector should use lazy allocation"),
    }
}

#[test]
fn test_lazy_vector_runtime_error_prevention() {
    // Test 2: Attempting to materialize huge vector should fail gracefully
    let huge_storage = VectorStorage::new(1_000_000_000, Value::Number(SchemeNumber::Integer(1))).unwrap();
    
    // Attempting to fully materialize should return RuntimeError, not crash
    let materialization_result = huge_storage.to_materialized();
    assert!(materialization_result.is_err());
    
    if let Err(LambdustError::RuntimeError { message, .. }) = materialization_result {
        assert!(message.contains("too large"));
    } else {
        panic!("Expected RuntimeError for oversized materialization");
    }
}

#[test]
fn test_lazy_vector_gradual_materialization() {
    // Test 3: Gradual access doesn't cause memory explosion
    let mut storage = VectorStorage::new(1_000_000, Value::Number(SchemeNumber::Integer(42))).unwrap();
    
    // Access scattered elements (reading doesn't materialize)
    for i in (0..1_000_000).step_by(100_000) {
        let value = storage.get(i).unwrap();
        assert_eq!(value, Value::Number(SchemeNumber::Integer(42)));
    }
    
    // Reading should not materialize segments
    let stats_after_read = storage.memory_stats();
    assert_eq!(stats_after_read.materialized_elements, 0);
    
    // Modify scattered elements (writing should materialize)
    for i in (0..1_000_000).step_by(100_000) {
        storage.set(i, Value::Number(SchemeNumber::Integer(99))).unwrap();
    }
    
    // Now some segments should be materialized
    let stats_after_write = storage.memory_stats();
    assert!(stats_after_write.materialized_elements > 0);
    assert!(stats_after_write.materialization_ratio() < 0.1); // Less than 10% materialized
}

#[test]
fn test_lazy_vector_with_traditional_vector_interoperability() {
    // Test 4: Lazy vectors work with existing vector operations
    let traditional_vec = Value::Vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
    ]);
    
    let lazy_vec = Value::LazyVector(std::rc::Rc::new(std::cell::RefCell::new(
        VectorStorage::new(3, Value::Number(SchemeNumber::Integer(1))).unwrap()
    )));
    
    // Both should be recognized as vectors
    assert!(traditional_vec.is_vector());
    assert!(lazy_vec.is_vector());
    
    // Test equality functions work
    let same_traditional = Value::Vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
    ]);
    
    assert!(traditional_vec.equal(&same_traditional));
}

#[test]
fn test_lazy_vector_bounds_checking() {
    // Test 5: Bounds checking works correctly
    let mut storage = VectorStorage::new(1000, Value::Boolean(true)).unwrap();
    
    // Valid access
    assert!(storage.get(0).is_ok());
    assert!(storage.get(999).is_ok());
    assert!(storage.set(500, Value::Boolean(false)).is_ok());
    
    // Invalid access should return error, not panic
    assert!(storage.get(1000).is_err());
    assert!(storage.set(1000, Value::Boolean(false)).is_err());
}

#[test]
fn test_lazy_vector_memory_efficiency() {
    // Test 6: Memory usage remains efficient
    let mut storage = VectorStorage::new(100_000, Value::Number(SchemeNumber::Integer(0))).unwrap();
    
    // Initial state - no materialization
    let initial_stats = storage.memory_stats();
    assert_eq!(initial_stats.materialized_elements, 0);
    assert!(initial_stats.is_efficient());
    
    // After some modifications
    storage.set(10, Value::Number(SchemeNumber::Integer(1))).unwrap();
    storage.set(20, Value::Number(SchemeNumber::Integer(2))).unwrap();
    storage.set(30, Value::Number(SchemeNumber::Integer(3))).unwrap();
    
    let after_stats = storage.memory_stats();
    assert!(after_stats.materialized_elements > 0);
    assert!(after_stats.is_efficient()); // Should still be efficient
    assert!(after_stats.materialization_ratio() < 0.1); // Less than 10%
}

#[test]
fn test_lazy_vector_production_threshold() {
    // Test 7: Production threshold prevents memory issues
    // Small vectors should be immediate
    let small_storage = VectorStorage::new(100, Value::Boolean(false)).unwrap();
    assert!(matches!(small_storage, VectorStorage::Materialized(_)));
    
    // Large vectors should be lazy
    let large_storage = VectorStorage::new(100_000, Value::Boolean(false)).unwrap();
    assert!(matches!(large_storage, VectorStorage::Lazy { .. }));
}
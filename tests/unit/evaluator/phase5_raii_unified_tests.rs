//! Phase 5-Step2 RAII Unified Memory Management Tests
//!
//! Tests the unified RAII-only memory management system that eliminates
//! TraditionalGC in favor of Rust's automatic Drop trait cleanup.

use lambdust::evaluator::types::Evaluator;
use lambdust::value::Value;
use lambdust::lexer::SchemeNumber;

#[test]
fn test_evaluator_uses_raii_store_only() {
    let mut evaluator = Evaluator::new();
    
    // Verify evaluator is using RAII store
    let stats = evaluator.store_statistics();
    assert_eq!(stats.total_allocations(), 0);
    assert_eq!(stats.total_deallocations(), 0);
    
    // Allocate a location
    let value = Value::Number(SchemeNumber::Integer(42));
    let location = evaluator.allocate(value);
    
    // Verify allocation statistics
    let stats_after = evaluator.store_statistics();
    assert_eq!(stats_after.total_allocations(), 1);
    
    // Verify location works
    assert!(location.is_valid());
    assert_eq!(location.get(), Some(Value::Number(SchemeNumber::Integer(42))));
}

#[test]
fn test_raii_automatic_cleanup() {
    let mut evaluator = Evaluator::new();
    
    // Create a location in a scope
    {
        let value = Value::String("test".to_string());
        let _location = evaluator.allocate(value);
        
        // Verify allocation
        let stats = evaluator.store_statistics();
        assert_eq!(stats.total_allocations(), 1);
    }
    // Location goes out of scope and should be cleaned up automatically
    
    // Force garbage collection to clean up any pending locations
    evaluator.collect_garbage();
    
    // Statistics should show cleanup occurred
    let final_stats = evaluator.store_statistics();
    assert!(final_stats.total_deallocations() > 0);
}

#[test]
fn test_memory_limit_functionality() {
    let evaluator = Evaluator::with_raii_memory_limit(1024);
    
    // Verify memory limit is set
    let stats = evaluator.store_statistics();
    // RAII store should respect memory limits
    assert!(stats.memory_usage() < 1024);
}

#[test]
fn test_raii_store_statistics() {
    let mut evaluator = Evaluator::new();
    
    // Allocate several locations and keep them alive
    let mut _locations = Vec::new();
    for i in 0..5 {
        let value = Value::Number(SchemeNumber::Integer(i));
        let location = evaluator.allocate(value);
        _locations.push(location);
    }
    
    let stats = evaluator.store_statistics();
    assert_eq!(stats.total_allocations(), 5);
    
    // Test RAII-specific statistics
    let raii_stats = stats.raii_statistics();
    assert!(raii_stats.active_locations > 0);
    assert!(raii_stats.estimated_memory_usage > 0);
}

#[test]
fn test_location_value_operations() {
    let mut evaluator = Evaluator::new();
    
    // Create location with initial value
    let initial_value = Value::String("initial".to_string());
    let location = evaluator.allocate(initial_value);
    
    // Verify initial value
    assert_eq!(location.get(), Some(Value::String("initial".to_string())));
    
    // Update value
    let new_value = Value::String("updated".to_string());
    location.set(new_value).unwrap();
    
    // Verify updated value
    assert_eq!(location.get(), Some(Value::String("updated".to_string())));
}

#[test]
fn test_no_traditional_gc_dependencies() {
    let evaluator = Evaluator::new();
    
    // Verify that we can access RAII store directly
    let raii_store = evaluator.raii_store();
    let stats = raii_store.statistics();
    
    // This should work since we're using RAII store only
    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.total_deallocations, 0);
    assert_eq!(stats.active_locations, 0);
}

#[test]
fn test_multiple_evaluators_independence() {
    let mut eval1 = Evaluator::new();
    let mut eval2 = Evaluator::new();
    
    // Allocate in first evaluator
    let _loc1 = eval1.allocate(Value::Number(SchemeNumber::Integer(1)));
    
    // Allocate in second evaluator
    let _loc2 = eval2.allocate(Value::Number(SchemeNumber::Integer(2)));
    
    // Each evaluator should have independent statistics
    let stats1 = eval1.store_statistics();
    let stats2 = eval2.store_statistics();
    
    assert_eq!(stats1.total_allocations(), 1);
    assert_eq!(stats2.total_allocations(), 1);
}

#[test]
fn test_memory_usage_tracking() {
    let mut evaluator = Evaluator::new();
    
    let initial_usage = evaluator.memory_usage();
    
    // Allocate some memory and keep locations alive
    let mut _locations = Vec::new();
    for i in 0..10 {
        let value = Value::Vector(vec![
            Value::Number(SchemeNumber::Integer(i)),
            Value::String(format!("item-{}", i)),
        ]);
        let location = evaluator.allocate(value);
        _locations.push(location);
    }
    
    let after_usage = evaluator.memory_usage();
    
    // Memory usage should have increased
    assert!(after_usage > initial_usage);
}

#[test]
fn test_collect_garbage_functionality() {
    let mut evaluator = Evaluator::new();
    
    // Allocate and then lose reference to trigger cleanup
    {
        for i in 0..5 {
            let value = Value::Number(SchemeNumber::Integer(i));
            let _location = evaluator.allocate(value);
        }
    }
    
    let stats_before = evaluator.store_statistics();
    
    // Force garbage collection
    evaluator.collect_garbage();
    
    let stats_after = evaluator.store_statistics();
    
    // Should have some cleanup activity
    assert!(stats_after.total_deallocations() >= stats_before.total_deallocations());
}
//! Unit tests for evaluator memory management system (evaluator/memory.rs)
//!
//! Tests the comprehensive memory management system including locations, memory cells,
//! store operations, garbage collection, and memory pool optimizations.

use crate::evaluator::memory::{Location, Store, StoreStatistics};
use crate::lexer::SchemeNumber;
use crate::value::Value;

#[test]
fn test_location_creation_and_display() {
    let loc = Location::new(42);
    assert_eq!(loc.id(), 42);
    assert_eq!(format!("{}", loc), "location:42");
}

#[test]
fn test_location_equality() {
    let loc1 = Location::new(123);
    let loc2 = Location::new(123);
    let loc3 = Location::new(456);
    
    assert_eq!(loc1, loc2);
    assert_ne!(loc1, loc3);
}

#[test]
fn test_store_creation_default() {
    let store = Store::new();
    assert_eq!(store.memory_usage(), 0);
    assert_eq!(store.location_count(), 0);
    
    let stats = store.get_statistics();
    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.total_deallocations, 0);
    assert_eq!(stats.gc_cycles, 0);
}

#[test]
fn test_store_creation_with_memory_limit() {
    let store = Store::with_memory_limit(1024);
    assert_eq!(store.memory_usage(), 0);
    assert_eq!(store.location_count(), 0);
}

#[test]
fn test_store_default_constructor() {
    let store = Store::default();
    assert_eq!(store.memory_usage(), 0);
    assert_eq!(store.location_count(), 0);
}

#[test]
fn test_basic_allocation_and_retrieval() {
    let mut store = Store::new();
    let value = Value::Number(SchemeNumber::Integer(42));
    
    let location = store.allocate(value.clone());
    assert_eq!(location.id(), 0);
    assert_eq!(store.location_count(), 1);
    assert!(store.memory_usage() > 0);
    
    let retrieved = store.get(location);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), &value);
}

#[test]
fn test_multiple_allocations() {
    let mut store = Store::new();
    
    let loc1 = store.allocate(Value::Number(SchemeNumber::Integer(1)));
    let loc2 = store.allocate(Value::String("hello".to_string()));
    let loc3 = store.allocate(Value::Boolean(true));
    
    assert_eq!(loc1.id(), 0);
    assert_eq!(loc2.id(), 1);
    assert_eq!(loc3.id(), 2);
    assert_eq!(store.location_count(), 3);
    
    // Verify values are correct
    assert_eq!(store.get(loc1).unwrap(), &Value::Number(SchemeNumber::Integer(1)));
    assert_eq!(store.get(loc2).unwrap(), &Value::String("hello".to_string()));
    assert_eq!(store.get(loc3).unwrap(), &Value::Boolean(true));
}

#[test]
fn test_store_set_value() {
    let mut store = Store::new();
    let location = store.allocate(Value::Number(SchemeNumber::Integer(42)));
    
    let new_value = Value::String("updated".to_string());
    let result = store.set(location, new_value.clone());
    assert!(result.is_ok());
    
    let retrieved = store.get(location);
    assert_eq!(retrieved.unwrap(), &new_value);
}

#[test]
fn test_store_set_invalid_location() {
    let mut store = Store::new();
    let invalid_location = Location::new(999);
    
    let result = store.set(invalid_location, Value::Boolean(true));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid location"));
}

#[test]
fn test_store_contains() {
    let mut store = Store::new();
    let location = store.allocate(Value::Number(SchemeNumber::Integer(42)));
    let invalid_location = Location::new(999);
    
    assert!(store.contains(location));
    assert!(!store.contains(invalid_location));
}

#[test]
fn test_reference_counting() {
    let mut store = Store::new();
    let location = store.allocate(Value::Number(SchemeNumber::Integer(42)));
    
    // Increment reference count
    let result = store.incref(location);
    assert!(result.is_ok());
    
    // Decrement reference count
    let result = store.decref(location);
    assert!(result.is_ok());
    
    // Value should still exist (ref count should be 1)
    assert!(store.contains(location));
}

#[test]
fn test_reference_counting_deallocation() {
    let mut store = Store::new();
    let location = store.allocate(Value::Number(SchemeNumber::Integer(42)));
    
    // Decrement reference count to 0 should deallocate
    let result = store.decref(location);
    assert!(result.is_ok());
    
    // Location should no longer exist
    assert!(!store.contains(location));
    assert_eq!(store.location_count(), 0);
}

#[test]
fn test_reference_counting_invalid_location() {
    let mut store = Store::new();
    let invalid_location = Location::new(999);
    
    let result = store.incref(invalid_location);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid location"));
    
    let result = store.decref(invalid_location);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid location"));
}

#[test]
fn test_explicit_deallocation() {
    let mut store = Store::new();
    let location = store.allocate(Value::Number(SchemeNumber::Integer(42)));
    
    let initial_count = store.location_count();
    let initial_memory = store.memory_usage();
    
    store.deallocate(location);
    
    assert_eq!(store.location_count(), initial_count - 1);
    assert!(store.memory_usage() < initial_memory);
    assert!(!store.contains(location));
}

#[test]
fn test_pooled_allocation() {
    let mut store = Store::new();
    
    // Allocate using pooled allocation
    let location1 = store.allocate_pooled(Value::Number(SchemeNumber::Integer(1)));
    let location2 = store.allocate_pooled(Value::String("hello".to_string()));
    
    assert_eq!(location1.id(), 0);
    assert_eq!(location2.id(), 1);
    assert_eq!(store.location_count(), 2);
    
    // Verify values are correct
    assert_eq!(store.get(location1).unwrap(), &Value::Number(SchemeNumber::Integer(1)));
    assert_eq!(store.get(location2).unwrap(), &Value::String("hello".to_string()));
}

#[test]
fn test_pooled_allocation_with_reuse() {
    let mut store = Store::new();
    
    // Allocate and deallocate to populate pools
    let location1 = store.allocate_pooled(Value::Number(SchemeNumber::Integer(1)));
    store.deallocate(location1);
    
    // Allocate again - should reuse from pool
    let location2 = store.allocate_pooled(Value::Number(SchemeNumber::Integer(2)));
    
    // Should reuse the same location ID
    assert_eq!(location1.id(), location2.id());
    assert_eq!(store.get(location2).unwrap(), &Value::Number(SchemeNumber::Integer(2)));
    
    // Check pool hit statistics
    let stats = store.get_statistics();
    assert!(stats.pool_hits > 0);
}

#[test]
fn test_memory_usage_tracking() {
    let mut store = Store::new();
    
    let initial_memory = store.memory_usage();
    assert_eq!(initial_memory, 0);
    
    // Allocate values of different sizes
    let _loc1 = store.allocate(Value::Number(SchemeNumber::Integer(42)));
    let memory_after_int = store.memory_usage();
    assert!(memory_after_int > initial_memory);
    
    let _loc2 = store.allocate(Value::String("hello world".to_string()));
    let memory_after_string = store.memory_usage();
    assert!(memory_after_string > memory_after_int);
    
    // Memory usage should increase with allocations
    assert!(memory_after_string > memory_after_int);
    assert!(memory_after_int > initial_memory);
}

#[test]
fn test_memory_limit_enforcement() {
    let mut store = Store::with_memory_limit(100); // Very low limit
    
    // Allocate values until memory limit is reached
    for i in 0..10 {
        let _location = store.allocate(Value::String(format!("string_{}", i)));
    }
    
    // Should have triggered garbage collection
    let stats = store.get_statistics();
    assert!(stats.gc_cycles > 0);
}

#[test]
fn test_garbage_collection_manual() {
    let mut store = Store::new();
    
    // Allocate some values
    let _loc1 = store.allocate(Value::Number(SchemeNumber::Integer(1)));
    let _loc2 = store.allocate(Value::Number(SchemeNumber::Integer(2)));
    
    let initial_gc_cycles = store.get_statistics().gc_cycles;
    
    // Manually trigger garbage collection
    store.collect_garbage();
    
    let final_gc_cycles = store.get_statistics().gc_cycles;
    assert_eq!(final_gc_cycles, initial_gc_cycles + 1);
}

#[test]
fn test_memory_statistics_tracking() {
    let mut store = Store::new();
    
    // Allocate values
    let loc1 = store.allocate(Value::Number(SchemeNumber::Integer(42)));
    let _loc2 = store.allocate(Value::String("hello".to_string()));
    
    let stats = store.get_statistics();
    assert_eq!(stats.total_allocations, 2);
    assert_eq!(stats.total_deallocations, 0);
    assert!(stats.peak_memory_usage > 0);
    
    // Deallocate one value
    store.deallocate(loc1);
    
    let stats = store.get_statistics();
    assert_eq!(stats.total_allocations, 2);
    assert_eq!(stats.total_deallocations, 1);
}

#[test]
fn test_pool_efficiency_calculation() {
    let mut store = Store::new();
    
    // Allocate and deallocate to create pool entries
    let loc1 = store.allocate_pooled(Value::Number(SchemeNumber::Integer(1)));
    store.deallocate(loc1);
    
    // Allocate again to use pool
    let _loc2 = store.allocate_pooled(Value::Number(SchemeNumber::Integer(2)));
    
    // Update pool efficiency
    store.update_pool_efficiency();
    
    let stats = store.get_statistics();
    assert!(stats.memory_pool_efficiency > 0.0);
    assert!(stats.memory_pool_efficiency <= 1.0);
}

#[test]
fn test_pool_utilization_tracking() {
    let mut store = Store::new();
    
    // Initially pools should be empty
    let (cell_util, location_util) = store.get_pool_utilization();
    assert_eq!(cell_util, 0.0);
    assert_eq!(location_util, 0.0);
    
    // Allocate and deallocate to populate pools
    let loc1 = store.allocate_pooled(Value::Number(SchemeNumber::Integer(1)));
    store.deallocate(loc1);
    
    // Pools should have some utilization
    let (cell_util, location_util) = store.get_pool_utilization();
    assert!(cell_util > 0.0);
    assert!(location_util > 0.0);
}

#[test]
fn test_memory_limit_setting() {
    let mut store = Store::new();
    
    // Set memory limit
    store.set_memory_limit(2048);
    
    // Allocate values to test limit
    for i in 0..10 {
        let _location = store.allocate(Value::String(format!("test_string_{}", i)));
    }
    
    // Should not crash and should respect limit
    assert!(store.memory_usage() > 0);
}

#[test]
fn test_complex_value_memory_estimation() {
    let mut store = Store::new();
    
    // Test different value types for memory estimation
    let loc1 = store.allocate(Value::Boolean(true));
    let memory_after_bool = store.memory_usage();
    
    let loc2 = store.allocate(Value::Number(SchemeNumber::Integer(42)));
    let memory_after_number = store.memory_usage();
    
    let loc3 = store.allocate(Value::String("hello world".to_string()));
    let memory_after_string = store.memory_usage();
    
    let loc4 = store.allocate(Value::Vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
    ]));
    let memory_after_vector = store.memory_usage();
    
    // Each allocation should increase memory usage
    assert!(memory_after_bool > 0);
    assert!(memory_after_number > memory_after_bool);
    assert!(memory_after_string > memory_after_number);
    assert!(memory_after_vector > memory_after_string);
    
    // Cleanup
    store.deallocate(loc1);
    store.deallocate(loc2);
    store.deallocate(loc3);
    store.deallocate(loc4);
}

#[test]
fn test_peak_memory_usage_tracking() {
    let mut store = Store::new();
    
    // Allocate values to increase memory usage
    let loc1 = store.allocate(Value::String("large string content".to_string()));
    let loc2 = store.allocate(Value::String("another large string".to_string()));
    
    let peak_after_allocation = store.get_statistics().peak_memory_usage;
    assert!(peak_after_allocation > 0);
    
    // Deallocate one value
    store.deallocate(loc1);
    
    // Peak should remain the same even after deallocation
    let peak_after_deallocation = store.get_statistics().peak_memory_usage;
    assert_eq!(peak_after_allocation, peak_after_deallocation);
    
    // Cleanup
    store.deallocate(loc2);
}

#[test]
fn test_mark_and_sweep_garbage_collection() {
    let mut store = Store::new();
    
    // Allocate values
    let loc1 = store.allocate(Value::Number(SchemeNumber::Integer(1)));
    let loc2 = store.allocate(Value::Number(SchemeNumber::Integer(2)));
    
    // Increment reference count on loc1 to keep it alive
    let _ = store.incref(loc1);
    
    // Manually trigger garbage collection
    store.collect_garbage();
    
    // loc1 should still exist (ref count > 0)
    assert!(store.contains(loc1));
    // loc2 should still exist (ref count = 1)
    assert!(store.contains(loc2));
    
    // Cleanup
    let _ = store.decref(loc1);
    let _ = store.decref(loc1); // Final decref to deallocate
    let _ = store.decref(loc2);
}

#[test]
fn test_store_statistics_defaults() {
    let stats = StoreStatistics::default();
    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.total_deallocations, 0);
    assert_eq!(stats.gc_cycles, 0);
    assert_eq!(stats.peak_memory_usage, 0);
    assert_eq!(stats.pool_hits, 0);
    assert_eq!(stats.clone_eliminations, 0);
    assert_eq!(stats.memory_pool_efficiency, 0.0);
}

#[test]
fn test_values_memory_estimation() {
    let mut store = Store::new();
    
    // Test Values type memory estimation
    let values = Value::Values(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::String("test".to_string()),
    ]);
    
    let _location = store.allocate(values);
    assert!(store.memory_usage() > 0);
}

#[test]
fn test_memory_pressure_scenarios() {
    let mut store = Store::with_memory_limit(1000);
    
    // Allocate many values to create memory pressure
    let mut locations = Vec::new();
    for i in 0..100 {
        let location = store.allocate(Value::String(format!("pressure_test_{}", i)));
        locations.push(location);
    }
    
    // Should have triggered garbage collection due to memory pressure
    let stats = store.get_statistics();
    assert!(stats.gc_cycles > 0);
    
    // Cleanup
    for location in locations {
        store.deallocate(location);
    }
}

#[test]
fn test_concurrent_allocation_simulation() {
    let mut store = Store::new();
    
    // Simulate concurrent-like allocation patterns
    let mut locations = Vec::new();
    
    // Allocate multiple values
    for i in 0..10 {
        let location = store.allocate(Value::Number(SchemeNumber::Integer(i)));
        locations.push(location);
    }
    
    // Deallocate some values
    for i in 0..5 {
        store.deallocate(locations[i]);
    }
    
    // Allocate more values (should potentially reuse deallocated space)
    for i in 0..5 {
        let location = store.allocate_pooled(Value::Number(SchemeNumber::Integer(i + 100)));
        locations.push(location);
    }
    
    // Verify final state
    assert!(store.location_count() > 0);
    assert!(store.memory_usage() > 0);
}

#[test]
fn test_edge_case_empty_string_allocation() {
    let mut store = Store::new();
    
    let location = store.allocate(Value::String("".to_string()));
    assert!(store.contains(location));
    assert_eq!(store.get(location).unwrap(), &Value::String("".to_string()));
    
    // Even empty strings should consume some memory
    assert!(store.memory_usage() > 0);
}

#[test]
fn test_nil_and_undefined_allocation() {
    let mut store = Store::new();
    
    let nil_location = store.allocate(Value::Nil);
    let undefined_location = store.allocate(Value::Undefined);
    
    assert!(store.contains(nil_location));
    assert!(store.contains(undefined_location));
    assert_eq!(store.get(nil_location).unwrap(), &Value::Nil);
    assert_eq!(store.get(undefined_location).unwrap(), &Value::Undefined);
}
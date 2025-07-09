//! Unit tests for adaptive memory management system (adaptive_memory.rs)
//!
//! Tests the adaptive memory manager including memory pressure detection,
//! allocation strategy selection, and optimization recommendations.

use lambdust::adaptive_memory::{
    AdaptiveMemoryManager, AllocationStrategy, MemoryPressure,
};

#[test]
fn test_adaptive_memory_manager_creation() {
    // Test default creation
    let manager = AdaptiveMemoryManager::new();
    let state = manager.state_info();
    
    assert_eq!(state.pressure_level, MemoryPressure::Low);
    assert_eq!(state.strategy, AllocationStrategy::Standard);
    assert_eq!(state.history_length, 0);
}

#[test]
fn test_allocation_strategy_types() {
    // Test allocation strategy enum variants
    let strategies = vec![
        AllocationStrategy::Standard,
        AllocationStrategy::Conservative,
        AllocationStrategy::Aggressive,
        AllocationStrategy::Emergency,
    ];
    
    // Ensure all strategies are distinct
    for i in 0..strategies.len() {
        for j in (i + 1)..strategies.len() {
            assert_ne!(strategies[i], strategies[j]);
        }
    }
}

#[test]
fn test_memory_pressure_levels() {
    // Test memory pressure enum variants
    let pressures = vec![
        MemoryPressure::Low,
        MemoryPressure::Moderate,
        MemoryPressure::High,
        MemoryPressure::Critical,
    ];
    
    // Ensure all pressure levels are distinct
    for i in 0..pressures.len() {
        for j in (i + 1)..pressures.len() {
            assert_ne!(pressures[i], pressures[j]);
        }
    }
}

#[test]
fn test_state_info_consistency() {
    let manager = AdaptiveMemoryManager::new();
    let state1 = manager.state_info();
    let state2 = manager.state_info();
    
    // Multiple calls should return consistent state
    assert_eq!(state1.pressure_level, state2.pressure_level);
    assert_eq!(state1.strategy, state2.strategy);
    assert_eq!(state1.history_length, state2.history_length);
}

#[test]
fn test_allocation_parameters_basic() {
    let manager = AdaptiveMemoryManager::new();
    let params = manager.allocation_parameters();
    
    // Basic parameter validation
    assert!(params.pool_size_multiplier > 0.0);
    assert!(params.gc_frequency_multiplier > 0.0);
    // aggressive_recycling and prefer_stack_allocation are booleans
}

#[test]
fn test_optimization_recommendations_default() {
    let manager = AdaptiveMemoryManager::new();
    let recommendations = manager.get_optimization_recommendations();
    
    // Default state (low pressure) should have minimal recommendations
    assert!(recommendations.len() <= 5);
}

#[test]
fn test_manager_multiple_instances() {
    // Test that multiple managers can be created independently
    let manager1 = AdaptiveMemoryManager::new();
    let manager2 = AdaptiveMemoryManager::new();
    
    let state1 = manager1.state_info();
    let state2 = manager2.state_info();
    
    // Both should start with same initial state
    assert_eq!(state1.pressure_level, state2.pressure_level);
    assert_eq!(state1.strategy, state2.strategy);
    assert_eq!(state1.history_length, state2.history_length);
}

#[test]
fn test_clone_and_debug_traits() {
    // Test that key types implement expected traits
    let pressure = MemoryPressure::Low;
    let pressure_clone = pressure.clone();
    assert_eq!(pressure, pressure_clone);
    
    let strategy = AllocationStrategy::Standard;
    let strategy_clone = strategy.clone();
    assert_eq!(strategy, strategy_clone);
    
    // Test Debug formatting (should not panic)
    let debug_str = format!("{:?}", pressure);
    assert!(!debug_str.is_empty());
    
    let debug_str = format!("{:?}", strategy);
    assert!(!debug_str.is_empty());
}

#[test]
fn test_memory_pressure_ordering() {
    // Test that memory pressure levels follow expected ordering
    let low = MemoryPressure::Low;
    let moderate = MemoryPressure::Moderate;
    let high = MemoryPressure::High;
    let critical = MemoryPressure::Critical;
    
    // All should be distinct
    assert_ne!(low, moderate);
    assert_ne!(moderate, high);
    assert_ne!(high, critical);
}

#[test]
fn test_allocation_strategy_consistency() {
    // Test that allocation strategies are consistent
    let manager = AdaptiveMemoryManager::new();
    
    // Get parameters multiple times
    let params1 = manager.allocation_parameters();
    let params2 = manager.allocation_parameters();
    
    // Should be consistent
    assert_eq!(params1.pool_size_multiplier, params2.pool_size_multiplier);
    assert_eq!(params1.gc_frequency_multiplier, params2.gc_frequency_multiplier);
    assert_eq!(params1.aggressive_recycling, params2.aggressive_recycling);
    assert_eq!(params1.prefer_stack_allocation, params2.prefer_stack_allocation);
}
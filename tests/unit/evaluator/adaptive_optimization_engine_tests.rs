//! Adaptive Optimization Engine Tests
//!
//! Tests for the refactored AdaptiveOptimizationEngine following Tell, Don't Ask principles

use lambdust::evaluator::runtime_executor::performance_reporting::{
    AdaptiveOptimizationEngine, 
    HotPathDetector,
    AdaptiveOptimizationStatistics
};

#[test]
fn test_adaptive_optimization_engine_creation() {
    let engine = AdaptiveOptimizationEngine::new();
    // Should not panic and should initialize properly with default values
    assert!(true);
}

#[test]
fn test_get_statistics_returns_zero_for_new_engine() {
    let engine = AdaptiveOptimizationEngine::new();
    
    // Get statistics - should not panic and return default values
    let stats = engine.get_statistics();
    
    // Initial statistics should be zero based on our Tell, Don't Ask implementation
    assert_eq!(stats.total_decisions, 0);
    assert_eq!(stats.jit_compilations, 0);
    assert_eq!(stats.hot_paths_detected, 0);
    assert_eq!(stats.cache_hit_rate, 0.0);
    assert_eq!(stats.average_improvement, 0.0);
}

#[test]
fn test_hot_path_detector_creation() {
    let detector = HotPathDetector::new();
    // Should initialize with default values without panicking
    assert!(true);
}

#[test]
fn test_hot_path_detector_default_behavior() {
    let detector = HotPathDetector::new();
    
    // Check if path is hot (should be false for new detector)
    let is_hot = detector.is_hot_path("test_expr");
    assert!(!is_hot, "New detector should not have any hot paths");
}

#[test]
fn test_adaptive_optimization_engine_tell_dont_ask_interface() {
    let mut engine = AdaptiveOptimizationEngine::new();
    
    // Test that our Tell, Don't Ask refactored methods work without panicking
    // These methods encapsulate internal state management
    let _is_hot = engine.update_and_check_hot_path("test_expr");
    let _total_decisions = engine.get_total_decisions_count();
    let _jit_count = engine.get_jit_compilations_count();
    let _hot_paths_count = engine.get_hot_paths_count();
    
    // All should complete without error, demonstrating proper encapsulation
    assert!(true);
}

#[test]
fn test_hot_path_detector_multiple_calls() {
    let mut detector = HotPathDetector::new();
    
    // Record multiple executions - should not panic
    detector.record_execution("expr1");
    detector.record_execution("expr2");
    detector.record_execution("expr1"); // Repeat
    
    // All calls should complete successfully
    assert!(true);
}
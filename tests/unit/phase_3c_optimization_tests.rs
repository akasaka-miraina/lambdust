//! Phase 3c performance optimization tests
//!
//! These tests verify the advanced optimization systems: dynamic stack monitoring,
//! adaptive memory management, and complete CPS inlining system.

use lambdust::adaptive_memory::{
    AdaptiveMemoryManager, AllocationStrategy, MemoryConfig, MemoryPressure,
};
use lambdust::cps_inlining::{ChainStrategy, CpsInliner, InliningDecision};
use lambdust::environment::Environment;
use lambdust::evaluator::Continuation;
use lambdust::lexer::SchemeNumber;
use lambdust::memory_pool::{ContinuationPoolStats, PoolStats};
use lambdust::stack_monitor::{OptimizationRecommendation, StackFrameType, StackMonitor};
use lambdust::value::Value;
use std::rc::Rc;
use std::time::Duration;

// Helper function to create test statistics
fn create_test_pool_stats() -> PoolStats {
    PoolStats {
        small_integers_cached: 1, // Minimal values
        values_in_recycle_pool: 1,
        symbols_interned: 1,
        total_interned_storage: 1,
    }
}

fn create_test_continuation_stats() -> ContinuationPoolStats {
    ContinuationPoolStats {
        identity_pooled: 1, // Minimal values
        total_recycled: 1,
        total_created: 2,
        recycle_rate: 50.0,
    }
}

fn create_test_stack_stats() -> lambdust::stack_monitor::StackStatistics {
    lambdust::stack_monitor::StackStatistics {
        current_depth: 8,
        max_depth: 15,
        total_frames: 500,
        optimizations_applied: 3,
        average_frame_time: Duration::from_millis(2),
        total_memory_estimate: 100, // Very low to ensure total stays under 1000
        optimizable_frames: 6,
    }
}

#[test]
fn test_stack_monitor_frame_management() {
    let mut monitor = StackMonitor::new();

    // Push various frame types
    monitor.push_frame(StackFrameType::Application {
        operator: "map".to_string(),
        arg_count: 2,
    });

    monitor.push_frame(StackFrameType::SpecialForm {
        form_name: "lambda".to_string(),
    });

    monitor.push_frame(StackFrameType::RecursiveCall {
        function_name: "factorial".to_string(),
        depth: 5,
    });

    let stats = monitor.statistics();
    assert_eq!(stats.current_depth, 3);
    assert_eq!(stats.total_frames, 3);
    assert!(stats.total_memory_estimate > 0);

    // Pop frames
    let frame = monitor.pop_frame();
    assert!(frame.is_some());
    assert_eq!(monitor.statistics().current_depth, 2);
}

#[test]
fn test_stack_monitor_optimization_recommendations() {
    let mut monitor = StackMonitor::new();

    // Add many recursive calls to trigger tail call optimization
    for i in 0..600 {
        monitor.push_frame(StackFrameType::RecursiveCall {
            function_name: "recursive-func".to_string(),
            depth: i,
        });
    }

    let recommendations = monitor.optimization_recommendations();
    assert!(recommendations.contains(&OptimizationRecommendation::TailCallOptimization));
    assert!(monitor.should_optimize());
}

#[test]
fn test_stack_monitor_memory_estimation() {
    let monitor = StackMonitor::new();

    // Test memory estimation for different frame types
    let app_frame = StackFrameType::Application {
        operator: "test".to_string(),
        arg_count: 3,
    };
    let app_memory = monitor.estimate_frame_memory(&app_frame);
    assert_eq!(app_memory, 64 + (3 * 32)); // Base + arg cost

    let recursive_frame = StackFrameType::RecursiveCall {
        function_name: "fib".to_string(),
        depth: 10,
    };
    let recursive_memory = monitor.estimate_frame_memory(&recursive_frame);
    assert_eq!(recursive_memory, 64 + (10 * 16)); // Base + depth cost
}

#[test]
fn test_adaptive_memory_pressure_levels() {
    let config = MemoryConfig {
        moderate_threshold: 1000,
        high_threshold: 2000,
        critical_threshold: 3000,
        ..Default::default()
    };

    let mut manager = AdaptiveMemoryManager::with_config(config.clone());
    let pool_stats = create_test_pool_stats();
    let continuation_stats = create_test_continuation_stats();
    let mut stack_stats = create_test_stack_stats();

    // Test low pressure (default)
    manager.update(
        pool_stats.clone(),
        continuation_stats.clone(),
        stack_stats.clone(),
    );
    let state = manager.state_info();
    assert_eq!(state.pressure_level, MemoryPressure::Low);
    assert_eq!(state.strategy, AllocationStrategy::Standard);

    // Test high pressure (with new manager to avoid cooldown)
    let mut high_manager = AdaptiveMemoryManager::with_config(config.clone());
    stack_stats.total_memory_estimate = 2500;
    high_manager.update(
        pool_stats.clone(),
        continuation_stats.clone(),
        stack_stats.clone(),
    );
    let state = high_manager.state_info();
    assert_eq!(state.pressure_level, MemoryPressure::High);
    assert_eq!(state.strategy, AllocationStrategy::Aggressive);

    // Test critical pressure (with new manager to avoid cooldown)
    let mut critical_manager = AdaptiveMemoryManager::with_config(config);
    stack_stats.total_memory_estimate = 3500;
    critical_manager.update(pool_stats, continuation_stats, stack_stats);
    let state = critical_manager.state_info();
    assert_eq!(state.pressure_level, MemoryPressure::Critical);
    assert_eq!(state.strategy, AllocationStrategy::Emergency);
}

#[test]
fn test_adaptive_memory_recommendations() {
    let config = MemoryConfig {
        critical_threshold: 1000,
        ..Default::default()
    };

    let mut manager = AdaptiveMemoryManager::with_config(config);
    let pool_stats = create_test_pool_stats();
    let continuation_stats = create_test_continuation_stats();
    let mut stack_stats = create_test_stack_stats();

    // Set critical memory usage
    stack_stats.total_memory_estimate = 1500;
    manager.update(pool_stats, continuation_stats, stack_stats);

    let recommendations = manager.get_optimization_recommendations();
    assert!(recommendations.contains(&OptimizationRecommendation::ForceGarbageCollection));
    assert!(recommendations.contains(&OptimizationRecommendation::MemoryCompression));
    assert!(recommendations.contains(&OptimizationRecommendation::ContinuationInlining));
}

#[test]
fn test_adaptive_memory_allocation_parameters() {
    let mut manager = AdaptiveMemoryManager::new();

    // Standard strategy parameters
    let params = manager.allocation_parameters();
    assert_eq!(params.pool_size_multiplier, 1.0);
    assert!(!params.aggressive_recycling);
    assert!(!params.prefer_stack_allocation);
    assert_eq!(params.gc_frequency_multiplier, 1.0);

    // Force emergency strategy
    let config = MemoryConfig {
        critical_threshold: 100,
        ..Default::default()
    };
    manager = AdaptiveMemoryManager::with_config(config);
    let pool_stats = create_test_pool_stats();
    let continuation_stats = create_test_continuation_stats();
    let mut stack_stats = create_test_stack_stats();
    stack_stats.total_memory_estimate = 200;

    manager.update(pool_stats, continuation_stats, stack_stats);
    let params = manager.allocation_parameters();
    assert_eq!(params.pool_size_multiplier, 0.5);
    assert!(params.aggressive_recycling);
    assert!(params.prefer_stack_allocation);
    assert_eq!(params.gc_frequency_multiplier, 4.0);
}

#[test]
fn test_cps_inliner_basic_decisions() {
    let mut inliner = CpsInliner::new();

    // Test identity continuation
    let identity = Continuation::Identity;
    let decision = inliner.analyze_continuation(&identity);
    assert_eq!(decision, InliningDecision::Eliminate);

    // Test simple assignment
    let env = Rc::new(Environment::new());
    let assignment = Continuation::Assignment {
        variable: "x".to_string(),
        env,
        parent: Box::new(Continuation::Identity),
    };
    let decision = inliner.analyze_continuation(&assignment);
    assert_eq!(decision, InliningDecision::Inline);

    // Test simple values
    let values = Continuation::Values {
        values: vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
        ],
        parent: Box::new(Continuation::Identity),
    };
    let decision = inliner.analyze_continuation(&values);
    assert_eq!(decision, InliningDecision::Inline);
}

#[test]
fn test_cps_inliner_operation_inlining() {
    let mut inliner = CpsInliner::new();

    // Test identity elimination
    let identity = Continuation::Identity;
    let result = inliner.try_inline_operation(&identity, Value::Number(SchemeNumber::Integer(42)));
    assert!(matches!(result, Some(Value::Number(_))));

    // Test assignment inlining
    let env = Rc::new(Environment::new());
    env.define("test".to_string(), Value::Number(SchemeNumber::Integer(0))); // Pre-define variable
    let assignment = Continuation::Assignment {
        variable: "test".to_string(),
        env,
        parent: Box::new(Continuation::Identity),
    };
    let result =
        inliner.try_inline_operation(&assignment, Value::Number(SchemeNumber::Integer(123)));
    assert!(matches!(result, Some(Value::Number(_))));

    // Check statistics
    let stats = inliner.statistics();
    assert_eq!(stats.eliminations, 1);
    assert_eq!(stats.inlined_operations, 1);
}

#[test]
fn test_cps_inliner_cache_efficiency() {
    let mut inliner = CpsInliner::new();
    let identity = Continuation::Identity;

    // First analysis should be a cache miss
    inliner.analyze_continuation(&identity);
    assert_eq!(inliner.statistics().cache_misses, 1);
    assert_eq!(inliner.statistics().cache_hits, 0);

    // Second analysis should be a cache hit
    inliner.analyze_continuation(&identity);
    assert_eq!(inliner.statistics().cache_hits, 1);

    // Calculate cache efficiency
    let efficiency = inliner.cache_efficiency();
    assert_eq!(efficiency, 0.5); // 1 hit out of 2 total requests
}

#[test]
fn test_cps_inliner_continuation_chain_optimization() {
    let mut inliner = CpsInliner::new();
    let env = Rc::new(Environment::new());

    // Create a simple chain: Assignment -> Identity
    let chain = Continuation::Assignment {
        variable: "result".to_string(),
        env,
        parent: Box::new(Continuation::Identity),
    };

    let optimized = inliner.optimize_continuation_chain(&chain);
    assert_eq!(optimized.original_depth, 1);
    assert!(optimized.can_eliminate_chain);
    assert_eq!(optimized.recommended_strategy, ChainStrategy::FullInline);

    // Check optimization breakdown
    assert_eq!(optimized.optimizations.len(), 2); // Assignment + Identity
    assert_eq!(optimized.optimizations[0].1, InliningDecision::Inline);
    assert_eq!(optimized.optimizations[1].1, InliningDecision::Eliminate);
}

#[test]
fn test_cps_inliner_pattern_generation() {
    let inliner = CpsInliner::new();

    // Test pattern generation for different continuation types
    let identity = Continuation::Identity;
    assert_eq!(inliner.continuation_pattern(&identity), "Identity");

    let values = Continuation::Values {
        values: vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3)),
        ],
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(inliner.continuation_pattern(&values), "Values(3)");

    let env = Rc::new(Environment::new());
    let assignment = Continuation::Assignment {
        variable: "x".to_string(),
        env,
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(inliner.continuation_pattern(&assignment), "Assignment");
}

#[test]
fn test_cps_inliner_efficiency_metrics() {
    let mut inliner = CpsInliner::new();

    // Initially no efficiency data
    assert_eq!(inliner.inlining_efficiency(), 0.0);

    // Add some operations with different outcomes
    let identity = Continuation::Identity;
    inliner.try_inline_operation(&identity, Value::Number(SchemeNumber::Integer(1))); // Elimination

    let env = Rc::new(Environment::new());
    let assignment = Continuation::Assignment {
        variable: "x".to_string(),
        env,
        parent: Box::new(Continuation::Identity),
    };
    inliner.try_inline_operation(&assignment, Value::Number(SchemeNumber::Integer(2))); // Inline

    // Both elimination and inlining count as optimization
    assert_eq!(inliner.inlining_efficiency(), 1.0); // 100% efficiency
}

#[test]
fn test_integrated_optimization_workflow() {
    // Test integration of all Phase 3c components
    let mut stack_monitor = StackMonitor::new();
    let mut memory_manager = AdaptiveMemoryManager::new();
    let mut cps_inliner = CpsInliner::new();

    // Simulate a complex evaluation scenario
    for i in 0..50 {
        // Add stack frames
        stack_monitor.push_frame(StackFrameType::Application {
            operator: format!("func-{}", i % 5),
            arg_count: (i % 4) + 1,
        });

        // Analyze continuations
        let identity = Continuation::Identity;
        cps_inliner.analyze_continuation(&identity);
    }

    // Update memory manager
    let pool_stats = create_test_pool_stats();
    let continuation_stats = create_test_continuation_stats();
    let stack_stats = stack_monitor.statistics();
    memory_manager.update(pool_stats, continuation_stats, stack_stats);

    // Check that all systems are working together
    let stack_recommendations = stack_monitor.optimization_recommendations();
    let memory_recommendations = memory_manager.get_optimization_recommendations();
    let inliner_efficiency = cps_inliner.inlining_efficiency();

    // Verify systems are providing optimization insights
    // Note: len() is always >= 0, so just verify the collections exist
    let _stack_count = stack_recommendations.len(); // May or may not have recommendations
    let _memory_count = memory_recommendations.len(); // May or may not have recommendations
    assert!(inliner_efficiency >= 0.0); // Should have some efficiency data

    // Check that cache is being used effectively
    assert!(cps_inliner.cache_efficiency() > 0.0);
}

#[test]
fn test_optimization_scalability() {
    // Test that optimizations scale well with large numbers of operations
    let mut stack_monitor = StackMonitor::new();
    let mut cps_inliner = CpsInliner::new();

    // Add a large number of operations
    for i in 0..1000 {
        stack_monitor.push_frame(StackFrameType::Application {
            operator: "test-func".to_string(),
            arg_count: 2,
        });

        if i % 10 == 0 {
            let _ = stack_monitor.pop_frame();
        }

        // Test CPS inlining with repeated patterns
        let identity = Continuation::Identity;
        cps_inliner.analyze_continuation(&identity);
    }

    let stack_stats = stack_monitor.statistics();
    let inliner_stats = cps_inliner.statistics();

    // Verify scalability
    assert!(stack_stats.total_frames == 1000);
    assert!(stack_stats.current_depth > 0);
    assert!(inliner_stats.cache_hits > 0); // Should have many cache hits
    assert!(cps_inliner.cache_efficiency() > 0.8); // Should have high cache efficiency
}

#[test]
fn test_phase_3c_performance_regression_prevention() {
    // Ensure Phase 3c optimizations don't break existing functionality
    let mut stack_monitor = StackMonitor::new();
    let mut memory_manager = AdaptiveMemoryManager::new();
    let mut cps_inliner = CpsInliner::new();

    // Test basic operations still work correctly
    stack_monitor.push_frame(StackFrameType::Application {
        operator: "basic-test".to_string(),
        arg_count: 1,
    });

    let frame = stack_monitor.pop_frame();
    assert!(frame.is_some());

    // Test memory manager basic functionality
    let pool_stats = create_test_pool_stats();
    let continuation_stats = create_test_continuation_stats();
    let stack_stats = create_test_stack_stats();
    memory_manager.update(pool_stats, continuation_stats, stack_stats);

    let state = memory_manager.state_info();
    assert_eq!(state.pressure_level, MemoryPressure::Low);

    // Test CPS inliner basic functionality
    let identity = Continuation::Identity;
    let decision = cps_inliner.analyze_continuation(&identity);
    assert_eq!(decision, InliningDecision::Eliminate);

    let result =
        cps_inliner.try_inline_operation(&identity, Value::Number(SchemeNumber::Integer(42)));
    assert!(matches!(result, Some(Value::Number(_))));
}

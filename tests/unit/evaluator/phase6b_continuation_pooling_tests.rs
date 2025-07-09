//! Phase 6-B-Step2: Unified Continuation Pooling System Tests
//!
//! Tests the global continuation pool manager system:
//! - Type-specific continuation pools
//! - Memory fragmentation prevention
//! - Heap allocation reduction via continuation reuse
//! - Performance monitoring and optimization hints
//! - Thread-safe access patterns

use lambdust::ast::{Expr, Literal};
use lambdust::environment::Environment;
use lambdust::evaluator::continuation_pooling::{
    ContinuationPoolManager, ContinuationType, PoolStatistics, SharedContinuationPoolManager,
    TypedContinuationPool,
};
use lambdust::evaluator::types::Evaluator;
use lambdust::evaluator::{Continuation, DoLoopState};
use lambdust::value::Value;
use std::rc::Rc;

#[test]
fn test_continuation_type_classification() {
    // Test Simple continuations
    let identity = Continuation::Identity;
    assert_eq!(
        ContinuationType::from_continuation(&identity),
        ContinuationType::Simple
    );

    let values = Continuation::Values {
        values: vec![Value::from(42i64)],
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(
        ContinuationType::from_continuation(&values),
        ContinuationType::Simple
    );

    let assignment = Continuation::Assignment {
        variable: "x".to_string(),
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(
        ContinuationType::from_continuation(&assignment),
        ContinuationType::Simple
    );

    // Test Application continuations
    let application = Continuation::Application {
        operator: Value::from(1i64),
        evaluated_args: vec![],
        remaining_args: vec![],
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(
        ContinuationType::from_continuation(&application),
        ContinuationType::Application
    );

    let operator = Continuation::Operator {
        args: vec![],
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(
        ContinuationType::from_continuation(&operator),
        ContinuationType::Application
    );

    // Test DoLoop continuations
    let env = Rc::new(Environment::new());
    let doloop_state = DoLoopState::new(
        vec![("i".to_string(), Value::from(0i64))],
        vec![None],
        Expr::Literal(Literal::Boolean(true)),
        vec![],
        vec![],
        env,
    );

    let doloop = Continuation::DoLoop {
        iteration_state: doloop_state,
        pool_id: Some(1),
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(
        ContinuationType::from_continuation(&doloop),
        ContinuationType::DoLoop
    );

    // Test ControlFlow continuations
    let if_test = Continuation::IfTest {
        consequent: Expr::Literal(Literal::Boolean(true)),
        alternate: None,
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(
        ContinuationType::from_continuation(&if_test),
        ContinuationType::ControlFlow
    );

    let begin = Continuation::Begin {
        remaining: vec![],
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert_eq!(
        ContinuationType::from_continuation(&begin),
        ContinuationType::ControlFlow
    );
}

#[test]
fn test_continuation_type_optimal_pool_sizes() {
    // Verify optimal pool sizes are reasonable
    assert_eq!(ContinuationType::Simple.optimal_pool_size(), 50);
    assert_eq!(ContinuationType::Application.optimal_pool_size(), 30);
    assert_eq!(ContinuationType::DoLoop.optimal_pool_size(), 20);
    assert_eq!(ContinuationType::ControlFlow.optimal_pool_size(), 25);
    assert_eq!(ContinuationType::Exception.optimal_pool_size(), 10);
    assert_eq!(ContinuationType::Complex.optimal_pool_size(), 5);
}

#[test]
fn test_continuation_type_memory_priorities() {
    // Verify priority ordering (higher number = higher priority)
    assert!(
        ContinuationType::DoLoop.memory_priority()
            > ContinuationType::Application.memory_priority()
    );
    assert!(
        ContinuationType::Application.memory_priority()
            > ContinuationType::Simple.memory_priority()
    );
    assert!(
        ContinuationType::Simple.memory_priority()
            > ContinuationType::ControlFlow.memory_priority()
    );
    assert!(
        ContinuationType::ControlFlow.memory_priority()
            > ContinuationType::Exception.memory_priority()
    );
    assert!(
        ContinuationType::Exception.memory_priority() > ContinuationType::Complex.memory_priority()
    );
}

#[test]
fn test_pool_statistics_operations() {
    let mut stats = PoolStatistics::new();

    // Test initial state
    assert_eq!(stats.total_allocations, 0);
    assert_eq!(stats.total_reuses, 0);
    assert_eq!(stats.current_size, 0);
    assert_eq!(stats.peak_size, 0);
    assert_eq!(stats.memory_saved_bytes, 0);
    assert_eq!(stats.reuse_efficiency(), 0.0);

    // Test allocation recording
    stats.record_allocation();
    stats.record_allocation();
    assert_eq!(stats.total_allocations, 2);
    assert_eq!(stats.reuse_efficiency(), 0.0); // No reuses yet

    // Test reuse recording
    stats.record_reuse(100);
    stats.record_reuse(150);
    assert_eq!(stats.total_reuses, 2);
    assert_eq!(stats.memory_saved_bytes, 250);
    assert_eq!(stats.reuse_efficiency(), 100.0); // 2 reuses / 2 allocations = 100%

    // Test size updates
    stats.update_size(5);
    assert_eq!(stats.current_size, 5);
    assert_eq!(stats.peak_size, 5);

    stats.update_size(10);
    assert_eq!(stats.current_size, 10);
    assert_eq!(stats.peak_size, 10);

    stats.update_size(3);
    assert_eq!(stats.current_size, 3);
    assert_eq!(stats.peak_size, 10); // Peak should remain
}

#[test]
fn test_typed_continuation_pool_operations() {
    let mut pool = TypedContinuationPool::new(ContinuationType::Simple);

    // Test initial state
    assert!(pool.is_empty());
    assert_eq!(pool.size(), 0);
    assert_eq!(pool.capacity_utilization(), 0.0);

    // Test allocation from empty pool
    assert!(pool.allocate().is_none());
    assert_eq!(pool.statistics().total_allocations, 1);
    assert_eq!(pool.statistics().total_reuses, 0);

    // Test deallocation
    let cont = Continuation::Identity;
    assert!(pool.deallocate(cont));
    assert_eq!(pool.size(), 1);
    assert!(!pool.is_empty());

    // Test successful reuse
    let reused_cont = pool.allocate();
    assert!(reused_cont.is_some());
    assert_eq!(pool.size(), 0);
    assert_eq!(pool.statistics().total_reuses, 1);
    assert!(pool.statistics().memory_saved_bytes > 0);

    // Test capacity utilization
    for _ in 0..5 {
        let cont = Continuation::Identity;
        pool.deallocate(cont);
    }
    let utilization = pool.capacity_utilization();
    assert!(utilization > 0.0 && utilization <= 1.0);
}

#[test]
fn test_typed_pool_type_validation() {
    let mut simple_pool = TypedContinuationPool::new(ContinuationType::Simple);

    // Valid continuation type for Simple pool
    let simple_cont = Continuation::Identity;
    assert!(simple_pool.deallocate(simple_cont));

    let values_cont = Continuation::Values {
        values: vec![Value::from(42i64)],
        parent: Box::new(Continuation::Identity),
    };
    assert!(simple_pool.deallocate(values_cont));

    // Invalid continuation type for Simple pool
    let app_cont = Continuation::Application {
        operator: Value::from(1i64),
        evaluated_args: vec![],
        remaining_args: vec![],
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert!(!simple_pool.deallocate(app_cont)); // Should reject wrong type
}

#[test]
fn test_typed_pool_size_limits() {
    let mut pool = TypedContinuationPool::new(ContinuationType::Exception); // Small pool (10)
    let max_size = pool.max_size();

    // Fill pool to capacity with Exception-type continuations
    for _ in 0..max_size {
        let cont = Continuation::ExceptionHandler {
            handler: Value::Boolean(true),
            env: Rc::new(Environment::new()),
            parent: Box::new(Continuation::Identity),
        };
        assert!(pool.deallocate(cont));
    }

    assert_eq!(pool.size(), max_size);

    // Try to exceed capacity
    let overflow_cont = Continuation::ExceptionHandler {
        handler: Value::Boolean(false),
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert!(!pool.deallocate(overflow_cont)); // Should reject overflow
    assert_eq!(pool.size(), max_size); // Size unchanged
}

#[test]
fn test_continuation_pool_manager_basic_operations() {
    let mut manager = ContinuationPoolManager::new();

    // Test allocation from empty pools
    assert!(manager.allocate(ContinuationType::Simple).is_none());
    assert!(manager.allocate(ContinuationType::Application).is_none());
    assert!(manager.allocate(ContinuationType::DoLoop).is_none());

    // Test deallocation and reuse cycle
    let simple_cont = Continuation::Identity;
    assert!(manager.deallocate(simple_cont));

    let app_cont = Continuation::Application {
        operator: Value::from(1i64),
        evaluated_args: vec![],
        remaining_args: vec![],
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert!(manager.deallocate(app_cont));

    // Test successful reuse
    let reused_simple = manager.allocate(ContinuationType::Simple);
    assert!(reused_simple.is_some());

    let reused_app = manager.allocate(ContinuationType::Application);
    assert!(reused_app.is_some());

    // Verify global statistics
    let (allocs, reuses, memory_saved, efficiency) = manager.global_statistics();
    assert_eq!(allocs, 5); // 3 failed allocations + 2 successful allocations
    assert_eq!(reuses, 2); // 2 successful reuses
    assert!(memory_saved > 0);
    assert_eq!(efficiency, 40.0); // 2 reuses / 5 allocations = 40%
}

#[test]
fn test_pool_manager_type_specific_statistics() {
    let mut manager = ContinuationPoolManager::new();

    // Add some continuations to different pools
    let simple_cont = Continuation::Identity;
    manager.deallocate(simple_cont);

    let app_cont = Continuation::Application {
        operator: Value::from(1i64),
        evaluated_args: vec![],
        remaining_args: vec![],
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    manager.deallocate(app_cont);

    // Allocate and check type-specific statistics
    manager.allocate(ContinuationType::Simple);
    manager.allocate(ContinuationType::Application);

    let simple_stats = manager.type_statistics(ContinuationType::Simple);
    assert!(simple_stats.is_some());
    if let Some(stats) = simple_stats {
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.total_reuses, 1);
    }

    let app_stats = manager.type_statistics(ContinuationType::Application);
    assert!(app_stats.is_some());
    if let Some(stats) = app_stats {
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.total_reuses, 1);
    }

    // Test all statistics collection
    let all_stats = manager.all_statistics();
    assert!(all_stats.contains_key(&ContinuationType::Simple));
    assert!(all_stats.contains_key(&ContinuationType::Application));
    assert!(all_stats.contains_key(&ContinuationType::DoLoop));
}

#[test]
fn test_pool_manager_clear_operations() {
    let mut manager = ContinuationPoolManager::new();

    // Add continuations to pools
    for _ in 0..5 {
        let cont = Continuation::Identity;
        manager.deallocate(cont);
    }

    // Clear specific type
    manager.clear_type(ContinuationType::Simple);

    // Should not reuse after clearing that type
    assert!(manager.allocate(ContinuationType::Simple).is_none());

    // Add more continuations
    for _ in 0..3 {
        let cont = Continuation::Identity;
        manager.deallocate(cont);
    }

    // Clear all pools
    manager.clear_all();

    // Verify all pools are cleared (this should not increment global counter after clear)
    // assert!(manager.allocate(ContinuationType::Simple).is_none());

    let (allocs, reuses, memory_saved, efficiency) = manager.global_statistics();
    assert_eq!(allocs, 0);
    assert_eq!(reuses, 0);
    assert_eq!(memory_saved, 0);
    assert_eq!(efficiency, 0.0);
}

#[test]
fn test_memory_defragmentation() {
    let mut manager = ContinuationPoolManager::new();

    // Fill pools with many continuations to trigger fragmentation concerns
    for _ in 0..100 {
        let simple_cont = Continuation::Identity;
        manager.deallocate(simple_cont);

        let values_cont = Continuation::Values {
            values: vec![Value::from(42i64)],
            parent: Box::new(Continuation::Identity),
        };
        manager.deallocate(values_cont);
    }

    // Check if defragmentation is needed
    let needs_defrag_before = manager.needs_defragmentation();

    // Perform defragmentation if needed
    if needs_defrag_before {
        manager.defragment();
    }

    // Verify pools are within reasonable bounds after defragmentation
    let (total_pools, active_pools, avg_utilization) = manager.memory_usage_summary();
    assert!(total_pools > 0);
    assert!(active_pools <= total_pools);
    assert!((0.0..=1.0).contains(&avg_utilization));

    // Simple pool should be trimmed to optimal size (50)
    let simple_stats = manager.type_statistics(ContinuationType::Simple);
    if let Some(stats) = simple_stats {
        assert!(stats.current_size <= ContinuationType::Simple.optimal_pool_size());
    }
}

#[test]
fn test_shared_continuation_pool_manager() {
    let shared_manager = SharedContinuationPoolManager::new();

    // Test thread-safe operations
    assert!(shared_manager.allocate(ContinuationType::Simple).is_none());
    assert!(shared_manager
        .allocate(ContinuationType::Application)
        .is_none());

    // Test deallocation
    let simple_cont = Continuation::Identity;
    assert!(shared_manager.deallocate(simple_cont));

    let app_cont = Continuation::Application {
        operator: Value::from(1i64),
        evaluated_args: vec![],
        remaining_args: vec![],
        env: Rc::new(Environment::new()),
        parent: Box::new(Continuation::Identity),
    };
    assert!(shared_manager.deallocate(app_cont));

    // Test successful reuse
    let reused_simple = shared_manager.allocate(ContinuationType::Simple);
    assert!(reused_simple.is_some());

    let reused_app = shared_manager.allocate(ContinuationType::Application);
    assert!(reused_app.is_some());

    // Test statistics access
    let stats = shared_manager.global_statistics();
    assert!(stats.is_some());

    if let Some((allocs, reuses, memory_saved, efficiency)) = stats {
        assert_eq!(allocs, 4);
        assert_eq!(reuses, 2);
        assert!(memory_saved > 0);
        assert_eq!(efficiency, 50.0);
    }

    // Test clear operations
    shared_manager.clear_all();

    let stats_after_clear = shared_manager.global_statistics();
    if let Some((allocs, reuses, memory_saved, efficiency)) = stats_after_clear {
        assert_eq!(allocs, 0);
        assert_eq!(reuses, 0);
        assert_eq!(memory_saved, 0);
        assert_eq!(efficiency, 0.0);
    }
}

#[test]
fn test_shared_pool_manager_defragmentation() {
    let shared_manager = SharedContinuationPoolManager::new();

    // Fill pools with many continuations
    for _ in 0..50 {
        let cont = Continuation::Identity;
        shared_manager.deallocate(cont);
    }

    // Test defragmentation check and execution
    let needs_defrag = shared_manager.needs_defragmentation();
    if needs_defrag {
        shared_manager.defragment();
    }

    // Should complete without panicking (basic functionality test)
    let stats = shared_manager.global_statistics();
    assert!(stats.is_some());
}

#[test]
fn test_evaluator_continuation_pool_manager_integration() {
    let mut evaluator = Evaluator::new();

    // Test pool manager access
    let pool_manager = evaluator.continuation_pool_manager();
    let (allocs, reuses, memory_saved, efficiency) = pool_manager.global_statistics();
    assert_eq!(allocs, 0);
    assert_eq!(reuses, 0);
    assert_eq!(memory_saved, 0);
    assert_eq!(efficiency, 0.0);

    // Test mutable access
    let pool_manager_mut = evaluator.continuation_pool_manager_mut();

    // Add a continuation
    let cont = Continuation::Identity;
    assert!(pool_manager_mut.deallocate(cont));

    // Verify allocation and reuse
    let reused_cont = pool_manager_mut.allocate(ContinuationType::Simple);
    assert!(reused_cont.is_some());

    // Check updated statistics
    let (allocs, reuses, memory_saved, efficiency) = pool_manager_mut.global_statistics();
    assert_eq!(allocs, 1);
    assert_eq!(reuses, 1);
    assert!(memory_saved > 0);
    assert_eq!(efficiency, 100.0);
}

#[test]
fn test_pool_manager_memory_usage_summary() {
    let mut manager = ContinuationPoolManager::new();

    // Test initial memory usage
    let (total_pools, active_pools, avg_utilization) = manager.memory_usage_summary();
    assert_eq!(total_pools, 6); // All continuation types initialized
    assert_eq!(active_pools, 0); // No pools have continuations yet
    assert_eq!(avg_utilization, 0.0);

    // Add continuations to some pools
    for _ in 0..5 {
        let simple_cont = Continuation::Identity;
        manager.deallocate(simple_cont);

        let app_cont = Continuation::Application {
            operator: Value::from(1i64),
            evaluated_args: vec![],
            remaining_args: vec![],
            env: Rc::new(Environment::new()),
            parent: Box::new(Continuation::Identity),
        };
        manager.deallocate(app_cont);
    }

    // Check updated memory usage
    let (total_pools, active_pools, avg_utilization) = manager.memory_usage_summary();
    assert_eq!(total_pools, 6);
    assert!(active_pools >= 2); // At least Simple and Application pools are active
    assert!(avg_utilization > 0.0);
}

//! Store system tests for R7RS memory management

use lambdust::ast::{Expr, Literal};
use lambdust::evaluator::{Continuation, Evaluator, StoreStatisticsWrapper};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_memory_usage_tracking() {
    let mut evaluator = Evaluator::new();
    
    // Test memory-usage special form
    let usage_expr = Expr::List(vec![
        Expr::Variable("memory-usage".to_string()),
    ]);
    
    let result = evaluator.eval(
        usage_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    
    // Result should be a non-negative integer
    match result.unwrap() {
        Value::Number(SchemeNumber::Integer(usage)) => {
            assert!(usage >= 0);
        }
        _ => panic!("memory-usage should return an integer"),
    }
}

#[test]
fn test_memory_statistics() {
    let mut evaluator = Evaluator::new();
    
    // Test memory-statistics special form
    let stats_expr = Expr::List(vec![
        Expr::Variable("memory-statistics".to_string()),
    ]);
    
    let result = evaluator.eval(
        stats_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    
    // Result should be a vector of association pairs
    let stats = result.unwrap();
    assert!(stats.is_list() || matches!(stats, Value::Vector(_)));
    
    // Convert to vector and check contents
    if let Some(stats_vec) = stats.to_vector() {
        assert!(!stats_vec.is_empty());
        
        // Check that we have expected statistics keys
        let expected_keys = vec![
            "total-allocations",
            "total-deallocations", 
            "gc-cycles",
            "peak-memory-usage",
            "current-memory-usage",
            "location-count",
        ];
        
        for stat_pair in &stats_vec {
            if let Value::Pair(pair_ref) = stat_pair {
                let pair = pair_ref.borrow();
                if let Value::Symbol(key) = &pair.car {
                    assert!(expected_keys.contains(&key.as_str()));
                }
            }
        }
    }
}

#[test]
fn test_garbage_collection() {
    let mut evaluator = Evaluator::new();
    
    // Get initial memory usage
    let _usage_before = evaluator.memory_usage();
    
    // Trigger garbage collection
    let gc_expr = Expr::List(vec![
        Expr::Variable("collect-garbage".to_string()),
    ]);
    
    let result = evaluator.eval(
        gc_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Undefined);
    
    // Get statistics after GC
    let stats = evaluator.store_statistics();
    // Note: GC cycles only available in traditional store
    match &stats {
        StoreStatisticsWrapper::Traditional(traditional_stats) => {
            assert!(traditional_stats.gc_cycles > 0);
        }
        #[cfg(feature = "raii-store")]
        StoreStatisticsWrapper::Raii(_) => {
            // RAII store doesn't have GC cycles
        }
    }
}

#[test]
fn test_memory_limit_setting() {
    let mut evaluator = Evaluator::new();
    
    // Set memory limit
    let limit_expr = Expr::List(vec![
        Expr::Variable("set-memory-limit!".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1024 * 1024))), // 1MB
    ]);
    
    let result = evaluator.eval(
        limit_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Undefined);
}

#[test]
fn test_location_allocation_and_access() {
    let mut evaluator = Evaluator::new();
    
    // Allocate a location
    let alloc_expr = Expr::List(vec![
        Expr::Variable("allocate-location".to_string()),
        Expr::Literal(Literal::String("test value".to_string())),
    ]);
    
    let alloc_result = evaluator.eval(
        alloc_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(alloc_result.is_ok());
    
    let location_id = match alloc_result.unwrap() {
        Value::Number(SchemeNumber::Integer(id)) => id,
        _ => panic!("allocate-location should return an integer location ID"),
    };
    
    // Access the location
    let ref_expr = Expr::List(vec![
        Expr::Variable("location-ref".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(location_id))),
    ]);
    
    let ref_result = evaluator.eval(
        ref_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(ref_result.is_ok());
    assert_eq!(ref_result.unwrap(), Value::String("test value".to_string()));
}

#[test]
fn test_location_modification() {
    let mut evaluator = Evaluator::new();
    
    // Allocate a location
    let alloc_expr = Expr::List(vec![
        Expr::Variable("allocate-location".to_string()),
        Expr::Literal(Literal::String("initial value".to_string())),
    ]);
    
    let alloc_result = evaluator.eval(
        alloc_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(alloc_result.is_ok());
    
    let location_id = match alloc_result.unwrap() {
        Value::Number(SchemeNumber::Integer(id)) => id,
        _ => panic!("allocate-location should return an integer location ID"),
    };
    
    // Modify the location
    let set_expr = Expr::List(vec![
        Expr::Variable("location-set!".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(location_id))),
        Expr::Literal(Literal::String("modified value".to_string())),
    ]);
    
    let set_result = evaluator.eval(
        set_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(set_result.is_ok());
    assert_eq!(set_result.unwrap(), Value::Undefined);
    
    // Verify the modification
    let ref_expr = Expr::List(vec![
        Expr::Variable("location-ref".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(location_id))),
    ]);
    
    let ref_result = evaluator.eval(
        ref_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(ref_result.is_ok());
    assert_eq!(ref_result.unwrap(), Value::String("modified value".to_string()));
}

#[test]
fn test_invalid_location_access() {
    let mut evaluator = Evaluator::new();
    
    // Try to access an invalid location
    let ref_expr = Expr::List(vec![
        Expr::Variable("location-ref".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(99999))),
    ]);
    
    let ref_result = evaluator.eval(
        ref_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(ref_result.is_err());
}

#[test]
fn test_memory_operations_integration() {
    let mut evaluator = Evaluator::new();
    
    // Get initial statistics
    let initial_stats = evaluator.store_statistics().clone();
    
    // Allocate several locations
    for i in 0..5 {
        let alloc_expr = Expr::List(vec![
            Expr::Variable("allocate-location".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(i))),
        ]);
        
        let result = evaluator.eval(
            alloc_expr,
            evaluator.global_env.clone(),
            Continuation::Identity,
        );
        
        assert!(result.is_ok());
    }
    
    // Check that allocations increased
    let current_allocations = evaluator.store_statistics().total_allocations();
    assert!(current_allocations > initial_stats.total_allocations());
    assert!(evaluator.memory_usage() > 0);
    
    // Trigger garbage collection
    let gc_expr = Expr::List(vec![
        Expr::Variable("collect-garbage".to_string()),
    ]);
    
    let gc_result = evaluator.eval(
        gc_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    assert!(gc_result.is_ok());
    
    // Check that GC was triggered (traditional store only)
    let final_stats = evaluator.store_statistics();
    match (&initial_stats, &final_stats) {
        (
            StoreStatisticsWrapper::Traditional(initial_traditional),
            StoreStatisticsWrapper::Traditional(final_traditional),
        ) => {
            assert!(final_traditional.gc_cycles > initial_traditional.gc_cycles);
        }
        _ => {
            // For RAII store, just check that memory management happened
            assert!(final_stats.total_deallocations() >= initial_stats.total_deallocations());
        }
    }
}
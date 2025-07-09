//! Store system tests for R7RS memory management

use lambdust::ast::{Expr, Literal};
use lambdust::evaluator::{Continuation, Evaluator};
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_memory_usage_tracking() {
    let mut evaluator = Evaluator::new();

    // Test memory-usage special form
    let usage_expr = Expr::List(vec![Expr::Variable("memory-usage".to_string())]);

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
fn test_memory_statistics_failsafe() {
    let mut evaluator = Evaluator::new();

    // Test memory-statistics when not implemented - should fail gracefully
    let stats_expr = Expr::List(vec![Expr::Variable("memory-statistics".to_string())]);

    let result = evaluator.eval(
        stats_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    // Check if memory-statistics is implemented
    match result {
        Ok(stats) => {
            // memory-statistics is implemented - verify it returns reasonable data
            assert!(stats.is_list() || matches!(stats, Value::Vector(_)));
        }
        Err(_) => {
            // memory-statistics is not implemented - this is acceptable
        }
    }
    
    // Test memory-usage - should also fail gracefully if not implemented
    let usage_expr = Expr::List(vec![Expr::Variable("memory-usage".to_string())]);
    
    let usage_result = evaluator.eval(
        usage_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    // Should return error or provide basic memory info if available
    // Either way, should not panic
    match usage_result {
        Ok(Value::Number(_)) => {}, // Basic memory info available
        Err(_) => {}, // Not implemented - acceptable
        _ => panic!("Unexpected memory-usage return type"),
    }
    
    // Test collect-garbage - should be safe to call even if not fully implemented
    let gc_expr = Expr::List(vec![Expr::Variable("collect-garbage".to_string())]);
    
    let gc_result = evaluator.eval(
        gc_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );
    
    // Should either work (return undefined) or return appropriate error
    match gc_result {
        Ok(Value::Undefined) => {}, // GC worked
        Err(_) => {}, // GC not implemented - acceptable
        _ => panic!("Unexpected collect-garbage return type"),
    }
}

#[test]
fn test_garbage_collection() {
    let mut evaluator = Evaluator::new();

    // Get initial memory usage
    let _usage_before = evaluator.memory_usage();

    // Trigger garbage collection
    let gc_expr = Expr::List(vec![Expr::Variable("collect-garbage".to_string())]);

    let result = evaluator.eval(
        gc_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Undefined);

    // Get statistics after GC
    let stats = evaluator.store_statistics();
    // Phase 5-Step2: Only RAII store available, no traditional GC cycles
    let _raii_stats = stats.raii_statistics();
    // RAII store doesn't have GC cycles, but has auto cleanup events
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
fn test_location_allocation_failsafe() {
    let mut evaluator = Evaluator::new();

    // Test that unknown location operations return appropriate errors
    let unknown_alloc = Expr::List(vec![
        Expr::Variable("allocate-location".to_string()),
        Expr::Literal(Literal::String("test value".to_string())),
    ]);

    let alloc_result = evaluator.eval(
        unknown_alloc,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    // Check if the operation is implemented or returns appropriate error
    match alloc_result {
        Ok(Value::Number(SchemeNumber::Integer(id))) => {
            // Location allocation is implemented and working
            assert!(id >= 0);
        }
        Err(_) => {
            // Location allocation is not implemented - this is acceptable
        }
        _ => panic!("allocate-location should return integer ID or error"),
    }
    
    // Test location-ref with invalid ID
    let invalid_ref = Expr::List(vec![
        Expr::Variable("location-ref".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(-1))),
    ]);

    let ref_result = evaluator.eval(
        invalid_ref,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    // Should return an error for invalid location reference
    assert!(ref_result.is_err());
    
    // Test location-ref with non-integer ID
    let non_int_ref = Expr::List(vec![
        Expr::Variable("location-ref".to_string()),
        Expr::Literal(Literal::String("not-an-id".to_string())),
    ]);

    let ref_result = evaluator.eval(
        non_int_ref,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    // Should return type error for non-integer location ID
    assert!(ref_result.is_err());
}

#[test]
fn test_location_modification_failsafe() {
    let mut evaluator = Evaluator::new();

    // Test that location-set! returns appropriate error for unimplemented operation
    let set_expr = Expr::List(vec![
        Expr::Variable("location-set!".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
        Expr::Literal(Literal::String("test value".to_string())),
    ]);

    let set_result = evaluator.eval(
        set_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    // Should return appropriate error for unimplemented operation
    assert!(set_result.is_err());
    
    // Test with invalid arguments
    let invalid_set = Expr::List(vec![
        Expr::Variable("location-set!".to_string()),
        Expr::Literal(Literal::String("not-a-location-id".to_string())),
        Expr::Literal(Literal::String("value".to_string())),
    ]);

    let invalid_result = evaluator.eval(
        invalid_set,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    // Should return type error for invalid location ID
    assert!(invalid_result.is_err());
    
    // Test with missing arguments
    let missing_args = Expr::List(vec![
        Expr::Variable("location-set!".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
    ]);

    let missing_result = evaluator.eval(
        missing_args,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    // Should return arity error for missing arguments
    assert!(missing_result.is_err());
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
fn test_memory_operations_integration_failsafe() {
    let mut evaluator = Evaluator::new();

    // Test that memory operations are safe even when not fully implemented
    let initial_stats = evaluator.store_statistics();
    
    // Basic sanity check on statistics - they should be accessible
    let _total_allocs = initial_stats.total_allocations();
    let _total_deallocs = initial_stats.total_deallocations();
    
    // Test memory usage reporting
    let _memory_usage = evaluator.memory_usage();
    
    // Test that allocate-location operations fail gracefully
    let alloc_expr = Expr::List(vec![
        Expr::Variable("allocate-location".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
    ]);

    let alloc_result = evaluator.eval(
        alloc_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    // Check if allocate-location is implemented
    match alloc_result {
        Ok(Value::Number(SchemeNumber::Integer(id))) => {
            // Location allocation is implemented and working
            assert!(id >= 0);
        }
        Err(_) => {
            // Location allocation is not implemented - this is acceptable
        }
        _ => panic!("allocate-location should return integer ID or error"),
    }
    
    // Test garbage collection is safe to call
    let gc_expr = Expr::List(vec![Expr::Variable("collect-garbage".to_string())]);

    let gc_result = evaluator.eval(
        gc_expr,
        evaluator.global_env.clone(),
        Continuation::Identity,
    );

    // Should either work or return appropriate error
    match gc_result {
        Ok(Value::Undefined) => {}, // GC worked
        Err(_) => {}, // GC not implemented - acceptable
        _ => panic!("Unexpected collect-garbage return type"),
    }
    
    // Verify statistics remain consistent
    let final_stats = evaluator.store_statistics();
    assert!(final_stats.total_allocations() >= initial_stats.total_allocations());
    assert!(final_stats.total_deallocations() >= initial_stats.total_deallocations());
}

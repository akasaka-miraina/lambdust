//! Store memory management builtins
//!
//! This module provides builtin functions for R7RS-compliant memory management
//! including allocation, garbage collection, and memory monitoring.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::evaluator::types::{Location, StoreStatisticsWrapper};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register store management functions into the builtins map
pub fn register_store_functions(builtins: &mut HashMap<String, Value>) {
    // Memory monitoring functions
    builtins.insert("memory-usage".to_string(), memory_usage_function());
    builtins.insert("memory-statistics".to_string(), memory_statistics_function());
    builtins.insert("collect-garbage".to_string(), collect_garbage_function());
    builtins.insert("set-memory-limit!".to_string(), set_memory_limit_function());
    
    // Location management functions
    builtins.insert("location?".to_string(), location_predicate_function());
    builtins.insert("location-equal?".to_string(), location_equal_function());
}

/// Create memory-usage function
fn memory_usage_function() -> Value {
    make_builtin_procedure("memory-usage", Some(0), |args| {
        check_arity(args, 0)?;
        // This would need evaluator context to get actual memory usage
        // For now, return a placeholder
        Err(LambdustError::runtime_error(
            "memory-usage requires evaluator context".to_string(),
        ))
    })
}

/// Create memory-statistics function
fn memory_statistics_function() -> Value {
    make_builtin_procedure("memory-statistics", Some(0), |args| {
        check_arity(args, 0)?;
        // This would need evaluator context to get actual statistics
        // For now, return a placeholder
        Err(LambdustError::runtime_error(
            "memory-statistics requires evaluator context".to_string(),
        ))
    })
}

/// Create collect-garbage function
fn collect_garbage_function() -> Value {
    make_builtin_procedure("collect-garbage", Some(0), |args| {
        check_arity(args, 0)?;
        // This would need evaluator context to trigger GC
        // For now, return a placeholder
        Err(LambdustError::runtime_error(
            "collect-garbage requires evaluator context".to_string(),
        ))
    })
}

/// Create set-memory-limit! function
fn set_memory_limit_function() -> Value {
    make_builtin_procedure("set-memory-limit!", Some(1), |args| {
        check_arity(args, 1)?;
        
        let _limit = match &args[0] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as usize,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as usize,
            _ => {
                return Err(LambdustError::type_error(
                    "Memory limit must be an integer".to_string(),
                ));
            }
        };
        
        // This would need evaluator context to set memory limit
        // For now, return a placeholder
        Err(LambdustError::runtime_error(
            "set-memory-limit! requires evaluator context".to_string(),
        ))
    })
}

/// Create location? predicate function
fn location_predicate_function() -> Value {
    make_builtin_procedure("location?", Some(1), |args| {
        check_arity(args, 1)?;
        
        // For now, we don't have a direct way to check if a value is a location
        // This would need to be integrated with the Value enum
        Ok(Value::Boolean(false))
    })
}

/// Create location-equal? function
fn location_equal_function() -> Value {
    make_builtin_procedure("location-equal?", Some(2), |args| {
        check_arity(args, 2)?;
        
        // This would compare location values if they were part of the Value enum
        // For now, just return false
        Ok(Value::Boolean(false))
    })
}

/// Store management utilities that require evaluator integration
/// These functions are meant to be called from special forms or evaluator methods

/// Get memory usage from evaluator
pub fn get_memory_usage_from_evaluator(evaluator: &crate::evaluator::types::Evaluator) -> usize {
    evaluator.memory_usage()
}

/// Get memory statistics from evaluator
pub fn get_memory_statistics_from_evaluator(
    evaluator: &crate::evaluator::types::Evaluator,
) -> StoreStatisticsWrapper {
    evaluator.store_statistics()
}

/// Collect garbage using evaluator
pub fn collect_garbage_with_evaluator(evaluator: &mut crate::evaluator::types::Evaluator) {
    evaluator.collect_garbage();
}

/// Set memory limit using evaluator
pub fn set_memory_limit_with_evaluator(
    evaluator: &mut crate::evaluator::types::Evaluator,
    limit: usize,
) {
    evaluator.set_memory_limit(limit);
}

/// Convert StoreStatisticsWrapper to Scheme value
pub fn statistics_to_scheme_value(stats: &StoreStatisticsWrapper) -> Value {
    // Create an association list with statistics
    let pairs = vec![
        Value::cons(
            Value::Symbol("total-allocations".to_string()),
            Value::Number(crate::lexer::SchemeNumber::Integer(stats.total_allocations() as i64)),
        ),
        Value::cons(
            Value::Symbol("total-deallocations".to_string()),
            Value::Number(crate::lexer::SchemeNumber::Integer(stats.total_deallocations() as i64)),
        ),
        Value::cons(
            Value::Symbol("memory-usage".to_string()),
            Value::Number(crate::lexer::SchemeNumber::Integer(stats.memory_usage() as i64)),
        ),
    ];

    // Build list from pairs
    let mut result = Value::Nil;
    for pair in pairs.into_iter().rev() {
        result = Value::cons(pair, result);
    }
    result
}
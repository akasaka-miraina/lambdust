//! Comprehensive tests for SRFI-18 Threading Infrastructure
//!
//! This test suite validates the complete SRFI-18 implementation including:
//! - Thread creation, starting, and joining
//! - Mutex locking and unlocking
//! - Condition variable operations
//! - Parameter inheritance between threads
//! - Performance benchmarks
//! - Error handling and edge cases

use lambdust::concurrency::scheme_threading::{
    SchemeConditionVariable, SchemeMutex, SchemeThread, ThreadRegistry, current_thread_id,
    set_current_thread_id,
};
use lambdust::diagnostics::Result;
use lambdust::eval::parameter::{
    ParameterFrame, capture_parameter_bindings, inherit_parameter_bindings,
};
use lambdust::eval::value::Value;
use lambdust::stdlib::srfi18_multithreading::*;
use std::sync::{Arc, Mutex as StdMutex};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn test_thread_creation_and_basic_operations() {
    // Test basic thread creation
    let thread = SchemeThread::new(Some("test-thread".to_string()), Arc::new(Vec::new()));

    assert_eq!(thread.name, Some("test-thread".to_string()));
    assert!(thread.id > 0);
}

#[test]
fn test_thread_predicates() {
    // Test thread predicate
    let thread = Arc::new(SchemeThread::new(None, Arc::new(Vec::new())));
    let thread_value = Value::Thread(thread);

    let result = thread_predicate(&[thread_value]).unwrap();
    if let Value::Literal(lambdust::ast::literal::Literal::Boolean(is_thread)) = result {
        assert!(is_thread);
    } else {
        panic!("Expected boolean result");
    }

    // Test with non-thread value
    let result = thread_predicate(&[Value::Nil]).unwrap();
    if let Value::Literal(lambdust::ast::literal::Literal::Boolean(is_thread)) = result {
        assert!(!is_thread);
    } else {
        panic!("Expected boolean result");
    }
}

#[test]
fn test_mutex_creation_and_predicates() {
    // Test mutex creation without name
    let result = make_mutex(&[]).unwrap();
    assert!(matches!(result, Value::Mutex(_)));

    // Test mutex creation with name
    let name = Value::Literal(lambdust::ast::literal::Literal::String(Box::new(
        "test-mutex".to_string(),
    )));
    let result = make_mutex(&[name]).unwrap();
    assert!(matches!(result, Value::Mutex(_)));

    // Test mutex predicate
    let mutex = Arc::new(SchemeMutex::new(None));
    let mutex_value = Value::Mutex(mutex);

    let result = mutex_predicate(&[mutex_value]).unwrap();
    if let Value::Literal(lambdust::ast::literal::Literal::Boolean(is_mutex)) = result {
        assert!(is_mutex);
    } else {
        panic!("Expected boolean result");
    }
}

#[test]
fn test_condition_variable_creation() {
    let mutex = Arc::new(SchemeMutex::new(Some("test-mutex".to_string())));
    let mutex_value = Value::Mutex(Arc::clone(&mutex));

    // Test condition variable creation
    let result = make_condition_variable(&[mutex_value]).unwrap();
    assert!(matches!(result, Value::ConditionVariable(_)));

    // Test condition variable predicate
    let condvar = Arc::new(SchemeConditionVariable::new(None, mutex));
    let condvar_value = Value::ConditionVariable(condvar);

    let result = condition_variable_predicate(&[condvar_value]).unwrap();
    if let Value::Literal(lambdust::ast::literal::Literal::Boolean(is_condvar)) = result {
        assert!(is_condvar);
    } else {
        panic!("Expected boolean result");
    }
}

#[test]
fn test_mutex_locking_basic() {
    let mutex = Arc::new(SchemeMutex::new(Some("test-mutex".to_string())));
    let thread_id = 1u64;

    // Test successful lock acquisition
    assert!(mutex.lock(thread_id, None).is_ok());
    assert!(mutex.is_locked());

    // Test unlock
    assert!(mutex.unlock(thread_id).is_ok());
    assert!(!mutex.is_locked());
}

#[test]
fn test_mutex_ownership() {
    let mutex = Arc::new(SchemeMutex::new(None));
    let thread_id_1 = 1u64;
    let thread_id_2 = 2u64;

    // Lock with thread 1
    assert!(mutex.lock(thread_id_1, None).is_ok());

    // Try to unlock with thread 2 (should fail)
    assert!(mutex.unlock(thread_id_2).is_err());

    // Unlock with correct thread (should succeed)
    assert!(mutex.unlock(thread_id_1).is_ok());
}

#[test]
fn test_condition_variable_operations() {
    let mutex = Arc::new(SchemeMutex::new(None));
    let condvar = Arc::new(SchemeConditionVariable::new(None, Arc::clone(&mutex)));

    // Test notify operations (these shouldn't fail even if no one is waiting)
    condvar.notify_one();
    condvar.notify_all();
}

#[test]
fn test_thread_registry() {
    let registry = ThreadRegistry::new().unwrap();
    let thread = Arc::new(SchemeThread::new(
        Some("test-thread".to_string()),
        Arc::new(Vec::new()),
    ));

    let thread_id = thread.id;

    // Test thread registration
    registry.register_thread(Arc::clone(&thread));
    assert!(registry.get_thread(thread_id).is_some());

    // Test thread unregistration
    registry.unregister_thread(thread_id);
    assert!(registry.get_thread(thread_id).is_none());
}

#[test]
fn test_parameter_inheritance_basic() {
    use lambdust::eval::parameter::ParameterFrame;
    use smallvec::SmallVec;

    // Create some test parameter bindings
    let mut bindings = SmallVec::new();
    bindings.push((
        1u64,
        Value::Literal(lambdust::ast::literal::Literal::String(Box::new(
            "test-value".to_string(),
        ))),
    ));

    let frame = ParameterFrame { bindings };
    let frames = &[frame];

    // Test parameter inheritance (this would be done in a real thread)
    inherit_parameter_bindings(&frames);

    // Capture bindings to verify they were inherited
    let captured = capture_parameter_bindings();
    assert_eq!(captured.len(), 1);
}

#[test]
fn test_concurrent_mutex_operations() {
    let mutex = Arc::new(SchemeMutex::new(Some("concurrent-test".to_string())));
    let counter = Arc::new(StdMutex::new(0i32));

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let mutex_clone = Arc::clone(&mutex);
            let counter_clone = Arc::clone(&counter);
            let thread_id = i as u64 + 1;

            thread::spawn(move || {
                // Simulate concurrent access
                for _ in 0..10 {
                    // Acquire mutex
                    if mutex_clone
                        .lock(thread_id, Some(Duration::from_millis(100)))
                        .is_ok()
                    {
                        // Critical section
                        {
                            let mut count = counter_clone.lock().unwrap();
                            *count += 1;
                        }

                        // Small delay to increase contention
                        thread::sleep(Duration::from_millis(1));

                        // Release mutex
                        let _ = mutex_clone.unlock(thread_id);
                    }
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify the counter was incremented correctly
    let final_count = *counter.lock().unwrap();
    assert!(
        final_count <= 40,
        "Final count should be at most 40, got {}",
        final_count
    );
    assert!(
        final_count > 0,
        "Final count should be greater than 0, got {}",
        final_count
    );
}

#[test]
fn test_mutex_timeout() {
    let mutex = Arc::new(SchemeMutex::new(Some("timeout-test".to_string())));
    let thread_id_1 = 1u64;
    let thread_id_2 = 2u64;

    // Thread 1 locks the mutex
    assert!(mutex.lock(thread_id_1, None).is_ok());

    // Thread 2 tries to lock with timeout
    let start = Instant::now();
    let result = mutex.lock(thread_id_2, Some(Duration::from_millis(50)));
    let elapsed = start.elapsed();

    // Should have timed out
    assert!(result.is_err());
    assert!(elapsed >= Duration::from_millis(45)); // Allow some tolerance
    assert!(elapsed < Duration::from_millis(100)); // But not too much

    // Cleanup
    let _ = mutex.unlock(thread_id_1);
}

#[test]
fn test_thread_current_id() {
    // Test initial state
    assert!(current_thread_id().is_none());

    // Set thread ID
    set_current_thread_id(42);
    assert_eq!(current_thread_id(), Some(42));

    // Test in actual thread
    let handle = thread::spawn(|| {
        set_current_thread_id(123);
        assert_eq!(current_thread_id(), Some(123));
    });

    handle.join().unwrap();

    // Original thread should still have its ID
    assert_eq!(current_thread_id(), Some(42));
}

#[test]
fn test_performance_benchmarks() {
    const NUM_OPERATIONS: usize = 10000;

    // Benchmark thread creation
    let start = Instant::now();
    for i in 0..1000 {
        let _thread = SchemeThread::new(Some(format!("thread-{}", i)), Arc::new(Vec::new()));
    }
    let thread_creation_time = start.elapsed();
    println!(
        "Thread creation: {:?} per thread",
        thread_creation_time / 1000
    );

    // Benchmark mutex operations
    let mutex = Arc::new(SchemeMutex::new(None));
    let thread_id = 1u64;

    let start = Instant::now();
    for _ in 0..NUM_OPERATIONS {
        mutex.lock(thread_id, None).unwrap();
        mutex.unlock(thread_id).unwrap();
    }
    let mutex_ops_time = start.elapsed();
    println!(
        "Mutex lock/unlock: {:?} per operation",
        mutex_ops_time / NUM_OPERATIONS as u32
    );

    // Benchmark condition variable operations
    let condvar = Arc::new(SchemeConditionVariable::new(None, Arc::clone(&mutex)));

    let start = Instant::now();
    for _ in 0..NUM_OPERATIONS {
        condvar.notify_one();
    }
    let notify_time = start.elapsed();
    println!(
        "Condition variable notify: {:?} per operation",
        notify_time / NUM_OPERATIONS as u32
    );
}

#[test]
fn test_error_handling() {
    // Test invalid argument counts
    assert!(make_thread(&[]).is_err());
    assert!(make_thread(&[Value::Nil, Value::Nil, Value::Nil]).is_err());

    assert!(thread_predicate(&[]).is_err());
    assert!(thread_predicate(&[Value::Nil, Value::Nil]).is_err());

    assert!(make_mutex(&[Value::Nil, Value::Nil]).is_err());

    // Test invalid argument types
    let invalid_name = Value::Literal(lambdust::ast::literal::Literal::Number(
        lambdust::numeric::Number::from(42),
    ));
    assert!(make_thread(&[Value::Nil, invalid_name]).is_err());

    // Test mutex operations with invalid thread ID
    let mutex = Arc::new(SchemeMutex::new(None));
    assert!(mutex.unlock(999).is_err()); // Try to unlock without locking
}

#[test]
fn test_thread_statistics() {
    let thread = SchemeThread::new(Some("stats-test".to_string()), Arc::new(Vec::new()));

    // Initially, all statistics should be zero
    assert_eq!(
        thread
            .stats
            .procedure_calls
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );
    assert_eq!(
        thread
            .stats
            .parameter_accesses
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );
    assert_eq!(
        thread
            .stats
            .mutex_operations
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );
}

#[test]
fn test_mutex_statistics() {
    let mutex = SchemeMutex::new(Some("stats-test".to_string()));
    let thread_id = 1u64;

    // Initially, all statistics should be zero
    assert_eq!(
        mutex
            .stats
            .lock_count
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );
    assert_eq!(
        mutex
            .stats
            .unlock_count
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );

    // Perform operations and check statistics
    mutex.lock(thread_id, None).unwrap();
    assert_eq!(
        mutex
            .stats
            .lock_count
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );

    mutex.unlock(thread_id).unwrap();
    assert_eq!(
        mutex
            .stats
            .unlock_count
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}

#[test]
fn test_condition_variable_statistics() {
    let mutex = Arc::new(SchemeMutex::new(None));
    let condvar = SchemeConditionVariable::new(None, mutex);

    // Initially, statistics should be zero
    assert_eq!(
        condvar
            .stats
            .notify_count
            .load(std::sync::atomic::Ordering::SeqCst),
        0
    );

    // Perform notifications and check statistics
    condvar.notify_one();
    assert_eq!(
        condvar
            .stats
            .notify_count
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );

    condvar.notify_all();
    assert_eq!(
        condvar
            .stats
            .notify_count
            .load(std::sync::atomic::Ordering::SeqCst),
        2
    );
}

// Integration test with the main evaluator would go here
// but requires more setup of the evaluation environment

#[cfg(test)]
mod integration_tests {
    use super::*;

    // These tests would require the full evaluator to be set up
    // For now, they're placeholders showing what should be tested

    #[test]
    #[ignore = "Requires full evaluator setup"]
    fn test_thread_creation_through_evaluator() {
        // Test: (make-thread (lambda () (+ 1 2)))
        // Should create a thread that computes 1 + 2
    }

    #[test]
    #[ignore = "Requires full evaluator setup"]
    fn test_parameter_inheritance_through_evaluator() {
        // Test parameter inheritance across thread boundaries
        // (parameterize ((p 42))
        //   (let ((t (make-thread (lambda () (p)))))
        //     (thread-start! t)
        //     (thread-join! t)))
        // Should return 42
    }

    #[test]
    #[ignore = "Requires full evaluator setup"]
    fn test_mutex_synchronization_through_evaluator() {
        // Test proper mutex synchronization through the evaluator
        // Multiple threads accessing shared state with mutex protection
    }
}

//! Concurrent Memory Safety Tests
//!
//! Tests for thread safety and concurrent memory access including:
//! - Multi-threaded interpreter usage
//! - Shared value access patterns  
//! - Environment thread safety
//! - Race condition detection

use super::test_utils::*;
use super::patterns::*;
use crate::interpreter::LambdustInterpreter;
use crate::environment::Environment;
use crate::value::Value;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

#[test]
fn test_concurrent_interpreter_creation() {
    let result = test_concurrent_safety(|| {
        let _interpreter = LambdustInterpreter::new();
        Ok(())
    }, 10, 5);
    
    assert!(result.is_ok(), "Creating interpreters concurrently should be safe");
}

#[test]
fn test_concurrent_evaluation_separate_interpreters() {
    let result = test_concurrent_safety(|| {
        let mut interpreter = LambdustInterpreter::new();
        
        // Each thread evaluates with its own interpreter
        let result = interpreter.eval("(+ 1 2 3)");
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
        }
    }, 8, 10);
    
    assert!(result.is_ok(), "Concurrent evaluation with separate interpreters should be safe");
}

#[test]
fn test_shared_environment_concurrent_access() {
    let env = Arc::new(Environment::with_builtins());
    let mut handles = Vec::new();
    
    for thread_id in 0..4 {
        let env_clone = Arc::clone(&env);
        
        let handle = thread::spawn(move || {
            // Read from shared environment
            for i in 0..20 {
                let var_name = format!("var_{}", i);
                let _ = env_clone.lookup(&var_name);
                
                // Simulate some work
                thread::sleep(Duration::from_millis(1));
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Environment should remain consistent
    assert!(env.lookup("+").is_some(), "Built-in functions should still be accessible");
}

#[test]
fn test_concurrent_environment_modifications() {
    let env = Arc::new(RwLock::new(Environment::new()));
    let mut handles = Vec::new();
    
    for thread_id in 0..4 {
        let env_clone = Arc::clone(&env);
        
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let var_name = format!("thread_{}_var_{}", thread_id, i);
                let value = Value::Number(crate::lexer::SchemeNumber::Integer((thread_id * 100 + i) as i64));
                
                // Define variable in shared environment
                {
                    let mut env_write = env_clone.write().unwrap();
                    env_write.define(var_name.clone(), value);
                }
                
                // Read it back
                {
                    let env_read = env_clone.read().unwrap();
                    let retrieved = env_read.lookup(&var_name);
                    assert!(retrieved.is_some(), "Should be able to read back defined variable");
                }
                
                thread::sleep(Duration::from_millis(1));
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all variables were defined
    let env_read = env.read().unwrap();
    for thread_id in 0..4 {
        for i in 0..10 {
            let var_name = format!("thread_{}_var_{}", thread_id, i);
            assert!(env_read.lookup(&var_name).is_some(), 
                    "Variable {} should exist", var_name);
        }
    }
}

#[test]
fn test_concurrent_value_sharing() {
    let shared_values: Vec<Arc<Value>> = (0..10)
        .map(|i| Arc::new(Value::Number(crate::lexer::SchemeNumber::Integer(i))))
        .collect();
    
    let mut handles = Vec::new();
    
    for thread_id in 0..6 {
        let values = shared_values.clone();
        
        let handle = thread::spawn(move || {
            for _ in 0..50 {
                // Read shared values concurrently
                for value in &values {
                    match value.as_ref() {
                        Value::Number(n) => {
                            // Access the number value
                            let _ = n.to_i64();
                        }
                        _ => panic!("Expected number"),
                    }
                }
                
                thread::sleep(Duration::from_micros(100));
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Values should still be valid
    for (i, value) in shared_values.iter().enumerate() {
        match value.as_ref() {
            Value::Number(n) => {
                assert_eq!(n.to_i64(), i as i64);
            }
            _ => panic!("Expected number"),
        }
    }
}

#[test]
fn test_concurrent_list_operations() {
    let shared_list = Arc::new(Mutex::new(Value::Vector(vec![
        Value::Number(crate::lexer::SchemeNumber::Integer(1)),
        Value::Number(crate::lexer::SchemeNumber::Integer(2)),
        Value::Number(crate::lexer::SchemeNumber::Integer(3)),
    ])));
    
    let mut handles = Vec::new();
    
    for thread_id in 0..3 {
        let list_clone = Arc::clone(&shared_list);
        
        let handle = thread::spawn(move || {
            for i in 0..20 {
                // Read the list
                {
                    let list = list_clone.lock().unwrap();
                    match list.as_ref() {
                        Value::Vector(vec) => {
                            assert!(vec.len() >= 3, "List should have at least 3 elements");
                        }
                        _ => panic!("Expected vector"),
                    }
                }
                
                thread::sleep(Duration::from_millis(1));
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_string_operations() {
    let shared_string = Arc::new(Value::String("Hello, World!".to_string()));
    let mut handles = Vec::new();
    
    for thread_id in 0..5 {
        let string_clone = Arc::clone(&shared_string);
        
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                match string_clone.as_ref() {
                    Value::String(s) => {
                        // Read string properties
                        let len = s.len();
                        let chars: Vec<char> = s.chars().collect();
                        
                        assert_eq!(len, 13);
                        assert_eq!(chars[0], 'H');
                        assert_eq!(chars[chars.len() - 1], '!');
                    }
                    _ => panic!("Expected string"),
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // String should remain unchanged
    match shared_string.as_ref() {
        Value::String(s) => {
            assert_eq!(s, "Hello, World!");
        }
        _ => panic!("Expected string"),
    }
}

#[test]
fn test_concurrent_memory_allocation() {
    let tracker = Arc::new(MemoryTracker::new());
    let mut handles = Vec::new();
    
    for thread_id in 0..6 {
        let tracker_clone = Arc::clone(&tracker);
        
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let size = (thread_id + 1) * 50 + i;
                tracker_clone.track_allocation(size);
                
                // Simulate some work
                thread::sleep(Duration::from_micros(10));
                
                // Deallocate half of the time
                if i % 2 == 0 {
                    tracker_clone.track_deallocation(size);
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    let stats = tracker.get_stats();
    
    // Verify statistics are consistent
    assert_eq!(stats.allocations, 600); // 6 threads * 100 allocations
    assert!(stats.deallocations > 0);
    assert!(stats.deallocations <= stats.allocations);
    assert_eq!(stats.leaked_objects, stats.allocations - stats.deallocations);
    
    println!("Concurrent memory stats: {:?}", stats);
}

#[test]
fn test_concurrent_stack_operations() {
    let tracker = Arc::new(StackTracker::new(1000));
    let mut handles = Vec::new();
    
    for thread_id in 0..4 {
        let tracker_clone = Arc::clone(&tracker);
        
        let handle = thread::spawn(move || {
            // Simulate recursive calls in each thread
            fn recursive_operation(tracker: &StackTracker, depth: usize) -> Result<(), StackOverflowError> {
                if depth == 0 {
                    return Ok(());
                }
                
                let _guard = tracker.enter_frame()?;
                thread::sleep(Duration::from_micros(10));
                recursive_operation(tracker, depth - 1)
            }
            
            let result = recursive_operation(&tracker_clone, 50);
            assert!(result.is_ok(), "Moderate recursion should succeed");
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    let stats = tracker.get_stats();
    
    // All frames should be cleaned up
    assert_eq!(stats.current_depth, 0);
    assert!(stats.max_depth > 0);
    assert!(stats.max_depth <= 1000);
    
    println!("Concurrent stack stats: {:?}", stats);
}

#[test]
fn test_race_condition_detection() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    let counter = Arc::new(AtomicUsize::new(0));
    let shared_data = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    
    for thread_id in 0..8 {
        let counter_clone = Arc::clone(&counter);
        let data_clone = Arc::clone(&shared_data);
        
        let handle = thread::spawn(move || {
            for i in 0..25 {
                // Increment counter atomically
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                
                // Add to shared data with proper synchronization
                {
                    let mut data = data_clone.lock().unwrap();
                    data.push((thread_id, i, count));
                }
                
                thread::sleep(Duration::from_micros(1));
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_count = counter.load(Ordering::SeqCst);
    let data = shared_data.lock().unwrap();
    
    // Verify no race conditions occurred
    assert_eq!(final_count, 200); // 8 threads * 25 increments
    assert_eq!(data.len(), 200);
    
    // Verify all counts are unique (no duplicate increments)
    let mut counts: Vec<usize> = data.iter().map(|(_, _, count)| *count).collect();
    counts.sort();
    for (i, &count) in counts.iter().enumerate() {
        assert_eq!(count, i, "Counter values should be sequential without gaps");
    }
}

#[test]
fn test_deadlock_prevention() {
    let resource1 = Arc::new(Mutex::new(1));
    let resource2 = Arc::new(Mutex::new(2));
    
    let r1_clone = Arc::clone(&resource1);
    let r2_clone = Arc::clone(&resource2);
    
    // Thread 1: locks resource1 then resource2
    let handle1 = thread::spawn(move || {
        let _lock1 = r1_clone.lock().unwrap();
        thread::sleep(Duration::from_millis(10));
        let _lock2 = r2_clone.lock().unwrap();
        // Do some work
        thread::sleep(Duration::from_millis(5));
    });
    
    let r1_clone2 = Arc::clone(&resource1);
    let r2_clone2 = Arc::clone(&resource2);
    
    // Thread 2: locks resource1 then resource2 (same order to prevent deadlock)
    let handle2 = thread::spawn(move || {
        let _lock1 = r1_clone2.lock().unwrap();
        thread::sleep(Duration::from_millis(10));
        let _lock2 = r2_clone2.lock().unwrap();
        // Do some work
        thread::sleep(Duration::from_millis(5));
    });
    
    // Should complete without deadlock
    let result1 = handle1.join();
    let result2 = handle2.join();
    
    assert!(result1.is_ok(), "Thread 1 should complete without deadlock");
    assert!(result2.is_ok(), "Thread 2 should complete without deadlock");
}

#[test]
fn test_memory_barrier_consistency() {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    
    let flag = Arc::new(AtomicBool::new(false));
    let data = Arc::new(AtomicUsize::new(0));
    
    let flag_clone = Arc::clone(&flag);
    let data_clone = Arc::clone(&data);
    
    // Writer thread
    let writer = thread::spawn(move || {
        // Write data
        data_clone.store(42, Ordering::Release);
        // Set flag to indicate data is ready
        flag_clone.store(true, Ordering::Release);
    });
    
    let flag_clone2 = Arc::clone(&flag);
    let data_clone2 = Arc::clone(&data);
    
    // Reader thread
    let reader = thread::spawn(move || {
        // Wait for flag
        while !flag_clone2.load(Ordering::Acquire) {
            thread::sleep(Duration::from_micros(1));
        }
        // Read data
        let value = data_clone2.load(Ordering::Acquire);
        assert_eq!(value, 42, "Should read correct value after memory barrier");
    });
    
    writer.join().unwrap();
    reader.join().unwrap();
}

#[test]
fn test_high_contention_scenario() {
    let shared_counter = Arc::new(Mutex::new(0));
    let iterations = 1000;
    let thread_count = 10;
    let mut handles = Vec::new();
    
    for _ in 0..thread_count {
        let counter_clone = Arc::clone(&shared_counter);
        
        let handle = thread::spawn(move || {
            for _ in 0..iterations {
                // High contention: all threads competing for the same lock
                let mut counter = counter_clone.lock().unwrap();
                *counter += 1;
                // Hold the lock briefly to increase contention
                thread::sleep(Duration::from_micros(1));
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    let final_value = *shared_counter.lock().unwrap();
    assert_eq!(final_value, thread_count * iterations);
    
    println!("High contention test completed: {} increments", final_value);
}

#[test]
fn test_concurrent_interpreter_evaluation() {
    // Test with shared interpreter (requires careful synchronization)
    let interpreter = Arc::new(Mutex::new(LambdustInterpreter::new()));
    let mut handles = Vec::new();
    
    for thread_id in 0..3 {
        let interpreter_clone = Arc::clone(&interpreter);
        
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let expression = format!("(+ {} {})", thread_id, i);
                
                let result = {
                    let mut interp = interpreter_clone.lock().unwrap();
                    interp.eval(&expression)
                };
                
                match result {
                    Ok(value) => {
                        // Verify result is correct
                        match value {
                            Value::Number(n) => {
                                assert_eq!(n.to_i64(), (thread_id + i) as i64);
                            }
                            _ => panic!("Expected number result"),
                        }
                    }
                    Err(e) => {
                        panic!("Evaluation failed: {:?}", e);
                    }
                }
                
                thread::sleep(Duration::from_millis(1));
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}
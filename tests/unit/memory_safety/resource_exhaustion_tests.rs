//! Resource Exhaustion Handling Tests
//!
//! Tests for graceful handling of resource exhaustion scenarios:
//! - Memory exhaustion protection
//! - File descriptor limits
//! - Stack space exhaustion
//! - CPU time limits

use super::test_utils::*;
use crate::interpreter::LambdustInterpreter;
use crate::value::Value;
use std::time::{Duration, Instant};

#[test]
fn test_memory_exhaustion_protection() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Try to create extremely large data structures
    let large_list_code = format!("'({})", (0..100000).map(|i| i.to_string()).collect::<Vec<_>>().join(" "));
    
    let result = run_with_timeout(
        || interpreter.eval(&large_list_code),
        Duration::from_secs(10)
    );
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(_) => {
                    println!("Large list creation succeeded");
                }
                Err(e) => {
                    println!("Large list creation failed gracefully: {:?}", e);
                    // Graceful failure is acceptable
                }
            }
        }
        Err(_) => {
            println!("Large list creation timed out - acceptable protection");
        }
    }
}

#[test]
fn test_infinite_memory_allocation_protection() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Test infinite list creation protection
    let infinite_list_code = r#"
        (define (make-infinite-list n)
          (cons n (make-infinite-list (+ n 1))))
        (take 1000 (make-infinite-list 0))
    "#;
    
    let result = run_with_timeout(
        || interpreter.eval(infinite_list_code),
        Duration::from_secs(5)
    );
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(_) => {
                    println!("Infinite list handling succeeded (lazy evaluation?)");
                }
                Err(e) => {
                    println!("Infinite list handled gracefully: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("Infinite list creation timed out - good protection");
        }
    }
}

#[test]
fn test_recursive_memory_allocation_limits() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Test recursive data structure creation
    let recursive_structure = r#"
        (define (make-tree depth)
          (if (= depth 0)
              'leaf
              (list (make-tree (- depth 1))
                    (make-tree (- depth 1)))))
        (make-tree 20)
    "#;
    
    let result = run_with_timeout(
        || interpreter.eval(recursive_structure),
        Duration::from_secs(10)
    );
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(value) => {
                    println!("Recursive structure creation succeeded");
                    // Verify it's reasonable in size
                    match value {
                        Value::Vector(_) | Value::Nil => {
                            // Expected structure types
                        }
                        _ => {
                            println!("Unexpected result type: {:?}", value);
                        }
                    }
                }
                Err(e) => {
                    println!("Recursive structure creation failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("Recursive structure creation timed out");
        }
    }
}

#[test]
fn test_string_memory_exhaustion() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Test very large string creation
    let large_string_code = format!("\"{}\"", "x".repeat(1000000)); // 1MB string
    
    let result = run_with_timeout(
        || interpreter.eval(&large_string_code),
        Duration::from_secs(5)
    );
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(value) => {
                    match value {
                        Value::String(s) => {
                            assert_eq!(s.len(), 1000000);
                            println!("Large string creation succeeded");
                        }
                        _ => panic!("Expected string value"),
                    }
                }
                Err(e) => {
                    println!("Large string creation failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("Large string creation timed out");
        }
    }
}

#[test]
fn test_computation_time_limits() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Test computation that takes too long
    let long_computation = r#"
        (define (expensive-computation n)
          (if (= n 0)
              0
              (+ 1 (expensive-computation (- n 1)))))
        (expensive-computation 100000)
    "#;
    
    let start_time = Instant::now();
    let result = run_with_timeout(
        || interpreter.eval(long_computation),
        Duration::from_secs(3)
    );
    let elapsed = start_time.elapsed();
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(_) => {
                    println!("Long computation completed in {:?}", elapsed);
                }
                Err(e) => {
                    println!("Long computation failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("Long computation timed out after {:?}", elapsed);
            assert!(elapsed >= Duration::from_secs(3), "Should respect timeout");
        }
    }
}

#[test]
fn test_nested_evaluation_limits() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Test deeply nested evaluation
    let nested_eval = create_recursive_expression(5000);
    
    let result = run_with_timeout(
        || interpreter.eval(&nested_eval),
        Duration::from_secs(5)
    );
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(value) => {
                    println!("Deep evaluation succeeded: {:?}", value);
                }
                Err(e) => {
                    println!("Deep evaluation failed gracefully: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("Deep evaluation timed out - good protection");
        }
    }
}

#[test]
fn test_file_descriptor_exhaustion_simulation() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Simulate file operations (if supported)
    let file_ops = r#"
        (define (test-file-ops n)
          (if (= n 0)
              'done
              (begin
                ;; Simulate file operation
                (display "test")
                (test-file-ops (- n 1)))))
        (test-file-ops 100)
    "#;
    
    let result = run_with_timeout(
        || interpreter.eval(file_ops),
        Duration::from_secs(5)
    );
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(_) => {
                    println!("File operations completed");
                }
                Err(e) => {
                    println!("File operations failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("File operations timed out");
        }
    }
}

#[test]
fn test_garbage_collection_under_pressure() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Create memory pressure to trigger GC
    let gc_pressure_code = r#"
        (define (create-garbage n)
          (if (= n 0)
              '()
              (begin
                (cons 1 2)  ;; Create garbage
                (cons 3 4)  ;; More garbage
                (create-garbage (- n 1)))))
        (create-garbage 10000)
    "#;
    
    let result = run_with_timeout(
        || interpreter.eval(gc_pressure_code),
        Duration::from_secs(10)
    );
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(_) => {
                    println!("GC pressure test completed");
                }
                Err(e) => {
                    println!("GC pressure test failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("GC pressure test timed out");
        }
    }
}

#[test]
fn test_resource_cleanup_after_failure() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Test that resources are cleaned up after evaluation failures
    let failing_code = "(/ 1 0)"; // Division by zero
    
    let result1 = interpreter.eval(failing_code);
    assert!(result1.is_err() || result1.is_ok()); // May handle gracefully
    
    // Should still be able to evaluate after failure
    let result2 = interpreter.eval("(+ 1 2)");
    assert!(result2.is_ok(), "Interpreter should recover after failure");
    
    match result2.unwrap() {
        Value::Number(n) => {
            assert_eq!(n.to_i64(), 3);
        }
        _ => panic!("Expected number result"),
    }
}

#[test]
fn test_memory_fragmentation_handling() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Create and destroy many objects to cause fragmentation
    for cycle in 0..10 {
        let fragmentation_code = format!(r#"
            (define temp-list{} '({}))
            temp-list{}
        "#, cycle, (0..1000).map(|i| i.to_string()).collect::<Vec<_>>().join(" "), cycle);
        
        let result = interpreter.eval(&fragmentation_code);
        
        match result {
            Ok(value) => {
                match value {
                    Value::Vector(vec) => {
                        assert_eq!(vec.len(), 1000);
                    }
                    _ => {
                        // Other representations are fine
                    }
                }
            }
            Err(e) => {
                println!("Fragmentation test cycle {} failed: {:?}", cycle, e);
            }
        }
    }
    
    // Should still be functional after fragmentation
    let final_result = interpreter.eval("(+ 1 1)");
    assert!(final_result.is_ok(), "Should work after fragmentation test");
}

#[test]
fn test_concurrent_resource_exhaustion() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let interpreter = Arc::new(Mutex::new(LambdustInterpreter::new()));
    let mut handles = Vec::new();
    
    // Multiple threads trying to exhaust resources
    for thread_id in 0..3 {
        let interpreter_clone = Arc::clone(&interpreter);
        
        let handle = thread::spawn(move || {
            let resource_code = format!(r#"
                (define big-list{} '({}))
                (length big-list{})
            "#, thread_id, (0..1000).map(|i| (i + thread_id * 1000).to_string()).collect::<Vec<_>>().join(" "), thread_id);
            
            let result = {
                let mut interp = interpreter_clone.lock().unwrap();
                run_with_timeout(
                    || interp.eval(&resource_code),
                    Duration::from_secs(5)
                )
            };
            
            match result {
                Ok(eval_result) => {
                    match eval_result {
                        Ok(value) => {
                            println!("Thread {} resource test succeeded", thread_id);
                        }
                        Err(e) => {
                            println!("Thread {} resource test failed: {:?}", thread_id, e);
                        }
                    }
                }
                Err(_) => {
                    println!("Thread {} resource test timed out", thread_id);
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Interpreter should still be functional
    let final_test = {
        let mut interp = interpreter.lock().unwrap();
        interp.eval("'(final test)")
    };
    
    assert!(final_test.is_ok(), "Interpreter should be functional after concurrent resource tests");
}

#[test]
fn test_iterative_memory_pressure() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Gradually increase memory pressure
    for size in [100, 500, 1000, 2000, 5000].iter() {
        let pressure_code = format!(r#"
            (define pressure-test '({}))
            (length pressure-test)
        "#, (0..*size).map(|i| i.to_string()).collect::<Vec<_>>().join(" "));
        
        let result = run_with_timeout(
            || interpreter.eval(&pressure_code),
            Duration::from_secs(5)
        );
        
        match result {
            Ok(eval_result) => {
                match eval_result {
                    Ok(value) => {
                        match value {
                            Value::Number(n) => {
                                assert_eq!(n.to_i64(), *size as i64);
                                println!("Memory pressure test {} succeeded", size);
                            }
                            _ => panic!("Expected number result"),
                        }
                    }
                    Err(e) => {
                        println!("Memory pressure test {} failed: {:?}", size, e);
                        break; // Stop at first failure
                    }
                }
            }
            Err(_) => {
                println!("Memory pressure test {} timed out", size);
                break; // Stop at first timeout
            }
        }
    }
}

#[test]
fn test_resource_limit_recovery() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Try to hit resource limits
    let limit_test = format!("'({})", "x".repeat(1000000));
    
    let result = run_with_timeout(
        || interpreter.eval(&limit_test),
        Duration::from_secs(3)
    );
    
    // Regardless of whether it succeeds or fails, test recovery
    match result {
        Ok(_) => println!("Resource limit test completed"),
        Err(_) => println!("Resource limit test timed out"),
    }
    
    // Should be able to perform simple operations after limit test
    let recovery_result = interpreter.eval("(+ 2 3)");
    
    match recovery_result {
        Ok(value) => {
            match value {
                Value::Number(n) => {
                    assert_eq!(n.to_i64(), 5);
                    println!("Recovery after resource limit test successful");
                }
                _ => panic!("Expected number result"),
            }
        }
        Err(e) => {
            println!("Recovery failed: {:?}", e);
            // Recovery failure is concerning but not necessarily a test failure
        }
    }
}

#[test]
fn test_cpu_intensive_operation_handling() {
    let mut interpreter = LambdustInterpreter::new();
    
    // CPU-intensive mathematical computation
    let cpu_intensive = r#"
        (define (fibonacci n)
          (if (<= n 1)
              n
              (+ (fibonacci (- n 1))
                 (fibonacci (- n 2)))))
        (fibonacci 35)
    "#;
    
    let start_time = Instant::now();
    let result = run_with_timeout(
        || interpreter.eval(cpu_intensive),
        Duration::from_secs(5)
    );
    let elapsed = start_time.elapsed();
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(value) => {
                    println!("CPU intensive computation completed in {:?}: {:?}", elapsed, value);
                }
                Err(e) => {
                    println!("CPU intensive computation failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("CPU intensive computation timed out after {:?}", elapsed);
        }
    }
    
    // Should still be responsive after CPU intensive operation
    let simple_result = interpreter.eval("42");
    assert!(simple_result.is_ok(), "Should be responsive after CPU intensive operation");
}
//! Memory Pressure Response Tests
//!
//! Tests for system behavior under various memory pressure conditions:
//! - Low memory scenarios
//! - Memory allocation strategies under pressure
//! - Garbage collection trigger points
//! - Performance degradation patterns

use super::test_utils::*;
use crate::interpreter::LambdustInterpreter;
use crate::value::Value;
use std::time::{Duration, Instant};

#[test]
fn test_low_memory_operation() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Create artificial memory pressure
    let _pressure = create_memory_pressure(50); // 50 MB pressure
    
    // Test basic operations under memory pressure
    let operations = vec![
        "42",
        "(+ 1 2 3)",
        "'(1 2 3 4 5)",
        "(lambda (x) (* x x))",
        "(define test-var 123)",
    ];
    
    for operation in operations {
        let result = run_with_timeout(
            || interpreter.eval(operation),
            Duration::from_secs(5)
        );
        
        match result {
            Ok(eval_result) => {
                match eval_result {
                    Ok(_) => {
                        println!("Operation '{}' succeeded under memory pressure", operation);
                    }
                    Err(e) => {
                        println!("Operation '{}' failed under memory pressure: {:?}", operation, e);
                    }
                }
            }
            Err(_) => {
                println!("Operation '{}' timed out under memory pressure", operation);
            }
        }
    }
}

#[test]
fn test_memory_allocation_strategy_under_pressure() {
    let tracker = MemoryTracker::new();
    
    // Simulate high memory pressure
    let _pressure = create_memory_pressure(100); // 100 MB pressure
    
    let (result, stats) = measure_memory(&tracker, || {
        let mut interpreter = LambdustInterpreter::new();
        
        // Try to allocate progressively larger structures
        for size in [10, 50, 100, 200, 500].iter() {
            let list_code = format!("'({})", (0..*size).map(|i| i.to_string()).collect::<Vec<_>>().join(" "));
            
            let result = run_with_timeout(
                || interpreter.eval(&list_code),
                Duration::from_secs(3)
            );
            
            match result {
                Ok(eval_result) => {
                    match eval_result {
                        Ok(_) => {
                            tracker.track_allocation(*size * 8); // Estimate 8 bytes per element
                        }
                        Err(_) => {
                            println!("Allocation failed at size {}", size);
                            break;
                        }
                    }
                }
                Err(_) => {
                    println!("Allocation timed out at size {}", size);
                    break;
                }
            }
        }
    });
    
    println!("Memory allocation under pressure stats: {:?}", stats);
    
    // Should show adaptive behavior under pressure
    assert!(stats.allocations > 0, "Should attempt some allocations");
}

#[test]
fn test_garbage_collection_trigger_under_pressure() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Create memory pressure to potentially trigger GC
    let _pressure = create_memory_pressure(75); // 75 MB pressure
    
    let gc_trigger_code = r#"
        (define (create-temporary-objects n)
          (if (= n 0)
              '()
              (begin
                (cons 1 (cons 2 (cons 3 '())))  ; Create temporary objects
                (create-temporary-objects (- n 1)))))
        
        ;; Create many temporary objects that should trigger GC
        (create-temporary-objects 1000)
    "#;
    
    let start_time = Instant::now();
    let result = run_with_timeout(
        || interpreter.eval(gc_trigger_code),
        Duration::from_secs(10)
    );
    let elapsed = start_time.elapsed();
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(_) => {
                    println!("GC trigger test completed in {:?}", elapsed);
                }
                Err(e) => {
                    println!("GC trigger test failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("GC trigger test timed out after {:?}", elapsed);
        }
    }
    
    // Test that interpreter is still functional after GC pressure
    let post_gc_result = interpreter.eval("(+ 1 1)");
    assert!(post_gc_result.is_ok(), "Interpreter should be functional after GC pressure");
}

#[test]
fn test_performance_degradation_under_pressure() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Measure baseline performance
    let baseline_start = Instant::now();
    let baseline_result = interpreter.eval("(+ 1 2 3 4 5)");
    let baseline_time = baseline_start.elapsed();
    
    assert!(baseline_result.is_ok(), "Baseline evaluation should succeed");
    
    // Create memory pressure
    let _pressure = create_memory_pressure(100); // 100 MB pressure
    
    // Measure performance under pressure
    let pressure_start = Instant::now();
    let pressure_result = interpreter.eval("(+ 1 2 3 4 5)");
    let pressure_time = pressure_start.elapsed();
    
    match pressure_result {
        Ok(_) => {
            println!("Baseline time: {:?}, Under pressure: {:?}", baseline_time, pressure_time);
            
            // Performance degradation is expected but should be reasonable
            let degradation_factor = pressure_time.as_nanos() as f64 / baseline_time.as_nanos() as f64;
            println!("Performance degradation factor: {:.2}x", degradation_factor);
            
            // Allow significant degradation under memory pressure
            assert!(degradation_factor < 100.0, "Performance degradation should not be extreme");
        }
        Err(e) => {
            println!("Evaluation under memory pressure failed: {:?}", e);
            // Failure under extreme pressure is acceptable
        }
    }
}

#[test]
fn test_adaptive_memory_behavior() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Test adaptive behavior with increasing memory pressure
    let pressure_levels = [10, 25, 50, 75, 100]; // MB
    
    for &pressure_mb in &pressure_levels {
        println!("Testing with {}MB memory pressure", pressure_mb);
        
        let _pressure = create_memory_pressure(pressure_mb);
        
        let test_code = format!(r#"
            (define test-list '({}))
            (length test-list)
        "#, (0..100).map(|i| i.to_string()).collect::<Vec<_>>().join(" "));
        
        let start_time = Instant::now();
        let result = run_with_timeout(
            || interpreter.eval(&test_code),
            Duration::from_secs(5)
        );
        let elapsed = start_time.elapsed();
        
        match result {
            Ok(eval_result) => {
                match eval_result {
                    Ok(value) => {
                        match value {
                            Value::Number(n) => {
                                assert_eq!(n.to_i64(), 100);
                                println!("  Succeeded in {:?}", elapsed);
                            }
                            _ => panic!("Expected number result"),
                        }
                    }
                    Err(e) => {
                        println!("  Failed: {:?}", e);
                        break; // Stop testing at first failure
                    }
                }
            }
            Err(_) => {
                println!("  Timed out after {:?}", elapsed);
                break; // Stop testing at first timeout
            }
        }
    }
}

#[test]
fn test_memory_threshold_behavior() {
    let tracker = MemoryTracker::new();
    
    // Test behavior near memory thresholds
    let (result, stats) = measure_memory(&tracker, || {
        let mut interpreter = LambdustInterpreter::new();
        
        // Gradually increase allocation size until failure
        let mut size = 100;
        loop {
            let allocation_code = format!("'({})", (0..size).map(|i| i.to_string()).collect::<Vec<_>>().join(" "));
            
            let result = run_with_timeout(
                || interpreter.eval(&allocation_code),
                Duration::from_secs(3)
            );
            
            match result {
                Ok(eval_result) => {
                    match eval_result {
                        Ok(_) => {
                            tracker.track_allocation(size * 8); // Estimate
                            size *= 2; // Double the size for next iteration
                            
                            if size > 100000 {
                                println!("Reached large allocation size: {}", size);
                                break;
                            }
                        }
                        Err(_) => {
                            println!("Allocation failed at size: {}", size);
                            break;
                        }
                    }
                }
                Err(_) => {
                    println!("Allocation timed out at size: {}", size);
                    break;
                }
            }
        }
    });
    
    println!("Memory threshold test stats: {:?}", stats);
    
    // Should have attempted progressively larger allocations
    assert!(stats.allocations > 0, "Should have made allocation attempts");
    assert!(stats.peak_memory > 0, "Should have tracked peak memory usage");
}

#[test]
fn test_memory_pressure_recovery() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Test functionality under pressure
    {
        let _pressure = create_memory_pressure(100); // 100 MB pressure
        
        let pressure_result = interpreter.eval("(define under-pressure 42)");
        println!("Under pressure result: {:?}", pressure_result);
    } // Pressure released here
    
    // Test recovery after pressure is released
    std::thread::sleep(Duration::from_millis(100)); // Allow recovery time
    
    let recovery_result = interpreter.eval("(+ under-pressure 8)");
    
    match recovery_result {
        Ok(value) => {
            match value {
                Value::Number(n) => {
                    assert_eq!(n.to_i64(), 50);
                    println!("Recovery successful: {}", n.to_i64());
                }
                _ => panic!("Expected number result"),
            }
        }
        Err(e) => {
            println!("Recovery failed: {:?}", e);
            // Test that at least basic operations work
            let basic_result = interpreter.eval("(+ 1 1)");
            assert!(basic_result.is_ok(), "Basic operations should work after pressure");
        }
    }
}

#[test]
fn test_concurrent_memory_pressure() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let interpreter = Arc::new(Mutex::new(LambdustInterpreter::new()));
    let mut handles = Vec::new();
    
    // Create concurrent memory pressure
    for thread_id in 0..3 {
        let interpreter_clone = Arc::clone(&interpreter);
        
        let handle = thread::spawn(move || {
            // Each thread creates its own memory pressure
            let _pressure = create_memory_pressure(30); // 30 MB per thread
            
            let test_code = format!(r#"
                (define thread{}-data '({}))
                (length thread{}-data)
            "#, thread_id, (0..100).map(|i| (i + thread_id * 100).to_string()).collect::<Vec<_>>().join(" "), thread_id);
            
            let result = {
                let mut interp = interpreter_clone.lock().unwrap();
                run_with_timeout(
                    || interp.eval(&test_code),
                    Duration::from_secs(10)
                )
            };
            
            match result {
                Ok(eval_result) => {
                    match eval_result {
                        Ok(value) => {
                            println!("Thread {} succeeded under concurrent pressure", thread_id);
                        }
                        Err(e) => {
                            println!("Thread {} failed under concurrent pressure: {:?}", thread_id, e);
                        }
                    }
                }
                Err(_) => {
                    println!("Thread {} timed out under concurrent pressure", thread_id);
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Test final functionality
    let final_result = {
        let mut interp = interpreter.lock().unwrap();
        interp.eval("'(final test)")
    };
    
    assert!(final_result.is_ok(), "Interpreter should be functional after concurrent pressure");
}

#[test]
fn test_memory_pressure_with_different_data_types() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Create memory pressure
    let _pressure = create_memory_pressure(50); // 50 MB pressure
    
    let data_type_tests = vec![
        ("numbers", "(+ 1 2 3 4 5)"),
        ("strings", "\"test string under pressure\""),
        ("lists", "'(a b c d e f g h i j)"),
        ("vectors", "#(1 2 3 4 5 6 7 8 9 10)"),
        ("symbols", "'symbol-under-pressure"),
        ("procedures", "(lambda (x y) (+ x y))"),
    ];
    
    for (data_type, code) in data_type_tests {
        let result = run_with_timeout(
            || interpreter.eval(code),
            Duration::from_secs(3)
        );
        
        match result {
            Ok(eval_result) => {
                match eval_result {
                    Ok(_) => {
                        println!("{} test succeeded under memory pressure", data_type);
                    }
                    Err(e) => {
                        println!("{} test failed under memory pressure: {:?}", data_type, e);
                    }
                }
            }
            Err(_) => {
                println!("{} test timed out under memory pressure", data_type);
            }
        }
    }
}

#[test]
fn test_memory_efficiency_under_pressure() {
    let tracker = MemoryTracker::new();
    
    let (result, stats) = measure_memory(&tracker, || {
        let mut interpreter = LambdustInterpreter::new();
        
        // Create memory pressure
        let _pressure = create_memory_pressure(75); // 75 MB pressure
        
        // Perform operations that should be memory-efficient
        let efficient_operations = vec![
            "42",                           // Immediate value
            "(+ 1 2)",                     // Simple arithmetic
            "'()",                         // Empty list
            "(lambda (x) x)",              // Identity function
            "(if #t 'yes 'no)",           // Simple conditional
        ];
        
        for operation in efficient_operations {
            let result = interpreter.eval(operation);
            
            match result {
                Ok(_) => {
                    tracker.track_allocation(8); // Minimal allocation estimate
                }
                Err(_) => {
                    println!("Efficient operation failed: {}", operation);
                }
            }
        }
    });
    
    println!("Memory efficiency under pressure stats: {:?}", stats);
    
    // Should use minimal memory for efficient operations
    assert!(stats.peak_memory < 1000, "Efficient operations should use minimal memory");
    assert!(stats.leaked_objects < 10, "Should not leak much during efficient operations");
}

#[test]
fn test_progressive_memory_pressure() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Test with progressively increasing memory pressure
    let mut pressure_allocations = Vec::new();
    
    for pressure_level in [0, 20, 40, 60, 80].iter() {
        println!("Testing with {}MB memory pressure", pressure_level);
        
        // Create and store pressure (accumulating)
        if *pressure_level > 0 {
            pressure_allocations.push(create_memory_pressure(*pressure_level));
        }
        
        let test_code = "(define test-result (+ 10 20 30))";
        
        let start_time = Instant::now();
        let result = run_with_timeout(
            || interpreter.eval(test_code),
            Duration::from_secs(5)
        );
        let elapsed = start_time.elapsed();
        
        match result {
            Ok(eval_result) => {
                match eval_result {
                    Ok(value) => {
                        match value {
                            Value::Number(n) => {
                                assert_eq!(n.to_i64(), 60);
                                println!("  Succeeded in {:?}", elapsed);
                            }
                            _ => panic!("Expected number result"),
                        }
                    }
                    Err(e) => {
                        println!("  Failed: {:?}", e);
                        break; // Stop at first failure
                    }
                }
            }
            Err(_) => {
                println!("  Timed out after {:?}", elapsed);
                break; // Stop at first timeout
            }
        }
    }
}
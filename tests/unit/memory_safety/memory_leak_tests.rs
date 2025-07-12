//! Memory Leak Detection Tests
//!
//! Tests for detecting and preventing memory leaks in:
//! - Value allocation and deallocation
//! - Environment cleanup
//! - Continuation memory management
//! - Circular reference detection

use super::test_utils::*;
use super::patterns::*;
use crate::interpreter::LambdustInterpreter;
use crate::value::Value;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_basic_memory_tracking() {
    let tracker = MemoryTracker::new();
    
    // Simulate allocations
    tracker.track_allocation(100);
    tracker.track_allocation(200);
    
    let stats = tracker.get_stats();
    assert_eq!(stats.allocations, 2);
    assert_eq!(stats.current_memory, 300);
    assert_eq!(stats.peak_memory, 300);
    
    // Simulate deallocations
    tracker.track_deallocation(100);
    
    let stats = tracker.get_stats();
    assert_eq!(stats.deallocations, 1);
    assert_eq!(stats.current_memory, 200);
    assert_eq!(stats.leaked_objects, 1); // 2 allocs - 1 dealloc
}

#[test]
fn test_memory_leak_detection_pattern() {
    let result = test_memory_leak_detection(|tracker| {
        // Simulate normal allocation/deallocation cycle
        tracker.track_allocation(100);
        tracker.track_allocation(200);
        tracker.track_deallocation(100);
        tracker.track_deallocation(200);
    }, 0); // No leaks allowed
    
    assert!(result.is_ok(), "Balanced allocation/deallocation should not leak");
    
    let leak_result = test_memory_leak_detection(|tracker| {
        // Simulate memory leak
        tracker.track_allocation(100);
        tracker.track_allocation(200);
        tracker.track_deallocation(100);
        // Missing deallocation for 200 bytes
    }, 0); // No leaks allowed
    
    assert!(leak_result.is_err(), "Missing deallocation should be detected as leak");
}

#[test]
fn test_value_allocation_cleanup() {
    let (mut interpreter, tracker) = create_tracked_interpreter();
    
    let (result, stats) = measure_memory(&tracker, || {
        // Create and evaluate various values
        let _ = interpreter.eval("42");
        let _ = interpreter.eval("\"hello world\"");
        let _ = interpreter.eval("'(1 2 3 4 5)");
        let _ = interpreter.eval("(lambda (x) (* x x))");
        let _ = interpreter.eval("(define test-var 123)");
    });
    
    println!("Memory stats after value operations: {:?}", stats);
    
    // Allow some leeway for interpreter internals
    assert!(stats.leaked_objects < 10, "Should not leak many objects during basic operations");
}

#[test]
fn test_large_data_structure_cleanup() {
    let (mut interpreter, tracker) = create_tracked_interpreter();
    
    let (result, stats) = measure_memory(&tracker, || {
        // Create large list
        let large_list_code = format!("'({})", (0..1000).map(|i| i.to_string()).collect::<Vec<_>>().join(" "));
        let _ = interpreter.eval(&large_list_code);
        
        // Create large vector
        let large_vector_code = format!("#({})", (0..1000).map(|i| i.to_string()).collect::<Vec<_>>().join(" "));
        let _ = interpreter.eval(&large_vector_code);
    });
    
    println!("Memory stats after large data structures: {:?}", stats);
    
    // Should not leak excessively
    assert!(stats.leaked_objects < 100, "Large data structures should not cause excessive leaks");
}

#[test]
fn test_recursive_data_structure_cleanup() {
    let (mut interpreter, tracker) = create_tracked_interpreter();
    
    let (result, stats) = measure_memory(&tracker, || {
        // Create nested structures
        let nested_code = r#"
            (define nested-list 
              '((1 (2 (3 (4 (5))))) 
                (a (b (c (d (e)))))
                (x (y (z)))))
        "#;
        let _ = interpreter.eval(nested_code);
        
        // Create circular references (if supported)
        let circular_code = r#"
            (define circular (cons 1 2))
            (set-cdr! circular circular)
        "#;
        let _ = interpreter.eval(circular_code);
    });
    
    println!("Memory stats after recursive structures: {:?}", stats);
    
    // Circular references are challenging but should be handled
    assert!(stats.leaked_objects < 50, "Recursive structures should not cause excessive leaks");
}

#[test]
fn test_environment_cleanup() {
    let (mut interpreter, tracker) = create_tracked_interpreter();
    
    let (result, stats) = measure_memory(&tracker, || {
        // Create nested environments
        let env_code = r#"
            (let ((a 1) (b 2) (c 3))
              (let ((d 4) (e 5) (f 6))
                (let ((g 7) (h 8) (i 9))
                  (+ a b c d e f g h i))))
        "#;
        let _ = interpreter.eval(env_code);
        
        // Test environment cleanup with function definitions
        let func_code = r#"
            (define (outer x)
              (define (inner y)
                (+ x y))
              inner)
            (define my-func (outer 10))
            (my-func 5)
        "#;
        let _ = interpreter.eval(func_code);
    });
    
    println!("Memory stats after environment operations: {:?}", stats);
    
    // Environments should be properly cleaned up
    assert!(stats.leaked_objects < 20, "Environment cleanup should not leak significantly");
}

#[test]
fn test_continuation_memory_management() {
    let (mut interpreter, tracker) = create_tracked_interpreter();
    
    let (result, stats) = measure_memory(&tracker, || {
        // Test call/cc if available
        let callcc_code = r#"
            (call/cc
              (lambda (escape)
                (let loop ((n 100))
                  (if (= n 0)
                      'done
                      (loop (- n 1))))))
        "#;
        let _ = interpreter.eval(callcc_code);
        
        // Test exception handling if available
        let exception_code = r#"
            (with-exception-handler
              (lambda (condition) 'handled)
              (lambda () (error "test error")))
        "#;
        let _ = interpreter.eval(exception_code);
    });
    
    println!("Memory stats after continuation operations: {:?}", stats);
    
    // Continuations can be complex but should not leak excessively
    assert!(stats.leaked_objects < 30, "Continuation operations should not cause excessive leaks");
}

#[test]
fn test_repeated_operations_memory_stability() {
    let (mut interpreter, tracker) = create_tracked_interpreter();
    
    let iterations = 100;
    
    let (result, stats) = measure_memory(&tracker, || {
        for i in 0..iterations {
            // Perform various operations repeatedly
            let _ = interpreter.eval(&format!("(+ {} {})", i, i + 1));
            let _ = interpreter.eval(&format!("(define temp{} {})", i, i * 2));
            let _ = interpreter.eval(&format!("'(item{})", i));
            
            // Clean up temporary definitions periodically
            if i % 10 == 0 {
                // Force some cleanup if possible
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    });
    
    println!("Memory stats after {} iterations: {:?}", iterations, stats);
    
    // Memory usage should be reasonable for repeated operations
    let max_reasonable_leaks = iterations / 10; // Allow 10% leak rate
    assert!(stats.leaked_objects < max_reasonable_leaks, 
            "Repeated operations should not cause excessive memory leaks");
}

#[test]
fn test_string_memory_management() {
    let (mut interpreter, tracker) = create_tracked_interpreter();
    
    let (result, stats) = measure_memory(&tracker, || {
        // Create many strings
        for i in 0..100 {
            let string_code = format!("\"string number {}\"", i);
            let _ = interpreter.eval(&string_code);
            
            // String operations
            let _ = interpreter.eval(&format!("(string-append \"prefix\" \"{}\")", i));
            let _ = interpreter.eval(&format!("(string-length \"test{}\")", i));
        }
        
        // Large string operations
        let large_string = "x".repeat(10000);
        let _ = interpreter.eval(&format!("\"{}\"", large_string));
    });
    
    println!("Memory stats after string operations: {:?}", stats);
    
    // String memory management should be efficient
    assert!(stats.leaked_objects < 50, "String operations should not leak excessively");
}

#[test]
fn test_procedure_memory_management() {
    let (mut interpreter, tracker) = create_tracked_interpreter();
    
    let (result, stats) = measure_memory(&tracker, || {
        // Create many procedures
        for i in 0..50 {
            let proc_code = format!("(lambda (x) (+ x {}))", i);
            let _ = interpreter.eval(&proc_code);
            
            // Define named procedures
            let def_code = format!("(define (func{} x) (* x {}))", i, i + 1);
            let _ = interpreter.eval(&def_code);
        }
        
        // Test procedure calls
        for i in 0..10 {
            let call_code = format!("(func{} {})", i, i * 2);
            let _ = interpreter.eval(&call_code);
        }
    });
    
    println!("Memory stats after procedure operations: {:?}", stats);
    
    // Procedure memory should be managed properly
    assert!(stats.leaked_objects < 100, "Procedure operations should not leak excessively");
}

#[test]
fn test_macro_memory_management() {
    let (mut interpreter, tracker) = create_tracked_interpreter();
    
    let (result, stats) = measure_memory(&tracker, || {
        // Define macros
        let macro_code = r#"
            (define-syntax when
              (syntax-rules ()
                ((when test stmt1 stmt2 ...)
                 (if test (begin stmt1 stmt2 ...)))))
        "#;
        let _ = interpreter.eval(macro_code);
        
        // Use macros repeatedly
        for i in 0..20 {
            let usage_code = format!("(when (> {} 0) (+ {} 1))", i, i);
            let _ = interpreter.eval(&usage_code);
        }
    });
    
    println!("Memory stats after macro operations: {:?}", stats);
    
    // Macro expansion should not leak significantly
    assert!(stats.leaked_objects < 30, "Macro operations should not leak excessively");
}

#[test]
fn test_garbage_collection_simulation() {
    let tracker = MemoryTracker::new();
    
    // Simulate allocation pattern that would benefit from GC
    for cycle in 0..10 {
        // Allocate objects
        for _ in 0..10 {
            tracker.track_allocation(100);
        }
        
        // Simulate GC by deallocating most objects
        for _ in 0..9 {
            tracker.track_deallocation(100);
        }
        
        // One object remains allocated each cycle
    }
    
    let stats = tracker.get_stats();
    println!("GC simulation stats: {:?}", stats);
    
    // Should have reasonable memory characteristics
    assert_eq!(stats.allocations, 100); // 10 cycles * 10 allocations
    assert_eq!(stats.deallocations, 90); // 10 cycles * 9 deallocations
    assert_eq!(stats.leaked_objects, 10); // 1 per cycle
    assert_eq!(stats.current_memory, 1000); // 10 objects * 100 bytes
}

#[test]
fn test_concurrent_memory_operations() {
    use std::sync::Arc;
    use std::thread;
    
    let tracker = Arc::new(MemoryTracker::new());
    let mut handles = Vec::new();
    
    // Spawn threads that perform memory operations
    for thread_id in 0..4 {
        let tracker_clone = Arc::clone(&tracker);
        
        let handle = thread::spawn(move || {
            for i in 0..25 {
                // Each thread allocates and deallocates
                tracker_clone.track_allocation(thread_id * 100 + i);
                
                if i % 2 == 0 {
                    tracker_clone.track_deallocation(thread_id * 100 + i);
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    let stats = tracker.get_stats();
    println!("Concurrent memory stats: {:?}", stats);
    
    // Verify thread safety and reasonable behavior
    assert_eq!(stats.allocations, 100); // 4 threads * 25 allocations
    assert!(stats.deallocations > 0); // Some deallocations should have occurred
    assert!(stats.leaked_objects <= 100); // Should not exceed total allocations
}

#[test]
fn test_memory_pressure_handling() {
    let (mut interpreter, tracker) = create_tracked_interpreter();
    
    // Create memory pressure
    let _pressure = create_memory_pressure(10); // 10 MB
    
    let (result, stats) = measure_memory(&tracker, || {
        // Try to perform operations under memory pressure
        let _ = interpreter.eval("(define pressure-test '(1 2 3 4 5))");
        let _ = interpreter.eval("(length pressure-test)");
    });
    
    println!("Memory stats under pressure: {:?}", stats);
    
    // Should handle memory pressure gracefully
    assert!(stats.leaked_objects < 10, "Should not leak under memory pressure");
}

#[test]
fn test_cleanup_verification() {
    let (mut interpreter, tracker) = create_tracked_interpreter();
    
    let cleanup_successful = test_cleanup(|| {
        // Perform various operations
        let _ = interpreter.eval("(define cleanup-test (lambda (x) (* x x)))");
        let _ = interpreter.eval("(cleanup-test 42)");
        let _ = interpreter.eval("'(a b c d e f g h i j)");
    });
    
    assert!(cleanup_successful, "Cleanup should be properly performed");
    
    let final_stats = tracker.get_stats();
    println!("Final cleanup stats: {:?}", final_stats);
}

#[test]
fn test_peak_memory_tracking() {
    let tracker = MemoryTracker::new();
    
    // Gradually increase memory usage
    tracker.track_allocation(100);  // Peak: 100
    tracker.track_allocation(200);  // Peak: 300
    tracker.track_allocation(150);  // Peak: 450
    
    assert_eq!(tracker.get_stats().peak_memory, 450);
    
    // Deallocate some memory
    tracker.track_deallocation(150);  // Current: 300, Peak: still 450
    tracker.track_deallocation(100);  // Current: 200, Peak: still 450
    
    let stats = tracker.get_stats();
    assert_eq!(stats.current_memory, 200);
    assert_eq!(stats.peak_memory, 450);
    
    // Add more memory (but not exceeding peak)
    tracker.track_allocation(100);  // Current: 300, Peak: still 450
    
    let final_stats = tracker.get_stats();
    assert_eq!(final_stats.current_memory, 300);
    assert_eq!(final_stats.peak_memory, 450);
}
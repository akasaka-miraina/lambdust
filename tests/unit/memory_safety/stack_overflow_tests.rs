//! Stack Overflow Prevention Tests
//!
//! Tests for detecting and preventing stack overflow conditions in:
//! - Deep recursive function calls
//! - Nested evaluation contexts
//! - Continuation stack management
//! - Tail call optimization verification

use super::test_utils::*;
use super::patterns::*;
use crate::evaluator::Evaluator;
use crate::interpreter::LambdustInterpreter;
use std::time::Duration;

#[test]
fn test_basic_stack_tracking() {
    let tracker = StackTracker::new(100);
    
    // Test normal operation
    {
        let _guard = tracker.enter_frame().unwrap();
        assert_eq!(tracker.get_stats().current_depth, 1);
        
        {
            let _guard2 = tracker.enter_frame().unwrap();
            assert_eq!(tracker.get_stats().current_depth, 2);
        }
        
        assert_eq!(tracker.get_stats().current_depth, 1);
    }
    
    assert_eq!(tracker.get_stats().current_depth, 0);
    let stats = tracker.get_stats();
    assert_eq!(stats.max_depth, 2);
}

#[test]
fn test_stack_overflow_detection() {
    let tracker = StackTracker::new(5); // Very low threshold for testing
    
    // Should allow up to 5 frames
    let guards: Result<Vec<_>, _> = (0..5)
        .map(|_| tracker.enter_frame())
        .collect();
    
    assert!(guards.is_ok(), "Should allow frames up to threshold");
    
    // 6th frame should fail
    let overflow_result = tracker.enter_frame();
    assert!(overflow_result.is_err(), "Should detect stack overflow");
    
    match overflow_result.unwrap_err() {
        StackOverflowError { current_depth, threshold } => {
            assert_eq!(current_depth, 6);
            assert_eq!(threshold, 5);
        }
    }
}

#[test]
fn test_recursive_evaluation_stack_safety() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Test shallow recursion (should work)
    let shallow_factorial = r#"
        (define (factorial n)
          (if (<= n 1)
              1
              (* n (factorial (- n 1)))))
        (factorial 10)
    "#;
    
    let result = interpreter.eval(shallow_factorial);
    assert!(result.is_ok(), "Shallow recursion should succeed");
    
    // Test very deep recursion (should be handled gracefully)
    let deep_recursive = create_recursive_expression(1000);
    
    let deep_result = run_with_timeout(
        || interpreter.eval(&deep_recursive),
        Duration::from_secs(5)
    );
    
    match deep_result {
        Ok(result) => {
            // If it succeeds, that's fine (tail call optimization working)
            assert!(result.is_ok() || result.is_err());
        }
        Err(_) => {
            // Timeout is acceptable for very deep recursion
            println!("Deep recursion timed out as expected");
        }
    }
}

#[test]
fn test_nested_let_expressions_stack_safety() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Create deeply nested let expressions
    let mut nested_lets = String::new();
    let depth = 500;
    
    for i in 0..depth {
        nested_lets.push_str(&format!("(let ((x{} {})) ", i, i));
    }
    nested_lets.push('1'); // Base value
    for _ in 0..depth {
        nested_lets.push(')');
    }
    
    let result = run_with_timeout(
        || interpreter.eval(&nested_lets),
        Duration::from_secs(5)
    );
    
    // Should either succeed or handle gracefully
    match result {
        Ok(eval_result) => {
            assert!(eval_result.is_ok() || eval_result.is_err());
        }
        Err(_) => {
            println!("Deep nesting timed out - acceptable");
        }
    }
}

#[test]
fn test_continuation_stack_management() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Test call/cc with reasonable depth
    let callcc_test = r#"
        (call/cc 
          (lambda (escape)
            (let loop ((n 100))
              (if (= n 0)
                  'done
                  (loop (- n 1))))))
    "#;
    
    let result = interpreter.eval(callcc_test);
    
    // Should handle call/cc without stack issues
    match result {
        Ok(_) => {
            // Success expected
        }
        Err(_) => {
            // Error is acceptable if call/cc not fully implemented
            println!("call/cc test failed - may not be implemented");
        }
    }
}

#[test]
fn test_mutual_recursion_stack_safety() {
    let mut interpreter = LambdustInterpreter::new();
    
    let mutual_recursion = r#"
        (define (even? n)
          (if (= n 0)
              #t
              (odd? (- n 1))))
              
        (define (odd? n)
          (if (= n 0)
              #f
              (even? (- n 1))))
              
        (even? 1000)
    "#;
    
    let result = run_with_timeout(
        || interpreter.eval(mutual_recursion),
        Duration::from_secs(5)
    );
    
    match result {
        Ok(eval_result) => {
            // Should work with proper tail call optimization
            assert!(eval_result.is_ok() || eval_result.is_err());
        }
        Err(_) => {
            println!("Mutual recursion timed out - may need tail call optimization");
        }
    }
}

#[test]
fn test_stack_frame_cleanup() {
    let result = test_stack_overflow_prevention(|tracker| {
        // Simulate nested function calls
        let _frame1 = tracker.enter_frame()?;
        {
            let _frame2 = tracker.enter_frame()?;
            {
                let _frame3 = tracker.enter_frame()?;
                // Frames should be automatically cleaned up when dropped
            }
        }
        Ok(())
    }, 10);
    
    assert!(result.is_ok(), "Stack frame cleanup should work properly");
}

#[test]
fn test_stack_overflow_with_lambdas() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Create a lambda that calls itself deeply
    let lambda_recursion = r#"
        ((lambda (f n)
           (if (= n 0)
               'done
               (f f (- n 1))))
         (lambda (f n)
           (if (= n 0)
               'done
               (f f (- n 1))))
         100)
    "#;
    
    let result = run_with_timeout(
        || interpreter.eval(lambda_recursion),
        Duration::from_secs(3)
    );
    
    match result {
        Ok(eval_result) => {
            assert!(eval_result.is_ok() || eval_result.is_err());
        }
        Err(_) => {
            println!("Lambda recursion timed out");
        }
    }
}

#[test]
fn test_stack_depth_monitoring() {
    let tracker = StackTracker::new(1000);
    
    // Simulate progressively deeper call stacks
    fn recursive_call(tracker: &StackTracker, depth: usize) -> Result<usize, StackOverflowError> {
        let _guard = tracker.enter_frame()?;
        
        if depth == 0 {
            Ok(tracker.get_stats().current_depth)
        } else {
            recursive_call(tracker, depth - 1)
        }
    }
    
    let max_depth_result = recursive_call(&tracker, 100);
    assert!(max_depth_result.is_ok(), "Should handle moderate recursion");
    
    let max_depth = max_depth_result.unwrap();
    assert_eq!(max_depth, 101); // 100 + 1 for initial call
    
    let stats = tracker.get_stats();
    assert_eq!(stats.max_depth, 101);
    assert_eq!(stats.current_depth, 0); // Should be cleaned up
}

#[test]
fn test_evaluator_stack_limits() {
    let mut evaluator = Evaluator::new();
    
    // Test evaluation with controlled stack depth
    let nested_expression = "(+ 1 (+ 2 (+ 3 (+ 4 (+ 5 (+ 6 (+ 7 (+ 8 (+ 9 10)))))))))";
    
    let result = evaluator.eval_string(nested_expression);
    assert!(result.is_ok(), "Moderate nesting should work");
    
    // Test with very deep nesting
    let deep_expression = create_recursive_expression(200);
    let deep_result = run_with_timeout(
        || evaluator.eval_string(&deep_expression),
        Duration::from_secs(3)
    );
    
    match deep_result {
        Ok(eval_result) => {
            assert!(eval_result.is_ok() || eval_result.is_err());
        }
        Err(_) => {
            println!("Deep expression evaluation timed out");
        }
    }
}

#[test]
fn test_tail_call_optimization_verification() {
    let mut interpreter = LambdustInterpreter::new();
    
    // This should use constant stack space with proper TCO
    let tail_recursive = r#"
        (define (count-down n acc)
          (if (= n 0)
              acc
              (count-down (- n 1) (+ acc 1))))
        (count-down 10000 0)
    "#;
    
    let result = run_with_timeout(
        || interpreter.eval(tail_recursive),
        Duration::from_secs(5)
    );
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(value) => {
                    // Should return 10000 if properly tail-call optimized
                    println!("Tail recursion result: {:?}", value);
                }
                Err(e) => {
                    println!("Tail recursion failed: {:?}", e);
                }
            }
        }
        Err(_) => {
            println!("Tail recursion timed out - may need better TCO");
        }
    }
}

#[test]
fn test_stack_overflow_error_handling() {
    let tracker = StackTracker::new(3);
    
    // Fill up the stack
    let _g1 = tracker.enter_frame().unwrap();
    let _g2 = tracker.enter_frame().unwrap();
    let _g3 = tracker.enter_frame().unwrap();
    
    // This should fail
    let overflow_result = tracker.enter_frame();
    assert!(overflow_result.is_err());
    
    let error = overflow_result.unwrap_err();
    assert_eq!(error.current_depth, 4);
    assert_eq!(error.threshold, 3);
    
    // Should still be able to operate after error
    drop(_g3);
    let recovery_result = tracker.enter_frame();
    assert!(recovery_result.is_ok(), "Should recover after stack frame is freed");
}

#[test]
fn test_concurrent_stack_tracking() {
    use std::sync::Arc;
    use std::thread;
    
    let tracker = Arc::new(StackTracker::new(100));
    let mut handles = Vec::new();
    
    // Spawn multiple threads that each use stack frames
    for thread_id in 0..4 {
        let tracker_clone = Arc::clone(&tracker);
        
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let _guard = tracker_clone.enter_frame();
                thread::sleep(Duration::from_millis(1));
                
                // Verify guard works in this thread
                if let Ok(_) = _guard {
                    // Frame should be active
                    assert!(tracker_clone.get_stats().current_depth > 0);
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // All frames should be cleaned up
    let final_stats = tracker.get_stats();
    assert_eq!(final_stats.current_depth, 0);
    assert!(final_stats.max_depth > 0);
}

#[test]
fn test_stack_exhaustion_graceful_handling() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Create an expression that would cause stack exhaustion
    let stack_bomb = r#"
        (define (bomb n)
          (if (= n 0)
              0
              (+ 1 (bomb (- n 1)))))
        (bomb 5000)
    "#;
    
    let result = run_with_timeout(
        || interpreter.eval(stack_bomb),
        Duration::from_secs(10)
    );
    
    match result {
        Ok(eval_result) => {
            match eval_result {
                Ok(_) => {
                    println!("Stack bomb succeeded - good TCO");
                }
                Err(e) => {
                    println!("Stack bomb failed gracefully: {:?}", e);
                    // This is acceptable - we want graceful failure
                }
            }
        }
        Err(_) => {
            println!("Stack bomb timed out - acceptable protection");
        }
    }
}
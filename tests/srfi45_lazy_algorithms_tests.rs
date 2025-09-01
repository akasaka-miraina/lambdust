//! Comprehensive tests for SRFI-45 (Primitives for Expressing Iterative Lazy Algorithms)
//!
//! This test suite verifies the correct implementation of SRFI-45 lazy evaluation
//! primitives including:
//! - `lazy` - creates lazy promises that support iterative algorithms
//! - `eager` - creates eager values that can be forced immediately  
//! - Enhanced `force` - works with both R5RS delay and SRFI-45 promises
//! - Iterative evaluation without stack overflow

use lambdust::*;

#[test]
fn test_srfi45_basic_lazy() {
    let mut lambdust = Lambdust::new();
    
    // Test basic lazy creation and forcing
    let result = lambdust.eval(r#"
        (define lazy-value (lazy (lambda () 42)))
        (force lazy-value)
    "#, Some("srfi45_basic_lazy"));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::integer(42));
}

#[test]
fn test_srfi45_basic_eager() {
    let mut lambdust = Lambdust::new();
    
    // Test basic eager creation and forcing  
    let result = lambdust.eval(r#"
        (define eager-value (eager 42))
        (force eager-value)
    "#, Some("srfi45_basic_eager"));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::integer(42));
}

#[test]
fn test_srfi45_lazy_chain() {
    let mut lambdust = Lambdust::new();
    
    // Test lazy promise chains (iterative lazy algorithms)
    let result = lambdust.eval(r#"
        (define lazy-chain
          (lazy (lambda ()
            (lazy (lambda ()
              (eager 42))))))
        (force lazy-chain)
    "#, Some("srfi45_lazy_chain"));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::integer(42));
}

#[test]
fn test_srfi45_iterative_fibonacci() {
    let mut lambdust = Lambdust::new();
    
    // Test iterative fibonacci using lazy evaluation
    // This should not cause stack overflow even for large n
    let result = lambdust.eval(r#"
        (define (fib n)
          (if (<= n 1)
              (eager n)
              (lazy (lambda ()
                (+ (force (fib (- n 1)))
                   (force (fib (- n 2))))))))
        
        (force (fib 10))
    "#, Some("srfi45_iterative_fibonacci"));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::integer(55)); // 10th fibonacci number
}

#[test]  
fn test_srfi45_lazy_vs_delay() {
    let mut lambdust = Lambdust::new();
    
    // Test that lazy and delay work differently for promise chains
    let result = lambdust.eval(r#"
        ;; With delay, this creates nested promises
        (define delayed-chain
          (delay (delay (delay 42))))
        
        ;; With lazy, this supports iterative evaluation  
        (define lazy-chain
          (lazy (lambda ()
            (lazy (lambda ()
              (lazy (lambda () 42)))))))
        
        (list (force (force (force delayed-chain)))
              (force lazy-chain))
    "#, Some("srfi45_lazy_vs_delay"));
    
    assert!(result.is_ok());
    if let Some(values) = result.unwrap().as_list() {
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], Value::integer(42));
        assert_eq!(values[1], Value::integer(42)); 
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_srfi45_eager_immediate_evaluation() {
    let mut lambdust = Lambdust::new();
    
    // Test that eager evaluates immediately, unlike delay/lazy
    let result = lambdust.eval(r#"
        (define side-effect-count 0)
        (define (increment!)
          (set! side-effect-count (+ side-effect-count 1))
          side-effect-count)
        
        ;; eager should evaluate immediately
        (define eager-promise (eager (increment!)))
        
        ;; Should have been evaluated once already
        (list side-effect-count (force eager-promise))
    "#, Some("srfi45_eager_immediate"));
    
    assert!(result.is_ok());
    if let Some(values) = result.unwrap().as_list() {
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], Value::integer(1)); // Side effect happened
        assert_eq!(values[1], Value::integer(1)); // Same result when forced
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_srfi45_memoization() {
    let mut lambdust = Lambdust::new();
    
    // Test that lazy promises are memoized properly  
    let result = lambdust.eval(r#"
        (define call-count 0)
        (define (expensive-computation)
          (set! call-count (+ call-count 1))
          42)
        
        (define lazy-promise (lazy (lambda () (expensive-computation))))
        
        ;; Force multiple times - should only compute once
        (list (force lazy-promise)
              (force lazy-promise) 
              (force lazy-promise)
              call-count)
    "#, Some("srfi45_memoization"));
    
    assert!(result.is_ok());
    if let Some(values) = result.unwrap().as_list() {
        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Value::integer(42));
        assert_eq!(values[1], Value::integer(42));
        assert_eq!(values[2], Value::integer(42));
        assert_eq!(values[3], Value::integer(1)); // Called only once
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_srfi45_r5rs_compatibility() {
    let mut lambdust = Lambdust::new();
    
    // Test that existing R5RS delay/force code still works
    let result = lambdust.eval(r#"
        ;; R5RS style delay/force
        (define delayed-value (delay (+ 1 2 3)))
        (define result1 (force delayed-value))
        
        ;; Should work with SRFI-45 force as well
        (define result2 (force delayed-value))
        
        (list result1 result2)
    "#, Some("srfi45_r5rs_compatibility"));
    
    assert!(result.is_ok());
    if let Some(values) = result.unwrap().as_list() {
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], Value::integer(6));
        assert_eq!(values[1], Value::integer(6));
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_srfi45_deep_chain_no_stack_overflow() {
    let mut lambdust = Lambdust::new();
    
    // Test deep promise chains don't cause stack overflow
    // This creates a chain of 100 lazy promises
    let result = lambdust.eval(r#"
        (define (make-lazy-chain n)
          (if (= n 0)
              (eager 42)
              (lazy (lambda () (make-lazy-chain (- n 1))))))
        
        (force (make-lazy-chain 100))
    "#, Some("srfi45_deep_chain"));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::integer(42));
}

#[test]
fn test_srfi45_mixed_delay_lazy() {
    let mut lambdust = Lambdust::new();
    
    // Test mixing delay and lazy in the same computation
    let result = lambdust.eval(r#"
        (define mixed-computation
          (delay
            (lazy (lambda ()
              (delay
                (eager (+ 20 22)))))))
        
        (force (force (force mixed-computation)))
    "#, Some("srfi45_mixed"));
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::integer(42));
}

#[test]
fn test_srfi45_promise_predicate() {
    let mut lambdust = Lambdust::new();
    
    // Test that promise? works with SRFI-45 promises
    let result = lambdust.eval(r#"
        (define lazy-p (lazy (lambda () 42)))
        (define eager-p (eager 42))
        (define delay-p (delay 42))
        
        (list (promise? lazy-p)
              (promise? eager-p) 
              (promise? delay-p)
              (promise? 42))
    "#, Some("srfi45_promise_predicate"));
    
    assert!(result.is_ok());
    if let Some(values) = result.unwrap().as_list() {
        assert_eq!(values.len(), 4);
        assert_eq!(values[0], Value::boolean(true));
        assert_eq!(values[1], Value::boolean(true));
        assert_eq!(values[2], Value::boolean(true));
        assert_eq!(values[3], Value::boolean(false));
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_srfi45_syntax_forms() {
    let mut lambdust = Lambdust::new();
    
    // Test that (lazy expr) and (eager expr) syntax forms work
    let result = lambdust.eval(r#"
        (define lazy-syntax (lazy (+ 20 22)))
        (define eager-syntax (eager (* 6 7)))
        
        (list (force lazy-syntax) (force eager-syntax))
    "#, Some("srfi45_syntax_forms"));
    
    assert!(result.is_ok());
    if let Some(values) = result.unwrap().as_list() {
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], Value::integer(42));
        assert_eq!(values[1], Value::integer(42));
    } else {
        panic!("Expected list result");
    }
}
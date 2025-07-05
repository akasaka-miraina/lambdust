//! Test call/cc deep non-local exit functionality

use lambdust::interpreter::LambdustInterpreter;

#[test]
fn test_call_cc_simple_escape() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Basic call/cc test
    let result = interpreter.eval_string("(call/cc (lambda (k) (k 42)))").unwrap();
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_call_cc_deep_nested_escape() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Deep nested escape - should return 10, not 11 (1 + 10)
    let result = interpreter.eval_string("(+ 1 (* 2 (call/cc (lambda (k) (k 10)))))").unwrap();
    assert_eq!(result.to_string(), "10");
}

#[test]
fn test_call_cc_very_deep_nested_escape() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Very deep nested escape - should return 5, not 16 (1 + 2 * (3 + 5))
    let result = interpreter.eval_string("(+ 1 (* 2 (+ 3 (call/cc (lambda (k) (k 5))))))").unwrap();
    assert_eq!(result.to_string(), "5");
}

#[test]
fn test_call_cc_complex_arithmetic_escape() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Complex arithmetic with call/cc escape
    let result = interpreter.eval_string("(* 3 (+ 4 (* 2 (call/cc (lambda (k) (k 7))))))").unwrap();
    assert_eq!(result.to_string(), "7");
}

#[test]
fn test_call_cc_nested_function_calls() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Define helper functions
    interpreter.eval_string("(define (double x) (* 2 x))").unwrap();
    interpreter.eval_string("(define (add-one x) (+ 1 x))").unwrap();
    
    // Nested function calls with call/cc escape
    let result = interpreter.eval_string("(double (add-one (call/cc (lambda (k) (k 20)))))").unwrap();
    assert_eq!(result.to_string(), "20");
}

#[test]
fn test_call_cc_with_begin() {
    let mut interpreter = LambdustInterpreter::new();
    
    // call/cc within begin block
    let result = interpreter.eval_string("(begin (+ 1 2) (* 3 (call/cc (lambda (k) (k 15)))))").unwrap();
    assert_eq!(result.to_string(), "15");
}

#[test]
//#[ignore = "Continuation reuse requires advanced implementation - future enhancement"]
fn test_call_cc_continuation_reuse() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Store continuation and reuse it
    interpreter.eval_string("(define saved-cont #f)").unwrap();
    let result1 = interpreter.eval_string("(+ 1 (call/cc (lambda (k) (set! saved-cont k) 10)))").unwrap();
    assert_eq!(result1.to_string(), "11");
    
    // Reuse the saved continuation  
    // Note: Continuation reuse now implemented with context preservation
    // The saved continuation preserves the computation context (+ 1 ...)
    // So (saved-cont 20) should return 21 (1 + 20)
    let result2 = interpreter.eval_string("(saved-cont 20)").unwrap();
    assert_eq!(result2.to_string(), "21"); // Reuse semantics: preserves context
}

#[test]
fn test_call_cc_nested_captures() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Nested call/cc captures
    let result = interpreter.eval_string(r#"
        (call/cc (lambda (outer)
          (+ 1 (call/cc (lambda (inner)
            (outer 42))))))
    "#).unwrap();
    assert_eq!(result.to_string(), "42");
}

#[test]
fn test_call_cc_with_if_conditions() {
    let mut interpreter = LambdustInterpreter::new();
    
    // call/cc with conditional expressions
    let result = interpreter.eval_string(r#"
        (if #t
            (+ 5 (call/cc (lambda (k) (k 100))))
            999)
    "#).unwrap();
    assert_eq!(result.to_string(), "100");
}

#[test]
fn test_call_cc_multiple_arguments() {
    let mut interpreter = LambdustInterpreter::new();
    
    // call/cc with multiple value return
    let result = interpreter.eval_string(r#"
        (+ (call/cc (lambda (k) (k 10 20 30))) 1)
    "#).unwrap();
    // Should return 10 (first value), ignoring the computation
    assert_eq!(result.to_string(), "10");
}
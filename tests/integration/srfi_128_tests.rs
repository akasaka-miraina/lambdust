//! Integration tests for SRFI 128: Comparators

use lambdust::LambdustInterpreter;

#[test]
fn test_srfi_128_basic_usage() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Import SRFI 128
    interpreter.eval("(import (srfi 128))").unwrap();
    
    // Test comparator? predicate
    let result = interpreter.eval("(comparator? real-comparator)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    let result = interpreter.eval("(comparator? 42)").unwrap();
    assert_eq!(result.to_string(), "#f");
}

#[test]
fn test_srfi_128_ordered_predicate() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 128))").unwrap();
    
    // Test comparator-ordered?
    let result = interpreter.eval("(comparator-ordered? real-comparator)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    let result = interpreter.eval("(comparator-ordered? default-comparator)").unwrap();
    assert_eq!(result.to_string(), "#f");
}

#[test]
fn test_srfi_128_hashable_predicate() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 128))").unwrap();
    
    // Test comparator-hashable?
    let result = interpreter.eval("(comparator-hashable? string-comparator)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    let result = interpreter.eval("(comparator-hashable? default-comparator)").unwrap();
    assert_eq!(result.to_string(), "#f");
}

#[test]
fn test_srfi_128_equality_operations() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 128))").unwrap();
    
    // Test =? with numbers
    let result = interpreter.eval("(=? real-comparator 5 5)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    let result = interpreter.eval("(=? real-comparator 5 10)").unwrap();
    assert_eq!(result.to_string(), "#f");
    
    let result = interpreter.eval("(=? real-comparator 3 3 3)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    let result = interpreter.eval("(=? real-comparator 3 3 4)").unwrap();
    assert_eq!(result.to_string(), "#f");
}

#[test]
fn test_srfi_128_ordering_operations() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 128))").unwrap();
    
    // Test <? with numbers
    let result = interpreter.eval("(<? real-comparator 1 5 10)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    let result = interpreter.eval("(<? real-comparator 5 1 10)").unwrap();
    assert_eq!(result.to_string(), "#f");
    
    let result = interpreter.eval("(<? real-comparator 1 5)").unwrap();
    assert_eq!(result.to_string(), "#t");
}

#[test]
fn test_srfi_128_string_operations() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 128))").unwrap();
    
    // Test string comparisons
    let result = interpreter.eval(r#"(=? string-comparator "apple" "apple")"#).unwrap();
    assert_eq!(result.to_string(), "#t");
    
    let result = interpreter.eval(r#"(=? string-comparator "apple" "banana")"#).unwrap();
    assert_eq!(result.to_string(), "#f");
    
    let result = interpreter.eval(r#"(<? string-comparator "apple" "banana" "cherry")"#).unwrap();
    assert_eq!(result.to_string(), "#t");
}

#[test]
fn test_srfi_128_boolean_comparator() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 128))").unwrap();
    
    // Test boolean comparisons
    let result = interpreter.eval("(=? boolean-comparator #t #t)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    let result = interpreter.eval("(=? boolean-comparator #t #f)").unwrap();
    assert_eq!(result.to_string(), "#f");
    
    let result = interpreter.eval("(<? boolean-comparator #f #t)").unwrap();
    assert_eq!(result.to_string(), "#t");
}

#[test]
fn test_srfi_128_symbol_comparator() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 128))").unwrap();
    
    // Test symbol comparisons
    let result = interpreter.eval("(=? symbol-comparator 'apple 'apple)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    let result = interpreter.eval("(=? symbol-comparator 'apple 'banana)").unwrap();
    assert_eq!(result.to_string(), "#f");
    
    let result = interpreter.eval("(<? symbol-comparator 'apple 'banana)").unwrap();
    assert_eq!(result.to_string(), "#t");
}

#[test]
fn test_srfi_128_make_comparator() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 128))").unwrap();
    
    // Create a custom comparator (basic test)
    let result = interpreter.eval("(define dummy-proc (lambda (x) #t))").unwrap();
    assert!(result.to_string().contains("undefined") || result.to_string().is_empty());
    
    let result = interpreter.eval("(define my-comp (make-comparator dummy-proc dummy-proc))").unwrap();
    assert!(result.to_string().contains("undefined") || result.to_string().is_empty());
    
    let result = interpreter.eval("(comparator? my-comp)").unwrap();
    assert_eq!(result.to_string(), "#t");
}

#[test]
fn test_srfi_128_mixed_types_error_handling() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 128))").unwrap();
    
    // Test that comparing incompatible types with appropriate comparators fails
    let result = interpreter.eval(r#"(=? string-comparator "hello" 42)"#);
    assert!(result.is_err() || result.unwrap().to_string() == "#f");
}

#[test]
fn test_srfi_128_all_standard_comparators() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 128))").unwrap();
    
    // Verify all standard comparators exist and are comparators
    let comparators = [
        "default-comparator",
        "boolean-comparator", 
        "real-comparator",
        "string-comparator",
        "symbol-comparator"
    ];
    
    for comp in &comparators {
        let code = format!("(comparator? {})", comp);
        let result = interpreter.eval(&code).unwrap();
        assert_eq!(result.to_string(), "#t", "Failed for {}", comp);
    }
}
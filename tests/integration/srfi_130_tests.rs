//! Integration tests for SRFI 130: Cursor-based String Library

use lambdust::LambdustInterpreter;

#[test]
fn test_srfi_130_basic_usage() {
    let mut interpreter = LambdustInterpreter::new();
    
    // Import SRFI 130
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Test string-cursor? predicate
    let result = interpreter.eval(r#"(define c (string-cursor-start "hello"))"#).unwrap();
    assert!(result.to_string().contains("undefined") || result.to_string().is_empty());
    
    let result = interpreter.eval("(string-cursor? c)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    let result = interpreter.eval(r#"(string-cursor? "hello")"#).unwrap();
    assert_eq!(result.to_string(), "#f");
}

#[test]
fn test_srfi_130_cursor_navigation() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Create cursor and navigate
    interpreter.eval(r#"(define start-c (string-cursor-start "hello"))"#).unwrap();
    interpreter.eval("(define next-c (string-cursor-next start-c))").unwrap();
    interpreter.eval("(define prev-c (string-cursor-prev next-c))").unwrap();
    
    // Test cursor equality
    let result = interpreter.eval("(string-cursor=? start-c prev-c)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    // Test cursor ordering
    let result = interpreter.eval("(string-cursor<? start-c next-c)").unwrap();
    assert_eq!(result.to_string(), "#t");
}

#[test]
fn test_srfi_130_cursor_reference() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Test string-cursor-ref
    interpreter.eval(r#"(define c (string-cursor-start "hello"))"#).unwrap();
    let result = interpreter.eval("(string-cursor-ref c)").unwrap();
    assert_eq!(result.to_string(), "#\\h");
    
    // Move to next position and test
    interpreter.eval("(define c2 (string-cursor-next c))").unwrap();
    let result = interpreter.eval("(string-cursor-ref c2)").unwrap();
    assert_eq!(result.to_string(), "#\\e");
}

#[test]
fn test_srfi_130_substring_operations() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Test substring/cursors
    interpreter.eval(r#"(define s "hello world")"#).unwrap();
    interpreter.eval("(define start-c (string-cursor-start s))").unwrap();
    interpreter.eval("(define end-c (string-cursor-end s))").unwrap();
    
    // Create cursor in middle
    interpreter.eval("(define mid-c (string-cursor-next (string-cursor-next start-c)))").unwrap();
    
    let result = interpreter.eval("(substring/cursors start-c mid-c)").unwrap();
    assert_eq!(result.to_string(), r#""he""#);
    
    let result = interpreter.eval("(substring/cursors mid-c end-c)").unwrap();
    assert_eq!(result.to_string(), r#""llo world""#);
}

#[test]
fn test_srfi_130_string_operations() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Test string-take-cursor
    let result = interpreter.eval(r#"(string-take-cursor "hello" 3)"#).unwrap();
    assert_eq!(result.to_string(), r#""hel""#);
    
    // Test string-drop-cursor
    let result = interpreter.eval(r#"(string-drop-cursor "hello" 2)"#).unwrap();
    assert_eq!(result.to_string(), r#""llo""#);
}

#[test]
fn test_srfi_130_string_searching() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Test string-index-cursor
    interpreter.eval(r#"(define c (string-index-cursor "hello" #\l))"#).unwrap();
    let result = interpreter.eval("(string-cursor? c)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    // Character should be at the found position
    let result = interpreter.eval("(string-cursor-ref c)").unwrap();
    assert_eq!(result.to_string(), "#\\l");
    
    // Test character not found
    let result = interpreter.eval(r#"(string-index-cursor "hello" #\x)"#).unwrap();
    assert_eq!(result.to_string(), "#f");
}

#[test]
fn test_srfi_130_string_contains() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Test string-contains-cursor
    interpreter.eval(r#"(define c (string-contains-cursor "hello world" "world"))"#).unwrap();
    let result = interpreter.eval("(string-cursor? c)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    // Get substring from cursor to end
    interpreter.eval(r#"(define s "hello world")"#).unwrap();
    interpreter.eval("(define end-c (string-cursor-end s))").unwrap();
    let result = interpreter.eval("(substring/cursors c end-c)").unwrap();
    assert_eq!(result.to_string(), r#""world""#);
    
    // Test substring not found
    let result = interpreter.eval(r#"(string-contains-cursor "hello" "xyz")"#).unwrap();
    assert_eq!(result.to_string(), "#f");
}

#[test]
fn test_srfi_130_unicode_support() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Test with Unicode string
    interpreter.eval(r#"(define unicode-c (string-cursor-start "こんにちは"))"#).unwrap();
    let result = interpreter.eval("(string-cursor-ref unicode-c)").unwrap();
    assert_eq!(result.to_string(), "#\\こ");
    
    // Navigate to next character
    interpreter.eval("(define next-unicode-c (string-cursor-next unicode-c))").unwrap();
    let result = interpreter.eval("(string-cursor-ref next-unicode-c)").unwrap();
    assert_eq!(result.to_string(), "#\\ん");
}

#[test]
fn test_srfi_130_cursor_bounds() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Test cursor start and end
    interpreter.eval(r#"(define s "hello")"#).unwrap();
    interpreter.eval("(define start-c (string-cursor-start s))").unwrap();
    interpreter.eval("(define end-c (string-cursor-end s))").unwrap();
    
    // Test that start < end
    let result = interpreter.eval("(string-cursor<? start-c end-c)").unwrap();
    assert_eq!(result.to_string(), "#t");
    
    // Test that start != end
    let result = interpreter.eval("(string-cursor=? start-c end-c)").unwrap();
    assert_eq!(result.to_string(), "#f");
}

#[test]
fn test_srfi_130_length_operations() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Test string-length/cursors
    interpreter.eval(r#"(define s "hello")"#).unwrap();
    interpreter.eval("(define start-c (string-cursor-start s))").unwrap();
    interpreter.eval("(define end-c (string-cursor-end s))").unwrap();
    
    let result = interpreter.eval("(string-length/cursors start-c end-c)").unwrap();
    assert_eq!(result.to_string(), "5");
    
    // Test partial length
    interpreter.eval("(define mid-c (string-cursor-next (string-cursor-next start-c)))").unwrap();
    let result = interpreter.eval("(string-length/cursors start-c mid-c)").unwrap();
    assert_eq!(result.to_string(), "2");
}

#[test]
fn test_srfi_130_error_handling() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Test type errors
    let result = interpreter.eval("(string-cursor? 42)");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().to_string(), "#f");
    
    // Test cursor mismatch errors (cursors from different strings)
    interpreter.eval(r#"(define c1 (string-cursor-start "hello"))"#).unwrap();
    interpreter.eval(r#"(define c2 (string-cursor-start "world"))"#).unwrap();
    
    let result = interpreter.eval("(string-cursor<? c1 c2)");
    assert!(result.is_err()); // Should fail because cursors are from different strings
}

#[test]
fn test_srfi_130_all_exports() {
    let mut interpreter = LambdustInterpreter::new();
    
    interpreter.eval("(import (srfi 130))").unwrap();
    
    // Test that all major procedures are available
    let procedures = [
        "string-cursor-start",
        "string-cursor-end",
        "string-cursor?",
        "string-cursor-next", 
        "string-cursor-prev",
        "string-cursor=?",
        "string-cursor<?",
        "string-cursor-ref",
        "substring/cursors",
        "string-take-cursor",
        "string-drop-cursor",
        "string-index-cursor",
        "string-contains-cursor",
        "string-length/cursors"
    ];
    
    for proc in &procedures {
        let code = format!(r#"(procedure? {})"#, proc);
        let result = interpreter.eval(&code).unwrap();
        assert_eq!(result.to_string(), "#t", "Procedure {} should be available", proc);
    }
}
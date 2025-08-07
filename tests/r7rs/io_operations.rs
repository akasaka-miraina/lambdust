//! R7RS I/O Operations Tests
//!
//! Tests for R7RS-small input/output operations including:
//! - Port operations (input-port?, output-port?, etc.)
//! - Reading operations (read, read-char, peek-char, etc.)
//! - Writing operations (write, display, newline, etc.)
//! - String ports (open-input-string, open-output-string, etc.)
//! - File operations (open-input-file, open-output-file, etc.)
//! - Port state management (close-port, port-open?, etc.)
//!
//! This module tests I/O operations required by R7RS-small.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value};

/// Run all I/O operations tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS I/O Operations tests...");
    
    if suite.skip_if_unimplemented("I/O operations") {
        println!("⚠ Skipping I/O operations tests (not implemented)");
        return Ok(());
    }
    
    test_port_predicates(suite)?;
    test_current_ports(suite)?;
    test_string_ports(suite)?;
    test_reading_operations(suite)?;
    test_writing_operations(suite)?;
    test_file_operations(suite)?;
    test_port_management(suite)?;
    test_io_edge_cases(suite)?;
    
    println!("✓ I/O operations tests passed");
    Ok(())
}

/// Test port type predicates
fn test_port_predicates(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Basic port predicates
    suite.assert_eval_true("(input-port? (current-input-port))")?;
    suite.assert_eval_true("(output-port? (current-output-port))")?;
    
    // Non-ports should return false
    suite.assert_eval_false("(input-port? 42)")?;
    suite.assert_eval_false("(output-port? \"hello\")")?;
    suite.assert_eval_false("(input-port? '())")?;
    suite.assert_eval_false("(output-port? #t)")?;
    
    Ok(())
}

/// Test current port operations
fn test_current_ports(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Current ports should exist and be of correct type
    suite.assert_eval_true("(input-port? (current-input-port))")?;
    suite.assert_eval_true("(output-port? (current-output-port))")?;
    suite.assert_eval_true("(output-port? (current-error-port))")?;
    
    // Port predicates on current ports
    suite.assert_eval_false("(output-port? (current-input-port))")?;
    suite.assert_eval_false("(input-port? (current-output-port))")?;
    
    Ok(())
}

/// Test string port operations
fn test_string_ports(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Input string ports
    suite.assert_eval_true("(input-port? (open-input-string \"hello\"))")?;
    
    // Reading from string ports
    let result = suite.eval("(call-with-input-string \"hello\" read-char)")?;
    match result {
        Value::Literal(Literal::Character('h')) => {},
        _ => return Err("Expected character 'h'".into()),
    }
    
    // Output string ports
    suite.assert_eval_true("(output-port? (open-output-string))")?;
    
    // Writing to string ports and getting contents
    suite.assert_eval_eq(r#"
        (call-with-output-string
          (lambda (port)
            (write-char #\h port)
            (write-char #\i port)))
    "#, Value::Literal(Literal::String("hi".to_string())))?;
    
    // Complex string port operations
    suite.assert_eval_eq(r#"
        (call-with-output-string
          (lambda (port)
            (display "hello" port)
            (newline port)
            (write 42 port)))
    "#, Value::Literal(Literal::String("hello\n42".to_string())))?;
    
    Ok(())
}

/// Test reading operations
fn test_reading_operations(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // read-char
    suite.assert_eval_eq(r#"
        (call-with-input-string "abc"
          (lambda (port)
            (read-char port)))
    "#, Value::Literal(Literal::Character('a')))?;
    
    // Multiple read-char calls
    suite.assert_eval_eq(r#"
        (call-with-input-string "abc"
          (lambda (port)
            (read-char port)
            (read-char port)))
    "#, Value::Literal(Literal::Character('b')))?;
    
    // peek-char (doesn't advance position)
    suite.assert_eval_eq(r#"
        (call-with-input-string "abc"
          (lambda (port)
            (peek-char port)
            (read-char port)))
    "#, Value::Literal(Literal::Character('a')))?;
    
    // char-ready? predicate
    suite.assert_eval_true(r#"
        (call-with-input-string "abc"
          (lambda (port)
            (char-ready? port)))
    "#)?;
    
    // read (full S-expressions)
    suite.assert_eval_eq(r#"
        (call-with-input-string "42"
          (lambda (port)
            (read port)))
    "#, Value::Literal(Literal::integer(42)))?;
    
    suite.assert_eval_eq(r#"
        (call-with-input-string "(a b c)"
          (lambda (port)
            (length (read port))))
    "#, Value::Literal(Literal::integer(3)))?;
    
    // EOF handling
    suite.assert_eval_true(r#"
        (call-with-input-string ""
          (lambda (port)
            (eof-object? (read-char port))))
    "#)?;
    
    Ok(())
}

/// Test writing operations
fn test_writing_operations(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // write-char
    suite.assert_eval_eq(r#"
        (call-with-output-string
          (lambda (port)
            (write-char #\x port)))
    "#, Value::Literal(Literal::String("x".to_string())))?;
    
    // write (with proper escaping)
    suite.assert_eval_eq(r#"
        (call-with-output-string
          (lambda (port)
            (write "hello" port)))
    "#, Value::Literal(Literal::String("\"hello\"".to_string())))?;
    
    suite.assert_eval_eq(r#"
        (call-with-output-string
          (lambda (port)
            (write 'symbol port)))
    "#, Value::Literal(Literal::String("symbol".to_string())))?;
    
    // display (without escaping)
    suite.assert_eval_eq(r#"
        (call-with-output-string
          (lambda (port)
            (display "hello" port)))
    "#, Value::Literal(Literal::String("hello".to_string())))?;
    
    // newline
    suite.assert_eval_eq(r#"
        (call-with-output-string
          (lambda (port)
            (display "line1" port)
            (newline port)
            (display "line2" port)))
    "#, Value::Literal(Literal::String("line1\nline2".to_string())))?;
    
    // Complex structures
    suite.assert_eval_eq(r#"
        (call-with-output-string
          (lambda (port)
            (write '(a b (c d) e) port)))
    "#, Value::Literal(Literal::String("(a b (c d) e)".to_string())))?;
    
    Ok(())
}

/// Test file operations (if supported)
fn test_file_operations(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("file operations") {
        return Ok(());
    }
    
    // This would require actual file system access
    // In a testing environment, we might skip these or use mock files
    
    // Basic file port creation (would need test files)
    // suite.assert_eval_true("(input-port? (open-input-file \"test.txt\"))")?;
    // suite.assert_eval_true("(output-port? (open-output-file \"output.txt\"))")?;
    
    // File existence checks
    // suite.assert_eval_true("(file-exists? \"existing-file.txt\")")?;
    // suite.assert_eval_false("(file-exists? \"non-existent-file.txt\")")?;
    
    println!("⚠ File operations tests skipped (require filesystem access)");
    Ok(())
}

/// Test port management operations
fn test_port_management(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Port state checking
    suite.assert_eval_true(r#"
        (let ((port (open-input-string "test")))
          (port-open? port))
    "#)?;
    
    // Closing ports
    suite.eval(r#"
        (define test-port (open-input-string "test"))
    "#)?;
    
    suite.assert_eval_true("(port-open? test-port)")?;
    suite.eval("(close-port test-port)")?;
    suite.assert_eval_false("(port-open? test-port)")?;
    
    // Operations on closed ports should error
    suite.assert_eval_error("(read-char test-port)")?;
    
    // with-input-from-file and with-output-to-file (if implemented)
    if !suite.skip_if_unimplemented("with-input/output") {
        // These would typically be used with actual files
        // In testing, we can use string ports
        
        suite.assert_eval_eq(r#"
            (with-input-from-string "42"
              (lambda () (read)))
        "#, Value::Literal(Literal::integer(42)))?;
        
        suite.assert_eval_eq(r#"
            (with-output-to-string
              (lambda () (display "test")))
        "#, Value::Literal(Literal::String("test".to_string())))?;
    }
    
    Ok(())
}

/// Test I/O edge cases and error conditions
fn test_io_edge_cases(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Reading from empty input
    suite.assert_eval_true(r#"
        (call-with-input-string ""
          (lambda (port)
            (eof-object? (read-char port))))
    "#)?;
    
    suite.assert_eval_true(r#"
        (call-with-input-string ""
          (lambda (port)
            (eof-object? (peek-char port))))
    "#)?;
    
    // Reading past end of input
    suite.eval("(define empty-port (open-input-string \"\"))")?;
    suite.assert_eval_true("(eof-object? (read-char empty-port))")?;
    suite.assert_eval_true("(eof-object? (read-char empty-port))")?;  // Still EOF
    
    // Writing to closed output port
    suite.eval("(define closed-output (open-output-string))")?;
    suite.eval("(close-port closed-output)")?;
    suite.assert_eval_error("(write-char #\\x closed-output)")?;
    
    // Type errors in I/O operations
    suite.assert_eval_error("(read-char 42)")?;        // Not a port
    suite.assert_eval_error("(write-char \"not-char\" (current-output-port))")?;  // Not a character
    suite.assert_eval_error("(display 'symbol 42)")?;  // Second arg not a port
    
    // Arity errors
    suite.assert_eval_error("(read-char)")?;           // Missing port argument
    suite.assert_eval_error("(write-char)")?;          // Missing arguments
    suite.assert_eval_error("(newline 'not-port)")?;   // Invalid port argument
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_io_operations_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("I/O operations tests should pass");
    }
    
    #[test]
    fn test_port_predicates_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_port_predicates(&mut suite).expect("Port predicate tests should pass");
    }
    
    #[test]
    fn test_string_ports_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_string_ports(&mut suite).expect("String port tests should pass");
    }
    
    #[test]
    fn test_reading_operations_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_reading_operations(&mut suite).expect("Reading operation tests should pass");
    }
    
    #[test]
    fn test_writing_operations_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_writing_operations(&mut suite).expect("Writing operation tests should pass");
    }
}
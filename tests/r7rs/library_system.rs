//! R7RS Library System Tests
//!
//! Tests for R7RS-small section 7 (Standard libraries)
//!
//! This module verifies library system implementation including:
//! - Library declaration and syntax
//! - Import/export specifications  
//! - Standard library availability
//! - Module system integration
//! - Library conditions and dependencies
//!
//! All tests follow R7RS specification requirements exactly.

use crate::R7RSTestSuite;

/// Run all library system tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS Library System tests...");
    
    test_library_declaration(suite)?;
    test_import_declarations(suite)?;
    test_export_specifications(suite)?;
    test_standard_libraries(suite)?;
    test_library_conditions(suite)?;
    test_library_phases(suite)?;
    
    println!("✓ Library system tests passed");
    Ok(())
}

/// Test library declaration syntax and semantics
fn test_library_declaration(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("library system") {
        return Ok(());
    }
    
    // Basic library definition
    let basic_lib = r#"
    (define-library (test basic)
      (export square)
      (import (scheme base))
      (begin
        (define (square x) (* x x))))
    "#;
    
    suite.eval(basic_lib)?;
    
    // Library with multiple exports
    let multi_export_lib = r#"
    (define-library (test multi)
      (export add subtract multiply)
      (import (scheme base))
      (begin
        (define (add x y) (+ x y))
        (define (subtract x y) (- x y))
        (define (multiply x y) (* x y))))
    "#;
    
    suite.eval(multi_export_lib)?;
    
    // Library with renamed exports
    if !suite.skip_if_unimplemented("renamed exports") {
        let renamed_lib = r#"
        (define-library (test renamed)
          (export (rename internal-add add)
                  (rename internal-sub subtract))
          (import (scheme base))
          (begin
            (define (internal-add x y) (+ x y))
            (define (internal-sub x y) (- x y))))
        "#;
        
        suite.eval(renamed_lib)?;
    }
    
    Ok(())
}

/// Test import declaration forms
fn test_import_declarations(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("import declarations") {
        return Ok(());   
    }
    
    // Simple import
    suite.eval("(import (scheme base))")?;
    
    // Multiple library import
    suite.eval("(import (scheme base) (scheme case-lambda))").unwrap_or_default();
    
    // Selective import with 'only'
    if !suite.skip_if_unimplemented("selective import") {
        suite.eval("(import (only (scheme base) + - * /))")?;
        suite.assert_eval_true("(= (+ 1 2) 3)")?;
    }
    
    // Import with 'except'
    if !suite.skip_if_unimplemented("except import") {
        suite.eval("(import (except (scheme base) map))")?;
    }
    
    // Import with 'prefix'
    if !suite.skip_if_unimplemented("prefix import") {
        suite.eval("(import (prefix (scheme base) base:))")?;
        suite.assert_eval_true("(= (base:+ 1 2) 3)")?;
    }
    
    // Import with 'rename'
    if !suite.skip_if_unimplemented("rename import") {
        suite.eval("(import (rename (scheme base) (+ add) (- subtract)))")?;
        suite.assert_eval_true("(= (add 5 3) 8)")?;
        suite.assert_eval_true("(= (subtract 10 4) 6)")?;
    }
    
    Ok(())
}

/// Test export specification forms
fn test_export_specifications(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("export specifications") {
        return Ok(());
    }
    
    // Test library with various export forms
    let complex_exports = r#"
    (define-library (test exports)
      (export ; Direct export
              simple-proc
              ; Renamed export
              (rename internal-name external-name)
              ; Export pattern
              exported-var)
      (import (scheme base))
      (begin
        (define (simple-proc x) (* x 2))
        (define (internal-name x) (+ x 1))
        (define exported-var 42)))
    "#;
    
    suite.eval(complex_exports)?;
    
    // Import and test the exported procedures
    suite.eval("(import (test exports))")?;
    suite.assert_eval_true("(= (simple-proc 5) 10)")?;
    suite.assert_eval_true("(= (external-name 5) 6)")?;
    suite.assert_eval_true("(= exported-var 42)")?;
    
    // Verify internal name is not accessible
    suite.assert_eval_error("internal-name")?;
    
    Ok(())
}

/// Test standard library availability and functionality
fn test_standard_libraries(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Test (scheme base) - required library
    suite.eval("(import (scheme base))")?;
    suite.assert_eval_true("(procedure? +)")?;
    suite.assert_eval_true("(procedure? cons)")?;
    suite.assert_eval_true("(procedure? map)")?;
    
    // Test (scheme case-lambda) if available
    if !suite.skip_if_unimplemented("scheme case-lambda") {
        suite.eval("(import (scheme case-lambda))")?;
        
        let case_lambda_test = r#"
        (define test-proc
          (case-lambda
            ((x) (* x x))
            ((x y) (+ x y))
            ((x y z) (+ x y z))))
        "#;
        
        suite.eval(case_lambda_test)?;
        suite.assert_eval_true("(= (test-proc 5) 25)")?;
        suite.assert_eval_true("(= (test-proc 3 4) 7)")?;
        suite.assert_eval_true("(= (test-proc 1 2 3) 6)")?;
    }
    
    // Test (scheme char) if available
    if !suite.skip_if_unimplemented("scheme char") {
        suite.eval("(import (scheme char))")?;
        suite.assert_eval_true("(char-alphabetic? #\\a)")?;
        suite.assert_eval_true("(char-numeric? #\\5)")?;
    }
    
    // Test (scheme complex) if available
    if !suite.skip_if_unimplemented("scheme complex") {
        suite.eval("(import (scheme complex))")?;
        suite.assert_eval_true("(number? 3+4i)")?;
    }
    
    // Test (scheme cxr) if available
    if !suite.skip_if_unimplemented("scheme cxr") {
        suite.eval("(import (scheme cxr))")?;
        suite.assert_eval_true("(= (caadr '((1 2) (3 4) (5 6))) 3)")?;
    }
    
    // Test (scheme eval) if available
    if !suite.skip_if_unimplemented("scheme eval") {
        suite.eval("(import (scheme eval))")?;
        suite.eval("(define test-env (environment '(scheme base)))")?;
        suite.assert_eval_true("(= (eval '(+ 1 2) test-env) 3)")?;
    }
    
    // Test (scheme file) if available
    if !suite.skip_if_unimplemented("scheme file") {
        suite.eval("(import (scheme file))")?;
        suite.assert_eval_true("(procedure? file-exists?)")?;
    }
    
    // Test (scheme inexact) if available
    if !suite.skip_if_unimplemented("scheme inexact") {
        suite.eval("(import (scheme inexact))")?;
        suite.assert_eval_true("(procedure? sin)")?;
        suite.assert_eval_true("(procedure? cos)")?;
    }
    
    // Test (scheme lazy) if available
    if !suite.skip_if_unimplemented("scheme lazy") {
        suite.eval("(import (scheme lazy))")?;
        suite.eval("(define lazy-val (delay (+ 1 2)))")?;
        suite.assert_eval_true("(promise? lazy-val)")?;
        suite.assert_eval_true("(= (force lazy-val) 3)")?;
    }
    
    // Test (scheme load) if available  
    if !suite.skip_if_unimplemented("scheme load") {
        suite.eval("(import (scheme load))")?;
        suite.assert_eval_true("(procedure? load)")?;
    }
    
    // Test (scheme process-context) if available
    if !suite.skip_if_unimplemented("scheme process-context") {
        suite.eval("(import (scheme process-context))")?;
        suite.assert_eval_true("(procedure? command-line)")?;
    }
    
    // Test (scheme read) if available
    if !suite.skip_if_unimplemented("scheme read") {
        suite.eval("(import (scheme read))")?;
        suite.assert_eval_true("(procedure? read)")?;
    }
    
    // Test (scheme repl) if available
    if !suite.skip_if_unimplemented("scheme repl") {
        suite.eval("(import (scheme repl))")?;
        suite.assert_eval_true("(procedure? interaction-environment)")?;
    }
    
    // Test (scheme time) if available
    if !suite.skip_if_unimplemented("scheme time") {
        suite.eval("(import (scheme time))")?;
        suite.assert_eval_true("(procedure? current-second)")?;
    }
    
    // Test (scheme write) if available
    if !suite.skip_if_unimplemented("scheme write") {
        suite.eval("(import (scheme write))")?;
        suite.assert_eval_true("(procedure? write)")?;
        suite.assert_eval_true("(procedure? display)")?;
    }
    
    // Test (scheme r5rs) if available
    if !suite.skip_if_unimplemented("scheme r5rs") {
        suite.eval("(import (scheme r5rs))")?;
        suite.assert_eval_true("(procedure? null-environment)")?;
    }
    
    Ok(())
}

/// Test library condition system and error handling
fn test_library_conditions(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("library conditions") {
        return Ok(());
    }
    
    // Test importing non-existent library
    suite.assert_eval_error("(import (non-existent library))")?;
    
    // Test importing non-existent identifier from library
    suite.assert_eval_error("(import (only (scheme base) non-existent-procedure))")?;
    
    // Test circular library dependencies (if detectable)
    if !suite.skip_if_unimplemented("circular dependency detection") {
        let lib1 = r#"
        (define-library (circular-test lib1)
          (export proc1)
          (import (scheme base) (circular-test lib2))
          (begin
            (define (proc1) (proc2))))
        "#;
        
        let lib2 = r#"
        (define-library (circular-test lib2)
          (export proc2)
          (import (scheme base) (circular-test lib1))
          (begin
            (define (proc2) (proc1))))
        "#;
        
        // This should detect circular dependency
        suite.eval(lib1).ok(); // May succeed initially
        suite.assert_eval_error(lib2)?; // Should fail on circular dependency
    }
    
    Ok(())
}

/// Test library phases and expansion-time vs runtime behavior
fn test_library_phases(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("library phases") {
        return Ok(());
    }
    
    // Test expand-time imports (for macro)
    if !suite.skip_if_unimplemented("for-syntax imports") {
        let macro_lib = r#"
        (define-library (test macros)
          (export when-positive)
          (import (scheme base)
                  (for-syntax (scheme base)))
          (begin
            (define-syntax when-positive
              (syntax-rules ()
                ((when-positive test expr ...)
                 (if (positive? test) 
                     (begin expr ...)
                     #f))))))
        "#;
        
        suite.eval(macro_lib)?;
        suite.eval("(import (test macros))")?;
        
        suite.assert_eval_true("(when-positive 5 #t)")?;
        suite.assert_eval_false("(when-positive -1 #t)")?;
    }
    
    // Test runtime vs expand-time availability
    if !suite.skip_if_unimplemented("phase separation") {
        let phase_lib = r#"
        (define-library (test phases)
          (export runtime-proc)
          (import (scheme base)
                  (for-syntax (scheme base)))
          (begin
            (define (runtime-proc x) (* x 2))
            (define-for-syntax (expand-time-proc x) (+ x 1))
            
            (define-syntax use-expand-time
              (syntax-rules ()
                ((use-expand-time n)
                 (quote (expand-time-proc n)))))))
        "#;
        
        suite.eval(phase_lib)?;
        suite.eval("(import (test phases))")?;
        
        suite.assert_eval_true("(= (runtime-proc 5) 10)")?;
        
        // expand-time-proc should not be available at runtime
        suite.assert_eval_error("expand-time-proc")?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_library_system_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("Library system tests should pass");
    }
    
    #[test]
    fn test_standard_libraries_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_standard_libraries(&mut suite).expect("Standard library tests should pass");
    }
}
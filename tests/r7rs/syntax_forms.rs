//! R7RS Syntax Forms Tests (Section 5)
//!
//! Tests for R7RS-small section 5 including:
//! - Section 5.1: Programs and libraries
//! - Section 5.2: Import declarations 
//! - Section 5.3: Variable definitions
//! - Section 5.4: Syntax definitions
//! - Section 5.5: Record-type definitions
//! - Section 5.6: Libraries
//! - Section 5.7: The REPL
//!
//! This module tests the syntactic forms and their proper parsing,
//! macro expansion, and semantic analysis.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value};

/// Run all syntax forms tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS Syntax Forms tests...");
    
    test_variable_definitions(suite)?;
    test_procedure_definitions(suite)?;
    test_syntax_definitions(suite)?;
    test_library_declarations(suite)?;
    test_import_export(suite)?;
    test_record_definitions(suite)?;
    test_include_forms(suite)?;
    test_conditional_expansion(suite)?;
    test_syntax_rules(suite)?;
    test_identifier_syntax(suite)?;
    
    println!("✓ Syntax forms tests passed");
    Ok(())
}

/// Test variable definitions
fn test_variable_definitions(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Test basic variable definition
    suite.eval("(define var1 42)")?;
    suite.assert_eval_eq("var1", Value::Literal(Literal::integer(42)))?;
    
    // Test variable definition with expression
    suite.eval("(define var2 (+ 1 2 3))")?;
    suite.assert_eval_eq("var2", Value::Literal(Literal::integer(6)))?;
    
    // Test variable redefinition at top level
    suite.eval("(define var3 'first)")?;
    suite.assert_eval_eq("var3", Value::Symbol("first".to_string()))?;
    suite.eval("(define var3 'second)")?;
    suite.assert_eval_eq("var3", Value::Symbol("second".to_string()))?;
    
    // Test undefined variable access
    suite.assert_eval_error("undefined-variable")?;
    
    Ok(())
}

/// Test procedure definitions
fn test_procedure_definitions(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Test basic procedure definition
    suite.eval("(define (add x y) (+ x y))")?;
    suite.assert_eval_eq("(add 3 4)", Value::Literal(Literal::integer(7)))?;
    
    // Test procedure with no parameters
    suite.eval("(define (get-constant) 42)")?;
    suite.assert_eval_eq("(get-constant)", Value::Literal(Literal::integer(42)))?;
    
    // Test procedure with rest parameter
    if !suite.skip_if_unimplemented("rest parameters") {
        suite.eval("(define (variadic first . rest) (cons first rest))")?;
        suite.assert_eval_eq("(variadic 1 2 3)", Value::Pair(
            Box::new(Value::Literal(Literal::integer(1))),
            Box::new(Value::List(vec![
                Value::Literal(Literal::integer(2)),
                Value::Literal(Literal::integer(3))
            ]))
        ))?;
    }
    
    // Test procedure with only rest parameter
    if !suite.skip_if_unimplemented("rest parameters") {
        suite.eval("(define (all-args . args) args)")?;
        suite.assert_eval_eq("(all-args 1 2 3)", Value::List(vec![
            Value::Literal(Literal::integer(1)),
            Value::Literal(Literal::integer(2)),
            Value::Literal(Literal::integer(3))
        ]))?;
        suite.assert_eval_eq("(all-args)", Value::List(vec![]))?;
    }
    
    // Test recursive procedure
    suite.eval("(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))")?;
    suite.assert_eval_eq("(factorial 5)", Value::Literal(Literal::integer(120)))?;
    
    // Test procedure with internal definitions
    let proc_with_internal = r#"
        (define (complex-proc x)
          (define helper1 (lambda (y) (* y 2)))
          (define helper2 (lambda (z) (+ z 1)))
          (helper2 (helper1 x)))
    "#;
    suite.eval(proc_with_internal)?;
    suite.assert_eval_eq("(complex-proc 5)", Value::Literal(Literal::integer(11)))?;
    
    Ok(())
}

/// Test syntax definitions (define-syntax)
fn test_syntax_definitions(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("define-syntax") {
        return Ok(());
    }
    
    // Test basic macro definition
    let when_macro = r#"
        (define-syntax when
          (syntax-rules ()
            ((when test body ...)
             (if test (begin body ...)))))
    "#;
    suite.eval(when_macro)?;
    
    // Test using the macro
    suite.assert_eval_eq("(when #t 1 2 3)", Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(when #f 1 2 3)", Value::Literal(Literal::Unspecified))?;
    
    // Test macro with multiple patterns
    let my_cond_macro = r#"
        (define-syntax my-cond
          (syntax-rules (else)
            ((my-cond (else body ...))
             (begin body ...))
            ((my-cond (test body ...))
             (if test (begin body ...)))
            ((my-cond (test body ...) clause ...)
             (if test (begin body ...) (my-cond clause ...)))))
    "#;
    suite.eval(my_cond_macro)?;
    
    suite.assert_eval_eq("(my-cond (#t 'yes) (else 'no))", Value::Symbol("yes".to_string()))?;
    suite.assert_eval_eq("(my-cond (#f 'no) (else 'yes))", Value::Symbol("yes".to_string()))?;
    
    // Test macro hygiene
    let hygienic_macro = r#"
        (define-syntax swap!
          (syntax-rules ()
            ((swap! x y)
             (let ((temp x))
               (set! x y)
               (set! y temp)))))
    "#;
    suite.eval(hygienic_macro)?;
    
    suite.eval("(define a 1)")?;
    suite.eval("(define b 2)")?;
    suite.eval("(define temp 99)")?; // This should not interfere
    suite.eval("(swap! a b)")?;
    suite.assert_eval_eq("a", Value::Literal(Literal::integer(2)))?;
    suite.assert_eval_eq("b", Value::Literal(Literal::integer(1)))?;
    suite.assert_eval_eq("temp", Value::Literal(Literal::integer(99)))?; // Should be unchanged
    
    Ok(())
}

/// Test library declarations
fn test_library_declarations(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("libraries") {
        return Ok(());
    }
    
    // Test basic library definition
    let simple_library = r#"
        (define-library (test simple)
          (export square double)
          (import (scheme base))
          (begin
            (define (square x) (* x x))
            (define (double x) (* x 2))))
    "#;
    suite.eval(simple_library)?;
    
    // Test library with conditional export
    let conditional_library = r#"
        (define-library (test conditional)
          (export 
            main-proc
            (cond-expand 
              (test-feature extra-proc)))
          (import (scheme base))
          (begin
            (define (main-proc) 'main)
            (cond-expand
              (test-feature
                (define (extra-proc) 'extra)))))
    "#;
    suite.eval(conditional_library)?;
    
    Ok(())
}

/// Test import and export declarations
fn test_import_export(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("import/export") {
        return Ok(());
    }
    
    // Test basic import
    suite.eval("(import (scheme base))")?;
    
    // Test selective import
    suite.eval("(import (only (scheme base) + - * /))")?;
    
    // Test import with renaming
    suite.eval("(import (rename (scheme base) (+ plus) (- minus)))")?;
    suite.assert_eval_eq("(plus 1 2)", Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(minus 5 2)", Value::Literal(Literal::integer(3)))?;
    
    // Test import with prefix
    suite.eval("(import (prefix (scheme base) base:))")?;
    suite.assert_eval_eq("(base:+ 1 2)", Value::Literal(Literal::integer(3)))?;
    
    // Test import except
    suite.eval("(import (except (scheme base) +))")?;
    // + should not be available now (in this context)
    
    Ok(())
}

/// Test record-type definitions
fn test_record_definitions(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("define-record-type") {
        return Ok(());
    }
    
    // Test basic record definition
    let point_record = r#"
        (define-record-type point
          (make-point x y)
          point?
          (x point-x set-point-x!)
          (y point-y set-point-y!))
    "#;
    suite.eval(point_record)?;
    
    // Test record creation and access
    suite.eval("(define p (make-point 3 4))")?;
    suite.assert_eval_true("(point? p)")?;
    suite.assert_eval_eq("(point-x p)", Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(point-y p)", Value::Literal(Literal::integer(4)))?;
    
    // Test record mutation
    suite.eval("(set-point-x! p 10)")?;
    suite.assert_eval_eq("(point-x p)", Value::Literal(Literal::integer(10)))?;
    
    // Test record predicate with non-record
    suite.assert_eval_false("(point? 42)")?;
    suite.assert_eval_false("(point? '(3 4))")?;
    
    // Test immutable record
    let color_record = r#"
        (define-record-type color
          (make-color r g b)
          color?
          (r color-r)
          (g color-g)
          (b color-b))
    "#;
    suite.eval(color_record)?;
    
    suite.eval("(define red (make-color 255 0 0))")?;
    suite.assert_eval_eq("(color-r red)", Value::Literal(Literal::integer(255)))?;
    
    Ok(())
}

/// Test include forms
fn test_include_forms(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("include") {
        return Ok(());
    }
    
    // Test include (would need actual files in real implementation)
    // For now, test that the syntax is recognized
    suite.assert_eval_error("(include \"nonexistent.scm\")")?; // Should error gracefully
    
    // Test include-ci (case-insensitive)
    if !suite.skip_if_unimplemented("include-ci") {
        suite.assert_eval_error("(include-ci \"nonexistent.scm\")")?;
    }
    
    Ok(())
}

/// Test conditional expansion (cond-expand)
fn test_conditional_expansion(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("cond-expand") {
        return Ok(());
    }
    
    // Test basic cond-expand
    let cond_expand_test = r#"
        (cond-expand
          (r7rs (define implementation 'r7rs))
          (else (define implementation 'other)))
    "#;
    suite.eval(cond_expand_test)?;
    suite.assert_eval_eq("implementation", Value::Symbol("r7rs".to_string()))?;
    
    // Test cond-expand with feature expressions
    let feature_test = r#"
        (cond-expand
          ((and r7rs (not r6rs)) (define test-result 'r7rs-only))
          (r6rs (define test-result 'r6rs))
          (else (define test-result 'unknown)))
    "#;
    suite.eval(feature_test)?;
    suite.assert_eval_eq("test-result", Value::Symbol("r7rs-only".to_string()))?;
    
    // Test library cond-expand
    let library_feature_test = r#"
        (cond-expand
          ((library (scheme base)) (define has-base #t))
          (else (define has-base #f)))
    "#;
    suite.eval(library_feature_test)?;
    suite.assert_eval_true("has-base")?;
    
    Ok(())
}

/// Test syntax-rules in detail
fn test_syntax_rules(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("syntax-rules") {
        return Ok(());
    }
    
    // Test ellipsis patterns
    let list_macro = r#"
        (define-syntax make-list
          (syntax-rules ()
            ((make-list item ...)
             (list item ...))))
    "#;
    suite.eval(list_macro)?;
    suite.assert_eval_eq("(make-list 1 2 3)", Value::List(vec![
        Value::Literal(Literal::integer(1)),
        Value::Literal(Literal::integer(2)),
        Value::Literal(Literal::integer(3))
    ]))?;
    
    // Test nested ellipsis
    let nested_macro = r#"
        (define-syntax let*
          (syntax-rules ()
            ((let* () body ...)
             (begin body ...))
            ((let* ((var val) binding ...) body ...)
             (let ((var val))
               (let* (binding ...) body ...)))))
    "#;
    suite.eval(nested_macro)?;
    
    suite.assert_eval_eq("(let* ((x 1) (y (+ x 1))) y)", Value::Literal(Literal::integer(2)))?;
    
    // Test literal identifiers
    let case_macro = r#"
        (define-syntax my-case
          (syntax-rules (else)
            ((my-case key ((datum ...) body ...) ... (else else-body ...))
             (let ((temp key))
               (cond ((memv temp '(datum ...)) (begin body ...)) ...
                     (else (begin else-body ...)))))
            ((my-case key ((datum ...) body ...) ...)
             (let ((temp key))
               (cond ((memv temp '(datum ...)) (begin body ...)) ...)))))
    "#;
    suite.eval(case_macro)?;
    
    suite.assert_eval_eq("(my-case 'b ((a) 'first) ((b c) 'second) (else 'other))", 
                       Value::Symbol("second".to_string()))?;
    
    Ok(())
}

/// Test identifier syntax
fn test_identifier_syntax(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("identifier-syntax") {
        return Ok(());
    }
    
    // Test basic identifier syntax
    let id_syntax = r#"
        (define-syntax current-time
          (identifier-syntax (get-current-time)))
    "#;
    suite.eval(id_syntax)?;
    
    // Test identifier syntax with set!
    let settable_id = r#"
        (define x 0)
        (define-syntax counter
          (identifier-syntax
            (id x)
            ((set! id val) (set! x val))))
    "#;
    suite.eval(settable_id)?;
    
    suite.assert_eval_eq("counter", Value::Literal(Literal::integer(0)))?;
    suite.eval("(set! counter 42)")?;
    suite.assert_eval_eq("counter", Value::Literal(Literal::integer(42)))?;
    suite.assert_eval_eq("x", Value::Literal(Literal::integer(42)))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_syntax_forms_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("Syntax forms tests should pass");
    }
    
    #[test]
    fn test_variable_definitions_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_variable_definitions(&mut suite).expect("Variable definition tests should pass");
    }
    
    #[test]
    fn test_procedure_definitions_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_procedure_definitions(&mut suite).expect("Procedure definition tests should pass");
    }
    
    #[test]
    fn test_syntax_definitions_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_syntax_definitions(&mut suite).expect("Syntax definition tests should pass");
    }
}
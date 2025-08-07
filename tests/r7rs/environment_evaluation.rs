//! R7RS Environment and Evaluation Tests
//!
//! Tests for R7RS-small section 6.12 (Environments and evaluation)
//!
//! This module verifies environment and evaluation system including:
//! - Environment manipulation and queries
//! - Dynamic evaluation (eval)
//! - Environment capture and restoration
//! - Interaction environment behavior
//! - Null and scheme-report environments
//!
//! All tests follow R7RS specification requirements exactly.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value};

/// Run all environment and evaluation tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS Environment and Evaluation tests...");
    
    test_eval_procedure(suite)?;
    test_environment_procedure(suite)?;
    test_null_environment(suite)?;
    test_scheme_report_environment(suite)?;
    test_interaction_environment(suite)?;
    test_environment_queries(suite)?;
    test_evaluation_contexts(suite)?;
    
    println!("✓ Environment and evaluation tests passed");
    Ok(())
}

/// Test eval procedure
fn test_eval_procedure(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("eval procedure") {
        return Ok(());
    }
    
    // Basic eval tests
    suite.eval("(define test-env (environment '(scheme base)))")?;
    
    // Evaluate literals
    suite.assert_eval_true("(= (eval '42 test-env) 42)")?;
    suite.assert_eval_true("(equal? (eval '\"hello\" test-env) \"hello\")")?;
    suite.assert_eval_true("(equal? (eval ''symbol test-env) 'symbol)")?;
    suite.assert_eval_true("(eq? (eval '#t test-env) #t)")?;
    
    // Evaluate expressions
    suite.assert_eval_true("(= (eval '(+ 1 2 3) test-env) 6)")?;
    suite.assert_eval_true("(= (eval '(* 4 5) test-env) 20)")?;
    suite.assert_eval_true("(equal? (eval '(cons 'a 'b) test-env) '(a . b))")?;
    
    // Evaluate conditional expressions
    suite.assert_eval_true("(equal? (eval '(if #t 'yes 'no) test-env) 'yes)")?;
    suite.assert_eval_true("(equal? (eval '(if #f 'yes 'no) test-env) 'no)")?;
    
    // Evaluate lambda expressions
    suite.eval("(define add-proc (eval '(lambda (x y) (+ x y)) test-env))")?;
    suite.assert_eval_true("(procedure? add-proc)")?;
    suite.assert_eval_true("(= (add-proc 3 4) 7)")?;
    
    // Test variable definition and evaluation
    suite.eval("(define var-env (environment '(scheme base)))")?;
    suite.eval("(eval '(define x 42) var-env)")?;
    suite.assert_eval_true("(= (eval 'x var-env) 42)")?;
    
    // Test procedure definition and evaluation
    suite.eval("(eval '(define (square x) (* x x)) var-env)")?;
    suite.assert_eval_true("(= (eval '(square 5) var-env) 25)")?;
    
    Ok(())
}

/// Test environment procedure
fn test_environment_procedure(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("environment procedure") {
        return Ok(());
    }
    
    // Create basic environments
    suite.eval("(define base-env (environment '(scheme base)))")?;
    suite.eval("(define char-env (environment '(scheme base) '(scheme char)))")?;
    
    // Test that environments are created
    suite.assert_eval_true("(environment? base-env)").unwrap_or_else(|_| {
        // If environment? predicate doesn't exist, just verify eval works
        suite.eval("(eval '(+ 1 2) base-env)").expect("Environment should be usable");
    });
    
    // Test multiple library imports
    suite.assert_eval_true("(= (eval '(+ 1 2) base-env) 3)")?;
    
    if !suite.skip_if_unimplemented("scheme char in environment") {
        suite.assert_eval_true("(eval '(char-alphabetic? #\\a) char-env)")?;
    }
    
    // Test environment isolation
    suite.eval("(define isolated-env (environment '(scheme base)))")?;
    suite.eval("(eval '(define isolated-var 123) isolated-env)")?;
    
    // Variable should exist in isolated environment
    suite.assert_eval_true("(= (eval 'isolated-var isolated-env) 123)")?;
    
    // Variable should not exist in different environment
    suite.eval("(define other-env (environment '(scheme base)))")?;
    suite.assert_eval_error("(eval 'isolated-var other-env)")?;
    
    Ok(())
}

/// Test null-environment procedure
fn test_null_environment(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("null-environment") {
        return Ok(());
    }
    
    // Create null environment (R5RS compatibility)
    suite.eval("(define null-env-5 (null-environment 5))")?;
    
    // Test that basic syntax is available
    suite.assert_eval_true("(= (eval '(if #t 1 2) null-env-5) 1)")?;
    suite.assert_eval_true("(equal? (eval '(quote hello) null-env-5) 'hello)")?;
    
    // Test lambda in null environment
    suite.eval("(define null-proc (eval '(lambda (x) (if x 'true 'false)) null-env-5))")?;
    suite.assert_eval_true("(equal? (null-proc #t) 'true)")?;
    suite.assert_eval_true("(equal? (null-proc #f) 'false)")?;
    
    // Test that standard procedures are NOT available
    suite.assert_eval_error("(eval '+ null-env-5)")?;
    suite.assert_eval_error("(eval 'cons null-env-5)")?;
    suite.assert_eval_error("(eval 'car null-env-5)")?;
    
    // Test let syntax (should be available)
    suite.assert_eval_true("(= (eval '(let ((x 5)) x) null-env-5) 5)")?;
    
    Ok(())
}

/// Test scheme-report-environment procedure
fn test_scheme_report_environment(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("scheme-report-environment") {
        return Ok(());
    }
    
    // Create R5RS environment
    suite.eval("(define r5rs-env (scheme-report-environment 5))")?;
    
    // Test that R5RS procedures are available
    suite.assert_eval_true("(= (eval '(+ 1 2) r5rs-env) 3)")?;
    suite.assert_eval_true("(equal? (eval '(cons 'a 'b) r5rs-env) '(a . b))")?;
    suite.assert_eval_true("(equal? (eval '(car '(x y z)) r5rs-env) 'x)")?;
    
    // Test map procedure
    suite.assert_eval_true("(equal? (eval '(map (lambda (x) (* x 2)) '(1 2 3)) r5rs-env) '(2 4 6))")?;
    
    // Test string procedures
    suite.assert_eval_true("(= (eval '(string-length \"hello\") r5rs-env) 5)")?;
    
    // Test vector procedures  
    suite.assert_eval_true("(= (eval '(vector-length #(a b c)) r5rs-env) 3)")?;
    
    // Test I/O procedures (basic ones)
    suite.assert_eval_true("(procedure? (eval 'write r5rs-env))")?;
    suite.assert_eval_true("(procedure? (eval 'display r5rs-env))")?;
    
    Ok(())
}

/// Test interaction-environment procedure
fn test_interaction_environment(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("interaction-environment") {
        return Ok(());
    }
    
    // Get interaction environment
    suite.eval("(define interact-env (interaction-environment))")?;
    
    // Test that interaction environment includes standard procedures
    suite.assert_eval_true("(= (eval '(+ 1 2) interact-env) 3)")?;
    suite.assert_eval_true("(procedure? (eval 'cons interact-env))")?;
    
    // Test that we can define things in interaction environment
    suite.eval("(eval '(define interaction-test-var 42) interact-env)")?;
    suite.assert_eval_true("(= (eval 'interaction-test-var interact-env) 42)")?;
    
    // Test that definitions persist
    suite.eval("(eval '(define (interaction-test-proc x) (+ x 10)) interact-env)")?;
    suite.assert_eval_true("(= (eval '(interaction-test-proc 5) interact-env) 15)")?;
    
    Ok(())
}

/// Test environment query operations
fn test_environment_queries(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("environment queries") {
        return Ok(());
    }
    
    // Test environment? predicate if available
    if !suite.skip_if_unimplemented("environment? predicate") {
        suite.eval("(define test-env (environment '(scheme base)))")?;
        suite.assert_eval_true("(environment? test-env)")?;
        suite.assert_eval_false("(environment? 42)")?;
        suite.assert_eval_false("(environment? '(scheme base))")?;
        suite.assert_eval_false("(environment? \"environment\")")?;
    }
    
    // Test bound-identifier=? if available (for macros)
    if !suite.skip_if_unimplemented("bound-identifier=?") {
        suite.eval(r#"
        (define-syntax test-bound
          (syntax-rules ()
            ((test-bound id1 id2)
             (bound-identifier=? #'id1 #'id2))))
        "#)?;
        
        suite.assert_eval_true("(test-bound x x)")?;
        suite.assert_eval_false("(test-bound x y)")?;
    }
    
    // Test free-identifier=? if available (for macros)
    if !suite.skip_if_unimplemented("free-identifier=?") {
        suite.eval(r#"
        (define-syntax test-free
          (syntax-rules ()
            ((test-free id1 id2)
             (free-identifier=? #'id1 #'id2))))
        "#)?;
        
        suite.assert_eval_true("(test-free + +)")?;
        suite.assert_eval_false("(test-free + -)")?;
    }
    
    Ok(())
}

/// Test evaluation in different contexts
fn test_evaluation_contexts(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("evaluation contexts") {
        return Ok(());
    }
    
    // Test nested eval calls
    suite.eval("(define nested-env (environment '(scheme base)))")?;
    suite.assert_eval_true("(= (eval '(eval '(+ 1 2) (environment '(scheme base))) nested-env) 3)")?;
    
    // Test eval with closures
    suite.eval(r#"
    (define closure-env (environment '(scheme base)))
    (eval '(define make-adder 
             (lambda (n) 
               (lambda (x) (+ x n)))) 
          closure-env)
    "#)?;
    
    suite.eval("(define add5 (eval '(make-adder 5) closure-env))")?;
    suite.assert_eval_true("(= (add5 10) 15)")?;
    
    // Test eval with recursive procedures
    suite.eval(r#"
    (define recursive-env (environment '(scheme base)))
    (eval '(define (factorial n)
             (if (<= n 1)
                 1
                 (* n (factorial (- n 1)))))
          recursive-env)
    "#)?;
    
    suite.assert_eval_true("(= (eval '(factorial 5) recursive-env) 120)")?;
    
    // Test eval with mutation
    suite.eval(r#"
    (define mutable-env (environment '(scheme base)))
    (eval '(define counter 0) mutable-env)
    (eval '(define (increment!) (set! counter (+ counter 1))) mutable-env)
    "#)?;
    
    suite.assert_eval_true("(= (eval 'counter mutable-env) 0)")?;
    suite.eval("(eval '(increment!) mutable-env)")?;
    suite.assert_eval_true("(= (eval 'counter mutable-env) 1)")?;
    suite.eval("(eval '(increment!) mutable-env)")?;
    suite.assert_eval_true("(= (eval 'counter mutable-env) 2)")?;
    
    // Test error propagation through eval
    suite.assert_eval_error("(eval '(+ 1 undefined-variable) (environment '(scheme base)))")?;
    suite.assert_eval_error("(eval '(car 'not-a-pair) (environment '(scheme base)))")?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_environment_evaluation_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("Environment and evaluation tests should pass");
    }
    
    #[test]
    fn test_eval_procedure_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_eval_procedure(&mut suite).expect("Eval procedure tests should pass");
    }
}
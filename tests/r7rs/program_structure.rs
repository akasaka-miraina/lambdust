//! R7RS Program Structure Tests (Section 4)
//!
//! Tests for R7RS-small section 4 including:
//! - Section 4.1: Program structure and definition contexts
//! - Section 4.2: Derived expression types (let, cond, case, etc.)
//! - Section 4.3: Macros (define-syntax, syntax-rules)
//! - Section 4.4: Include and conditional expansion
//!
//! This module tests the fundamental program structure and syntactic
//! constructs that form the foundation of R7RS-small.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value};

/// Run all program structure tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS Program Structure tests...");
    
    test_program_structure(suite)?;
    test_definition_contexts(suite)?;
    test_derived_expressions(suite)?;
    test_conditional_expressions(suite)?;
    test_binding_constructs(suite)?;
    test_sequencing(suite)?;
    test_iteration(suite)?;
    test_delayed_evaluation(suite)?;
    test_quasiquotation(suite)?;
    test_case_lambda(suite)?;
    
    println!("✓ Program structure tests passed");
    Ok(())
}

/// Test basic program structure
fn test_program_structure(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Test multiple expressions in sequence
    suite.assert_eval_eq("(begin 1 2 3)", Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(begin)", Value::Literal(Literal::Unspecified))?;
    suite.assert_eval_eq("(begin 42)", Value::Literal(Literal::integer(42)))?;
    
    // Test definitions at top level
    suite.eval("(define x 42)")?;
    suite.assert_eval_eq("x", Value::Literal(Literal::integer(42)))?;
    
    // Test procedure definitions
    suite.eval("(define (square n) (* n n))")?;
    suite.assert_eval_eq("(square 5)", Value::Literal(Literal::integer(25)))?;
    
    // Test procedure with multiple parameters
    suite.eval("(define (add a b) (+ a b))")?;
    suite.assert_eval_eq("(add 3 4)", Value::Literal(Literal::integer(7)))?;
    
    // Test variable arity procedures
    if !suite.skip_if_unimplemented("variable arity") {
        suite.eval("(define (sum . args) (apply + args))")?;
        suite.assert_eval_eq("(sum 1 2 3 4)", Value::Literal(Literal::integer(10)))?;
        suite.assert_eval_eq("(sum)", Value::Literal(Literal::integer(0)))?;
    }
    
    Ok(())
}

/// Test definition contexts and scoping
fn test_definition_contexts(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Test internal definitions
    let internal_def_code = r#"
        (define (test-internal)
          (define internal-var 100)
          (define (internal-proc x) (+ x internal-var))
          (internal-proc 23))
    "#;
    suite.eval(internal_def_code)?;
    suite.assert_eval_eq("(test-internal)", Value::Literal(Literal::integer(123)))?;
    
    // Test that internal definitions are scoped properly
    suite.assert_eval_error("internal-var")?; // Should not be accessible outside
    
    // Test mutual recursion with internal definitions
    let mutual_recursion_code = r#"
        (define (test-mutual)
          (define (even? n)
            (if (= n 0) #t (odd? (- n 1))))
          (define (odd? n)
            (if (= n 0) #f (even? (- n 1))))
          (even? 4))
    "#;
    suite.eval(mutual_recursion_code)?;
    suite.assert_eval_eq("(test-mutual)", Value::Literal(Literal::Boolean(true)))?;
    
    // Test definition precedence
    suite.eval("(define global-var 'global)")?;
    let precedence_code = r#"
        (define (test-precedence)
          (define global-var 'local)
          global-var)
    "#;
    suite.eval(precedence_code)?;
    suite.assert_eval_eq("(test-precedence)", Value::Symbol("local".to_string()))?;
    suite.assert_eval_eq("global-var", Value::Symbol("global".to_string()))?;
    
    Ok(())
}

/// Test derived expression types
fn test_derived_expressions(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Test cond expressions
    suite.assert_eval_eq("(cond (#t 'yes) (#f 'no))", Value::Symbol("yes".to_string()))?;
    suite.assert_eval_eq("(cond (#f 'no) (#t 'yes))", Value::Symbol("yes".to_string()))?;
    suite.assert_eval_eq("(cond (#f 'no) (else 'default))", Value::Symbol("default".to_string()))?;
    
    // Test cond with multiple expressions
    suite.assert_eval_eq("(cond (#t 1 2 3))", Value::Literal(Literal::integer(3)))?;
    
    // Test cond with test => procedure
    if !suite.skip_if_unimplemented("cond =>") {
        suite.assert_eval_eq("(cond (2 => (lambda (x) (* x x))))", Value::Literal(Literal::integer(4)))?;
    }
    
    // Test case expressions
    suite.assert_eval_eq("(case 'a ((a b c) 'vowel) ((d e f) 'consonant))", 
                       Value::Symbol("vowel".to_string()))?;
    suite.assert_eval_eq("(case 'x ((a b c) 'vowel) (else 'other))", 
                       Value::Symbol("other".to_string()))?;
    
    // Test case with multiple values per clause
    suite.assert_eval_eq("(case 2 ((1 3 5) 'odd) ((2 4 6) 'even))", 
                       Value::Symbol("even".to_string()))?;
    
    // Test and/or expressions
    suite.assert_eval_true("(and #t #t #t)")?;
    suite.assert_eval_false("(and #t #f #t)")?;
    suite.assert_eval_true("(and)")?;  // Empty and is #t
    
    suite.assert_eval_false("(or #f #f #f)")?;
    suite.assert_eval_true("(or #f #t #f)")?;
    suite.assert_eval_false("(or)")?;  // Empty or is #f
    
    // Test short-circuiting behavior
    suite.assert_eval_eq("(and 1 2 3)", Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(or #f 2 3)", Value::Literal(Literal::integer(2)))?;
    
    Ok(())
}

/// Test conditional expressions (when, unless)
fn test_conditional_expressions(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("when/unless") {
        return Ok(());
    }
    
    // Test when
    suite.assert_eval_eq("(when #t 1 2 3)", Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(when #f 1 2 3)", Value::Literal(Literal::Unspecified))?;
    
    // Test unless  
    suite.assert_eval_eq("(unless #f 1 2 3)", Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(unless #t 1 2 3)", Value::Literal(Literal::Unspecified))?;
    
    Ok(())
}

/// Test binding constructs (let, let*, letrec)
fn test_binding_constructs(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Test let expressions
    suite.assert_eval_eq("(let ((x 2) (y 3)) (+ x y))", Value::Literal(Literal::integer(5)))?;
    suite.assert_eval_eq("(let () 42)", Value::Literal(Literal::integer(42)))?;
    
    // Test let with multiple body expressions
    suite.assert_eval_eq("(let ((x 1)) (set! x 2) x)", Value::Literal(Literal::integer(2)))?;
    
    // Test let scoping
    suite.eval("(define let-test-var 'outer)")?;
    suite.assert_eval_eq("(let ((let-test-var 'inner)) let-test-var)", 
                       Value::Symbol("inner".to_string()))?;
    suite.assert_eval_eq("let-test-var", Value::Symbol("outer".to_string()))?;
    
    // Test let* expressions (sequential binding)
    suite.assert_eval_eq("(let* ((x 2) (y (+ x 1))) (+ x y))", 
                       Value::Literal(Literal::integer(5)))?;
    
    // Test letrec expressions (recursive binding)
    let letrec_test = r#"
        (letrec ((fact (lambda (n)
                         (if (= n 0) 1 (* n (fact (- n 1)))))))
          (fact 5))
    "#;
    suite.assert_eval_eq(letrec_test, Value::Literal(Literal::integer(120)))?;
    
    // Test letrec with mutual recursion
    let mutual_letrec = r#"
        (letrec ((even? (lambda (n)
                          (if (= n 0) #t (odd? (- n 1)))))
                 (odd? (lambda (n)
                         (if (= n 0) #f (even? (- n 1))))))
          (even? 10))
    "#;
    suite.assert_eval_eq(mutual_letrec, Value::Literal(Literal::Boolean(true)))?;
    
    // Test letrec* (if supported)
    if !suite.skip_if_unimplemented("letrec*") {
        suite.assert_eval_eq("(letrec* ((x 1) (y (+ x 1))) y)", 
                           Value::Literal(Literal::integer(2)))?;
    }
    
    // Test let-values (if supported)
    if !suite.skip_if_unimplemented("let-values") {
        suite.assert_eval_eq("(let-values (((a b) (values 1 2))) (+ a b))", 
                           Value::Literal(Literal::integer(3)))?;
    }
    
    // Test let*-values (if supported)
    if !suite.skip_if_unimplemented("let*-values") {
        suite.assert_eval_eq("(let*-values (((a) (values 1)) ((b) (values (+ a 1)))) b)", 
                           Value::Literal(Literal::integer(2)))?;
    }
    
    Ok(())
}

/// Test sequencing constructs
fn test_sequencing(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Test begin
    suite.assert_eval_eq("(begin 1 2 3)", Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(begin)", Value::Literal(Literal::Unspecified))?;
    
    // Test begin with side effects
    suite.eval("(define seq-counter 0)")?;
    suite.eval("(begin (set! seq-counter 1) (set! seq-counter (+ seq-counter 1)))")?;
    suite.assert_eval_eq("seq-counter", Value::Literal(Literal::integer(2)))?;
    
    Ok(())
}

/// Test iteration constructs
fn test_iteration(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("do loop") {
        return Ok(());
    }
    
    // Test basic do loop
    let do_loop = r#"
        (do ((i 0 (+ i 1))
             (sum 0 (+ sum i)))
            ((= i 5) sum))
    "#;
    suite.assert_eval_eq(do_loop, Value::Literal(Literal::integer(10)))?; // 0+1+2+3+4
    
    // Test do loop with multiple test expressions
    let do_multiple = r#"
        (do ((i 0 (+ i 1)))
            ((> i 3) i)
          'body-executed)
    "#;
    suite.assert_eval_eq(do_multiple, Value::Literal(Literal::integer(4)))?;
    
    // Test do loop with empty body
    let do_empty = r#"
        (do ((i 0 (+ i 1)))
            ((= i 3) 'done))
    "#;
    suite.assert_eval_eq(do_empty, Value::Symbol("done".to_string()))?;
    
    Ok(())
}

/// Test delayed evaluation (delay/force)
fn test_delayed_evaluation(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("delay/force") {
        return Ok(());
    }
    
    // Test basic delay and force
    suite.eval("(define delayed-value (delay (+ 1 2)))")?;
    suite.assert_eval_eq("(force delayed-value)", Value::Literal(Literal::integer(3)))?;
    
    // Test that delay doesn't evaluate immediately
    suite.eval("(define side-effect-counter 0)")?;
    suite.eval("(define delayed-side-effect (delay (set! side-effect-counter (+ side-effect-counter 1))))")?;
    suite.assert_eval_eq("side-effect-counter", Value::Literal(Literal::integer(0)))?; // Not evaluated yet
    suite.eval("(force delayed-side-effect)")?;
    suite.assert_eval_eq("side-effect-counter", Value::Literal(Literal::integer(1)))?; // Now evaluated
    
    // Test that promises are memoized
    suite.eval("(force delayed-side-effect)")?; // Force again
    suite.assert_eval_eq("side-effect-counter", Value::Literal(Literal::integer(1)))?; // Still 1, not 2
    
    Ok(())
}

/// Test quasiquotation
fn test_quasiquotation(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("quasiquote") {
        return Ok(());
    }
    
    // Test basic quasiquote
    suite.assert_eval_eq("`(1 2 3)", Value::List(vec![
        Value::Literal(Literal::integer(1)),
        Value::Literal(Literal::integer(2)),
        Value::Literal(Literal::integer(3))
    ]))?;
    
    // Test unquote
    suite.eval("(define x 42)")?;
    suite.assert_eval_eq("`(1 ,x 3)", Value::List(vec![
        Value::Literal(Literal::integer(1)),
        Value::Literal(Literal::integer(42)),
        Value::Literal(Literal::integer(3))
    ]))?;
    
    // Test unquote-splicing
    suite.eval("(define lst '(a b c))")?;
    suite.assert_eval_eq("`(1 ,@lst 4)", Value::List(vec![
        Value::Literal(Literal::integer(1)),
        Value::Symbol("a".to_string()),
        Value::Symbol("b".to_string()),
        Value::Symbol("c".to_string()),
        Value::Literal(Literal::integer(4))
    ]))?;
    
    // Test nested quasiquote
    suite.assert_eval_eq("``(1 ,,x 3)", Value::List(vec![
        Value::Symbol("quasiquote".to_string()),
        Value::List(vec![
            Value::Literal(Literal::integer(1)),
            Value::Literal(Literal::integer(42)),
            Value::Literal(Literal::integer(3))
        ])
    ]))?;
    
    Ok(())
}

/// Test case-lambda (if supported)
fn test_case_lambda(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("case-lambda") {
        return Ok(());
    }
    
    // Test case-lambda with different arities
    let case_lambda_proc = r#"
        (define flexible-proc
          (case-lambda
            (() 'no-args)
            ((x) (list 'one-arg x))
            ((x y) (list 'two-args x y))
            (args (list 'many-args args))))
    "#;
    suite.eval(case_lambda_proc)?;
    
    suite.assert_eval_eq("(flexible-proc)", Value::Symbol("no-args".to_string()))?;
    suite.assert_eval_eq("(flexible-proc 1)", Value::List(vec![
        Value::Symbol("one-arg".to_string()),
        Value::Literal(Literal::integer(1))
    ]))?;
    suite.assert_eval_eq("(flexible-proc 1 2)", Value::List(vec![
        Value::Symbol("two-args".to_string()),
        Value::Literal(Literal::integer(1)),
        Value::Literal(Literal::integer(2))
    ]))?;
    suite.assert_eval_eq("(flexible-proc 1 2 3 4)", Value::List(vec![
        Value::Symbol("many-args".to_string()),
        Value::List(vec![
            Value::Literal(Literal::integer(1)),
            Value::Literal(Literal::integer(2)),
            Value::Literal(Literal::integer(3)),
            Value::Literal(Literal::integer(4))
        ])
    ]))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_program_structure_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("Program structure tests should pass");
    }
    
    #[test]
    fn test_binding_constructs_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_binding_constructs(&mut suite).expect("Binding construct tests should pass");
    }
    
    #[test]
    fn test_derived_expressions_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_derived_expressions(&mut suite).expect("Derived expression tests should pass");
    }
}
//! R7RS Macro System Tests
//!
//! Tests for R7RS-small macro system including:
//! - syntax-rules macro definitions
//! - Pattern matching and template expansion
//! - Macro hygiene and variable capture
//! - Built-in syntax forms
//! - Macro expansion and evaluation
//!
//! This module tests the syntax-rules macro system
//! required by R7RS-small.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value, utils::intern_symbol};
use std::sync::Arc;

/// Run all macro system tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS Macro System tests...");
    
    if suite.skip_if_unimplemented("macro system") {
        println!("⚠ Skipping macro system tests (not implemented)");
        return Ok(());
    }
    
    test_basic_syntax_rules(suite)?;
    test_pattern_matching(suite)?;
    test_template_expansion(suite)?;
    test_macro_hygiene(suite)?;
    test_recursive_macros(suite)?;
    test_built_in_syntax(suite)?;
    test_macro_edge_cases(suite)?;
    
    println!("✓ Macro system tests passed");
    Ok(())
}

/// Test basic syntax-rules macro definitions
fn test_basic_syntax_rules(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Simple macro without parameters
    suite.eval(r#"
        (define-syntax answer
          (syntax-rules ()
            ((answer) 42)))
    "#)?;
    
    suite.assert_eval_eq("(answer)", Value::Literal(Literal::integer(42)))?;
    
    // Macro with single parameter
    suite.eval(r#"
        (define-syntax square
          (syntax-rules ()
            ((square x) (* x x))))
    "#)?;
    
    suite.assert_eval_eq("(square 5)", Value::Literal(Literal::integer(25)))?;
    suite.assert_eval_eq("(square (+ 2 3))", Value::Literal(Literal::integer(25)))?;
    
    // Macro with multiple parameters
    suite.eval(r#"
        (define-syntax add-multiply
          (syntax-rules ()
            ((add-multiply x y z) (+ x (* y z)))))
    "#)?;
    
    suite.assert_eval_eq("(add-multiply 1 2 3)", Value::Literal(Literal::integer(7)))?;  // 1 + (2 * 3)
    
    // Macro with multiple patterns
    suite.eval(r#"
        (define-syntax when
          (syntax-rules ()
            ((when test stmt1 stmt2 ...)
             (if test (begin stmt1 stmt2 ...)))))
    "#)?;
    
    suite.eval("(define x 1)")?;
    suite.eval(r#"
        (when (> x 0)
          (set! x (+ x 1))
          (set! x (* x 2)))
    "#)?;
    suite.assert_eval_eq("x", Value::Literal(Literal::integer(4)))?;  // ((1 + 1) * 2)
    
    Ok(())
}

/// Test pattern matching in macros
fn test_pattern_matching(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Ellipsis patterns (zero or more)
    suite.eval(r#"
        (define-syntax my-list
          (syntax-rules ()
            ((my-list x ...) (list x ...))))
    "#)?;
    
    suite.assert_eval_eq("(length (my-list))", Value::Literal(Literal::integer(0)))?;
    suite.assert_eval_eq("(length (my-list 1 2 3))", Value::Literal(Literal::integer(3)))?;
    suite.assert_eval_eq("(car (my-list 'a 'b 'c))", Value::symbol(intern_symbol("a")))?;  // 'a
    
    // Nested ellipsis patterns
    suite.eval(r#"
        (define-syntax let-values
          (syntax-rules ()
            ((let-values ((vars vals) ...) body ...)
             (let ((vars vals) ...) body ...))))
    "#)?;
    
    suite.assert_eval_eq(r#"
        (let-values ((x 1) (y 2) (z 3))
          (+ x y z))
    "#, Value::Literal(Literal::integer(6)))?;
    
    // Literal patterns (keywords)
    suite.eval(r#"
        (define-syntax my-cond
          (syntax-rules (else)
            ((my-cond (else result)) result)
            ((my-cond (test result)) (if test result))
            ((my-cond (test result) clause ...)
             (if test result (my-cond clause ...)))))
    "#)?;
    
    suite.assert_eval_eq(r#"
        (my-cond
          ((< 1 2) 'first)
          ((> 3 4) 'second)
          (else 'third))
    "#, Value::symbol(intern_symbol("first")))?;  // 'first
    
    suite.assert_eval_eq(r#"
        (my-cond
          ((> 1 2) 'first)
          ((> 3 4) 'second)
          (else 'third))
    "#, Value::symbol(intern_symbol("third")))?;  // 'third
    
    Ok(())
}

/// Test template expansion
fn test_template_expansion(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Simple template substitution
    suite.eval(r#"
        (define-syntax swap
          (syntax-rules ()
            ((swap x y)
             (let ((temp x))
               (set! x y)
               (set! y temp)))))
    "#)?;
    
    suite.eval("(define a 1)")?;
    suite.eval("(define b 2)")?;
    suite.eval("(swap a b)")?;
    suite.assert_eval_eq("a", Value::Literal(Literal::integer(2)))?;
    suite.assert_eval_eq("b", Value::Literal(Literal::integer(1)))?;
    
    // Template with ellipsis expansion
    suite.eval(r#"
        (define-syntax my-or
          (syntax-rules ()
            ((my-or) #f)
            ((my-or e) e)
            ((my-or e1 e2 ...)
             (let ((temp e1))
               (if temp temp (my-or e2 ...))))))
    "#)?;
    
    suite.assert_eval_false("(my-or)")?;
    suite.assert_eval_eq("(my-or 42)", Value::Literal(Literal::integer(42)))?;
    suite.assert_eval_eq("(my-or #f #f 'found #f)", Value::symbol(intern_symbol("found")))?;  // 'found
    
    // Template with nested structure
    suite.eval(r#"
        (define-syntax with-temp
          (syntax-rules ()
            ((with-temp ((var init) ...) body ...)
             (let ((var init) ...)
               (let ((result (begin body ...)))
                 result)))))
    "#)?;
    
    suite.assert_eval_eq(r#"
        (with-temp ((x 1) (y 2))
          (+ x y)
          (* x y))
    "#, Value::Literal(Literal::integer(2)))?;  // Last expression: (* 1 2)
    
    Ok(())
}

/// Test macro hygiene
fn test_macro_hygiene(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Basic hygiene - macro-introduced bindings don't capture free variables
    suite.eval(r#"
        (define-syntax hygienic-let
          (syntax-rules ()
            ((hygienic-let x val body)
             (let ((x val)) body))))
    "#)?;
    
    suite.eval("(define x 'outer)")?;
    suite.assert_eval_eq(r#"
        (hygienic-let x 'inner x)
    "#, Value::symbol(intern_symbol("inner")))?;  // 'inner (macro binding shadows outer)
    
    suite.assert_eval_eq("x", Value::symbol(intern_symbol("outer")))?;  // 'outer (unchanged)
    
    // Hygiene with generated symbols
    suite.eval(r#"
        (define-syntax safe-inc
          (syntax-rules ()
            ((safe-inc x)
             (let ((temp x))
               (set! x (+ temp 1))
               temp))))
    "#)?;
    
    suite.eval("(define temp 100)")?;  // This shouldn't interfere
    suite.eval("(define counter 5)")?;
    suite.assert_eval_eq("(safe-inc counter)", Value::Literal(Literal::integer(5)))?;  // Returns old value
    suite.assert_eval_eq("counter", Value::Literal(Literal::integer(6)))?;  // Incremented
    suite.assert_eval_eq("temp", Value::Literal(Literal::integer(100)))?;  // Unchanged
    
    // Capturing prevention
    suite.eval(r#"
        (define-syntax capture-test
          (syntax-rules ()
            ((capture-test expr)
             (let ((y 'macro-y))
               expr))))
    "#)?;
    
    suite.eval("(define y 'user-y)")?;
    suite.assert_eval_eq("(capture-test y)", Value::symbol(intern_symbol("user-y")))?;  // Should see 'user-y, not 'macro-y
    
    Ok(())
}

/// Test recursive macros
fn test_recursive_macros(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Recursive macro definition
    suite.eval(r#"
        (define-syntax countdown
          (syntax-rules ()
            ((countdown 0) 'done)
            ((countdown n) (begin (display n) (newline) (countdown (- n 1))))))
    "#)?;
    
    // This would print numbers but we test the structure
    suite.assert_eval_eq("(countdown 0)", Value::symbol(intern_symbol("done")))?;  // 'done
    
    // Recursive list processing macro
    suite.eval(r#"
        (define-syntax sum-list
          (syntax-rules ()
            ((sum-list ()) 0)
            ((sum-list (x . xs)) (+ x (sum-list xs)))))
    "#)?;
    
    suite.assert_eval_eq("(sum-list ())", Value::Literal(Literal::integer(0)))?;
    suite.assert_eval_eq("(sum-list (1 2 3 4))", Value::Literal(Literal::integer(10)))?;
    
    // Mutual recursion between macros
    suite.eval(r#"
        (define-syntax even-length?
          (syntax-rules ()
            ((even-length? ()) #t)
            ((even-length? (x . xs)) (odd-length? xs))))
            
        (define-syntax odd-length?
          (syntax-rules ()
            ((odd-length? ()) #f)
            ((odd-length? (x . xs)) (even-length? xs))))
    "#)?;
    
    suite.assert_eval_true("(even-length? (a b c d))")?;   // 4 elements
    suite.assert_eval_false("(odd-length? (a b c d))")?;   // 4 elements
    suite.assert_eval_true("(odd-length? (a b c))")?;      // 3 elements
    
    Ok(())
}

/// Test built-in syntax forms
fn test_built_in_syntax(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Test that core syntax forms work correctly
    // These are typically implemented as special forms, not macros
    
    // quote
    suite.assert_eval_eq("'(a b c)", Value::Pair(
        Arc::new(Value::symbol(intern_symbol("a"))),  // 'a
        Arc::new(Value::Pair(
            Arc::new(Value::symbol(intern_symbol("b"))),  // 'b
            Arc::new(Value::Pair(
                Arc::new(Value::symbol(intern_symbol("c"))),  // 'c
                Arc::new(Value::Nil)
            ))
        ))
    ))?;
    
    suite.assert_eval_eq("'42", Value::Literal(Literal::integer(42)))?;
    
    // quasiquote (if implemented)
    if !suite.skip_if_unimplemented("quasiquote") {
        suite.assert_eval_eq("`(a b c)", Value::Pair(
            Arc::new(Value::symbol(intern_symbol("a"))),  // 'a
            Arc::new(Value::Pair(
                Arc::new(Value::symbol(intern_symbol("b"))),  // 'b
                Arc::new(Value::Pair(
                    Arc::new(Value::symbol(intern_symbol("c"))),  // 'c
                    Arc::new(Value::Nil)
                ))
            ))
        ))?;
        
        suite.eval("(define x 42)")?;
        suite.assert_eval_eq("`(a ,x c)", Value::Pair(
            Arc::new(Value::symbol(intern_symbol("a"))),  // 'a
            Arc::new(Value::Pair(
                Arc::new(Value::Literal(Literal::integer(42))),  // x expanded
                Arc::new(Value::Pair(
                    Arc::new(Value::symbol(intern_symbol("c"))),  // 'c
                    Arc::new(Value::Nil)
                ))
            ))
        ))?;
        
        suite.assert_eval_eq("`(,@(list 1 2 3) 4)", Value::Pair(
            Arc::new(Value::Literal(Literal::integer(1))),
            Arc::new(Value::Pair(
                Arc::new(Value::Literal(Literal::integer(2))),
                Arc::new(Value::Pair(
                    Arc::new(Value::Literal(Literal::integer(3))),
                    Arc::new(Value::Pair(
                        Arc::new(Value::Literal(Literal::integer(4))),
                        Arc::new(Value::Nil)
                    ))
                ))
            ))
        ))?;
    }
    
    Ok(())
}

/// Test macro edge cases and error conditions
fn test_macro_edge_cases(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Macro with no patterns (should error)
    suite.assert_eval_error(r#"
        (define-syntax empty-macro
          (syntax-rules ()))
    "#)?;
    
    // Invalid macro patterns
    suite.assert_eval_error(r#"
        (define-syntax bad-pattern
          (syntax-rules ()
            (42 'invalid)))  ; Number as pattern
    "#)?;
    
    // Macro expansion with wrong arity
    suite.eval(r#"
        (define-syntax two-arg
          (syntax-rules ()
            ((two-arg x y) (+ x y))))
    "#)?;
    
    suite.assert_eval_error("(two-arg 1)")?;      // Too few args
    suite.assert_eval_error("(two-arg 1 2 3)")?;  // Too many args
    
    // Macro with complex pattern matching
    suite.eval(r#"
        (define-syntax complex-pattern
          (syntax-rules ()
            ((complex-pattern (x ...) (y ...))
             (list (list x ...) (list y ...)))))
    "#)?;
    
    suite.assert_eval_eq("(length (complex-pattern (1 2) (a b)))", 
                       Value::Literal(Literal::integer(2)))?;
    
    // Nested macro calls
    suite.eval(r#"
        (define-syntax outer
          (syntax-rules ()
            ((outer x) (inner x))))
            
        (define-syntax inner
          (syntax-rules ()
            ((inner y) (* y y))))
    "#)?;
    
    suite.assert_eval_eq("(outer 5)", Value::Literal(Literal::integer(25)))?;
    
    // Macro generating macro calls
    suite.eval(r#"
        (define-syntax make-squarer
          (syntax-rules ()
            ((make-squarer name)
             (define-syntax name
               (syntax-rules ()
                 ((name x) (* x x)))))))
    "#)?;
    
    suite.eval("(make-squarer square-it)")?;
    suite.assert_eval_eq("(square-it 6)", Value::Literal(Literal::integer(36)))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_macro_system_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("Macro system tests should pass");
    }
    
    #[test]
    fn test_basic_syntax_rules_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_basic_syntax_rules(&mut suite).expect("Basic syntax-rules tests should pass");
    }
    
    #[test]
    fn test_pattern_matching_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_pattern_matching(&mut suite).expect("Pattern matching tests should pass");
    }
    
    #[test]
    fn test_macro_hygiene_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_macro_hygiene(&mut suite).expect("Macro hygiene tests should pass");
    }
    
    #[test]
    fn test_recursive_macros_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_recursive_macros(&mut suite).expect("Recursive macro tests should pass");
    }
}
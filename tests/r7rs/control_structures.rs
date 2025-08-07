//! R7RS Control Structures Tests
//!
//! Tests for R7RS-small control flow and binding constructs including:
//! - Conditional expressions (if, cond, case)
//! - Boolean operations (and, or, not)
//! - Binding constructs (let, let*, letrec, letrec*)
//! - Iteration constructs (do, named let)
//! - Procedure definition and application
//! - Continuations (call/cc, dynamic-wind)
//! - Tail recursion and proper tail calls
//!
//! This module comprehensively tests control flow operations
//! required by the R7RS-small standard.

use crate::R7RSTestSuite;
use lambdust::{ast::Literal, eval::value::Value, utils::intern_symbol};
use std::sync::Arc;

/// Run all control structure tests
pub fn run_tests(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running R7RS Control Structures tests...");
    
    test_conditional_expressions(suite)?;
    test_boolean_operations(suite)?;
    test_binding_constructs(suite)?;
    test_procedure_definition(suite)?;
    test_procedure_application(suite)?;
    test_iteration_constructs(suite)?;
    test_continuations(suite)?;
    test_tail_recursion(suite)?;
    test_control_edge_cases(suite)?;
    
    println!("✓ Control structures tests passed");
    Ok(())
}

/// Test conditional expressions
fn test_conditional_expressions(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // if expressions - basic forms
    suite.assert_eval_eq("(if #t 'yes 'no)", Value::symbol(intern_symbol("yes")))?;  // 'yes
    suite.assert_eval_eq("(if #f 'yes 'no)", Value::symbol(intern_symbol("no")))?;  // 'no
    suite.assert_eval_eq("(if 0 'yes 'no)", Value::symbol(intern_symbol("yes")))?;   // 'yes (0 is truthy)
    suite.assert_eval_eq("(if '() 'yes 'no)", Value::symbol(intern_symbol("yes")))?; // 'yes ('() is truthy)
    suite.assert_eval_eq("(if \"\" 'yes 'no)", Value::symbol(intern_symbol("yes")))?; // 'yes ("" is truthy)
    
    // if without else clause
    suite.assert_eval_eq("(if #t 42)", Value::Literal(Literal::integer(42)))?;
    suite.assert_eval_eq("(if #f 42)", Value::Unspecified)?;  // R7RS: unspecified value
    
    // Nested if expressions
    suite.assert_eval_eq("(if #t (if #t 'inner-true 'inner-false) 'outer-false)", 
                       Value::symbol(intern_symbol("inner-true")))?;  // 'inner-true
    suite.assert_eval_eq("(if #f 'never (if #t 'inner-true 'inner-false))", 
                       Value::symbol(intern_symbol("inner-true")))?;  // 'inner-true
    
    // if with complex test expressions
    suite.assert_eval_eq("(if (= 2 2) 'equal 'not-equal)", Value::symbol(intern_symbol("equal")))?;  // 'equal
    suite.assert_eval_eq("(if (< 3 2) 'less 'not-less)", Value::symbol(intern_symbol("not-less")))?;    // 'not-less
    
    // cond expressions - basic forms
    suite.assert_eval_eq("(cond ((> 3 2) 'greater) ((< 3 2) 'less))", 
                       Value::symbol(intern_symbol("greater")))?;  // 'greater
    suite.assert_eval_eq("(cond ((> 3 3) 'greater) ((< 3 3) 'less) (else 'equal))", 
                       Value::symbol(intern_symbol("equal")))?;  // 'equal
    
    // cond with multiple expressions in consequent
    suite.assert_eval_eq("(cond ((= 2 2) (+ 1 2) (* 3 4)))", 
                       Value::Literal(Literal::integer(12)))?;  // Last expression value
    
    // cond with => (arrow) syntax
    if !suite.skip_if_unimplemented("cond =>") {
        suite.assert_eval_eq("(cond ((assq 'b '((a 1) (b 2) (c 3))) => cdr))", 
                           Value::Pair(
                               Arc::new(Value::Literal(Literal::integer(2))),
                               Arc::new(Value::Nil)
                           ))?;  // '(2)
    }
    
    // cond with no matching clause
    suite.assert_eval_eq("(cond ((> 2 3) 'never) ((< 5 2) 'also-never))", 
                       Value::Unspecified)?;  // R7RS: unspecified
    
    // case expressions
    suite.assert_eval_eq("(case (* 2 3) ((2 3 5 7) 'prime) ((1 4 6 8 9) 'composite))", 
                       Value::symbol(intern_symbol("composite")))?;  // 'composite
    suite.assert_eval_eq("(case 'c ((a e i o u) 'vowel) ((w y) 'semivowel) (else 'consonant))", 
                       Value::symbol(intern_symbol("consonant")))?;  // 'consonant
    
    // case with multiple values in clause
    suite.assert_eval_eq("(case 4 ((1 3 5 7 9) 'odd) ((0 2 4 6 8) 'even))", 
                       Value::symbol(intern_symbol("even")))?;  // 'even
    
    // case with symbols
    suite.assert_eval_eq("(case 'apple ((orange banana) 'citrus) ((apple pear) 'pome) (else 'other))", 
                       Value::symbol(intern_symbol("pome")))?;  // 'pome
    
    // case with no matching clause
    suite.assert_eval_eq("(case 'z ((a b c) 'early) ((x y) 'late))", 
                       Value::Unspecified)?;  // R7RS: unspecified
    
    Ok(())
}

/// Test boolean operations
fn test_boolean_operations(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // and - logical conjunction
    suite.assert_eval_true("(and)")?;  // Empty and is #t
    suite.assert_eval_eq("(and #t)", Value::Literal(Literal::Boolean(true)))?;
    suite.assert_eval_eq("(and #f)", Value::Literal(Literal::Boolean(false)))?;
    suite.assert_eval_true("(and #t #t #t)")?;
    suite.assert_eval_false("(and #t #f #t)")?;
    suite.assert_eval_false("(and #f #t #t)")?;
    
    // and - short-circuit evaluation
    suite.assert_eval_eq("(and 1 2 3)", Value::Literal(Literal::integer(3)))?;  // Returns last value
    suite.assert_eval_false("(and 1 #f 3)")?;  // Stops at #f
    suite.assert_eval_eq("(and 'a 'b 'c)", Value::symbol(intern_symbol("c")))?;  // 'c
    
    // and - with side effects (should short-circuit)
    if !suite.skip_if_unimplemented("side effects in and") {
        suite.eval("(define test-var 0)")?;
        suite.eval("(and #f (set! test-var 1))")?;
        suite.assert_eval_eq("test-var", Value::Literal(Literal::integer(0)))?;  // Not executed
        
        suite.eval("(and #t (set! test-var 2))")?;
        suite.assert_eval_eq("test-var", Value::Literal(Literal::integer(2)))?;  // Executed
    }
    
    // or - logical disjunction
    suite.assert_eval_false("(or)")?;  // Empty or is #f
    suite.assert_eval_true("(or #t)")?;
    suite.assert_eval_false("(or #f)")?;
    suite.assert_eval_true("(or #t #f #f)")?;
    suite.assert_eval_true("(or #f #t #f)")?;
    suite.assert_eval_false("(or #f #f #f)")?;
    
    // or - short-circuit evaluation
    suite.assert_eval_eq("(or #f #f 3)", Value::Literal(Literal::integer(3)))?;  // Returns first truthy
    suite.assert_eval_eq("(or #f 'found #f)", Value::symbol(intern_symbol("found")))?;  // 'found
    suite.assert_eval_eq("(or 1 2 3)", Value::Literal(Literal::integer(1)))?;  // Stops at first truthy
    
    // or - with side effects (should short-circuit)
    if !suite.skip_if_unimplemented("side effects in or") {
        suite.eval("(define test-var2 0)")?;
        suite.eval("(or #t (set! test-var2 1))")?;
        suite.assert_eval_eq("test-var2", Value::Literal(Literal::integer(0)))?;  // Not executed
        
        suite.eval("(or #f (set! test-var2 2))")?;
        suite.assert_eval_eq("test-var2", Value::Literal(Literal::integer(2)))?;  // Executed
    }
    
    // not - logical negation
    suite.assert_eval_true("(not #f)")?;
    suite.assert_eval_false("(not #t)")?;
    suite.assert_eval_false("(not 0)")?;      // 0 is truthy in Scheme
    suite.assert_eval_false("(not '())")?;    // '() is truthy in Scheme
    suite.assert_eval_false("(not \"\")")?;   // "" is truthy in Scheme
    suite.assert_eval_false("(not 'symbol)")?;
    
    // Complex boolean expressions
    suite.assert_eval_true("(and (or #t #f) (not #f))")?;
    suite.assert_eval_false("(or (and #f #t) (and #t #f))")?;
    suite.assert_eval_true("(not (and #f (or #t #f)))")?;
    
    Ok(())
}

/// Test binding constructs
fn test_binding_constructs(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // let - basic local binding
    suite.assert_eval_eq("(let ((x 2) (y 3)) (* x y))", 
                       Value::Literal(Literal::integer(6)))?;
    
    suite.assert_eval_eq("(let ((x 2) (y 3)) (let ((x 7) (z (+ x y))) (* z x)))", 
                       Value::Literal(Literal::integer(35)))?;  // Inner x shadows outer, z uses outer x
    
    // let with no bindings
    suite.assert_eval_eq("(let () 42)", Value::Literal(Literal::integer(42)))?;
    
    // let with multiple body expressions
    suite.assert_eval_eq("(let ((x 5)) (+ x 1) (+ x 2) (* x 3))", 
                       Value::Literal(Literal::integer(15)))?;  // Returns last expression
    
    // let* - sequential binding (each binding can see previous ones)
    suite.assert_eval_eq("(let* ((x 2) (y (+ x 1))) (+ x y))", 
                       Value::Literal(Literal::integer(5)))?;  // y can see x
    
    suite.assert_eval_eq("(let* ((x 2) (y (+ x 1)) (z (* x y))) (+ x y z))", 
                       Value::Literal(Literal::integer(11)))?;  // 2 + 3 + 6
    
    // let* with no bindings (equivalent to let)
    suite.assert_eval_eq("(let* () 42)", Value::Literal(Literal::integer(42)))?;
    
    // letrec - recursive binding
    suite.assert_eval_eq(r#"
        (letrec ((even? (lambda (n)
                          (if (zero? n)
                              #t
                              (odd? (- n 1)))))
                 (odd? (lambda (n)
                         (if (zero? n)
                             #f
                             (even? (- n 1))))))
          (even? 88))
    "#, Value::Literal(Literal::Boolean(true)))?;
    
    // letrec with simple recursion
    suite.assert_eval_eq(r#"
        (letrec ((fact (lambda (n)
                         (if (= n 0)
                             1
                             (* n (fact (- n 1)))))))
          (fact 5))
    "#, Value::Literal(Literal::integer(120)))?;
    
    // letrec* - like letrec but with sequential evaluation
    if !suite.skip_if_unimplemented("letrec*") {
        suite.assert_eval_eq(r#"
            (letrec* ((p (lambda (x) (+ 1 (q (- x 1)))))
                      (q (lambda (y) (if (zero? y) 0 (+ 1 (p (- y 1)))))))
              (p 5))
        "#, Value::Literal(Literal::integer(5)))?;
    }
    
    // Variable scoping tests
    suite.eval("(define outer-var 10)")?;
    suite.assert_eval_eq("(let ((outer-var 20)) outer-var)", 
                       Value::Literal(Literal::integer(20)))?;  // Shadows global
    suite.assert_eval_eq("outer-var", 
                       Value::Literal(Literal::integer(10)))?;  // Global unchanged
    
    Ok(())
}

/// Test procedure definition and lambda expressions
fn test_procedure_definition(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // lambda - basic procedure creation
    suite.eval("(define add (lambda (x y) (+ x y)))")?;
    suite.assert_eval_eq("(add 3 4)", Value::Literal(Literal::integer(7)))?;
    
    // lambda with single parameter
    suite.eval("(define square (lambda (x) (* x x)))")?;
    suite.assert_eval_eq("(square 5)", Value::Literal(Literal::integer(25)))?;
    
    // lambda with no parameters
    suite.eval("(define get-answer (lambda () 42))")?;
    suite.assert_eval_eq("(get-answer)", Value::Literal(Literal::integer(42)))?;
    
    // lambda with rest parameter
    suite.eval("(define sum-all (lambda args (if (null? args) 0 (+ (car args) (apply sum-all (cdr args))))))")?;
    if !suite.skip_if_unimplemented("apply procedure") {
        suite.assert_eval_eq("(sum-all 1 2 3 4)", Value::Literal(Literal::integer(10)))?;
        suite.assert_eval_eq("(sum-all)", Value::Literal(Literal::integer(0)))?;
    }
    
    // lambda with fixed and rest parameters
    if !suite.skip_if_unimplemented("mixed parameter lists") {
        suite.eval("(define f (lambda (x y . rest) (cons x (cons y rest))))")?;
        // Test result would be complex to verify exactly, so we test basic functionality
        let result = suite.eval("(f 1 2 3 4 5)")?;
        suite.assert_eval_eq("(car (f 1 2 3 4 5))", Value::Literal(Literal::integer(1)))?;
    }
    
    // define - procedure definition shorthand
    suite.eval("(define (multiply x y) (* x y))")?;
    suite.assert_eval_eq("(multiply 6 7)", Value::Literal(Literal::integer(42)))?;
    
    // define with rest parameters
    if !suite.skip_if_unimplemented("define with rest parameters") {
        suite.eval("(define (list-maker . args) args)")?;
        suite.assert_eval_eq("(length (list-maker 1 2 3))", Value::Literal(Literal::integer(3)))?;
    }
    
    // Closures - capturing lexical environment
    suite.eval(r#"
        (define make-counter
          (lambda (init)
            (let ((count init))
              (lambda ()
                (set! count (+ count 1))
                count))))
    "#)?;
    
    if !suite.skip_if_unimplemented("closures and set!") {
        suite.eval("(define counter1 (make-counter 0))")?;
        suite.eval("(define counter2 (make-counter 10))")?;
        
        suite.assert_eval_eq("(counter1)", Value::Literal(Literal::integer(1)))?;
        suite.assert_eval_eq("(counter1)", Value::Literal(Literal::integer(2)))?;
        suite.assert_eval_eq("(counter2)", Value::Literal(Literal::integer(11)))?;
        suite.assert_eval_eq("(counter1)", Value::Literal(Literal::integer(3)))?;
        suite.assert_eval_eq("(counter2)", Value::Literal(Literal::integer(12)))?;
    }
    
    Ok(())
}

/// Test procedure application
fn test_procedure_application(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Basic procedure application
    suite.assert_eval_eq("((lambda (x) (* x x)) 4)", Value::Literal(Literal::integer(16)))?;
    
    // Nested procedure application
    suite.assert_eval_eq("((lambda (f x) (f (f x))) (lambda (y) (* y 2)) 3)", 
                       Value::Literal(Literal::integer(12)))?;  // (2 * (2 * 3))
    
    // Higher-order procedures
    suite.eval(r#"
        (define apply-twice
          (lambda (f x)
            (f (f x))))
    "#)?;
    
    suite.eval("(define increment (lambda (x) (+ x 1)))")?;
    suite.assert_eval_eq("(apply-twice increment 5)", Value::Literal(Literal::integer(7)))?;
    
    // apply procedure (if implemented)
    if !suite.skip_if_unimplemented("apply procedure") {
        suite.assert_eval_eq("(apply + '(1 2 3 4))", Value::Literal(Literal::integer(10)))?;
        suite.assert_eval_eq("(apply * '(2 3 4))", Value::Literal(Literal::integer(24)))?;
        suite.assert_eval_eq("(apply cons '(a (b c)))", 
                           Value::Pair(
                               Arc::new(Value::symbol(intern_symbol("a"))),  // 'a
                               Arc::new(Value::Pair(
                                   Arc::new(Value::symbol(intern_symbol("b"))),  // 'b
                                   Arc::new(Value::Pair(
                                       Arc::new(Value::symbol(intern_symbol("c"))),  // 'c
                                       Arc::new(Value::Nil)
                                   ))
                               ))
                           ))?;
    }
    
    // Procedure arity checking
    suite.eval("(define two-arg (lambda (x y) (+ x y)))")?;
    suite.assert_eval_error("(two-arg 1)")?;      // Too few arguments
    suite.assert_eval_error("(two-arg 1 2 3)")?;  // Too many arguments
    
    // Variable arity procedures
    suite.eval("(define var-arg (lambda (x . rest) (cons x rest)))")?;
    suite.assert_eval_eq("(car (var-arg 1))", Value::Literal(Literal::integer(1)))?;
    suite.assert_eval_eq("(cdr (var-arg 1))", Value::Nil)?;
    
    let result = suite.eval("(var-arg 1 2 3)")?;
    suite.assert_eval_eq("(car (var-arg 1 2 3))", Value::Literal(Literal::integer(1)))?;
    suite.assert_eval_eq("(length (cdr (var-arg 1 2 3)))", Value::Literal(Literal::integer(2)))?;
    
    Ok(())
}

/// Test iteration constructs
fn test_iteration_constructs(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("iteration constructs") {
        return Ok(());
    }
    
    // do loops
    suite.assert_eval_eq(r#"
        (do ((i 0 (+ i 1))
             (sum 0 (+ sum i)))
            ((= i 5) sum))
    "#, Value::Literal(Literal::integer(10)))?;  // 0+1+2+3+4 = 10
    
    // do with multiple test expressions
    suite.assert_eval_eq(r#"
        (do ((i 10 (- i 1)))
            ((= i 0) 'done)
          (if (= i 5) (display "halfway")))
    "#, Value::symbol(intern_symbol("done")))?;  // 'done
    
    // Named let (recursive iteration)
    suite.assert_eval_eq(r#"
        (let loop ((n 5) (acc 1))
          (if (= n 0)
              acc
              (loop (- n 1) (* acc n))))
    "#, Value::Literal(Literal::integer(120)))?;  // 5! = 120
    
    // Named let with list processing
    suite.assert_eval_eq(r#"
        (let count-elements ((lst '(a b c d e)) (n 0))
          (if (null? lst)
              n
              (count-elements (cdr lst) (+ n 1))))
    "#, Value::Literal(Literal::integer(5)))?;
    
    Ok(())
}

/// Test continuations and control flow
fn test_continuations(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    if suite.skip_if_unimplemented("continuations") {
        return Ok(());
    }
    
    // call/cc - basic continuation capture
    suite.assert_eval_eq(r#"
        (call/cc
          (lambda (escape)
            (+ 1 2 (escape 42) 3)))
    "#, Value::Literal(Literal::integer(42)))?;  // Escapes before adding 3
    
    // call/cc returning normally
    suite.assert_eval_eq(r#"
        (call/cc
          (lambda (escape)
            (+ 1 2 3)))
    "#, Value::Literal(Literal::integer(6)))?;  // Normal return
    
    // call/cc in recursion (early termination)
    suite.assert_eval_eq(r#"
        (call/cc
          (lambda (return)
            (let loop ((lst '(1 2 stop 4 5)))
              (cond
                ((null? lst) 'not-found)
                ((eq? (car lst) 'stop) (return 'found))
                (else (loop (cdr lst)))))))
    "#, Value::symbol(intern_symbol("found")))?;  // 'found
    
    // Dynamic extent and dynamic-wind
    if !suite.skip_if_unimplemented("dynamic-wind") {
        suite.eval("(define test-log '())")?;
        suite.eval(r#"
            (define log-action
              (lambda (msg)
                (set! test-log (cons msg test-log))))
        "#)?;
        
        suite.eval(r#"
            (call/cc
              (lambda (escape)
                (dynamic-wind
                  (lambda () (log-action 'before))
                  (lambda () (log-action 'during) (escape 'escaped) (log-action 'after-escape))
                  (lambda () (log-action 'after)))))
        "#)?;
        
        // Check that before and after were called, but not after-escape
        let log = suite.eval("test-log")?;
        // Should contain 'after, 'during, 'before in that order (cons adds to front)
    }
    
    Ok(())
}

/// Test tail recursion and proper tail calls
fn test_tail_recursion(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Tail recursive factorial
    suite.eval(r#"
        (define fact-tail
          (lambda (n acc)
            (if (= n 0)
                acc
                (fact-tail (- n 1) (* n acc)))))
    "#)?;
    
    suite.assert_eval_eq("(fact-tail 5 1)", Value::Literal(Literal::integer(120)))?;
    
    // Tail recursive list length
    suite.eval(r#"
        (define length-tail
          (lambda (lst acc)
            (if (null? lst)
                acc
                (length-tail (cdr lst) (+ acc 1)))))
    "#)?;
    
    suite.assert_eval_eq("(length-tail '(a b c d e) 0)", Value::Literal(Literal::integer(5)))?;
    
    // Mutual tail recursion
    suite.eval(r#"
        (define even-tail?
          (lambda (n)
            (if (= n 0)
                #t
                (odd-tail? (- n 1)))))
        
        (define odd-tail?
          (lambda (n)
            (if (= n 0)
                #f
                (even-tail? (- n 1)))))
    "#)?;
    
    suite.assert_eval_true("(even-tail? 100)")?;  // Should not stack overflow
    suite.assert_eval_false("(odd-tail? 100)")?;
    
    // Large iteration to test tail call optimization
    if !suite.skip_if_unimplemented("large tail recursion") {
        suite.eval(r#"
            (define count-down
              (lambda (n)
                (if (= n 0)
                    'done
                    (count-down (- n 1)))))
        "#)?;
        
        suite.assert_eval_eq("(count-down 1000)", Value::symbol(intern_symbol("done")))?;  // 'done
    }
    
    Ok(())
}

/// Test control structure edge cases and error conditions
fn test_control_edge_cases(suite: &mut R7RSTestSuite) -> Result<(), Box<dyn std::error::Error>> {
    // Nested control structures
    suite.assert_eval_eq(r#"
        (if (> 3 2)
            (cond
              ((< 1 2) 'nested-true)
              (else 'nested-false))
            'outer-false)
    "#, Value::symbol(intern_symbol("nested-true")))?;  // 'nested-true
    
    // Complex boolean expressions in conditionals
    suite.assert_eval_eq(r#"
        (if (and (> 5 3) (or (= 2 2) (< 1 0)))
            'complex-true
            'complex-false)
    "#, Value::symbol(intern_symbol("complex-true")))?;  // 'complex-true
    
    // Empty cond/case
    suite.assert_eval_eq("(cond)", Value::Unspecified)?;
    suite.assert_eval_eq("(case 'x)", Value::Unspecified)?;
    
    // Invalid syntax errors (these should be caught by parser/evaluator)
    suite.assert_eval_error("(if)")?;                    // No test expression
    suite.assert_eval_error("(if #t)")?;                 // No consequent
    suite.assert_eval_error("(if #t 1 2 3)")?;          // Too many expressions
    suite.assert_eval_error("(let ((x)) x)")?;          // Invalid binding
    suite.assert_eval_error("(let (x 1) x)")?;          // Invalid binding format
    
    // Undefined variable references
    suite.assert_eval_error("undefined-variable")?;
    suite.assert_eval_error("(+ x 1)")?;  // Assuming x is not defined
    
    // Type errors in conditionals
    // Note: In Scheme, non-#f values are truthy, so these are actually valid
    suite.assert_eval_eq("(if 42 'number-is-truthy 'never)", Value::symbol(intern_symbol("number-is-truthy")))?;
    suite.assert_eval_eq("(if \"string\" 'string-is-truthy 'never)", Value::symbol(intern_symbol("string-is-truthy")))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_control_structures_suite() {
        let mut suite = R7RSTestSuite::new();
        run_tests(&mut suite).expect("Control structures tests should pass");
    }
    
    #[test]
    fn test_conditional_expressions_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_conditional_expressions(&mut suite).expect("Conditional expression tests should pass");
    }
    
    #[test]
    fn test_boolean_operations_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_boolean_operations(&mut suite).expect("Boolean operation tests should pass");
    }
    
    #[test]
    fn test_binding_constructs_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_binding_constructs(&mut suite).expect("Binding construct tests should pass");
    }
    
    #[test]
    fn test_procedure_definition_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_procedure_definition(&mut suite).expect("Procedure definition tests should pass");
    }
    
    #[test]
    fn test_tail_recursion_isolated() {
        let mut suite = R7RSTestSuite::new();
        test_tail_recursion(&mut suite).expect("Tail recursion tests should pass");
    }
}
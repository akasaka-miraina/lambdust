//! SRFI-26 (Notation for Specializing Parameters without Currying) Comprehensive Compliance Tests
//!
//! This module provides comprehensive tests for SRFI-26 compliance in Lambdust.
//! SRFI-26 provides cut and cute, syntactic sugar for creating specialized procedures
//! by fixing some arguments and leaving others as parameters.
//!
//! Key SRFI-26 Features:
//! - cut: Creates a procedure with some arguments specialized (evaluated once)  
//! - cute: Creates a procedure with some arguments specialized (evaluated each call)
//! - <> placeholder for unspecialized arguments
//! - <...> placeholder for rest arguments
//! - Integration with higher-order functions
//! - Partial application without explicit lambda
//!
//! Reference: https://srfi.schemers.org/srfi-26/srfi-26.html

use lambdust::eval::evaluator::Evaluator;
use lambdust::eval::value::{Value, ThreadSafeEnvironment};
use lambdust::stdlib::create_standard_environment;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper: Create evaluator with SRFI-26 loaded
    fn create_srfi26_evaluator() -> (Evaluator, Arc<ThreadSafeEnvironment>) {
        let env = create_standard_environment();
        let evaluator = Evaluator::new();
        
        // Load SRFI-26
        let import_code = r#"(import (srfi 26))"#;
        if let Ok(import_expr) = lambdust::parser::parse(import_code) {
            let _ = evaluator.eval(&import_expr, &env);
        }
        
        (evaluator, env)
    }

    /// Test helper: Evaluate expression and return result
    fn eval_expr(code: &str) -> Result<Value, String> {
        let (mut evaluator, env) = create_srfi26_evaluator();
        
        match lambdust::parser::parse(code) {
            Ok(expr) => {
                match evaluator.eval(&expr, &env) {
                    Ok(value) => Ok(value),
                    Err(error) => Err(format!("Evaluation error: {:?}", error)),
                }
            },
            Err(parse_error) => Err(format!("Parse error: {:?}", parse_error))
        }
    }

    /// Test helper: Evaluate and expect boolean result
    fn expect_boolean(code: &str, expected: bool) -> bool {
        match eval_expr(code) {
            Ok(Value::Literal(lambdust::ast::Literal::Boolean(b))) => b == expected,
            Ok(other) => {
                println!("Expected boolean {}, got: {:?}", expected, other);
                false
            },
            Err(e) => {
                println!("Error evaluating '{}': {}", code, e);
                false
            }
        }
    }

    /// Test helper: Evaluate and expect integer result
    fn expect_integer(code: &str, expected: i64) -> bool {
        match eval_expr(code) {
            Ok(Value::Literal(lambdust::ast::Literal::Number(n))) => n as i64 == expected,
            Ok(other) => {
                println!("Expected integer {}, got: {:?}", expected, other);
                false
            },
            Err(e) => {
                println!("Error evaluating '{}': {}", code, e);
                false
            }
        }
    }

    /// Test helper: Evaluate and expect string result
    fn expect_string(code: &str, expected: &str) -> bool {
        match eval_expr(code) {
            Ok(Value::Literal(lambdust::ast::Literal::String(s))) => s == expected,
            Ok(other) => {
                println!("Expected string '{}', got: {:?}", expected, other);
                false
            },
            Err(e) => {
                println!("Error evaluating '{}': {}", code, e);
                false
            }
        }
    }

    // ============= BASIC CUT TESTS =============

    #[test]
    fn test_basic_cut() {
        // Basic cut with single placeholder
        assert!(expect_integer(r#"((cut + 1 <>) 2)"#, 3));
        assert!(expect_integer(r#"((cut + <> 1) 2)"#, 3));
        assert!(expect_integer(r#"((cut * 3 <>) 4)"#, 12));
    }

    #[test]
    fn test_cut_multiple_placeholders() {
        // cut with multiple placeholders
        assert!(expect_integer(r#"((cut + <> <>) 5 7)"#, 12));
        assert!(expect_integer(r#"((cut * <> <>) 3 4)"#, 12));
        assert!(expect_integer(r#"((cut - <> <>) 10 3)"#, 7));
    }

    #[test]
    fn test_cut_mixed_fixed_and_placeholders() {
        // cut with mix of fixed arguments and placeholders
        assert!(expect_integer(r#"((cut + 10 <> 5) 2)"#, 17));
        assert!(expect_integer(r#"((cut * <> 3 <>) 2 4)"#, 24));  // (* 2 3 4)
    }

    #[test]
    fn test_cut_with_no_placeholders() {
        // cut with no placeholders (should create thunk)
        assert!(expect_integer(r#"((cut + 1 2))"#, 3));
        assert!(expect_integer(r#"((cut * 5 6))"#, 30));
    }

    #[test]
    fn test_cut_with_rest_arguments() {
        // cut with <...> for rest arguments
        assert!(expect_integer(r#"((cut + 10 <...>) 1 2 3 4)"#, 20));  // (+ 10 1 2 3 4)
        assert!(expect_integer(r#"((cut * 2 <...>) 3 4 5)"#, 120));    // (* 2 3 4 5)
    }

    #[test]
    fn test_cut_with_list_operations() {
        // cut with list operations
        assert!(expect_boolean(r#"
            (equal? ((cut cons 'a <>) '(b c d)) '(a b c d))
        "#, true));
        
        assert!(expect_boolean(r#"
            (equal? ((cut append <> '(3 4)) '(1 2)) '(1 2 3 4))
        "#, true));
        
        assert!(expect_integer(r#"((cut length <>) '(a b c d e))"#, 5));
    }

    // ============= BASIC CUTE TESTS =============

    #[test]
    fn test_basic_cute() {
        // Basic cute - like cut but re-evaluates specializations
        assert!(expect_integer(r#"((cute + 1 <>) 2)"#, 3));
        assert!(expect_integer(r#"((cute * <> 3) 4)"#, 12));
    }

    #[test]
    fn test_cute_vs_cut_evaluation_difference() {
        // Test the difference between cut and cute evaluation
        assert!(expect_boolean(r#"
            (let ((counter 0))
              (define (increment!) 
                (set! counter (+ counter 1))
                counter)
              
              ;; cut evaluates increment! once when creating the procedure
              (let ((cut-proc (cut + (increment!) <>)))
                (let ((result1 (cut-proc 10))
                      (result2 (cut-proc 20)))
                  ;; Both calls should use the same value from increment!
                  (and (= result1 11)    ; 1 + 10
                       (= result2 21)    ; 1 + 20
                       (= counter 1))))) ; increment! called only once
        "#, true));

        assert!(expect_boolean(r#"
            (let ((counter 0))
              (define (increment!) 
                (set! counter (+ counter 1))
                counter)
              
              ;; cute evaluates increment! each time the procedure is called
              (let ((cute-proc (cute + (increment!) <>)))
                (let ((result1 (cute-proc 10))
                      (result2 (cute-proc 20)))
                  ;; Each call should use a fresh value from increment!
                  (and (= result1 11)    ; 1 + 10
                       (= result2 22)    ; 2 + 20  
                       (= counter 2))))) ; increment! called twice
        "#, true));
    }

    #[test]
    fn test_cute_with_side_effects() {
        // Test cute with side-effecting expressions
        assert!(expect_boolean(r#"
            (let ((x 0))
              (let ((proc (cute + (begin (set! x (+ x 1)) x) <>)))
                (let ((r1 (proc 10))
                      (r2 (proc 20)))
                  (and (= r1 11)   ; 1 + 10
                       (= r2 22)   ; 2 + 20
                       (= x 2)))))
        "#, true));
    }

    // ============= ADVANCED CUT/CUTE TESTS =============

    #[test]
    fn test_cut_with_string_operations() {
        // cut with string operations
        assert!(expect_string(r#"((cut string-append "Hello, " <>) "World")"#, "Hello, World"));
        
        assert!(expect_boolean(r#"
            (equal? (map (cut string-append "prefix-" <>) '("a" "b" "c"))
                   '("prefix-a" "prefix-b" "prefix-c"))
        "#, true));
    }

    #[test]
    fn test_cut_with_comparison_operations() {
        // cut with comparison operations
        assert!(expect_boolean(r#"
            (let ((greater-than-5 (cut > <> 5)))
              (and (greater-than-5 10)
                   (not (greater-than-5 3))))
        "#, true));

        assert!(expect_boolean(r#"
            (equal? (filter (cut > <> 5) '(1 6 3 8 2 9 4 7))
                   '(6 8 9 7))
        "#, true));
    }

    #[test]
    fn test_cut_with_higher_order_functions() {
        // cut used with map, filter, fold, etc.
        assert!(expect_boolean(r#"
            (equal? (map (cut + 10 <>) '(1 2 3 4 5))
                   '(11 12 13 14 15))
        "#, true));

        assert!(expect_boolean(r#"
            (equal? (filter (cut > <> 0) '(-2 1 -3 4 -5 6))
                   '(1 4 6))
        "#, true));

        assert!(expect_integer(r#"
            (fold (cut + <> <>) 0 '(1 2 3 4 5))
        "#, 15));
    }

    #[test]
    fn test_cut_with_nested_procedures() {
        // cut with nested procedure calls
        assert!(expect_boolean(r#"
            (equal? (map (cut map (cut + 1 <>) <>) 
                        '((1 2) (3 4) (5 6)))
                   '((2 3) (4 5) (6 7)))
        "#, true));
    }

    // ============= COMPLEX PLACEHOLDER PATTERNS =============

    #[test]
    fn test_cut_complex_placeholder_patterns() {
        // Test various complex placeholder patterns
        assert!(expect_integer(r#"((cut + <> 5 <> 2) 1 3)"#, 11));  // (+ 1 5 3 2)
        
        assert!(expect_boolean(r#"
            (equal? ((cut list 'a <> 'b <> 'c) 1 2) '(a 1 b 2 c))
        "#, true));

        assert!(expect_integer(r#"((cut * 2 <> 3 <...>) 4 5 6)"#, 720));  // (* 2 4 3 5 6)
    }

    #[test]
    fn test_cut_with_only_rest_arguments() {
        // cut with only <...> placeholder
        assert!(expect_integer(r#"((cut + <...>) 1 2 3 4 5)"#, 15));
        assert!(expect_integer(r#"((cut * <...>) 2 3 4)"#, 24));
        
        assert!(expect_boolean(r#"
            (equal? ((cut list <...>) 'a 'b 'c 'd) '(a b c d))
        "#, true));
    }

    // ============= INTEGRATION WITH OTHER SRFIS =============

    #[test]
    fn test_cut_with_srfi1_operations() {
        // Assuming SRFI-1 is also available, test integration
        assert!(expect_boolean(r#"
            ;; Using cut with SRFI-1 list operations
            (let ((double (cut * 2 <>))
                  (positive? (cut > <> 0)))
              (equal? (map double (filter positive? '(-2 1 -3 4 -5 6)))
                     '(2 8 12)))
        "#, true));
    }

    #[test]
    fn test_cut_with_records() {
        // Test cut with record operations (assuming records are available)
        assert!(expect_boolean(r#"
            ;; Simulate record operations with list-based "records"
            (let ((make-person (lambda (name age) (list 'person name age)))
                  (person-name (lambda (p) (cadr p)))
                  (person-age (lambda (p) (caddr p))))
              
              (let ((get-name (cut person-name <>))
                    (make-adult (cut make-person <> 21)))
                
                (let ((alice (make-person "Alice" 25))
                      (bob (make-adult "Bob")))
                  (and (equal? (get-name alice) "Alice")
                       (equal? (person-age bob) 21)))))
        "#, true));
    }

    // ============= ERROR CONDITIONS AND EDGE CASES =============

    #[test]
    fn test_cut_procedure_creation() {
        // Test that cut/cute create proper procedures
        assert!(expect_boolean(r#"(procedure? (cut + <> 1))"#, true));
        assert!(expect_boolean(r#"(procedure? (cute * <> 2))"#, true));
        assert!(expect_boolean(r#"(procedure? (cut cons 'a <>))"#, true));
    }

    #[test]
    fn test_cut_arity_checking() {
        // Test that cut respects arity requirements
        // A cut with 2 placeholders should require 2 arguments
        
        // This should work
        assert!(expect_integer(r#"((cut + <> <>) 3 4)"#, 7));
        
        // These might error in a strict implementation, but we test successful cases
        assert!(expect_boolean(r#"(procedure? (cut + <> <>))"#, true));
    }

    #[test]
    fn test_cut_with_zero_arity_procedures() {
        // Test cut with procedures that take no arguments
        assert!(expect_boolean(r#"
            (let ((get-true (cut eq? #t #t)))
              (get-true))
        "#, true));
    }

    // ============= PRACTICAL USAGE EXAMPLES =============

    #[test]
    fn test_cut_for_event_handlers() {
        // Simulate event handler creation with cut
        assert!(expect_boolean(r#"
            (let ((events '())
                  (log-event (lambda (type data)
                              (set! events (cons (list type data) events)))))
              
              (let ((log-click (cut log-event 'click <>))
                    (log-keypress (cut log-event 'keypress <>)))
                
                (log-click "button1")
                (log-keypress "enter")
                
                (equal? events '((keypress "enter") (click "button1")))))
        "#, true));
    }

    #[test]
    fn test_cut_for_configuration() {
        // Use cut to create configured procedures
        assert!(expect_boolean(r#"
            (let ((format-currency (cut string-append "$" <> ".00"))
                  (format-percent (cut string-append <> "%")))
              
              (and (equal? (format-currency "42") "$42.00")
                   (equal? (format-percent "95") "95%")))
        "#, true));
    }

    #[test]
    fn test_cut_for_partial_application_chains() {
        // Chain multiple cuts together
        assert!(expect_integer(r#"
            (let* ((add-10 (cut + 10 <>))
                   (multiply-by-2 (cut * 2 <>))
                   (add-10-then-double (lambda (x) (multiply-by-2 (add-10 x)))))
              (add-10-then-double 5))  ; (5 + 10) * 2 = 30
        "#, 30));
    }

    #[test]
    fn test_cut_with_curry_like_behavior() {
        // Use cut to simulate currying
        assert!(expect_integer(r#"
            (let ((add (lambda (x y z) (+ x y z))))
              (let ((add-1 (cut add 1 <> <>))
                    (add-1-2 (cut add 1 2 <>)))
                (add-1-2 3)))  ; (+ 1 2 3) = 6
        "#, 6));
    }

    // ============= COMPREHENSIVE COMPLIANCE TESTS =============

    #[test]
    fn test_srfi26_comprehensive_compliance() {
        // Comprehensive test covering all major SRFI-26 features
        assert!(expect_boolean(r#"
            (let* (;; Basic cut functionality
                   (add-10 (cut + 10 <>))
                   (mult-by-3 (cut * <> 3))
                   (make-pair (cut cons <> <>))
                   
                   ;; Basic cute functionality
                   (counter 0)
                   (increment (lambda () (set! counter (+ counter 1)) counter))
                   (add-incremented (cute + (increment) <>))
                   
                   ;; Rest arguments
                   (sum-with-100 (cut + 100 <...>))
                   
                   ;; Complex patterns
                   (complex-op (cut + <> 5 <> 10 <...>)))
              
              (and ;; Test basic cut operations
                   (= (add-10 5) 15)
                   (= (mult-by-3 4) 12)
                   (equal? (make-pair 'a 'b) '(a . b))
                   
                   ;; Test cute vs cut evaluation
                   (let ((r1 (add-incremented 10))  ; counter becomes 1, result is 11
                         (r2 (add-incremented 20))) ; counter becomes 2, result is 22
                     (and (= r1 11) (= r2 22) (= counter 2)))
                   
                   ;; Test rest arguments
                   (= (sum-with-100 1 2 3 4) 110)  ; 100 + 1 + 2 + 3 + 4
                   
                   ;; Test complex patterns
                   (= (complex-op 1 2 3 4) 25)  ; 1 + 5 + 2 + 10 + 3 + 4
                   
                   ;; Test that results are procedures
                   (procedure? add-10)
                   (procedure? add-incremented)
                   (procedure? sum-with-100)))
        "#, true));
    }

    #[test]
    fn test_cut_cute_integration_example() {
        // Example showing practical use of cut and cute together
        assert!(expect_boolean(r#"
            (let ((database '())
                  (id-counter 0))
              
              ;; Helper functions  
              (define (get-next-id!)
                (set! id-counter (+ id-counter 1))
                id-counter)
              
              (define (add-record! record)
                (set! database (cons record database)))
              
              ;; Use cut for fixed operations
              (let ((make-user-record (cut list 'user <> <>))
                    (find-by-type (cut filter (lambda (r) (eq? (car r) <>)) database)))
                
                ;; Use cute for operations that need fresh values
                (let ((create-user (cute add-record! (make-user-record <> (get-next-id!)))))
                  
                  ;; Create some users
                  (create-user "Alice")
                  (create-user "Bob")
                  (create-user "Charlie")
                  
                  ;; Verify the results
                  (let ((users (find-by-type 'user)))
                    (and (= (length users) 3)
                         (= (length database) 3)
                         (= id-counter 3)
                         ;; Each user should have a unique ID
                         (equal? (map caddr users) '(3 2 1)))))))  ; IDs in reverse order due to cons
        "#, true));
    }
}
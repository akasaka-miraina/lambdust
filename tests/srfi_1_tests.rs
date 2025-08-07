//! SRFI-1 (List Library) Comprehensive Compliance Tests
//!
//! This module provides comprehensive tests for SRFI-1 compliance in Lambdust.
//! SRFI-1 is the fundamental list processing library for Scheme, providing
//! extensive functional programming tools for list manipulation.
//!
//! Key SRFI-1 Categories:
//! - Constructors: cons, list, xcons, cons*, make-list, list-tabulate, etc.
//! - Predicates: proper-list?, circular-list?, dotted-list?, null-list?, etc.
//! - Selectors: first...tenth, car+cdr, take, drop, split-at, etc.
//! - Fold/Unfold/Map: fold, fold-right, unfold, map variants, etc.
//! - Filtering: filter, partition, remove, etc.  
//! - Searching: find, any, every, list-index, etc.
//! - Deleting: delete, delete-duplicates, etc.
//! - Association lists: alist-cons, alist-delete, etc.
//! - Set operations: lset-union, lset-intersection, etc.
//!
//! Reference: https://srfi.schemers.org/srfi-1/srfi-1.html

use lambdust::eval::evaluator::Evaluator;
use lambdust::eval::value::{Value, ThreadSafeEnvironment};
use lambdust::stdlib::create_standard_environment;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper: Create evaluator with SRFI-1 loaded
    fn create_srfi1_evaluator() -> (Evaluator, Arc<ThreadSafeEnvironment>) {
        let env = create_standard_environment();
        let evaluator = Evaluator::new();
        
        // Load SRFI-1
        let import_code = r#"(import (srfi 1))"#;
        if let Ok(import_expr) = lambdust::parser::parse(import_code) {
            let _ = evaluator.eval(&import_expr, &env);
        }
        
        (evaluator, env)
    }

    /// Test helper: Evaluate expression and return result
    fn eval_expr(code: &str) -> Result<Value, String> {
        let (mut evaluator, env) = create_srfi1_evaluator();
        
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

    // ============= CONSTRUCTOR TESTS =============

    #[test]
    fn test_xcons() {
        assert!(expect_boolean(r#"(equal? (xcons 2 1) '(1 . 2))"#, true));
        assert!(expect_boolean(r#"(equal? (xcons 'b 'a) '(a . b))"#, true));
    }

    #[test]
    fn test_cons_star() {
        // Single argument
        assert!(expect_boolean(r#"(equal? (cons* 1) 1)"#, true));
        
        // Two arguments  
        assert!(expect_boolean(r#"(equal? (cons* 1 2) '(1 . 2))"#, true));
        
        // Multiple arguments
        assert!(expect_boolean(r#"(equal? (cons* 1 2 3 4) '(1 2 3 . 4))"#, true));
        assert!(expect_boolean(r#"(equal? (cons* 1 2 3 '(4 5)) '(1 2 3 4 5))"#, true));
    }

    #[test]
    fn test_make_list() {
        // Basic make-list
        assert!(expect_integer(r#"(length (make-list 5))"#, 5));
        
        // With fill value
        assert!(expect_boolean(r#"(every (lambda (x) (eq? x 'a)) (make-list 3 'a))"#, true));
        
        // Zero length
        assert!(expect_boolean(r#"(null? (make-list 0))"#, true));
    }

    #[test]
    fn test_list_tabulate() {
        assert!(expect_boolean(r#"(equal? (list-tabulate 4 values) '(0 1 2 3))"#, true));
        assert!(expect_boolean(r#"(equal? (list-tabulate 3 (lambda (i) (* i i))) '(0 1 4))"#, true));
        assert!(expect_boolean(r#"(null? (list-tabulate 0 values))"#, true));
    }

    #[test]
    fn test_list_copy() {
        assert!(expect_boolean(r#"(equal? (list-copy '(1 2 3)) '(1 2 3))"#, true));
        assert!(expect_boolean(r#"(null? (list-copy '()))"#, true));
        
        // Test that it's actually a copy (not eq)
        assert!(expect_boolean(r#"
            (let ((orig '(1 2 3)))
              (let ((copy (list-copy orig)))
                (and (equal? orig copy)
                     (not (eq? orig copy)))))
        "#, true));
    }

    #[test]
    fn test_circular_list() {
        assert!(expect_boolean(r#"
            (let ((circ (circular-list 1 2 3)))
              (and (= (car circ) 1)
                   (= (cadr circ) 2)
                   (= (caddr circ) 3)
                   (= (car (cdddr circ)) 1)))  ; wraps around
        "#, true));
    }

    #[test]
    fn test_iota() {
        // Basic iota
        assert!(expect_boolean(r#"(equal? (iota 5) '(0 1 2 3 4))"#, true));
        
        // With start
        assert!(expect_boolean(r#"(equal? (iota 3 5) '(5 6 7))"#, true));
        
        // With start and step
        assert!(expect_boolean(r#"(equal? (iota 4 1 2) '(1 3 5 7))"#, true));
        
        // Zero count
        assert!(expect_boolean(r#"(null? (iota 0))"#, true));
    }

    // ============= PREDICATE TESTS =============

    #[test]
    fn test_proper_list() {
        assert!(expect_boolean(r#"(proper-list? '())"#, true));
        assert!(expect_boolean(r#"(proper-list? '(1 2 3))"#, true));
        assert!(expect_boolean(r#"(proper-list? '(1 . 2))"#, false));
        assert!(expect_boolean(r#"(proper-list? 42)"#, false));
        
        // Test with circular list
        assert!(expect_boolean(r#"
            (let ((circ (circular-list 1 2 3)))
              (not (proper-list? circ)))
        "#, true));
    }

    #[test]
    fn test_circular_list_predicate() {
        assert!(expect_boolean(r#"(circular-list? (circular-list 1 2 3))"#, true));
        assert!(expect_boolean(r#"(circular-list? '(1 2 3))"#, false));
        assert!(expect_boolean(r#"(circular-list? '())"#, false));
        assert!(expect_boolean(r#"(circular-list? '(1 . 2))"#, false));
    }

    #[test]
    fn test_dotted_list() {
        assert!(expect_boolean(r#"(dotted-list? '(1 . 2))"#, true));
        assert!(expect_boolean(r#"(dotted-list? '(1 2 . 3))"#, true));
        assert!(expect_boolean(r#"(dotted-list? '(1 2 3))"#, false));
        assert!(expect_boolean(r#"(dotted-list? '())"#, false));
        assert!(expect_boolean(r#"(dotted-list? (circular-list 1 2))"#, false));
    }

    #[test]
    fn test_null_list() {
        assert!(expect_boolean(r#"(null-list? '())"#, true));
        assert!(expect_boolean(r#"(null-list? '(1 2 3))"#, false));
        
        // Should raise error for non-list
        // Note: This test depends on error handling implementation
    }

    #[test]
    fn test_not_pair() {
        assert!(expect_boolean(r#"(not-pair? 42)"#, true));
        assert!(expect_boolean(r#"(not-pair? '())"#, true));
        assert!(expect_boolean(r#"(not-pair? '(1 . 2))"#, false));
        assert!(expect_boolean(r#"(not-pair? '(1 2 3))"#, false));
    }

    #[test]
    fn test_list_equal() {
        // Basic equality
        assert!(expect_boolean(r#"(list= equal? '(1 2 3) '(1 2 3))"#, true));
        assert!(expect_boolean(r#"(list= equal? '(1 2 3) '(1 2 4))"#, false));
        
        // Custom equality predicate
        assert!(expect_boolean(r#"(list= = '(1 2 3) '(1 2 3))"#, true));
        
        // Multiple lists
        assert!(expect_boolean(r#"(list= equal? '(1 2) '(1 2) '(1 2))"#, true));
        assert!(expect_boolean(r#"(list= equal? '(1 2) '(1 2) '(1 3))"#, false));
        
        // Empty lists
        assert!(expect_boolean(r#"(list= equal?)"#, true));
        assert!(expect_boolean(r#"(list= equal? '())"#, true));
    }

    // ============= SELECTOR TESTS =============

    #[test]
    fn test_numbered_selectors() {
        let test_list = "'(1 2 3 4 5 6 7 8 9 10 11)";
        
        assert!(expect_integer(&format!("(first {})", test_list), 1));
        assert!(expect_integer(&format!("(second {})", test_list), 2));
        assert!(expect_integer(&format!("(third {})", test_list), 3));
        assert!(expect_integer(&format!("(fourth {})", test_list), 4));
        assert!(expect_integer(&format!("(fifth {})", test_list), 5));
        assert!(expect_integer(&format!("(sixth {})", test_list), 6));
        assert!(expect_integer(&format!("(seventh {})", test_list), 7));
        assert!(expect_integer(&format!("(eighth {})", test_list), 8));
        assert!(expect_integer(&format!("(ninth {})", test_list), 9));
        assert!(expect_integer(&format!("(tenth {})", test_list), 10));
    }

    #[test]
    fn test_car_plus_cdr() {
        assert!(expect_boolean(r#"
            (call-with-values 
              (lambda () (car+cdr '(1 2 3)))
              (lambda (a d) (and (= a 1) (equal? d '(2 3)))))
        "#, true));
    }

    #[test]
    fn test_take_drop() {
        // take
        assert!(expect_boolean(r#"(equal? (take '(1 2 3 4 5) 3) '(1 2 3))"#, true));
        assert!(expect_boolean(r#"(equal? (take '(1 2 3) 0) '())"#, true));
        
        // drop
        assert!(expect_boolean(r#"(equal? (drop '(1 2 3 4 5) 2) '(3 4 5))"#, true));
        assert!(expect_boolean(r#"(equal? (drop '(1 2 3) 3) '())"#, true));
    }

    #[test]
    fn test_take_drop_right() {
        // take-right
        assert!(expect_boolean(r#"(equal? (take-right '(1 2 3 4 5) 2) '(4 5))"#, true));
        
        // drop-right
        assert!(expect_boolean(r#"(equal? (drop-right '(1 2 3 4 5) 2) '(1 2 3))"#, true));
    }

    #[test]
    fn test_split_at() {
        assert!(expect_boolean(r#"
            (call-with-values
              (lambda () (split-at '(1 2 3 4 5) 2))
              (lambda (prefix suffix)
                (and (equal? prefix '(1 2))
                     (equal? suffix '(3 4 5)))))
        "#, true));
    }

    #[test]
    fn test_last_operations() {
        assert!(expect_integer(r#"(last '(1 2 3 4 5))"#, 5));
        assert!(expect_boolean(r#"(equal? (last-pair '(1 2 3)) '(3))"#, true));
    }

    // ============= MISCELLANEOUS TESTS =============

    #[test]
    fn test_length_plus() {
        assert!(expect_integer(r#"(length+ '(1 2 3 4))"#, 4));
        assert!(expect_boolean(r#"(not (length+ (circular-list 1 2 3)))"#, true));
        assert!(expect_boolean(r#"(not (length+ '(1 2 . 3)))"#, true));
    }

    // ============= FOLD AND MAP TESTS =============

    #[test]
    fn test_fold() {
        // Basic fold
        assert!(expect_integer(r#"(fold + 0 '(1 2 3 4))"#, 10));
        assert!(expect_boolean(r#"(equal? (fold cons '() '(1 2 3)) '(3 2 1))"#, true));
        
        // Multiple list fold
        assert!(expect_integer(r#"(fold + 0 '(1 2) '(3 4))"#, 10));
    }

    #[test]
    fn test_fold_right() {
        assert!(expect_integer(r#"(fold-right + 0 '(1 2 3 4))"#, 10));
        assert!(expect_boolean(r#"(equal? (fold-right cons '() '(1 2 3)) '(1 2 3))"#, true));
    }

    #[test]
    fn test_pair_fold() {
        assert!(expect_integer(r#"(pair-fold (lambda (pair acc) (+ 1 acc)) 0 '(a b c d))"#, 4));
    }

    #[test]
    fn test_reduce() {
        assert!(expect_integer(r#"(reduce + 0 '(1 2 3 4))"#, 10));
        assert!(expect_integer(r#"(reduce * 1 '())"#, 1));  // Identity case
    }

    #[test]
    fn test_unfold() {
        assert!(expect_boolean(r#"
            (equal? (unfold (lambda (x) (> x 5))
                           (lambda (x) x)
                           (lambda (x) (+ x 1))
                           1)
                   '(1 2 3 4 5))
        "#, true));
    }

    #[test]
    fn test_append_map() {
        assert!(expect_boolean(r#"
            (equal? (append-map (lambda (x) (list x (* x 2))) '(1 2 3))
                   '(1 2 2 4 3 6))
        "#, true));
    }

    #[test]
    fn test_filter_map() {
        assert!(expect_boolean(r#"
            (equal? (filter-map (lambda (x) (and (even? x) (* x 2))) '(1 2 3 4 5 6))
                   '(4 8 12))
        "#, true));
    }

    // ============= FILTERING TESTS =============

    #[test]
    fn test_filter() {
        assert!(expect_boolean(r#"(equal? (filter even? '(1 2 3 4 5 6)) '(2 4 6))"#, true));
        assert!(expect_boolean(r#"(equal? (filter (lambda (x) (> x 0)) '(-1 0 1 2)) '(1 2))"#, true));
    }

    #[test]
    fn test_partition() {
        assert!(expect_boolean(r#"
            (call-with-values
              (lambda () (partition even? '(1 2 3 4 5 6)))
              (lambda (evens odds)
                (and (equal? evens '(2 4 6))
                     (equal? odds '(1 3 5)))))
        "#, true));
    }

    #[test]
    fn test_remove() {
        assert!(expect_boolean(r#"(equal? (remove even? '(1 2 3 4 5 6)) '(1 3 5))"#, true));
    }

    // ============= SEARCHING TESTS =============

    #[test]
    fn test_find() {
        assert!(expect_integer(r#"(find even? '(1 3 5 4 7 8))"#, 4));
        assert!(expect_boolean(r#"(not (find even? '(1 3 5 7)))"#, true));
    }

    #[test]
    fn test_find_tail() {
        assert!(expect_boolean(r#"(equal? (find-tail even? '(1 3 5 4 7 8)) '(4 7 8))"#, true));
        assert!(expect_boolean(r#"(not (find-tail even? '(1 3 5 7)))"#, true));
    }

    #[test]
    fn test_any() {
        assert!(expect_boolean(r#"(any even? '(1 3 5 4 7))"#, true));
        assert!(expect_boolean(r#"(not (any even? '(1 3 5 7)))"#, true));
        assert!(expect_boolean(r#"(not (any even? '()))"#, true));
    }

    #[test]
    fn test_every() {
        assert!(expect_boolean(r#"(every even? '(2 4 6 8))"#, true));
        assert!(expect_boolean(r#"(not (every even? '(2 4 5 8)))"#, true));
        assert!(expect_boolean(r#"(every even? '())"#, true));  // Vacuous truth
    }

    #[test]
    fn test_list_index() {
        assert!(expect_integer(r#"(list-index even? '(1 3 5 4 7 8))"#, 3));
        assert!(expect_boolean(r#"(not (list-index even? '(1 3 5 7)))"#, true));
    }

    #[test]
    fn test_take_while_drop_while() {
        // take-while
        assert!(expect_boolean(r#"(equal? (take-while (lambda (x) (< x 5)) '(1 2 3 6 4 7)) '(1 2 3))"#, true));
        
        // drop-while
        assert!(expect_boolean(r#"(equal? (drop-while (lambda (x) (< x 5)) '(1 2 3 6 4 7)) '(6 4 7))"#, true));
    }

    #[test]
    fn test_span_break() {
        // span
        assert!(expect_boolean(r#"
            (call-with-values
              (lambda () (span (lambda (x) (< x 5)) '(1 2 3 6 4 7)))
              (lambda (prefix suffix)
                (and (equal? prefix '(1 2 3))
                     (equal? suffix '(6 4 7)))))
        "#, true));
        
        // break
        assert!(expect_boolean(r#"
            (call-with-values
              (lambda () (break (lambda (x) (>= x 5)) '(1 2 3 6 4 7)))
              (lambda (prefix suffix)
                (and (equal? prefix '(1 2 3))
                     (equal? suffix '(6 4 7)))))
        "#, true));
    }

    // ============= DELETING TESTS =============

    #[test]
    fn test_delete() {
        assert!(expect_boolean(r#"(equal? (delete 2 '(1 2 3 2 4 2 5)) '(1 3 4 5))"#, true));
        assert!(expect_boolean(r#"(equal? (delete 'x '(a b c) eq?) '(a b c))"#, true));
    }

    #[test]
    fn test_delete_duplicates() {
        assert!(expect_boolean(r#"(equal? (delete-duplicates '(1 2 3 2 4 1 5)) '(3 2 4 1 5))"#, true));
    }

    // ============= ASSOCIATION LIST TESTS =============

    #[test]
    fn test_alist_cons() {
        assert!(expect_boolean(r#"(equal? (alist-cons 'x 1 '((a . 2) (b . 3))) '((x . 1) (a . 2) (b . 3)))"#, true));
    }

    #[test]
    fn test_alist_copy() {
        assert!(expect_boolean(r#"
            (let ((orig '((a . 1) (b . 2))))
              (let ((copy (alist-copy orig)))
                (and (equal? orig copy)
                     (not (eq? orig copy)))))
        "#, true));
    }

    #[test]
    fn test_alist_delete() {
        assert!(expect_boolean(r#"(equal? (alist-delete 'b '((a . 1) (b . 2) (c . 3))) '((a . 1) (c . 3)))"#, true));
    }

    // ============= SET OPERATION TESTS =============

    #[test]
    fn test_lset_subset() {
        assert!(expect_boolean(r#"(lset<= equal? '(1 2) '(1 2 3 4))"#, true));
        assert!(expect_boolean(r#"(not (lset<= equal? '(1 2 5) '(1 2 3 4)))"#, true));
    }

    #[test]
    fn test_lset_equal() {
        assert!(expect_boolean(r#"(lset= equal? '(1 2 3) '(3 2 1))"#, true));
        assert!(expect_boolean(r#"(not (lset= equal? '(1 2 3) '(1 2 4)))"#, true));
    }

    #[test]
    fn test_lset_adjoin() {
        assert!(expect_boolean(r#"(equal? (lset-adjoin equal? '(1 2 3) 4 2 5) '(5 4 1 2 3))"#, true));
    }

    #[test]
    fn test_lset_union() {
        assert!(expect_boolean(r#"
            (let ((result (lset-union equal? '(1 2) '(2 3) '(3 4))))
              (and (= (length result) 4)
                   (every (lambda (x) (member x result)) '(1 2 3 4))))
        "#, true));
    }

    #[test]
    fn test_lset_intersection() {
        assert!(expect_boolean(r#"
            (let ((result (lset-intersection equal? '(1 2 3) '(2 3 4) '(3 4 5))))
              (equal? result '(3)))
        "#, true));
    }

    #[test]
    fn test_lset_difference() {
        assert!(expect_boolean(r#"
            (let ((result (lset-difference equal? '(1 2 3 4) '(2 4) '(1))))
              (equal? result '(3)))
        "#, true));
    }

    #[test]
    fn test_lset_xor() {
        assert!(expect_boolean(r#"
            (let ((result (lset-xor equal? '(1 2 3) '(2 3 4))))
              (and (member 1 result)
                   (member 4 result)
                   (not (member 2 result))
                   (not (member 3 result))))
        "#, true));
    }

    // ============= COMPREHENSIVE INTEGRATION TESTS =============

    #[test]
    fn test_srfi1_comprehensive_integration() {
        // Complex test combining multiple SRFI-1 features
        assert!(expect_boolean(r#"
            (let* ((numbers (iota 10 1))           ; '(1 2 3 4 5 6 7 8 9 10)
                   (evens (filter even? numbers))  ; '(2 4 6 8 10)
                   (squares (map (lambda (x) (* x x)) evens)) ; '(4 16 36 64 100)
                   (sum (fold + 0 squares))         ; 220
                   (pairs (map cons evens squares)) ; '((2 . 4) (4 . 16) ...)
                   (filtered-pairs (filter (lambda (p) (< (cdr p) 50)) pairs)))
              (and (= sum 220)
                   (= (length filtered-pairs) 3)
                   (every pair? pairs)
                   (proper-list? squares)))
        "#, true));
    }

    #[test]
    fn test_srfi1_circular_list_operations() {
        // Test operations with circular lists
        assert!(expect_boolean(r#"
            (let ((circ (circular-list 1 2 3)))
              (and (circular-list? circ)
                   (not (proper-list? circ))
                   (not (length+ circ))
                   (= (car circ) 1)
                   (= (cadr circ) 2)
                   (= (caddr circ) 3)
                   (= (car (cdddr circ)) 1)))  ; wraps around
        "#, true));
    }

    #[test]
    fn test_srfi1_performance_characteristics() {
        // Test that basic operations have reasonable performance
        assert!(expect_boolean(r#"
            (let ((large-list (iota 1000)))
              (and (= (length large-list) 1000)
                   (= (length (take large-list 500)) 500)
                   (= (length (drop large-list 500)) 500)
                   (= (fold + 0 (take large-list 100)) 4950)))  ; sum of 0..99
        "#, true));
    }

    #[test]
    fn test_srfi1_error_conditions() {
        // Test proper error handling for invalid operations
        // Note: These tests depend on the error handling implementation
        
        // take with negative argument should error
        // drop with negative argument should error  
        // list-ref with out-of-bounds index should error
        // etc.
        
        // For now, we'll test conditions that should work
        assert!(expect_boolean(r#"(null? (take '(1 2 3) 0))"#, true));
        assert!(expect_boolean(r#"(equal? (drop '(1 2 3) 0) '(1 2 3))"#, true));
    }
}
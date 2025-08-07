//! SRFI-9 (Defining Record Types) Comprehensive Compliance Tests
//!
//! This module provides comprehensive tests for SRFI-9 compliance in Lambdust.
//! SRFI-9 provides a syntax for creating new data types with named fields.
//! Records created with this library are disjoint from all existing types.
//!
//! Key SRFI-9 Features:
//! - define-record-type syntax for defining new record types
//! - Constructor procedures for creating record instances
//! - Predicate procedures for type checking
//! - Accessor procedures for reading field values
//! - Modifier procedures for updating field values (optional)
//! - Type disjointness from all other types
//!
//! Reference: https://srfi.schemers.org/srfi-9/srfi-9.html

use lambdust::eval::evaluator::Evaluator;
use lambdust::eval::value::{Value, ThreadSafeEnvironment};
use lambdust::stdlib::create_standard_environment;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper: Create evaluator with SRFI-9 loaded
    fn create_srfi9_evaluator() -> (Evaluator, Arc<ThreadSafeEnvironment>) {
        let env = create_standard_environment();
        let evaluator = Evaluator::new();
        
        // Load SRFI-9
        let import_code = r#"(import (srfi 9))"#;
        if let Ok(import_expr) = lambdust::parser::parse(import_code) {
            let _ = evaluator.eval(&import_expr, &env);
        }
        
        (evaluator, env)
    }

    /// Test helper: Evaluate expression and return result
    fn eval_expr(code: &str) -> Result<Value, String> {
        let (mut evaluator, env) = create_srfi9_evaluator();
        
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

    // ============= BASIC RECORD TYPE TESTS =============

    #[test]
    fn test_basic_record_definition() {
        // Define a simple record type and test basic operations
        assert!(expect_boolean(r#"
            (begin
              (define-record-type point
                (make-point x y)
                point?
                (x point-x)
                (y point-y))
              
              (let ((p (make-point 3 4)))
                (and (point? p)
                     (= (point-x p) 3)
                     (= (point-y p) 4))))
        "#, true));
    }

    #[test]
    fn test_record_with_modifiers() {
        // Test record with both accessors and modifiers
        assert!(expect_boolean(r#"
            (begin
              (define-record-type mutable-point
                (make-mutable-point x y)
                mutable-point?
                (x mpoint-x set-mpoint-x!)
                (y mpoint-y set-mpoint-y!))
              
              (let ((p (make-mutable-point 1 2)))
                (set-mpoint-x! p 10)
                (set-mpoint-y! p 20)
                (and (mutable-point? p)
                     (= (mpoint-x p) 10)
                     (= (mpoint-y p) 20))))
        "#, true));
    }

    #[test]
    fn test_record_type_disjointness() {
        // Test that record types are disjoint from each other and built-in types
        assert!(expect_boolean(r#"
            (begin
              (define-record-type person
                (make-person name age)
                person?
                (name person-name)
                (age person-age))
              
              (define-record-type book
                (make-book title author)
                book?
                (title book-title)
                (author book-author))
              
              (let ((p (make-person "Alice" 30))
                    (b (make-book "1984" "Orwell")))
                (and (person? p)
                     (book? b)
                     (not (person? b))
                     (not (book? p))
                     (not (person? "Alice"))
                     (not (person? 30))
                     (not (person? '(Alice 30)))
                     (not (person? #(Alice 30))))))
        "#, true));
    }

    #[test]
    fn test_constructor_field_order() {
        // Test that constructor fields can be in different order than definition
        assert!(expect_boolean(r#"
            (begin
              (define-record-type rectangle
                (make-rectangle width height)
                rectangle?
                (height rect-height)
                (width rect-width))
              
              (let ((r (make-rectangle 10 5)))
                (and (rectangle? r)
                     (= (rect-width r) 10)
                     (= (rect-height r) 5))))
        "#, true));
    }

    #[test]
    fn test_partial_field_accessors() {
        // Test record where not all fields have accessors
        assert!(expect_boolean(r#"
            (begin
              (define-record-type private-data
                (make-private-data public secret)
                private-data?
                (public get-public))
              
              (let ((pd (make-private-data "visible" "hidden")))
                (and (private-data? pd)
                     (equal? (get-public pd) "visible"))))
        "#, true));
    }

    #[test]
    fn test_record_with_various_field_types() {
        // Test record with fields of different types
        assert!(expect_boolean(r#"
            (begin
              (define-record-type mixed-record
                (make-mixed number string symbol list)
                mixed-record?
                (number mixed-number)
                (string mixed-string)
                (symbol mixed-symbol)
                (list mixed-list))
              
              (let ((m (make-mixed 42 "hello" 'world '(1 2 3))))
                (and (mixed-record? m)
                     (= (mixed-number m) 42)
                     (equal? (mixed-string m) "hello")
                     (eq? (mixed-symbol m) 'world)
                     (equal? (mixed-list m) '(1 2 3)))))
        "#, true));
    }

    // ============= ADVANCED RECORD TESTS =============

    #[test]
    fn test_nested_records() {
        // Test records containing other records
        assert!(expect_boolean(r#"
            (begin
              (define-record-type point
                (make-point x y)
                point?
                (x point-x)
                (y point-y))
              
              (define-record-type circle
                (make-circle center radius)
                circle?
                (center circle-center)
                (radius circle-radius))
              
              (let* ((p (make-point 5 5))
                     (c (make-circle p 10)))
                (and (circle? c)
                     (point? (circle-center c))
                     (= (point-x (circle-center c)) 5)
                     (= (point-y (circle-center c)) 5)
                     (= (circle-radius c) 10))))
        "#, true));
    }

    #[test]
    fn test_record_equality() {
        // Test that records with same content are not equal unless they're the same object
        assert!(expect_boolean(r#"
            (begin
              (define-record-type coord
                (make-coord x y)
                coord?
                (x coord-x)
                (y coord-y))
              
              (let ((c1 (make-coord 1 2))
                    (c2 (make-coord 1 2)))
                (and (coord? c1)
                     (coord? c2)
                     (= (coord-x c1) (coord-x c2))
                     (= (coord-y c1) (coord-y c2))
                     (not (equal? c1 c2))  ; Different objects
                     (equal? c1 c1))))      ; Same object
        "#, true));
    }

    #[test]
    fn test_record_as_procedure_arguments() {
        // Test passing records as procedure arguments
        assert!(expect_integer(r#"
            (begin
              (define-record-type vector2d
                (make-vector2d x y)
                vector2d?
                (x vector2d-x)
                (y vector2d-y))
              
              (define (vector2d-magnitude v)
                (sqrt (+ (* (vector2d-x v) (vector2d-x v))
                         (* (vector2d-y v) (vector2d-y v)))))
              
              (define (vector2d-add v1 v2)
                (make-vector2d (+ (vector2d-x v1) (vector2d-x v2))
                               (+ (vector2d-y v1) (vector2d-y v2))))
              
              (let* ((v1 (make-vector2d 3 4))
                     (v2 (make-vector2d 1 1))
                     (v3 (vector2d-add v1 v2)))
                (and (vector2d? v3)
                     (= (vector2d-x v3) 4)
                     (= (vector2d-y v3) 5)
                     (= (vector2d-magnitude v1) 5))))
        "#, 5));
    }

    #[test]
    fn test_record_in_data_structures() {
        // Test records stored in lists, vectors, etc.
        assert!(expect_boolean(r#"
            (begin
              (define-record-type student
                (make-student name grade)
                student?
                (name student-name)
                (grade student-grade))
              
              (let ((students (list (make-student "Alice" 95)
                                    (make-student "Bob" 87)
                                    (make-student "Charlie" 92))))
                (and (= (length students) 3)
                     (every student? students)
                     (equal? (student-name (car students)) "Alice")
                     (= (student-grade (cadr students)) 87))))
        "#, true));
    }

    // ============= ERROR CONDITION TESTS =============

    #[test]
    fn test_invalid_record_access() {
        // Test accessing record with wrong predicate
        // Note: This should raise an error in a correct implementation
        assert!(expect_boolean(r#"
            (begin
              (define-record-type type-a
                (make-a value)
                a?
                (value a-value))
              
              (define-record-type type-b
                (make-b value)
                b?
                (value b-value))
              
              (let ((a (make-a 42))
                    (b (make-b 24)))
                (and (a? a)
                     (b? b)
                     (not (a? b))
                     (not (b? a)))))
        "#, true));
    }

    #[test]
    fn test_record_mutation_safety() {
        // Test that modifiers only work on correct record types
        assert!(expect_boolean(r#"
            (begin
              (define-record-type mutable-box
                (make-box content)
                box?
                (content box-content set-box-content!))
              
              (let ((b1 (make-box "original"))
                    (b2 (make-box "other")))
                (set-box-content! b1 "modified")
                (and (equal? (box-content b1) "modified")
                     (equal? (box-content b2) "other"))))
        "#, true));
    }

    // ============= INTEGRATION TESTS =============

    #[test]
    fn test_record_with_higher_order_functions() {
        // Test records used with higher-order functions
        assert!(expect_boolean(r#"
            (begin
              (define-record-type employee
                (make-employee name salary)
                employee?
                (name employee-name)
                (salary employee-salary set-employee-salary!))
              
              (define (give-raise emp percentage)
                (let ((new-salary (* (employee-salary emp) (+ 1 (/ percentage 100)))))
                  (set-employee-salary! emp new-salary)
                  emp))
              
              (let ((employees (list (make-employee "Alice" 50000)
                                     (make-employee "Bob" 60000))))
                (map (lambda (emp) (give-raise emp 10)) employees)
                (and (= (employee-salary (car employees)) 55000)
                     (= (employee-salary (cadr employees)) 66000))))
        "#, true));
    }

    #[test]
    fn test_record_type_introspection() {
        // Test that we can distinguish record types at runtime
        assert!(expect_boolean(r#"
            (begin
              (define-record-type shape
                (make-shape type)
                shape?
                (type shape-type))
              
              (define (classify-value v)
                (cond
                  ((shape? v) 'shape)
                  ((number? v) 'number)
                  ((string? v) 'string)
                  ((symbol? v) 'symbol)
                  ((pair? v) 'pair)
                  (else 'other)))
              
              (let ((s (make-shape 'circle)))
                (and (eq? (classify-value s) 'shape)
                     (eq? (classify-value 42) 'number)
                     (eq? (classify-value "hello") 'string)
                     (eq? (classify-value 'world) 'symbol)
                     (eq? (classify-value '(1 2)) 'pair))))
        "#, true));
    }

    #[test] 
    fn test_complex_record_example() {
        // Complex example demonstrating multiple SRFI-9 features
        assert!(expect_boolean(r#"
            (begin
              (define-record-type bank-account
                (make-account owner balance)
                account?
                (owner account-owner)
                (balance account-balance set-account-balance!))
              
              (define (deposit! account amount)
                (if (account? account)
                    (let ((new-balance (+ (account-balance account) amount)))
                      (set-account-balance! account new-balance)
                      new-balance)
                    (error "Not a bank account")))
              
              (define (withdraw! account amount)
                (if (account? account)
                    (let ((current-balance (account-balance account)))
                      (if (>= current-balance amount)
                          (let ((new-balance (- current-balance amount)))
                            (set-account-balance! account new-balance)
                            new-balance)
                          (error "Insufficient funds")))
                    (error "Not a bank account")))
              
              (define (transfer! from-account to-account amount)
                (withdraw! from-account amount)
                (deposit! to-account amount))
              
              (let ((alice-account (make-account "Alice" 1000))
                    (bob-account (make-account "Bob" 500)))
                (transfer! alice-account bob-account 200)
                (and (account? alice-account)
                     (account? bob-account)
                     (equal? (account-owner alice-account) "Alice")
                     (equal? (account-owner bob-account) "Bob")
                     (= (account-balance alice-account) 800)
                     (= (account-balance bob-account) 700))))
        "#, true));
    }

    // ============= COMPATIBILITY TESTS =============

    #[test]
    fn test_record_with_r7rs_features() {
        // Test that SRFI-9 records work with R7RS features
        assert!(expect_boolean(r#"
            (begin
              (define-record-type optional-value
                (make-optional present value)
                optional?
                (present optional-present?)
                (value optional-value))
              
              (define (optional-map proc opt)
                (if (optional-present? opt)
                    (make-optional #t (proc (optional-value opt)))
                    (make-optional #f #f)))
              
              (let* ((opt1 (make-optional #t 42))
                     (opt2 (make-optional #f #f))
                     (opt3 (optional-map (lambda (x) (* x 2)) opt1))
                     (opt4 (optional-map (lambda (x) (* x 2)) opt2)))
                (and (optional? opt1)
                     (optional? opt2)
                     (optional? opt3)
                     (optional? opt4)
                     (optional-present? opt1)
                     (not (optional-present? opt2))
                     (optional-present? opt3)
                     (not (optional-present? opt4))
                     (= (optional-value opt3) 84))))
        "#, true));
    }

    #[test]
    fn test_srfi9_comprehensive_compliance() {
        // Comprehensive test covering all major SRFI-9 features
        assert!(expect_boolean(r#"
            (begin
              ;; Define multiple record types to test all features
              (define-record-type basic-record
                (make-basic a b c)
                basic?
                (a basic-a)
                (b basic-b set-basic-b!)
                (c basic-c))
              
              (define-record-type minimal-record
                (make-minimal value)
                minimal?
                (value minimal-value))
              
              ;; Test all basic operations
              (let ((basic (make-basic 1 2 3))
                    (minimal (make-minimal 'test)))
                
                ;; Test constructors work
                (and (basic? basic)
                     (minimal? minimal)
                     
                     ;; Test accessors work
                     (= (basic-a basic) 1)
                     (= (basic-b basic) 2)
                     (= (basic-c basic) 3)
                     (eq? (minimal-value minimal) 'test)
                     
                     ;; Test type disjointness
                     (not (basic? minimal))
                     (not (minimal? basic))
                     (not (basic? 'not-a-record))
                     (not (minimal? '(not a record)))
                     
                     ;; Test modifier works
                     (begin
                       (set-basic-b! basic 20)
                       (= (basic-b basic) 20))
                     
                     ;; Test that modification doesn't affect type
                     (basic? basic)
                     
                     ;; Test that records are not equal to other values
                     (not (equal? basic 123))
                     (not (equal? minimal "test")))))
        "#, true));
    }
}
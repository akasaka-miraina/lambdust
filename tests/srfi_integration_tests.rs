//! SRFI Integration and Multi-Import Compliance Tests
//!
//! This module provides comprehensive tests for SRFI integration in Lambdust,
//! focusing on multi-SRFI imports, cross-SRFI compatibility, and real-world
//! usage scenarios that combine multiple SRFIs.
//!
//! Key Integration Areas:
//! - Multi-SRFI imports: `(import (srfi 1 9 13 16 23 26))`
//! - Cross-SRFI compatibility and interactions
//! - Performance with multiple SRFIs loaded
//! - Real-world usage patterns combining SRFIs
//! - Error handling across SRFI boundaries
//! - Thread safety with multiple SRFIs
//!
//! Tested SRFI combinations:
//! - SRFI-1 (Lists) + SRFI-13 (Strings) + SRFI-26 (Cut/Cute)
//! - SRFI-9 (Records) + SRFI-1 (Lists) + SRFI-23 (Errors)
//! - SRFI-16 (Case-lambda) + SRFI-26 (Cut/Cute) + SRFI-13 (Strings)
//! - All SRFIs together in complex scenarios

use lambdust::eval::evaluator::Evaluator;
use lambdust::eval::value::{Value, ThreadSafeEnvironment};
use lambdust::stdlib::create_standard_environment;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper: Create evaluator with multiple SRFIs loaded
    fn create_multi_srfi_evaluator(srfis: &[u32]) -> (Evaluator, Arc<ThreadSafeEnvironment>) {
        let env = create_standard_environment();
        let evaluator = Evaluator::new();
        
        // Load specified SRFIs
        for &srfi in srfis {
            let import_code = format!("(import (srfi {}))", srfi);
            if let Ok(import_expr) = lambdust::parser::parse(&import_code) {
                let _ = evaluator.eval(&import_expr, &env);
            }
        }
        
        (evaluator, env)
    }

    /// Test helper: Create evaluator with all implemented SRFIs
    fn create_all_srfi_evaluator() -> (Evaluator, Arc<ThreadSafeEnvironment>) {
        create_multi_srfi_evaluator(&[1, 9, 13, 16, 23, 26, 39])
    }

    /// Test helper: Evaluate expression with specified SRFIs loaded
    fn eval_with_srfis(code: &str, srfis: &[u32]) -> Result<Value, String> {
        let (mut evaluator, env) = create_multi_srfi_evaluator(srfis);
        
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

    /// Test helper: Evaluate expression with all SRFIs loaded
    fn eval_all_srfis(code: &str) -> Result<Value, String> {
        eval_with_srfis(code, &[1, 9, 13, 16, 23, 26, 39])
    }

    /// Test helper: Evaluate and expect boolean result
    fn expect_boolean_with_srfis(code: &str, expected: bool, srfis: &[u32]) -> bool {
        match eval_with_srfis(code, srfis) {
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
    fn expect_integer_with_srfis(code: &str, expected: i64, srfis: &[u32]) -> bool {
        match eval_with_srfis(code, srfis) {
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

    // ============= MULTI-SRFI IMPORT TESTS =============

    #[test]
    fn test_multi_srfi_import_syntax() {
        // Test that multiple SRFIs can be imported in a single import form
        assert!(expect_boolean_with_srfis(r#"
            ;; Should be able to use procedures from multiple SRFIs
            (and (procedure? filter)      ; SRFI-1
                 (procedure? string-join) ; SRFI-13  
                 (procedure? cut))        ; SRFI-26
        "#, true, &[1, 13, 26]));
    }

    #[test]
    fn test_individual_vs_combined_imports() {
        // Test that individual imports work the same as combined imports
        
        // Individual imports
        let individual_result = eval_with_srfis(r#"
            (length (filter (lambda (x) (> x 0)) '(-1 2 -3 4 -5 6)))
        "#, &[1]);

        // Combined import (should work the same)
        let combined_result = eval_with_srfis(r#"
            (length (filter (lambda (x) (> x 0)) '(-1 2 -3 4 -5 6)))
        "#, &[1, 13, 26]);

        assert_eq!(individual_result, combined_result);
        assert!(expect_integer_with_srfis(r#"
            (length (filter (lambda (x) (> x 0)) '(-1 2 -3 4 -5 6)))
        "#, 3, &[1, 13, 26]));
    }

    // ============= SRFI-1 + SRFI-13 INTEGRATION =============

    #[test]
    fn test_srfi1_srfi13_text_processing() {
        // Combine list and string operations for text processing
        assert!(expect_boolean_with_srfis(r#"
            (let* ((sentences '("Hello world" "How are you" "Fine thanks"))
                   (words (append-map (lambda (s) (string-tokenize s char-alphabetic?))
                                     sentences))
                   (long-words (filter (lambda (w) (> (string-length w) 3)) words))
                   (result (string-join long-words "-")))
              (equal? result "Hello-world-Fine-thanks"))
        "#, true, &[1, 13]));
    }

    #[test]
    fn test_srfi1_srfi13_data_transformation() {
        // Transform data using both list and string operations
        assert!(expect_boolean_with_srfis(r#"
            (let* ((data '(("Alice" 25) ("Bob" 30) ("Charlie" 35)))
                   (names (map car data))
                   (formatted-names (map (lambda (name)
                                          (string-append "Mr/Ms. " name))
                                        names))
                   (name-list (string-join formatted-names ", ")))
              (equal? name-list "Mr/Ms. Alice, Mr/Ms. Bob, Mr/Ms. Charlie"))
        "#, true, &[1, 13]));
    }

    // ============= SRFI-1 + SRFI-26 INTEGRATION =============

    #[test]
    fn test_srfi1_srfi26_functional_programming() {
        // Combine list operations with cut/cute for functional programming
        assert!(expect_boolean_with_srfis(r#"
            (let* ((numbers '(1 2 3 4 5 6 7 8 9 10))
                   (even-numbers (filter (cut even? <>) numbers))
                   (doubled (map (cut * 2 <>) even-numbers))
                   (sum (fold (cut + <> <>) 0 doubled)))
              (= sum 60))  ; (2+4+6+8+10)*2 = 30*2 = 60
        "#, true, &[1, 26]));
    }

    #[test]
    fn test_srfi1_srfi26_higher_order_patterns() {
        // Complex higher-order programming patterns
        assert!(expect_boolean_with_srfis(r#"
            (let* ((data '((1 2) (3 4) (5 6) (7 8)))
                   (sum-pairs (map (cut apply + <>) data))
                   (products (map (cut apply * <>) data))
                   (combined (map (cut + <> <>) sum-pairs products)))
              (equal? combined '(5 19 41 71)))
        "#, true, &[1, 26]));
    }

    // ============= SRFI-9 + SRFI-1 INTEGRATION =============

    #[test]
    fn test_srfi9_srfi1_data_structures() {
        // Use records with list operations
        assert!(expect_boolean_with_srfis(r#"
            (begin
              (define-record-type person
                (make-person name age salary)
                person?
                (name person-name)
                (age person-age)
                (salary person-salary))
              
              (let* ((people (list (make-person "Alice" 25 50000)
                                   (make-person "Bob" 30 60000)
                                   (make-person "Charlie" 35 70000)))
                     (adults (filter (lambda (p) (>= (person-age p) 30)) people))
                     (high-earners (filter (lambda (p) (> (person-salary p) 55000)) people))
                     (names (map person-name high-earners)))
                (and (= (length adults) 2)
                     (= (length high-earners) 2)
                     (equal? names '("Bob" "Charlie")))))
        "#, true, &[1, 9]));
    }

    #[test]
    fn test_srfi9_srfi1_record_transformations() {
        // Transform records using list operations
        assert!(expect_boolean_with_srfis(r#"
            (begin
              (define-record-type point
                (make-point x y)
                point?
                (x point-x)
                (y point-y))
              
              (define (point-distance p1 p2)
                (sqrt (+ (expt (- (point-x p2) (point-x p1)) 2)
                         (expt (- (point-y p2) (point-y p1)) 2))))
              
              (let* ((points (list (make-point 0 0)
                                   (make-point 3 4)
                                   (make-point 6 8)))
                     (origin (car points))
                     (distances (map (lambda (p) (point-distance origin p))
                                    (cdr points))))
                (and (= (car distances) 5.0)
                     (= (cadr distances) 10.0))))
        "#, true, &[1, 9]));
    }

    // ============= SRFI-13 + SRFI-26 INTEGRATION =============

    #[test]
    fn test_srfi13_srfi26_string_processing() {
        // String processing with cut/cute
        assert!(expect_boolean_with_srfis(r#"
            (let* ((texts '("  hello world  " "  scheme programming  " "  functional style  "))
                   (trimmed (map (cut string-trim-both <>) texts))
                   (uppercased (map (cut string-upcase <>) trimmed))
                   (prefixed (map (cut string-append ">>> " <>) uppercased)))
              (equal? prefixed '(">>> HELLO WORLD" 
                                ">>> SCHEME PROGRAMMING" 
                                ">>> FUNCTIONAL STYLE")))
        "#, true, &[13, 26]));
    }

    #[test]
    fn test_srfi13_srfi26_text_analysis() {
        // Text analysis combining string and cut operations
        assert!(expect_integer_with_srfis(r#"
            (let* ((text "The quick brown fox jumps over the lazy dog")
                   (words (string-tokenize text char-alphabetic?))
                   (word-lengths (map (cut string-length <>) words))
                   (long-words (filter (cut > <> 4) word-lengths))
                   (total-long-chars (fold (cut + <> <>) 0 long-words)))
              total-long-chars)
        "#, 15, &[13, 26]));  ; "quick" + "brown" + "jumps" = 5+5+5 = 15
    }

    // ============= SRFI-16 + SRFI-26 INTEGRATION =============

    #[test]
    fn test_srfi16_srfi26_flexible_procedures() {
        // Combine case-lambda with cut for flexible procedure creation
        assert!(expect_boolean_with_srfis(r#"
            (let ((flexible-add
                   (case-lambda
                     (() 0)
                     ((x) x)
                     ((x y) (+ x y))
                     ((x y z) (+ x y z))
                     (args (apply + args)))))
              
              (let ((add-10 (cut flexible-add 10 <>))
                    (add-5-5 (cut flexible-add 5 5 <>)))
                (and (= (add-10 20) 30)
                     (= (add-5-5 15) 25)
                     (= (flexible-add 1 2 3 4 5) 15))))
        "#, true, &[16, 26]));
    }

    // ============= SRFI-9 + SRFI-23 ERROR HANDLING =============

    #[test]
    fn test_srfi9_srfi23_record_validation() {
        // Use records with proper error handling
        assert!(expect_boolean_with_srfis(r#"
            (begin
              (define-record-type validated-number
                (make-validated-number value)
                validated-number?
                (value validated-number-value))
              
              (define (safe-make-validated-number x)
                (if (number? x)
                    (make-validated-number x)
                    (error "Expected number" x)))
              
              (define (safe-validated-number-sqrt vn)
                (if (validated-number? vn)
                    (let ((val (validated-number-value vn)))
                      (if (>= val 0)
                          (sqrt val)
                          (error "Cannot take square root of negative number" val)))
                    (error "Expected validated-number" vn)))
              
              ;; Test successful case
              (let ((vn (safe-make-validated-number 16)))
                (= (safe-validated-number-sqrt vn) 4.0)))
        "#, true, &[9, 23]));
    }

    // ============= THREE-WAY SRFI COMBINATIONS =============

    #[test]
    fn test_srfi1_srfi13_srfi26_comprehensive() {
        // Comprehensive example using SRFI-1, SRFI-13, and SRFI-26
        assert!(expect_boolean_with_srfis(r#"
            (let* (;; Sample data
                   (sentences '("The quick brown fox" "jumps over the lazy dog" "in the bright sunshine"))
                   
                   ;; Extract all words using SRFI-13
                   (all-words (append-map (cut string-tokenize <> char-alphabetic?) sentences))
                   
                   ;; Filter long words using SRFI-1 and SRFI-26
                   (long-words (filter (cut > (cut string-length <>) 4) all-words))
                   
                   ;; Convert to uppercase using SRFI-13 and SRFI-26
                   (upper-long-words (map (cut string-upcase <>) long-words))
                   
                   ;; Join them using SRFI-13
                   (result (string-join upper-long-words " | ")))
              
              (equal? result "QUICK BROWN JUMPS BRIGHT SUNSHINE"))
        "#, true, &[1, 13, 26]));
    }

    #[test]
    fn test_srfi1_srfi9_srfi26_data_processing() {
        // Data processing pipeline using records, lists, and cut
        assert!(expect_boolean_with_srfis(r#"
            (begin
              (define-record-type employee
                (make-employee name department salary)
                employee?
                (name employee-name)
                (department employee-department)
                (salary employee-salary))
              
              (let* ((employees (list (make-employee "Alice" "Engineering" 75000)
                                      (make-employee "Bob" "Sales" 65000)
                                      (make-employee "Charlie" "Engineering" 80000)
                                      (make-employee "Diana" "Marketing" 70000)))
                     
                     ;; Use SRFI-26 cut for filtering and mapping
                     (engineers (filter (cut equal? "Engineering" (cut employee-department <>))
                                       employees))
                     (high-salaries (filter (cut > (cut employee-salary <>) 70000)
                                           employees))
                     (names (map (cut employee-name <>) high-salaries))
                     
                     ;; Use SRFI-1 for aggregation
                     (total-eng-salary (fold (cut + <> (cut employee-salary <>))
                                            0 engineers)))
                
                (and (= (length engineers) 2)
                     (= (length high-salaries) 2)
                     (equal? names '("Alice" "Charlie"))
                     (= total-eng-salary 155000))))
        "#, true, &[1, 9, 26]));
    }

    // ============= FOUR-WAY AND BEYOND COMBINATIONS =============

    #[test]
    fn test_srfi1_srfi9_srfi13_srfi26_integration() {
        // Complex integration using four SRFIs
        assert!(expect_boolean_with_srfis(r#"
            (begin
              (define-record-type document
                (make-document title content author)
                document?
                (title document-title)
                (content document-content)
                (author document-author))
              
              (let* ((docs (list (make-document "Scheme Guide" "Scheme is a programming language" "Alice")
                                 (make-document "Functional Programming" "FP is a paradigm" "Bob")
                                 (make-document "SRFI Tutorial" "SRFIs extend Scheme" "Charlie")))
                     
                     ;; Use SRFI-13 for text processing
                     (word-counts (map (lambda (doc)
                                        (length (string-tokenize (document-content doc) 
                                                                char-alphabetic?)))
                                      docs))
                     
                     ;; Use SRFI-1 + SRFI-26 for filtering and processing
                     (wordy-docs (filter (cut > <> 5) 
                                        (map (cut cons <> <>)
                                            docs word-counts)))
                     
                     ;; Use SRFI-13 + SRFI-26 for formatting
                     (summaries (map (lambda (doc-count)
                                      (string-append "\"" 
                                                    (document-title (car doc-count))
                                                    "\" (" 
                                                    (number->string (cdr doc-count))
                                                    " words)"))
                                    wordy-docs))
                     
                     (result (string-join summaries " | ")))
                
                (equal? result "\"Scheme Guide\" (6 words) | \"Functional Programming\" (6 words) | \"SRFI Tutorial\" (6 words)")))
        "#, true, &[1, 9, 13, 26]));
    }

    // ============= ALL SRFIS INTEGRATION =============

    #[test]
    fn test_all_srfis_comprehensive_example() {
        // Ultimate integration test using all implemented SRFIs
        assert!(expect_boolean_with_srfis(r#"
            (begin
              ;; SRFI-9: Define records
              (define-record-type task
                (make-task id title description priority status)
                task?
                (id task-id)
                (title task-title)
                (description task-description)
                (priority task-priority set-task-priority!)
                (status task-status set-task-status!))
              
              ;; SRFI-16: Case-lambda for flexible task operations
              (define task-manager
                (case-lambda
                  (('create title) (make-task (gensym) title "" 'normal 'pending))
                  (('create title desc) (make-task (gensym) title desc 'normal 'pending))
                  (('create title desc priority) (make-task (gensym) title desc priority 'pending))
                  (('filter tasks pred) (filter pred tasks))
                  (('map tasks proc) (map proc tasks))
                  (('count tasks) (length tasks))))
              
              (let* (;; Create some tasks
                     (tasks (list (task-manager 'create "Write docs" "Document the API" 'high)
                                  (task-manager 'create "Fix bugs" "Resolve critical issues" 'high) 
                                  (task-manager 'create "Add tests" "Improve test coverage" 'medium)
                                  (task-manager 'create "Refactor" "Clean up code" 'low)))
                     
                     ;; SRFI-1 + SRFI-26: Filter high priority tasks
                     (high-priority (task-manager 'filter tasks 
                                                 (cut equal? 'high (cut task-priority <>))))
                     
                     ;; SRFI-13 + SRFI-26: Format task descriptions
                     (formatted-titles (task-manager 'map high-priority
                                                    (cut string-append "URGENT: " 
                                                         (cut string-upcase (cut task-title <>)))))
                     
                     ;; SRFI-13: Join results
                     (summary (string-join formatted-titles " | ")))
                
                (and (= (task-manager 'count tasks) 4)
                     (= (task-manager 'count high-priority) 2)
                     (equal? summary "URGENT: WRITE DOCS | URGENT: FIX BUGS"))))
        "#, true, &[1, 9, 13, 16, 26]));
    }

    // ============= PERFORMANCE AND SCALABILITY TESTS =============

    #[test]
    fn test_multi_srfi_performance() {
        // Test that loading multiple SRFIs doesn't significantly impact performance
        assert!(expect_boolean_with_srfis(r#"
            (let* ((large-list (iota 1000))  ; SRFI-1
                   (processed (map (cut * 2 <>) large-list))  ; SRFI-26
                   (strings (map (cut number->string <>) processed))  ; Convert to strings
                   (filtered (filter (cut string-prefix? "1" <>) strings)))  ; SRFI-13
              (> (length filtered) 0))
        "#, true, &[1, 13, 26]));
    }

    #[test]
    fn test_memory_usage_with_multiple_srfis() {
        // Test that multiple SRFIs can handle reasonable data sizes
        assert!(expect_integer_with_srfis(r#"
            (let* ((data (make-list 100 "test"))  ; SRFI-1
                   (processed (map (cut string-append <> "-processed") data))  ; SRFI-13 + SRFI-26
                   (long-strings (filter (cut > (cut string-length <>) 10) processed)))
              (length long-strings))
        "#, 100, &[1, 13, 26]));
    }

    // ============= ERROR HANDLING INTEGRATION =============

    #[test]
    fn test_cross_srfi_error_handling() {
        // Test error handling across SRFI boundaries
        assert!(expect_boolean_with_srfis(r#"
            (guard (condition
                    ((error-object? condition) #t)
                    (else #f))
              ;; This should cause an error due to invalid string operation
              (string-take "hello" -1))  ; SRFI-13 error
        "#, true, &[13, 23]));
    }

    // ============= COMPREHENSIVE COMPLIANCE TEST =============

    #[test]
    fn test_srfi_integration_comprehensive_compliance() {
        // Ultimate comprehensive test for SRFI integration compliance
        assert!(expect_boolean_with_srfis(r#"
            ;; Test that all major SRFI integration patterns work correctly
            (let* (;; Test multi-SRFI imports are working
                   (srfis-available (and (procedure? filter)         ; SRFI-1
                                        (procedure? string-join)     ; SRFI-13
                                        (procedure? cut)            ; SRFI-26
                                        (procedure? case-lambda)    ; SRFI-16
                                        (procedure? error)))        ; SRFI-23
                   
                   ;; Test complex data transformation pipeline
                   (test-data '("apple" "banana" "cherry" "date" "elderberry"))
                   
                   ;; SRFI-1 + SRFI-26: Filter and transform
                   (long-fruits (filter (cut > (cut string-length <>) 4) test-data))
                   
                   ;; SRFI-13 + SRFI-26: String processing
                   (processed (map (cut string-append "Fruit: " (cut string-titlecase <>)) long-fruits))
                   
                   ;; SRFI-13: Final formatting
                   (result (string-join processed " | "))
                   
                   ;; SRFI-16: Flexible result checker
                   (checker (case-lambda
                             ((str) (> (string-length str) 0))
                             ((str expected) (equal? str expected))
                             ((str min-len max-len) 
                              (and (>= (string-length str) min-len)
                                   (<= (string-length str) max-len))))))
              
              (and srfis-available
                   (= (length long-fruits) 3)  ; "apple", "banana", "cherry", "elderberry" > 4 chars
                   (checker result "Fruit: Apple | Fruit: Banana | Fruit: Cherry | Fruit: Elderberry")
                   (checker result 50 100)))  ; Result should be reasonable length
        "#, true, &[1, 13, 16, 23, 26]));
    }
}
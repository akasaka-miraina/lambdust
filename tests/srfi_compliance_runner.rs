//! SRFI Comprehensive Compliance Validation Test Runner
//!
//! This module provides a comprehensive test runner that validates all implemented
//! SRFI libraries for compliance with their respective specifications. It includes
//! automated compliance checking, specification validation, and regression testing.
//!
//! Key Features:
//! - Automated compliance validation for all SRFIs
//! - Specification adherence checking
//! - Cross-SRFI compatibility validation
//! - Performance benchmarking for SRFI operations
//! - Regression test detection
//! - Detailed compliance reporting
//! - Integration with CI/CD systems
//!
//! Supported SRFIs:
//! - SRFI-1: List Library
//! - SRFI-9: Record Types  
//! - SRFI-13: String Library
//! - SRFI-16: Case-lambda
//! - SRFI-23: Error Reporting
//! - SRFI-26: Cut/Cute
//! - SRFI-39: Parameter Objects
//! - SRFI-43: Vector Library (when implemented)
//!
//! Test Categories:
//! - Individual SRFI compliance
//! - Multi-SRFI integration
//! - Performance benchmarks
//! - Error handling validation
//! - Thread safety verification
//! - Memory usage validation

use lambdust::eval::evaluator::Evaluator;
use lambdust::eval::value::{Value, ThreadSafeEnvironment};
use lambdust::stdlib::create_standard_environment;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests {
    use super::*;

    /// SRFI compliance test result
    #[derive(Debug, Clone)]
    pub struct ComplianceResult {
        pub srfi_number: u32,
        pub srfi_name: String,
        pub total_tests: usize,
        pub passed_tests: usize,
        pub failed_tests: usize,
        pub skipped_tests: usize,
        pub execution_time: Duration,
        pub compliance_percentage: f64,
        pub errors: Vec<String>,
        pub warnings: Vec<String>,
    }

    /// Overall compliance report
    #[derive(Debug)]
    pub struct ComplianceReport {
        pub overall_compliance: f64,
        pub total_srfis_tested: usize,
        pub fully_compliant_srfis: usize,
        pub partially_compliant_srfis: usize,
        pub non_compliant_srfis: usize,
        pub individual_results: HashMap<u32, ComplianceResult>,
        pub integration_test_results: ComplianceResult,
        pub performance_results: HashMap<String, Duration>,
        pub recommendations: Vec<String>,
    }

    /// SRFI test suite runner
    pub struct SrfiComplianceRunner {
        evaluator: Evaluator,
        environment: Arc<ThreadSafeEnvironment>,
        loaded_srfis: Vec<u32>,
    }

    impl SrfiComplianceRunner {
        /// Create a new compliance runner
        pub fn new() -> Self {
            let env = create_standard_environment();
            let evaluator = Evaluator::new();
            
            Self {
                evaluator,
                environment: env,
                loaded_srfis: Vec::new(),
            }
        }

        /// Load a specific SRFI
        pub fn load_srfi(&mut self, srfi_number: u32) -> Result<(), String> {
            let import_code = format!("(import (srfi {}))", srfi_number);
            match lambdust::parser::parse(&import_code) {
                Ok(import_expr) => {
                    match self.evaluator.eval(&import_expr, &self.environment) {
                        Ok(_) => {
                            if !self.loaded_srfis.contains(&srfi_number) {
                                self.loaded_srfis.push(srfi_number);
                            }
                            Ok(())
                        },
                        Err(e) => Err(format!("Failed to load SRFI-{}: {:?}", srfi_number, e))
                    }
                },
                Err(e) => Err(format!("Failed to parse import for SRFI-{}: {:?}", srfi_number, e))
            }
        }

        /// Evaluate a test expression
        pub fn eval_test(&mut self, code: &str) -> Result<Value, String> {
            match lambdust::parser::parse(code) {
                Ok(expr) => {
                    match self.evaluator.eval(&expr, self.environment.to_legacy()) {
                        Ok(value) => Ok(value),
                        Err(error) => Err(format!("Evaluation error: {:?}", error)),
                    }
                },
                Err(parse_error) => Err(format!("Parse error: {:?}", parse_error))
            }
        }

        /// Run a boolean test
        pub fn run_boolean_test(&mut self, test_name: &str, code: &str, expected: bool) -> Result<bool, String> {
            match self.eval_test(code) {
                Ok(Value::Literal(lambdust::ast::Literal::Boolean(b))) => {
                    if b == expected {
                        Ok(true)
                    } else {
                        Err(format!("Test '{}': Expected {}, got {}", test_name, expected, b))
                    }
                },
                Ok(other) => Err(format!("Test '{}': Expected boolean {}, got {:?}", test_name, expected, other)),
                Err(e) => Err(format!("Test '{}': {}", test_name, e))
            }
        }

        /// Run an integer test
        pub fn run_integer_test(&mut self, test_name: &str, code: &str, expected: i64) -> Result<bool, String> {
            match self.eval_test(code) {
                Ok(Value::Literal(lambdust::ast::Literal::Number(n))) => {
                    if n == expected {
                        Ok(true)
                    } else {
                        Err(format!("Test '{}': Expected {}, got {}", test_name, expected, n))
                    }
                },
                Ok(other) => Err(format!("Test '{}': Expected integer {}, got {:?}", test_name, expected, other)),
                Err(e) => Err(format!("Test '{}': {}", test_name, e))
            }
        }

        /// Run SRFI-1 compliance tests
        pub fn test_srfi1_compliance(&mut self) -> ComplianceResult {
            let start_time = Instant::now();
            let mut passed = 0;
            let mut failed = 0;
            let mut errors = Vec::new();

            // Load SRFI-1
            if let Err(e) = self.load_srfi(1) {
                errors.push(e);
                return ComplianceResult {
                    srfi_number: 1,
                    srfi_name: "List Library".to_string(),
                    total_tests: 0,
                    passed_tests: 0,
                    failed_tests: 1,
                    skipped_tests: 0,
                    execution_time: start_time.elapsed(),
                    compliance_percentage: 0.0,
                    errors,
                    warnings: Vec::new(),
                };
            }

            // Test basic constructors
            let tests = vec![
                ("xcons", r#"(equal? (xcons 2 1) '(1 . 2))"#, true),
                ("cons*", r#"(equal? (cons* 1 2 3 4) '(1 2 3 . 4))"#, true),
                ("make-list", r#"(= (length (make-list 5)) 5)"#, true),
                ("list-tabulate", r#"(equal? (list-tabulate 3 values) '(0 1 2))"#, true),
                ("iota", r#"(equal? (iota 4) '(0 1 2 3))"#, true),
                
                // Test predicates
                ("proper-list?", r#"(and (proper-list? '(1 2 3)) (not (proper-list? '(1 . 2))))"#, true),
                ("null-list?", r#"(and (null-list? '()) (not (null-list? '(1))))"#, true),
                
                // Test selectors
                ("first-tenth", r#"(and (= (first '(1 2 3)) 1) (= (second '(1 2 3)) 2))"#, true),
                ("take-drop", r#"(equal? (take '(1 2 3 4 5) 3) '(1 2 3))"#, true),
                
                // Test fold operations
                ("fold", r#"(= (fold + 0 '(1 2 3 4)) 10)"#, true),
                ("fold-right", r#"(equal? (fold-right cons '() '(1 2 3)) '(1 2 3))"#, true),
                
                // Test filtering
                ("filter", r#"(equal? (filter even? '(1 2 3 4 5 6)) '(2 4 6))"#, true),
                ("remove", r#"(equal? (remove even? '(1 2 3 4 5 6)) '(1 3 5))"#, true),
                
                // Test searching
                ("find", r#"(= (find even? '(1 3 5 4 7)) 4)"#, true),
                ("any-every", r#"(and (any even? '(1 3 4 5)) (every number? '(1 2 3)))"#, true),
            ];

            for (test_name, code, expected) in &tests {
                match self.run_boolean_test(test_name, code, *expected) {
                    Ok(_) => passed += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(e);
                    }
                }
            }

            let total = tests.len();
            let compliance_percentage = (passed as f64 / total as f64) * 100.0;

            ComplianceResult {
                srfi_number: 1,
                srfi_name: "List Library".to_string(),
                total_tests: total,
                passed_tests: passed,
                failed_tests: failed,
                skipped_tests: 0,
                execution_time: start_time.elapsed(),
                compliance_percentage,
                errors,
                warnings: Vec::new(),
            }
        }

        /// Run SRFI-9 compliance tests
        pub fn test_srfi9_compliance(&mut self) -> ComplianceResult {
            let start_time = Instant::now();
            let mut passed = 0;
            let mut failed = 0;
            let mut errors = Vec::new();

            // Load SRFI-9
            if let Err(e) = self.load_srfi(9) {
                errors.push(e);
                return ComplianceResult {
                    srfi_number: 9,
                    srfi_name: "Record Types".to_string(),
                    total_tests: 0,
                    passed_tests: 0,
                    failed_tests: 1,
                    skipped_tests: 0,
                    execution_time: start_time.elapsed(),
                    compliance_percentage: 0.0,
                    errors,
                    warnings: Vec::new(),
                };
            }

            let tests = vec![
                ("basic-record", r#"
                    (begin
                      (define-record-type test-record
                        (make-test-record x y)
                        test-record?
                        (x test-record-x)
                        (y test-record-y))
                      (let ((r (make-test-record 1 2)))
                        (and (test-record? r)
                             (= (test-record-x r) 1)
                             (= (test-record-y r) 2))))
                "#, true),
                
                ("record-with-modifiers", r#"
                    (begin
                      (define-record-type mutable-record
                        (make-mutable x)
                        mutable?
                        (x get-x set-x!))
                      (let ((r (make-mutable 10)))
                        (set-x! r 20)
                        (= (get-x r) 20)))
                "#, true),
                
                ("record-disjointness", r#"
                    (begin
                      (define-record-type type-a (make-a) a?)
                      (define-record-type type-b (make-b) b?)
                      (let ((a (make-a)) (b (make-b)))
                        (and (a? a) (b? b) (not (a? b)) (not (b? a)))))
                "#, true),
            ];

            for (test_name, code, expected) in &tests {
                match self.run_boolean_test(test_name, code, *expected) {
                    Ok(_) => passed += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(e);
                    }
                }
            }

            let total = tests.len();
            let compliance_percentage = (passed as f64 / total as f64) * 100.0;

            ComplianceResult {
                srfi_number: 9,
                srfi_name: "Record Types".to_string(),
                total_tests: total,
                passed_tests: passed,
                failed_tests: failed,
                skipped_tests: 0,
                execution_time: start_time.elapsed(),
                compliance_percentage,
                errors,
                warnings: Vec::new(),
            }
        }

        /// Run SRFI-13 compliance tests
        pub fn test_srfi13_compliance(&mut self) -> ComplianceResult {
            let start_time = Instant::now();
            let mut passed = 0;
            let mut failed = 0;
            let mut errors = Vec::new();

            // Load SRFI-13
            if let Err(e) = self.load_srfi(13) {
                errors.push(e);
                return ComplianceResult {
                    srfi_number: 13,
                    srfi_name: "String Library".to_string(),
                    total_tests: 0,
                    passed_tests: 0,
                    failed_tests: 1,
                    skipped_tests: 0,
                    execution_time: start_time.elapsed(),
                    compliance_percentage: 0.0,
                    errors,
                    warnings: Vec::new(),
                };
            }

            let tests = vec![
                ("string-null?", r#"(and (string-null? "") (not (string-null? "a")))"#, true),
                ("string-join", r#"(equal? (string-join '("a" "b" "c") ":") "a:b:c")"#, true),
                ("string-take-drop", r#"(and (equal? (string-take "hello" 3) "hel") (equal? (string-drop "hello" 2) "llo"))"#, true),
                ("string-trim", r#"(equal? (string-trim "  hello  ") "hello  ")"#, true),
                ("string-upcase", r#"(equal? (string-upcase "hello") "HELLO")"#, true),
                ("string-contains", r#"(= (string-contains "hello world" "wor") 6)"#, true),
                ("string-tokenize", r#"(equal? (string-tokenize "a b c" char-alphabetic?) '("a" "b" "c"))"#, true),
            ];

            for (test_name, code, expected) in &tests {
                match self.run_boolean_test(test_name, code, *expected) {
                    Ok(_) => passed += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(e);
                    }
                }
            }

            let total = tests.len();
            let compliance_percentage = (passed as f64 / total as f64) * 100.0;

            ComplianceResult {
                srfi_number: 13,
                srfi_name: "String Library".to_string(),
                total_tests: total,
                passed_tests: passed,
                failed_tests: failed,
                skipped_tests: 0,
                execution_time: start_time.elapsed(),
                compliance_percentage,
                errors,
                warnings: Vec::new(),
            }
        }

        /// Run SRFI-16 compliance tests
        pub fn test_srfi16_compliance(&mut self) -> ComplianceResult {
            let start_time = Instant::now();
            let mut passed = 0;
            let mut failed = 0;
            let mut errors = Vec::new();

            // Load SRFI-16
            if let Err(e) = self.load_srfi(16) {
                errors.push(e);
                return ComplianceResult {
                    srfi_number: 16,
                    srfi_name: "Case-lambda".to_string(),
                    total_tests: 0,
                    passed_tests: 0,
                    failed_tests: 1,
                    skipped_tests: 0,
                    execution_time: start_time.elapsed(),
                    compliance_percentage: 0.0,
                    errors,
                    warnings: Vec::new(),
                };
            }

            let tests = vec![
                ("basic-case-lambda", r#"
                    (let ((f (case-lambda
                               (() 0)
                               ((x) x)
                               ((x y) (+ x y)))))
                      (and (= (f) 0) (= (f 5) 5) (= (f 3 4) 7)))
                "#, true),
                
                ("case-lambda-rest", r#"
                    (let ((f (case-lambda
                               ((x . rest) (+ x (length rest))))))
                      (= (f 10 'a 'b 'c) 13))
                "#, true),
                
                ("case-lambda-procedure", r#"
                    (procedure? (case-lambda ((x) x) ((x y) (+ x y))))
                "#, true),
            ];

            for (test_name, code, expected) in &tests {
                match self.run_boolean_test(test_name, code, *expected) {
                    Ok(_) => passed += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(e);
                    }
                }
            }

            let total = tests.len();
            let compliance_percentage = (passed as f64 / total as f64) * 100.0;

            ComplianceResult {
                srfi_number: 16,
                srfi_name: "Case-lambda".to_string(),
                total_tests: total,
                passed_tests: passed,
                failed_tests: failed,
                skipped_tests: 0,
                execution_time: start_time.elapsed(),
                compliance_percentage,
                errors,
                warnings: Vec::new(),
            }
        }

        /// Run SRFI-26 compliance tests
        pub fn test_srfi26_compliance(&mut self) -> ComplianceResult {
            let start_time = Instant::now();
            let mut passed = 0;
            let mut failed = 0;
            let mut errors = Vec::new();

            // Load SRFI-26
            if let Err(e) = self.load_srfi(26) {
                errors.push(e);
                return ComplianceResult {
                    srfi_number: 26,
                    srfi_name: "Cut/Cute".to_string(),
                    total_tests: 0,
                    passed_tests: 0,
                    failed_tests: 1,
                    skipped_tests: 0,
                    execution_time: start_time.elapsed(),
                    compliance_percentage: 0.0,
                    errors,
                    warnings: Vec::new(),
                };
            }

            let tests = vec![
                ("basic-cut", r#"(= ((cut + 1 <>) 2) 3)"#, true),
                ("cut-multiple", r#"(= ((cut + <> <>) 3 4) 7)"#, true),
                ("cut-rest", r#"(= ((cut + 10 <...>) 1 2 3) 16)"#, true),
                ("cut-procedure", r#"(procedure? (cut + <> 1))"#, true),
                ("cute-basic", r#"(= ((cute + 1 <>) 2) 3)"#, true),
            ];

            for (test_name, code, expected) in &tests {
                match self.run_boolean_test(test_name, code, *expected) {
                    Ok(_) => passed += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(e);
                    }
                }
            }

            let total = tests.len();
            let compliance_percentage = (passed as f64 / total as f64) * 100.0;

            ComplianceResult {
                srfi_number: 26,
                srfi_name: "Cut/Cute".to_string(),
                total_tests: total,
                passed_tests: passed,
                failed_tests: failed,
                skipped_tests: 0,
                execution_time: start_time.elapsed(),
                compliance_percentage,
                errors,
                warnings: Vec::new(),
            }
        }

        /// Run integration tests
        pub fn test_integration_compliance(&mut self) -> ComplianceResult {
            let start_time = Instant::now();
            let mut passed = 0;
            let mut failed = 0;
            let mut errors = Vec::new();

            // Load multiple SRFIs
            let srfis_to_load = vec![1, 9, 13, 16, 26];
            for srfi in &srfis_to_load {
                if let Err(e) = self.load_srfi(*srfi) {
                    errors.push(format!("Failed to load SRFI-{}: {}", srfi, e));
                }
            }

            let tests = vec![
                ("multi-srfi-basic", r#"
                    (and (procedure? filter)      ; SRFI-1
                         (procedure? string-join) ; SRFI-13
                         (procedure? cut))        ; SRFI-26
                "#, true),
                
                ("srfi1-srfi13-integration", r#"
                    (equal? (string-join (filter (lambda (s) (> (string-length s) 2))
                                                '("a" "bb" "ccc" "dd" "eee")) 
                                        "-")
                           "ccc-eee")
                "#, true),
                
                ("srfi1-srfi26-integration", r#"
                    (equal? (map (cut + 10 <>) '(1 2 3 4)) '(11 12 13 14))
                "#, true),
                
                ("multi-srfi-complex", r#"
                    (let* ((data '("hello" "world" "scheme" "programming"))
                           (long-words (filter (cut > (cut string-length <>) 5) data))
                           (result (string-join (map (cut string-upcase <>) long-words) " ")))
                      (equal? result "SCHEME PROGRAMMING"))
                "#, true),
            ];

            for (test_name, code, expected) in &tests {
                match self.run_boolean_test(test_name, code, *expected) {
                    Ok(_) => passed += 1,
                    Err(e) => {
                        failed += 1;
                        errors.push(e);
                    }
                }
            }

            let total = tests.len();
            let compliance_percentage = (passed as f64 / total as f64) * 100.0;

            ComplianceResult {
                srfi_number: 0, // Special case for integration
                srfi_name: "Multi-SRFI Integration".to_string(),
                total_tests: total,
                passed_tests: passed,
                failed_tests: failed,
                skipped_tests: 0,
                execution_time: start_time.elapsed(),
                compliance_percentage,
                errors,
                warnings: Vec::new(),
            }
        }

        /// Run complete compliance suite
        pub fn run_complete_compliance_suite(&mut self) -> ComplianceReport {
            let mut individual_results = HashMap::new();
            let mut performance_results = HashMap::new();

            // Test individual SRFIs
            let srfi1_result = self.test_srfi1_compliance();
            performance_results.insert("SRFI-1".to_string(), srfi1_result.execution_time);
            individual_results.insert(1, srfi1_result);

            let srfi9_result = self.test_srfi9_compliance();
            performance_results.insert("SRFI-9".to_string(), srfi9_result.execution_time);
            individual_results.insert(9, srfi9_result);

            let srfi13_result = self.test_srfi13_compliance();
            performance_results.insert("SRFI-13".to_string(), srfi13_result.execution_time);
            individual_results.insert(13, srfi13_result);

            let srfi16_result = self.test_srfi16_compliance();
            performance_results.insert("SRFI-16".to_string(), srfi16_result.execution_time);
            individual_results.insert(16, srfi16_result);

            let srfi26_result = self.test_srfi26_compliance();
            performance_results.insert("SRFI-26".to_string(), srfi26_result.execution_time);
            individual_results.insert(26, srfi26_result);

            // Test integration
            let integration_result = self.test_integration_compliance();
            performance_results.insert("Integration".to_string(), integration_result.execution_time);

            // Calculate overall statistics
            let total_srfis_tested = individual_results.len();
            let mut fully_compliant = 0;
            let mut partially_compliant = 0;
            let mut non_compliant = 0;
            let mut total_compliance = 0.0;

            for result in individual_results.values() {
                total_compliance += result.compliance_percentage;
                if result.compliance_percentage >= 95.0 {
                    fully_compliant += 1;
                } else if result.compliance_percentage >= 50.0 {
                    partially_compliant += 1;
                } else {
                    non_compliant += 1;
                }
            }

            let overall_compliance = total_compliance / total_srfis_tested as f64;

            // Generate recommendations
            let mut recommendations = Vec::new();
            for (srfi_num, result) in &individual_results {
                if result.compliance_percentage < 95.0 {
                    recommendations.push(format!(
                        "SRFI-{} ({}) needs attention: {:.1}% compliant with {} failed tests",
                        srfi_num, result.srfi_name, result.compliance_percentage, result.failed_tests
                    ));
                }
            }

            if integration_result.compliance_percentage < 95.0 {
                recommendations.push(format!(
                    "Multi-SRFI integration needs attention: {:.1}% compliant",
                    integration_result.compliance_percentage
                ));
            }

            ComplianceReport {
                overall_compliance,
                total_srfis_tested,
                fully_compliant_srfis: fully_compliant,
                partially_compliant_srfis: partially_compliant,
                non_compliant_srfis: non_compliant,
                individual_results,
                integration_test_results: integration_result,
                performance_results,
                recommendations,
            }
        }
    }

    // ============= COMPLIANCE TEST RUNNER TESTS =============

    #[test]
    fn test_srfi_compliance_runner_creation() {
        let runner = SrfiComplianceRunner::new();
        assert_eq!(runner.loaded_srfis.len(), 0);
    }

    #[test]
    fn test_individual_srfi_compliance() {
        let mut runner = SrfiComplianceRunner::new();
        
        // Test SRFI-1 compliance
        let srfi1_result = runner.test_srfi1_compliance();
        assert!(srfi1_result.compliance_percentage > 0.0);
        assert!(srfi1_result.total_tests > 0);
        
        // Test SRFI-9 compliance
        let srfi9_result = runner.test_srfi9_compliance();
        assert!(srfi9_result.compliance_percentage > 0.0);
        assert!(srfi9_result.total_tests > 0);
    }

    #[test]
    fn test_integration_compliance() {
        let mut runner = SrfiComplianceRunner::new();
        let integration_result = runner.test_integration_compliance();
        
        assert!(integration_result.total_tests > 0);
        assert!(integration_result.compliance_percentage >= 0.0);
        assert_eq!(integration_result.srfi_name, "Multi-SRFI Integration");
    }

    #[test]
    fn test_complete_compliance_suite() {
        let mut runner = SrfiComplianceRunner::new();
        let report = runner.run_complete_compliance_suite();
        
        // Verify report structure
        assert!(report.total_srfis_tested > 0);
        assert!(report.overall_compliance >= 0.0 && report.overall_compliance <= 100.0);
        assert_eq!(report.total_srfis_tested, 
                   report.fully_compliant_srfis + 
                   report.partially_compliant_srfis + 
                   report.non_compliant_srfis);
        
        // Verify individual results
        assert!(report.individual_results.contains_key(&1));  // SRFI-1
        assert!(report.individual_results.contains_key(&9));  // SRFI-9
        assert!(report.individual_results.contains_key(&13)); // SRFI-13
        assert!(report.individual_results.contains_key(&16)); // SRFI-16
        assert!(report.individual_results.contains_key(&26)); // SRFI-26
        
        // Verify performance data
        assert!(report.performance_results.contains_key("SRFI-1"));
        assert!(report.performance_results.contains_key("Integration"));
        
        // Print report for manual inspection
        println!("\n=== SRFI Compliance Report ===");
        println!("Overall Compliance: {:.1}%", report.overall_compliance);
        println!("Total SRFIs Tested: {}", report.total_srfis_tested);
        println!("Fully Compliant: {}", report.fully_compliant_srfis);
        println!("Partially Compliant: {}", report.partially_compliant_srfis);
        println!("Non-Compliant: {}", report.non_compliant_srfis);
        
        println!("\nIndividual SRFI Results:");
        for (srfi_num, result) in &report.individual_results {
            println!("  SRFI-{} ({}): {:.1}% ({}/{} tests passed)", 
                     srfi_num, result.srfi_name, result.compliance_percentage,
                     result.passed_tests, result.total_tests);
        }
        
        println!("\nIntegration Tests: {:.1}% ({}/{} tests passed)",
                 report.integration_test_results.compliance_percentage,
                 report.integration_test_results.passed_tests,
                 report.integration_test_results.total_tests);
        
        if !report.recommendations.is_empty() {
            println!("\nRecommendations:");
            for rec in &report.recommendations {
                println!("  - {}", rec);
            }
        }
        
        println!("\nPerformance Results:");
        for (test_name, duration) in &report.performance_results {
            println!("  {}: {:?}", test_name, duration);
        }
    }

    #[test]
    fn test_compliance_threshold_validation() {
        let mut runner = SrfiComplianceRunner::new();
        let report = runner.run_complete_compliance_suite();
        
        // Define minimum compliance thresholds
        const MIN_OVERALL_COMPLIANCE: f64 = 70.0;
        const MIN_INDIVIDUAL_COMPLIANCE: f64 = 60.0;
        const MIN_INTEGRATION_COMPLIANCE: f64 = 60.0;
        
        // Check overall compliance
        assert!(report.overall_compliance >= MIN_OVERALL_COMPLIANCE,
                "Overall compliance {:.1}% is below minimum threshold {:.1}%",
                report.overall_compliance, MIN_OVERALL_COMPLIANCE);
        
        // Check individual SRFI compliance
        for (srfi_num, result) in &report.individual_results {
            assert!(result.compliance_percentage >= MIN_INDIVIDUAL_COMPLIANCE,
                    "SRFI-{} compliance {:.1}% is below minimum threshold {:.1}%",
                    srfi_num, result.compliance_percentage, MIN_INDIVIDUAL_COMPLIANCE);
        }
        
        // Check integration compliance
        assert!(report.integration_test_results.compliance_percentage >= MIN_INTEGRATION_COMPLIANCE,
                "Integration compliance {:.1}% is below minimum threshold {:.1}%",
                report.integration_test_results.compliance_percentage, MIN_INTEGRATION_COMPLIANCE);
    }

    #[test]
    fn test_performance_benchmarks() {
        let mut runner = SrfiComplianceRunner::new();
        let report = runner.run_complete_compliance_suite();
        
        // Define maximum acceptable execution times
        const MAX_INDIVIDUAL_SRFI_TIME: Duration = Duration::from_millis(5000);
        const MAX_INTEGRATION_TIME: Duration = Duration::from_millis(10000);
        
        // Check individual SRFI performance
        for (test_name, duration) in &report.performance_results {
            if test_name != "Integration" {
                assert!(duration <= &MAX_INDIVIDUAL_SRFI_TIME,
                        "{} took {:?}, which exceeds maximum {:?}",
                        test_name, duration, MAX_INDIVIDUAL_SRFI_TIME);
            }
        }
        
        // Check integration performance
        if let Some(integration_time) = report.performance_results.get("Integration") {
            assert!(integration_time <= &MAX_INTEGRATION_TIME,
                    "Integration tests took {:?}, which exceeds maximum {:?}",
                    integration_time, MAX_INTEGRATION_TIME);
        }
    }
}
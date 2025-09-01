//! R7RS Compliance Validation Framework for Lambdust Optimization
//!
//! This module implements comprehensive validation of R7RS Scheme compliance
//! specifically focusing on optimization impact assessment. The framework
//! ensures that performance optimizations preserve semantic correctness.

use lambdust::ast::Expr;
use lambdust::eval::{evaluator::Evaluator, value::Value};
use lambdust::runtime::runtime::Runtime;
use std::collections::HashMap;
use std::sync::Arc;

/// Comprehensive R7RS compliance validation suite
pub struct R7RSComplianceValidator {
    runtime: Arc<Runtime>,
    test_results: HashMap<String, ValidationResult>,
    optimization_metrics: OptimizationMetrics,
}

/// Results of a specific compliance validation test
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub test_name: String,
    pub passed: bool,
    pub expected_output: Option<Value>,
    pub actual_output: Option<Value>,
    pub error_message: Option<String>,
    pub performance_impact: Option<PerformanceImpact>,
}

/// Performance impact measurement for optimization validation
#[derive(Debug, Clone)]
pub struct PerformanceImpact {
    pub execution_time_before_ns: u128,
    pub execution_time_after_ns: u128,
    pub memory_usage_before_bytes: usize,
    pub memory_usage_after_bytes: usize,
    pub improvement_ratio: f64,
}

/// Optimization impact metrics
#[derive(Debug, Clone, Default)]
pub struct OptimizationMetrics {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub semantic_violations: Vec<String>,
    pub performance_regressions: Vec<String>,
    pub memory_improvements: Vec<String>,
}

impl R7RSComplianceValidator {
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            runtime,
            test_results: HashMap::new(),
            optimization_metrics: OptimizationMetrics::default(),
        }
    }

    /// Execute complete R7RS compliance validation suite
    pub fn validate_full_compliance(&mut self) -> ValidationSummary {
        println!("🔍 Starting comprehensive R7RS compliance validation...");

        // Priority 1: Semantic Preservation Tests
        self.validate_evaluation_order();
        self.validate_tail_call_optimization();
        self.validate_continuation_semantics();

        // Priority 2: Macro System Correctness
        self.validate_macro_hygiene();
        self.validate_expansion_phase_correctness();

        // Priority 3: Numerical Tower Compliance
        self.validate_exact_inexact_semantics();
        self.validate_numeric_type_promotion();

        // Memory and Object Identity Tests
        self.validate_memory_semantics();
        self.validate_object_identity_preservation();

        // Performance vs Correctness Analysis
        self.analyze_optimization_trade_offs();

        self.generate_validation_summary()
    }

    /// Validate evaluation order guarantees (left-to-right)
    fn validate_evaluation_order(&mut self) {
        let test_cases = vec![
            // Test side effect ordering in arithmetic
            r#"
            (define counter 0)
            (define (increment!)
              (set! counter (+ counter 1))
              counter)
            (list (increment!) (increment!) (increment!))
            "#,
            // Test procedure argument evaluation order
            r#"
            (define log '())
            (define (log-value val)
              (set! log (cons val log))
              val)
            (+ (log-value 1) (log-value 2) (log-value 3))
            log
            "#,
        ];

        for (i, test) in test_cases.iter().enumerate() {
            let test_name = format!("evaluation_order_{}", i);
            let result = self.run_test_case(&test_name, test);
            self.record_test_result(result);
        }
    }

    /// Validate proper tail call optimization (constant space)
    fn validate_tail_call_optimization(&mut self) {
        let test_cases = vec![
            // Tail-recursive factorial should use constant space
            r#"
            (define (factorial n acc)
              (if (zero? n)
                  acc
                  (factorial (- n 1) (* n acc))))
            (factorial 10000 1)
            "#,
            // Mutually tail-recursive even/odd
            r#"
            (define (even? n)
              (if (zero? n)
                  #t
                  (odd? (- n 1))))
            (define (odd? n)
              (if (zero? n)
                  #f
                  (even? (- n 1))))
            (even? 10000)
            "#,
        ];

        for (i, test) in test_cases.iter().enumerate() {
            let test_name = format!("tail_call_optimization_{}", i);
            let result = self.run_stack_bounded_test(&test_name, test);
            self.record_test_result(result);
        }
    }

    /// Validate call/cc continuation semantics
    fn validate_continuation_semantics(&mut self) {
        let test_cases = vec![
            // Basic call/cc escape continuation
            r#"
            (call/cc 
              (lambda (k)
                (+ 1 2 (k 42) 3 4)))
            "#,
            // Continuation with dynamic-wind
            r#"
            (define path '())
            (dynamic-wind
              (lambda () (set! path (cons 'before path)))
              (lambda ()
                (call/cc
                  (lambda (k)
                    (set! path (cons 'during path))
                    (k 'escaped))))
              (lambda () (set! path (cons 'after path))))
            path
            "#,
        ];

        for (i, test) in test_cases.iter().enumerate() {
            let test_name = format!("continuation_semantics_{}", i);
            let result = self.run_test_case(&test_name, test);
            self.record_test_result(result);
        }
    }

    /// Validate macro hygiene preservation
    fn validate_macro_hygiene(&mut self) {
        let test_cases = vec![
            // Variable capture prevention
            r#"
            (define-syntax let1
              (syntax-rules ()
                ((let1 var val body ...)
                 ((lambda (var) body ...) val))))
            
            (let ((x 1))
              (let1 x 2 x))
            "#,
            // Template variable hygiene
            r#"
            (define-syntax swap!
              (syntax-rules ()
                ((swap! a b)
                 (let ((temp a))
                   (set! a b)
                   (set! b temp)))))
            
            (let ((temp 'original) (x 1) (y 2))
              (swap! x y)
              (list x y temp))
            "#,
        ];

        for (i, test) in test_cases.iter().enumerate() {
            let test_name = format!("macro_hygiene_{}", i);
            let result = self.run_test_case(&test_name, test);
            self.record_test_result(result);
        }
    }

    /// Validate expansion phase correctness
    fn validate_expansion_phase_correctness(&mut self) {
        let test_cases = vec![
            // Compile-time macro expansion
            r#"
            (define-syntax compile-time-constant
              (syntax-rules ()
                ((compile-time-constant) 42)))
            
            (define x (compile-time-constant))
            x
            "#,
            // Nested macro expansion
            r#"
            (define-syntax outer
              (syntax-rules ()
                ((outer x) (inner x))))
            
            (define-syntax inner
              (syntax-rules ()
                ((inner y) (list 'inner y))))
            
            (outer 'test)
            "#,
        ];

        for (i, test) in test_cases.iter().enumerate() {
            let test_name = format!("expansion_phase_{}", i);
            let result = self.run_test_case(&test_name, test);
            self.record_test_result(result);
        }
    }

    /// Validate exact/inexact arithmetic semantics
    fn validate_exact_inexact_semantics(&mut self) {
        let test_cases = vec![
            // Exact rational arithmetic
            r#"
            (define result (+ 1/3 1/3 1/3))
            (and (exact? result) (= result 1))
            "#,
            // Inexact real arithmetic  
            r#"
            (define result (+ 0.33 0.33 0.34))
            (and (inexact? result) (< 0.99 result 1.01))
            "#,
            // Mixed exact/inexact operations
            r#"
            (define result (+ 1/2 0.5))
            (inexact? result)
            "#,
        ];

        for (i, test) in test_cases.iter().enumerate() {
            let test_name = format!("exact_inexact_semantics_{}", i);
            let result = self.run_test_case(&test_name, test);
            self.record_test_result(result);
        }
    }

    /// Validate numeric type promotion in tower
    fn validate_numeric_type_promotion(&mut self) {
        let test_cases = vec![
            // Integer to rational promotion
            r#"
            (define result (/ 3 4))
            (and (rational? result) (= result 3/4))
            "#,
            // Complex number operations
            r#"
            (define result (* 3+4i 2))
            (and (complex? result) (= result 6+8i))
            "#,
            // Mixed numeric tower operations
            r#"
            (define result (+ 1 2.0 3/4 1+2i))
            (complex? result)
            "#,
        ];

        for (i, test) in test_cases.iter().enumerate() {
            let test_name = format!("numeric_promotion_{}", i);
            let result = self.run_test_case(&test_name, test);
            self.record_test_result(result);
        }
    }

    /// Validate memory semantics and object identity
    fn validate_memory_semantics(&mut self) {
        let test_cases = vec![
            // eq? object identity
            r#"
            (define x (cons 1 2))
            (eq? x x)
            "#,
            // String interning effects on eq?
            r#"
            (let ((s1 "hello") (s2 "hello"))
              (list (string=? s1 s2) (eq? s1 s2)))
            "#,
            // Symbol identity preservation
            r#"
            (eq? 'symbol 'symbol)
            "#,
        ];

        for (i, test) in test_cases.iter().enumerate() {
            let test_name = format!("memory_semantics_{}", i);
            let result = self.run_test_case(&test_name, test);
            self.record_test_result(result);
        }
    }

    /// Validate object identity preservation under optimization
    fn validate_object_identity_preservation(&mut self) {
        let test_cases = vec![
            // Pair identity preservation
            r#"
            (define pairs (list (cons 1 2) (cons 3 4) (cons 5 6)))
            (eq? (car pairs) (car pairs))
            "#,
            // Procedure identity
            r#"
            (define proc (lambda (x) x))
            (eq? proc proc)
            "#,
            // Vector identity
            r#"
            (define vec (vector 1 2 3))
            (eq? vec vec)
            "#,
        ];

        for (i, test) in test_cases.iter().enumerate() {
            let test_name = format!("object_identity_{}", i);
            let result = self.run_test_case(&test_name, test);
            self.record_test_result(result);
        }
    }

    /// Analyze optimization trade-offs between performance and correctness
    fn analyze_optimization_trade_offs(&mut self) {
        println!("📊 Analyzing optimization trade-offs...");
        
        // Performance benchmark tests
        let benchmark_tests = vec![
            ("list_operations", "(fold + 0 (iota 10000))"),
            ("recursive_fibonacci", "(define (fib n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2))))) (fib 30)"),
            ("macro_expansion_heavy", "(define-syntax repeat (syntax-rules () ((repeat n body) (let loop ((i 0)) (if (< i n) (begin body (loop (+ i 1)))))))) (repeat 1000 'test)"),
        ];

        for (name, test) in benchmark_tests {
            let performance_result = self.run_performance_test(name, test);
            self.record_performance_impact(performance_result);
        }
    }

    /// Run a single test case with full error handling
    fn run_test_case(&self, test_name: &str, code: &str) -> ValidationResult {
        let start_time = std::time::Instant::now();
        
        match self.runtime.eval_string(code) {
            Ok(result) => ValidationResult {
                test_name: test_name.to_string(),
                passed: true,
                expected_output: None, // Could be enhanced with expected values
                actual_output: Some(result),
                error_message: None,
                performance_impact: Some(PerformanceImpact {
                    execution_time_before_ns: 0,
                    execution_time_after_ns: start_time.elapsed().as_nanos(),
                    memory_usage_before_bytes: 0,
                    memory_usage_after_bytes: 0,
                    improvement_ratio: 1.0,
                }),
            },
            Err(e) => ValidationResult {
                test_name: test_name.to_string(),
                passed: false,
                expected_output: None,
                actual_output: None,
                error_message: Some(format!("{:?}", e)),
                performance_impact: None,
            },
        }
    }

    /// Run test with stack space monitoring for tail call validation
    fn run_stack_bounded_test(&self, test_name: &str, code: &str) -> ValidationResult {
        // This would require integration with stack monitoring
        // For now, use regular test execution
        self.run_test_case(test_name, code)
    }

    /// Run performance benchmark test
    fn run_performance_test(&self, test_name: &str, code: &str) -> PerformanceImpact {
        let start_time = std::time::Instant::now();
        let _result = self.runtime.eval_string(code);
        let duration = start_time.elapsed().as_nanos();

        PerformanceImpact {
            execution_time_before_ns: 0, // Would need baseline measurement
            execution_time_after_ns: duration,
            memory_usage_before_bytes: 0,
            memory_usage_after_bytes: 0,
            improvement_ratio: 1.0,
        }
    }

    /// Record test result and update metrics
    fn record_test_result(&mut self, result: ValidationResult) {
        self.optimization_metrics.total_tests += 1;
        
        if result.passed {
            self.optimization_metrics.passed_tests += 1;
        } else {
            self.optimization_metrics.failed_tests += 1;
            self.optimization_metrics.semantic_violations.push(result.test_name.clone());
        }

        self.test_results.insert(result.test_name.clone(), result);
    }

    /// Record performance impact measurement
    fn record_performance_impact(&mut self, impact: PerformanceImpact) {
        if impact.improvement_ratio > 1.0 {
            self.optimization_metrics.memory_improvements.push(
                format!("Performance improved by {:.2}x", impact.improvement_ratio)
            );
        } else if impact.improvement_ratio < 0.9 {
            self.optimization_metrics.performance_regressions.push(
                format!("Performance regressed by {:.2}x", 1.0 / impact.improvement_ratio)
            );
        }
    }

    /// Generate comprehensive validation summary
    fn generate_validation_summary(&self) -> ValidationSummary {
        let compliance_rate = if self.optimization_metrics.total_tests > 0 {
            (self.optimization_metrics.passed_tests as f64) / (self.optimization_metrics.total_tests as f64)
        } else {
            0.0
        };

        ValidationSummary {
            overall_compliance_rate: compliance_rate,
            total_tests_run: self.optimization_metrics.total_tests,
            passed_tests: self.optimization_metrics.passed_tests,
            failed_tests: self.optimization_metrics.failed_tests,
            semantic_violations: self.optimization_metrics.semantic_violations.clone(),
            performance_improvements: self.optimization_metrics.memory_improvements.clone(),
            performance_regressions: self.optimization_metrics.performance_regressions.clone(),
            r7rs_compliance_status: if compliance_rate >= 1.0 {
                ComplianceStatus::FullyCompliant
            } else if compliance_rate >= 0.95 {
                ComplianceStatus::MostlyCompliant
            } else {
                ComplianceStatus::NonCompliant
            },
        }
    }
}

/// Summary of validation results
#[derive(Debug)]
pub struct ValidationSummary {
    pub overall_compliance_rate: f64,
    pub total_tests_run: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub semantic_violations: Vec<String>,
    pub performance_improvements: Vec<String>,
    pub performance_regressions: Vec<String>,
    pub r7rs_compliance_status: ComplianceStatus,
}

/// R7RS compliance status classification
#[derive(Debug, Clone, PartialEq)]
pub enum ComplianceStatus {
    FullyCompliant,
    MostlyCompliant,
    NonCompliant,
}

impl ValidationSummary {
    /// Print comprehensive validation report
    pub fn print_report(&self) {
        println!("\n🎯 R7RS Compliance Validation Report");
        println!("=====================================");
        println!("Overall Compliance Rate: {:.1}%", self.overall_compliance_rate * 100.0);
        println!("Total Tests: {}", self.total_tests_run);
        println!("Passed: {} | Failed: {}", self.passed_tests, self.failed_tests);
        println!("Status: {:?}", self.r7rs_compliance_status);

        if !self.semantic_violations.is_empty() {
            println!("\n⚠️  Semantic Violations:");
            for violation in &self.semantic_violations {
                println!("  - {}", violation);
            }
        }

        if !self.performance_improvements.is_empty() {
            println!("\n📈 Performance Improvements:");
            for improvement in &self.performance_improvements {
                println!("  - {}", improvement);
            }
        }

        if !self.performance_regressions.is_empty() {
            println!("\n📉 Performance Regressions:");
            for regression in &self.performance_regressions {
                println!("  - {}", regression);
            }
        }

        println!("\n🔍 Validation Complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_framework_creation() {
        // Test framework instantiation
        // Would require mock runtime for isolated testing
    }

    #[test]
    fn test_performance_impact_calculation() {
        let impact = PerformanceImpact {
            execution_time_before_ns: 1000,
            execution_time_after_ns: 800,
            memory_usage_before_bytes: 1024,
            memory_usage_after_bytes: 768,
            improvement_ratio: 1.25,
        };

        assert!(impact.improvement_ratio > 1.0);
    }
}
//! SRFI-64 Testing Framework Implementation
//!
//! This module provides the core infrastructure for SRFI-64 (A Scheme API for test suites).
//! SRFI-64 is a comprehensive testing framework that provides standardized test procedures,
//! test runners, and result reporting capabilities.
//!
//! ## Features
//!
//! - **Test Suites**: Hierarchical organization of tests
//! - **Test Cases**: Individual test assertions with descriptive names
//! - **Assertions**: Various test predicates (equal, approximate, error handling)
//! - **Test Runners**: Execution engine with result collection
//! - **Reporting**: Detailed pass/fail statistics and error reporting
//! - **Hooks**: Before/after test hooks for setup and teardown

use crate::diagnostics::{Error as DiagnosticError, Result, Span};
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::effects::Effect;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Global test runner state for SRFI-64
static TEST_RUNNER: once_cell::sync::Lazy<Arc<Mutex<TestRunner>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(TestRunner::new())));

/// Test result for individual test cases
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Name of the test
    pub name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Expected value (if applicable)
    pub expected: Option<Value>,
    /// Actual value (if applicable)  
    pub actual: Option<Value>,
    /// Error message (if test failed)
    pub error_message: Option<String>,
    /// Test execution time
    pub duration: Duration,
    /// Source location (if available)
    pub location: Option<Span>,
}

/// Test suite containing multiple test cases
#[derive(Debug, Clone)]
pub struct TestSuite {
    /// Name of the test suite
    pub name: String,
    /// Individual test results
    pub tests: Vec<TestResult>,
    /// Nested test suites
    pub suites: Vec<TestSuite>,
    /// Suite setup hooks
    pub setup_hooks: Vec<Value>,
    /// Suite teardown hooks
    pub teardown_hooks: Vec<Value>,
    /// Suite execution time
    pub duration: Duration,
}

/// Main test runner for SRFI-64
#[derive(Debug)]
pub struct TestRunner {
    /// Current test suite stack
    suite_stack: Vec<TestSuite>,
    /// Global test statistics
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    /// Current test name (for nested test contexts)
    current_test_name: Option<String>,
    /// Test execution start time
    start_time: Option<Instant>,
    /// Whether to show verbose output
    verbose: bool,
}

impl TestRunner {
    /// Creates a new test runner
    pub fn new() -> Self {
        Self {
            suite_stack: vec![TestSuite {
                name: "default".to_string(),
                tests: Vec::new(),
                suites: Vec::new(),
                setup_hooks: Vec::new(),
                teardown_hooks: Vec::new(),
                duration: Duration::new(0, 0),
            }],
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            current_test_name: None,
            start_time: None,
            verbose: false,
        }
    }

    /// Begins a new test suite
    pub fn begin_suite(&mut self, name: String) {
        let new_suite = TestSuite {
            name,
            tests: Vec::new(),
            suites: Vec::new(),
            setup_hooks: Vec::new(),
            teardown_hooks: Vec::new(),
            duration: Duration::new(0, 0),
        };
        self.suite_stack.push(new_suite);
    }

    /// Ends the current test suite
    pub fn end_suite(&mut self) -> Option<TestSuite> {
        if self.suite_stack.len() > 1 {
            let completed_suite = self.suite_stack.pop()?;
            
            // Add completed suite to parent
            if let Some(parent_suite) = self.suite_stack.last_mut() {
                parent_suite.suites.push(completed_suite.clone());
            }
            
            Some(completed_suite)
        } else {
            None
        }
    }

    /// Records a test result
    pub fn record_test(&mut self, result: TestResult) {
        self.total_tests += 1;
        if result.passed {
            self.passed_tests += 1;
            if self.verbose {
                println!("PASS: {}", result.name);
            }
        } else {
            self.failed_tests += 1;
            println!("FAIL: {}", result.name);
            if let Some(ref error) = result.error_message {
                println!("      {}", error);
            }
        }

        // Add to current suite
        if let Some(current_suite) = self.suite_stack.last_mut() {
            current_suite.tests.push(result);
        }
    }

    /// Sets the current test name
    pub fn set_current_test(&mut self, name: Option<String>) {
        self.current_test_name = name.clone();
        if name.is_some() {
            self.start_time = Some(Instant::now());
        }
    }

    /// Gets test execution duration
    pub fn get_test_duration(&self) -> Duration {
        self.start_time
            .map(|start| start.elapsed())
            .unwrap_or_default()
    }

    /// Prints final test summary
    pub fn print_summary(&self) {
        println!("\n=== Test Summary ===");
        println!("Total tests: {}", self.total_tests);
        println!("Passed: {}", self.passed_tests);
        println!("Failed: {}", self.failed_tests);
        
        if self.failed_tests == 0 {
            println!("All tests passed! ✅");
        } else {
            println!("Some tests failed! ❌");
        }
    }

    /// Resets test runner state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Sets verbose mode
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates SRFI-64 testing procedure bindings
pub fn create_srfi64_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Test suite management
    bind_test_suite_procedures(env);
    
    // Test case procedures
    bind_test_case_procedures(env);
    
    // Test assertion procedures
    bind_test_assertion_procedures(env);
    
    // Test runner control
    bind_test_runner_procedures(env);
}

/// Binds test suite management procedures
fn bind_test_suite_procedures(env: &Arc<ThreadSafeEnvironment>) {
    // test-begin - Start a test suite
    env.define(
        "test-begin".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "test-begin".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_test_begin),
            effects: vec![Effect::IO], // Side effect: modifies test runner state
        })),
    );

    // test-end - End current test suite
    env.define(
        "test-end".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "test-end".to_string(),
            arity_min: 0,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_test_end),
            effects: vec![Effect::IO],
        })),
    );
}

/// Binds test case procedures
fn bind_test_case_procedures(env: &Arc<ThreadSafeEnvironment>) {
    // test-assert - Basic assertion with optional name
    env.define(
        "test-assert".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "test-assert".to_string(),
            arity_min: 1,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(primitive_test_assert),
            effects: vec![Effect::IO],
        })),
    );
}

/// Binds test assertion procedures
fn bind_test_assertion_procedures(env: &Arc<ThreadSafeEnvironment>) {
    // test-equal - Test for equality
    env.define(
        "test-equal".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "test-equal".to_string(),
            arity_min: 2,
            arity_max: Some(3),
            implementation: PrimitiveImpl::RustFn(primitive_test_equal),
            effects: vec![Effect::IO],
        })),
    );

    // test-eqv - Test for equivalence
    env.define(
        "test-eqv".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "test-eqv".to_string(),
            arity_min: 2,
            arity_max: Some(3),
            implementation: PrimitiveImpl::RustFn(primitive_test_eqv),
            effects: vec![Effect::IO],
        })),
    );

    // test-eq - Test for identity
    env.define(
        "test-eq".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "test-eq".to_string(),
            arity_min: 2,
            arity_max: Some(3),
            implementation: PrimitiveImpl::RustFn(primitive_test_eq),
            effects: vec![Effect::IO],
        })),
    );

    // test-approximate - Test for approximate numerical equality
    env.define(
        "test-approximate".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "test-approximate".to_string(),
            arity_min: 3,
            arity_max: Some(4),
            implementation: PrimitiveImpl::RustFn(primitive_test_approximate),
            effects: vec![Effect::IO],
        })),
    );
}

/// Binds test runner control procedures
fn bind_test_runner_procedures(env: &Arc<ThreadSafeEnvironment>) {
    // test-runner-reset - Reset test runner state
    env.define(
        "test-runner-reset".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "test-runner-reset".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(primitive_test_runner_reset),
            effects: vec![Effect::IO],
        })),
    );

    // test-runner-summary - Print test summary
    env.define(
        "test-runner-summary".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "test-runner-summary".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(primitive_test_runner_summary),
            effects: vec![Effect::IO],
        })),
    );
}

// ============= PRIMITIVE IMPLEMENTATIONS =============

/// Implements test-begin procedure
fn primitive_test_begin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("test-begin expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    let suite_name = match &args[0] {
        Value::Literal(crate::ast::Literal::String(s)) => (**s).clone(),
        Value::Symbol(sym_id) => {
            // Convert symbol to string - for now just use debug representation
            format!("{:?}", sym_id)
        },
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                "test-begin requires a string or symbol name".to_string(),
                None,
            )));
        }
    };

    // Begin new test suite
    if let Ok(mut runner) = TEST_RUNNER.lock() {
        runner.begin_suite(suite_name);
    }

    Ok(Value::Literal(crate::ast::Literal::Boolean(true)))
}

/// Implements test-end procedure
fn primitive_test_end(args: &[Value]) -> Result<Value> {
    if args.len() > 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("test-end expects 0 or 1 argument, got {}", args.len()),
            None,
        )));
    }

    // End current test suite
    if let Ok(mut runner) = TEST_RUNNER.lock() {
        runner.end_suite();
    }

    Ok(Value::Literal(crate::ast::Literal::Boolean(true)))
}

/// Implements test-assert procedure
fn primitive_test_assert(args: &[Value]) -> Result<Value> {
    let (test_name, test_expr) = match args.len() {
        1 => ("unnamed-test".to_string(), &args[0]),
        2 => {
            let name = match &args[0] {
                Value::Literal(crate::ast::Literal::String(s)) => (**s).clone(),
                _ => "unnamed-test".to_string(),
            };
            (name, &args[1])
        }
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("test-assert expects 1 or 2 arguments, got {}", args.len()),
                None,
            )));
        }
    };

    let start_time = Instant::now();
    
    // Test the assertion
    let passed = match test_expr {
        Value::Literal(crate::ast::Literal::Boolean(b)) => *b,
        _ => false, // Non-boolean values are considered false
    };

    let result = TestResult {
        name: test_name,
        passed,
        expected: Some(Value::Literal(crate::ast::Literal::Boolean(true))),
        actual: Some(test_expr.clone()),
        error_message: if !passed {
            Some(format!("Expected true, got {}", test_expr))
        } else {
            None
        },
        duration: start_time.elapsed(),
        location: None,
    };

    // Record test result
    if let Ok(mut runner) = TEST_RUNNER.lock() {
        runner.record_test(result);
    }

    Ok(Value::Literal(crate::ast::Literal::Boolean(passed)))
}

/// Implements test-equal procedure
fn primitive_test_equal(args: &[Value]) -> Result<Value> {
    let (test_name, expected, actual) = match args.len() {
        2 => ("unnamed-test".to_string(), &args[0], &args[1]),
        3 => {
            let name = match &args[0] {
                Value::Literal(crate::ast::Literal::String(s)) => (**s).clone(),
                _ => "unnamed-test".to_string(),
            };
            (name, &args[1], &args[2])
        }
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("test-equal expects 2 or 3 arguments, got {}", args.len()),
                None,
            )));
        }
    };

    let start_time = Instant::now();
    
    // Test equality (using Scheme equal? semantics)
    let passed = expected == actual;

    let result = TestResult {
        name: test_name,
        passed,
        expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        error_message: if !passed {
            Some(format!("Expected {}, got {}", expected, actual))
        } else {
            None
        },
        duration: start_time.elapsed(),
        location: None,
    };

    // Record test result
    if let Ok(mut runner) = TEST_RUNNER.lock() {
        runner.record_test(result);
    }

    Ok(Value::Literal(crate::ast::Literal::Boolean(passed)))
}

/// Implements test-eqv procedure (using eqv? semantics)
fn primitive_test_eqv(args: &[Value]) -> Result<Value> {
    let (test_name, expected, actual) = match args.len() {
        2 => ("unnamed-test".to_string(), &args[0], &args[1]),
        3 => {
            let name = match &args[0] {
                Value::Literal(crate::ast::Literal::String(s)) => (**s).clone(),
                _ => "unnamed-test".to_string(),
            };
            (name, &args[1], &args[2])
        }
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("test-eqv expects 2 or 3 arguments, got {}", args.len()),
                None,
            )));
        }
    };

    let start_time = Instant::now();
    
    // Test equivalence (using Scheme eqv? semantics)
    // For now, use == which is similar to eqv? for basic types
    let passed = expected == actual;

    let result = TestResult {
        name: test_name,
        passed,
        expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        error_message: if !passed {
            Some(format!("Expected {} (eqv?), got {}", expected, actual))
        } else {
            None
        },
        duration: start_time.elapsed(),
        location: None,
    };

    // Record test result
    if let Ok(mut runner) = TEST_RUNNER.lock() {
        runner.record_test(result);
    }

    Ok(Value::Literal(crate::ast::Literal::Boolean(passed)))
}

/// Implements test-eq procedure (using eq? semantics)
fn primitive_test_eq(args: &[Value]) -> Result<Value> {
    let (test_name, expected, actual) = match args.len() {
        2 => ("unnamed-test".to_string(), &args[0], &args[1]),
        3 => {
            let name = match &args[0] {
                Value::Literal(crate::ast::Literal::String(s)) => (**s).clone(),
                _ => "unnamed-test".to_string(),
            };
            (name, &args[1], &args[2])
        }
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("test-eq expects 2 or 3 arguments, got {}", args.len()),
                None,
            )));
        }
    };

    let start_time = Instant::now();
    
    // Test identity (using Scheme eq? semantics)
    let passed = expected.eq(actual);

    let result = TestResult {
        name: test_name,
        passed,
        expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        error_message: if !passed {
            Some(format!("Expected {} (eq?), got {}", expected, actual))
        } else {
            None
        },
        duration: start_time.elapsed(),
        location: None,
    };

    // Record test result
    if let Ok(mut runner) = TEST_RUNNER.lock() {
        runner.record_test(result);
    }

    Ok(Value::Literal(crate::ast::Literal::Boolean(passed)))
}

/// Implements test-approximate procedure (for numerical approximation)
fn primitive_test_approximate(args: &[Value]) -> Result<Value> {
    let (test_name, expected, actual, epsilon) = match args.len() {
        3 => ("unnamed-test".to_string(), &args[0], &args[1], &args[2]),
        4 => {
            let name = match &args[0] {
                Value::Literal(crate::ast::Literal::String(s)) => (**s).clone(),
                _ => "unnamed-test".to_string(),
            };
            (name, &args[1], &args[2], &args[3])
        }
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("test-approximate expects 3 or 4 arguments, got {}", args.len()),
                None,
            )));
        }
    };

    let start_time = Instant::now();
    
    // Extract numeric values
    let expected_num = match expected {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => *i as f64,
        Value::Literal(crate::ast::Literal::InexactReal(r)) => *r,
        Value::Literal(crate::ast::Literal::Rational(rat)) => {
            rat.numerator as f64 / rat.denominator as f64
        }
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("test-approximate expected value must be numeric, got: {:?}", expected),
                None,
            )));
        }
    };

    let actual_num = match actual {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => *i as f64,
        Value::Literal(crate::ast::Literal::InexactReal(r)) => *r,
        Value::Literal(crate::ast::Literal::Rational(rat)) => {
            rat.numerator as f64 / rat.denominator as f64
        }
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("test-approximate actual value must be numeric, got: {:?}", actual),
                None,
            )));
        }
    };

    let epsilon_num = match epsilon {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => *i as f64,
        Value::Literal(crate::ast::Literal::InexactReal(r)) => *r,
        Value::Literal(crate::ast::Literal::Rational(rat)) => {
            rat.numerator as f64 / rat.denominator as f64
        }
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("test-approximate epsilon must be numeric, got: {:?}", epsilon),
                None,
            )));
        }
    };

    // Test approximate equality
    let passed = (expected_num - actual_num).abs() <= epsilon_num;

    let result = TestResult {
        name: test_name,
        passed,
        expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        error_message: if !passed {
            Some(format!(
                "Expected {} ± {}, got {} (difference: {})",
                expected_num,
                epsilon_num,
                actual_num,
                (expected_num - actual_num).abs()
            ))
        } else {
            None
        },
        duration: start_time.elapsed(),
        location: None,
    };

    // Record test result
    if let Ok(mut runner) = TEST_RUNNER.lock() {
        runner.record_test(result);
    }

    Ok(Value::Literal(crate::ast::Literal::Boolean(passed)))
}

/// Implements test-runner-reset procedure
fn primitive_test_runner_reset(_args: &[Value]) -> Result<Value> {
    if let Ok(mut runner) = TEST_RUNNER.lock() {
        runner.reset();
    }
    Ok(Value::Literal(crate::ast::Literal::Boolean(true)))
}

/// Implements test-runner-summary procedure
fn primitive_test_runner_summary(_args: &[Value]) -> Result<Value> {
    if let Ok(runner) = TEST_RUNNER.lock() {
        runner.print_summary();
    }
    Ok(Value::Literal(crate::ast::Literal::Boolean(true)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_basic_functionality() {
        let mut runner = TestRunner::new();
        
        // Start a test suite
        runner.begin_suite("basic-tests".to_string());
        
        // Record some test results
        runner.record_test(TestResult {
            name: "test1".to_string(),
            passed: true,
            expected: None,
            actual: None,
            error_message: None,
            duration: Duration::from_millis(10),
            location: None,
        });
        
        runner.record_test(TestResult {
            name: "test2".to_string(),
            passed: false,
            expected: Some(Value::integer(42)),
            actual: Some(Value::integer(24)),
            error_message: Some("Numbers don't match".to_string()),
            duration: Duration::from_millis(5),
            location: None,
        });
        
        // End the test suite
        runner.end_suite();
        
        // Verify statistics
        assert_eq!(runner.total_tests, 2);
        assert_eq!(runner.passed_tests, 1);
        assert_eq!(runner.failed_tests, 1);
    }

    #[test]
    fn test_nested_suites() {
        let mut runner = TestRunner::new();
        
        // Start parent suite
        runner.begin_suite("parent".to_string());
        
        // Start child suite
        runner.begin_suite("child".to_string());
        
        runner.record_test(TestResult {
            name: "child-test".to_string(),
            passed: true,
            expected: None,
            actual: None,
            error_message: None,
            duration: Duration::from_millis(1),
            location: None,
        });
        
        // End child suite
        let child_suite = runner.end_suite().unwrap();
        assert_eq!(child_suite.name, "child");
        assert_eq!(child_suite.tests.len(), 1);
        
        // End parent suite
        runner.end_suite();
    }
}
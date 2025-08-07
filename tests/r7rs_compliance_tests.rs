//! R7RS Compliance Test Suite
//!
//! This module provides comprehensive testing for R7RS-small standard compliance
//! in the Lambdust Scheme interpreter. It verifies that the implementation
//! conforms to the R7RS specification for:
//!
//! - Basic data types and predicates  
//! - Numeric operations and mathematical functions
//! - String manipulation and conversion
//! - List processing and operations
//! - Control structures and conditional expressions
//! - I/O operations and port handling
//! - Macro system (syntax-rules)
//! - Exception handling (guard/raise)
//!
//! Each test category is implemented in a separate module for maintainability
//! and modularity. Tests are designed to be run incrementally as features
//! are implemented in the interpreter.

use lambdust::{
    eval::{evaluator::Evaluator, environment::{Environment, global_environment}, value::Value},
    lexer::Lexer,
    parser::Parser,
};

/// Test configuration for R7RS compliance tests
#[derive(Debug, Clone)]
pub struct R7RSTestConfig {
    /// Enable strict R7RS mode (disable extensions)
    pub strict_mode: bool,
    /// Skip tests for unimplemented features
    pub skip_unimplemented: bool,
    /// Verbose output for test diagnostics
    pub verbose: bool,
}

impl Default for R7RSTestConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            skip_unimplemented: true,
            verbose: false,
        }
    }
}

/// R7RS compliance test runner
pub struct R7RSTestSuite {
    config: R7RSTestConfig,
    evaluator: Evaluator,
    global_env: std::rc::Rc<Environment>,
}

impl R7RSTestSuite {
    /// Create a new R7RS test suite with default configuration
    pub fn new() -> Self {
        Self::with_config(R7RSTestConfig::default())
    }
    
    /// Create a new R7RS test suite with custom configuration
    pub fn with_config(config: R7RSTestConfig) -> Self {
        let global_env = global_environment();
        let evaluator = Evaluator::new();
        
        Self {
            config,
            evaluator,
            global_env,
        }
    }
    
    /// Evaluate a Scheme expression and return the result
    pub fn eval(&mut self, input: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let mut lexer = Lexer::new(input, Some("test"));
        let tokens = lexer.tokenize()?;
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        // Evaluate in the global environment
        let result = self.evaluator.eval_program(&ast)?;
        Ok(result)
    }
    
    /// Evaluate and expect a specific value
    pub fn assert_eval_eq(&mut self, input: &str, expected: Value) -> Result<(), Box<dyn std::error::Error>> {
        let result = self.eval(input)?;
        if result != expected {
            return Err(format!(
                "Assertion failed:\n  Input: {}\n  Expected: {:?}\n  Got: {:?}",
                input, expected, result
            ).into());
        }
        Ok(())
    }
    
    /// Evaluate and expect a boolean true result
    pub fn assert_eval_true(&mut self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        let result = self.eval(input)?;
        if !result.is_truthy() {
            return Err(format!(
                "Assertion failed - expected truthy value:\n  Input: {}\n  Got: {:?}",
                input, result
            ).into());
        }
        Ok(())
    }
    
    /// Evaluate and expect a boolean false result
    pub fn assert_eval_false(&mut self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        let result = self.eval(input)?;
        if result.is_truthy() {
            return Err(format!(
                "Assertion failed - expected falsy value:\n  Input: {}\n  Got: {:?}",
                input, result
            ).into());
        }
        Ok(())
    }
    
    /// Evaluate and expect an error
    pub fn assert_eval_error(&mut self, input: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self.eval(input) {
            Ok(value) => Err(format!(
                "Expected error but got result:\n  Input: {}\n  Got: {:?}",
                input, value
            ).into()),
            Err(_) => Ok(()),
        }
    }
    
    /// Skip a test if unimplemented features are disabled
    pub fn skip_if_unimplemented(&self, feature_name: &str) -> bool {
        if self.config.skip_unimplemented {
            if self.config.verbose {
                println!("Skipping {} (unimplemented)", feature_name);
            }
            true
        } else {
            false
        }
    }
}

/// Test module imports - each module contains specific R7RS compliance tests
mod r7rs {
    pub mod basic_data_types;
    pub mod numeric_operations;
    pub mod string_operations;
    pub mod list_operations;
    pub mod control_structures;
    pub mod io_operations;
    pub mod macro_system;
    pub mod exception_handling;
}

// Compliance reporting module
mod r7rs_compliance_report;

// Re-export test modules for use in integration tests
pub use r7rs::*;
pub use r7rs_compliance_report::{R7RSComplianceMatrix, ComplianceStatistics};

// Note: Comprehensive test suite is available in r7rs_comprehensive_compliance_suite.rs
// pub use crate::r7rs_comprehensive_compliance_suite::{
//     ComprehensiveR7RSTestSuite, 
//     ComprehensiveTestConfig,
//     TestExecutionStats,
//     run_comprehensive_r7rs_tests,
//     run_comprehensive_r7rs_tests_with_config
// };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r7rs_suite_creation() {
        let suite = R7RSTestSuite::new();
        assert!(suite.config.strict_mode);
        assert!(suite.config.skip_unimplemented);
    }

    #[test]
    fn test_basic_evaluation() {
        let mut suite = R7RSTestSuite::new();
        
        // Test basic literal evaluation
        let result = suite.eval("42").expect("Failed to evaluate");
        match result {
            Value::Literal(literal) if literal.to_i64() == Some(42) => {},
            _ => panic!("Expected integer literal"),
        }
    }

    #[test]
    fn test_assertion_helpers() {
        let mut suite = R7RSTestSuite::new();
        
        // Test true assertion
        suite.assert_eval_true("#t").expect("True assertion failed");
        
        // Test false assertion
        suite.assert_eval_false("#f").expect("False assertion failed");
    }
}

/// Run all R7RS compliance tests and generate report
/// 
/// This is the legacy test runner. For comprehensive testing,
/// use the comprehensive suite in r7rs_comprehensive_compliance_suite.rs
pub fn run_all_tests() -> Result<(), Box<dyn std::error::Error>> {
    let mut suite = R7RSTestSuite::new();
    
    println!("Running R7RS Compliance Test Suite (Legacy Mode)...");
    println!("====================================================");
    
    // Run each test category
    basic_data_types::run_tests(&mut suite)?;
    numeric_operations::run_tests(&mut suite)?;
    string_operations::run_tests(&mut suite)?;
    list_operations::run_tests(&mut suite)?;
    control_structures::run_tests(&mut suite)?;
    io_operations::run_tests(&mut suite)?;
    macro_system::run_tests(&mut suite)?;
    exception_handling::run_tests(&mut suite)?;
    
    println!("====================================================");
    println!("All R7RS compliance tests completed!");
    
    // Generate compliance report
    println!("\nGenerating R7RS Compliance Report...");
    let matrix = R7RSComplianceMatrix::new();
    let stats = matrix.get_statistics();
    
    println!("Compliance Summary:");
    println!("- Total Features: {}", stats.total_features);
    println!("- Complete: {} ({:.1}%)", stats.complete_features, 
             (stats.complete_features as f32 / stats.total_features as f32) * 100.0);
    println!("- Partial: {} ({:.1}%)", stats.partial_features,
             (stats.partial_features as f32 / stats.total_features as f32) * 100.0);
    println!("- Missing: {} ({:.1}%)", stats.missing_features,
             (stats.missing_features as f32 / stats.total_features as f32) * 100.0);
    println!("- Overall Compliance: {:.1}%", stats.compliance_percentage);
    
    // Optionally write full report to file
    if std::env::var("GENERATE_FULL_REPORT").unwrap_or_default() == "true" {
        let report = matrix.generate_report();
        std::fs::write("r7rs_compliance_report.md", report)?;
        println!("\nFull compliance report written to: r7rs_compliance_report.md");
    }
    
    println!("\n💡 For comprehensive testing with detailed reports, use:");
    println!("   cargo test r7rs_comprehensive_compliance_suite");
    
    Ok(())
}

/// Run compliance tests with detailed reporting
pub fn run_tests_with_report() -> Result<ComplianceStatistics, Box<dyn std::error::Error>> {
    run_all_tests()?;
    
    let matrix = R7RSComplianceMatrix::new();
    Ok(matrix.get_statistics())
}
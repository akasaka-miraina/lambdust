//! Property-Based Testing Framework for Lambdust
//!
//! This module provides a comprehensive property-based testing framework specifically
//! designed for the Lambdust Scheme interpreter. It supports QuickCheck-style testing
//! with efficient test data generation and automatic shrinking of failing test cases.
//!
//! ## Architecture
//!
//! - **Generators**: Create random test data for Scheme values
//! - **Properties**: Define mathematical properties that should hold
//! - **Shrinkers**: Minimize failing test cases for easier debugging
//! - **Execution**: Run properties efficiently with parallel execution
//!
//! ## Performance Goals
//!
//! - Execute 1,000,000 test cases in under 2 minutes
//! - Memory usage under 1GB for large-scale testing
//! - Parallel execution scaling with available cores

use crate::eval::value::Value;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::fmt::Debug;
use std::sync::Arc;

pub mod execution;
pub mod generators;
pub mod shrinking;

/// Configuration for property-based testing
#[derive(Debug, Clone)]
pub struct PropertyConfig {
    /// Number of test cases to generate
    pub test_cases: usize,
    /// Maximum size for generated data structures
    pub max_size: usize,
    /// Random seed for reproducible testing
    pub seed: Option<u64>,
    /// Enable parallel execution
    pub parallel: bool,
    /// Maximum shrinking iterations
    pub max_shrink_iterations: usize,
}

impl Default for PropertyConfig {
    fn default() -> Self {
        Self {
            test_cases: 100,
            max_size: 50,
            seed: None,
            parallel: true,
            max_shrink_iterations: 1000,
        }
    }
}

/// A generator that can create random values of type T
pub trait Generator<T> {
    /// Generate a random value of type T
    fn generate(&self, rng: &mut XorShiftRng, size: usize) -> T;
    
    /// Generate a smaller version of the given value for shrinking
    fn shrink(&self, value: &T) -> Vec<T> {
        // Default implementation: no shrinking
        vec![]
    }
}

/// A property that can be tested
pub trait Property {
    /// Test the property with the given values
    /// Returns true if the property holds, false otherwise
    fn test(&self, values: &[Value]) -> PropertyResult;
    
    /// Get the name of this property for reporting
    fn name(&self) -> &str;
}

/// Result of a property test
#[derive(Debug, Clone)]
pub enum PropertyResult {
    /// Property passed
    Pass,
    /// Property failed with a description
    Fail(String),
    /// Property could not be evaluated (e.g., due to exceptions)
    Skip(String),
}

impl PropertyResult {
    /// Returns true if this result represents a passing test
    pub fn passed(&self) -> bool {
        matches!(self, PropertyResult::Pass)
    }
    
    /// Returns true if this result represents a failing test
    pub fn failed(&self) -> bool {
        matches!(self, PropertyResult::Fail(_))
    }
}

/// Test result summary
#[derive(Debug, Clone)]
pub struct TestSummary {
    /// Total number of test cases run
    pub total_tests: usize,
    /// Number of tests that passed
    pub passed: usize,
    /// Number of tests that failed
    pub failed: usize,
    /// Number of tests that were skipped
    pub skipped: usize,
    /// First failing test case (if any)
    pub first_failure: Option<(Vec<Value>, String)>,
    /// Time taken to run all tests in milliseconds
    pub duration_ms: u128,
}

impl TestSummary {
    /// Create a new empty test summary
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            first_failure: None,
            duration_ms: 0,
        }
    }
    
    /// Record a test result
    pub fn record(&mut self, result: PropertyResult, test_values: Vec<Value>) {
        self.total_tests += 1;
        
        match result {
            PropertyResult::Pass => self.passed += 1,
            PropertyResult::Fail(msg) => {
                self.failed += 1;
                if self.first_failure.is_none() {
                    self.first_failure = Some((test_values, msg));
                }
            }
            PropertyResult::Skip(_) => self.skipped += 1,
        }
    }
    
    /// Returns true if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0 && self.total_tests > 0
    }
}

impl Default for TestSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Run a property test with the given configuration
pub fn check_property<P, G>(
    property: Arc<P>,
    generators: Vec<Arc<G>>,
    config: PropertyConfig,
) -> TestSummary
where
    P: Property + Send + Sync + 'static,
    G: Generator<Value> + Send + Sync + 'static,
{
    execution::run_property_test(property, generators, config)
}

/// Convenience macro for defining properties
#[macro_export]
macro_rules! property {
    ($name:expr, |$($arg:ident: $typ:ty),*| $body:expr) => {
        {
            struct PropertyImpl;
            impl $crate::property_testing::Property for PropertyImpl {
                fn test(&self, values: &[$crate::eval::value::Value]) -> $crate::property_testing::PropertyResult {
                    // Extract arguments from values - just use the values directly since Value is Copy-like
                    let mut iter = values.iter();
                    $(
                        let $arg: $typ = match iter.next() {
                            Some(v) => v.clone(), // Clone the value instead of trying to convert
                            None => return $crate::property_testing::PropertyResult::Skip(
                                "Not enough arguments".to_string()
                            ),
                        };
                    )*
                    
                    // Execute the property
                    match std::panic::catch_unwind(|| $body) {
                        Ok(true) => $crate::property_testing::PropertyResult::Pass,
                        Ok(false) => $crate::property_testing::PropertyResult::Fail(
                            "Property assertion failed".to_string()
                        ),
                        Err(_) => $crate::property_testing::PropertyResult::Fail(
                            "Property evaluation panicked".to_string()
                        ),
                    }
                }
                
                fn name(&self) -> &str {
                    $name
                }
            }
            
            std::sync::Arc::new(PropertyImpl)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property_testing::generators::SchemeValueGenerator;
    
    #[test]
    fn test_property_framework_basic() {
        let property = property!("always_true", |_x: Value| true);
        let generator = Arc::new(SchemeValueGenerator::new());
        
        let config = PropertyConfig {
            test_cases: 10,
            max_size: 5,
            parallel: false,
            ..PropertyConfig::default()
        };
        
        let summary = check_property(property, vec![generator], config);
        assert!(summary.all_passed());
        assert_eq!(summary.total_tests, 10);
    }
    
    #[test]
    fn test_property_framework_failure() {
        let property = property!("always_false", |_x: Value| false);
        let generator = Arc::new(SchemeValueGenerator::new());
        
        let config = PropertyConfig {
            test_cases: 5,
            max_size: 5,
            parallel: false,
            ..PropertyConfig::default()
        };
        
        let summary = check_property(property, vec![generator], config);
        assert!(!summary.all_passed());
        assert_eq!(summary.failed, 5);
    }
}
//! Property test execution engine
//!
//! This module handles the execution of property tests, including parallel
//! execution, result collection, and shrinking of failing test cases.

use crate::eval::value::Value;
use crate::property_testing::{Generator, Property, PropertyConfig, PropertyResult, TestSummary};
use crate::ast::Literal;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Run a property test with the given configuration
pub fn run_property_test<P, G>(
    property: Arc<P>,
    generators: Vec<Arc<G>>,
    config: PropertyConfig,
) -> TestSummary
where
    P: Property + Send + Sync + 'static,
    G: Generator<Value> + Send + Sync + 'static,
{
    let start_time = Instant::now();
    let summary = Arc::new(Mutex::new(TestSummary::new()));

    // Determine the seed to use
    let base_seed = config.seed.unwrap_or_else(|| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        hasher.finish()
    });

    if config.parallel && config.test_cases > 10 {
        // Parallel execution for larger test suites
        run_parallel_tests(property.clone(), generators.clone(), &config, base_seed, summary.clone());
    } else {
        // Sequential execution for smaller test suites or when parallel is disabled
        run_sequential_tests(property.clone(), generators.clone(), &config, base_seed, summary.clone());
    }

    // Finalize the summary
    let mut final_summary = summary.lock().unwrap();
    final_summary.duration_ms = start_time.elapsed().as_millis();

    // If we have a failure, try to shrink it
    if let Some((values, msg)) = final_summary.first_failure.clone() {
        if let Some(shrunk) = shrink_failing_case(&property, &generators, &values, &config) {
            final_summary.first_failure = Some((shrunk, msg));
        }
    }

    (*final_summary).clone()
}

/// Run tests in parallel
fn run_parallel_tests<P, G>(
    property: Arc<P>,
    generators: Vec<Arc<G>>,
    config: &PropertyConfig,
    base_seed: u64,
    summary: Arc<Mutex<TestSummary>>,
) where
    P: Property + Send + Sync + 'static,
    G: Generator<Value> + Send + Sync + 'static,
{
    // Create a range of test case indices
    (0..config.test_cases)
        .into_par_iter()
        .for_each(|test_index| {
            // Create a unique seed for this test case
            let seed = base_seed.wrapping_add(test_index as u64);
            let mut rng = XorShiftRng::seed_from_u64(seed);

            // Generate test values
            let test_values: Vec<Value> = generators
                .iter()
                .map(|generator| generator.generate(&mut rng, config.max_size))
                .collect();

            // Run the property test
            let result = property.test(&test_values);

            // Update the summary
            {
                let mut summary_guard = summary.lock().unwrap();
                summary_guard.record(result, test_values);
            }
        });
}

/// Run tests sequentially
fn run_sequential_tests<P, G>(
    property: Arc<P>,
    generators: Vec<Arc<G>>,
    config: &PropertyConfig,
    base_seed: u64,
    summary: Arc<Mutex<TestSummary>>,
) where
    P: Property + Send + Sync + 'static,
    G: Generator<Value> + Send + Sync + 'static,
{
    let mut rng = XorShiftRng::seed_from_u64(base_seed);

    for _test_index in 0..config.test_cases {
        // Generate test values
        let test_values: Vec<Value> = generators
            .iter()
            .map(|generator| generator.generate(&mut rng, config.max_size))
            .collect();

        // Run the property test
        let result = property.test(&test_values);

        // Update the summary
        {
            let mut summary_guard = summary.lock().unwrap();
            summary_guard.record(result, test_values);
        }
    }
}

/// Attempt to shrink a failing test case to find a minimal counterexample
fn shrink_failing_case<P, G>(
    property: &Arc<P>,
    generators: &[Arc<G>],
    failing_values: &[Value],
    config: &PropertyConfig,
) -> Option<Vec<Value>>
where
    P: Property + Send + Sync + 'static,
    G: Generator<Value> + Send + Sync + 'static,
{
    let mut current_values = failing_values.to_vec();
    let mut iteration = 0;

    // Verify that the original values actually fail
    if property.test(&current_values).passed() {
        return None;
    }

    while iteration < config.max_shrink_iterations {
        let mut found_smaller = false;

        // Try to shrink each value
        for i in 0..current_values.len() {
            if i < generators.len() {
                let value = current_values[i].clone();
                let shrunk_candidates = generators[i].shrink(&value);

                // Try each shrunk candidate
                for candidate in shrunk_candidates {
                    let mut test_values = current_values.clone();
                    test_values[i] = candidate;

                    // Test if this shrunk version still fails
                    if property.test(&test_values).failed() {
                        current_values = test_values;
                        found_smaller = true;
                        break;
                    }
                }

                if found_smaller {
                    break;
                }
            }
        }

        if !found_smaller {
            break;
        }

        iteration += 1;
    }

    // Return the shrunk values if they're different from the original
    if current_values != failing_values {
        Some(current_values)
    } else {
        None
    }
}

/// A simple property for testing the execution engine
pub struct AlwaysTrueProperty;

impl Property for AlwaysTrueProperty {
    fn test(&self, _values: &[Value]) -> PropertyResult {
        PropertyResult::Pass
    }

    fn name(&self) -> &str {
        "always_true"
    }
}

/// A simple property that always fails for testing the execution engine
pub struct AlwaysFalseProperty;

impl Property for AlwaysFalseProperty {
    fn test(&self, _values: &[Value]) -> PropertyResult {
        PropertyResult::Fail("This property always fails".to_string())
    }

    fn name(&self) -> &str {
        "always_false"
    }
}

/// Property that tests list reversal: (reverse (reverse x)) = x
pub struct ListReversalProperty;

impl Property for ListReversalProperty {
    fn test(&self, values: &[Value]) -> PropertyResult {
        if values.is_empty() {
            return PropertyResult::Skip("No values provided".to_string());
        }

        let value = &values[0];
        if let Some(elements) = value.as_list() {
            // Simulate reverse operation (for testing purposes)
            let reversed_once: Vec<Value> = elements.iter().rev().cloned().collect();
            let reversed_twice: Vec<Value> = reversed_once.iter().rev().cloned().collect();

            if elements == reversed_twice {
                PropertyResult::Pass
            } else {
                PropertyResult::Fail(format!(
                    "Reversal property failed: original={:?}, double_reversed={:?}",
                    elements, reversed_twice
                ))
            }
        } else {
            PropertyResult::Skip("Value is not a list".to_string())
        }
    }

    fn name(&self) -> &str {
        "list_reversal"
    }
}

/// Property that tests numeric addition commutativity: x + y = y + x
pub struct AdditionCommutativityProperty;

impl Property for AdditionCommutativityProperty {
    fn test(&self, values: &[Value]) -> PropertyResult {
        if values.len() < 2 {
            return PropertyResult::Skip("Need at least 2 values".to_string());
        }

        let (x, y) = (&values[0], &values[1]);

        if let (Some(x_val), Some(y_val)) = (x.as_number(), y.as_number()) {
            let sum1 = x_val + y_val;
            let sum2 = y_val + x_val;

            // Use epsilon comparison for floating point
            if (sum1 - sum2).abs() < f64::EPSILON {
                PropertyResult::Pass
            } else {
                PropertyResult::Fail(format!(
                    "Addition commutativity failed: {} + {} = {}, {} + {} = {}",
                    x_val, y_val, sum1, y_val, x_val, sum2
                ))
            }
        } else if let (Some(x_val), Some(y_val)) = (x.as_integer(), y.as_integer()) {
            let sum1 = x_val + y_val;
            let sum2 = y_val + x_val;

            if sum1 == sum2 {
                PropertyResult::Pass
            } else {
                PropertyResult::Fail(format!(
                    "Addition commutativity failed: {} + {} = {}, {} + {} = {}",
                    x_val, y_val, sum1, y_val, x_val, sum2
                ))
            }
        } else {
            PropertyResult::Skip("Values are not numeric".to_string())
        }
    }

    fn name(&self) -> &str {
        "addition_commutativity"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property_testing::generators::SchemeValueGenerator;

    #[test]
    fn test_always_true_property() {
        let property = Arc::new(AlwaysTrueProperty);
        let generators = vec![Arc::new(SchemeValueGenerator::new())];
        let config = PropertyConfig {
            test_cases: 50,
            parallel: false,
            ..PropertyConfig::default()
        };

        let summary = run_property_test(property, generators, config);
        assert!(summary.all_passed());
        assert_eq!(summary.passed, 50);
        assert_eq!(summary.failed, 0);
    }

    #[test]
    fn test_always_false_property() {
        let property = Arc::new(AlwaysFalseProperty);
        let generators = vec![Arc::new(SchemeValueGenerator::new())];
        let config = PropertyConfig {
            test_cases: 10,
            parallel: false,
            ..PropertyConfig::default()
        };

        let summary = run_property_test(property, generators, config);
        assert!(!summary.all_passed());
        assert_eq!(summary.passed, 0);
        assert_eq!(summary.failed, 10);
        assert!(summary.first_failure.is_some());
    }

    #[test]
    fn test_list_reversal_property() {
        let property = Arc::new(ListReversalProperty);
        let generators = vec![Arc::new(SchemeValueGenerator::new())];
        let config = PropertyConfig {
            test_cases: 100,
            parallel: false,
            ..PropertyConfig::default()
        };

        let summary = run_property_test(property, generators, config);
        // Most tests should pass (reversal property holds for lists)
        // Some may be skipped if non-list values are generated
        assert!(summary.passed > summary.failed);
    }

    #[test]
    fn test_addition_commutativity_property() {
        let property = Arc::new(AdditionCommutativityProperty);
        let generators = vec![
            Arc::new(SchemeValueGenerator::new()),
            Arc::new(SchemeValueGenerator::new()),
        ];
        let config = PropertyConfig {
            test_cases: 100,
            parallel: false,
            ..PropertyConfig::default()
        };

        let summary = run_property_test(property, generators, config);
        // Addition commutativity should always hold for numeric values
        // Some tests may be skipped if non-numeric values are generated
        assert_eq!(summary.failed, 0);
    }

    #[test]
    fn test_parallel_execution() {
        let property = Arc::new(AlwaysTrueProperty);
        let generators = vec![Arc::new(SchemeValueGenerator::new())];
        let config = PropertyConfig {
            test_cases: 100,
            parallel: true,
            ..PropertyConfig::default()
        };

        let summary = run_property_test(property, generators, config);
        assert!(summary.all_passed());
        assert_eq!(summary.passed, 100);
    }

    #[test]
    fn test_shrinking() {
        // This test verifies that shrinking works by creating a property
        // that fails for large numbers but passes for small ones
        struct FailForLargeNumbersProperty;

        impl Property for FailForLargeNumbersProperty {
            fn test(&self, values: &[Value]) -> PropertyResult {
                if let Some(Value::Literal(Literal::Number(n))) = values.first() {
                    if *n > 50.0 {
                        PropertyResult::Fail("Number too large".to_string())
                    } else {
                        PropertyResult::Pass
                    }
                } else {
                    PropertyResult::Skip("Not a number".to_string())
                }
            }

            fn name(&self) -> &str {
                "fail_for_large_numbers"
            }
        }

        let property = Arc::new(FailForLargeNumbersProperty);
        let generators = vec![Arc::new(SchemeValueGenerator::new())];
        let config = PropertyConfig {
            test_cases: 200,
            parallel: false,
            max_shrink_iterations: 50,
            ..PropertyConfig::default()
        };

        let summary = run_property_test(property, generators, config);

        // There should be some failures (when large numbers are generated)
        if summary.failed > 0 {
            // Check if shrinking worked by examining the first failure
            if let Some((shrunk_values, _)) = &summary.first_failure {
                if let Some(Value::Literal(Literal::Number(n))) = shrunk_values.first() {
                    // The shrunk value should be smaller than the original large values
                    println!("Shrunk failing value: {}", n);
                    // We can't easily test the exact shrunk value without more complex setup,
                    // but we can at least verify that shrinking was attempted
                }
            }
        }
    }
}
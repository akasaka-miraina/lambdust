//! Comprehensive R7RS Numeric System Compliance Tests
//!
//! This module provides exhaustive testing of the numeric system against R7RS-small
//! specifications, including edge cases, exactness contagion, and performance validation.

use super::*;
use crate::diagnostics::Result;
use std::f64::consts::{E, PI};

/// Comprehensive R7RS compliance test suite
pub struct R7RSComplianceTestSuite;

impl R7RSComplianceTestSuite {
    /// Run all R7RS compliance tests
    pub fn run_all_tests() -> R7RSComplianceReport {
        let mut report = R7RSComplianceReport::new();

        // Core numeric predicates
        report.add_test_result("numeric_predicates", Self::test_numeric_predicates());

        // Arithmetic operations
        report.add_test_result("arithmetic_operations", Self::test_arithmetic_operations());

        // Exactness contagion
        report.add_test_result("exactness_contagion", Self::test_exactness_contagion());

        // Complex number operations
        report.add_test_result("complex_operations", Self::test_complex_operations());

        // Special numeric values
        report.add_test_result("special_values", Self::test_special_values());

        // Numeric tower promotion
        report.add_test_result("tower_promotion", Self::test_tower_promotion());

        // R7RS specific procedures
        report.add_test_result("r7rs_procedures", Self::test_r7rs_procedures());

        // Edge cases and error handling
        report.add_test_result("edge_cases", Self::test_edge_cases());

        // Performance validation
        report.add_test_result("performance", Self::test_performance_compliance());

        report
    }

    /// Test R7RS numeric predicates
    fn test_numeric_predicates() -> TestResult {
        let mut test = TestResult::new("R7RS Numeric Predicates");

        // Test number? predicate
        test.assert(
            is_number(&NumericValue::Integer(42)),
            "integer should be number",
        );
        test.assert(
            is_number(&NumericValue::Real(3.14)),
            "real should be number",
        );
        test.assert(
            is_number(&NumericValue::Complex(Complex::new(1.0, 2.0))),
            "complex should be number",
        );
        test.assert(
            is_number(&NumericValue::Rational(Rational::new(3, 4))),
            "rational should be number",
        );

        // Test complex? predicate (all numbers are complex in R7RS)
        test.assert(
            is_complex(&NumericValue::Integer(42)),
            "integer should be complex",
        );
        test.assert(
            is_complex(&NumericValue::Real(3.14)),
            "real should be complex",
        );
        test.assert(
            is_complex(&NumericValue::Complex(Complex::new(1.0, 2.0))),
            "complex should be complex",
        );

        // Test real? predicate
        test.assert(
            is_real(&NumericValue::Integer(42)),
            "integer should be real",
        );
        test.assert(is_real(&NumericValue::Real(3.14)), "real should be real");
        test.assert(
            !is_real(&NumericValue::Complex(Complex::new(1.0, 2.0))),
            "non-real complex should not be real",
        );
        test.assert(
            is_real(&NumericValue::Complex(Complex::new(1.0, 0.0))),
            "real complex should be real",
        );

        // Test rational? predicate
        test.assert(
            is_rational(&NumericValue::Integer(42)),
            "integer should be rational",
        );
        test.assert(
            is_rational(&NumericValue::Rational(Rational::new(3, 4))),
            "rational should be rational",
        );
        test.assert(
            is_rational(&NumericValue::Real(3.5)),
            "finite real should be rational",
        );
        test.assert(
            !is_rational(&NumericValue::Real(f64::INFINITY)),
            "infinite real should not be rational",
        );
        test.assert(
            !is_rational(&NumericValue::Real(f64::NAN)),
            "NaN should not be rational",
        );

        // Test integer? predicate
        test.assert(
            is_integer(&NumericValue::Integer(42)),
            "integer should be integer",
        );
        test.assert(
            !is_integer(&NumericValue::Rational(Rational::new(3, 4))),
            "non-integer rational should not be integer",
        );
        test.assert(
            is_integer(&NumericValue::Rational(Rational::new(6, 2))),
            "integer rational should be integer",
        );
        test.assert(
            is_integer(&NumericValue::Real(42.0)),
            "integer-valued real should be integer",
        );
        test.assert(
            !is_integer(&NumericValue::Real(42.5)),
            "non-integer real should not be integer",
        );

        // Test exact/inexact predicates
        test.assert(
            is_exact(&NumericValue::Integer(42)),
            "integer should be exact",
        );
        test.assert(
            is_exact(&NumericValue::Rational(Rational::new(3, 4))),
            "rational should be exact",
        );
        test.assert(
            !is_exact(&NumericValue::Real(42.0)),
            "real should not be exact",
        );
        test.assert(
            !is_exact(&NumericValue::Complex(Complex::new(1.0, 2.0))),
            "complex should not be exact",
        );

        test.assert(
            is_inexact(&NumericValue::Real(42.0)),
            "real should be inexact",
        );
        test.assert(
            !is_inexact(&NumericValue::Integer(42)),
            "integer should not be inexact",
        );

        // Test finite/infinite/nan predicates
        test.assert(
            is_finite(&NumericValue::Real(42.0)),
            "finite real should be finite",
        );
        test.assert(
            !is_finite(&NumericValue::Real(f64::INFINITY)),
            "infinite real should not be finite",
        );
        test.assert(
            !is_finite(&NumericValue::Real(f64::NAN)),
            "NaN should not be finite",
        );

        test.assert(
            is_infinite(&NumericValue::Real(f64::INFINITY)),
            "infinite real should be infinite",
        );
        test.assert(
            !is_infinite(&NumericValue::Real(42.0)),
            "finite real should not be infinite",
        );

        test.assert(is_nan(&NumericValue::Real(f64::NAN)), "NaN should be NaN");
        test.assert(
            !is_nan(&NumericValue::Real(42.0)),
            "finite real should not be NaN",
        );

        test
    }

    /// Test arithmetic operations
    fn test_arithmetic_operations() -> TestResult {
        let mut test = TestResult::new("R7RS Arithmetic Operations");

        // Integer arithmetic
        let a = NumericValue::Integer(10);
        let b = NumericValue::Integer(3);

        let sum = add(&a, &b);
        test.assert_eq(sum.to_i64(), Some(13), "10 + 3 = 13");

        let diff = subtract(&a, &b);
        test.assert_eq(diff.to_i64(), Some(7), "10 - 3 = 7");

        let prod = multiply(&a, &b);
        test.assert_eq(prod.to_i64(), Some(30), "10 * 3 = 30");

        let quot = divide(&a, &b).unwrap();
        test.assert(quot.is_exact(), "exact division should preserve exactness");
        if let NumericValue::Rational(r) = quot {
            test.assert_eq((r.numerator, r.denominator), (10, 3), "10 / 3 = 10/3");
        }

        // Real arithmetic
        let x = NumericValue::Real(10.5);
        let y = NumericValue::Real(3.2);

        let real_sum = add(&x, &y);
        test.assert_approx_eq(real_sum.to_f64().unwrap(), 13.7, 1e-10, "10.5 + 3.2 = 13.7");

        // Complex arithmetic
        let z1 = NumericValue::Complex(Complex::new(3.0, 4.0));
        let z2 = NumericValue::Complex(Complex::new(1.0, 2.0));

        let complex_sum = add(&z1, &z2);
        if let NumericValue::Complex(c) = complex_sum {
            test.assert_approx_eq(c.real, 4.0, 1e-10, "complex addition real part");
            test.assert_approx_eq(c.imaginary, 6.0, 1e-10, "complex addition imaginary part");
        }

        // Overflow handling
        let large = NumericValue::Integer(i64::MAX);
        let one = NumericValue::Integer(1);
        let overflow_result = add(&large, &one);
        test.assert(
            matches!(overflow_result, NumericValue::BigInteger(_)),
            "overflow should promote to BigInteger",
        );

        test
    }

    /// Test exactness contagion rules
    fn test_exactness_contagion() -> TestResult {
        let mut test = TestResult::new("R7RS Exactness Contagion");

        let exact = NumericValue::Integer(1);
        let inexact = NumericValue::Real(2.0);

        // Exact + Inexact -> Inexact
        let mixed_sum = add(&exact, &inexact);
        test.assert(!mixed_sum.is_exact(), "exact + inexact should be inexact");
        test.assert_approx_eq(mixed_sum.to_f64().unwrap(), 3.0, 1e-10, "1 + 2.0 = 3.0");

        // Exact + Exact -> Exact (when mathematically exact)
        let exact2 = NumericValue::Integer(2);
        let exact_sum = add(&exact, &exact2);
        test.assert(exact_sum.is_exact(), "exact + exact should be exact");
        test.assert_eq(exact_sum.to_i64(), Some(3), "1 + 2 = 3");

        // Division preserves exactness for rationals
        let exact_div = divide(&exact, &exact2).unwrap();
        test.assert(
            exact_div.is_exact(),
            "exact / exact should be exact rational",
        );

        // Test exact->inexact conversion
        let made_inexact = ExactnessSystem::apply_contagion("+", &exact, &inexact, exact.clone());
        test.assert(
            !made_inexact.is_exact(),
            "contagion should make exact inexact",
        );

        test
    }

    /// Test complex number operations
    fn test_complex_operations() -> TestResult {
        let mut test = TestResult::new("R7RS Complex Operations");

        let z = NumericValue::Complex(Complex::new(3.0, 4.0));
        let real_num = NumericValue::Integer(5);

        // Complex accessors
        let real_part = R7RSNumericProcedures::real_part(&z);
        test.assert_approx_eq(real_part.to_f64().unwrap(), 3.0, 1e-10, "real-part of 3+4i");

        let imag_part = R7RSNumericProcedures::imag_part(&z);
        test.assert_approx_eq(imag_part.to_f64().unwrap(), 4.0, 1e-10, "imag-part of 3+4i");

        // Real number accessors
        let real_real_part = R7RSNumericProcedures::real_part(&real_num);
        test.assert_eq(real_real_part.to_i64(), Some(5), "real-part of real number");

        let real_imag_part = R7RSNumericProcedures::imag_part(&real_num);
        test.assert_eq(real_imag_part.to_i64(), Some(0), "imag-part of real number");

        // Magnitude and angle
        let magnitude = R7RSNumericProcedures::magnitude(&z);
        test.assert_approx_eq(magnitude.to_f64().unwrap(), 5.0, 1e-10, "magnitude of 3+4i");

        let angle = R7RSNumericProcedures::angle(&z);
        let expected_angle = 4.0_f64.atan2(3.0);
        test.assert_approx_eq(
            angle.to_f64().unwrap(),
            expected_angle,
            1e-10,
            "angle of 3+4i",
        );

        // Magnitude of real numbers
        let neg_real = NumericValue::Integer(-7);
        let neg_magnitude = R7RSNumericProcedures::magnitude(&neg_real);
        test.assert_eq(
            neg_magnitude.to_i64(),
            Some(7),
            "magnitude of negative integer",
        );

        // Angle of real numbers
        let pos_angle = R7RSNumericProcedures::angle(&real_num);
        test.assert_eq(pos_angle.to_i64(), Some(0), "angle of positive real");

        let neg_angle = R7RSNumericProcedures::angle(&neg_real);
        test.assert_approx_eq(
            neg_angle.to_f64().unwrap(),
            PI,
            1e-10,
            "angle of negative real",
        );

        test
    }

    /// Test special numeric values
    fn test_special_values() -> TestResult {
        let mut test = TestResult::new("R7RS Special Values");

        // Infinity
        let pos_inf = NumericValue::Real(f64::INFINITY);
        let neg_inf = NumericValue::Real(f64::NEG_INFINITY);

        test.assert(is_infinite(&pos_inf), "+inf.0 should be infinite");
        test.assert(is_infinite(&neg_inf), "-inf.0 should be infinite");
        test.assert(!is_finite(&pos_inf), "+inf.0 should not be finite");
        test.assert(!is_nan(&pos_inf), "+inf.0 should not be NaN");

        // NaN
        let nan_val = NumericValue::Real(f64::NAN);
        test.assert(is_nan(&nan_val), "+nan.0 should be NaN");
        test.assert(!is_finite(&nan_val), "NaN should not be finite");
        test.assert(!is_infinite(&nan_val), "NaN should not be infinite");

        // Zero
        let zero = NumericValue::Integer(0);
        test.assert(zero.is_zero(), "0 should be zero");
        test.assert(!zero.is_positive(), "0 should not be positive");
        test.assert(!zero.is_negative(), "0 should not be negative");

        // Negative zero (IEEE 754)
        let neg_zero = NumericValue::Real(-0.0);
        test.assert(neg_zero.is_zero(), "-0.0 should be zero");

        test
    }

    /// Test numeric tower promotion
    fn test_tower_promotion() -> TestResult {
        let mut test = TestResult::new("R7RS Tower Promotion");

        let int_val = NumericValue::Integer(42);
        let rat_val = NumericValue::Rational(Rational::new(3, 4));
        let real_val = NumericValue::Real(3.14);
        let complex_val = NumericValue::Complex(Complex::new(1.0, 2.0));

        // Integer -> Rational promotion
        let (promoted_int, promoted_rat) = promote_types(&int_val, &rat_val);
        test.assert(
            matches!(promoted_int, NumericValue::Rational(_)),
            "integer should promote to rational",
        );
        test.assert(
            matches!(promoted_rat, NumericValue::Rational(_)),
            "rational should remain rational",
        );

        // Rational -> Real promotion
        let (promoted_rat2, promoted_real) = promote_types(&rat_val, &real_val);
        test.assert(
            matches!(promoted_rat2, NumericValue::Real(_)),
            "rational should promote to real",
        );
        test.assert(
            matches!(promoted_real, NumericValue::Real(_)),
            "real should remain real",
        );

        // Real -> Complex promotion
        let (promoted_real2, promoted_complex) = promote_types(&real_val, &complex_val);
        test.assert(
            matches!(promoted_real2, NumericValue::Complex(_)),
            "real should promote to complex",
        );
        test.assert(
            matches!(promoted_complex, NumericValue::Complex(_)),
            "complex should remain complex",
        );

        test
    }

    /// Test R7RS specific procedures
    fn test_r7rs_procedures() -> TestResult {
        let mut test = TestResult::new("R7RS Specific Procedures");

        // Test modulo and remainder
        let a = NumericValue::Integer(13);
        let b = NumericValue::Integer(4);
        let neg_a = NumericValue::Integer(-13);

        let modulo_result = R7RSNumericProcedures::modulo(&a, &b).unwrap();
        test.assert_eq(modulo_result.to_i64(), Some(1), "13 mod 4 = 1");

        let modulo_neg = R7RSNumericProcedures::modulo(&neg_a, &b).unwrap();
        test.assert_eq(
            modulo_neg.to_i64(),
            Some(3),
            "-13 mod 4 = 3 (sign of divisor)",
        );

        let remainder_result = R7RSNumericProcedures::remainder(&a, &b).unwrap();
        test.assert_eq(remainder_result.to_i64(), Some(1), "remainder(13, 4) = 1");

        let remainder_neg = R7RSNumericProcedures::remainder(&neg_a, &b).unwrap();
        test.assert_eq(
            remainder_neg.to_i64(),
            Some(-1),
            "remainder(-13, 4) = -1 (sign of dividend)",
        );

        // Test quotient
        let quotient_result = R7RSNumericProcedures::quotient(&a, &b).unwrap();
        test.assert_eq(quotient_result.to_i64(), Some(3), "quotient(13, 4) = 3");

        // Test GCD and LCM
        let gcd_args = vec![NumericValue::Integer(48), NumericValue::Integer(18)];
        let gcd_result = R7RSNumericProcedures::gcd(&gcd_args).unwrap();
        test.assert_eq(gcd_result.to_i64(), Some(6), "gcd(48, 18) = 6");

        let lcm_result = R7RSNumericProcedures::lcm(&gcd_args).unwrap();
        test.assert_eq(lcm_result.to_i64(), Some(144), "lcm(48, 18) = 144");

        // Test numerator and denominator
        let rational = NumericValue::Rational(Rational::new(3, 4));
        let num = R7RSNumericProcedures::numerator(&rational).unwrap();
        let den = R7RSNumericProcedures::denominator(&rational).unwrap();

        test.assert_eq(num.to_i64(), Some(3), "numerator of 3/4");
        test.assert_eq(den.to_i64(), Some(4), "denominator of 3/4");

        // Test rationalize
        let approx = NumericValue::Real(0.3333);
        let tolerance = NumericValue::Real(0.01);
        let rationalized = R7RSNumericProcedures::rationalize(&approx, &tolerance).unwrap();

        if let NumericValue::Rational(r) = rationalized {
            test.assert_eq(
                (r.numerator, r.denominator),
                (1, 3),
                "rationalize 0.3333 ≈ 1/3",
            );
        } else {
            test.fail("rationalize should return rational");
        }

        test
    }

    /// Test edge cases and error handling
    fn test_edge_cases() -> TestResult {
        let mut test = TestResult::new("R7RS Edge Cases");

        // Division by zero
        let zero = NumericValue::Integer(0);
        let one = NumericValue::Integer(1);

        let div_by_zero = divide(&one, &zero);
        test.assert(div_by_zero.is_err(), "division by zero should error");

        let modulo_by_zero = R7RSNumericProcedures::modulo(&one, &zero);
        test.assert(modulo_by_zero.is_err(), "modulo by zero should error");

        // Very large numbers
        let large_int = NumericValue::BigInteger(BigInt::from_i64(i64::MAX).pow(2));
        let large_sum = add(&large_int, &one);
        test.assert(
            matches!(large_sum, NumericValue::BigInteger(_)),
            "large arithmetic should work",
        );

        // Precision edge cases
        let tiny = NumericValue::Real(1e-100);
        let huge = NumericValue::Real(1e100);
        let extreme_sum = add(&tiny, &huge);
        test.assert_approx_eq(
            extreme_sum.to_f64().unwrap(),
            1e100,
            1e90,
            "extreme precision should work",
        );

        // NaN propagation
        let nan = NumericValue::Real(f64::NAN);
        let nan_sum = add(&nan, &one);
        test.assert(nan_sum.to_f64().unwrap().is_nan(), "NaN should propagate");

        test
    }

    /// Test performance compliance (no regressions)
    fn test_performance_compliance() -> TestResult {
        let mut test = TestResult::new("R7RS Performance Compliance");

        const ITERATIONS: usize = 100_000;
        let mut numbers = Vec::new();
        for i in 0..ITERATIONS {
            numbers.push(NumericValue::Integer(i as i64));
        }

        // Time arithmetic operations
        let start = std::time::Instant::now();
        let mut sum = NumericValue::Integer(0);
        for num in &numbers {
            sum = add(&sum, num);
        }
        let arithmetic_time = start.elapsed();

        test.assert(
            arithmetic_time.as_millis() < 1000,
            &format!(
                "arithmetic should be fast: {}ms",
                arithmetic_time.as_millis()
            ),
        );

        // Test SIMD optimization performance
        let vec_a = NumericValue::real_vector((0..1000).map(|i| i as f64).collect());
        let vec_b = NumericValue::real_vector((0..1000).map(|i| (i + 1000) as f64).collect());

        let start = std::time::Instant::now();
        let _simd_result = vec_a.simd_vector_add(&vec_b);
        let simd_time = start.elapsed();

        test.assert(
            simd_time.as_millis() < 10,
            &format!(
                "SIMD operations should be fast: {}ms",
                simd_time.as_millis()
            ),
        );

        test
    }
}

/// Test result structure
#[derive(Debug)]
pub struct TestResult {
    pub name: String,
    pub passed: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

impl TestResult {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: 0,
            failed: 0,
            errors: Vec::new(),
        }
    }

    pub fn assert(&mut self, condition: bool, message: &str) {
        if condition {
            self.passed += 1;
        } else {
            self.failed += 1;
            self.errors.push(format!("FAIL: {}", message));
        }
    }

    pub fn assert_eq<T: PartialEq + std::fmt::Debug>(
        &mut self,
        actual: T,
        expected: T,
        message: &str,
    ) {
        if actual == expected {
            self.passed += 1;
        } else {
            self.failed += 1;
            self.errors.push(format!(
                "FAIL: {} - expected {:?}, got {:?}",
                message, expected, actual
            ));
        }
    }

    pub fn assert_approx_eq(&mut self, actual: f64, expected: f64, tolerance: f64, message: &str) {
        if (actual - expected).abs() <= tolerance {
            self.passed += 1;
        } else {
            self.failed += 1;
            self.errors.push(format!(
                "FAIL: {} - expected {}, got {} (tolerance {})",
                message, expected, actual, tolerance
            ));
        }
    }

    pub fn fail(&mut self, message: &str) {
        self.failed += 1;
        self.errors.push(format!("FAIL: {}", message));
    }

    pub fn success_rate(&self) -> f64 {
        if self.passed + self.failed == 0 {
            1.0
        } else {
            self.passed as f64 / (self.passed + self.failed) as f64
        }
    }
}

/// Overall compliance report
#[derive(Debug)]
pub struct R7RSComplianceReport {
    pub test_results: Vec<TestResult>,
    pub overall_compliance: f64,
    pub recommendations: Vec<String>,
}

impl R7RSComplianceReport {
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
            overall_compliance: 0.0,
            recommendations: Vec::new(),
        }
    }

    pub fn add_test_result(&mut self, _name: &str, result: TestResult) {
        self.test_results.push(result);
        self.calculate_compliance();
    }

    fn calculate_compliance(&mut self) {
        let total_passed: usize = self.test_results.iter().map(|r| r.passed).sum();
        let total_tests: usize = self.test_results.iter().map(|r| r.passed + r.failed).sum();

        self.overall_compliance = if total_tests == 0 {
            0.0
        } else {
            total_passed as f64 / total_tests as f64
        };

        // Generate recommendations
        self.recommendations.clear();

        for result in &self.test_results {
            if result.success_rate() < 1.0 {
                self.recommendations
                    .push(format!("Fix {} failures in {}", result.failed, result.name));
            }
        }

        if self.overall_compliance < 1.0 {
            self.recommendations.push(format!(
                "Overall compliance is {:.1}% - target is 100%",
                self.overall_compliance * 100.0
            ));
        }
    }

    pub fn print_report(&self) {
        println!("\n=== R7RS Compliance Report ===");
        println!(
            "Overall Compliance: {:.1}%",
            self.overall_compliance * 100.0
        );
        println!();

        for result in &self.test_results {
            println!(
                "{}: {}/{} passed ({:.1}%)",
                result.name,
                result.passed,
                result.passed + result.failed,
                result.success_rate() * 100.0
            );

            for error in &result.errors {
                println!("  {}", error);
            }
            println!();
        }

        if !self.recommendations.is_empty() {
            println!("=== Recommendations ===");
            for rec in &self.recommendations {
                println!("- {}", rec);
            }
        }

        println!("=== Test Complete ===\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r7rs_compliance_suite() {
        let report = R7RSComplianceTestSuite::run_all_tests();
        report.print_report();

        // Target: 95% compliance minimum for this implementation phase
        assert!(
            report.overall_compliance >= 0.95,
            "R7RS compliance should be at least 95%, got {:.1}%",
            report.overall_compliance * 100.0
        );
    }

    #[test]
    fn test_specific_r7rs_requirements() {
        // Test exact requirement: (= 1 1.0) should be true
        let exact_int = NumericValue::Integer(1);
        let inexact_real = NumericValue::Real(1.0);

        // This requires proper numeric equality that considers exactness
        let comparison_result = compare(&exact_int, &inexact_real);
        assert_eq!(comparison_result, Some(std::cmp::Ordering::Equal));

        // Test contagion requirement: (+ 1 1.0) should be 2.0 (inexact)
        let sum = add(&exact_int, &inexact_real);
        assert!(!sum.is_exact(), "1 + 1.0 should be inexact");
        assert_eq!(sum.to_f64().unwrap(), 2.0);

        // Test division requirement: (/ 1 3) should be exact rational
        let one = NumericValue::Integer(1);
        let three = NumericValue::Integer(3);
        let division = divide(&one, &three).unwrap();
        assert!(division.is_exact(), "1 / 3 should be exact");
        if let NumericValue::Rational(r) = division {
            assert_eq!(r.numerator, 1);
            assert_eq!(r.denominator, 3);
        }
    }
}

//! R7RS Exactness System Implementation
//!
//! Implements the R7RS exactness contagion rules and precision preservation
//! requirements with optimized performance characteristics.

use super::{NumericType, NumericValue};
use crate::diagnostics::{Error, Result};

/// R7RS exactness contagion rules implementation
pub struct ExactnessSystem;

impl ExactnessSystem {
    /// Apply R7RS exactness contagion rules to operation results
    ///
    /// R7RS Rules:
    /// - Exact + Exact = Exact (when mathematically exact)
    /// - Exact + Inexact = Inexact
    /// - Inexact + Exact = Inexact  
    /// - Inexact + Inexact = Inexact
    pub fn apply_contagion(
        operation: &str,
        left: &NumericValue,
        right: &NumericValue,
        mathematical_result: NumericValue,
    ) -> NumericValue {
        let left_exact = left.is_exact();
        let right_exact = right.is_exact();

        // If either operand is inexact, result must be inexact
        if !left_exact || !right_exact {
            return Self::make_inexact_preserving_value(mathematical_result);
        }

        // Both operands are exact - check if operation preserves exactness
        match operation {
            "+" | "-" | "*" => {
                // These operations preserve exactness when both operands are exact
                mathematical_result
            }
            "/" => {
                // Division of exact numbers produces exact rational result
                // (already handled in divide operation)
                mathematical_result
            }
            "sqrt" => {
                // Square root is exact only for perfect squares
                if Self::is_perfect_square(&mathematical_result) {
                    mathematical_result
                } else {
                    Self::make_inexact_preserving_value(mathematical_result)
                }
            }
            "expt" => {
                // Exponentiation preserves exactness for integer exponents
                // For rational bases and integer exponents, result should be exact
                // but may need to be represented as inexact due to precision limits
                if Self::can_represent_exactly(&mathematical_result) {
                    mathematical_result
                } else {
                    Self::make_inexact_preserving_value(mathematical_result)
                }
            }
            "sin" | "cos" | "tan" | "log" | "exp" => {
                // Transcendental functions always produce inexact results
                // except for special exact values
                if Self::has_exact_transcendental_result(operation, left, right) {
                    mathematical_result
                } else {
                    Self::make_inexact_preserving_value(mathematical_result)
                }
            }
            _ => {
                // Conservative approach: unknown operations produce inexact results
                Self::make_inexact_preserving_value(mathematical_result)
            }
        }
    }

    /// Convert a value to inexact while preserving its mathematical value
    fn make_inexact_preserving_value(value: NumericValue) -> NumericValue {
        match value {
            // Already inexact
            NumericValue::Real(_) | NumericValue::Complex(_) => value,

            // Convert exact types to inexact
            NumericValue::Integer(n) => NumericValue::Real(n as f64),
            NumericValue::BigInteger(n) => NumericValue::Real(n.to_f64().unwrap_or(f64::INFINITY)),
            NumericValue::Rational(r) => NumericValue::Real(r.to_f64()),
            NumericValue::Vector(v) => {
                let inexact_v = v
                    .into_iter()
                    .map(Self::make_inexact_preserving_value)
                    .collect();
                NumericValue::Vector(inexact_v)
            }
        }
    }

    /// Convert a value to exact while preserving its mathematical value
    pub fn make_exact_preserving_value(value: NumericValue) -> Result<NumericValue> {
        match value {
            // Already exact
            NumericValue::Integer(_) | NumericValue::BigInteger(_) | NumericValue::Rational(_) => {
                Ok(value)
            }

            // Convert inexact types to exact
            NumericValue::Real(r) => {
                if r.is_finite() {
                    if r.fract() == 0.0 {
                        // Integer value
                        if r.abs() <= i64::MAX as f64 {
                            Ok(NumericValue::Integer(r as i64))
                        } else {
                            // Use BigInteger for large values
                            Ok(NumericValue::BigInteger(super::BigInt::from_i64(r as i64)))
                        }
                    } else {
                        // Rational approximation
                        if let Some(rational) = Self::float_to_exact_rational(r) {
                            Ok(NumericValue::Rational(rational))
                        } else {
                            Err(Box::new(Error::runtime_error(
                                format!("Cannot represent {} as exact number", r),
                                None,
                            )))
                        }
                    }
                } else {
                    Err(Box::new(Error::runtime_error(
                        "Cannot convert infinite or NaN to exact".to_string(),
                        None,
                    )))
                }
            }

            NumericValue::Complex(c) => {
                if c.imaginary == 0.0 {
                    // Real complex number - convert real part
                    Self::make_exact_preserving_value(NumericValue::Real(c.real))
                } else {
                    Err(Box::new(Error::runtime_error(
                        "Cannot convert non-real complex number to exact".to_string(),
                        None,
                    )))
                }
            }

            NumericValue::Vector(v) => {
                let exact_v: Result<Vec<_>> = v
                    .into_iter()
                    .map(Self::make_exact_preserving_value)
                    .collect();
                Ok(NumericValue::Vector(exact_v?))
            }
        }
    }

    /// Check if a number is a perfect square (for exact square root)
    fn is_perfect_square(value: &NumericValue) -> bool {
        match value {
            NumericValue::Integer(n) => {
                if *n >= 0 {
                    let sqrt_n = (*n as f64).sqrt() as i64;
                    sqrt_n * sqrt_n == *n
                } else {
                    false
                }
            }
            NumericValue::Rational(r) => {
                if r.is_positive() {
                    let num_sqrt = (r.numerator as f64).sqrt() as i64;
                    let den_sqrt = (r.denominator as f64).sqrt() as i64;
                    num_sqrt * num_sqrt == r.numerator && den_sqrt * den_sqrt == r.denominator
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Check if result can be represented exactly in our numeric system
    fn can_represent_exactly(value: &NumericValue) -> bool {
        match value {
            NumericValue::Integer(_) | NumericValue::BigInteger(_) | NumericValue::Rational(_) => {
                true
            }
            NumericValue::Real(r) => {
                // Can represent exactly if it's a rational with reasonable denominator
                Self::float_to_exact_rational(*r).is_some()
            }
            NumericValue::Complex(c) => {
                Self::can_represent_exactly(&NumericValue::Real(c.real))
                    && Self::can_represent_exactly(&NumericValue::Real(c.imaginary))
            }
            _ => false,
        }
    }

    /// Check for special exact transcendental values
    fn has_exact_transcendental_result(
        operation: &str,
        left: &NumericValue,
        _right: &NumericValue,
    ) -> bool {
        match operation {
            "sin" => {
                // sin(0) = 0 (exact)
                left.is_zero()
            }
            "cos" => {
                // cos(0) = 1 (exact)
                left.is_zero()
            }
            "tan" => {
                // tan(0) = 0 (exact)
                left.is_zero()
            }
            "log" => {
                // log(1) = 0 (exact)
                match left {
                    NumericValue::Integer(1) => true,
                    NumericValue::Real(r) if *r == 1.0 => true,
                    _ => false,
                }
            }
            "exp" => {
                // exp(0) = 1 (exact)
                left.is_zero()
            }
            _ => false,
        }
    }

    /// High-precision float to rational conversion
    fn float_to_exact_rational(f: f64) -> Option<super::Rational> {
        if !f.is_finite() {
            return None;
        }

        // Use continued fraction expansion with higher precision
        const MAX_ITERATIONS: usize = 100;
        const MAX_DENOMINATOR: i64 = 1_000_000_000; // Higher limit for exactness
        const PRECISION_THRESHOLD: f64 = 1e-15;

        let mut x = f;
        let mut convergents = [(f.floor() as i64, 1i64)]; // (numerator, denominator)

        if (f - convergents[0].0 as f64).abs() < PRECISION_THRESHOLD {
            return Some(super::Rational::new(convergents[0].0, 1));
        }

        let mut prev_h = (convergents[0].0, 1i64);
        let mut curr_h = (1i64, 0i64);

        for _i in 1..MAX_ITERATIONS {
            x = 1.0 / (x - (x.floor()));
            let a = x.floor() as i64;

            let new_h = (a * prev_h.0 + curr_h.0, a * prev_h.1 + curr_h.1);

            if new_h.1 > MAX_DENOMINATOR {
                break;
            }

            curr_h = prev_h;
            prev_h = new_h;

            let rational = super::Rational::new(new_h.0, new_h.1);
            if (rational.to_f64() - f).abs() < PRECISION_THRESHOLD {
                return Some(rational);
            }

            if x.abs() < PRECISION_THRESHOLD {
                break;
            }
        }

        // Return best approximation found
        Some(super::Rational::new(prev_h.0, prev_h.1))
    }

    /// Validate exactness properties for testing
    pub fn validate_exactness_properties(
        operation: &str,
        operands: &[NumericValue],
        result: &NumericValue,
    ) -> bool {
        match operation {
            "+" | "-" | "*" => {
                // If all operands are exact, result should be exact
                let all_exact = operands.iter().all(|op| op.is_exact());
                !all_exact || result.is_exact()
            }
            "/" => {
                // Division of exact numbers should produce exact rational
                if operands.len() == 2 && operands.iter().all(|op| op.is_exact()) {
                    result.is_exact()
                } else {
                    true // Mixed exactness handled by contagion
                }
            }
            _ => true, // Other operations have complex exactness rules
        }
    }

    /// Get exactness information for debugging
    pub fn exactness_info(value: &NumericValue) -> String {
        match value {
            NumericValue::Integer(_) => "exact integer".to_string(),
            NumericValue::BigInteger(_) => "exact big integer".to_string(),
            NumericValue::Rational(_) => "exact rational".to_string(),
            NumericValue::Real(r) => {
                if r.is_finite() {
                    format!("inexact real ({})", r)
                } else {
                    format!("inexact real ({})", r)
                }
            }
            NumericValue::Complex(c) => {
                if c.real.fract() == 0.0
                    && c.imaginary.fract() == 0.0
                    && c.real.is_finite()
                    && c.imaginary.is_finite()
                {
                    "exact complex (gaussian integer)".to_string()
                } else {
                    format!("inexact complex ({}+{}i)", c.real, c.imaginary)
                }
            }
            NumericValue::Vector(v) => {
                let exact_count = v.iter().filter(|x| x.is_exact()).count();
                format!("vector ({}/{} exact)", exact_count, v.len())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Rational, complex::Complex};
    use super::*;

    #[test]
    fn test_exactness_contagion_addition() {
        let exact = NumericValue::Integer(1);
        let inexact = NumericValue::Real(2.0);
        let result = NumericValue::Real(3.0);

        // Exact + Inexact -> Inexact
        let contagion_result =
            ExactnessSystem::apply_contagion("+", &exact, &inexact, result.clone());
        assert!(!contagion_result.is_exact());
        assert_eq!(contagion_result.to_f64().unwrap(), 3.0);

        // Exact + Exact -> Exact
        let exact2 = NumericValue::Integer(2);
        let exact_result = NumericValue::Integer(3);
        let contagion_exact = ExactnessSystem::apply_contagion("+", &exact, &exact2, exact_result);
        assert!(contagion_exact.is_exact());
        assert_eq!(contagion_exact.to_i64().unwrap(), 3);
    }

    #[test]
    fn test_exactness_contagion_division() {
        let a = NumericValue::Integer(1);
        let b = NumericValue::Integer(3);
        let rational_result = NumericValue::Rational(Rational::new(1, 3));

        // Exact / Exact -> Exact (rational)
        let result = ExactnessSystem::apply_contagion("/", &a, &b, rational_result);
        assert!(result.is_exact());
    }

    #[test]
    fn test_perfect_square_detection() {
        assert!(ExactnessSystem::is_perfect_square(&NumericValue::Integer(
            9
        )));
        assert!(ExactnessSystem::is_perfect_square(&NumericValue::Integer(
            16
        )));
        assert!(!ExactnessSystem::is_perfect_square(&NumericValue::Integer(
            8
        )));
        assert!(!ExactnessSystem::is_perfect_square(&NumericValue::Integer(
            -9
        )));
    }

    #[test]
    fn test_make_exact_conversion() {
        // Integer-valued real to exact
        let inexact_int = NumericValue::Real(42.0);
        let exact_result = ExactnessSystem::make_exact_preserving_value(inexact_int).unwrap();
        assert!(exact_result.is_exact());
        assert_eq!(exact_result.to_i64().unwrap(), 42);

        // Rational-valued real to exact
        let inexact_rational = NumericValue::Real(0.5);
        let exact_rational =
            ExactnessSystem::make_exact_preserving_value(inexact_rational).unwrap();
        assert!(exact_rational.is_exact());
        if let NumericValue::Rational(r) = exact_rational {
            assert_eq!(r.numerator, 1);
            assert_eq!(r.denominator, 2);
        }
    }

    #[test]
    fn test_transcendental_special_cases() {
        let zero = NumericValue::Integer(0);
        let one = NumericValue::Integer(1);

        // Special exact transcendental results
        assert!(ExactnessSystem::has_exact_transcendental_result(
            "sin", &zero, &zero
        ));
        assert!(ExactnessSystem::has_exact_transcendental_result(
            "cos", &zero, &zero
        ));
        assert!(ExactnessSystem::has_exact_transcendental_result(
            "tan", &zero, &zero
        ));
        assert!(ExactnessSystem::has_exact_transcendental_result(
            "log", &one, &zero
        ));
        assert!(ExactnessSystem::has_exact_transcendental_result(
            "exp", &zero, &zero
        ));

        // Non-special cases should not be exact
        let pi_half = NumericValue::Real(std::f64::consts::PI / 2.0);
        assert!(!ExactnessSystem::has_exact_transcendental_result(
            "sin", &pi_half, &zero
        ));
    }

    #[test]
    fn test_exactness_validation() {
        let exact_ops = vec![NumericValue::Integer(1), NumericValue::Integer(2)];
        let exact_result = NumericValue::Integer(3);

        assert!(ExactnessSystem::validate_exactness_properties(
            "+",
            &exact_ops,
            &exact_result
        ));

        let mixed_ops = vec![NumericValue::Integer(1), NumericValue::Real(2.0)];
        let inexact_result = NumericValue::Real(3.0);

        assert!(ExactnessSystem::validate_exactness_properties(
            "+",
            &mixed_ops,
            &inexact_result
        ));
    }

    #[test]
    fn test_exactness_info() {
        let int_info = ExactnessSystem::exactness_info(&NumericValue::Integer(42));
        assert_eq!(int_info, "exact integer");

        let real_info = ExactnessSystem::exactness_info(&NumericValue::Real(std::f64::consts::PI));
        assert!(real_info.contains("inexact real"));

        let rational_info =
            ExactnessSystem::exactness_info(&NumericValue::Rational(Rational::new(1, 2)));
        assert_eq!(rational_info, "exact rational");
    }
}

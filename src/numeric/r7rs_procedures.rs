//! R7RS-compliant numeric procedures
//!
//! This module implements the complete set of numeric procedures required by R7RS-small,
//! ensuring strict compliance with the specification while leveraging performance optimizations.

use super::{BigInt, NumericType, NumericValue, Rational, complex::Complex};
use crate::diagnostics::{Error, Result};
use std::cmp::Ordering;

/// R7RS numeric procedure implementations with strict compliance
pub struct R7RSNumericProcedures;

impl R7RSNumericProcedures {
    /// Implements R7RS `modulo` procedure
    ///
    /// (modulo n1 n2) -> number
    /// Returns the remainder of dividing n1 by n2 with the same sign as n2
    pub fn modulo(dividend: &NumericValue, divisor: &NumericValue) -> Result<NumericValue> {
        if divisor.is_zero() {
            return Err(Box::new(Error::runtime_error(
                "Division by zero in modulo".to_string(),
                None,
            )));
        }

        // R7RS: result has same sign as divisor
        match (dividend, divisor) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => {
                let result = ((a % b) + b) % b;
                Ok(NumericValue::Integer(result))
            }
            (NumericValue::Real(a), NumericValue::Real(b)) => {
                let result = ((a % b) + b) % b;
                Ok(NumericValue::Real(result))
            }
            _ => {
                // Promote to common type and retry
                let (promoted_dividend, promoted_divisor) =
                    super::tower::promote_types(dividend, divisor);
                Self::modulo(&promoted_dividend, &promoted_divisor)
            }
        }
    }

    /// Implements R7RS `remainder` procedure
    ///
    /// (remainder n1 n2) -> number  
    /// Returns the remainder of dividing n1 by n2 with the same sign as n1
    pub fn remainder(dividend: &NumericValue, divisor: &NumericValue) -> Result<NumericValue> {
        if divisor.is_zero() {
            return Err(Box::new(Error::runtime_error(
                "Division by zero in remainder".to_string(),
                None,
            )));
        }

        // R7RS: result has same sign as dividend
        match (dividend, divisor) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => {
                Ok(NumericValue::Integer(a % b))
            }
            (NumericValue::Real(a), NumericValue::Real(b)) => Ok(NumericValue::Real(a % b)),
            _ => {
                let (promoted_dividend, promoted_divisor) =
                    super::tower::promote_types(dividend, divisor);
                Self::remainder(&promoted_dividend, &promoted_divisor)
            }
        }
    }

    /// Implements R7RS `quotient` procedure
    ///
    /// (quotient n1 n2) -> integer
    /// Returns the quotient of dividing n1 by n2, truncated toward zero
    pub fn quotient(dividend: &NumericValue, divisor: &NumericValue) -> Result<NumericValue> {
        if divisor.is_zero() {
            return Err(Box::new(Error::runtime_error(
                "Division by zero in quotient".to_string(),
                None,
            )));
        }

        match (dividend, divisor) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => {
                Ok(NumericValue::Integer(a / b))
            }
            (NumericValue::Real(a), NumericValue::Real(b)) => {
                let result = (a / b).trunc();
                Ok(NumericValue::Real(result))
            }
            _ => {
                let (promoted_dividend, promoted_divisor) =
                    super::tower::promote_types(dividend, divisor);
                Self::quotient(&promoted_dividend, &promoted_divisor)
            }
        }
    }

    /// Implements R7RS `gcd` procedure
    ///
    /// (gcd n1 ...) -> integer
    /// Returns the greatest common divisor of the arguments
    pub fn gcd(args: &[NumericValue]) -> Result<NumericValue> {
        if args.is_empty() {
            return Ok(NumericValue::Integer(0));
        }

        let mut result = args[0].clone();
        for arg in &args[1..] {
            result = Self::gcd_two(&result, arg)?;
        }
        Ok(result)
    }

    /// Helper function to compute GCD of two numbers
    fn gcd_two(a: &NumericValue, b: &NumericValue) -> Result<NumericValue> {
        // Convert to integers for GCD computation
        let a_int = Self::to_integer_for_gcd(a)?;
        let b_int = Self::to_integer_for_gcd(b)?;

        match (&a_int, &b_int) {
            (NumericValue::Integer(a), NumericValue::Integer(b)) => {
                Ok(NumericValue::Integer(Self::euclidean_gcd(a.abs(), b.abs())))
            }
            (NumericValue::BigInteger(a), NumericValue::BigInteger(b)) => {
                // Use BigInt GCD implementation
                Ok(NumericValue::BigInteger(a.gcd(b)))
            }
            _ => {
                let (promoted_a, promoted_b) = super::tower::promote_types(&a_int, &b_int);
                Self::gcd_two(&promoted_a, &promoted_b)
            }
        }
    }

    /// Euclidean GCD algorithm for i64
    fn euclidean_gcd(mut a: i64, mut b: i64) -> i64 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }

    /// Implements R7RS `lcm` procedure
    ///
    /// (lcm n1 ...) -> integer
    /// Returns the least common multiple of the arguments
    pub fn lcm(args: &[NumericValue]) -> Result<NumericValue> {
        if args.is_empty() {
            return Ok(NumericValue::Integer(1));
        }

        let mut result = args[0].clone();
        for arg in &args[1..] {
            result = Self::lcm_two(&result, arg)?;
        }
        Ok(result)
    }

    /// Helper function to compute LCM of two numbers
    fn lcm_two(a: &NumericValue, b: &NumericValue) -> Result<NumericValue> {
        let gcd_val = Self::gcd_two(a, b)?;
        let a_div_gcd = super::tower::divide(a, &gcd_val)
            .map_err(|e| Box::new(Error::runtime_error(e, None)))?;
        let result = super::tower::multiply(&a_div_gcd, b);

        // Simplify rational with denominator 1 back to integer (R7RS requirement)
        let simplified_result = Self::simplify_rational_to_integer(result);
        Ok(simplified_result)
    }

    /// Convert number to integer for GCD/LCM (R7RS requirement)
    fn to_integer_for_gcd(value: &NumericValue) -> Result<NumericValue> {
        match value {
            NumericValue::Integer(_) | NumericValue::BigInteger(_) => Ok(value.clone()),
            NumericValue::Real(r) if r.fract() == 0.0 && r.is_finite() => {
                Ok(NumericValue::Integer(*r as i64))
            }
            NumericValue::Rational(rat) if rat.denominator == 1 => {
                Ok(NumericValue::Integer(rat.numerator))
            }
            _ => Err(Box::new(Error::runtime_error(
                "GCD/LCM requires integer arguments".to_string(),
                None,
            ))),
        }
    }

    /// Simplify rational with denominator 1 to integer (R7RS requirement)
    fn simplify_rational_to_integer(value: NumericValue) -> NumericValue {
        match value {
            NumericValue::Rational(rat) if rat.denominator == 1 => {
                NumericValue::Integer(rat.numerator)
            }
            _ => value,
        }
    }

    /// Implements R7RS `numerator` procedure
    ///
    /// (numerator q) -> integer
    /// Returns the numerator of the rational number q
    pub fn numerator(value: &NumericValue) -> Result<NumericValue> {
        match value {
            NumericValue::Integer(n) => Ok(NumericValue::Integer(*n)),
            NumericValue::Rational(r) => Ok(NumericValue::Integer(r.numerator)),
            NumericValue::Real(r) => {
                // Convert to rational first
                if let Some(rational) = Self::float_to_rational(*r) {
                    Ok(NumericValue::Integer(rational.numerator))
                } else {
                    Err(Box::new(Error::runtime_error(
                        "Cannot determine numerator of inexact number".to_string(),
                        None,
                    )))
                }
            }
            _ => Err(Box::new(Error::runtime_error(
                "numerator requires a real number".to_string(),
                None,
            ))),
        }
    }

    /// Implements R7RS `denominator` procedure
    ///
    /// (denominator q) -> integer
    /// Returns the denominator of the rational number q
    pub fn denominator(value: &NumericValue) -> Result<NumericValue> {
        match value {
            NumericValue::Integer(_) => Ok(NumericValue::Integer(1)),
            NumericValue::Rational(r) => Ok(NumericValue::Integer(r.denominator)),
            NumericValue::Real(r) => {
                if let Some(rational) = Self::float_to_rational(*r) {
                    Ok(NumericValue::Integer(rational.denominator))
                } else {
                    Err(Box::new(Error::runtime_error(
                        "Cannot determine denominator of inexact number".to_string(),
                        None,
                    )))
                }
            }
            _ => Err(Box::new(Error::runtime_error(
                "denominator requires a real number".to_string(),
                None,
            ))),
        }
    }

    /// Implements R7RS `rationalize` procedure
    ///
    /// (rationalize x e) -> rational
    /// Returns the simplest rational number that differs from x by no more than e
    pub fn rationalize(x: &NumericValue, tolerance: &NumericValue) -> Result<NumericValue> {
        let x_real = x.to_f64().ok_or_else(|| {
            Box::new(Error::runtime_error(
                "rationalize requires real numbers".to_string(),
                None,
            ))
        })?;

        let tol_real = tolerance.to_f64().ok_or_else(|| {
            Box::new(Error::runtime_error(
                "rationalize requires real tolerance".to_string(),
                None,
            ))
        })?;

        if tol_real < 0.0 {
            return Err(Box::new(Error::runtime_error(
                "rationalize tolerance must be non-negative".to_string(),
                None,
            )));
        }

        // Find the simplest rational in the interval [x-e, x+e]
        let lower = x_real - tol_real;
        let upper = x_real + tol_real;

        if let Some(rational) = Self::simplest_rational_in_interval(lower, upper) {
            Ok(NumericValue::Rational(rational))
        } else {
            // Fallback to exact conversion of x
            if let Some(exact) = Self::float_to_rational(x_real) {
                Ok(NumericValue::Rational(exact))
            } else {
                Ok(NumericValue::Real(x_real))
            }
        }
    }

    /// R7RS complex number accessors
    /// (real-part z) -> real
    pub fn real_part(z: &NumericValue) -> NumericValue {
        match z {
            NumericValue::Complex(c) => NumericValue::Real(c.real),
            _ => z.clone(), // Real numbers are their own real part
        }
    }

    /// (imag-part z) -> real  
    pub fn imag_part(z: &NumericValue) -> NumericValue {
        match z {
            NumericValue::Complex(c) => NumericValue::Real(c.imaginary),
            _ => NumericValue::Integer(0), // Real numbers have zero imaginary part
        }
    }

    /// (magnitude z) -> real
    pub fn magnitude(z: &NumericValue) -> NumericValue {
        match z {
            NumericValue::Complex(c) => {
                let mag = (c.real * c.real + c.imaginary * c.imaginary).sqrt();
                NumericValue::Real(mag)
            }
            NumericValue::Integer(n) => NumericValue::Integer(n.abs()),
            NumericValue::Real(r) => NumericValue::Real(r.abs()),
            NumericValue::Rational(r) => {
                if r.is_negative() {
                    NumericValue::Rational(-*r)
                } else {
                    NumericValue::Rational(*r)
                }
            }
            _ => z.clone(),
        }
    }

    /// (angle z) -> real
    pub fn angle(z: &NumericValue) -> NumericValue {
        match z {
            NumericValue::Complex(c) => NumericValue::Real(c.imaginary.atan2(c.real)),
            NumericValue::Real(r) if *r < 0.0 => NumericValue::Real(std::f64::consts::PI),
            NumericValue::Integer(n) if *n < 0 => NumericValue::Real(std::f64::consts::PI),
            _ => NumericValue::Integer(0), // Positive reals have angle 0
        }
    }

    /// R7RS exactness procedures with proper contagion rules
    /// (exact->inexact z) -> number
    pub fn exact_to_inexact(z: &NumericValue) -> NumericValue {
        match z {
            NumericValue::Integer(n) => NumericValue::Real(*n as f64),
            NumericValue::Rational(r) => NumericValue::Real(r.to_f64()),
            NumericValue::Complex(c) => NumericValue::Complex(*c), // Already inexact
            NumericValue::Real(_) => z.clone(),                    // Already inexact
            NumericValue::BigInteger(n) => NumericValue::Real(n.to_f64().unwrap_or(f64::INFINITY)),
            _ => z.clone(),
        }
    }

    /// (inexact->exact z) -> number  
    pub fn inexact_to_exact(z: &NumericValue) -> Result<NumericValue> {
        match z {
            NumericValue::Real(r) => {
                if let Some(rational) = Self::float_to_rational(*r) {
                    Ok(NumericValue::Rational(rational))
                } else {
                    Err(Box::new(Error::runtime_error(
                        "Cannot convert inexact number to exact".to_string(),
                        None,
                    )))
                }
            }
            NumericValue::Integer(_) | NumericValue::Rational(_) => {
                Ok(z.clone()) // Already exact
            }
            _ => Err(Box::new(Error::runtime_error(
                "Cannot convert complex number to exact".to_string(),
                None,
            ))),
        }
    }

    /// Helper functions
    /// Convert float to rational using continued fractions
    fn float_to_rational(f: f64) -> Option<Rational> {
        if !f.is_finite() {
            return None;
        }

        // Use continued fraction expansion for high precision conversion
        const MAX_ITERATIONS: usize = 50;
        const MAX_DENOMINATOR: i64 = 1_000_000;

        let mut x = f;
        let mut a = x.floor() as i64;
        let mut h = [a, 1];
        let mut k = [1, 0];

        if (f - a as f64).abs() < f64::EPSILON {
            return Some(Rational::new(a, 1));
        }

        for _ in 1..MAX_ITERATIONS {
            x = 1.0 / (x - a as f64);
            a = x.floor() as i64;

            let h_new = a * h[0] + h[1];
            let k_new = a * k[0] + k[1];

            if k_new > MAX_DENOMINATOR {
                break;
            }

            h[1] = h[0];
            h[0] = h_new;
            k[1] = k[0];
            k[0] = k_new;

            let rational = Rational::new(h[0], k[0]);
            if (rational.to_f64() - f).abs() < 1e-15 {
                return Some(rational);
            }
        }

        Some(Rational::new(h[0], k[0]))
    }

    /// Find the simplest rational in an interval
    fn simplest_rational_in_interval(lower: f64, upper: f64) -> Option<Rational> {
        if lower > upper {
            return None;
        }

        // Check for integers first (simplest rationals)
        let lower_ceil = lower.ceil() as i64;
        let upper_floor = upper.floor() as i64;

        if lower_ceil <= upper_floor {
            return Some(Rational::new(lower_ceil, 1));
        }

        // Use Stern-Brocot tree to find simplest fraction
        Self::stern_brocot_search(lower, upper, 0, 1, 1, 1)
    }

    /// Stern-Brocot tree search for simplest rational
    fn stern_brocot_search(
        lower: f64,
        upper: f64,
        mut a: i64,
        mut b: i64,
        mut c: i64,
        mut d: i64,
    ) -> Option<Rational> {
        const MAX_DEPTH: usize = 50;
        let mut depth = 0;

        while depth < MAX_DEPTH {
            let mediant_num = a + c;
            let mediant_den = b + d;
            let mediant_val = mediant_num as f64 / mediant_den as f64;

            if mediant_val >= lower && mediant_val <= upper {
                return Some(Rational::new(mediant_num, mediant_den));
            } else if mediant_val < lower {
                a = mediant_num;
                b = mediant_den;
            } else {
                c = mediant_num;
                d = mediant_den;
            }

            depth += 1;
        }

        None
    }
}

/// R7RS numeric predicates with exact specifications
pub fn is_number(value: &NumericValue) -> bool {
    matches!(
        value,
        NumericValue::Integer(_)
            | NumericValue::BigInteger(_)
            | NumericValue::Rational(_)
            | NumericValue::Real(_)
            | NumericValue::Complex(_)
    )
}

pub fn is_complex(value: &NumericValue) -> bool {
    is_number(value) // All numbers are complex in R7RS
}

pub fn is_real(value: &NumericValue) -> bool {
    match value {
        NumericValue::Complex(c) => c.imaginary == 0.0,
        _ => is_number(value) && !matches!(value, NumericValue::Complex(_)),
    }
}

pub fn is_rational(value: &NumericValue) -> bool {
    match value {
        NumericValue::Integer(_) | NumericValue::BigInteger(_) | NumericValue::Rational(_) => true,
        NumericValue::Real(r) => r.is_finite(),
        NumericValue::Complex(c) => c.imaginary == 0.0 && c.real.is_finite(),
        _ => false,
    }
}

pub fn is_integer(value: &NumericValue) -> bool {
    match value {
        NumericValue::Integer(_) | NumericValue::BigInteger(_) => true,
        NumericValue::Rational(r) => r.denominator == 1,
        NumericValue::Real(r) => r.fract() == 0.0 && r.is_finite(),
        NumericValue::Complex(c) => {
            c.imaginary == 0.0 && c.real.fract() == 0.0 && c.real.is_finite()
        }
        _ => false,
    }
}

pub fn is_exact(value: &NumericValue) -> bool {
    match value {
        NumericValue::Integer(_) | NumericValue::BigInteger(_) | NumericValue::Rational(_) => true,
        NumericValue::Complex(_) => {
            // Complex numbers represented with floating point components are inexact in R7RS
            // (Only theoretical exact complex numbers with rational components would be exact,
            // but our Complex type uses f64 components)
            false
        }
        _ => false,
    }
}

pub fn is_inexact(value: &NumericValue) -> bool {
    !is_exact(value) && is_number(value)
}

pub fn is_finite(value: &NumericValue) -> bool {
    match value {
        NumericValue::Real(r) => r.is_finite(),
        NumericValue::Complex(c) => c.real.is_finite() && c.imaginary.is_finite(),
        _ => true, // Integers and rationals are always finite
    }
}

pub fn is_infinite(value: &NumericValue) -> bool {
    match value {
        NumericValue::Real(r) => r.is_infinite(),
        NumericValue::Complex(c) => c.real.is_infinite() || c.imaginary.is_infinite(),
        _ => false,
    }
}

pub fn is_nan(value: &NumericValue) -> bool {
    match value {
        NumericValue::Real(r) => r.is_nan(),
        NumericValue::Complex(c) => c.real.is_nan() || c.imaginary.is_nan(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modulo_remainder() {
        let a = NumericValue::Integer(13);
        let b = NumericValue::Integer(4);
        let neg_a = NumericValue::Integer(-13);
        let neg_b = NumericValue::Integer(-4);

        // R7RS modulo: result has sign of divisor
        assert_eq!(
            R7RSNumericProcedures::modulo(&a, &b).unwrap(),
            NumericValue::Integer(1)
        );
        assert_eq!(
            R7RSNumericProcedures::modulo(&neg_a, &b).unwrap(),
            NumericValue::Integer(3)
        );
        assert_eq!(
            R7RSNumericProcedures::modulo(&a, &neg_b).unwrap(),
            NumericValue::Integer(-3)
        );

        // R7RS remainder: result has sign of dividend
        assert_eq!(
            R7RSNumericProcedures::remainder(&a, &b).unwrap(),
            NumericValue::Integer(1)
        );
        assert_eq!(
            R7RSNumericProcedures::remainder(&neg_a, &b).unwrap(),
            NumericValue::Integer(-1)
        );
    }

    #[test]
    fn test_gcd_lcm() {
        let a = NumericValue::Integer(48);
        let b = NumericValue::Integer(18);

        let gcd_result = R7RSNumericProcedures::gcd(&[a.clone(), b.clone()]).unwrap();
        assert_eq!(gcd_result, NumericValue::Integer(6));

        let lcm_result = R7RSNumericProcedures::lcm(&[a, b]).unwrap();
        assert_eq!(lcm_result, NumericValue::Integer(144));
    }

    #[test]
    fn test_complex_accessors() {
        let c = NumericValue::Complex(Complex::new(3.0, 4.0));

        assert_eq!(
            R7RSNumericProcedures::real_part(&c),
            NumericValue::Real(3.0)
        );
        assert_eq!(
            R7RSNumericProcedures::imag_part(&c),
            NumericValue::Real(4.0)
        );
        assert_eq!(
            R7RSNumericProcedures::magnitude(&c),
            NumericValue::Real(5.0)
        );

        // Real number should return itself for real-part, 0 for imag-part
        let r = NumericValue::Integer(42);
        assert_eq!(
            R7RSNumericProcedures::real_part(&r),
            NumericValue::Integer(42)
        );
        assert_eq!(
            R7RSNumericProcedures::imag_part(&r),
            NumericValue::Integer(0)
        );
    }

    #[test]
    fn test_numeric_predicates() {
        let int_val = NumericValue::Integer(42);
        let real_val = NumericValue::Real(3.14);
        let complex_val = NumericValue::Complex(Complex::new(1.0, 2.0));
        let rational_val = NumericValue::Rational(Rational::new(3, 4));

        assert!(is_number(&int_val));
        assert!(is_complex(&int_val)); // All numbers are complex
        assert!(is_real(&int_val));
        assert!(is_rational(&int_val));
        assert!(is_integer(&int_val));
        assert!(is_exact(&int_val));
        assert!(!is_inexact(&int_val));

        assert!(is_number(&real_val));
        assert!(is_complex(&real_val));
        assert!(is_real(&real_val));
        assert!(is_rational(&real_val)); // Finite reals are rational
        assert!(!is_integer(&real_val));
        assert!(!is_exact(&real_val));
        assert!(is_inexact(&real_val));

        assert!(is_number(&complex_val));
        assert!(is_complex(&complex_val));
        assert!(!is_real(&complex_val));
        assert!(!is_rational(&complex_val));
        assert!(!is_integer(&complex_val));
        assert!(!is_exact(&complex_val));
        assert!(is_inexact(&complex_val));
    }

    #[test]
    fn test_rationalize() {
        let x = NumericValue::Real(0.3333);
        let tolerance = NumericValue::Real(0.01);

        let result = R7RSNumericProcedures::rationalize(&x, &tolerance).unwrap();
        if let NumericValue::Rational(r) = result {
            assert_eq!(r.numerator, 1);
            assert_eq!(r.denominator, 3);
        } else {
            panic!("Expected rational result");
        }
    }
}

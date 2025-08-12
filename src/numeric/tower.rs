#![allow(unused_variables)]
//! Numeric tower implementation with automatic type promotion and coercion
//!
//! Implements the Scheme numeric tower: Integer → Rational → Real → Complex
//! with automatic type promotion, precision preservation, and optimized operations.

use super::{NumericValue, NumericType, Complex, Rational, BigInt};
use std::cmp::Ordering;

/// Automatic type promotion following the numeric tower
pub fn promote_types(left: &NumericValue, right: &NumericValue) -> (NumericValue, NumericValue) {
    use NumericType::*;
    
    let left_type = left.numeric_type();
    let right_type = right.numeric_type();
    
    match left_type.max(right_type) {
        Integer => (left.clone(), right.clone()),
        BigInteger => (promote_to_bigint(left), promote_to_bigint(right)),
        Rational => (promote_to_rational(left), promote_to_rational(right)),
        Real => (promote_to_real(left), promote_to_real(right)),
        Complex => (promote_to_complex(left), promote_to_complex(right)),
        NumericType::Vector => (left.clone(), right.clone()), // Vectors are handled specially
    }
}

/// Promotes a numeric value to big integer
pub fn promote_to_bigint(value: &NumericValue) -> NumericValue {
    match value {
        NumericValue::Integer(n) => NumericValue::BigInteger(BigInt::from_i64(*n)),
        NumericValue::BigInteger(_) => value.clone(),
        NumericValue::Rational(r) if r.denominator == 1 => {
            NumericValue::BigInteger(BigInt::from_i64(r.numerator))
        }
        NumericValue::Real(r) if r.fract() == 0.0 && r.is_finite() => {
            NumericValue::BigInteger(BigInt::from_i64(*r as i64))
        }
        NumericValue::Complex(c) if c.imaginary == 0.0 && c.real.fract() == 0.0 && c.real.is_finite() => {
            NumericValue::BigInteger(BigInt::from_i64(c.real as i64))
        }
        NumericValue::Vector(v) => {
            NumericValue::Vector(v.iter().map(promote_to_bigint).collect())
        }
        _ => value.clone(), // Cannot promote non-integer values
    }
}

/// Promotes a numeric value to rational
pub fn promote_to_rational(value: &NumericValue) -> NumericValue {
    match value {
        NumericValue::Integer(n) => NumericValue::Rational(Rational::from_integer(*n)),
        NumericValue::BigInteger(n) => {
            if let Some(i) = n.to_i64() {
                NumericValue::Rational(Rational::from_integer(i))
            } else {
                // For very large integers, we might need to extend Rational to support BigInt
                // For now, convert to real
                NumericValue::Real(n.to_f64().unwrap_or(f64::INFINITY))
            }
        }
        NumericValue::Rational(_) => value.clone(),
        NumericValue::Real(r) => {
            // Try to convert to exact rational if it's a simple fraction
            if let Some(rational) = float_to_rational(*r) {
                NumericValue::Rational(rational)
            } else {
                value.clone()
            }
        }
        NumericValue::Complex(c) if c.imaginary == 0.0 => {
            promote_to_rational(&NumericValue::Real(c.real))
        }
        NumericValue::Vector(v) => {
            NumericValue::Vector(v.iter().map(promote_to_rational).collect())
        }
        _ => value.clone(),
    }
}

/// Promotes a numeric value to real
pub fn promote_to_real(value: &NumericValue) -> NumericValue {
    match value {
        NumericValue::Integer(n) => NumericValue::Real(*n as f64),
        NumericValue::BigInteger(n) => NumericValue::Real(n.to_f64().unwrap_or(f64::INFINITY)),
        NumericValue::Rational(r) => NumericValue::Real(r.to_f64()),
        NumericValue::Real(_) => value.clone(),
        NumericValue::Complex(c) if c.imaginary == 0.0 => NumericValue::Real(c.real),
        NumericValue::Vector(v) => {
            NumericValue::Vector(v.iter().map(promote_to_real).collect())
        }
        _ => value.clone(),
    }
}

/// Promotes a numeric value to complex
pub fn promote_to_complex(value: &NumericValue) -> NumericValue {
    match value {
        NumericValue::Integer(n) => NumericValue::Complex(Complex::from_real(*n as f64)),
        NumericValue::BigInteger(n) => {
            NumericValue::Complex(Complex::from_real(n.to_f64().unwrap_or(f64::INFINITY)))
        }
        NumericValue::Rational(r) => NumericValue::Complex(Complex::from_real(r.to_f64())),
        NumericValue::Real(r) => NumericValue::Complex(Complex::from_real(*r)),
        NumericValue::Complex(_) => value.clone(),
        NumericValue::Vector(v) => {
            NumericValue::Vector(v.iter().map(promote_to_complex).collect())
        }
    }
}

/// Adds two numeric values using type promotion and overflow handling.
/// 
/// Automatically promotes types to ensure compatibility and prevents overflow
/// by upgrading to BigInteger when needed.
pub fn add(left: &NumericValue, right: &NumericValue) -> NumericValue {
    let (left_promoted, right_promoted) = promote_types(left, right);
    
    match (&left_promoted, &right_promoted) {
        (NumericValue::Integer(a), NumericValue::Integer(b)) => {
            // Check for overflow
            if let Some(result) = a.checked_add(*b) {
                NumericValue::Integer(result)
            } else {
                // Promote to BigInt on overflow
                let a_big = BigInt::from_i64(*a);
                let b_big = BigInt::from_i64(*b);
                NumericValue::BigInteger(&a_big + &b_big)
            }
        }
        (NumericValue::BigInteger(a), NumericValue::BigInteger(b)) => {
            NumericValue::BigInteger(a + b)
        }
        (NumericValue::Rational(a), NumericValue::Rational(b)) => {
            NumericValue::Rational(*a + *b)
        }
        (NumericValue::Real(a), NumericValue::Real(b)) => {
            NumericValue::Real(a + b)
        }
        (NumericValue::Complex(a), NumericValue::Complex(b)) => {
            NumericValue::Complex(*a + *b)
        }
        _ => unreachable!("Type promotion should ensure matching types"),
    }
}

/// Subtracts the right numeric value from the left using type promotion.
/// 
/// Handles overflow by automatically promoting to BigInteger when necessary.
pub fn subtract(left: &NumericValue, right: &NumericValue) -> NumericValue {
    let (left_promoted, right_promoted) = promote_types(left, right);
    
    match (&left_promoted, &right_promoted) {
        (NumericValue::Integer(a), NumericValue::Integer(b)) => {
            if let Some(result) = a.checked_sub(*b) {
                NumericValue::Integer(result)
            } else {
                let a_big = BigInt::from_i64(*a);
                let b_big = BigInt::from_i64(*b);
                NumericValue::BigInteger(&a_big - &b_big)
            }
        }
        (NumericValue::BigInteger(a), NumericValue::BigInteger(b)) => {
            NumericValue::BigInteger(a - b)
        }
        (NumericValue::Rational(a), NumericValue::Rational(b)) => {
            NumericValue::Rational(*a - *b)
        }
        (NumericValue::Real(a), NumericValue::Real(b)) => {
            NumericValue::Real(a - b)
        }
        (NumericValue::Complex(a), NumericValue::Complex(b)) => {
            NumericValue::Complex(*a - *b)
        }
        _ => unreachable!("Type promotion should ensure matching types"),
    }
}

/// Multiplies two numeric values with type promotion and overflow handling.
/// 
/// Promotes types as needed and upgrades to BigInteger on overflow.
pub fn multiply(left: &NumericValue, right: &NumericValue) -> NumericValue {
    let (left_promoted, right_promoted) = promote_types(left, right);
    
    match (&left_promoted, &right_promoted) {
        (NumericValue::Integer(a), NumericValue::Integer(b)) => {
            if let Some(result) = a.checked_mul(*b) {
                NumericValue::Integer(result)
            } else {
                let a_big = BigInt::from_i64(*a);
                let b_big = BigInt::from_i64(*b);
                NumericValue::BigInteger(&a_big * &b_big)
            }
        }
        (NumericValue::BigInteger(a), NumericValue::BigInteger(b)) => {
            NumericValue::BigInteger(a * b)
        }
        (NumericValue::Rational(a), NumericValue::Rational(b)) => {
            NumericValue::Rational(*a * *b)
        }
        (NumericValue::Real(a), NumericValue::Real(b)) => {
            NumericValue::Real(a * b)
        }
        (NumericValue::Complex(a), NumericValue::Complex(b)) => {
            NumericValue::Complex(*a * *b)
        }
        _ => unreachable!("Type promotion should ensure matching types"),
    }
}

/// Divides the left numeric value by the right with type promotion.
/// 
/// Returns an error if division by zero is attempted.
pub fn divide(left: &NumericValue, right: &NumericValue) -> Result<NumericValue, String> {
    if right.is_zero() {
        return Err("Division by zero".to_string());
    }
    
    let (left_promoted, right_promoted) = promote_types(left, right);
    
    match (&left_promoted, &right_promoted) {
        (NumericValue::Integer(a), NumericValue::Integer(b)) => {
            // Integer division promotes to rational to preserve exactness
            let rational_result = Rational::new(*a, *b);
            Ok(NumericValue::Rational(rational_result))
        }
        (NumericValue::BigInteger(a), NumericValue::BigInteger(b)) => {
            // For BigInt, we'd need to extend Rational to support BigInt numerators/denominators
            // For now, convert to real division
            let a_f = a.to_f64().unwrap_or(f64::INFINITY);
            let b_f = b.to_f64().unwrap_or(f64::INFINITY);
            Ok(NumericValue::Real(a_f / b_f))
        }
        (NumericValue::Rational(a), NumericValue::Rational(b)) => {
            Ok(NumericValue::Rational(*a / *b))
        }
        (NumericValue::Real(a), NumericValue::Real(b)) => {
            Ok(NumericValue::Real(a / b))
        }
        (NumericValue::Complex(a), NumericValue::Complex(b)) => {
            Ok(NumericValue::Complex(*a / *b))
        }
        _ => unreachable!("Type promotion should ensure matching types"),
    }
}

/// Negates a numeric value with overflow handling.
/// 
/// Promotes to BigInteger if negation would cause overflow.
pub fn negate(value: &NumericValue) -> NumericValue {
    match value {
        NumericValue::Integer(n) => {
            if let Some(result) = n.checked_neg() {
                NumericValue::Integer(result)
            } else {
                NumericValue::BigInteger(-BigInt::from_i64(*n))
            }
        }
        NumericValue::BigInteger(n) => NumericValue::BigInteger(-n.clone()),
        NumericValue::Rational(r) => NumericValue::Rational(-*r),
        NumericValue::Real(r) => NumericValue::Real(-r),
        NumericValue::Complex(c) => NumericValue::Complex(-*c),
        NumericValue::Vector(v) => {
            NumericValue::Vector(v.iter().map(negate).collect())
        }
    }
}

/// Compares two numeric values for ordering.
/// 
/// Returns None if either value is complex (non-real), as complex numbers cannot be ordered.
pub fn compare(left: &NumericValue, right: &NumericValue) -> Option<Ordering> {
    // Complex numbers cannot be ordered
    if !left.is_real() || !right.is_real() {
        return None;
    }
    
    let (left_promoted, right_promoted) = promote_types(left, right);
    
    match (&left_promoted, &right_promoted) {
        (NumericValue::Integer(a), NumericValue::Integer(b)) => Some(a.cmp(b)),
        (NumericValue::BigInteger(a), NumericValue::BigInteger(b)) => Some(a.cmp(b)),
        (NumericValue::Rational(a), NumericValue::Rational(b)) => Some(a.cmp(b)),
        (NumericValue::Real(a), NumericValue::Real(b)) => a.partial_cmp(b),
        (NumericValue::Complex(a), NumericValue::Complex(b)) => {
            // Only compare if both are real
            if a.is_real() && b.is_real() {
                a.real.partial_cmp(&b.real)
            } else {
                None
            }
        }
        _ => unreachable!("Type promotion should ensure matching types"),
    }
}

/// Raises a numeric value to the power of another numeric value.
/// 
/// Handles integer powers efficiently and promotes to complex for general cases.
pub fn power(base: &NumericValue, exponent: &NumericValue) -> NumericValue {
    match (base, exponent) {
        // Integer^Integer with small exponents
        (NumericValue::Integer(b), NumericValue::Integer(e)) if *e >= 0 && *e <= 100 => {
            if let Some(result) = b.checked_pow(*e as u32) {
                NumericValue::Integer(result)
            } else {
                let base_big = BigInt::from_i64(*b);
                let exp_big = BigInt::from_i64(*e);
                // For now, convert to real; in full implementation, use BigInt power
                NumericValue::Real((*b as f64).powf(*e as f64))
            }
        }
        // Rational^Integer
        (NumericValue::Rational(r), NumericValue::Integer(e)) if *e >= -100 && *e <= 100 => {
            NumericValue::Rational(r.powi(*e as i32))
        }
        // General case: promote to complex and use complex power
        _ => {
            let base_complex = promote_to_complex(base);
            let exp_complex = promote_to_complex(exponent);
            
            if let (NumericValue::Complex(b), NumericValue::Complex(e)) = (&base_complex, &exp_complex) {
                NumericValue::Complex(b.pow(*e))
            } else {
                unreachable!("Promotion to complex should always succeed")
            }
        }
    }
}

/// Computes the square root of a numeric value.
/// 
/// Returns integer result when possible, otherwise promotes to real or complex.
pub fn sqrt(value: &NumericValue) -> NumericValue {
    match value {
        NumericValue::Integer(n) if *n >= 0 => {
            let sqrt_f = (*n as f64).sqrt();
            if sqrt_f.fract() == 0.0 {
                NumericValue::Integer(sqrt_f as i64)
            } else {
                NumericValue::Real(sqrt_f)
            }
        }
        NumericValue::Real(r) if *r >= 0.0 => NumericValue::Real(r.sqrt()),
        _ => {
            // For negative reals or complex numbers
            let complex_val = promote_to_complex(value);
            if let NumericValue::Complex(c) = complex_val {
                NumericValue::Complex(c.sqrt())
            } else {
                unreachable!("Promotion to complex should always succeed")
            }
        }
    }
}

/// Attempts to convert a float to a simple rational
fn float_to_rational(f: f64) -> Option<Rational> {
    if !f.is_finite() {
        return None;
    }
    
    // Simple algorithm for common fractions
    const MAX_DENOMINATOR: i64 = 10000;
    
    for denominator in 1..=MAX_DENOMINATOR {
        let numerator = (f * denominator as f64).round() as i64;
        let rational = Rational::new(numerator, denominator);
        
        if (rational.to_f64() - f).abs() < 1e-10 {
            return Some(rational);
        }
    }
    
    None
}

/// Converts a numeric value to its exact representation.
/// 
/// Attempts to convert real numbers to rational representation when possible.
pub fn make_exact(value: &NumericValue) -> NumericValue {
    match value {
        NumericValue::Integer(_) | NumericValue::BigInteger(_) | NumericValue::Rational(_) => {
            value.clone() // Already exact
        }
        NumericValue::Real(r) => {
            if let Some(rational) = float_to_rational(*r) {
                NumericValue::Rational(rational)
            } else {
                value.clone() // Cannot make exact
            }
        }
        NumericValue::Complex(c) if c.is_real() => {
            make_exact(&NumericValue::Real(c.real))
        }
        NumericValue::Complex(_) => value.clone(), // Cannot make complex exact in general
        NumericValue::Vector(v) => {
            NumericValue::Vector(v.iter().map(make_exact).collect())
        }
    }
}

/// Converts a numeric value to its inexact (floating-point) representation.
/// 
/// Promotes exact values to their real number equivalents.
pub fn make_inexact(value: &NumericValue) -> NumericValue {
    match value {
        NumericValue::Real(_) | NumericValue::Complex(_) => {
            value.clone() // Already inexact
        }
        _ => promote_to_real(value), // Convert to real (inexact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_promotion() {
        let int_val = NumericValue::integer(42);
        let rat_val = NumericValue::rational(3, 4);
        
        let (promoted_int, promoted_rat) = promote_types(&int_val, &rat_val);
        
        assert!(matches!(promoted_int, NumericValue::Rational(_)));
        assert!(matches!(promoted_rat, NumericValue::Rational(_)));
    }

    #[test]
    fn test_arithmetic_operations() {
        let a = NumericValue::integer(10);
        let b = NumericValue::integer(3);
        
        let sum = add(&a, &b);
        assert_eq!(sum.to_i64(), Some(13));
        
        let diff = subtract(&a, &b);
        assert_eq!(diff.to_i64(), Some(7));
        
        let prod = multiply(&a, &b);
        assert_eq!(prod.to_i64(), Some(30));
        
        let quot = divide(&a, &b).unwrap();
        if let NumericValue::Rational(r) = quot {
            assert_eq!(r.numerator, 10);
            assert_eq!(r.denominator, 3);
        } else {
            panic!("Expected rational result");
        }
    }

    #[test]
    fn test_overflow_handling() {
        let large_int = NumericValue::integer(i64::MAX);
        let one = NumericValue::integer(1);
        
        let result = add(&large_int, &one);
        assert!(matches!(result, NumericValue::BigInteger(_)));
    }

    #[test]
    fn test_exactness_operations() {
        let inexact = NumericValue::real(0.5);
        let exact = make_exact(&inexact);
        
        if let NumericValue::Rational(r) = exact {
            assert_eq!(r.numerator, 1);
            assert_eq!(r.denominator, 2);
        } else {
            panic!("Expected rational result");
        }
    }

    #[test]
    fn test_complex_arithmetic() {
        let a = NumericValue::complex(3.0, 4.0);
        let b = NumericValue::complex(1.0, 2.0);
        
        let sum = add(&a, &b);
        if let NumericValue::Complex(c) = sum {
            assert_eq!(c.real, 4.0);
            assert_eq!(c.imaginary, 6.0);
        } else {
            panic!("Expected complex result");
        }
    }

    #[test]
    fn test_comparison() {
        let a = NumericValue::integer(5);
        let b = NumericValue::rational(10, 2);
        
        assert_eq!(compare(&a, &b), Some(Ordering::Equal));
        
        let c = NumericValue::complex(1.0, 1.0);
        assert_eq!(compare(&a, &c), None); // Complex numbers can't be compared
    }
}
//! R7RS-large compatible high-performance numeric library
//!
//! This module provides a comprehensive implementation of numeric types and operations
//! following the R7RS-large specification with performance optimizations using Rust's
//! zero-cost abstractions and type system.
//!
//! ## Numeric Tower
//! - Integer → Rational → Real → Complex
//! - Automatic type promotion and coercion
//! - Precision preservation where possible
//!
//! ## Key Features
//! - Complex number arithmetic with optimizations
//! - Rational number system with GCD-based reduction
//! - Big integer support for arbitrary precision
//! - Advanced mathematical functions
//! - SIMD optimizations where applicable

/// Complex number implementation with arithmetic operations.
pub mod complex;
/// Rational number system with GCD-based reduction.
pub mod rational;
/// Arbitrary precision big integer implementation.
pub mod bigint;
/// Numeric tower with automatic type promotion and coercion.
pub mod tower;
/// Advanced mathematical functions and operations.
pub mod functions;
/// Mathematical constants and predefined values.
pub mod constants;
/// Primitive numeric operations and conversions.
pub mod primitives;
/// Integration with the language's evaluation system.
pub mod integration;
/// Performance optimizations and specialized algorithms.
pub mod optimization;
/// Demonstration and example code for numeric operations.
pub mod demo;
/// SIMD-optimized numeric operations for performance.
pub mod simd_optimization;

pub use complex::*;
pub use rational::*;
pub use bigint::*;
pub use tower::*;
pub use functions::*;
pub use constants::*;
pub use primitives::*;
pub use integration::*;
pub use optimization::*;
pub use demo::*;
pub use simd_optimization::{
    SimdNumericOps, SimdConfig, SimdBenchmarkResults,
    add_numeric_arrays_optimized, dot_product_optimized,
};

use crate::ast::Literal;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Unified numeric value type that encompasses all numeric types in the tower
#[derive(Debug, Clone, PartialEq)]
pub enum NumericValue {
    /// Machine integer (i64) - fastest for small integers
    Integer(i64),
    /// Arbitrary precision integer
    BigInteger(BigInt),
    /// Exact rational number
    Rational(Rational),
    /// IEEE 754 double precision floating point
    Real(f64),
    /// Complex number (real + imaginary parts)
    Complex(Complex),
}

/// Numeric type classification for the tower
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumericType {
    Integer = 0,
    BigInteger = 1,
    Rational = 2,
    Real = 3,
    Complex = 4,
}

impl NumericValue {
    /// Creates an integer value
    pub fn integer(n: i64) -> Self {
        Self::Integer(n)
    }

    /// Creates a big integer value
    pub fn big_integer(n: BigInt) -> Self {
        Self::BigInteger(n)
    }

    /// Creates a rational value
    pub fn rational(num: i64, den: i64) -> Self {
        Self::Rational(Rational::new(num, den))
    }

    /// Creates a real value
    pub fn real(r: f64) -> Self {
        Self::Real(r)
    }

    /// Creates a complex value
    pub fn complex(real: f64, imag: f64) -> Self {
        Self::Complex(Complex::new(real, imag))
    }

    /// Gets the numeric type for tower operations
    pub fn numeric_type(&self) -> NumericType {
        match self {
            Self::Integer(_) => NumericType::Integer,
            Self::BigInteger(_) => NumericType::BigInteger,
            Self::Rational(_) => NumericType::Rational,
            Self::Real(_) => NumericType::Real,
            Self::Complex(_) => NumericType::Complex,
        }
    }

    /// Checks if this number is exact (rational or integer)
    pub fn is_exact(&self) -> bool {
        matches!(self, 
            Self::Integer(_) | 
            Self::BigInteger(_) | 
            Self::Rational(_)
        )
    }

    /// Checks if this number is inexact (real or complex)
    pub fn is_inexact(&self) -> bool {
        matches!(self, Self::Real(_) | Self::Complex(_))
    }

    /// Checks if this number is real (not complex with non-zero imaginary part)
    pub fn is_real(&self) -> bool {
        match self {
            Self::Complex(c) => c.imaginary == 0.0,
            _ => true,
        }
    }

    /// Checks if this number is an integer
    pub fn is_integer(&self) -> bool {
        match self {
            Self::Integer(_) | Self::BigInteger(_) => true,
            Self::Rational(r) => r.denominator == 1,
            Self::Real(r) => r.fract() == 0.0 && r.is_finite(),
            Self::Complex(c) => c.imaginary == 0.0 && c.real.fract() == 0.0 && c.real.is_finite(),
        }
    }

    /// Checks if this number is zero
    pub fn is_zero(&self) -> bool {
        match self {
            Self::Integer(n) => *n == 0,
            Self::BigInteger(n) => n.is_zero(),
            Self::Rational(r) => r.numerator == 0,
            Self::Real(r) => *r == 0.0,
            Self::Complex(c) => c.real == 0.0 && c.imaginary == 0.0,
        }
    }

    /// Checks if this number is positive
    pub fn is_positive(&self) -> bool {
        match self {
            Self::Integer(n) => *n > 0,
            Self::BigInteger(n) => n.is_positive(),
            Self::Rational(r) => r.is_positive(),
            Self::Real(r) => *r > 0.0,
            Self::Complex(_) => false, // Complex numbers are not ordered
        }
    }

    /// Checks if this number is negative
    pub fn is_negative(&self) -> bool {
        match self {
            Self::Integer(n) => *n < 0,
            Self::BigInteger(n) => n.is_negative(),
            Self::Rational(r) => r.is_negative(),
            Self::Real(r) => *r < 0.0,
            Self::Complex(_) => false, // Complex numbers are not ordered
        }
    }

    /// Converts to f64 if possible (with potential precision loss)
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Self::Integer(n) => Some(*n as f64),
            Self::BigInteger(n) => n.to_f64(),
            Self::Rational(r) => Some(r.to_f64()),
            Self::Real(r) => Some(*r),
            Self::Complex(c) if c.imaginary == 0.0 => Some(c.real),
            Self::Complex(_) => None,
        }
    }

    /// Converts to i64 if possible (exact integers only)
    pub fn to_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(n) => Some(*n),
            Self::BigInteger(n) => n.to_i64(),
            Self::Rational(r) if r.denominator == 1 => Some(r.numerator),
            Self::Real(r) if r.fract() == 0.0 && r.is_finite() => {
                let i = *r as i64;
                if i as f64 == *r { Some(i) } else { None }
            }
            Self::Complex(c) if c.imaginary == 0.0 && c.real.fract() == 0.0 && c.real.is_finite() => {
                let i = c.real as i64;
                if i as f64 == c.real { Some(i) } else { None }
            }
            _ => None,
        }
    }

    /// Converts from a Literal
    pub fn from_literal(lit: &Literal) -> Option<Self> {
        match lit {
            Literal::Number(n) => Some(Self::Real(*n)),
            Literal::Rational { numerator, denominator } => {
                Some(Self::Rational(Rational::new(*numerator, *denominator)))
            }
            Literal::Complex { real, imaginary } => {
                Some(Self::Complex(Complex::new(*real, *imaginary)))
            }
            _ => None,
        }
    }

    /// Converts to a Literal
    pub fn to_literal(&self) -> Literal {
        match self {
            Self::Integer(n) => Literal::Number(*n as f64),
            Self::BigInteger(n) => {
                // If it fits in f64 range, use Number; otherwise use string representation
                if let Some(f) = n.to_f64() {
                    Literal::Number(f)
                } else {
                    // For now, fall back to f64 representation
                    // In a full implementation, we might want to extend Literal to support BigInt
                    Literal::Number(n.to_f64().unwrap_or(f64::INFINITY))
                }
            }
            Self::Rational(r) => Literal::Rational {
                numerator: r.numerator,
                denominator: r.denominator,
            },
            Self::Real(r) => Literal::Number(*r),
            Self::Complex(c) => Literal::Complex {
                real: c.real,
                imaginary: c.imaginary,
            },
        }
    }

    /// Adds two numeric values using the numeric tower
    pub fn add(&self, other: &Self) -> Result<Self, String> {
        Ok(crate::numeric::tower::add(self, other))
    }

    /// Multiplies two numeric values using the numeric tower  
    pub fn multiply(&self, other: &Self) -> Result<Self, String> {
        Ok(crate::numeric::tower::multiply(self, other))
    }

    /// Divides two numeric values using the numeric tower
    pub fn divide(&self, other: &Self) -> Result<Self, String> {
        crate::numeric::tower::divide(self, other)
    }

    /// Subtracts two numeric values using the numeric tower
    pub fn subtract(&self, other: &Self) -> Result<Self, String> {
        Ok(crate::numeric::tower::subtract(self, other))
    }
}

impl fmt::Display for NumericValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(n) => write!(f, "{}", n),
            Self::BigInteger(n) => write!(f, "{}", n),
            Self::Rational(r) => write!(f, "{}", r),
            Self::Real(r) => {
                if r.fract() == 0.0 && r.is_finite() {
                    write!(f, "{}.0", *r as i64)
                } else {
                    write!(f, "{}", r)
                }
            }
            Self::Complex(c) => write!(f, "{}", c),
        }
    }
}

impl Hash for NumericValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.numeric_type().hash(state);
        match self {
            Self::Integer(n) => n.hash(state),
            Self::BigInteger(n) => n.hash(state),
            Self::Rational(r) => r.hash(state),
            Self::Real(r) => r.to_bits().hash(state),
            Self::Complex(c) => c.hash(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_value_creation() {
        let int_val = NumericValue::integer(42);
        let rat_val = NumericValue::rational(3, 4);
        let real_val = NumericValue::real(3.14);
        let complex_val = NumericValue::complex(1.0, 2.0);

        assert_eq!(int_val.numeric_type(), NumericType::Integer);
        assert_eq!(rat_val.numeric_type(), NumericType::Rational);
        assert_eq!(real_val.numeric_type(), NumericType::Real);
        assert_eq!(complex_val.numeric_type(), NumericType::Complex);
    }

    #[test]
    fn test_numeric_predicates() {
        let int_val = NumericValue::integer(42);
        let rat_val = NumericValue::rational(3, 4);
        let real_val = NumericValue::real(3.14);
        let complex_val = NumericValue::complex(1.0, 2.0);

        assert!(int_val.is_exact());
        assert!(rat_val.is_exact());
        assert!(real_val.is_inexact());
        assert!(complex_val.is_inexact());

        assert!(int_val.is_real());
        assert!(rat_val.is_real());
        assert!(real_val.is_real());
        assert!(!complex_val.is_real());

        assert!(int_val.is_integer());
        assert!(!rat_val.is_integer());
        assert!(!real_val.is_integer());
        assert!(!complex_val.is_integer());
    }

    #[test]
    fn test_literal_conversion() {
        let lit = Literal::Rational { numerator: 3, denominator: 4 };
        let num_val = NumericValue::from_literal(&lit).unwrap();
        let back_lit = num_val.to_literal();

        assert_eq!(lit, back_lit);
    }
}
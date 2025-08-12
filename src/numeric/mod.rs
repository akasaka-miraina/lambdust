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
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod simd_optimization;
/// Stub SIMD implementation for non-x86 architectures.
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub mod simd_optimization_stub;
/// SIMD performance benchmarking and analysis suite.
#[cfg(feature = "simd-benchmarks")]
pub mod simd_benchmarks;

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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use simd_optimization::{
    SimdNumericOps, SimdOperationType, AlignedBuffer, CpuFeatures,
};

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub use simd_optimization_stub::{
    SimdNumericOps, SimdOperationType, AlignedBuffer, CpuFeatures,
};

// Re-export SIMD configuration and optimized functions
pub use SimdNumericOps as SimdConfig;

/// Optimized addition of numeric arrays using SIMD
pub fn add_numeric_arrays_optimized(a: &[f64], b: &[f64]) -> crate::diagnostics::Result<Vec<f64>> {
    let mut result = vec![0.0; a.len()];
    let simd_ops_guard = get_simd_ops();
    let mut simd_ops = simd_ops_guard.lock()
        .map_err(|_| crate::diagnostics::Error::runtime_error("Failed to acquire SIMD lock".to_string(), None))?;
    simd_ops.add_f64_arrays(a, b, &mut result)?;
    Ok(result)
}

/// Optimized dot product using SIMD  
pub fn dot_product_optimized(a: &[f64], b: &[f64]) -> crate::diagnostics::Result<f64> {
    let simd_ops_guard = get_simd_ops();
    let mut simd_ops = simd_ops_guard.lock()
        .map_err(|_| crate::diagnostics::Error::runtime_error("Failed to acquire SIMD lock".to_string(), None))?;
    simd_ops.dot_product_f64(a, b)
}

use crate::ast::Literal;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Global SIMD optimization engine for high-performance numeric computations
static GLOBAL_SIMD_OPS: Lazy<Arc<Mutex<SimdNumericOps>>> = Lazy::new(|| {
    Arc::new(Mutex::new(SimdNumericOps::new()))
});

/// Gets the global SIMD operations engine
pub fn get_simd_ops() -> Arc<Mutex<SimdNumericOps>> {
    GLOBAL_SIMD_OPS.clone()
}

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
    /// Vector of numeric values for SIMD optimization
    Vector(Vec<NumericValue>),
}

/// Numeric type classification for the tower
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumericType {
    /// Machine-sized integer (i64)
    Integer = 0,
    /// Arbitrary precision integer
    BigInteger = 1,
    /// Exact rational number (numerator/denominator)
    Rational = 2,
    /// IEEE 754 double precision floating point
    Real = 3,
    /// Complex number with real and imaginary parts
    Complex = 4,
    /// Vector of numeric values
    Vector = 5,
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

    /// Creates a vector value
    pub fn vector(values: Vec<NumericValue>) -> Self {
        Self::Vector(values)
    }
    
    /// Creates a vector of real values (optimized for SIMD)
    pub fn real_vector(values: Vec<f64>) -> Self {
        let num_values: Vec<NumericValue> = values.into_iter()
            .map(NumericValue::real)
            .collect();
        Self::Vector(num_values)
    }

    /// Gets the numeric type for tower operations
    pub fn numeric_type(&self) -> NumericType {
        match self {
            Self::Integer(_) => NumericType::Integer,
            Self::BigInteger(_) => NumericType::BigInteger,
            Self::Rational(_) => NumericType::Rational,
            Self::Real(_) => NumericType::Real,
            Self::Complex(_) => NumericType::Complex,
            Self::Vector(_) => NumericType::Vector,
        }
    }

    /// Checks if this number is exact (rational or integer)
    pub fn is_exact(&self) -> bool {
        match self {
            Self::Integer(_) | Self::BigInteger(_) | Self::Rational(_) => true,
            Self::Vector(v) => v.iter().all(|x| x.is_exact()),
            _ => false,
        }
    }

    /// Checks if this number is inexact (real or complex)
    pub fn is_inexact(&self) -> bool {
        match self {
            Self::Real(_) | Self::Complex(_) => true,
            Self::Vector(v) => v.iter().any(|x| x.is_inexact()),
            _ => false,
        }
    }

    /// Checks if this number is real (not complex with non-zero imaginary part)
    pub fn is_real(&self) -> bool {
        match self {
            Self::Complex(c) => c.imaginary == 0.0,
            Self::Vector(v) => v.iter().all(|x| x.is_real()),
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
            Self::Vector(v) => v.iter().all(|x| x.is_integer()),
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
            Self::Vector(v) => v.iter().all(|x| x.is_zero()),
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
            Self::Vector(v) => v.iter().all(|x| x.is_positive()),
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
            Self::Vector(v) => v.iter().all(|x| x.is_negative()),
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
            Self::Vector(_) => None, // Vectors don't convert to single f64
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
            Self::Vector(_) => None, // Vectors don't convert to single i64
            _ => None, // Other types don't convert to i64
        }
    }

    /// Converts from a Literal
    pub fn from_literal(lit: &Literal) -> Option<Self> {
        match lit {
            Literal::ExactInteger(n) => Some(Self::Integer(*n)),
            Literal::InexactReal(n) => Some(Self::Real(*n)),
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
            Self::Integer(n) => Literal::ExactInteger(*n),
            Self::BigInteger(n) => {
                // If it fits in i64 range, use ExactInteger; otherwise use InexactReal
                if let Some(i) = n.to_i64() {
                    Literal::ExactInteger(i)
                } else {
                    // For very large integers, convert to inexact representation
                    Literal::InexactReal(n.to_f64().unwrap_or(f64::INFINITY))
                }
            }
            Self::Rational(r) => Literal::Rational {
                numerator: r.numerator,
                denominator: r.denominator,
            },
            Self::Real(r) => Literal::InexactReal(*r),
            Self::Complex(c) => Literal::Complex {
                real: c.real,
                imaginary: c.imaginary,
            },
            Self::Vector(_) => {
                // Vectors are represented as strings for now
                // In the future, this could be a Vector literal type
                Literal::String(format!("{self}"))
            }
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

    /// SIMD-optimized vector addition for compatible vectors
    pub fn simd_vector_add(&self, other: &Self) -> Result<Self, String> {
        match (self, other) {
            (Self::Vector(a), Self::Vector(b)) if a.len() == b.len() => {
                // Try to extract f64 vectors for SIMD optimization
                let a_f64: Result<Vec<f64>, _> = a.iter()
                    .map(|v| v.to_f64().ok_or("Not convertible to f64"))
                    .collect();
                let b_f64: Result<Vec<f64>, _> = b.iter()
                    .map(|v| v.to_f64().ok_or("Not convertible to f64"))
                    .collect();

                match (a_f64, b_f64) {
                    (Ok(a_vals), Ok(b_vals)) => {
                        // Use SIMD optimization
                        let simd_ops_arc = get_simd_ops();
                        let mut simd_ops = simd_ops_arc.lock()
                            .map_err(|_| "Failed to acquire SIMD lock")?;
                        let mut result = vec![0.0; a_vals.len()];
                        simd_ops.add_f64_arrays(&a_vals, &b_vals, &mut result)
                            .map_err(|e| format!("SIMD error: {e}"))?;
                        Ok(Self::real_vector(result))
                    }
                    _ => {
                        // Fallback to element-wise addition
                        let result: Result<Vec<_>, _> = a.iter()
                            .zip(b.iter())
                            .map(|(x, y)| x.add(y))
                            .collect();
                        Ok(Self::Vector(result?))
                    }
                }
            }
            _ => Err("Cannot perform SIMD vector addition on non-matching vectors".to_string())
        }
    }

    /// SIMD-optimized vector multiplication for compatible vectors
    pub fn simd_vector_multiply(&self, other: &Self) -> Result<Self, String> {
        match (self, other) {
            (Self::Vector(a), Self::Vector(b)) if a.len() == b.len() => {
                let a_f64: Result<Vec<f64>, _> = a.iter()
                    .map(|v| v.to_f64().ok_or("Not convertible to f64"))
                    .collect();
                let b_f64: Result<Vec<f64>, _> = b.iter()
                    .map(|v| v.to_f64().ok_or("Not convertible to f64"))
                    .collect();

                match (a_f64, b_f64) {
                    (Ok(a_vals), Ok(b_vals)) => {
                        let simd_ops_arc = get_simd_ops();
                        let mut simd_ops = simd_ops_arc.lock()
                            .map_err(|_| "Failed to acquire SIMD lock")?;
                        let mut result = vec![0.0; a_vals.len()];
                        simd_ops.multiply_f64_arrays(&a_vals, &b_vals, &mut result)
                            .map_err(|e| format!("SIMD error: {e}"))?;
                        Ok(Self::real_vector(result))
                    }
                    _ => {
                        let result: Result<Vec<_>, _> = a.iter()
                            .zip(b.iter())
                            .map(|(x, y)| x.multiply(y))
                            .collect();
                        Ok(Self::Vector(result?))
                    }
                }
            }
            _ => Err("Cannot perform SIMD vector multiplication on non-matching vectors".to_string())
        }
    }

    /// SIMD-optimized dot product for compatible vectors
    pub fn simd_dot_product(&self, other: &Self) -> Result<Self, String> {
        match (self, other) {
            (Self::Vector(a), Self::Vector(b)) if a.len() == b.len() => {
                let a_f64: Result<Vec<f64>, _> = a.iter()
                    .map(|v| v.to_f64().ok_or("Not convertible to f64"))
                    .collect();
                let b_f64: Result<Vec<f64>, _> = b.iter()
                    .map(|v| v.to_f64().ok_or("Not convertible to f64"))
                    .collect();

                match (a_f64, b_f64) {
                    (Ok(a_vals), Ok(b_vals)) => {
                        let simd_ops_arc = get_simd_ops();
                        let mut simd_ops = simd_ops_arc.lock()
                            .map_err(|_| "Failed to acquire SIMD lock")?;
                        let result = simd_ops.dot_product_f64(&a_vals, &b_vals)
                            .map_err(|e| format!("SIMD error: {e}"))?;
                        Ok(Self::Real(result))
                    }
                    _ => {
                        // Fallback: compute sum of element-wise products
                        let products: Result<Vec<_>, _> = a.iter()
                            .zip(b.iter())
                            .map(|(x, y)| x.multiply(y))
                            .collect();
                        let sum = products?.into_iter()
                            .try_fold(Self::Integer(0), |acc, x| acc.add(&x))?;
                        Ok(sum)
                    }
                }
            }
            _ => Err("Cannot perform dot product on non-matching vectors".to_string())
        }
    }

    /// Extracts f64 values from a numeric vector if possible
    pub fn to_f64_vector(&self) -> Option<Vec<f64>> {
        match self {
            Self::Vector(v) => {
                let f64_vec: Result<Vec<f64>, _> = v.iter()
                    .map(|x| x.to_f64().ok_or(()))
                    .collect();
                f64_vec.ok()
            }
            _ => None
        }
    }

    /// Gets vector length if this is a vector
    pub fn vector_length(&self) -> Option<usize> {
        match self {
            Self::Vector(v) => Some(v.len()),
            _ => None
        }
    }

    /// Checks if this value can benefit from SIMD optimization
    pub fn is_simd_optimizable(&self) -> bool {
        match self {
            Self::Vector(v) => {
                v.len() >= 8 && // Minimum size for SIMD benefit
                v.iter().all(|x| x.to_f64().is_some()) // All elements convertible to f64
            }
            _ => false
        }
    }
}

impl fmt::Display for NumericValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(n) => write!(f, "{n}"),
            Self::BigInteger(n) => write!(f, "{n}"),
            Self::Rational(r) => write!(f, "{r}"),
            Self::Real(r) => {
                if r.fract() == 0.0 && r.is_finite() {
                    write!(f, "{}.0", *r as i64)
                } else {
                    write!(f, "{r}")
                }
            }
            Self::Complex(c) => write!(f, "{c}"),
            Self::Vector(v) => {
                write!(f, "#(")?;
                for (i, val) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{val}")?;
                }
                write!(f, ")")
            }
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
            Self::Vector(v) => v.hash(state),
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
        let real_val = NumericValue::real(std::f64::consts::PI);
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
        let real_val = NumericValue::real(std::f64::consts::PI);
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

    #[test]
    fn test_vector_creation_and_operations() {
        let vec_a = NumericValue::real_vector(vec![1.0, 2.0, 3.0, 4.0]);
        let vec_b = NumericValue::real_vector(vec![5.0, 6.0, 7.0, 8.0]);

        assert_eq!(vec_a.numeric_type(), NumericType::Vector);
        assert_eq!(vec_a.vector_length(), Some(4));
        assert!(vec_a.is_simd_optimizable());

        // Test SIMD vector addition
        let result = vec_a.simd_vector_add(&vec_b).unwrap();
        if let NumericValue::Vector(result_vals) = result {
            assert_eq!(result_vals.len(), 4);
            // Expected: [6.0, 8.0, 10.0, 12.0]
            assert_eq!(result_vals[0].to_f64().unwrap(), 6.0);
            assert_eq!(result_vals[1].to_f64().unwrap(), 8.0);
            assert_eq!(result_vals[2].to_f64().unwrap(), 10.0);
            assert_eq!(result_vals[3].to_f64().unwrap(), 12.0);
        } else {
            panic!("Expected vector result");
        }
    }

    #[test]
    fn test_simd_dot_product() {
        let vec_a = NumericValue::real_vector(vec![1.0, 2.0, 3.0, 4.0]);
        let vec_b = NumericValue::real_vector(vec![2.0, 3.0, 4.0, 5.0]);

        let result = vec_a.simd_dot_product(&vec_b).unwrap();
        // Expected: 1*2 + 2*3 + 3*4 + 4*5 = 2 + 6 + 12 + 20 = 40
        assert_eq!(result.to_f64().unwrap(), 40.0);
    }

    #[test]
    fn test_vector_predicates() {
        let positive_vec = NumericValue::real_vector(vec![1.0, 2.0, 3.0]);
        let mixed_vec = NumericValue::vector(vec![
            NumericValue::integer(1),
            NumericValue::real(2.5),
            NumericValue::integer(3)
        ]);

        assert!(positive_vec.is_positive());
        assert!(positive_vec.is_real());
        assert!(!positive_vec.is_exact());
        assert!(positive_vec.is_inexact());

        assert!(mixed_vec.is_positive());
        assert!(mixed_vec.is_real());
        assert!(!mixed_vec.is_exact()); // Contains real number
    }

    #[test]
    fn test_vector_display() {
        let vec = NumericValue::vector(vec![
            NumericValue::integer(1),
            NumericValue::real(2.5),
            NumericValue::rational(3, 4)
        ]);

        let display_str = format!("{}", vec);
        assert!(display_str.starts_with("#("));
        assert!(display_str.ends_with(")"));
    }

    #[test]
    fn test_vector_f64_extraction() {
        let real_vec = NumericValue::real_vector(vec![1.0, 2.0, 3.0]);
        let mixed_vec = NumericValue::vector(vec![
            NumericValue::integer(1),
            NumericValue::rational(3, 2), // 1.5
            NumericValue::complex(2.0, 1.0) // Not convertible
        ]);

        assert_eq!(real_vec.to_f64_vector(), Some(vec![1.0, 2.0, 3.0]));
        assert_eq!(mixed_vec.to_f64_vector(), None); // Contains complex number
    }
}
//! Complex number implementation optimized for R7RS-large
//!
//! Provides high-performance complex arithmetic with support for:
//! - Rectangular and polar coordinate systems
//! - Trigonometric and exponential functions
//! - Optimized arithmetic operations
//! - IEEE 754 compliance for special values

use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Sub, Mul, Div, Neg};

/// High-performance complex number with optimized operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub real: f64,
    pub imaginary: f64,
}

impl Complex {
    /// Creates a new complex number
    pub fn new(real: f64, imaginary: f64) -> Self {
        Self { real, imaginary }
    }

    /// Creates a complex number from real part only
    pub fn from_real(real: f64) -> Self {
        Self::new(real, 0.0)
    }

    /// Creates a complex number from imaginary part only
    pub fn from_imaginary(imaginary: f64) -> Self {
        Self::new(0.0, imaginary)
    }

    /// Creates a complex number from polar coordinates (magnitude, angle)
    pub fn from_polar(magnitude: f64, angle: f64) -> Self {
        Self::new(
            magnitude * angle.cos(),
            magnitude * angle.sin(),
        )
    }

    /// Zero complex number
    pub const ZERO: Self = Self { real: 0.0, imaginary: 0.0 };

    /// One complex number
    pub const ONE: Self = Self { real: 1.0, imaginary: 0.0 };

    /// Imaginary unit (i)
    pub const I: Self = Self { real: 0.0, imaginary: 1.0 };

    /// Negative imaginary unit (-i)
    pub const NEG_I: Self = Self { real: 0.0, imaginary: -1.0 };

    /// Computes the magnitude (absolute value) of the complex number
    pub fn magnitude(&self) -> f64 {
        self.real.hypot(self.imaginary)
    }

    /// Computes the squared magnitude (avoids sqrt for performance)
    pub fn magnitude_squared(&self) -> f64 {
        self.real * self.real + self.imaginary * self.imaginary
    }

    /// Computes the argument (angle) of the complex number
    pub fn argument(&self) -> f64 {
        self.imaginary.atan2(self.real)
    }

    /// Returns the complex conjugate
    pub fn conjugate(&self) -> Self {
        Self::new(self.real, -self.imaginary)
    }

    /// Checks if this is a real number (imaginary part is zero)
    pub fn is_real(&self) -> bool {
        self.imaginary == 0.0 || self.imaginary.abs() < f64::EPSILON
    }

    /// Checks if this is a pure imaginary number (real part is zero)
    pub fn is_imaginary(&self) -> bool {
        self.real == 0.0 || self.real.abs() < f64::EPSILON
    }

    /// Checks if this is zero
    pub fn is_zero(&self) -> bool {
        self.real == 0.0 && self.imaginary == 0.0
    }

    /// Checks if this is finite (both parts are finite)
    pub fn is_finite(&self) -> bool {
        self.real.is_finite() && self.imaginary.is_finite()
    }

    /// Checks if this is infinite (either part is infinite)
    pub fn is_infinite(&self) -> bool {
        self.real.is_infinite() || self.imaginary.is_infinite()
    }

    /// Checks if this is NaN (either part is NaN)
    pub fn is_nan(&self) -> bool {
        self.real.is_nan() || self.imaginary.is_nan()
    }

    /// Raises this complex number to a real power
    pub fn powf(&self, exponent: f64) -> Self {
        if self.is_zero() {
            if exponent > 0.0 {
                Self::ZERO
            } else if exponent == 0.0 {
                Self::ONE
            } else {
                Self::new(f64::INFINITY, f64::NAN)
            }
        } else {
            let magnitude = self.magnitude();
            let argument = self.argument();
            let new_magnitude = magnitude.powf(exponent);
            let new_argument = argument * exponent;
            Self::from_polar(new_magnitude, new_argument)
        }
    }

    /// Raises this complex number to a complex power
    pub fn pow(&self, exponent: Complex) -> Self {
        if self.is_zero() {
            if exponent.is_zero() {
                Self::ONE
            } else if exponent.real > 0.0 || exponent.imaginary != 0.0 {
                Self::ZERO
            } else {
                Self::new(f64::INFINITY, f64::NAN)
            }
        } else {
            // z^w = exp(w * ln(z))
            (exponent * self.ln()).exp()
        }
    }

    /// Natural logarithm
    pub fn ln(&self) -> Self {
        Self::new(self.magnitude().ln(), self.argument())
    }

    /// Exponential function
    pub fn exp(&self) -> Self {
        let exp_real = self.real.exp();
        Self::new(
            exp_real * self.imaginary.cos(),
            exp_real * self.imaginary.sin(),
        )
    }

    /// Square root
    pub fn sqrt(&self) -> Self {
        if self.is_real() && self.real >= 0.0 {
            Self::from_real(self.real.sqrt())
        } else {
            let magnitude = self.magnitude().sqrt();
            let argument = self.argument() / 2.0;
            Self::from_polar(magnitude, argument)
        }
    }

    /// Sine function
    pub fn sin(&self) -> Self {
        Self::new(
            self.real.sin() * self.imaginary.cosh(),
            self.real.cos() * self.imaginary.sinh(),
        )
    }

    /// Cosine function
    pub fn cos(&self) -> Self {
        Self::new(
            self.real.cos() * self.imaginary.cosh(),
            -self.real.sin() * self.imaginary.sinh(),
        )
    }

    /// Tangent function
    pub fn tan(&self) -> Self {
        let sin = self.sin();
        let cos = self.cos();
        sin / cos
    }

    /// Hyperbolic sine function
    pub fn sinh(&self) -> Self {
        Self::new(
            self.real.sinh() * self.imaginary.cos(),
            self.real.cosh() * self.imaginary.sin(),
        )
    }

    /// Hyperbolic cosine function
    pub fn cosh(&self) -> Self {
        Self::new(
            self.real.cosh() * self.imaginary.cos(),
            self.real.sinh() * self.imaginary.sin(),
        )
    }

    /// Hyperbolic tangent function
    pub fn tanh(&self) -> Self {
        let sinh = self.sinh();
        let cosh = self.cosh();
        sinh / cosh
    }

    /// Arcsine function
    pub fn asin(&self) -> Self {
        // asin(z) = -i * ln(i*z + sqrt(1 - z^2))
        let i = Self::I;
        -i * (i * *self + (Self::ONE - *self * *self).sqrt()).ln()
    }

    /// Arccosine function
    pub fn acos(&self) -> Self {
        // acos(z) = -i * ln(z + i * sqrt(1 - z^2))
        let i = Self::I;
        -i * (*self + i * (Self::ONE - *self * *self).sqrt()).ln()
    }

    /// Arctangent function
    pub fn atan(&self) -> Self {
        // atan(z) = (i/2) * ln((i + z) / (i - z))
        let i = Self::I;
        let half_i = Self::new(0.0, 0.5);
        half_i * ((i + *self) / (i - *self)).ln()
    }

    /// Converts to polar representation (magnitude, argument)
    pub fn to_polar(&self) -> (f64, f64) {
        (self.magnitude(), self.argument())
    }

    /// Formats as a string with given precision
    pub fn format_with_precision(&self, precision: usize) -> String {
        if self.is_real() {
            format!("{:.prec$}", self.real, prec = precision)
        } else if self.real == 0.0 {
            if self.imaginary == 1.0 {
                "i".to_string()
            } else if self.imaginary == -1.0 {
                "-i".to_string()
            } else {
                format!("{:.prec$}i", self.imaginary, prec = precision)
            }
        } else if self.imaginary > 0.0 {
            if self.imaginary == 1.0 {
                format!("{:.prec$}+i", self.real, prec = precision)
            } else {
                format!("{:.prec$}+{:.prec$}i", self.real, self.imaginary, prec = precision)
            }
        } else if self.imaginary == -1.0 {
            format!("{:.prec$}-i", self.real, prec = precision)
        } else {
            format!("{:.prec$}{:.prec$}i", self.real, self.imaginary, prec = precision)
        }
    }
}

// Arithmetic operations

impl Add for Complex {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.real + other.real, self.imaginary + other.imaginary)
    }
}

impl Sub for Complex {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.real - other.real, self.imaginary - other.imaginary)
    }
}

impl Mul for Complex {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        // (a + bi) * (c + di) = (ac - bd) + (ad + bc)i
        Self::new(
            self.real * other.real - self.imaginary * other.imaginary,
            self.real * other.imaginary + self.imaginary * other.real,
        )
    }
}

impl Div for Complex {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        if other.is_zero() {
            // Division by zero results in infinity
            Self::new(f64::INFINITY, f64::NAN)
        } else {
            let denominator = other.magnitude_squared();
            let numerator = self * other.conjugate();
            Self::new(
                numerator.real / denominator,
                numerator.imaginary / denominator,
            )
        }
    }
}

impl Neg for Complex {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.real, -self.imaginary)
    }
}

// Operations with real numbers

impl Add<f64> for Complex {
    type Output = Self;

    fn add(self, other: f64) -> Self {
        Self::new(self.real + other, self.imaginary)
    }
}

impl Sub<f64> for Complex {
    type Output = Self;

    fn sub(self, other: f64) -> Self {
        Self::new(self.real - other, self.imaginary)
    }
}

impl Mul<f64> for Complex {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        Self::new(self.real * other, self.imaginary * other)
    }
}

impl Div<f64> for Complex {
    type Output = Self;

    fn div(self, other: f64) -> Self {
        if other == 0.0 {
            Self::new(f64::INFINITY, f64::INFINITY)
        } else {
            Self::new(self.real / other, self.imaginary / other)
        }
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_real() {
            write!(f, "{}", self.real)
        } else if self.real == 0.0 {
            if self.imaginary == 1.0 {
                write!(f, "i")
            } else if self.imaginary == -1.0 {
                write!(f, "-i")
            } else {
                write!(f, "{}i", self.imaginary)
            }
        } else if self.imaginary > 0.0 {
            if self.imaginary == 1.0 {
                write!(f, "{}+i", self.real)
            } else {
                write!(f, "{}+{}i", self.real, self.imaginary)
            }
        } else if self.imaginary == -1.0 {
            write!(f, "{}-i", self.real)
        } else {
            write!(f, "{}{}i", self.real, self.imaginary)
        }
    }
}

impl Hash for Complex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.real.to_bits().hash(state);
        self.imaginary.to_bits().hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_complex_creation() {
        let c1 = Complex::new(3.0, 4.0);
        let c2 = Complex::from_real(5.0);
        let c3 = Complex::from_imaginary(2.0);
        let c4 = Complex::from_polar(5.0, PI / 2.0);

        assert_eq!(c1.real, 3.0);
        assert_eq!(c1.imaginary, 4.0);
        assert_eq!(c2.real, 5.0);
        assert_eq!(c2.imaginary, 0.0);
        assert_eq!(c3.real, 0.0);
        assert_eq!(c3.imaginary, 2.0);
        assert!((c4.real - 0.0).abs() < 1e-10);
        assert!((c4.imaginary - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_complex_arithmetic() {
        let c1 = Complex::new(1.0, 2.0);
        let c2 = Complex::new(3.0, 4.0);

        let sum = c1 + c2;
        assert_eq!(sum, Complex::new(4.0, 6.0));

        let diff = c2 - c1;
        assert_eq!(diff, Complex::new(2.0, 2.0));

        let prod = c1 * c2;
        assert_eq!(prod, Complex::new(-5.0, 10.0));

        let quot = c2 / c1;
        assert_eq!(quot, Complex::new(2.2, 0.4));
    }

    #[test]
    fn test_complex_magnitude_and_argument() {
        let c = Complex::new(3.0, 4.0);
        assert_eq!(c.magnitude(), 5.0);
        assert!((c.argument() - 0.9272952180016122).abs() < 1e-10);

        let polar = c.to_polar();
        assert_eq!(polar.0, 5.0);
        assert!((polar.1 - 0.9272952180016122).abs() < 1e-10);
    }

    #[test]
    fn test_complex_power() {
        let c = Complex::new(1.0, 1.0);
        let c_squared = c.powf(2.0);
        let expected = Complex::new(0.0, 2.0);
        
        assert!((c_squared.real - expected.real).abs() < 1e-10);
        assert!((c_squared.imaginary - expected.imaginary).abs() < 1e-10);
    }

    #[test]
    fn test_complex_sqrt() {
        let c = Complex::new(-1.0, 0.0);
        let sqrt_c = c.sqrt();
        let expected = Complex::new(0.0, 1.0);
        
        assert!((sqrt_c.real - expected.real).abs() < 1e-10);
        assert!((sqrt_c.imaginary - expected.imaginary).abs() < 1e-10);
    }

    #[test]
    fn test_complex_exp_ln() {
        let c = Complex::new(1.0, PI);
        let exp_c = c.exp();
        let ln_exp_c = exp_c.ln();
        
        // exp and ln should be inverses (modulo 2πi)
        assert!((ln_exp_c.real - c.real).abs() < 1e-10);
        assert!((ln_exp_c.imaginary - c.imaginary).abs() < 1e-10);
    }

    #[test]
    fn test_complex_trigonometric() {
        let c = Complex::new(0.5, 0.3);
        let sin_c = c.sin();
        let cos_c = c.cos();
        
        // sin^2 + cos^2 = 1
        let identity = sin_c * sin_c + cos_c * cos_c;
        assert!((identity.real - 1.0).abs() < 1e-10);
        assert!(identity.imaginary.abs() < 1e-10);
    }

    #[test]
    fn test_complex_display() {
        assert_eq!(format!("{}", Complex::new(3.0, 0.0)), "3");
        assert_eq!(format!("{}", Complex::new(0.0, 1.0)), "i");
        assert_eq!(format!("{}", Complex::new(0.0, -1.0)), "-i");
        assert_eq!(format!("{}", Complex::new(3.0, 4.0)), "3+4i");
        assert_eq!(format!("{}", Complex::new(3.0, -4.0)), "3-4i");
        assert_eq!(format!("{}", Complex::new(3.0, 1.0)), "3+i");
        assert_eq!(format!("{}", Complex::new(3.0, -1.0)), "3-i");
    }
}
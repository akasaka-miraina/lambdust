//! Rational number implementation with optimized arithmetic
//!
//! Provides exact rational arithmetic using GCD-based reduction and
//! optimized algorithms for common operations.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Sub, Mul, Div, Neg};

/// Exact rational number representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    pub numerator: i64,
    pub denominator: i64,
}

impl Rational {
    /// Creates a new rational number with automatic reduction
    pub fn new(numerator: i64, denominator: i64) -> Self {
        if denominator == 0 {
            panic!("Rational number cannot have zero denominator");
        }

        if numerator == 0 {
            return Self {
                numerator: 0,
                denominator: 1,
            };
        }

        // Reduce the fraction using GCD
        let gcd = gcd(numerator.unsigned_abs(), denominator.unsigned_abs()) as i64;
        let mut num = numerator / gcd;
        let mut den = denominator / gcd;

        // Ensure denominator is positive
        if den < 0 {
            num = -num;
            den = -den;
        }

        Self {
            numerator: num,
            denominator: den,
        }
    }

    /// Creates a rational from an integer
    pub fn from_integer(n: i64) -> Self {
        Self {
            numerator: n,
            denominator: 1,
        }
    }

    /// Zero rational number
    pub const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };

    /// One rational number
    pub const ONE: Self = Self {
        numerator: 1,
        denominator: 1,
    };

    /// Negative one rational number
    pub const NEG_ONE: Self = Self {
        numerator: -1,
        denominator: 1,
    };

    /// Half rational number
    pub const HALF: Self = Self {
        numerator: 1,
        denominator: 2,
    };

    /// Checks if this rational is zero
    pub fn is_zero(&self) -> bool {
        self.numerator == 0
    }

    /// Checks if this rational is positive
    pub fn is_positive(&self) -> bool {
        self.numerator > 0
    }

    /// Checks if this rational is negative
    pub fn is_negative(&self) -> bool {
        self.numerator < 0
    }

    /// Checks if this rational is an integer
    pub fn is_integer(&self) -> bool {
        self.denominator == 1
    }

    /// Returns the absolute value
    pub fn abs(&self) -> Self {
        Self {
            numerator: self.numerator.abs(),
            denominator: self.denominator,
        }
    }

    /// Returns the reciprocal
    pub fn reciprocal(&self) -> Self {
        if self.numerator == 0 {
            panic!("Cannot compute reciprocal of zero");
        }
        Self::new(self.denominator, self.numerator)
    }

    /// Converts to floating point (with potential precision loss)
    pub fn to_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    /// Converts to integer if possible (exact integers only)
    pub fn to_i64(&self) -> Option<i64> {
        if self.denominator == 1 {
            Some(self.numerator)
        } else {
            None
        }
    }

    /// Raises this rational to an integer power
    pub fn powi(&self, exponent: i32) -> Self {
        if exponent == 0 {
            Self::ONE
        } else if exponent > 0 {
            Self {
                numerator: self.numerator.pow(exponent as u32),
                denominator: self.denominator.pow(exponent as u32),
            }
        } else {
            let abs_exp = (-exponent) as u32;
            Self {
                numerator: self.denominator.pow(abs_exp),
                denominator: self.numerator.pow(abs_exp),
            }
        }
    }

    /// Returns the floor of this rational
    pub fn floor(&self) -> i64 {
        if self.numerator >= 0 {
            self.numerator / self.denominator
        } else {
            (self.numerator - self.denominator + 1) / self.denominator
        }
    }

    /// Returns the ceiling of this rational
    pub fn ceil(&self) -> i64 {
        if self.numerator >= 0 {
            (self.numerator + self.denominator - 1) / self.denominator
        } else {
            self.numerator / self.denominator
        }
    }

    /// Returns the truncated value (towards zero)
    pub fn trunc(&self) -> i64 {
        self.numerator / self.denominator
    }

    /// Returns the fractional part
    pub fn fract(&self) -> Self {
        let integer_part = self.trunc();
        *self - Self::from_integer(integer_part)
    }

    /// Continued fraction representation (partial)
    pub fn to_continued_fraction(&self, max_terms: usize) -> Vec<i64> {
        let mut result = Vec::new();
        let mut num = self.numerator;
        let mut den = self.denominator;

        for _ in 0..max_terms {
            if den == 0 {
                break;
            }

            let quotient = num / den;
            result.push(quotient);

            let remainder = num % den;
            num = den;
            den = remainder;
        }

        result
    }

    /// Creates a rational from a continued fraction
    pub fn from_continued_fraction(terms: &[i64]) -> Self {
        if terms.is_empty() {
            return Self::ZERO;
        }

        let mut result = Self::from_integer(terms[terms.len() - 1]);

        for &term in terms.iter().rev().skip(1) {
            result = result.reciprocal() + Self::from_integer(term);
        }

        result
    }

    /// Mediant of two rationals (used in Farey sequences)
    pub fn mediant(&self, other: &Self) -> Self {
        Self::new(
            self.numerator + other.numerator,
            self.denominator + other.denominator,
        )
    }
}

impl Add for Rational {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        // a/b + c/d = (ad + bc) / (bd)
        let numerator = self.numerator * other.denominator + other.numerator * self.denominator;
        let denominator = self.denominator * other.denominator;
        Self::new(numerator, denominator)
    }
}

impl Sub for Rational {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        // a/b - c/d = (ad - bc) / (bd)
        let numerator = self.numerator * other.denominator - other.numerator * self.denominator;
        let denominator = self.denominator * other.denominator;
        Self::new(numerator, denominator)
    }
}

impl Mul for Rational {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        // (a/b) * (c/d) = (ac) / (bd)
        Self::new(
            self.numerator * other.numerator,
            self.denominator * other.denominator,
        )
    }
}

impl Div for Rational {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        if other.is_zero() {
            panic!("Division by zero");
        }
        // (a/b) / (c/d) = (a/b) * (d/c) = (ad) / (bc)
        Self::new(
            self.numerator * other.denominator,
            self.denominator * other.numerator,
        )
    }
}

impl Neg for Rational {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            numerator: -self.numerator,
            denominator: self.denominator,
        }
    }
}

// Operations with integers

impl Add<i64> for Rational {
    type Output = Self;

    fn add(self, other: i64) -> Self {
        self + Self::from_integer(other)
    }
}

impl Sub<i64> for Rational {
    type Output = Self;

    fn sub(self, other: i64) -> Self {
        self - Self::from_integer(other)
    }
}

impl Mul<i64> for Rational {
    type Output = Self;

    fn mul(self, other: i64) -> Self {
        Self::new(self.numerator * other, self.denominator)
    }
}

impl Div<i64> for Rational {
    type Output = Self;

    fn div(self, other: i64) -> Self {
        if other == 0 {
            panic!("Division by zero");
        }
        Self::new(self.numerator, self.denominator * other)
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare a/b with c/d by comparing ad with bc
        let left = self.numerator * other.denominator;
        let right = other.numerator * self.denominator;
        left.cmp(&right)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

impl Hash for Rational {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.numerator.hash(state);
        self.denominator.hash(state);
    }
}

/// Computes the greatest common divisor using Euclid's algorithm
pub fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Computes the least common multiple
pub fn lcm(a: u64, b: u64) -> u64 {
    if a == 0 && b == 0 {
        0
    } else {
        (a / gcd(a, b)) * b
    }
}

/// Extended Euclidean algorithm
/// Returns (gcd, x, y) such that ax + by = gcd
pub fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x, y) = extended_gcd(b, a % b);
        (g, y, x - (a / b) * y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rational_creation() {
        let r1 = Rational::new(3, 4);
        assert_eq!(r1.numerator, 3);
        assert_eq!(r1.denominator, 4);

        let r2 = Rational::new(6, 8);
        assert_eq!(r2.numerator, 3);
        assert_eq!(r2.denominator, 4);

        let r3 = Rational::new(-3, 4);
        assert_eq!(r3.numerator, -3);
        assert_eq!(r3.denominator, 4);

        let r4 = Rational::new(3, -4);
        assert_eq!(r4.numerator, -3);
        assert_eq!(r4.denominator, 4);
    }

    #[test]
    fn test_rational_arithmetic() {
        let r1 = Rational::new(1, 2);
        let r2 = Rational::new(1, 3);

        let sum = r1 + r2;
        assert_eq!(sum, Rational::new(5, 6));

        let diff = r1 - r2;
        assert_eq!(diff, Rational::new(1, 6));

        let prod = r1 * r2;
        assert_eq!(prod, Rational::new(1, 6));

        let quot = r1 / r2;
        assert_eq!(quot, Rational::new(3, 2));
    }

    #[test]
    fn test_rational_comparison() {
        let r1 = Rational::new(1, 2);
        let r2 = Rational::new(2, 4);
        let r3 = Rational::new(1, 3);

        assert_eq!(r1, r2);
        assert!(r1 > r3);
        assert!(r3 < r1);
    }

    #[test]
    fn test_rational_power() {
        let r = Rational::new(2, 3);
        let r_squared = r.powi(2);
        assert_eq!(r_squared, Rational::new(4, 9));

        let r_inv = r.powi(-1);
        assert_eq!(r_inv, Rational::new(3, 2));
    }

    #[test]
    fn test_continued_fraction() {
        let r = Rational::new(22, 7); // Approximation of π
        let cf = r.to_continued_fraction(10);
        assert_eq!(cf, vec![3, 7]);

        let reconstructed = Rational::from_continued_fraction(&cf);
        assert_eq!(reconstructed, r);
    }

    #[test]
    fn test_rational_display() {
        assert_eq!(format!("{}", Rational::new(3, 4)), "3/4");
        assert_eq!(format!("{}", Rational::new(5, 1)), "5");
        assert_eq!(format!("{}", Rational::new(-3, 4)), "-3/4");
        assert_eq!(format!("{}", Rational::new(0, 1)), "0");
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(48, 18), 6);
        assert_eq!(gcd(17, 13), 1);
        assert_eq!(gcd(0, 5), 5);
        assert_eq!(gcd(5, 0), 5);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm(4, 6), 12);
        assert_eq!(lcm(17, 13), 221);
        assert_eq!(lcm(0, 5), 0);
    }

    #[test]
    fn test_extended_gcd() {
        let (g, x, y) = extended_gcd(240, 46);
        assert_eq!(g, 2);
        assert_eq!(240 * x + 46 * y, g);
    }
}
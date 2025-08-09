//! Big integer implementation for arbitrary precision arithmetic
//!
//! Provides unlimited precision integer arithmetic with optimized algorithms
//! for common operations including multiplication, division, and modular arithmetic.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Sub, Mul, Div, Rem, Neg, Shl, Shr};

/// Arbitrary precision integer
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigInt {
    /// Digits stored in little-endian order (least significant first)
    /// Each digit is a u32 to allow for efficient multiplication
    digits: Vec<u32>,
    /// Sign: true for positive, false for negative
    positive: bool,
}

const BASE: u64 = 1u64 << 32; // 2^32
const BASE_MASK: u64 = BASE - 1;

impl BigInt {
    /// Creates a new BigInt from an i64
    pub fn from_i64(value: i64) -> Self {
        if value == 0 {
            return Self::zero();
        }

        let positive = value >= 0;
        let abs_value = value.unsigned_abs();
        
        let mut digits = Vec::new();
        let mut remaining = abs_value;
        
        while remaining > 0 {
            digits.push((remaining & BASE_MASK) as u32);
            remaining >>= 32;
        }

        Self { digits, positive }
    }

    /// Creates a new BigInt from a u64
    pub fn from_u64(value: u64) -> Self {
        if value == 0 {
            return Self::zero();
        }

        let mut digits = Vec::new();
        let mut remaining = value;
        
        while remaining > 0 {
            digits.push((remaining & BASE_MASK) as u32);
            remaining >>= 32;
        }

        Self { digits, positive: true }
    }

    /// Creates a BigInt from a string in the given radix (2-36)
    pub fn from_str_radix(s: &str, radix: u32) -> Result<Self, String> {
        if !(2..=36).contains(&radix) {
            return Err("Radix must be between 2 and 36".to_string());
        }

        let s = s.trim();
        if s.is_empty() {
            return Err("Empty string".to_string());
        }

        let (positive, digits_str) = if let Some(stripped) = s.strip_prefix('-') {
            (false, stripped)
        } else if let Some(stripped) = s.strip_prefix('+') {
            (true, stripped)
        } else {
            (true, s)
        };

        if digits_str.is_empty() {
            return Err("No digits after sign".to_string());
        }

        let mut result = Self::zero();
        let radix_bigint = Self::from_u64(radix as u64);

        for ch in digits_str.chars() {
            let digit_value = match ch.to_digit(radix) {
                Some(d) => d,
                None => return Err(format!("Invalid digit '{ch}' for radix {radix}")),
            };

            result = result * radix_bigint.clone() + Self::from_u64(digit_value as u64);
        }

        result.positive = positive || result.is_zero();
        Ok(result)
    }

    /// Zero constant
    pub fn zero() -> Self {
        Self {
            digits: vec![],
            positive: true,
        }
    }

    /// One constant
    pub fn one() -> Self {
        Self {
            digits: vec![1],
            positive: true,
        }
    }

    /// Checks if this BigInt is zero
    pub fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    /// Checks if this BigInt is positive
    pub fn is_positive(&self) -> bool {
        self.positive && !self.is_zero()
    }

    /// Checks if this BigInt is negative
    pub fn is_negative(&self) -> bool {
        !self.positive && !self.is_zero()
    }

    /// Returns the absolute value
    pub fn abs(&self) -> Self {
        Self {
            digits: self.digits.clone(),
            positive: true,
        }
    }

    /// Converts to i64 if possible
    pub fn to_i64(&self) -> Option<i64> {
        if self.is_zero() {
            return Some(0);
        }

        if self.digits.len() > 2 {
            return None; // Too large
        }

        let mut value = 0u64;
        for (i, &digit) in self.digits.iter().enumerate() {
            value |= (digit as u64) << (32 * i);
        }

        if self.positive {
            if value <= i64::MAX as u64 {
                Some(value as i64)
            } else {
                None
            }
        } else if value <= (i64::MAX as u64) + 1 {
            Some(-(value as i64))
        } else {
            None
        }
    }

    /// Converts to f64 (with potential precision loss)
    pub fn to_f64(&self) -> Option<f64> {
        if self.is_zero() {
            return Some(0.0);
        }

        let mut result = 0.0;
        let mut base_power = 1.0;

        for &digit in &self.digits {
            result += (digit as f64) * base_power;
            base_power *= BASE as f64;
            
            if !result.is_finite() {
                return None; // Overflow
            }
        }

        Some(if self.positive { result } else { -result })
    }

    /// Returns the number of bits required to represent this BigInt
    pub fn bits(&self) -> usize {
        if self.is_zero() {
            return 0;
        }

        let most_significant_digit = self.digits[self.digits.len() - 1];
        let leading_zeros = most_significant_digit.leading_zeros() as usize;
        (self.digits.len() - 1) * 32 + (32 - leading_zeros)
    }

    /// Removes leading zero digits
    fn normalize(&mut self) {
        while self.digits.last() == Some(&0) {
            self.digits.pop();
        }
        
        if self.digits.is_empty() {
            self.positive = true; // Zero is positive
        }
    }

    /// Compares the absolute values of two BigInts
    fn abs_cmp(&self, other: &Self) -> Ordering {
        match self.digits.len().cmp(&other.digits.len()) {
            Ordering::Equal => {
                for (a, b) in self.digits.iter().rev().zip(other.digits.iter().rev()) {
                    match a.cmp(b) {
                        Ordering::Equal => continue,
                        other => return other,
                    }
                }
                Ordering::Equal
            }
            other => other,
        }
    }

    /// Addition of absolute values
    fn abs_add(&self, other: &Self) -> Self {
        let mut result = Vec::new();
        let mut carry = 0u64;
        let max_len = self.digits.len().max(other.digits.len());

        for i in 0..max_len {
            let a = self.digits.get(i).copied().unwrap_or(0) as u64;
            let b = other.digits.get(i).copied().unwrap_or(0) as u64;
            let sum = a + b + carry;
            
            result.push((sum & BASE_MASK) as u32);
            carry = sum >> 32;
        }

        if carry > 0 {
            result.push(carry as u32);
        }

        let mut bigint = Self {
            digits: result,
            positive: true,
        };
        bigint.normalize();
        bigint
    }

    /// Subtraction of absolute values (assumes self >= other)
    fn abs_sub(&self, other: &Self) -> Self {
        let mut result = Vec::new();
        let mut borrow = 0i64;

        for i in 0..self.digits.len() {
            let a = self.digits[i] as i64;
            let b = other.digits.get(i).copied().unwrap_or(0) as i64;
            let diff = a - b - borrow;
            
            if diff < 0 {
                result.push((diff + BASE as i64) as u32);
                borrow = 1;
            } else {
                result.push(diff as u32);
                borrow = 0;
            }
        }

        let mut bigint = Self {
            digits: result,
            positive: true,
        };
        bigint.normalize();
        bigint
    }

    /// Multiplication using grade school algorithm
    /// For large numbers, this could be optimized with Karatsuba or FFT
    fn multiply(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }

        let mut result = vec![0u32; self.digits.len() + other.digits.len()];

        for (i, &a) in self.digits.iter().enumerate() {
            let mut carry = 0u64;
            for (j, &b) in other.digits.iter().enumerate() {
                let product = (a as u64) * (b as u64) + (result[i + j] as u64) + carry;
                result[i + j] = (product & BASE_MASK) as u32;
                carry = product >> 32;
            }
            if carry > 0 {
                result[i + other.digits.len()] = carry as u32;
            }
        }

        let mut bigint = Self {
            digits: result,
            positive: self.positive == other.positive,
        };
        bigint.normalize();
        bigint
    }

    /// Division with remainder
    pub fn div_rem(&self, other: &Self) -> (Self, Self) {
        if other.is_zero() {
            panic!("Division by zero");
        }

        if self.is_zero() {
            return (Self::zero(), Self::zero());
        }

        let abs_cmp = self.abs_cmp(other);
        match abs_cmp {
            Ordering::Less => (Self::zero(), self.clone()),
            Ordering::Equal => {
                let quotient = if self.positive == other.positive {
                    Self::one()
                } else {
                    -Self::one()
                };
                (quotient, Self::zero())
            }
            Ordering::Greater => {
                // Use long division algorithm
                self.long_division(other)
            }
        }
    }

    /// Long division algorithm for big integers
    fn long_division(&self, divisor: &Self) -> (Self, Self) {
        let mut quotient = Self::zero();
        let mut remainder = Self::zero();

        for &digit in self.digits.iter().rev() {
            remainder = remainder.multiply(&Self::from_u64(BASE));
            remainder = remainder.abs_add(&Self::from_u64(digit as u64));

            let mut digit_quotient = 0u32;
            while remainder.abs_cmp(&divisor.abs()) != Ordering::Less {
                remainder = remainder.abs_sub(&divisor.abs());
                digit_quotient += 1;
            }

            quotient = quotient.multiply(&Self::from_u64(BASE));
            quotient = quotient.abs_add(&Self::from_u64(digit_quotient as u64));
        }

        quotient.positive = self.positive == divisor.positive;
        remainder.positive = self.positive;

        quotient.normalize();
        remainder.normalize();

        (quotient, remainder)
    }

    /// Modular exponentiation: self^exponent mod modulus
    pub fn mod_exp(&self, exponent: &Self, modulus: &Self) -> Self {
        if modulus.is_zero() {
            panic!("Modulus cannot be zero");
        }

        if exponent.is_zero() {
            return Self::one() % modulus.clone();
        }

        let mut result = Self::one();
        let mut base = self % modulus;
        let mut exp = exponent.abs();

        while !exp.is_zero() {
            if exp.digits.first().unwrap_or(&0) & 1 == 1 {
                result = (result * base.clone()) % modulus.clone();
            }
            base = (base.clone()) * base.clone() % modulus.clone();
            exp = exp >> 1;
        }

        result
    }

    /// GCD using Euclidean algorithm
    pub fn gcd(&self, other: &Self) -> Self {
        let mut a = self.abs();
        let mut b = other.abs();

        while !b.is_zero() {
            let temp = b.clone();
            b = &a % &b;
            a = temp;
        }

        a
    }

    /// LCM calculation
    pub fn lcm(&self, other: &Self) -> Self {
        if self.is_zero() && other.is_zero() {
            Self::zero()
        } else {
            let gcd = self.gcd(other);
            (self.abs() / gcd.clone()) * other.abs()
        }
    }

    /// Prime factorization trial division (basic implementation)
    pub fn factor(&self) -> Vec<Self> {
        if self.is_zero() || self.abs_cmp(&Self::one()) == Ordering::Equal {
            return vec![];
        }

        let mut factors = Vec::new();
        let mut n = self.abs();
        let mut divisor = Self::from_i64(2);

        while &divisor * &divisor <= n {
            while &n % &divisor == Self::zero() {
                factors.push(divisor.clone());
                n = n / divisor.clone();
            }
            divisor = divisor + Self::one();
        }

        if n > Self::one() {
            factors.push(n);
        }

        factors
    }

    /// Miller-Rabin primality test (probabilistic)
    pub fn is_prime(&self, rounds: usize) -> bool {
        if self <= &Self::one() {
            return false;
        }
        if self == &Self::from_i64(2) || self == &Self::from_i64(3) {
            return true;
        }
        if self.digits.first().unwrap_or(&0) & 1 == 0 {
            return false; // Even number
        }

        // Write n-1 as d * 2^r
        let n_minus_1 = self - &Self::one();
        let mut d = n_minus_1.clone();
        let mut r = 0;
        while d.digits.first().unwrap_or(&0) & 1 == 0 {
            d = d >> 1;
            r += 1;
        }

        // Witness loop
        for _ in 0..rounds {
            let a = Self::from_i64(2); // Simplified witness selection
            let mut x = a.mod_exp(&d, self);
            
            if x == Self::one() || x == n_minus_1 {
                continue;
            }

            let mut composite = true;
            for _ in 0..r - 1 {
                x = x.mod_exp(&Self::from_i64(2), self);
                if x == n_minus_1 {
                    composite = false;
                    break;
                }
            }

            if composite {
                return false;
            }
        }

        true
    }
}

// Arithmetic operations

impl Add for BigInt {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        &self + &other
    }
}

impl Add for &BigInt {
    type Output = BigInt;

    fn add(self, other: &BigInt) -> BigInt {
        match (self.positive, other.positive) {
            (true, true) => self.abs_add(other),
            (false, false) => {
                let mut result = self.abs_add(other);
                result.positive = false;
                result
            }
            (true, false) => match self.abs_cmp(other) {
                Ordering::Greater | Ordering::Equal => self.abs_sub(other),
                Ordering::Less => {
                    let mut result = other.abs_sub(self);
                    result.positive = false;
                    result
                }
            },
            (false, true) => match self.abs_cmp(other) {
                Ordering::Greater => {
                    let mut result = self.abs_sub(other);
                    result.positive = false;
                    result
                }
                Ordering::Less | Ordering::Equal => other.abs_sub(self),
            },
        }
    }
}

impl Sub for BigInt {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        &self - &other
    }
}

impl Sub for &BigInt {
    type Output = BigInt;

    fn sub(self, other: &BigInt) -> BigInt {
        self + &(-other.clone())
    }
}

impl Mul for BigInt {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        &self * &other
    }
}

impl Mul for &BigInt {
    type Output = BigInt;

    fn mul(self, other: &BigInt) -> BigInt {
        self.multiply(other)
    }
}

impl Div for BigInt {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        &self / &other
    }
}

impl Div for &BigInt {
    type Output = BigInt;

    fn div(self, other: &BigInt) -> BigInt {
        let (quotient, _) = self.div_rem(other);
        quotient
    }
}

impl Rem for BigInt {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        &self % &other
    }
}

impl Rem for &BigInt {
    type Output = BigInt;

    fn rem(self, other: &BigInt) -> BigInt {
        let (_, remainder) = self.div_rem(other);
        remainder
    }
}

impl Neg for BigInt {
    type Output = Self;

    fn neg(mut self) -> Self {
        if !self.is_zero() {
            self.positive = !self.positive;
        }
        self
    }
}

// Bit operations

impl Shl<usize> for BigInt {
    type Output = Self;

    fn shl(mut self, shift: usize) -> Self {
        if self.is_zero() || shift == 0 {
            return self;
        }

        let digit_shift = shift / 32;
        let bit_shift = shift % 32;

        // Add zero digits for digit shift
        let mut new_digits = vec![0; digit_shift];
        new_digits.extend(&self.digits);
        self.digits = new_digits;

        // Handle bit shift within digits
        if bit_shift > 0 {
            let mut carry = 0u32;
            for digit in &mut self.digits {
                let new_carry = *digit >> (32 - bit_shift);
                *digit = (*digit << bit_shift) | carry;
                carry = new_carry;
            }
            if carry > 0 {
                self.digits.push(carry);
            }
        }

        self.normalize();
        self
    }
}

impl Shr<usize> for BigInt {
    type Output = Self;

    fn shr(mut self, shift: usize) -> Self {
        if self.is_zero() || shift == 0 {
            return self;
        }

        let digit_shift = shift / 32;
        let bit_shift = shift % 32;

        // Remove digits for digit shift
        if digit_shift >= self.digits.len() {
            return Self::zero();
        }
        self.digits.drain(0..digit_shift);

        // Handle bit shift within digits
        if bit_shift > 0 && !self.digits.is_empty() {
            let mut borrow = 0u32;
            for digit in self.digits.iter_mut().rev() {
                let new_borrow = *digit << (32 - bit_shift);
                *digit = (*digit >> bit_shift) | borrow;
                borrow = new_borrow;
            }
        }

        self.normalize();
        self
    }
}

impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.positive, other.positive) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (true, true) => self.abs_cmp(other),
            (false, false) => other.abs_cmp(self),
        }
    }
}

impl fmt::Display for BigInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }

        let mut result = String::new();
        let mut n = self.abs();
        let ten = Self::from_i64(10);

        while !n.is_zero() {
            let (quotient, remainder) = n.div_rem(&ten);
            let digit = remainder.digits.first().copied().unwrap_or(0);
            result.push((b'0' + digit as u8) as char);
            n = quotient;
        }

        if !self.positive {
            result.push('-');
        }

        // Reverse to get correct order
        result.chars().rev().collect::<String>().fmt(f)
    }
}

impl Hash for BigInt {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.positive.hash(state);
        self.digits.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bigint_creation() {
        let zero = BigInt::zero();
        let one = BigInt::one();
        let from_i64 = BigInt::from_i64(42);
        let from_u64 = BigInt::from_u64(123);

        assert!(zero.is_zero());
        assert!(one.is_positive());
        assert_eq!(from_i64.to_i64(), Some(42));
        assert_eq!(from_u64.to_i64(), Some(123));
    }

    #[test]
    fn test_bigint_arithmetic() {
        let a = BigInt::from_i64(123);
        let b = BigInt::from_i64(456);

        let sum = &a + &b;
        assert_eq!(sum.to_i64(), Some(579));

        let diff = &b - &a;
        assert_eq!(diff.to_i64(), Some(333));

        let prod = &a * &b;
        assert_eq!(prod.to_i64(), Some(56088));

        let quot = &prod / &a;
        assert_eq!(quot.to_i64(), Some(456));
    }

    #[test]
    fn test_bigint_string_parsing() {
        let num = BigInt::from_str_radix("12345", 10).unwrap();
        assert_eq!(num.to_i64(), Some(12345));

        let hex_num = BigInt::from_str_radix("FF", 16).unwrap();
        assert_eq!(hex_num.to_i64(), Some(255));

        let negative = BigInt::from_str_radix("-42", 10).unwrap();
        assert_eq!(negative.to_i64(), Some(-42));
    }

    #[test]
    fn test_bigint_large_numbers() {
        let large1 = BigInt::from_str_radix("12345678901234567890", 10).unwrap();
        let large2 = BigInt::from_str_radix("98765432109876543210", 10).unwrap();

        let sum = &large1 + &large2;
        let expected = BigInt::from_str_radix("111111111011111111100", 10).unwrap();
        assert_eq!(sum, expected);
    }

    #[test]
    fn test_bigint_bit_operations() {
        let num = BigInt::from_i64(5); // 101 in binary
        let shifted_left = num.clone() << 2; // Should be 20 (10100)
        let shifted_right = num >> 1; // Should be 2 (10)

        assert_eq!(shifted_left.to_i64(), Some(20));
        assert_eq!(shifted_right.to_i64(), Some(2));
    }

    #[test]
    fn test_bigint_gcd() {
        let a = BigInt::from_i64(48);
        let b = BigInt::from_i64(18);
        let gcd = a.gcd(&b);
        assert_eq!(gcd.to_i64(), Some(6));
    }

    #[test]
    fn test_bigint_display() {
        assert_eq!(format!("{}", BigInt::zero()), "0");
        assert_eq!(format!("{}", BigInt::from_i64(42)), "42");
        assert_eq!(format!("{}", BigInt::from_i64(-42)), "-42");
    }
}
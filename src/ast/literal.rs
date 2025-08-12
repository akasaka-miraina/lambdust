//! Literal value types for the Lambdust AST.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};

/// Literal values in the Lambdust language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    /// Exact integer numbers (preserves exactness)
    ExactInteger(i64),
    
    /// Inexact floating-point numbers
    InexactReal(f64),
    
    /// Legacy number representation for backward compatibility
    #[deprecated(note = "Use ExactInteger or InexactReal instead")]
    Number(f64),
    
    /// Rational numbers (exact fractions)
    Rational { numerator: i64, denominator: i64 },
    
    /// Complex numbers (can be exact or inexact depending on components)
    Complex { real: f64, imaginary: f64 },
    
    /// String literals
    String(String),
    
    /// Character literals
    Character(char),
    
    /// Boolean values
    Boolean(bool),
    
    /// Bytevector literals
    Bytevector(Vec<u8>),
    
    /// The empty list (nil)
    Nil,
    
    /// Unspecified value (result of side-effecting operations)
    Unspecified,
}

impl Literal {
    /// Creates an exact integer literal.
    pub fn integer(value: i64) -> Self {
        Self::ExactInteger(value)
    }

    /// Creates an inexact real literal.
    pub fn float(value: f64) -> Self {
        Self::InexactReal(value)
    }
    
    /// Creates a number literal - chooses exact or inexact based on value
    pub fn number(value: f64) -> Self {
        if value.fract() == 0.0 && value.is_finite() && value.abs() <= i64::MAX as f64 {
            Self::ExactInteger(value as i64)
        } else {
            Self::InexactReal(value)
        }
    }

    /// Creates a rational literal.
    pub fn rational(numerator: i64, denominator: i64) -> Self {
        if denominator == 0 {
            panic!("Rational number cannot have zero denominator");
        }
        
        // Normalize the rational number
        let gcd = gcd(numerator.unsigned_abs(), denominator.unsigned_abs()) as i64;
        let num = numerator / gcd;
        let den = denominator / gcd;
        
        // Ensure denominator is positive
        if den < 0 {
            Self::Rational { numerator: -num, denominator: -den }
        } else {
            Self::Rational { numerator: num, denominator: den }
        }
    }

    /// Creates a complex literal.
    pub fn complex(real: f64, imaginary: f64) -> Self {
        Self::Complex { real, imaginary }
    }

    /// Creates a string literal.
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Creates a character literal.
    pub fn character(value: char) -> Self {
        Self::Character(value)
    }

    /// Creates a boolean literal.
    pub fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    /// Creates a bytevector literal.
    pub fn bytevector(value: Vec<u8>) -> Self {
        Self::Bytevector(value)
    }

    /// Returns true if this literal is a number.
    pub fn is_number(&self) -> bool {
        matches!(self, 
            Literal::ExactInteger(_) | 
            Literal::InexactReal(_) |
            Literal::Number(_) |
            Literal::Rational { .. } | 
            Literal::Complex { .. }
        )
    }

    /// Returns true if this literal is exact (integer or rational).
    pub fn is_exact(&self) -> bool {
        match self {
            Literal::ExactInteger(_) | Literal::Rational { .. } => true,
            Literal::Number(n) => n.fract() == 0.0 && n.is_finite(),
            Literal::Complex { real, imaginary } => {
                // Complex is exact only if both parts are exact (represent as rationals)
                real.fract() == 0.0 && imaginary.fract() == 0.0
            }
            _ => false,
        }
    }

    /// Returns true if this literal is inexact (floating point).
    pub fn is_inexact(&self) -> bool {
        match self {
            Literal::InexactReal(_) => true,
            Literal::Number(n) => n.fract() != 0.0 || !n.is_finite(),
            Literal::Complex { real, imaginary } => {
                // Complex is inexact if any part is inexact
                real.fract() != 0.0 || imaginary.fract() != 0.0 || !real.is_finite() || !imaginary.is_finite()
            }
            _ => false,
        }
    }

    /// Returns true if this literal is real (not complex).
    pub fn is_real(&self) -> bool {
        matches!(self, 
            Literal::ExactInteger(_) |
            Literal::InexactReal(_) | 
            Literal::Number(_) |
            Literal::Rational { .. }
        ) || matches!(self, Literal::Complex { imaginary, .. } if *imaginary == 0.0)
    }

    /// Returns true if this literal is an integer.
    pub fn is_integer(&self) -> bool {
        match self {
            Literal::ExactInteger(_) => true,
            Literal::InexactReal(n) => n.fract() == 0.0 && n.is_finite(),
            Literal::Number(n) => n.fract() == 0.0 && n.is_finite(),
            Literal::Rational { denominator, .. } => *denominator == 1,
            Literal::Complex { real, imaginary } => {
                *imaginary == 0.0 && real.fract() == 0.0 && real.is_finite()
            }
            Literal::String(_) | Literal::Character(_) | Literal::Boolean(_) 
            | Literal::Bytevector(_) | Literal::Nil | Literal::Unspecified => false,
        }
    }

    /// Converts this literal to a floating-point number if possible.
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Literal::ExactInteger(n) => Some(*n as f64),
            Literal::InexactReal(n) => Some(*n),
            Literal::Number(n) => Some(*n),
            Literal::Rational { numerator, denominator } => {
                Some(*numerator as f64 / *denominator as f64)
            }
            Literal::Complex { real, imaginary } if *imaginary == 0.0 => Some(*real),
            _ => None,
        }
    }

    /// Converts this literal to an integer if possible.
    pub fn to_i64(&self) -> Option<i64> {
        match self {
            Literal::ExactInteger(n) => Some(*n),
            Literal::InexactReal(n) if n.fract() == 0.0 && n.is_finite() => {
                let i = *n as i64;
                if i as f64 == *n { Some(i) } else { None }
            }
            Literal::Number(n) if n.fract() == 0.0 && n.is_finite() => {
                let i = *n as i64;
                if i as f64 == *n { Some(i) } else { None }
            }
            Literal::Rational { numerator, denominator } if *denominator == 1 => Some(*numerator),
            _ => None,
        }
    }

    /// Converts this literal to a non-negative usize if possible.
    pub fn to_usize(&self) -> Option<usize> {
        self.to_i64().and_then(|i| {
            if i >= 0 {
                Some(i as usize)
            } else {
                None
            }
        })
    }

    /// Returns true if this literal is truthy in Scheme semantics.
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Literal::Boolean(false))
    }

    /// Returns true if this literal is falsy in Scheme semantics.
    pub fn is_falsy(&self) -> bool {
        matches!(self, Literal::Boolean(false))
    }

    /// Helper: Extract numeric value as f64 for compatibility during migration
    /// This allows existing code that pattern matches on `Number(n)` to work
    pub fn as_numeric_f64(&self) -> Option<f64> {
        match self {
            Literal::ExactInteger(i) => Some(*i as f64),
            Literal::InexactReal(f) => Some(*f),
            Literal::Number(n) => Some(*n),
            Literal::Rational { numerator, denominator } => {
                Some(*numerator as f64 / *denominator as f64)
            }
            Literal::Complex { real, imaginary } if *imaginary == 0.0 => Some(*real),
            _ => None,
        }
    }
    
    /// Helper: Check if this is any numeric literal (for pattern matching replacement)
    pub fn is_any_numeric(&self) -> bool {
        self.is_number()
    }
    
    /// Convert from legacy f64 representation (for compatibility during migration)
    #[deprecated(note = "Use integer() or float() instead")]
    pub fn from_f64(value: f64) -> Self {
        if value.fract() == 0.0 && value.is_finite() && value.abs() <= i64::MAX as f64 {
            Self::ExactInteger(value as i64)
        } else {
            Self::InexactReal(value)
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::ExactInteger(n) => {
                write!(f, "{n}")
            }
            Literal::InexactReal(n) => {
                if n.is_infinite() {
                    if n.is_sign_positive() {
                        write!(f, "+inf.0")
                    } else {
                        write!(f, "-inf.0")
                    }
                } else if n.is_nan() {
                    write!(f, "+nan.0")
                } else if n.fract() == 0.0 {
                    write!(f, "{}.0", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Literal::Number(n) => {
                if n.is_infinite() {
                    if n.is_sign_positive() {
                        write!(f, "+inf.0")
                    } else {
                        write!(f, "-inf.0")
                    }
                } else if n.is_nan() {
                    write!(f, "+nan.0")
                } else if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Literal::Rational { numerator, denominator } => {
                if *denominator == 1 {
                    write!(f, "{numerator}")
                } else {
                    write!(f, "{numerator}/{denominator}")
                }
            }
            Literal::Complex { real, imaginary } => {
                if *real == 0.0 {
                    if *imaginary == 1.0 {
                        write!(f, "i")
                    } else if *imaginary == -1.0 {
                        write!(f, "-i")
                    } else {
                        write!(f, "{imaginary}i")
                    }
                } else if *imaginary == 0.0 {
                    write!(f, "{real}")
                } else if *imaginary > 0.0 {
                    if *imaginary == 1.0 {
                        write!(f, "{real}+i")
                    } else {
                        write!(f, "{real}+{imaginary}i")
                    }
                } else if *imaginary == -1.0 {
                    write!(f, "{real}-i")
                } else {
                    write!(f, "{real}{imaginary}i")
                }
            }
            Literal::String(s) => write!(f, "\"{}\"", escape_string(s)),
            Literal::Character(c) => {
                match c {
                    ' ' => write!(f, "#\\space"),
                    '\n' => write!(f, "#\\newline"),
                    '\t' => write!(f, "#\\tab"),
                    '\r' => write!(f, "#\\return"),
                    _ => write!(f, "#\\{c}"),
                }
            }
            Literal::Boolean(true) => write!(f, "#t"),
            Literal::Boolean(false) => write!(f, "#f"),
            Literal::Bytevector(bv) => {
                write!(f, "#u8(")?;
                for (i, byte) in bv.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{byte}")?;
                }
                write!(f, ")")
            }
            Literal::Nil => write!(f, "()"),
            Literal::Unspecified => write!(f, "#<unspecified>"),
        }
    }
}

/// Escapes special characters in a string for display.
fn escape_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\t' => "\\t".to_string(),
            '\r' => "\\r".to_string(),
            c if c.is_control() => format!("\\x{:02x}", c as u8),
            c => c.to_string(),
        })
        .collect()
}

/// Computes the greatest common divisor of two numbers.
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Literal::ExactInteger(n) => {
                0u8.hash(state);
                n.hash(state);
            }
            Literal::InexactReal(n) => {
                // For floating point numbers, we use their bit representation
                // This ensures that equal numbers have the same hash
                1u8.hash(state);
                n.to_bits().hash(state);
            }
            Literal::Number(n) => {
                // Legacy number representation - use discriminant 10 to avoid conflicts
                10u8.hash(state);
                n.to_bits().hash(state);
            }
            Literal::Rational { numerator, denominator } => {
                2u8.hash(state);
                numerator.hash(state);
                denominator.hash(state);
            }
            Literal::Complex { real, imaginary } => {
                3u8.hash(state);
                real.to_bits().hash(state);
                imaginary.to_bits().hash(state);
            }
            Literal::String(s) => {
                4u8.hash(state);
                s.hash(state);
            }
            Literal::Character(c) => {
                5u8.hash(state);
                c.hash(state);
            }
            Literal::Boolean(b) => {
                6u8.hash(state);
                b.hash(state);
            }
            Literal::Bytevector(bv) => {
                7u8.hash(state);
                bv.hash(state);
            }
            Literal::Nil => {
                8u8.hash(state);
            }
            Literal::Unspecified => {
                9u8.hash(state);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_creation() {
        let int_lit = Literal::integer(42);
        let float_lit = Literal::float(3.05);
        
        assert!(int_lit.is_number());
        assert!(int_lit.is_integer());
        assert_eq!(int_lit.to_i64(), Some(42));
        
        assert!(float_lit.is_number());
        assert!(!float_lit.is_integer());
        assert_eq!(float_lit.to_f64(), Some(3.05));
    }

    #[test]
    fn test_rational_creation() {
        let rat = Literal::rational(3, 4);
        
        match rat {
            Literal::Rational { numerator, denominator } => {
                assert_eq!(numerator, 3);
                assert_eq!(denominator, 4);
            }
            _ => panic!("Expected rational"),
        }
        
        assert!(rat.is_exact());
        assert!(!rat.is_inexact());
        assert_eq!(rat.to_f64(), Some(0.75));
    }

    #[test]
    fn test_rational_normalization() {
        let rat = Literal::rational(6, 8);
        
        match rat {
            Literal::Rational { numerator, denominator } => {
                assert_eq!(numerator, 3);
                assert_eq!(denominator, 4);
            }
            _ => panic!("Expected rational"),
        }
    }

    #[test]
    fn test_complex_creation() {
        let complex = Literal::complex(3.0, 4.0);
        
        assert!(complex.is_number());
        assert!(!complex.is_real());
        assert!(!complex.is_integer());
    }

    #[test]
    fn test_truthiness() {
        assert!(Literal::boolean(true).is_truthy());
        assert!(!Literal::boolean(false).is_truthy());
        assert!(Literal::integer(0).is_truthy()); // 0 is truthy in Scheme
        assert!(Literal::string("").is_truthy()); // empty string is truthy
        assert!(Literal::Nil.is_truthy()); // empty list is truthy
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Literal::integer(42)), "42");
        assert_eq!(format!("{}", Literal::float(3.05)), "3.05");
        assert_eq!(format!("{}", Literal::float(3.0)), "3.0");
        assert_eq!(format!("{}", Literal::rational(3, 4)), "3/4");
        assert_eq!(format!("{}", Literal::complex(3.0, 4.0)), "3+4i");
        assert_eq!(format!("{}", Literal::complex(0.0, 1.0)), "i");
        assert_eq!(format!("{}", Literal::complex(3.0, -1.0)), "3-i");
        assert_eq!(format!("{}", Literal::string("hello".to_string())), "\"hello\"");
        assert_eq!(format!("{}", Literal::character('a')), "#\\a");
        assert_eq!(format!("{}", Literal::character(' ')), "#\\space");
        assert_eq!(format!("{}", Literal::boolean(true)), "#t");
        assert_eq!(format!("{}", Literal::boolean(false)), "#f");
        assert_eq!(format!("{}", Literal::Nil), "()");
        
        // Special float values
        assert_eq!(format!("{}", Literal::InexactReal(f64::INFINITY)), "+inf.0");
        assert_eq!(format!("{}", Literal::InexactReal(f64::NEG_INFINITY)), "-inf.0");
        assert_eq!(format!("{}", Literal::InexactReal(f64::NAN)), "+nan.0");
    }

    #[test]
    fn test_string_escaping() {
        let s = "hello\n\"world\"";
        let escaped = escape_string(s);
        assert_eq!(escaped, "hello\\n\\\"world\\\"");
    }
}
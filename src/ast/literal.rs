//! Literal value types for the Lambdust AST.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use crate::utils::InternedString;

/// Greatest common divisor calculation for rational number normalization.
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Literal values in the Lambdust language.
/// 
/// This enum is optimized for memory usage by boxing large variants
/// to reduce the overall size from 32 bytes to approximately 16 bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    /// Exact integer numbers (preserves exactness)
    ExactInteger(i64),
    
    /// Inexact floating-point numbers
    InexactReal(f64),
    
    /// Legacy number representation for backward compatibility
    #[deprecated(note = "Use ExactInteger or InexactReal instead")]
    Number(f64),
    
    /// Rational numbers (exact fractions) - boxed for size optimization
    Rational(Box<RationalLiteral>),
    
    /// Complex numbers (can be exact or inexact depending on components) - boxed for size optimization
    Complex(Box<ComplexLiteral>),
    
    /// String literals (traditional) - boxed for size optimization
    String(Box<String>),
    
    /// Interned string literals (memory-optimized)
    InternedString(InternedString),
    
    /// Character literals
    Character(char),
    
    /// Boolean values
    Boolean(bool),
    
    /// Bytevector literals - boxed for size optimization
    Bytevector(Box<Vec<u8>>),
    
    /// The empty list (nil)
    Nil,
    
    /// Unspecified value (result of side-effecting operations)
    Unspecified,
}

/// Rational number representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RationalLiteral {
    /// Numerator of the rational number
    pub numerator: i64,
    /// Denominator of the rational number (always positive and non-zero)
    pub denominator: i64,
}

impl RationalLiteral {
    /// Creates a new rational literal with normalization.
    pub fn new(numerator: i64, denominator: i64) -> Self {
        if denominator == 0 {
            panic!("Rational number cannot have zero denominator");
        }
        
        // Normalize the rational number
        let gcd = gcd(numerator.unsigned_abs(), denominator.unsigned_abs()) as i64;
        let num = numerator / gcd;
        let den = denominator / gcd;
        
        // Ensure denominator is positive
        if den < 0 {
            Self { numerator: -num, denominator: -den }
        } else {
            Self { numerator: num, denominator: den }
        }
    }
    
    /// Converts to floating-point approximation.
    pub fn to_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
    
    /// Returns true if this represents an integer (denominator is 1).
    pub fn is_integer(&self) -> bool {
        self.denominator == 1
    }
}

/// Complex number representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComplexLiteral {
    /// Real part of the complex number
    pub real: f64,
    /// Imaginary part of the complex number
    pub imaginary: f64,
}

impl ComplexLiteral {
    /// Creates a new complex literal.
    pub fn new(real: f64, imaginary: f64) -> Self {
        Self { real, imaginary }
    }
    
    /// Returns true if this is a real number (imaginary part is zero).
    pub fn is_real(&self) -> bool {
        self.imaginary == 0.0
    }
    
    /// Returns true if this is an integer (real part is integer and imaginary is zero).
    pub fn is_integer(&self) -> bool {
        self.imaginary == 0.0 && self.real.fract() == 0.0 && self.real.is_finite()
    }
    
    /// Returns true if this is exact (both parts are exact).
    pub fn is_exact(&self) -> bool {
        self.real.fract() == 0.0 && self.imaginary.fract() == 0.0
    }
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
        Self::Rational(Box::new(RationalLiteral::new(numerator, denominator)))
    }

    /// Creates a complex literal.
    pub fn complex(real: f64, imaginary: f64) -> Self {
        Self::Complex(Box::new(ComplexLiteral::new(real, imaginary)))
    }

    /// Creates a string literal (traditional).
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(Box::new(value.into()))
    }
    
    /// Creates an interned string literal (memory-optimized).
    pub fn interned_string(s: &str) -> Self {
        use crate::utils::intern;
        Self::InternedString(intern(s))
    }
    
    /// Creates a string literal, choosing interned vs regular based on heuristics.
    /// Short, common strings are interned for memory efficiency.
    pub fn smart_string(s: impl Into<String>) -> Self {
        let string = s.into();
        
        // Heuristics for when to intern:
        // 1. Short strings (likely to be repeated)
        // 2. Common patterns (empty, single char, etc.)
        if string.len() <= 32 || crate::ast::literal_helpers::is_common_string(&string) {
            Self::InternedString(crate::utils::intern(&string))
        } else {
            Self::String(Box::new(string))
        }
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
        Self::Bytevector(Box::new(value))
    }

    /// Returns true if this literal is a number.
    pub fn is_number(&self) -> bool {
        matches!(self, 
            Literal::ExactInteger(_) | 
            Literal::InexactReal(_) |
            Literal::Number(_) |
            Literal::Rational(_) | 
            Literal::Complex(_)
        )
    }

    /// Returns true if this literal is exact (integer or rational).
    pub fn is_exact(&self) -> bool {
        match self {
            Literal::ExactInteger(_) | Literal::Rational(_) => true,
            Literal::Number(n) => n.fract() == 0.0 && n.is_finite(),
            Literal::Complex(c) => c.is_exact(),
            _ => false,
        }
    }

    /// Returns true if this literal is inexact (floating point).
    pub fn is_inexact(&self) -> bool {
        match self {
            Literal::InexactReal(_) => true,
            Literal::Number(n) => n.fract() != 0.0 || !n.is_finite(),
            Literal::Complex(c) => !c.is_exact(),
            _ => false,
        }
    }

    /// Returns true if this literal is real (not complex).
    pub fn is_real(&self) -> bool {
        matches!(self, 
            Literal::ExactInteger(_) |
            Literal::InexactReal(_) | 
            Literal::Number(_) |
            Literal::Rational(_)
        ) || matches!(self, Literal::Complex(c) if c.is_real())
    }

    /// Returns true if this literal is an integer.
    pub fn is_integer(&self) -> bool {
        match self {
            Literal::ExactInteger(_) => true,
            Literal::InexactReal(n) => n.fract() == 0.0 && n.is_finite(),
            Literal::Number(n) => n.fract() == 0.0 && n.is_finite(),
            Literal::Rational(r) => r.is_integer(),
            Literal::Complex(c) => c.is_integer(),
            Literal::String(_) | Literal::InternedString(_) | Literal::Character(_) | Literal::Boolean(_) 
            | Literal::Bytevector(_) | Literal::Nil | Literal::Unspecified => false,
        }
    }

    /// Converts this literal to a floating-point number if possible.
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Literal::ExactInteger(n) => Some(*n as f64),
            Literal::InexactReal(n) => Some(*n),
            Literal::Number(n) => Some(*n),
            Literal::Rational(r) => Some(r.to_f64()),
            Literal::Complex(c) if c.is_real() => Some(c.real),
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
            Literal::Rational(r) if r.is_integer() => Some(r.numerator),
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
    
    /// Returns true if this literal is a string (regular or interned).
    pub fn is_string(&self) -> bool {
        matches!(self, Literal::String(_) | Literal::InternedString(_))
    }
    
    /// Gets the string value if this is a string literal (regular or interned).
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Literal::String(s) => Some(s),
            Literal::InternedString(s) => Some(s.as_str()),
            _ => None,
        }
    }
    
    /// Gets the string value as an owned String if this is a string literal.
    pub fn to_string_value(&self) -> Option<String> {
        match self {
            Literal::String(s) => Some((**s).clone()),
            Literal::InternedString(s) => Some(s.to_string()),
            _ => None,
        }
    }

    /// Helper: Extract numeric value as f64 for compatibility during migration
    /// This allows existing code that pattern matches on `Number(n)` to work
    pub fn as_numeric_f64(&self) -> Option<f64> {
        match self {
            Literal::ExactInteger(i) => Some(*i as f64),
            Literal::InexactReal(f) => Some(*f),
            Literal::Number(n) => Some(*n),
            Literal::Rational(r) => Some(r.to_f64()),
            Literal::Complex(c) if c.is_real() => Some(c.real),
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
            Literal::Rational(r) => {
                if r.denominator == 1 {
                    write!(f, "{}", r.numerator)
                } else {
                    write!(f, "{}/{}", r.numerator, r.denominator)
                }
            }
            Literal::Complex(c) => {
                if c.real == 0.0 {
                    if c.imaginary == 1.0 {
                        write!(f, "i")
                    } else if c.imaginary == -1.0 {
                        write!(f, "-i")
                    } else {
                        write!(f, "{}i", c.imaginary)
                    }
                } else if c.imaginary == 0.0 {
                    write!(f, "{}", c.real)
                } else if c.imaginary > 0.0 {
                    if c.imaginary == 1.0 {
                        write!(f, "{}+i", c.real)
                    } else {
                        write!(f, "{}+{}i", c.real, c.imaginary)
                    }
                } else if c.imaginary == -1.0 {
                    write!(f, "{}-i", c.real)
                } else {
                    write!(f, "{}{}i", c.real, c.imaginary)
                }
            }
            Literal::String(s) => write!(f, "\"{}\"", escape_string(s)),
            Literal::InternedString(s) => write!(f, "\"{}\"", escape_string(s.as_str())),
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
            Literal::Rational(r) => {
                2u8.hash(state);
                r.numerator.hash(state);
                r.denominator.hash(state);
            }
            Literal::Complex(c) => {
                3u8.hash(state);
                c.real.to_bits().hash(state);
                c.imaginary.to_bits().hash(state);
            }
            Literal::String(s) => {
                4u8.hash(state);
                s.hash(state);
            }
            Literal::InternedString(s) => {
                11u8.hash(state);
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
            Literal::Rational(r) => {
                assert_eq!(r.numerator, 3);
                assert_eq!(r.denominator, 4);
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
            Literal::Rational(r) => {
                assert_eq!(r.numerator, 3);
                assert_eq!(r.denominator, 4);
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
        assert!(Literal::String("").is_truthy()); // empty string is truthy
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
        assert_eq!(format!("{}", Literal::String("hello".to_string())), "\"hello\"");
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
    
    #[test]
    fn test_interned_string_creation() {
        let regular = Literal::String("hello");
        let interned = Literal::interned_string("hello");
        let smart = Literal::smart_string("hello"); // Should be interned due to length
        
        // They should be equal
        assert_eq!(regular, interned);
        assert_eq!(regular, smart);
        
        // String extraction
        assert_eq!(regular.as_str(), Some("hello"));
        assert_eq!(interned.as_str(), Some("hello"));
        assert_eq!(smart.as_str(), Some("hello"));
        
        // Both should be strings
        assert!(regular.is_string());
        assert!(interned.is_string());
        assert!(smart.is_string());
    }
    
    #[test]
    fn test_smart_string_heuristics() {
        // Short strings should be interned
        let short = Literal::smart_string("hi");
        assert!(matches!(short, Literal::InternedString(_)));
        
        // Common strings should be interned
        let common = Literal::smart_string("error");
        assert!(matches!(common, Literal::InternedString(_)));
        
        // Long uncommon strings should stay regular
        let long = Literal::smart_string("this is a very long and uncommon string that should not be interned");
        assert!(matches!(long, Literal::String(_)));
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Literal::ExactInteger(a), Literal::ExactInteger(b)) => a == b,
            (Literal::InexactReal(a), Literal::InexactReal(b)) => a == b,
            (Literal::Number(a), Literal::Number(b)) => a == b,
            (Literal::Rational(r1), Literal::Rational(r2)) => r1 == r2,
            (Literal::Complex(c1), Literal::Complex(c2)) => c1 == c2,
            
            // String comparison handles both regular and interned strings
            (Literal::String(s1), Literal::String(s2)) => s1 == s2,
            (Literal::InternedString(s1), Literal::InternedString(s2)) => s1 == s2,
            (Literal::String(s1), Literal::InternedString(s2)) => s1.as_str() == s2.as_str(),
            (Literal::InternedString(s1), Literal::String(s2)) => s1.as_str() == s2.as_str(),
            
            (Literal::Character(a), Literal::Character(b)) => a == b,
            (Literal::Boolean(a), Literal::Boolean(b)) => a == b,
            (Literal::Bytevector(a), Literal::Bytevector(b)) => a == b,
            (Literal::Nil, Literal::Nil) => true,
            (Literal::Unspecified, Literal::Unspecified) => true,
            _ => false,
        }
    }
}

impl Eq for Literal {}
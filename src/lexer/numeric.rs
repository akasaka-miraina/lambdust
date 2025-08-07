//! Numeric literal parsing and validation for Lambdust.
//!
//! This module provides comprehensive parsing and validation for all numeric
//! formats supported by R7RS Scheme, including:
//! - Integers (decimal, binary, octal, hexadecimal)
//! - Real numbers (floats with optional exponents)
//! - Rational numbers (exact fractions)
//! - Complex numbers (rectangular and polar forms)
//!
//! The parsing follows R7RS numeric syntax precisely while providing
//! detailed error reporting for invalid formats.

use crate::diagnostics::{Error, Result, Span};

/// Validates an integer literal according to R7RS syntax.
pub fn validate_integer(text: &str, span: Span) -> Result<()> {
    if text.is_empty() {
        return Err(Box::new(Error::lex_error("Empty integer literal", span).into()))
    }

    let text = text.trim();
    
    // Handle sign
    let (_sign_len, unsigned) = match text.chars().next() {
        Some('+') | Some('-') => (1, &text[1..]),
        _ => (0, text),
    };

    if unsigned.is_empty() {
        return Err(Box::new(Error::lex_error("Integer literal cannot be just a sign", span).into()))
    }

    // Check for different number bases
    if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
        // Hexadecimal
        let hex_digits = &unsigned[2..];
        if hex_digits.is_empty() {
            return Err(Box::new(Error::lex_error("Hexadecimal literal must have digits after 0x", span).into()))
        }
        for ch in hex_digits.chars() {
            if !ch.is_ascii_hexdigit() {
                return Err(Box::new(Error::lex_error(
                    format!("Invalid hexadecimal digit: '{ch}'"), 
                    span
                ).into()))
            }
        }
    } else if unsigned.starts_with("0b") || unsigned.starts_with("0B") {
        // Binary
        let bin_digits = &unsigned[2..];
        if bin_digits.is_empty() {
            return Err(Box::new(Error::lex_error("Binary literal must have digits after 0b", span).into()))
        }
        for ch in bin_digits.chars() {
            if !matches!(ch, '0' | '1') {
                return Err(Box::new(Error::lex_error(
                    format!("Invalid binary digit: '{ch}'"), 
                    span
                ).into()))
            }
        }
    } else if unsigned.starts_with("0o") || unsigned.starts_with("0O") {
        // Octal
        let oct_digits = &unsigned[2..];
        if oct_digits.is_empty() {
            return Err(Box::new(Error::lex_error("Octal literal must have digits after 0o", span).into()))
        }
        for ch in oct_digits.chars() {
            if !('0'..='7').contains(&ch) {
                return Err(Box::new(Error::lex_error(
                    format!("Invalid octal digit: '{ch}'"), 
                    span
                ).into()))
            }
        }
    } else {
        // Decimal
        for ch in unsigned.chars() {
            if !ch.is_ascii_digit() {
                return Err(Box::new(Error::lex_error(
                    format!("Invalid decimal digit: '{ch}'"), 
                    span
                ).into()))
            }
        }
    }

    Ok(())
}

/// Validates a real (floating-point) number literal according to R7RS syntax.
pub fn validate_real(text: &str, span: Span) -> Result<()> {
    if text.is_empty() {
        return Err(Box::new(Error::lex_error("Empty real number literal", span).into()))
    }

    let text = text.trim();
    
    // Handle sign
    let unsigned = match text.chars().next() {
        Some('+') | Some('-') => &text[1..],
        _ => text,
    };

    if unsigned.is_empty() {
        return Err(Box::new(Error::lex_error("Real number literal cannot be just a sign", span).into()))
    }

    // Check for scientific notation
    let (mantissa, exponent) = if let Some(e_pos) = unsigned.find(['e', 'E']) {
        let mantissa = &unsigned[..e_pos];
        let exponent = &unsigned[e_pos + 1..];
        
        if exponent.is_empty() {
            return Err(Box::new(Error::lex_error("Exponent cannot be empty", span).into()))
        }
        
        // Validate exponent (must be integer)
        let exp_unsigned = match exponent.chars().next() {
            Some('+') | Some('-') => &exponent[1..],
            _ => exponent,
        };
        
        if exp_unsigned.is_empty() {
            return Err(Box::new(Error::lex_error("Exponent cannot be just a sign", span).into()))
        }
        
        for ch in exp_unsigned.chars() {
            if !ch.is_ascii_digit() {
                return Err(Box::new(Error::lex_error(
                    format!("Invalid digit in exponent: '{ch}'"), 
                    span
                ).into()))
            }
        }
        
        (mantissa, Some(exponent))
    } else {
        (unsigned, None)
    };

    // Validate mantissa
    if mantissa.is_empty() {
        return Err(Box::new(Error::lex_error("Mantissa cannot be empty", span).into()))
    }

    let dot_count = mantissa.matches('.').count();
    if dot_count > 1 {
        return Err(Box::new(Error::lex_error("Real number cannot have multiple decimal points", span).into()))
    }

    if dot_count == 0 && exponent.is_none() {
        return Err(Box::new(Error::lex_error("Real number must have decimal point or exponent", span).into()))
    }

    // Check that all characters are digits or decimal point
    for ch in mantissa.chars() {
        if !ch.is_ascii_digit() && ch != '.' {
            return Err(Box::new(Error::lex_error(
                format!("Invalid character in real number: '{ch}'"), 
                span
            ).into()))
        }
    }

    // Ensure there's at least one digit
    if !mantissa.chars().any(|c| c.is_ascii_digit()) {
        return Err(Box::new(Error::lex_error("Real number must contain at least one digit", span).into()))
    }

    Ok(())
}

/// Validates a rational number literal according to R7RS syntax.
pub fn validate_rational(text: &str, span: Span) -> Result<()> {
    if text.is_empty() {
        return Err(Box::new(Error::lex_error("Empty rational number literal", span).into()))
    }

    let text = text.trim();
    
    // Handle sign
    let unsigned = match text.chars().next() {
        Some('+') | Some('-') => &text[1..],
        _ => text,
    };

    if unsigned.is_empty() {
        return Err(Box::new(Error::lex_error("Rational number literal cannot be just a sign", span).into()))
    }

    // Split on '/'
    let parts: Vec<&str> = unsigned.split('/').collect();
    if parts.len() != 2 {
        return Err(Box::new(Error::lex_error("Rational number must have exactly one '/' character", span).into()))
    }

    let numerator = parts[0];
    let denominator = parts[1];

    // Validate numerator
    if numerator.is_empty() {
        return Err(Box::new(Error::lex_error("Rational number numerator cannot be empty", span).into()))
    }
    
    for ch in numerator.chars() {
        if !ch.is_ascii_digit() {
            return Err(Box::new(Error::lex_error(
                format!("Invalid digit in numerator: '{ch}'"), 
                span
            ).into()))
        }
    }

    // Validate denominator
    if denominator.is_empty() {
        return Err(Box::new(Error::lex_error("Rational number denominator cannot be empty", span).into()))
    }
    
    if denominator == "0" {
        return Err(Box::new(Error::lex_error("Rational number denominator cannot be zero", span).into()))
    }
    
    for ch in denominator.chars() {
        if !ch.is_ascii_digit() {
            return Err(Box::new(Error::lex_error(
                format!("Invalid digit in denominator: '{ch}'"), 
                span
            ).into()))
        }
    }

    Ok(())
}

/// Validates a complex number literal according to R7RS syntax.
pub fn validate_complex(text: &str, span: Span) -> Result<()> {
    if text.is_empty() {
        return Err(Box::new(Error::lex_error("Empty complex number literal", span).into()))
    }

    let text = text.trim();

    // Handle special cases: +i, -i, i
    if text == "i" || text == "+i" || text == "-i" {
        return Ok(());
    }

    if !text.ends_with('i') {
        return Err(Box::new(Error::lex_error("Complex number must end with 'i'", span).into()))
    }

    let without_i = &text[..text.len() - 1];
    
    // Find the position of + or - that separates real and imaginary parts
    // We need to be careful not to match the sign at the beginning
    let mut split_pos = None;
    let mut depth = 0;
    
    for (i, ch) in without_i.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            '+' | '-' if depth == 0 && i > 0 => {
                // Check if this is not part of an exponent
                let prev_chars: Vec<char> = without_i[..i].chars().rev().take(2).collect();
                if !prev_chars.is_empty() && matches!(prev_chars[0], 'e' | 'E') {
                    continue; // This is an exponent sign
                }
                split_pos = Some(i);
                break;
            }
            _ => {}
        }
    }

    if let Some(pos) = split_pos {
        // Has both real and imaginary parts
        let real_part = &without_i[..pos];
        let imag_part = &without_i[pos..];
        
        // Validate real part
        if !real_part.is_empty() {
            if real_part.contains('.') || real_part.contains(['e', 'E']) {
                validate_real(real_part, span)?;
            } else if real_part.contains('/') {
                validate_rational(real_part, span)?;
            } else {
                validate_integer(real_part, span)?;
            }
        }
        
        // Validate imaginary part (without the sign)
        let imag_unsigned = if imag_part.starts_with(['+', '-']) {
            &imag_part[1..]
        } else {
            imag_part
        };
        
        if !imag_unsigned.is_empty() {
            if imag_unsigned.contains('.') || imag_unsigned.contains(['e', 'E']) {
                validate_real(&format!("{}{}",
                    if imag_part.starts_with(['+', '-']) { &imag_part[..1] } else { "" },
                    imag_unsigned
                ), span)?;
            } else if imag_unsigned.contains('/') {
                validate_rational(&format!("{}{}",
                    if imag_part.starts_with(['+', '-']) { &imag_part[..1] } else { "" },
                    imag_unsigned
                ), span)?;
            } else {
                validate_integer(&format!("{}{}",
                    if imag_part.starts_with(['+', '-']) { &imag_part[..1] } else { "" },
                    imag_unsigned
                ), span)?;
            }
        }
    } else {
        // Only imaginary part
        if without_i.is_empty() {
            return Err(Box::new(Error::lex_error("Complex number 'i' must have a coefficient", span).into()))
        }
        
        // Validate the coefficient
        if without_i.contains('.') || without_i.contains(['e', 'E']) {
            validate_real(without_i, span)?;
        } else if without_i.contains('/') {
            validate_rational(without_i, span)?;
        } else {
            validate_integer(without_i, span)?;
        }
    }

    Ok(())
}

/// Parses an integer literal into its numeric value.
pub fn parse_integer(text: &str) -> Option<i64> {
    let text = text.trim();
    
    // Handle different bases
    if text.starts_with("0x") || text.starts_with("0X") {
        i64::from_str_radix(&text[2..], 16).ok()
    } else if text.starts_with("0b") || text.starts_with("0B") {
        i64::from_str_radix(&text[2..], 2).ok()
    } else if text.starts_with("0o") || text.starts_with("0O") {
        i64::from_str_radix(&text[2..], 8).ok()
    } else {
        text.parse().ok()
    }
}

/// Parses a real number literal into its numeric value.
pub fn parse_real(text: &str) -> Option<f64> {
    text.trim().parse().ok()
}

/// Represents a rational number as numerator/denominator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rational {
    pub numerator: i64,
    pub denominator: u64,
}

impl Rational {
    /// Creates a new rational number, automatically reducing to lowest terms.
    pub fn new(numerator: i64, denominator: u64) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        
        let gcd = gcd(numerator.unsigned_abs(), denominator);
        Some(Self {
            numerator: numerator / gcd as i64,
            denominator: denominator / gcd,
        })
    }
    
    /// Converts to floating point approximation.
    pub fn to_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
}

/// Parses a rational number literal.
pub fn parse_rational(text: &str) -> Option<Rational> {
    let text = text.trim();
    
    let (sign, unsigned) = match text.chars().next() {
        Some('-') => (-1, &text[1..]),
        Some('+') => (1, &text[1..]),
        _ => (1, text),
    };
    
    let parts: Vec<&str> = unsigned.split('/').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let numerator: u64 = parts[0].parse().ok()?;
    let denominator: u64 = parts[1].parse().ok()?;
    
    Rational::new(sign * numerator as i64, denominator)
}

/// Represents a complex number.
#[derive(Debug, Clone, PartialEq)]
pub struct Complex {
    pub real: f64,
    pub imag: f64,
}

impl Complex {
    /// Creates a new complex number.
    pub fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }
    
    /// Creates a purely imaginary number.
    pub fn imaginary(imag: f64) -> Self {
        Self::new(0.0, imag)
    }
    
    /// Creates a purely real number.
    pub fn real(real: f64) -> Self {
        Self::new(real, 0.0)
    }
}

/// Parses a complex number literal.
pub fn parse_complex(text: &str) -> Option<Complex> {
    let text = text.trim();
    
    // Handle special cases
    match text {
        "i" => return Some(Complex::imaginary(1.0)),
        "+i" => return Some(Complex::imaginary(1.0)),
        "-i" => return Some(Complex::imaginary(-1.0)),
        _ => {}
    }
    
    if !text.ends_with('i') {
        return None;
    }
    
    let without_i = &text[..text.len() - 1];
    
    // Find split position for real and imaginary parts
    let mut split_pos = None;
    for (i, ch) in without_i.char_indices() {
        if matches!(ch, '+' | '-') && i > 0 {
            // Check if this is not part of an exponent
            let prev_char = without_i.chars().nth(i - 1)?;
            if !matches!(prev_char, 'e' | 'E') {
                split_pos = Some(i);
                break;
            }
        }
    }
    
    if let Some(pos) = split_pos {
        // Both real and imaginary parts
        let real_part = &without_i[..pos];
        let imag_part = &without_i[pos..];
        
        let real = if real_part.is_empty() {
            0.0
        } else {
            real_part.parse().ok()?
        };
        
        let imag = if imag_part == "+" {
            1.0
        } else if imag_part == "-" {
            -1.0
        } else {
            imag_part.parse().ok()?
        };
        
        Some(Complex::new(real, imag))
    } else {
        // Only imaginary part
        let imag = if without_i.is_empty() {
            1.0
        } else {
            without_i.parse().ok()?
        };
        
        Some(Complex::imaginary(imag))
    }
}

/// Computes the greatest common divisor using Euclid's algorithm.
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_validation() {
        let span = Span::new(0, 3);
        
        assert!(validate_integer("123", span).is_ok());
        assert!(validate_integer("+123", span).is_ok());
        assert!(validate_integer("-123", span).is_ok());
        assert!(validate_integer("0x1F", span).is_ok());
        assert!(validate_integer("0b1010", span).is_ok());
        assert!(validate_integer("0o777", span).is_ok());
        
        assert!(validate_integer("", span).is_err());
        assert!(validate_integer("+", span).is_err());
        assert!(validate_integer("12.3", span).is_err());
        assert!(validate_integer("0xGG", span).is_err());
    }

    #[test]
    fn test_real_validation() {
        let span = Span::new(0, 5);
        
        assert!(validate_real("123.45", span).is_ok());
        assert!(validate_real(".123", span).is_ok());
        assert!(validate_real("123.", span).is_ok());
        assert!(validate_real("1e10", span).is_ok());
        assert!(validate_real("1.23e-4", span).is_ok());
        assert!(validate_real("+1.23", span).is_ok());
        assert!(validate_real("-1.23", span).is_ok());
        
        assert!(validate_real("", span).is_err());
        assert!(validate_real("123", span).is_err()); // No decimal point or exponent
        assert!(validate_real("1.2.3", span).is_err());
        assert!(validate_real("1e", span).is_err());
    }

    #[test]
    fn test_rational_validation() {
        let span = Span::new(0, 5);
        
        assert!(validate_rational("1/2", span).is_ok());
        assert!(validate_rational("22/7", span).is_ok());
        assert!(validate_rational("+3/4", span).is_ok());
        assert!(validate_rational("-5/6", span).is_ok());
        
        assert!(validate_rational("", span).is_err());
        assert!(validate_rational("1/0", span).is_err());
        assert!(validate_rational("1/", span).is_err());
        assert!(validate_rational("/2", span).is_err());
        assert!(validate_rational("1/2/3", span).is_err());
    }

    #[test]
    fn test_complex_validation() {
        let span = Span::new(0, 5);
        
        assert!(validate_complex("3+4i", span).is_ok());
        assert!(validate_complex("3-4i", span).is_ok());
        assert!(validate_complex("3i", span).is_ok());
        assert!(validate_complex("+3i", span).is_ok());
        assert!(validate_complex("-3i", span).is_ok());
        assert!(validate_complex("i", span).is_ok());
        assert!(validate_complex("+i", span).is_ok());
        assert!(validate_complex("-i", span).is_ok());
        assert!(validate_complex("1.5+2.5i", span).is_ok());
        
        assert!(validate_complex("", span).is_err());
        assert!(validate_complex("3+4", span).is_err()); // No 'i'
    }

    #[test]
    fn test_integer_parsing() {
        assert_eq!(parse_integer("123"), Some(123));
        assert_eq!(parse_integer("-123"), Some(-123));
        assert_eq!(parse_integer("0x1F"), Some(31));
        assert_eq!(parse_integer("0b1010"), Some(10));
        assert_eq!(parse_integer("0o10"), Some(8));
        
        assert_eq!(parse_integer("invalid"), None);
    }

    #[test]
    fn test_rational_parsing() {
        assert_eq!(parse_rational("1/2"), Some(Rational::new(1, 2).unwrap()));
        assert_eq!(parse_rational("6/9"), Some(Rational::new(2, 3).unwrap())); // Reduced
        assert_eq!(parse_rational("-3/4"), Some(Rational::new(-3, 4).unwrap()));
        
        assert_eq!(parse_rational("invalid"), None);
        assert_eq!(parse_rational("1/0"), None);
    }

    #[test]
    fn test_complex_parsing() {
        assert_eq!(parse_complex("3+4i"), Some(Complex::new(3.0, 4.0)));
        assert_eq!(parse_complex("3-4i"), Some(Complex::new(3.0, -4.0)));
        assert_eq!(parse_complex("3i"), Some(Complex::new(0.0, 3.0)));
        assert_eq!(parse_complex("i"), Some(Complex::new(0.0, 1.0)));
        assert_eq!(parse_complex("-i"), Some(Complex::new(0.0, -1.0)));
        
        assert_eq!(parse_complex("invalid"), None);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(17, 13), 1);
        assert_eq!(gcd(100, 25), 25);
    }
}
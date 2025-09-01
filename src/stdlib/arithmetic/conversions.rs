//! Number conversion operations for Lambdust
//! 
//! This module implements R7RS-compliant number conversion operations including
//! string conversion, rationalization, and complex number operations.

use crate::diagnostics::Result;
use crate::eval::value::Value;

/// Implements number->string primitive
pub fn primitive_number_to_string(args: &[Value]) -> Result<Value> {
    let (number, radix) = match args.len() {
        1 => (&args[0], 10),
        2 => {
            let radix = match &args[1] {
                Value::Literal(crate::ast::Literal::ExactInteger(r)) => *r as u32,
                _ => return Err(crate::diagnostics::Error::type_error(
                    "Radix must be an integer",
                    crate::diagnostics::Span::default(),
                ).into())
            };
            if radix < 2 || radix > 36 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Radix must be between 2 and 36",
                    None,
                ).into());
            }
            (&args[0], radix)
        }
        _ => return Err(crate::diagnostics::Error::runtime_error(
            "number->string expects one or two arguments",
            None,
        ).into())
    };
    
    let string_result = match number {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            if radix == 10 {
                x.to_string()
            } else {
                format_integer_with_radix(*x, radix)
            }
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            if radix == 10 {
                x.to_string()
            } else {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Cannot convert real numbers to non-decimal radix",
                    None,
                ).into());
            }
        }
        _ => return Err(crate::diagnostics::Error::type_error(
            "number->string requires a number",
            crate::diagnostics::Span::default(),
        ).into())
    };
    
    Ok(Value::string(string_result))
}

/// Implements string->number primitive
pub fn primitive_string_to_number(args: &[Value]) -> Result<Value> {
    let (string_val, radix) = match args.len() {
        1 => (&args[0], 10),
        2 => {
            let radix = match &args[1] {
                Value::Literal(crate::ast::Literal::ExactInteger(r)) => *r as u32,
                _ => return Err(crate::diagnostics::Error::type_error(
                    "Radix must be an integer",
                    crate::diagnostics::Span::default(),
                ).into())
            };
            if radix < 2 || radix > 36 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Radix must be between 2 and 36",
                    None,
                ).into());
            }
            (&args[0], radix)
        }
        _ => return Err(crate::diagnostics::Error::runtime_error(
            "string->number expects one or two arguments",
            None,
        ).into())
    };
    
    let string_content = match string_val {
        Value::Literal(crate::ast::Literal::String(s)) => s.as_str(),
        _ => return Err(crate::diagnostics::Error::type_error(
            "string->number requires a string",
            crate::diagnostics::Span::default(),
        ).into())
    };
    
    // Try to parse as integer first
    if let Ok(int_val) = parse_integer_with_radix(string_content, radix) {
        return Ok(Value::Literal(crate::ast::Literal::ExactInteger(int_val)));
    }
    
    // Try to parse as float if radix is 10
    if radix == 10 {
        if let Ok(float_val) = string_content.parse::<f64>() {
            return Ok(Value::Literal(crate::ast::Literal::InexactReal(float_val)));
        }
    }
    
    // Return #f if parsing failed
    Ok(Value::boolean(false))
}

/// Implements rationalize primitive
pub fn primitive_rationalize(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "rationalize expects exactly two arguments",
            None,
        ).into());
    }
    
    let x = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => *i as f64,
        Value::Literal(crate::ast::Literal::InexactReal(r)) => *r,
        _ => return Err(crate::diagnostics::Error::type_error(
            "rationalize first argument must be a number",
            crate::diagnostics::Span::default(),
        ).into())
    };
    
    let tolerance = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => *i as f64,
        Value::Literal(crate::ast::Literal::InexactReal(r)) => *r,
        _ => return Err(crate::diagnostics::Error::type_error(
            "rationalize second argument must be a number",
            crate::diagnostics::Span::default(),
        ).into())
    };
    
    // Simple rationalization: if x is close to an integer within tolerance, return that integer
    let rounded = x.round();
    if (x - rounded).abs() <= tolerance.abs() {
        Ok(Value::Literal(crate::ast::Literal::ExactInteger(rounded as i64)))
    } else {
        Ok(Value::Literal(crate::ast::Literal::InexactReal(x)))
    }
}

/// Implements make-rectangular (complex number constructor)
pub fn primitive_make_rectangular(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "make-rectangular expects exactly two arguments",
            None,
        ).into());
    }
    
    let _real = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => *i as f64,
        Value::Literal(crate::ast::Literal::InexactReal(r)) => *r,
        _ => return Err(crate::diagnostics::Error::type_error(
            "make-rectangular real part must be a number",
            crate::diagnostics::Span::default(),
        ).into())
    };
    
    let _imag = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => *i as f64,
        Value::Literal(crate::ast::Literal::InexactReal(r)) => *r,
        _ => return Err(crate::diagnostics::Error::type_error(
            "make-rectangular imaginary part must be a number",
            crate::diagnostics::Span::default(),
        ).into())
    };
    
    // For now, return the real part if imaginary part is zero
    if _imag.abs() < f64::EPSILON {
        if _real.fract() == 0.0 {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(_real as i64)))
        } else {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(_real)))
        }
    } else {
        // TODO: Implement proper complex number support
        Err(crate::diagnostics::Error::runtime_error(
            "Complex numbers not yet fully supported",
            None,
        ).into())
    }
}

/// Implements make-polar (complex number constructor from polar coordinates)
pub fn primitive_make_polar(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "make-polar expects exactly two arguments",
            None,
        ).into());
    }
    
    let magnitude = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => *i as f64,
        Value::Literal(crate::ast::Literal::InexactReal(r)) => *r,
        _ => return Err(crate::diagnostics::Error::type_error(
            "make-polar magnitude must be a number",
            crate::diagnostics::Span::default(),
        ).into())
    };
    
    let angle = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => *i as f64,
        Value::Literal(crate::ast::Literal::InexactReal(r)) => *r,
        _ => return Err(crate::diagnostics::Error::type_error(
            "make-polar angle must be a number",
            crate::diagnostics::Span::default(),
        ).into())
    };
    
    // Convert polar to rectangular
    let real = magnitude * angle.cos();
    let imag = magnitude * angle.sin();
    
    // For now, return the real part if imaginary part is zero
    if imag.abs() < f64::EPSILON {
        if real.fract() == 0.0 {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(real as i64)))
        } else {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(real)))
        }
    } else {
        // TODO: Implement proper complex number support
        Err(crate::diagnostics::Error::runtime_error(
            "Complex numbers not yet fully supported",
            None,
        ).into())
    }
}

/// Implements real-part (extract real part of complex number)
pub fn primitive_real_part(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "real-part expects exactly one argument",
            None,
        ).into());
    }
    
    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => Ok(Value::Literal(crate::ast::Literal::ExactInteger(*i))),
        Value::Literal(crate::ast::Literal::InexactReal(r)) => Ok(Value::Literal(crate::ast::Literal::InexactReal(*r))),
        _ => Err(crate::diagnostics::Error::type_error(
            "real-part requires a number",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

/// Implements imag-part (extract imaginary part of complex number)
pub fn primitive_imag_part(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "imag-part expects exactly one argument",
            None,
        ).into());
    }
    
    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(_)) |
        Value::Literal(crate::ast::Literal::InexactReal(_)) => {
            // Real numbers have zero imaginary part
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(0)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "imag-part requires a number",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

/// Implements magnitude (absolute value/modulus of complex number)
pub fn primitive_magnitude(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "magnitude expects exactly one argument",
            None,
        ).into());
    }
    
    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => Ok(Value::Literal(crate::ast::Literal::ExactInteger(i.abs()))),
        Value::Literal(crate::ast::Literal::InexactReal(r)) => Ok(Value::Literal(crate::ast::Literal::InexactReal(r.abs()))),
        _ => Err(crate::diagnostics::Error::type_error(
            "magnitude requires a number",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

/// Implements angle (argument/phase of complex number)
pub fn primitive_angle(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "angle expects exactly one argument",
            None,
        ).into());
    }
    
    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(i)) => {
            if *i >= 0 {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(0.0)))
            } else {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(std::f64::consts::PI)))
            }
        }
        Value::Literal(crate::ast::Literal::InexactReal(r)) => {
            if *r >= 0.0 {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(0.0)))
            } else {
                Ok(Value::Literal(crate::ast::Literal::InexactReal(std::f64::consts::PI)))
            }
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "angle requires a number",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

// Helper functions for radix conversion
fn format_integer_with_radix(mut value: i64, radix: u32) -> String {
    if value == 0 {
        return "0".to_string();
    }
    
    let negative = value < 0;
    if negative {
        value = -value;
    }
    
    let mut digits = Vec::new();
    while value > 0 {
        let digit = (value % radix as i64) as u8;
        let char_digit = if digit < 10 {
            (b'0' + digit) as char
        } else {
            (b'a' + digit - 10) as char
        };
        digits.push(char_digit);
        value /= radix as i64;
    }
    
    if negative {
        digits.push('-');
    }
    
    digits.reverse();
    digits.into_iter().collect()
}

fn parse_integer_with_radix(s: &str, radix: u32) -> std::result::Result<i64, std::num::ParseIntError> {
    i64::from_str_radix(s, radix)
}
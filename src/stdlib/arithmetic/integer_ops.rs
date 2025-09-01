//! Integer operations for Lambdust
//! 
//! This module implements R7RS-compliant integer-specific operations including
//! quotient, remainder, modulo, gcd, lcm, and division variants.

use crate::diagnostics::Result;
use crate::eval::value::Value;

/// Implements quotient primitive
pub fn primitive_quotient(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "quotient expects exactly two arguments",
            None,
        ).into());
    }
    
    match (&args[0], &args[1]) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            if *y == 0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(x / y)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "quotient requires two integers",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

/// Implements remainder primitive
pub fn primitive_remainder(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "remainder expects exactly two arguments",
            None,
        ).into());
    }
    
    match (&args[0], &args[1]) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            if *y == 0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(x % y)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "remainder requires two integers",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

/// Implements modulo primitive
pub fn primitive_modulo(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "modulo expects exactly two arguments",
            None,
        ).into());
    }
    
    match (&args[0], &args[1]) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            if *y == 0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            // Implement Scheme's modulo semantics (result has same sign as divisor)
            let result = x % y;
            if (result > 0 && *y < 0) || (result < 0 && *y > 0) {
                Ok(Value::Literal(crate::ast::Literal::ExactInteger(result + y)))
            } else {
                Ok(Value::Literal(crate::ast::Literal::ExactInteger(result)))
            }
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "modulo requires two integers",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

/// Implements abs primitive
pub fn primitive_abs(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(crate::diagnostics::Error::runtime_error(
            "abs expects exactly one argument",
            None,
        ).into());
    }
    
    match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(x.abs())))
        }
        Value::Literal(crate::ast::Literal::InexactReal(x)) => {
            Ok(Value::Literal(crate::ast::Literal::InexactReal(x.abs())))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "abs requires a number",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

/// Implements gcd primitive
pub fn primitive_gcd(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Literal(crate::ast::Literal::ExactInteger(0)));
    }
    
    let mut result = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => x.abs(),
        _ => return Err(crate::diagnostics::Error::type_error(
            "gcd requires integers",
            crate::diagnostics::Span::default(),
        ).into())
    };
    
    for arg in &args[1..] {
        match arg {
            Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
                result = gcd_two(result, x.abs());
            }
            _ => return Err(crate::diagnostics::Error::type_error(
                "gcd requires integers",
                crate::diagnostics::Span::default(),
            ).into())
        }
    }
    
    Ok(Value::Literal(crate::ast::Literal::ExactInteger(result)))
}

/// Implements lcm primitive
pub fn primitive_lcm(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::Literal(crate::ast::Literal::ExactInteger(1)));
    }
    
    let mut result = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(x)) => x.abs(),
        _ => return Err(crate::diagnostics::Error::type_error(
            "lcm requires integers",
            crate::diagnostics::Span::default(),
        ).into())
    };
    
    for arg in &args[1..] {
        match arg {
            Value::Literal(crate::ast::Literal::ExactInteger(x)) => {
                let x_abs = x.abs();
                if result == 0 || x_abs == 0 {
                    result = 0;
                } else {
                    result = (result * x_abs) / gcd_two(result, x_abs);
                }
            }
            _ => return Err(crate::diagnostics::Error::type_error(
                "lcm requires integers",
                crate::diagnostics::Span::default(),
            ).into())
        }
    }
    
    Ok(Value::Literal(crate::ast::Literal::ExactInteger(result)))
}

/// Implements floor-quotient primitive
pub fn primitive_floor_quotient(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "floor-quotient expects exactly two arguments",
            None,
        ).into());
    }
    
    match (&args[0], &args[1]) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            if *y == 0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            let result = (*x as f64 / *y as f64).floor() as i64;
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(result)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "floor-quotient requires two integers",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

/// Implements floor-remainder primitive
pub fn primitive_floor_remainder(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "floor-remainder expects exactly two arguments",
            None,
        ).into());
    }
    
    match (&args[0], &args[1]) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            if *y == 0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            let q = (*x as f64 / *y as f64).floor() as i64;
            let result = x - (q * y);
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(result)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "floor-remainder requires two integers",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

/// Implements truncate-quotient primitive
pub fn primitive_truncate_quotient(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "truncate-quotient expects exactly two arguments",
            None,
        ).into());
    }
    
    match (&args[0], &args[1]) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            if *y == 0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            let result = (*x as f64 / *y as f64).trunc() as i64;
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(result)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "truncate-quotient requires two integers",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

/// Implements truncate-remainder primitive
pub fn primitive_truncate_remainder(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(crate::diagnostics::Error::runtime_error(
            "truncate-remainder expects exactly two arguments",
            None,
        ).into());
    }
    
    match (&args[0], &args[1]) {
        (Value::Literal(crate::ast::Literal::ExactInteger(x)), 
         Value::Literal(crate::ast::Literal::ExactInteger(y))) => {
            if *y == 0 {
                return Err(crate::diagnostics::Error::runtime_error(
                    "Division by zero",
                    None,
                ).into());
            }
            let q = (*x as f64 / *y as f64).trunc() as i64;
            let result = x - (q * y);
            Ok(Value::Literal(crate::ast::Literal::ExactInteger(result)))
        }
        _ => Err(crate::diagnostics::Error::type_error(
            "truncate-remainder requires two integers",
            crate::diagnostics::Span::default(),
        ).into())
    }
}

// Helper function for GCD calculation using Euclidean algorithm
fn gcd_two(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}
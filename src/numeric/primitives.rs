//! Numeric primitives integration with the R7RS-large numeric library
//!
//! Provides primitive functions that integrate the advanced numeric system
//! with the Lambdust runtime environment.

use crate::diagnostics::{Result, Error, Span};
use crate::eval::Value;
use crate::ast::Literal;
use super::{NumericValue, tower, functions, constants};

/// Converts a Value to NumericValue if possible
pub fn value_to_numeric(value: &Value) -> Result<NumericValue> {
    match value {
        Value::Literal(literal) => {
            NumericValue::from_literal(literal)
                .ok_or_else(|| Error::type_error("Expected numeric value", Span::default()))
        }
        _ => Err(Box::new(Error::type_error("Expected numeric value", Span::default().boxed())),
    }
}

/// Converts NumericValue back to Value
pub fn numeric_to_value(numeric: &NumericValue) -> Value {
    Value::Literal(numeric.to_literal())
}

/// Helper to extract numeric values from argument list
pub fn extract_numeric_args(args: &[Value]) -> Result<Vec<NumericValue>> {
    args.iter()
        .map(value_to_numeric)
        .collect()
}

/// Helper to extract exactly N numeric arguments
pub fn extract_n_numeric_args<const N: usize>(args: &[Value]) -> Result<[NumericValue; N]> {
    if args.len() != N {
        return Err(Box::new(Error::arity_error("extract_n_numeric_args", N, args.len().boxed()));
    }
    
    let numeric_args = extract_numeric_args(args)?;
    numeric_args.try_into()
        .map_err(|_| Error::internal_error("Failed to convert to fixed-size array"))
}

// ============= BASIC ARITHMETIC PRIMITIVES =============

/// Addition with automatic type promotion
pub fn primitive_add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(numeric_to_value(&NumericValue::integer(0)));
    }
    
    let numeric_args = extract_numeric_args(args)?;
    let result = numeric_args.iter()
        .skip(1)
        .fold(numeric_args[0].clone()), |acc, arg| tower::add(&acc, arg));
    
    Ok(numeric_to_value(&result))
}

/// Subtraction with automatic type promotion
pub fn primitive_subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::arity_error("subtract", 1, 0.into().boxed()));
    }
    
    let numeric_args = extract_numeric_args(args)?;
    
    let result = if args.len() == 1 {
        // Unary minus
        tower::negate(&numeric_args[0])
    } else {
        // Binary subtraction
        numeric_args.iter()
            .skip(1)
            .fold(numeric_args[0].clone()), |acc, arg| tower::subtract(&acc, arg))
    };
    
    Ok(numeric_to_value(&result))
}

/// Multiplication with automatic type promotion
pub fn primitive_multiply(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(numeric_to_value(&NumericValue::integer(1)));
    }
    
    let numeric_args = extract_numeric_args(args)?;
    let result = numeric_args.iter()
        .skip(1)
        .fold(numeric_args[0].clone()), |acc, arg| tower::multiply(&acc, arg));
    
    Ok(numeric_to_value(&result))
}

/// Division with automatic type promotion
pub fn primitive_divide(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::arity_error("divide", 1, 0.into().boxed()));
    }
    
    let numeric_args = extract_numeric_args(args)?;
    
    let result = if args.len() == 1 {
        // Reciprocal
        tower::divide(&NumericValue::integer(1), &numeric_args[0])
            .map_err(|msg| Error::runtime_error(&msg, Some(Span::default())))?
    } else {
        // Binary division
        let mut result = numeric_args[0].clone());
        for arg in &numeric_args[1..] {
            result = tower::divide(&result, arg)
                .map_err(|msg| Error::runtime_error(&msg, Some(Span::default())))?;
        }
        result
    };
    
    Ok(numeric_to_value(&result))
}

// ============= COMPARISON PRIMITIVES =============

/// Numeric equality
pub fn primitive_numeric_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::arity_error("numeric-function", 2, args.len().boxed()));
    }
    
    let numeric_args = extract_numeric_args(args)?;
    let first = &numeric_args[0];
    
    let all_equal = numeric_args.iter().skip(1).all(|arg| {
        matches!(tower::compare(first, arg), Some(std::cmp::Ordering::Equal))
    });
    
    Ok(Value::boolean(all_equal))
}

/// Numeric less than
pub fn primitive_less_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::arity_error("numeric-function", 2, args.len().boxed()));
    }
    
    let numeric_args = extract_numeric_args(args)?;
    
    let monotonic = numeric_args.windows(2).all(|pair| {
        matches!(tower::compare(&pair[0], &pair[1]), Some(std::cmp::Ordering::Less))
    });
    
    Ok(Value::boolean(monotonic))
}

/// Numeric greater than
pub fn primitive_greater_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::arity_error("numeric-function", 2, args.len().boxed()));
    }
    
    let numeric_args = extract_numeric_args(args)?;
    
    let monotonic = numeric_args.windows(2).all(|pair| {
        matches!(tower::compare(&pair[0], &pair[1]), Some(std::cmp::Ordering::Greater))
    });
    
    Ok(Value::boolean(monotonic))
}

/// Numeric less than or equal
pub fn primitive_less_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::arity_error("numeric-function", 2, args.len().boxed()));
    }
    
    let numeric_args = extract_numeric_args(args)?;
    
    let monotonic = numeric_args.windows(2).all(|pair| {
        matches!(tower::compare(&pair[0], &pair[1]), 
               Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal))
    });
    
    Ok(Value::boolean(monotonic))
}

/// Numeric greater than or equal
pub fn primitive_greater_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::arity_error("numeric-function", 2, args.len().boxed()));
    }
    
    let numeric_args = extract_numeric_args(args)?;
    
    let monotonic = numeric_args.windows(2).all(|pair| {
        matches!(tower::compare(&pair[0], &pair[1]), 
               Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal))
    });
    
    Ok(Value::boolean(monotonic))
}

// ============= ADVANCED MATHEMATICAL FUNCTIONS =============

/// Exponential function
pub fn primitive_exp(args: &[Value]) -> Result<Value> {
    let [arg] = extract_n_numeric_args(args)?;
    
    let result = match arg {
        NumericValue::Real(r) => NumericValue::real(r.exp()),
        NumericValue::Complex(c) => NumericValue::complex(c.exp().real, c.exp().imaginary),
        _ => {
            // Promote to real/complex and compute
            let promoted = tower::promote_to_real(&arg);
            if let NumericValue::Real(r) = promoted {
                NumericValue::real(r.exp())
            } else {
                return Err(Box::new(Error::type_error("Cannot compute exp of this type", Span::default().boxed()));
            }
        }
    };
    
    Ok(numeric_to_value(&result))
}

/// Natural logarithm
pub fn primitive_log(args: &[Value]) -> Result<Value> {
    let [arg] = extract_n_numeric_args(args)?;
    
    let result = match arg {
        NumericValue::Real(r) if r > 0.0 => NumericValue::real(r.ln()),
        NumericValue::Complex(c) => {
            let log_c = c.ln();
            NumericValue::complex(log_c.real, log_c.imaginary)
        }
        _ => {
            // For negative reals or other types, promote to complex
            let promoted = tower::promote_to_complex(&arg);
            if let NumericValue::Complex(c) = promoted {
                let log_c = c.ln();
                NumericValue::complex(log_c.real, log_c.imaginary)
            } else {
                return Err(Box::new(Error::type_error("Cannot compute log of this type", Span::default().boxed()));
            }
        }
    };
    
    Ok(numeric_to_value(&result))
}

/// Square root
pub fn primitive_sqrt(args: &[Value]) -> Result<Value> {
    let [arg] = extract_n_numeric_args(args)?;
    let result = tower::sqrt(&arg);
    Ok(numeric_to_value(&result))
}

/// Power function
pub fn primitive_expt(args: &[Value]) -> Result<Value> {
    let [base, exponent] = extract_n_numeric_args(args)?;
    let result = tower::power(&base, &exponent);
    Ok(numeric_to_value(&result))
}

/// Sine function
pub fn primitive_sin(args: &[Value]) -> Result<Value> {
    let [arg] = extract_n_numeric_args(args)?;
    
    let result = match arg {
        NumericValue::Real(r) => NumericValue::real(r.sin()),
        NumericValue::Complex(c) => {
            let sin_c = c.sin();
            NumericValue::complex(sin_c.real, sin_c.imaginary)
        }
        _ => {
            let promoted = tower::promote_to_real(&arg);
            if let NumericValue::Real(r) = promoted {
                NumericValue::real(r.sin())
            } else {
                return Err(Box::new(Error::type_error("Cannot compute sin of this type", Span::default().boxed()));
            }
        }
    };
    
    Ok(numeric_to_value(&result))
}

/// Cosine function
pub fn primitive_cos(args: &[Value]) -> Result<Value> {
    let [arg] = extract_n_numeric_args(args)?;
    
    let result = match arg {
        NumericValue::Real(r) => NumericValue::real(r.cos()),
        NumericValue::Complex(c) => {
            let cos_c = c.cos();
            NumericValue::complex(cos_c.real, cos_c.imaginary)
        }
        _ => {
            let promoted = tower::promote_to_real(&arg);
            if let NumericValue::Real(r) = promoted {
                NumericValue::real(r.cos())
            } else {
                return Err(Box::new(Error::type_error("Cannot compute cos of this type", Span::default().boxed()));
            }
        }
    };
    
    Ok(numeric_to_value(&result))
}

/// Tangent function
pub fn primitive_tan(args: &[Value]) -> Result<Value> {
    let [arg] = extract_n_numeric_args(args)?;
    
    let result = match arg {
        NumericValue::Real(r) => NumericValue::real(r.tan()),
        NumericValue::Complex(c) => {
            let tan_c = c.tan();
            NumericValue::complex(tan_c.real, tan_c.imaginary)
        }
        _ => {
            let promoted = tower::promote_to_real(&arg);
            if let NumericValue::Real(r) = promoted {
                NumericValue::real(r.tan())
            } else {
                return Err(Box::new(Error::type_error("Cannot compute tan of this type", Span::default().boxed()));
            }
        }
    };
    
    Ok(numeric_to_value(&result))
}

// ============= SPECIAL FUNCTIONS =============

/// Gamma function
pub fn primitive_gamma(args: &[Value]) -> Result<Value> {
    let [arg] = extract_n_numeric_args(args)?;
    
    let result = match arg {
        NumericValue::Real(r) => NumericValue::real(functions::gamma(r)),
        _ => {
            let promoted = tower::promote_to_real(&arg);
            if let NumericValue::Real(r) = promoted {
                NumericValue::real(functions::gamma(r))
            } else {
                return Err(Box::new(Error::type_error("Gamma function requires real argument", Span::default().boxed()));
            }
        }
    };
    
    Ok(numeric_to_value(&result))
}

/// Error function
pub fn primitive_erf(args: &[Value]) -> Result<Value> {
    let [arg] = extract_n_numeric_args(args)?;
    
    let result = match arg {
        NumericValue::Real(r) => NumericValue::real(functions::erf(r)),
        _ => {
            let promoted = tower::promote_to_real(&arg);
            if let NumericValue::Real(r) = promoted {
                NumericValue::real(functions::erf(r))
            } else {
                return Err(Box::new(Error::type_error("Error function requires real argument", Span::default().boxed()));
            }
        }
    };
    
    Ok(numeric_to_value(&result))
}

/// Bessel function J0
pub fn primitive_bessel_j0(args: &[Value]) -> Result<Value> {
    let [arg] = extract_n_numeric_args(args)?;
    
    let result = match arg {
        NumericValue::Real(r) => NumericValue::real(functions::bessel_j0(r)),
        _ => {
            let promoted = tower::promote_to_real(&arg);
            if let NumericValue::Real(r) = promoted {
                NumericValue::real(functions::bessel_j0(r))
            } else {
                return Err(Box::new(Error::type_error("Bessel function requires real argument", Span::default().boxed()));
            }
        }
    };
    
    Ok(numeric_to_value(&result))
}

// ============= TYPE PREDICATES =============

/// Check if value is a number
pub fn primitive_number_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("numeric-function", 1, args.len().boxed()));
    }
    
    let is_number = matches!(&args[0], Value::Literal(lit) if lit.is_number());
    Ok(Value::boolean(is_number))
}

/// Check if value is exact
pub fn primitive_exact_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("numeric-function", 1, args.len().boxed()));
    }
    
    let is_exact = match value_to_numeric(&args[0]) {
        Ok(num) => num.is_exact(),
        Err(_) => false,
    };
    
    Ok(Value::boolean(is_exact))
}

/// Check if value is inexact
pub fn primitive_inexact_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("numeric-function", 1, args.len().boxed()));
    }
    
    let is_inexact = match value_to_numeric(&args[0]) {
        Ok(num) => num.is_inexact(),
        Err(_) => false,
    };
    
    Ok(Value::boolean(is_inexact))
}

/// Check if value is integer
pub fn primitive_integer_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("numeric-function", 1, args.len().boxed()));
    }
    
    let is_integer = match value_to_numeric(&args[0]) {
        Ok(num) => num.is_integer(),
        Err(_) => false,
    };
    
    Ok(Value::boolean(is_integer))
}

/// Check if value is real
pub fn primitive_real_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("numeric-function", 1, args.len().boxed()));
    }
    
    let is_real = match value_to_numeric(&args[0]) {
        Ok(num) => num.is_real(),
        Err(_) => false,
    };
    
    Ok(Value::boolean(is_real))
}

/// Check if value is complex
pub fn primitive_complex_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("numeric-function", 1, args.len().boxed()));
    }
    
    let is_complex = value_to_numeric(&args[0]).is_ok();
    Ok(Value::boolean(is_complex))
}

// ============= EXACTNESS CONVERSION =============

/// Convert to exact representation
pub fn primitive_exact(args: &[Value]) -> Result<Value> {
    let [arg] = extract_n_numeric_args(args)?;
    let result = tower::make_exact(&arg);
    Ok(numeric_to_value(&result))
}

/// Convert to inexact representation
pub fn primitive_inexact(args: &[Value]) -> Result<Value> {
    let [arg] = extract_n_numeric_args(args)?;
    let result = tower::make_inexact(&arg);
    Ok(numeric_to_value(&result))
}

// ============= CONSTANTS =============

/// Get mathematical or physical constant
pub fn primitive_constant(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("numeric-function", 1, args.len().boxed()));
    }
    
    let name = match &args[0] {
        Value::Literal(Literal::String(s)) => s.clone()),
        Value::Symbol(id) => {
            // Convert symbol to string for lookup
            if let Some(name) = crate::utils::symbol_name(*id) {
                name
            } else {
                return Err(Box::new(Error::type_error("Unknown symbol", Span::default().boxed()));
            }
        }
        _ => return Err(Box::new(Error::type_error("Expected string or symbol", Span::default().boxed())),
    };
    
    if let Some(constant) = constants::get_constant(&name) {
        Ok(numeric_to_value(&constant))
    } else {
        Err(Box::new(Error::runtime_error(&format!("Unknown constant: {}", name), Some(Span::default())))
    }
}

/// List available constants
pub fn primitive_list_constants(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(Box::new(Error::arity_error("numeric-function", 0, args.len().boxed()));
    }
    
    let constant_names = constants::list_constants();
    let values: Vec<Value> = constant_names.into_iter()
        .map(|name| Value::string(name))
        .collect();
    
    Ok(Value::list(values))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_primitives() {
        let args = vec![
            Value::Literal(Literal::Number(3.0)),
            Value::Literal(Literal::Number(4.0)),
        ];
        
        let result = primitive_add(&args).unwrap();
        assert_eq!(result.as_number().unwrap(), 7.0);
        
        let result = primitive_multiply(&args).unwrap();
        assert_eq!(result.as_number().unwrap(), 12.0);
    }

    #[test]
    fn test_comparison_primitives() {
        let args = vec![
            Value::Literal(Literal::Number(3.0)),
            Value::Literal(Literal::Number(4.0)),
            Value::Literal(Literal::Number(5.0)),
        ];
        
        let result = primitive_less_than(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_greater_than(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_type_predicates() {
        let number_val = Value::Literal(Literal::Number(3.14));
        let string_val = Value::Literal(Literal::String("hello".to_string()));
        
        let result = primitive_number_p(&[number_val.clone())]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_number_p(&[string_val]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        let result = primitive_inexact_p(&[number_val]).unwrap();
        assert_eq!(result, Value::boolean(true));
    }

    #[test]
    fn test_mathematical_functions() {
        let arg = Value::Literal(Literal::Number(1.0));
        
        let result = primitive_sin(&[arg.clone())]).unwrap();
        let sin_1 = result.as_number().unwrap();
        assert!((sin_1 - 1.0_f64.sin()).abs() < 1e-10);
        
        let result = primitive_exp(&[arg]).unwrap();
        let exp_1 = result.as_number().unwrap();
        assert!((exp_1 - std::f64::consts::E).abs() < 1e-10);
    }

    #[test]
    fn test_constant_lookup() {
        let pi_arg = Value::string("pi");
        let result = primitive_constant(&[pi_arg]).unwrap();
        let pi_val = result.as_number().unwrap();
        assert!((pi_val - std::f64::consts::PI).abs() < 1e-10);
        
        let list_result = primitive_list_constants(&[]).unwrap();
        assert!(list_result.is_list());
    }
}
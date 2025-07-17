//! SRFI 144: Flonums
//!
//! This SRFI defines a set of operations for fixed-precision floating-point numbers (flonums).
//! Flonums are IEEE 754 double-precision floating-point numbers that support efficient
//! mathematical operations without boxing.

use crate::error::{LambdustError, Result};
use crate::value::{Value, Procedure};
use crate::lexer::SchemeNumber;
use std::collections::HashMap;

/// SRFI 144 module implementation
pub struct Srfi144Module;

impl crate::srfi::SrfiModule for Srfi144Module {
    fn srfi_id(&self) -> u32 {
        144
    }

    fn name(&self) -> &'static str {
        "SRFI 144"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["flonums"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Core flonum operations
        exports.insert("flonum?".to_string(), Value::Procedure(Procedure::Builtin { name: "flonum?".to_string(), arity: Some(1), func: flonum_p }));
        exports.insert("fl=?".to_string(), Value::Procedure(Procedure::Builtin { name: "fl=?".to_string(), arity: None, func: fl_equal }));
        exports.insert("fl<?".to_string(), Value::Procedure(Procedure::Builtin { name: "fl<?".to_string(), arity: None, func: fl_less }));
        exports.insert("fl>?".to_string(), Value::Procedure(Procedure::Builtin { name: "fl>?".to_string(), arity: None, func: fl_greater }));
        exports.insert("fl<=?".to_string(), Value::Procedure(Procedure::Builtin { name: "fl<=?".to_string(), arity: None, func: fl_less_equal }));
        exports.insert("fl>=?".to_string(), Value::Procedure(Procedure::Builtin { name: "fl>=?".to_string(), arity: None, func: fl_greater_equal }));
        
        // Arithmetic predicates
        exports.insert("flinteger?".to_string(), Value::Procedure(Procedure::Builtin { name: "flinteger?".to_string(), arity: Some(1), func: fl_integer_p }));
        exports.insert("flzero?".to_string(), Value::Procedure(Procedure::Builtin { name: "flzero?".to_string(), arity: Some(1), func: fl_zero_p }));
        exports.insert("flpositive?".to_string(), Value::Procedure(Procedure::Builtin { name: "flpositive?".to_string(), arity: Some(1), func: fl_positive_p }));
        exports.insert("flnegative?".to_string(), Value::Procedure(Procedure::Builtin { name: "flnegative?".to_string(), arity: Some(1), func: fl_negative_p }));
        exports.insert("flodd?".to_string(), Value::Procedure(Procedure::Builtin { name: "flodd?".to_string(), arity: Some(1), func: fl_odd_p }));
        exports.insert("fleven?".to_string(), Value::Procedure(Procedure::Builtin { name: "fleven?".to_string(), arity: Some(1), func: fl_even_p }));
        exports.insert("flfinite?".to_string(), Value::Procedure(Procedure::Builtin { name: "flfinite?".to_string(), arity: Some(1), func: fl_finite_p }));
        exports.insert("flinfinite?".to_string(), Value::Procedure(Procedure::Builtin { name: "flinfinite?".to_string(), arity: Some(1), func: fl_infinite_p }));
        exports.insert("flnan?".to_string(), Value::Procedure(Procedure::Builtin { name: "flnan?".to_string(), arity: Some(1), func: fl_nan_p }));
        exports.insert("flnormalized?".to_string(), Value::Procedure(Procedure::Builtin { name: "flnormalized?".to_string(), arity: Some(1), func: fl_normalized_p }));
        exports.insert("fldenormalized?".to_string(), Value::Procedure(Procedure::Builtin { name: "fldenormalized?".to_string(), arity: Some(1), func: fl_denormalized_p }));
        
        // Min/max operations
        exports.insert("flmax".to_string(), Value::Procedure(Procedure::Builtin { name: "flmax".to_string(), arity: None, func: fl_max }));
        exports.insert("flmin".to_string(), Value::Procedure(Procedure::Builtin { name: "flmin".to_string(), arity: None, func: fl_min }));
        
        // Basic arithmetic
        exports.insert("fl+".to_string(), Value::Procedure(Procedure::Builtin { name: "fl+".to_string(), arity: None, func: fl_add }));
        exports.insert("fl-".to_string(), Value::Procedure(Procedure::Builtin { name: "fl-".to_string(), arity: None, func: fl_subtract }));
        exports.insert("fl*".to_string(), Value::Procedure(Procedure::Builtin { name: "fl*".to_string(), arity: None, func: fl_multiply }));
        exports.insert("fl/".to_string(), Value::Procedure(Procedure::Builtin { name: "fl/".to_string(), arity: Some(2), func: fl_divide }));
        exports.insert("flabs".to_string(), Value::Procedure(Procedure::Builtin { name: "flabs".to_string(), arity: Some(1), func: fl_abs }));
        exports.insert("flsign".to_string(), Value::Procedure(Procedure::Builtin { name: "flsign".to_string(), arity: Some(1), func: fl_sign }));
        exports.insert("flsquare".to_string(), Value::Procedure(Procedure::Builtin { name: "flsquare".to_string(), arity: Some(1), func: fl_square }));
        exports.insert("flsqrt".to_string(), Value::Procedure(Procedure::Builtin { name: "flsqrt".to_string(), arity: Some(1), func: fl_sqrt }));
        
        // Exponential and logarithmic functions
        exports.insert("flexp".to_string(), Value::Procedure(Procedure::Builtin { name: "flexp".to_string(), arity: Some(1), func: fl_exp }));
        exports.insert("flexp2".to_string(), Value::Procedure(Procedure::Builtin { name: "flexp2".to_string(), arity: Some(1), func: fl_exp2 }));
        exports.insert("flexpsubone".to_string(), Value::Procedure(Procedure::Builtin { name: "flexpm1".to_string(), arity: Some(1), func: fl_expm1 }));
        exports.insert("fllog".to_string(), Value::Procedure(Procedure::Builtin { name: "fllog".to_string(), arity: Some(1), func: fl_log }));
        exports.insert("fllog2".to_string(), Value::Procedure(Procedure::Builtin { name: "fllog2".to_string(), arity: Some(1), func: fl_log2 }));
        exports.insert("fllog10".to_string(), Value::Procedure(Procedure::Builtin { name: "fllog10".to_string(), arity: Some(1), func: fl_log10 }));
        exports.insert("fllogoneplusx".to_string(), Value::Procedure(Procedure::Builtin { name: "fllog1p".to_string(), arity: Some(1), func: fl_log1p }));
        
        // Trigonometric functions
        exports.insert("flsin".to_string(), Value::Procedure(Procedure::Builtin { name: "flsin".to_string(), arity: Some(1), func: fl_sin }));
        exports.insert("flcos".to_string(), Value::Procedure(Procedure::Builtin { name: "flcos".to_string(), arity: Some(1), func: fl_cos }));
        exports.insert("fltan".to_string(), Value::Procedure(Procedure::Builtin { name: "fltan".to_string(), arity: Some(1), func: fl_tan }));
        exports.insert("flasin".to_string(), Value::Procedure(Procedure::Builtin { name: "flasin".to_string(), arity: Some(1), func: fl_asin }));
        exports.insert("flacos".to_string(), Value::Procedure(Procedure::Builtin { name: "flacos".to_string(), arity: Some(1), func: fl_acos }));
        exports.insert("flatan".to_string(), Value::Procedure(Procedure::Builtin { name: "flatan".to_string(), arity: Some(1), func: fl_atan }));
        exports.insert("flatan2".to_string(), Value::Procedure(Procedure::Builtin { name: "flatan2".to_string(), arity: Some(2), func: fl_atan2 }));
        
        // Hyperbolic functions
        exports.insert("flsinh".to_string(), Value::Procedure(Procedure::Builtin { name: "flsinh".to_string(), arity: Some(1), func: fl_sinh }));
        exports.insert("flcosh".to_string(), Value::Procedure(Procedure::Builtin { name: "flcosh".to_string(), arity: Some(1), func: fl_cosh }));
        exports.insert("fltanh".to_string(), Value::Procedure(Procedure::Builtin { name: "fltanh".to_string(), arity: Some(1), func: fl_tanh }));
        exports.insert("flasinh".to_string(), Value::Procedure(Procedure::Builtin { name: "flasinh".to_string(), arity: Some(1), func: fl_asinh }));
        exports.insert("flacosh".to_string(), Value::Procedure(Procedure::Builtin { name: "flacosh".to_string(), arity: Some(1), func: fl_acosh }));
        exports.insert("flatanh".to_string(), Value::Procedure(Procedure::Builtin { name: "flatanh".to_string(), arity: Some(1), func: fl_atanh }));
        
        // Rounding functions
        exports.insert("flfloor".to_string(), Value::Procedure(Procedure::Builtin { name: "flfloor".to_string(), arity: Some(1), func: fl_floor }));
        exports.insert("flceiling".to_string(), Value::Procedure(Procedure::Builtin { name: "flceiling".to_string(), arity: Some(1), func: fl_ceiling }));
        exports.insert("flround".to_string(), Value::Procedure(Procedure::Builtin { name: "flround".to_string(), arity: Some(1), func: fl_round }));
        exports.insert("fltruncate".to_string(), Value::Procedure(Procedure::Builtin { name: "fltruncate".to_string(), arity: Some(1), func: fl_truncate }));
        
        // IEEE 754 specific functions
        exports.insert("flremquo".to_string(), Value::Procedure(Procedure::Builtin { name: "flremquo".to_string(), arity: Some(2), func: fl_remquo }));
        exports.insert("flnextafter".to_string(), Value::Procedure(Procedure::Builtin { name: "flnextafter".to_string(), arity: Some(2), func: fl_nextafter }));
        exports.insert("flscalbn".to_string(), Value::Procedure(Procedure::Builtin { name: "flscalbn".to_string(), arity: Some(2), func: fl_scalbn }));
        exports.insert("flldexp".to_string(), Value::Procedure(Procedure::Builtin { name: "flldexp".to_string(), arity: Some(2), func: fl_ldexp }));
        exports.insert("flfrexp".to_string(), Value::Procedure(Procedure::Builtin { name: "flfrexp".to_string(), arity: Some(1), func: fl_frexp }));
        
        // Constants
        exports.insert("fl-e".to_string(), Value::Number(SchemeNumber::Real(std::f64::consts::E)));
        exports.insert("fl-pi".to_string(), Value::Number(SchemeNumber::Real(std::f64::consts::PI)));
        exports.insert("fl-1/pi".to_string(), Value::Number(SchemeNumber::Real(std::f64::consts::FRAC_1_PI)));
        exports.insert("fl-2/pi".to_string(), Value::Number(SchemeNumber::Real(std::f64::consts::FRAC_2_PI)));
        exports.insert("fl-pi/2".to_string(), Value::Number(SchemeNumber::Real(std::f64::consts::FRAC_PI_2)));
        exports.insert("fl-pi/4".to_string(), Value::Number(SchemeNumber::Real(std::f64::consts::FRAC_PI_4)));
        exports.insert("fl-2*pi".to_string(), Value::Number(SchemeNumber::Real(2.0 * std::f64::consts::PI)));
        exports.insert("fl-log2-of-e".to_string(), Value::Number(SchemeNumber::Real(std::f64::consts::LOG2_E)));
        exports.insert("fl-log10-of-e".to_string(), Value::Number(SchemeNumber::Real(std::f64::consts::LOG10_E)));
        exports.insert("fl-log-of-2".to_string(), Value::Number(SchemeNumber::Real(std::f64::consts::LN_2)));
        exports.insert("fl-log-of-10".to_string(), Value::Number(SchemeNumber::Real(std::f64::consts::LN_10)));
        
        // IEEE 754 constants
        exports.insert("fl-epsilon".to_string(), Value::Number(SchemeNumber::Real(f64::EPSILON)));
        exports.insert("fl-fastest".to_string(), Value::Number(SchemeNumber::Real(f64::MIN_POSITIVE)));
        exports.insert("fl-least".to_string(), Value::Number(SchemeNumber::Real(f64::MIN)));
        exports.insert("fl-greatest".to_string(), Value::Number(SchemeNumber::Real(f64::MAX)));
        exports.insert("fl-nan".to_string(), Value::Number(SchemeNumber::Real(f64::NAN)));
        exports.insert("fl-infinity".to_string(), Value::Number(SchemeNumber::Real(f64::INFINITY)));
        exports.insert("fl-negative-infinity".to_string(), Value::Number(SchemeNumber::Real(f64::NEG_INFINITY)));
        
        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // For now, return all exports regardless of parts
        Ok(self.exports())
    }
}

impl Srfi144Module {
    /// Creates a new SRFI-144 module instance
    pub fn new() -> Self {
        Self
    }
}

/// Check if value is a flonum
fn flonum_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    Ok(Value::Boolean(matches!(args[0], Value::Number(SchemeNumber::Real(_)))))
}

/// Flonum equality comparison
fn fl_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let first = extract_flonum(&args[0])?;
    for arg in &args[1..] {
        let n = extract_flonum(arg)?;
        if first != n {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Flonum less-than comparison
fn fl_less(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    for window in args.windows(2) {
        let a = extract_flonum(&window[0])?;
        let b = extract_flonum(&window[1])?;
        if a >= b {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Flonum greater-than comparison
fn fl_greater(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    for window in args.windows(2) {
        let a = extract_flonum(&window[0])?;
        let b = extract_flonum(&window[1])?;
        if a <= b {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Flonum less-than-or-equal comparison
fn fl_less_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    for window in args.windows(2) {
        let a = extract_flonum(&window[0])?;
        let b = extract_flonum(&window[1])?;
        if a > b {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Flonum greater-than-or-equal comparison
fn fl_greater_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    for window in args.windows(2) {
        let a = extract_flonum(&window[0])?;
        let b = extract_flonum(&window[1])?;
        if a < b {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Test if flonum is an integer
fn fl_integer_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Boolean(n.fract() == 0.0 && n.is_finite()))
}

/// Test if flonum is zero
fn fl_zero_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Boolean(n == 0.0))
}

/// Test if flonum is positive
fn fl_positive_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Boolean(n > 0.0))
}

/// Test if flonum is negative
fn fl_negative_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Boolean(n < 0.0))
}

/// Test if flonum is odd
fn fl_odd_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    if !n.is_finite() || n.fract() != 0.0 {
        return Ok(Value::Boolean(false));
    }
    Ok(Value::Boolean((n as i64) & 1 == 1))
}

/// Test if flonum is even
fn fl_even_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    if !n.is_finite() || n.fract() != 0.0 {
        return Ok(Value::Boolean(false));
    }
    Ok(Value::Boolean((n as i64) & 1 == 0))
}

/// Test if flonum is finite
fn fl_finite_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Boolean(n.is_finite()))
}

/// Test if flonum is infinite
fn fl_infinite_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Boolean(n.is_infinite()))
}

/// Test if flonum is NaN
fn fl_nan_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Boolean(n.is_nan()))
}

/// Test if flonum is normalized
fn fl_normalized_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Boolean(n.is_normal()))
}

/// Test if flonum is denormalized
fn fl_denormalized_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Boolean(n.is_subnormal()))
}

/// Maximum of flonums
fn fl_max(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    let mut max = extract_flonum(&args[0])?;
    for arg in &args[1..] {
        let n = extract_flonum(arg)?;
        max = max.max(n);
    }
    Ok(Value::Number(SchemeNumber::Real(max)))
}

/// Minimum of flonums
fn fl_min(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    let mut min = extract_flonum(&args[0])?;
    for arg in &args[1..] {
        let n = extract_flonum(arg)?;
        min = min.min(n);
    }
    Ok(Value::Number(SchemeNumber::Real(min)))
}

/// Flonum addition
fn fl_add(args: &[Value]) -> Result<Value> {
    let mut result = 0.0;
    for arg in args {
        let n = extract_flonum(arg)?;
        result += n;
    }
    Ok(Value::Number(SchemeNumber::Real(result)))
}

/// Flonum subtraction
fn fl_subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    let first = extract_flonum(&args[0])?;
    if args.len() == 1 {
        return Ok(Value::Number(SchemeNumber::Real(-first)));
    }
    
    let mut result = first;
    for arg in &args[1..] {
        let n = extract_flonum(arg)?;
        result -= n;
    }
    Ok(Value::Number(SchemeNumber::Real(result)))
}

/// Flonum multiplication
fn fl_multiply(args: &[Value]) -> Result<Value> {
    let mut result = 1.0;
    for arg in args {
        let n = extract_flonum(arg)?;
        result *= n;
    }
    Ok(Value::Number(SchemeNumber::Real(result)))
}

/// Flonum division
fn fl_divide(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let a = extract_flonum(&args[0])?;
    let b = extract_flonum(&args[1])?;
    Ok(Value::Number(SchemeNumber::Real(a / b)))
}

/// Flonum absolute value
fn fl_abs(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.abs())))
}

/// Flonum sign
fn fl_sign(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    let result = if n > 0.0 { 1.0 } else if n < 0.0 { -1.0 } else { n };
    Ok(Value::Number(SchemeNumber::Real(result)))
}

/// Flonum square
fn fl_square(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n * n)))
}

/// Flonum square root
fn fl_sqrt(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.sqrt())))
}

/// Exponential function
fn fl_exp(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.exp())))
}

/// Base-2 exponential function
fn fl_exp2(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.exp2())))
}

/// exp(x) - 1
fn fl_expm1(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.exp_m1())))
}

/// Natural logarithm
fn fl_log(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.ln())))
}

/// Base-2 logarithm
fn fl_log2(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.log2())))
}

/// Base-10 logarithm
fn fl_log10(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.log10())))
}

/// ln(1 + x)
fn fl_log1p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.ln_1p())))
}

/// Sine function
fn fl_sin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.sin())))
}

/// Cosine function
fn fl_cos(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.cos())))
}

/// Tangent function
fn fl_tan(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.tan())))
}

/// Arcsine function
fn fl_asin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.asin())))
}

/// Arccosine function
fn fl_acos(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.acos())))
}

/// Arctangent function
fn fl_atan(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.atan())))
}

/// Two-argument arctangent
fn fl_atan2(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let y = extract_flonum(&args[0])?;
    let x = extract_flonum(&args[1])?;
    Ok(Value::Number(SchemeNumber::Real(y.atan2(x))))
}

/// Hyperbolic sine
fn fl_sinh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.sinh())))
}

/// Hyperbolic cosine
fn fl_cosh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.cosh())))
}

/// Hyperbolic tangent
fn fl_tanh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.tanh())))
}

/// Inverse hyperbolic sine
fn fl_asinh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.asinh())))
}

/// Inverse hyperbolic cosine
fn fl_acosh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.acosh())))
}

/// Inverse hyperbolic tangent
fn fl_atanh(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.atanh())))
}

/// Floor function
fn fl_floor(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.floor())))
}

/// Ceiling function
fn fl_ceiling(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.ceil())))
}

/// Round function
fn fl_round(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.round())))
}

/// Truncate function
fn fl_truncate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_flonum(&args[0])?;
    Ok(Value::Number(SchemeNumber::Real(n.trunc())))
}

/// IEEE 754 remainder and quotient
fn fl_remquo(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let x = extract_flonum(&args[0])?;
    let y = extract_flonum(&args[1])?;
    
    // IEEE 754 remquo: remainder and quotient (simplified implementation)
    let quo = (x / y).round() as i32;
    let rem = x - (quo as f64) * y;
    
    // Return as a pair (list) - (remainder quotient)
    let rem_val = Value::Number(SchemeNumber::Real(rem));
    let quo_val = Value::Number(SchemeNumber::Integer(quo as i64));
    Ok(Value::from_vector(vec![rem_val, quo_val]))
}

/// Next representable floating-point value
fn fl_nextafter(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let x = extract_flonum(&args[0])?;
    let y = extract_flonum(&args[1])?;
    
    // Simplified nextafter implementation
    if x == y {
        Ok(Value::Number(SchemeNumber::Real(x)))
    } else if x < y {
        Ok(Value::Number(SchemeNumber::Real(x + f64::EPSILON)))
    } else {
        Ok(Value::Number(SchemeNumber::Real(x - f64::EPSILON)))
    }
}

/// Scale by power of 2
fn fl_scalbn(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let x = extract_flonum(&args[0])?;
    let n = extract_fixnum_for_fl(&args[1])?;
    
    Ok(Value::Number(SchemeNumber::Real(x * (2.0_f64).powi(n as i32))))
}

/// Load exponent (alias for scalbn)
fn fl_ldexp(args: &[Value]) -> Result<Value> {
    fl_scalbn(args)
}

/// Extract fraction and exponent
fn fl_frexp(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let x = extract_flonum(&args[0])?;
    
    if x == 0.0 {
        return Ok(Value::from_vector(vec![Value::Number(SchemeNumber::Real(0.0)), Value::Number(SchemeNumber::Integer(0))]));
    }
    
    let exp = x.log2().floor() as i64 + 1;
    let mantissa = x / (2.0_f64).powi(exp as i32);
    
    Ok(Value::from_vector(vec![Value::Number(SchemeNumber::Real(mantissa)), Value::Number(SchemeNumber::Integer(exp))]))
}

/// Helper function to extract flonum from Value
fn extract_flonum(value: &Value) -> Result<f64> {
    match value {
        Value::Number(SchemeNumber::Real(f)) => Ok(*f),
        Value::Number(SchemeNumber::Integer(i)) => Ok(*i as f64),
        _ => Err(LambdustError::type_error(
            format!("expected flonum, got {:?}", value)
        )),
    }
}

/// Helper function to extract fixnum for flonum operations
fn extract_fixnum_for_fl(value: &Value) -> Result<i64> {
    match value {
        Value::Number(SchemeNumber::Integer(i)) => Ok(*i),
        _ => Err(LambdustError::type_error(
            format!("expected integer, got {:?}", value)
        )),
    }
}
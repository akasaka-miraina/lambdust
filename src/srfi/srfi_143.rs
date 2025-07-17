//! SRFI 143: Fixnums
//!
//! This SRFI defines a set of operations for fixed-precision integers (fixnums).
//! Fixnums are a subset of integers that can be represented in a single machine word
//! and support efficient operations without boxing.

use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;

/// SRFI 143 module implementation
pub struct Srfi143Module;

impl crate::srfi::SrfiModule for Srfi143Module {
    fn srfi_id(&self) -> u32 {
        143
    }

    fn name(&self) -> &'static str {
        "SRFI 143"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["fixnums"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Helper function to create builtin procedures
        let make_builtin = |name: &str, arity: Option<usize>, func: fn(&[Value]) -> crate::Result<Value>| {
            Value::Procedure(crate::value::Procedure::Builtin {
                name: name.to_string(),
                arity,
                func,
            })
        };
        
        // Core fixnum operations
        exports.insert("fixnum?".to_string(), make_builtin("fixnum?", Some(1), fixnum_p));
        exports.insert("fx=?".to_string(), make_builtin("fx=?", Some(2), fx_equal));
        exports.insert("fx<?".to_string(), make_builtin("fx<?", None, fx_less));
        exports.insert("fx>?".to_string(), make_builtin("fx>?", None, fx_greater));
        exports.insert("fx<=?".to_string(), make_builtin("fx<=?", None, fx_less_equal));
        exports.insert("fx>=?".to_string(), make_builtin("fx>=?", None, fx_greater_equal));
        
        // Arithmetic predicates
        exports.insert("fxzero?".to_string(), make_builtin("fxzero?", Some(1), fx_zero_p));
        exports.insert("fxpositive?".to_string(), make_builtin("fxpositive?", Some(1), fx_positive_p));
        exports.insert("fxnegative?".to_string(), make_builtin("fxnegative?", Some(1), fx_negative_p));
        exports.insert("fxodd?".to_string(), make_builtin("fxodd?", Some(1), fx_odd_p));
        exports.insert("fxeven?".to_string(), make_builtin("fxeven?", Some(1), fx_even_p));
        exports.insert("fxmax".to_string(), make_builtin("fxmax", None, fx_max));
        exports.insert("fxmin".to_string(), make_builtin("fxmin", None, fx_min));
        
        // Basic arithmetic
        exports.insert("fx+".to_string(), make_builtin("fx+", None, fx_add));
        exports.insert("fx-".to_string(), make_builtin("fx-", None, fx_subtract));
        exports.insert("fx*".to_string(), make_builtin("fx*", None, fx_multiply));
        exports.insert("fx/".to_string(), make_builtin("fx/", Some(2), fx_divide));
        exports.insert("fxabs".to_string(), make_builtin("fxabs", Some(1), fx_abs));
        exports.insert("fxsquare".to_string(), make_builtin("fxsquare", Some(1), fx_square));
        exports.insert("fxsqrt".to_string(), make_builtin("fxsqrt", Some(1), fx_sqrt));
        
        // Bitwise operations
        exports.insert("fxnot".to_string(), make_builtin("fxnot", Some(1), fx_not));
        exports.insert("fxand".to_string(), make_builtin("fxand", None, fx_and));
        exports.insert("fxior".to_string(), make_builtin("fxior", None, fx_ior));
        exports.insert("fxxor".to_string(), make_builtin("fxxor", None, fx_xor));
        exports.insert("fxarithmetic-shift".to_string(), make_builtin("fxarithmetic-shift", Some(2), fx_arithmetic_shift));
        exports.insert("fxarithmetic-shift-left".to_string(), make_builtin("fxarithmetic-shift-left", Some(2), fx_arithmetic_shift_left));
        exports.insert("fxarithmetic-shift-right".to_string(), make_builtin("fxarithmetic-shift-right", Some(2), fx_arithmetic_shift_right));
        exports.insert("fxbit-count".to_string(), make_builtin("fxbit-count", Some(1), fx_bit_count));
        exports.insert("fxlength".to_string(), make_builtin("fxlength", Some(1), fx_length));
        exports.insert("fxif".to_string(), make_builtin("fxif", Some(3), fx_if));
        exports.insert("fxbit-set?".to_string(), make_builtin("fxbit-set?", Some(2), fx_bit_set_p));
        exports.insert("fxcopy-bit".to_string(), make_builtin("fxcopy-bit", Some(3), fx_copy_bit));
        exports.insert("fxfirst-set-bit".to_string(), make_builtin("fxfirst-set-bit", Some(1), fx_first_set_bit));
        
        // Range constants
        exports.insert("fx-width".to_string(), Value::Number(crate::lexer::SchemeNumber::Integer(64)));
        exports.insert("fx-least".to_string(), Value::Number(crate::lexer::SchemeNumber::Integer(i64::MIN)));
        exports.insert("fx-greatest".to_string(), Value::Number(crate::lexer::SchemeNumber::Integer(i64::MAX)));
        
        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // For now, return all exports regardless of parts
        Ok(self.exports())
    }
}

impl Srfi143Module {
    /// Creates a new SRFI-143 module instance
    pub fn new() -> Self {
        Self
    }
}

/// Check if value is a fixnum (in this implementation, all integers are fixnums)
fn fixnum_p(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    Ok(Value::Boolean(matches!(args[0], Value::Number(crate::lexer::SchemeNumber::Integer(_)))))
}

/// Fixnum equality comparison
fn fx_equal(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let first = extract_fixnum(&args[0])?;
    for arg in &args[1..] {
        let n = extract_fixnum(arg)?;
        if first != n {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Fixnum less-than comparison
fn fx_less(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    for window in args.windows(2) {
        let a = extract_fixnum(&window[0])?;
        let b = extract_fixnum(&window[1])?;
        if a >= b {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Fixnum greater-than comparison
fn fx_greater(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    for window in args.windows(2) {
        let a = extract_fixnum(&window[0])?;
        let b = extract_fixnum(&window[1])?;
        if a <= b {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Fixnum less-than-or-equal comparison
fn fx_less_equal(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    for window in args.windows(2) {
        let a = extract_fixnum(&window[0])?;
        let b = extract_fixnum(&window[1])?;
        if a > b {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Fixnum greater-than-or-equal comparison
fn fx_greater_equal(args: &[Value]) -> crate::Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    for window in args.windows(2) {
        let a = extract_fixnum(&window[0])?;
        let b = extract_fixnum(&window[1])?;
        if a < b {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

/// Test if fixnum is zero
fn fx_zero_p(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    Ok(Value::Boolean(n == 0))
}

/// Test if fixnum is positive
fn fx_positive_p(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    Ok(Value::Boolean(n > 0))
}

/// Test if fixnum is negative
fn fx_negative_p(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    Ok(Value::Boolean(n < 0))
}

/// Test if fixnum is odd
fn fx_odd_p(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    Ok(Value::Boolean(n & 1 == 1))
}

/// Test if fixnum is even
fn fx_even_p(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    Ok(Value::Boolean(n & 1 == 0))
}

/// Maximum of fixnums
fn fx_max(args: &[Value]) -> crate::Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    let mut max = extract_fixnum(&args[0])?;
    for arg in &args[1..] {
        let n = extract_fixnum(arg)?;
        if n > max {
            max = n;
        }
    }
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(max)))
}

/// Minimum of fixnums
fn fx_min(args: &[Value]) -> crate::Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    let mut min = extract_fixnum(&args[0])?;
    for arg in &args[1..] {
        let n = extract_fixnum(arg)?;
        if n < min {
            min = n;
        }
    }
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(min)))
}

/// Fixnum addition
fn fx_add(args: &[Value]) -> crate::Result<Value> {
    let mut result = 0i64;
    for arg in args {
        let n = extract_fixnum(arg)?;
        result = result.checked_add(n)
            .ok_or_else(|| LambdustError::runtime_error("fixnum overflow".to_string()))?;
    }
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Fixnum subtraction
fn fx_subtract(args: &[Value]) -> crate::Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    let first = extract_fixnum(&args[0])?;
    if args.len() == 1 {
        return Ok(Value::Number(crate::lexer::SchemeNumber::Integer(-first)));
    }
    
    let mut result = first;
    for arg in &args[1..] {
        let n = extract_fixnum(arg)?;
        result = result.checked_sub(n)
            .ok_or_else(|| LambdustError::runtime_error("fixnum underflow".to_string()))?;
    }
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Fixnum multiplication
fn fx_multiply(args: &[Value]) -> crate::Result<Value> {
    let mut result = 1i64;
    for arg in args {
        let n = extract_fixnum(arg)?;
        result = result.checked_mul(n)
            .ok_or_else(|| LambdustError::runtime_error("fixnum overflow".to_string()))?;
    }
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Fixnum division
fn fx_divide(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let a = extract_fixnum(&args[0])?;
    let b = extract_fixnum(&args[1])?;
    
    if b == 0 {
        return Err(LambdustError::runtime_error("division by zero".to_string()));
    }
    
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(a / b)))
}

/// Fixnum absolute value
fn fx_abs(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(n.abs())))
}

/// Fixnum square
fn fx_square(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    let result = n.checked_mul(n)
        .ok_or_else(|| LambdustError::runtime_error("fixnum overflow".to_string()))?;
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Fixnum square root (returns floor of square root)
fn fx_sqrt(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    if n < 0 {
        return Err(LambdustError::runtime_error("square root of negative number".to_string()));
    }
    
    let result = (n as f64).sqrt() as i64;
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Bitwise NOT
fn fx_not(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(!n)))
}

/// Bitwise AND
fn fx_and(args: &[Value]) -> crate::Result<Value> {
    let mut result = -1i64; // Start with all bits set
    for arg in args {
        let n = extract_fixnum(arg)?;
        result &= n;
    }
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Bitwise OR
fn fx_ior(args: &[Value]) -> crate::Result<Value> {
    let mut result = 0i64;
    for arg in args {
        let n = extract_fixnum(arg)?;
        result |= n;
    }
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Bitwise XOR
fn fx_xor(args: &[Value]) -> crate::Result<Value> {
    let mut result = 0i64;
    for arg in args {
        let n = extract_fixnum(arg)?;
        result ^= n;
    }
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Arithmetic shift
fn fx_arithmetic_shift(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    let count = extract_fixnum(&args[1])?;
    
    let result = if count >= 0 {
        n.checked_shl(count as u32)
            .unwrap_or(0) // Overflow to 0
    } else {
        n >> (-count as u32).min(63)
    };
    
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Arithmetic shift left
fn fx_arithmetic_shift_left(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    let count = extract_fixnum(&args[1])?;
    
    if count < 0 {
        return Err(LambdustError::runtime_error("negative shift count".to_string()));
    }
    
    let result = n.checked_shl(count as u32).unwrap_or(0);
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Arithmetic shift right
fn fx_arithmetic_shift_right(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    let count = extract_fixnum(&args[1])?;
    
    if count < 0 {
        return Err(LambdustError::runtime_error("negative shift count".to_string()));
    }
    
    let result = n >> (count as u32).min(63);
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Count number of bits set in fixnum
fn fx_bit_count(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    let count = if n >= 0 {
        n.count_ones() as i64
    } else {
        // For negative numbers, count zeros (complement)
        (!n).count_ones() as i64
    };
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(count)))
}

/// Return the number of bits in fixnum representation
fn fx_length(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    let length = if n == 0 {
        0
    } else if n > 0 {
        64 - n.leading_zeros() as i64
    } else {
        64 - (!n).leading_zeros() as i64
    };
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(length)))
}

/// Bitwise if operation: (fxif mask n1 n2) = (fxior (fxand mask n1) (fxand (fxnot mask) n2))
fn fx_if(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }
    
    let mask = extract_fixnum(&args[0])?;
    let n1 = extract_fixnum(&args[1])?;
    let n2 = extract_fixnum(&args[2])?;
    
    let result = (mask & n1) | (!mask & n2);
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Test if bit is set
fn fx_bit_set_p(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let index = extract_fixnum(&args[0])?;
    let n = extract_fixnum(&args[1])?;
    
    if index < 0 || index >= 64 {
        return Ok(Value::Boolean(false));
    }
    
    let is_set = (n & (1 << index)) != 0;
    Ok(Value::Boolean(is_set))
}

/// Copy bit from one position to another
fn fx_copy_bit(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }
    
    let index = extract_fixnum(&args[0])?;
    let n = extract_fixnum(&args[1])?;
    let bit = extract_fixnum(&args[2])?;
    
    if index < 0 || index >= 64 {
        return Ok(Value::Number(crate::lexer::SchemeNumber::Integer(n)));
    }
    
    let mask = 1 << index;
    let result = if bit != 0 {
        n | mask
    } else {
        n & !mask
    };
    
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(result)))
}

/// Find first set bit (0-indexed from right)
fn fx_first_set_bit(args: &[Value]) -> crate::Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let n = extract_fixnum(&args[0])?;
    if n == 0 {
        return Ok(Value::Number(crate::lexer::SchemeNumber::Integer(-1)));
    }
    
    let index = n.trailing_zeros() as i64;
    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(index)))
}

/// Helper function to extract fixnum from Value
fn extract_fixnum(value: &Value) -> Result<i64> {
    match value {
        Value::Number(crate::lexer::SchemeNumber::Integer(n)) => Ok(*n),
        _ => Err(LambdustError::type_error(format!("expected fixnum, got {:?}", value))),
    }
}
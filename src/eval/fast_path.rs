//! Fast path optimizations for common Scheme operations.
//!
//! This module provides specialized implementations for frequently used operations
//! that can bypass the general evaluation machinery for better performance.

#![allow(missing_docs)]

use crate::eval::{Value, OptimizedValue};
use crate::ast::Literal;
use crate::diagnostics::{Result, Error};
use crate::utils::SymbolId;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Fast path operations that can be optimized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FastPathOp {
    // Arithmetic operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    
    // Comparison operations
    NumEqual,
    NumLess,
    NumGreater,
    NumLessEqual,
    NumGreaterEqual,
    
    // List operations
    Cons,
    Car,
    Cdr,
    ListLength,
    ListRef,
    
    // Type predicates
    IsNull,
    IsPair,
    IsNumber,
    IsString,
    IsSymbol,
    IsBoolean,
    IsProcedure,
    
    // Boolean operations
    Not,
    
    // String operations
    StringLength,
    StringRef,
    StringAppend,
    
    // Vector operations
    VectorLength,
    VectorRef,
    VectorSet,
}

/// Registry of fast path operations mapped to symbol IDs.
static FAST_PATH_REGISTRY: Lazy<HashMap<SymbolId, FastPathOp>> = Lazy::new(|| {
    let mut registry = HashMap::new();
    
    // Helper to register operations
    let mut register = |name: &str, op: FastPathOp| {
        let symbol_id = crate::utils::intern_symbol(name);
        registry.insert(symbol_id, op);
    };
    
    // Arithmetic operations
    register("+", FastPathOp::Add);
    register("-", FastPathOp::Subtract);
    register("*", FastPathOp::Multiply);
    register("/", FastPathOp::Divide);
    register("modulo", FastPathOp::Modulo);
    register("remainder", FastPathOp::Modulo);
    
    // Comparison operations
    register("=", FastPathOp::NumEqual);
    register("<", FastPathOp::NumLess);
    register(">", FastPathOp::NumGreater);
    register("<=", FastPathOp::NumLessEqual);
    register(">=", FastPathOp::NumGreaterEqual);
    
    // List operations
    register("cons", FastPathOp::Cons);
    register("car", FastPathOp::Car);
    register("cdr", FastPathOp::Cdr);
    register("length", FastPathOp::ListLength);
    register("list-ref", FastPathOp::ListRef);
    
    // Type predicates
    register("null?", FastPathOp::IsNull);
    register("pair?", FastPathOp::IsPair);
    register("number?", FastPathOp::IsNumber);
    register("string?", FastPathOp::IsString);
    register("symbol?", FastPathOp::IsSymbol);
    register("boolean?", FastPathOp::IsBoolean);
    register("procedure?", FastPathOp::IsProcedure);
    
    // Boolean operations
    register("not", FastPathOp::Not);
    
    // String operations
    register("string-length", FastPathOp::StringLength);
    register("string-ref", FastPathOp::StringRef);
    register("string-append", FastPathOp::StringAppend);
    
    // Vector operations
    register("vector-length", FastPathOp::VectorLength);
    register("vector-ref", FastPathOp::VectorRef);
    register("vector-set!", FastPathOp::VectorSet);
    
    registry
});

/// Checks if an operation can use the fast path.
pub fn is_fast_path_operation(symbol_id: SymbolId) -> Option<FastPathOp> {
    FAST_PATH_REGISTRY.get(&symbol_id).copied()
}

/// Executes a fast path operation with the given arguments.
pub fn execute_fast_path(op: FastPathOp, args: &[Value]) -> Result<Value> {
    match op {
        // Arithmetic operations
        FastPathOp::Add => fast_add(args),
        FastPathOp::Subtract => fast_subtract(args),
        FastPathOp::Multiply => fast_multiply(args),
        FastPathOp::Divide => fast_divide(args),
        FastPathOp::Modulo => fast_modulo(args),
        
        // Comparison operations
        FastPathOp::NumEqual => fast_num_equal(args),
        FastPathOp::NumLess => fast_num_less(args),
        FastPathOp::NumGreater => fast_num_greater(args),
        FastPathOp::NumLessEqual => fast_num_less_equal(args),
        FastPathOp::NumGreaterEqual => fast_num_greater_equal(args),
        
        // List operations
        FastPathOp::Cons => fast_cons(args),
        FastPathOp::Car => fast_car(args),
        FastPathOp::Cdr => fast_cdr(args),
        FastPathOp::ListLength => fast_list_length(args),
        FastPathOp::ListRef => fast_list_ref(args),
        
        // Type predicates
        FastPathOp::IsNull => fast_is_null(args),
        FastPathOp::IsPair => fast_is_pair(args),
        FastPathOp::IsNumber => fast_is_number(args),
        FastPathOp::IsString => fast_is_string(args),
        FastPathOp::IsSymbol => fast_is_symbol(args),
        FastPathOp::IsBoolean => fast_is_boolean(args),
        FastPathOp::IsProcedure => fast_is_procedure(args),
        
        // Boolean operations
        FastPathOp::Not => fast_not(args),
        
        // String operations
        FastPathOp::StringLength => fast_string_length(args),
        FastPathOp::StringRef => fast_string_ref(args),
        FastPathOp::StringAppend => fast_string_append(args),
        
        // Vector operations
        FastPathOp::VectorLength => fast_vector_length(args),
        FastPathOp::VectorRef => fast_vector_ref(args),
        FastPathOp::VectorSet => fast_vector_set(args),
    }
}

/// Optimized execution for OptimizedValue operations.
pub fn execute_fast_path_optimized(op: FastPathOp, args: &[OptimizedValue]) -> Result<OptimizedValue> {
    match op {
        // Arithmetic operations (most commonly optimized)
        FastPathOp::Add => fast_add_optimized(args),
        FastPathOp::Subtract => fast_subtract_optimized(args),
        FastPathOp::Multiply => fast_multiply_optimized(args),
        FastPathOp::Divide => fast_divide_optimized(args),
        
        // Type predicates (very common and simple)
        FastPathOp::IsNull => Ok(OptimizedValue::boolean(args.len() == 1 && matches!(args[0].tag, crate::eval::optimized_value::ValueTag::Nil))),
        FastPathOp::IsNumber => Ok(OptimizedValue::boolean(args.len() == 1 && args[0].is_number())),
        FastPathOp::IsBoolean => Ok(OptimizedValue::boolean(args.len() == 1 && matches!(args[0].tag, crate::eval::optimized_value::ValueTag::Boolean))),
        
        // List operations  
        FastPathOp::Cons => {
            if args.len() == 2 {
                Ok(OptimizedValue::pair(args[0].clone(), args[1].clone()))
            } else {
                Err(Box::new(Error::arity_error("cons", 2, args.len())))
            }
        }
        
        // For other operations, fall back to regular Value operations
        _ => {
            // Convert OptimizedValue to Value for fallback
            let value_args: Result<Vec<Value>> = args.iter().map(convert_optimized_to_value).collect();
            let value_args = value_args?;
            let result = execute_fast_path(op, &value_args)?;
            convert_value_to_optimized(&result)
        }
    }
}

// =============================================================================
// ARITHMETIC OPERATIONS (highly optimized for numbers)
// =============================================================================

fn fast_add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(0));
    }
    
    let mut sum = 0.0;
    let mut all_integers = true;
    let mut int_sum = 0i64;
    
    for arg in args {
        match arg.as_number() {
            Some(n) => {
                sum += n;
                if all_integers {
                    if let Some(i) = arg.as_integer() {
                        int_sum = int_sum.saturating_add(i);
                    } else {
                        all_integers = false;
                    }
                }
            }
            None => return Err(Box::new(Error::type_mismatch_error("number", arg.clone()))),
        }
    }
    
    if all_integers && int_sum as f64 == sum {
        Ok(Value::integer(int_sum))
    } else {
        Ok(Value::number(sum))
    }
}

fn fast_add_optimized(args: &[OptimizedValue]) -> Result<OptimizedValue> {
    if args.is_empty() {
        return Ok(OptimizedValue::fixnum(0));
    }
    
    let mut sum = 0.0;
    let mut all_small_integers = true;
    let mut int_sum = 0i32;
    
    for arg in args {
        match arg.as_number() {
            Some(n) => {
                sum += n;
                if all_small_integers {
                    if let Some(i) = arg.as_integer() {
                        if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                            int_sum = int_sum.saturating_add(i as i32);
                        } else {
                            all_small_integers = false;
                        }
                    } else {
                        all_small_integers = false;
                    }
                }
            }
            None => return Err(Box::new(Error::type_mismatch_error("number", convert_optimized_to_value(arg)?))),
        }
    }
    
    if all_small_integers && int_sum as f64 == sum {
        Ok(OptimizedValue::fixnum(int_sum.into()))
    } else {
        Ok(OptimizedValue::number(sum))
    }
}

fn fast_subtract(args: &[Value]) -> Result<Value> {
    match args.len() {
        0 => Err(Box::new(Error::arity_error("-", 1, 0))),
        1 => {
            match args[0].as_number() {
                Some(n) => Ok(Value::number(-n)),
                None => Err(Box::new(Error::type_mismatch_error("number", args[0].clone()))),
            }
        }
        _ => {
            let first = args[0].as_number().ok_or_else(|| Error::type_mismatch_error("number", args[0].clone()))?;
            let mut result = first;
            
            for arg in &args[1..] {
                match arg.as_number() {
                    Some(n) => result -= n,
                    None => return Err(Box::new(Error::type_mismatch_error("number", arg.clone()))),
                }
            }
            
            Ok(Value::number(result))
        }
    }
}

fn fast_subtract_optimized(args: &[OptimizedValue]) -> Result<OptimizedValue> {
    match args.len() {
        0 => Err(Box::new(Error::arity_error("-", 1, 0))),
        1 => {
            match args[0].as_number() {
                Some(n) => Ok(OptimizedValue::number(-n)),
                None => Err(Box::new(Error::type_mismatch_error("number", convert_optimized_to_value(&args[0])?))),
            }
        }
        _ => {
            let first = args[0].as_number().ok_or_else(|| Error::type_mismatch_error("number", convert_optimized_to_value(&args[0]).unwrap()))?;
            let mut result = first;
            
            for arg in &args[1..] {
                match arg.as_number() {
                    Some(n) => result -= n,
                    None => return Err(Box::new(Error::type_mismatch_error("number", convert_optimized_to_value(arg)?))),
                }
            }
            
            // Try to return as fixnum if possible
            if result.fract() == 0.0 && result >= i32::MIN as f64 && result <= i32::MAX as f64 {
                Ok(OptimizedValue::fixnum((result as i32).into()))
            } else {
                Ok(OptimizedValue::number(result))
            }
        }
    }
}

fn fast_multiply(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(1));
    }
    
    let mut product = 1.0;
    for arg in args {
        match arg.as_number() {
            Some(n) => product *= n,
            None => return Err(Box::new(Error::type_mismatch_error("number", arg.clone()))),
        }
    }
    
    Ok(Value::number(product))
}

fn fast_multiply_optimized(args: &[OptimizedValue]) -> Result<OptimizedValue> {
    if args.is_empty() {
        return Ok(OptimizedValue::fixnum(1));
    }
    
    let mut product = 1.0;
    for arg in args {
        match arg.as_number() {
            Some(n) => product *= n,
            None => return Err(Box::new(Error::type_mismatch_error("number", convert_optimized_to_value(arg)?))),
        }
    }
    
    // Try to return as fixnum if possible
    if product.fract() == 0.0 && product >= i32::MIN as f64 && product <= i32::MAX as f64 {
        Ok(OptimizedValue::fixnum((product as i32).into()))
    } else {
        Ok(OptimizedValue::number(product))
    }
}

fn fast_divide(args: &[Value]) -> Result<Value> {
    match args.len() {
        0 => Err(Box::new(Error::arity_error("/", 1, 0))),
        1 => {
            match args[0].as_number() {
                Some(n) => {
                    if n == 0.0 {
                        Err(Box::new(Error::runtime_error("Division by zero".to_string(), None)))
                    } else {
                        Ok(Value::number(1.0 / n))
                    }
                }
                None => Err(Box::new(Error::type_mismatch_error("number", args[0].clone()))),
            }
        }
        _ => {
            let first = args[0].as_number().ok_or_else(|| Error::type_mismatch_error("number", args[0].clone()))?;
            let mut result = first;
            
            for arg in &args[1..] {
                match arg.as_number() {
                    Some(n) => {
                        if n == 0.0 {
                            return Err(Box::new(Error::runtime_error("Division by zero".to_string(), None)));
                        }
                        result /= n;
                    }
                    None => return Err(Box::new(Error::type_mismatch_error("number", arg.clone()))),
                }
            }
            
            Ok(Value::number(result))
        }
    }
}

fn fast_divide_optimized(args: &[OptimizedValue]) -> Result<OptimizedValue> {
    match args.len() {
        0 => Err(Box::new(Error::arity_error("/", 1, 0))),
        1 => {
            match args[0].as_number() {
                Some(n) => {
                    if n == 0.0 {
                        Err(Box::new(Error::runtime_error("Division by zero".to_string(), None)))
                    } else {
                        Ok(OptimizedValue::number(1.0 / n))
                    }
                }
                None => Err(Box::new(Error::type_mismatch_error("number", convert_optimized_to_value(&args[0])?))),
            }
        }
        _ => {
            let first = args[0].as_number().ok_or_else(|| Error::type_mismatch_error("number", convert_optimized_to_value(&args[0]).unwrap()))?;
            let mut result = first;
            
            for arg in &args[1..] {
                match arg.as_number() {
                    Some(n) => {
                        if n == 0.0 {
                            return Err(Box::new(Error::runtime_error("Division by zero".to_string(), None)));
                        }
                        result /= n;
                    }
                    None => return Err(Box::new(Error::type_mismatch_error("number", convert_optimized_to_value(arg)?))),
                }
            }
            
            Ok(OptimizedValue::number(result))
        }
    }
}

fn fast_modulo(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::arity_error("modulo", 2, args.len())));
    }
    
    let a = args[0].as_number().ok_or_else(|| Error::type_mismatch_error("number", args[0].clone()))?;
    let b = args[1].as_number().ok_or_else(|| Error::type_mismatch_error("number", args[1].clone()))?;
    
    if b == 0.0 {
        return Err(Box::new(Error::runtime_error("Division by zero in modulo".to_string(), None)));
    }
    
    Ok(Value::number(a % b))
}

// =============================================================================
// COMPARISON OPERATIONS
// =============================================================================

fn fast_num_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::arity_error("=", 2, args.len())));
    }
    
    let first = args[0].as_number().ok_or_else(|| Error::type_mismatch_error("number", args[0].clone()))?;
    
    for arg in &args[1..] {
        let n = arg.as_number().ok_or_else(|| Error::type_mismatch_error("number", arg.clone()))?;
        if first != n {
            return Ok(Value::f());
        }
    }
    
    Ok(Value::t())
}

fn fast_num_less(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::arity_error("<", 2, args.len())));
    }
    
    let mut prev = args[0].as_number().ok_or_else(|| Error::type_mismatch_error("number", args[0].clone()))?;
    
    for arg in &args[1..] {
        let n = arg.as_number().ok_or_else(|| Error::type_mismatch_error("number", arg.clone()))?;
        if prev >= n {
            return Ok(Value::f());
        }
        prev = n;
    }
    
    Ok(Value::t())
}

fn fast_num_greater(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::arity_error(">", 2, args.len())));
    }
    
    let mut prev = args[0].as_number().ok_or_else(|| Error::type_mismatch_error("number", args[0].clone()))?;
    
    for arg in &args[1..] {
        let n = arg.as_number().ok_or_else(|| Error::type_mismatch_error("number", arg.clone()))?;
        if prev <= n {
            return Ok(Value::f());
        }
        prev = n;
    }
    
    Ok(Value::t())
}

fn fast_num_less_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::arity_error("<=", 2, args.len())));
    }
    
    let mut prev = args[0].as_number().ok_or_else(|| Error::type_mismatch_error("number", args[0].clone()))?;
    
    for arg in &args[1..] {
        let n = arg.as_number().ok_or_else(|| Error::type_mismatch_error("number", arg.clone()))?;
        if prev > n {
            return Ok(Value::f());
        }
        prev = n;
    }
    
    Ok(Value::t())
}

fn fast_num_greater_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(Error::arity_error(">=", 2, args.len())));
    }
    
    let mut prev = args[0].as_number().ok_or_else(|| Error::type_mismatch_error("number", args[0].clone()))?;
    
    for arg in &args[1..] {
        let n = arg.as_number().ok_or_else(|| Error::type_mismatch_error("number", arg.clone()))?;
        if prev < n {
            return Ok(Value::f());
        }
        prev = n;
    }
    
    Ok(Value::t())
}

// =============================================================================
// LIST OPERATIONS
// =============================================================================

fn fast_cons(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::arity_error("cons", 2, args.len())));
    }
    
    Ok(Value::pair(args[0].clone(), args[1].clone()))
}

fn fast_car(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("car", 1, args.len())));
    }
    
    match &args[0] {
        Value::Pair(car, _) => Ok((**car).clone()),
        _ => Err(Box::new(Error::type_mismatch_error("pair", args[0].clone()))),
    }
}

fn fast_cdr(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("cdr", 1, args.len())));
    }
    
    match &args[0] {
        Value::Pair(_, cdr) => Ok((**cdr).clone()),
        _ => Err(Box::new(Error::type_mismatch_error("pair", args[0].clone()))),
    }
}

fn fast_list_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("length", 1, args.len())));
    }
    
    let mut current = &args[0];
    let mut length = 0;
    
    loop {
        match current {
            Value::Nil => return Ok(Value::integer(length)),
            Value::Pair(_, cdr) => {
                length += 1;
                current = cdr;
            }
            _ => return Err(Box::new(Error::type_mismatch_error("proper list", args[0].clone()))),
        }
    }
}

fn fast_list_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::arity_error("list-ref", 2, args.len())));
    }
    
    let index = args[1].as_integer().ok_or_else(|| Error::type_mismatch_error("integer", args[1].clone()))?;
    
    if index < 0 {
        return Err(Box::new(Error::runtime_error("Index out of bounds".to_string(), None)));
    }
    
    let mut current = &args[0];
    let mut i = 0;
    
    loop {
        match current {
            Value::Nil => return Err(Box::new(Error::runtime_error("Index out of bounds".to_string(), None))),
            Value::Pair(car, cdr) => {
                if i == index {
                    return Ok((**car).clone());
                }
                i += 1;
                current = cdr;
            }
            _ => return Err(Box::new(Error::type_mismatch_error("proper list", args[0].clone()))),
        }
    }
}

// =============================================================================
// TYPE PREDICATES
// =============================================================================

fn fast_is_null(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("null?", 1, args.len())));
    }
    
    Ok(Value::boolean(args[0].is_nil()))
}

fn fast_is_pair(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("pair?", 1, args.len())));
    }
    
    Ok(Value::boolean(args[0].is_pair()))
}

fn fast_is_number(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("number?", 1, args.len())));
    }
    
    Ok(Value::boolean(args[0].is_number()))
}

fn fast_is_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("string?", 1, args.len())));
    }
    
    Ok(Value::boolean(args[0].is_string()))
}

fn fast_is_symbol(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("symbol?", 1, args.len())));
    }
    
    Ok(Value::boolean(args[0].is_symbol()))
}

fn fast_is_boolean(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("boolean?", 1, args.len())));
    }
    
    Ok(Value::boolean(matches!(args[0], Value::Literal(Literal::Boolean(_)))))
}

fn fast_is_procedure(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("procedure?", 1, args.len())));
    }
    
    Ok(Value::boolean(args[0].is_procedure()))
}

// =============================================================================
// BOOLEAN OPERATIONS  
// =============================================================================

fn fast_not(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("not", 1, args.len())));
    }
    
    Ok(Value::boolean(args[0].is_falsy()))
}

// =============================================================================
// STRING OPERATIONS
// =============================================================================

fn fast_string_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("string-length", 1, args.len())));
    }
    
    match args[0].as_string() {
        Some(s) => Ok(Value::integer(s.chars().count() as i64)),
        None => Err(Box::new(Error::type_mismatch_error("string", args[0].clone()))),
    }
}

fn fast_string_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::arity_error("string-ref", 2, args.len())));
    }
    
    let s = args[0].as_string().ok_or_else(|| Error::type_mismatch_error("string", args[0].clone()))?;
    let index = args[1].as_integer().ok_or_else(|| Error::type_mismatch_error("integer", args[1].clone()))?;
    
    if index < 0 {
        return Err(Box::new(Error::runtime_error("String index out of bounds".to_string(), None)));
    }
    
    let chars: Vec<char> = s.chars().collect();
    let index_usize = index as usize;
    if index_usize >= chars.len() {
        return Err(Box::new(Error::runtime_error("String index out of bounds".to_string(), None)));
    }
    
    Ok(Value::Literal(Literal::Character(chars[index_usize])))
}

fn fast_string_append(args: &[Value]) -> Result<Value> {
    let mut result = String::new();
    
    for arg in args {
        match arg.as_string() {
            Some(s) => result.push_str(s),
            None => return Err(Box::new(Error::type_mismatch_error("string", arg.clone()))),
        }
    }
    
    Ok(Value::string(result))
}

// =============================================================================
// VECTOR OPERATIONS
// =============================================================================

fn fast_vector_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::arity_error("vector-length", 1, args.len())));
    }
    
    match &args[0] {
        Value::Vector(vec) => {
            if let Ok(vec_ref) = vec.read() {
                Ok(Value::integer(vec_ref.len() as i64))
            } else {
                Err(Box::new(Error::runtime_error("Vector access error".to_string(), None)))
            }
        }
        _ => Err(Box::new(Error::type_mismatch_error("vector", args[0].clone()))),
    }
}

fn fast_vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::arity_error("vector-ref", 2, args.len())));
    }
    
    let index = args[1].as_integer().ok_or_else(|| Error::type_mismatch_error("integer", args[1].clone()))?;
    
    if index < 0 {
        return Err(Box::new(Error::runtime_error("Vector index out of bounds".to_string(), None)));
    }
    
    match &args[0] {
        Value::Vector(vec) => {
            if let Ok(vec_ref) = vec.read() {
                let index_usize = index as usize;
                if index_usize >= vec_ref.len() {
                    return Err(Box::new(Error::runtime_error("Vector index out of bounds".to_string(), None)));
                }
                Ok(vec_ref[index_usize].clone())
            } else {
                Err(Box::new(Error::runtime_error("Vector access error".to_string(), None)))
            }
        }
        _ => Err(Box::new(Error::type_mismatch_error("vector", args[0].clone()))),
    }
}

fn fast_vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::arity_error("vector-set!", 3, args.len())));
    }
    
    let index = args[1].as_integer().ok_or_else(|| Error::type_mismatch_error("integer", args[1].clone()))?;
    
    if index < 0 {
        return Err(Box::new(Error::runtime_error("Vector index out of bounds".to_string(), None)));
    }
    
    match &args[0] {
        Value::Vector(vec) => {
            if let Ok(mut vec_ref) = vec.write() {
                let index_usize = index as usize;
                if index_usize >= vec_ref.len() {
                    return Err(Box::new(Error::runtime_error("Vector index out of bounds".to_string(), None)));
                }
                vec_ref[index_usize] = args[2].clone();
                Ok(Value::Unspecified)
            } else {
                Err(Box::new(Error::runtime_error("Vector access error".to_string(), None)))
            }
        }
        _ => Err(Box::new(Error::type_mismatch_error("vector", args[0].clone()))),
    }
}

// =============================================================================
// CONVERSION UTILITIES
// =============================================================================

/// Converts an OptimizedValue to a Value (for fallback compatibility).
fn convert_optimized_to_value(optimized: &OptimizedValue) -> Result<Value> {
    // This is a simplified conversion - in a real implementation,
    // you'd need to handle all the different value types properly
    match optimized.tag {
        crate::eval::optimized_value::ValueTag::Nil => Ok(Value::Nil),
        crate::eval::optimized_value::ValueTag::Boolean => {
            let b = unsafe { optimized.data.immediate != 0 };
            Ok(Value::boolean(b))
        }
        crate::eval::optimized_value::ValueTag::Fixnum => {
            let n = unsafe { optimized.data.immediate as i32 };
            Ok(Value::integer(n as i64))
        }
        crate::eval::optimized_value::ValueTag::Character => {
            let ch = unsafe { char::from_u32(optimized.data.immediate as u32).unwrap_or('?') };
            Ok(Value::Literal(Literal::Character(ch)))
        }
        crate::eval::optimized_value::ValueTag::Unspecified => Ok(Value::Unspecified),
        _ => {
            // For complex types, this would need more sophisticated conversion
            Err(Box::new(Error::runtime_error("Cannot convert optimized value to regular value".to_string(), None)))
        }
    }
}

/// Converts a Value to an OptimizedValue (for optimization).
fn convert_value_to_optimized(value: &Value) -> Result<OptimizedValue> {
    match value {
        Value::Nil => Ok(OptimizedValue::nil()),
        Value::Literal(Literal::Boolean(b)) => Ok(OptimizedValue::boolean(*b)),
        Value::Literal(literal) if literal.is_number() => {
            if let Some(n) = literal.to_f64() {
                Ok(OptimizedValue::number(n))
            } else {
                Ok(OptimizedValue::number(0.0))
            }
        }
        Value::Literal(Literal::Character(ch)) => Ok(OptimizedValue::character(*ch)),
        Value::Literal(Literal::String(s)) => Ok(OptimizedValue::string(s.clone())),
        Value::Unspecified => Ok(OptimizedValue::unspecified()),
        _ => {
            // For complex types, this would need more sophisticated conversion
            Err(Box::new(Error::runtime_error("Cannot convert value to optimized value".to_string(), None)))
        }
    }
}

/// Statistics about fast path usage.
#[derive(Debug, Clone)]
pub struct FastPathStats {
    /// Number of fast path operations executed
    pub total_fast_path_calls: usize,
    /// Number of regular evaluation calls  
    pub total_regular_calls: usize,
    /// Fast path hit rate as percentage
    pub hit_rate: f64,
    /// Time saved by fast path (estimated microseconds)
    pub estimated_time_saved_us: u64,
}

/// Global fast path statistics.
static FAST_PATH_STATS: Lazy<std::sync::Mutex<FastPathStats>> = Lazy::new(|| {
    std::sync::Mutex::new(FastPathStats {
        total_fast_path_calls: 0,
        total_regular_calls: 0,
        hit_rate: 0.0,
        estimated_time_saved_us: 0,
    })
});

/// Records a fast path operation execution.
pub fn record_fast_path_hit() {
    if let Ok(mut stats) = FAST_PATH_STATS.lock() {
        stats.total_fast_path_calls += 1;
        stats.estimated_time_saved_us += 5; // Rough estimate: 5μs saved per fast path
        let total = stats.total_fast_path_calls + stats.total_regular_calls;
        if total > 0 {
            stats.hit_rate = (stats.total_fast_path_calls as f64 / total as f64) * 100.0;
        }
    }
}

/// Records a regular evaluation operation.
pub fn record_regular_evaluation() {
    if let Ok(mut stats) = FAST_PATH_STATS.lock() {
        stats.total_regular_calls += 1;
        let total = stats.total_fast_path_calls + stats.total_regular_calls;
        if total > 0 {
            stats.hit_rate = (stats.total_fast_path_calls as f64 / total as f64) * 100.0;
        }
    }
}

/// Gets current fast path statistics.
pub fn get_fast_path_stats() -> FastPathStats {
    if let Ok(stats) = FAST_PATH_STATS.lock() {
        stats.clone()
    } else {
        FastPathStats {
            total_fast_path_calls: 0,
            total_regular_calls: 0,
            hit_rate: 0.0,
            estimated_time_saved_us: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fast_arithmetic() {
        let args = vec![Value::integer(5), Value::integer(3)];
        let result = fast_add(&args).unwrap();
        assert_eq!(result.as_integer(), Some(8));
        
        let result = fast_subtract(&args).unwrap();
        assert_eq!(result.as_integer(), Some(2));
        
        let result = fast_multiply(&args).unwrap();
        assert_eq!(result.as_integer(), Some(15));
    }
    
    #[test]
    fn test_fast_comparisons() {
        let args = vec![Value::integer(5), Value::integer(3)];
        let result = fast_num_greater(&args).unwrap();
        assert!(result.is_truthy());
        
        let result = fast_num_less(&args).unwrap();
        assert!(result.is_falsy());
        
        let args = vec![Value::integer(5), Value::integer(5)];
        let result = fast_num_equal(&args).unwrap();
        assert!(result.is_truthy());
    }
    
    #[test]
    fn test_fast_list_operations() {
        let list = Value::pair(Value::integer(1), Value::pair(Value::integer(2), Value::Nil));
        
        let args = vec![list.clone()];
        let result = fast_list_length(&args).unwrap();
        assert_eq!(result.as_integer(), Some(2));
        
        let args = vec![list.clone()];
        let result = fast_car(&args).unwrap();
        assert_eq!(result.as_integer(), Some(1));
        
        let args = vec![list.clone(), Value::integer(1)];
        let result = fast_list_ref(&args).unwrap();
        assert_eq!(result.as_integer(), Some(2));
    }
    
    #[test]
    fn test_fast_type_predicates() {
        let args = vec![Value::Nil];
        let result = fast_is_null(&args).unwrap();
        assert!(result.is_truthy());
        
        let args = vec![Value::integer(42)];
        let result = fast_is_number(&args).unwrap();
        assert!(result.is_truthy());
        
        let result = fast_is_string(&args).unwrap();
        assert!(result.is_falsy());
    }
    
    #[test]
    fn test_optimized_arithmetic() {
        let args = vec![OptimizedValue::fixnum(5), OptimizedValue::fixnum(3)];
        let result = fast_add_optimized(&args).unwrap();
        assert_eq!(result.as_integer(), Some(8));
        
        let result = fast_multiply_optimized(&args).unwrap();
        assert_eq!(result.as_integer(), Some(15));
    }
}
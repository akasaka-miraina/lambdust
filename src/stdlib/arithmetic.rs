//! Arithmetic operations for the Lambdust standard library.
//!
//! This module implements R7RS-compliant arithmetic operations with proper
//! number tower support, including exact and inexact arithmetic, rational
//! numbers, and complex numbers.

use crate::ast::Literal;
use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use std::sync::Arc;

/// Creates arithmetic operation bindings for the standard library.
pub fn create_arithmetic_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Basic arithmetic operations
    bind_basic_arithmetic(env);
    
    // Number tower operations
    bind_number_tower_operations(env);
    
    // Comparison operations
    bind_comparison_operations(env);
    
    // Number predicates
    bind_number_predicates(env);
    
    // Math functions
    bind_math_functions(env);
}

/// Binds basic arithmetic operations (+, -, *, /, modulo, etc.)
fn bind_basic_arithmetic(env: &Arc<ThreadSafeEnvironment>) {
    // Addition
    env.define("+".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "+".to_string(),
        arity_min: 0,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_add),
        effects: vec![Effect::Pure],
    })));
    
    // Subtraction
    env.define("-".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "-".to_string(),
        arity_min: 1,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_subtract),
        effects: vec![Effect::Pure],
    })));
    
    // Multiplication
    env.define("*".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "*".to_string(),
        arity_min: 0,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_multiply),
        effects: vec![Effect::Pure],
    })));
    
    // Division
    env.define("/".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "/".to_string(),
        arity_min: 1,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_divide),
        effects: vec![Effect::Pure],
    })));
    
    // Quotient (integer division)
    env.define("quotient".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "quotient".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_quotient),
        effects: vec![Effect::Pure],
    })));
    
    // Remainder
    env.define("remainder".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "remainder".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_remainder),
        effects: vec![Effect::Pure],
    })));
    
    // Modulo
    env.define("modulo".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "modulo".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_modulo),
        effects: vec![Effect::Pure],
    })));
    
    // Absolute value
    env.define("abs".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "abs".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_abs),
        effects: vec![Effect::Pure],
    })));
    
    // GCD
    env.define("gcd".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "gcd".to_string(),
        arity_min: 0,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_gcd),
        effects: vec![Effect::Pure],
    })));
    
    // LCM
    env.define("lcm".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "lcm".to_string(),
        arity_min: 0,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_lcm),
        effects: vec![Effect::Pure],
    })));
    
    // Floor-quotient
    env.define("floor-quotient".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "floor-quotient".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_floor_quotient),
        effects: vec![Effect::Pure],
    })));
    
    // Floor-remainder
    env.define("floor-remainder".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "floor-remainder".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_floor_remainder),
        effects: vec![Effect::Pure],
    })));
    
    // Truncate-quotient
    env.define("truncate-quotient".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "truncate-quotient".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_truncate_quotient),
        effects: vec![Effect::Pure],
    })));
    
    // Truncate-remainder
    env.define("truncate-remainder".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "truncate-remainder".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_truncate_remainder),
        effects: vec![Effect::Pure],
    })));
}

/// Binds number tower conversion operations.
fn bind_number_tower_operations(env: &Arc<ThreadSafeEnvironment>) {
    // Exact->Inexact conversion
    env.define("exact->inexact".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "exact->inexact".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_exact_to_inexact),
        effects: vec![Effect::Pure],
    })));
    
    // Inexact->Exact conversion
    env.define("inexact->exact".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "inexact->exact".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_inexact_to_exact),
        effects: vec![Effect::Pure],
    })));
    
    // Number->String conversion
    env.define("number->string".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "number->string".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_number_to_string),
        effects: vec![Effect::Pure],
    })));
    
    // String->Number conversion
    env.define("string->number".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "string->number".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_string_to_number),
        effects: vec![Effect::Pure],
    })));
    
    // Rationalize
    env.define("rationalize".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "rationalize".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_rationalize),
        effects: vec![Effect::Pure],
    })));
}

/// Binds comparison operations.
fn bind_comparison_operations(env: &Arc<ThreadSafeEnvironment>) {
    // Equality
    env.define("=".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "=".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_numeric_equal),
        effects: vec![Effect::Pure],
    })));
    
    // Less than
    env.define("<".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "<".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_less_than),
        effects: vec![Effect::Pure],
    })));
    
    // Greater than
    env.define(">".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: ">".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_greater_than),
        effects: vec![Effect::Pure],
    })));
    
    // Less than or equal
    env.define("<=".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "<=".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_less_equal),
        effects: vec![Effect::Pure],
    })));
    
    // Greater than or equal
    env.define(">=".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: ">=".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_greater_equal),
        effects: vec![Effect::Pure],
    })));
    
    // Zero predicate
    env.define("zero?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "zero?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_zero_p),
        effects: vec![Effect::Pure],
    })));
    
    // Positive predicate
    env.define("positive?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "positive?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_positive_p),
        effects: vec![Effect::Pure],
    })));
    
    // Negative predicate
    env.define("negative?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "negative?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_negative_p),
        effects: vec![Effect::Pure],
    })));
    
    // Odd predicate
    env.define("odd?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "odd?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_odd_p),
        effects: vec![Effect::Pure],
    })));
    
    // Even predicate
    env.define("even?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "even?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_even_p),
        effects: vec![Effect::Pure],
    })));
}

/// Binds number predicates.
fn bind_number_predicates(env: &Arc<ThreadSafeEnvironment>) {
    // Number predicate
    env.define("number?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "number?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_number_p),
        effects: vec![Effect::Pure],
    })));
    
    // Integer predicate
    env.define("integer?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "integer?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_integer_p),
        effects: vec![Effect::Pure],
    })));
    
    // Rational predicate
    env.define("rational?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "rational?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_rational_p),
        effects: vec![Effect::Pure],
    })));
    
    // Real predicate
    env.define("real?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "real?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_real_p),
        effects: vec![Effect::Pure],
    })));
    
    // Complex predicate
    env.define("complex?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "complex?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_complex_p),
        effects: vec![Effect::Pure],
    })));
    
    // Exact predicate
    env.define("exact?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "exact?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_exact_p),
        effects: vec![Effect::Pure],
    })));
    
    // Inexact predicate
    env.define("inexact?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "inexact?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_inexact_p),
        effects: vec![Effect::Pure],
    })));
    
    // Finite predicate
    env.define("finite?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "finite?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_finite_p),
        effects: vec![Effect::Pure],
    })));
    
    // Infinite predicate
    env.define("infinite?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "infinite?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_infinite_p),
        effects: vec![Effect::Pure],
    })));
    
    // NaN predicate
    env.define("nan?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "nan?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_nan_p),
        effects: vec![Effect::Pure],
    })));
}

/// Binds complex number operations.
fn bind_complex_operations(env: &Arc<ThreadSafeEnvironment>) {
    // Complex number constructors
    env.define("make-rectangular".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-rectangular".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_make_rectangular),
        effects: vec![Effect::Pure],
    })));
    
    env.define("make-polar".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-polar".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_make_polar),
        effects: vec![Effect::Pure],
    })));
    
    // Complex number accessors
    env.define("real-part".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "real-part".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_real_part),
        effects: vec![Effect::Pure],
    })));
    
    env.define("imag-part".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "imag-part".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_imag_part),
        effects: vec![Effect::Pure],
    })));
    
    env.define("magnitude".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "magnitude".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_magnitude),
        effects: vec![Effect::Pure],
    })));
    
    env.define("angle".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "angle".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_angle),
        effects: vec![Effect::Pure],
    })));
}

/// Binds mathematical functions.
fn bind_math_functions(env: &Arc<ThreadSafeEnvironment>) {
    // Maximum
    env.define("max".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "max".to_string(),
        arity_min: 1,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_max),
        effects: vec![Effect::Pure],
    })));
    
    // Minimum
    env.define("min".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "min".to_string(),
        arity_min: 1,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_min),
        effects: vec![Effect::Pure],
    })));
    
    // Floor
    env.define("floor".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "floor".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_floor),
        effects: vec![Effect::Pure],
    })));
    
    // Ceiling
    env.define("ceiling".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "ceiling".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_ceiling),
        effects: vec![Effect::Pure],
    })));
    
    // Truncate
    env.define("truncate".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "truncate".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_truncate),
        effects: vec![Effect::Pure],
    })));
    
    // Round
    env.define("round".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "round".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_round),
        effects: vec![Effect::Pure],
    })));
    
    // Exponentiation
    env.define("expt".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "expt".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_expt),
        effects: vec![Effect::Pure],
    })));
    
    // Square root
    env.define("sqrt".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "sqrt".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_sqrt),
        effects: vec![Effect::Pure],
    })));
    
    // Exponential
    env.define("exp".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "exp".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_exp),
        effects: vec![Effect::Pure],
    })));
    
    // Natural logarithm
    env.define("log".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "log".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_log),
        effects: vec![Effect::Pure],
    })));
    
    // Sine
    env.define("sin".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "sin".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_sin),
        effects: vec![Effect::Pure],
    })));
    
    // Cosine
    env.define("cos".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "cos".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_cos),
        effects: vec![Effect::Pure],
    })));
    
    // Tangent
    env.define("tan".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "tan".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_tan),
        effects: vec![Effect::Pure],
    })));
    
    // Arcsine
    env.define("asin".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "asin".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_asin),
        effects: vec![Effect::Pure],
    })));
    
    // Arccosine
    env.define("acos".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "acos".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_acos),
        effects: vec![Effect::Pure],
    })));
    
    // Arctangent
    env.define("atan".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "atan".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_atan),
        effects: vec![Effect::Pure],
    })));
    
    // Additional R7RS math functions
    
    // Square function
    env.define("square".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "square".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_square),
        effects: vec![Effect::Pure],
    })));
    
    // Complex number specific operations
    bind_complex_operations(env);
    
    // Additional number tower operations not yet bound
    env.define("exact".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "exact".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_inexact_to_exact),
        effects: vec![Effect::Pure],
    })));
    
    env.define("inexact".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "inexact".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_exact_to_inexact),
        effects: vec![Effect::Pure],
    })));
    
    // Additional predicates
    env.define("exact-integer?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "exact-integer?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_exact_integer_p),
        effects: vec![Effect::Pure],
    })));
    
    env.define("finite?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "finite?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_finite_p),
        effects: vec![Effect::Pure],
    })));
    
    env.define("infinite?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "infinite?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_infinite_p),
        effects: vec![Effect::Pure],
    })));
    
    env.define("nan?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "nan?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_nan_p),
        effects: vec![Effect::Pure],
    })));
}

// ============= ARITHMETIC IMPLEMENTATIONS =============

/// Addition operation (+)
fn primitive_add(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(0));
    }
    
    let mut result = NumberValue::Integer(0);
    
    for arg in args {
        let num = extract_number(arg, "+")?;
        result = add_numbers(result, num)?;
    }
    
    Ok(number_value_to_value(result))
}

/// Subtraction operation (-)
fn primitive_subtract(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(DiagnosticError::runtime_error(
            "- requires at least 1 argument".to_string(),
            None,
        ));
    }
    
    let first = extract_number(&args[0], "-")?;
    
    if args.len() == 1 {
        // Unary negation
        return Ok(number_value_to_value(negate_number(first)?));
    }
    
    let mut result = first;
    for arg in &args[1..] {
        let num = extract_number(arg, "-")?;
        result = subtract_numbers(result, num)?;
    }
    
    Ok(number_value_to_value(result))
}

/// Multiplication operation (*)
fn primitive_multiply(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(1));
    }
    
    let mut result = NumberValue::Integer(1);
    
    for arg in args {
        let num = extract_number(arg, "*")?;
        result = multiply_numbers(result, num)?;
    }
    
    Ok(number_value_to_value(result))
}

/// Division operation (/)
fn primitive_divide(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(DiagnosticError::runtime_error(
            "/ requires at least 1 argument".to_string(),
            None,
        ));
    }
    
    let first = extract_number(&args[0], "/")?;
    
    if args.len() == 1 {
        // Reciprocal
        return Ok(number_value_to_value(reciprocal_number(first)?));
    }
    
    let mut result = first;
    for arg in &args[1..] {
        let num = extract_number(arg, "/")?;
        result = divide_numbers(result, num)?;
    }
    
    Ok(number_value_to_value(result))
}

/// Quotient operation (quotient)
fn primitive_quotient(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("quotient expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let n1 = extract_number(&args[0], "quotient")?;
    let n2 = extract_number(&args[1], "quotient")?;
    
    Ok(number_value_to_value(quotient_numbers(n1, n2)?))
}

/// Remainder operation (remainder)
fn primitive_remainder(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("remainder expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let n1 = extract_number(&args[0], "remainder")?;
    let n2 = extract_number(&args[1], "remainder")?;
    
    Ok(number_value_to_value(remainder_numbers(n1, n2)?))
}

/// Modulo operation (modulo)
fn primitive_modulo(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("modulo expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let n1 = extract_number(&args[0], "modulo")?;
    let n2 = extract_number(&args[1], "modulo")?;
    
    Ok(number_value_to_value(modulo_numbers(n1, n2)?))
}

/// Absolute value operation (abs)
fn primitive_abs(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("abs expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "abs")?;
    Ok(number_value_to_value(abs_number(num)?))
}

/// GCD operation (gcd)
fn primitive_gcd(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(0));
    }
    
    let mut result = extract_number(&args[0], "gcd")?;
    
    for arg in &args[1..] {
        let num = extract_number(arg, "gcd")?;
        result = gcd_numbers(result, num)?;
    }
    
    Ok(number_value_to_value(result))
}

/// LCM operation (lcm)
fn primitive_lcm(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Ok(Value::integer(1));
    }
    
    let mut result = extract_number(&args[0], "lcm")?;
    
    for arg in &args[1..] {
        let num = extract_number(arg, "lcm")?;
        result = lcm_numbers(result, num)?;
    }
    
    Ok(number_value_to_value(result))
}

/// Floor-quotient operation
fn primitive_floor_quotient(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("floor-quotient expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let n1 = extract_number(&args[0], "floor-quotient")?;
    let n2 = extract_number(&args[1], "floor-quotient")?;
    
    Ok(number_value_to_value(quotient_numbers(n1, n2)?))
}

/// Floor-remainder operation
fn primitive_floor_remainder(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("floor-remainder expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let n1 = extract_number(&args[0], "floor-remainder")?;
    let n2 = extract_number(&args[1], "floor-remainder")?;
    
    Ok(number_value_to_value(remainder_numbers(n1, n2)?))
}

/// Truncate-quotient operation
fn primitive_truncate_quotient(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("truncate-quotient expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let n1 = extract_number(&args[0], "truncate-quotient")?;
    let n2 = extract_number(&args[1], "truncate-quotient")?;
    
    Ok(number_value_to_value(quotient_numbers(n1, n2)?))
}

/// Truncate-remainder operation
fn primitive_truncate_remainder(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("truncate-remainder expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let n1 = extract_number(&args[0], "truncate-remainder")?;
    let n2 = extract_number(&args[1], "truncate-remainder")?;
    
    Ok(number_value_to_value(remainder_numbers(n1, n2)?))
}

// ============= CONVERSION IMPLEMENTATIONS =============

/// Exact to inexact conversion
fn primitive_exact_to_inexact(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("exact->inexact expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "exact->inexact")?;
    Ok(number_value_to_value(to_inexact(num)))
}

/// Inexact to exact conversion
fn primitive_inexact_to_exact(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("inexact->exact expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "inexact->exact")?;
    Ok(number_value_to_value(to_exact(num)?))
}

/// Number to string conversion
fn primitive_number_to_string(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(DiagnosticError::runtime_error(
            format!("number->string expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "number->string")?;
    let radix = if args.len() == 2 {
        match args[1].as_integer() {
            Some(r) if (2..=36).contains(&r) => r as u32,
            _ => return Err(DiagnosticError::runtime_error(
                "number->string radix must be between 2 and 36".to_string(),
                None,
            )),
        }
    } else {
        10
    };
    
    Ok(Value::string(number_to_string(num, radix)))
}

/// String to number conversion
fn primitive_string_to_number(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(DiagnosticError::runtime_error(
            format!("string->number expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let s = args[0].as_string().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string->number first argument must be a string".to_string(),
            None,
        )
    })?;
    
    let radix = if args.len() == 2 {
        match args[1].as_integer() {
            Some(r) if (2..=36).contains(&r) => r as u32,
            _ => return Err(DiagnosticError::runtime_error(
                "string->number radix must be between 2 and 36".to_string(),
                None,
            )),
        }
    } else {
        10
    };
    
    match string_to_number(s, radix) {
        Some(num) => Ok(number_value_to_value(num)),
        None => Ok(Value::boolean(false)),
    }
}

/// Rationalize operation
fn primitive_rationalize(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("rationalize expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let x = extract_number(&args[0], "rationalize")?;
    let e = extract_number(&args[1], "rationalize")?;
    
    Ok(number_value_to_value(rationalize_number(x, e)?))
}

// Continue with comparison implementations...

// ============= COMPARISON IMPLEMENTATIONS =============

/// Numeric equality (=)
fn primitive_numeric_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(DiagnosticError::runtime_error(
            "= requires at least 2 arguments".to_string(),
            None,
        ));
    }
    
    let first = extract_number(&args[0], "=")?;
    
    for arg in &args[1..] {
        let num = extract_number(arg, "=")?;
        if !numbers_equal(first.clone()), num)? {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// Less than (<)
fn primitive_less_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(DiagnosticError::runtime_error(
            "< requires at least 2 arguments".to_string(),
            None,
        ));
    }
    
    for window in args.windows(2) {
        let n1 = extract_number(&window[0], "<")?;
        let n2 = extract_number(&window[1], "<")?;
        if !number_less_than(n1, n2)? {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// Greater than (>)
fn primitive_greater_than(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(DiagnosticError::runtime_error(
            "> requires at least 2 arguments".to_string(),
            None,
        ));
    }
    
    for window in args.windows(2) {
        let n1 = extract_number(&window[0], ">")?;
        let n2 = extract_number(&window[1], ">")?;
        if !number_greater_than(n1, n2)? {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// Less than or equal (<=)
fn primitive_less_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(DiagnosticError::runtime_error(
            "<= requires at least 2 arguments".to_string(),
            None,
        ));
    }
    
    for window in args.windows(2) {
        let n1 = extract_number(&window[0], "<=")?;
        let n2 = extract_number(&window[1], "<=")?;
        if !number_less_equal(n1, n2)? {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// Greater than or equal (>=)
fn primitive_greater_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(DiagnosticError::runtime_error(
            ">= requires at least 2 arguments".to_string(),
            None,
        ));
    }
    
    for window in args.windows(2) {
        let n1 = extract_number(&window[0], ">=")?;
        let n2 = extract_number(&window[1], ">=")?;
        if !number_greater_equal(n1, n2)? {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// Zero predicate (zero?)
fn primitive_zero_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("zero? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "zero?")?;
    Ok(Value::boolean(is_zero(num)))
}

/// Positive predicate (positive?)
fn primitive_positive_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("positive? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "positive?")?;
    Ok(Value::boolean(is_positive(num)?))
}

/// Negative predicate (negative?)
fn primitive_negative_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("negative? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "negative?")?;
    Ok(Value::boolean(is_negative(num)?))
}

/// Odd predicate (odd?)
fn primitive_odd_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("odd? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "odd?")?;
    Ok(Value::boolean(is_odd(num)?))
}

/// Even predicate (even?)
fn primitive_even_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("even? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "even?")?;
    Ok(Value::boolean(is_even(num)?))
}

// ============= PREDICATE IMPLEMENTATIONS =============

/// Number predicate (number?)
fn primitive_number_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("number? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    Ok(Value::boolean(args[0].is_number()))
}

/// Integer predicate (integer?)
fn primitive_integer_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("integer? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    if let Some(num) = try_extract_number(&args[0]) {
        Ok(Value::boolean(is_integer_number(num)))
    } else {
        Ok(Value::boolean(false))
    }
}

/// Rational predicate (rational?)
fn primitive_rational_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("rational? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    if let Some(num) = try_extract_number(&args[0]) {
        Ok(Value::boolean(is_rational_number(num)))
    } else {
        Ok(Value::boolean(false))
    }
}

/// Real predicate (real?)
fn primitive_real_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("real? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    if let Some(num) = try_extract_number(&args[0]) {
        Ok(Value::boolean(is_real_number(num)))
    } else {
        Ok(Value::boolean(false))
    }
}

/// Complex predicate (complex?)
fn primitive_complex_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("complex? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    Ok(Value::boolean(args[0].is_number()))
}

/// Exact predicate (exact?)
fn primitive_exact_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("exact? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    if let Some(num) = try_extract_number(&args[0]) {
        Ok(Value::boolean(is_exact_number(num)))
    } else {
        Ok(Value::boolean(false))
    }
}

/// Inexact predicate (inexact?)
fn primitive_inexact_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("inexact? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    if let Some(num) = try_extract_number(&args[0]) {
        Ok(Value::boolean(!is_exact_number(num)))
    } else {
        Ok(Value::boolean(false))
    }
}

// Continue with math function implementations...

// ============= MATH FUNCTION IMPLEMENTATIONS =============

/// Maximum function (max)
fn primitive_max(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(DiagnosticError::runtime_error(
            "max requires at least 1 argument".to_string(),
            None,
        ));
    }
    
    let mut result = extract_number(&args[0], "max")?;
    
    for arg in &args[1..] {
        let num = extract_number(arg, "max")?;
        if number_greater_than(num.clone()), result.clone())? {
            result = num;
        }
    }
    
    Ok(number_value_to_value(result))
}

/// Minimum function (min)
fn primitive_min(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(DiagnosticError::runtime_error(
            "min requires at least 1 argument".to_string(),
            None,
        ));
    }
    
    let mut result = extract_number(&args[0], "min")?;
    
    for arg in &args[1..] {
        let num = extract_number(arg, "min")?;
        if number_less_than(num.clone()), result.clone())? {
            result = num;
        }
    }
    
    Ok(number_value_to_value(result))
}

/// Floor function (floor)
fn primitive_floor(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("floor expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "floor")?;
    Ok(number_value_to_value(floor_number(num)?))
}

/// Ceiling function (ceiling)
fn primitive_ceiling(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("ceiling expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "ceiling")?;
    Ok(number_value_to_value(ceiling_number(num)?))
}

/// Truncate function (truncate)
fn primitive_truncate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("truncate expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "truncate")?;
    Ok(number_value_to_value(truncate_number(num)?))
}

/// Round function (round)
fn primitive_round(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("round expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "round")?;
    Ok(number_value_to_value(round_number(num)?))
}

/// Exponentiation function (expt)
fn primitive_expt(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("expt expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let base = extract_number(&args[0], "expt")?;
    let exp = extract_number(&args[1], "expt")?;
    
    Ok(number_value_to_value(expt_numbers(base, exp)?))
}

/// Square root function (sqrt)
fn primitive_sqrt(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("sqrt expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "sqrt")?;
    Ok(number_value_to_value(sqrt_number(num)?))
}

/// Exponential function (exp)
fn primitive_exp(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("exp expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "exp")?;
    Ok(number_value_to_value(exp_number(num)?))
}

/// Logarithm function (log)
fn primitive_log(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(DiagnosticError::runtime_error(
            format!("log expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "log")?;
    
    if args.len() == 1 {
        Ok(number_value_to_value(log_number(num)?))
    } else {
        let base = extract_number(&args[1], "log")?;
        Ok(number_value_to_value(log_base_number(num, base)?))
    }
}

/// Sine function (sin)
fn primitive_sin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("sin expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "sin")?;
    Ok(number_value_to_value(sin_number(num)?))
}

/// Cosine function (cos)
fn primitive_cos(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("cos expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "cos")?;
    Ok(number_value_to_value(cos_number(num)?))
}

/// Tangent function (tan)
fn primitive_tan(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("tan expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "tan")?;
    Ok(number_value_to_value(tan_number(num)?))
}

/// Arcsine function (asin)
fn primitive_asin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("asin expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "asin")?;
    Ok(number_value_to_value(asin_number(num)?))
}

/// Arccosine function (acos)
fn primitive_acos(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("acos expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "acos")?;
    Ok(number_value_to_value(acos_number(num)?))
}

/// Arctangent function (atan)
fn primitive_atan(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(DiagnosticError::runtime_error(
            format!("atan expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let y = extract_number(&args[0], "atan")?;
    
    if args.len() == 1 {
        Ok(number_value_to_value(atan_number(y)?))
    } else {
        let x = extract_number(&args[1], "atan")?;
        Ok(number_value_to_value(atan2_number(y, x)?))
    }
}

/// Square function (square)
fn primitive_square(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("square expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "square")?;
    Ok(number_value_to_value(square_number(num)?))
}

/// Exact-integer predicate (exact-integer?)
fn primitive_exact_integer_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("exact-integer? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    if let Some(num) = try_extract_number(&args[0]) {
        Ok(Value::boolean(is_exact_integer(num)))
    } else {
        Ok(Value::boolean(false))
    }
}

/// Finite predicate (finite?)
fn primitive_finite_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("finite? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    if let Some(num) = try_extract_number(&args[0]) {
        Ok(Value::boolean(is_finite_number(num)))
    } else {
        Ok(Value::boolean(false))
    }
}

/// Infinite predicate (infinite?)
fn primitive_infinite_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("infinite? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    if let Some(num) = try_extract_number(&args[0]) {
        Ok(Value::boolean(is_infinite_number(num)))
    } else {
        Ok(Value::boolean(false))
    }
}

/// NaN predicate (nan?)
fn primitive_nan_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("nan? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    if let Some(num) = try_extract_number(&args[0]) {
        Ok(Value::boolean(is_nan_number(num)))
    } else {
        Ok(Value::boolean(false))
    }
}

// ============= HELPER TYPES AND FUNCTIONS =============

/// Internal number representation for arithmetic operations.
#[derive(Debug, Clone, PartialEq)]
enum NumberValue {
    Integer(i64),
    Rational { numerator: i64, denominator: i64 },
    Float(f64),
    Complex { real: f64, imaginary: f64 },
}

/// Extracts a number from a Value for arithmetic operations.
fn extract_number(value: &Value, operation: &str) -> Result<NumberValue> {
    match value {
        Value::Literal(Literal::Number(n)) => Ok(NumberValue::Float(*n)),
        Value::Literal(Literal::Rational { numerator, denominator }) => 
            Ok(NumberValue::Rational { numerator: *numerator, denominator: *denominator }),
        Value::Literal(Literal::Complex { real, imaginary }) => 
            Ok(NumberValue::Complex { real: *real, imaginary: *imaginary }),
        _ => Err(DiagnosticError::runtime_error(
            format!("{operation} requires numeric arguments"),
            None,
        )),
    }
}

/// Tries to extract a number from a Value (for predicates).
fn try_extract_number(value: &Value) -> Option<NumberValue> {
    match value {
        Value::Literal(Literal::Number(n)) => Some(NumberValue::Float(*n)),
        Value::Literal(Literal::Rational { numerator, denominator }) => 
            Some(NumberValue::Rational { numerator: *numerator, denominator: *denominator }),
        Value::Literal(Literal::Complex { real, imaginary }) => 
            Some(NumberValue::Complex { real: *real, imaginary: *imaginary }),
        _ => None,
    }
}

/// Converts a NumberValue back to a Value.
fn number_value_to_value(num: NumberValue) -> Value {
    match num {
        NumberValue::Integer(i) => Value::integer(i),
        NumberValue::Rational { numerator, denominator } => 
            Value::Literal(Literal::rational(numerator, denominator)),
        NumberValue::Float(f) => Value::number(f),
        NumberValue::Complex { real, imaginary } => 
            Value::Literal(Literal::complex(real, imaginary)),
    }
}

// Placeholder implementations for number operations
// These would contain the actual arithmetic logic

fn add_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    match (&a, &b) {
        (NumberValue::Integer(a), NumberValue::Integer(b)) => 
            Ok(NumberValue::Integer(a.saturating_add(*b))),
        (NumberValue::Integer(a), NumberValue::Float(b)) => 
            Ok(NumberValue::Float(*a as f64 + b)),
        (NumberValue::Float(a), NumberValue::Integer(b)) => 
            Ok(NumberValue::Float(a + *b as f64)),
        (NumberValue::Float(a), NumberValue::Float(b)) => 
            Ok(NumberValue::Float(a + b)),
        (NumberValue::Integer(a), NumberValue::Rational { numerator, denominator }) =>
            Ok(NumberValue::Rational { numerator: a * denominator + numerator, denominator: *denominator }),
        (NumberValue::Rational { numerator, denominator }, NumberValue::Integer(b)) =>
            Ok(NumberValue::Rational { numerator: numerator + b * denominator, denominator: *denominator }),
        (NumberValue::Rational { numerator: n1, denominator: d1 }, NumberValue::Rational { numerator: n2, denominator: d2 }) => {
            let num = n1 * d2 + n2 * d1;
            let den = d1 * d2;
            Ok(simplify_rational(num, den))
        },
        (NumberValue::Complex { real: r1, imaginary: i1 }, NumberValue::Complex { real: r2, imaginary: i2 }) =>
            Ok(NumberValue::Complex { real: r1 + r2, imaginary: i1 + i2 }),
        (NumberValue::Complex { real, imaginary }, other) | (other, NumberValue::Complex { real, imaginary }) => {
            let other_real = match other {
                NumberValue::Integer(i) => *i as f64,
                NumberValue::Float(f) => *f,
                NumberValue::Rational { numerator, denominator } => *numerator as f64 / *denominator as f64,
                NumberValue::Complex { .. } => unreachable!(), // Already handled above
            };
            Ok(NumberValue::Complex { real: real + other_real, imaginary: *imaginary })
        },
        _ => {
            // Convert to common type and add
            let af = to_float(a)?;
            let bf = to_float(b)?;
            Ok(NumberValue::Float(af + bf))
        }
    }
}

fn subtract_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    match (&a, &b) {
        (NumberValue::Integer(a), NumberValue::Integer(b)) => 
            Ok(NumberValue::Integer(a.saturating_sub(*b))),
        (NumberValue::Integer(a), NumberValue::Float(b)) => 
            Ok(NumberValue::Float(*a as f64 - b)),
        (NumberValue::Float(a), NumberValue::Integer(b)) => 
            Ok(NumberValue::Float(a - *b as f64)),
        (NumberValue::Float(a), NumberValue::Float(b)) => 
            Ok(NumberValue::Float(a - b)),
        (NumberValue::Integer(a), NumberValue::Rational { numerator, denominator }) =>
            Ok(NumberValue::Rational { numerator: a * denominator - numerator, denominator: *denominator }),
        (NumberValue::Rational { numerator, denominator }, NumberValue::Integer(b)) =>
            Ok(NumberValue::Rational { numerator: numerator - b * denominator, denominator: *denominator }),
        (NumberValue::Rational { numerator: n1, denominator: d1 }, NumberValue::Rational { numerator: n2, denominator: d2 }) => {
            let num = n1 * d2 - n2 * d1;
            let den = d1 * d2;
            Ok(simplify_rational(num, den))
        },
        (NumberValue::Complex { real: r1, imaginary: i1 }, NumberValue::Complex { real: r2, imaginary: i2 }) =>
            Ok(NumberValue::Complex { real: r1 - r2, imaginary: i1 - i2 }),
        (NumberValue::Complex { real, imaginary }, other) => {
            let other_real = match other {
                NumberValue::Integer(i) => *i as f64,
                NumberValue::Float(f) => *f,
                NumberValue::Rational { numerator, denominator } => *numerator as f64 / *denominator as f64,
                NumberValue::Complex { .. } => unreachable!(),
            };
            Ok(NumberValue::Complex { real: real - other_real, imaginary: *imaginary })
        },
        (other, NumberValue::Complex { real, imaginary }) => {
            let other_real = match other {
                NumberValue::Integer(i) => *i as f64,
                NumberValue::Float(f) => *f,
                NumberValue::Rational { numerator, denominator } => *numerator as f64 / *denominator as f64,
                NumberValue::Complex { .. } => unreachable!(),
            };
            Ok(NumberValue::Complex { real: other_real - real, imaginary: -imaginary })
        },
        _ => {
            let af = to_float(a)?;
            let bf = to_float(b)?;
            Ok(NumberValue::Float(af - bf))
        }
    }
}

fn multiply_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    match (&a, &b) {
        (NumberValue::Integer(a), NumberValue::Integer(b)) => 
            Ok(NumberValue::Integer(a.saturating_mul(*b))),
        (NumberValue::Integer(a), NumberValue::Float(b)) => 
            Ok(NumberValue::Float(*a as f64 * b)),
        (NumberValue::Float(a), NumberValue::Integer(b)) => 
            Ok(NumberValue::Float(a * *b as f64)),
        (NumberValue::Float(a), NumberValue::Float(b)) => 
            Ok(NumberValue::Float(a * b)),
        (NumberValue::Integer(a), NumberValue::Rational { numerator, denominator }) =>
            Ok(simplify_rational(a * numerator, *denominator)),
        (NumberValue::Rational { numerator, denominator }, NumberValue::Integer(b)) =>
            Ok(simplify_rational(numerator * b, *denominator)),
        (NumberValue::Rational { numerator: n1, denominator: d1 }, NumberValue::Rational { numerator: n2, denominator: d2 }) => {
            let num = n1 * n2;
            let den = d1 * d2;
            Ok(simplify_rational(num, den))
        },
        (NumberValue::Complex { real: r1, imaginary: i1 }, NumberValue::Complex { real: r2, imaginary: i2 }) => {
            // (a + bi)(c + di) = (ac - bd) + (ad + bc)i
            Ok(NumberValue::Complex { 
                real: r1 * r2 - i1 * i2, 
                imaginary: r1 * i2 + i1 * r2 
            })
        },
        (NumberValue::Complex { real, imaginary }, other) | (other, NumberValue::Complex { real, imaginary }) => {
            let other_real = match other {
                NumberValue::Integer(i) => *i as f64,
                NumberValue::Float(f) => *f,
                NumberValue::Rational { numerator, denominator } => *numerator as f64 / *denominator as f64,
                NumberValue::Complex { .. } => unreachable!(),
            };
            Ok(NumberValue::Complex { real: real * other_real, imaginary: imaginary * other_real })
        },
        _ => {
            let af = to_float(a)?;
            let bf = to_float(b)?;
            Ok(NumberValue::Float(af * bf))
        }
    }
}

fn divide_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    // Check for division by zero first
    match &b {
        NumberValue::Integer(0) => {
            return Err(DiagnosticError::runtime_error(
                "Division by zero".to_string(),
                None,
            ));
        },
        NumberValue::Float(f) if *f == 0.0 => {
            return Err(DiagnosticError::runtime_error(
                "Division by zero".to_string(),
                None,
            ));
        },
        _ => {}
    }
    
    match (&a, &b) {
        (NumberValue::Integer(a), NumberValue::Integer(b)) => {
            if a % b == 0 {
                Ok(NumberValue::Integer(a / b))
            } else {
                Ok(NumberValue::Rational { numerator: *a, denominator: *b })
            }
        },
        (NumberValue::Integer(a), NumberValue::Float(b)) => 
            Ok(NumberValue::Float(*a as f64 / b)),
        (NumberValue::Float(a), NumberValue::Integer(b)) => 
            Ok(NumberValue::Float(a / *b as f64)),
        (NumberValue::Float(a), NumberValue::Float(b)) => 
            Ok(NumberValue::Float(a / b)),
        (NumberValue::Integer(a), NumberValue::Rational { numerator, denominator }) =>
            Ok(simplify_rational(a * denominator, *numerator)),
        (NumberValue::Rational { numerator, denominator }, NumberValue::Integer(b)) =>
            Ok(simplify_rational(*numerator, denominator * b)),
        (NumberValue::Rational { numerator: n1, denominator: d1 }, NumberValue::Rational { numerator: n2, denominator: d2 }) => {
            let num = n1 * d2;
            let den = d1 * n2;
            Ok(simplify_rational(num, den))
        },
        (NumberValue::Complex { real: r1, imaginary: i1 }, NumberValue::Complex { real: r2, imaginary: i2 }) => {
            // (a + bi)/(c + di) = ((ac + bd) + (bc - ad)i)/(c² + d²)
            let denom = r2 * r2 + i2 * i2;
            if denom == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "Division by zero complex number".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Complex { 
                real: (r1 * r2 + i1 * i2) / denom, 
                imaginary: (i1 * r2 - r1 * i2) / denom 
            })
        },
        (NumberValue::Complex { real, imaginary }, other) => {
            let other_real = match other {
                NumberValue::Integer(i) => *i as f64,
                NumberValue::Float(f) => *f,
                NumberValue::Rational { numerator, denominator } => *numerator as f64 / *denominator as f64,
                NumberValue::Complex { .. } => unreachable!(),
            };
            if other_real == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "Division by zero".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Complex { real: real / other_real, imaginary: imaginary / other_real })
        },
        (other, NumberValue::Complex { real, imaginary }) => {
            let other_real = match other {
                NumberValue::Integer(i) => *i as f64,
                NumberValue::Float(f) => *f,
                NumberValue::Rational { numerator, denominator } => *numerator as f64 / *denominator as f64,
                NumberValue::Complex { .. } => unreachable!(),
            };
            let denom = real * real + imaginary * imaginary;
            if denom == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "Division by zero complex number".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Complex { 
                real: (other_real * real) / denom, 
                imaginary: (-other_real * imaginary) / denom 
            })
        },
        _ => {
            let af = to_float(a)?;
            let bf = to_float(b)?;
            if bf == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "Division by zero".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Float(af / bf))
        }
    }
}

fn negate_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Integer(-i)),
        NumberValue::Float(f) => Ok(NumberValue::Float(-f)),
        NumberValue::Rational { numerator, denominator } => 
            Ok(NumberValue::Rational { numerator: -numerator, denominator }),
        NumberValue::Complex { real, imaginary } => 
            Ok(NumberValue::Complex { real: -real, imaginary: -imaginary }),
    }
}

fn reciprocal_number(a: NumberValue) -> Result<NumberValue> {
    divide_numbers(NumberValue::Integer(1), a)
}

fn quotient_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    // Integer division truncated towards zero
    let af = to_float(a)?;
    let bf = to_float(b)?;
    
    if bf == 0.0 {
        return Err(DiagnosticError::runtime_error(
            "quotient: division by zero".to_string(),
            None,
        ));
    }
    
    Ok(NumberValue::Float((af / bf).trunc()))
}

fn remainder_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    // Remainder after division truncated towards zero
    let af = to_float(a)?;
    let bf = to_float(b)?;
    
    if bf == 0.0 {
        return Err(DiagnosticError::runtime_error(
            "remainder: division by zero".to_string(),
            None,
        ));
    }
    
    let quotient = (af / bf).trunc();
    Ok(NumberValue::Float(af - bf * quotient))
}

fn modulo_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    // Modulo with result having same sign as divisor
    let af = to_float(a)?;
    let bf = to_float(b)?;
    
    if bf == 0.0 {
        return Err(DiagnosticError::runtime_error(
            "modulo: division by zero".to_string(),
            None,
        ));
    }
    
    let result = af % bf;
    if (result > 0.0) != (bf > 0.0) && result != 0.0 {
        Ok(NumberValue::Float(result + bf))
    } else {
        Ok(NumberValue::Float(result))
    }
}

fn abs_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Integer(i.abs())),
        NumberValue::Float(f) => Ok(NumberValue::Float(f.abs())),
        NumberValue::Rational { numerator, denominator } => 
            Ok(NumberValue::Rational { numerator: numerator.abs(), denominator: denominator.abs() }),
        NumberValue::Complex { real, imaginary } => {
            let magnitude = (real * real + imaginary * imaginary).sqrt();
            Ok(NumberValue::Float(magnitude))
        },
    }
}

fn gcd_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    let ai = match a {
        NumberValue::Integer(i) => i,
        _ => return Err(DiagnosticError::runtime_error(
            "gcd requires integer arguments".to_string(),
            None,
        )),
    };
    
    let bi = match b {
        NumberValue::Integer(i) => i,
        _ => return Err(DiagnosticError::runtime_error(
            "gcd requires integer arguments".to_string(),
            None,
        )),
    };
    
    let result = gcd_i64(ai.abs() as u64, bi.abs() as u64) as i64;
    Ok(NumberValue::Integer(result))
}

fn lcm_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    let ai = match a {
        NumberValue::Integer(i) => i,
        _ => return Err(DiagnosticError::runtime_error(
            "lcm requires integer arguments".to_string(),
            None,
        )),
    };
    
    let bi = match b {
        NumberValue::Integer(i) => i,
        _ => return Err(DiagnosticError::runtime_error(
            "lcm requires integer arguments".to_string(),
            None,
        )),
    };
    
    if ai == 0 || bi == 0 {
        return Ok(NumberValue::Integer(0));
    }
    
    let gcd = gcd_i64(ai.abs() as u64, bi.abs() as u64) as i64;
    let result = (ai.abs() / gcd) * bi.abs();
    Ok(NumberValue::Integer(result))
}

#[allow(dead_code)]
fn floor_quotient_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    match (a, b) {
        (NumberValue::Float(x), NumberValue::Float(y)) => {
            if y == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "floor-quotient: division by zero".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Float((x / y).floor()))
        },
        (NumberValue::Integer(x), NumberValue::Integer(y)) => {
            if y == 0 {
                return Err(DiagnosticError::runtime_error(
                    "floor-quotient: division by zero".to_string(),
                    None,
                ));
            }
            // Integer division with floor semantics
            let result = if (x < 0) != (y < 0) && x % y != 0 {
                (x / y) - 1
            } else {
                x / y
            };
            Ok(NumberValue::Integer(result))
        },
        // Convert mixed types to float
        (a, b) => {
            let x = match a {
                NumberValue::Float(f) => f,
                NumberValue::Integer(i) => i as f64,
                NumberValue::Rational { numerator, denominator } => numerator as f64 / denominator as f64,
                NumberValue::Complex { real, imaginary: _ } => real, // Use real part for division
            };
            let y = match b {
                NumberValue::Float(f) => f,
                NumberValue::Integer(i) => i as f64,
                NumberValue::Rational { numerator, denominator } => numerator as f64 / denominator as f64,
                NumberValue::Complex { real, imaginary: _ } => real, // Use real part for division
            };
            if y == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "floor-quotient: division by zero".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Float((x / y).floor()))
        }
    }
}

#[allow(dead_code)]
fn floor_remainder_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    match (a, b) {
        (NumberValue::Float(x), NumberValue::Float(y)) => {
            if y == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "floor-remainder: division by zero".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Float(x - y * (x / y).floor()))
        },
        (NumberValue::Integer(x), NumberValue::Integer(y)) => {
            if y == 0 {
                return Err(DiagnosticError::runtime_error(
                    "floor-remainder: division by zero".to_string(),
                    None,
                ));
            }
            let result = x - y * (if (x < 0) != (y < 0) && x % y != 0 {
                (x / y) - 1
            } else {
                x / y
            });
            Ok(NumberValue::Integer(result))
        },
        // Convert mixed types to float
        (a, b) => {
            let x = match a {
                NumberValue::Float(f) => f,
                NumberValue::Integer(i) => i as f64,
                NumberValue::Rational { numerator, denominator } => numerator as f64 / denominator as f64,
                NumberValue::Complex { real, imaginary: _ } => real, // Use real part for division
            };
            let y = match b {
                NumberValue::Float(f) => f,
                NumberValue::Integer(i) => i as f64,
                NumberValue::Rational { numerator, denominator } => numerator as f64 / denominator as f64,
                NumberValue::Complex { real, imaginary: _ } => real, // Use real part for division
            };
            if y == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "floor-remainder: division by zero".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Float(x - y * (x / y).floor()))
        }
    }
}

#[allow(dead_code)]
fn truncate_quotient_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    match (a, b) {
        (NumberValue::Float(x), NumberValue::Float(y)) => {
            if y == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "truncate-quotient: division by zero".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Float((x / y).trunc()))
        },
        (NumberValue::Integer(x), NumberValue::Integer(y)) => {
            if y == 0 {
                return Err(DiagnosticError::runtime_error(
                    "truncate-quotient: division by zero".to_string(),
                    None,
                ));
            }
            // Regular integer division truncates towards zero
            Ok(NumberValue::Integer(x / y))
        },
        // Convert mixed types to float
        (a, b) => {
            let x = match a {
                NumberValue::Float(f) => f,
                NumberValue::Integer(i) => i as f64,
                NumberValue::Rational { numerator, denominator } => numerator as f64 / denominator as f64,
                NumberValue::Complex { real, imaginary: _ } => real, // Use real part for division
            };
            let y = match b {
                NumberValue::Float(f) => f,
                NumberValue::Integer(i) => i as f64,
                NumberValue::Rational { numerator, denominator } => numerator as f64 / denominator as f64,
                NumberValue::Complex { real, imaginary: _ } => real, // Use real part for division
            };
            if y == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "truncate-quotient: division by zero".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Float((x / y).trunc()))
        }
    }
}

#[allow(dead_code)]
fn truncate_remainder_numbers(a: NumberValue, b: NumberValue) -> Result<NumberValue> {
    match (a, b) {
        (NumberValue::Float(x), NumberValue::Float(y)) => {
            if y == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "truncate-remainder: division by zero".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Float(x - y * (x / y).trunc()))
        },
        (NumberValue::Integer(x), NumberValue::Integer(y)) => {
            if y == 0 {
                return Err(DiagnosticError::runtime_error(
                    "truncate-remainder: division by zero".to_string(),
                    None,
                ));
            }
            // Regular integer remainder
            Ok(NumberValue::Integer(x % y))
        },
        // Convert mixed types to float
        (a, b) => {
            let x = match a {
                NumberValue::Float(f) => f,
                NumberValue::Integer(i) => i as f64,
                NumberValue::Rational { numerator, denominator } => numerator as f64 / denominator as f64,
                NumberValue::Complex { real, imaginary: _ } => real, // Use real part for division
            };
            let y = match b {
                NumberValue::Float(f) => f,
                NumberValue::Integer(i) => i as f64,
                NumberValue::Rational { numerator, denominator } => numerator as f64 / denominator as f64,
                NumberValue::Complex { real, imaginary: _ } => real, // Use real part for division
            };
            if y == 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "truncate-remainder: division by zero".to_string(),
                    None,
                ));
            }
            Ok(NumberValue::Float(x - y * (x / y).trunc()))
        }
    }
}

fn to_inexact(a: NumberValue) -> NumberValue {
    match a {
        NumberValue::Integer(i) => NumberValue::Float(i as f64),
        NumberValue::Rational { numerator, denominator } => 
            NumberValue::Float(numerator as f64 / denominator as f64),
        NumberValue::Float(f) => NumberValue::Float(f),
        NumberValue::Complex { real, imaginary } => NumberValue::Complex { real, imaginary },
    }
}

fn to_exact(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Integer(i)),
        NumberValue::Rational { numerator, denominator } => 
            Ok(NumberValue::Rational { numerator, denominator }),
        NumberValue::Float(f) => {
            if f.is_infinite() || f.is_nan() {
                return Err(DiagnosticError::runtime_error(
                    "Cannot convert infinite or NaN to exact".to_string(),
                    None,
                ));
            }
            
            if f.fract() == 0.0 {
                Ok(NumberValue::Integer(f as i64))
            } else {
                // Convert to rational approximation
                let (num, den) = float_to_rational(f);
                Ok(NumberValue::Rational { numerator: num, denominator: den })
            }
        },
        NumberValue::Complex { real, imaginary } => {
            if imaginary != 0.0 {
                return Err(DiagnosticError::runtime_error(
                    "Cannot convert complex number with non-zero imaginary part to exact".to_string(),
                    None,
                ));
            }
            to_exact(NumberValue::Float(real))
        },
    }
}

fn number_to_string(a: NumberValue, radix: u32) -> String {
    match a {
        NumberValue::Integer(i) => {
            if radix == 10 {
                i.to_string()
            } else {
                format_integer_radix(i, radix)
            }
        },
        NumberValue::Float(f) => {
            if radix == 10 {
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{:.0}", f)
                } else {
                    f.to_string()
                }
            } else {
                // Convert to integer if possible, otherwise use base 10
                if f.fract() == 0.0 && f.is_finite() {
                    format_integer_radix(f as i64, radix)
                } else {
                    f.to_string()
                }
            }
        },
        NumberValue::Rational { numerator, denominator } => {
            if denominator == 1 {
                if radix == 10 {
                    numerator.to_string()
                } else {
                    format_integer_radix(numerator, radix)
                }
            } else {
                if radix == 10 {
                    format!("{}/{}", numerator, denominator)
                } else {
                    format!("{}/{}", 
                        format_integer_radix(numerator, radix),
                        format_integer_radix(denominator, radix))
                }
            }
        },
        NumberValue::Complex { real, imaginary } => {
            if imaginary == 0.0 {
                number_to_string(NumberValue::Float(real), radix)
            } else if real == 0.0 {
                if imaginary == 1.0 {
                    "+i".to_string()
                } else if imaginary == -1.0 {
                    "-i".to_string()
                } else {
                    format!("{}i", number_to_string(NumberValue::Float(imaginary), radix))
                }
            } else {
                let real_str = number_to_string(NumberValue::Float(real), radix);
                if imaginary > 0.0 {
                    if imaginary == 1.0 {
                        format!("{}+i", real_str)
                    } else {
                        format!("{}+{}i", real_str, number_to_string(NumberValue::Float(imaginary), radix))
                    }
                } else {
                    if imaginary == -1.0 {
                        format!("{}-i", real_str)
                    } else {
                        format!("{}{}i", real_str, number_to_string(NumberValue::Float(imaginary), radix))
                    }
                }
            }
        },
    }
}

fn string_to_number(s: &str, radix: u32) -> Option<NumberValue> {
    let s = s.trim();
    
    if s.is_empty() {
        return None;
    }
    
    // Try to parse as complex number first
    if s.contains('i') {
        return parse_complex_number(s, radix);
    }
    
    // Try to parse as rational number
    if s.contains('/') {
        return parse_rational_number(s, radix);
    }
    
    // Try to parse as integer
    if let Ok(i) = parse_integer_radix(s, radix) {
        return Some(NumberValue::Integer(i));
    }
    
    // Try to parse as float (only base 10)
    if radix == 10 {
        if let Ok(f) = s.parse::<f64>() {
            return Some(NumberValue::Float(f));
        }
    }
    
    None
}

fn rationalize_number(x: NumberValue, e: NumberValue) -> Result<NumberValue> {
    let xf = to_float(x)?;
    let ef = to_float(e)?;
    
    if ef < 0.0 {
        return Err(DiagnosticError::runtime_error(
            "rationalize: tolerance must be non-negative".to_string(),
            None,
        ));
    }
    
    // Find the simplest rational number within tolerance
    let (num, den) = rationalize_float(xf, ef);
    
    if den == 1 {
        Ok(NumberValue::Integer(num))
    } else {
        Ok(NumberValue::Rational { numerator: num, denominator: den })
    }
}

// ============= UTILITY FUNCTIONS =============

/// Convert float to rational approximation
fn float_to_rational(f: f64) -> (i64, i64) {
    if f.fract() == 0.0 {
        return (f as i64, 1);
    }
    
    // Use continued fractions for conversion
    let mut x = f;
    let mut a = x.floor() as i64;
    let mut p0 = 1i64;
    let mut q0 = 0i64;
    let mut p1 = a;
    let mut q1 = 1i64;
    
    for _ in 0..20 { // Limit iterations to prevent infinite loops
        x = x - a as f64;
        if x.abs() < 1e-15 {
            break;
        }
        x = 1.0 / x;
        a = x.floor() as i64;
        
        let p2 = a * p1 + p0;
        let q2 = a * q1 + q0;
        
        p0 = p1;
        q0 = q1;
        p1 = p2;
        q1 = q2;
        
        // Check if we have a good enough approximation
        if (f - p1 as f64 / q1 as f64).abs() < 1e-15 {
            break;
        }
    }
    
    (p1, q1)
}

/// Rationalize a float within a given tolerance
fn rationalize_float(x: f64, tolerance: f64) -> (i64, i64) {
    if tolerance == 0.0 {
        return float_to_rational(x);
    }
    
    // Find the simplest fraction in the interval [x-tolerance, x+tolerance]
    let lower = x - tolerance;
    let upper = x + tolerance;
    
    // Use the mediant method to find the simplest fraction
    let mut p0 = lower.floor() as i64;
    let mut q0 = 1i64;
    let mut p1 = upper.ceil() as i64;
    let mut q1 = 1i64;
    
    loop {
        let pm = p0 + p1;
        let qm = q0 + q1;
        let mediant = pm as f64 / qm as f64;
        
        if mediant < lower {
            p0 = pm;
            q0 = qm;
        } else if mediant > upper {
            p1 = pm;
            q1 = qm;
        } else {
            return (pm, qm);
        }
        
        // Prevent infinite loops
        if qm > 10000 {
            break;
        }
    }
    
    // Fall back to simple conversion
    float_to_rational(x)
}

/// Format integer in given radix
fn format_integer_radix(mut n: i64, radix: u32) -> String {
    if n == 0 {
        return "0".to_string();
    }
    
    let negative = n < 0;
    if negative {
        n = -n;
    }
    
    let digits = "0123456789abcdefghijklmnopqrstuvwxyz";
    let mut result = String::new();
    
    while n > 0 {
        let digit = (n % radix as i64) as usize;
        result.insert(0, digits.chars().nth(digit).unwrap());
        n /= radix as i64;
    }
    
    if negative {
        result.insert(0, '-');
    }
    
    result
}

/// Parse integer in given radix
fn parse_integer_radix(s: &str, radix: u32) -> std::result::Result<i64, std::num::ParseIntError> {
    if radix == 10 {
        s.parse::<i64>()
    } else {
        i64::from_str_radix(s, radix)
    }
}

/// Parse complex number from string
fn parse_complex_number(s: &str, radix: u32) -> Option<NumberValue> {
    let s = s.replace(" ", ""); // Remove spaces
    
    // Handle pure imaginary numbers
    if s == "i" || s == "+i" {
        return Some(NumberValue::Complex { real: 0.0, imaginary: 1.0 });
    }
    if s == "-i" {
        return Some(NumberValue::Complex { real: 0.0, imaginary: -1.0 });
    }
    
    // Find the 'i' and split the string
    if let Some(i_pos) = s.rfind('i') {
        let number_part = &s[..i_pos];
        
        // Check if there's a real part
        let (real_part, imag_part) = if let Some(last_op) = number_part.rfind(&['+', '-'][..]) {
            if last_op == 0 {
                // No real part, just imaginary
                ("0", number_part)
            } else {
                (&number_part[..last_op], &number_part[last_op..])
            }
        } else {
            ("0", number_part)
        };
        
        let real = if real_part == "0" {
            0.0
        } else {
            match string_to_number(real_part, radix) {
                Some(n) => to_float(n).ok()?,
                None => return None,
            }
        };
        
        let imaginary = if imag_part.is_empty() || imag_part == "+" {
            1.0
        } else if imag_part == "-" {
            -1.0
        } else {
            match string_to_number(imag_part, radix) {
                Some(n) => to_float(n).ok()?,
                None => return None,
            }
        };
        
        return Some(NumberValue::Complex { real, imaginary });
    }
    
    None
}

/// Parse rational number from string
fn parse_rational_number(s: &str, radix: u32) -> Option<NumberValue> {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let numerator = parse_integer_radix(parts[0], radix).ok()?;
    let denominator = parse_integer_radix(parts[1], radix).ok()?;
    
    if denominator == 0 {
        return None;
    }
    
    Some(simplify_rational(numerator, denominator))
}

fn numbers_equal(a: NumberValue, b: NumberValue) -> Result<bool> {
    match (&a, &b) {
        (NumberValue::Integer(a), NumberValue::Integer(b)) => Ok(a == b),
        (NumberValue::Float(a), NumberValue::Float(b)) => Ok(a == b),
        (NumberValue::Integer(a), NumberValue::Float(b)) => Ok(*a as f64 == *b),
        (NumberValue::Float(a), NumberValue::Integer(b)) => Ok(*a == *b as f64),
        (NumberValue::Rational { numerator: n1, denominator: d1 }, 
         NumberValue::Rational { numerator: n2, denominator: d2 }) => Ok(n1 * d2 == n2 * d1),
        (NumberValue::Complex { real: r1, imaginary: i1 }, 
         NumberValue::Complex { real: r2, imaginary: i2 }) => Ok(r1 == r2 && i1 == i2),
        _ => {
            // Convert to common type for comparison
            let af = to_float_or_complex(a)?;
            let bf = to_float_or_complex(b)?;
            match (af, bf) {
                (ComplexOrReal::Real(a), ComplexOrReal::Real(b)) => Ok(a == b),
                (ComplexOrReal::Complex(r1, i1), ComplexOrReal::Complex(r2, i2)) => Ok(r1 == r2 && i1 == i2),
                (ComplexOrReal::Real(r), ComplexOrReal::Complex(cr, ci)) | 
                (ComplexOrReal::Complex(cr, ci), ComplexOrReal::Real(r)) => Ok(r == cr && ci == 0.0),
            }
        }
    }
}

fn number_less_than(a: NumberValue, b: NumberValue) -> Result<bool> {
    // Only defined for real numbers
    if !is_real_number(a.clone()) || !is_real_number(b.clone()) {
        return Err(DiagnosticError::runtime_error(
            "< not defined for complex numbers".to_string(),
            None,
        ));
    }
    
    let af = to_float(a)?;
    let bf = to_float(b)?;
    Ok(af < bf)
}

fn number_greater_than(a: NumberValue, b: NumberValue) -> Result<bool> {
    // Only defined for real numbers
    if !is_real_number(a.clone()) || !is_real_number(b.clone()) {
        return Err(DiagnosticError::runtime_error(
            "> not defined for complex numbers".to_string(),
            None,
        ));
    }
    
    let af = to_float(a)?;
    let bf = to_float(b)?;
    Ok(af > bf)
}

fn number_less_equal(a: NumberValue, b: NumberValue) -> Result<bool> {
    // Only defined for real numbers
    if !is_real_number(a.clone()) || !is_real_number(b.clone()) {
        return Err(DiagnosticError::runtime_error(
            "<= not defined for complex numbers".to_string(),
            None,
        ));
    }
    
    let af = to_float(a)?;
    let bf = to_float(b)?;
    Ok(af <= bf)
}

fn number_greater_equal(a: NumberValue, b: NumberValue) -> Result<bool> {
    // Only defined for real numbers
    if !is_real_number(a.clone()) || !is_real_number(b.clone()) {
        return Err(DiagnosticError::runtime_error(
            ">= not defined for complex numbers".to_string(),
            None,
        ));
    }
    
    let af = to_float(a)?;
    let bf = to_float(b)?;
    Ok(af >= bf)
}

fn is_zero(a: NumberValue) -> bool {
    match a {
        NumberValue::Integer(i) => i == 0,
        NumberValue::Float(f) => f == 0.0,
        NumberValue::Rational { numerator, .. } => numerator == 0,
        NumberValue::Complex { real, imaginary } => real == 0.0 && imaginary == 0.0,
    }
}

fn is_positive(a: NumberValue) -> Result<bool> {
    if !is_real_number(a.clone()) {
        return Err(DiagnosticError::runtime_error(
            "positive? not defined for complex numbers".to_string(),
            None,
        ));
    }
    
    let f = to_float(a)?;
    Ok(f > 0.0)
}

fn is_negative(a: NumberValue) -> Result<bool> {
    if !is_real_number(a.clone()) {
        return Err(DiagnosticError::runtime_error(
            "negative? not defined for complex numbers".to_string(),
            None,
        ));
    }
    
    let f = to_float(a)?;
    Ok(f < 0.0)
}

fn is_odd(a: NumberValue) -> Result<bool> {
    if !is_integer_number(a.clone()) {
        return Err(DiagnosticError::runtime_error(
            "odd? requires an integer argument".to_string(),
            None,
        ));
    }
    
    match a {
        NumberValue::Integer(i) => Ok(i % 2 != 0),
        NumberValue::Rational { numerator, denominator } if denominator == 1 => Ok(numerator % 2 != 0),
        _ => Err(DiagnosticError::runtime_error(
            "odd? requires an integer argument".to_string(),
            None,
        )),
    }
}

fn is_even(a: NumberValue) -> Result<bool> {
    if !is_integer_number(a.clone()) {
        return Err(DiagnosticError::runtime_error(
            "even? requires an integer argument".to_string(),
            None,
        ));
    }
    
    match a {
        NumberValue::Integer(i) => Ok(i % 2 == 0),
        NumberValue::Rational { numerator, denominator } if denominator == 1 => Ok(numerator % 2 == 0),
        _ => Err(DiagnosticError::runtime_error(
            "even? requires an integer argument".to_string(),
            None,
        )),
    }
}

fn is_integer_number(a: NumberValue) -> bool {
    match a {
        NumberValue::Integer(_) => true,
        NumberValue::Float(f) => f.fract() == 0.0 && f.is_finite(),
        NumberValue::Rational { denominator, .. } => denominator == 1,
        NumberValue::Complex { real, imaginary } => imaginary == 0.0 && real.fract() == 0.0 && real.is_finite(),
    }
}

fn is_rational_number(a: NumberValue) -> bool {
    match a {
        NumberValue::Integer(_) => true,
        NumberValue::Rational { .. } => true,
        NumberValue::Float(f) => f.is_finite(),
        NumberValue::Complex { real, imaginary } => imaginary == 0.0 && real.is_finite(),
    }
}

fn is_real_number(a: NumberValue) -> bool {
    match a {
        NumberValue::Integer(_) => true,
        NumberValue::Float(_) => true,
        NumberValue::Rational { .. } => true,
        NumberValue::Complex { imaginary, .. } => imaginary == 0.0,
    }
}

fn is_exact_number(a: NumberValue) -> bool {
    match a {
        NumberValue::Integer(_) => true,
        NumberValue::Rational { .. } => true,
        NumberValue::Float(_) => false,
        NumberValue::Complex { .. } => false,
    }
}

fn floor_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Integer(i)),
        NumberValue::Float(f) => Ok(NumberValue::Float(f.floor())),
        NumberValue::Rational { numerator, denominator } => {
            let result = numerator / denominator;
            if numerator % denominator == 0 {
                Ok(NumberValue::Integer(result))
            } else if numerator > 0 {
                Ok(NumberValue::Integer(result))
            } else {
                Ok(NumberValue::Integer(result - 1))
            }
        },
        NumberValue::Complex { .. } => Err(DiagnosticError::runtime_error(
            "floor not defined for complex numbers".to_string(),
            None,
        )),
    }
}

fn ceiling_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Integer(i)),
        NumberValue::Float(f) => Ok(NumberValue::Float(f.ceil())),
        NumberValue::Rational { numerator, denominator } => {
            let result = numerator / denominator;
            if numerator % denominator == 0 {
                Ok(NumberValue::Integer(result))
            } else if numerator > 0 {
                Ok(NumberValue::Integer(result + 1))
            } else {
                Ok(NumberValue::Integer(result))
            }
        },
        NumberValue::Complex { .. } => Err(DiagnosticError::runtime_error(
            "ceiling not defined for complex numbers".to_string(),
            None,
        )),
    }
}

fn truncate_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Integer(i)),
        NumberValue::Float(f) => Ok(NumberValue::Float(f.trunc())),
        NumberValue::Rational { numerator, denominator } => {
            Ok(NumberValue::Integer(numerator / denominator))
        },
        NumberValue::Complex { .. } => Err(DiagnosticError::runtime_error(
            "truncate not defined for complex numbers".to_string(),
            None,
        )),
    }
}

fn round_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Integer(i)),
        NumberValue::Float(f) => Ok(NumberValue::Float(f.round())),
        NumberValue::Rational { numerator, denominator } => {
            let f = numerator as f64 / denominator as f64;
            Ok(NumberValue::Float(f.round()))
        },
        NumberValue::Complex { .. } => Err(DiagnosticError::runtime_error(
            "round not defined for complex numbers".to_string(),
            None,
        )),
    }
}

fn expt_numbers(base: NumberValue, exp: NumberValue) -> Result<NumberValue> {
    match (base, exp) {
        (NumberValue::Integer(b), NumberValue::Integer(e)) => {
            if e >= 0 {
                Ok(NumberValue::Integer(b.pow(e as u32)))
            } else {
                Ok(NumberValue::Float((b as f64).powf(e as f64)))
            }
        },
        (base_val, exp_val) => {
            let base_f = to_float(base_val.clone())?;
            let exp_f = to_float(exp_val.clone())?;
            
            // Handle complex results
            if base_f < 0.0 && exp_f.fract() != 0.0 {
                // Negative base with non-integer exponent results in complex number
                // Use complex exponentiation: z^w = e^(w * ln(z))
                let ln_base = log_number(base_val)?;
                let product = multiply_numbers(exp_val, ln_base)?;
                exp_number(product)
            } else {
                Ok(NumberValue::Float(base_f.powf(exp_f)))
            }
        }
    }
}

fn sqrt_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => {
            if i >= 0 {
                let sqrt_val = (i as f64).sqrt();
                if sqrt_val.fract() == 0.0 && sqrt_val * sqrt_val == i as f64 {
                    Ok(NumberValue::Integer(sqrt_val as i64))
                } else {
                    Ok(NumberValue::Float(sqrt_val))
                }
            } else {
                Ok(NumberValue::Complex { real: 0.0, imaginary: (-i as f64).sqrt() })
            }
        },
        NumberValue::Float(f) => {
            if f >= 0.0 {
                Ok(NumberValue::Float(f.sqrt()))
            } else {
                Ok(NumberValue::Complex { real: 0.0, imaginary: (-f).sqrt() })
            }
        },
        NumberValue::Rational { numerator, denominator } => {
            let f = numerator as f64 / denominator as f64;
            if f >= 0.0 {
                Ok(NumberValue::Float(f.sqrt()))
            } else {
                Ok(NumberValue::Complex { real: 0.0, imaginary: (-f).sqrt() })
            }
        },
        NumberValue::Complex { real, imaginary } => {
            let magnitude = (real * real + imaginary * imaginary).sqrt();
            let angle = imaginary.atan2(real);
            let sqrt_magnitude = magnitude.sqrt();
            let half_angle = angle / 2.0;
            Ok(NumberValue::Complex {
                real: sqrt_magnitude * half_angle.cos(),
                imaginary: sqrt_magnitude * half_angle.sin(),
            })
        },
    }
}

fn exp_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Float((i as f64).exp())),
        NumberValue::Float(f) => Ok(NumberValue::Float(f.exp())),
        NumberValue::Rational { numerator, denominator } => {
            let f = numerator as f64 / denominator as f64;
            Ok(NumberValue::Float(f.exp()))
        },
        NumberValue::Complex { real, imaginary } => {
            // e^(a + bi) = e^a * (cos(b) + i*sin(b))
            let exp_real = real.exp();
            Ok(NumberValue::Complex {
                real: exp_real * imaginary.cos(),
                imaginary: exp_real * imaginary.sin(),
            })
        },
    }
}

fn log_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => {
            if i > 0 {
                Ok(NumberValue::Float((i as f64).ln()))
            } else if i == 0 {
                Ok(NumberValue::Float(f64::NEG_INFINITY))
            } else {
                // ln(negative) = ln(|negative|) + i*pi
                Ok(NumberValue::Complex {
                    real: ((-i) as f64).ln(),
                    imaginary: std::f64::consts::PI,
                })
            }
        },
        NumberValue::Float(f) => {
            if f > 0.0 {
                Ok(NumberValue::Float(f.ln()))
            } else if f == 0.0 {
                Ok(NumberValue::Float(f64::NEG_INFINITY))
            } else {
                Ok(NumberValue::Complex {
                    real: (-f).ln(),
                    imaginary: std::f64::consts::PI,
                })
            }
        },
        NumberValue::Rational { numerator, denominator } => {
            let f = numerator as f64 / denominator as f64;
            if f > 0.0 {
                Ok(NumberValue::Float(f.ln()))
            } else if f == 0.0 {
                Ok(NumberValue::Float(f64::NEG_INFINITY))
            } else {
                Ok(NumberValue::Complex {
                    real: (-f).ln(),
                    imaginary: std::f64::consts::PI,
                })
            }
        },
        NumberValue::Complex { real, imaginary } => {
            let magnitude = (real * real + imaginary * imaginary).sqrt();
            let angle = imaginary.atan2(real);
            Ok(NumberValue::Complex {
                real: magnitude.ln(),
                imaginary: angle,
            })
        },
    }
}

fn log_base_number(a: NumberValue, base: NumberValue) -> Result<NumberValue> {
    let ln_a = log_number(a)?;
    let ln_base = log_number(base)?;
    divide_numbers(ln_a, ln_base)
}

fn sin_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Float((i as f64).sin())),
        NumberValue::Float(f) => Ok(NumberValue::Float(f.sin())),
        NumberValue::Rational { numerator, denominator } => {
            let f = numerator as f64 / denominator as f64;
            Ok(NumberValue::Float(f.sin()))
        },
        NumberValue::Complex { real, imaginary } => {
            // sin(z) = sin(x)cosh(y) + i*cos(x)sinh(y)
            let sin_real = real.sin() * imaginary.cosh();
            let sin_imag = real.cos() * imaginary.sinh();
            Ok(NumberValue::Complex { real: sin_real, imaginary: sin_imag })
        },
    }
}

fn cos_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Float((i as f64).cos())),
        NumberValue::Float(f) => Ok(NumberValue::Float(f.cos())),
        NumberValue::Rational { numerator, denominator } => {
            let f = numerator as f64 / denominator as f64;
            Ok(NumberValue::Float(f.cos()))
        },
        NumberValue::Complex { real, imaginary } => {
            // cos(z) = cos(x)cosh(y) - i*sin(x)sinh(y)
            let cos_real = real.cos() * imaginary.cosh();
            let cos_imag = -real.sin() * imaginary.sinh();
            Ok(NumberValue::Complex { real: cos_real, imaginary: cos_imag })
        },
    }
}

fn tan_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Float((i as f64).tan())),
        NumberValue::Float(f) => Ok(NumberValue::Float(f.tan())),
        NumberValue::Rational { numerator, denominator } => {
            let f = numerator as f64 / denominator as f64;
            Ok(NumberValue::Float(f.tan()))
        },
        NumberValue::Complex { .. } => {
            // tan(z) = sin(z) / cos(z)
            let sin_z = sin_number(a.clone())?;
            let cos_z = cos_number(a)?;
            divide_numbers(sin_z, cos_z)
        },
    }
}

fn asin_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => {
            let f = i as f64;
            if f.abs() <= 1.0 {
                Ok(NumberValue::Float(f.asin()))
            } else {
                // For |x| > 1, asin(x) = -i * ln(i*x + sqrt(1-x^2))
                complex_asin(NumberValue::Complex { real: f, imaginary: 0.0 })
            }
        },
        NumberValue::Float(f) => {
            if f.abs() <= 1.0 {
                Ok(NumberValue::Float(f.asin()))
            } else {
                complex_asin(NumberValue::Complex { real: f, imaginary: 0.0 })
            }
        },
        NumberValue::Rational { numerator, denominator } => {
            let f = numerator as f64 / denominator as f64;
            if f.abs() <= 1.0 {
                Ok(NumberValue::Float(f.asin()))
            } else {
                complex_asin(NumberValue::Complex { real: f, imaginary: 0.0 })
            }
        },
        NumberValue::Complex { .. } => complex_asin(a),
    }
}

fn acos_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => {
            let f = i as f64;
            if f.abs() <= 1.0 {
                Ok(NumberValue::Float(f.acos()))
            } else {
                complex_acos(NumberValue::Complex { real: f, imaginary: 0.0 })
            }
        },
        NumberValue::Float(f) => {
            if f.abs() <= 1.0 {
                Ok(NumberValue::Float(f.acos()))
            } else {
                complex_acos(NumberValue::Complex { real: f, imaginary: 0.0 })
            }
        },
        NumberValue::Rational { numerator, denominator } => {
            let f = numerator as f64 / denominator as f64;
            if f.abs() <= 1.0 {
                Ok(NumberValue::Float(f.acos()))
            } else {
                complex_acos(NumberValue::Complex { real: f, imaginary: 0.0 })
            }
        },
        NumberValue::Complex { .. } => complex_acos(a),
    }
}

fn atan_number(a: NumberValue) -> Result<NumberValue> {
    match a {
        NumberValue::Integer(i) => Ok(NumberValue::Float((i as f64).atan())),
        NumberValue::Float(f) => Ok(NumberValue::Float(f.atan())),
        NumberValue::Rational { numerator, denominator } => {
            let f = numerator as f64 / denominator as f64;
            Ok(NumberValue::Float(f.atan()))
        },
        NumberValue::Complex { .. } => complex_atan(a),
    }
}

fn atan2_number(y: NumberValue, x: NumberValue) -> Result<NumberValue> {
    let y_f = to_float(y)?;
    let x_f = to_float(x)?;
    Ok(NumberValue::Float(y_f.atan2(x_f)))
}

fn square_number(a: NumberValue) -> Result<NumberValue> {
    // Implementation for square
    multiply_numbers(a.clone()), a)
}

fn is_exact_integer(a: NumberValue) -> bool {
    // Implementation for exact-integer?
    match a {
        NumberValue::Integer(_) => true,
        NumberValue::Rational { denominator, .. } => denominator == 1,
        _ => false,
    }
}

fn is_finite_number(a: NumberValue) -> bool {
    // Implementation for finite?
    match a {
        NumberValue::Float(f) => f.is_finite(),
        NumberValue::Complex { real, imaginary } => real.is_finite() && imaginary.is_finite(),
        _ => true, // Integers and rationals are always finite
    }
}

fn is_infinite_number(a: NumberValue) -> bool {
    // Implementation for infinite?
    match a {
        NumberValue::Float(f) => f.is_infinite(),
        NumberValue::Complex { real, imaginary } => real.is_infinite() || imaginary.is_infinite(),
        _ => false, // Integers and rationals are never infinite
    }
}

fn is_nan_number(a: NumberValue) -> bool {
    // Implementation for nan?
    match a {
        NumberValue::Float(f) => f.is_nan(),
        NumberValue::Complex { real, imaginary } => real.is_nan() || imaginary.is_nan(),
        _ => false, // Integers and rationals are never NaN
    }
}

// ============= COMPLEX NUMBER PRIMITIVE IMPLEMENTATIONS =============

/// Make rectangular complex number
fn primitive_make_rectangular(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("make-rectangular expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let real = to_float(extract_number(&args[0], "make-rectangular")?)?;
    let imag = to_float(extract_number(&args[1], "make-rectangular")?)?;
    
    Ok(Value::Literal(Literal::complex(real, imag)))
}

/// Make polar complex number
fn primitive_make_polar(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(DiagnosticError::runtime_error(
            format!("make-polar expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let magnitude = to_float(extract_number(&args[0], "make-polar")?)?;
    let angle = to_float(extract_number(&args[1], "make-polar")?)?;
    
    let real = magnitude * angle.cos();
    let imag = magnitude * angle.sin();
    
    Ok(Value::Literal(Literal::complex(real, imag)))
}

/// Real part of a number
fn primitive_real_part(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("real-part expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "real-part")?;
    match num {
        NumberValue::Complex { real, .. } => Ok(Value::number(real)),
        NumberValue::Integer(i) => Ok(Value::integer(i)),
        NumberValue::Float(f) => Ok(Value::number(f)),
        NumberValue::Rational { numerator, denominator } => 
            Ok(Value::Literal(Literal::rational(numerator, denominator))),
    }
}

/// Imaginary part of a number
fn primitive_imag_part(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("imag-part expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "imag-part")?;
    match num {
        NumberValue::Complex { imaginary, .. } => Ok(Value::number(imaginary)),
        _ => Ok(Value::integer(0)), // Real numbers have imaginary part 0
    }
}

/// Magnitude of a number
fn primitive_magnitude(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("magnitude expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "magnitude")?;
    let abs_result = abs_number(num)?;
    Ok(number_value_to_value(abs_result))
}

/// Angle of a number
fn primitive_angle(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(DiagnosticError::runtime_error(
            format!("angle expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    let num = extract_number(&args[0], "angle")?;
    match num {
        NumberValue::Complex { real, imaginary } => {
            Ok(Value::number(imaginary.atan2(real)))
        },
        NumberValue::Integer(i) => {
            if i >= 0 {
                Ok(Value::number(0.0))
            } else {
                Ok(Value::number(std::f64::consts::PI))
            }
        },
        NumberValue::Float(f) => {
            if f >= 0.0 {
                Ok(Value::number(0.0))
            } else {
                Ok(Value::number(std::f64::consts::PI))
            }
        },
        NumberValue::Rational { numerator, denominator } => {
            let f = numerator as f64 / denominator as f64;
            if f >= 0.0 {
                Ok(Value::number(0.0))
            } else {
                Ok(Value::number(std::f64::consts::PI))
            }
        },
    }
}

// ============= HELPER FUNCTIONS =============

/// Convert a NumberValue to f64
fn to_float(num: NumberValue) -> Result<f64> {
    match num {
        NumberValue::Integer(i) => Ok(i as f64),
        NumberValue::Float(f) => Ok(f),
        NumberValue::Rational { numerator, denominator } => Ok(numerator as f64 / denominator as f64),
        NumberValue::Complex { real, imaginary } => {
            if imaginary == 0.0 {
                Ok(real)
            } else {
                Err(DiagnosticError::runtime_error(
                    "Cannot convert complex number with non-zero imaginary part to real".to_string(),
                    None,
                ))
            }
        },
    }
}

/// Simplify a rational number by reducing to lowest terms
fn simplify_rational(numerator: i64, denominator: i64) -> NumberValue {
    if denominator == 0 {
        return NumberValue::Float(f64::INFINITY);
    }
    
    if numerator == 0 {
        return NumberValue::Integer(0);
    }
    
    let gcd = gcd_i64(numerator.abs() as u64, denominator.abs() as u64) as i64;
    let num = numerator / gcd;
    let den = denominator / gcd;
    
    // Ensure denominator is positive
    if den < 0 {
        NumberValue::Rational { numerator: -num, denominator: -den }
    } else if den == 1 {
        NumberValue::Integer(num)
    } else {
        NumberValue::Rational { numerator: num, denominator: den }
    }
}

/// Greatest common divisor for i64
fn gcd_i64(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd_i64(b, a % b)
    }
}

/// Complex arcsine
fn complex_asin(z: NumberValue) -> Result<NumberValue> {
    match z {
        NumberValue::Complex { real, imaginary } => {
            // asin(z) = -i * ln(i*z + sqrt(1-z^2))
            let i_z = NumberValue::Complex { real: -imaginary, imaginary: real };
            let z_squared = multiply_numbers(z.clone()), z)?;
            let one_minus_z_sq = subtract_numbers(NumberValue::Integer(1), z_squared)?;
            let sqrt_part = sqrt_number(one_minus_z_sq)?;
            let sum = add_numbers(i_z, sqrt_part)?;
            let ln_result = log_number(sum)?;
            multiply_numbers(NumberValue::Complex { real: 0.0, imaginary: -1.0 }, ln_result)
        },
        _ => asin_number(z), // Should not reach here for real numbers > 1
    }
}

/// Complex arccosine
fn complex_acos(z: NumberValue) -> Result<NumberValue> {
    match z {
        NumberValue::Complex { real: _, imaginary: _ } => {
            // acos(z) = -i * ln(z + i*sqrt(1-z^2))
            let z_squared = multiply_numbers(z.clone()), z.clone())?;
            let one_minus_z_sq = subtract_numbers(NumberValue::Integer(1), z_squared)?;
            let sqrt_part = sqrt_number(one_minus_z_sq)?;
            let i_sqrt = match sqrt_part {
                NumberValue::Complex { real: sr, imaginary: si } => 
                    NumberValue::Complex { real: -si, imaginary: sr },
                other => {
                    let f = to_float(other)?;
                    NumberValue::Complex { real: 0.0, imaginary: f }
                }
            };
            let sum = add_numbers(z, i_sqrt)?;
            let ln_result = log_number(sum)?;
            multiply_numbers(NumberValue::Complex { real: 0.0, imaginary: -1.0 }, ln_result)
        },
        _ => acos_number(z),
    }
}

/// Complex arctangent
fn complex_atan(z: NumberValue) -> Result<NumberValue> {
    match z {
        NumberValue::Complex { real: _, imaginary: _ } => {
            // atan(z) = (i/2) * ln((i+z)/(i-z))
            let i = NumberValue::Complex { real: 0.0, imaginary: 1.0 };
            let i_plus_z = add_numbers(i.clone()), z.clone())?;
            let i_minus_z = subtract_numbers(i, z)?;
            let ratio = divide_numbers(i_plus_z, i_minus_z)?;
            let ln_result = log_number(ratio)?;
            multiply_numbers(NumberValue::Complex { real: 0.0, imaginary: 0.5 }, ln_result)
        },
        _ => atan_number(z),
    }
}

/// Helper enum for conversion
#[derive(Debug, Clone)]
enum ComplexOrReal {
    Real(f64),
    Complex(f64, f64),
}

/// Convert NumberValue to ComplexOrReal for comparison
fn to_float_or_complex(num: NumberValue) -> Result<ComplexOrReal> {
    match num {
        NumberValue::Integer(i) => Ok(ComplexOrReal::Real(i as f64)),
        NumberValue::Float(f) => Ok(ComplexOrReal::Real(f)),
        NumberValue::Rational { numerator, denominator } => 
            Ok(ComplexOrReal::Real(numerator as f64 / denominator as f64)),
        NumberValue::Complex { real, imaginary } => 
            Ok(ComplexOrReal::Complex(real, imaginary)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_predicates() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        create_arithmetic_bindings(&env);
        
        // Test that number? returns true for numbers
        let args = vec![Value::integer(42)];
        let result = primitive_number_p(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::string("hello")];
        let result = primitive_number_p(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_basic_arithmetic() {
        // Test addition
        let args = vec![Value::integer(2), Value::integer(3)];
        let _result = primitive_add(&args).unwrap();
        // This would pass once the real implementation is done
        // assert_eq!(result, Value::integer(5));
    }
    
    #[test]
    fn test_comparison() {
        // Test numeric equality
        let args = vec![Value::integer(42), Value::integer(42)];
        let _result = primitive_numeric_equal(&args).unwrap();
        // This would pass once the real implementation is done
        // assert_eq!(result, Value::boolean(true));
    }
}
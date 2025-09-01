//! Arithmetic operations module for Lambdust
//! 
//! This module provides R7RS-compliant numeric operations organized into
//! separate submodules for better compilation performance.

pub mod basic_ops;
pub mod comparison;
pub mod predicates;
pub mod rounding;
pub mod math_functions;
pub mod integer_ops;
pub mod conversions;

// Re-export main public interface
pub use basic_ops::*;
pub use comparison::*;
pub use predicates::*;
pub use rounding::*;
pub use math_functions::*;
pub use integer_ops::*;
pub use conversions::*;

use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::effects::Effect;
use std::sync::Arc;

/// Creates arithmetic bindings in the given environment
pub fn create_arithmetic_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Helper function to bind arithmetic primitives
    fn bind_pure_arithmetic_primitive(
        env: &Arc<ThreadSafeEnvironment>,
        name: &str,
        min_arity: usize,
        max_arity: Option<usize>,
        implementation: fn(&[Value]) -> crate::diagnostics::Result<Value>,
    ) {
        env.define(
            name.to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: name.to_string(),
                arity_min: min_arity,
                arity_max: max_arity,
                implementation: PrimitiveImpl::Native(implementation),
                effects: vec![Effect::Pure],
            })),
        );
    }

    // Basic arithmetic operations
    bind_pure_arithmetic_primitive(env, "+", 0, None, primitive_add);
    bind_pure_arithmetic_primitive(env, "-", 1, None, primitive_subtract);  
    bind_pure_arithmetic_primitive(env, "*", 0, None, primitive_multiply);
    bind_pure_arithmetic_primitive(env, "/", 1, None, primitive_divide);
    
    // Integer operations
    bind_pure_arithmetic_primitive(env, "quotient", 2, Some(2), primitive_quotient);
    bind_pure_arithmetic_primitive(env, "remainder", 2, Some(2), primitive_remainder);
    bind_pure_arithmetic_primitive(env, "modulo", 2, Some(2), primitive_modulo);
    bind_pure_arithmetic_primitive(env, "abs", 1, Some(1), primitive_abs);
    bind_pure_arithmetic_primitive(env, "gcd", 0, None, primitive_gcd);
    bind_pure_arithmetic_primitive(env, "lcm", 0, None, primitive_lcm);
    bind_pure_arithmetic_primitive(env, "floor-quotient", 2, Some(2), primitive_floor_quotient);
    bind_pure_arithmetic_primitive(env, "floor-remainder", 2, Some(2), primitive_floor_remainder);
    bind_pure_arithmetic_primitive(env, "truncate-quotient", 2, Some(2), primitive_truncate_quotient);
    bind_pure_arithmetic_primitive(env, "truncate-remainder", 2, Some(2), primitive_truncate_remainder);
    
    // Comparison operations
    bind_pure_arithmetic_primitive(env, "=", 0, None, primitive_numeric_equal);
    bind_pure_arithmetic_primitive(env, "<", 2, None, primitive_less_than);
    bind_pure_arithmetic_primitive(env, ">", 2, None, primitive_greater_than);
    bind_pure_arithmetic_primitive(env, "<=", 2, None, primitive_less_equal);
    bind_pure_arithmetic_primitive(env, ">=", 2, None, primitive_greater_equal);
    
    // Predicates
    bind_pure_arithmetic_primitive(env, "zero?", 1, Some(1), primitive_zero_p);
    bind_pure_arithmetic_primitive(env, "positive?", 1, Some(1), primitive_positive_p);
    bind_pure_arithmetic_primitive(env, "negative?", 1, Some(1), primitive_negative_p);
    bind_pure_arithmetic_primitive(env, "odd?", 1, Some(1), primitive_odd_p);
    bind_pure_arithmetic_primitive(env, "even?", 1, Some(1), primitive_even_p);
    bind_pure_arithmetic_primitive(env, "number?", 1, Some(1), primitive_number_p);
    bind_pure_arithmetic_primitive(env, "integer?", 1, Some(1), primitive_integer_p);
    bind_pure_arithmetic_primitive(env, "rational?", 1, Some(1), primitive_rational_p);
    bind_pure_arithmetic_primitive(env, "real?", 1, Some(1), primitive_real_p);
    bind_pure_arithmetic_primitive(env, "complex?", 1, Some(1), primitive_complex_p);
    bind_pure_arithmetic_primitive(env, "exact?", 1, Some(1), primitive_exact_p);
    bind_pure_arithmetic_primitive(env, "inexact?", 1, Some(1), primitive_inexact_p);
    bind_pure_arithmetic_primitive(env, "exact-integer?", 1, Some(1), primitive_exact_integer_p);
    bind_pure_arithmetic_primitive(env, "finite?", 1, Some(1), primitive_finite_p);
    bind_pure_arithmetic_primitive(env, "infinite?", 1, Some(1), primitive_infinite_p);
    bind_pure_arithmetic_primitive(env, "nan?", 1, Some(1), primitive_nan_p);
    
    // Rounding and conversion
    bind_pure_arithmetic_primitive(env, "max", 1, None, primitive_max);
    bind_pure_arithmetic_primitive(env, "min", 1, None, primitive_min);
    bind_pure_arithmetic_primitive(env, "floor", 1, Some(1), primitive_floor);
    bind_pure_arithmetic_primitive(env, "ceiling", 1, Some(1), primitive_ceiling);
    bind_pure_arithmetic_primitive(env, "truncate", 1, Some(1), primitive_truncate);
    bind_pure_arithmetic_primitive(env, "round", 1, Some(1), primitive_round);
    bind_pure_arithmetic_primitive(env, "exact->inexact", 1, Some(1), primitive_exact_to_inexact);
    bind_pure_arithmetic_primitive(env, "inexact->exact", 1, Some(1), primitive_inexact_to_exact);
    
    // Mathematical functions
    bind_pure_arithmetic_primitive(env, "expt", 2, Some(2), primitive_expt);
    bind_pure_arithmetic_primitive(env, "sqrt", 1, Some(1), primitive_sqrt);
    bind_pure_arithmetic_primitive(env, "exp", 1, Some(1), primitive_exp);
    bind_pure_arithmetic_primitive(env, "log", 1, Some(2), primitive_log);
    bind_pure_arithmetic_primitive(env, "sin", 1, Some(1), primitive_sin);
    bind_pure_arithmetic_primitive(env, "cos", 1, Some(1), primitive_cos);
    bind_pure_arithmetic_primitive(env, "tan", 1, Some(1), primitive_tan);
    bind_pure_arithmetic_primitive(env, "asin", 1, Some(1), primitive_asin);
    bind_pure_arithmetic_primitive(env, "acos", 1, Some(1), primitive_acos);
    bind_pure_arithmetic_primitive(env, "atan", 1, Some(2), primitive_atan);
    bind_pure_arithmetic_primitive(env, "square", 1, Some(1), primitive_square);
    
    // String conversion and complex numbers
    bind_pure_arithmetic_primitive(env, "number->string", 1, Some(2), primitive_number_to_string);
    bind_pure_arithmetic_primitive(env, "string->number", 1, Some(2), primitive_string_to_number);
    bind_pure_arithmetic_primitive(env, "rationalize", 2, Some(2), primitive_rationalize);
    bind_pure_arithmetic_primitive(env, "make-rectangular", 2, Some(2), primitive_make_rectangular);
    bind_pure_arithmetic_primitive(env, "make-polar", 2, Some(2), primitive_make_polar);
    bind_pure_arithmetic_primitive(env, "real-part", 1, Some(1), primitive_real_part);
    bind_pure_arithmetic_primitive(env, "imag-part", 1, Some(1), primitive_imag_part);
    bind_pure_arithmetic_primitive(env, "magnitude", 1, Some(1), primitive_magnitude);
    bind_pure_arithmetic_primitive(env, "angle", 1, Some(1), primitive_angle);
}
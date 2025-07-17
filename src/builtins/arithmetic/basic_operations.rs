//! Basic Arithmetic Operations Module
//!
//! このモジュールは基本的な算術演算（+, -, *, /）を実装します。
//! Scheme R7RS準拠の数値計算と型変換を提供します。

use crate::builtins::utils::{
    apply_numeric_operation, check_arity_range, expect_number, make_builtin_procedure,
};
use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;
use crate::value::Value;

/// Division helper function with zero-check
fn div_numbers(a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
    match (a, b) {
        (_, SchemeNumber::Integer(0)) => Err(LambdustError::division_by_zero()),
        (_, SchemeNumber::Real(f)) if *f == 0.0 => Err(LambdustError::division_by_zero()),
        (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
            if x % y == 0 {
                Ok(SchemeNumber::Integer(x / y))
            } else {
                Ok(SchemeNumber::Real(*x as f64 / *y as f64))
            }
        }
        _ => apply_numeric_operation(a, b, "divide", |x, y| x / y),
    }
}

/// Addition operation (+)
#[must_use] pub fn arithmetic_add() -> Value {
    make_builtin_procedure("+", None, |args| {
        let mut result = SchemeNumber::Integer(0);
        for arg in args {
            let num = expect_number(arg, "+")?;
            result = apply_numeric_operation(&result, num, "add", |x, y| x + y)?;
        }
        Ok(Value::Number(result))
    })
}

/// Subtraction operation (-)
#[must_use] pub fn arithmetic_sub() -> Value {
    make_builtin_procedure("-", None, |args| {
        check_arity_range(args, 1, None)?;

        let first_num = expect_number(&args[0], "-")?;
        if args.len() == 1 {
            // Unary minus (negation)
            match first_num {
                SchemeNumber::Integer(x) => Ok(Value::Number(SchemeNumber::Integer(-x))),
                SchemeNumber::Real(x) => Ok(Value::Number(SchemeNumber::Real(-x))),
                _ => Err(LambdustError::type_error("Cannot negate this number")),
            }
        } else {
            // Binary/n-ary subtraction
            let mut result = first_num.clone();
            for arg in &args[1..] {
                let num = expect_number(arg, "-")?;
                result = apply_numeric_operation(&result, num, "subtract", |x, y| x - y)?;
            }
            Ok(Value::Number(result))
        }
    })
}

/// Multiplication operation (*)
#[must_use] pub fn arithmetic_mul() -> Value {
    make_builtin_procedure("*", None, |args| {
        let mut result = SchemeNumber::Integer(1);
        for arg in args {
            let num = expect_number(arg, "*")?;
            result = apply_numeric_operation(&result, num, "multiply", |x, y| x * y)?;
        }
        Ok(Value::Number(result))
    })
}

/// Division operation (/)
#[must_use] pub fn arithmetic_div() -> Value {
    make_builtin_procedure("/", None, |args| {
        check_arity_range(args, 1, None)?;

        let first_num = expect_number(&args[0], "/")?;
        if args.len() == 1 {
            // Reciprocal (1/x)
            match first_num {
                SchemeNumber::Integer(0) => Err(LambdustError::division_by_zero()),
                SchemeNumber::Real(f) if *f == 0.0 => Err(LambdustError::division_by_zero()),
                SchemeNumber::Integer(x) => Ok(Value::Number(SchemeNumber::Real(1.0 / *x as f64))),
                SchemeNumber::Real(x) => Ok(Value::Number(SchemeNumber::Real(1.0 / x))),
                _ => Err(LambdustError::type_error(
                    "Cannot take reciprocal of this number",
                )),
            }
        } else {
            // Binary/n-ary division
            let mut result = first_num.clone();
            for arg in &args[1..] {
                let num = expect_number(arg, "/")?;
                result = div_numbers(&result, num)?;
            }
            Ok(Value::Number(result))
        }
    })
}

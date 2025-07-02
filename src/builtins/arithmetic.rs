//! Arithmetic operations for Scheme

use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register all arithmetic functions
pub fn register_arithmetic_functions(builtins: &mut HashMap<String, Value>) {
    // Basic arithmetic operations
    builtins.insert("+".to_string(), arithmetic_add());
    builtins.insert("-".to_string(), arithmetic_sub());
    builtins.insert("*".to_string(), arithmetic_mul());
    builtins.insert("/".to_string(), arithmetic_div());

    // Comparison operations
    builtins.insert("=".to_string(), arithmetic_eq());
    builtins.insert("<".to_string(), arithmetic_lt());
    builtins.insert("<=".to_string(), arithmetic_le());
    builtins.insert(">".to_string(), arithmetic_gt());
    builtins.insert(">=".to_string(), arithmetic_ge());

    // Extended numeric functions
    builtins.insert("abs".to_string(), numeric_abs());
    builtins.insert("quotient".to_string(), numeric_quotient());
    builtins.insert("remainder".to_string(), numeric_remainder());
    builtins.insert("modulo".to_string(), numeric_modulo());
    builtins.insert("gcd".to_string(), numeric_gcd());
    builtins.insert("lcm".to_string(), numeric_lcm());
    builtins.insert("floor".to_string(), numeric_floor());
    builtins.insert("ceiling".to_string(), numeric_ceiling());
    builtins.insert("truncate".to_string(), numeric_truncate());
    builtins.insert("round".to_string(), numeric_round());
    builtins.insert("sqrt".to_string(), numeric_sqrt());
    builtins.insert("expt".to_string(), numeric_expt());
    builtins.insert("min".to_string(), numeric_min());
    builtins.insert("max".to_string(), numeric_max());

    // Numeric predicates
    builtins.insert("odd?".to_string(), predicate_odd());
    builtins.insert("even?".to_string(), predicate_even());
    builtins.insert("zero?".to_string(), predicate_zero());
    builtins.insert("positive?".to_string(), predicate_positive());
    builtins.insert("negative?".to_string(), predicate_negative());
}

// Helper function for adding two SchemeNumbers
fn add_numbers(a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
    match (a, b) {
        (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => Ok(SchemeNumber::Integer(x + y)),
        (SchemeNumber::Real(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(x + y)),
        (SchemeNumber::Integer(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(*x as f64 + y)),
        (SchemeNumber::Real(x), SchemeNumber::Integer(y)) => Ok(SchemeNumber::Real(x + *y as f64)),
        _ => Err(LambdustError::type_error(format!(
            "Cannot add {a} and {b}"
        ))),
    }
}

// Helper function for subtracting two SchemeNumbers
fn sub_numbers(a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
    match (a, b) {
        (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => Ok(SchemeNumber::Integer(x - y)),
        (SchemeNumber::Real(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(x - y)),
        (SchemeNumber::Integer(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(*x as f64 - y)),
        (SchemeNumber::Real(x), SchemeNumber::Integer(y)) => Ok(SchemeNumber::Real(x - *y as f64)),
        _ => Err(LambdustError::type_error(format!(
            "Cannot subtract {} from {}",
            b, a
        ))),
    }
}

// Helper function for multiplying two SchemeNumbers
fn mul_numbers(a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
    match (a, b) {
        (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => Ok(SchemeNumber::Integer(x * y)),
        (SchemeNumber::Real(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(x * y)),
        (SchemeNumber::Integer(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(*x as f64 * y)),
        (SchemeNumber::Real(x), SchemeNumber::Integer(y)) => Ok(SchemeNumber::Real(x * *y as f64)),
        _ => Err(LambdustError::type_error(format!(
            "Cannot multiply {} and {}",
            a, b
        ))),
    }
}

// Helper function for dividing two SchemeNumbers
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
        (SchemeNumber::Real(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(x / y)),
        (SchemeNumber::Integer(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(*x as f64 / y)),
        (SchemeNumber::Real(x), SchemeNumber::Integer(y)) => Ok(SchemeNumber::Real(x / *y as f64)),
        _ => Err(LambdustError::type_error(format!(
            "Cannot divide {} by {}",
            a, b
        ))),
    }
}

// Basic arithmetic operations

fn arithmetic_add() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "+".to_string(),
        arity: None, // Variadic
        func: |args| {
            let mut result = SchemeNumber::Integer(0);
            for arg in args {
                if let Some(num) = arg.as_number() {
                    result = add_numbers(&result, num)?;
                } else {
                    return Err(LambdustError::type_error(format!(
                        "+: expected number, got {}",
                        arg
                    )));
                }
            }
            Ok(Value::Number(result))
        },
    })
}

fn arithmetic_sub() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "-".to_string(),
        arity: None, // At least 1 argument
        func: |args| {
            if args.is_empty() {
                return Err(LambdustError::arity_error(1, 0));
            }

            if let Some(first_num) = args[0].as_number() {
                if args.len() == 1 {
                    // Unary minus
                    match first_num {
                        SchemeNumber::Integer(x) => Ok(Value::Number(SchemeNumber::Integer(-x))),
                        SchemeNumber::Real(x) => Ok(Value::Number(SchemeNumber::Real(-x))),
                        _ => Err(LambdustError::type_error("Cannot negate this number")),
                    }
                } else {
                    // Binary minus
                    let mut result = first_num.clone();
                    for arg in &args[1..] {
                        if let Some(num) = arg.as_number() {
                            result = sub_numbers(&result, num)?;
                        } else {
                            return Err(LambdustError::type_error(format!(
                                "-: expected number, got {}",
                                arg
                            )));
                        }
                    }
                    Ok(Value::Number(result))
                }
            } else {
                Err(LambdustError::type_error(format!(
                    "-: expected number, got {}",
                    args[0]
                )))
            }
        },
    })
}

fn arithmetic_mul() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "*".to_string(),
        arity: None, // Variadic
        func: |args| {
            let mut result = SchemeNumber::Integer(1);
            for arg in args {
                if let Some(num) = arg.as_number() {
                    result = mul_numbers(&result, num)?;
                } else {
                    return Err(LambdustError::type_error(format!(
                        "*: expected number, got {}",
                        arg
                    )));
                }
            }
            Ok(Value::Number(result))
        },
    })
}

fn arithmetic_div() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "/".to_string(),
        arity: None, // At least 1 argument
        func: |args| {
            if args.is_empty() {
                return Err(LambdustError::arity_error(1, 0));
            }

            if let Some(first_num) = args[0].as_number() {
                if args.len() == 1 {
                    // Reciprocal
                    match first_num {
                        SchemeNumber::Integer(0) => Err(LambdustError::division_by_zero()),
                        SchemeNumber::Real(f) if *f == 0.0 => {
                            Err(LambdustError::division_by_zero())
                        }
                        SchemeNumber::Integer(x) => {
                            Ok(Value::Number(SchemeNumber::Real(1.0 / *x as f64)))
                        }
                        SchemeNumber::Real(x) => Ok(Value::Number(SchemeNumber::Real(1.0 / x))),
                        _ => Err(LambdustError::type_error(
                            "Cannot take reciprocal of this number",
                        )),
                    }
                } else {
                    // Division
                    let mut result = first_num.clone();
                    for arg in &args[1..] {
                        if let Some(num) = arg.as_number() {
                            result = div_numbers(&result, num)?;
                        } else {
                            return Err(LambdustError::type_error(format!(
                                "/: expected number, got {}",
                                arg
                            )));
                        }
                    }
                    Ok(Value::Number(result))
                }
            } else {
                Err(LambdustError::type_error(format!(
                    "/: expected number, got {}",
                    args[0]
                )))
            }
        },
    })
}

// Comparison operations

fn arithmetic_eq() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "=".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            for arg in args {
                if !arg.is_number() {
                    return Err(LambdustError::type_error(format!(
                        "=: expected number, got {}",
                        arg
                    )));
                }
            }

            let first = args[0].as_number().unwrap();
            for arg in &args[1..] {
                let num = arg.as_number().unwrap();
                if !numbers_equal(first, num) {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn arithmetic_lt() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "<".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            for i in 0..args.len() - 1 {
                let current = args[i].as_number().ok_or_else(|| {
                    LambdustError::type_error(format!("<: expected number, got {}", args[i]))
                })?;
                let next = args[i + 1].as_number().ok_or_else(|| {
                    LambdustError::type_error(format!("<: expected number, got {}", args[i + 1]))
                })?;

                if !number_less_than(current, next) {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn arithmetic_le() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "<=".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            for i in 0..args.len() - 1 {
                let current = args[i].as_number().ok_or_else(|| {
                    LambdustError::type_error(format!("<=: expected number, got {}", args[i]))
                })?;
                let next = args[i + 1].as_number().ok_or_else(|| {
                    LambdustError::type_error(format!("<=: expected number, got {}", args[i + 1]))
                })?;

                if number_less_than(next, current) {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn arithmetic_gt() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: ">".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            for i in 0..args.len() - 1 {
                let current = args[i].as_number().ok_or_else(|| {
                    LambdustError::type_error(format!(">: expected number, got {}", args[i]))
                })?;
                let next = args[i + 1].as_number().ok_or_else(|| {
                    LambdustError::type_error(format!(">: expected number, got {}", args[i + 1]))
                })?;

                if !number_less_than(next, current) {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn arithmetic_ge() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: ">=".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            for i in 0..args.len() - 1 {
                let current = args[i].as_number().ok_or_else(|| {
                    LambdustError::type_error(format!(">=: expected number, got {}", args[i]))
                })?;
                let next = args[i + 1].as_number().ok_or_else(|| {
                    LambdustError::type_error(format!(">=: expected number, got {}", args[i + 1]))
                })?;

                if number_less_than(current, next) {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

// Extended numeric functions

fn numeric_abs() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "abs".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => Ok(Value::Number(SchemeNumber::Integer(n.abs()))),
                Some(SchemeNumber::Real(n)) => Ok(Value::Number(SchemeNumber::Real(n.abs()))),
                _ => Err(LambdustError::type_error(format!(
                    "abs: expected number, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn numeric_quotient() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "quotient".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            let a = args[0].as_number().ok_or_else(|| {
                LambdustError::type_error(format!("quotient: expected number, got {}", args[0]))
            })?;
            let b = args[1].as_number().ok_or_else(|| {
                LambdustError::type_error(format!("quotient: expected number, got {}", args[1]))
            })?;

            match (a, b) {
                (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                    if *y == 0 {
                        Err(LambdustError::division_by_zero())
                    } else {
                        Ok(Value::Number(SchemeNumber::Integer(x / y)))
                    }
                }
                _ => Err(LambdustError::type_error("quotient: expected integers")),
            }
        },
    })
}

fn numeric_remainder() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "remainder".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            let a = args[0].as_number().ok_or_else(|| {
                LambdustError::type_error(format!("remainder: expected number, got {}", args[0]))
            })?;
            let b = args[1].as_number().ok_or_else(|| {
                LambdustError::type_error(format!("remainder: expected number, got {}", args[1]))
            })?;

            match (a, b) {
                (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                    if *y == 0 {
                        Err(LambdustError::division_by_zero())
                    } else {
                        Ok(Value::Number(SchemeNumber::Integer(x % y)))
                    }
                }
                _ => Err(LambdustError::type_error("remainder: expected integers")),
            }
        },
    })
}

fn numeric_modulo() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "modulo".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            let a = args[0].as_number().ok_or_else(|| {
                LambdustError::type_error(format!("modulo: expected number, got {}", args[0]))
            })?;
            let b = args[1].as_number().ok_or_else(|| {
                LambdustError::type_error(format!("modulo: expected number, got {}", args[1]))
            })?;

            match (a, b) {
                (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                    if *y == 0 {
                        Err(LambdustError::division_by_zero())
                    } else {
                        // Scheme modulo: result has same sign as divisor
                        let result = ((x % y) + y) % y;
                        Ok(Value::Number(SchemeNumber::Integer(result)))
                    }
                }
                _ => Err(LambdustError::type_error("modulo: expected integers")),
            }
        },
    })
}

fn numeric_gcd() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "gcd".to_string(),
        arity: None, // Variadic
        func: |args| {
            if args.is_empty() {
                return Ok(Value::Number(SchemeNumber::Integer(0)));
            }

            let mut result = 0i64;
            for arg in args {
                match arg.as_number() {
                    Some(SchemeNumber::Integer(n)) => {
                        result = gcd_helper(result, *n);
                    }
                    _ => {
                        return Err(LambdustError::type_error(format!(
                            "gcd: expected integer, got {}",
                            arg
                        )));
                    }
                }
            }
            Ok(Value::Number(SchemeNumber::Integer(result)))
        },
    })
}

fn numeric_lcm() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "lcm".to_string(),
        arity: None, // Variadic
        func: |args| {
            if args.is_empty() {
                return Ok(Value::Number(SchemeNumber::Integer(1)));
            }

            let mut result = 1i64;
            for arg in args {
                match arg.as_number() {
                    Some(SchemeNumber::Integer(n)) => {
                        if *n == 0 {
                            return Ok(Value::Number(SchemeNumber::Integer(0)));
                        }
                        result = lcm_helper(result, *n);
                    }
                    _ => {
                        return Err(LambdustError::type_error(format!(
                            "lcm: expected integer, got {}",
                            arg
                        )));
                    }
                }
            }
            Ok(Value::Number(SchemeNumber::Integer(result)))
        },
    })
}

fn numeric_floor() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "floor".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => Ok(Value::Number(SchemeNumber::Integer(*n))),
                Some(SchemeNumber::Real(n)) => {
                    Ok(Value::Number(SchemeNumber::Integer(n.floor() as i64)))
                }
                _ => Err(LambdustError::type_error(format!(
                    "floor: expected number, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn numeric_ceiling() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "ceiling".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => Ok(Value::Number(SchemeNumber::Integer(*n))),
                Some(SchemeNumber::Real(n)) => {
                    Ok(Value::Number(SchemeNumber::Integer(n.ceil() as i64)))
                }
                _ => Err(LambdustError::type_error(format!(
                    "ceiling: expected number, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn numeric_truncate() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "truncate".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => Ok(Value::Number(SchemeNumber::Integer(*n))),
                Some(SchemeNumber::Real(n)) => {
                    Ok(Value::Number(SchemeNumber::Integer(n.trunc() as i64)))
                }
                _ => Err(LambdustError::type_error(format!(
                    "truncate: expected number, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn numeric_round() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "round".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => Ok(Value::Number(SchemeNumber::Integer(*n))),
                Some(SchemeNumber::Real(n)) => {
                    Ok(Value::Number(SchemeNumber::Integer(n.round() as i64)))
                }
                _ => Err(LambdustError::type_error(format!(
                    "round: expected number, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn numeric_sqrt() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "sqrt".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => {
                    if *n < 0 {
                        Err(LambdustError::runtime_error("sqrt: domain error"))
                    } else {
                        let result = (*n as f64).sqrt();
                        if result.fract() == 0.0 {
                            Ok(Value::Number(SchemeNumber::Integer(result as i64)))
                        } else {
                            Ok(Value::Number(SchemeNumber::Real(result)))
                        }
                    }
                }
                Some(SchemeNumber::Real(n)) => {
                    if *n < 0.0 {
                        Err(LambdustError::runtime_error("sqrt: domain error"))
                    } else {
                        Ok(Value::Number(SchemeNumber::Real(n.sqrt())))
                    }
                }
                _ => Err(LambdustError::type_error(format!(
                    "sqrt: expected number, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn numeric_expt() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "expt".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            let base = args[0].as_number().ok_or_else(|| {
                LambdustError::type_error(format!("expt: expected number, got {}", args[0]))
            })?;
            let exp = args[1].as_number().ok_or_else(|| {
                LambdustError::type_error(format!("expt: expected number, got {}", args[1]))
            })?;

            match (base, exp) {
                (SchemeNumber::Integer(b), SchemeNumber::Integer(e)) => {
                    if *e >= 0 {
                        let result = (*b as f64).powi(*e as i32);
                        if result.fract() == 0.0 && result.is_finite() {
                            Ok(Value::Number(SchemeNumber::Integer(result as i64)))
                        } else {
                            Ok(Value::Number(SchemeNumber::Real(result)))
                        }
                    } else {
                        let result = (*b as f64).powf(*e as f64);
                        Ok(Value::Number(SchemeNumber::Real(result)))
                    }
                }
                _ => {
                    let b_f = match base {
                        SchemeNumber::Integer(n) => *n as f64,
                        SchemeNumber::Real(n) => *n,
                        _ => unreachable!(),
                    };
                    let e_f = match exp {
                        SchemeNumber::Integer(n) => *n as f64,
                        SchemeNumber::Real(n) => *n,
                        _ => unreachable!(),
                    };
                    let result = b_f.powf(e_f);
                    Ok(Value::Number(SchemeNumber::Real(result)))
                }
            }
        },
    })
}

fn numeric_min() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "min".to_string(),
        arity: None, // At least 1 argument
        func: |args| {
            if args.is_empty() {
                return Err(LambdustError::arity_error(1, 0));
            }

            let mut min_val = args[0]
                .as_number()
                .ok_or_else(|| {
                    LambdustError::type_error(format!("min: expected number, got {}", args[0]))
                })?
                .clone();

            for arg in &args[1..] {
                let num = arg.as_number().ok_or_else(|| {
                    LambdustError::type_error(format!("min: expected number, got {}", arg))
                })?;

                if number_less_than(num, &min_val) {
                    min_val = num.clone();
                }
            }

            Ok(Value::Number(min_val))
        },
    })
}

fn numeric_max() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "max".to_string(),
        arity: None, // At least 1 argument
        func: |args| {
            if args.is_empty() {
                return Err(LambdustError::arity_error(1, 0));
            }

            let mut max_val = args[0]
                .as_number()
                .ok_or_else(|| {
                    LambdustError::type_error(format!("max: expected number, got {}", args[0]))
                })?
                .clone();

            for arg in &args[1..] {
                let num = arg.as_number().ok_or_else(|| {
                    LambdustError::type_error(format!("max: expected number, got {}", arg))
                })?;

                if number_less_than(&max_val, num) {
                    max_val = num.clone();
                }
            }

            Ok(Value::Number(max_val))
        },
    })
}

// Numeric predicates

fn predicate_odd() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "odd?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => Ok(Value::Boolean(n % 2 != 0)),
                _ => Err(LambdustError::type_error(format!(
                    "odd?: expected integer, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn predicate_even() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "even?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => Ok(Value::Boolean(n % 2 == 0)),
                _ => Err(LambdustError::type_error(format!(
                    "even?: expected integer, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn predicate_zero() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "zero?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => Ok(Value::Boolean(*n == 0)),
                Some(SchemeNumber::Real(n)) => Ok(Value::Boolean(*n == 0.0)),
                _ => Err(LambdustError::type_error(format!(
                    "zero?: expected number, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn predicate_positive() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "positive?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => Ok(Value::Boolean(*n > 0)),
                Some(SchemeNumber::Real(n)) => Ok(Value::Boolean(*n > 0.0)),
                _ => Err(LambdustError::type_error(format!(
                    "positive?: expected number, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn predicate_negative() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "negative?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => Ok(Value::Boolean(*n < 0)),
                Some(SchemeNumber::Real(n)) => Ok(Value::Boolean(*n < 0.0)),
                _ => Err(LambdustError::type_error(format!(
                    "negative?: expected number, got {}",
                    args[0]
                ))),
            }
        },
    })
}

// Helper functions

fn numbers_equal(a: &SchemeNumber, b: &SchemeNumber) -> bool {
    match (a, b) {
        (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => x == y,
        (SchemeNumber::Real(x), SchemeNumber::Real(y)) => x == y,
        (SchemeNumber::Integer(x), SchemeNumber::Real(y)) => *x as f64 == *y,
        (SchemeNumber::Real(x), SchemeNumber::Integer(y)) => *x == *y as f64,
        _ => false,
    }
}

fn number_less_than(a: &SchemeNumber, b: &SchemeNumber) -> bool {
    match (a, b) {
        (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => x < y,
        (SchemeNumber::Real(x), SchemeNumber::Real(y)) => x < y,
        (SchemeNumber::Integer(x), SchemeNumber::Real(y)) => (*x as f64) < *y,
        (SchemeNumber::Real(x), SchemeNumber::Integer(y)) => *x < (*y as f64),
        _ => false,
    }
}

fn gcd_helper(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

fn lcm_helper(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a.abs() / gcd_helper(a, b)) * b.abs()
    }
}

//! Arithmetic operations for Scheme

use crate::builtins::utils::{
    apply_numeric_operation, check_arity, check_arity_range, compare_numbers, expect_number,
    is_even, is_negative, is_odd, is_positive, is_zero, make_builtin_procedure,
};
use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;
use crate::make_predicate;
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
    builtins.insert("odd?".to_string(), make_predicate!("odd?", is_odd));
    builtins.insert("even?".to_string(), make_predicate!("even?", is_even));
    builtins.insert("zero?".to_string(), make_predicate!("zero?", is_zero));
    builtins.insert(
        "positive?".to_string(),
        make_predicate!("positive?", is_positive),
    );
    builtins.insert(
        "negative?".to_string(),
        make_predicate!("negative?", is_negative),
    );

    // SRFI 141: Integer Division
    // Floor division family
    builtins.insert("floor-quotient".to_string(), srfi_141_floor_quotient());
    builtins.insert("floor-remainder".to_string(), srfi_141_floor_remainder());
    builtins.insert("floor/".to_string(), srfi_141_floor_div());

    // Ceiling division family
    builtins.insert("ceiling-quotient".to_string(), srfi_141_ceiling_quotient());
    builtins.insert(
        "ceiling-remainder".to_string(),
        srfi_141_ceiling_remainder(),
    );
    builtins.insert("ceiling/".to_string(), srfi_141_ceiling_div());

    // Truncate division family
    builtins.insert(
        "truncate-quotient".to_string(),
        srfi_141_truncate_quotient(),
    );
    builtins.insert(
        "truncate-remainder".to_string(),
        srfi_141_truncate_remainder(),
    );
    builtins.insert("truncate/".to_string(), srfi_141_truncate_div());

    // Round division family
    builtins.insert("round-quotient".to_string(), srfi_141_round_quotient());
    builtins.insert("round-remainder".to_string(), srfi_141_round_remainder());
    builtins.insert("round/".to_string(), srfi_141_round_div());

    // Euclidean division family
    builtins.insert(
        "euclidean-quotient".to_string(),
        srfi_141_euclidean_quotient(),
    );
    builtins.insert(
        "euclidean-remainder".to_string(),
        srfi_141_euclidean_remainder(),
    );
    builtins.insert("euclidean/".to_string(), srfi_141_euclidean_div());

    // Balanced division family
    builtins.insert(
        "balanced-quotient".to_string(),
        srfi_141_balanced_quotient(),
    );
    builtins.insert(
        "balanced-remainder".to_string(),
        srfi_141_balanced_remainder(),
    );
    builtins.insert("balanced/".to_string(), srfi_141_balanced_div());
}

// Helper function for handling division with proper integer results
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

// Basic arithmetic operations

fn arithmetic_add() -> Value {
    make_builtin_procedure("+", None, |args| {
        let mut result = SchemeNumber::Integer(0);
        for arg in args {
            let num = expect_number(arg, "+")?;
            result = apply_numeric_operation(&result, num, "add", |x, y| x + y)?;
        }
        Ok(Value::Number(result))
    })
}

fn arithmetic_sub() -> Value {
    make_builtin_procedure("-", None, |args| {
        check_arity_range(args, 1, None)?;

        let first_num = expect_number(&args[0], "-")?;
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
                let num = expect_number(arg, "-")?;
                result = apply_numeric_operation(&result, num, "subtract", |x, y| x - y)?;
            }
            Ok(Value::Number(result))
        }
    })
}

fn arithmetic_mul() -> Value {
    make_builtin_procedure("*", None, |args| {
        let mut result = SchemeNumber::Integer(1);
        for arg in args {
            let num = expect_number(arg, "*")?;
            result = apply_numeric_operation(&result, num, "multiply", |x, y| x * y)?;
        }
        Ok(Value::Number(result))
    })
}

fn arithmetic_div() -> Value {
    make_builtin_procedure("/", None, |args| {
        check_arity_range(args, 1, None)?;

        let first_num = expect_number(&args[0], "/")?;
        if args.len() == 1 {
            // Reciprocal
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
            // Division
            let mut result = first_num.clone();
            for arg in &args[1..] {
                let num = expect_number(arg, "/")?;
                result = div_numbers(&result, num)?;
            }
            Ok(Value::Number(result))
        }
    })
}

// Comparison operations

fn arithmetic_eq() -> Value {
    make_builtin_procedure("=", None, |args| {
        check_arity_range(args, 2, None)?;

        let first = expect_number(&args[0], "=")?;
        for arg in &args[1..] {
            let num = expect_number(arg, "=")?;
            if !compare_numbers(first, num, |x, y| (x - y).abs() < f64::EPSILON) {
                return Ok(Value::Boolean(false));
            }
        }
        Ok(Value::Boolean(true))
    })
}

fn arithmetic_lt() -> Value {
    make_builtin_procedure("<", None, |args| {
        check_arity_range(args, 2, None)?;

        for i in 0..args.len() - 1 {
            let current = expect_number(&args[i], "<")?;
            let next = expect_number(&args[i + 1], "<")?;

            if !compare_numbers(current, next, |x, y| x < y) {
                return Ok(Value::Boolean(false));
            }
        }
        Ok(Value::Boolean(true))
    })
}

fn arithmetic_le() -> Value {
    make_builtin_procedure("<=", None, |args| {
        check_arity_range(args, 2, None)?;

        for i in 0..args.len() - 1 {
            let current = expect_number(&args[i], "<=")?;
            let next = expect_number(&args[i + 1], "<=")?;

            if !compare_numbers(current, next, |x, y| x <= y) {
                return Ok(Value::Boolean(false));
            }
        }
        Ok(Value::Boolean(true))
    })
}

fn arithmetic_gt() -> Value {
    make_builtin_procedure(">", None, |args| {
        check_arity_range(args, 2, None)?;

        for i in 0..args.len() - 1 {
            let current = expect_number(&args[i], ">")?;
            let next = expect_number(&args[i + 1], ">")?;

            if !compare_numbers(current, next, |x, y| x > y) {
                return Ok(Value::Boolean(false));
            }
        }
        Ok(Value::Boolean(true))
    })
}

fn arithmetic_ge() -> Value {
    make_builtin_procedure(">=", None, |args| {
        check_arity_range(args, 2, None)?;

        for i in 0..args.len() - 1 {
            let current = expect_number(&args[i], ">=")?;
            let next = expect_number(&args[i + 1], ">=")?;

            if !compare_numbers(current, next, |x, y| x >= y) {
                return Ok(Value::Boolean(false));
            }
        }
        Ok(Value::Boolean(true))
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

// Numeric predicates are now implemented using the make_predicate! macro
// for consistency and reduced code duplication

// Helper functions

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

// SRFI 141: Integer Division implementations
// Provides comprehensive integer division operations with six families:
// floor, ceiling, truncate, round, Euclidean, and balanced division

/// Helper function to perform floor division
fn floor_division(x: i64, y: i64) -> (i64, i64) {
    if y == 0 {
        panic!("Division by zero");
    }

    let q = x / y;
    let r = x % y;

    if (r != 0) && ((r < 0) != (y < 0)) {
        (q - 1, r + y)
    } else {
        (q, r)
    }
}

/// Helper function to perform ceiling division
fn ceiling_division(x: i64, y: i64) -> (i64, i64) {
    let (q, r) = floor_division(x, y);
    if r == 0 {
        (q, r)
    } else {
        (q + 1, r - y)
    }
}

/// Helper function to perform truncate division (same as built-in / and %)
fn truncate_division(x: i64, y: i64) -> (i64, i64) {
    if y == 0 {
        panic!("Division by zero");
    }
    (x / y, x % y)
}

/// Helper function to perform round division
fn round_division(x: i64, y: i64) -> (i64, i64) {
    if y == 0 {
        panic!("Division by zero");
    }

    let (q, r) = floor_division(x, y);

    if r.abs() < (y.abs() + 1) / 2 {
        (q, r)
    } else if r.abs() > (y.abs() + 1) / 2 {
        if r > 0 {
            (q + 1, r - y)
        } else {
            (q - 1, r + y)
        }
    } else {
        // Tie case: round to even
        if q % 2 == 0 {
            (q, r)
        } else if r > 0 {
            (q + 1, r - y)
        } else {
            (q - 1, r + y)
        }
    }
}

/// Helper function to perform Euclidean division
fn euclidean_division(x: i64, y: i64) -> (i64, i64) {
    if y == 0 {
        panic!("Division by zero");
    }

    let (q, r) = floor_division(x, y);

    if r >= 0 {
        (q, r)
    } else if y > 0 {
        (q - 1, r + y)
    } else {
        (q + 1, r - y)
    }
}

/// Helper function to perform balanced division
fn balanced_division(x: i64, y: i64) -> (i64, i64) {
    if y == 0 {
        panic!("Division by zero");
    }

    let (q, r) = euclidean_division(x, y);
    let half_y = y.abs() / 2;

    if r <= half_y {
        (q, r)
    } else if y > 0 {
        (q + 1, r - y)
    } else {
        (q - 1, r + y)
    }
}

// Floor division family
fn srfi_141_floor_quotient() -> Value {
    make_builtin_procedure("floor-quotient", Some(2), |args| {
        check_arity(args, 2)?;
        let x = expect_number(&args[0], "floor-quotient")?;
        let y = expect_number(&args[1], "floor-quotient")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, _) = floor_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(q)))
            }
            _ => Err(LambdustError::type_error(
                "floor-quotient requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_floor_remainder() -> Value {
    make_builtin_procedure("floor-remainder", Some(2), |args| {
        check_arity(args, 2)?;
        let x = expect_number(&args[0], "floor-remainder")?;
        let y = expect_number(&args[1], "floor-remainder")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (_, r) = floor_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(r)))
            }
            _ => Err(LambdustError::type_error(
                "floor-remainder requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_floor_div() -> Value {
    make_builtin_procedure("floor/", Some(2), |args| {
        check_arity(args, 2)?;
        let x = expect_number(&args[0], "floor/")?;
        let y = expect_number(&args[1], "floor/")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, r) = floor_division(*x, *y);
                Ok(Value::Values(vec![
                    Value::Number(SchemeNumber::Integer(q)),
                    Value::Number(SchemeNumber::Integer(r)),
                ]))
            }
            _ => Err(LambdustError::type_error(
                "floor/ requires integer arguments".to_string(),
            )),
        }
    })
}

// Ceiling division family
fn srfi_141_ceiling_quotient() -> Value {
    make_builtin_procedure("ceiling-quotient", Some(2), |args| {
        check_arity(args, 2)?;
        let x = expect_number(&args[0], "ceiling-quotient")?;
        let y = expect_number(&args[1], "ceiling-quotient")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, _) = ceiling_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(q)))
            }
            _ => Err(LambdustError::type_error(
                "ceiling-quotient requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_ceiling_remainder() -> Value {
    make_builtin_procedure("ceiling-remainder", Some(2), |args| {
        check_arity(args, 2)?;
        let x = expect_number(&args[0], "ceiling-remainder")?;
        let y = expect_number(&args[1], "ceiling-remainder")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (_, r) = ceiling_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(r)))
            }
            _ => Err(LambdustError::type_error(
                "ceiling-remainder requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_ceiling_div() -> Value {
    make_builtin_procedure("ceiling/", Some(2), |args| {
        check_arity(args, 2)?;
        let x = expect_number(&args[0], "ceiling/")?;
        let y = expect_number(&args[1], "ceiling/")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, r) = ceiling_division(*x, *y);
                Ok(Value::Values(vec![
                    Value::Number(SchemeNumber::Integer(q)),
                    Value::Number(SchemeNumber::Integer(r)),
                ]))
            }
            _ => Err(LambdustError::type_error(
                "ceiling/ requires integer arguments".to_string(),
            )),
        }
    })
}

// Truncate division family
fn srfi_141_truncate_quotient() -> Value {
    make_builtin_procedure("truncate-quotient", Some(2), |args| {
        check_arity(args, 2)?;
        let x = expect_number(&args[0], "truncate-quotient")?;
        let y = expect_number(&args[1], "truncate-quotient")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, _) = truncate_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(q)))
            }
            _ => Err(LambdustError::type_error(
                "truncate-quotient requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_truncate_remainder() -> Value {
    make_builtin_procedure("truncate-remainder", Some(2), |args| {
        let x = expect_number(&args[0], "truncate-remainder")?;
        let y = expect_number(&args[1], "truncate-remainder")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (_, r) = truncate_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(r)))
            }
            _ => Err(LambdustError::type_error(
                "truncate-remainder requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_truncate_div() -> Value {
    make_builtin_procedure("truncate/", Some(2), |args| {
        let x = expect_number(&args[0], "truncate/")?;
        let y = expect_number(&args[1], "truncate/")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, r) = truncate_division(*x, *y);
                Ok(Value::Values(vec![
                    Value::Number(SchemeNumber::Integer(q)),
                    Value::Number(SchemeNumber::Integer(r)),
                ]))
            }
            _ => Err(LambdustError::type_error(
                "truncate/ requires integer arguments".to_string(),
            )),
        }
    })
}

// Round division family
fn srfi_141_round_quotient() -> Value {
    make_builtin_procedure("round-quotient", Some(2), |args| {
        let x = expect_number(&args[0], "round-quotient")?;
        let y = expect_number(&args[1], "round-quotient")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, _) = round_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(q)))
            }
            _ => Err(LambdustError::type_error(
                "round-quotient requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_round_remainder() -> Value {
    make_builtin_procedure("round-remainder", Some(2), |args| {
        let x = expect_number(&args[0], "round-remainder")?;
        let y = expect_number(&args[1], "round-remainder")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (_, r) = round_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(r)))
            }
            _ => Err(LambdustError::type_error(
                "round-remainder requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_round_div() -> Value {
    make_builtin_procedure("round/", Some(2), |args| {
        let x = expect_number(&args[0], "round/")?;
        let y = expect_number(&args[1], "round/")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, r) = round_division(*x, *y);
                Ok(Value::Values(vec![
                    Value::Number(SchemeNumber::Integer(q)),
                    Value::Number(SchemeNumber::Integer(r)),
                ]))
            }
            _ => Err(LambdustError::type_error(
                "round/ requires integer arguments".to_string(),
            )),
        }
    })
}

// Euclidean division family
fn srfi_141_euclidean_quotient() -> Value {
    make_builtin_procedure("euclidean-quotient", Some(2), |args| {
        let x = expect_number(&args[0], "euclidean-quotient")?;
        let y = expect_number(&args[1], "euclidean-quotient")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, _) = euclidean_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(q)))
            }
            _ => Err(LambdustError::type_error(
                "euclidean-quotient requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_euclidean_remainder() -> Value {
    make_builtin_procedure("euclidean-remainder", Some(2), |args| {
        let x = expect_number(&args[0], "euclidean-remainder")?;
        let y = expect_number(&args[1], "euclidean-remainder")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (_, r) = euclidean_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(r)))
            }
            _ => Err(LambdustError::type_error(
                "euclidean-remainder requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_euclidean_div() -> Value {
    make_builtin_procedure("euclidean/", Some(2), |args| {
        let x = expect_number(&args[0], "euclidean/")?;
        let y = expect_number(&args[1], "euclidean/")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, r) = euclidean_division(*x, *y);
                Ok(Value::Values(vec![
                    Value::Number(SchemeNumber::Integer(q)),
                    Value::Number(SchemeNumber::Integer(r)),
                ]))
            }
            _ => Err(LambdustError::type_error(
                "euclidean/ requires integer arguments".to_string(),
            )),
        }
    })
}

// Balanced division family
fn srfi_141_balanced_quotient() -> Value {
    make_builtin_procedure("balanced-quotient", Some(2), |args| {
        let x = expect_number(&args[0], "balanced-quotient")?;
        let y = expect_number(&args[1], "balanced-quotient")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, _) = balanced_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(q)))
            }
            _ => Err(LambdustError::type_error(
                "balanced-quotient requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_balanced_remainder() -> Value {
    make_builtin_procedure("balanced-remainder", Some(2), |args| {
        let x = expect_number(&args[0], "balanced-remainder")?;
        let y = expect_number(&args[1], "balanced-remainder")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (_, r) = balanced_division(*x, *y);
                Ok(Value::Number(SchemeNumber::Integer(r)))
            }
            _ => Err(LambdustError::type_error(
                "balanced-remainder requires integer arguments".to_string(),
            )),
        }
    })
}

fn srfi_141_balanced_div() -> Value {
    make_builtin_procedure("balanced/", Some(2), |args| {
        let x = expect_number(&args[0], "balanced/")?;
        let y = expect_number(&args[1], "balanced/")?;

        match (x, y) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    return Err(LambdustError::division_by_zero());
                }
                let (q, r) = balanced_division(*x, *y);
                Ok(Value::Values(vec![
                    Value::Number(SchemeNumber::Integer(q)),
                    Value::Number(SchemeNumber::Integer(r)),
                ]))
            }
            _ => Err(LambdustError::type_error(
                "balanced/ requires integer arguments".to_string(),
            )),
        }
    })
}

//! Arithmetic Operations Module
//!
//! このモジュールはScheme算術演算の包括的な実装を提供します。
//! 基本演算、比較演算、拡張関数、数値述語を含みます。
//!
//! ## モジュール構成
//!
//! - `basic_operations`: 基本演算（+, -, *, /）

pub mod basic_operations;

// Re-export main functions for backward compatibility
pub use basic_operations::{
    arithmetic_add, arithmetic_sub, arithmetic_mul, arithmetic_div,
};

use crate::builtins::utils::{
    check_arity, check_arity_range, expect_number,
    is_even, is_negative, is_odd, is_positive, is_zero, make_builtin_procedure,
    compare_numbers_ordering,
};
use crate::LambdustError;
use crate::lexer::SchemeNumber;
use crate::make_predicate;
use crate::value::Value;
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
    builtins.insert("positive?".to_string(), make_predicate!("positive?", is_positive));
    builtins.insert("negative?".to_string(), make_predicate!("negative?", is_negative));
}

// Comparison operations (simplified implementations)
fn arithmetic_eq() -> Value {
    make_builtin_procedure("=", None, |args| {
        check_arity_range(args, 2, None)?;
        for i in 1..args.len() {
            let a = expect_number(&args[i - 1], "=")?;
            let b = expect_number(&args[i], "=")?;
            if compare_numbers_ordering(a, b) != std::cmp::Ordering::Equal {
                return Ok(Value::Boolean(false));
            }
        }
        Ok(Value::Boolean(true))
    })
}

fn arithmetic_lt() -> Value {
    make_builtin_procedure("<", None, |args| {
        check_arity_range(args, 2, None)?;
        for i in 1..args.len() {
            let a = expect_number(&args[i - 1], "<")?;
            let b = expect_number(&args[i], "<")?;
            if compare_numbers_ordering(a, b) != std::cmp::Ordering::Less {
                return Ok(Value::Boolean(false));
            }
        }
        Ok(Value::Boolean(true))
    })
}

fn arithmetic_le() -> Value {
    make_builtin_procedure("<=", None, |args| {
        check_arity_range(args, 2, None)?;
        for i in 1..args.len() {
            let a = expect_number(&args[i - 1], "<=")?;
            let b = expect_number(&args[i], "<=")?;
            match compare_numbers_ordering(a, b) {
                std::cmp::Ordering::Less | std::cmp::Ordering::Equal => continue,
                std::cmp::Ordering::Greater => return Ok(Value::Boolean(false)),
            }
        }
        Ok(Value::Boolean(true))
    })
}

fn arithmetic_gt() -> Value {
    make_builtin_procedure(">", None, |args| {
        check_arity_range(args, 2, None)?;
        for i in 1..args.len() {
            let a = expect_number(&args[i - 1], ">")?;
            let b = expect_number(&args[i], ">")?;
            if compare_numbers_ordering(a, b) != std::cmp::Ordering::Greater {
                return Ok(Value::Boolean(false));
            }
        }
        Ok(Value::Boolean(true))
    })
}

fn arithmetic_ge() -> Value {
    make_builtin_procedure(">=", None, |args| {
        check_arity_range(args, 2, None)?;
        for i in 1..args.len() {
            let a = expect_number(&args[i - 1], ">=")?;
            let b = expect_number(&args[i], ">=")?;
            match compare_numbers_ordering(a, b) {
                std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => continue,
                std::cmp::Ordering::Less => return Ok(Value::Boolean(false)),
            }
        }
        Ok(Value::Boolean(true))
    })
}

// Extended numeric functions (simplified implementations)
fn numeric_abs() -> Value {
    make_builtin_procedure("abs", Some(1), |args| {
        check_arity(args, 1)?;
        let num = expect_number(&args[0], "abs")?;
        match num {
            SchemeNumber::Integer(x) => Ok(Value::Number(SchemeNumber::Integer(x.abs()))),
            SchemeNumber::Real(x) => Ok(Value::Number(SchemeNumber::Real(x.abs()))),
            _ => Err(LambdustError::type_error("abs requires a number")),
        }
    })
}

fn numeric_quotient() -> Value {
    make_builtin_procedure("quotient", Some(2), |args| {
        check_arity(args, 2)?;
        let a = expect_number(&args[0], "quotient")?;
        let b = expect_number(&args[1], "quotient")?;
        
        match (a, b) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    Err(LambdustError::division_by_zero())
                } else {
                    Ok(Value::Number(SchemeNumber::Integer(x / y)))
                }
            }
            _ => Err(LambdustError::type_error("quotient requires integers")),
        }
    })
}

fn numeric_remainder() -> Value {
    make_builtin_procedure("remainder", Some(2), |args| {
        check_arity(args, 2)?;
        let a = expect_number(&args[0], "remainder")?;
        let b = expect_number(&args[1], "remainder")?;
        
        match (a, b) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    Err(LambdustError::division_by_zero())
                } else {
                    Ok(Value::Number(SchemeNumber::Integer(x % y)))
                }
            }
            _ => Err(LambdustError::type_error("remainder requires integers")),
        }
    })
}

fn numeric_modulo() -> Value {
    make_builtin_procedure("modulo", Some(2), |args| {
        check_arity(args, 2)?;
        let a = expect_number(&args[0], "modulo")?;
        let b = expect_number(&args[1], "modulo")?;
        
        match (a, b) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if *y == 0 {
                    Err(LambdustError::division_by_zero())
                } else {
                    let result = ((x % y) + y) % y;
                    Ok(Value::Number(SchemeNumber::Integer(result)))
                }
            }
            _ => Err(LambdustError::type_error("modulo requires integers")),
        }
    })
}

fn numeric_gcd() -> Value {
    make_builtin_procedure("gcd", None, |args| {
        if args.is_empty() {
            return Ok(Value::Number(SchemeNumber::Integer(0)));
        }
        
        let mut result = 0i64;
        for arg in args {
            let num = expect_number(arg, "gcd")?;
            if let SchemeNumber::Integer(x) = num {
                result = gcd_helper(result, *x);
            } else {
                return Err(LambdustError::type_error("gcd requires integers"));
            }
        }
        Ok(Value::Number(SchemeNumber::Integer(result)))
    })
}

fn numeric_lcm() -> Value {
    make_builtin_procedure("lcm", None, |args| {
        if args.is_empty() {
            return Ok(Value::Number(SchemeNumber::Integer(1)));
        }
        
        let mut result = 1i64;
        for arg in args {
            let num = expect_number(arg, "lcm")?;
            if let SchemeNumber::Integer(x) = num {
                result = lcm_helper(result, *x);
            } else {
                return Err(LambdustError::type_error("lcm requires integers"));
            }
        }
        Ok(Value::Number(SchemeNumber::Integer(result)))
    })
}

fn numeric_floor() -> Value {
    make_builtin_procedure("floor", Some(1), |args| {
        check_arity(args, 1)?;
        let num = expect_number(&args[0], "floor")?;
        match num {
            SchemeNumber::Integer(x) => Ok(Value::Number(SchemeNumber::Integer(*x))),
            SchemeNumber::Real(x) => Ok(Value::Number(SchemeNumber::Integer(x.floor() as i64))),
            _ => Err(LambdustError::type_error("floor requires a number")),
        }
    })
}

fn numeric_ceiling() -> Value {
    make_builtin_procedure("ceiling", Some(1), |args| {
        check_arity(args, 1)?;
        let num = expect_number(&args[0], "ceiling")?;
        match num {
            SchemeNumber::Integer(x) => Ok(Value::Number(SchemeNumber::Integer(*x))),
            SchemeNumber::Real(x) => Ok(Value::Number(SchemeNumber::Integer(x.ceil() as i64))),
            _ => Err(LambdustError::type_error("ceiling requires a number")),
        }
    })
}

fn numeric_truncate() -> Value {
    make_builtin_procedure("truncate", Some(1), |args| {
        check_arity(args, 1)?;
        let num = expect_number(&args[0], "truncate")?;
        match num {
            SchemeNumber::Integer(x) => Ok(Value::Number(SchemeNumber::Integer(*x))),
            SchemeNumber::Real(x) => Ok(Value::Number(SchemeNumber::Integer(x.trunc() as i64))),
            _ => Err(LambdustError::type_error("truncate requires a number")),
        }
    })
}

fn numeric_round() -> Value {
    make_builtin_procedure("round", Some(1), |args| {
        check_arity(args, 1)?;
        let num = expect_number(&args[0], "round")?;
        match num {
            SchemeNumber::Integer(x) => Ok(Value::Number(SchemeNumber::Integer(*x))),
            SchemeNumber::Real(x) => Ok(Value::Number(SchemeNumber::Integer(x.round() as i64))),
            _ => Err(LambdustError::type_error("round requires a number")),
        }
    })
}

fn numeric_sqrt() -> Value {
    make_builtin_procedure("sqrt", Some(1), |args| {
        check_arity(args, 1)?;
        let num = expect_number(&args[0], "sqrt")?;
        match num {
            SchemeNumber::Integer(x) => {
                if *x < 0 {
                    Err(LambdustError::runtime_error("sqrt of negative number"))
                } else {
                    Ok(Value::Number(SchemeNumber::Real((*x as f64).sqrt())))
                }
            }
            SchemeNumber::Real(x) => {
                if *x < 0.0 {
                    Err(LambdustError::runtime_error("sqrt of negative number"))
                } else {
                    Ok(Value::Number(SchemeNumber::Real(x.sqrt())))
                }
            }
            _ => Err(LambdustError::type_error("sqrt requires a number")),
        }
    })
}

fn numeric_expt() -> Value {
    make_builtin_procedure("expt", Some(2), |args| {
        check_arity(args, 2)?;
        let base = expect_number(&args[0], "expt")?;
        let exp = expect_number(&args[1], "expt")?;
        
        match (base, exp) {
            (SchemeNumber::Integer(b), SchemeNumber::Integer(e)) => {
                if *e >= 0 {
                    Ok(Value::Number(SchemeNumber::Integer(b.pow(*e as u32))))
                } else {
                    Ok(Value::Number(SchemeNumber::Real((*b as f64).powf(*e as f64))))
                }
            }
            _ => {
                let b = match base {
                    SchemeNumber::Integer(x) => *x as f64,
                    SchemeNumber::Real(x) => *x,
                    _ => return Err(LambdustError::type_error("expt requires numbers")),
                };
                let e = match exp {
                    SchemeNumber::Integer(x) => *x as f64,
                    SchemeNumber::Real(x) => *x,
                    _ => return Err(LambdustError::type_error("expt requires numbers")),
                };
                Ok(Value::Number(SchemeNumber::Real(b.powf(e))))
            }
        }
    })
}

fn numeric_min() -> Value {
    make_builtin_procedure("min", None, |args| {
        check_arity_range(args, 1, None)?;
        let mut min = expect_number(&args[0], "min")?;
        for arg in &args[1..] {
            let num = expect_number(arg, "min")?;
            if compare_numbers_ordering(num, min) == std::cmp::Ordering::Less {
                min = num;
            }
        }
        Ok(Value::Number(min.clone()))
    })
}

fn numeric_max() -> Value {
    make_builtin_procedure("max", None, |args| {
        check_arity_range(args, 1, None)?;
        let mut max = expect_number(&args[0], "max")?;
        for arg in &args[1..] {
            let num = expect_number(arg, "max")?;
            if compare_numbers_ordering(num, max) == std::cmp::Ordering::Greater {
                max = num;
            }
        }
        Ok(Value::Number(max.clone()))
    })
}

// Helper functions
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_arithmetic_functions() {
        let mut builtins = HashMap::new();
        register_arithmetic_functions(&mut builtins);
        
        assert!(builtins.contains_key("+"));
        assert!(builtins.contains_key("-"));
        assert!(builtins.contains_key("*"));
        assert!(builtins.contains_key("/"));
        assert!(builtins.contains_key("="));
        assert!(builtins.contains_key("<"));
        assert!(builtins.contains_key("abs"));
        assert!(builtins.contains_key("sqrt"));
    }

    #[test]
    fn test_comparison_operations() {
        let eq_proc = arithmetic_eq();
        if let Value::Procedure(proc) = eq_proc {
            let args = vec![
                Value::Number(SchemeNumber::Integer(5)),
                Value::Number(SchemeNumber::Integer(5)),
            ];
            let result = proc.call(&args).unwrap();
            assert_eq!(result, Value::Boolean(true));
        }
    }

    #[test]
    fn test_extended_operations() {
        let abs_proc = numeric_abs();
        if let Value::Procedure(proc) = abs_proc {
            let args = vec![Value::Number(SchemeNumber::Integer(-5))];
            let result = proc.call(&args).unwrap();
            assert_eq!(result, Value::Number(SchemeNumber::Integer(5)));
        }
    }
}
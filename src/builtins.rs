//! Built-in procedures for the Scheme interpreter

use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Create a map of all built-in procedures
pub fn create_builtins() -> HashMap<String, Value> {
    let mut builtins = HashMap::new();

    // Arithmetic operations
    builtins.insert("+".to_string(), arithmetic_add());
    builtins.insert("-".to_string(), arithmetic_sub());
    builtins.insert("*".to_string(), arithmetic_mul());
    builtins.insert("/".to_string(), arithmetic_div());
    builtins.insert("=".to_string(), arithmetic_eq());
    builtins.insert("<".to_string(), arithmetic_lt());
    builtins.insert("<=".to_string(), arithmetic_le());
    builtins.insert(">".to_string(), arithmetic_gt());
    builtins.insert(">=".to_string(), arithmetic_ge());

    // List operations
    builtins.insert("car".to_string(), list_car());
    builtins.insert("cdr".to_string(), list_cdr());
    builtins.insert("cons".to_string(), list_cons());
    builtins.insert("list".to_string(), list_list());
    builtins.insert("length".to_string(), list_length());
    builtins.insert("append".to_string(), list_append());
    builtins.insert("reverse".to_string(), list_reverse());

    // Type predicates
    builtins.insert("null?".to_string(), predicate_null());
    builtins.insert("pair?".to_string(), predicate_pair());
    builtins.insert("list?".to_string(), predicate_list());
    builtins.insert("number?".to_string(), predicate_number());
    builtins.insert("string?".to_string(), predicate_string());
    builtins.insert("symbol?".to_string(), predicate_symbol());
    builtins.insert("boolean?".to_string(), predicate_boolean());
    builtins.insert("procedure?".to_string(), predicate_procedure());

    // Equality predicates
    builtins.insert("eq?".to_string(), equality_eq());
    builtins.insert("eqv?".to_string(), equality_eqv());
    builtins.insert("equal?".to_string(), equality_equal());

    // String operations
    builtins.insert("string-length".to_string(), string_length());
    builtins.insert("string-ref".to_string(), string_ref());
    builtins.insert("string-append".to_string(), string_append());
    builtins.insert("substring".to_string(), string_substring());

    // Conversion functions
    builtins.insert("number->string".to_string(), convert_number_to_string());
    builtins.insert("string->number".to_string(), convert_string_to_number());
    builtins.insert("symbol->string".to_string(), convert_symbol_to_string());
    builtins.insert("string->symbol".to_string(), convert_string_to_symbol());

    // I/O functions (basic)
    builtins.insert("display".to_string(), io_display());
    builtins.insert("newline".to_string(), io_newline());

    // Logical operations
    builtins.insert("not".to_string(), logical_not());

    builtins
}

// Arithmetic operations

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
                    return Err(LambdustError::TypeError(format!(
                        "+ expects numbers, got {arg}"
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
        arity: None, // Variadic
        func: |args| {
            if args.is_empty() {
                return Err(LambdustError::ArityError {
                    expected: 1,
                    actual: 0,
                });
            }

            let first = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("- expects numbers, got {}", args[0]))
            })?;

            if args.len() == 1 {
                // Unary minus
                Ok(Value::Number(negate_number(first)?))
            } else {
                let mut result = first.clone();
                for arg in &args[1..] {
                    if let Some(num) = arg.as_number() {
                        result = sub_numbers(&result, num)?;
                    } else {
                        return Err(LambdustError::TypeError(format!(
                            "- expects numbers, got {arg}"
                        )));
                    }
                }
                Ok(Value::Number(result))
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
                    return Err(LambdustError::TypeError(format!(
                        "* expects numbers, got {arg}"
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
        arity: None, // Variadic
        func: |args| {
            if args.is_empty() {
                return Err(LambdustError::ArityError {
                    expected: 1,
                    actual: 0,
                });
            }

            let first = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("/ expects numbers, got {}", args[0]))
            })?;

            if args.len() == 1 {
                // Reciprocal
                Ok(Value::Number(div_numbers(
                    &SchemeNumber::Integer(1),
                    first,
                )?))
            } else {
                let mut result = first.clone();
                for arg in &args[1..] {
                    if let Some(num) = arg.as_number() {
                        result = div_numbers(&result, num)?;
                    } else {
                        return Err(LambdustError::TypeError(format!(
                            "/ expects numbers, got {arg}"
                        )));
                    }
                }
                Ok(Value::Number(result))
            }
        },
    })
}

fn arithmetic_eq() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "=".to_string(),
        arity: None, // Variadic
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError {
                    expected: 2,
                    actual: args.len(),
                });
            }

            let first = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("= expects numbers, got {}", args[0]))
            })?;

            for arg in &args[1..] {
                if let Some(num) = arg.as_number() {
                    if !numbers_equal(first, num) {
                        return Ok(Value::Boolean(false));
                    }
                } else {
                    return Err(LambdustError::TypeError(format!(
                        "= expects numbers, got {arg}"
                    )));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn arithmetic_lt() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "<".to_string(),
        arity: None, // Variadic
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError {
                    expected: 2,
                    actual: args.len(),
                });
            }

            for i in 0..args.len() - 1 {
                let a = args[i].as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!("< expects numbers, got {}", args[i]))
                })?;
                let b = args[i + 1].as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!("< expects numbers, got {}", args[i + 1]))
                })?;

                if !number_lt(a, b) {
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
        arity: None,
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError {
                    expected: 2,
                    actual: args.len(),
                });
            }

            for i in 0..args.len() - 1 {
                let a = args[i].as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!("<= expects numbers, got {}", args[i]))
                })?;
                let b = args[i + 1].as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!("<= expects numbers, got {}", args[i + 1]))
                })?;

                if !number_le(a, b) {
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
        arity: None,
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError {
                    expected: 2,
                    actual: args.len(),
                });
            }

            for i in 0..args.len() - 1 {
                let a = args[i].as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!("> expects numbers, got {}", args[i]))
                })?;
                let b = args[i + 1].as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!("> expects numbers, got {}", args[i + 1]))
                })?;

                if !number_gt(a, b) {
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
        arity: None,
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError {
                    expected: 2,
                    actual: args.len(),
                });
            }

            for i in 0..args.len() - 1 {
                let a = args[i].as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!(">= expects numbers, got {}", args[i]))
                })?;
                let b = args[i + 1].as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!(">= expects numbers, got {}", args[i + 1]))
                })?;

                if !number_ge(a, b) {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

// List operations

fn list_car() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "car".to_string(),
        arity: Some(1),
        func: |args| match &args[0] {
            Value::Pair(car, _) => Ok((**car).clone()),
            Value::Nil => Err(LambdustError::TypeError("car: empty list".to_string())),
            _ => Err(LambdustError::TypeError("car: not a pair".to_string())),
        },
    })
}

fn list_cdr() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "cdr".to_string(),
        arity: Some(1),
        func: |args| match &args[0] {
            Value::Pair(_, cdr) => Ok((**cdr).clone()),
            Value::Nil => Err(LambdustError::TypeError("cdr: empty list".to_string())),
            _ => Err(LambdustError::TypeError("cdr: not a pair".to_string())),
        },
    })
}

fn list_cons() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "cons".to_string(),
        arity: Some(2),
        func: |args| Ok(Value::cons(args[0].clone(), args[1].clone())),
    })
}

fn list_list() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "list".to_string(),
        arity: None, // Variadic
        func: |args| Ok(Value::from_vector(args.to_vec())),
    })
}

fn list_length() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "length".to_string(),
        arity: Some(1),
        func: |args| match args[0].list_length() {
            Some(len) => Ok(Value::from(len as i64)),
            None => Err(LambdustError::TypeError(
                "length: not a proper list".to_string(),
            )),
        },
    })
}

fn list_append() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "append".to_string(),
        arity: None, // Variadic
        func: |args| {
            if args.is_empty() {
                return Ok(Value::Nil);
            }

            let mut result = Vec::new();
            for (i, arg) in args.iter().enumerate() {
                if i == args.len() - 1 {
                    // Last argument - can be anything
                    if let Some(vec) = arg.to_vector() {
                        result.extend(vec);
                    } else {
                        // If not a proper list, create improper list
                        if result.is_empty() {
                            return Ok(arg.clone());
                        } else {
                            let mut list_result = arg.clone();
                            for val in result.into_iter().rev() {
                                list_result = Value::cons(val, list_result);
                            }
                            return Ok(list_result);
                        }
                    }
                } else {
                    // Other arguments must be proper lists
                    if let Some(vec) = arg.to_vector() {
                        result.extend(vec);
                    } else {
                        return Err(LambdustError::TypeError("append: not a list".to_string()));
                    }
                }
            }
            Ok(Value::from_vector(result))
        },
    })
}

fn list_reverse() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "reverse".to_string(),
        arity: Some(1),
        func: |args| match args[0].to_vector() {
            Some(mut vec) => {
                vec.reverse();
                Ok(Value::from_vector(vec))
            }
            None => Err(LambdustError::TypeError(
                "reverse: not a proper list".to_string(),
            )),
        },
    })
}

// Type predicates

fn predicate_null() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "null?".to_string(),
        arity: Some(1),
        func: |args| Ok(Value::Boolean(args[0].is_nil())),
    })
}

fn predicate_pair() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "pair?".to_string(),
        arity: Some(1),
        func: |args| Ok(Value::Boolean(args[0].is_pair())),
    })
}

fn predicate_list() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "list?".to_string(),
        arity: Some(1),
        func: |args| Ok(Value::Boolean(args[0].is_list())),
    })
}

fn predicate_number() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "number?".to_string(),
        arity: Some(1),
        func: |args| Ok(Value::Boolean(args[0].is_number())),
    })
}

fn predicate_string() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string?".to_string(),
        arity: Some(1),
        func: |args| Ok(Value::Boolean(args[0].is_string())),
    })
}

fn predicate_symbol() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "symbol?".to_string(),
        arity: Some(1),
        func: |args| Ok(Value::Boolean(args[0].is_symbol())),
    })
}

fn predicate_boolean() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "boolean?".to_string(),
        arity: Some(1),
        func: |args| Ok(Value::Boolean(matches!(args[0], Value::Boolean(_)))),
    })
}

fn predicate_procedure() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "procedure?".to_string(),
        arity: Some(1),
        func: |args| Ok(Value::Boolean(args[0].is_procedure())),
    })
}

// Equality predicates

fn equality_eq() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "eq?".to_string(),
        arity: Some(2),
        func: |args| Ok(Value::Boolean(args[0].scheme_eq(&args[1]))),
    })
}

fn equality_eqv() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "eqv?".to_string(),
        arity: Some(2),
        func: |args| Ok(Value::Boolean(args[0].eqv(&args[1]))),
    })
}

fn equality_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "equal?".to_string(),
        arity: Some(2),
        func: |args| Ok(Value::Boolean(args[0].equal(&args[1]))),
    })
}

// String operations

fn string_length() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-length".to_string(),
        arity: Some(1),
        func: |args| {
            if let Some(s) = args[0].as_string() {
                Ok(Value::from(s.len() as i64))
            } else {
                Err(LambdustError::TypeError(
                    "string-length: not a string".to_string(),
                ))
            }
        },
    })
}

fn string_ref() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-ref".to_string(),
        arity: Some(2),
        func: |args| {
            let s = args[0]
                .as_string()
                .ok_or_else(|| LambdustError::TypeError("string-ref: not a string".to_string()))?;
            let n = args[1]
                .as_number()
                .ok_or_else(|| LambdustError::TypeError("string-ref: not a number".to_string()))?;

            if let SchemeNumber::Integer(i) = n {
                if *i >= 0 && (*i as usize) < s.len() {
                    if let Some(c) = s.chars().nth(*i as usize) {
                        Ok(Value::Character(c))
                    } else {
                        Err(LambdustError::RuntimeError(
                            "string-ref: index out of bounds".to_string(),
                        ))
                    }
                } else {
                    Err(LambdustError::RuntimeError(
                        "string-ref: index out of bounds".to_string(),
                    ))
                }
            } else {
                Err(LambdustError::TypeError(
                    "string-ref: index must be an integer".to_string(),
                ))
            }
        },
    })
}

fn string_append() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-append".to_string(),
        arity: None, // Variadic
        func: |args| {
            let mut result = String::new();
            for arg in args {
                if let Some(s) = arg.as_string() {
                    result.push_str(s);
                } else {
                    return Err(LambdustError::TypeError(
                        "string-append: not a string".to_string(),
                    ));
                }
            }
            Ok(Value::String(result))
        },
    })
}

fn string_substring() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "substring".to_string(),
        arity: Some(3),
        func: |args| {
            let s = args[0]
                .as_string()
                .ok_or_else(|| LambdustError::TypeError("substring: not a string".to_string()))?;
            let start = args[1].as_number().ok_or_else(|| {
                LambdustError::TypeError("substring: start not a number".to_string())
            })?;
            let end = args[2].as_number().ok_or_else(|| {
                LambdustError::TypeError("substring: end not a number".to_string())
            })?;

            if let (SchemeNumber::Integer(start_i), SchemeNumber::Integer(end_i)) = (start, end) {
                if *start_i >= 0 && *end_i >= *start_i && (*end_i as usize) <= s.len() {
                    let chars: Vec<char> = s.chars().collect();
                    let substring: String =
                        chars[*start_i as usize..*end_i as usize].iter().collect();
                    Ok(Value::String(substring))
                } else {
                    Err(LambdustError::RuntimeError(
                        "substring: invalid indices".to_string(),
                    ))
                }
            } else {
                Err(LambdustError::TypeError(
                    "substring: indices must be integers".to_string(),
                ))
            }
        },
    })
}

// Conversion functions

fn convert_number_to_string() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "number->string".to_string(),
        arity: Some(1),
        func: |args| {
            if let Some(n) = args[0].as_number() {
                Ok(Value::String(format!("{n}")))
            } else {
                Err(LambdustError::TypeError(
                    "number->string: not a number".to_string(),
                ))
            }
        },
    })
}

fn convert_string_to_number() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string->number".to_string(),
        arity: Some(1),
        func: |args| {
            if let Some(s) = args[0].as_string() {
                // Try to parse as integer first
                if let Ok(i) = s.parse::<i64>() {
                    Ok(Value::Number(SchemeNumber::Integer(i)))
                } else if let Ok(f) = s.parse::<f64>() {
                    Ok(Value::Number(SchemeNumber::Real(f)))
                } else {
                    Ok(Value::Boolean(false))
                }
            } else {
                Err(LambdustError::TypeError(
                    "string->number: not a string".to_string(),
                ))
            }
        },
    })
}

fn convert_symbol_to_string() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "symbol->string".to_string(),
        arity: Some(1),
        func: |args| {
            if let Some(s) = args[0].as_symbol() {
                Ok(Value::String(s.to_string()))
            } else {
                Err(LambdustError::TypeError(
                    "symbol->string: not a symbol".to_string(),
                ))
            }
        },
    })
}

fn convert_string_to_symbol() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string->symbol".to_string(),
        arity: Some(1),
        func: |args| {
            if let Some(s) = args[0].as_string() {
                Ok(Value::Symbol(s.to_string()))
            } else {
                Err(LambdustError::TypeError(
                    "string->symbol: not a string".to_string(),
                ))
            }
        },
    })
}

// I/O functions

fn io_display() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "display".to_string(),
        arity: Some(1),
        func: |args| {
            print!("{}", args[0]);
            Ok(Value::Undefined)
        },
    })
}

fn io_newline() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "newline".to_string(),
        arity: Some(0),
        func: |_args| {
            println!();
            Ok(Value::Undefined)
        },
    })
}

// Logical operations

fn logical_not() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "not".to_string(),
        arity: Some(1),
        func: |args| Ok(Value::Boolean(!args[0].is_truthy())),
    })
}

// Helper functions for arithmetic

fn add_numbers(a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
    match (a, b) {
        (SchemeNumber::Integer(a), SchemeNumber::Integer(b)) => Ok(SchemeNumber::Integer(a + b)),
        (SchemeNumber::Integer(a), SchemeNumber::Real(b)) => Ok(SchemeNumber::Real(*a as f64 + b)),
        (SchemeNumber::Real(a), SchemeNumber::Integer(b)) => Ok(SchemeNumber::Real(a + *b as f64)),
        (SchemeNumber::Real(a), SchemeNumber::Real(b)) => Ok(SchemeNumber::Real(a + b)),
        _ => Err(LambdustError::RuntimeError(
            "Unsupported number types".to_string(),
        )),
    }
}

fn sub_numbers(a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
    match (a, b) {
        (SchemeNumber::Integer(a), SchemeNumber::Integer(b)) => Ok(SchemeNumber::Integer(a - b)),
        (SchemeNumber::Integer(a), SchemeNumber::Real(b)) => Ok(SchemeNumber::Real(*a as f64 - b)),
        (SchemeNumber::Real(a), SchemeNumber::Integer(b)) => Ok(SchemeNumber::Real(a - *b as f64)),
        (SchemeNumber::Real(a), SchemeNumber::Real(b)) => Ok(SchemeNumber::Real(a - b)),
        _ => Err(LambdustError::RuntimeError(
            "Unsupported number types".to_string(),
        )),
    }
}

fn mul_numbers(a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
    match (a, b) {
        (SchemeNumber::Integer(a), SchemeNumber::Integer(b)) => Ok(SchemeNumber::Integer(a * b)),
        (SchemeNumber::Integer(a), SchemeNumber::Real(b)) => Ok(SchemeNumber::Real(*a as f64 * b)),
        (SchemeNumber::Real(a), SchemeNumber::Integer(b)) => Ok(SchemeNumber::Real(a * *b as f64)),
        (SchemeNumber::Real(a), SchemeNumber::Real(b)) => Ok(SchemeNumber::Real(a * b)),
        _ => Err(LambdustError::RuntimeError(
            "Unsupported number types".to_string(),
        )),
    }
}

fn div_numbers(a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
    match (a, b) {
        (SchemeNumber::Integer(a), SchemeNumber::Integer(b)) => {
            if *b == 0 {
                Err(LambdustError::DivisionByZero)
            } else if a % b == 0 {
                Ok(SchemeNumber::Integer(a / b))
            } else {
                Ok(SchemeNumber::Rational(*a, *b))
            }
        }
        (SchemeNumber::Integer(a), SchemeNumber::Real(b)) => {
            if *b == 0.0 {
                Err(LambdustError::DivisionByZero)
            } else {
                Ok(SchemeNumber::Real(*a as f64 / b))
            }
        }
        (SchemeNumber::Real(a), SchemeNumber::Integer(b)) => {
            if *b == 0 {
                Err(LambdustError::DivisionByZero)
            } else {
                Ok(SchemeNumber::Real(a / *b as f64))
            }
        }
        (SchemeNumber::Real(a), SchemeNumber::Real(b)) => {
            if *b == 0.0 {
                Err(LambdustError::DivisionByZero)
            } else {
                Ok(SchemeNumber::Real(a / b))
            }
        }
        _ => Err(LambdustError::RuntimeError(
            "Unsupported number types".to_string(),
        )),
    }
}

fn negate_number(n: &SchemeNumber) -> Result<SchemeNumber> {
    match n {
        SchemeNumber::Integer(i) => Ok(SchemeNumber::Integer(-i)),
        SchemeNumber::Real(r) => Ok(SchemeNumber::Real(-r)),
        SchemeNumber::Rational(n, d) => Ok(SchemeNumber::Rational(-n, *d)),
        SchemeNumber::Complex(r, i) => Ok(SchemeNumber::Complex(-r, -i)),
    }
}

fn numbers_equal(a: &SchemeNumber, b: &SchemeNumber) -> bool {
    match (a, b) {
        (SchemeNumber::Integer(a), SchemeNumber::Integer(b)) => a == b,
        (SchemeNumber::Integer(a), SchemeNumber::Real(b)) => *a as f64 == *b,
        (SchemeNumber::Real(a), SchemeNumber::Integer(b)) => *a == *b as f64,
        (SchemeNumber::Real(a), SchemeNumber::Real(b)) => a == b,
        _ => false,
    }
}

fn number_lt(a: &SchemeNumber, b: &SchemeNumber) -> bool {
    match (a, b) {
        (SchemeNumber::Integer(a), SchemeNumber::Integer(b)) => a < b,
        (SchemeNumber::Integer(a), SchemeNumber::Real(b)) => (*a as f64) < *b,
        (SchemeNumber::Real(a), SchemeNumber::Integer(b)) => *a < (*b as f64),
        (SchemeNumber::Real(a), SchemeNumber::Real(b)) => a < b,
        _ => false,
    }
}

fn number_le(a: &SchemeNumber, b: &SchemeNumber) -> bool {
    numbers_equal(a, b) || number_lt(a, b)
}

fn number_gt(a: &SchemeNumber, b: &SchemeNumber) -> bool {
    !number_le(a, b)
}

fn number_ge(a: &SchemeNumber, b: &SchemeNumber) -> bool {
    !number_lt(a, b)
}

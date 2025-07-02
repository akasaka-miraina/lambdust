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

    // Vector operations
    builtins.insert("vector".to_string(), vector_constructor());
    builtins.insert("vector?".to_string(), predicate_vector());
    builtins.insert("vector-length".to_string(), vector_length());
    builtins.insert("vector-ref".to_string(), vector_ref());
    builtins.insert("make-vector".to_string(), vector_make());
    builtins.insert("vector->list".to_string(), vector_to_list());
    builtins.insert("list->vector".to_string(), list_to_vector());

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
    builtins.insert("odd?".to_string(), predicate_odd());
    builtins.insert("even?".to_string(), predicate_even());
    builtins.insert("zero?".to_string(), predicate_zero());
    builtins.insert("positive?".to_string(), predicate_positive());
    builtins.insert("negative?".to_string(), predicate_negative());

    // Character operations
    builtins.insert("char?".to_string(), predicate_char());
    builtins.insert("char=?".to_string(), char_equal());
    builtins.insert("char<?".to_string(), char_less_than());
    builtins.insert("char>?".to_string(), char_greater_than());
    builtins.insert("char<=?".to_string(), char_less_equal());
    builtins.insert("char>=?".to_string(), char_greater_equal());
    builtins.insert("char->integer".to_string(), char_to_integer());
    builtins.insert("integer->char".to_string(), integer_to_char());

    // Extended string operations
    builtins.insert("string=?".to_string(), string_equal());
    builtins.insert("string<?".to_string(), string_less_than());
    builtins.insert("string>?".to_string(), string_greater_than());
    builtins.insert("string<=?".to_string(), string_less_equal());
    builtins.insert("string>=?".to_string(), string_greater_equal());
    builtins.insert("make-string".to_string(), string_make());
    builtins.insert("string".to_string(), string_constructor());

    // Additional I/O functions
    builtins.insert("read".to_string(), io_read());
    builtins.insert("write".to_string(), io_write());
    builtins.insert("read-char".to_string(), io_read_char());
    builtins.insert("peek-char".to_string(), io_peek_char());
    builtins.insert("write-char".to_string(), io_write_char());
    builtins.insert("eof-object?".to_string(), predicate_eof_object());

    // Additional type conversion functions
    builtins.insert("char->string".to_string(), convert_char_to_string());
    builtins.insert("string->list".to_string(), convert_string_to_list());
    builtins.insert("list->string".to_string(), convert_list_to_string());

    // Note: Higher-order functions (apply, map, for-each) are implemented as special forms in evaluator.rs

    // Record operations (SRFI 9)
    builtins.insert("make-record".to_string(), record_make());
    builtins.insert("record-of-type?".to_string(), record_predicate());
    builtins.insert("record-field".to_string(), record_field_get());
    builtins.insert("record-set-field!".to_string(), record_field_set());

    // Error handling functions
    builtins.insert("error".to_string(), error_function());

    // Multiple values functions
    builtins.insert("values".to_string(), values_function());
    builtins.insert("call-with-values".to_string(), call_with_values_function());

    // Destructive list operations (clone-based implementation)
    builtins.insert("set-car!".to_string(), list_set_car());
    builtins.insert("set-cdr!".to_string(), list_set_cdr());

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
                return Err(LambdustError::ArityError(1, 0));
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
                return Err(LambdustError::ArityError(1, 0));
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
                return Err(LambdustError::ArityError(2, args.len()));
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
                return Err(LambdustError::ArityError(2, args.len()));
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
                return Err(LambdustError::ArityError(2, args.len()));
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
                return Err(LambdustError::ArityError(2, args.len()));
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
                return Err(LambdustError::ArityError(2, args.len()));
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
            Value::Nil => Err(LambdustError::type_error("car: empty list".to_string())),
            _ => Err(LambdustError::type_error("car: not a pair".to_string())),
        },
    })
}

fn list_cdr() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "cdr".to_string(),
        arity: Some(1),
        func: |args| match &args[0] {
            Value::Pair(_, cdr) => Ok((**cdr).clone()),
            Value::Nil => Err(LambdustError::type_error("cdr: empty list".to_string())),
            _ => Err(LambdustError::type_error("cdr: not a pair".to_string())),
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
                        return Err(LambdustError::type_error("append: not a list".to_string()));
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
                .ok_or_else(|| LambdustError::type_error("string-ref: not a string".to_string()))?;
            let n = args[1]
                .as_number()
                .ok_or_else(|| LambdustError::type_error("string-ref: not a number".to_string()))?;

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
                .ok_or_else(|| LambdustError::type_error("substring: not a string".to_string()))?;
            let start = args[1].as_number().ok_or_else(|| {
                LambdustError::type_error("substring: start not a number".to_string())
            })?;
            let end = args[2].as_number().ok_or_else(|| {
                LambdustError::type_error("substring: end not a number".to_string())
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
                Err(LambdustError::DivisionByZero())
            } else if a % b == 0 {
                Ok(SchemeNumber::Integer(a / b))
            } else {
                Ok(SchemeNumber::Rational(*a, *b))
            }
        }
        (SchemeNumber::Integer(a), SchemeNumber::Real(b)) => {
            if *b == 0.0 {
                Err(LambdustError::DivisionByZero())
            } else {
                Ok(SchemeNumber::Real(*a as f64 / b))
            }
        }
        (SchemeNumber::Real(a), SchemeNumber::Integer(b)) => {
            if *b == 0 {
                Err(LambdustError::DivisionByZero())
            } else {
                Ok(SchemeNumber::Real(a / *b as f64))
            }
        }
        (SchemeNumber::Real(a), SchemeNumber::Real(b)) => {
            if *b == 0.0 {
                Err(LambdustError::DivisionByZero())
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

// Vector operations

fn vector_constructor() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "vector".to_string(),
        arity: None, // Variadic
        func: |args| {
            Ok(Value::Vector(args.to_vec()))
        },
    })
}

fn predicate_vector() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "vector?".to_string(),
        arity: Some(1),
        func: |args| {
            Ok(Value::Boolean(matches!(args[0], Value::Vector(_))))
        },
    })
}

fn vector_length() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "vector-length".to_string(),
        arity: Some(1),
        func: |args| {
            match &args[0] {
                Value::Vector(v) => Ok(Value::Number(SchemeNumber::Integer(v.len() as i64))),
                _ => Err(LambdustError::TypeError(format!(
                    "vector-length: expected vector, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn vector_ref() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "vector-ref".to_string(),
        arity: Some(2),
        func: |args| {
            let vector = match &args[0] {
                Value::Vector(v) => v,
                _ => return Err(LambdustError::TypeError(format!(
                    "vector-ref: expected vector, got {}",
                    args[0]
                ))),
            };

            let index = match &args[1] {
                Value::Number(SchemeNumber::Integer(i)) => *i as usize,
                _ => return Err(LambdustError::TypeError(format!(
                    "vector-ref: expected integer index, got {}",
                    args[1]
                ))),
            };

            if index >= vector.len() {
                return Err(LambdustError::RuntimeError(format!(
                    "vector-ref: index {} out of bounds for vector of length {}",
                    index,
                    vector.len()
                )));
            }

            Ok(vector[index].clone())
        },
    })
}

fn vector_make() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "make-vector".to_string(),
        arity: None, // 1 or 2 arguments
        func: |args| {
            if args.is_empty() || args.len() > 2 {
                return Err(LambdustError::ArityError(1, args.len()));
            }

            let length = match &args[0] {
                Value::Number(SchemeNumber::Integer(i)) => {
                    if *i < 0 {
                        return Err(LambdustError::RuntimeError(
                            "make-vector: length must be non-negative".to_string()
                        ));
                    }
                    *i as usize
                },
                _ => return Err(LambdustError::TypeError(format!(
                    "make-vector: expected integer length, got {}",
                    args[0]
                ))),
            };

            let fill_value = if args.len() == 2 {
                args[1].clone()
            } else {
                Value::Undefined
            };

            Ok(Value::Vector(vec![fill_value; length]))
        },
    })
}

fn vector_to_list() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "vector->list".to_string(),
        arity: Some(1),
        func: |args| {
            match &args[0] {
                Value::Vector(v) => Ok(Value::from_vector(v.clone())),
                _ => Err(LambdustError::TypeError(format!(
                    "vector->list: expected vector, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn list_to_vector() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "list->vector".to_string(),
        arity: Some(1),
        func: |args| {
            if let Some(list) = args[0].to_vector() {
                Ok(Value::Vector(list))
            } else {
                Err(LambdustError::TypeError(format!(
                    "list->vector: expected list, got {}",
                    args[0]
                )))
            }
        },
    })
}

#[cfg(test)]
mod vector_tests {
    use super::*;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_vector_constructor() {
        let func = vector_constructor();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
            let args = vec![
                Value::Number(SchemeNumber::Integer(1)),
                Value::Number(SchemeNumber::Integer(2)),
                Value::Number(SchemeNumber::Integer(3)),
            ];
            let result = func(&args).unwrap();
            assert_eq!(result, Value::Vector(args));
        }
    }

    #[test]
    fn test_vector_predicate() {
        let func = predicate_vector();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
            // Test with vector
            let vector = Value::Vector(vec![Value::Number(SchemeNumber::Integer(1))]);
            let result = func(&[vector]).unwrap();
            assert_eq!(result, Value::Boolean(true));

            // Test with non-vector
            let number = Value::Number(SchemeNumber::Integer(42));
            let result = func(&[number]).unwrap();
            assert_eq!(result, Value::Boolean(false));
        }
    }

    #[test]
    fn test_vector_length() {
        let func = vector_length();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
            let vector = Value::Vector(vec![
                Value::Number(SchemeNumber::Integer(1)),
                Value::Number(SchemeNumber::Integer(2)),
                Value::Number(SchemeNumber::Integer(3)),
            ]);
            let result = func(&[vector]).unwrap();
            assert_eq!(result, Value::Number(SchemeNumber::Integer(3)));
        }
    }

    #[test]
    fn test_vector_ref() {
        let func = vector_ref();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
            let vector = Value::Vector(vec![
                Value::Number(SchemeNumber::Integer(10)),
                Value::Number(SchemeNumber::Integer(20)),
                Value::Number(SchemeNumber::Integer(30)),
            ]);
            let index = Value::Number(SchemeNumber::Integer(1));
            let result = func(&[vector, index]).unwrap();
            assert_eq!(result, Value::Number(SchemeNumber::Integer(20)));
        }
    }

    #[test]
    fn test_make_vector() {
        let func = vector_make();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
            // Test with length only
            let length = Value::Number(SchemeNumber::Integer(3));
            let result = func(&[length]).unwrap();
            assert_eq!(result, Value::Vector(vec![Value::Undefined; 3]));

            // Test with length and fill value
            let length = Value::Number(SchemeNumber::Integer(2));
            let fill = Value::Number(SchemeNumber::Integer(42));
            let result = func(&[length, fill.clone()]).unwrap();
            assert_eq!(result, Value::Vector(vec![fill.clone(), fill]));
        }
    }

    #[test]
    fn test_vector_list_conversion() {
        // Test vector->list
        let v2l_func = vector_to_list();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = v2l_func {
            let vector = Value::Vector(vec![
                Value::Number(SchemeNumber::Integer(1)),
                Value::Number(SchemeNumber::Integer(2)),
            ]);
            let result = func(&[vector]).unwrap();
            let expected = Value::from_vector(vec![
                Value::Number(SchemeNumber::Integer(1)),
                Value::Number(SchemeNumber::Integer(2)),
            ]);
            assert_eq!(result, expected);
        }

        // Test list->vector
        let l2v_func = list_to_vector();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = l2v_func {
            let list = Value::from_vector(vec![
                Value::Number(SchemeNumber::Integer(1)),
                Value::Number(SchemeNumber::Integer(2)),
            ]);
            let result = func(&[list]).unwrap();
            let expected = Value::Vector(vec![
                Value::Number(SchemeNumber::Integer(1)),
                Value::Number(SchemeNumber::Integer(2)),
            ]);
            assert_eq!(result, expected);
        }
    }
}

// Extended numeric functions

fn numeric_abs() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "abs".to_string(),
        arity: Some(1),
        func: |args| {
            let num = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("abs: expected number, got {}", args[0]))
            })?;

            let result = match num {
                SchemeNumber::Integer(n) => SchemeNumber::Integer(n.abs()),
                SchemeNumber::Real(n) => SchemeNumber::Real(n.abs()),
                SchemeNumber::Rational(num, den) => {
                    SchemeNumber::Rational(num.abs(), *den)
                }
                SchemeNumber::Complex(real, imag) => {
                    let magnitude = (real * real + imag * imag).sqrt();
                    SchemeNumber::Real(magnitude)
                }
            };

            Ok(Value::Number(result))
        },
    })
}

fn numeric_quotient() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "quotient".to_string(),
        arity: Some(2),
        func: |args| {
            let n1 = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("quotient: expected number, got {}", args[0]))
            })?;
            let n2 = args[1].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("quotient: expected number, got {}", args[1]))
            })?;

            let result = match (n1, n2) {
                (SchemeNumber::Integer(a), SchemeNumber::Integer(b)) => {
                    if *b == 0 {
                        return Err(LambdustError::RuntimeError(
                            "quotient: division by zero".to_string()
                        ));
                    }
                    SchemeNumber::Integer(a / b)
                }
                _ => return Err(LambdustError::TypeError(
                    "quotient: expected integers".to_string()
                )),
            };

            Ok(Value::Number(result))
        },
    })
}

fn numeric_remainder() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "remainder".to_string(),
        arity: Some(2),
        func: |args| {
            let n1 = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("remainder: expected number, got {}", args[0]))
            })?;
            let n2 = args[1].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("remainder: expected number, got {}", args[1]))
            })?;

            let result = match (n1, n2) {
                (SchemeNumber::Integer(a), SchemeNumber::Integer(b)) => {
                    if *b == 0 {
                        return Err(LambdustError::RuntimeError(
                            "remainder: division by zero".to_string()
                        ));
                    }
                    SchemeNumber::Integer(a % b)
                }
                _ => return Err(LambdustError::TypeError(
                    "remainder: expected integers".to_string()
                )),
            };

            Ok(Value::Number(result))
        },
    })
}

fn numeric_modulo() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "modulo".to_string(),
        arity: Some(2),
        func: |args| {
            let n1 = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("modulo: expected number, got {}", args[0]))
            })?;
            let n2 = args[1].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("modulo: expected number, got {}", args[1]))
            })?;

            let result = match (n1, n2) {
                (SchemeNumber::Integer(a), SchemeNumber::Integer(b)) => {
                    if *b == 0 {
                        return Err(LambdustError::RuntimeError(
                            "modulo: division by zero".to_string()
                        ));
                    }
                    // Proper modulo (different from remainder for negative numbers)
                    let rem = a % b;
                    if (rem < 0 && *b > 0) || (rem > 0 && *b < 0) {
                        SchemeNumber::Integer(rem + b)
                    } else {
                        SchemeNumber::Integer(rem)
                    }
                }
                _ => return Err(LambdustError::TypeError(
                    "modulo: expected integers".to_string()
                )),
            };

            Ok(Value::Number(result))
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
                let num = arg.as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!("gcd: expected number, got {}", arg))
                })?;

                let n = match num {
                    SchemeNumber::Integer(n) => n.abs(),
                    _ => return Err(LambdustError::TypeError(
                        "gcd: expected integers".to_string()
                    )),
                };

                result = gcd_two(result, n);
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
                let num = arg.as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!("lcm: expected number, got {}", arg))
                })?;

                let n = match num {
                    SchemeNumber::Integer(n) => n.abs(),
                    _ => return Err(LambdustError::TypeError(
                        "lcm: expected integers".to_string()
                    )),
                };

                if n == 0 {
                    return Ok(Value::Number(SchemeNumber::Integer(0)));
                }

                result = lcm_two(result, n);
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
            let num = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("floor: expected number, got {}", args[0]))
            })?;

            let result = match num {
                SchemeNumber::Integer(n) => SchemeNumber::Integer(*n),
                SchemeNumber::Real(n) => SchemeNumber::Integer(n.floor() as i64),
                SchemeNumber::Rational(num, den) => {
                    let quotient = num / den;
                    SchemeNumber::Integer(quotient)
                }
                _ => return Err(LambdustError::TypeError(
                    "floor: not supported for complex numbers".to_string()
                )),
            };

            Ok(Value::Number(result))
        },
    })
}

fn numeric_ceiling() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "ceiling".to_string(),
        arity: Some(1),
        func: |args| {
            let num = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("ceiling: expected number, got {}", args[0]))
            })?;

            let result = match num {
                SchemeNumber::Integer(n) => SchemeNumber::Integer(*n),
                SchemeNumber::Real(n) => SchemeNumber::Integer(n.ceil() as i64),
                SchemeNumber::Rational(num, den) => {
                    let quotient = num / den;
                    let remainder = num % den;
                    if remainder > 0 {
                        SchemeNumber::Integer(quotient + 1)
                    } else {
                        SchemeNumber::Integer(quotient)
                    }
                }
                _ => return Err(LambdustError::TypeError(
                    "ceiling: not supported for complex numbers".to_string()
                )),
            };

            Ok(Value::Number(result))
        },
    })
}

fn numeric_truncate() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "truncate".to_string(),
        arity: Some(1),
        func: |args| {
            let num = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("truncate: expected number, got {}", args[0]))
            })?;

            let result = match num {
                SchemeNumber::Integer(n) => SchemeNumber::Integer(*n),
                SchemeNumber::Real(n) => SchemeNumber::Integer(n.trunc() as i64),
                SchemeNumber::Rational(num, den) => {
                    SchemeNumber::Integer(num / den)
                }
                _ => return Err(LambdustError::TypeError(
                    "truncate: not supported for complex numbers".to_string()
                )),
            };

            Ok(Value::Number(result))
        },
    })
}

fn numeric_round() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "round".to_string(),
        arity: Some(1),
        func: |args| {
            let num = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("round: expected number, got {}", args[0]))
            })?;

            let result = match num {
                SchemeNumber::Integer(n) => SchemeNumber::Integer(*n),
                SchemeNumber::Real(n) => SchemeNumber::Integer(n.round() as i64),
                SchemeNumber::Rational(num, den) => {
                    let float_val = *num as f64 / *den as f64;
                    SchemeNumber::Integer(float_val.round() as i64)
                }
                _ => return Err(LambdustError::TypeError(
                    "round: not supported for complex numbers".to_string()
                )),
            };

            Ok(Value::Number(result))
        },
    })
}

fn numeric_sqrt() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "sqrt".to_string(),
        arity: Some(1),
        func: |args| {
            let num = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("sqrt: expected number, got {}", args[0]))
            })?;

            let result = match num {
                SchemeNumber::Integer(n) => {
                    if *n < 0 {
                        return Err(LambdustError::RuntimeError(
                            "sqrt: negative number".to_string()
                        ));
                    }
                    SchemeNumber::Real((*n as f64).sqrt())
                }
                SchemeNumber::Real(n) => {
                    if *n < 0.0 {
                        return Err(LambdustError::RuntimeError(
                            "sqrt: negative number".to_string()
                        ));
                    }
                    SchemeNumber::Real(n.sqrt())
                }
                SchemeNumber::Rational(num, den) => {
                    let float_val = *num as f64 / *den as f64;
                    if float_val < 0.0 {
                        return Err(LambdustError::RuntimeError(
                            "sqrt: negative number".to_string()
                        ));
                    }
                    SchemeNumber::Real(float_val.sqrt())
                }
                _ => return Err(LambdustError::TypeError(
                    "sqrt: not supported for complex numbers".to_string()
                )),
            };

            Ok(Value::Number(result))
        },
    })
}

fn numeric_expt() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "expt".to_string(),
        arity: Some(2),
        func: |args| {
            let base = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("expt: expected number, got {}", args[0]))
            })?;
            let exp = args[1].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("expt: expected number, got {}", args[1]))
            })?;

            let result = match (base, exp) {
                (SchemeNumber::Integer(b), SchemeNumber::Integer(e)) => {
                    if *e >= 0 {
                        SchemeNumber::Integer(b.pow(*e as u32))
                    } else {
                        SchemeNumber::Real((*b as f64).powf(*e as f64))
                    }
                }
                (base_num, exp_num) => {
                    let b = match base_num {
                        SchemeNumber::Integer(n) => *n as f64,
                        SchemeNumber::Real(n) => *n,
                        SchemeNumber::Rational(num, den) => *num as f64 / *den as f64,
                        _ => return Err(LambdustError::TypeError(
                            "expt: complex numbers not supported".to_string()
                        )),
                    };
                    let e = match exp_num {
                        SchemeNumber::Integer(n) => *n as f64,
                        SchemeNumber::Real(n) => *n,
                        SchemeNumber::Rational(num, den) => *num as f64 / *den as f64,
                        _ => return Err(LambdustError::TypeError(
                            "expt: complex numbers not supported".to_string()
                        )),
                    };
                    SchemeNumber::Real(b.powf(e))
                }
            };

            Ok(Value::Number(result))
        },
    })
}

fn numeric_min() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "min".to_string(),
        arity: None, // Variadic, but at least 1
        func: |args| {
            if args.is_empty() {
                return Err(LambdustError::ArityError(1, 0));
            }

            let mut min_val = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("min: expected number, got {}", args[0]))
            })?;

            for arg in &args[1..] {
                let num = arg.as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!("min: expected number, got {}", arg))
                })?;

                if number_lt(num, min_val) {
                    min_val = num;
                }
            }

            Ok(Value::Number(min_val.clone()))
        },
    })
}

fn numeric_max() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "max".to_string(),
        arity: None, // Variadic, but at least 1
        func: |args| {
            if args.is_empty() {
                return Err(LambdustError::ArityError(1, 0));
            }

            let mut max_val = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("max: expected number, got {}", args[0]))
            })?;

            for arg in &args[1..] {
                let num = arg.as_number().ok_or_else(|| {
                    LambdustError::TypeError(format!("max: expected number, got {}", arg))
                })?;

                if number_gt(num, max_val) {
                    max_val = num;
                }
            }

            Ok(Value::Number(max_val.clone()))
        },
    })
}

// Numeric predicates

fn predicate_odd() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "odd?".to_string(),
        arity: Some(1),
        func: |args| {
            let num = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("odd?: expected number, got {}", args[0]))
            })?;

            let result = match num {
                SchemeNumber::Integer(n) => n % 2 != 0,
                _ => return Err(LambdustError::TypeError(
                    "odd?: expected integer".to_string()
                )),
            };

            Ok(Value::Boolean(result))
        },
    })
}

fn predicate_even() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "even?".to_string(),
        arity: Some(1),
        func: |args| {
            let num = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("even?: expected number, got {}", args[0]))
            })?;

            let result = match num {
                SchemeNumber::Integer(n) => n % 2 == 0,
                _ => return Err(LambdustError::TypeError(
                    "even?: expected integer".to_string()
                )),
            };

            Ok(Value::Boolean(result))
        },
    })
}

fn predicate_zero() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "zero?".to_string(),
        arity: Some(1),
        func: |args| {
            let num = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("zero?: expected number, got {}", args[0]))
            })?;

            let result = match num {
                SchemeNumber::Integer(n) => *n == 0,
                SchemeNumber::Real(n) => *n == 0.0,
                SchemeNumber::Rational(num, _) => *num == 0,
                SchemeNumber::Complex(real, imag) => *real == 0.0 && *imag == 0.0,
            };

            Ok(Value::Boolean(result))
        },
    })
}

fn predicate_positive() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "positive?".to_string(),
        arity: Some(1),
        func: |args| {
            let num = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("positive?: expected number, got {}", args[0]))
            })?;

            let result = match num {
                SchemeNumber::Integer(n) => *n > 0,
                SchemeNumber::Real(n) => *n > 0.0,
                SchemeNumber::Rational(num, den) => (*num > 0 && *den > 0) || (*num < 0 && *den < 0),
                _ => return Err(LambdustError::TypeError(
                    "positive?: not supported for complex numbers".to_string()
                )),
            };

            Ok(Value::Boolean(result))
        },
    })
}

fn predicate_negative() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "negative?".to_string(),
        arity: Some(1),
        func: |args| {
            let num = args[0].as_number().ok_or_else(|| {
                LambdustError::TypeError(format!("negative?: expected number, got {}", args[0]))
            })?;

            let result = match num {
                SchemeNumber::Integer(n) => *n < 0,
                SchemeNumber::Real(n) => *n < 0.0,
                SchemeNumber::Rational(num, den) => (*num < 0 && *den > 0) || (*num > 0 && *den < 0),
                _ => return Err(LambdustError::TypeError(
                    "negative?: not supported for complex numbers".to_string()
                )),
            };

            Ok(Value::Boolean(result))
        },
    })
}

// Helper functions for GCD and LCM

fn gcd_two(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        gcd_two(b, a % b)
    }
}

fn lcm_two(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a * b).abs() / gcd_two(a, b)
    }
}

// Character operations

fn predicate_char() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char?".to_string(),
        arity: Some(1),
        func: |args| {
            Ok(Value::Boolean(matches!(args[0], Value::Character(_))))
        },
    })
}

fn char_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char=?".to_string(),
        arity: None, // Variadic, at least 2
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }

            let first_char = match &args[0] {
                Value::Character(c) => *c,
                _ => return Err(LambdustError::TypeError(format!(
                    "char=?: expected character, got {}", args[0]
                ))),
            };

            for arg in &args[1..] {
                let c = match arg {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char=?: expected character, got {}", arg
                    ))),
                };

                if c != first_char {
                    return Ok(Value::Boolean(false));
                }
            }

            Ok(Value::Boolean(true))
        },
    })
}

fn char_less_than() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char<?".to_string(),
        arity: None, // Variadic, at least 2
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }

            let mut prev_char = match &args[0] {
                Value::Character(c) => *c,
                _ => return Err(LambdustError::TypeError(format!(
                    "char<?: expected character, got {}", args[0]
                ))),
            };

            for arg in &args[1..] {
                let c = match arg {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char<?: expected character, got {}", arg
                    ))),
                };

                if prev_char >= c {
                    return Ok(Value::Boolean(false));
                }
                prev_char = c;
            }

            Ok(Value::Boolean(true))
        },
    })
}

fn char_greater_than() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char>?".to_string(),
        arity: None, // Variadic, at least 2
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }

            let mut prev_char = match &args[0] {
                Value::Character(c) => *c,
                _ => return Err(LambdustError::TypeError(format!(
                    "char>?: expected character, got {}", args[0]
                ))),
            };

            for arg in &args[1..] {
                let c = match arg {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char>?: expected character, got {}", arg
                    ))),
                };

                if prev_char <= c {
                    return Ok(Value::Boolean(false));
                }
                prev_char = c;
            }

            Ok(Value::Boolean(true))
        },
    })
}

fn char_less_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char<=?".to_string(),
        arity: None, // Variadic, at least 2
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }

            let mut prev_char = match &args[0] {
                Value::Character(c) => *c,
                _ => return Err(LambdustError::TypeError(format!(
                    "char<=?: expected character, got {}", args[0]
                ))),
            };

            for arg in &args[1..] {
                let c = match arg {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char<=?: expected character, got {}", arg
                    ))),
                };

                if prev_char > c {
                    return Ok(Value::Boolean(false));
                }
                prev_char = c;
            }

            Ok(Value::Boolean(true))
        },
    })
}

fn char_greater_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char>=?".to_string(),
        arity: None, // Variadic, at least 2
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }

            let mut prev_char = match &args[0] {
                Value::Character(c) => *c,
                _ => return Err(LambdustError::TypeError(format!(
                    "char>=?: expected character, got {}", args[0]
                ))),
            };

            for arg in &args[1..] {
                let c = match arg {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char>=?: expected character, got {}", arg
                    ))),
                };

                if prev_char < c {
                    return Ok(Value::Boolean(false));
                }
                prev_char = c;
            }

            Ok(Value::Boolean(true))
        },
    })
}

fn char_to_integer() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char->integer".to_string(),
        arity: Some(1),
        func: |args| {
            let c = match &args[0] {
                Value::Character(c) => *c,
                _ => return Err(LambdustError::TypeError(format!(
                    "char->integer: expected character, got {}", args[0]
                ))),
            };

            Ok(Value::Number(SchemeNumber::Integer(c as u32 as i64)))
        },
    })
}

fn integer_to_char() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "integer->char".to_string(),
        arity: Some(1),
        func: |args| {
            let n = match &args[0] {
                Value::Number(SchemeNumber::Integer(n)) => *n,
                _ => return Err(LambdustError::TypeError(format!(
                    "integer->char: expected integer, got {}", args[0]
                ))),
            };

            if n < 0 || n > u32::MAX as i64 {
                return Err(LambdustError::RuntimeError(format!(
                    "integer->char: integer {} out of character range", n
                )));
            }

            let c = char::from_u32(n as u32).ok_or_else(|| {
                LambdustError::RuntimeError(format!(
                    "integer->char: invalid Unicode code point {}", n
                ))
            })?;

            Ok(Value::Character(c))
        },
    })
}

// Extended string operations

fn string_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string=?".to_string(),
        arity: None, // Variadic, at least 2
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }

            let first_str = match &args[0] {
                Value::String(s) => s,
                _ => return Err(LambdustError::TypeError(format!(
                    "string=?: expected string, got {}", args[0]
                ))),
            };

            for arg in &args[1..] {
                let s = match arg {
                    Value::String(s) => s,
                    _ => return Err(LambdustError::TypeError(format!(
                        "string=?: expected string, got {}", arg
                    ))),
                };

                if s != first_str {
                    return Ok(Value::Boolean(false));
                }
            }

            Ok(Value::Boolean(true))
        },
    })
}

fn string_less_than() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string<?".to_string(),
        arity: None, // Variadic, at least 2
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }

            let mut prev_str = match &args[0] {
                Value::String(s) => s,
                _ => return Err(LambdustError::TypeError(format!(
                    "string<?: expected string, got {}", args[0]
                ))),
            };

            for arg in &args[1..] {
                let s = match arg {
                    Value::String(s) => s,
                    _ => return Err(LambdustError::TypeError(format!(
                        "string<?: expected string, got {}", arg
                    ))),
                };

                if prev_str >= s {
                    return Ok(Value::Boolean(false));
                }
                prev_str = s;
            }

            Ok(Value::Boolean(true))
        },
    })
}

fn string_greater_than() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string>?".to_string(),
        arity: None, // Variadic, at least 2
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }

            let mut prev_str = match &args[0] {
                Value::String(s) => s,
                _ => return Err(LambdustError::TypeError(format!(
                    "string>?: expected string, got {}", args[0]
                ))),
            };

            for arg in &args[1..] {
                let s = match arg {
                    Value::String(s) => s,
                    _ => return Err(LambdustError::TypeError(format!(
                        "string>?: expected string, got {}", arg
                    ))),
                };

                if prev_str <= s {
                    return Ok(Value::Boolean(false));
                }
                prev_str = s;
            }

            Ok(Value::Boolean(true))
        },
    })
}

fn string_less_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string<=?".to_string(),
        arity: None, // Variadic, at least 2
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }

            let mut prev_str = match &args[0] {
                Value::String(s) => s,
                _ => return Err(LambdustError::TypeError(format!(
                    "string<=?: expected string, got {}", args[0]
                ))),
            };

            for arg in &args[1..] {
                let s = match arg {
                    Value::String(s) => s,
                    _ => return Err(LambdustError::TypeError(format!(
                        "string<=?: expected string, got {}", arg
                    ))),
                };

                if prev_str > s {
                    return Ok(Value::Boolean(false));
                }
                prev_str = s;
            }

            Ok(Value::Boolean(true))
        },
    })
}

fn string_greater_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string>=?".to_string(),
        arity: None, // Variadic, at least 2
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }

            let mut prev_str = match &args[0] {
                Value::String(s) => s,
                _ => return Err(LambdustError::TypeError(format!(
                    "string>=?: expected string, got {}", args[0]
                ))),
            };

            for arg in &args[1..] {
                let s = match arg {
                    Value::String(s) => s,
                    _ => return Err(LambdustError::TypeError(format!(
                        "string>=?: expected string, got {}", arg
                    ))),
                };

                if prev_str < s {
                    return Ok(Value::Boolean(false));
                }
                prev_str = s;
            }

            Ok(Value::Boolean(true))
        },
    })
}

fn string_make() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "make-string".to_string(),
        arity: None, // 1 or 2 arguments
        func: |args| {
            if args.is_empty() || args.len() > 2 {
                return Err(LambdustError::ArityError(1, args.len()));
            }

            let length = match &args[0] {
                Value::Number(SchemeNumber::Integer(n)) => {
                    if *n < 0 {
                        return Err(LambdustError::RuntimeError(
                            "make-string: length must be non-negative".to_string()
                        ));
                    }
                    *n as usize
                },
                _ => return Err(LambdustError::TypeError(format!(
                    "make-string: expected integer length, got {}", args[0]
                ))),
            };

            let fill_char = if args.len() == 2 {
                match &args[1] {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "make-string: expected character, got {}", args[1]
                    ))),
                }
            } else {
                ' ' // Default fill character
            };

            Ok(Value::String(fill_char.to_string().repeat(length)))
        },
    })
}

fn string_constructor() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string".to_string(),
        arity: None, // Variadic
        func: |args| {
            let mut result = String::new();
            
            for arg in args {
                let c = match arg {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "string: expected character, got {}", arg
                    ))),
                };
                result.push(c);
            }

            Ok(Value::String(result))
        },
    })
}

// Record operations (SRFI 9)

fn record_make() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "make-record".to_string(),
        arity: Some(2), // type-name and field-list
        func: |args| {
            use crate::value::{Record, RecordType};
            
            let type_name = match &args[0] {
                Value::Symbol(name) => name.clone(),
                _ => return Err(LambdustError::TypeError(format!(
                    "make-record: expected symbol for type name, got {}", args[0]
                ))),
            };
            
            let fields = match args[1].to_vector() {
                Some(vec) => vec,
                None => return Err(LambdustError::TypeError(format!(
                    "make-record: expected list for field values, got {}", args[1]
                ))),
            };
            
            // Create a basic record type (in a full implementation, this would be looked up)
            let record_type = RecordType {
                name: type_name.clone(),
                field_names: (0..fields.len()).map(|i| format!("field-{}", i)).collect(),
                constructor_name: format!("make-{}", type_name),
                predicate_name: format!("{}?", type_name),
            };
            
            let record = Record {
                record_type,
                fields,
            };
            
            Ok(Value::Record(record))
        },
    })
}

fn record_predicate() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "record-of-type?".to_string(),
        arity: Some(2), // object and type-name
        func: |args| {
            let type_name = match &args[1] {
                Value::Symbol(name) => name,
                _ => return Err(LambdustError::TypeError(format!(
                    "record-of-type?: expected symbol for type name, got {}", args[1]
                ))),
            };
            
            let is_record_of_type = match &args[0] {
                Value::Record(record) => &record.record_type.name == type_name,
                _ => false,
            };
            
            Ok(Value::Boolean(is_record_of_type))
        },
    })
}

fn record_field_get() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "record-field".to_string(),
        arity: Some(2), // record and field-index
        func: |args| {
            let record = match &args[0] {
                Value::Record(r) => r,
                _ => return Err(LambdustError::TypeError(format!(
                    "record-field: expected record, got {}", args[0]
                ))),
            };
            
            let index = match &args[1] {
                Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as usize,
                _ => return Err(LambdustError::TypeError(format!(
                    "record-field: expected integer for field index, got {}", args[1]
                ))),
            };
            
            if index >= record.fields.len() {
                return Err(LambdustError::RuntimeError(format!(
                    "record-field: field index {} out of range for record with {} fields",
                    index, record.fields.len()
                )));
            }
            
            Ok(record.fields[index].clone())
        },
    })
}

fn record_field_set() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "record-set-field!".to_string(),
        arity: Some(3), // record, field-index, and new-value
        func: |args| {
            // Note: This is a simplified implementation. In a real implementation,
            // records would need to be mutable or we'd need to return a new record.
            // For now, we'll return an error indicating mutation is not supported.
            Err(LambdustError::RuntimeError(
                "record-set-field!: record mutation not yet implemented".to_string()
            ))
        },
    })
}

// Additional I/O functions

fn io_read() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "read".to_string(),
        arity: Some(0), // Takes no arguments for now (stdin)
        func: |_args| {
            // For now, return a simple implementation that reads from stdin
            // In a complete implementation, this would parse S-expressions
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        Ok(Value::Nil)
                    } else {
                        // Try to parse as a number first
                        if let Ok(n) = trimmed.parse::<i64>() {
                            Ok(Value::Number(SchemeNumber::Integer(n)))
                        } else if let Ok(f) = trimmed.parse::<f64>() {
                            Ok(Value::Number(SchemeNumber::Real(f)))
                        } else if trimmed == "#t" {
                            Ok(Value::Boolean(true))
                        } else if trimmed == "#f" {
                            Ok(Value::Boolean(false))
                        } else if trimmed.starts_with('"') && trimmed.ends_with('"') {
                            Ok(Value::String(trimmed[1..trimmed.len()-1].to_string()))
                        } else {
                            Ok(Value::Symbol(trimmed.to_string()))
                        }
                    }
                },
                Err(e) => Err(LambdustError::IoError(e.to_string())),
            }
        },
    })
}

fn io_write() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "write".to_string(),
        arity: Some(1),
        func: |args| {
            print!("{}", args[0]);
            use std::io::{stdout, Write};
            stdout().flush().map_err(|e| LambdustError::IoError(e.to_string()))?;
            Ok(Value::Undefined)
        },
    })
}

fn io_read_char() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "read-char".to_string(),
        arity: Some(0), // Takes no arguments for now (stdin)
        func: |_args| {
            use std::io::{self, Read};
            let mut buffer = [0; 1];
            match io::stdin().read_exact(&mut buffer) {
                Ok(_) => Ok(Value::Character(buffer[0] as char)),
                Err(_) => Ok(Value::Symbol("eof-object".to_string())), // EOF
            }
        },
    })
}

fn io_peek_char() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "peek-char".to_string(),
        arity: Some(0),
        func: |_args| {
            // Simple implementation - just return space for now
            // A complete implementation would need buffered input
            Ok(Value::Character(' '))
        },
    })
}

fn io_write_char() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "write-char".to_string(),
        arity: Some(1),
        func: |args| {
            let ch = match &args[0] {
                Value::Character(c) => *c,
                _ => return Err(LambdustError::TypeError(format!(
                    "write-char: expected character, got {}", args[0]
                ))),
            };
            
            print!("{}", ch);
            use std::io::{stdout, Write};
            stdout().flush().map_err(|e| LambdustError::IoError(e.to_string()))?;
            Ok(Value::Undefined)
        },
    })
}

fn predicate_eof_object() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "eof-object?".to_string(),
        arity: Some(1),
        func: |args| {
            let is_eof = match &args[0] {
                Value::Symbol(s) => s == "eof-object",
                _ => false,
            };
            Ok(Value::Boolean(is_eof))
        },
    })
}

// Additional type conversion functions

fn convert_char_to_string() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char->string".to_string(),
        arity: Some(1),
        func: |args| {
            let ch = match &args[0] {
                Value::Character(c) => *c,
                _ => return Err(LambdustError::TypeError(format!(
                    "char->string: expected character, got {}", args[0]
                ))),
            };
            Ok(Value::String(ch.to_string()))
        },
    })
}

fn convert_string_to_list() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string->list".to_string(),
        arity: Some(1),
        func: |args| {
            let s = match &args[0] {
                Value::String(s) => s,
                _ => return Err(LambdustError::TypeError(format!(
                    "string->list: expected string, got {}", args[0]
                ))),
            };
            
            let chars: Vec<Value> = s.chars().map(Value::Character).collect();
            Ok(Value::from_vector(chars))
        },
    })
}

fn convert_list_to_string() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "list->string".to_string(),
        arity: Some(1),
        func: |args| {
            let chars = match args[0].to_vector() {
                Some(vec) => vec,
                None => return Err(LambdustError::TypeError(format!(
                    "list->string: expected list, got {}", args[0]
                ))),
            };
            
            let mut result = String::new();
            for val in chars {
                let ch = match val {
                    Value::Character(c) => c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "list->string: expected list of characters, got {}", val
                    ))),
                };
                result.push(ch);
            }
            
            Ok(Value::String(result))
        },
    })
}

// Error handling functions

/// Implements the `error` function for raising errors
fn error_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "error".to_string(),
        arity: None, // Variadic - can take message and optional irritants
        func: |args| {
            if args.is_empty() {
                return Err(LambdustError::runtime_error("error: expected at least one argument".to_string()));
            }

            // First argument should be the error message
            let message = match &args[0] {
                Value::String(s) => s.clone(),
                Value::Symbol(s) => s.clone(),
                other => format!("{}", other),
            };

            // Additional arguments are irritants (values that provide context)
            let mut full_message = message;
            if args.len() > 1 {
                full_message.push_str(": ");
                for (i, irritant) in args[1..].iter().enumerate() {
                    if i > 0 {
                        full_message.push_str(", ");
                    }
                    full_message.push_str(&format!("{}", irritant));
                }
            }

            Err(LambdustError::RuntimeError {
                message: full_message,
                location: crate::error::SourceSpan::unknown(),
                stack_trace: Vec::new(),
            })
        },
    })
}

// Multiple values functions

/// Implements the `values` function for creating multiple values
fn values_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "values".to_string(),
        arity: None, // Variadic - can take any number of arguments
        func: |args| {
            Ok(Value::Values(args.to_vec()))
        },
    })
}

// Destructive list operations (clone-based)

/// Implements the `set-car!` function 
/// Note: This creates a new pair rather than true mutation due to Rust's ownership model
fn list_set_car() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "set-car!".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::RuntimeError {
                    message: "set-car!: expected exactly 2 arguments".to_string(),
                    location: crate::error::SourceSpan::unknown(),
                    stack_trace: Vec::new(),
                });
            }

            match &args[0] {
                Value::Pair(_, cdr) => {
                    // Create new pair with new car and existing cdr
                    let new_pair = Value::Pair(Box::new(args[1].clone()), cdr.clone());
                    
                    // Note: In a true Scheme implementation, this would mutate the original pair
                    // Here we return the new pair, but this doesn't provide true mutation semantics
                    // A complete implementation would require using Rc<RefCell<>> throughout the Value system
                    
                    Ok(new_pair)
                }
                _ => Err(LambdustError::RuntimeError {
                    message: format!("set-car!: expected pair, got {}", args[0]),
                    location: crate::error::SourceSpan::unknown(),
                    stack_trace: Vec::new(),
                }),
            }
        },
    })
}

/// Implements the `set-cdr!` function
/// Note: This creates a new pair rather than true mutation due to Rust's ownership model  
fn list_set_cdr() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "set-cdr!".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::RuntimeError {
                    message: "set-cdr!: expected exactly 2 arguments".to_string(),
                    location: crate::error::SourceSpan::unknown(),
                    stack_trace: Vec::new(),
                });
            }

            match &args[0] {
                Value::Pair(car, _) => {
                    // Create new pair with existing car and new cdr
                    let new_pair = Value::Pair(car.clone(), Box::new(args[1].clone()));
                    
                    // Note: Same limitation as set-car! - this doesn't provide true mutation semantics
                    Ok(new_pair)
                }
                _ => Err(LambdustError::RuntimeError {
                    message: format!("set-cdr!: expected pair, got {}", args[0]),
                    location: crate::error::SourceSpan::unknown(),
                    stack_trace: Vec::new(),
                }),
            }
        },
    })
}

/// Implements the `call-with-values` function for consuming multiple values
fn call_with_values_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "call-with-values".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::RuntimeError {
                    message: "call-with-values: expected exactly 2 arguments".to_string(),
                    location: crate::error::SourceSpan::unknown(),
                    stack_trace: Vec::new(),
                });
            }

            // For now, return a placeholder implementation
            // A complete implementation would require evaluator access to call procedures
            Err(LambdustError::RuntimeError {
                message: "call-with-values: not yet fully implemented - requires evaluator integration".to_string(),
                location: crate::error::SourceSpan::unknown(),
                stack_trace: Vec::new(),
            })
        },
    })
}

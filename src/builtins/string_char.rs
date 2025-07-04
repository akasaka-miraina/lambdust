//! String and character operations for Scheme

use crate::builtins::utils::{
    check_arity, expect_character, expect_integer_index, expect_number, expect_string,
    make_builtin_procedure, string_char_at,
};
use crate::error::LambdustError;
use crate::lexer::SchemeNumber;
use crate::value::{Procedure, Value};
use crate::{make_char_comparison, make_string_comparison};
use std::collections::HashMap;

/// Register all string and character functions
pub fn register_string_char_functions(builtins: &mut HashMap<String, Value>) {
    // String operations
    builtins.insert("string-length".to_string(), string_length());
    builtins.insert("string-ref".to_string(), string_ref());
    builtins.insert("string-append".to_string(), string_append());
    builtins.insert("substring".to_string(), string_substring());
    builtins.insert(
        "string=?".to_string(),
        make_string_comparison!("string=?", ==),
    );
    builtins.insert(
        "string<?".to_string(),
        make_string_comparison!("string<?", <),
    );
    builtins.insert(
        "string>?".to_string(),
        make_string_comparison!("string>?", >),
    );
    builtins.insert(
        "string<=?".to_string(),
        make_string_comparison!("string<=?", <=),
    );
    builtins.insert(
        "string>=?".to_string(),
        make_string_comparison!("string>=?", >=),
    );
    builtins.insert("make-string".to_string(), string_make());
    builtins.insert("string".to_string(), string_constructor());

    // Character operations
    builtins.insert("char=?".to_string(), make_char_comparison!("char=?", ==));
    builtins.insert("char<?".to_string(), make_char_comparison!("char<?", <));
    builtins.insert("char>?".to_string(), make_char_comparison!("char>?", >));
    builtins.insert("char<=?".to_string(), make_char_comparison!("char<=?", <=));
    builtins.insert("char>=?".to_string(), make_char_comparison!("char>=?", >=));
    builtins.insert("char->integer".to_string(), char_to_integer());
    builtins.insert("integer->char".to_string(), integer_to_char());

    // Conversion functions
    builtins.insert("char->string".to_string(), convert_char_to_string());
    builtins.insert("string->list".to_string(), convert_string_to_list());
    builtins.insert("list->string".to_string(), convert_list_to_string());
    builtins.insert("number->string".to_string(), convert_number_to_string());
    builtins.insert("string->number".to_string(), convert_string_to_number());
    builtins.insert("symbol->string".to_string(), convert_symbol_to_string());
    builtins.insert("string->symbol".to_string(), convert_string_to_symbol());
}

// String operations

fn string_length() -> Value {
    make_builtin_procedure("string-length", Some(1), |args| {
        check_arity(args, 1)?;
        let s = expect_string(&args[0], "string-length")?;
        Ok(Value::Number(SchemeNumber::Integer(
            s.chars().count() as i64
        )))
    })
}

fn string_ref() -> Value {
    make_builtin_procedure("string-ref", Some(2), |args| {
        check_arity(args, 2)?;
        let s = expect_string(&args[0], "string-ref")?;
        let index = expect_integer_index(&args[1], "string-ref")?;
        let ch = string_char_at(s, index, "string-ref")?;
        Ok(Value::Character(ch))
    })
}

fn string_append() -> Value {
    make_builtin_procedure("string-append", None, |args| {
        let mut result = String::new();
        for arg in args {
            let s = expect_string(arg, "string-append")?;
            result.push_str(s);
        }
        Ok(Value::String(result))
    })
}

fn string_substring() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "substring".to_string(),
        arity: None, // 2 or 3 arguments
        func: |args| {
            if args.len() < 2 || args.len() > 3 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            let s = args[0].as_string().ok_or_else(|| {
                LambdustError::type_error(format!("substring: expected string, got {}", args[0]))
            })?;

            let start = match args[1].as_number() {
                Some(SchemeNumber::Integer(i)) => *i as usize,
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "substring: expected integer start, got {}",
                        args[1]
                    )));
                }
            };

            let chars: Vec<char> = s.chars().collect();
            let end = if args.len() == 3 {
                match args[2].as_number() {
                    Some(SchemeNumber::Integer(i)) => *i as usize,
                    _ => {
                        return Err(LambdustError::type_error(format!(
                            "substring: expected integer end, got {}",
                            args[2]
                        )));
                    }
                }
            } else {
                chars.len()
            };

            if start > chars.len() || end > chars.len() || start > end {
                return Err(LambdustError::runtime_error(format!(
                    "substring: invalid range [{}, {}) for string of length {}",
                    start,
                    end,
                    chars.len()
                )));
            }

            let result: String = chars[start..end].iter().collect();
            Ok(Value::String(result))
        },
    })
}

// String and character comparison functions are now implemented using macros
// for consistency and reduced code duplication

fn string_make() -> Value {
    make_builtin_procedure("make-string", None, |args| {
        if args.is_empty() || args.len() > 2 {
            return Err(LambdustError::arity_error(1, args.len()));
        }

        let length = expect_integer_index(&args[0], "make-string")?;

        let fill_char = if args.len() == 2 {
            expect_character(&args[1], "make-string")?
        } else {
            ' ' // Default space character
        };

        Ok(Value::String(
            std::iter::repeat_n(fill_char, length).collect(),
        ))
    })
}

fn string_constructor() -> Value {
    make_builtin_procedure("string", None, |args| {
        let mut result = String::new();
        for arg in args {
            let ch = expect_character(arg, "string")?;
            result.push(ch);
        }
        Ok(Value::String(result))
    })
}

fn char_to_integer() -> Value {
    make_builtin_procedure("char->integer", Some(1), |args| {
        check_arity(args, 1)?;
        let ch = expect_character(&args[0], "char->integer")?;
        Ok(Value::Number(SchemeNumber::Integer(ch as u8 as i64)))
    })
}

fn integer_to_char() -> Value {
    make_builtin_procedure("integer->char", Some(1), |args| {
        check_arity(args, 1)?;
        let num = expect_number(&args[0], "integer->char")?;

        match num {
            SchemeNumber::Integer(n) => {
                if *n >= 0 && *n <= 127 {
                    Ok(Value::Character(*n as u8 as char))
                } else {
                    Err(LambdustError::runtime_error(format!(
                        "integer->char: value {} out of ASCII range",
                        n
                    )))
                }
            }
            _ => Err(LambdustError::type_error("integer->char: expected integer")),
        }
    })
}

// Conversion functions

fn convert_char_to_string() -> Value {
    make_builtin_procedure("char->string", Some(1), |args| {
        check_arity(args, 1)?;
        let ch = expect_character(&args[0], "char->string")?;
        Ok(Value::String(ch.to_string()))
    })
}

fn convert_string_to_list() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string->list".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_string() {
                Some(s) => {
                    let chars: Vec<Value> = s.chars().map(Value::Character).collect();
                    Ok(Value::from_vector(chars))
                }
                None => Err(LambdustError::type_error(format!(
                    "string->list: expected string, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn convert_list_to_string() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "list->string".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            let chars = match args[0].to_vector() {
                Some(vec) => vec,
                None => {
                    return Err(LambdustError::type_error(format!(
                        "list->string: expected list, got {}",
                        args[0]
                    )));
                }
            };

            let mut result = String::new();
            for val in chars {
                let ch = match val {
                    Value::Character(c) => c,
                    _ => {
                        return Err(LambdustError::type_error(format!(
                            "list->string: expected list of characters, got {}",
                            val
                        )));
                    }
                };
                result.push(ch);
            }

            Ok(Value::String(result))
        },
    })
}

fn convert_number_to_string() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "number->string".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_number() {
                Some(n) => Ok(Value::String(n.to_string())),
                None => Err(LambdustError::type_error(format!(
                    "number->string: expected number, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn convert_string_to_number() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string->number".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_string() {
                Some(s) => {
                    if let Ok(i) = s.parse::<i64>() {
                        Ok(Value::Number(SchemeNumber::Integer(i)))
                    } else if let Ok(f) = s.parse::<f64>() {
                        Ok(Value::Number(SchemeNumber::Real(f)))
                    } else {
                        Ok(Value::Boolean(false)) // Return #f if not a valid number
                    }
                }
                None => Err(LambdustError::type_error(format!(
                    "string->number: expected string, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn convert_symbol_to_string() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "symbol->string".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_symbol() {
                Some(s) => Ok(Value::String(s.to_string())),
                None => Err(LambdustError::type_error(format!(
                    "symbol->string: expected symbol, got {}",
                    args[0]
                ))),
            }
        },
    })
}

fn convert_string_to_symbol() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string->symbol".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match args[0].as_string() {
                Some(s) => Ok(Value::Symbol(s.to_string())),
                None => Err(LambdustError::type_error(format!(
                    "string->symbol: expected string, got {}",
                    args[0]
                ))),
            }
        },
    })
}

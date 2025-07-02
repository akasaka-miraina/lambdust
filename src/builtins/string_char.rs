//! String and character operations for Scheme

use crate::error::LambdustError;
use crate::lexer::SchemeNumber;
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register all string and character functions
pub fn register_string_char_functions(builtins: &mut HashMap<String, Value>) {
    // String operations
    builtins.insert("string-length".to_string(), string_length());
    builtins.insert("string-ref".to_string(), string_ref());
    builtins.insert("string-append".to_string(), string_append());
    builtins.insert("substring".to_string(), string_substring());
    builtins.insert("string=?".to_string(), string_equal());
    builtins.insert("string<?".to_string(), string_less_than());
    builtins.insert("string>?".to_string(), string_greater_than());
    builtins.insert("string<=?".to_string(), string_less_equal());
    builtins.insert("string>=?".to_string(), string_greater_equal());
    builtins.insert("make-string".to_string(), string_make());
    builtins.insert("string".to_string(), string_constructor());

    // Character operations
    builtins.insert("char=?".to_string(), char_equal());
    builtins.insert("char<?".to_string(), char_less_than());
    builtins.insert("char>?".to_string(), char_greater_than());
    builtins.insert("char<=?".to_string(), char_less_equal());
    builtins.insert("char>=?".to_string(), char_greater_equal());
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
    Value::Procedure(Procedure::Builtin {
        name: "string-length".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            match args[0].as_string() {
                Some(s) => Ok(Value::Number(SchemeNumber::Integer(s.chars().count() as i64))),
                None => Err(LambdustError::TypeError(format!(
                    "string-length: expected string, got {}", args[0]
                ))),
            }
        },
    })
}

fn string_ref() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-ref".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            let s = args[0].as_string().ok_or_else(|| {
                LambdustError::TypeError(format!("string-ref: expected string, got {}", args[0]))
            })?;
            
            let index = match args[1].as_number() {
                Some(SchemeNumber::Integer(i)) => *i as usize,
                _ => return Err(LambdustError::TypeError(format!(
                    "string-ref: expected integer index, got {}", args[1]
                ))),
            };
            
            let chars: Vec<char> = s.chars().collect();
            if index >= chars.len() {
                return Err(LambdustError::RuntimeError(format!(
                    "string-ref: index {} out of bounds for string of length {}", 
                    index, chars.len()
                )));
            }
            
            Ok(Value::Character(chars[index]))
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
                match arg.as_string() {
                    Some(s) => result.push_str(s),
                    None => return Err(LambdustError::TypeError(format!(
                        "string-append: expected string, got {}", arg
                    ))),
                }
            }
            Ok(Value::String(result))
        },
    })
}

fn string_substring() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "substring".to_string(),
        arity: None, // 2 or 3 arguments
        func: |args| {
            if args.len() < 2 || args.len() > 3 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            let s = args[0].as_string().ok_or_else(|| {
                LambdustError::TypeError(format!("substring: expected string, got {}", args[0]))
            })?;
            
            let start = match args[1].as_number() {
                Some(SchemeNumber::Integer(i)) => *i as usize,
                _ => return Err(LambdustError::TypeError(format!(
                    "substring: expected integer start, got {}", args[1]
                ))),
            };
            
            let chars: Vec<char> = s.chars().collect();
            let end = if args.len() == 3 {
                match args[2].as_number() {
                    Some(SchemeNumber::Integer(i)) => *i as usize,
                    _ => return Err(LambdustError::TypeError(format!(
                        "substring: expected integer end, got {}", args[2]
                    ))),
                }
            } else {
                chars.len()
            };
            
            if start > chars.len() || end > chars.len() || start > end {
                return Err(LambdustError::RuntimeError(format!(
                    "substring: invalid range [{}, {}) for string of length {}", 
                    start, end, chars.len()
                )));
            }
            
            let result: String = chars[start..end].iter().collect();
            Ok(Value::String(result))
        },
    })
}

fn string_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string=?".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            let first = args[0].as_string().ok_or_else(|| {
                LambdustError::TypeError(format!("string=?: expected string, got {}", args[0]))
            })?;
            
            for arg in &args[1..] {
                let s = arg.as_string().ok_or_else(|| {
                    LambdustError::TypeError(format!("string=?: expected string, got {}", arg))
                })?;
                if s != first {
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
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            for i in 0..args.len() - 1 {
                let current = args[i].as_string().ok_or_else(|| {
                    LambdustError::TypeError(format!("string<?: expected string, got {}", args[i]))
                })?;
                let next = args[i + 1].as_string().ok_or_else(|| {
                    LambdustError::TypeError(format!("string<?: expected string, got {}", args[i + 1]))
                })?;
                
                if current >= next {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn string_greater_than() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string>?".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            for i in 0..args.len() - 1 {
                let current = args[i].as_string().ok_or_else(|| {
                    LambdustError::TypeError(format!("string>?: expected string, got {}", args[i]))
                })?;
                let next = args[i + 1].as_string().ok_or_else(|| {
                    LambdustError::TypeError(format!("string>?: expected string, got {}", args[i + 1]))
                })?;
                
                if current <= next {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn string_less_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string<=?".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            for i in 0..args.len() - 1 {
                let current = args[i].as_string().ok_or_else(|| {
                    LambdustError::TypeError(format!("string<=?: expected string, got {}", args[i]))
                })?;
                let next = args[i + 1].as_string().ok_or_else(|| {
                    LambdustError::TypeError(format!("string<=?: expected string, got {}", args[i + 1]))
                })?;
                
                if current > next {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn string_greater_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string>=?".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            for i in 0..args.len() - 1 {
                let current = args[i].as_string().ok_or_else(|| {
                    LambdustError::TypeError(format!("string>=?: expected string, got {}", args[i]))
                })?;
                let next = args[i + 1].as_string().ok_or_else(|| {
                    LambdustError::TypeError(format!("string>=?: expected string, got {}", args[i + 1]))
                })?;
                
                if current < next {
                    return Ok(Value::Boolean(false));
                }
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
            
            let length = match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) if *n >= 0 => *n as usize,
                _ => return Err(LambdustError::TypeError(format!(
                    "make-string: expected non-negative integer, got {}", args[0]
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
                match arg {
                    Value::Character(c) => result.push(*c),
                    _ => return Err(LambdustError::TypeError(format!(
                        "string: expected character, got {}", arg
                    ))),
                }
            }
            Ok(Value::String(result))
        },
    })
}

// Character operations

fn char_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char=?".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            let first = match &args[0] {
                Value::Character(c) => *c,
                _ => return Err(LambdustError::TypeError(format!(
                    "char=?: expected character, got {}", args[0]
                ))),
            };
            
            for arg in &args[1..] {
                match arg {
                    Value::Character(c) => {
                        if *c != first {
                            return Ok(Value::Boolean(false));
                        }
                    }
                    _ => return Err(LambdustError::TypeError(format!(
                        "char=?: expected character, got {}", arg
                    ))),
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn char_less_than() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char<?".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            for i in 0..args.len() - 1 {
                let current = match &args[i] {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char<?: expected character, got {}", args[i]
                    ))),
                };
                let next = match &args[i + 1] {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char<?: expected character, got {}", args[i + 1]
                    ))),
                };
                
                if current >= next {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn char_greater_than() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char>?".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            for i in 0..args.len() - 1 {
                let current = match &args[i] {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char>?: expected character, got {}", args[i]
                    ))),
                };
                let next = match &args[i + 1] {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char>?: expected character, got {}", args[i + 1]
                    ))),
                };
                
                if current <= next {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn char_less_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char<=?".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            for i in 0..args.len() - 1 {
                let current = match &args[i] {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char<=?: expected character, got {}", args[i]
                    ))),
                };
                let next = match &args[i + 1] {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char<=?: expected character, got {}", args[i + 1]
                    ))),
                };
                
                if current > next {
                    return Ok(Value::Boolean(false));
                }
            }
            Ok(Value::Boolean(true))
        },
    })
}

fn char_greater_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char>=?".to_string(),
        arity: None, // At least 2 arguments
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            
            for i in 0..args.len() - 1 {
                let current = match &args[i] {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char>=?: expected character, got {}", args[i]
                    ))),
                };
                let next = match &args[i + 1] {
                    Value::Character(c) => *c,
                    _ => return Err(LambdustError::TypeError(format!(
                        "char>=?: expected character, got {}", args[i + 1]
                    ))),
                };
                
                if current < next {
                    return Ok(Value::Boolean(false));
                }
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
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            
            match &args[0] {
                Value::Character(c) => Ok(Value::Number(SchemeNumber::Integer(*c as u8 as i64))),
                _ => Err(LambdustError::TypeError(format!(
                    "char->integer: expected character, got {}", args[0]
                ))),
            }
        },
    })
}

fn integer_to_char() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "integer->char".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            
            match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) => {
                    if *n >= 0 && *n <= 127 {
                        Ok(Value::Character(*n as u8 as char))
                    } else {
                        Err(LambdustError::RuntimeError(format!(
                            "integer->char: value {} out of ASCII range", n
                        )))
                    }
                }
                _ => Err(LambdustError::TypeError(format!(
                    "integer->char: expected integer, got {}", args[0]
                ))),
            }
        },
    })
}

// Conversion functions

fn convert_char_to_string() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char->string".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            
            match &args[0] {
                Value::Character(c) => Ok(Value::String(c.to_string())),
                _ => Err(LambdustError::TypeError(format!(
                    "char->string: expected character, got {}", args[0]
                ))),
            }
        },
    })
}

fn convert_string_to_list() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string->list".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            
            match args[0].as_string() {
                Some(s) => {
                    let chars: Vec<Value> = s.chars().map(Value::Character).collect();
                    Ok(Value::from_vector(chars))
                }
                None => Err(LambdustError::TypeError(format!(
                    "string->list: expected string, got {}", args[0]
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
                return Err(LambdustError::ArityError(1, args.len()));
            }
            
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

fn convert_number_to_string() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "number->string".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            
            match args[0].as_number() {
                Some(n) => Ok(Value::String(n.to_string())),
                None => Err(LambdustError::TypeError(format!(
                    "number->string: expected number, got {}", args[0]
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
                return Err(LambdustError::ArityError(1, args.len()));
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
                None => Err(LambdustError::TypeError(format!(
                    "string->number: expected string, got {}", args[0]
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
                return Err(LambdustError::ArityError(1, args.len()));
            }
            
            match args[0].as_symbol() {
                Some(s) => Ok(Value::String(s.to_string())),
                None => Err(LambdustError::TypeError(format!(
                    "symbol->string: expected symbol, got {}", args[0]
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
                return Err(LambdustError::ArityError(1, args.len()));
            }
            
            match args[0].as_string() {
                Some(s) => Ok(Value::Symbol(s.to_string())),
                None => Err(LambdustError::TypeError(format!(
                    "string->symbol: expected string, got {}", args[0]
                ))),
            }
        },
    })
}
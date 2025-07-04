//! SRFI 13: String Libraries implementation
//!
//! This module implements the SRFI 13 String Libraries, providing
//! comprehensive string processing functions for R7RS Scheme.

use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register SRFI 13 functions into the builtins map
pub fn register_srfi_13_functions(builtins: &mut HashMap<String, Value>) {
    // String constructors and accessors
    builtins.insert("string-null?".to_string(), string_null_function());
    builtins.insert("string-every".to_string(), string_every_function());
    builtins.insert("string-any".to_string(), string_any_function());

    // String comparison
    builtins.insert("string-compare".to_string(), string_compare_function());
    builtins.insert(
        "string-compare-ci".to_string(),
        string_compare_ci_function(),
    );
    builtins.insert("string-hash".to_string(), string_hash_function());
    builtins.insert("string-hash-ci".to_string(), string_hash_ci_function());

    // String prefix & suffix
    builtins.insert("string-prefix?".to_string(), string_prefix_function());
    builtins.insert("string-suffix?".to_string(), string_suffix_function());
    builtins.insert("string-prefix-ci?".to_string(), string_prefix_ci_function());
    builtins.insert("string-suffix-ci?".to_string(), string_suffix_ci_function());

    // String search
    builtins.insert("string-index".to_string(), string_index_function());
    builtins.insert(
        "string-index-right".to_string(),
        string_index_right_function(),
    );
    builtins.insert("string-skip".to_string(), string_skip_function());
    builtins.insert(
        "string-skip-right".to_string(),
        string_skip_right_function(),
    );
    builtins.insert("string-count".to_string(), string_count_function());
    builtins.insert("string-contains".to_string(), string_contains_function());
    builtins.insert(
        "string-contains-ci".to_string(),
        string_contains_ci_function(),
    );

    // String modification
    builtins.insert("string-take".to_string(), string_take_function());
    builtins.insert("string-drop".to_string(), string_drop_function());
    builtins.insert(
        "string-take-right".to_string(),
        string_take_right_function(),
    );
    builtins.insert(
        "string-drop-right".to_string(),
        string_drop_right_function(),
    );
    builtins.insert("string-pad".to_string(), string_pad_function());
    builtins.insert("string-pad-right".to_string(), string_pad_right_function());
    builtins.insert("string-trim".to_string(), string_trim_function());
    builtins.insert(
        "string-trim-right".to_string(),
        string_trim_right_function(),
    );
    builtins.insert("string-trim-both".to_string(), string_trim_both_function());

    // String joining and splitting
    builtins.insert(
        "string-concatenate".to_string(),
        string_concatenate_function(),
    );
    builtins.insert(
        "string-concatenate-reverse".to_string(),
        string_concatenate_reverse_function(),
    );
    builtins.insert("string-join".to_string(), string_join_function());

    // String replacement
    builtins.insert("string-replace".to_string(), string_replace_function());

    // String tokenization
    builtins.insert("string-tokenize".to_string(), string_tokenize_function());

    // String filtering
    builtins.insert("string-filter".to_string(), string_filter_function());
    builtins.insert("string-delete".to_string(), string_delete_function());
}

/// Create string-null? function
fn string_null_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-null?".to_string(),
        arity: Some(1),
        func: string_null,
    })
}

/// String-null? - test if string is empty
pub fn string_null(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Boolean(s.is_empty())),
        _ => Err(LambdustError::type_error(
            "Argument must be a string".to_string(),
        )),
    }
}

/// Create string-every function
fn string_every_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-every".to_string(),
        arity: None, // Variable arity
        func: string_every,
    })
}

/// String-every - test if predicate is true for every character
pub fn string_every(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let _pred = &args[0];
    let string = &args[1];

    match string {
        Value::String(_s) => {
            // This would need evaluator integration to call the predicate
            Err(LambdustError::runtime_error(
                "string-every requires evaluator integration for predicate calls".to_string(),
            ))
        }
        _ => Err(LambdustError::type_error(
            "Second argument must be a string".to_string(),
        )),
    }
}

/// Create string-any function
fn string_any_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-any".to_string(),
        arity: None, // Variable arity
        func: string_any,
    })
}

/// String-any - test if predicate is true for any character
pub fn string_any(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let _pred = &args[0];
    let string = &args[1];

    match string {
        Value::String(_s) => {
            // This would need evaluator integration to call the predicate
            Err(LambdustError::runtime_error(
                "string-any requires evaluator integration for predicate calls".to_string(),
            ))
        }
        _ => Err(LambdustError::type_error(
            "Second argument must be a string".to_string(),
        )),
    }
}

/// Create string-compare function
fn string_compare_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-compare".to_string(),
        arity: Some(5), // s1, s2, proc<, proc=, proc>
        func: string_compare,
    })
}

/// String-compare - compare strings and call appropriate procedure
pub fn string_compare(args: &[Value]) -> Result<Value> {
    if args.len() != 5 {
        return Err(LambdustError::arity_error(5, args.len()));
    }

    let s1 = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let s2 = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be a string".to_string(),
            ));
        }
    };

    let _proc_lt = &args[2];
    let _proc_eq = &args[3];
    let _proc_gt = &args[4];

    // This would need evaluator integration to call the appropriate procedure
    match s1.cmp(s2) {
        std::cmp::Ordering::Less => Err(LambdustError::runtime_error(
            "string-compare requires evaluator integration for procedure calls".to_string(),
        )),
        std::cmp::Ordering::Equal => Err(LambdustError::runtime_error(
            "string-compare requires evaluator integration for procedure calls".to_string(),
        )),
        std::cmp::Ordering::Greater => Err(LambdustError::runtime_error(
            "string-compare requires evaluator integration for procedure calls".to_string(),
        )),
    }
}

/// Create string-compare-ci function
fn string_compare_ci_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-compare-ci".to_string(),
        arity: Some(5),
        func: string_compare_ci,
    })
}

/// String-compare-ci - case-insensitive string comparison
pub fn string_compare_ci(args: &[Value]) -> Result<Value> {
    if args.len() != 5 {
        return Err(LambdustError::arity_error(5, args.len()));
    }

    let s1 = match &args[0] {
        Value::String(s) => s.to_lowercase(),
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let s2 = match &args[1] {
        Value::String(s) => s.to_lowercase(),
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be a string".to_string(),
            ));
        }
    };

    let _proc_lt = &args[2];
    let _proc_eq = &args[3];
    let _proc_gt = &args[4];

    // This would need evaluator integration to call the appropriate procedure
    match s1.cmp(&s2) {
        std::cmp::Ordering::Less => Err(LambdustError::runtime_error(
            "string-compare-ci requires evaluator integration for procedure calls".to_string(),
        )),
        std::cmp::Ordering::Equal => Err(LambdustError::runtime_error(
            "string-compare-ci requires evaluator integration for procedure calls".to_string(),
        )),
        std::cmp::Ordering::Greater => Err(LambdustError::runtime_error(
            "string-compare-ci requires evaluator integration for procedure calls".to_string(),
        )),
    }
}

/// Create string-hash function
fn string_hash_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-hash".to_string(),
        arity: None, // 1 or 2 args
        func: string_hash,
    })
}

/// String-hash - compute hash of string
pub fn string_hash(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => {
                return Err(LambdustError::type_error(
                    "Second argument must be an integer".to_string(),
                ));
            }
        }
    } else {
        u32::MAX
    };

    // Simple hash implementation
    let mut hash: u32 = 0;
    for c in string.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(c as u32);
    }

    let result = if bound != u32::MAX {
        hash % bound
    } else {
        hash
    };

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
        result as i64,
    )))
}

/// Create string-hash-ci function
fn string_hash_ci_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-hash-ci".to_string(),
        arity: None, // 1 or 2 args
        func: string_hash_ci,
    })
}

/// String-hash-ci - case-insensitive hash of string
pub fn string_hash_ci(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s.to_lowercase(),
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let bound = if args.len() == 2 {
        match &args[1] {
            Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i as u32,
            Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as u32,
            _ => {
                return Err(LambdustError::type_error(
                    "Second argument must be an integer".to_string(),
                ));
            }
        }
    } else {
        u32::MAX
    };

    // Simple hash implementation on lowercase string
    let mut hash: u32 = 0;
    for c in string.chars() {
        hash = hash.wrapping_mul(31).wrapping_add(c as u32);
    }

    let result = if bound != u32::MAX {
        hash % bound
    } else {
        hash
    };

    Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
        result as i64,
    )))
}

/// Create string-prefix? function
fn string_prefix_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-prefix?".to_string(),
        arity: None, // 2-4 args
        func: string_prefix,
    })
}

/// String-prefix? - test if first string is prefix of second
pub fn string_prefix(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let s1 = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let s2 = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be a string".to_string(),
            ));
        }
    };

    // For simplicity, ignore optional start/end parameters for now
    Ok(Value::Boolean(s2.starts_with(s1)))
}

/// Create string-suffix? function
fn string_suffix_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-suffix?".to_string(),
        arity: None, // 2-4 args
        func: string_suffix,
    })
}

/// String-suffix? - test if first string is suffix of second
pub fn string_suffix(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let s1 = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let s2 = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be a string".to_string(),
            ));
        }
    };

    // For simplicity, ignore optional start/end parameters for now
    Ok(Value::Boolean(s2.ends_with(s1)))
}

/// Create string-prefix-ci? function
fn string_prefix_ci_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-prefix-ci?".to_string(),
        arity: None, // 2-4 args
        func: string_prefix_ci,
    })
}

/// String-prefix-ci? - case-insensitive prefix test
pub fn string_prefix_ci(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let s1 = match &args[0] {
        Value::String(s) => s.to_lowercase(),
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let s2 = match &args[1] {
        Value::String(s) => s.to_lowercase(),
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be a string".to_string(),
            ));
        }
    };

    Ok(Value::Boolean(s2.starts_with(&s1)))
}

/// Create string-suffix-ci? function
fn string_suffix_ci_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-suffix-ci?".to_string(),
        arity: None, // 2-4 args
        func: string_suffix_ci,
    })
}

/// String-suffix-ci? - case-insensitive suffix test
pub fn string_suffix_ci(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let s1 = match &args[0] {
        Value::String(s) => s.to_lowercase(),
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let s2 = match &args[1] {
        Value::String(s) => s.to_lowercase(),
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be a string".to_string(),
            ));
        }
    };

    Ok(Value::Boolean(s2.ends_with(&s1)))
}

// Placeholder functions for complex operations that need evaluator integration

/// Create string-index function (placeholder)
fn string_index_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-index".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-index requires evaluator integration".to_string(),
            ))
        },
    })
}

/// Create string-index-right function (placeholder)
fn string_index_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-index-right".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-index-right requires evaluator integration".to_string(),
            ))
        },
    })
}

/// Create string-skip function (placeholder)
fn string_skip_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-skip".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-skip requires evaluator integration".to_string(),
            ))
        },
    })
}

/// Create string-skip-right function (placeholder)
fn string_skip_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-skip-right".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-skip-right requires evaluator integration".to_string(),
            ))
        },
    })
}

/// Create string-count function (placeholder)
fn string_count_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-count".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-count requires evaluator integration".to_string(),
            ))
        },
    })
}

/// Create string-contains function
fn string_contains_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-contains".to_string(),
        arity: None, // 2-4 args
        func: string_contains,
    })
}

/// String-contains - find substring
pub fn string_contains(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let s1 = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let s2 = match &args[1] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be a string".to_string(),
            ));
        }
    };

    match s1.find(s2) {
        Some(index) => Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
            index as i64,
        ))),
        None => Ok(Value::Boolean(false)),
    }
}

/// Create string-contains-ci function
fn string_contains_ci_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-contains-ci".to_string(),
        arity: None, // 2-4 args
        func: string_contains_ci,
    })
}

/// String-contains-ci - case-insensitive substring search
pub fn string_contains_ci(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let s1 = match &args[0] {
        Value::String(s) => s.to_lowercase(),
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let s2 = match &args[1] {
        Value::String(s) => s.to_lowercase(),
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be a string".to_string(),
            ));
        }
    };

    match s1.find(&s2) {
        Some(index) => Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
            index as i64,
        ))),
        None => Ok(Value::Boolean(false)),
    }
}

// Simple string manipulation functions that don't need evaluator integration

/// Create string-take function
fn string_take_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-take".to_string(),
        arity: Some(2),
        func: string_take,
    })
}

/// String-take - take first n characters
pub fn string_take(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let n = match &args[1] {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be an integer".to_string(),
            ));
        }
    };

    if n < 0 {
        return Err(LambdustError::runtime_error(
            "Cannot take negative number of characters".to_string(),
        ));
    }

    let chars: Vec<char> = string.chars().collect();
    let take_count = std::cmp::min(n as usize, chars.len());
    let result: String = chars.into_iter().take(take_count).collect();

    Ok(Value::String(result))
}

/// Create string-drop function
fn string_drop_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-drop".to_string(),
        arity: Some(2),
        func: string_drop,
    })
}

/// String-drop - drop first n characters
pub fn string_drop(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let n = match &args[1] {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be an integer".to_string(),
            ));
        }
    };

    if n < 0 {
        return Err(LambdustError::runtime_error(
            "Cannot drop negative number of characters".to_string(),
        ));
    }

    let chars: Vec<char> = string.chars().collect();
    let drop_count = std::cmp::min(n as usize, chars.len());
    let result: String = chars.into_iter().skip(drop_count).collect();

    Ok(Value::String(result))
}

/// Create string-take-right function
fn string_take_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-take-right".to_string(),
        arity: Some(2),
        func: string_take_right,
    })
}

/// String-take-right - take last n characters
pub fn string_take_right(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let n = match &args[1] {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be an integer".to_string(),
            ));
        }
    };

    if n < 0 {
        return Err(LambdustError::runtime_error(
            "Cannot take negative number of characters".to_string(),
        ));
    }

    let chars: Vec<char> = string.chars().collect();
    let len = chars.len();
    let take_count = std::cmp::min(n as usize, len);
    let start = len.saturating_sub(take_count);
    let result: String = chars.into_iter().skip(start).collect();

    Ok(Value::String(result))
}

/// Create string-drop-right function
fn string_drop_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-drop-right".to_string(),
        arity: Some(2),
        func: string_drop_right,
    })
}

/// String-drop-right - drop last n characters
pub fn string_drop_right(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }

    let string = match &args[0] {
        Value::String(s) => s,
        _ => {
            return Err(LambdustError::type_error(
                "First argument must be a string".to_string(),
            ));
        }
    };

    let n = match &args[1] {
        Value::Number(crate::lexer::SchemeNumber::Integer(i)) => *i,
        Value::Number(crate::lexer::SchemeNumber::Real(f)) if f.fract() == 0.0 => *f as i64,
        _ => {
            return Err(LambdustError::type_error(
                "Second argument must be an integer".to_string(),
            ));
        }
    };

    if n < 0 {
        return Err(LambdustError::runtime_error(
            "Cannot drop negative number of characters".to_string(),
        ));
    }

    let chars: Vec<char> = string.chars().collect();
    let len = chars.len();
    let drop_count = std::cmp::min(n as usize, len);
    let take_count = len.saturating_sub(drop_count);
    let result: String = chars.into_iter().take(take_count).collect();

    Ok(Value::String(result))
}

// Placeholder functions for more complex operations

/// Create string-pad function (placeholder)
fn string_pad_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-pad".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-pad not yet implemented".to_string(),
            ))
        },
    })
}

/// Create string-pad-right function (placeholder)
fn string_pad_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-pad-right".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-pad-right not yet implemented".to_string(),
            ))
        },
    })
}

/// Create string-trim function (placeholder)
fn string_trim_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-trim".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-trim requires evaluator integration".to_string(),
            ))
        },
    })
}

/// Create string-trim-right function (placeholder)
fn string_trim_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-trim-right".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-trim-right requires evaluator integration".to_string(),
            ))
        },
    })
}

/// Create string-trim-both function (placeholder)
fn string_trim_both_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-trim-both".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-trim-both requires evaluator integration".to_string(),
            ))
        },
    })
}

/// Create string-concatenate function
fn string_concatenate_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-concatenate".to_string(),
        arity: Some(1),
        func: string_concatenate,
    })
}

/// String-concatenate - concatenate list of strings
pub fn string_concatenate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }

    let list = &args[0];
    if !list.is_list() {
        return Err(LambdustError::type_error(
            "Argument must be a list".to_string(),
        ));
    }

    let list_vec = list
        .to_vector()
        .ok_or_else(|| LambdustError::type_error("Argument must be a proper list"))?;

    let mut result = String::new();
    for item in list_vec {
        match item {
            Value::String(s) => result.push_str(&s),
            _ => {
                return Err(LambdustError::type_error(
                    "All list elements must be strings".to_string(),
                ));
            }
        }
    }

    Ok(Value::String(result))
}

/// Create string-concatenate-reverse function (placeholder)
fn string_concatenate_reverse_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-concatenate-reverse".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-concatenate-reverse not yet implemented".to_string(),
            ))
        },
    })
}

/// Create string-join function (placeholder)
fn string_join_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-join".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-join not yet implemented".to_string(),
            ))
        },
    })
}

/// Create string-replace function (placeholder)
fn string_replace_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-replace".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-replace not yet implemented".to_string(),
            ))
        },
    })
}

/// Create string-tokenize function (placeholder)
fn string_tokenize_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-tokenize".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-tokenize requires evaluator integration".to_string(),
            ))
        },
    })
}

/// Create string-filter function (placeholder)
fn string_filter_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-filter".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-filter requires evaluator integration".to_string(),
            ))
        },
    })
}

/// Create string-delete function (placeholder)
fn string_delete_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-delete".to_string(),
        arity: None,
        func: |_args| {
            Err(LambdustError::runtime_error(
                "string-delete requires evaluator integration".to_string(),
            ))
        },
    })
}

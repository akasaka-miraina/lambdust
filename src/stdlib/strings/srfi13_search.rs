//! SRFI-13 String Libraries - Search operations
//!
//! This module implements string searching functions from SRFI-13:
//! - string-index: Find first character matching criterion
//! - string-contains: Substring search
//! - string-tokenize: Split string by character set

use crate::diagnostics::{Error, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::stdlib::strings::common::{CharacterSet, extract_string, validate_string_bounds};
use std::sync::Arc;

/// Binds SRFI-13 search operations to the environment
pub fn bind_search_operations(env: &Arc<ThreadSafeEnvironment>) {
    // string-index - find first character matching criterion
    env.define(
        "string-index".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-index".to_string(),
            arity_min: 2,
            arity_max: Some(4), // string criterion [start] [end]
            implementation: PrimitiveImpl::RustFn(primitive_string_index),
            effects: vec![Effect::Pure],
        })),
    );

    // string-contains - substring search
    env.define(
        "string-contains".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-contains".to_string(),
            arity_min: 2,
            arity_max: Some(6), // s1 s2 [start1] [end1] [start2] [end2]
            implementation: PrimitiveImpl::RustFn(primitive_string_contains),
            effects: vec![Effect::Pure],
        })),
    );

    // string-tokenize - split by character set
    env.define(
        "string-tokenize".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-tokenize".to_string(),
            arity_min: 1,
            arity_max: Some(4), // string [token-set] [start] [end]
            implementation: PrimitiveImpl::RustFn(primitive_string_tokenize),
            effects: vec![Effect::Pure],
        })),
    );
}

/// string-index string criterion [start [end]]
/// Returns index of first character matching criterion, or #f
fn primitive_string_index(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(Error::runtime_error(
            format!("string-index expects 2-4 arguments, got {}", args.len()),
            None,
        )));
    }

    let string = extract_string(&args[0], "string-index")?;
    let criterion = extract_criterion(&args[1])?;

    let len = string.chars().count();
    let start = if args.len() > 2 {
        extract_start_index(&args[2], len)?
    } else {
        0
    };

    let end = if args.len() > 3 {
        extract_end_index(&args[3], len)?
    } else {
        len
    };

    validate_string_bounds(string, start, Some(end), "string-index")?;

    // Search for first matching character
    let chars: Vec<char> = string.chars().collect();
    for (i, &ch) in chars[start..end].iter().enumerate() {
        if criterion.contains(ch) {
            return Ok(Value::integer((start + i) as i64));
        }
    }

    Ok(Value::boolean(false)) // #f when not found
}

/// string-contains s1 s2 [start1 [end1 [start2 [end2]]]]
/// Returns starting index of s2 in s1, or #f
fn primitive_string_contains(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 6 {
        return Err(Box::new(Error::runtime_error(
            format!("string-contains expects 2-6 arguments, got {}", args.len()),
            None,
        )));
    }

    let s1 = extract_string(&args[0], "string-contains")?;
    let s2_full = extract_string(&args[1], "string-contains")?;

    let s1_len = s1.chars().count();
    let s2_full_len = s2_full.chars().count();

    let start1 = if args.len() > 2 {
        extract_start_index(&args[2], s1_len)?
    } else {
        0
    };

    let end1 = if args.len() > 3 {
        extract_end_index(&args[3], s1_len)?
    } else {
        s1_len
    };

    let start2 = if args.len() > 4 {
        extract_start_index(&args[4], s2_full_len)?
    } else {
        0
    };

    let end2 = if args.len() > 5 {
        extract_end_index(&args[5], s2_full_len)?
    } else {
        s2_full_len
    };

    validate_string_bounds(s1, start1, Some(end1), "string-contains")?;
    validate_string_bounds(s2_full, start2, Some(end2), "string-contains")?;

    // Extract substrings
    let s1_chars: Vec<char> = s1.chars().skip(start1).take(end1 - start1).collect();
    let s2_chars: Vec<char> = s2_full.chars().skip(start2).take(end2 - start2).collect();

    if s2_chars.is_empty() {
        return Ok(Value::integer(start1 as i64)); // Empty string matches at start
    }

    // Search for substring
    let s1_str: String = s1_chars.iter().collect();
    let s2_str: String = s2_chars.iter().collect();

    if let Some(pos) = s1_str.find(&s2_str) {
        let char_pos = s1_str[..pos].chars().count();
        Ok(Value::integer((start1 + char_pos) as i64))
    } else {
        Ok(Value::boolean(false))
    }
}

/// string-tokenize string [token-set [start [end]]]
/// Split string into tokens by character set
fn primitive_string_tokenize(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 4 {
        return Err(Box::new(Error::runtime_error(
            format!("string-tokenize expects 1-4 arguments, got {}", args.len()),
            None,
        )));
    }

    let string = extract_string(&args[0], "string-index")?;

    // Default token set is graphic characters (non-whitespace)
    let token_set = if args.len() > 1 {
        CharacterSet::from_value(&args[1])?
    } else {
        CharacterSet::Predicate(|c| !c.is_ascii_whitespace())
    };

    let len = string.chars().count();
    let start = if args.len() > 2 {
        extract_start_index(&args[2], len)?
    } else {
        0
    };

    let end = if args.len() > 3 {
        extract_end_index(&args[3], len)?
    } else {
        len
    };

    validate_string_bounds(string, start, Some(end), "string-index")?;

    // Tokenize the substring
    let chars: Vec<char> = string.chars().skip(start).take(end - start).collect();
    let mut tokens = Vec::new();
    let mut current_token = String::new();

    for ch in chars {
        if token_set.contains(ch) {
            current_token.push(ch);
        } else if !current_token.is_empty() {
            tokens.push(Value::string(&current_token));
            current_token.clear();
        }
    }

    // Add final token if exists
    if !current_token.is_empty() {
        tokens.push(Value::string(&current_token));
    }

    Ok(Value::list(tokens))
}

/// Extract search criterion from Scheme value
fn extract_criterion(value: &Value) -> Result<CharacterSet> {
    CharacterSet::from_value(value)
}

/// Extract start index from Scheme value
fn extract_start_index(value: &Value, max_len: usize) -> Result<usize> {
    match value {
        Value::Literal(literal) => {
            if let Some(n) = literal.to_i64() {
                if n < 0 {
                    return Err(Box::new(Error::runtime_error(
                        "Start index cannot be negative".to_string(),
                        None,
                    )));
                }
                let index = n as usize;
                if index > max_len {
                    return Err(Box::new(Error::runtime_error(
                        format!("Start index {} exceeds string length {}", index, max_len),
                        None,
                    )));
                }
                Ok(index)
            } else {
                Err(Box::new(Error::runtime_error(
                    "Start index must be an integer".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "Start index must be an integer".to_string(),
            None,
        ))),
    }
}

/// Extract end index from Scheme value
fn extract_end_index(value: &Value, max_len: usize) -> Result<usize> {
    match value {
        Value::Literal(literal) => {
            if let Some(n) = literal.to_i64() {
                if n < 0 {
                    return Err(Box::new(Error::runtime_error(
                        "End index cannot be negative".to_string(),
                        None,
                    )));
                }
                let index = n as usize;
                if index > max_len {
                    return Err(Box::new(Error::runtime_error(
                        format!("End index {} exceeds string length {}", index, max_len),
                        None,
                    )));
                }
                Ok(index)
            } else {
                Err(Box::new(Error::runtime_error(
                    "End index must be an integer".to_string(),
                    None,
                )))
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "End index must be an integer".to_string(),
            None,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_index_basic() {
        // string-index "hello" #\e => 1
        let args = vec![
            Value::string("hello"),
            Value::Literal(crate::ast::literal::Literal::Character('e')),
        ];
        let result = primitive_string_index(&args).unwrap();
        assert_eq!(result.as_integer(), Some(1));
    }

    #[test]
    fn test_string_index_not_found() {
        // string-index "hello" #\x => #f
        let args = vec![
            Value::string("hello"),
            Value::Literal(crate::ast::literal::Literal::Character('x')),
        ];
        let result = primitive_string_index(&args).unwrap();
        assert_eq!(result.as_boolean(), Some(false));
    }

    #[test]
    fn test_string_contains_basic() {
        // string-contains "hello world" "wor" => 6
        let args = vec![Value::string("hello world"), Value::string("wor")];
        let result = primitive_string_contains(&args).unwrap();
        assert_eq!(result.as_integer(), Some(6));
    }

    #[test]
    fn test_string_tokenize_basic() {
        // string-tokenize "hello world test"
        let args = vec![Value::string("hello world test")];
        let result = primitive_string_tokenize(&args).unwrap();

        let tokens = result.as_list().unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].as_string(), Some("hello"));
        assert_eq!(tokens[1].as_string(), Some("world"));
        assert_eq!(tokens[2].as_string(), Some("test"));
    }
}

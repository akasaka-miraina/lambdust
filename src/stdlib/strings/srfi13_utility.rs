//! SRFI-13 String Utility Operations
//!
//! This module provides SRFI-13 compliant string utility functions
//! for the Lambdust standard library.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::stdlib::strings::common::{CharacterSet, extract_string};
use std::sync::Arc;

/// Binds SRFI-13 string utility operations to the environment
pub fn bind_utility_operations(env: &Arc<ThreadSafeEnvironment>) {
    // string-pad
    env.define(
        "string-pad".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-pad".to_string(),
            arity_min: 2,
            arity_max: Some(4), // string, length, char?, start?
            implementation: PrimitiveImpl::RustFn(primitive_string_pad),
            effects: vec![Effect::Pure],
        })),
    );

    // string-pad-right
    env.define(
        "string-pad-right".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-pad-right".to_string(),
            arity_min: 2,
            arity_max: Some(4), // string, length, char?, start?
            implementation: PrimitiveImpl::RustFn(primitive_string_pad_right),
            effects: vec![Effect::Pure],
        })),
    );

    // string-trim
    env.define(
        "string-trim".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-trim".to_string(),
            arity_min: 1,
            arity_max: Some(3), // string, char-set?, start?
            implementation: PrimitiveImpl::RustFn(primitive_string_trim),
            effects: vec![Effect::Pure],
        })),
    );

    // string-trim-right
    env.define(
        "string-trim-right".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-trim-right".to_string(),
            arity_min: 1,
            arity_max: Some(3), // string, char-set?, start?
            implementation: PrimitiveImpl::RustFn(primitive_string_trim_right),
            effects: vec![Effect::Pure],
        })),
    );

    // string-trim-both
    env.define(
        "string-trim-both".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-trim-both".to_string(),
            arity_min: 1,
            arity_max: Some(3), // string, char-set?, start?
            implementation: PrimitiveImpl::RustFn(primitive_string_trim_both),
            effects: vec![Effect::Pure],
        })),
    );

    // string-replace
    env.define(
        "string-replace".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-replace".to_string(),
            arity_min: 4,
            arity_max: Some(6), // string1, string2, start1, end1, start2?, end2?
            implementation: PrimitiveImpl::RustFn(primitive_string_replace),
            effects: vec![Effect::Pure],
        })),
    );

    // string-reverse
    env.define(
        "string-reverse".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "string-reverse".to_string(),
            arity_min: 1,
            arity_max: Some(3), // string, start?, end?
            implementation: PrimitiveImpl::RustFn(primitive_string_reverse),
            effects: vec![Effect::Pure],
        })),
    );
}

/// Extract padding character from arguments, default to space
fn extract_pad_char(args: &[Value], index: usize) -> Result<char> {
    if index >= args.len() {
        return Ok(' '); // default to space
    }

    match &args[index] {
        Value::Literal(crate::ast::literal::Literal::Character(ch)) => Ok(*ch),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            "padding character must be a character".to_string(),
            None,
        ))),
    }
}

/// Extract integer argument with bounds checking
fn extract_bounded_integer(value: &Value, name: &str, min: i64, max: Option<i64>) -> Result<usize> {
    let n = value.as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(format!("{} must be an integer", name), None)
    })?;

    if n < min {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("{} must be >= {}", name, min),
            None,
        )));
    }

    if let Some(max_val) = max {
        if n > max_val {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!("{} must be <= {}", name, max_val),
                None,
            )));
        }
    }

    Ok(n as usize)
}

/// string-pad - Pad string to specified length on the left
fn primitive_string_pad(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-pad expects 2-4 arguments, got {}", args.len()),
            None,
        )));
    }

    let s = extract_string(&args[0], "string-pad")?;
    let target_len = extract_bounded_integer(&args[1], "length", 0, None)?;
    let pad_char = if args.len() > 2 {
        extract_pad_char(args, 2)?
    } else {
        ' '
    };
    let start = if args.len() > 3 {
        extract_bounded_integer(&args[3], "start", 0, Some(s.len() as i64))?
    } else {
        0
    };

    let chars: Vec<char> = s.chars().collect();
    let substring: String = chars[start..].iter().collect();
    let current_len = substring.chars().count();

    if current_len >= target_len {
        // Truncate if too long
        let truncated: String = substring.chars().take(target_len).collect();
        Ok(Value::string(truncated))
    } else {
        // Pad on the left
        let padding_needed = target_len - current_len;
        let padding: String = pad_char.to_string().repeat(padding_needed);
        Ok(Value::string(format!("{}{}", padding, substring)))
    }
}

/// string-pad-right - Pad string to specified length on the right
fn primitive_string_pad_right(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-pad-right expects 2-4 arguments, got {}", args.len()),
            None,
        )));
    }

    let s = extract_string(&args[0], "string-pad-right")?;
    let target_len = extract_bounded_integer(&args[1], "length", 0, None)?;
    let pad_char = if args.len() > 2 {
        extract_pad_char(args, 2)?
    } else {
        ' '
    };
    let start = if args.len() > 3 {
        extract_bounded_integer(&args[3], "start", 0, Some(s.len() as i64))?
    } else {
        0
    };

    let chars: Vec<char> = s.chars().collect();
    let substring: String = chars[start..].iter().collect();
    let current_len = substring.chars().count();

    if current_len >= target_len {
        // Truncate if too long
        let truncated: String = substring.chars().take(target_len).collect();
        Ok(Value::string(truncated))
    } else {
        // Pad on the right
        let padding_needed = target_len - current_len;
        let padding: String = pad_char.to_string().repeat(padding_needed);
        Ok(Value::string(format!("{}{}", substring, padding)))
    }
}

/// Extract criterion for trimming (character set or predicate)
fn extract_trim_criterion(args: &[Value], index: usize) -> Result<CharacterSet> {
    if index >= args.len() {
        // Default to whitespace character set
        let whitespace_chars: Vec<char> = " \t\n\r\u{000B}\u{000C}".chars().collect();
        return Ok(CharacterSet::String(whitespace_chars.into_iter().collect()));
    }

    match &args[index] {
        Value::Literal(crate::ast::literal::Literal::Character(ch)) => {
            Ok(CharacterSet::Character(*ch))
        }
        _ => {
            // Try to interpret as character set
            CharacterSet::from_value(&args[index])
        }
    }
}

/// string-trim - Remove characters from the left
fn primitive_string_trim(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-trim expects 1-3 arguments, got {}", args.len()),
            None,
        )));
    }

    let s = extract_string(&args[0], "string-trim")?;
    let criterion = if args.len() > 1 {
        extract_trim_criterion(args, 1)?
    } else {
        let whitespace_chars: Vec<char> = " \t\n\r\u{000B}\u{000C}".chars().collect();
        CharacterSet::String(whitespace_chars.into_iter().collect())
    };
    let start = if args.len() > 2 {
        extract_bounded_integer(&args[2], "start", 0, Some(s.len() as i64))?
    } else {
        0
    };

    let chars: Vec<char> = s.chars().collect();
    let mut trim_start = start;

    while trim_start < chars.len() && criterion.contains(chars[trim_start]) {
        trim_start += 1;
    }

    let result: String = chars[trim_start..].iter().collect();
    Ok(Value::string(result))
}

/// string-trim-right - Remove characters from the right
fn primitive_string_trim_right(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "string-trim-right expects 1-3 arguments, got {}",
                args.len()
            ),
            None,
        )));
    }

    let s = extract_string(&args[0], "string-trim-right")?;
    let criterion = if args.len() > 1 {
        extract_trim_criterion(args, 1)?
    } else {
        let whitespace_chars: Vec<char> = " \t\n\r\u{000B}\u{000C}".chars().collect();
        CharacterSet::String(whitespace_chars.into_iter().collect())
    };
    let start = if args.len() > 2 {
        extract_bounded_integer(&args[2], "start", 0, Some(s.len() as i64))?
    } else {
        0
    };

    let chars: Vec<char> = s.chars().collect();
    let mut trim_end = chars.len();

    while trim_end > start && criterion.contains(chars[trim_end - 1]) {
        trim_end -= 1;
    }

    let result: String = chars[start..trim_end].iter().collect();
    Ok(Value::string(result))
}

/// string-trim-both - Remove characters from both ends
fn primitive_string_trim_both(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-trim-both expects 1-3 arguments, got {}", args.len()),
            None,
        )));
    }

    // First trim from the left, then from the right
    let left_trimmed = primitive_string_trim(args)?;
    let left_trimmed_str = left_trimmed.as_string().unwrap();

    // Reconstruct args for right trimming
    let right_trim_args = if args.len() > 1 {
        vec![Value::string(left_trimmed_str), args[1].clone()]
    } else {
        vec![Value::string(left_trimmed_str)]
    };

    primitive_string_trim_right(&right_trim_args)
}

/// string-replace - Replace substring with another string
fn primitive_string_replace(args: &[Value]) -> Result<Value> {
    if args.len() < 4 || args.len() > 6 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-replace expects 4-6 arguments, got {}", args.len()),
            None,
        )));
    }

    let s1 = extract_string(&args[0], "string-replace")?;
    let s2 = extract_string(&args[1], "string-replace")?;
    let start1 = extract_bounded_integer(&args[2], "start1", 0, Some(s1.len() as i64))?;
    let end1 = extract_bounded_integer(&args[3], "end1", start1 as i64, Some(s1.len() as i64))?;
    let start2 = if args.len() > 4 {
        extract_bounded_integer(&args[4], "start2", 0, Some(s2.len() as i64))?
    } else {
        0
    };
    let end2 = if args.len() > 5 {
        extract_bounded_integer(&args[5], "end2", start2 as i64, Some(s2.len() as i64))?
    } else {
        s2.len()
    };

    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();

    // Build result: prefix + replacement + suffix
    let prefix: String = chars1[..start1].iter().collect();
    let replacement: String = chars2[start2..end2].iter().collect();
    let suffix: String = chars1[end1..].iter().collect();

    Ok(Value::string(format!(
        "{}{}{}",
        prefix, replacement, suffix
    )))
}

/// string-reverse - Reverse string or substring
fn primitive_string_reverse(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-reverse expects 1-3 arguments, got {}", args.len()),
            None,
        )));
    }

    let s = extract_string(&args[0], "string-reverse")?;
    let start = if args.len() > 1 {
        extract_bounded_integer(&args[1], "start", 0, Some(s.len() as i64))?
    } else {
        0
    };
    let end = if args.len() > 2 {
        extract_bounded_integer(&args[2], "end", start as i64, Some(s.len() as i64))?
    } else {
        s.len()
    };

    let chars: Vec<char> = s.chars().collect();
    let mut result_chars = chars[..start].to_vec();

    // Reverse the specified range
    let mut reversed_range: Vec<char> = chars[start..end].to_vec();
    reversed_range.reverse();
    result_chars.extend(reversed_range);

    // Add the suffix
    result_chars.extend(&chars[end..]);

    let result: String = result_chars.iter().collect();
    Ok(Value::string(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::literal::Literal;

    #[test]
    fn test_string_pad() {
        // Basic padding
        let args = vec![Value::string("hello"), Value::integer(10)];
        let result = primitive_string_pad(&args).unwrap();
        assert_eq!(result.as_string(), Some("     hello"));

        // Custom padding character
        let args = vec![
            Value::string("hello"),
            Value::integer(10),
            Value::Literal(Literal::Character('*')),
        ];
        let result = primitive_string_pad(&args).unwrap();
        assert_eq!(result.as_string(), Some("*****hello"));

        // Truncation when too long
        let args = vec![Value::string("hello world"), Value::integer(5)];
        let result = primitive_string_pad(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello"));
    }

    #[test]
    fn test_string_pad_right() {
        // Basic right padding
        let args = vec![Value::string("hello"), Value::integer(10)];
        let result = primitive_string_pad_right(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello     "));

        // Custom padding character
        let args = vec![
            Value::string("hello"),
            Value::integer(10),
            Value::Literal(Literal::Character('-')),
        ];
        let result = primitive_string_pad_right(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello-----"));
    }

    #[test]
    fn test_string_trim() {
        // Basic whitespace trimming
        let args = vec![Value::string("   hello world   ")];
        let result = primitive_string_trim(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello world   "));

        // Custom character trimming
        let args = vec![
            Value::string("***hello***"),
            Value::Literal(Literal::Character('*')),
        ];
        let result = primitive_string_trim(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello***"));
    }

    #[test]
    fn test_string_trim_right() {
        // Basic whitespace trimming
        let args = vec![Value::string("   hello world   ")];
        let result = primitive_string_trim_right(&args).unwrap();
        assert_eq!(result.as_string(), Some("   hello world"));

        // Custom character trimming
        let args = vec![
            Value::string("***hello***"),
            Value::Literal(Literal::Character('*')),
        ];
        let result = primitive_string_trim_right(&args).unwrap();
        assert_eq!(result.as_string(), Some("***hello"));
    }

    #[test]
    fn test_string_trim_both() {
        // Basic whitespace trimming
        let args = vec![Value::string("   hello world   ")];
        let result = primitive_string_trim_both(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello world"));

        // Custom character trimming
        let args = vec![
            Value::string("***hello***"),
            Value::Literal(Literal::Character('*')),
        ];
        let result = primitive_string_trim_both(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello"));
    }

    #[test]
    fn test_string_replace() {
        // Basic replacement
        let args = vec![
            Value::string("hello world"),
            Value::string("REPLACED"),
            Value::integer(6),  // start1: replace "world"
            Value::integer(11), // end1
        ];
        let result = primitive_string_replace(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello REPLACED"));

        // Partial replacement string
        let args = vec![
            Value::string("hello world"),
            Value::string("ABCDEF"),
            Value::integer(6),  // start1: replace "world"
            Value::integer(11), // end1
            Value::integer(1),  // start2: use "BCD" from "ABCDEF"
            Value::integer(4),  // end2
        ];
        let result = primitive_string_replace(&args).unwrap();
        assert_eq!(result.as_string(), Some("hello BCD"));
    }

    #[test]
    fn test_string_reverse() {
        // Full string reverse
        let args = vec![Value::string("hello")];
        let result = primitive_string_reverse(&args).unwrap();
        assert_eq!(result.as_string(), Some("olleh"));

        // Partial reverse
        let args = vec![
            Value::string("hello world"),
            Value::integer(2), // start
            Value::integer(7), // end - reverse "llo w"
        ];
        let result = primitive_string_reverse(&args).unwrap();
        assert_eq!(result.as_string(), Some("hew ollorld"));

        // Empty string
        let args = vec![Value::string("")];
        let result = primitive_string_reverse(&args).unwrap();
        assert_eq!(result.as_string(), Some(""));
    }

    #[test]
    fn test_utility_arity_errors() {
        // Test wrong number of arguments
        let result = primitive_string_pad(&[Value::string("test")]);
        assert!(result.is_err());

        let result = primitive_string_trim(&[]);
        assert!(result.is_err());

        let result = primitive_string_replace(&[Value::string("a"), Value::string("b")]);
        assert!(result.is_err());
    }

    #[test]
    fn test_utility_type_errors() {
        // Test with wrong argument types
        let result = primitive_string_pad(&[Value::integer(42), Value::integer(10)]);
        assert!(result.is_err());

        let result = primitive_string_trim(&[Value::boolean(true)]);
        assert!(result.is_err());
    }
}

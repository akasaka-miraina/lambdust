//! SRFI-28: Basic format strings
//!
//! This module provides SRFI-28 compliant basic format string functionality.
//! SRFI-28 defines a simple format procedure for basic string formatting.
//!
//! ## Supported format directives:
//! - `~a` - Any object (display representation)
//! - `~s` - S-expression (write representation)  
//! - `~d` - Decimal integer
//! - `~%` - Newline
//! - `~~` - Literal tilde
//!
//! ## Examples:
//! ```scheme
//! (format "~a + ~a = ~d" 1 2 3)    ; => "1 + 2 = 3"
//! (format "Hello, ~a!~%" "World") ; => "Hello, World!\n"
//! (format "String: ~s" "text")    ; => "String: \"text\""
//! ```

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use std::sync::Arc;

/// Initialize SRFI-28 format functionality in the environment.
pub fn init_srfi28_format(env: &Arc<ThreadSafeEnvironment>) {
    env.define(
        "format".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "format".to_string(),
            arity_min: 1,
            arity_max: None, // Variable number of arguments
            implementation: PrimitiveImpl::RustFn(primitive_format),
            effects: vec![Effect::Pure],
        })),
    );
}

/// SRFI-28 format procedure implementation
fn primitive_format(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "format expects at least 1 argument (format string)".to_string(),
            None,
        )));
    }

    // Extract format string
    let format_string = match &args[0] {
        Value::Literal(literal) => match literal {
            crate::ast::literal::Literal::String(s) => s.as_ref(),
            crate::ast::literal::Literal::InternedString(s) => s.as_str(),
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "format: first argument must be a string".to_string(),
                    None,
                )));
            }
        },
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                "format: first argument must be a string".to_string(),
                None,
            )));
        }
    };

    // Format arguments (skip the format string)
    let format_args = &args[1..];

    // Process format string
    match format_string_with_args(format_string, format_args) {
        Ok(result) => Ok(Value::string(result)),
        Err(e) => Err(Box::new(DiagnosticError::runtime_error(
            format!("format: {}", e),
            None,
        ))),
    }
}

/// Process format string with arguments
fn format_string_with_args(
    format_str: &str,
    args: &[Value],
) -> std::result::Result<String, String> {
    let mut result = String::new();
    let mut chars = format_str.chars().peekable();
    let mut arg_index = 0;

    while let Some(ch) = chars.next() {
        if ch == '~' {
            if let Some(&next_ch) = chars.peek() {
                chars.next(); // consume the directive character
                match next_ch {
                    'a' => {
                        // Any object - display representation
                        if arg_index >= args.len() {
                            return Err("not enough arguments for format directive ~a".to_string());
                        }
                        result.push_str(&args[arg_index].display_string());
                        arg_index += 1;
                    }
                    's' => {
                        // S-expression - write representation
                        if arg_index >= args.len() {
                            return Err("not enough arguments for format directive ~s".to_string());
                        }
                        result.push_str(&format!("{}", args[arg_index]));
                        arg_index += 1;
                    }
                    'd' => {
                        // Decimal integer
                        if arg_index >= args.len() {
                            return Err("not enough arguments for format directive ~d".to_string());
                        }
                        match &args[arg_index] {
                            Value::Literal(literal) => match literal {
                                crate::ast::literal::Literal::ExactInteger(n) => {
                                    result.push_str(&n.to_string());
                                }
                                crate::ast::literal::Literal::InexactReal(r) => {
                                    // Convert float to integer representation
                                    result.push_str(&(*r as i64).to_string());
                                }
                                _ => {
                                    return Err(format!(
                                        "format ~d expects a number, got: {}",
                                        args[arg_index]
                                    ));
                                }
                            },
                            _ => {
                                return Err(format!(
                                    "format ~d expects a number, got: {}",
                                    args[arg_index]
                                ));
                            }
                        }
                        arg_index += 1;
                    }
                    '%' => {
                        // Newline
                        result.push('\n');
                    }
                    '~' => {
                        // Literal tilde
                        result.push('~');
                    }
                    _ => {
                        return Err(format!("unknown format directive: ~{}", next_ch));
                    }
                }
            } else {
                return Err("incomplete format directive: ~ at end of string".to_string());
            }
        } else {
            result.push(ch);
        }
    }

    // Check for unused arguments
    if arg_index < args.len() {
        return Err("too many arguments for format string".to_string());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::literal::Literal;

    #[test]
    fn test_format_basic_any() {
        let args = vec![
            Value::string("Hello ~a!".to_string()),
            Value::string("World".to_string()),
        ];
        let result = primitive_format(&args).unwrap();
        assert_eq!(result.as_string(), Some("Hello World!"));
    }

    #[test]
    fn test_format_s_expression() {
        let args = vec![
            Value::string("Value: ~s".to_string()),
            Value::string("test".to_string()),
        ];
        let result = primitive_format(&args).unwrap();
        assert_eq!(result.as_string(), Some("Value: \"test\""));
    }

    #[test]
    fn test_format_decimal() {
        let args = vec![
            Value::string("Number: ~d".to_string()),
            Value::Literal(Literal::ExactInteger(42)),
        ];
        let result = primitive_format(&args).unwrap();
        assert_eq!(result.as_string(), Some("Number: 42"));
    }

    #[test]
    fn test_format_newline() {
        let args = vec![Value::string("Line 1~%Line 2".to_string())];
        let result = primitive_format(&args).unwrap();
        assert_eq!(result.as_string(), Some("Line 1\nLine 2"));
    }

    #[test]
    fn test_format_literal_tilde() {
        let args = vec![Value::string("Tilde: ~~".to_string())];
        let result = primitive_format(&args).unwrap();
        assert_eq!(result.as_string(), Some("Tilde: ~"));
    }

    #[test]
    fn test_format_multiple_directives() {
        let args = vec![
            Value::string("~a + ~a = ~d~%".to_string()),
            Value::Literal(Literal::ExactInteger(1)),
            Value::Literal(Literal::ExactInteger(2)),
            Value::Literal(Literal::ExactInteger(3)),
        ];
        let result = primitive_format(&args).unwrap();
        assert_eq!(result.as_string(), Some("1 + 2 = 3\n"));
    }

    #[test]
    fn test_format_error_not_enough_args() {
        let args = vec![
            Value::string("~a ~a".to_string()),
            Value::string("only one".to_string()),
        ];
        let result = primitive_format(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_error_too_many_args() {
        let args = vec![
            Value::string("~a".to_string()),
            Value::string("arg1".to_string()),
            Value::string("arg2".to_string()),
        ];
        let result = primitive_format(&args);
        assert!(result.is_err());
    }
}

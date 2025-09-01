//! SRFI-115 Standard Library Procedures
//!
//! Implementation of all SRFI-115 procedures for working with symbolic regular expressions.

use crate::diagnostics::Result;
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::stdlib::srfi115_regex::{
    get_regex_engine,
    compiler::CompilationFlags,
    match_object::MatchObject,
    error::{RegexError, RegexResult},
};
use std::collections::HashMap;
use std::sync::Arc;

/// Implementation of `(regexp sre [flags])`.
pub fn primitive_regexp(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp expects 1 or 2 arguments",
            None
        )));
    }

    let sre = &args[0];
    let flags = if args.len() > 1 {
        Some(&args[1])
    } else {
        None
    };

    let engine_mutex = get_regex_engine();
    let mut engine = engine_mutex.lock()
        .map_err(|_| Box::new(crate::diagnostics::Error::runtime_error(
            "Failed to acquire regex engine lock",
            None
        )))?;
    let _compiled = engine.compile(sre, flags)
        .map_err(|e| Box::new(crate::diagnostics::Error::runtime_error(
            format!("regex compilation failed: {}", e),
            None
        )))?;
    
    // Store compiled regex in a custom extension type
    // For now, use string representation
    Ok(Value::string(format!("#<regexp {:?}>", sre)))
}

/// Implementation of `(regexp? obj)`.
pub fn primitive_regexp_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp? expects exactly 1 argument",
            None
        )));
    }

    let is_regexp = match &args[0] {
        v if v.is_string() => {
            if let Some(s) = v.as_string() {
                s.starts_with("#<regexp")
            } else {
                false
            }
        }
        _ => false,
    };

    Ok(Value::boolean(is_regexp))
}

/// Implementation of `(regexp-matches regexp string [start] [end])`.
pub fn primitive_regexp_matches(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp-matches expects 2-4 arguments",
            None
        )));
    }

    let _regexp_obj = &args[0];
    let text = extract_string(&args[1])?;
    
    let _start = if args.len() > 2 {
        Some(extract_integer(&args[2])? as usize)
    } else {
        None
    };
    
    let _end = if args.len() > 3 {
        Some(extract_integer(&args[3])? as usize)
    } else {
        None
    };

    // TODO: Extract actual compiled regex and perform matching
    // For now, return false indicating no match
    Ok(Value::boolean(false))
}

/// Implementation of `(regexp-search regexp string [start] [end])`.
pub fn primitive_regexp_search(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp-search expects 2-4 arguments",
            None
        )));
    }

    let _regexp_obj = &args[0];
    let _text = extract_string(&args[1])?;
    
    let _start = if args.len() > 2 {
        Some(extract_integer(&args[2])? as usize)
    } else {
        None
    };
    
    let _end = if args.len() > 3 {
        Some(extract_integer(&args[3])? as usize)
    } else {
        None
    };

    // TODO: Extract actual compiled regex and perform search
    // For now, return false indicating no match found
    Ok(Value::boolean(false))
}

/// Implementation of `(regexp-replace regexp string replacement [start] [end])`.
pub fn primitive_regexp_replace(args: &[Value]) -> Result<Value> {
    if args.len() < 3 || args.len() > 5 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp-replace expects 3-5 arguments",
            None
        )));
    }

    let _regexp_obj = &args[0];
    let string = extract_string(&args[1])?;
    let _replacement = extract_string(&args[2])?;
    
    let _start = if args.len() > 3 {
        extract_integer(&args[3])? as usize
    } else {
        0
    };
    
    let _end = if args.len() > 4 {
        extract_integer(&args[4])? as usize
    } else {
        string.len()
    };

    // TODO: Extract actual regexp and perform replacement
    // For now, just return the original string
    Ok(Value::string(string))
}

/// Implementation of `(regexp-replace-all regexp string replacement [start] [end])`.
pub fn primitive_regexp_replace_all(args: &[Value]) -> Result<Value> {
    if args.len() < 3 || args.len() > 5 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp-replace-all expects 3-5 arguments",
            None
        )));
    }

    // For now, delegate to regexp-replace
    primitive_regexp_replace(args)
}

/// Implementation of `(regexp-split regexp string [start] [end] [count])`.
pub fn primitive_regexp_split(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 5 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp-split expects 2-5 arguments",
            None
        )));
    }

    let _regexp_obj = &args[0];
    let string = extract_string(&args[1])?;
    
    let start = if args.len() > 2 {
        extract_integer(&args[2])? as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        extract_integer(&args[3])? as usize
    } else {
        string.len()
    };
    
    let _count = if args.len() > 4 {
        Some(extract_integer(&args[4])? as usize)
    } else {
        None
    };

    if start > end || end > string.len() {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "Invalid start/end positions",
            None
        )));
    }

    // TODO: Extract actual regexp and perform split
    // For now, return the input string as a single-element list
    Ok(Value::list(vec![Value::string(string[start..end].to_string())]))
}

/// Implementation of `(regexp-match? obj)`.
pub fn primitive_regexp_match_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp-match? expects exactly 1 argument",
            None
        )));
    }

    let is_match = match &args[0] {
        v if v.is_string() => {
            if let Some(s) = v.as_string() {
                s.starts_with("#<match")
            } else {
                false
            }
        }
        _ => false,
    };

    Ok(Value::boolean(is_match))
}

/// Implementation of `(regexp-match-count match)`.
pub fn primitive_regexp_match_count(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp-match-count expects exactly 1 argument",
            None
        )));
    }

    // TODO: Extract actual match object and return submatch count
    // For now, return 0
    Ok(Value::integer(0))
}

/// Implementation of `(regexp-match-submatch match n)`.
pub fn primitive_regexp_match_submatch(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp-match-submatch expects exactly 2 arguments",
            None
        )));
    }

    let _match_obj = &args[0];
    let n = extract_integer(&args[1])?;

    if n < 1 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "Submatch index must be positive",
            None
        )));
    }

    // TODO: Extract actual match object and return submatch
    // For now, return #f (no submatch)
    Ok(Value::boolean(false))
}

/// Implementation of `(regexp-match-submatch-start match n)`.
pub fn primitive_regexp_match_submatch_start(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp-match-submatch-start expects exactly 2 arguments",
            None
        )));
    }

    let _match_obj = &args[0];
    let n = extract_integer(&args[1])?;

    if n < 1 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "Submatch index must be positive",
            None
        )));
    }

    // TODO: Extract actual match object and return submatch start position
    // For now, return #f
    Ok(Value::boolean(false))
}

/// Implementation of `(regexp-match-submatch-end match n)`.
pub fn primitive_regexp_match_submatch_end(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp-match-submatch-end expects exactly 2 arguments",
            None
        )));
    }

    let _match_obj = &args[0];
    let n = extract_integer(&args[1])?;

    if n < 1 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "Submatch index must be positive",
            None
        )));
    }

    // TODO: Extract actual match object and return submatch end position
    // For now, return #f
    Ok(Value::boolean(false))
}

/// Implementation of `(regexp-match-named-submatch match name)`.
pub fn primitive_regexp_match_named_submatch(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "regexp-match-named-submatch expects exactly 2 arguments",
            None
        )));
    }

    let _match_obj = &args[0];
    let _name = match &args[1] {
        v if v.is_string() => v.as_string().unwrap_or_default().to_string(),
        Value::Symbol(s) => crate::utils::symbol_name(*s).unwrap_or_default(),
        _ => return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "Submatch name must be a string or symbol",
            None
        ))),
    };

    // TODO: Extract actual match object and return named submatch
    // For now, return #f
    Ok(Value::boolean(false))
}

/// Helper function to extract a string from a Value.
fn extract_string(value: &Value) -> Result<String> {
    match value {
        v if v.is_string() => Ok(v.as_string().unwrap_or_default().to_string()),
        _ => Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("Expected string, got {:?}", value),
            None
        ))),
    }
}

/// Helper function to extract an integer from a Value.
fn extract_integer(value: &Value) -> Result<i64> {
    match value {
        v if v.is_number() => {
            if let Some(int_val) = v.as_integer() {
                Ok(int_val)
            } else {
                Err(Box::new(crate::diagnostics::Error::runtime_error(
                    "Number is not an integer",
                    None
                )))
            }
        }
        _ => Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("Expected integer, got {:?}", value),
            None
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regexp_creation() {
        let args = vec![Value::string("hello")];
        let result = primitive_regexp(&args).unwrap();
        
        match result {
            v if v.is_string() => {
                if let Some(s) = v.as_string() {
                    assert!(s.starts_with("#<regexp"));
                } else {
                    panic!("Expected string value");
                }
            },
            _ => panic!("Expected regexp object"),
        }
    }

    #[test]
    fn test_regexp_predicate() {
        let regexp_obj = Value::string("#<regexp \"hello\">");
        let result = primitive_regexp_p(&[regexp_obj]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let not_regexp = Value::string("hello");
        let result = primitive_regexp_p(&[not_regexp]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_argument_validation() {
        // Test too few arguments
        let result = primitive_regexp(&[]);
        assert!(result.is_err());
        
        // Test too many arguments
        let result = primitive_regexp(&[
            Value::string("hello"),
            Value::integer(1),
            Value::integer(2),
        ]);
        assert!(result.is_err());
        
        // Test correct number of arguments
        let result = primitive_regexp(&[Value::string("hello")]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_extraction() {
        assert_eq!(extract_string(&Value::string("hello")).unwrap(), "hello");
        assert!(extract_string(&Value::integer(42)).is_err());
    }

    #[test]
    fn test_integer_extraction() {
        assert_eq!(extract_integer(&Value::integer(42)).unwrap(), 42);
        assert!(extract_integer(&Value::string("hello")).is_err());
    }
}
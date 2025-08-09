//! Character operations for the Lambdust standard library.
//!
//! This module implements R7RS-compliant character operations including
//! character predicates, comparison, and conversion functions.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::utils::intern_symbol;
use std::sync::Arc;

/// Creates character operation bindings for the standard library.
pub fn create_character_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Character predicates
    bind_character_predicates(env);
    
    // Character comparison
    bind_character_comparison(env);
    
    // Character conversion
    bind_character_conversion(env);
    
    // Character case operations
    bind_character_case_operations(env);
}

/// Binds character predicates.
fn bind_character_predicates(env: &Arc<ThreadSafeEnvironment>) {
    // char?
    env.define("char?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_p),
        effects: vec![Effect::Pure],
    })));
    
    // char-alphabetic?
    env.define("char-alphabetic?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-alphabetic?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_alphabetic_p),
        effects: vec![Effect::Pure],
    })));
    
    // char-numeric?
    env.define("char-numeric?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-numeric?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_numeric_p),
        effects: vec![Effect::Pure],
    })));
    
    // char-whitespace?
    env.define("char-whitespace?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-whitespace?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_whitespace_p),
        effects: vec![Effect::Pure],
    })));
    
    // char-upper-case?
    env.define("char-upper-case?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-upper-case?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_upper_case_p),
        effects: vec![Effect::Pure],
    })));
    
    // char-lower-case?
    env.define("char-lower-case?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-lower-case?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_lower_case_p),
        effects: vec![Effect::Pure],
    })));
    
    // char-title-case?
    env.define("char-title-case?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-title-case?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_title_case_p),
        effects: vec![Effect::Pure],
    })));
    
    // char-general-category
    env.define("char-general-category".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-general-category".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_general_category),
        effects: vec![Effect::Pure],
    })));
}

/// Binds character comparison operations.
fn bind_character_comparison(env: &Arc<ThreadSafeEnvironment>) {
    // char=?
    env.define("char=?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char=?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_char_equal),
        effects: vec![Effect::Pure],
    })));
    
    // char<?
    env.define("char<?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char<?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_char_less),
        effects: vec![Effect::Pure],
    })));
    
    // char>?
    env.define("char>?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char>?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_char_greater),
        effects: vec![Effect::Pure],
    })));
    
    // char<=?
    env.define("char<=?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char<=?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_char_less_equal),
        effects: vec![Effect::Pure],
    })));
    
    // char>=?
    env.define("char>=?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char>=?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_char_greater_equal),
        effects: vec![Effect::Pure],
    })));
    
    // Case-insensitive versions
    // char-ci=?
    env.define("char-ci=?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-ci=?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_char_ci_equal),
        effects: vec![Effect::Pure],
    })));
    
    // char-ci<?
    env.define("char-ci<?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-ci<?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_char_ci_less),
        effects: vec![Effect::Pure],
    })));
    
    // char-ci>?
    env.define("char-ci>?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-ci>?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_char_ci_greater),
        effects: vec![Effect::Pure],
    })));
    
    // char-ci<=?
    env.define("char-ci<=?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-ci<=?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_char_ci_less_equal),
        effects: vec![Effect::Pure],
    })));
    
    // char-ci>=?
    env.define("char-ci>=?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-ci>=?".to_string(),
        arity_min: 2,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_char_ci_greater_equal),
        effects: vec![Effect::Pure],
    })));
}

/// Binds character conversion operations.
fn bind_character_conversion(env: &Arc<ThreadSafeEnvironment>) {
    // char->integer
    env.define("char->integer".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char->integer".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_to_integer),
        effects: vec![Effect::Pure],
    })));
    
    // integer->char
    env.define("integer->char".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "integer->char".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_integer_to_char),
        effects: vec![Effect::Pure],
    })));
}

/// Binds character case operations.
fn bind_character_case_operations(env: &Arc<ThreadSafeEnvironment>) {
    // char-upcase
    env.define("char-upcase".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-upcase".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_upcase),
        effects: vec![Effect::Pure],
    })));
    
    // char-downcase
    env.define("char-downcase".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-downcase".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_downcase),
        effects: vec![Effect::Pure],
    })));
    
    // char-foldcase
    env.define("char-foldcase".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-foldcase".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_foldcase),
        effects: vec![Effect::Pure],
    })));
    
    // char-titlecase
    env.define("char-titlecase".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "char-titlecase".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_char_titlecase),
        effects: vec![Effect::Pure],
    })));
}

// ============= IMPLEMENTATIONS =============

/// char? predicate
fn primitive_char_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let is_char = matches!(args[0], Value::Literal(crate::ast::Literal::Character(_)));
    Ok(Value::boolean(is_char))
}

/// char-alphabetic? predicate
fn primitive_char_alphabetic_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-alphabetic? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-alphabetic?")?;
    Ok(Value::boolean(ch.is_alphabetic()))
}

/// char-numeric? predicate
fn primitive_char_numeric_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-numeric? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-numeric?")?;
    Ok(Value::boolean(ch.is_numeric()))
}

/// char-whitespace? predicate
fn primitive_char_whitespace_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-whitespace? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-whitespace?")?;
    Ok(Value::boolean(ch.is_whitespace()))
}

/// char-upper-case? predicate
fn primitive_char_upper_case_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-upper-case? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-upper-case?")?;
    Ok(Value::boolean(ch.is_uppercase()))
}

/// char-lower-case? predicate
fn primitive_char_lower_case_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-lower-case? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-lower-case?")?;
    Ok(Value::boolean(ch.is_lowercase()))
}

/// char-title-case? predicate
fn primitive_char_title_case_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-title-case? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-title-case?")?;
    // Unicode titlecase is rare; for ASCII it's the same as uppercase
    // For proper Unicode support, we'd need more complex logic
    Ok(Value::boolean(ch.is_uppercase() && ch.is_alphabetic()))
}

/// char-general-category function
fn primitive_char_general_category(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-general-category expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-general-category")?;
    
    // Simplified Unicode general category classification
    // In a full implementation, this would use proper Unicode tables
    let category = if ch.is_alphabetic() {
        if ch.is_uppercase() {
            "Lu" // Letter, uppercase  
        } else if ch.is_lowercase() {
            "Ll" // Letter, lowercase
        } else {
            "Lo" // Letter, other
        }
    } else if ch.is_numeric() {
        "Nd" // Number, decimal digit
    } else if ch.is_whitespace() {
        "Zs" // Separator, space
    } else if ch.is_control() {
        "Cc" // Other, control
    } else if ch.is_ascii_punctuation() {
        "Po" // Punctuation, other
    } else {
        "Cn" // Other, not assigned (fallback)
    };
    
    Ok(Value::symbol(intern_symbol(category)))
}

/// Character comparison implementations
fn primitive_char_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "char=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    let first = extract_character(&args[0], "char=?")?;
    
    for arg in &args[1..] {
        let ch = extract_character(arg, "char=?")?;
        if first != ch {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

fn primitive_char_less(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "char<? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let c1 = extract_character(&window[0], "char<?")?;
        let c2 = extract_character(&window[1], "char<?")?;
        if c1 >= c2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

fn primitive_char_greater(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "char>? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let c1 = extract_character(&window[0], "char>?")?;
        let c2 = extract_character(&window[1], "char>?")?;
        if c1 <= c2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

fn primitive_char_less_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "char<=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let c1 = extract_character(&window[0], "char<=?")?;
        let c2 = extract_character(&window[1], "char<=?")?;
        if c1 > c2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

fn primitive_char_greater_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "char>=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let c1 = extract_character(&window[0], "char>=?")?;
        let c2 = extract_character(&window[1], "char>=?")?;
        if c1 < c2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// Case-insensitive character comparison implementations
fn primitive_char_ci_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "char-ci=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    let first = extract_character(&args[0], "char-ci=?")?.to_ascii_lowercase();
    
    for arg in &args[1..] {
        let ch = extract_character(arg, "char-ci=?")?.to_ascii_lowercase();
        if first != ch {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

fn primitive_char_ci_less(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "char-ci<? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let c1 = extract_character(&window[0], "char-ci<?")?.to_ascii_lowercase();
        let c2 = extract_character(&window[1], "char-ci<?")?.to_ascii_lowercase();
        if c1 >= c2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

fn primitive_char_ci_greater(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "char-ci>? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let c1 = extract_character(&window[0], "char-ci>?")?.to_ascii_lowercase();
        let c2 = extract_character(&window[1], "char-ci>?")?.to_ascii_lowercase();
        if c1 <= c2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

fn primitive_char_ci_less_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "char-ci<=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let c1 = extract_character(&window[0], "char-ci<=?")?.to_ascii_lowercase();
        let c2 = extract_character(&window[1], "char-ci<=?")?.to_ascii_lowercase();
        if c1 > c2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

fn primitive_char_ci_greater_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "char-ci>=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let c1 = extract_character(&window[0], "char-ci>=?")?.to_ascii_lowercase();
        let c2 = extract_character(&window[1], "char-ci>=?")?.to_ascii_lowercase();
        if c1 < c2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// Character conversion implementations
fn primitive_char_to_integer(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char->integer expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char->integer")?;
    Ok(Value::integer(ch as u32 as i64))
}

fn primitive_integer_to_char(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("integer->char expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let n = args[0].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "integer->char requires an integer argument".to_string(),
            None,
        )
    })?;
    
    if n < 0 || n > char::MAX as u32 as i64 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "integer->char argument out of character range".to_string(),
            None,
        )));
    }
    
    match char::from_u32(n as u32) {
        Some(ch) => Ok(Value::Literal(crate::ast::Literal::Character(ch))),
        None => Err(Box::new(DiagnosticError::runtime_error(
            "integer->char invalid character code".to_string(),
            None,
        ))),
    }
}

/// Character case operations
fn primitive_char_upcase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-upcase expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-upcase")?;
    Ok(Value::Literal(crate::ast::Literal::Character(ch.to_ascii_uppercase())))
}

fn primitive_char_downcase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-downcase expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-downcase")?;
    Ok(Value::Literal(crate::ast::Literal::Character(ch.to_ascii_lowercase())))
}

fn primitive_char_foldcase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-foldcase expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-foldcase")?;
    // Foldcase is like downcase for most purposes
    Ok(Value::Literal(crate::ast::Literal::Character(ch.to_ascii_lowercase())))
}

/// char-titlecase case conversion
fn primitive_char_titlecase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-titlecase expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-titlecase")?;
    // For most characters, titlecase is the same as uppercase
    // In a full Unicode implementation, this would handle special titlecase mappings
    Ok(Value::Literal(crate::ast::Literal::Character(ch.to_ascii_uppercase())))
}

// ============= HELPER FUNCTIONS =============

/// Extracts a character from a Value.
fn extract_character(value: &Value, operation: &str) -> Result<char> {
    match value {
        Value::Literal(crate::ast::Literal::Character(c)) => Ok(*c),
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires character arguments"),
            None,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_char_predicates() {
        let char_a = Value::Literal(crate::ast::Literal::Character('a'));
        let result = primitive_char_p(&[char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_alphabetic_p(&[char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let char_5 = Value::Literal(crate::ast::Literal::Character('5'));
        let result = primitive_char_numeric_p(&[char_5]).unwrap();
        assert_eq!(result, Value::boolean(true));
    }
    
    #[test]
    fn test_char_comparison() {
        let char_a = Value::Literal(crate::ast::Literal::Character('a'));
        let char_b = Value::Literal(crate::ast::Literal::Character('b'));
        
        let result = primitive_char_equal(&[char_a.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_less(&[char_a.clone(), char_b.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_greater(&[char_b, char_a]).unwrap();
        assert_eq!(result, Value::boolean(true));
    }
    
    #[test]
    fn test_char_conversion() {
        let char_a = Value::Literal(crate::ast::Literal::Character('a'));
        let result = primitive_char_to_integer(&[char_a]).unwrap();
        assert_eq!(result, Value::integer(97)); // ASCII value of 'a'
        
        let int_97 = Value::integer(97);
        let result = primitive_integer_to_char(&[int_97]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('a')));
    }
    
    #[test]
    fn test_char_case() {
        let char_a = Value::Literal(crate::ast::Literal::Character('a'));
        let result = primitive_char_upcase(&[char_a]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('A')));
        
        let char_a_upper = Value::Literal(crate::ast::Literal::Character('A'));
        let result = primitive_char_downcase(&[char_a_upper]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('a')));
    }
}
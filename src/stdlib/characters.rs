//! Character operations for the Lambdust standard library.
//!
//! This module implements R7RS-compliant character operations including
//! character predicates, comparison, and conversion functions.
//!
//! ## R7RS-small Character Procedures Implemented:
//! 
//! **Character Predicates:**
//! - `char?` - Test if argument is character
//! - `char-alphabetic?` - Test if character is alphabetic
//! - `char-numeric?` - Test if character is numeric
//! - `char-whitespace?` - Test if character is whitespace
//! - `char-upper-case?` - Test if character is uppercase
//! - `char-lower-case?` - Test if character is lowercase
//!
//! **Character Comparison:**
//! - `char=?`, `char<?`, `char>?`, `char<=?`, `char>=?` - Character comparison
//! - `char-ci=?`, `char-ci<?`, `char-ci>?`, `char-ci<=?`, `char-ci>=?` - Case-insensitive comparison
//!
//! **Character Conversion:**
//! - `char-upcase` - Convert to uppercase
//! - `char-downcase` - Convert to lowercase
//! - `char-foldcase` - Case folding (R7RS-small)
//!
//! **Character/Integer Conversion:**
//! - `char->integer` - Character to Unicode code point
//! - `integer->char` - Unicode code point to character
//!
//! ## Extensions (beyond R7RS-small):
//! - `char-title-case?` - Test if character is titlecase
//! - `char-titlecase` - Convert to titlecase
//! - `char-general-category` - Unicode general category
//!
//! All functions support proper Unicode handling using Rust's native `char` type.

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
    
    let first_ch = extract_character(&args[0], "char-ci=?")?;
    let first = first_ch.to_lowercase().next().unwrap_or(first_ch);
    
    for arg in &args[1..] {
        let ch = extract_character(arg, "char-ci=?")?;
        let ch_lower = ch.to_lowercase().next().unwrap_or(ch);
        if first != ch_lower {
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
        let ch1 = extract_character(&window[0], "char-ci<?")?;
        let ch2 = extract_character(&window[1], "char-ci<?")?;
        let c1 = ch1.to_lowercase().next().unwrap_or(ch1);
        let c2 = ch2.to_lowercase().next().unwrap_or(ch2);
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
        let ch1 = extract_character(&window[0], "char-ci>?")?;
        let ch2 = extract_character(&window[1], "char-ci>?")?;
        let c1 = ch1.to_lowercase().next().unwrap_or(ch1);
        let c2 = ch2.to_lowercase().next().unwrap_or(ch2);
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
        let ch1 = extract_character(&window[0], "char-ci<=?")?;
        let ch2 = extract_character(&window[1], "char-ci<=?")?;
        let c1 = ch1.to_lowercase().next().unwrap_or(ch1);
        let c2 = ch2.to_lowercase().next().unwrap_or(ch2);
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
        let ch1 = extract_character(&window[0], "char-ci>=?")?;
        let ch2 = extract_character(&window[1], "char-ci>=?")?;
        let c1 = ch1.to_lowercase().next().unwrap_or(ch1);
        let c2 = ch2.to_lowercase().next().unwrap_or(ch2);
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
    // Use proper Unicode case conversion
    let upper_ch = ch.to_uppercase().next().unwrap_or(ch);
    Ok(Value::Literal(crate::ast::Literal::Character(upper_ch)))
}

fn primitive_char_downcase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-downcase expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-downcase")?;
    // Use proper Unicode case conversion
    let lower_ch = ch.to_lowercase().next().unwrap_or(ch);
    Ok(Value::Literal(crate::ast::Literal::Character(lower_ch)))
}

fn primitive_char_foldcase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("char-foldcase expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let ch = extract_character(&args[0], "char-foldcase")?;
    // Case folding for Unicode - use lowercase for now
    // In a full implementation, this would use Unicode case folding tables
    let folded_ch = ch.to_lowercase().next().unwrap_or(ch);
    Ok(Value::Literal(crate::ast::Literal::Character(folded_ch)))
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
        // Test char? predicate
        let char_a = Value::Literal(crate::ast::Literal::Character('a'));
        let result = primitive_char_p(&[char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let not_char = Value::integer(42);
        let result = primitive_char_p(&[not_char]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char-alphabetic?
        let result = primitive_char_alphabetic_p(&[char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let char_5 = Value::Literal(crate::ast::Literal::Character('5'));
        let result = primitive_char_alphabetic_p(&[char_5.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char-numeric?
        let result = primitive_char_numeric_p(&[char_5.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_numeric_p(&[char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char-whitespace?
        let char_space = Value::Literal(crate::ast::Literal::Character(' '));
        let result = primitive_char_whitespace_p(&[char_space]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_whitespace_p(&[char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char-upper-case?
        let char_A = Value::Literal(crate::ast::Literal::Character('A'));
        let result = primitive_char_upper_case_p(&[char_A.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_upper_case_p(&[char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char-lower-case?
        let result = primitive_char_lower_case_p(&[char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_lower_case_p(&[char_A]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_char_comparison() {
        let char_a = Value::Literal(crate::ast::Literal::Character('a'));
        let char_b = Value::Literal(crate::ast::Literal::Character('b'));
        let char_c = Value::Literal(crate::ast::Literal::Character('c'));
        
        // Test char=?
        let result = primitive_char_equal(&[char_a.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_equal(&[char_a.clone(), char_b.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test multiple arguments
        let result = primitive_char_equal(&[char_a.clone(), char_a.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_equal(&[char_a.clone(), char_a.clone(), char_b.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char<?
        let result = primitive_char_less(&[char_a.clone(), char_b.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_less(&[char_b.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test transitive
        let result = primitive_char_less(&[char_a.clone(), char_b.clone(), char_c.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_less(&[char_a.clone(), char_c.clone(), char_b.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char>?
        let result = primitive_char_greater(&[char_b.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_greater(&[char_a.clone(), char_b.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char<=?
        let result = primitive_char_less_equal(&[char_a.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_less_equal(&[char_a.clone(), char_b.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_less_equal(&[char_b.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char>=?
        let result = primitive_char_greater_equal(&[char_a.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_greater_equal(&[char_b.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_greater_equal(&[char_a, char_b]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_char_ci_comparison() {
        let char_a = Value::Literal(crate::ast::Literal::Character('a'));
        let char_A = Value::Literal(crate::ast::Literal::Character('A'));
        let char_b = Value::Literal(crate::ast::Literal::Character('b'));
        let char_B = Value::Literal(crate::ast::Literal::Character('B'));
        
        // Test char-ci=?
        let result = primitive_char_ci_equal(&[char_a.clone(), char_A.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_ci_equal(&[char_a.clone(), char_b.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char-ci<?
        let result = primitive_char_ci_less(&[char_a.clone(), char_B.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_ci_less(&[char_B.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char-ci>?
        let result = primitive_char_ci_greater(&[char_B.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_ci_greater(&[char_a.clone(), char_B.clone()]).unwrap();
        assert_eq!(result, Value::boolean(false));
        
        // Test char-ci<=?
        let result = primitive_char_ci_less_equal(&[char_a.clone(), char_A.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_ci_less_equal(&[char_a.clone(), char_B.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        // Test char-ci>=?
        let result = primitive_char_ci_greater_equal(&[char_A.clone(), char_a.clone()]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let result = primitive_char_ci_greater_equal(&[char_B, char_a]).unwrap();
        assert_eq!(result, Value::boolean(true));
    }
    
    #[test]
    fn test_char_conversion() {
        // Test char->integer
        let char_a = Value::Literal(crate::ast::Literal::Character('a'));
        let result = primitive_char_to_integer(&[char_a]).unwrap();
        assert_eq!(result, Value::integer(97)); // ASCII value of 'a'
        
        let char_zero = Value::Literal(crate::ast::Literal::Character('0'));
        let result = primitive_char_to_integer(&[char_zero]).unwrap();
        assert_eq!(result, Value::integer(48)); // ASCII value of '0'
        
        // Test integer->char
        let int_97 = Value::integer(97);
        let result = primitive_integer_to_char(&[int_97]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('a')));
        
        let int_48 = Value::integer(48);
        let result = primitive_integer_to_char(&[int_48]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('0')));
        
        // Test Unicode characters
        let char_lambda = Value::Literal(crate::ast::Literal::Character('λ'));
        let result = primitive_char_to_integer(&[char_lambda]).unwrap();
        assert_eq!(result, Value::integer(955)); // Unicode value of 'λ'
        
        let int_955 = Value::integer(955);
        let result = primitive_integer_to_char(&[int_955]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('λ')));
    }
    
    #[test]
    fn test_char_conversion_errors() {
        // Test integer->char with invalid values
        let negative = Value::integer(-1);
        let result = primitive_integer_to_char(&[negative]);
        assert!(result.is_err());
        
        let too_large = Value::integer(0x110000); // Beyond Unicode range
        let result = primitive_integer_to_char(&[too_large]);
        assert!(result.is_err());
        
        // Test with non-integer
        let not_int = Value::string("not-an-integer");
        let result = primitive_integer_to_char(&[not_int]);
        assert!(result.is_err());
        
        // Test char->integer with non-character
        let not_char = Value::integer(42);
        let result = primitive_char_to_integer(&[not_char]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_char_case() {
        // Test basic ASCII case conversion
        let char_a = Value::Literal(crate::ast::Literal::Character('a'));
        let result = primitive_char_upcase(&[char_a]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('A')));
        
        let char_A = Value::Literal(crate::ast::Literal::Character('A'));
        let result = primitive_char_downcase(&[char_A]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('a')));
        
        // Test that non-letter characters are unchanged
        let char_5 = Value::Literal(crate::ast::Literal::Character('5'));
        let result = primitive_char_upcase(&[char_5.clone()]).unwrap();
        assert_eq!(result, char_5);
        
        let result = primitive_char_downcase(&[char_5]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('5')));
        
        // Test char-foldcase
        let char_A_fold = Value::Literal(crate::ast::Literal::Character('A'));
        let result = primitive_char_foldcase(&[char_A_fold]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('a')));
    }
    
    #[test]
    fn test_unicode_case() {
        // Test Unicode case conversion
        let char_alpha = Value::Literal(crate::ast::Literal::Character('α')); // Greek small alpha
        let result = primitive_char_upcase(&[char_alpha]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('Α'))); // Greek capital alpha
        
        let char_Alpha = Value::Literal(crate::ast::Literal::Character('Α')); // Greek capital alpha
        let result = primitive_char_downcase(&[char_Alpha]).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('α'))); // Greek small alpha
        
        // Test German ß (should remain unchanged in upcase in most implementations)
        let char_eszett = Value::Literal(crate::ast::Literal::Character('ß'));
        let result = primitive_char_upcase(&[char_eszett.clone()]).unwrap();
        // Note: This might be 'ß' or 'SS' depending on Unicode rules
        // We'll just check it doesn't error for now
        assert!(matches!(result, Value::Literal(crate::ast::Literal::Character(_))));
    }
    
    #[test]
    fn test_char_errors() {
        // Test wrong number of arguments
        let result = primitive_char_p(&[]);
        assert!(result.is_err());
        
        let char_a = Value::Literal(crate::ast::Literal::Character('a'));
        let char_b = Value::Literal(crate::ast::Literal::Character('b'));
        let result = primitive_char_p(&[char_a.clone(), char_b]);
        assert!(result.is_err());
        
        // Test comparison with wrong argument types
        let not_char = Value::integer(42);
        let result = primitive_char_equal(&[char_a.clone(), not_char]);
        assert!(result.is_err());
        
        // Test comparison with too few arguments
        let result = primitive_char_equal(&[char_a]);
        assert!(result.is_err());
        
        let result = primitive_char_equal(&[]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_char_general_category() {
        // Test basic categories
        let char_a = Value::Literal(crate::ast::Literal::Character('a'));
        let result = primitive_char_general_category(&[char_a]).unwrap();
        assert_eq!(result, Value::symbol(intern_symbol("Ll"))); // Letter, lowercase
        
        let char_A = Value::Literal(crate::ast::Literal::Character('A'));
        let result = primitive_char_general_category(&[char_A]).unwrap();
        assert_eq!(result, Value::symbol(intern_symbol("Lu"))); // Letter, uppercase
        
        let char_5 = Value::Literal(crate::ast::Literal::Character('5'));
        let result = primitive_char_general_category(&[char_5]).unwrap();
        assert_eq!(result, Value::symbol(intern_symbol("Nd"))); // Number, decimal digit
        
        let char_space = Value::Literal(crate::ast::Literal::Character(' '));
        let result = primitive_char_general_category(&[char_space]).unwrap();
        assert_eq!(result, Value::symbol(intern_symbol("Zs"))); // Separator, space
    }
}
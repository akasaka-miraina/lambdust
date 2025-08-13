//! String manipulation functions for the Lambdust standard library.
//!
//! This module implements R7RS-compliant string operations and SRFI-13
//! String Library including comprehensive string processing utilities.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use crate::ast::Literal;
use crate::utils::symbol_name;
use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Helper macro to bind a primitive procedure with both normal and builtin: names
macro_rules! bind_primitive {
    ($env:expr, $name:expr, $arity_min:expr, $arity_max:expr, $implementation:expr, $effects:expr) => {
        let proc = Arc::new(PrimitiveProcedure {
            name: $name.to_owned(),
            arity_min: $arity_min,
            arity_max: $arity_max,
            implementation: PrimitiveImpl::RustFn($implementation),
            effects: $effects,
        });
        let name_owned = $name.to_owned();
        $env.define(name_owned.clone(), Value::Primitive(proc.clone()));
        $env.define(format!("builtin:{}", name_owned), Value::Primitive(proc));
    };
}

// ============= CHARACTER SET SUPPORT =============

/// Represents a character set for SRFI-13 operations
#[derive(Clone, Debug)]
pub enum CharacterSet {
    /// A predicate function that tests characters
    Predicate(fn(char) -> bool),
    /// A character literal  
    Character(char),
    /// A string containing characters to match
    String(String),
    /// Default whitespace character set
    Whitespace,
}

impl CharacterSet {
    /// Test if a character is in this character set
    pub fn contains(&self, ch: char) -> bool {
        match self {
            CharacterSet::Predicate(f) => f(ch),
            CharacterSet::Character(c) => ch == *c,
            CharacterSet::String(s) => s.contains(ch),
            CharacterSet::Whitespace => ch.is_whitespace(),
        }
    }
    
    /// Create a character set from a Scheme value
    pub fn from_value(value: &Value) -> Result<CharacterSet> {
        match value {
            Value::Literal(Literal::Character(ch)) => Ok(CharacterSet::Character(*ch)),
            _ => {
                if let Some(s) = value.as_string() {
                    Ok(CharacterSet::String(s.to_string()))
                } else {
                    Err(Box::new(DiagnosticError::runtime_error(
                        "Character set must be a character, string, or predicate".to_string(),
                        None,
                    )))
                }
            }
        }
    }
}

/// Default character sets for common operations
impl Default for CharacterSet {
    fn default() -> Self {
        CharacterSet::Whitespace
    }
}

/// Creates string operation bindings for the standard library.
pub fn create_string_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // String creation and basic operations
    bind_string_creation_operations(env);
    
    // String predicates (R7RS + SRFI-13)
    bind_string_predicates(env);
    
    // String accessors and mutators
    bind_string_accessors(env);
    
    // String comparison (R7RS + SRFI-13)
    bind_string_comparison(env);
    
    // String manipulation (R7RS + SRFI-13)
    bind_string_manipulation(env);
    
    // String conversion (R7RS)
    bind_string_conversion(env);
    
    // String iteration (high-order functions)
    bind_string_iteration(env);
    
    // Additional utilities (existing)
    bind_string_utilities(env);
    
    // SRFI-13 String Library bindings
    bind_srfi13_constructors(env);
    bind_srfi13_selection(env);
    bind_srfi13_searching(env);
    bind_srfi13_modification(env);
}

/// Binds string creation and basic operations.
fn bind_string_creation_operations(env: &Arc<ThreadSafeEnvironment>) {
    bind_primitive!(env, "make-string", 1, Some(2), primitive_make_string, vec![Effect::Pure]);
    bind_primitive!(env, "string", 0, None, primitive_string, vec![Effect::Pure]);
    bind_primitive!(env, "string-length", 1, Some(1), primitive_string_length, vec![Effect::Pure]);
    bind_primitive!(env, "string-copy", 1, Some(3), primitive_string_copy, vec![Effect::Pure]);
}

/// Binds string predicates.
fn bind_string_predicates(env: &Arc<ThreadSafeEnvironment>) {
    // R7RS predicates
    bind_primitive!(env, "string?", 1, Some(1), primitive_string_p, vec![Effect::Pure]);
    bind_primitive!(env, "string-null?", 1, Some(1), primitive_string_null_p, vec![Effect::Pure]);
    
    // SRFI-13 predicates
    bind_primitive!(env, "string-every", 2, Some(4), primitive_string_every, vec![Effect::Pure]);
    bind_primitive!(env, "string-any", 2, Some(4), primitive_string_any, vec![Effect::Pure]);
}

/// Binds string accessors and mutators.
fn bind_string_accessors(env: &Arc<ThreadSafeEnvironment>) {
    bind_primitive!(env, "string-ref", 2, Some(2), primitive_string_ref, vec![Effect::Pure]);
    bind_primitive!(env, "string-set!", 3, Some(3), primitive_string_set, vec![Effect::State]);
}

/// Binds string comparison operations.
fn bind_string_comparison(env: &Arc<ThreadSafeEnvironment>) {
    // R7RS Case-sensitive comparisons
    bind_primitive!(env, "string=?", 2, None, primitive_string_equal, vec![Effect::Pure]);
    bind_primitive!(env, "string<?", 2, None, primitive_string_less, vec![Effect::Pure]);
    bind_primitive!(env, "string>?", 2, None, primitive_string_greater, vec![Effect::Pure]);
    bind_primitive!(env, "string<=?", 2, None, primitive_string_less_equal, vec![Effect::Pure]);
    bind_primitive!(env, "string>=?", 2, None, primitive_string_greater_equal, vec![Effect::Pure]);
    
    // R7RS Case-insensitive comparisons
    bind_primitive!(env, "string-ci=?", 2, None, primitive_string_ci_equal, vec![Effect::Pure]);
    bind_primitive!(env, "string-ci<?", 2, None, primitive_string_ci_less, vec![Effect::Pure]);
    bind_primitive!(env, "string-ci>?", 2, None, primitive_string_ci_greater, vec![Effect::Pure]);
    bind_primitive!(env, "string-ci<=?", 2, None, primitive_string_ci_less_equal, vec![Effect::Pure]);
    bind_primitive!(env, "string-ci>=?", 2, None, primitive_string_ci_greater_equal, vec![Effect::Pure]);
    
    // SRFI-13 Extended comparison and hashing
    // TODO: Implement these functions
    // bind_primitive!(env, "string-compare", 2, Some(6), primitive_string_compare, vec![Effect::Pure]);
    // bind_primitive!(env, "string-compare-ci", 2, Some(6), primitive_string_compare_ci, vec![Effect::Pure]);
    // bind_primitive!(env, "string-hash", 1, Some(3), primitive_string_hash, vec![Effect::Pure]);
    // bind_primitive!(env, "string-hash-ci", 1, Some(3), primitive_string_hash_ci, vec![Effect::Pure]);
}

/// Binds string manipulation operations.
fn bind_string_manipulation(env: &Arc<ThreadSafeEnvironment>) {
    // R7RS string manipulation
    bind_primitive!(env, "string-append", 0, None, primitive_string_append, vec![Effect::Pure]);
    bind_primitive!(env, "substring", 3, Some(3), primitive_substring, vec![Effect::Pure]);
    bind_primitive!(env, "string-fill!", 2, Some(4), primitive_string_fill, vec![Effect::State]);
    bind_primitive!(env, "string-copy!", 3, Some(5), primitive_string_copy_mut, vec![Effect::State]);
    bind_primitive!(env, "string-upcase", 1, Some(1), primitive_string_upcase, vec![Effect::Pure]);
    bind_primitive!(env, "string-downcase", 1, Some(1), primitive_string_downcase, vec![Effect::Pure]);
    bind_primitive!(env, "string-foldcase", 1, Some(1), primitive_string_foldcase, vec![Effect::Pure]);
    
    // SRFI-13 enhanced case operations
    bind_primitive!(env, "string-titlecase", 1, Some(3), primitive_string_titlecase, vec![Effect::Pure]);
    bind_primitive!(env, "string-reverse", 1, Some(3), primitive_string_reverse, vec![Effect::Pure]);
}

/// Binds string conversion operations.
fn bind_string_conversion(env: &Arc<ThreadSafeEnvironment>) {
    bind_primitive!(env, "string->list", 1, Some(3), primitive_string_to_list, vec![Effect::Pure]);
    bind_primitive!(env, "list->string", 1, Some(1), primitive_list_to_string, vec![Effect::Pure]);
    bind_primitive!(env, "string->vector", 1, Some(3), primitive_string_to_vector, vec![Effect::Pure]);
    bind_primitive!(env, "vector->string", 1, Some(3), primitive_vector_to_string, vec![Effect::Pure]);
    bind_primitive!(env, "string->utf8", 1, Some(3), primitive_string_to_utf8, vec![Effect::Pure]);
    bind_primitive!(env, "utf8->string", 1, Some(3), primitive_utf8_to_string, vec![Effect::Pure]);
}

/// Binds string iteration (higher-order) operations.
fn bind_string_iteration(env: &Arc<ThreadSafeEnvironment>) {
    bind_primitive!(env, "string-for-each", 2, None, primitive_string_for_each, vec![Effect::Pure]);
    bind_primitive!(env, "string-map", 2, None, primitive_string_map, vec![Effect::Pure]);
}

/// Binds additional string utility operations.
fn bind_string_utilities(env: &Arc<ThreadSafeEnvironment>) {
    // Additional utilities beyond R7RS but useful
    bind_primitive!(env, "string-append-list", 1, Some(1), primitive_string_append_list, vec![Effect::Pure]);
    bind_primitive!(env, "string-trim", 1, Some(2), primitive_string_trim, vec![Effect::Pure]);
    bind_primitive!(env, "string-trim-left", 1, Some(2), primitive_string_trim_left, vec![Effect::Pure]);
    bind_primitive!(env, "string-trim-right", 1, Some(2), primitive_string_trim_right, vec![Effect::Pure]);
    bind_primitive!(env, "string-split", 2, Some(2), primitive_string_split, vec![Effect::Pure]);
    bind_primitive!(env, "string-join", 1, Some(2), primitive_string_join, vec![Effect::Pure]);
    bind_primitive!(env, "string-contains?", 2, Some(2), primitive_string_contains_p, vec![Effect::Pure]);
    bind_primitive!(env, "string-replace", 3, Some(3), primitive_string_replace, vec![Effect::Pure]);
}

// ============= STRING CREATION IMPLEMENTATIONS =============

/// make-string procedure
pub fn primitive_make_string(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("make-string expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let length = args[0].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "make-string first argument must be a non-negative integer".to_string(),
            None,
        )
    })?;
    
    if length < 0 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "make-string length must be non-negative".to_string(),
            None,
        )));
    }
    
    let fill_char = if args.len() == 2 {
        extract_character(&args[1], "make-string")?
    } else {
        ' ' // Default fill character (space)
    };
    
    // R7RS-small specifies that make-string creates mutable strings
    Ok(Value::mutable_string_filled(length as usize, fill_char))
}

/// string constructor from characters
pub fn primitive_string(args: &[Value]) -> Result<Value> {
    let mut result = String::new();
    
    for arg in args {
        let ch = extract_character(arg, "string")?;
        result.push(ch);
    }
    
    Ok(Value::string(result))
}

/// string-length procedure
pub fn primitive_string_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-length expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let length = args[0].string_length().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-length requires a string argument".to_string(),
            None,
        )
    })?;
    
    Ok(Value::integer(length as i64))
}

/// string-copy procedure
pub fn primitive_string_copy(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-copy expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-copy")?;
    let chars: Vec<char> = s.chars().collect();
    let length = chars.len();
    
    let start = if args.len() > 1 {
        let start_idx = args[1].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "string-copy start index must be an integer".to_string(),
                None,
            )
        })? as usize;
        
        if start_idx > length {
            return Err(Box::new(DiagnosticError::runtime_error(
                "string-copy start index out of bounds".to_string(),
                None,
            )));
        }
        start_idx
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        let end_idx = args[2].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "string-copy end index must be an integer".to_string(),
                None,
            )
        })? as usize;
        
        if end_idx > length || end_idx < start {
            return Err(Box::new(DiagnosticError::runtime_error(
                "string-copy end index out of bounds".to_string(),
                None,
            )));
        }
        end_idx
    } else {
        length
    };
    
    let result: String = chars[start..end].iter().collect();
    Ok(Value::string(result))
}

// ============= STRING PREDICATE IMPLEMENTATIONS =============

/// string? predicate
fn primitive_string_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::boolean(args[0].is_string()))
}

/// string-null? predicate
pub fn primitive_string_null_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-null? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    if let Some(s) = args[0].as_string() {
        Ok(Value::boolean(s.is_empty()))
    } else {
        Err(Box::new(DiagnosticError::runtime_error(
            "string-null? requires string argument".to_string(),
            None,
        )))
    }
}

// ============= STRING ACCESSOR IMPLEMENTATIONS =============

/// string-ref procedure
pub fn primitive_string_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-ref expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let index = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-ref index must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    match &args[0] {
        Value::Literal(crate::ast::Literal::String(s)) => {
            let chars: Vec<char> = s.chars().collect();
            
            if index >= chars.len() {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "string-ref index out of bounds".to_string(),
                    None,
                )));
            }
            
            Ok(Value::Literal(crate::ast::Literal::Character(chars[index])))
        }
        Value::MutableString(chars_arc) => {
            let chars = chars_arc.read().map_err(|_| {
                DiagnosticError::runtime_error(
                    "string-ref failed to acquire read lock on string".to_string(),
                    None,
                )
            })?;
            
            if index >= chars.len() {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "string-ref index out of bounds".to_string(),
                    None,
                )));
            }
            
            Ok(Value::Literal(crate::ast::Literal::Character(chars[index])))
        }
        _ => {
            Err(Box::new(DiagnosticError::runtime_error(
                "string-ref first argument must be a string".to_string(),
                None,
            )))
        }
    }
}

/// string-set! procedure (mutation)
fn primitive_string_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-set! expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let string_val = &args[0];
    let index = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-set! index must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    let new_char = match &args[2] {
        Value::Literal(crate::ast::Literal::Character(ch)) => *ch,
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                "string-set! third argument must be a character".to_string(),
                None,
            )))
        }
    };
    
    match string_val {
        Value::MutableString(chars_arc) => {
            let mut chars = chars_arc.write().map_err(|_| {
                DiagnosticError::runtime_error(
                    "string-set! failed to acquire write lock on string".to_string(),
                    None,
                )
            })?;
            
            if index >= chars.len() {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("string-set! index {} out of bounds for string of length {}", 
                            index, chars.len()),
                    None,
                )));
            }
            
            chars[index] = new_char;
            Ok(Value::Unspecified)
        }
        Value::Literal(crate::ast::Literal::String(_)) => {
            Err(Box::new(DiagnosticError::runtime_error(
                "string-set! can only be used with mutable strings".to_string(),
                None,
            )))
        }
        _ => {
            Err(Box::new(DiagnosticError::runtime_error(
                "string-set! first argument must be a string".to_string(),
                None,
            )))
        }
    }
}

// ============= STRING COMPARISON IMPLEMENTATIONS =============

/// string=? procedure
pub fn primitive_string_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    let first = extract_string_owned(&args[0], "string=?")?;
    
    for arg in &args[1..] {
        let s = extract_string_owned(arg, "string=?")?;
        if first != s {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// string<? procedure
pub fn primitive_string_less(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string<? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let s1 = extract_string_owned(&window[0], "string<?")?;
        let s2 = extract_string_owned(&window[1], "string<?")?;
        if s1 >= s2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// string>? procedure
pub fn primitive_string_greater(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string>? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let s1 = extract_string_owned(&window[0], "string>?")?;
        let s2 = extract_string_owned(&window[1], "string>?")?;
        if s1 <= s2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// string<=? procedure
pub fn primitive_string_less_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string<=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let s1 = extract_string_owned(&window[0], "string<=?")?;
        let s2 = extract_string_owned(&window[1], "string<=?")?;
        if s1 > s2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// string>=? procedure
pub fn primitive_string_greater_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string>=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let s1 = extract_string_owned(&window[0], "string>=?")?;
        let s2 = extract_string_owned(&window[1], "string>=?")?;
        if s1 < s2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// Case-insensitive string comparison implementations
pub fn primitive_string_ci_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-ci=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    let first = extract_string_owned(&args[0], "string-ci=?")?.to_lowercase();
    
    for arg in &args[1..] {
        let s = extract_string_owned(arg, "string-ci=?")?.to_lowercase();
        if first != s {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

pub fn primitive_string_ci_less(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-ci<? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let s1 = extract_string_owned(&window[0], "string-ci<?")?.to_lowercase();
        let s2 = extract_string_owned(&window[1], "string-ci<?")?.to_lowercase();
        if s1 >= s2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

pub fn primitive_string_ci_greater(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-ci>? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let s1 = extract_string_owned(&window[0], "string-ci>?")?.to_lowercase();
        let s2 = extract_string_owned(&window[1], "string-ci>?")?.to_lowercase();
        if s1 <= s2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

pub fn primitive_string_ci_less_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-ci<=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let s1 = extract_string_owned(&window[0], "string-ci<=?")?.to_lowercase();
        let s2 = extract_string_owned(&window[1], "string-ci<=?")?.to_lowercase();
        if s1 > s2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

pub fn primitive_string_ci_greater_equal(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-ci>=? requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    for window in args.windows(2) {
        let s1 = extract_string_owned(&window[0], "string-ci>=?")?.to_lowercase();
        let s2 = extract_string_owned(&window[1], "string-ci>=?")?.to_lowercase();
        if s1 < s2 {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

// ============= STRING MANIPULATION IMPLEMENTATIONS =============

/// string-append procedure
pub fn primitive_string_append(args: &[Value]) -> Result<Value> {
    let mut result = String::new();
    
    for arg in args {
        let s = extract_string_owned(arg, "string-append")?;
        result.push_str(&s);
    }
    
    Ok(Value::string(result))
}

/// substring procedure
pub fn primitive_substring(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("substring expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string_owned(&args[0], "substring")?;
    let start = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "substring start index must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    let end = args[2].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "substring end index must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    let chars: Vec<char> = s.chars().collect();
    let length = chars.len();
    
    if start > length || end > length || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "substring indices out of bounds".to_string(),
            None,
        )));
    }
    
    let result: String = chars[start..end].iter().collect();
    Ok(Value::string(result))
}

/// string-fill! procedure (mutation)
fn primitive_string_fill(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-fill! expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let string_val = &args[0];
    let fill_char = match &args[1] {
        Value::Literal(crate::ast::Literal::Character(ch)) => *ch,
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                "string-fill! second argument must be a character".to_string(),
                None,
            )))
        }
    };
    
    // Optional start index (defaults to 0)
    let start_idx = if args.len() > 2 {
        args[2].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "string-fill! start index must be an integer".to_string(),
                None,
            )
        })? as usize
    } else {
        0
    };
    
    match string_val {
        Value::MutableString(chars_arc) => {
            let mut chars = chars_arc.write().map_err(|_| {
                DiagnosticError::runtime_error(
                    "string-fill! failed to acquire write lock on string".to_string(),
                    None,
                )
            })?;
            
            let string_len = chars.len();
            
            // Optional end index (defaults to string length)
            let end_idx = if args.len() > 3 {
                let end = args[3].as_integer().ok_or_else(|| {
                    DiagnosticError::runtime_error(
                        "string-fill! end index must be an integer".to_string(),
                        None,
                    )
                })? as usize;
                
                if end > string_len {
                    return Err(Box::new(DiagnosticError::runtime_error(
                        format!("string-fill! end index {end} out of bounds for string of length {string_len}"),
                        None,
                    )));
                }
                end
            } else {
                string_len
            };
            
            // Validate indices
            if start_idx > string_len {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!("string-fill! start index {start_idx} out of bounds for string of length {string_len}"),
                    None,
                )));
            }
            
            if start_idx > end_idx {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "string-fill! start index must not be greater than end index".to_string(),
                    None,
                )));
            }
            
            // Fill the range with the character
            for i in start_idx..end_idx {
                chars[i] = fill_char;
            }
            
            Ok(Value::Unspecified)
        }
        Value::Literal(crate::ast::Literal::String(_)) => {
            Err(Box::new(DiagnosticError::runtime_error(
                "string-fill! can only be used with mutable strings".to_string(),
                None,
            )))
        }
        _ => {
            Err(Box::new(DiagnosticError::runtime_error(
                "string-fill! first argument must be a string".to_string(),
                None,
            )))
        }
    }
}

/// string-copy! procedure (mutation)
fn primitive_string_copy_mut(args: &[Value]) -> Result<Value> {
    if args.len() < 3 || args.len() > 5 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-copy! expects 3 to 5 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // Note: In a full implementation, this would require mutable string support
    Err(Box::new(DiagnosticError::runtime_error(
        "string-copy! requires mutable string support (not yet implemented)".to_string(),
        None,
    )))
}

/// string-upcase procedure
pub fn primitive_string_upcase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-upcase expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-upcase")?;
    Ok(Value::string(s.to_uppercase()))
}

/// string-downcase procedure
pub fn primitive_string_downcase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-downcase expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-downcase")?;
    Ok(Value::string(s.to_lowercase()))
}

/// string-foldcase procedure
pub fn primitive_string_foldcase(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-foldcase expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-foldcase")?;
    // Foldcase is like lowercase but more aggressive for case-insensitive comparison
    Ok(Value::string(s.to_lowercase()))
}

// ============= STRING CONVERSION IMPLEMENTATIONS =============

/// string->list procedure
pub fn primitive_string_to_list(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string->list expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string->list")?;
    let chars: Vec<char> = s.chars().collect();
    let length = chars.len();
    
    let start = if args.len() > 1 {
        args[1].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "string->list start index must be an integer".to_string(),
                None,
            )
        })? as usize
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        args[2].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "string->list end index must be an integer".to_string(),
                None,
            )
        })? as usize
    } else {
        length
    };
    
    if start > length || end > length || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string->list indices out of bounds".to_string(),
            None,
        )));
    }
    
    let char_values: Vec<Value> = chars[start..end]
        .iter()
        .map(|&c| Value::Literal(crate::ast::Literal::Character(c)))
        .collect();
    
    Ok(Value::list(char_values))
}

/// list->string procedure
pub fn primitive_list_to_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("list->string expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let list = args[0].as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "list->string requires a list argument".to_string(),
            None,
        )
    })?;
    
    let mut result = String::new();
    
    for value in list {
        let ch = extract_character(&value, "list->string")?;
        result.push(ch);
    }
    
    Ok(Value::string(result))
}

/// string->vector procedure
fn primitive_string_to_vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string->vector expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string->vector")?;
    let chars: Vec<char> = s.chars().collect();
    let length = chars.len();
    
    let start = if args.len() > 1 {
        args[1].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "string->vector start index must be an integer".to_string(),
                None,
            )
        })? as usize
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        args[2].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "string->vector end index must be an integer".to_string(),
                None,
            )
        })? as usize
    } else {
        length
    };
    
    if start > length || end > length || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string->vector indices out of bounds".to_string(),
            None,
        )));
    }
    
    let char_values: Vec<Value> = chars[start..end]
        .iter()
        .map(|&c| Value::Literal(crate::ast::Literal::Character(c)))
        .collect();
    
    Ok(Value::vector(char_values))
}

/// vector->string procedure
fn primitive_vector_to_string(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("vector->string expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let vector = match &args[0] {
        Value::Vector(v) => v.read().unwrap().clone(),
        _ => return Err(Box::new(DiagnosticError::runtime_error(
            "vector->string requires a vector argument".to_string(),
            None,
        ))),
    };
    
    let length = vector.len();
    
    let start = if args.len() > 1 {
        args[1].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "vector->string start index must be an integer".to_string(),
                None,
            )
        })? as usize
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        args[2].as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "vector->string end index must be an integer".to_string(),
                None,
            )
        })? as usize
    } else {
        length
    };
    
    if start > length || end > length || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "vector->string indices out of bounds".to_string(),
            None,
        )));
    }
    
    let mut result = String::new();
    
    for value in &vector[start..end] {
        let ch = extract_character(value, "vector->string")?;
        result.push(ch);
    }
    
    Ok(Value::string(result))
}

/// string->utf8 procedure
fn primitive_string_to_utf8(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string->utf8 expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string->utf8")?;
    let bytes: Vec<u8> = s.as_bytes().to_vec();
    
    // Convert bytes to a vector of integers
    let byte_values: Vec<Value> = bytes
        .iter()
        .map(|&b| Value::integer(b as i64))
        .collect();
    
    Ok(Value::vector(byte_values))
}

/// utf8->string procedure
fn primitive_utf8_to_string(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("utf8->string expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let vector = match &args[0] {
        Value::Vector(v) => v.read().unwrap().clone(),
        _ => return Err(Box::new(DiagnosticError::runtime_error(
            "utf8->string requires a vector argument".to_string(),
            None,
        ))),
    };
    
    let mut bytes = Vec::new();
    
    for value in vector {
        let byte = value.as_integer().ok_or_else(|| {
            DiagnosticError::runtime_error(
                "utf8->string vector must contain integers".to_string(),
                None,
            )
        })?;
        
        if !(0..=255).contains(&byte) {
            return Err(Box::new(DiagnosticError::runtime_error(
                "utf8->string byte values must be between 0 and 255".to_string(),
                None,
            )));
        }
        
        bytes.push(byte as u8);
    }
    
    match String::from_utf8(bytes) {
        Ok(s) => Ok(Value::string(s)),
        Err(_) => Err(Box::new(DiagnosticError::runtime_error(
            "utf8->string invalid UTF-8 sequence".to_string(),
            None,
        ))),
    }
}

// ============= HELPER FUNCTIONS =============

/// Extracts a string from a Value (returns a reference to immutable strings only).
fn extract_string<'a>(value: &'a Value, operation: &str) -> Result<&'a str> {
    value.as_string().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires string arguments"),
            None,
        ))
    })
}

/// Extracts a string from a Value as an owned String (works with both mutable and immutable strings).
fn extract_string_owned(value: &Value, operation: &str) -> Result<String> {
    value.as_string_owned().ok_or_else(|| {
        Box::new(DiagnosticError::runtime_error(
            format!("{operation} requires string arguments"),
            None,
        ))
    })
}

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

// ============= STRING ITERATION IMPLEMENTATIONS =============

/// string-for-each procedure - R7RS required
fn primitive_string_for_each(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-for-each requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    let procedure = &args[0];
    
    // Verify procedure is callable
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-for-each first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    let strings: Result<Vec<&str>> = args[1..].iter()
        .map(|v| extract_string(v, "string-for-each"))
        .collect();
    let strings = strings?;
    
    if strings.is_empty() {
        return Ok(Value::Unspecified);
    }
    
    // Find minimum length among all strings
    let min_length = strings.iter()
        .map(|s| s.chars().count())
        .min()
        .unwrap_or(0);
    
    // If any string is empty, return unspecified immediately
    if min_length == 0 {
        return Ok(Value::Unspecified);
    }
    
    // Convert strings to character vectors for efficient indexing
    let char_vectors: Vec<Vec<char>> = strings.iter()
        .map(|s| s.chars().collect())
        .collect();
    
    // Apply procedure to each character position for side effects
    for i in 0..min_length {
        let char_args: Vec<Value> = char_vectors.iter()
            .map(|chars| Value::Literal(crate::ast::Literal::Character(chars[i])))
            .collect();
        
        // Apply the procedure - for now we can only handle primitive procedures
        match procedure {
            Value::Primitive(prim) => {
                match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => {
                        // Call the function but ignore the result (for-each is for side effects)
                        func(&char_args)?;
                    },
                    PrimitiveImpl::Native(func) => {
                        // Call the function but ignore the result (for-each is for side effects)
                        func(&char_args)?;
                    },
                    PrimitiveImpl::EvaluatorIntegrated(_) => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "string-for-each with evaluator-integrated functions requires evaluator access".to_string(),
                            None,
                        )));
                    },
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "string-for-each with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                }
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "string-for-each with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(Value::Unspecified)
}

/// string-map procedure - R7RS required
fn primitive_string_map(args: &[Value]) -> Result<Value> {
    if args.len() < 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-map requires at least 2 arguments".to_string(),
            None,
        )));
    }
    
    let procedure = &args[0];
    
    // Verify procedure is callable
    if !procedure.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-map first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    let strings: Result<Vec<&str>> = args[1..].iter()
        .map(|v| extract_string(v, "string-map"))
        .collect();
    let strings = strings?;
    
    if strings.is_empty() {
        return Ok(Value::string(""));
    }
    
    // Find minimum length among all strings
    let min_length = strings.iter()
        .map(|s| s.chars().count())
        .min()
        .unwrap_or(0);
    
    // If any string is empty, return empty string
    if min_length == 0 {
        return Ok(Value::string(""));
    }
    
    // Convert strings to character vectors for efficient indexing
    let char_vectors: Vec<Vec<char>> = strings.iter()
        .map(|s| s.chars().collect())
        .collect();
    
    let mut result_chars = Vec::new();
    
    // Apply procedure to each character position
    for i in 0..min_length {
        let char_args: Vec<Value> = char_vectors.iter()
            .map(|chars| Value::Literal(crate::ast::Literal::Character(chars[i])))
            .collect();
        
        // Apply the procedure - for now we can only handle primitive procedures
        match procedure {
            Value::Primitive(prim) => {
                let result = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&char_args)?,
                    PrimitiveImpl::Native(func) => func(&char_args)?,
                    PrimitiveImpl::EvaluatorIntegrated(_) => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "string-map with evaluator-integrated functions requires evaluator access".to_string(),
                            None,
                        )));
                    }
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "string-map with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
                
                // The result should be a character
                match result {
                    Value::Literal(crate::ast::Literal::Character(ch)) => {
                        result_chars.push(ch);
                    },
                    _ => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "string-map procedure must return a character".to_string(),
                            None,
                        )));
                    }
                }
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "string-map with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(Value::string(result_chars.iter().collect::<String>()))
}

// ============= ADDITIONAL UTILITY IMPLEMENTATIONS =============

/// string-append-list procedure (used by Scheme module)
fn primitive_string_append_list(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-append-list expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let list = args[0].as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-append-list requires a list argument".to_string(),
            None,
        )
    })?;
    
    let mut result = String::new();
    
    for value in list {
        let s = extract_string(&value, "string-append-list")?;
        result.push_str(s);
    }
    
    Ok(Value::string(result))
}

/// string-trim procedure
fn primitive_string_trim(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-trim expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-trim")?;
    
    // Default whitespace characters
    let default_chars = [' ', '\t', '\n', '\r'];
    let trim_chars = if args.len() > 1 {
        // Custom character set (simplified - would need proper implementation)
        &default_chars[..]
    } else {
        &default_chars[..]
    };
    
    let result = s.trim_matches(|c| trim_chars.contains(&c));
    Ok(Value::string(result.to_string()))
}

/// string-trim-left procedure
fn primitive_string_trim_left(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-trim-left expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-trim-left")?;
    
    let default_chars = [' ', '\t', '\n', '\r'];
    let trim_chars = &default_chars[..];
    
    let result = s.trim_start_matches(|c| trim_chars.contains(&c));
    Ok(Value::string(result.to_string()))
}

/// string-trim-right procedure
fn primitive_string_trim_right(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-trim-right expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-trim-right")?;
    
    let default_chars = [' ', '\t', '\n', '\r'];
    let trim_chars = &default_chars[..];
    
    let result = s.trim_end_matches(|c| trim_chars.contains(&c));
    Ok(Value::string(result.to_string()))
}

/// string-split procedure
fn primitive_string_split(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-split expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-split")?;
    let delimiter = extract_string(&args[1], "string-split")?;
    
    let parts: Vec<Value> = s.split(delimiter)
        .map(|part| Value::string(part.to_string()))
        .collect();
    
    Ok(Value::list(parts))
}

/// string-join procedure
fn primitive_string_join(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-join expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let list = args[0].as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-join requires a list argument".to_string(),
            None,
        )
    })?;
    
    let delimiter = if args.len() > 1 {
        extract_string(&args[1], "string-join")?
    } else {
        ""
    };
    
    let strings: Result<Vec<&str>> = list.iter()
        .map(|v| extract_string(v, "string-join"))
        .collect();
    let strings = strings?;
    
    Ok(Value::string(strings.join(delimiter)))
}

/// string-contains? procedure
fn primitive_string_contains_p(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-contains? expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let haystack = extract_string(&args[0], "string-contains?")?;
    let needle = extract_string(&args[1], "string-contains?")?;
    
    Ok(Value::boolean(haystack.contains(needle)))
}

/// string-replace procedure
fn primitive_string_replace(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-replace expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-replace")?;
    let old = extract_string(&args[1], "string-replace")?;
    let new = extract_string(&args[2], "string-replace")?;
    
    Ok(Value::string(s.replace(old, new)))
}

// ============= SRFI-13 BINDING FUNCTIONS =============

/// Binds SRFI-13 string constructors.
fn bind_srfi13_constructors(env: &Arc<ThreadSafeEnvironment>) {
    bind_primitive!(env, "string-tabulate", 2, Some(2), primitive_string_tabulate, vec![Effect::Pure]);
    bind_primitive!(env, "reverse-list->string", 1, Some(1), primitive_reverse_list_to_string, vec![Effect::Pure]);
    bind_primitive!(env, "string-join", 1, Some(3), primitive_string_join_srfi13, vec![Effect::Pure]);
    bind_primitive!(env, "string-concatenate", 1, Some(1), primitive_string_concatenate, vec![Effect::Pure]);
    bind_primitive!(env, "string-concatenate-reverse", 1, Some(2), primitive_string_concatenate_reverse, vec![Effect::Pure]);
}

/// Binds SRFI-13 string selection operations.
fn bind_srfi13_selection(env: &Arc<ThreadSafeEnvironment>) {
    bind_primitive!(env, "string-take", 2, Some(2), primitive_string_take, vec![Effect::Pure]);
    bind_primitive!(env, "string-drop", 2, Some(2), primitive_string_drop, vec![Effect::Pure]);
    bind_primitive!(env, "string-take-right", 2, Some(2), primitive_string_take_right, vec![Effect::Pure]);
    bind_primitive!(env, "string-drop-right", 2, Some(2), primitive_string_drop_right, vec![Effect::Pure]);
    bind_primitive!(env, "string-pad", 2, Some(4), primitive_string_pad, vec![Effect::Pure]);
    bind_primitive!(env, "string-pad-right", 2, Some(4), primitive_string_pad_right, vec![Effect::Pure]);
    bind_primitive!(env, "string-trim", 1, Some(4), primitive_string_trim_srfi13, vec![Effect::Pure]);
    bind_primitive!(env, "string-trim-right", 1, Some(4), primitive_string_trim_right_srfi13, vec![Effect::Pure]);
    bind_primitive!(env, "string-trim-both", 1, Some(4), primitive_string_trim_both, vec![Effect::Pure]);
}

/// Binds SRFI-13 string searching operations.
fn bind_srfi13_searching(env: &Arc<ThreadSafeEnvironment>) {
    bind_primitive!(env, "string-index", 2, Some(4), primitive_string_index, vec![Effect::Pure]);
    bind_primitive!(env, "string-rindex", 2, Some(4), primitive_string_rindex, vec![Effect::Pure]);
    bind_primitive!(env, "string-index-right", 2, Some(4), primitive_string_index_right, vec![Effect::Pure]);
    bind_primitive!(env, "string-skip", 2, Some(4), primitive_string_skip, vec![Effect::Pure]);
    bind_primitive!(env, "string-skip-right", 2, Some(4), primitive_string_skip_right, vec![Effect::Pure]);
    bind_primitive!(env, "string-contains", 2, Some(4), primitive_string_contains, vec![Effect::Pure]);
    bind_primitive!(env, "string-contains-ci", 2, Some(4), primitive_string_contains_ci, vec![Effect::Pure]);
    bind_primitive!(env, "string-count", 2, Some(4), primitive_string_count, vec![Effect::Pure]);
}

/// Binds SRFI-13 string modification operations.
fn bind_srfi13_modification(env: &Arc<ThreadSafeEnvironment>) {
    bind_primitive!(env, "string-replace", 4, Some(6), primitive_string_replace_srfi13, vec![Effect::Pure]);
}

// ============= SRFI-13 IMPLEMENTATION =============

// SRFI-13 String Predicates

/// string-every procedure - test if every character satisfies predicate
pub fn primitive_string_every(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-every expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let charset = if let Ok(ch) = extract_character(&args[0], "string-every") {
        CharacterSet::Character(ch)
    } else if let Some(s) = args[0].as_string() {
        CharacterSet::String(s.to_string())
    } else {
        // For now, we don't support procedure predicates
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-every predicate procedures not yet supported".to_string(),
            None,
        )));
    };
    
    let s = extract_string(&args[1], "string-every")?;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-every indices out of bounds".to_string(),
            None,
        )));
    }
    
    for ch in &chars[start..end] {
        if !charset.contains(*ch) {
            return Ok(Value::boolean(false));
        }
    }
    
    Ok(Value::boolean(true))
}

/// string-any procedure - test if any character satisfies predicate
pub fn primitive_string_any(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-any expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let charset = if let Ok(ch) = extract_character(&args[0], "string-any") {
        CharacterSet::Character(ch)
    } else if let Some(s) = args[0].as_string() {
        CharacterSet::String(s.to_string())
    } else {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-any predicate procedures not yet supported".to_string(),
            None,
        )));
    };
    
    let s = extract_string(&args[1], "string-any")?;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-any indices out of bounds".to_string(),
            None,
        )));
    }
    
    for ch in &chars[start..end] {
        if charset.contains(*ch) {
            return Ok(Value::boolean(true));
        }
    }
    
    Ok(Value::boolean(false))
}

// SRFI-13 String Constructors

/// string-tabulate procedure - create string by applying procedure to indices
pub fn primitive_string_tabulate(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-tabulate expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let proc = &args[0];
    let len = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-tabulate length must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    if !proc.is_procedure() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-tabulate first argument must be a procedure".to_string(),
            None,
        )));
    }
    
    let mut result = String::new();
    
    for i in 0..len {
        let index_arg = vec![Value::integer(i as i64)];
        
        match proc {
            Value::Primitive(prim) => {
                let ch_val = match &prim.implementation {
                    PrimitiveImpl::RustFn(func) => func(&index_arg)?,
                    PrimitiveImpl::Native(func) => func(&index_arg)?,
                    PrimitiveImpl::EvaluatorIntegrated(_) => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "string-tabulate with evaluator-integrated functions requires evaluator access".to_string(),
                            None,
                        )));
                    }
                    PrimitiveImpl::ForeignFn { .. } => {
                        return Err(Box::new(DiagnosticError::runtime_error(
                            "string-tabulate with foreign functions not yet implemented".to_string(),
                            None,
                        )));
                    }
                };
                
                let ch = extract_character(&ch_val, "string-tabulate")?;
                result.push(ch);
            },
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    "string-tabulate with user-defined procedures requires evaluator integration (not yet implemented)".to_string(),
                    None,
                )));
            }
        }
    }
    
    Ok(Value::string(result))
}

/// reverse-list->string procedure
pub fn primitive_reverse_list_to_string(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("reverse-list->string expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let list = args[0].as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "reverse-list->string requires a list argument".to_string(),
            None,
        )
    })?;
    
    let mut result = String::new();
    
    // Process list in reverse order
    for value in list.iter().rev() {
        let ch = extract_character(value, "reverse-list->string")?;
        result.push(ch);
    }
    
    Ok(Value::string(result))
}

/// string-join procedure (SRFI-13 enhanced version)
pub fn primitive_string_join_srfi13(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-join expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let strings = args[0].as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-join requires a list argument".to_string(),
            None,
        )
    })?;
    
    let delimiter = if args.len() > 1 {
        extract_string(&args[1], "string-join")?
    } else {
        " "
    };
    
    let grammar = if args.len() > 2 {
        match args[2].as_symbol() {
            Some(sym) => {
                match symbol_name(sym) {
                    Some(name) => name.to_string(),
                    None => return Err(Box::new(DiagnosticError::runtime_error(
                        "string-join grammar symbol not found".to_string(),
                        None,
                    ))),
                }
            },
            _ => return Err(Box::new(DiagnosticError::runtime_error(
                "string-join grammar must be a symbol".to_string(),
                None,
            ))),
        }
    } else {
        "infix".to_string()
    };
    
    let string_list: Result<Vec<String>> = strings.iter()
        .map(|v| Ok(extract_string(v, "string-join")?.to_string()))
        .collect();
    let string_list = string_list?;
    
    let result = match grammar.as_str() {
        "infix" => string_list.join(delimiter),
        "prefix" => {
            if string_list.is_empty() {
                String::new()
            } else {
                format!("{}{}", delimiter, string_list.join(delimiter))
            }
        },
        "suffix" => {
            if string_list.is_empty() {
                String::new()
            } else {
                format!("{}{}", string_list.join(delimiter), delimiter)
            }
        },
        _ => return Err(Box::new(DiagnosticError::runtime_error(
            "string-join grammar must be 'infix, 'prefix, or 'suffix".to_string(),
            None,
        ))),
    };
    
    Ok(Value::string(result))
}

/// string-concatenate procedure
pub fn primitive_string_concatenate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-concatenate expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    let strings = args[0].as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-concatenate requires a list argument".to_string(),
            None,
        )
    })?;
    
    let mut result = String::new();
    
    for value in strings {
        let s = extract_string(&value, "string-concatenate")?;
        result.push_str(s);
    }
    
    Ok(Value::string(result))
}

/// string-concatenate-reverse procedure
pub fn primitive_string_concatenate_reverse(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-concatenate-reverse expects 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let strings = args[0].as_list().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-concatenate-reverse requires a list argument".to_string(),
            None,
        )
    })?;
    
    let final_string = if args.len() > 1 {
        extract_string(&args[1], "string-concatenate-reverse")?
    } else {
        ""
    };
    
    let mut result = String::new();
    
    // Concatenate in reverse order
    for value in strings.iter().rev() {
        let s = extract_string(value, "string-concatenate-reverse")?;
        result.push_str(s);
    }
    
    result.push_str(final_string);
    
    Ok(Value::string(result))
}

// SRFI-13 String Selection

/// string-take procedure - take first n characters
pub fn primitive_string_take(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-take expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-take")?;
    let n = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-take second argument must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    let chars: Vec<char> = s.chars().collect();
    
    if n > chars.len() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-take index out of bounds".to_string(),
            None,
        )));
    }
    
    let result: String = chars[..n].iter().collect();
    Ok(Value::string(result))
}

/// string-drop procedure - drop first n characters
pub fn primitive_string_drop(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-drop expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-drop")?;
    let n = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-drop second argument must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    let chars: Vec<char> = s.chars().collect();
    
    if n > chars.len() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-drop index out of bounds".to_string(),
            None,
        )));
    }
    
    let result: String = chars[n..].iter().collect();
    Ok(Value::string(result))
}

/// string-take-right procedure - take last n characters
pub fn primitive_string_take_right(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-take-right expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-take-right")?;
    let n = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-take-right second argument must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    let chars: Vec<char> = s.chars().collect();
    
    if n > chars.len() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-take-right index out of bounds".to_string(),
            None,
        )));
    }
    
    let start = chars.len() - n;
    let result: String = chars[start..].iter().collect();
    Ok(Value::string(result))
}

/// string-drop-right procedure - drop last n characters
pub fn primitive_string_drop_right(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-drop-right expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-drop-right")?;
    let n = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-drop-right second argument must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    let chars: Vec<char> = s.chars().collect();
    
    if n > chars.len() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-drop-right index out of bounds".to_string(),
            None,
        )));
    }
    
    let end = chars.len() - n;
    let result: String = chars[..end].iter().collect();
    Ok(Value::string(result))
}

/// string-pad procedure - pad string to given width
pub fn primitive_string_pad(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-pad expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-pad")?;
    let len = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-pad length must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    let pad_char = if args.len() > 2 {
        extract_character(&args[2], "string-pad")?
    } else {
        ' '
    };
    
    let start = if args.len() > 3 {
        args[3].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let chars: Vec<char> = s.chars().collect();
    let s_len = chars.len();
    
    if start > s_len {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-pad start index out of bounds".to_string(),
            None,
        )));
    }
    
    let text: String = chars[start..].iter().collect();
    let text_len = text.chars().count();
    
    if text_len >= len {
        // Truncate from the left
        let text_chars: Vec<char> = text.chars().collect();
        let result: String = text_chars[text_len - len..].iter().collect();
        Ok(Value::string(result))
    } else {
        // Pad on the left
        let padding = pad_char.to_string().repeat(len - text_len);
        Ok(Value::string(format!("{padding}{text}")))
    }
}

/// string-pad-right procedure - pad string to given width on the right
pub fn primitive_string_pad_right(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-pad-right expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-pad-right")?;
    let len = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-pad-right length must be an integer".to_string(),
            None,
        )
    })? as usize;
    
    let pad_char = if args.len() > 2 {
        extract_character(&args[2], "string-pad-right")?
    } else {
        ' '
    };
    
    let start = if args.len() > 3 {
        args[3].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let chars: Vec<char> = s.chars().collect();
    let s_len = chars.len();
    
    if start > s_len {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-pad-right start index out of bounds".to_string(),
            None,
        )));
    }
    
    let text: String = chars[start..].iter().collect();
    let text_len = text.chars().count();
    
    if text_len >= len {
        // Truncate from the right
        let text_chars: Vec<char> = text.chars().collect();
        let result: String = text_chars[..len].iter().collect();
        Ok(Value::string(result))
    } else {
        // Pad on the right
        let padding = pad_char.to_string().repeat(len - text_len);
        Ok(Value::string(format!("{text}{padding}")))
    }
}

// SRFI-13 Enhanced Trimming

/// string-trim procedure (SRFI-13 enhanced version)
pub fn primitive_string_trim_srfi13(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-trim expects 1 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-trim")?;
    
    let charset = if args.len() > 1 {
        CharacterSet::from_value(&args[1])?
    } else {
        CharacterSet::default()
    };
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-trim indices out of bounds".to_string(),
            None,
        )));
    }
    
    let slice = &chars[start..end];
    
    // Find first non-matching character
    let trim_start = slice.iter().position(|&ch| !charset.contains(ch)).unwrap_or(slice.len());
    
    let result: String = slice[trim_start..].iter().collect();
    Ok(Value::string(result))
}

/// string-trim-right procedure (SRFI-13 enhanced version)
pub fn primitive_string_trim_right_srfi13(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-trim-right expects 1 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-trim-right")?;
    
    let charset = if args.len() > 1 {
        CharacterSet::from_value(&args[1])?
    } else {
        CharacterSet::default()
    };
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-trim-right indices out of bounds".to_string(),
            None,
        )));
    }
    
    let slice = &chars[start..end];
    
    // Find last non-matching character from the right
    let trim_end = slice.iter().rposition(|&ch| !charset.contains(ch))
        .map(|pos| pos + 1)
        .unwrap_or(0);
    
    let result: String = slice[..trim_end].iter().collect();
    Ok(Value::string(result))
}

/// string-trim-both procedure
pub fn primitive_string_trim_both(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-trim-both expects 1 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-trim-both")?;
    
    let charset = if args.len() > 1 {
        CharacterSet::from_value(&args[1])?
    } else {
        CharacterSet::default()
    };
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-trim-both indices out of bounds".to_string(),
            None,
        )));
    }
    
    let slice = &chars[start..end];
    
    // Find first non-matching character
    let trim_start = slice.iter().position(|&ch| !charset.contains(ch)).unwrap_or(slice.len());
    
    if trim_start == slice.len() {
        return Ok(Value::string(String::new()));
    }
    
    // Find last non-matching character from the right  
    let trim_end = slice.iter().rposition(|&ch| !charset.contains(ch))
        .map(|pos| pos + 1)
        .unwrap_or(0);
    
    let result: String = slice[trim_start..trim_end].iter().collect();
    Ok(Value::string(result))
}

// SRFI-13 String Searching

/// string-index procedure - find first occurrence of character in character set
pub fn primitive_string_index(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-index expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-index")?;
    let charset = CharacterSet::from_value(&args[1])?;
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-index indices out of bounds".to_string(),
            None,
        )));
    }
    
    for (i, &ch) in chars[start..end].iter().enumerate() {
        if charset.contains(ch) {
            return Ok(Value::integer((start + i) as i64));
        }
    }
    
    Ok(Value::boolean(false))
}

/// string-rindex procedure - find last occurrence from left
pub fn primitive_string_rindex(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-rindex expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-rindex")?;
    let charset = CharacterSet::from_value(&args[1])?;
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-rindex indices out of bounds".to_string(),
            None,
        )));
    }
    
    for (i, &ch) in chars[start..end].iter().enumerate().rev() {
        if charset.contains(ch) {
            return Ok(Value::integer((start + i) as i64));
        }
    }
    
    Ok(Value::boolean(false))
}

/// string-index-right procedure - find first occurrence from right
pub fn primitive_string_index_right(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-index-right expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-index-right")?;
    let charset = CharacterSet::from_value(&args[1])?;
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-index-right indices out of bounds".to_string(),
            None,
        )));
    }
    
    for i in (start..end).rev() {
        if charset.contains(chars[i]) {
            return Ok(Value::integer(i as i64));
        }
    }
    
    Ok(Value::boolean(false))
}

/// string-skip procedure - find first character NOT in character set
pub fn primitive_string_skip(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-skip expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-skip")?;
    let charset = CharacterSet::from_value(&args[1])?;
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-skip indices out of bounds".to_string(),
            None,
        )));
    }
    
    for (i, &ch) in chars[start..end].iter().enumerate() {
        if !charset.contains(ch) {
            return Ok(Value::integer((start + i) as i64));
        }
    }
    
    Ok(Value::boolean(false))
}

/// string-skip-right procedure - find first character NOT in character set from right
pub fn primitive_string_skip_right(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-skip-right expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-skip-right")?;
    let charset = CharacterSet::from_value(&args[1])?;
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-skip-right indices out of bounds".to_string(),
            None,
        )));
    }
    
    for i in (start..end).rev() {
        if !charset.contains(chars[i]) {
            return Ok(Value::integer(i as i64));
        }
    }
    
    Ok(Value::boolean(false))
}

/// string-contains procedure - find substring, return index or false
pub fn primitive_string_contains(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-contains expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let haystack = extract_string(&args[0], "string-contains")?;
    let needle = extract_string(&args[1], "string-contains")?;
    
    let h_chars: Vec<char> = haystack.chars().collect();
    let len = h_chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-contains indices out of bounds".to_string(),
            None,
        )));
    }
    
    let search_str: String = h_chars[start..end].iter().collect();
    
    match search_str.find(needle) {
        Some(pos) => Ok(Value::integer((start + pos) as i64)),
        None => Ok(Value::boolean(false)),
    }
}

/// string-contains-ci procedure - case-insensitive substring search
pub fn primitive_string_contains_ci(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-contains-ci expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let haystack = extract_string(&args[0], "string-contains-ci")?;
    let needle = extract_string(&args[1], "string-contains-ci")?;
    
    let h_chars: Vec<char> = haystack.chars().collect();
    let len = h_chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-contains-ci indices out of bounds".to_string(),
            None,
        )));
    }
    
    let search_str: String = h_chars[start..end].iter().collect();
    let search_lower = search_str.to_lowercase();
    let needle_lower = needle.to_lowercase();
    
    match search_lower.find(&needle_lower) {
        Some(pos) => Ok(Value::integer((start + pos) as i64)),
        None => Ok(Value::boolean(false)),
    }
}

/// string-count procedure - count occurrences of characters in character set
pub fn primitive_string_count(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 4 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-count expects 2 to 4 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-count")?;
    let charset = CharacterSet::from_value(&args[1])?;
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 2 {
        args[2].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 3 {
        args[3].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-count indices out of bounds".to_string(),
            None,
        )));
    }
    
    let count = chars[start..end].iter()
        .filter(|&&ch| charset.contains(ch))
        .count();
    
    Ok(Value::integer(count as i64))
}

// SRFI-13 String Comparison and Hashing

/// string-compare procedure - three-way comparison
pub fn primitive_string_compare(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 6 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-compare expects 2 to 6 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s1 = extract_string(&args[0], "string-compare")?;
    let s2 = extract_string(&args[1], "string-compare")?;
    
    // For now, implement basic three-way comparison
    // Full SRFI-13 allows procedures for =, <, > cases
    let cmp = s1.cmp(s2);
    
    match cmp {
        std::cmp::Ordering::Less => Ok(Value::integer(-1)),
        std::cmp::Ordering::Equal => Ok(Value::integer(0)),
        std::cmp::Ordering::Greater => Ok(Value::integer(1)),
    }
}

/// string-compare-ci procedure - case-insensitive three-way comparison
pub fn primitive_string_compare_ci(args: &[Value]) -> Result<Value> {
    if args.len() < 2 || args.len() > 6 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-compare-ci expects 2 to 6 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s1 = extract_string(&args[0], "string-compare-ci")?;
    let s2 = extract_string(&args[1], "string-compare-ci")?;
    
    let s1_lower = s1.to_lowercase();
    let s2_lower = s2.to_lowercase();
    
    let cmp = s1_lower.cmp(&s2_lower);
    
    match cmp {
        std::cmp::Ordering::Less => Ok(Value::integer(-1)),
        std::cmp::Ordering::Equal => Ok(Value::integer(0)),
        std::cmp::Ordering::Greater => Ok(Value::integer(1)),
    }
}

/// string-hash procedure - compute hash of string
pub fn primitive_string_hash(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-hash expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-hash")?;
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 1 {
        args[1].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        args[2].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-hash indices out of bounds".to_string(),
            None,
        )));
    }
    
    let substring: String = chars[start..end].iter().collect();
    
    let mut hasher = DefaultHasher::new();
    substring.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Return as positive integer within i64 range
    Ok(Value::integer((hash as i64).abs()))
}

/// string-hash-ci procedure - case-insensitive hash
pub fn primitive_string_hash_ci(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-hash-ci expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-hash-ci")?;
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 1 {
        args[1].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        args[2].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-hash-ci indices out of bounds".to_string(),
            None,
        )));
    }
    
    let substring: String = chars[start..end].iter().collect();
    let lower = substring.to_lowercase();
    
    let mut hasher = DefaultHasher::new();
    lower.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Return as positive integer within i64 range
    Ok(Value::integer((hash as i64).abs()))
}

// SRFI-13 String Modification

/// string-titlecase procedure - convert to title case
pub fn primitive_string_titlecase(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-titlecase expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-titlecase")?;
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 1 {
        args[1].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        args[2].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-titlecase indices out of bounds".to_string(),
            None,
        )));
    }
    
    let mut result = String::new();
    let mut at_word_start = true;
    
    for &ch in &chars[start..end] {
        if ch.is_alphabetic() {
            if at_word_start {
                result.extend(ch.to_uppercase());
                at_word_start = false;
            } else {
                result.extend(ch.to_lowercase());
            }
        } else {
            result.push(ch);
            at_word_start = ch.is_whitespace() || ch.is_ascii_punctuation();
        }
    }
    
    // Copy unchanged parts
    let prefix: String = chars[..start].iter().collect();
    let suffix: String = chars[end..].iter().collect();
    
    Ok(Value::string(format!("{prefix}{result}{suffix}")))
}

/// string-reverse procedure - reverse string
pub fn primitive_string_reverse(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-reverse expects 1 to 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-reverse")?;
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    let start = if args.len() > 1 {
        args[1].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        args[2].as_integer().unwrap_or(len as i64) as usize
    } else {
        len
    };
    
    if start > len || end > len || start > end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-reverse indices out of bounds".to_string(),
            None,
        )));
    }
    
    let mut reversed_chars: Vec<char> = chars[start..end].to_vec();
    reversed_chars.reverse();
    
    // Copy unchanged parts
    let prefix: String = chars[..start].iter().collect();
    let suffix: String = chars[end..].iter().collect();
    let reversed: String = reversed_chars.iter().collect();
    
    Ok(Value::string(format!("{prefix}{reversed}{suffix}")))
}

/// string-replace procedure (SRFI-13 enhanced version)
pub fn primitive_string_replace_srfi13(args: &[Value]) -> Result<Value> {
    if args.len() < 4 || args.len() > 6 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("string-replace expects 4 to 6 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let s = extract_string(&args[0], "string-replace")?;
    let from_start = args[1].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-replace from-start must be an integer".to_string(),
            None,
        )
    })? as usize;
    let from_end = args[2].as_integer().ok_or_else(|| {
        DiagnosticError::runtime_error(
            "string-replace from-end must be an integer".to_string(),
            None,
        )
    })? as usize;
    let replacement = extract_string(&args[3], "string-replace")?;
    
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    
    if from_start > len || from_end > len || from_start > from_end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-replace indices out of bounds".to_string(),
            None,
        )));
    }
    
    // Optional start/end for replacement string
    let repl_start = if args.len() > 4 {
        args[4].as_integer().unwrap_or(0) as usize
    } else {
        0
    };
    
    let repl_chars: Vec<char> = replacement.chars().collect();
    let repl_len = repl_chars.len();
    let repl_end = if args.len() > 5 {
        args[5].as_integer().unwrap_or(repl_len as i64) as usize
    } else {
        repl_len
    };
    
    if repl_start > repl_len || repl_end > repl_len || repl_start > repl_end {
        return Err(Box::new(DiagnosticError::runtime_error(
            "string-replace replacement indices out of bounds".to_string(),
            None,
        )));
    }
    
    let prefix: String = chars[..from_start].iter().collect();
    let suffix: String = chars[from_end..].iter().collect();
    let replacement_part: String = repl_chars[repl_start..repl_end].iter().collect();
    
    Ok(Value::string(format!("{prefix}{replacement_part}{suffix}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_string_predicates() {
        let args = vec![Value::string("hello")];
        let result = primitive_string_p(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::integer(42)];
        let result = primitive_string_p(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_string_length() {
        let args = vec![Value::string("hello")];
        let result = primitive_string_length(&args).unwrap();
        assert_eq!(result, Value::integer(5));
        
        let args = vec![Value::string("")];
        let result = primitive_string_length(&args).unwrap();
        assert_eq!(result, Value::integer(0));
    }
    
    #[test]
    fn test_string_append() {
        let args = vec![
            Value::string("hello"),
            Value::string(" "),
            Value::string("world"),
        ];
        let result = primitive_string_append(&args).unwrap();
        assert_eq!(result, Value::string("hello world"));
        
        // Empty append
        let args = vec![];
        let result = primitive_string_append(&args).unwrap();
        assert_eq!(result, Value::string(""));
    }
    
    #[test]
    fn test_string_ref() {
        let args = vec![Value::string("hello"), Value::integer(1)];
        let result = primitive_string_ref(&args).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('e')));
    }
    
    #[test]
    fn test_string_comparison() {
        let args = vec![Value::string("abc"), Value::string("abc")];
        let result = primitive_string_equal(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::string("abc"), Value::string("def")];
        let result = primitive_string_less(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
    }
    
    #[test]
    fn test_case_conversion() {
        let args = vec![Value::string("Hello World")];
        let result = primitive_string_upcase(&args).unwrap();
        assert_eq!(result, Value::string("HELLO WORLD"));
        
        let args = vec![Value::string("Hello World")];
        let result = primitive_string_downcase(&args).unwrap();
        assert_eq!(result, Value::string("hello world"));
    }
    
    #[test]
    fn test_substring() {
        let args = vec![
            Value::string("hello world"),
            Value::integer(6),
            Value::integer(11),
        ];
        let result = primitive_substring(&args).unwrap();
        assert_eq!(result, Value::string("world"));
    }
    
    #[test]
    fn test_string_map_basic() {
        // Test string-map with char-upcase
        let upcase_proc = Arc::new(PrimitiveProcedure {
            name: "char-upcase".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| {
                match &args[0] {
                    Value::Literal(crate::ast::Literal::Character(ch)) => {
                        Ok(Value::Literal(crate::ast::Literal::Character(ch.to_ascii_uppercase())))
                    },
                    _ => Ok(Value::Unspecified)
                }
            }),
            effects: vec![Effect::Pure],
        });
        
        let string = Value::string("hello");
        let args = vec![Value::Primitive(upcase_proc), string];
        let result = primitive_string_map(&args).unwrap();
        
        assert_eq!(result, Value::string("HELLO"));
    }
    
    #[test]
    fn test_string_map_multiple_strings() {
        // Test string-map with multiple strings - returns first character
        let first_char_proc = Arc::new(PrimitiveProcedure {
            name: "first-char".to_string(),
            arity_min: 1,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(|args| {
                // Just return the first character argument
                Ok(args[0].clone())
            }),
            effects: vec![Effect::Pure],
        });
        
        let string1 = Value::string("abc");
        let string2 = Value::string("xyz");
        let args = vec![Value::Primitive(first_char_proc), string1, string2];
        let result = primitive_string_map(&args).unwrap();
        
        assert_eq!(result, Value::string("axy"));
    }
    
    #[test]
    fn test_string_map_different_lengths() {
        // Test string-map with strings of different lengths - should use shortest
        let identity_proc = Arc::new(PrimitiveProcedure {
            name: "identity".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| Ok(args[0].clone())),
            effects: vec![Effect::Pure],
        });
        
        let string1 = Value::string("ab");
        let string2 = Value::string("xyz");
        let args = vec![Value::Primitive(identity_proc), string1, string2];
        let result = primitive_string_map(&args).unwrap();
        
        assert_eq!(result, Value::string("ab"));
    }
    
    #[test]
    fn test_string_map_empty_string() {
        let identity_proc = Arc::new(PrimitiveProcedure {
            name: "identity".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| Ok(args[0].clone())),
            effects: vec![Effect::Pure],
        });
        
        let empty_string = Value::string("");
        let args = vec![Value::Primitive(identity_proc), empty_string];
        let result = primitive_string_map(&args).unwrap();
        
        assert_eq!(result, Value::string(""));
    }
    
    #[test]
    fn test_string_for_each_basic() {
        // Test string-for-each with a simple side-effect procedure
        let identity_proc = Arc::new(PrimitiveProcedure {
            name: "identity".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| Ok(args[0].clone())),
            effects: vec![Effect::Pure],
        });
        
        let string = Value::string("hello");
        let args = vec![Value::Primitive(identity_proc), string];
        let result = primitive_string_for_each(&args).unwrap();
        
        // string-for-each should return unspecified
        assert_eq!(result, Value::Unspecified);
    }
    
    #[test]
    fn test_string_for_each_multiple_strings() {
        let first_arg_proc = Arc::new(PrimitiveProcedure {
            name: "first-arg".to_string(),
            arity_min: 1,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(|args| Ok(args[0].clone())),
            effects: vec![Effect::Pure],
        });
        
        let string1 = Value::string("ab");
        let string2 = Value::string("xy");
        let args = vec![Value::Primitive(first_arg_proc), string1, string2];
        let result = primitive_string_for_each(&args).unwrap();
        
        assert_eq!(result, Value::Unspecified);
    }
    
    #[test]
    fn test_string_map_for_each_errors() {
        // Test errors for both string-map and string-for-each
        
        // Non-procedure first argument
        let args = vec![Value::integer(42), Value::string("hello")];
        assert!(primitive_string_map(&args).is_err());
        assert!(primitive_string_for_each(&args).is_err());
        
        // Non-string argument
        let proc = Arc::new(PrimitiveProcedure {
            name: "test".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|args| Ok(args[0].clone())),
            effects: vec![Effect::Pure],
        });
        let args = vec![Value::Primitive(proc.clone()), Value::integer(42)];
        assert!(primitive_string_map(&args).is_err());
        assert!(primitive_string_for_each(&args).is_err());
        
        // Too few arguments
        assert!(primitive_string_map(&[]).is_err());
        assert!(primitive_string_for_each(&[]).is_err());
        
        let args = vec![Value::Primitive(proc)];
        assert!(primitive_string_map(&args).is_err());
        assert!(primitive_string_for_each(&args).is_err());
    }
    
    #[test]
    fn test_string_map_return_type_error() {
        // Test that string-map requires character return type
        let bad_proc = Arc::new(PrimitiveProcedure {
            name: "bad-proc".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(|_args| Ok(Value::integer(42))),
            effects: vec![Effect::Pure],
        });
        
        let string = Value::string("a");
        let args = vec![Value::Primitive(bad_proc), string];
        let result = primitive_string_map(&args);
        
        assert!(result.is_err());
    }

    // ============= R7RS-SMALL MUTABLE STRING TESTS =============

    #[test]
    fn test_make_string_mutable() {
        // Test make-string creates mutable strings
        let args = vec![Value::integer(5), Value::Literal(crate::ast::Literal::Character('x'))];
        let result = primitive_make_string(&args).unwrap();
        
        // Should be a mutable string
        assert!(result.is_mutable_string());
        assert_eq!(result.string_length(), Some(5));
        assert_eq!(result.as_string_owned(), Some("xxxxx".to_string()));
    }

    #[test]
    fn test_make_string_default_fill() {
        // Test make-string with default fill character (space)
        let args = vec![Value::integer(3)];
        let result = primitive_make_string(&args).unwrap();
        
        assert!(result.is_mutable_string());
        assert_eq!(result.string_length(), Some(3));
        assert_eq!(result.as_string_owned(), Some("   ".to_string()));
    }

    #[test]
    fn test_string_set() {
        // Test string-set! on mutable string
        let mut_str = Value::mutable_string("hello");
        let args = vec![
            mut_str.clone(),
            Value::integer(1),
            Value::Literal(crate::ast::Literal::Character('a'))
        ];
        
        let result = primitive_string_set(&args).unwrap();
        assert_eq!(result, Value::Unspecified);
        
        // Check that the string was modified
        assert_eq!(mut_str.as_string_owned(), Some("hallo".to_string()));
    }

    #[test]
    fn test_string_set_immutable_error() {
        // Test string-set! on immutable string should fail
        let immut_str = Value::string("hello");
        let args = vec![
            immut_str,
            Value::integer(1),
            Value::Literal(crate::ast::Literal::Character('a'))
        ];
        
        let result = primitive_string_set(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_set_bounds_error() {
        // Test string-set! with out-of-bounds index
        let mut_str = Value::mutable_string("hi");
        let args = vec![
            mut_str,
            Value::integer(5), // Out of bounds
            Value::Literal(crate::ast::Literal::Character('a'))
        ];
        
        let result = primitive_string_set(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_fill() {
        // Test string-fill! on mutable string
        let mut_str = Value::mutable_string("hello");
        let args = vec![
            mut_str.clone(),
            Value::Literal(crate::ast::Literal::Character('z'))
        ];
        
        let result = primitive_string_fill(&args).unwrap();
        assert_eq!(result, Value::Unspecified);
        
        // Check that the string was filled
        assert_eq!(mut_str.as_string_owned(), Some("zzzzz".to_string()));
    }

    #[test]
    fn test_string_fill_partial() {
        // Test string-fill! with start and end indices
        let mut_str = Value::mutable_string("hello");
        let args = vec![
            mut_str.clone(),
            Value::Literal(crate::ast::Literal::Character('x')),
            Value::integer(1), // start
            Value::integer(4)  // end
        ];
        
        let result = primitive_string_fill(&args).unwrap();
        assert_eq!(result, Value::Unspecified);
        
        // Should be "hxxxo"
        assert_eq!(mut_str.as_string_owned(), Some("hxxxo".to_string()));
    }

    #[test]
    fn test_string_fill_immutable_error() {
        // Test string-fill! on immutable string should fail
        let immut_str = Value::string("hello");
        let args = vec![
            immut_str,
            Value::Literal(crate::ast::Literal::Character('x'))
        ];
        
        let result = primitive_string_fill(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_ref_mutable() {
        // Test string-ref works with mutable strings
        let mut_str = Value::mutable_string("hello");
        let args = vec![mut_str, Value::integer(2)];
        
        let result = primitive_string_ref(&args).unwrap();
        assert_eq!(result, Value::Literal(crate::ast::Literal::Character('l')));
    }

    #[test]
    fn test_string_length_mutable() {
        // Test string-length works with mutable strings
        let mut_str = Value::mutable_string("test");
        let args = vec![mut_str];
        
        let result = primitive_string_length(&args).unwrap();
        assert_eq!(result, Value::integer(4));
    }

    #[test]
    fn test_string_comparison_mixed() {
        // Test string comparison between mutable and immutable strings
        let immut_str = Value::string("hello");
        let mut_str = Value::mutable_string("hello");
        
        // Test equality
        let args = vec![immut_str.clone(), mut_str.clone()];
        let result = primitive_string_equal(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        // Test inequality
        let mut_str2 = Value::mutable_string("world");
        let args = vec![immut_str, mut_str2];
        let result = primitive_string_less(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
    }

    #[test]
    fn test_string_append_mixed() {
        // Test string-append with mixed mutable and immutable strings
        let immut_str = Value::string("hello");
        let mut_str = Value::mutable_string(" world");
        
        let args = vec![immut_str, mut_str];
        let result = primitive_string_append(&args).unwrap();
        
        assert_eq!(result, Value::string("hello world"));
        assert!(result.is_immutable_string()); // Result should be immutable
    }

    #[test]
    fn test_substring_mutable() {
        // Test substring works with mutable strings
        let mut_str = Value::mutable_string("hello world");
        let args = vec![mut_str, Value::integer(6), Value::integer(11)];
        
        let result = primitive_substring(&args).unwrap();
        assert_eq!(result, Value::string("world"));
        assert!(result.is_immutable_string()); // Result should be immutable
    }

    #[test]
    fn test_r7rs_small_string_predicates() {
        // Test string? predicate works with both string types
        let immut_str = Value::string("test");
        let mut_str = Value::mutable_string("test");
        let not_str = Value::integer(42);
        
        assert_eq!(primitive_string_p(&vec![immut_str]).unwrap(), Value::boolean(true));
        assert_eq!(primitive_string_p(&vec![mut_str]).unwrap(), Value::boolean(true));
        assert_eq!(primitive_string_p(&vec![not_str]).unwrap(), Value::boolean(false));
    }

    #[test]
    fn test_error_handling_comprehensive() {
        // Test comprehensive error handling for all functions
        
        // Wrong argument count for make-string
        assert!(primitive_make_string(&vec![]).is_err());
        assert!(primitive_make_string(&vec![Value::integer(1), Value::Literal(crate::ast::Literal::Character('x')), Value::integer(2)]).is_err());
        
        // Non-integer length for make-string
        assert!(primitive_make_string(&vec![Value::string("not-a-number")]).is_err());
        
        // Negative length for make-string
        assert!(primitive_make_string(&vec![Value::integer(-1)]).is_err());
        
        // Wrong argument count for string-set!
        assert!(primitive_string_set(&vec![]).is_err());
        assert!(primitive_string_set(&vec![Value::mutable_string("test")]).is_err());
        
        // Wrong argument types for string-set!
        assert!(primitive_string_set(&vec![
            Value::integer(42),
            Value::integer(0),
            Value::Literal(crate::ast::Literal::Character('a'))
        ]).is_err());
        
        // Wrong argument count for string-fill!
        assert!(primitive_string_fill(&vec![Value::mutable_string("test")]).is_err());
    }
}
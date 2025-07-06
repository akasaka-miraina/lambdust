//! SRFI 13: String Libraries - Search operations
//!
//! This module implements string searching, prefix/suffix operations.

use crate::builtins::utils::{check_arity_range, expect_two_strings, make_builtin_procedure};
use crate::error::LambdustError;
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register search and prefix/suffix functions
pub fn register_functions(builtins: &mut HashMap<String, Value>) {
    // String prefix & suffix
    builtins.insert("string-prefix?".to_string(), string_prefix_function());
    builtins.insert("string-suffix?".to_string(), string_suffix_function());
    builtins.insert("string-prefix-ci?".to_string(), string_prefix_ci_function());
    builtins.insert("string-suffix-ci?".to_string(), string_suffix_ci_function());

    // String search
    builtins.insert("string-index".to_string(), string_index_function());
    builtins.insert("string-index-right".to_string(), string_index_right_function());
    builtins.insert("string-skip".to_string(), string_skip_function());
    builtins.insert("string-skip-right".to_string(), string_skip_right_function());
    builtins.insert("string-count".to_string(), string_count_function());
    builtins.insert("string-contains".to_string(), string_contains_function());
    builtins.insert("string-contains-ci".to_string(), string_contains_ci_function());
}

/// Create string-prefix? function
fn string_prefix_function() -> Value {
    make_builtin_procedure("string-prefix?", Some(2), |args| {
        check_arity_range(args, 2, Some(4))?;
        let (s1, s2) = expect_two_strings(args, "string-prefix?")?;
        Ok(Value::Boolean(s2.starts_with(s1)))
    })
}

/// Create string-suffix? function  
fn string_suffix_function() -> Value {
    make_builtin_procedure("string-suffix?", Some(2), |args| {
        check_arity_range(args, 2, Some(4))?;
        let (s1, s2) = expect_two_strings(args, "string-suffix?")?;
        Ok(Value::Boolean(s2.ends_with(s1)))
    })
}

/// Create string-prefix-ci? function
fn string_prefix_ci_function() -> Value {
    make_builtin_procedure("string-prefix-ci?", Some(2), |args| {
        check_arity_range(args, 2, Some(4))?;
        let (s1, s2) = expect_two_strings(args, "string-prefix-ci?")?;
        Ok(Value::Boolean(s2.to_lowercase().starts_with(&s1.to_lowercase())))
    })
}

/// Create string-suffix-ci? function
fn string_suffix_ci_function() -> Value {
    make_builtin_procedure("string-suffix-ci?", Some(2), |args| {
        check_arity_range(args, 2, Some(4))?;
        let (s1, s2) = expect_two_strings(args, "string-suffix-ci?")?;
        Ok(Value::Boolean(s2.to_lowercase().ends_with(&s1.to_lowercase())))
    })
}

/// Create string-contains function
fn string_contains_function() -> Value {
    make_builtin_procedure("string-contains", Some(2), |args| {
        check_arity_range(args, 2, Some(4))?;
        let (s1, s2) = expect_two_strings(args, "string-contains")?;
        
        match s1.find(s2) {
            Some(index) => Ok(Value::Number(crate::lexer::SchemeNumber::Integer(index as i64))),
            None => Ok(Value::Boolean(false)),
        }
    })
}

/// Create string-contains-ci function
fn string_contains_ci_function() -> Value {
    make_builtin_procedure("string-contains-ci", Some(2), |args| {
        check_arity_range(args, 2, Some(4))?;
        let (s1, s2) = expect_two_strings(args, "string-contains-ci")?;
        
        match s1.to_lowercase().find(&s2.to_lowercase()) {
            Some(index) => Ok(Value::Number(crate::lexer::SchemeNumber::Integer(index as i64))),
            None => Ok(Value::Boolean(false)),
        }
    })
}

// Placeholder functions for advanced search operations
fn string_index_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-index".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error(
            "string-index requires evaluator integration for predicate calls".to_string(),
        )),
    })
}

fn string_index_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-index-right".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error(
            "string-index-right requires evaluator integration for predicate calls".to_string(),
        )),
    })
}

fn string_skip_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-skip".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error(
            "string-skip requires evaluator integration for predicate calls".to_string(),
        )),
    })
}

fn string_skip_right_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-skip-right".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error(
            "string-skip-right requires evaluator integration for predicate calls".to_string(),
        )),
    })
}

fn string_count_function() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string-count".to_string(),
        arity: None,
        func: |_args| Err(LambdustError::runtime_error(
            "string-count requires evaluator integration for predicate calls".to_string(),
        )),
    })
}
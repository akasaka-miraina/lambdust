//! Built-in functions for custom type predicates management

use crate::builtins::utils::{check_arity, check_arity_range, make_builtin_procedure};
use crate::error::LambdustError;
use crate::value::{Value, evaluate_global_custom_predicate, register_global_custom_predicate, global_custom_predicate_registry};
use std::collections::HashMap;

/// Register all custom predicate management functions
pub fn register_custom_predicate_functions(builtins: &mut HashMap<String, Value>) {
    // Core custom predicate functions
    builtins.insert("define-predicate".to_string(), define_predicate());
    builtins.insert("remove-predicate".to_string(), remove_predicate());
    builtins.insert("predicate-defined?".to_string(), predicate_defined());
    builtins.insert("list-predicates".to_string(), list_predicates());
    builtins.insert("predicate-info".to_string(), predicate_info());
    builtins.insert("clear-predicates".to_string(), clear_predicates());
    
    // Extension point for evaluating custom predicates
    builtins.insert("apply-predicate".to_string(), apply_predicate());
}

/// (define-predicate name predicate-procedure [description])
/// Define a custom type predicate
fn define_predicate() -> Value {
    make_builtin_procedure("define-predicate", None, |args| {
        check_arity_range(args, 2, Some(3))?;

        // Extract name (must be a string or symbol)
        let name = match &args[0] {
            Value::String(s) => s.clone(),
            Value::ShortString(s) => s.as_str().to_string(),
            Value::Symbol(s) => s.clone(),
            Value::ShortSymbol(s) => s.as_str().to_string(),
            _ => return Err(LambdustError::runtime_error("define-predicate: first argument must be a string or symbol")),
        };

        // Extract predicate procedure
        let predicate_proc = match &args[1] {
            Value::Procedure(proc) => proc.clone(),
            _ => return Err(LambdustError::runtime_error("define-predicate: second argument must be a procedure")),
        };

        // Extract optional description
        let description = if args.len() >= 3 {
            match &args[2] {
                Value::String(s) => Some(s.clone()),
                Value::ShortString(s) => Some(s.as_str().to_string()),
                _ => return Err(LambdustError::runtime_error("define-predicate: third argument must be a string")),
            }
        } else {
            None
        };

        // Create a simple predicate function based on procedure name
        // This is a simplified approach until full evaluator integration is available
        let procedure_name = match &predicate_proc {
            crate::value::Procedure::Builtin { name, .. } => name.clone(),
            _ => "unknown".to_string(),
        };
        
        let predicate_fn = move |value: &Value| -> bool {
            // For now, implement common predicate patterns based on procedure name
            match procedure_name.as_str() {
                "number?" => matches!(value, crate::value::Value::Number(_)),
                "string?" => matches!(value, crate::value::Value::String(_) | crate::value::Value::ShortString(_)),
                "boolean?" => matches!(value, crate::value::Value::Boolean(_)),
                "symbol?" => matches!(value, crate::value::Value::Symbol(_) | crate::value::Value::ShortSymbol(_)),
                "null?" => matches!(value, crate::value::Value::Nil),
                "pair?" => value.is_pair(),
                "list?" => value.is_list(),
                "vector?" => matches!(value, crate::value::Value::Vector(_)),
                "procedure?" => matches!(value, crate::value::Value::Procedure(_)),
                _ => false, // Unknown predicate - return false as default
            }
        };

        // Register the custom predicate
        match register_global_custom_predicate(name.clone(), description, predicate_fn) {
            Ok(()) => Ok(Value::Symbol(name)),
            Err(e) => Err(e),
        }
    })
}

/// (remove-predicate name)
/// Remove a custom predicate
fn remove_predicate() -> Value {
    make_builtin_procedure("remove-predicate", Some(1), |args| {
        check_arity(args, 1)?;

        let name = match &args[0] {
            Value::String(s) => s,
            Value::ShortString(s) => s.as_str(),
            Value::Symbol(s) => s,
            Value::ShortSymbol(s) => s.as_str(),
            _ => return Err(LambdustError::runtime_error("remove-predicate: argument must be a string or symbol")),
        };

        match global_custom_predicate_registry().unregister(name) {
            Ok(removed) => Ok(Value::Boolean(removed)),
            Err(e) => Err(e),
        }
    })
}

/// (predicate-defined? name)
/// Check if a custom predicate is defined
fn predicate_defined() -> Value {
    make_builtin_procedure("predicate-defined?", Some(1), |args| {
        check_arity(args, 1)?;

        let name = match &args[0] {
            Value::String(s) => s,
            Value::ShortString(s) => s.as_str(),
            Value::Symbol(s) => s,
            Value::ShortSymbol(s) => s.as_str(),
            _ => return Err(LambdustError::runtime_error("predicate-defined?: argument must be a string or symbol")),
        };

        match global_custom_predicate_registry().is_registered(name) {
            Ok(registered) => Ok(Value::Boolean(registered)),
            Err(e) => Err(e),
        }
    })
}

/// (list-predicates)
/// List all registered custom predicates
fn list_predicates() -> Value {
    make_builtin_procedure("list-predicates", Some(0), |args| {
        check_arity(args, 0)?;

        match global_custom_predicate_registry().list_predicates() {
            Ok(names) => {
                let values: Vec<Value> = names.into_iter()
                    .map(Value::String)
                    .collect();
                Ok(list_to_scheme_list(values))
            },
            Err(e) => Err(e),
        }
    })
}

/// (predicate-info name)
/// Get information about a custom predicate
fn predicate_info() -> Value {
    make_builtin_procedure("predicate-info", Some(1), |args| {
        check_arity(args, 1)?;

        let name = match &args[0] {
            Value::String(s) => s,
            Value::ShortString(s) => s.as_str(),
            Value::Symbol(s) => s,
            Value::ShortSymbol(s) => s.as_str(),
            _ => return Err(LambdustError::runtime_error("predicate-info: argument must be a string or symbol")),
        };

        match global_custom_predicate_registry().get_info(name) {
            Ok(Some(info)) => {
                // Return an association list with predicate information
                let mut result = Vec::new();
                
                // Name
                result.push(make_pair(
                    Value::Symbol("name".to_string()),
                    Value::String(info.name),
                ));
                
                // Description (if available)
                if let Some(desc) = info.description {
                    result.push(make_pair(
                        Value::Symbol("description".to_string()),
                        Value::String(desc),
                    ));
                }
                
                Ok(list_to_scheme_list(result))
            },
            Ok(None) => Ok(Value::Boolean(false)),
            Err(e) => Err(e),
        }
    })
}

/// (clear-predicates)
/// Clear all custom predicates
fn clear_predicates() -> Value {
    make_builtin_procedure("clear-predicates", Some(0), |args| {
        check_arity(args, 0)?;

        match global_custom_predicate_registry().clear() {
            Ok(()) => Ok(Value::Boolean(true)),
            Err(e) => Err(e),
        }
    })
}

/// (apply-predicate name value)
/// Apply a custom predicate to a value
fn apply_predicate() -> Value {
    make_builtin_procedure("apply-predicate", Some(2), |args| {
        check_arity(args, 2)?;

        let name = match &args[0] {
            Value::String(s) => s,
            Value::ShortString(s) => s.as_str(),
            Value::Symbol(s) => s,
            Value::ShortSymbol(s) => s.as_str(),
            _ => return Err(LambdustError::runtime_error("apply-predicate: first argument must be a string or symbol")),
        };

        let value = &args[1];

        match evaluate_global_custom_predicate(name, value) {
            Ok(Some(result)) => Ok(Value::Boolean(result)),
            Ok(None) => Err(LambdustError::runtime_error(format!("apply-predicate: predicate '{name}' not found"))),
            Err(e) => Err(e),
        }
    })
}

/// Helper function to create a pair (cons cell)
fn make_pair(first_value: Value, second_value: Value) -> Value {
    Value::Pair(std::rc::Rc::new(std::cell::RefCell::new(crate::value::PairData { car: first_value, cdr: second_value })))
}

/// Helper function to convert a Vec<Value> to a Scheme list
fn list_to_scheme_list(mut values: Vec<Value>) -> Value {
    values.reverse();
    let mut result = Value::Nil;
    for value in values {
        result = make_pair(value, result);
    }
    result
}

//! Vector operations for Scheme

use crate::error::LambdustError;
use crate::lexer::SchemeNumber;
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register all vector functions
pub fn register_vector_functions(builtins: &mut HashMap<String, Value>) {
    builtins.insert("vector".to_string(), vector_constructor());
    builtins.insert("vector-length".to_string(), vector_length());
    builtins.insert("vector-ref".to_string(), vector_ref());
    builtins.insert("make-vector".to_string(), vector_make());
    builtins.insert("vector->list".to_string(), vector_to_list());
    builtins.insert("list->vector".to_string(), list_to_vector());
}

fn vector_constructor() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "vector".to_string(),
        arity: None, // Variadic
        func: |args| {
            Ok(Value::Vector(args.to_vec()))
        },
    })
}

fn vector_length() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "vector-length".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            
            match &args[0] {
                Value::Vector(v) => Ok(Value::Number(SchemeNumber::Integer(v.len() as i64))),
                _ => Err(LambdustError::type_error(format!(
                    "vector-length: expected vector, got {}", args[0]
                ))),
            }
        },
    })
}

fn vector_ref() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "vector-ref".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }
            
            let vector = match &args[0] {
                Value::Vector(v) => v,
                _ => return Err(LambdustError::type_error(format!(
                    "vector-ref: expected vector, got {}", args[0]
                ))),
            };
            
            let index = match args[1].as_number() {
                Some(SchemeNumber::Integer(i)) if *i >= 0 => *i as usize,
                _ => return Err(LambdustError::type_error(format!(
                    "vector-ref: expected non-negative integer, got {}", args[1]
                ))),
            };
            
            if index >= vector.len() {
                return Err(LambdustError::runtime_error(format!(
                    "vector-ref: index {} out of bounds for vector of length {}", 
                    index, vector.len()
                )));
            }
            
            Ok(vector[index].clone())
        },
    })
}

fn vector_make() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "make-vector".to_string(),
        arity: None, // 1 or 2 arguments
        func: |args| {
            if args.is_empty() || args.len() > 2 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            
            let length = match args[0].as_number() {
                Some(SchemeNumber::Integer(n)) if *n >= 0 => *n as usize,
                _ => return Err(LambdustError::type_error(format!(
                    "make-vector: expected non-negative integer, got {}", args[0]
                ))),
            };
            
            let fill_value = if args.len() == 2 {
                args[1].clone()
            } else {
                Value::Boolean(false) // Default fill value
            };
            
            Ok(Value::Vector(vec![fill_value; length]))
        },
    })
}

fn vector_to_list() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "vector->list".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            
            match &args[0] {
                Value::Vector(v) => Ok(Value::from_vector(v.clone())),
                _ => Err(LambdustError::type_error(format!(
                    "vector->list: expected vector, got {}", args[0]
                ))),
            }
        },
    })
}

fn list_to_vector() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "list->vector".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            
            match args[0].to_vector() {
                Some(vec) => Ok(Value::Vector(vec)),
                None => Err(LambdustError::type_error(format!(
                    "list->vector: expected list, got {}", args[0]
                ))),
            }
        },
    })
}
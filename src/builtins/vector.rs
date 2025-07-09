//! Vector operations for Scheme

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::LambdustError;
use crate::lexer::SchemeNumber;
use crate::value::{Value, VectorStorage};
use std::collections::HashMap;

/// Register all vector functions
pub fn register_vector_functions(builtins: &mut HashMap<String, Value>) {
    builtins.insert("vector".to_string(), vector_constructor());
    builtins.insert("vector-length".to_string(), vector_length());
    builtins.insert("vector-ref".to_string(), vector_ref());
    builtins.insert("vector-set!".to_string(), vector_set());
    builtins.insert("make-vector".to_string(), vector_make());
    builtins.insert("vector->list".to_string(), vector_to_list());
    builtins.insert("list->vector".to_string(), list_to_vector());
}

fn vector_constructor() -> Value {
    make_builtin_procedure("vector", None, |args| Ok(Value::Vector(args.to_vec())))
}

fn vector_length() -> Value {
    make_builtin_procedure("vector-length", Some(1), |args| {
        check_arity(args, 1)?;
        match &args[0] {
            Value::Vector(v) => Ok(Value::Number(SchemeNumber::Integer(v.len() as i64))),
            Value::LazyVector(lazy_vec) => {
                let storage = lazy_vec.borrow();
                Ok(Value::Number(SchemeNumber::Integer(storage.len() as i64)))
            }
            _ => Err(LambdustError::type_error(format!(
                "vector-length: expected vector, got {}",
                args[0]
            ))),
        }
    })
}

fn vector_ref() -> Value {
    make_builtin_procedure("vector-ref", Some(2), |args| {
        check_arity(args, 2)?;

        let index = match args[1].as_number() {
            Some(SchemeNumber::Integer(i)) if *i >= 0 => *i as usize,
            _ => {
                return Err(LambdustError::type_error(format!(
                    "vector-ref: expected non-negative integer, got {}",
                    args[1]
                )));
            }
        };

        match &args[0] {
            Value::Vector(vector) => {
                if index >= vector.len() {
                    return Err(LambdustError::runtime_error(format!(
                        "vector-ref: index {} out of bounds for vector of length {}",
                        index,
                        vector.len()
                    )));
                }
                Ok(vector[index].clone())
            }
            Value::LazyVector(lazy_vec) => {
                let mut storage = lazy_vec.borrow_mut();
                storage.get(index)
            }
            _ => Err(LambdustError::type_error(format!(
                "vector-ref: expected vector, got {}",
                args[0]
            ))),
        }
    })
}

fn vector_set() -> Value {
    make_builtin_procedure("vector-set!", Some(3), |args| {
        check_arity(args, 3)?;

        let index = match args[1].as_number() {
            Some(SchemeNumber::Integer(i)) if *i >= 0 => *i as usize,
            _ => {
                return Err(LambdustError::type_error(format!(
                    "vector-set!: expected non-negative integer, got {}",
                    args[1]
                )));
            }
        };

        match &args[0] {
            Value::Vector(_) => {
                // Traditional vectors are immutable in our implementation
                // This is a design choice - we could make them mutable if needed
                Err(LambdustError::runtime_error(
                    "vector-set!: traditional vectors are immutable, use lazy vectors for mutation"
                ))
            }
            Value::LazyVector(lazy_vec) => {
                let mut storage = lazy_vec.borrow_mut();
                storage.set(index, args[2].clone())?;
                Ok(Value::Undefined) // vector-set! returns unspecified value
            }
            _ => Err(LambdustError::type_error(format!(
                "vector-set!: expected vector, got {}",
                args[0]
            ))),
        }
    })
}

fn vector_make() -> Value {
    make_builtin_procedure("make-vector", None, |args| {
        if args.is_empty() || args.len() > 2 {
            return Err(LambdustError::arity_error(1, args.len()));
        }

        let length = match args[0].as_number() {
            Some(SchemeNumber::Integer(n)) if *n >= 0 => *n as usize,
            _ => {
                return Err(LambdustError::type_error(format!(
                    "make-vector: expected non-negative integer, got {}",
                    args[0]
                )));
            }
        };

        let fill_value = if args.len() == 2 {
            args[1].clone()
        } else {
            Value::Boolean(false) // Default fill value
        };

        // Use lazy vector storage for memory efficiency
        match VectorStorage::new(length, fill_value) {
            Ok(VectorStorage::Materialized(vec)) => {
                // Small vector - use traditional Vector
                Ok(Value::Vector(vec))
            }
            Ok(storage) => {
                // Large vector - use LazyVector
                Ok(Value::LazyVector(std::rc::Rc::new(std::cell::RefCell::new(storage))))
            }
            Err(e) => Err(e),
        }
    })
}

fn vector_to_list() -> Value {
    make_builtin_procedure("vector->list", Some(1), |args| {
        check_arity(args, 1)?;

        match &args[0] {
            Value::Vector(v) => Ok(Value::from_vector(v.clone())),
            Value::LazyVector(lazy_vec) => {
                let storage = lazy_vec.borrow();
                let materialized = storage.to_materialized()?;
                Ok(Value::from_vector(materialized))
            }
            _ => Err(LambdustError::type_error(format!(
                "vector->list: expected vector, got {}",
                args[0]
            ))),
        }
    })
}

fn list_to_vector() -> Value {
    make_builtin_procedure("list->vector", Some(1), |args| {
        check_arity(args, 1)?;

        match args[0].to_vector() {
            Some(vec) => Ok(Value::Vector(vec)),
            None => Err(LambdustError::type_error(format!(
                "list->vector: expected list, got {}",
                args[0]
            ))),
        }
    })
}

//! List operations for Scheme

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::LambdustError;
use crate::value::Value;
use std::collections::HashMap;

/// Register all list operation functions
pub fn register_list_functions(builtins: &mut HashMap<String, Value>) {
    // Basic list operations
    builtins.insert("car".to_string(), list_car());
    builtins.insert("cdr".to_string(), list_cdr());
    builtins.insert("cons".to_string(), list_cons());
    builtins.insert("list".to_string(), list_list());
    builtins.insert("length".to_string(), list_length());
    builtins.insert("append".to_string(), list_append());
    builtins.insert("reverse".to_string(), list_reverse());

    // Destructive list operations (clone-based implementation)
    builtins.insert("set-car!".to_string(), list_set_car());
    builtins.insert("set-cdr!".to_string(), list_set_cdr());

    // List predicates
    builtins.insert("null?".to_string(), predicate_null());
    builtins.insert("pair?".to_string(), predicate_pair());
    builtins.insert("list?".to_string(), predicate_list());
}

// Basic list operations

fn list_car() -> Value {
    make_builtin_procedure("car", Some(1), |args| {
        check_arity(args, 1)?;
        match &args[0] {
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                Ok(pair.car.clone())
            }
            _ => Err(LambdustError::type_error(format!(
                "car: expected pair, got {}",
                args[0]
            ))),
        }
    })
}

fn list_cdr() -> Value {
    make_builtin_procedure("cdr", Some(1), |args| {
        check_arity(args, 1)?;
        match &args[0] {
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                Ok(pair.cdr.clone())
            }
            _ => Err(LambdustError::type_error(format!(
                "cdr: expected pair, got {}",
                args[0]
            ))),
        }
    })
}

fn list_cons() -> Value {
    make_builtin_procedure("cons", Some(2), |args| {
        check_arity(args, 2)?;
        Ok(Value::cons(args[0].clone(), args[1].clone()))
    })
}

fn list_list() -> Value {
    make_builtin_procedure("list", None, |args| Ok(Value::from_vector(args.to_vec())))
}

fn list_length() -> Value {
    make_builtin_procedure("length", Some(1), |args| {
        check_arity(args, 1)?;
        match args[0].list_length() {
            Some(len) => Ok(Value::Number(crate::lexer::SchemeNumber::Integer(
                len as i64,
            ))),
            None => Err(LambdustError::type_error(format!(
                "length: expected proper list, got {}",
                args[0]
            ))),
        }
    })
}

fn list_append() -> Value {
    make_builtin_procedure("append", None, |args| {
        if args.is_empty() {
            return Ok(Value::Nil);
        }

        let mut result = Vec::new();

        // Convert all but last argument to vectors and append
        for (i, arg) in args.iter().enumerate() {
            if i == args.len() - 1 {
                // Last argument can be any value (for dotted lists)
                if arg.is_nil() {
                    // If last is nil, we have a proper list
                    break;
                } else if let Some(vec) = arg.to_vector() {
                    // If last is a list, append it
                    result.extend(vec);
                    break;
                }
                // If last is not a list, create dotted list
                if result.is_empty() {
                    return Ok(arg.clone());
                }
                // Build proper list from result and make last cdr the non-list value
                let mut list = arg.clone();
                for item in result.into_iter().rev() {
                    list = Value::cons(item, list);
                }
                return Ok(list);
            }
            // All other arguments must be lists
            match arg.to_vector() {
                Some(vec) => result.extend(vec),
                None => {
                    return Err(LambdustError::type_error(format!(
                        "append: expected list, got {arg}"
                    )));
                }
            }
        }

        Ok(Value::from_vector(result))
    })
}

fn list_reverse() -> Value {
    make_builtin_procedure("reverse", Some(1), |args| {
        check_arity(args, 1)?;
        match args[0].to_vector() {
            Some(mut vec) => {
                vec.reverse();
                Ok(Value::from_vector(vec))
            }
            None => Err(LambdustError::type_error(format!(
                "reverse: expected list, got {}",
                args[0]
            ))),
        }
    })
}

// Destructive list operations (clone-based)

fn list_set_car() -> Value {
    make_builtin_procedure("set-car!", Some(2), |args| {
        check_arity(args, 2)?;

        match &args[0] {
            Value::Pair(_) => {
                // Use the new set_car! functionality for true mutation
                args[0]
                    .set_car(args[1].clone())
                    .map_err(LambdustError::runtime_error)?;
                Ok(args[0].clone())
            }
            _ => Err(LambdustError::type_error(format!(
                "set-car!: expected pair, got {}",
                args[0]
            ))),
        }
    })
}

fn list_set_cdr() -> Value {
    make_builtin_procedure("set-cdr!", Some(2), |args| {
        check_arity(args, 2)?;

        match &args[0] {
            Value::Pair(_) => {
                // Use the new set_cdr! functionality for true mutation
                args[0]
                    .set_cdr(args[1].clone())
                    .map_err(LambdustError::runtime_error)?;
                Ok(args[0].clone())
            }
            _ => Err(LambdustError::type_error(format!(
                "set-cdr!: expected pair, got {}",
                args[0]
            ))),
        }
    })
}

// List predicates

fn predicate_null() -> Value {
    make_builtin_procedure("null?", Some(1), |args| {
        check_arity(args, 1)?;
        Ok(Value::Boolean(args[0].is_nil()))
    })
}

fn predicate_pair() -> Value {
    make_builtin_procedure("pair?", Some(1), |args| {
        check_arity(args, 1)?;
        Ok(Value::Boolean(args[0].is_pair()))
    })
}

fn predicate_list() -> Value {
    make_builtin_procedure("list?", Some(1), |args| {
        check_arity(args, 1)?;
        Ok(Value::Boolean(args[0].is_list()))
    })
}

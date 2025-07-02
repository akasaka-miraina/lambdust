//! List operations for Scheme

use crate::error::LambdustError;
use crate::value::{Procedure, Value};
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
    Value::Procedure(Procedure::Builtin {
        name: "car".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            match &args[0] {
                Value::Pair(car, _) => Ok((**car).clone()),
                _ => Err(LambdustError::TypeError(format!(
                    "car: expected pair, got {}", args[0]
                ))),
            }
        },
    })
}

fn list_cdr() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "cdr".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            match &args[0] {
                Value::Pair(_, cdr) => Ok((**cdr).clone()),
                _ => Err(LambdustError::TypeError(format!(
                    "cdr: expected pair, got {}", args[0]
                ))),
            }
        },
    })
}

fn list_cons() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "cons".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            Ok(Value::Pair(Box::new(args[0].clone()), Box::new(args[1].clone())))
        },
    })
}

fn list_list() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "list".to_string(),
        arity: None, // Variadic
        func: |args| {
            Ok(Value::from_vector(args.to_vec()))
        },
    })
}

fn list_length() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "length".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            match args[0].list_length() {
                Some(len) => Ok(Value::Number(crate::lexer::SchemeNumber::Integer(len as i64))),
                None => Err(LambdustError::TypeError(format!(
                    "length: expected proper list, got {}", args[0]
                ))),
            }
        },
    })
}

fn list_append() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "append".to_string(),
        arity: None, // Variadic
        func: |args| {
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
                    } else {
                        // If last is not a list, create dotted list
                        if result.is_empty() {
                            return Ok(arg.clone());
                        } else {
                            // Build proper list from result and make last cdr the non-list value
                            let mut list = arg.clone();
                            for item in result.into_iter().rev() {
                                list = Value::Pair(Box::new(item), Box::new(list));
                            }
                            return Ok(list);
                        }
                    }
                } else {
                    // All other arguments must be lists
                    match arg.to_vector() {
                        Some(vec) => result.extend(vec),
                        None => return Err(LambdustError::TypeError(format!(
                            "append: expected list, got {}", arg
                        ))),
                    }
                }
            }
            
            Ok(Value::from_vector(result))
        },
    })
}

fn list_reverse() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "reverse".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            match args[0].to_vector() {
                Some(mut vec) => {
                    vec.reverse();
                    Ok(Value::from_vector(vec))
                }
                None => Err(LambdustError::TypeError(format!(
                    "reverse: expected list, got {}", args[0]
                ))),
            }
        },
    })
}

// Destructive list operations (clone-based)

fn list_set_car() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "set-car!".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::RuntimeError {
                    message: "set-car!: expected exactly 2 arguments".to_string(),
                    location: crate::error::SourceSpan::unknown(),
                    stack_trace: Vec::new(),
                });
            }

            match &args[0] {
                Value::Pair(_, cdr) => {
                    // Create new pair with new car and existing cdr
                    let new_pair = Value::Pair(Box::new(args[1].clone()), cdr.clone());
                    
                    // Note: In a true Scheme implementation, this would mutate the original pair
                    // Here we return the new pair, but this doesn't provide true mutation semantics
                    // A complete implementation would require using Rc<RefCell<>> throughout the Value system
                    
                    Ok(new_pair)
                }
                _ => Err(LambdustError::RuntimeError {
                    message: format!("set-car!: expected pair, got {}", args[0]),
                    location: crate::error::SourceSpan::unknown(),
                    stack_trace: Vec::new(),
                }),
            }
        },
    })
}

fn list_set_cdr() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "set-cdr!".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::RuntimeError {
                    message: "set-cdr!: expected exactly 2 arguments".to_string(),
                    location: crate::error::SourceSpan::unknown(),
                    stack_trace: Vec::new(),
                });
            }

            match &args[0] {
                Value::Pair(car, _) => {
                    // Create new pair with existing car and new cdr
                    let new_pair = Value::Pair(car.clone(), Box::new(args[1].clone()));
                    
                    // Note: Same limitation as set-car! - this doesn't provide true mutation semantics
                    Ok(new_pair)
                }
                _ => Err(LambdustError::RuntimeError {
                    message: format!("set-cdr!: expected pair, got {}", args[0]),
                    location: crate::error::SourceSpan::unknown(),
                    stack_trace: Vec::new(),
                }),
            }
        },
    })
}

// List predicates

fn predicate_null() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "null?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            Ok(Value::Boolean(args[0].is_nil()))
        },
    })
}

fn predicate_pair() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "pair?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            Ok(Value::Boolean(args[0].is_pair()))
        },
    })
}

fn predicate_list() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "list?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            Ok(Value::Boolean(args[0].is_list()))
        },
    })
}
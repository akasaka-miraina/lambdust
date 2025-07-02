//! Type predicates for Scheme

use crate::error::LambdustError;
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// Register all predicate functions
pub fn register_predicate_functions(builtins: &mut HashMap<String, Value>) {
    // Type predicates
    builtins.insert("number?".to_string(), predicate_number());
    builtins.insert("string?".to_string(), predicate_string());
    builtins.insert("symbol?".to_string(), predicate_symbol());
    builtins.insert("boolean?".to_string(), predicate_boolean());
    builtins.insert("procedure?".to_string(), predicate_procedure());
    builtins.insert("char?".to_string(), predicate_char());
    builtins.insert("vector?".to_string(), predicate_vector());

    // Equality predicates
    builtins.insert("eq?".to_string(), equality_eq());
    builtins.insert("eqv?".to_string(), equality_eqv());
    builtins.insert("equal?".to_string(), equality_equal());

    // Logical operations
    builtins.insert("not".to_string(), logical_not());

    // I/O predicates
    builtins.insert("eof-object?".to_string(), predicate_eof_object());
}

// Type predicates

fn predicate_number() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "number?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            Ok(Value::Boolean(args[0].is_number()))
        },
    })
}

fn predicate_string() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "string?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            Ok(Value::Boolean(args[0].is_string()))
        },
    })
}

fn predicate_symbol() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "symbol?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            Ok(Value::Boolean(args[0].is_symbol()))
        },
    })
}

fn predicate_boolean() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "boolean?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            Ok(Value::Boolean(matches!(args[0], Value::Boolean(_))))
        },
    })
}

fn predicate_procedure() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "procedure?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            Ok(Value::Boolean(args[0].is_procedure()))
        },
    })
}

fn predicate_char() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "char?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            Ok(Value::Boolean(matches!(args[0], Value::Character(_))))
        },
    })
}

fn predicate_vector() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "vector?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            Ok(Value::Boolean(matches!(args[0], Value::Vector(_))))
        },
    })
}

fn predicate_eof_object() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "eof-object?".to_string(),
        arity: Some(1),
        func: |_args| {
            // For now, we don't have EOF objects implemented
            // This is a placeholder implementation
            Ok(Value::Boolean(false))
        },
    })
}

// Equality predicates

fn equality_eq() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "eq?".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            Ok(Value::Boolean(args[0].scheme_eq(&args[1])))
        },
    })
}

fn equality_eqv() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "eqv?".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            Ok(Value::Boolean(args[0].eqv(&args[1])))
        },
    })
}

fn equality_equal() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "equal?".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::ArityError(2, args.len()));
            }
            Ok(Value::Boolean(args[0].equal(&args[1])))
        },
    })
}

// Logical operations

fn logical_not() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "not".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::ArityError(1, args.len()));
            }
            Ok(Value::Boolean(!args[0].is_truthy()))
        },
    })
}
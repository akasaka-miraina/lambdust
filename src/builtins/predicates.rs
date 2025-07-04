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

    // Number exactness predicates
    builtins.insert("exact?".to_string(), predicate_exact());
    builtins.insert("inexact?".to_string(), predicate_inexact());

    // Number type predicates
    builtins.insert("integer?".to_string(), predicate_integer());
    builtins.insert("rational?".to_string(), predicate_rational());
    builtins.insert("real?".to_string(), predicate_real());
    builtins.insert("complex?".to_string(), predicate_complex());
}

// Type predicates

fn predicate_number() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "number?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
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
                return Err(LambdustError::arity_error(1, args.len()));
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
                return Err(LambdustError::arity_error(1, args.len()));
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
                return Err(LambdustError::arity_error(1, args.len()));
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
                return Err(LambdustError::arity_error(1, args.len()));
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
                return Err(LambdustError::arity_error(1, args.len()));
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
                return Err(LambdustError::arity_error(1, args.len()));
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
                return Err(LambdustError::arity_error(2, args.len()));
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
                return Err(LambdustError::arity_error(2, args.len()));
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
                return Err(LambdustError::arity_error(2, args.len()));
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
                return Err(LambdustError::arity_error(1, args.len()));
            }
            Ok(Value::Boolean(!args[0].is_truthy()))
        },
    })
}

// Number exactness predicates

fn predicate_exact() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "exact?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            match &args[0] {
                Value::Number(n) => {
                    use crate::lexer::SchemeNumber;
                    let is_exact =
                        matches!(n, SchemeNumber::Integer(_) | SchemeNumber::Rational(_, _));
                    Ok(Value::Boolean(is_exact))
                }
                _ => Err(LambdustError::type_error(
                    "exact?: argument must be a number".to_string(),
                )),
            }
        },
    })
}

fn predicate_inexact() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "inexact?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            match &args[0] {
                Value::Number(n) => {
                    use crate::lexer::SchemeNumber;
                    let is_inexact =
                        matches!(n, SchemeNumber::Real(_) | SchemeNumber::Complex(_, _));
                    Ok(Value::Boolean(is_inexact))
                }
                _ => Err(LambdustError::type_error(
                    "inexact?: argument must be a number".to_string(),
                )),
            }
        },
    })
}

// Number type predicates

fn predicate_integer() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "integer?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            match &args[0] {
                Value::Number(n) => {
                    use crate::lexer::SchemeNumber;
                    let is_integer = matches!(n, SchemeNumber::Integer(_))
                        || matches!(n, SchemeNumber::Real(r) if r.fract() == 0.0);
                    Ok(Value::Boolean(is_integer))
                }
                _ => Ok(Value::Boolean(false)),
            }
        },
    })
}

fn predicate_rational() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "rational?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            match &args[0] {
                Value::Number(n) => {
                    use crate::lexer::SchemeNumber;
                    let is_rational = matches!(
                        n,
                        SchemeNumber::Integer(_)
                            | SchemeNumber::Rational(_, _)
                            | SchemeNumber::Real(_)
                    );
                    Ok(Value::Boolean(is_rational))
                }
                _ => Ok(Value::Boolean(false)),
            }
        },
    })
}

fn predicate_real() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "real?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            match &args[0] {
                Value::Number(n) => {
                    use crate::lexer::SchemeNumber;
                    let is_real = match n {
                        SchemeNumber::Integer(_) => true,
                        SchemeNumber::Rational(_, _) => true,
                        SchemeNumber::Real(_) => true,
                        SchemeNumber::Complex(_, imag) => *imag == 0.0,
                    };
                    Ok(Value::Boolean(is_real))
                }
                _ => Ok(Value::Boolean(false)),
            }
        },
    })
}

fn predicate_complex() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "complex?".to_string(),
        arity: Some(1),
        func: |args| {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }
            match &args[0] {
                Value::Number(_) => Ok(Value::Boolean(true)), // All numbers are complex
                _ => Ok(Value::Boolean(false)),
            }
        },
    })
}

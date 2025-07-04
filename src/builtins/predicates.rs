//! Type predicates for Scheme

use crate::builtins::utils::{
    check_arity, make_builtin_procedure,
    is_exact_number, is_inexact_number, is_integer, is_rational, is_real, is_complex, is_eof_object,
};
use crate::make_predicate;
use crate::value::Value;
use std::collections::HashMap;

/// Register all predicate functions
pub fn register_predicate_functions(builtins: &mut HashMap<String, Value>) {
    // Type predicates - using macro for consistency and less boilerplate
    builtins.insert("number?".to_string(), make_predicate!("number?", Value::is_number));
    builtins.insert("string?".to_string(), make_predicate!("string?", Value::is_string));
    builtins.insert("symbol?".to_string(), make_predicate!("symbol?", Value::is_symbol));
    builtins.insert("boolean?".to_string(), make_predicate!("boolean?", Value::is_boolean));
    builtins.insert("procedure?".to_string(), make_predicate!("procedure?", Value::is_procedure));
    builtins.insert("char?".to_string(), make_predicate!("char?", Value::is_character));
    builtins.insert("vector?".to_string(), make_predicate!("vector?", Value::is_vector));
    builtins.insert("pair?".to_string(), make_predicate!("pair?", Value::is_pair));
    builtins.insert("null?".to_string(), make_predicate!("null?", Value::is_null));
    builtins.insert("list?".to_string(), make_predicate!("list?", Value::is_list));

    // Equality predicates
    builtins.insert("eq?".to_string(), equality_eq());
    builtins.insert("eqv?".to_string(), equality_eqv());
    builtins.insert("equal?".to_string(), equality_equal());

    // Logical operations
    builtins.insert("not".to_string(), logical_not());

    // I/O predicates
    builtins.insert("eof-object?".to_string(), make_predicate!("eof-object?", is_eof_object));

    // Number exactness predicates
    builtins.insert("exact?".to_string(), make_predicate!("exact?", is_exact_number));
    builtins.insert("inexact?".to_string(), make_predicate!("inexact?", is_inexact_number));

    // Number type predicates
    builtins.insert("integer?".to_string(), make_predicate!("integer?", is_integer));
    builtins.insert("rational?".to_string(), make_predicate!("rational?", is_rational));
    builtins.insert("real?".to_string(), make_predicate!("real?", is_real));
    builtins.insert("complex?".to_string(), make_predicate!("complex?", is_complex));
}

// Complex equality and logical predicates that need custom implementation

fn equality_eq() -> Value {
    make_builtin_procedure("eq?", Some(2), |args| {
        check_arity(args, 2)?;
        Ok(Value::Boolean(args[0].scheme_eq(&args[1])))
    })
}

fn equality_eqv() -> Value {
    make_builtin_procedure("eqv?", Some(2), |args| {
        check_arity(args, 2)?;
        Ok(Value::Boolean(args[0].eqv(&args[1])))
    })
}

fn equality_equal() -> Value {
    make_builtin_procedure("equal?", Some(2), |args| {
        check_arity(args, 2)?;
        Ok(Value::Boolean(args[0].equal(&args[1])))
    })
}

fn logical_not() -> Value {
    make_builtin_procedure("not", Some(1), |args| {
        check_arity(args, 1)?;
        Ok(Value::Boolean(!args[0].is_truthy()))
    })
}


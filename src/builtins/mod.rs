//! Built-in functions module organization

pub mod arithmetic;
pub mod control_flow;
pub mod error_handling;
pub mod io;
pub mod list_ops;
pub mod misc;
pub mod predicates;
pub mod string_char;
pub mod vector;

use crate::value::Value;
use std::collections::HashMap;

/// Create a map of all built-in procedures
pub fn create_builtins() -> HashMap<String, Value> {
    let mut builtins = HashMap::new();

    // Register functions from each module
    arithmetic::register_arithmetic_functions(&mut builtins);
    control_flow::register_control_flow_functions(&mut builtins);
    list_ops::register_list_functions(&mut builtins);
    predicates::register_predicate_functions(&mut builtins);
    string_char::register_string_char_functions(&mut builtins);
    vector::register_vector_functions(&mut builtins);
    io::register_io_functions(&mut builtins);
    error_handling::register_error_functions(&mut builtins);
    misc::register_misc_functions(&mut builtins);

    builtins
}
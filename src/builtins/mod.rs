//! Built-in functions module organization

pub mod arithmetic;
pub mod control_flow;
pub mod error_handling;
pub mod higher_order;
pub mod io;
pub mod lazy;
pub mod list_ops;
pub mod misc;
pub mod predicates;
pub mod srfi;
// SRFI modules moved to src/srfi/
pub mod store;
pub mod string_char;
pub mod utils;
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
    // Note: higher_order functions are now handled as special forms in the evaluator
    higher_order::register_higher_order_functions(&mut builtins);
    predicates::register_predicate_functions(&mut builtins);
    string_char::register_string_char_functions(&mut builtins);
    vector::register_vector_functions(&mut builtins);
    io::register_io_functions(&mut builtins);
    error_handling::register_error_functions(&mut builtins);
    lazy::register_lazy_functions(&mut builtins);
    misc::register_misc_functions(&mut builtins);
    store::register_store_functions(&mut builtins);
    srfi::register_srfi_functions(&mut builtins);

    // SRFI functions now handled by module system (only in srfi-support builds)
    #[cfg(feature = "srfi-support")]
    {
        crate::srfi::srfi_1::register_srfi_1_functions(&mut builtins);
        crate::srfi::srfi_13::register_srfi_13_functions(&mut builtins);
        crate::srfi::srfi_69::register_srfi_69_functions(&mut builtins);
    }

    builtins
}

//! List processing functions for the Lambdust standard library.
//!
//! This module implements R7RS-compliant list operations including
//! list construction, manipulation, higher-order functions, and
//! list predicates. It is organized into several sub-modules for
//! better maintainability and code organization.

use crate::eval::value::ThreadSafeEnvironment;
use std::sync::Arc;

// Sub-modules containing different categories of list operations
pub mod common;
pub mod basic;
pub mod predicates;
pub mod accessors;
pub mod manipulation;
pub mod higher_order;
pub mod utilities;
pub mod srfi1;

// Re-export commonly used functions
pub use common::{
    is_proper_list,
    copy_list,
    values_equal,
    values_eq,
    values_eqv,
    is_circular_list,
    is_dotted_list,
    get_value_type_name,
    MAP_ARITY_ERROR,
    FOR_EACH_ARITY_ERROR,
};

/// Creates list operation bindings for the standard library.
/// 
/// This is the main entry point for setting up all list-related
/// functions in the environment. It delegates to specialized
/// binding functions in each sub-module.
pub fn create_list_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Basic list operations (cons, car, cdr, list, make-list)
    basic::bind_basic_list_operations(env);
    
    // List predicates (pair?, null?, list?)
    predicates::bind_list_predicates(env);
    
    // List accessors (list-ref, length, list-tail, car/cdr combinations)
    accessors::bind_list_accessors(env);
    
    // List manipulation (append, reverse, set-car!, set-cdr!, list-set!)
    manipulation::bind_list_manipulation(env);
    
    // Higher-order functions (map, filter, fold-left, fold-right, for-each)
    higher_order::bind_higher_order_functions(env);
    
    // List utilities (member, assoc, sort variants)
    utilities::bind_list_utilities(env);
    
    // SRFI-1 extensions (take, drop, take-while, etc.)
    srfi1::bind_srfi1_extensions(env);
}
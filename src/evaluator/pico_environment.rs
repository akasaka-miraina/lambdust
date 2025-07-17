//! R7RS-pico Initial Environment Setup
//!
//! This module provides functions to create the initial environment for R7RS-pico
//! with the minimal set of built-in procedures required by the specification.

#[cfg(feature = "pico")]
use crate::environment::Environment;
#[cfg(feature = "pico")]
use crate::value::{Procedure, Value};
#[cfg(feature = "pico")]
use std::rc::Rc;

/// Create the initial R7RS-pico environment with built-in procedures
///
/// This function creates an environment containing the minimal set of
/// built-in procedures required by the R7RS-pico specification:
///
/// - Arithmetic: +, -, *, =, <, >
/// - List operations: cons, car, cdr
/// - Predicates: null?, pair?, number?, boolean?, symbol?, procedure?
/// - Equivalence: eqv?
#[cfg(feature = "pico")]
pub fn create_pico_initial_environment() -> Rc<Environment> {
    let env = Environment::new();
    
    // Arithmetic operations
    add_builtin(&env, "+", "Addition of two numbers");
    add_builtin(&env, "-", "Subtraction or negation");
    add_builtin(&env, "*", "Multiplication of two numbers");
    add_builtin(&env, "=", "Numeric equality");
    add_builtin(&env, "<", "Numeric less than");
    add_builtin(&env, ">", "Numeric greater than");
    
    // List operations
    add_builtin(&env, "cons", "Construct a pair");
    add_builtin(&env, "car", "First element of a pair");
    add_builtin(&env, "cdr", "Second element of a pair");
    
    // Type predicates
    add_builtin(&env, "null?", "Test for empty list");
    add_builtin(&env, "pair?", "Test for pair");
    add_builtin(&env, "number?", "Test for number");
    add_builtin(&env, "boolean?", "Test for boolean");
    add_builtin(&env, "symbol?", "Test for symbol");
    add_builtin(&env, "procedure?", "Test for procedure");
    
    // Equivalence
    add_builtin(&env, "eqv?", "Test for equivalence");
    
    Rc::new(env)
}

/// Add a built-in procedure to the environment
#[cfg(feature = "pico")]
fn add_builtin(env: &Environment, name: &str, _description: &str) {
    let procedure = Value::Procedure(Procedure::Builtin {
        name: name.to_string(),
        arity: None, // Will be checked by PicoEvaluator
        func: pico_builtin_placeholder,
    });
    
    env.define(name.to_string(), procedure);
}

/// Placeholder function for pico built-ins
/// The actual implementation is handled by PicoEvaluator::apply_builtin
#[cfg(feature = "pico")]
pub fn pico_builtin_placeholder(_args: &[Value]) -> crate::error::Result<Value> {
    Ok(Value::Undefined)
}

/// Get the list of all built-in procedure names in R7RS-pico
#[cfg(feature = "pico")]
pub fn get_pico_builtin_names() -> Vec<&'static str> {
    vec![
        // Arithmetic
        "+", "-", "*", "=", "<", ">",
        // List operations
        "cons", "car", "cdr",
        // Predicates
        "null?", "pair?", "number?", "boolean?", "symbol?", "procedure?",
        // Equivalence
        "eqv?",
    ]
}

/// Check if a name is a built-in procedure in R7RS-pico
#[cfg(feature = "pico")]
pub fn is_pico_builtin(name: &str) -> bool {
    get_pico_builtin_names().contains(&name)
}

/// Get R7RS-pico language feature summary
#[cfg(feature = "pico")]
pub fn get_pico_features() -> PicoFeatures {
    PicoFeatures {
        semantic_model: "U -> E (Environment to Expressed value)".to_string(),
        supported_types: vec![
            "boolean".to_string(),
            "number (integer only)".to_string(),
            "symbol".to_string(),
            "pair".to_string(),
            "procedure".to_string(),
            "null".to_string(),
        ],
        special_forms: vec![
            "lambda".to_string(),
            "if".to_string(),
            "define".to_string(),
            "quote".to_string(),
        ],
        excluded_features: vec![
            "Side effects (set!)".to_string(),
            "Continuations (call/cc)".to_string(),
            "Complex numbers".to_string(),
            "Vectors".to_string(),
            "Strings (limited support)".to_string(),
            "Input/output procedures".to_string(),
            "Macros".to_string(),
        ],
        builtin_procedures: get_pico_builtin_names().len(),
        memory_model: "No side effects, implementations may use any storage model".to_string(),
        tail_recursion: "Required to be properly tail-recursive".to_string(),
    }
}

/// R7RS-pico language feature summary
#[cfg(feature = "pico")]
#[derive(Debug, Clone)]
pub struct PicoFeatures {
    /// Semantic evaluation model
    pub semantic_model: String,
    /// Supported data types
    pub supported_types: Vec<String>,
    /// Special forms supported
    pub special_forms: Vec<String>,
    /// Features excluded from full R7RS
    pub excluded_features: Vec<String>,
    /// Number of built-in procedures
    pub builtin_procedures: usize,
    /// Memory model description
    pub memory_model: String,
    /// Tail recursion requirement
    pub tail_recursion: String,
}

#[cfg(feature = "pico")]
mod tests {

}
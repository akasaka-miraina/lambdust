//! Standard library functions for monadic composition and effect handling.
//!
//! This module provides the standard library functions that support the effect
//! system, including monadic operations, effect handlers, and utility functions
//! for working with effects in Lambdust programs.

use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::effects::{Effect, MonadicValue};
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
// use std::collections::HashMap;
use std::sync::Arc;

/// Creates the standard library bindings for effects and monads.
pub fn create_effect_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Monadic operations
    bind_monadic_functions(env);
    
    // Effect handler functions
    bind_effect_handler_functions(env);
    
    // IO operations
    bind_io_functions(env);
    
    // State operations
    bind_state_functions(env);
    
    // Error operations
    bind_error_functions(env);
    
    // Utility functions
    bind_utility_functions(env);
}

/// Binds monadic composition functions.
fn bind_monadic_functions(env: &Arc<ThreadSafeEnvironment>) {
    // return - lift a value into a monad
    env.define("return".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "return".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_return),
        effects: vec![Effect::Pure],
    })));
    
    // >>= - monadic bind operation
    env.define(">>=".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: ">>=".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_bind),
        effects: vec![Effect::Pure],
    })));
    
    // >> - monadic sequence operation
    env.define(">>".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: ">>".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_sequence),
        effects: vec![Effect::Pure],
    })));
    
    // fmap - functorial map
    env.define("fmap".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "fmap".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_fmap),
        effects: vec![Effect::Pure],
    })));
    
    // join - monadic join operation
    env.define("join".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "join".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_join),
        effects: vec![Effect::Pure],
    })));
    
    // lift2 - lift a binary function into monadic context
    env.define("lift2".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "lift2".to_string(),
        arity_min: 3,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_lift2),
        effects: vec![Effect::Pure],
    })));
}

/// Binds effect handler functions.
fn bind_effect_handler_functions(env: &Arc<ThreadSafeEnvironment>) {
    // with-handler - run computation with an effect handler
    env.define("with-handler".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "with-handler".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_with_handler),
        effects: vec![Effect::Pure], // Handler itself is pure, but may execute effects
    })));
    
    // define-effect-handler - define a new effect handler
    env.define("define-effect-handler".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "define-effect-handler".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_define_effect_handler),
        effects: vec![Effect::State], // Modifies the handler registry
    })));
    
    // handle - handle a specific effect
    env.define("handle".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "handle".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_handle),
        effects: vec![Effect::Pure],
    })));
}

/// Binds IO-specific functions.
fn bind_io_functions(env: &Arc<ThreadSafeEnvironment>) {
    // io-return - create a pure IO computation
    env.define("io-return".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "io-return".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_io_return),
        effects: vec![Effect::IO],
    })));
    
    // io-bind - bind IO computations
    env.define("io-bind".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "io-bind".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_io_bind),
        effects: vec![Effect::IO],
    })));
    
    // run-io - execute an IO computation
    env.define("run-io".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "run-io".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_run_io),
        effects: vec![Effect::IO],
    })));
}

/// Binds state-specific functions.
fn bind_state_functions(env: &Arc<ThreadSafeEnvironment>) {
    // state-return - create a pure state computation
    env.define("state-return".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "state-return".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_state_return),
        effects: vec![Effect::State],
    })));
    
    // get-state - get the current state
    env.define("get-state".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-state".to_string(),
        arity_min: 0,
        arity_max: Some(0),
        implementation: PrimitiveImpl::RustFn(primitive_get_state),
        effects: vec![Effect::State],
    })));
    
    // put-state - set the state
    env.define("put-state".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "put-state".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_put_state),
        effects: vec![Effect::State],
    })));
    
    // modify-state - modify the state with a function
    env.define("modify-state".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "modify-state".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_modify_state),
        effects: vec![Effect::State],
    })));
    
    // run-state - execute a state computation
    env.define("run-state".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "run-state".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_run_state),
        effects: vec![Effect::State],
    })));
}

/// Binds error-specific functions.
fn bind_error_functions(env: &Arc<ThreadSafeEnvironment>) {
    // error-return - create a successful error computation
    env.define("error-return".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "error-return".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_error_return),
        effects: vec![Effect::Error],
    })));
    
    // throw-error - throw an error
    env.define("throw-error".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "throw-error".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_throw_error),
        effects: vec![Effect::Error],
    })));
    
    // catch-error - catch and handle errors
    env.define("catch-error".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "catch-error".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_catch_error),
        effects: vec![Effect::Error],
    })));
    
    // run-error - execute an error computation
    env.define("run-error".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "run-error".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_run_error),
        effects: vec![Effect::Error],
    })));
}

/// Binds utility functions.
fn bind_utility_functions(env: &Arc<ThreadSafeEnvironment>) {
    // effect-pure? - check if a computation is pure
    env.define("effect-pure?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "effect-pure?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_effect_pure_p),
        effects: vec![Effect::Pure],
    })));
    
    // get-effects - get the effects of a computation
    env.define("get-effects".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "get-effects".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_get_effects),
        effects: vec![Effect::Pure],
    })));
    
    // lift-effect - lift a computation into a specific effect
    env.define("lift-effect".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "lift-effect".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_lift_effect),
        effects: vec![Effect::Pure],
    })));
}

// ============= PRIMITIVE IMPLEMENTATIONS =============

/// return operation - lifts a value into a monad.
fn primitive_return(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("return expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // Create a pure monadic value
    let _monadic_val = MonadicValue::pure(args[0].clone());
    
    // For now, return the wrapped value as a string representation
    // In a full implementation, this would return a proper monadic value
    Ok(Value::string(format!("Monadic({})", args[0])))
}

/// >>= operation - monadic bind.
fn primitive_bind(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(">>= expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // In a full implementation, this would:
    // 1. Extract the monadic value from args[0]
    // 2. Apply the function in args[1] to the extracted value
    // 3. Return the resulting monadic computation
    
    // For now, return a placeholder
    Ok(Value::string(format!("Bind({}, {})", args[0], args[1])))
}

/// >> operation - monadic sequence.
fn primitive_sequence(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(">> expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // For now, return a placeholder
    Ok(Value::string(format!("Sequence({}, {})", args[0], args[1])))
}

/// fmap operation - functorial map.
fn primitive_fmap(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("fmap expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // For now, return a placeholder
    Ok(Value::string(format!("Fmap({}, {})", args[0], args[1])))
}

/// join operation - monadic join.
fn primitive_join(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("join expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // For now, return a placeholder
    Ok(Value::string(format!("Join({})", args[0])))
}

/// lift2 operation - lift binary function.
fn primitive_lift2(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("lift2 expects 3 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // For now, return a placeholder
    Ok(Value::string(format!("Lift2({}, {}, {})", args[0], args[1], args[2])))
}

/// with-handler operation.
fn primitive_with_handler(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("with-handler expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // For now, return a placeholder
    Ok(Value::string(format!("WithHandler({}, {})", args[0], args[1])))
}

/// define-effect-handler operation.
fn primitive_define_effect_handler(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("define-effect-handler expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // For now, return unspecified
    Ok(Value::Unspecified)
}

/// handle operation.
fn primitive_handle(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("handle expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    // For now, return a placeholder
    Ok(Value::string(format!("Handle({}, {})", args[0], args[1])))
}

// IO-specific primitives

fn primitive_io_return(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("io-return expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::string(format!("IO({})", args[0])))
}

fn primitive_io_bind(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("io-bind expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::string(format!("IOBind({}, {})", args[0], args[1])))
}

fn primitive_run_io(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("run-io expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // For now, just return the argument (simulating execution)
    Ok(args[0].clone())
}

// State-specific primitives

fn primitive_state_return(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("state-return expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::string(format!("State({})", args[0])))
}

fn primitive_get_state(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("get-state expects 0 arguments, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::string("GetState".to_string()))
}

fn primitive_put_state(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("put-state expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::string(format!("PutState({})", args[0])))
}

fn primitive_modify_state(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("modify-state expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::string(format!("ModifyState({})", args[0])))
}

fn primitive_run_state(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("run-state expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::string(format!("RunState({}, {})", args[0], args[1])))
}

// Error-specific primitives

fn primitive_error_return(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("error-return expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::string(format!("ErrorReturn({})", args[0])))
}

fn primitive_throw_error(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("throw-error expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // Actually throw an error
    Err(Box::new(DiagnosticError::runtime_error(
        format!("Thrown error: {}", args[0]),
        None,
    )))
}

fn primitive_catch_error(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("catch-error expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::string(format!("CatchError({}, {})", args[0], args[1])))
}

fn primitive_run_error(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("run-error expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // For now, just return the argument
    Ok(args[0].clone())
}

// Utility primitives

fn primitive_effect_pure_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("effect-pure? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // For now, assume everything is pure unless it's a special monadic value
    let is_pure = !args[0].as_string().map(|s| s.contains("IO") || s.contains("State") || s.contains("Error"))
        .unwrap_or(false);
    
    Ok(Value::boolean(is_pure))
}

fn primitive_get_effects(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("get-effects expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    // For now, return a list of effects as strings
    let effects = if let Some(s) = args[0].as_string() {
        if s.contains("IO") {
            vec![Value::string("IO".to_string())]
        } else if s.contains("State") {
            vec![Value::string("State".to_string())]
        } else if s.contains("Error") {
            vec![Value::string("Error".to_string())]
        } else {
            vec![Value::string("Pure".to_string())]
        }
    } else {
        vec![Value::string("Pure".to_string())]
    };
    
    Ok(Value::list(effects))
}

fn primitive_lift_effect(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("lift-effect expects 2 arguments, got {}", args.len()),
            None,
        )));
    }
    
    let effect_name = args[0].as_string().unwrap_or("Unknown");
    let value = &args[1];
    
    Ok(Value::string(format!("{effect_name}({value})")))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_monadic_return() {
        let args = vec![Value::integer(42)];
        let result = primitive_return(&args).unwrap();
        assert!(result.as_string().unwrap().contains("42"));
    }
    
    #[test]
    fn test_effect_pure_check() {
        let pure_val = vec![Value::integer(42)];
        let result = primitive_effect_pure_p(&pure_val).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let io_val = vec![Value::string("IO(something)".to_string())];
        let result = primitive_effect_pure_p(&io_val).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_error_throwing() {
        let args = vec![Value::string("Test error".to_string())];
        let result = primitive_throw_error(&args);
        assert!(result.is_err());
    }
}
//! SRFI-121: Generators implementation
//!
//! This module implements SRFI-121, which provides generators for lazy sequences.
//! Generators are stateful procedures that can be invoked repeatedly to produce
//! a sequence of values. They support lazy evaluation and can represent infinite sequences.

use crate::diagnostics::Result as LambdustResult;
use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use std::sync::Arc;

/// Creates generator bindings for the standard library.
pub fn create_generator_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Core generator operations
    bind_core_operations(env);
    
    // Generator constructors
    bind_constructors(env);
    
    // Generator utilities  
    bind_utilities(env);
}

/// Binds core generator operations.
fn bind_core_operations(env: &Arc<ThreadSafeEnvironment>) {
    // %make-generator
    env.define("%make-generator".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "%make-generator".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_make_generator),
        effects: vec![Effect::Pure],
    })));
    
    // %generator-next
    env.define("%generator-next".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "%generator-next".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_generator_next),
        effects: vec![Effect::IO], // Generator may have side effects
    })));
    
    // %generator-exhausted?
    env.define("%generator-exhausted?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "%generator-exhausted?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_generator_exhausted_p),
        effects: vec![Effect::Pure],
    })));
    
    // %generator?
    env.define("%generator?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "%generator?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_generator_p),
        effects: vec![Effect::Pure],
    })));
}

/// Binds generator constructor procedures.
fn bind_constructors(env: &Arc<ThreadSafeEnvironment>) {
    // generator
    env.define("generator".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator".to_string(),
        arity_min: 0,
        arity_max: None, // Variadic
        implementation: PrimitiveImpl::RustFn(primitive_generator),
        effects: vec![Effect::Pure],
    })));
    
    // make-range-generator
    env.define("make-range-generator".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-range-generator".to_string(),
        arity_min: 1,
        arity_max: Some(3), // start, end?, step?
        implementation: PrimitiveImpl::RustFn(primitive_make_range_generator),
        effects: vec![Effect::Pure],
    })));
    
    // make-iota-generator
    env.define("make-iota-generator".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-iota-generator".to_string(),
        arity_min: 0,
        arity_max: Some(3), // count?, start?, step?
        implementation: PrimitiveImpl::RustFn(primitive_make_iota_generator),
        effects: vec![Effect::Pure],
    })));
    
    // list->generator
    env.define("list->generator".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list->generator".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_list_to_generator),
        effects: vec![Effect::Pure],
    })));
    
    // vector->generator
    env.define("vector->generator".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "vector->generator".to_string(),
        arity_min: 1,
        arity_max: Some(3), // vector, start?, end?
        implementation: PrimitiveImpl::RustFn(primitive_vector_to_generator),
        effects: vec![Effect::Pure],
    })));
    
    // string->generator
    env.define("string->generator".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "string->generator".to_string(),
        arity_min: 1,
        arity_max: Some(3), // string, start?, end?
        implementation: PrimitiveImpl::RustFn(primitive_string_to_generator),
        effects: vec![Effect::Pure],
    })));
}

/// Binds generator utility procedures.
fn bind_utilities(env: &Arc<ThreadSafeEnvironment>) {
    // generator->list
    env.define("generator->list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator->list".to_string(),
        arity_min: 1,
        arity_max: Some(2), // generator, length?
        implementation: PrimitiveImpl::RustFn(primitive_generator_to_list),
        effects: vec![Effect::IO], // May consume generator
    })));
    
    // generator-fold
    env.define("generator-fold".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator-fold".to_string(),
        arity_min: 3,
        arity_max: Some(3), // kons, knil, generator
        implementation: PrimitiveImpl::RustFn(primitive_generator_fold),
        effects: vec![Effect::IO], // May call user procedures and consume generator
    })));
    
    // EOF object for generators
    env.define("*eof-object*".to_string(), Value::symbol_from_str("*eof-object*"));
}

// ============= CORE OPERATIONS =============

/// %make-generator primitive - creates a generator from a procedure
fn primitive_make_generator(args: &[Value]) -> LambdustResult<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("%make-generator: expected 1 argument, got {}", args.len()),
            span: None,
        }));
    }
    
    let thunk = args[0].clone();
    if !thunk.is_procedure() {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "%make-generator: argument must be a procedure".to_string(),
            span: None,
        }));
    }
    
    // For now, we'll create a simple generator that returns the thunk
    // In a full implementation, this would need access to the evaluator
    // to actually call the thunk repeatedly
    let env = Arc::new(crate::eval::value::ThreadSafeEnvironment::default());
    Ok(Value::generator_from_procedure(thunk, env))
}

/// %generator-next primitive - gets the next value from a generator
fn primitive_generator_next(args: &[Value]) -> LambdustResult<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("%generator-next: expected 1 argument, got {}", args.len()),
            span: None,
        }));
    }
    
    match &args[0] {
        Value::Generator(generator) => {
            generator.next()
        }
        _ => Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "%generator-next: argument must be a generator".to_string(),
            span: None,
        }))
    }
}

/// %generator-exhausted? primitive - checks if a generator is exhausted
fn primitive_generator_exhausted_p(args: &[Value]) -> LambdustResult<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("%generator-exhausted?: expected 1 argument, got {}", args.len()),
            span: None,
        }));
    }
    
    match &args[0] {
        Value::Generator(generator) => {
            Ok(Value::boolean(generator.is_exhausted()))
        }
        _ => Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "%generator-exhausted?: argument must be a generator".to_string(),
            span: None,
        }))
    }
}

/// %generator? primitive - type predicate for generators
fn primitive_generator_p(args: &[Value]) -> LambdustResult<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("%generator?: expected 1 argument, got {}", args.len()),
            span: None,
        }));
    }
    
    Ok(Value::boolean(args[0].is_generator()))
}

// ============= CONSTRUCTORS =============

/// generator primitive - creates generator from explicit values
fn primitive_generator(args: &[Value]) -> LambdustResult<Value> {
    Ok(Value::generator_from_values(args.to_vec()))
}

/// make-range-generator primitive
fn primitive_make_range_generator(args: &[Value]) -> LambdustResult<Value> {
    match args.len() {
        1 => {
            // (make-range-generator start) - infinite range from start with step 1
            let start = args[0].as_number()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-range-generator: start must be a number".to_string(),
                    span: None,
                }))?;
            Ok(Value::generator_range(start, None, 1.0))
        }
        2 => {
            // (make-range-generator start end) - finite range with step 1
            let start = args[0].as_number()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-range-generator: start must be a number".to_string(),
                    span: None,
                }))?;
            let end = args[1].as_number()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-range-generator: end must be a number".to_string(),
                    span: None,
                }))?;
            Ok(Value::generator_range(start, Some(end), 1.0))
        }
        3 => {
            // (make-range-generator start end step) - finite range with custom step
            let start = args[0].as_number()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-range-generator: start must be a number".to_string(),
                    span: None,
                }))?;
            let end = args[1].as_number()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-range-generator: end must be a number".to_string(),
                    span: None,
                }))?;
            let step = args[2].as_number()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-range-generator: step must be a number".to_string(),
                    span: None,
                }))?;
            
            if step == 0.0 {
                return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-range-generator: step must not be zero".to_string(),
                    span: None,
                }));
            }
            
            Ok(Value::generator_range(start, Some(end), step))
        }
        _ => Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("make-range-generator: expected 1-3 arguments, got {}", args.len()),
            span: None,
        }))
    }
}

/// make-iota-generator primitive
fn primitive_make_iota_generator(args: &[Value]) -> LambdustResult<Value> {
    match args.len() {
        0 => {
            // (make-iota-generator) - infinite counting from 0 with step 1
            Ok(Value::generator_iota(None, 0, 1))
        }
        1 => {
            // (make-iota-generator count) - finite counting from 0 with step 1
            let count = args[0].as_integer()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-iota-generator: count must be an integer".to_string(),
                    span: None,
                }))?;
            
            if count < 0 {
                return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-iota-generator: count must be non-negative".to_string(),
                    span: None,
                }));
            }
            
            Ok(Value::generator_iota(Some(count as usize), 0, 1))
        }
        2 => {
            // (make-iota-generator count start) - finite counting from start with step 1
            let count = args[0].as_integer()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-iota-generator: count must be an integer".to_string(),
                    span: None,
                }))?;
            let start = args[1].as_integer()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-iota-generator: start must be an integer".to_string(),
                    span: None,
                }))?;
            
            if count < 0 {
                return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-iota-generator: count must be non-negative".to_string(),
                    span: None,
                }));
            }
            
            Ok(Value::generator_iota(Some(count as usize), start, 1))
        }
        3 => {
            // (make-iota-generator count start step) - finite counting with custom step
            let count = args[0].as_integer()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-iota-generator: count must be an integer".to_string(),
                    span: None,
                }))?;
            let start = args[1].as_integer()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-iota-generator: start must be an integer".to_string(),
                    span: None,
                }))?;
            let step = args[2].as_integer()
                .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-iota-generator: step must be an integer".to_string(),
                    span: None,
                }))?;
            
            if count < 0 {
                return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-iota-generator: count must be non-negative".to_string(),
                    span: None,
                }));
            }
            
            if step == 0 {
                return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                    message: "make-iota-generator: step must not be zero".to_string(),
                    span: None,
                }));
            }
            
            Ok(Value::generator_iota(Some(count as usize), start, step))
        }
        _ => Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("make-iota-generator: expected 0-3 arguments, got {}", args.len()),
            span: None,
        }))
    }
}

/// list->generator primitive
fn primitive_list_to_generator(args: &[Value]) -> LambdustResult<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("list->generator: expected 1 argument, got {}", args.len()),
            span: None,
        }));
    }
    
    Ok(Value::generator_from_list(args[0].clone()))
}

/// vector->generator primitive
fn primitive_vector_to_generator(args: &[Value]) -> LambdustResult<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("vector->generator: expected 1-3 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let vector = match &args[0] {
        Value::Vector(vec) => vec.clone(),
        _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "vector->generator: first argument must be a vector".to_string(),
            span: None,
        }))
    };
    
    // For now, we'll ignore start/end parameters and just convert the whole vector
    // In a full implementation, we would respect start/end bounds
    Ok(Value::generator_from_vector(vector))
}

/// string->generator primitive
fn primitive_string_to_generator(args: &[Value]) -> LambdustResult<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("string->generator: expected 1-3 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let string = args[0].as_string_owned()
        .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
            message: "string->generator: first argument must be a string".to_string(),
            span: None,
        }))?;
    
    // For now, we'll ignore start/end parameters and just convert the whole string
    // In a full implementation, we would respect start/end bounds
    Ok(Value::generator_from_string(string))
}

// ============= UTILITIES =============

/// generator->list primitive
fn primitive_generator_to_list(args: &[Value]) -> LambdustResult<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("generator->list: expected 1-2 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let generator = match &args[0] {
        Value::Generator(gen_ref) => gen_ref,
        _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator->list: first argument must be a generator".to_string(),
            span: None,
        }))
    };
    
    let max_length = if args.len() == 2 {
        Some(args[1].as_integer()
            .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                message: "generator->list: second argument must be an integer".to_string(),
                span: None,
            }))?
            .max(0) as usize)
    } else {
        None
    };
    
    let mut values = Vec::new();
    let mut count = 0;
    
    while max_length.is_none_or(|max| count < max) {
        match generator.next() {
            Ok(value) => {
                if value == *generator.eof_object() {
                    break;
                }
                values.push(value);
                count += 1;
            }
            Err(e) => return Err(e),
        }
    }
    
    Ok(Value::list(values))
}

/// generator-fold primitive
fn primitive_generator_fold(_args: &[Value]) -> LambdustResult<Value> {
    // This is a placeholder implementation
    // A full implementation would need access to the evaluator
    // to call the kons procedure repeatedly
    Err(Box::new(crate::diagnostics::Error::RuntimeError {
        message: "generator-fold: not yet implemented (requires evaluator integration)".to_string(),
        span: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generator_predicate() {
        let generator = Value::generator_from_values(vec![Value::integer(1), Value::integer(2)]);
        let result = primitive_generator_p(&[generator]).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let not_gen = Value::integer(42);
        let result = primitive_generator_p(&[not_gen]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
    
    #[test]
    fn test_make_range_generator() {
        let args = vec![Value::number(0.0), Value::number(3.0)];
        let generator = primitive_make_range_generator(&args).unwrap();
        
        assert!(generator.is_generator());
    }
    
    #[test]
    fn test_make_iota_generator() {
        let args = vec![Value::integer(5), Value::integer(10), Value::integer(2)];
        let generator = primitive_make_iota_generator(&args).unwrap();
        
        assert!(generator.is_generator());
    }
    
    #[test]
    fn test_generator_constructor() {
        let args = vec![Value::integer(1), Value::string("hello"), Value::boolean(true)];
        let generator = primitive_generator(&args).unwrap();
        
        assert!(generator.is_generator());
    }
}
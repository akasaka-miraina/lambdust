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
    
    // generator-unfold
    env.define("generator-unfold".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator-unfold".to_string(),
        arity_min: 4,
        arity_max: Some(4), // stop-pred, mapper, successor, seed
        implementation: PrimitiveImpl::RustFn(primitive_generator_unfold),
        effects: vec![Effect::Pure],
    })));
    
    // generator-tabulate
    env.define("generator-tabulate".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator-tabulate".to_string(),
        arity_min: 1,
        arity_max: Some(2), // func, count?
        implementation: PrimitiveImpl::RustFn(primitive_generator_tabulate),
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
    
    // generator->vector
    env.define("generator->vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator->vector".to_string(),
        arity_min: 1,
        arity_max: Some(2), // generator, length?
        implementation: PrimitiveImpl::RustFn(primitive_generator_to_vector),
        effects: vec![Effect::IO], // May consume generator
    })));
    
    // generator->string
    env.define("generator->string".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator->string".to_string(),
        arity_min: 1,
        arity_max: Some(2), // generator, length?
        implementation: PrimitiveImpl::RustFn(primitive_generator_to_string),
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
    
    // generator-map
    env.define("generator-map".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator-map".to_string(),
        arity_min: 2,
        arity_max: Some(2), // mapper, generator
        implementation: PrimitiveImpl::RustFn(primitive_generator_map),
        effects: vec![Effect::Pure],
    })));
    
    // generator-filter
    env.define("generator-filter".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator-filter".to_string(),
        arity_min: 2,
        arity_max: Some(2), // predicate, generator
        implementation: PrimitiveImpl::RustFn(primitive_generator_filter),
        effects: vec![Effect::Pure],
    })));
    
    // generator-take
    env.define("generator-take".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator-take".to_string(),
        arity_min: 2,
        arity_max: Some(2), // generator, count
        implementation: PrimitiveImpl::RustFn(primitive_generator_take),
        effects: vec![Effect::Pure],
    })));
    
    // generator-drop
    env.define("generator-drop".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator-drop".to_string(),
        arity_min: 2,
        arity_max: Some(2), // generator, count
        implementation: PrimitiveImpl::RustFn(primitive_generator_drop),
        effects: vec![Effect::Pure],
    })));
    
    // generator-append
    env.define("generator-append".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator-append".to_string(),
        arity_min: 2,
        arity_max: Some(2), // first, second
        implementation: PrimitiveImpl::RustFn(primitive_generator_append),
        effects: vec![Effect::Pure],
    })));
    
    // generator-concatenate
    env.define("generator-concatenate".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator-concatenate".to_string(),
        arity_min: 0,
        arity_max: None, // Variadic
        implementation: PrimitiveImpl::RustFn(primitive_generator_concatenate),
        effects: vec![Effect::Pure],
    })));
    
    // generator-zip
    env.define("generator-zip".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "generator-zip".to_string(),
        arity_min: 0,
        arity_max: None, // Variadic
        implementation: PrimitiveImpl::RustFn(primitive_generator_zip),
        effects: vec![Effect::Pure],
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
    
    // Get vector length
    let vector_len = {
        let vec_guard = vector.read().map_err(|_| {
            Box::new(crate::diagnostics::Error::RuntimeError {
                message: "vector->generator: failed to read vector".to_string(),
                span: None,
            })
        })?;
        vec_guard.len()
    };
    
    let start = if args.len() > 1 {
        let start_val = args[1].as_integer()
            .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                message: "vector->generator: start index must be an integer".to_string(),
                span: None,
            }))?;
        
        if start_val < 0 || start_val as usize > vector_len {
            return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                message: "vector->generator: start index out of bounds".to_string(),
                span: None,
            }));
        }
        
        start_val as usize
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        let end_val = args[2].as_integer()
            .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                message: "vector->generator: end index must be an integer".to_string(),
                span: None,
            }))?;
        
        if end_val < 0 || end_val as usize > vector_len {
            return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                message: "vector->generator: end index out of bounds".to_string(),
                span: None,
            }));
        }
        
        if (end_val as usize) < start {
            return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                message: "vector->generator: end index must be >= start index".to_string(),
                span: None,
            }));
        }
        
        end_val as usize
    } else {
        vector_len
    };
    
    // Extract the subvector
    let subvector = {
        let vec_guard = vector.read().map_err(|_| {
            Box::new(crate::diagnostics::Error::RuntimeError {
                message: "vector->generator: failed to read vector".to_string(),
                span: None,
            })
        })?;
        
        let slice = &vec_guard[start..end];
        Arc::new(std::sync::RwLock::new(slice.to_vec()))
    };
    
    Ok(Value::generator_from_vector(subvector))
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
    
    // Convert to character vector to handle Unicode properly
    let chars: Vec<char> = string.chars().collect();
    let string_len = chars.len();
    
    let start = if args.len() > 1 {
        let start_val = args[1].as_integer()
            .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                message: "string->generator: start index must be an integer".to_string(),
                span: None,
            }))?;
        
        if start_val < 0 || start_val as usize > string_len {
            return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                message: "string->generator: start index out of bounds".to_string(),
                span: None,
            }));
        }
        
        start_val as usize
    } else {
        0
    };
    
    let end = if args.len() > 2 {
        let end_val = args[2].as_integer()
            .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                message: "string->generator: end index must be an integer".to_string(),
                span: None,
            }))?;
        
        if end_val < 0 || end_val as usize > string_len {
            return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                message: "string->generator: end index out of bounds".to_string(),
                span: None,
            }));
        }
        
        if (end_val as usize) < start {
            return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                message: "string->generator: end index must be >= start index".to_string(),
                span: None,
            }));
        }
        
        end_val as usize
    } else {
        string_len
    };
    
    // Extract the substring
    let substring: String = chars[start..end].iter().collect();
    
    Ok(Value::generator_from_string(substring))
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

/// generator->vector primitive
fn primitive_generator_to_vector(args: &[Value]) -> LambdustResult<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("generator->vector: expected 1-2 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let generator = match &args[0] {
        Value::Generator(gen_ref) => gen_ref,
        _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator->vector: first argument must be a generator".to_string(),
            span: None,
        }))
    };
    
    let max_length = if args.len() == 2 {
        Some(args[1].as_integer()
            .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                message: "generator->vector: second argument must be an integer".to_string(),
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
    
    Ok(Value::vector(values))
}

/// generator->string primitive
fn primitive_generator_to_string(args: &[Value]) -> LambdustResult<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("generator->string: expected 1-2 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let generator = match &args[0] {
        Value::Generator(gen_ref) => gen_ref,
        _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator->string: first argument must be a generator".to_string(),
            span: None,
        }))
    };
    
    let max_length = if args.len() == 2 {
        Some(args[1].as_integer()
            .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                message: "generator->string: second argument must be an integer".to_string(),
                span: None,
            }))?
            .max(0) as usize)
    } else {
        None
    };
    
    let mut chars = Vec::new();
    let mut count = 0;
    
    while max_length.is_none_or(|max| count < max) {
        match generator.next() {
            Ok(value) => {
                if value == *generator.eof_object() {
                    break;
                }
                
                // Convert value to character
                match value {
                    Value::Literal(crate::ast::Literal::Character(ch)) => {
                        chars.push(ch);
                        count += 1;
                    }
                    _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                        message: "generator->string: generator must yield characters".to_string(),
                        span: None,
                    }))
                }
            }
            Err(e) => return Err(e),
        }
    }
    
    let string: String = chars.into_iter().collect();
    Ok(Value::string(string))
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

/// generator-unfold primitive
fn primitive_generator_unfold(args: &[Value]) -> LambdustResult<Value> {
    if args.len() != 4 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("generator-unfold: expected 4 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let stop_predicate = args[0].clone();
    let mapper = args[1].clone();
    let successor = args[2].clone();
    let seed = args[3].clone();
    
    // Verify all are procedures
    if !stop_predicate.is_procedure() {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-unfold: stop predicate must be a procedure".to_string(),
            span: None,
        }));
    }
    
    if !mapper.is_procedure() {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-unfold: mapper must be a procedure".to_string(),
            span: None,
        }));
    }
    
    if !successor.is_procedure() {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-unfold: successor must be a procedure".to_string(),
            span: None,
        }));
    }
    
    Ok(Value::generator_unfold(stop_predicate, mapper, successor, seed))
}

/// generator-tabulate primitive
fn primitive_generator_tabulate(args: &[Value]) -> LambdustResult<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("generator-tabulate: expected 1-2 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let func = args[0].clone();
    if !func.is_procedure() {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-tabulate: first argument must be a procedure".to_string(),
            span: None,
        }));
    }
    
    let max_count = if args.len() == 2 {
        let count = args[1].as_integer()
            .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
                message: "generator-tabulate: second argument must be an integer".to_string(),
                span: None,
            }))?;
        
        if count < 0 {
            return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                message: "generator-tabulate: count must be non-negative".to_string(),
                span: None,
            }));
        }
        
        Some(count as usize)
    } else {
        None
    };
    
    Ok(Value::generator_tabulate(func, max_count))
}

/// generator-map primitive
fn primitive_generator_map(args: &[Value]) -> LambdustResult<Value> {
    if args.len() != 2 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("generator-map: expected 2 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let mapper = args[0].clone();
    if !mapper.is_procedure() {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-map: first argument must be a procedure".to_string(),
            span: None,
        }));
    }
    
    let source = match &args[1] {
        Value::Generator(generator) => generator.clone(),
        _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-map: second argument must be a generator".to_string(),
            span: None,
        }))
    };
    
    Ok(Value::generator_map(source, mapper))
}

/// generator-filter primitive
fn primitive_generator_filter(args: &[Value]) -> LambdustResult<Value> {
    if args.len() != 2 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("generator-filter: expected 2 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let predicate = args[0].clone();
    if !predicate.is_procedure() {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-filter: first argument must be a procedure".to_string(),
            span: None,
        }));
    }
    
    let source = match &args[1] {
        Value::Generator(generator) => generator.clone(),
        _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-filter: second argument must be a generator".to_string(),
            span: None,
        }))
    };
    
    Ok(Value::generator_filter(source, predicate))
}

/// generator-take primitive
fn primitive_generator_take(args: &[Value]) -> LambdustResult<Value> {
    if args.len() != 2 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("generator-take: expected 2 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let source = match &args[0] {
        Value::Generator(generator) => generator.clone(),
        _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-take: first argument must be a generator".to_string(),
            span: None,
        }))
    };
    
    let count = args[1].as_integer()
        .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-take: second argument must be an integer".to_string(),
            span: None,
        }))?;
    
    if count < 0 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-take: count must be non-negative".to_string(),
            span: None,
        }));
    }
    
    Ok(Value::generator_take(source, count as usize))
}

/// generator-drop primitive
fn primitive_generator_drop(args: &[Value]) -> LambdustResult<Value> {
    if args.len() != 2 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("generator-drop: expected 2 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let source = match &args[0] {
        Value::Generator(generator) => generator.clone(),
        _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-drop: first argument must be a generator".to_string(),
            span: None,
        }))
    };
    
    let count = args[1].as_integer()
        .ok_or_else(|| Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-drop: second argument must be an integer".to_string(),
            span: None,
        }))?;
    
    if count < 0 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-drop: count must be non-negative".to_string(),
            span: None,
        }));
    }
    
    Ok(Value::generator_drop(source, count as usize))
}

/// generator-append primitive
fn primitive_generator_append(args: &[Value]) -> LambdustResult<Value> {
    if args.len() != 2 {
        return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: format!("generator-append: expected 2 arguments, got {}", args.len()),
            span: None,
        }));
    }
    
    let first = match &args[0] {
        Value::Generator(generator) => generator.clone(),
        _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-append: first argument must be a generator".to_string(),
            span: None,
        }))
    };
    
    let second = match &args[1] {
        Value::Generator(generator) => generator.clone(),
        _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
            message: "generator-append: second argument must be a generator".to_string(),
            span: None,
        }))
    };
    
    Ok(Value::generator_append(first, second))
}

/// generator-concatenate primitive
fn primitive_generator_concatenate(args: &[Value]) -> LambdustResult<Value> {
    let mut generators = Vec::new();
    
    for arg in args {
        match arg {
            Value::Generator(generator) => generators.push(generator.clone()),
            _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                message: "generator-concatenate: all arguments must be generators".to_string(),
                span: None,
            }))
        }
    }
    
    Ok(Value::generator_concatenate(generators))
}

/// generator-zip primitive
fn primitive_generator_zip(args: &[Value]) -> LambdustResult<Value> {
    let mut sources = Vec::new();
    
    for arg in args {
        match arg {
            Value::Generator(generator) => sources.push(generator.clone()),
            _ => return Err(Box::new(crate::diagnostics::Error::RuntimeError {
                message: "generator-zip: all arguments must be generators".to_string(),
                span: None,
            }))
        }
    }
    
    Ok(Value::generator_zip(sources))
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
    
    #[test]
    fn test_generator_take() {
        // Create a simple range generator
        let args = vec![Value::number(0.0), Value::number(10.0)];
        let range_gen = primitive_make_range_generator(&args).unwrap();
        
        // Take first 3 values
        let take_args = vec![range_gen, Value::integer(3)];
        let take_gen = primitive_generator_take(&take_args).unwrap();
        
        assert!(take_gen.is_generator());
    }
    
    #[test]
    fn test_generator_append() {
        let first_args = vec![Value::integer(1), Value::integer(2)];
        let first_gen = primitive_generator(&first_args).unwrap();
        
        let second_args = vec![Value::integer(3), Value::integer(4)];
        let second_gen = primitive_generator(&second_args).unwrap();
        
        let append_args = vec![first_gen, second_gen];
        let appended = primitive_generator_append(&append_args).unwrap();
        
        assert!(appended.is_generator());
    }
    
    #[test]
    fn test_generator_to_vector() {
        let args = vec![Value::integer(10), Value::integer(20), Value::integer(30)];
        let generator = primitive_generator(&args).unwrap();
        
        let to_vec_args = vec![generator];
        let result = primitive_generator_to_vector(&to_vec_args).unwrap();
        
        assert!(result.is_vector());
    }
    
    #[test]
    fn test_string_to_generator_with_bounds() {
        let args = vec![Value::string("hello"), Value::integer(1), Value::integer(4)];
        let generator = primitive_string_to_generator(&args).unwrap();
        
        assert!(generator.is_generator());
    }
}
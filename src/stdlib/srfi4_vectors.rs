//! SRFI-4: Homogeneous numeric vector datatypes
//!
//! This module implements the SRFI-4 specification for homogeneous numeric vectors.
//! SRFI-4 introduces eight distinct vector types that store homogeneous numeric data
//! efficiently, providing both memory efficiency and improved performance for numeric
//! computations.

use crate::diagnostics::{Error, Result};
use crate::eval::value::{HomogeneousVectorValue, PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::effects::Effect;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Initialize all SRFI-4 procedures in the given environment.
pub fn init_srfi4_vectors(env: &Arc<ThreadSafeEnvironment>) {
    // Initialize all SRFI-4 homogeneous vector types
    
    // Initialize procedures for each vector type
    init_u8vector_procedures(env);
    
    init_s8vector_procedures(env);
    
    init_u16vector_procedures(env);
    init_s16vector_procedures(env);
    init_u32vector_procedures(env);
    init_s32vector_procedures(env);
    init_f32vector_procedures(env);
    init_f64vector_procedures(env);
    
    println!("DEBUG: SRFI-4 initialization completed");
}

/// Initialize u8vector procedures.
fn init_u8vector_procedures(env: &Arc<ThreadSafeEnvironment>) {
    // make-u8vector
    env.define(
        "make-u8vector".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "make-u8vector".to_string(),
            arity_min: 1,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(make_u8vector),
            effects: vec![Effect::Pure],
        })),
    );

    // u8vector
    env.define(
        "u8vector".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "u8vector".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(u8vector),
            effects: vec![Effect::Pure],
        })),
    );

    // u8vector?
    env.define(
        "u8vector?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "u8vector?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(u8vector_predicate),
            effects: vec![Effect::Pure],
        })),
    );

    // u8vector-length
    env.define(
        "u8vector-length".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "u8vector-length".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(u8vector_length),
            effects: vec![Effect::Pure],
        })),
    );

    // u8vector-ref
    env.define(
        "u8vector-ref".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "u8vector-ref".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(u8vector_ref),
            effects: vec![Effect::Pure],
        })),
    );

    // u8vector-set!
    env.define(
        "u8vector-set!".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "u8vector-set!".to_string(),
            arity_min: 3,
            arity_max: Some(3),
            implementation: PrimitiveImpl::RustFn(u8vector_set),
            effects: vec![Effect::Mutation],
        })),
    );

    // u8vector->list
    env.define(
        "u8vector->list".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "u8vector->list".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(u8vector_to_list),
            effects: vec![Effect::Pure],
        })),
    );

    // list->u8vector
    env.define(
        "list->u8vector".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "list->u8vector".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(list_to_u8vector),
            effects: vec![Effect::Pure],
        })),
    );
}

//================================================
// U8VECTOR IMPLEMENTATION (Unsigned 8-bit integers)
//================================================

/// Create a new u8vector with specified length and optional fill value.
fn make_u8vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("make-u8vector: expected 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let length = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "make-u8vector: length must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "make-u8vector: first argument must be an integer".to_string(),
            None,
        ))),
    };

    let fill_value = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(0..=255).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        "make-u8vector: fill value must be in range 0-255".to_string(),
                        None,
                    )));
                }
                *n as u8
            }
            _ => return Err(Box::new(Error::runtime_error(
                "make-u8vector: fill value must be an integer".to_string(),
                None,
            ))),
        }
    } else {
        0u8
    };

    let vector = vec![fill_value; length];
    let hv = HomogeneousVectorValue::U8Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

/// Create a u8vector from given arguments.
fn u8vector(args: &[Value]) -> Result<Value> {
    let mut vector = Vec::new();
    
    for arg in args {
        match arg {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(0..=255).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        format!("u8vector: value {} out of range 0-255", n),
                        None,
                    )));
                }
                vector.push(*n as u8);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "u8vector: all arguments must be integers in range 0-255".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::U8Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

/// Test if value is a u8vector.
fn u8vector_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("u8vector?: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let is_u8vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            matches!(hv.as_ref(), HomogeneousVectorValue::U8Vector(_))
        }
        _ => false,
    };

    Ok(Value::boolean(is_u8vector))
}

/// Get the length of a u8vector.
fn u8vector_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("u8vector-length: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U8Vector(v) => {
                    Ok(Value::integer(v.borrow().len() as i64))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "u8vector-length: argument must be a u8vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "u8vector-length: argument must be a u8vector".to_string(),
            None,
        ))),
    }
}

/// Get an element from a u8vector.
fn u8vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("u8vector-ref: expected 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U8Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "u8vector-ref: first argument must be a u8vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u8vector-ref: first argument must be a u8vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "u8vector-ref: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u8vector-ref: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let borrowed = vector.borrow();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("u8vector-ref: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    Ok(Value::integer(borrowed[index] as i64))
}

/// Set an element in a u8vector.
fn u8vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error(
            format!("u8vector-set!: expected 3 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U8Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "u8vector-set!: first argument must be a u8vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u8vector-set!: first argument must be a u8vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "u8vector-set!: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u8vector-set!: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let value = match &args[2] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if !(0..=255).contains(n) {
                return Err(Box::new(Error::runtime_error(
                    format!("u8vector-set!: value {} out of range 0-255", n),
                    None,
                )));
            }
            *n as u8
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u8vector-set!: third argument must be an integer in range 0-255".to_string(),
            None,
        ))),
    };

    let mut borrowed = vector.borrow_mut();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("u8vector-set!: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    borrowed[index] = value;
    Ok(Value::Unspecified)
}

/// Convert u8vector to list.
fn u8vector_to_list(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("u8vector->list: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U8Vector(v) => {
                    let borrowed = v.borrow();
                    let values: Vec<Value> = borrowed
                        .iter()
                        .map(|&x| Value::integer(x as i64))
                        .collect();
                    Ok(Value::list(values))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "u8vector->list: argument must be a u8vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "u8vector->list: argument must be a u8vector".to_string(),
            None,
        ))),
    }
}

/// Convert list to u8vector.
fn list_to_u8vector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("list->u8vector: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let list = &args[0];
    let values = match list.as_list() {
        Some(list_values) => list_values,
        None => return Err(Box::new(Error::runtime_error(
            "list->u8vector: argument must be a list".to_string(),
            None,
        ))),
    };

    let mut vector = Vec::new();
    for value in values {
        match value {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if n < 0 || n > 255 {
                    return Err(Box::new(Error::runtime_error(
                        format!("list->u8vector: value {} out of range 0-255", n),
                        None,
                    )));
                }
                vector.push(n as u8);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "list->u8vector: all list elements must be integers in range 0-255".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::U8Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

//================================================
// VECTOR TYPE GENERATION MACRO
//================================================

macro_rules! define_vector_type {
    (
        $type_name:ident, 
        $rust_type:ty, 
        $min_val:literal, 
        $max_val:literal, 
        $variant:ident, 
        $name_str:literal
    ) => {
        paste::paste! {
            // Initialization function
            fn [<init_ $type_name _procedures>](env: &Arc<ThreadSafeEnvironment>) {
                // Constructor procedures
                env.define(
                    concat!("make-", $name_str).to_string(),
                    Value::Primitive(Arc::new(PrimitiveProcedure {
                        name: concat!("make-", $name_str).to_string(),
                        arity_min: 1,
                        arity_max: Some(2),
                        implementation: PrimitiveImpl::RustFn([<make_ $type_name>]),
                        effects: vec![Effect::Pure],
                    })),
                );

                env.define(
                    $name_str.to_string(),
                    Value::Primitive(Arc::new(PrimitiveProcedure {
                        name: $name_str.to_string(),
                        arity_min: 0,
                        arity_max: None,
                        implementation: PrimitiveImpl::RustFn($type_name),
                        effects: vec![Effect::Pure],
                    })),
                );

                // Predicate
                env.define(
                    concat!($name_str, "?").to_string(),
                    Value::Primitive(Arc::new(PrimitiveProcedure {
                        name: concat!($name_str, "?").to_string(),
                        arity_min: 1,
                        arity_max: Some(1),
                        implementation: PrimitiveImpl::RustFn([<$type_name _predicate>]),
                        effects: vec![Effect::Pure],
                    })),
                );

                // Accessors
                env.define(
                    concat!($name_str, "-length").to_string(),
                    Value::Primitive(Arc::new(PrimitiveProcedure {
                        name: concat!($name_str, "-length").to_string(),
                        arity_min: 1,
                        arity_max: Some(1),
                        implementation: PrimitiveImpl::RustFn([<$type_name _length>]),
                        effects: vec![Effect::Pure],
                    })),
                );

                env.define(
                    concat!($name_str, "-ref").to_string(),
                    Value::Primitive(Arc::new(PrimitiveProcedure {
                        name: concat!($name_str, "-ref").to_string(),
                        arity_min: 2,
                        arity_max: Some(2),
                        implementation: PrimitiveImpl::RustFn([<$type_name _ref>]),
                        effects: vec![Effect::Pure],
                    })),
                );

                env.define(
                    concat!($name_str, "-set!").to_string(),
                    Value::Primitive(Arc::new(PrimitiveProcedure {
                        name: concat!($name_str, "-set!").to_string(),
                        arity_min: 3,
                        arity_max: Some(3),
                        implementation: PrimitiveImpl::RustFn([<$type_name _set>]),
                        effects: vec![Effect::Mutation],
                    })),
                );

                // Conversions
                env.define(
                    concat!($name_str, "->list").to_string(),
                    Value::Primitive(Arc::new(PrimitiveProcedure {
                        name: concat!($name_str, "->list").to_string(),
                        arity_min: 1,
                        arity_max: Some(1),
                        implementation: PrimitiveImpl::RustFn([<$type_name _to_list>]),
                        effects: vec![Effect::Pure],
                    })),
                );

                env.define(
                    concat!("list->", $name_str).to_string(),
                    Value::Primitive(Arc::new(PrimitiveProcedure {
                        name: concat!("list->", $name_str).to_string(),
                        arity_min: 1,
                        arity_max: Some(1),
                        implementation: PrimitiveImpl::RustFn([<list_to_ $type_name>]),
                        effects: vec![Effect::Pure],
                    })),
                );
            }

            // Implementation functions
            fn [<make_ $type_name>](args: &[Value]) -> Result<Value> {
                if args.is_empty() || args.len() > 2 {
                    return Err(Box::new(Error::runtime_error(
                        format!(concat!("make-", $name_str, ": expected 1 or 2 arguments, got {}"), args.len()),
                        None,
                    )));
                }

                let length = match &args[0] {
                    Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                        if *n < 0 {
                            return Err(Box::new(Error::runtime_error(
                                concat!("make-", $name_str, ": length must be non-negative").to_string(),
                                None,
                            )));
                        }
                        *n as usize
                    }
                    _ => return Err(Box::new(Error::runtime_error(
                        concat!("make-", $name_str, ": first argument must be an integer").to_string(),
                        None,
                    ))),
                };

                let fill_value = if args.len() == 2 {
                    [<extract_ $rust_type>](&args[1], concat!("make-", $name_str), $min_val, $max_val)?
                } else {
                    0 as $rust_type
                };

                let vector = vec![fill_value; length];
                let hv = HomogeneousVectorValue::$variant(Rc::new(RefCell::new(vector)));
                Ok(Value::HomogeneousVector(Arc::new(hv)))
            }

            fn $type_name(args: &[Value]) -> Result<Value> {
                let mut vector = Vec::new();
                
                *for arg in args {
                    let value = [<extract_ $rust_type>](arg, $name_str, $min_val, $max_val)?;
                    vector.push(value);
                }

                let hv = HomogeneousVectorValue::$variant(Rc::new(RefCell::new(vector)));
                Ok(Value::HomogeneousVector(Arc::new(hv)))
            }

            fn [<$type_name _predicate>](args: &[Value]) -> Result<Value> {
                if args.len() != 1 {
                    return Err(Box::new(Error::runtime_error(
                        format!(concat!($name_str, "?: expected 1 argument, got {}"), args.len()),
                        None,
                    )));
                }

                let is_vector = match &args[0] {
                    Value::HomogeneousVector(hv) => {
                        matches!(hv.as_ref(), HomogeneousVectorValue::$variant(_))
                    }
                    _ => false,
                };

                Ok(Value::boolean(is_vector))
            }

            fn [<$type_name _length>](args: &[Value]) -> Result<Value> {
                if args.len() != 1 {
                    return Err(Box::new(Error::runtime_error(
                        format!(concat!($name_str, "-length: expected 1 argument, got {}"), args.len()),
                        None,
                    )));
                }

                match &args[0] {
                    Value::HomogeneousVector(hv) => {
                        match hv.as_ref() {
                            HomogeneousVectorValue::$variant(v) => {
                                Ok(Value::integer(v.borrow().len() as i64))
                            }
                            _ => Err(Box::new(Error::runtime_error(
                                concat!($name_str, "-length: argument must be a ", $name_str).to_string(),
                                None,
                            ))),
                        }
                    }
                    _ => Err(Box::new(Error::runtime_error(
                        concat!($name_str, "-length: argument must be a ", $name_str).to_string(),
                        None,
                    ))),
                }
            }

            fn [<$type_name _ref>](args: &[Value]) -> Result<Value> {
                if args.len() != 2 {
                    return Err(Box::new(Error::runtime_error(
                        format!(concat!($name_str, "-ref: expected 2 arguments, got {}"), args.len()),
                        None,
                    )));
                }

                let vector = [<extract_vector_ $rust_type>](&args[0], concat!($name_str, "-ref"), &HomogeneousVectorValue::$variant)?;
                let index = extract_index(&args[1], concat!($name_str, "-ref"))?;

                let borrowed = vector.borrow();
                if index >= borrowed.len() {
                    return Err(Box::new(Error::runtime_error(
                        format!(concat!($name_str, "-ref: index {} out of bounds for vector of length {}"), 
                               index, borrowed.len()),
                        None,
                    )));
                }

                let value = borrowed[index];
                [<return_value_for_ $rust_type>](value)
            }

            fn [<$type_name _set>](args: &[Value]) -> Result<Value> {
                if args.len() != 3 {
                    return Err(Box::new(Error::runtime_error(
                        format!(concat!($name_str, "-set!: expected 3 arguments, got {}"), args.len()),
                        None,
                    )));
                }

                let vector = [<extract_vector_ $rust_type>](&args[0], concat!($name_str, "-set!"), &HomogeneousVectorValue::$variant)?;
                let index = extract_index(&args[1], concat!($name_str, "-set!"))?;
                let value = [<extract_ $rust_type>](&args[2], concat!($name_str, "-set!"), $min_val, $max_val)?;

                let mut borrowed = vector.borrow_mut();
                if index >= borrowed.len() {
                    return Err(Box::new(Error::runtime_error(
                        format!(concat!($name_str, "-set!: index {} out of bounds for vector of length {}"), 
                               index, borrowed.len()),
                        None,
                    )));
                }

                borrowed[index] = value;
                Ok(Value::Unspecified)
            }

            fn [<$type_name _to_list>](args: &[Value]) -> Result<Value> {
                if args.len() != 1 {
                    return Err(Box::new(Error::runtime_error(
                        format!(concat!($name_str, "->list: expected 1 argument, got {}"), args.len()),
                        None,
                    )));
                }

                match &args[0] {
                    Value::HomogeneousVector(hv) => {
                        match hv.as_ref() {
                            HomogeneousVectorValue::$variant(v) => {
                                let borrowed = v.borrow();
                                let values: Vec<Value> = borrowed
                                    .iter()
                                    .map(|&x| [<return_value_for_ $rust_type>](x).unwrap())
                                    .collect();
                                Ok(Value::list(values))
                            }
                            _ => Err(Box::new(Error::runtime_error(
                                concat!($name_str, "->list: argument must be a ", $name_str).to_string(),
                                None,
                            ))),
                        }
                    }
                    _ => Err(Box::new(Error::runtime_error(
                        concat!($name_str, "->list: argument must be a ", $name_str).to_string(),
                        None,
                    ))),
                }
            }

            fn [<list_to_ $type_name>](args: &[Value]) -> Result<Value> {
                if args.len() != 1 {
                    return Err(Box::new(Error::runtime_error(
                        format!(concat!("list->", $name_str, ": expected 1 argument, got {}"), args.len()),
                        None,
                    )));
                }

                let list = &args[0];
                let values = match list.as_list() {
                    Some(list_values) => list_values,
                    None => return Err(Box::new(Error::runtime_error(
                        concat!("list->", $name_str, ": argument must be a list").to_string(),
                        None,
                    ))),
                };

                let mut vector = Vec::new();
                *for value in values {
                    let converted = [<extract_ $rust_type>](value, concat!("list->", $name_str), $min_val, $max_val)?;
                    vector.push(converted);
                }

                let hv = HomogeneousVectorValue::$variant(Rc::new(RefCell::new(vector)));
                Ok(Value::HomogeneousVector(Arc::new(hv)))
            }
        }
    };
}

//================================================
// HELPER FUNCTIONS FOR TYPE EXTRACTION
//================================================

// Helper functions for value extraction and conversion
fn extract_index(value: &Value, context: &str) -> Result<usize> {
    match value {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    format!("{}: index must be non-negative", context),
                    None,
                )));
            }
            Ok(*n as usize)
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: index must be an integer", context),
            None,
        ))),
    }
}

// Specific extractors for each type
fn extract_i8(value: &Value, context: &str, min_val: i8, max_val: i8) -> Result<i8> {
    match value {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < min_val as i64 || *n > max_val as i64 {
                return Err(Box::new(Error::runtime_error(
                    format!("{}: value {} out of range {} to {}", context, n, min_val, max_val),
                    None,
                )));
            }
            Ok(*n as i8)
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: value must be an integer in range {} to {}", context, min_val, max_val),
            None,
        ))),
    }
}

fn extract_u16(value: &Value, context: &str, min_val: u16, max_val: u16) -> Result<u16> {
    match value {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < min_val as i64 || *n > max_val as i64 {
                return Err(Box::new(Error::runtime_error(
                    format!("{}: value {} out of range {} to {}", context, n, min_val, max_val),
                    None,
                )));
            }
            Ok(*n as u16)
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: value must be an integer in range {} to {}", context, min_val, max_val),
            None,
        ))),
    }
}

fn extract_i16(value: &Value, context: &str, min_val: i16, max_val: i16) -> Result<i16> {
    match value {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < min_val as i64 || *n > max_val as i64 {
                return Err(Box::new(Error::runtime_error(
                    format!("{}: value {} out of range {} to {}", context, n, min_val, max_val),
                    None,
                )));
            }
            Ok(*n as i16)
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: value must be an integer in range {} to {}", context, min_val, max_val),
            None,
        ))),
    }
}

fn extract_u32(value: &Value, context: &str, min_val: u32, max_val: u32) -> Result<u32> {
    match value {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < min_val as i64 || *n > max_val as i64 {
                return Err(Box::new(Error::runtime_error(
                    format!("{}: value {} out of range {} to {}", context, n, min_val, max_val),
                    None,
                )));
            }
            Ok(*n as u32)
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: value must be an integer in range {} to {}", context, min_val, max_val),
            None,
        ))),
    }
}

fn extract_i32(value: &Value, context: &str, min_val: i32, max_val: i32) -> Result<i32> {
    match value {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < min_val as i64 || *n > max_val as i64 {
                return Err(Box::new(Error::runtime_error(
                    format!("{}: value {} out of range {} to {}", context, n, min_val, max_val),
                    None,
                )));
            }
            Ok(*n as i32)
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: value must be an integer in range {} to {}", context, min_val, max_val),
            None,
        ))),
    }
}

fn extract_f32(value: &Value, context: &str, _min_val: f32, _max_val: f32) -> Result<f32> {
    match value {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            Ok(*n as f32)
        }
        Value::Literal(crate::ast::Literal::InexactReal(f)) => {
            Ok(*f as f32)
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: value must be a number", context),
            None,
        ))),
    }
}

fn extract_f64(value: &Value, context: &str, _min_val: f64, _max_val: f64) -> Result<f64> {
    match value {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            Ok(*n as f64)
        }
        Value::Literal(crate::ast::Literal::InexactReal(f)) => {
            Ok(*f)
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: value must be a number", context),
            None,
        ))),
    }
}

// Vector extraction helpers
fn extract_vector_u16(value: &Value, context: &str, _matcher: &HomogeneousVectorValue) -> Result<Rc<RefCell<Vec<u16>>>> {
    match value {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U16Vector(v) => Ok(v.clone()),
                _ => Err(Box::new(Error::runtime_error(
                    format!("{}: first argument must be a u16vector", context),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: first argument must be a u16vector", context),
            None,
        ))),
    }
}

fn extract_vector_i16(value: &Value, context: &str, _matcher: &HomogeneousVectorValue) -> Result<Rc<RefCell<Vec<i16>>>> {
    match value {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S16Vector(v) => Ok(v.clone()),
                _ => Err(Box::new(Error::runtime_error(
                    format!("{}: first argument must be an s16vector", context),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: first argument must be an s16vector", context),
            None,
        ))),
    }
}

fn extract_vector_u32(value: &Value, context: &str, _matcher: &HomogeneousVectorValue) -> Result<Rc<RefCell<Vec<u32>>>> {
    match value {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U32Vector(v) => Ok(v.clone()),
                _ => Err(Box::new(Error::runtime_error(
                    format!("{}: first argument must be a u32vector", context),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: first argument must be a u32vector", context),
            None,
        ))),
    }
}

fn extract_vector_i32(value: &Value, context: &str, _matcher: &HomogeneousVectorValue) -> Result<Rc<RefCell<Vec<i32>>>> {
    match value {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S32Vector(v) => Ok(v.clone()),
                _ => Err(Box::new(Error::runtime_error(
                    format!("{}: first argument must be an s32vector", context),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: first argument must be an s32vector", context),
            None,
        ))),
    }
}

fn extract_vector_f32(value: &Value, context: &str, _matcher: &HomogeneousVectorValue) -> Result<Rc<RefCell<Vec<f32>>>> {
    match value {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::F32Vector(v) => Ok(v.clone()),
                _ => Err(Box::new(Error::runtime_error(
                    format!("{}: first argument must be an f32vector", context),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: first argument must be an f32vector", context),
            None,
        ))),
    }
}

fn extract_vector_f64(value: &Value, context: &str, _matcher: &HomogeneousVectorValue) -> Result<Rc<RefCell<Vec<f64>>>> {
    match value {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::F64Vector(v) => Ok(v.clone()),
                _ => Err(Box::new(Error::runtime_error(
                    format!("{}: first argument must be an f64vector", context),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: first argument must be an f64vector", context),
            None,
        ))),
    }
}

fn extract_vector_i8<'a>(value: &'a Value, context: &str, _matcher: &dyn Fn(&HomogeneousVectorValue) -> bool) -> Result<Rc<RefCell<Vec<i8>>>> {
    match value {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S8Vector(v) => Ok(v.clone()),
                _ => Err(Box::new(Error::runtime_error(
                    format!("{}: first argument must be an s8vector", context),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{}: first argument must be an s8vector", context),
            None,
        ))),
    }
}

// Return value helpers
fn return_value_for_i8(value: i8) -> Result<Value> {
    Ok(Value::integer(value as i64))
}

fn return_value_for_u16(value: u16) -> Result<Value> {
    Ok(Value::integer(value as i64))
}

fn return_value_for_i16(value: i16) -> Result<Value> {
    Ok(Value::integer(value as i64))
}

fn return_value_for_u32(value: u32) -> Result<Value> {
    Ok(Value::integer(value as i64))
}

fn return_value_for_i32(value: i32) -> Result<Value> {
    Ok(Value::integer(value as i64))
}

fn return_value_for_f32(value: f32) -> Result<Value> {
    Ok(Value::number(value as f64))
}

fn return_value_for_f64(value: f64) -> Result<Value> {
    Ok(Value::number(value))
}

//================================================
// S8VECTOR IMPLEMENTATION (Signed 8-bit integers)  
//================================================

fn init_s8vector_procedures(env: &Arc<ThreadSafeEnvironment>) {
    // make-s8vector
    env.define(
        "make-s8vector".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "make-s8vector".to_string(),
            arity_min: 1,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(make_s8vector),
            effects: vec![Effect::Pure],
        })),
    );

    // s8vector
    env.define(
        "s8vector".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "s8vector".to_string(),
            arity_min: 0,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(s8vector),
            effects: vec![Effect::Pure],
        })),
    );

    // s8vector?
    env.define(
        "s8vector?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "s8vector?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(s8vector_predicate),
            effects: vec![Effect::Pure],
        })),
    );

    // s8vector-length
    env.define(
        "s8vector-length".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "s8vector-length".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(s8vector_length),
            effects: vec![Effect::Pure],
        })),
    );

    // s8vector-ref
    env.define(
        "s8vector-ref".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "s8vector-ref".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(s8vector_ref),
            effects: vec![Effect::Pure],
        })),
    );

    // s8vector-set!
    env.define(
        "s8vector-set!".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "s8vector-set!".to_string(),
            arity_min: 3,
            arity_max: Some(3),
            implementation: PrimitiveImpl::RustFn(s8vector_set),
            effects: vec![Effect::Mutation],
        })),
    );

    // s8vector->list
    env.define(
        "s8vector->list".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "s8vector->list".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(s8vector_to_list),
            effects: vec![Effect::Pure],
        })),
    );

    // list->s8vector
    env.define(
        "list->s8vector".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "list->s8vector".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(list_to_s8vector),
            effects: vec![Effect::Pure],
        })),
    );
}

/// Create a new s8vector with specified length and optional fill value.
fn make_s8vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("make-s8vector: expected 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let length = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "make-s8vector: length must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "make-s8vector: first argument must be an integer".to_string(),
            None,
        ))),
    };

    let fill_value = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(-128..=127).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        "make-s8vector: fill value must be in range -128 to 127".to_string(),
                        None,
                    )));
                }
                *n as i8
            }
            _ => return Err(Box::new(Error::runtime_error(
                "make-s8vector: fill value must be an integer".to_string(),
                None,
            ))),
        }
    } else {
        0i8
    };

    let vector = vec![fill_value; length];
    let hv = HomogeneousVectorValue::S8Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

/// Create a s8vector from given arguments.
fn s8vector(args: &[Value]) -> Result<Value> {
    let mut vector = Vec::new();
    
    for arg in args {
        match arg {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(-128..=127).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        format!("s8vector: value {} out of range -128 to 127", n),
                        None,
                    )));
                }
                vector.push(*n as i8);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "s8vector: all arguments must be integers in range -128 to 127".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::S8Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

/// Test if value is a s8vector.
fn s8vector_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("s8vector?: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let is_s8vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            matches!(hv.as_ref(), HomogeneousVectorValue::S8Vector(_))
        }
        _ => false,
    };

    Ok(Value::boolean(is_s8vector))
}

/// Get the length of a s8vector.
fn s8vector_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("s8vector-length: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S8Vector(v) => {
                    Ok(Value::integer(v.borrow().len() as i64))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "s8vector-length: argument must be a s8vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "s8vector-length: argument must be a s8vector".to_string(),
            None,
        ))),
    }
}

/// Get an element from a s8vector.
fn s8vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("s8vector-ref: expected 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S8Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "s8vector-ref: first argument must be a s8vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s8vector-ref: first argument must be a s8vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "s8vector-ref: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s8vector-ref: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let borrowed = vector.borrow();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("s8vector-ref: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    Ok(Value::integer(borrowed[index] as i64))
}

/// Set an element in a s8vector.
fn s8vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error(
            format!("s8vector-set!: expected 3 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S8Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "s8vector-set!: first argument must be a s8vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s8vector-set!: first argument must be a s8vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "s8vector-set!: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s8vector-set!: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let value = match &args[2] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if !(-128..=127).contains(n) {
                return Err(Box::new(Error::runtime_error(
                    format!("s8vector-set!: value {} out of range -128 to 127", n),
                    None,
                )));
            }
            *n as i8
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s8vector-set!: third argument must be an integer in range -128 to 127".to_string(),
            None,
        ))),
    };

    let mut borrowed = vector.borrow_mut();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("s8vector-set!: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    borrowed[index] = value;
    Ok(Value::Unspecified)
}

/// Convert s8vector to list.
fn s8vector_to_list(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("s8vector->list: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S8Vector(v) => {
                    let borrowed = v.borrow();
                    let values: Vec<Value> = borrowed
                        .iter()
                        .map(|&x| Value::integer(x as i64))
                        .collect();
                    Ok(Value::list(values))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "s8vector->list: argument must be a s8vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "s8vector->list: argument must be a s8vector".to_string(),
            None,
        ))),
    }
}

/// Convert list to s8vector.
fn list_to_s8vector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("list->s8vector: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let list = &args[0];
    let values = match list.as_list() {
        Some(list_values) => list_values,
        None => return Err(Box::new(Error::runtime_error(
            "list->s8vector: argument must be a list".to_string(),
            None,
        ))),
    };

    let mut vector = Vec::new();
    for value in values {
        match value {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if n < -128 || n > 127 {
                    return Err(Box::new(Error::runtime_error(
                        format!("list->s8vector: value {} out of range -128 to 127", n),
                        None,
                    )));
                }
                vector.push(n as i8);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "list->s8vector: all list elements must be integers in range -128 to 127".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::S8Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

// Manual implementation for u16vector (0 to 65535)
fn make_u16vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("make-u16vector: expected 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let length = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "make-u16vector: length must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "make-u16vector: first argument must be an integer".to_string(),
            None,
        ))),
    };

    let fill = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(0..=65535).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        format!("make-u16vector: fill value {} out of range 0 to 65535", n),
                        None,
                    )));
                }
                *n as u16
            }
            _ => return Err(Box::new(Error::runtime_error(
                "make-u16vector: fill value must be an integer in range 0 to 65535".to_string(),
                None,
            ))),
        }
    } else {
        0
    };

    let vector = vec![fill; length];
    let hv = HomogeneousVectorValue::U16Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn u16vector(args: &[Value]) -> Result<Value> {
    let mut vector = Vec::new();
    for arg in args {
        match arg {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(0..=65535).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        format!("u16vector: value {} out of range 0 to 65535", n),
                        None,
                    )));
                }
                vector.push(*n as u16);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "u16vector: all arguments must be integers in range 0 to 65535".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::U16Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn u16vector_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("u16vector?: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let is_u16vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            matches!(hv.as_ref(), HomogeneousVectorValue::U16Vector(_))
        }
        _ => false,
    };

    Ok(Value::boolean(is_u16vector))
}

fn u16vector_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("u16vector-length: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U16Vector(v) => {
                    Ok(Value::integer(v.borrow().len() as i64))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "u16vector-length: argument must be a u16vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "u16vector-length: argument must be a u16vector".to_string(),
            None,
        ))),
    }
}

fn u16vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("u16vector-ref: expected 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U16Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "u16vector-ref: first argument must be a u16vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u16vector-ref: first argument must be a u16vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "u16vector-ref: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u16vector-ref: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let borrowed = vector.borrow();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("u16vector-ref: index {} out of bounds for vector of length {}", index, borrowed.len()),
            None,
        )));
    }

    let value = borrowed[index];
    Ok(Value::integer(value as i64))
}

fn u16vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error(
            format!("u16vector-set!: expected 3 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U16Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "u16vector-set!: first argument must be a u16vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u16vector-set!: first argument must be a u16vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "u16vector-set!: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u16vector-set!: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let value = match &args[2] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if !(0..=65535).contains(n) {
                return Err(Box::new(Error::runtime_error(
                    format!("u16vector-set!: value {} out of range 0 to 65535", n),
                    None,
                )));
            }
            *n as u16
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u16vector-set!: third argument must be an integer in range 0 to 65535".to_string(),
            None,
        ))),
    };

    let mut borrowed = vector.borrow_mut();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("u16vector-set!: index {} out of bounds for vector of length {}", index, borrowed.len()),
            None,
        )));
    }

    borrowed[index] = value;
    Ok(Value::Unspecified)
}

fn u16vector_to_list(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("u16vector->list: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U16Vector(v) => {
                    let borrowed = v.borrow();
                    let values: Vec<Value> = borrowed
                        .iter()
                        .map(|&x| Value::integer(x as i64))
                        .collect();
                    Ok(Value::list(values))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "u16vector->list: argument must be a u16vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "u16vector->list: argument must be a u16vector".to_string(),
            None,
        ))),
    }
}

fn list_to_u16vector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("list->u16vector: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let list = &args[0];
    let values = match list.as_list() {
        Some(list_values) => list_values,
        None => return Err(Box::new(Error::runtime_error(
            "list->u16vector: argument must be a list".to_string(),
            None,
        ))),
    };

    let mut vector = Vec::new();
    for value in values {
        match value {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if n < 0 || n > 65535 {
                    return Err(Box::new(Error::runtime_error(
                        format!("list->u16vector: value {} out of range 0 to 65535", n),
                        None,
                    )));
                }
                vector.push(n as u16);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "list->u16vector: all list elements must be integers in range 0 to 65535".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::U16Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn init_u16vector_procedures(env: &Arc<ThreadSafeEnvironment>) {
    
    env.define("make-u16vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-u16vector".to_string(), arity_min: 1, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(make_u16vector), effects: vec![Effect::Pure],
    })));
    
    env.define("u16vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u16vector".to_string(), arity_min: 0, arity_max: None,
        implementation: PrimitiveImpl::RustFn(u16vector), effects: vec![Effect::Pure],
    })));
    
    env.define("u16vector?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u16vector?".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(u16vector_predicate), effects: vec![Effect::Pure],
    })));
    
    env.define("u16vector-length".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u16vector-length".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(u16vector_length), effects: vec![Effect::Pure],
    })));
    
    env.define("u16vector-ref".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u16vector-ref".to_string(), arity_min: 2, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(u16vector_ref), effects: vec![Effect::Pure],
    })));
    
    env.define("u16vector-set!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u16vector-set!".to_string(), arity_min: 3, arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(u16vector_set), effects: vec![Effect::Mutation],
    })));
    
    env.define("u16vector->list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u16vector->list".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(u16vector_to_list), effects: vec![Effect::Pure],
    })));
    
    env.define("list->u16vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list->u16vector".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(list_to_u16vector), effects: vec![Effect::Pure],
    })));
}

//================================================
// S16VECTOR IMPLEMENTATION (Signed 16-bit integers: -32768 to 32767)
//================================================

fn make_s16vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("make-s16vector: expected 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let length = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "make-s16vector: length must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "make-s16vector: first argument must be an integer".to_string(),
            None,
        ))),
    };

    let fill_value = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(-32768..=32767).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        "make-s16vector: fill value must be in range -32768 to 32767".to_string(),
                        None,
                    )));
                }
                *n as i16
            }
            _ => return Err(Box::new(Error::runtime_error(
                "make-s16vector: fill value must be an integer".to_string(),
                None,
            ))),
        }
    } else {
        0i16
    };

    let vector = vec![fill_value; length];
    let hv = HomogeneousVectorValue::S16Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn s16vector(args: &[Value]) -> Result<Value> {
    let mut vector = Vec::new();
    
    for arg in args {
        match arg {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(-32768..=32767).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        format!("s16vector: value {} out of range -32768 to 32767", n),
                        None,
                    )));
                }
                vector.push(*n as i16);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "s16vector: all arguments must be integers in range -32768 to 32767".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::S16Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn s16vector_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("s16vector?: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let is_s16vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            matches!(hv.as_ref(), HomogeneousVectorValue::S16Vector(_))
        }
        _ => false,
    };

    Ok(Value::boolean(is_s16vector))
}

fn s16vector_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("s16vector-length: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S16Vector(v) => {
                    Ok(Value::integer(v.borrow().len() as i64))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "s16vector-length: argument must be a s16vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "s16vector-length: argument must be a s16vector".to_string(),
            None,
        ))),
    }
}

fn s16vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("s16vector-ref: expected 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S16Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "s16vector-ref: first argument must be a s16vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s16vector-ref: first argument must be a s16vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "s16vector-ref: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s16vector-ref: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let borrowed = vector.borrow();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("s16vector-ref: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    Ok(Value::integer(borrowed[index] as i64))
}

fn s16vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error(
            format!("s16vector-set!: expected 3 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S16Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "s16vector-set!: first argument must be a s16vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s16vector-set!: first argument must be a s16vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "s16vector-set!: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s16vector-set!: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let value = match &args[2] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if !(-32768..=32767).contains(n) {
                return Err(Box::new(Error::runtime_error(
                    format!("s16vector-set!: value {} out of range -32768 to 32767", n),
                    None,
                )));
            }
            *n as i16
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s16vector-set!: third argument must be an integer in range -32768 to 32767".to_string(),
            None,
        ))),
    };

    let mut borrowed = vector.borrow_mut();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("s16vector-set!: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    borrowed[index] = value;
    Ok(Value::Unspecified)
}

fn s16vector_to_list(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("s16vector->list: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S16Vector(v) => {
                    let borrowed = v.borrow();
                    let values: Vec<Value> = borrowed
                        .iter()
                        .map(|&x| Value::integer(x as i64))
                        .collect();
                    Ok(Value::list(values))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "s16vector->list: argument must be a s16vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "s16vector->list: argument must be a s16vector".to_string(),
            None,
        ))),
    }
}

fn list_to_s16vector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("list->s16vector: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let list = &args[0];
    let values = match list.as_list() {
        Some(list_values) => list_values,
        None => return Err(Box::new(Error::runtime_error(
            "list->s16vector: argument must be a list".to_string(),
            None,
        ))),
    };

    let mut vector = Vec::new();
    for value in values {
        match value {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if n < -32768 || n > 32767 {
                    return Err(Box::new(Error::runtime_error(
                        format!("list->s16vector: value {} out of range -32768 to 32767", n),
                        None,
                    )));
                }
                vector.push(n as i16);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "list->s16vector: all list elements must be integers in range -32768 to 32767".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::S16Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn init_s16vector_procedures(env: &Arc<ThreadSafeEnvironment>) {
    
    env.define("make-s16vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-s16vector".to_string(), arity_min: 1, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(make_s16vector), effects: vec![Effect::Pure],
    })));
    
    env.define("s16vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s16vector".to_string(), arity_min: 0, arity_max: None,
        implementation: PrimitiveImpl::RustFn(s16vector), effects: vec![Effect::Pure],
    })));
    
    env.define("s16vector?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s16vector?".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(s16vector_predicate), effects: vec![Effect::Pure],
    })));
    
    env.define("s16vector-length".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s16vector-length".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(s16vector_length), effects: vec![Effect::Pure],
    })));
    
    env.define("s16vector-ref".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s16vector-ref".to_string(), arity_min: 2, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(s16vector_ref), effects: vec![Effect::Pure],
    })));
    
    env.define("s16vector-set!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s16vector-set!".to_string(), arity_min: 3, arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(s16vector_set), effects: vec![Effect::Mutation],
    })));
    
    env.define("s16vector->list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s16vector->list".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(s16vector_to_list), effects: vec![Effect::Pure],
    })));
    
    env.define("list->s16vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list->s16vector".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(list_to_s16vector), effects: vec![Effect::Pure],
    })));
}

//================================================
// U32VECTOR IMPLEMENTATION (Unsigned 32-bit integers: 0 to 4294967295)
//================================================

fn make_u32vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("make-u32vector: expected 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let length = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "make-u32vector: length must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "make-u32vector: first argument must be an integer".to_string(),
            None,
        ))),
    };

    let fill_value = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(0..=4294967295).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        "make-u32vector: fill value must be in range 0 to 4294967295".to_string(),
                        None,
                    )));
                }
                *n as u32
            }
            _ => return Err(Box::new(Error::runtime_error(
                "make-u32vector: fill value must be an integer".to_string(),
                None,
            ))),
        }
    } else {
        0u32
    };

    let vector = vec![fill_value; length];
    let hv = HomogeneousVectorValue::U32Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn u32vector(args: &[Value]) -> Result<Value> {
    let mut vector = Vec::new();
    
    for arg in args {
        match arg {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(0..=4294967295).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        format!("u32vector: value {} out of range 0 to 4294967295", n),
                        None,
                    )));
                }
                vector.push(*n as u32);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "u32vector: all arguments must be integers in range 0 to 4294967295".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::U32Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn u32vector_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("u32vector?: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let is_u32vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            matches!(hv.as_ref(), HomogeneousVectorValue::U32Vector(_))
        }
        _ => false,
    };

    Ok(Value::boolean(is_u32vector))
}

fn u32vector_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("u32vector-length: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U32Vector(v) => {
                    Ok(Value::integer(v.borrow().len() as i64))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "u32vector-length: argument must be a u32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "u32vector-length: argument must be a u32vector".to_string(),
            None,
        ))),
    }
}

fn u32vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("u32vector-ref: expected 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U32Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "u32vector-ref: first argument must be a u32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u32vector-ref: first argument must be a u32vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "u32vector-ref: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u32vector-ref: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let borrowed = vector.borrow();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("u32vector-ref: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    Ok(Value::integer(borrowed[index] as i64))
}

fn u32vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error(
            format!("u32vector-set!: expected 3 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U32Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "u32vector-set!: first argument must be a u32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u32vector-set!: first argument must be a u32vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "u32vector-set!: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u32vector-set!: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let value = match &args[2] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if !(0..=4294967295).contains(n) {
                return Err(Box::new(Error::runtime_error(
                    format!("u32vector-set!: value {} out of range 0 to 4294967295", n),
                    None,
                )));
            }
            *n as u32
        }
        _ => return Err(Box::new(Error::runtime_error(
            "u32vector-set!: third argument must be an integer in range 0 to 4294967295".to_string(),
            None,
        ))),
    };

    let mut borrowed = vector.borrow_mut();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("u32vector-set!: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    borrowed[index] = value;
    Ok(Value::Unspecified)
}

fn u32vector_to_list(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("u32vector->list: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::U32Vector(v) => {
                    let borrowed = v.borrow();
                    let values: Vec<Value> = borrowed
                        .iter()
                        .map(|&x| Value::integer(x as i64))
                        .collect();
                    Ok(Value::list(values))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "u32vector->list: argument must be a u32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "u32vector->list: argument must be a u32vector".to_string(),
            None,
        ))),
    }
}

fn list_to_u32vector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("list->u32vector: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let list = &args[0];
    let values = match list.as_list() {
        Some(list_values) => list_values,
        None => return Err(Box::new(Error::runtime_error(
            "list->u32vector: argument must be a list".to_string(),
            None,
        ))),
    };

    let mut vector = Vec::new();
    for value in values {
        match value {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if n < 0 || n > 4294967295 {
                    return Err(Box::new(Error::runtime_error(
                        format!("list->u32vector: value {} out of range 0 to 4294967295", n),
                        None,
                    )));
                }
                vector.push(n as u32);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "list->u32vector: all list elements must be integers in range 0 to 4294967295".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::U32Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn init_u32vector_procedures(env: &Arc<ThreadSafeEnvironment>) {
    
    env.define("make-u32vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-u32vector".to_string(), arity_min: 1, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(make_u32vector), effects: vec![Effect::Pure],
    })));
    
    env.define("u32vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u32vector".to_string(), arity_min: 0, arity_max: None,
        implementation: PrimitiveImpl::RustFn(u32vector), effects: vec![Effect::Pure],
    })));
    
    env.define("u32vector?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u32vector?".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(u32vector_predicate), effects: vec![Effect::Pure],
    })));
    
    env.define("u32vector-length".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u32vector-length".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(u32vector_length), effects: vec![Effect::Pure],
    })));
    
    env.define("u32vector-ref".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u32vector-ref".to_string(), arity_min: 2, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(u32vector_ref), effects: vec![Effect::Pure],
    })));
    
    env.define("u32vector-set!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u32vector-set!".to_string(), arity_min: 3, arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(u32vector_set), effects: vec![Effect::Mutation],
    })));
    
    env.define("u32vector->list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "u32vector->list".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(u32vector_to_list), effects: vec![Effect::Pure],
    })));
    
    env.define("list->u32vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list->u32vector".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(list_to_u32vector), effects: vec![Effect::Pure],
    })));
}

//================================================
// S32VECTOR IMPLEMENTATION (Signed 32-bit integers: -2147483648 to 2147483647)
//================================================

fn make_s32vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("make-s32vector: expected 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let length = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "make-s32vector: length must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "make-s32vector: first argument must be an integer".to_string(),
            None,
        ))),
    };

    let fill_value = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(-2147483648..=2147483647).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        "make-s32vector: fill value must be in range -2147483648 to 2147483647".to_string(),
                        None,
                    )));
                }
                *n as i32
            }
            _ => return Err(Box::new(Error::runtime_error(
                "make-s32vector: fill value must be an integer".to_string(),
                None,
            ))),
        }
    } else {
        0i32
    };

    let vector = vec![fill_value; length];
    let hv = HomogeneousVectorValue::S32Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn s32vector(args: &[Value]) -> Result<Value> {
    let mut vector = Vec::new();
    
    for arg in args {
        match arg {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if !(-2147483648..=2147483647).contains(n) {
                    return Err(Box::new(Error::runtime_error(
                        format!("s32vector: value {} out of range -2147483648 to 2147483647", n),
                        None,
                    )));
                }
                vector.push(*n as i32);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "s32vector: all arguments must be integers in range -2147483648 to 2147483647".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::S32Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn s32vector_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("s32vector?: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let is_s32vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            matches!(hv.as_ref(), HomogeneousVectorValue::S32Vector(_))
        }
        _ => false,
    };

    Ok(Value::boolean(is_s32vector))
}

fn s32vector_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("s32vector-length: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S32Vector(v) => {
                    Ok(Value::integer(v.borrow().len() as i64))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "s32vector-length: argument must be a s32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "s32vector-length: argument must be a s32vector".to_string(),
            None,
        ))),
    }
}

fn s32vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("s32vector-ref: expected 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S32Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "s32vector-ref: first argument must be a s32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s32vector-ref: first argument must be a s32vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "s32vector-ref: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s32vector-ref: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let borrowed = vector.borrow();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("s32vector-ref: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    Ok(Value::integer(borrowed[index] as i64))
}

fn s32vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error(
            format!("s32vector-set!: expected 3 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S32Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "s32vector-set!: first argument must be a s32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s32vector-set!: first argument must be a s32vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "s32vector-set!: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s32vector-set!: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let value = match &args[2] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if !(-2147483648..=2147483647).contains(n) {
                return Err(Box::new(Error::runtime_error(
                    format!("s32vector-set!: value {} out of range -2147483648 to 2147483647", n),
                    None,
                )));
            }
            *n as i32
        }
        _ => return Err(Box::new(Error::runtime_error(
            "s32vector-set!: third argument must be an integer in range -2147483648 to 2147483647".to_string(),
            None,
        ))),
    };

    let mut borrowed = vector.borrow_mut();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("s32vector-set!: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    borrowed[index] = value;
    Ok(Value::Unspecified)
}

fn s32vector_to_list(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("s32vector->list: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::S32Vector(v) => {
                    let borrowed = v.borrow();
                    let values: Vec<Value> = borrowed
                        .iter()
                        .map(|&x| Value::integer(x as i64))
                        .collect();
                    Ok(Value::list(values))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "s32vector->list: argument must be a s32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "s32vector->list: argument must be a s32vector".to_string(),
            None,
        ))),
    }
}

fn list_to_s32vector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("list->s32vector: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let list = &args[0];
    let values = match list.as_list() {
        Some(list_values) => list_values,
        None => return Err(Box::new(Error::runtime_error(
            "list->s32vector: argument must be a list".to_string(),
            None,
        ))),
    };

    let mut vector = Vec::new();
    for value in values {
        match value {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                if n < -2147483648 || n > 2147483647 {
                    return Err(Box::new(Error::runtime_error(
                        format!("list->s32vector: value {} out of range -2147483648 to 2147483647", n),
                        None,
                    )));
                }
                vector.push(n as i32);
            }
            _ => return Err(Box::new(Error::runtime_error(
                "list->s32vector: all list elements must be integers in range -2147483648 to 2147483647".to_string(),
                None,
            ))),
        }
    }

    let hv = HomogeneousVectorValue::S32Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn init_s32vector_procedures(env: &Arc<ThreadSafeEnvironment>) {
    
    env.define("make-s32vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-s32vector".to_string(), arity_min: 1, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(make_s32vector), effects: vec![Effect::Pure],
    })));
    
    env.define("s32vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s32vector".to_string(), arity_min: 0, arity_max: None,
        implementation: PrimitiveImpl::RustFn(s32vector), effects: vec![Effect::Pure],
    })));
    
    env.define("s32vector?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s32vector?".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(s32vector_predicate), effects: vec![Effect::Pure],
    })));
    
    env.define("s32vector-length".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s32vector-length".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(s32vector_length), effects: vec![Effect::Pure],
    })));
    
    env.define("s32vector-ref".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s32vector-ref".to_string(), arity_min: 2, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(s32vector_ref), effects: vec![Effect::Pure],
    })));
    
    env.define("s32vector-set!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s32vector-set!".to_string(), arity_min: 3, arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(s32vector_set), effects: vec![Effect::Mutation],
    })));
    
    env.define("s32vector->list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "s32vector->list".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(s32vector_to_list), effects: vec![Effect::Pure],
    })));
    
    env.define("list->s32vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list->s32vector".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(list_to_s32vector), effects: vec![Effect::Pure],
    })));
}

//================================================
// F32VECTOR IMPLEMENTATION (32-bit floating point)
//================================================

fn make_f32vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("make-f32vector: expected 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let length = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "make-f32vector: length must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "make-f32vector: first argument must be an integer".to_string(),
            None,
        ))),
    };

    let fill_value = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                *n as f32
            }
            Value::Literal(crate::ast::Literal::InexactReal(f)) => {
                *f as f32
            }
            _ => return Err(Box::new(Error::runtime_error(
                "make-f32vector: fill value must be a number".to_string(),
                None,
            ))),
        }
    } else {
        0.0f32
    };

    let vector = vec![fill_value; length];
    let hv = HomogeneousVectorValue::F32Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn f32vector(args: &[Value]) -> Result<Value> {
    let mut vector = Vec::new();
    
    for arg in args {
        let value = match arg {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                *n as f32
            }
            Value::Literal(crate::ast::Literal::InexactReal(f)) => {
                *f as f32
            }
            _ => return Err(Box::new(Error::runtime_error(
                "f32vector: all arguments must be numbers".to_string(),
                None,
            ))),
        };
        vector.push(value);
    }

    let hv = HomogeneousVectorValue::F32Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn f32vector_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("f32vector?: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let is_f32vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            matches!(hv.as_ref(), HomogeneousVectorValue::F32Vector(_))
        }
        _ => false,
    };

    Ok(Value::boolean(is_f32vector))
}

fn f32vector_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("f32vector-length: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::F32Vector(v) => {
                    Ok(Value::integer(v.borrow().len() as i64))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "f32vector-length: argument must be a f32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "f32vector-length: argument must be a f32vector".to_string(),
            None,
        ))),
    }
}

fn f32vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("f32vector-ref: expected 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::F32Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "f32vector-ref: first argument must be a f32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "f32vector-ref: first argument must be a f32vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "f32vector-ref: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "f32vector-ref: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let borrowed = vector.borrow();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("f32vector-ref: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    Ok(Value::number(borrowed[index] as f64))
}

fn f32vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error(
            format!("f32vector-set!: expected 3 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::F32Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "f32vector-set!: first argument must be a f32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "f32vector-set!: first argument must be a f32vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "f32vector-set!: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "f32vector-set!: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let value = match &args[2] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            *n as f32
        }
        Value::Literal(crate::ast::Literal::InexactReal(f)) => {
            *f as f32
        }
        _ => return Err(Box::new(Error::runtime_error(
            "f32vector-set!: third argument must be a number".to_string(),
            None,
        ))),
    };

    let mut borrowed = vector.borrow_mut();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("f32vector-set!: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    borrowed[index] = value;
    Ok(Value::Unspecified)
}

fn f32vector_to_list(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("f32vector->list: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::F32Vector(v) => {
                    let borrowed = v.borrow();
                    let values: Vec<Value> = borrowed
                        .iter()
                        .map(|&x| Value::number(x as f64))
                        .collect();
                    Ok(Value::list(values))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "f32vector->list: argument must be a f32vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "f32vector->list: argument must be a f32vector".to_string(),
            None,
        ))),
    }
}

fn list_to_f32vector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("list->f32vector: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let list = &args[0];
    let values = match list.as_list() {
        Some(list_values) => list_values,
        None => return Err(Box::new(Error::runtime_error(
            "list->f32vector: argument must be a list".to_string(),
            None,
        ))),
    };

    let mut vector = Vec::new();
    for value in values {
        let converted = match value {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                n as f32
            }
            Value::Literal(crate::ast::Literal::InexactReal(f)) => {
                f as f32
            }
            _ => return Err(Box::new(Error::runtime_error(
                "list->f32vector: all list elements must be numbers".to_string(),
                None,
            ))),
        };
        vector.push(converted);
    }

    let hv = HomogeneousVectorValue::F32Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn init_f32vector_procedures(env: &Arc<ThreadSafeEnvironment>) {
    
    env.define("make-f32vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-f32vector".to_string(), arity_min: 1, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(make_f32vector), effects: vec![Effect::Pure],
    })));
    
    env.define("f32vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f32vector".to_string(), arity_min: 0, arity_max: None,
        implementation: PrimitiveImpl::RustFn(f32vector), effects: vec![Effect::Pure],
    })));
    
    env.define("f32vector?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f32vector?".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(f32vector_predicate), effects: vec![Effect::Pure],
    })));
    
    env.define("f32vector-length".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f32vector-length".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(f32vector_length), effects: vec![Effect::Pure],
    })));
    
    env.define("f32vector-ref".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f32vector-ref".to_string(), arity_min: 2, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(f32vector_ref), effects: vec![Effect::Pure],
    })));
    
    env.define("f32vector-set!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f32vector-set!".to_string(), arity_min: 3, arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(f32vector_set), effects: vec![Effect::Mutation],
    })));
    
    env.define("f32vector->list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f32vector->list".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(f32vector_to_list), effects: vec![Effect::Pure],
    })));
    
    env.define("list->f32vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list->f32vector".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(list_to_f32vector), effects: vec![Effect::Pure],
    })));
}

//================================================
// F64VECTOR IMPLEMENTATION (64-bit floating point)
//================================================

fn make_f64vector(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("make-f64vector: expected 1 or 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let length = match &args[0] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "make-f64vector: length must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "make-f64vector: first argument must be an integer".to_string(),
            None,
        ))),
    };

    let fill_value = if args.len() == 2 {
        match &args[1] {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                *n as f64
            }
            Value::Literal(crate::ast::Literal::InexactReal(f)) => {
                *f
            }
            _ => return Err(Box::new(Error::runtime_error(
                "make-f64vector: fill value must be a number".to_string(),
                None,
            ))),
        }
    } else {
        0.0f64
    };

    let vector = vec![fill_value; length];
    let hv = HomogeneousVectorValue::F64Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn f64vector(args: &[Value]) -> Result<Value> {
    let mut vector = Vec::new();
    
    for arg in args {
        let value = match arg {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                *n as f64
            }
            Value::Literal(crate::ast::Literal::InexactReal(f)) => {
                *f
            }
            _ => return Err(Box::new(Error::runtime_error(
                "f64vector: all arguments must be numbers".to_string(),
                None,
            ))),
        };
        vector.push(value);
    }

    let hv = HomogeneousVectorValue::F64Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn f64vector_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("f64vector?: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let is_f64vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            matches!(hv.as_ref(), HomogeneousVectorValue::F64Vector(_))
        }
        _ => false,
    };

    Ok(Value::boolean(is_f64vector))
}

fn f64vector_length(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("f64vector-length: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::F64Vector(v) => {
                    Ok(Value::integer(v.borrow().len() as i64))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "f64vector-length: argument must be a f64vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "f64vector-length: argument must be a f64vector".to_string(),
            None,
        ))),
    }
}

fn f64vector_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("f64vector-ref: expected 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::F64Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "f64vector-ref: first argument must be a f64vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "f64vector-ref: first argument must be a f64vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "f64vector-ref: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "f64vector-ref: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let borrowed = vector.borrow();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("f64vector-ref: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    Ok(Value::number(borrowed[index]))
}

fn f64vector_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error(
            format!("f64vector-set!: expected 3 arguments, got {}", args.len()),
            None,
        )));
    }

    let vector = match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::F64Vector(v) => v.clone(),
                _ => return Err(Box::new(Error::runtime_error(
                    "f64vector-set!: first argument must be a f64vector".to_string(),
                    None,
                ))),
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "f64vector-set!: first argument must be a f64vector".to_string(),
            None,
        ))),
    };

    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            if *n < 0 {
                return Err(Box::new(Error::runtime_error(
                    "f64vector-set!: index must be non-negative".to_string(),
                    None,
                )));
            }
            *n as usize
        }
        _ => return Err(Box::new(Error::runtime_error(
            "f64vector-set!: second argument must be an integer".to_string(),
            None,
        ))),
    };

    let value = match &args[2] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
            *n as f64
        }
        Value::Literal(crate::ast::Literal::InexactReal(f)) => {
            *f
        }
        _ => return Err(Box::new(Error::runtime_error(
            "f64vector-set!: third argument must be a number".to_string(),
            None,
        ))),
    };

    let mut borrowed = vector.borrow_mut();
    if index >= borrowed.len() {
        return Err(Box::new(Error::runtime_error(
            format!("f64vector-set!: index {} out of bounds for vector of length {}", 
                   index, borrowed.len()),
            None,
        )));
    }

    borrowed[index] = value;
    Ok(Value::Unspecified)
}

fn f64vector_to_list(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("f64vector->list: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::HomogeneousVector(hv) => {
            match hv.as_ref() {
                HomogeneousVectorValue::F64Vector(v) => {
                    let borrowed = v.borrow();
                    let values: Vec<Value> = borrowed
                        .iter()
                        .map(|&x| Value::number(x))
                        .collect();
                    Ok(Value::list(values))
                }
                _ => Err(Box::new(Error::runtime_error(
                    "f64vector->list: argument must be a f64vector".to_string(),
                    None,
                ))),
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            "f64vector->list: argument must be a f64vector".to_string(),
            None,
        ))),
    }
}

fn list_to_f64vector(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("list->f64vector: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    let list = &args[0];
    let values = match list.as_list() {
        Some(list_values) => list_values,
        None => return Err(Box::new(Error::runtime_error(
            "list->f64vector: argument must be a list".to_string(),
            None,
        ))),
    };

    let mut vector = Vec::new();
    for value in values {
        let converted = match value {
            Value::Literal(crate::ast::Literal::ExactInteger(n)) => {
                n as f64
            }
            Value::Literal(crate::ast::Literal::InexactReal(f)) => {
                f
            }
            _ => return Err(Box::new(Error::runtime_error(
                "list->f64vector: all list elements must be numbers".to_string(),
                None,
            ))),
        };
        vector.push(converted);
    }

    let hv = HomogeneousVectorValue::F64Vector(Rc::new(RefCell::new(vector)));
    Ok(Value::HomogeneousVector(Arc::new(hv)))
}

fn init_f64vector_procedures(env: &Arc<ThreadSafeEnvironment>) {
    
    env.define("make-f64vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-f64vector".to_string(), arity_min: 1, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(make_f64vector), effects: vec![Effect::Pure],
    })));
    
    env.define("f64vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f64vector".to_string(), arity_min: 0, arity_max: None,
        implementation: PrimitiveImpl::RustFn(f64vector), effects: vec![Effect::Pure],
    })));
    
    env.define("f64vector?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f64vector?".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(f64vector_predicate), effects: vec![Effect::Pure],
    })));
    
    env.define("f64vector-length".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f64vector-length".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(f64vector_length), effects: vec![Effect::Pure],
    })));
    
    env.define("f64vector-ref".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f64vector-ref".to_string(), arity_min: 2, arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(f64vector_ref), effects: vec![Effect::Pure],
    })));
    
    env.define("f64vector-set!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f64vector-set!".to_string(), arity_min: 3, arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(f64vector_set), effects: vec![Effect::Mutation],
    })));
    
    env.define("f64vector->list".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "f64vector->list".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(f64vector_to_list), effects: vec![Effect::Pure],
    })));
    
    env.define("list->f64vector".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "list->f64vector".to_string(), arity_min: 1, arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(list_to_f64vector), effects: vec![Effect::Pure],
    })));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_make_u8vector() {
        // Test with length only
        let args = vec![Value::integer(5)];
        let result = make_u8vector(&args).unwrap();
        
        match result {
            Value::HomogeneousVector(hv) => {
                match hv.as_ref() {
                    HomogeneousVectorValue::U8Vector(v) => {
                        assert_eq!(v.borrow().len(), 5);
                        assert_eq!(v.borrow()[0], 0);
                    }
                    _ => panic!("Expected U8Vector"),
                }
            }
            _ => panic!("Expected HomogeneousVector"),
        }

        // Test with length and fill
        let args = vec![Value::integer(3), Value::integer(42)];
        let result = make_u8vector(&args).unwrap();
        
        match result {
            Value::HomogeneousVector(hv) => {
                match hv.as_ref() {
                    HomogeneousVectorValue::U8Vector(v) => {
                        assert_eq!(v.borrow().len(), 3);
                        assert_eq!(v.borrow()[0], 42);
                        assert_eq!(v.borrow()[2], 42);
                    }
                    _ => panic!("Expected U8Vector"),
                }
            }
            _ => panic!("Expected HomogeneousVector"),
        }
    }

    #[test]
    fn test_u8vector() {
        let args = vec![Value::integer(1), Value::integer(2), Value::integer(255)];
        let result = u8vector(&args).unwrap();
        
        match result {
            Value::HomogeneousVector(hv) => {
                match hv.as_ref() {
                    HomogeneousVectorValue::U8Vector(v) => {
                        let borrowed = v.borrow();
                        assert_eq!(borrowed.len(), 3);
                        assert_eq!(borrowed[0], 1);
                        assert_eq!(borrowed[1], 2);
                        assert_eq!(borrowed[2], 255);
                    }
                    _ => panic!("Expected U8Vector"),
                }
            }
            _ => panic!("Expected HomogeneousVector"),
        }
    }

    #[test]
    fn test_u8vector_predicate() {
        let vec = u8vector(&[Value::integer(1), Value::integer(2)]).unwrap();
        let result = u8vector_predicate(&[vec]).unwrap();
        assert_eq!(result, Value::boolean(true));

        let not_vec = Value::integer(42);
        let result = u8vector_predicate(&[not_vec]).unwrap();
        assert_eq!(result, Value::boolean(false));
    }

    #[test]
    fn test_s16vector_range() {
        // Test s16vector range (-32768 to 32767)
        let args = vec![Value::integer(-32768), Value::integer(0), Value::integer(32767)];
        let result = s16vector(&args).unwrap();
        
        match result {
            Value::HomogeneousVector(hv) => {
                match hv.as_ref() {
                    HomogeneousVectorValue::S16Vector(v) => {
                        let borrowed = v.borrow();
                        assert_eq!(borrowed.len(), 3);
                        assert_eq!(borrowed[0], -32768);
                        assert_eq!(borrowed[1], 0);
                        assert_eq!(borrowed[2], 32767);
                    }
                    _ => panic!("Expected S16Vector"),
                }
            }
            _ => panic!("Expected HomogeneousVector"),
        }
    }

    #[test]
    fn test_u32vector_range() {
        // Test u32vector range (0 to 4294967295)
        let args = vec![Value::integer(0), Value::integer(1000000), Value::integer(4294967295)];
        let result = u32vector(&args).unwrap();
        
        match result {
            Value::HomogeneousVector(hv) => {
                match hv.as_ref() {
                    HomogeneousVectorValue::U32Vector(v) => {
                        let borrowed = v.borrow();
                        assert_eq!(borrowed.len(), 3);
                        assert_eq!(borrowed[0], 0);
                        assert_eq!(borrowed[1], 1000000);
                        assert_eq!(borrowed[2], 4294967295);
                    }
                    _ => panic!("Expected U32Vector"),
                }
            }
            _ => panic!("Expected HomogeneousVector"),
        }
    }

    #[test]
    fn test_s32vector_range() {
        // Test s32vector range (-2147483648 to 2147483647)
        let args = vec![Value::integer(-2147483648), Value::integer(0), Value::integer(2147483647)];
        let result = s32vector(&args).unwrap();
        
        match result {
            Value::HomogeneousVector(hv) => {
                match hv.as_ref() {
                    HomogeneousVectorValue::S32Vector(v) => {
                        let borrowed = v.borrow();
                        assert_eq!(borrowed.len(), 3);
                        assert_eq!(borrowed[0], -2147483648);
                        assert_eq!(borrowed[1], 0);
                        assert_eq!(borrowed[2], 2147483647);
                    }
                    _ => panic!("Expected S32Vector"),
                }
            }
            _ => panic!("Expected HomogeneousVector"),
        }
    }

    #[test]
    fn test_f32vector() {
        // Test f32vector with mixed integer and float values
        let args = vec![Value::integer(42), Value::number(3.14159)];
        let result = f32vector(&args).unwrap();
        
        match result {
            Value::HomogeneousVector(hv) => {
                match hv.as_ref() {
                    HomogeneousVectorValue::F32Vector(v) => {
                        let borrowed = v.borrow();
                        assert_eq!(borrowed.len(), 2);
                        assert_eq!(borrowed[0], 42.0);
                        assert!((borrowed[1] - 3.14159).abs() < 0.001); // f32 precision
                    }
                    _ => panic!("Expected F32Vector"),
                }
            }
            _ => panic!("Expected HomogeneousVector"),
        }
    }

    #[test]
    fn test_f64vector() {
        // Test f64vector with mixed integer and float values
        let args = vec![Value::integer(42), Value::number(3.141592653589793)];
        let result = f64vector(&args).unwrap();
        
        match result {
            Value::HomogeneousVector(hv) => {
                match hv.as_ref() {
                    HomogeneousVectorValue::F64Vector(v) => {
                        let borrowed = v.borrow();
                        assert_eq!(borrowed.len(), 2);
                        assert_eq!(borrowed[0], 42.0);
                        assert_eq!(borrowed[1], 3.141592653589793);
                    }
                    _ => panic!("Expected F64Vector"),
                }
            }
            _ => panic!("Expected HomogeneousVector"),
        }
    }

    #[test]
    fn test_all_vector_predicates() {
        // Test all vector type predicates
        let u16_vec = u16vector(&[Value::integer(1000)]).unwrap();
        let s16_vec = s16vector(&[Value::integer(-1000)]).unwrap();
        let u32_vec = u32vector(&[Value::integer(100000)]).unwrap();
        let s32_vec = s32vector(&[Value::integer(-100000)]).unwrap();
        let f32_vec = f32vector(&[Value::number(3.14)]).unwrap();
        let f64_vec = f64vector(&[Value::number(2.71828)]).unwrap();
        
        // Each predicate should return true for its own type
        assert_eq!(u16vector_predicate(&[u16_vec.clone()]).unwrap(), Value::boolean(true));
        assert_eq!(s16vector_predicate(&[s16_vec.clone()]).unwrap(), Value::boolean(true));
        assert_eq!(u32vector_predicate(&[u32_vec.clone()]).unwrap(), Value::boolean(true));
        assert_eq!(s32vector_predicate(&[s32_vec.clone()]).unwrap(), Value::boolean(true));
        assert_eq!(f32vector_predicate(&[f32_vec.clone()]).unwrap(), Value::boolean(true));
        assert_eq!(f64vector_predicate(&[f64_vec.clone()]).unwrap(), Value::boolean(true));
        
        // Cross-type predicates should return false
        assert_eq!(u16vector_predicate(&[s16_vec]).unwrap(), Value::boolean(false));
        assert_eq!(s16vector_predicate(&[u32_vec]).unwrap(), Value::boolean(false));
        assert_eq!(u32vector_predicate(&[s32_vec]).unwrap(), Value::boolean(false));
        assert_eq!(s32vector_predicate(&[f32_vec]).unwrap(), Value::boolean(false));
        assert_eq!(f32vector_predicate(&[f64_vec]).unwrap(), Value::boolean(false));
        assert_eq!(f64vector_predicate(&[u16_vec]).unwrap(), Value::boolean(false));
    }
}
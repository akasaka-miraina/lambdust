//! Simplified SRFI-9 implementation using environment binding
//!
//! This approach directly binds constructor, predicate, and accessor functions
//! to the environment, avoiding complex procedure creation.

use crate::diagnostics::{Error, Result};
use crate::effects::Effect;
use crate::eval::value::{FieldInfo, RecordType};
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::stdlib::records_simple::{
    is_record_of_type, make_record, next_record_type_id, register_record_type,
};
use std::sync::Arc;

/// Installs simplified SRFI-9 record functionality
pub fn install_simple_srfi9(env: &Arc<ThreadSafeEnvironment>) {
    // Install define-point-type as an example
    env.define(
        "define-point-type".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "define-point-type".to_string(),
            arity_min: 0,
            arity_max: Some(0),
            implementation: PrimitiveImpl::RustFn(primitive_define_point_type),
            effects: vec![Effect::State],
        })),
    );
}

/// Creates point record type with make-point, point?, point-x, point-y, point-x-set!, point-y-set!
fn primitive_define_point_type(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "define-point-type takes no arguments".to_string(),
            None,
        )));
    }

    // Create and register point type
    let type_id = next_record_type_id();
    let record_type = RecordType {
        id: type_id,
        name: "point".to_string(),
        field_names: vec!["x".to_string(), "y".to_string()],
        constructor_name: Some("make-point".to_string()),
        predicate_name: Some("point?".to_string()),
        field_info: vec![
            FieldInfo {
                name: "x".to_string(),
                accessor: "point-x".to_string(),
                mutator: Some("point-x-set!".to_string()),
            },
            FieldInfo {
                name: "y".to_string(),
                accessor: "point-y".to_string(),
                mutator: Some("point-y-set!".to_string()),
            },
        ],
    };

    register_record_type(record_type)?;

    // Since we can't store type_id in closures, we'll use a global map approach
    // For now, just return the type_id and let the caller set up the environment
    Ok(Value::number(type_id as f64))
}

/// Global function to create point constructor
pub fn create_point_constructor(env: &Arc<ThreadSafeEnvironment>, type_id: u64) {
    env.define(
        "make-point".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "make-point".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(make_point_constructor),
            effects: vec![Effect::State],
        })),
    );
}

fn make_point_constructor(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("make-point expects 2 arguments, got {}", args.len()),
            None,
        )));
    }

    // Look up the point type by name
    use crate::stdlib::records_simple::lookup_record_type_id_by_name;
    let type_id = lookup_record_type_id_by_name("point").unwrap_or(1);

    let record = make_record(type_id, args.to_vec())?;
    Ok(Value::record(record))
}

pub fn create_point_predicate(env: &Arc<ThreadSafeEnvironment>, type_id: u64) {
    env.define(
        "point?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "point?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(point_predicate),
            effects: vec![Effect::Pure],
        })),
    );
}

fn point_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("point? expects 1 argument, got {}", args.len()),
            None,
        )));
    }

    // Look up the point type by name
    use crate::stdlib::records_simple::lookup_record_type_id_by_name;
    let type_id = lookup_record_type_id_by_name("point").unwrap_or(1);

    let is_point = is_record_of_type(&args[0], type_id);
    Ok(Value::boolean(is_point))
}

pub fn create_point_accessors(env: &Arc<ThreadSafeEnvironment>) {
    // point-x accessor
    env.define(
        "point-x".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "point-x".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(point_x_accessor),
            effects: vec![Effect::Pure],
        })),
    );

    // point-y accessor
    env.define(
        "point-y".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "point-y".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(point_y_accessor),
            effects: vec![Effect::Pure],
        })),
    );

    // point-x-set! mutator
    env.define(
        "point-x-set!".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "point-x-set!".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(point_x_mutator),
            effects: vec![Effect::State],
        })),
    );

    // point-y-set! mutator
    env.define(
        "point-y-set!".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "point-y-set!".to_string(),
            arity_min: 2,
            arity_max: Some(2),
            implementation: PrimitiveImpl::RustFn(point_y_mutator),
            effects: vec![Effect::State],
        })),
    );
}

fn point_x_accessor(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "point-x expects 1 argument".to_string(),
            None,
        )));
    }

    let record = match &args[0] {
        Value::Record(rec) => rec,
        _ => {
            return Err(Box::new(Error::runtime_error(
                "point-x expects a record argument".to_string(),
                None,
            )));
        }
    };

    let fields = record.fields.borrow();
    fields.first().cloned().ok_or_else(|| {
        Box::new(Error::runtime_error(
            "Record has no x field".to_string(),
            None,
        ))
    })
}

fn point_y_accessor(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "point-y expects 1 argument".to_string(),
            None,
        )));
    }

    let record = match &args[0] {
        Value::Record(rec) => rec,
        _ => {
            return Err(Box::new(Error::runtime_error(
                "point-y expects a record argument".to_string(),
                None,
            )));
        }
    };

    let fields = record.fields.borrow();
    fields.get(1).cloned().ok_or_else(|| {
        Box::new(Error::runtime_error(
            "Record has no y field".to_string(),
            None,
        ))
    })
}

fn point_x_mutator(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            "point-x-set! expects 2 arguments".to_string(),
            None,
        )));
    }

    let record = match &args[0] {
        Value::Record(rec) => rec,
        _ => {
            return Err(Box::new(Error::runtime_error(
                "point-x-set! expects a record argument".to_string(),
                None,
            )));
        }
    };

    let mut fields = record.fields.borrow_mut();
    if fields.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "Record has no x field".to_string(),
            None,
        )));
    }

    fields[0] = args[1].clone();
    Ok(Value::Unspecified)
}

fn point_y_mutator(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            "point-y-set! expects 2 arguments".to_string(),
            None,
        )));
    }

    let record = match &args[0] {
        Value::Record(rec) => rec,
        _ => {
            return Err(Box::new(Error::runtime_error(
                "point-y-set! expects a record argument".to_string(),
                None,
            )));
        }
    };

    let mut fields = record.fields.borrow_mut();
    if fields.len() < 2 {
        return Err(Box::new(Error::runtime_error(
            "Record has no y field".to_string(),
            None,
        )));
    }

    fields[1] = args[1].clone();
    Ok(Value::Unspecified)
}

/// Install complete point type
pub fn install_point_type(env: &Arc<ThreadSafeEnvironment>) {
    // Clear existing registries to avoid conflicts in testing
    crate::stdlib::records_simple::clear_record_registries();

    // Create and register the record type first
    let type_id = next_record_type_id();
    let record_type = RecordType {
        id: type_id,
        name: "point".to_string(),
        field_names: vec!["x".to_string(), "y".to_string()],
        constructor_name: Some("make-point".to_string()),
        predicate_name: Some("point?".to_string()),
        field_info: vec![
            FieldInfo {
                name: "x".to_string(),
                accessor: "point-x".to_string(),
                mutator: Some("point-x-set!".to_string()),
            },
            FieldInfo {
                name: "y".to_string(),
                accessor: "point-y".to_string(),
                mutator: Some("point-y-set!".to_string()),
            },
        ],
    };

    if let Err(e) = register_record_type(record_type) {
        eprintln!("Failed to register point type: {:?}", e);
        return;
    }

    // Install all the procedures
    create_point_constructor(env, type_id);
    create_point_predicate(env, type_id);
    create_point_accessors(env);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_type_installation() {
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        install_point_type(&env);

        // Check that functions are defined
        assert!(env.lookup("make-point").is_some());
        assert!(env.lookup("point?").is_some());
        assert!(env.lookup("point-x").is_some());
        assert!(env.lookup("point-y").is_some());
        assert!(env.lookup("point-x-set!").is_some());
        assert!(env.lookup("point-y-set!").is_some());
    }
}

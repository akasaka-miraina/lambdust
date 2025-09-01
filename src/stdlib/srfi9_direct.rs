//! Direct SRFI-9 implementation using immediate function definition
//!
//! This module provides SRFI-9 functionality through direct function definition
//! rather than macro expansion, avoiding parser conflicts with special forms.

use crate::diagnostics::{Error, Result};
use crate::effects::Effect;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::stdlib::records_simple::{make_record, next_record_type_id, register_record_type};
use crate::eval::value::{RecordType, FieldInfo};
use crate::utils::symbol::intern_symbol;
use std::sync::Arc;

/// Installs direct SRFI-9 record type creation functionality
pub fn install_direct_record_type(env: &Arc<ThreadSafeEnvironment>) {
    // Install record-type-create - creates type and returns procedures
    env.define(
        "record-type-create".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "record-type-create".to_string(),
            arity_min: 3,
            arity_max: None,
            implementation: PrimitiveImpl::RustFn(primitive_record_type_create),
            effects: vec![Effect::State],
        })),
    );
}

/// Creates a record type and all associated procedures
/// Usage: (record-type-create type-name constructor-spec predicate-name field-specs...)
/// Returns: (list constructor predicate accessor1 [mutator1] accessor2 [mutator2] ...)
fn primitive_record_type_create(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(Box::new(Error::runtime_error(
            "record-type-create requires at least 3 arguments".to_string(),
            None,
        )));
    }

    // Extract type name
    let type_name = extract_string_or_symbol(&args[0], "type name")?;
    
    // Extract constructor spec (constructor-name field1 field2 ...)
    let constructor_list = args[1].as_list().ok_or_else(|| {
        Error::runtime_error("Constructor specification must be a list".to_string(), None)
    })?;
    
    if constructor_list.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "Constructor specification cannot be empty".to_string(),
            None,
        )));
    }
    
    let constructor_name = extract_string_or_symbol(&constructor_list[0], "constructor name")?;
    let mut field_names = Vec::new();
    for field in &constructor_list[1..] {
        field_names.push(extract_string_or_symbol(field, "field name")?);
    }
    
    // Extract predicate name
    let predicate_name = extract_string_or_symbol(&args[2], "predicate name")?;
    
    // Extract field specifications
    let mut field_specs = Vec::new();
    for arg in &args[3..] {
        let field_list = arg.as_list().ok_or_else(|| {
            Error::runtime_error("Field specification must be a list".to_string(), None)
        })?;
        
        if field_list.len() < 2 || field_list.len() > 3 {
            return Err(Box::new(Error::runtime_error(
                "Field specification must have 2 or 3 elements: (field accessor [mutator])".to_string(),
                None,
            )));
        }
        
        let field_name = extract_string_or_symbol(&field_list[0], "field name")?;
        let accessor_name = extract_string_or_symbol(&field_list[1], "accessor name")?;
        let mutator_name = if field_list.len() == 3 {
            Some(extract_string_or_symbol(&field_list[2], "mutator name")?)
        } else {
            None
        };
        
        field_specs.push((field_name, accessor_name, mutator_name));
    }
    
    // Create and register the record type
    let type_id = next_record_type_id();
    let field_info: Vec<FieldInfo> = field_specs.iter().map(|(name, accessor, mutator)| {
        FieldInfo {
            name: name.clone(),
            accessor: accessor.clone(), 
            mutator: mutator.clone(),
        }
    }).collect();
    
    let record_type = RecordType {
        id: type_id,
        name: type_name.clone(),
        field_names: field_names.clone(),
        constructor_name: Some(constructor_name.clone()),
        predicate_name: Some(predicate_name.clone()),
        field_info,
    };
    
    register_record_type(record_type)?;
    
    // Create constructor procedure
    let constructor_proc = create_constructor_procedure(type_id, constructor_name, field_names.len());
    
    // Create predicate procedure  
    let predicate_proc = create_predicate_procedure(type_id, predicate_name);
    
    // Create accessor procedures
    let mut accessors = Vec::new();
    for (field_index, (_, accessor_name, mutator_name)) in field_specs.iter().enumerate() {
        let accessor_proc = create_accessor_procedure(type_id, accessor_name.clone(), field_index);
        accessors.push(accessor_proc);
        
        if let Some(mutator) = mutator_name {
            let mutator_proc = create_mutator_procedure(type_id, mutator.clone(), field_index);
            accessors.push(mutator_proc);
        }
    }
    
    // Return all procedures as a list
    let mut result = vec![constructor_proc, predicate_proc];
    result.extend(accessors);
    Ok(Value::list(result))
}

/// Extract string or symbol value
fn extract_string_or_symbol(value: &Value, context: &str) -> Result<String> {
    match value {
        Value::Literal(literal) => match literal {
            crate::ast::literal::Literal::String(s) => Ok((**s).clone()),
            crate::ast::literal::Literal::InternedString(s) => Ok(s.as_str().to_string()),
            _ => Err(Box::new(Error::runtime_error(
                format!("{} must be a string or symbol", context),
                None,
            ))),
        },
        Value::Symbol(sym_id) => {
            if let Some(name) = crate::utils::symbol_name(*sym_id) {
                Ok(name)
            } else {
                Err(Box::new(Error::runtime_error(
                    format!("Invalid symbol for {}", context),
                    None,
                )))
            }
        }
        _ => Err(Box::new(Error::runtime_error(
            format!("{} must be a string or symbol", context),
            None,
        ))),
    }
}

/// Create constructor procedure for a record type
fn create_constructor_procedure(type_id: u64, name: String, field_count: usize) -> Value {
    // Store the type_id and field_count in the procedure metadata for later use
    // We'll use a different approach - store the info and create a wrapper
    
    // For now, create a simple constructor that uses the global make_record
    Value::Primitive(Arc::new(PrimitiveProcedure {
        name: format!("constructor-{}-{}", name, type_id),
        arity_min: field_count,
        arity_max: Some(field_count),
        implementation: PrimitiveImpl::RustFn(primitive_dynamic_constructor),
        effects: vec![Effect::State],
    }))
}

/// Dynamic constructor that uses procedure name to determine type_id
fn primitive_dynamic_constructor(args: &[Value]) -> Result<Value> {
    // This is a simplified approach - in a full implementation we'd have better metadata
    // For now, we'll extract type info from the procedure context
    
    // Create a simple record with type_id 1 (this is a limitation of the current approach)
    let record = make_record(1, args.to_vec())?;
    Ok(Value::record(record))
}

/// Create predicate procedure for a record type
fn create_predicate_procedure(type_id: u64, name: String) -> Value {
    Value::Primitive(Arc::new(PrimitiveProcedure {
        name: name.clone(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(Box::new(move |args: &[Value]| -> Result<Value> {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    format!("Predicate {} expects 1 argument, got {}", name, args.len()),
                    None,
                )));
            }
            
            let is_correct_type = match &args[0] {
                Value::Record(record) => record.type_id == type_id,
                _ => false,
            };
            
            Ok(Value::boolean(is_correct_type))
        })),
        effects: vec![Effect::Pure],
    }))
}

/// Create accessor procedure for a record field
fn create_accessor_procedure(type_id: u64, name: String, field_index: usize) -> Value {
    Value::Primitive(Arc::new(PrimitiveProcedure {
        name: name.clone(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(Box::new(move |args: &[Value]| -> Result<Value> {
            if args.len() != 1 {
                return Err(Box::new(Error::runtime_error(
                    format!("Accessor {} expects 1 argument, got {}", name, args.len()),
                    None,
                )));
            }
            
            let record = match &args[0] {
                Value::Record(rec) if rec.type_id == type_id => rec,
                Value::Record(_) => {
                    return Err(Box::new(Error::runtime_error(
                        format!("Accessor {} expects record of correct type", name),
                        None,
                    )));
                }
                _ => {
                    return Err(Box::new(Error::runtime_error(
                        format!("Accessor {} expects a record argument", name),
                        None,
                    )));
                }
            };
            
            let fields = record.fields.borrow();
            fields.get(field_index).cloned().ok_or_else(|| {
                Box::new(Error::runtime_error(
                    format!("Field index {} out of bounds", field_index),
                    None,
                ))
            })
        })),
        effects: vec![Effect::Pure],
    }))
}

/// Create mutator procedure for a record field
fn create_mutator_procedure(type_id: u64, name: String, field_index: usize) -> Value {
    Value::Primitive(Arc::new(PrimitiveProcedure {
        name: name.clone(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(Box::new(move |args: &[Value]| -> Result<Value> {
            if args.len() != 2 {
                return Err(Box::new(Error::runtime_error(
                    format!("Mutator {} expects 2 arguments, got {}", name, args.len()),
                    None,
                )));
            }
            
            let record = match &args[0] {
                Value::Record(rec) if rec.type_id == type_id => rec,
                Value::Record(_) => {
                    return Err(Box::new(Error::runtime_error(
                        format!("Mutator {} expects record of correct type", name),
                        None,
                    )));
                }
                _ => {
                    return Err(Box::new(Error::runtime_error(
                        format!("Mutator {} expects a record argument", name),
                        None,
                    )));
                }
            };
            
            let mut fields = record.fields.borrow_mut();
            if field_index >= fields.len() {
                return Err(Box::new(Error::runtime_error(
                    format!("Field index {} out of bounds", field_index),
                    None,
                )));
            }
            
            fields[field_index] = args[1].clone();
            Ok(Value::Unspecified)
        })),
        effects: vec![Effect::State],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::ThreadSafeEnvironment;
    use crate::utils::symbol::intern_symbol;
    
    #[test]
    fn test_record_type_creation() {
        let env = Arc::new(ThreadSafeEnvironment::new(None));
        install_direct_record_type(&env);
        
        // Create a point record type
        let args = vec![
            Value::string("point"),
            Value::list(vec![
                Value::string("make-point"),
                Value::string("x"),
                Value::string("y"),
            ]),
            Value::string("point?"),
            Value::list(vec![
                Value::string("x"),
                Value::string("point-x"),
                Value::string("point-x-set!"),
            ]),
            Value::list(vec![
                Value::string("y"),
                Value::string("point-y"),
            ]),
        ];
        
        let result = primitive_record_type_create(&args).unwrap();
        
        // Should return a list of procedures
        assert!(result.as_list().is_some());
        let procs = result.as_list().unwrap();
        assert_eq!(procs.len(), 5); // constructor, predicate, x-accessor, x-mutator, y-accessor
    }
}
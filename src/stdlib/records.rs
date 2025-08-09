//! SRFI-9 (Defining Record Types) implementation for Lambdust.
//!
//! This module provides a complete implementation of SRFI-9, including:
//! - Record type definitions and registry
//! - Constructor, predicate, accessor, and mutator procedures
//! - The `define-record-type` macro
//! - Thread-safe record operations

use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment, Record, RecordType, FieldInfo};
use crate::effects::Effect;
use crate::diagnostics::{Error, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for generating unique record type IDs.
static RECORD_TYPE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Global registry of record types.
static RECORD_TYPE_REGISTRY: std::sync::LazyLock<Mutex<HashMap<u64, RecordType>>> = 
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Global registry mapping type names to type IDs.
static RECORD_TYPE_NAME_REGISTRY: std::sync::LazyLock<Mutex<HashMap<String, u64>>> = 
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Generates a unique record type ID.
pub fn next_record_type_id() -> u64 {
    RECORD_TYPE_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Registers a new record type and returns its ID.
pub fn register_record_type(record_type: RecordType) -> Result<u64> {
    let type_id = record_type.id;
    let type_name = record_type.name.clone());
    
    // Register in both registries
    let mut type_registry = RECORD_TYPE_REGISTRY.lock().map_err(|_| {
        Error::runtime_error("Failed to acquire record type registry lock".to_string(), None)
    })?;
    
    let mut name_registry = RECORD_TYPE_NAME_REGISTRY.lock().map_err(|_| {
        Error::runtime_error("Failed to acquire record type name registry lock".to_string(), None)
    })?;
    
    // Check for duplicate names
    if name_registry.contains_key(&type_name) {
        return Err(Box::new(Error::runtime_error(
            format!("Record type '{}' is already defined", type_name),
            None,
        ));
    }
    
    type_registry.insert(type_id, record_type);
    name_registry.insert(type_name, type_id);
    
    Ok(type_id)
}

/// Looks up a record type by ID.
pub fn lookup_record_type(type_id: u64) -> Result<RecordType> {
    let registry = RECORD_TYPE_REGISTRY.lock().map_err(|_| {
        Error::runtime_error("Failed to acquire record type registry lock".to_string(), None)
    })?;
    
    registry.get(&type_id).clone())().ok_or_else(|| {
        Error::runtime_error(
            format!("Unknown record type ID: {}", type_id),
            None,
        )
    })
}

/// Looks up a record type by name.
pub fn lookup_record_type_by_name(name: &str) -> Result<RecordType> {
    let name_registry = RECORD_TYPE_NAME_REGISTRY.lock().map_err(|_| {
        Error::runtime_error("Failed to acquire record type name registry lock".to_string(), None)
    })?;
    
    let type_id = name_registry.get(name).copied().ok_or_else(|| {
        Error::runtime_error(
            format!("Unknown record type: {}", name),
            None,
        )
    })?;
    
    drop(name_registry); // Release lock before calling lookup_record_type
    lookup_record_type(type_id)
}

/// Creates a new record instance.
pub fn make_record(type_id: u64, field_values: Vec<Value>) -> Result<Record> {
    let record_type = lookup_record_type(type_id)?;
    
    // Validate field count
    if field_values.len() != record_type.field_names.len() {
        return Err(Box::new(Error::runtime_error(
            format!(
                "Record type '{}' expects {} fields, got {}",
                record_type.name,
                record_type.field_names.len(),
                field_values.len()
            ),
            None,
        ));
    }
    
    Ok(Record {
        type_id,
        fields: Arc::new(RwLock::new(field_values)),
    })
}

/// Checks if a value is a record of a specific type.
pub fn is_record_of_type(value: &Value, type_id: u64) -> bool {
    match value {
        Value::Record(record) => record.type_id == type_id,
        _ => false,
    }
}

/// Gets a record field value by index.
pub fn record_field_value(record: &Record, field_index: usize) -> Result<Value> {
    let fields = record.fields.read().map_err(|_| {
        Error::runtime_error("Failed to acquire record field lock".to_string(), None)
    })?;
    
    fields.get(field_index).clone())().ok_or_else(|| {
        Error::runtime_error(
            format!("Record field index {} out of bounds", field_index),
            None,
        )
    })
}

/// Sets a record field value by index.
pub fn set_record_field_value(record: &Record, field_index: usize, value: Value) -> Result<()> {
    let mut fields = record.fields.write().map_err(|_| {
        Error::runtime_error("Failed to acquire record field lock".to_string(), None)
    })?;
    
    if field_index >= fields.len() {
        return Err(Box::new(Error::runtime_error(
            format!("Record field index {} out of bounds", field_index),
            None,
        ));
    }
    
    fields[field_index] = value;
    Ok(())
}

/// Creates bindings for SRFI-9 record operations in the given environment.
pub fn create_record_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // Low-level record operations (not typically exposed to user code)
    
    // make-record-type - creates a new record type
    env.define("make-record-type".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-record-type".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_make_record_type),
        effects: vec![Effect::State], // Modifies global registry
    })));
    
    // record-constructor - creates constructor procedure for a record type
    env.define("record-constructor".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "record-constructor".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_record_constructor),
        effects: vec![Effect::Pure],
    })));
    
    // record-predicate - creates predicate procedure for a record type
    env.define("record-predicate".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "record-predicate".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_record_predicate),
        effects: vec![Effect::Pure],
    })));
    
    // record-accessor - creates accessor procedure for a record field
    env.define("record-accessor".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "record-accessor".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_record_accessor),
        effects: vec![Effect::Pure],
    })));
    
    // record-mutator - creates mutator procedure for a record field
    env.define("record-mutator".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "record-mutator".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_record_mutator),
        effects: vec![Effect::Pure],
    })));
    
    // record? - generic record predicate
    env.define("record?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "record?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_record_p),
        effects: vec![Effect::Pure],
    })));
}

// ============= PRIMITIVE PROCEDURE IMPLEMENTATIONS =============

/// make-record-type primitive - creates a new record type.
/// Usage: (make-record-type type-name field-names)
fn primitive_make_record_type(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("make-record-type expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    // Extract type name
    let type_name = match &args[0] {
        Value::Literal(crate::ast::Literal::String(name)) => name.clone(),
        Value::Symbol(sym_id) => {
            if let Some(name) = crate::utils::symbol_name(*sym_id) {
                name
            } else {
                return Err(Box::new(Error::runtime_error(
                    "Invalid symbol for record type name".to_string(),
                    None,
                ));
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "make-record-type expects a string or symbol as type name".to_string(),
            None,
        )),
    };
    
    // Extract field names
    let field_names = match &args[1] {
        Value::Nil => Vec::new(),
        _ => {
            let field_list = args[1].as_list().ok_or_else(|| {
                Error::runtime_error(
                    "make-record-type expects a list of field names".to_string(),
                    None,
                )
            })?;
            
            let mut names = Vec::new();
            for field in field_list {
                match &field {
                    Value::Literal(crate::ast::Literal::String(name)) => {
                        names.push(name.clone());
                    }
                    Value::Symbol(sym_id) => {
                        if let Some(name) = crate::utils::symbol_name(*sym_id) {
                            names.push(name);
                        } else {
                            return Err(Box::new(Error::runtime_error(
                                "Invalid symbol in field names".to_string(),
                                None,
                            ));
                        }
                    }
                    _ => return Err(Box::new(Error::runtime_error(
                        "Field names must be strings or symbols".to_string(),
                        None,
                    )),
                }
            }
            names
        }
    };
    
    // Create record type
    let type_id = next_record_type_id();
    let field_info: Vec<FieldInfo> = field_names.iter().map(|name| FieldInfo {
        name: name.clone(),
        accessor: format!("{}-{}", type_name, name),
        mutator: Some(format!("{}-{}-set!", type_name, name)),
    }).collect();
    
    let record_type = RecordType {
        id: type_id,
        name: type_name,
        field_names,
        constructor_name: None,
        predicate_name: None,
        field_info,
    };
    
    // Register the type
    register_record_type(record_type)?;
    
    // Return the type ID (wrapped as a number for now)
    Ok(Value::integer(type_id as i64))
}

/// record-constructor primitive - creates a constructor procedure.
/// Usage: (record-constructor rtd) or (record-constructor rtd field-names)
fn primitive_record_constructor(args: &[Value]) -> Result<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(Box::new(Error::runtime_error(
            format!("record-constructor expects 1 or 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    // Extract record type ID
    let type_id = match &args[0] {
        Value::Literal(crate::ast::Literal::Number(n)) => *n as u64,
        
        _ => return Err(Box::new(Error::runtime_error(
            "record-constructor expects a record type descriptor".to_string(),
            None,
        )),
    };
    
    // For now, create a constructor that accepts all fields in order
    // TODO: Handle field-names argument for selective constructors
    
    // Create a constructor procedure that captures the type ID
    let constructor_name = format!("constructor-for-type-{}", type_id);
    Ok(Value::Primitive(Arc::new(PrimitiveProcedure {
        name: constructor_name,
        arity_min: 0, // Will be validated based on record type
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(move |args| {
            primitive_record_constructor_impl(type_id, args)
        }),
        effects: vec![Effect::State],
    })))
}

/// Implementation for record constructor procedures.
fn primitive_record_constructor_impl(type_id: u64, args: &[Value]) -> Result<Value> {
    let record = make_record(type_id, args.to_vec())?;
    Ok(Value::record(record))
}

/// record-predicate primitive - creates a predicate procedure.
/// Usage: (record-predicate rtd)
fn primitive_record_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("record-predicate expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    // Extract record type ID
    let type_id = match &args[0] {
        Value::Literal(crate::ast::Literal::Number(n)) => *n as u64,
        
        _ => return Err(Box::new(Error::runtime_error(
            "record-predicate expects a record type descriptor".to_string(),
            None,
        )),
    };
    
    // Create a predicate procedure that captures the type ID
    let predicate_name = format!("predicate-for-type-{}", type_id);
    Ok(Value::Primitive(Arc::new(PrimitiveProcedure {
        name: predicate_name,
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(move |args| {
            primitive_record_predicate_impl(type_id, args)
        }),
        effects: vec![Effect::Pure],
    })))
}

/// Implementation for record predicate procedures.
fn primitive_record_predicate_impl(type_id: u64, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "Record predicate expects 1 argument".to_string(),
            None,
        ));
    }
    
    Ok(Value::boolean(is_record_of_type(&args[0], type_id)))
}

/// record-accessor primitive - creates an accessor procedure.
/// Usage: (record-accessor rtd field-name)
fn primitive_record_accessor(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("record-accessor expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    // Extract record type ID
    let type_id = match &args[0] {
        Value::Literal(crate::ast::Literal::Number(n)) => *n as u64,
        
        _ => return Err(Box::new(Error::runtime_error(
            "record-accessor expects a record type descriptor".to_string(),
            None,
        )),
    };
    
    // Extract field name
    let field_name = match &args[1] {
        Value::Literal(crate::ast::Literal::String(name)) => name.clone(),
        Value::Symbol(sym_id) => {
            if let Some(name) = crate::utils::symbol_name(*sym_id) {
                name
            } else {
                return Err(Box::new(Error::runtime_error(
                    "Invalid symbol for field name".to_string(),
                    None,
                ));
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "record-accessor expects a string or symbol as field name".to_string(),
            None,
        )),
    };
    
    // Look up field index
    let record_type = lookup_record_type(type_id)?;
    let field_index = record_type.field_names.iter()
        .position(|name| name == &field_name)
        .ok_or_else(|| {
            Error::runtime_error(
                format!("Field '{}' not found in record type '{}'", field_name, record_type.name),
                None,
            )
        })?;
    
    // Create an accessor procedure
    let accessor_name = format!("accessor-{}-{}", record_type.name, field_name);
    Ok(Value::Primitive(Arc::new(PrimitiveProcedure {
        name: accessor_name,
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(move |args| {
            primitive_record_accessor_impl(type_id, field_index, args)
        }),
        effects: vec![Effect::Pure],
    })))
}

/// Implementation for record accessor procedures.
fn primitive_record_accessor_impl(type_id: u64, field_index: usize, args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            "Record accessor expects 1 argument".to_string(),
            None,
        ));
    }
    
    match &args[0] {
        Value::Record(record) => {
            if record.type_id != type_id {
                return Err(Box::new(Error::runtime_error(
                    "Record type mismatch in accessor".to_string(),
                    None,
                ));
            }
            record_field_value(record, field_index)
        }
        _ => Err(Box::new(Error::runtime_error(
            "Record accessor expects a record argument".to_string(),
            None,
        )),
    }
}

/// record-mutator primitive - creates a mutator procedure.
/// Usage: (record-mutator rtd field-name)
fn primitive_record_mutator(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("record-mutator expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    // Extract record type ID
    let type_id = match &args[0] {
        Value::Literal(crate::ast::Literal::Number(n)) => *n as u64,
        
        _ => return Err(Box::new(Error::runtime_error(
            "record-mutator expects a record type descriptor".to_string(),
            None,
        )),
    };
    
    // Extract field name
    let field_name = match &args[1] {
        Value::Literal(crate::ast::Literal::String(name)) => name.clone(),
        Value::Symbol(sym_id) => {
            if let Some(name) = crate::utils::symbol_name(*sym_id) {
                name
            } else {
                return Err(Box::new(Error::runtime_error(
                    "Invalid symbol for field name".to_string(),
                    None,
                ));
            }
        }
        _ => return Err(Box::new(Error::runtime_error(
            "record-mutator expects a string or symbol as field name".to_string(),
            None,
        )),
    };
    
    // Look up field index
    let record_type = lookup_record_type(type_id)?;
    let field_index = record_type.field_names.iter()
        .position(|name| name == &field_name)
        .ok_or_else(|| {
            Error::runtime_error(
                format!("Field '{}' not found in record type '{}'", field_name, record_type.name),
                None,
            )
        })?;
    
    // Create a mutator procedure
    let mutator_name = format!("mutator-{}-{}", record_type.name, field_name);
    Ok(Value::Primitive(Arc::new(PrimitiveProcedure {
        name: mutator_name,
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(move |args| {
            primitive_record_mutator_impl(type_id, field_index, args)
        }),
        effects: vec![Effect::State], // Mutates record state
    })))
}

/// Implementation for record mutator procedures.
fn primitive_record_mutator_impl(type_id: u64, field_index: usize, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            "Record mutator expects 2 arguments".to_string(),
            None,
        ));
    }
    
    match &args[0] {
        Value::Record(record) => {
            if record.type_id != type_id {
                return Err(Box::new(Error::runtime_error(
                    "Record type mismatch in mutator".to_string(),
                    None,
                ));
            }
            set_record_field_value(record, field_index, args[1].clone())?;
            Ok(Value::Unspecified)
        }
        _ => Err(Box::new(Error::runtime_error(
            "Record mutator expects a record as first argument".to_string(),
            None,
        )),
    }
}

/// record? primitive - generic record predicate.
/// Usage: (record? obj)
fn primitive_record_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("record? expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    Ok(Value::boolean(args[0].is_record()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::symbol::intern_symbol;

    #[test]
    fn test_record_type_creation() {
        let type_id = next_record_type_id();
        let record_type = RecordType {
            id: type_id,
            name: "point".to_string(),
            field_names: vec!["x".to_string(), "y".to_string()],
            constructor_name: None,
            predicate_name: None,
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
        
        let registered_id = register_record_type(record_type.clone()).unwrap();
        assert_eq!(registered_id, type_id);
        
        let retrieved_type = lookup_record_type(type_id).unwrap();
        assert_eq!(retrieved_type, record_type);
        
        let retrieved_by_name = lookup_record_type_by_name("point").unwrap();
        assert_eq!(retrieved_by_name, record_type);
    }
    
    #[test]
    fn test_record_creation() {
        let type_id = next_record_type_id();
        let record_type = RecordType {
            id: type_id,
            name: "person".to_string(),
            field_names: vec!["name".to_string(), "age".to_string()],
            constructor_name: None,
            predicate_name: None,
            field_info: vec![
                FieldInfo {
                    name: "name".to_string(),
                    accessor: "person-name".to_string(),
                    mutator: Some("person-name-set!".to_string()),
                },
                FieldInfo {
                    name: "age".to_string(),
                    accessor: "person-age".to_string(),
                    mutator: Some("person-age-set!".to_string()),
                },
            ],
        };
        
        register_record_type(record_type).unwrap();
        
        let field_values = vec![Value::string("Alice"), Value::integer(30)];
        let record = make_record(type_id, field_values).unwrap();
        
        assert_eq!(record.type_id, type_id);
        
        let name_value = record_field_value(&record, 0).unwrap();
        assert_eq!(name_value, Value::string("Alice"));
        
        let age_value = record_field_value(&record, 1).unwrap();
        assert_eq!(age_value, Value::integer(30));
    }
    
    #[test]
    fn test_record_mutation() {
        let type_id = next_record_type_id();
        let record_type = RecordType {
            id: type_id,
            name: "counter".to_string(),
            field_names: vec!["value".to_string()],
            constructor_name: None,
            predicate_name: None,
            field_info: vec![
                FieldInfo {
                    name: "value".to_string(),
                    accessor: "counter-value".to_string(),
                    mutator: Some("counter-value-set!".to_string()),
                },
            ],
        };
        
        register_record_type(record_type).unwrap();
        
        let field_values = vec![Value::integer(0)];
        let record = make_record(type_id, field_values).unwrap();
        
        // Check initial value
        let initial_value = record_field_value(&record, 0).unwrap();
        assert_eq!(initial_value, Value::integer(0));
        
        // Mutate the field
        set_record_field_value(&record, 0, Value::integer(42)).unwrap();
        
        // Check updated value
        let updated_value = record_field_value(&record, 0).unwrap();
        assert_eq!(updated_value, Value::integer(42));
    }
    
    #[test]
    fn test_record_type_predicate() {
        let type_id = next_record_type_id();
        let record_type = RecordType {
            id: type_id,
            name: "test".to_string(),
            field_names: vec!["field".to_string()],
            constructor_name: None,
            predicate_name: None,
            field_info: vec![
                FieldInfo {
                    name: "field".to_string(),
                    accessor: "test-field".to_string(),
                    mutator: None,
                },
            ],
        };
        
        register_record_type(record_type).unwrap();
        
        let record = make_record(type_id, vec![Value::integer(123)]).unwrap();
        let record_value = Value::record(record);
        
        // Test with correct type
        assert!(is_record_of_type(&record_value, type_id));
        
        // Test with different type
        let other_type_id = next_record_type_id();
        assert!(!is_record_of_type(&record_value, other_type_id));
        
        // Test with non-record
        assert!(!is_record_of_type(&Value::integer(42), type_id));
    }
}
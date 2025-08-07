//! Simplified SRFI-9 (Defining Record Types) implementation for Lambdust.
//!
//! This is a simplified implementation that avoids closure issues by using
//! a different approach for dynamic primitive creation.

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

/// Creates bindings for SRFI-9 record operations in the given environment.
pub fn create_record_bindings(env: &Arc<ThreadSafeEnvironment>) {
    // make-record - creates a record instance from type ID and values
    env.define("make-record".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "make-record".to_string(),
        arity_min: 1,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_make_record),
        effects: vec![Effect::State],
    })));
    
    // record-type-id - extracts type ID from a record
    env.define("record-type-id".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "record-type-id".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_record_type_id),
        effects: vec![Effect::Pure],
    })));
    
    // record-field-ref - get field value by index
    env.define("record-field-ref".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "record-field-ref".to_string(),
        arity_min: 2,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(primitive_record_field_ref),
        effects: vec![Effect::Pure],
    })));
    
    // record-field-set! - set field value by index
    env.define("record-field-set!".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "record-field-set!".to_string(),
        arity_min: 3,
        arity_max: Some(3),
        implementation: PrimitiveImpl::RustFn(primitive_record_field_set),
        effects: vec![Effect::State],
    })));
    
    // record? - generic record predicate
    env.define("record?".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "record?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(primitive_record_p),
        effects: vec![Effect::Pure],
    })));
    
    // define-record-type-helper - helper for the macro expansion
    env.define("define-record-type-helper".to_string(), Value::Primitive(Arc::new(PrimitiveProcedure {
        name: "define-record-type-helper".to_string(),
        arity_min: 3,
        arity_max: None,
        implementation: PrimitiveImpl::RustFn(primitive_define_record_type_helper),
        effects: vec![Effect::State],
    })));
}

// ============= PRIMITIVE PROCEDURE IMPLEMENTATIONS =============

/// make-record primitive - creates a record instance.
/// Usage: (make-record type-id field1 field2 ...)
fn primitive_make_record(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(Error::runtime_error(
            "make-record requires at least a type ID".to_string(),
            None,
        ));
    }
    
    // Extract record type ID
    let type_id = match &args[0] {
        Value::Literal(crate::ast::Literal::Number(n)) => *n as u64,
        _ => return Err(Box::new(Error::runtime_error(
            "make-record expects a numeric type ID as first argument".to_string(),
            None,
        )),
    };
    
    // Remaining arguments are field values
    let field_values = args[1..].to_vec();
    let record = make_record(type_id, field_values)?;
    Ok(Value::record(record))
}

/// record-type-id primitive - gets the type ID of a record.
/// Usage: (record-type-id record)
fn primitive_record_type_id(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(Error::runtime_error(
            format!("record-type-id expects 1 argument, got {}", args.len()),
            None,
        ));
    }
    
    match &args[0] {
        Value::Record(record) => Ok(Value::number(record.type_id as f64)),
        _ => Err(Box::new(Error::runtime_error(
            "record-type-id expects a record argument".to_string(),
            None,
        )),
    }
}

/// record-field-ref primitive - gets a field value by index.
/// Usage: (record-field-ref record index)
fn primitive_record_field_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(Error::runtime_error(
            format!("record-field-ref expects 2 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let record = match &args[0] {
        Value::Record(rec) => rec,
        _ => return Err(Box::new(Error::runtime_error(
            "record-field-ref expects a record as first argument".to_string(),
            None,
        )),
    };
    
    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::Number(n)) => *n as usize,
        _ => return Err(Box::new(Error::runtime_error(
            "record-field-ref expects a numeric index as second argument".to_string(),
            None,
        )),
    };
    
    let fields = record.fields.read().map_err(|_| {
        Error::runtime_error("Failed to acquire record field lock".to_string(), None)
    })?;
    
    fields.get(index).clone())().ok_or_else(|| {
        Error::runtime_error(
            format!("Record field index {} out of bounds", index),
            None,
        )
    })
}

/// record-field-set! primitive - sets a field value by index.
/// Usage: (record-field-set! record index value)
fn primitive_record_field_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(Box::new(Error::runtime_error(
            format!("record-field-set! expects 3 arguments, got {}", args.len()),
            None,
        ));
    }
    
    let record = match &args[0] {
        Value::Record(rec) => rec,
        _ => return Err(Box::new(Error::runtime_error(
            "record-field-set! expects a record as first argument".to_string(),
            None,
        )),
    };
    
    let index = match &args[1] {
        Value::Literal(crate::ast::Literal::Number(n)) => *n as usize,
        _ => return Err(Box::new(Error::runtime_error(
            "record-field-set! expects a numeric index as second argument".to_string(),
            None,
        )),
    };
    
    let new_value = args[2].clone());
    
    let mut fields = record.fields.write().map_err(|_| {
        Error::runtime_error("Failed to acquire record field lock".to_string(), None)
    })?;
    
    if index >= fields.len() {
        return Err(Box::new(Error::runtime_error(
            format!("Record field index {} out of bounds", index),
            None,
        ));
    }
    
    fields[index] = new_value;
    Ok(Value::Unspecified)
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

/// define-record-type-helper primitive - helps with macro expansion.
/// Usage: (define-record-type-helper type-name field-names constructor-name predicate-name [field-specs...])
fn primitive_define_record_type_helper(args: &[Value]) -> Result<Value> {
    if args.len() < 4 {
        return Err(Box::new(Error::runtime_error(
            "define-record-type-helper requires at least 4 arguments".to_string(),
            None,
        ));
    }
    
    // Extract type name
    let type_name = match &args[0] {
        Value::Literal(crate::ast::Literal::String(name)) => name.clone()),
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
            "Record type name must be a string or symbol".to_string(),
            None,
        )),
    };
    
    // Extract field names
    let field_names = match &args[1] {
        Value::Nil => Vec::new(),
        _ => {
            let field_list = args[1].as_list().ok_or_else(|| {
                Error::runtime_error(
                    "Field names must be a list".to_string(),
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
    let field_info: Vec<FieldInfo> = field_names.iter().enumerate().map(|(_i, name)| FieldInfo {
        name: name.clone()),
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
    
    // Return the type ID
    Ok(Value::number(type_id as f64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::symbol::intern_symbol;

    #[test]
    fn test_simple_record_creation() {
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
        
        let registered_id = register_record_type(record_type).unwrap();
        assert_eq!(registered_id, type_id);
        
        let field_values = vec![Value::integer(10), Value::integer(20)];
        let record = make_record(type_id, field_values).unwrap();
        
        assert_eq!(record.type_id, type_id);
    }
    
    #[test]
    fn test_record_predicate() {
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
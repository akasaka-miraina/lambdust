//! SRFI 150: Hygienic ERR5RS Record Syntax (reduced)
//!
//! This SRFI provides a specification and portable implementation of an extension 
//! of the ERR5RS record syntax, where field names inserted by macro transformers 
//! are effectively renamed as if the macro transformer inserted a binding.

use crate::error::{LambdustError, Result};
use crate::value::{Value, Procedure};
use std::collections::HashMap;

/// SRFI 150 module implementation
pub struct Srfi150Module;

impl crate::srfi::SrfiModule for Srfi150Module {
    fn srfi_id(&self) -> u32 {
        150
    }

    fn name(&self) -> &'static str {
        "SRFI 150"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["records"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Core record operations
        exports.insert("define-record-type".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "define-record-type".to_string(), 
            arity: None, 
            func: define_record_type_hygienic 
        }));
        
        // Hygienic field name utilities
        exports.insert("field-name-equal?".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "field-name-equal?".to_string(), 
            arity: Some(2), 
            func: field_name_equal_p 
        }));
        exports.insert("hygienic-field-name?".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "hygienic-field-name?".to_string(), 
            arity: Some(1), 
            func: hygienic_field_name_p 
        }));
        exports.insert("field-name-binding-equal?".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "field-name-binding-equal?".to_string(), 
            arity: Some(3), 
            func: field_name_binding_equal_p 
        }));
        
        // Record type inspection
        exports.insert("record-type?".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "record-type?".to_string(), 
            arity: Some(1), 
            func: record_type_p 
        }));
        exports.insert("record-type-name".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "record-type-name".to_string(), 
            arity: Some(1), 
            func: record_type_name 
        }));
        exports.insert("record-type-field-names".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "record-type-field-names".to_string(), 
            arity: Some(1), 
            func: record_type_field_names 
        }));
        exports.insert("record-type-parent".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "record-type-parent".to_string(), 
            arity: Some(1), 
            func: record_type_parent 
        }));
        
        // Record instance operations
        exports.insert("record?".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "record?".to_string(), 
            arity: None, 
            func: record_p 
        }));
        exports.insert("record-rtd".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "record-rtd".to_string(), 
            arity: Some(1), 
            func: record_rtd 
        }));
        
        // Field access and mutation
        exports.insert("record-ref".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "record-ref".to_string(), 
            arity: Some(2), 
            func: record_ref 
        }));
        exports.insert("record-set!".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "record-set!".to_string(), 
            arity: Some(3), 
            func: record_set 
        }));
        
        // Field name comparison and shadowing
        exports.insert("field-name-shadows?".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "field-name-shadows?".to_string(), 
            arity: Some(3), 
            func: field_name_shadows_p 
        }));
        exports.insert("resolve-field-name".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "resolve-field-name".to_string(), 
            arity: Some(2), 
            func: resolve_field_name 
        }));
        
        // Hygiene validation
        exports.insert("validate-field-hygiene".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "validate-field-hygiene".to_string(), 
            arity: Some(2), 
            func: validate_field_hygiene 
        }));
        exports.insert("macro-inserted-field?".to_string(), Value::Procedure(Procedure::Builtin { 
            name: "macro-inserted-field?".to_string(), 
            arity: Some(2), 
            func: macro_inserted_field_p 
        }));
        
        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // For now, return all exports regardless of parts
        Ok(self.exports())
    }
}

impl Srfi150Module {
    /// Creates a new SRFI-150 module instance
    pub fn new() -> Self {
        Self
    }
}

/// Internal record type representation
#[derive(Clone, Debug)]
pub struct RecordType {
    /// Record type name
    pub name: String,
    /// Field names in order
    pub field_names: Vec<String>,
    /// Parent record type (for inheritance)
    pub parent: Option<Box<RecordType>>,
    /// Whether the record type is sealed
    pub sealed: bool,
    /// Whether the record type is opaque
    pub opaque: bool,
    /// Constructor specification
    pub constructor_spec: Option<String>,
    /// Predicate function name
    pub predicate_name: Option<String>,
}

impl RecordType {
    /// Creates a new record type with the given name and field names
    pub fn new(name: String, field_names: Vec<String>) -> Self {
        Self {
            name,
            field_names,
            parent: None,
            sealed: false,
            opaque: false,
            constructor_spec: None,
            predicate_name: None,
        }
    }
    
    /// Sets the parent record type for inheritance
    pub fn with_parent(mut self, parent: RecordType) -> Self {
        self.parent = Some(Box::new(parent));
        self
    }
    
    /// Returns all field names including those from parent record types
    pub fn all_field_names(&self) -> Vec<String> {
        let mut fields = Vec::new();
        
        // Add parent fields first
        if let Some(parent) = &self.parent {
            fields.extend(parent.all_field_names());
        }
        
        // Add own fields (may shadow parent fields)
        fields.extend(self.field_names.clone());
        fields
    }
}

/// Internal record instance representation
#[derive(Clone, Debug)]
pub struct Record {
    /// The record type definition
    pub record_type: RecordType,
    /// Field values by field name
    pub fields: HashMap<String, Value>,
}

impl Record {
    /// Creates a new record instance with the given record type
    pub fn new(record_type: RecordType) -> Self {
        let mut fields = HashMap::new();
        
        // Initialize all fields to #f
        for field_name in record_type.all_field_names() {
            fields.insert(field_name, Value::Boolean(false));
        }
        
        Self {
            record_type,
            fields,
        }
    }
}

/// Hygienic define-record-type
fn define_record_type_hygienic(args: &[Value]) -> Result<Value> {
    if args.len() < 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }
    
    // Parse record type name
    let type_name = extract_symbol(&args[0])?;
    
    // Parse constructor specification
    let _constructor_spec = &args[1];
    
    // Parse predicate name  
    let _predicate_name = extract_symbol(&args[2])?;
    
    // Parse field specifications
    let field_specs = &args[3..];
    let mut field_names = Vec::new();
    
    for spec in field_specs {
        match spec {
            Value::Symbol(name) => {
                field_names.push(name.clone());
            }
            Value::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                if let Value::Symbol(name) = &pair.car {
                    field_names.push(name.clone());
                }
            }
            _ => return Err(LambdustError::syntax_error("invalid field specification".to_string())),
        }
    }
    
    // Create record type with hygienic field names
    let record_type = RecordType::new(type_name, field_names);
    
    // In a real implementation, this would:
    // 1. Check field name hygiene according to SRFI 150 rules
    // 2. Generate constructor, predicate, and accessor procedures
    // 3. Register the record type in the environment
    
    Ok(Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "RecordType".to_string(),
        data: std::sync::Arc::new(record_type),
    }))
}

/// Test if two field names are equal under hygienic semantics
fn field_name_equal_p(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let name1 = &args[0];
    let name2 = &args[1];
    
    // In SRFI 150, two identifiers appearing in the same instance of 
    // define-record-type are considered equal if binding one identifier 
    // would bind the other identifier
    let is_equal = hygienic_identifier_equal(name1, name2);
    Ok(Value::Boolean(is_equal))
}

/// Check if a field name is hygienic
fn hygienic_field_name_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let _field_name = &args[0];
    
    // In this implementation, we consider all field names hygienic
    // In a real implementation, this would check if the field name
    // maintains proper binding relationships
    Ok(Value::Boolean(true))
}

/// Test if two field names have equal bindings
fn field_name_binding_equal_p(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }
    
    let _name1 = &args[0];
    let _name2 = &args[1];
    let _context = &args[2];
    
    // This would check if two field names have equal bindings in the given context
    Ok(Value::Boolean(false))
}

/// Check if value is a record type
fn record_type_p(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let is_record_type = extract_record_type(&args[0]).is_ok();
    Ok(Value::Boolean(is_record_type))
}

/// Get record type name
fn record_type_name(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let record_type = extract_record_type(&args[0])?;
    Ok(Value::Symbol(record_type.name.clone()))
}

/// Get record type field names
fn record_type_field_names(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let record_type = extract_record_type(&args[0])?;
    let field_names: Vec<Value> = record_type.all_field_names()
        .into_iter()
        .map(Value::Symbol)
        .collect();
    Ok(Value::from_vector(field_names))
}

/// Get record type parent
fn record_type_parent(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let record_type = extract_record_type(&args[0])?;
    match &record_type.parent {
        Some(parent) => Ok(Value::External(crate::bridge::ExternalObject {
            id: 0, // Would be assigned by registry
            type_name: "RecordType".to_string(),
            data: std::sync::Arc::new(parent.as_ref().clone()),
        })),
        None => Ok(Value::Boolean(false)),
    }
}

/// Check if value is a record instance
fn record_p(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(LambdustError::arity_error(1, 0));
    }
    
    if args.len() == 1 {
        // Check if value is any record
        let is_record = extract_record(&args[0]).is_ok();
        Ok(Value::Boolean(is_record))
    } else {
        // Check if value is a record of specific type
        let _record = extract_record(&args[0])?;
        let _record_type = extract_record_type(&args[1])?;
        
        // Would check if record is instance of given type
        Ok(Value::Boolean(true))
    }
}

/// Get record type descriptor from record instance
fn record_rtd(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(LambdustError::arity_error(1, args.len()));
    }
    
    let record = extract_record(&args[0])?;
    Ok(Value::External(crate::bridge::ExternalObject {
        id: 0, // Would be assigned by registry
        type_name: "RecordType".to_string(),
        data: std::sync::Arc::new(record.record_type.clone()),
    }))
}

/// Get field value from record
fn record_ref(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let record = extract_record(&args[0])?;
    let field_name = extract_symbol(&args[1])?;
    
    match record.fields.get(&field_name) {
        Some(value) => Ok(value.clone()),
        None => Err(LambdustError::runtime_error(format!("field not found: {}", field_name))),
    }
}

/// Set field value in record
fn record_set(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }
    
    let _record = extract_record(&args[0])?;
    let _field_name = extract_symbol(&args[1])?;
    let _value = &args[2];
    
    // In a mutable implementation, this would update the field
    // For now, return undefined
    Ok(Value::Undefined)
}

/// Check if field name shadows parent field
fn field_name_shadows_p(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(LambdustError::arity_error(3, args.len()));
    }
    
    let _child_field = &args[0];
    let _parent_field = &args[1];
    let _record_type = &args[2];
    
    // This would check if a field in a child record type shadows
    // a field with the same name in the parent
    Ok(Value::Boolean(false))
}

/// Resolve field name in record type hierarchy
fn resolve_field_name(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let field_name = extract_symbol(&args[0])?;
    let record_type = extract_record_type(&args[1])?;
    
    // Check if field exists in record type (including inherited fields)
    if record_type.all_field_names().contains(&field_name) {
        Ok(Value::Symbol(field_name))
    } else {
        Err(LambdustError::runtime_error(format!("field not found: {}", field_name)))
    }
}

/// Validate field hygiene in record definition
fn validate_field_hygiene(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let _field_names = &args[0];
    let _context = &args[1];
    
    // This would validate that field names maintain proper hygiene
    // according to SRFI 150 rules
    Ok(Value::Boolean(true))
}

/// Check if field was inserted by macro transformation
fn macro_inserted_field_p(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(LambdustError::arity_error(2, args.len()));
    }
    
    let _field_name = &args[0];
    let _context = &args[1];
    
    // This would check if a field name was inserted by a macro transformer
    Ok(Value::Boolean(false))
}

/// Helper function to check hygienic identifier equality
fn hygienic_identifier_equal(id1: &Value, id2: &Value) -> bool {
    match (id1, id2) {
        (Value::Symbol(s1), Value::Symbol(s2)) => {
            // In a full implementation, this would check if binding one
            // identifier would bind the other (hygienic equality)
            s1 == s2
        }
        _ => false,
    }
}

/// Helper function to extract symbol from Value
fn extract_symbol(value: &Value) -> Result<String> {
    match value {
        Value::Symbol(s) => Ok(s.clone()),
        _ => Err(LambdustError::type_error("expected symbol".to_string())),
    }
}

/// Helper function to extract record type from external object
fn extract_record_type(value: &Value) -> Result<&RecordType> {
    match value {
        Value::External(obj) => {
            obj.data.downcast_ref::<RecordType>()
                .ok_or_else(|| LambdustError::type_error("expected record type".to_string()))
        }
        _ => Err(LambdustError::type_error("expected record type".to_string())),
    }
}

/// Helper function to extract record from external object
fn extract_record(value: &Value) -> Result<&Record> {
    match value {
        Value::External(obj) => {
            obj.data.downcast_ref::<Record>()
                .ok_or_else(|| LambdustError::type_error("expected record".to_string()))
        }
        _ => Err(LambdustError::type_error("expected record".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
        use crate::srfi::SrfiModule;
    use std::sync::Arc;

    #[test]
    fn test_record_type_creation() {
        let record_type = RecordType::new(
            "person".to_string(),
            vec!["name".to_string(), "age".to_string()]
        );
        
        assert_eq!(record_type.name, "person");
        assert_eq!(record_type.field_names.len(), 2);
        assert!(record_type.field_names.contains(&"name".to_string()));
        assert!(record_type.field_names.contains(&"age".to_string()));
    }

    #[test]
    fn test_record_type_with_parent() {
        let parent = RecordType::new(
            "entity".to_string(),
            vec!["id".to_string()]
        );
        
        let child = RecordType::new(
            "person".to_string(),
            vec!["name".to_string()]
        ).with_parent(parent);
        
        let all_fields = child.all_field_names();
        assert_eq!(all_fields.len(), 2);
        assert!(all_fields.contains(&"id".to_string()));
        assert!(all_fields.contains(&"name".to_string()));
    }

    #[test]
    fn test_field_name_equal() {
        
        
        let name1 = Value::Symbol("field".to_string());
        let name2 = Value::Symbol("field".to_string());
        let name3 = Value::Symbol("other".to_string());
        
        let result = field_name_equal_p(&[name1.clone(), name2]).unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        let result = field_name_equal_p(&[name1, name3]).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_hygienic_field_name() {
        
        
        let field_name = Value::Symbol("field".to_string());
        let result = hygienic_field_name_p(&[field_name]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_record_type_predicates() {
        
        
        let record_type = RecordType::new(
            "test".to_string(),
            vec!["field".to_string()]
        );
        let rt_value = Value::External(crate::bridge::ExternalObject {
            id: 0, // Would be assigned by registry
            type_name: "RecordType".to_string(),
            data: std::sync::Arc::new(record_type),
        });
        
        // Test record-type?
        let result = record_type_p(&[rt_value.clone()]).unwrap();
        assert_eq!(result, Value::Boolean(true));
        
        // Test record-type-name
        let result = record_type_name(&[rt_value.clone()]).unwrap();
        assert_eq!(result, Value::Symbol("test".to_string()));
        
        // Test record-type-field-names
        let result = record_type_field_names(&[rt_value]).unwrap();
        let fields = result.to_vector().unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], Value::Symbol("field".to_string()));
    }

    #[test]
    fn test_define_record_type() {
        
        
        let type_name = Value::Symbol("person".to_string());
        let constructor = Value::Symbol("make-person".to_string());
        let predicate = Value::Symbol("person?".to_string());
        let field1 = Value::Symbol("name".to_string());
        let field2 = Value::Symbol("age".to_string());
        
        let result = define_record_type_hygienic(&[
            type_name, constructor, predicate, field1, field2
        ]).unwrap();
        
        // Should return a record type
        assert!(extract_record_type(&result).is_ok());
    }

    #[test]
    fn test_srfi_150_module() {
        let module = Srfi150Module::new();
        assert_eq!(module.srfi_id(), 150);
        assert_eq!(module.name(), "SRFI 150");
        assert_eq!(module.parts(), vec!["records"]);
        
        let exports = module.exports();
        assert!(exports.contains_key("define-record-type"));
        assert!(exports.contains_key("field-name-equal?"));
        assert!(exports.contains_key("record-type?"));
        assert!(exports.contains_key("record?"));
        
        // Test exports_for_parts
        let partial_exports = module.exports_for_parts(&["records"]).unwrap();
        assert_eq!(partial_exports.len(), exports.len());
    }
}
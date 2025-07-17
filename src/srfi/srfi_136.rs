//! SRFI 136: Extensible Record Types
//!
//! This SRFI provides conservative extensions to the record type facility
//! specified in SRFI 9 and R7RS. The key addition is the ability to create
//! subtypes of existing record types, and to provide runtime record-type descriptors.

use crate::bridge::ExternalObject;
use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::{Record, RecordType, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Runtime record-type descriptor for SRFI 136
#[derive(Debug, Clone, PartialEq)]
pub struct RecordTypeDescriptor {
    /// Type name as symbol
    pub name: String,
    /// Parent record type descriptor (if any)
    pub parent: Option<Arc<RecordTypeDescriptor>>,
    /// Field specifications
    pub fields: Vec<FieldSpec>,
    /// Whether this type is sealed (cannot be subtyped)
    pub sealed: bool,
    /// Unique type identifier
    pub type_id: usize,
}

/// Field specification for record types
#[derive(Debug, Clone, PartialEq)]
pub struct FieldSpec {
    /// Field name
    pub name: String,
    /// Whether field is mutable
    pub mutable: bool,
    /// Default value (simplified for thread safety)
    pub default: Option<String>,
}

impl RecordTypeDescriptor {
    /// Create a new record type descriptor
    pub fn new(
        name: String,
        fields: Vec<FieldSpec>,
        parent: Option<Arc<RecordTypeDescriptor>>,
    ) -> Self {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT_TYPE_ID: AtomicUsize = AtomicUsize::new(1);
        let type_id = NEXT_TYPE_ID.fetch_add(1, Ordering::SeqCst);

        RecordTypeDescriptor {
            name,
            parent,
            fields,
            sealed: false,
            type_id,
        }
    }

    /// Get all fields including inherited ones
    #[must_use] pub fn all_fields(&self) -> Vec<FieldSpec> {
        let mut all_fields = Vec::new();

        // Add parent fields first
        if let Some(parent) = &self.parent {
            all_fields.extend(parent.all_fields());
        }

        // Add own fields
        all_fields.extend(self.fields.clone());

        all_fields
    }

    /// Check if this type is a subtype of another
    #[must_use] pub fn is_subtype_of(&self, other: &RecordTypeDescriptor) -> bool {
        if self.type_id == other.type_id {
            return true;
        }

        if let Some(parent) = &self.parent {
            parent.is_subtype_of(other)
        } else {
            false
        }
    }

    /// Get field count including inherited fields
    #[must_use] pub fn field_count(&self) -> usize {
        self.all_fields().len()
    }
}

/// Extended record type that includes runtime descriptor
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedRecordType {
    /// Legacy record type for compatibility
    pub legacy_type: RecordType,
    /// Runtime type descriptor
    pub descriptor: Arc<RecordTypeDescriptor>,
}

/// Extended record instance
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedRecord {
    /// Record type descriptor
    pub type_descriptor: Arc<RecordTypeDescriptor>,
    /// Field values (including inherited fields)
    pub fields: Vec<Value>,
}

/// SRFI 136 implementation
pub struct Srfi136;

impl super::SrfiModule for Srfi136 {
    fn srfi_id(&self) -> u32 {
        136
    }

    fn name(&self) -> &'static str {
        "Extensible Record Types"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec![]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Predicates
        exports.insert(
            "record?".to_string(),
            make_builtin_procedure("record?", Some(1), |args| {
                check_arity(args, 1)?;
                Ok(Value::Boolean(matches!(args[0], Value::Record(_))))
            }),
        );

        exports.insert(
            "record-type-descriptor?".to_string(),
            make_builtin_procedure("record-type-descriptor?", Some(1), |args| {
                check_arity(args, 1)?;
                Ok(Value::Boolean(is_record_type_descriptor(&args[0])))
            }),
        );

        // Runtime introspection
        exports.insert(
            "record-type-descriptor".to_string(),
            make_builtin_procedure("record-type-descriptor", Some(1), |args| {
                check_arity(args, 1)?;
                get_record_type_descriptor(&args[0])
            }),
        );

        exports.insert(
            "record-type-name".to_string(),
            make_builtin_procedure("record-type-name", Some(1), |args| {
                check_arity(args, 1)?;
                get_record_type_name(&args[0])
            }),
        );

        exports.insert(
            "record-type-parent".to_string(),
            make_builtin_procedure("record-type-parent", Some(1), |args| {
                check_arity(args, 1)?;
                get_record_type_parent(&args[0])
            }),
        );

        exports.insert(
            "record-type-fields".to_string(),
            make_builtin_procedure("record-type-fields", Some(1), |args| {
                check_arity(args, 1)?;
                get_record_type_fields(&args[0])
            }),
        );

        // Constructor
        exports.insert(
            "make-record-type-descriptor".to_string(),
            make_builtin_procedure("make-record-type-descriptor", None, |args| {
                if args.len() < 2 || args.len() > 3 {
                    return Err(LambdustError::arity_error(2, args.len()));
                }
                make_record_type_descriptor_proc(args)
            }),
        );

        exports.insert(
            "make-record".to_string(),
            make_builtin_procedure("make-record", Some(2), |args| {
                check_arity(args, 2)?;
                make_record_from_descriptor(&args[0], &args[1])
            }),
        );

        exports
    }

    fn exports_for_parts(&self, _parts: &[&str]) -> Result<HashMap<String, Value>> {
        // SRFI 136 has no parts
        Ok(self.exports())
    }
}

/// Generate unique ID for external objects
fn generate_external_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

/// Check if value is a record type descriptor
fn is_record_type_descriptor(value: &Value) -> bool {
    // For now, we'll use a special external type
    matches!(value, Value::External(ext) if ext.type_name == "RecordTypeDescriptor")
}

/// Get record type descriptor from a record
fn get_record_type_descriptor(value: &Value) -> Result<Value> {
    match value {
        Value::Record(record) => {
            // Convert legacy record type to descriptor
            let descriptor = Arc::new(RecordTypeDescriptor::new(
                record.record_type.name.clone(),
                record
                    .record_type
                    .field_names
                    .iter()
                    .map(|name| FieldSpec {
                        name: name.clone(),
                        mutable: true, // Legacy records are mutable by default
                        default: None,
                    })
                    .collect(),
                None, // No parent for legacy records
            ));

            Ok(Value::External(ExternalObject {
                id: generate_external_id(),
                type_name: "RecordTypeDescriptor".to_string(),
                data: descriptor,
            }))
        }
        _ => Err(LambdustError::type_error("Expected record".to_string())),
    }
}

/// Get record type name
fn get_record_type_name(value: &Value) -> Result<Value> {
    match value {
        Value::External(ext) if ext.type_name == "RecordTypeDescriptor" => {
            if let Some(descriptor) = ext.data.downcast_ref::<RecordTypeDescriptor>() {
                Ok(Value::Symbol(descriptor.name.clone()))
            } else {
                Err(LambdustError::type_error(
                    "Invalid record type descriptor".to_string(),
                ))
            }
        }
        _ => Err(LambdustError::type_error(
            "Expected record type descriptor".to_string(),
        )),
    }
}

/// Get record type parent
fn get_record_type_parent(value: &Value) -> Result<Value> {
    match value {
        Value::External(ext) if ext.type_name == "RecordTypeDescriptor" => {
            if let Some(descriptor) = ext.data.downcast_ref::<RecordTypeDescriptor>() {
                if let Some(parent) = &descriptor.parent {
                    Ok(Value::External(ExternalObject {
                        id: generate_external_id(),
                        type_name: "RecordTypeDescriptor".to_string(),
                        data: Arc::new((**parent).clone()),
                    }))
                } else {
                    Ok(Value::Boolean(false))
                }
            } else {
                Err(LambdustError::type_error(
                    "Invalid record type descriptor".to_string(),
                ))
            }
        }
        _ => Err(LambdustError::type_error(
            "Expected record type descriptor".to_string(),
        )),
    }
}

/// Get record type fields
fn get_record_type_fields(value: &Value) -> Result<Value> {
    match value {
        Value::External(ext) if ext.type_name == "RecordTypeDescriptor" => {
            if let Some(descriptor) = ext.data.downcast_ref::<RecordTypeDescriptor>() {
                let fields: Vec<Value> = descriptor
                    .fields
                    .iter()
                    .map(|field| Value::Symbol(field.name.clone()))
                    .collect();
                Ok(Value::Vector(fields))
            } else {
                Err(LambdustError::type_error(
                    "Invalid record type descriptor".to_string(),
                ))
            }
        }
        _ => Err(LambdustError::type_error(
            "Expected record type descriptor".to_string(),
        )),
    }
}

/// Create record type descriptor
fn make_record_type_descriptor_proc(args: &[Value]) -> Result<Value> {
    let name = match &args[0] {
        Value::Symbol(s) => s.clone(),
        Value::String(s) => s.clone(),
        _ => {
            return Err(LambdustError::type_error(
                "Expected symbol or string for name".to_string(),
            ));
        }
    };

    // Convert field specifications to vector if needed
    let field_specs = match &args[1] {
        Value::Vector(specs) => specs.clone(),
        // Handle quoted vectors that become lists
        Value::Pair(_) => {
            if let Some(vec) = args[1].to_vector() {
                vec
            } else {
                return Err(LambdustError::type_error(
                    "Expected vector for field specifications".to_string(),
                ));
            }
        }
        _ => {
            return Err(LambdustError::type_error(
                "Expected vector for field specifications".to_string(),
            ));
        }
    };

    let fields = {
        let mut fields = Vec::new();
        for spec in &field_specs {
            match spec {
                Value::Symbol(field_name) => {
                    fields.push(FieldSpec {
                        name: field_name.clone(),
                        mutable: true,
                        default: None,
                    });
                }
                Value::Vector(field_spec_vec) if !field_spec_vec.is_empty() => {
                    if let Value::Symbol(field_name) = &field_spec_vec[0] {
                        fields.push(FieldSpec {
                            name: field_name.clone(),
                            mutable: true, // Simplified for now
                            default: None,
                        });
                    } else {
                        return Err(LambdustError::type_error(
                            "Expected symbol for field name".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(LambdustError::type_error(
                        "Invalid field specification".to_string(),
                    ));
                }
            }
        }
        fields
    };

    let parent = if args.len() > 2 {
        match &args[2] {
            Value::External(ext) if ext.type_name == "RecordTypeDescriptor" => {
                if let Some(parent_desc) = ext.data.downcast_ref::<RecordTypeDescriptor>() {
                    Some(Arc::new(parent_desc.clone()))
                } else {
                    return Err(LambdustError::type_error(
                        "Invalid parent record type descriptor".to_string(),
                    ));
                }
            }
            Value::Boolean(false) => None,
            _ => {
                return Err(LambdustError::type_error(
                    "Expected record type descriptor or #f for parent".to_string(),
                ));
            }
        }
    } else {
        None
    };

    let descriptor = RecordTypeDescriptor::new(name, fields, parent);

    Ok(Value::External(ExternalObject {
        id: generate_external_id(),
        type_name: "RecordTypeDescriptor".to_string(),
        data: Arc::new(descriptor),
    }))
}

/// Create record from descriptor and field vector
fn make_record_from_descriptor(rtd_value: &Value, fields_value: &Value) -> Result<Value> {
    let descriptor = match rtd_value {
        Value::External(ext) if ext.type_name == "RecordTypeDescriptor" => {
            if let Some(desc) = ext.data.downcast_ref::<RecordTypeDescriptor>() {
                Arc::new(desc.clone())
            } else {
                return Err(LambdustError::type_error(
                    "Invalid record type descriptor".to_string(),
                ));
            }
        }
        _ => {
            return Err(LambdustError::type_error(
                "Expected record type descriptor".to_string(),
            ));
        }
    };

    // Convert field values to vector if needed
    let fields = match fields_value {
        Value::Vector(field_values) => field_values.clone(),
        // Handle quoted vectors that become lists
        Value::Pair(_) => {
            if let Some(vec) = fields_value.to_vector() {
                vec
            } else {
                return Err(LambdustError::type_error(
                    "Expected vector for field values".to_string(),
                ));
            }
        }
        _ => {
            return Err(LambdustError::type_error(
                "Expected vector for field values".to_string(),
            ));
        }
    };

    let all_fields = descriptor.all_fields();
    if fields.len() != all_fields.len() {
        return Err(LambdustError::runtime_error(format!(
            "Field count mismatch: expected {}, got {}",
            all_fields.len(),
            fields.len()
        )));
    }

    // Create a legacy RecordType for compatibility
    let legacy_type = crate::value::RecordType {
        name: descriptor.name.clone(),
        field_names: all_fields.iter().map(|f| f.name.clone()).collect(),
        constructor_name: format!("make-{}", descriptor.name),
        predicate_name: format!("{}?", descriptor.name),
    };

    Ok(Value::Record(Record {
        record_type: legacy_type,
        fields,
    }))
}


//! SRFI 9: Defining Record Types
//!
//! This SRFI provides a syntax for defining record types with named fields.

use super::SrfiModule;
use crate::error::{LambdustError, Result};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// SRFI 9 implementation
pub struct Srfi9;

impl SrfiModule for Srfi9 {
    fn srfi_id(&self) -> u32 {
        9
    }
    
    fn name(&self) -> &'static str {
        "Defining Record Types"
    }
    
    fn parts(&self) -> Vec<&'static str> {
        vec!["records", "types"]
    }
    
    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Record operations (from existing builtins/misc.rs)
        exports.insert("make-record".to_string(), record_make());
        exports.insert("record-of-type?".to_string(), record_predicate());
        exports.insert("record-field".to_string(), record_field_get());
        exports.insert("record-set-field!".to_string(), record_field_set());
        
        exports
    }
    
    fn exports_for_parts(&self, parts: &[&str]) -> Result<HashMap<String, Value>> {
        let all_exports = self.exports();
        let mut filtered = HashMap::new();
        
        for part in parts {
            match *part {
                "records" => {
                    // All record operations
                    for (name, value) in &all_exports {
                        filtered.insert(name.clone(), value.clone());
                    }
                }
                "types" => {
                    // Type-related operations (subset)
                    if let Some(value) = all_exports.get("record-of-type?") {
                        filtered.insert("record-of-type?".to_string(), value.clone());
                    }
                }
                _ => {
                    return Err(LambdustError::runtime_error(
                        format!("Unknown SRFI 9 part: {}", part)
                    ));
                }
            }
        }
        
        Ok(filtered)
    }
}

// Record operation implementations (migrated from builtins/misc.rs)

/// Creates a record instance
fn record_make() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "make-record".to_string(),
        arity: None, // Variadic
        func: |args| {
            if args.len() < 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            // First argument should be the record type (as a symbol/string)
            let type_name = match &args[0] {
                Value::Symbol(s) => s.clone(),
                Value::String(s) => s.clone(),
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "make-record: expected type name as symbol or string, got {}",
                        args[0]
                    )));
                }
            };

            // Remaining arguments are field values
            let field_values = args[1..].to_vec();

            // Create a basic record type and record
            let record_type = crate::value::RecordType {
                name: type_name,
                field_names: (0..field_values.len())
                    .map(|i| format!("field{}", i))
                    .collect(),
                constructor_name: "make-record".to_string(),
                predicate_name: "record?".to_string(),
            };

            let record = crate::value::Record {
                record_type,
                fields: field_values,
            };

            Ok(Value::Record(record))
        },
    })
}

/// Checks if a value is a record of a specific type
fn record_predicate() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "record-of-type?".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            let type_name = match &args[1] {
                Value::Symbol(s) => s.clone(),
                Value::String(s) => s.clone(),
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "record-of-type?: expected type name as symbol or string, got {}",
                        args[1]
                    )));
                }
            };

            Ok(Value::Boolean(args[0].is_record_of_type(&type_name)))
        },
    })
}

/// Gets a field value from a record
fn record_field_get() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "record-field".to_string(),
        arity: Some(2),
        func: |args| {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            let record = match args[0].as_record() {
                Some(r) => r,
                None => {
                    return Err(LambdustError::type_error(format!(
                        "record-field: expected record, got {}",
                        args[0]
                    )));
                }
            };

            let index = match args[1].as_number() {
                Some(crate::lexer::SchemeNumber::Integer(i)) if *i >= 0 => *i as usize,
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "record-field: expected non-negative integer index, got {}",
                        args[1]
                    )));
                }
            };

            if index >= record.fields.len() {
                return Err(LambdustError::runtime_error(format!(
                    "record-field: index {} out of bounds for record with {} fields",
                    index,
                    record.fields.len()
                )));
            }

            Ok(record.fields[index].clone())
        },
    })
}

/// Sets a field value in a record (returns new record)
fn record_field_set() -> Value {
    Value::Procedure(Procedure::Builtin {
        name: "record-set-field!".to_string(),
        arity: Some(3),
        func: |args| {
            if args.len() != 3 {
                return Err(LambdustError::arity_error(3, args.len()));
            }

            let record = match args[0].as_record() {
                Some(r) => r,
                None => {
                    return Err(LambdustError::type_error(format!(
                        "record-set-field!: expected record, got {}",
                        args[0]
                    )));
                }
            };

            let index = match args[1].as_number() {
                Some(crate::lexer::SchemeNumber::Integer(i)) if *i >= 0 => *i as usize,
                _ => {
                    return Err(LambdustError::type_error(format!(
                        "record-set-field!: expected non-negative integer index, got {}",
                        args[1]
                    )));
                }
            };

            if index >= record.fields.len() {
                return Err(LambdustError::runtime_error(format!(
                    "record-set-field!: index {} out of bounds for record with {} fields",
                    index,
                    record.fields.len()
                )));
            }

            // Create new record with updated field
            let mut new_fields = record.fields.clone();
            new_fields[index] = args[2].clone();

            let new_record = crate::value::Record {
                record_type: record.record_type.clone(),
                fields: new_fields,
            };

            Ok(Value::Record(new_record))
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::SchemeNumber;
    
    #[test]
    fn test_srfi_9_info() {
        let srfi9 = Srfi9;
        assert_eq!(srfi9.srfi_id(), 9);
        assert_eq!(srfi9.name(), "Defining Record Types");
        assert!(srfi9.parts().contains(&"records"));
        assert!(srfi9.parts().contains(&"types"));
    }
    
    #[test]
    fn test_srfi_9_exports() {
        let srfi9 = Srfi9;
        let exports = srfi9.exports();
        
        assert!(exports.contains_key("make-record"));
        assert!(exports.contains_key("record-of-type?"));
        assert!(exports.contains_key("record-field"));
        assert!(exports.contains_key("record-set-field!"));
    }
    
    #[test]
    fn test_record_operations() {
        // Test make-record
        let make_record = record_make();
        if let Value::Procedure(Procedure::Builtin { func, .. }) = make_record {
            let args = vec![
                Value::Symbol("person".to_string()),
                Value::String("Alice".to_string()),
                Value::Number(SchemeNumber::Integer(30)),
            ];
            let result = func(&args).unwrap();
            assert!(matches!(result, Value::Record(_)));
        }
    }
}
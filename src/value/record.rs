//! Record types (SRFI 9)

use super::Value;

/// Record type representation (SRFI 9)
#[derive(Debug, Clone, PartialEq)]
pub struct RecordType {
    /// Type name
    pub name: String,
    /// Field names in order
    pub field_names: Vec<String>,
    /// Constructor name
    pub constructor_name: String,
    /// Predicate name
    pub predicate_name: String,
}

/// Record instance (SRFI 9)
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// Record type information
    pub record_type: RecordType,
    /// Field values
    pub fields: Vec<Value>,
}

impl Value {
    /// Check if this value is a record
    pub fn is_record(&self) -> bool {
        matches!(self, Value::Record(_))
    }

    /// Get the record if this is a record
    pub fn as_record(&self) -> Option<&Record> {
        match self {
            Value::Record(r) => Some(r),
            _ => None,
        }
    }

    /// Check if this is a record of a specific type
    pub fn is_record_of_type(&self, type_name: &str) -> bool {
        match self {
            Value::Record(record) => record.record_type.name == type_name,
            _ => false,
        }
    }
}
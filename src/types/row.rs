use super::{Type, TypeVar};
use std::collections::HashMap;

/// Row for record and variant types (row polymorphism).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Fields in the row
    pub fields: HashMap<String, Type>,
    /// Row variable for extension
    pub rest: Option<TypeVar>,
}

impl Row {
    /// Creates an empty row.
    pub fn empty() -> Self {
        Self {
            fields: HashMap::new(),
            rest: None,
        }
    }
    
    /// Creates a closed row with the given fields.
    pub fn closed(fields: HashMap<String, Type>) -> Self {
        Self {
            fields,
            rest: None,
        }
    }
    
    /// Creates an open row with the given fields and rest variable.
    pub fn open(fields: HashMap<String, Type>, rest: TypeVar) -> Self {
        Self {
            fields,
            rest: Some(rest),
        }
    }
    
    /// Extends this row with a new field.
    pub fn extend(&mut self, name: String, type_: Type) {
        self.fields.insert(name, type_);
    }
    
    /// Returns true if this row is closed (no rest variable).
    pub fn is_closed(&self) -> bool {
        self.rest.is_none()
    }
}

impl std::hash::Hash for Row {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the sorted field names and types
        let mut sorted_fields: Vec<_> = self.fields.iter().collect();
        sorted_fields.sort_by_key(|(k, _)| *k);
        for (key, ty) in sorted_fields {
            key.hash(state);
            ty.hash(state);
        }
        self.rest.hash(state);
    }
}
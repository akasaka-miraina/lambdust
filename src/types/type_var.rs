use std::sync::atomic::{AtomicU64, Ordering};
use std::fmt;

/// Global type variable counter for generating unique type variables.
static TYPE_VAR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generates a unique type variable ID.
fn next_type_var_id() -> u64 {
    TYPE_VAR_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Type variable with unique identifier and optional name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar {
    pub id: u64,
    pub name: Option<String>,
}

impl Default for TypeVar {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeVar {
    /// Creates a new type variable with a unique ID.
    pub fn new() -> Self {
        Self {
            id: next_type_var_id(),
            name: None,
        }
    }
    
    /// Creates a new type variable with a name.
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            id: next_type_var_id(),
            name: Some(name.into()),
        }
    }
    
    /// Creates a type variable with a specific ID (for testing).
    pub fn with_id(id: u64) -> Self {
        Self { id, name: None }
    }
}

impl fmt::Display for TypeVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "{name}")
        } else {
            write!(f, "t{}", self.id)
        }
    }
}
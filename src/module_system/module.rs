use super::{ModuleId, ModuleMetadata};
use crate::eval::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// A loaded and compiled module ready for use.
#[derive(Debug, Clone)]
pub struct Module {
    /// Unique identifier for this module
    pub id: ModuleId,
    /// Exported symbols and their values
    pub exports: HashMap<String, Value>,
    /// Dependencies that this module imports
    pub dependencies: Vec<ModuleId>,
    /// Optional source location for debugging
    pub source: Option<ModuleSource>,
    /// Module metadata
    pub metadata: ModuleMetadata,
}

/// Source information for a module.
#[derive(Debug, Clone)]
pub enum ModuleSource {
    /// Built-in module (no source file)
    Builtin,
    /// Module loaded from a file
    File(PathBuf),
    /// Module compiled from source code
    Source(String),
}
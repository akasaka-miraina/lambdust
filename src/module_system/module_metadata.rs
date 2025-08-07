use std::collections::HashMap;

/// Metadata about a module.
#[derive(Debug, Clone, Default)]
pub struct ModuleMetadata {
    /// Module version (if specified)
    pub version: Option<String>,
    /// Module description
    pub description: Option<String>,
    /// Module author(s)
    pub authors: Vec<String>,
    /// Additional metadata fields
    pub extra: HashMap<String, String>,
}
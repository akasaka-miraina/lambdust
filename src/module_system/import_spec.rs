use super::ModuleId;
use std::collections::HashMap;

/// Import specification for bringing symbols into scope.
#[derive(Debug, Clone)]
pub struct ImportSpec {
    /// The module to import from
    pub module_id: ModuleId,
    /// Import configuration
    pub config: ImportConfig,
}

/// Configuration for how symbols are imported.
#[derive(Debug, Clone)]
pub enum ImportConfig {
    /// Import all exported symbols
    All,
    /// Import only specified symbols
    Only(Vec<String>),
    /// Import all except specified symbols
    Except(Vec<String>),
    /// Rename imported symbols
    Rename(HashMap<String, String>),
    /// Add prefix to all imported symbols
    Prefix(String),
}
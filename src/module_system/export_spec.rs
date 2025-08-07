use std::collections::HashMap;

/// Export specification for making symbols available to other modules.
#[derive(Debug, Clone)]
pub struct ExportSpec {
    /// Symbols to export
    pub symbols: Vec<String>,
    /// Export configuration
    pub config: ExportConfig,
}

/// Configuration for how symbols are exported.
#[derive(Debug, Clone)]
pub enum ExportConfig {
    /// Export symbols as-is
    Direct,
    /// Export with renaming
    Rename(HashMap<String, String>),
}
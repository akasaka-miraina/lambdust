//! Configuration for parser behavior.

/// Configuration for parser behavior
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Maximum number of errors to collect before stopping
    pub max_errors: usize,
    /// Whether to use aggressive error recovery
    pub aggressive_recovery: bool,
    /// Whether to preserve whitespace in AST
    pub preserve_whitespace: bool,
    /// Whether to allow incomplete expressions
    pub allow_incomplete: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            max_errors: 10,
            aggressive_recovery: true,
            preserve_whitespace: false,
            allow_incomplete: false,
        }
    }
}
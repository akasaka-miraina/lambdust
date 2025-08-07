//! Configuration for the enhanced REPL.

use std::collections::HashMap;

/// Configuration for the enhanced REPL
#[derive(Debug, Clone)]
pub struct ReplConfig {
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
    /// Enable auto-completion
    pub auto_completion: bool,
    /// Enable debugger integration
    pub debugger_enabled: bool,
    /// Maximum history entries to keep
    pub max_history: usize,
    /// Enable session saving/loading
    pub session_management: bool,
    /// Enable performance profiling
    pub profiling_enabled: bool,
    /// Custom key bindings
    pub key_bindings: HashMap<String, String>,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            auto_completion: true,
            debugger_enabled: true,
            max_history: 1000,
            session_management: true,
            profiling_enabled: false,
            key_bindings: HashMap::new(),
        }
    }
}
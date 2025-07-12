//! Language Server Protocol implementation for Lambdust Scheme
//!
//! This module provides a Language Server Protocol (LSP) implementation that enables
//! rich IDE integration with features like:
//! - Syntax checking and diagnostics
//! - Code completion with intelligent suggestions
//! - Hover information with type and documentation
//! - Symbol resolution and workspace navigation
//! - Integration with formal verification system
//! - Real-time performance analysis
//! - REPL integration for interactive development
//!
//! Note: LSP functionality requires external dependencies (tokio, lsp_types, url)
//! that are not included in the default Cargo.toml configuration.

#[cfg(feature = "language-server")]
pub mod server;
#[cfg(feature = "language-server")]
pub mod handlers;
#[cfg(feature = "language-server")]
pub mod diagnostics;
#[cfg(feature = "language-server")]
pub mod completion;
#[cfg(feature = "language-server")]
pub mod hover;
#[cfg(feature = "language-server")]
pub mod symbols;
#[cfg(feature = "language-server")]
pub mod position;
#[cfg(feature = "language-server")]
pub mod document;

// Re-export main types for convenience
#[cfg(feature = "language-server")]
pub use server::LambdustLanguageServer;
#[cfg(feature = "language-server")]
pub use diagnostics::DiagnosticsEngine;
#[cfg(feature = "language-server")]
pub use completion::CompletionProvider;
#[cfg(feature = "language-server")]
pub use hover::HoverProvider;
#[cfg(feature = "language-server")]
pub use symbols::SymbolProvider;
#[cfg(feature = "language-server")]
pub use position::{Position, Range, PositionUtils};
#[cfg(feature = "language-server")]
pub use document::DocumentManager;

use crate::error::{LambdustError, Result};
use std::path::PathBuf;

/// Configuration for the Language Server
#[derive(Debug, Clone)]
pub struct LspConfig {
    /// Enable debug mode for additional logging
    pub debug_mode: bool,
    
    /// Enable formal verification integration
    pub enable_verification: bool,
    
    /// Enable performance analysis integration
    pub enable_performance_analysis: bool,
    
    /// Maximum number of diagnostics to report per document
    pub max_diagnostics: usize,
    
    /// Completion trigger characters
    pub completion_triggers: Vec<String>,
    
    /// Workspace root directory
    pub workspace_root: Option<PathBuf>,
    
    /// Enable REPL integration
    pub enable_repl_integration: bool,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            debug_mode: false,
            enable_verification: true,
            enable_performance_analysis: true,
            max_diagnostics: 100,
            completion_triggers: vec![
                "(".to_string(),
                " ".to_string(),
                "\t".to_string(),
            ],
            workspace_root: None,
            enable_repl_integration: true,
        }
    }
}

/// LSP capability information
#[derive(Debug, Clone)]
pub struct LspCapabilities {
    /// Text document synchronization
    pub text_document_sync: bool,
    
    /// Completion support
    pub completion: bool,
    
    /// Hover support  
    pub hover: bool,
    
    /// Diagnostics support
    pub diagnostics: bool,
    
    /// Symbol navigation support
    pub symbol_navigation: bool,
    
    /// Definition provider
    pub definition: bool,
    
    /// References provider
    pub references: bool,
    
    /// Document formatting
    pub formatting: bool,
    
    /// Semantic highlighting
    pub semantic_highlighting: bool,
}

impl Default for LspCapabilities {
    fn default() -> Self {
        Self {
            text_document_sync: true,
            completion: true,
            hover: true,
            diagnostics: true,
            symbol_navigation: true,
            definition: true,
            references: true,
            formatting: true,
            semantic_highlighting: true,
        }
    }
}

/// Initialize LSP server with configuration
#[cfg(feature = "language-server")]
pub fn initialize_lsp_server(config: LspConfig) -> Result<LambdustLanguageServer> {
    LambdustLanguageServer::new(config)
}

/// LSP server error types
#[derive(Debug, thiserror::Error)]
pub enum LspError {
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("Invalid position: line {line}, character {character}")]
    InvalidPosition { line: u32, character: u32 },
    
    #[error("Workspace error: {0}")]
    Workspace(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<LspError> for LambdustError {
    fn from(err: LspError) -> Self {
        LambdustError::runtime_error(format!("LSP error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_config_default() {
        let config = LspConfig::default();
        assert!(!config.debug_mode);
        assert!(config.enable_verification);
        assert!(config.enable_performance_analysis);
        assert_eq!(config.max_diagnostics, 100);
        assert!(config.completion_triggers.contains(&"(".to_string()));
    }

    #[test]
    fn test_lsp_capabilities_default() {
        let capabilities = LspCapabilities::default();
        assert!(capabilities.completion);
        assert!(capabilities.hover);
        assert!(capabilities.diagnostics);
        assert!(capabilities.symbol_navigation);
    }
}
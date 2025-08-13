//! Enhanced REPL system for Lambdust with debugging, history, and completion features.
//!
//! This module provides a modern, developer-friendly interactive experience with:
//! - Step-through debugging with breakpoints
//! - Persistent command history with search
//! - Intelligent autocompletion 
//! - Syntax highlighting and bracket matching
//! - Multi-line input with proper indentation
//! - Code introspection and development tools

// Minimal REPL for lightweight builds
#[cfg(feature = "minimal-repl")]
pub mod minimal_repl;

// Enhanced REPL components (for full-featured builds)
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub mod completion;
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub mod debugger;
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub mod editor;
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub mod history;
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub mod highlighting;
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub mod inspector;
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub mod session;

// Individual structure modules
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub mod repl_config;
#[cfg(feature = "enhanced-repl")]
pub mod enhanced_repl;

use crate::{Lambdust, Result};

// Re-exports for minimal REPL
#[cfg(feature = "minimal-repl")]
pub use minimal_repl::{MinimalRepl, MinimalReplConfig, start_minimal_repl, start_minimal_repl_with_config};

// Re-exports for enhanced REPL components
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub use completion::CompletionProvider;
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub use debugger::{Debugger, DebugCommand, BreakpointManager};
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub use editor::EnhancedEditor;
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub use history::{HistoryManager, HistorySearch};
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub use highlighting::SyntaxHighlighter;
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub use inspector::{CodeInspector, IntrospectionCommand};
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub use session::{SessionManager, SessionState};

// Re-export individual structures for enhanced REPL
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub use repl_config::*;
#[cfg(feature = "enhanced-repl")]
pub use enhanced_repl::*;
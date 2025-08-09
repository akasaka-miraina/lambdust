//! Enhanced REPL system for Lambdust with debugging, history, and completion features.
//!
//! This module provides a modern, developer-friendly interactive experience with:
//! - Step-through debugging with breakpoints
//! - Persistent command history with search
//! - Intelligent autocompletion 
//! - Syntax highlighting and bracket matching
//! - Multi-line input with proper indentation
//! - Code introspection and development tools

pub mod completion;
pub mod debugger;
pub mod editor;
pub mod history;
pub mod highlighting;
pub mod inspector;
pub mod session;

// Individual structure modules
pub mod repl_config;
/// Enhanced REPL implementation with advanced features.
pub mod enhanced_repl;

use crate::{Lambdust, Result};

pub use completion::CompletionProvider;
pub use debugger::{Debugger, DebugCommand, BreakpointManager};
pub use editor::EnhancedEditor;
pub use history::{HistoryManager, HistorySearch};
pub use highlighting::SyntaxHighlighter;
pub use inspector::{CodeInspector, IntrospectionCommand};
pub use session::{SessionManager, SessionState};

// Re-export individual structures
pub use repl_config::*;
pub use enhanced_repl::*;
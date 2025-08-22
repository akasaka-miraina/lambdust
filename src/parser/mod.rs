//! Parser for the Lambdust language.
//!
//! This module implements a recursive descent parser that converts a stream of tokens
//! into an Abstract Syntax Tree (AST) representing Lambdust programs.

use crate::ast::{Expr, Formals, Literal, Program};
use crate::diagnostics::{Error, Result, SourceMap, Span, Spanned};
use crate::lexer::{Token, TokenKind};
use std::collections::HashMap;
use std::sync::Arc;

/// Type alias for parse results that might return multiple errors
pub type ParseResult<T> = std::result::Result<T, Vec<Error>>;
/// Type alias for list parsing results (elements and optional tail for dotted pairs)
pub type ListElements = (Vec<Spanned<Expr>>, Option<Spanned<Expr>>);

// ParserConfig moved to parser_config.rs

/// Expression parsing utilities.
pub mod expression;
/// Literal parsing utilities.
pub mod literals;
/// Special form parsing utilities.
pub mod special_forms;
/// Type expression parsing utilities.
pub mod type_expr_parser;

// Individual structure modules
/// Core parser implementation.
pub mod parser;
/// Parser builder for configurable parser construction.
pub mod parser_builder;
/// Parser configuration management.
pub mod parser_config;
/// Parser error recovery configuration.
pub mod recovery_config;

pub use parser::*;
pub use parser_builder::*;
/// Integration tests for character literal parsing.
#[cfg(test)]
// mod character_test_integration;
// Re-export individual structures
pub use parser_config::*;
pub use recovery_config::*;

// Re-export specific items if needed
// pub use expression::*;
// pub use literals::*;
// pub use special_forms::*;

/// 内製パーサーコンビネータシステム
pub mod combinators;

// Advanced parsing modules for IDE integration and error recovery
/// Contextual error message generation
pub mod contextual_errors;
/// Advanced error recovery strategies
pub mod error_recovery;
/// IDE integration support
pub mod ide_support;
/// Language Server Protocol v3.17 integration
pub mod lsp_integration;
/// Partial parsing for incomplete code
pub mod partial_parser;
/// Real-time feedback system
pub mod realtime_feedback;

// Re-export advanced parsing features
pub use contextual_errors::*;
pub use error_recovery::*;
pub use ide_support::*;
pub use lsp_integration::*;
pub use partial_parser::*;
pub use realtime_feedback::*;

// Comprehensive test suite for enhanced parsing features
#[cfg(test)]
pub mod enhanced_parsing_tests;
#[cfg(test)]
pub mod minimal_test;
#[cfg(test)]
pub mod parser_integration_test;

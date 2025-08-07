//! Parser for the Lambdust language.
//!
//! This module implements a recursive descent parser that converts a stream of tokens
//! into an Abstract Syntax Tree (AST) representing Lambdust programs.

use crate::ast::{
    Expr, Formals, Literal, Program
};
use crate::diagnostics::{Error, Result, Span, Spanned, SourceMap};
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

// Individual structure modules
/// Parser configuration management.
pub mod parser_config;
/// Core parser implementation.
pub mod parser;
/// Parser error recovery configuration.
pub mod recovery_config;
/// Parser builder for configurable parser construction.
pub mod parser_builder;

// Re-export individual structures
pub use parser_config::*;
pub use parser::*;
pub use recovery_config::*;
pub use parser_builder::*;

// Re-export specific items if needed
// pub use expression::*;
// pub use literals::*;
// pub use special_forms::*;




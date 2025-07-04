//! Unit tests module
//!
//! This module contains all unit tests that were previously embedded
//! in source files using #[cfg(test)]. Unit tests verify the behavior
//! of individual functions and components in isolation.

// Core functionality unit tests
pub mod lexer_tests;
pub mod parser_tests;
pub mod lib_tests;

// Builtin functions unit tests
pub mod higher_order_tests;

// Additional unit tests will be added here as they are extracted
// from source files and moved to this directory.
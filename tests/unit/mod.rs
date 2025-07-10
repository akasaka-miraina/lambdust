//! Unit tests module
//!
//! This module contains all unit tests that were previously embedded
//! in source files using #[cfg(test)]. Unit tests verify the behavior
//! of individual functions and components in isolation.

// Core functionality unit tests
pub mod adaptive_memory_tests;
pub mod ast_tests;
pub mod bridge_tests;
pub mod environment;
pub mod environment_tests;
pub mod error_handling_tests;
pub mod evaluator;
pub mod evaluator_import_tests;
pub mod evaluator_tests;
pub mod host_tests;
pub mod interpreter_tests;
pub mod lexer_tests;
pub mod lib_tests;
// pub mod macro_integration_tests; // 削除済み
pub mod macros_tests;
pub mod marshal_tests;
pub mod memory_pool_tests;
pub mod parser_tests;
pub mod performance_optimization_tests;
pub mod phase_3c_optimization_tests;
pub mod stack_monitor_tests;
pub mod value;
pub mod value_tests;

// Builtin functions unit tests
pub mod builtins;
pub mod higher_order_tests;
pub mod lambda_integration_tests;

// SRFI and module system unit tests
pub mod module_system_tests;
pub mod srfi;

// Additional unit tests will be added here as they are extracted
// from source files and moved to this directory.

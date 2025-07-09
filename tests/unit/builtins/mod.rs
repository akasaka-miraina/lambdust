//! Unit tests for builtin functions
//!
//! This module contains extracted unit tests for builtin function modules,
//! organized to match the structure of the src/builtins/ directory.

pub mod arithmetic_tests;
pub mod lazy_tests;
pub mod list_ops_tests;
pub mod srfi_13_tests;
pub mod srfi_1_tests;
pub mod srfi_69_tests;
pub mod srfi_tests;
pub mod string_char_tests;
// I/O operations tests
pub mod io_tests;
// Error handling tests
pub mod error_handling_tests;
// Store management tests
pub mod store_tests;
// Higher-order function tests
pub mod higher_order_tests;
// Predicate function tests
pub mod predicates_tests;
// Miscellaneous function tests
pub mod misc_tests;

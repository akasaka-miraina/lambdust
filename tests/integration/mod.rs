//! Integration tests module
//!
//! This module contains all integration tests that verify the behavior
//! of the complete system by testing multiple components working together.
//! These tests use the public API and test end-to-end functionality.

// Core system integration tests
pub mod bridge_tests;
pub mod error_handling_tests;
pub mod evaluator_tests;
pub mod exception_handling_tests;
pub mod integration_tests;
pub mod r7rs_compliance_tests;
pub mod syntax_rules_tests;

// SRFI implementation integration tests
pub mod srfi_1_tests;
pub mod srfi_13_tests;
pub mod srfi_69_tests;
pub mod srfi_97_tests;
pub mod srfi_tests;
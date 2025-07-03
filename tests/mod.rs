//! Integration test module index
//! 
//! This module organizes all integration tests for the Lambdust Scheme interpreter.
//! Integration tests verify the complete functionality from an external perspective.

/// Basic interpreter functionality tests
pub mod integration_tests;

/// Formal evaluator and R7RS compliance tests  
pub mod evaluator_tests;

/// SRFI module system and implementation tests
pub mod srfi_tests;

/// Bridge API tests (some currently disabled)
pub mod bridge_tests;

/// Error handling and robustness tests
pub mod error_handling_tests;

/// R7RS specification compliance tests
pub mod r7rs_compliance_tests;
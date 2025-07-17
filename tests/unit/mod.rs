//! Unit tests for Lambdust Scheme interpreter
//!
//! This module contains unit tests for individual components of the Lambdust
//! Scheme interpreter. Each sub-module focuses on testing a specific module
//! or component in isolation.
//!
//! ## Mock-Based Testing
//!
//! Many tests use mockall for isolated component testing, allowing us to:
//! - Test components in isolation
//! - Verify component interactions
//! - Test error conditions and edge cases
//! - Achieve high test coverage with controlled scenarios

pub mod evaluator;
pub mod environment;
pub mod executor;
pub mod value;
pub mod parser;
pub mod macros;
pub mod type_system;
pub mod builtins;
pub mod integration_mock_tests;
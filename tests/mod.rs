//! Test organization module
//!
//! This module organizes all tests for the Lambdust Scheme interpreter into
//! two main categories: unit tests and integration tests.

/// Unit tests - Test individual functions and components in isolation
/// These tests were previously embedded in source files using #[cfg(test)]
pub mod unit;

/// Integration tests - Test complete system functionality end-to-end
/// These tests use the public API and verify component interactions
pub mod integration;

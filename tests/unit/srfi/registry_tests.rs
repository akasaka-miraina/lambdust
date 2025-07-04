//! Unit tests for SRFI registry functionality
//!
//! These tests were extracted from src/srfi/registry.rs

use lambdust::srfi::SrfiRegistry;

#[test]
fn test_registry_creation() {
    let registry = SrfiRegistry::with_standard_srfis();
    let available = registry.available_srfis();

    // Should have the standard SRFIs available
    assert!(available.contains(&9)); // SRFI 9
    assert!(available.contains(&45)); // SRFI 45
    assert!(available.contains(&46)); // SRFI 46
    assert!(available.contains(&97)); // SRFI 97
}

#[test]
fn test_srfi_availability() {
    let registry = SrfiRegistry::with_standard_srfis();

    assert!(registry.has_srfi(9));
    assert!(registry.has_srfi(45));
    assert!(registry.has_srfi(46));
    assert!(registry.has_srfi(97));
    assert!(!registry.has_srfi(999)); // Non-existent SRFI
}

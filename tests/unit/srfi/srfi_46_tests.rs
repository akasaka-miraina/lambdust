//! Unit tests for SRFI 46 (Basic Syntax-rules Extensions)
//!
//! These tests were extracted from src/srfi/srfi_46.rs

use lambdust::srfi::srfi_46::Srfi46;
use lambdust::srfi::SrfiModule;

#[test]
fn test_srfi_46_info() {
    let srfi46 = Srfi46;
    assert_eq!(srfi46.srfi_id(), 46);
    assert_eq!(srfi46.name(), "Basic Syntax-rules Extensions");
    assert!(srfi46.parts().contains(&"syntax"));
    assert!(srfi46.parts().contains(&"ellipsis"));
}

#[test]
fn test_srfi_46_exports() {
    let srfi46 = Srfi46;
    let exports = srfi46.exports();

    // SRFI 46 is primarily about macro syntax, so no runtime exports
    assert!(exports.is_empty());
}

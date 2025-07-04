//! Unit tests for module system functionality
//!
//! These tests were extracted from src/module_system.rs

use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;
use lambdust::module_system::{ImportSpec, ModuleSystem};

#[test]
fn test_parse_srfi_import_spec() {
    let module_system = ModuleSystem::new();

    let expr = Expr::List(vec![
        Expr::Variable("srfi".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(9))),
    ]);

    let specs = module_system.parse_import_specs(&[expr]).unwrap();
    assert_eq!(specs.len(), 1);

    if let ImportSpec::Srfi(srfi_import) = &specs[0] {
        assert_eq!(srfi_import.id, 9);
    } else {
        panic!("Expected SRFI import spec");
    }
}

#[test]
fn test_parse_library_import_spec() {
    let module_system = ModuleSystem::new();

    let expr = Expr::List(vec![
        Expr::Variable("scheme".to_string()),
        Expr::Variable("base".to_string()),
    ]);

    let specs = module_system.parse_import_specs(&[expr]).unwrap();
    assert_eq!(specs.len(), 1);

    if let ImportSpec::Library(library_import) = &specs[0] {
        assert_eq!(library_import.parts, vec!["scheme", "base"]);
    } else {
        panic!("Expected library import spec");
    }
}

#[test]
fn test_available_srfis() {
    let module_system = ModuleSystem::new();
    let available = module_system.available_srfis();

    assert!(available.contains(&9));
    assert!(available.contains(&45));
    assert!(available.contains(&46));
}

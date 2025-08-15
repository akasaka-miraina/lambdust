//! Basic Integration Test for Dependent Type System
//!
//! This is a standalone integration test to verify the basic functionality
//! of the dependent type system without external dependencies.

use lambdust::types::dependent::core::{DependentType, DependentTerm};

/// Test basic universe type creation and equality
#[test]
fn test_dependent_type_universe_basic() {
    // Test universe construction
    let universe_0 = DependentType::Universe(0);
    let universe_1 = DependentType::Universe(1);
    
    // Test equality
    assert_eq!(universe_0, DependentType::Universe(0));
    assert_ne!(universe_0, universe_1);
    
    println!("✅ Universe types work correctly");
}

/// Test variable term construction
#[test]
fn test_dependent_term_variables() {
    let var_x = DependentTerm::Variable("x".to_string());
    let var_y = DependentTerm::Variable("y".to_string());
    
    // Test equality based on name
    assert_eq!(var_x, DependentTerm::Variable("x".to_string()));
    assert_ne!(var_x, var_y);
    
    println!("✅ Variable terms work correctly");
}

/// Test Pi-type (dependent function type) construction
#[test]
fn test_dependent_type_pi_types() {
    let pi_type = DependentType::Pi {
        var: "x".to_string(),
        domain: Box::new(DependentType::Universe(0)),
        codomain: Box::new(DependentType::Universe(1)),
    };
    
    // Verify structure
    match &pi_type {
        DependentType::Pi { var, domain, codomain } => {
            assert_eq!(var, "x");
            assert_eq!(**domain, DependentType::Universe(0));
            assert_eq!(**codomain, DependentType::Universe(1));
        }
        _ => panic!("Should be a Pi-type"),
    }
    
    println!("✅ Π-type construction works correctly");
}

/// Test Sigma-type (dependent pair type) construction
#[test]
fn test_dependent_type_sigma_types() {
    let sigma_type = DependentType::Sigma {
        var: "x".to_string(),
        first: Box::new(DependentType::Universe(0)),
        second: Box::new(DependentType::Universe(1)),
    };
    
    // Verify structure
    match &sigma_type {
        DependentType::Sigma { var, first, second } => {
            assert_eq!(var, "x");
            assert_eq!(**first, DependentType::Universe(0));
            assert_eq!(**second, DependentType::Universe(1));
        }
        _ => panic!("Should be a Sigma-type"),
    }
    
    println!("✅ Σ-type construction works correctly");
}

/// Test Lambda term construction
#[test]
fn test_dependent_term_lambda() {
    let lambda = DependentTerm::Lambda {
        param: "x".to_string(),
        param_type: Box::new(DependentType::Universe(0)),
        body: Box::new(DependentTerm::Variable("x".to_string())),
    };
    
    // Verify structure
    match &lambda {
        DependentTerm::Lambda { param, param_type, body } => {
            assert_eq!(param, "x");
            assert_eq!(**param_type, DependentType::Universe(0));
            match body.as_ref() {
                DependentTerm::Variable(name) => {
                    assert_eq!(name, "x");
                }
                _ => panic!("Body should be a variable"),
            }
        }
        _ => panic!("Should be a Lambda"),
    }
    
    println!("✅ Lambda term construction works correctly");
}

/// Test Application term construction
#[test]
fn test_dependent_term_application() {
    let application = DependentTerm::Application {
        function: Box::new(DependentTerm::Variable("f".to_string())),
        argument: Box::new(DependentTerm::Variable("x".to_string())),
    };
    
    // Verify structure
    match &application {
        DependentTerm::Application { function, argument } => {
            match (function.as_ref(), argument.as_ref()) {
                (DependentTerm::Variable(f_name), DependentTerm::Variable(x_name)) => {
                    assert_eq!(f_name, "f");
                    assert_eq!(x_name, "x");
                }
                _ => panic!("Function and argument should be variables"),
            }
        }
        _ => panic!("Should be an Application"),
    }
    
    println!("✅ Application term construction works correctly");
}

/// Test Identity type construction
#[test]
fn test_dependent_type_identity() {
    let identity_type = DependentType::Identity {
        ty: Box::new(DependentType::Universe(0)),
        left: Box::new(DependentTerm::Variable("a".to_string())),
        right: Box::new(DependentTerm::Variable("b".to_string())),
    };
    
    // Verify structure
    match &identity_type {
        DependentType::Identity { ty, left, right } => {
            assert_eq!(**ty, DependentType::Universe(0));
            match (left.as_ref(), right.as_ref()) {
                (DependentTerm::Variable(l_name), DependentTerm::Variable(r_name)) => {
                    assert_eq!(l_name, "a");
                    assert_eq!(r_name, "b");
                }
                _ => panic!("Left and right should be variables"),
            }
        }
        _ => panic!("Should be an Identity type"),
    }
    
    println!("✅ Identity type construction works correctly");
}

/// Test Inductive type construction
#[test]
fn test_dependent_type_inductive() {
    let nat_type = DependentType::Inductive {
        name: "Nat".to_string(),
        parameters: vec![],
        universe_level: 0,
        constructors: vec![
            ("zero".to_string(), DependentType::Universe(0)),
            ("succ".to_string(), DependentType::Pi {
                var: "_".to_string(),
                domain: Box::new(DependentType::Universe(0)),
                codomain: Box::new(DependentType::Universe(0)),
            }),
        ],
        induction_principle: None,
    };
    
    // Verify structure
    match &nat_type {
        DependentType::Inductive { name, parameters, universe_level, constructors, .. } => {
            assert_eq!(name, "Nat");
            assert!(parameters.is_empty());
            assert_eq!(*universe_level, 0);
            assert_eq!(constructors.len(), 2);
            assert_eq!(constructors[0].0, "zero");
            assert_eq!(constructors[1].0, "succ");
        }
        _ => panic!("Should be an Inductive type"),
    }
    
    println!("✅ Inductive type construction works correctly");
}

/// Test Dependent Pair term construction
#[test]
fn test_dependent_term_pair() {
    let pair = DependentTerm::Pair {
        first: Box::new(DependentTerm::Variable("a".to_string())),
        second: Box::new(DependentTerm::Variable("b".to_string())),
    };
    
    // Verify structure
    match &pair {
        DependentTerm::Pair { first, second } => {
            match (first.as_ref(), second.as_ref()) {
                (DependentTerm::Variable(first_name), DependentTerm::Variable(second_name)) => {
                    assert_eq!(first_name, "a");
                    assert_eq!(second_name, "b");
                }
                _ => panic!("First and second should be variables"),
            }
        }
        _ => panic!("Should be a Pair"),
    }
    
    println!("✅ Dependent pair term construction works correctly");
}

/// Test Projection term construction
#[test]
fn test_dependent_term_projection() {
    let pair_var = DependentTerm::Variable("p".to_string());
    
    let first_proj = DependentTerm::Projection {
        pair: Box::new(pair_var.clone()),
        is_first: true,
    };
    
    let second_proj = DependentTerm::Projection {
        pair: Box::new(pair_var),
        is_first: false,
    };
    
    // Verify structure
    match &first_proj {
        DependentTerm::Projection { pair, is_first } => {
            assert!(*is_first);
            match pair.as_ref() {
                DependentTerm::Variable(name) => assert_eq!(name, "p"),
                _ => panic!("Pair should be a variable"),
            }
        }
        _ => panic!("Should be a Projection"),
    }
    
    match &second_proj {
        DependentTerm::Projection { is_first, .. } => {
            assert!(!*is_first);
        }
        _ => panic!("Should be a Projection"),
    }
    
    println!("✅ Projection term construction works correctly");
}

/// Test Reflexivity term construction
#[test]
fn test_dependent_term_refl() {
    let refl = DependentTerm::Refl {
        ty: Box::new(DependentType::Universe(0)),
    };
    
    // Verify structure
    match &refl {
        DependentTerm::Refl { ty } => {
            assert_eq!(**ty, DependentType::Universe(0));
        }
        _ => panic!("Should be a Refl"),
    }
    
    println!("✅ Reflexivity term construction works correctly");
}

/// Test Constructor term construction
#[test]
fn test_dependent_term_constructor() {
    let constructor = DependentTerm::Constructor {
        name: "zero".to_string(),
        args: vec![],
        result_type: Box::new(DependentType::Universe(0)),
    };
    
    // Verify structure
    match &constructor {
        DependentTerm::Constructor { name, args, result_type } => {
            assert_eq!(name, "zero");
            assert!(args.is_empty());
            assert_eq!(**result_type, DependentType::Universe(0));
        }
        _ => panic!("Should be a Constructor"),
    }
    
    println!("✅ Constructor term construction works correctly");
}

/// Test complex nested types
#[test]
fn test_complex_nested_types() {
    // Create a complex nested type: (A : Type₀) → (B : Type₀) → (f : A → B) → A → B
    let complex_type = DependentType::Pi {
        var: "A".to_string(),
        domain: Box::new(DependentType::Universe(0)),
        codomain: Box::new(DependentType::Pi {
            var: "B".to_string(),
            domain: Box::new(DependentType::Universe(0)),
            codomain: Box::new(DependentType::Pi {
                var: "f".to_string(),
                domain: Box::new(DependentType::Pi {
                    var: "x".to_string(),
                    domain: Box::new(DependentType::Universe(0)), // Simplified for now
                    codomain: Box::new(DependentType::Universe(0)),
                }),
                codomain: Box::new(DependentType::Pi {
                    var: "x".to_string(),
                    domain: Box::new(DependentType::Universe(0)), // Simplified for now
                    codomain: Box::new(DependentType::Universe(0)),
                }),
            }),
        }),
    };
    
    // Verify the structure exists and is well-formed
    match &complex_type {
        DependentType::Pi { var, domain, codomain } => {
            assert_eq!(var, "A");
            assert_eq!(**domain, DependentType::Universe(0));
            // The nested structure should be well-formed
            assert!(matches!(codomain.as_ref(), DependentType::Pi { .. }));
        }
        _ => panic!("Should be a Pi-type"),
    }
    
    println!("✅ Complex nested types work correctly");
}

/// Performance test for type construction
#[test]
fn test_type_construction_performance() {
    use std::time::Instant;
    
    let start = Instant::now();
    
    // Create many types quickly
    for i in 0..1000 {
        let _universe = DependentType::Universe(i % 10);
        let _variable = DependentTerm::Variable(format!("var_{i}"));
        let _pi_type = DependentType::Pi {
            var: format!("param_{i}"),
            domain: Box::new(DependentType::Universe(0)),
            codomain: Box::new(DependentType::Universe(1)),
        };
    }
    
    let elapsed = start.elapsed();
    println!("Created 3000 types/terms in {elapsed:?}");
    
    // Should be very fast
    assert!(elapsed.as_millis() < 100, "Type construction should be fast");
    
    println!("✅ Type construction performance is acceptable");
}

/// Test equality and hashing
#[test]
fn test_equality_and_hashing() {
    use std::collections::HashSet;
    
    // Test that equal types are equal and hash the same
    let type1 = DependentType::Universe(0);
    let type2 = DependentType::Universe(0);
    let type3 = DependentType::Universe(1);
    
    assert_eq!(type1, type2);
    assert_ne!(type1, type3);
    
    // Test hashing
    let mut set = HashSet::new();
    set.insert(type1.clone());
    assert!(set.contains(&type2));
    assert!(!set.contains(&type3));
    
    // Test terms
    let term1 = DependentTerm::Variable("x".to_string());
    let term2 = DependentTerm::Variable("x".to_string());
    let term3 = DependentTerm::Variable("y".to_string());
    
    assert_eq!(term1, term2);
    assert_ne!(term1, term3);
    
    println!("✅ Equality and hashing work correctly");
}

/// Test comprehensive dependent type system functionality
#[test]
fn test_comprehensive_dependent_type_system() {
    println!("\n🧪 Comprehensive Dependent Type System Test");
    println!("===========================================");
    
    // Test all major type constructors
    test_dependent_type_universe_basic();
    test_dependent_term_variables();
    test_dependent_type_pi_types();
    test_dependent_type_sigma_types();
    test_dependent_term_lambda();
    test_dependent_term_application();
    test_dependent_type_identity();
    test_dependent_type_inductive();
    test_dependent_term_pair();
    test_dependent_term_projection();
    test_dependent_term_refl();
    test_dependent_term_constructor();
    
    // Test functionality
    test_complex_nested_types();
    test_type_construction_performance();
    test_equality_and_hashing();
    
    println!("\n🎉 All dependent type system tests passed!");
    println!("The core dependent type system is working correctly.");
}
//! Type System Unit Tests
//!
//! Comprehensive test suite for the Polynomial Universe Type System covering:
//! - Universe polymorphic type classes and their theoretical foundations
//! - Dependent type theory integration and correctness
//! - Monad algebra distributive laws and composition
//! - Homotopy Type Theory (HoTT) features and univalence
//! - Parallel type checking performance and correctness
//! - Incremental type inference with caching efficiency

pub mod polynomial_types_tests;
pub mod universe_polymorphic_tests;
pub mod dependent_types_tests;
pub mod monad_algebra_tests;
pub mod hott_integration_tests;
pub mod parallel_type_checker_tests;
pub mod incremental_inference_tests;
pub mod type_system_integration_tests;

use crate::type_system::*;
use crate::value::Value;
use crate::lexer::SchemeNumber;

/// Test utility functions for type system testing
pub mod test_utils {
    use super::*;
    
    /// Create a simple natural number value for testing
    pub fn nat_value(n: i64) -> Value {
        Value::Number(SchemeNumber::Integer(n))
    }
    
    /// Create a string value for testing
    pub fn string_value(s: &str) -> Value {
        Value::String(s.to_string())
    }
    
    /// Create a boolean value for testing
    pub fn bool_value(b: bool) -> Value {
        Value::Boolean(b)
    }
    
    /// Create a test polynomial universe system
    pub fn create_test_system() -> PolynomialUniverseSystem {
        let mut system = PolynomialUniverseSystem::new();
        
        // Initialize with standard classes and transformers
        let _ = system.initialize_standard_universe_classes();
        let _ = system.initialize_standard_transformers();
        
        system
    }
    
    /// Create test universe polymorphic class (Functor)
    pub fn create_functor_class() -> UniversePolymorphicClass {
        UniversePolymorphicClass {
            name: "Functor".to_string(),
            universe_parameter: "u".to_string(),
            type_parameters: vec![
                UniversePolymorphicParameter {
                    name: "f".to_string(),
                    universe_constraint: UniverseConstraint::AtLeast(UniverseLevel::new(1)),
                    kind_constraint: Some(KindConstraint::TypeConstructor(1)),
                }
            ],
            methods: vec![
                UniversePolymorphicMethod {
                    name: "fmap".to_string(),
                    signature: UniversePolymorphicType::ForAllUniverse {
                        universe_var: "u".to_string(),
                        constraint: UniverseConstraint::Any,
                        body: Box::new(UniversePolymorphicType::Concrete {
                            poly_type: PolynomialType::Base(polynomial_types::BaseType::Natural),
                            universe: UniverseLevel::new(0),
                        }),
                    },
                    default_impl: None,
                    laws: vec![
                        UniversePolymorphicLaw {
                            name: "functor_identity".to_string(),
                            universe_quantifiers: vec!["u".to_string()],
                            type_quantifiers: vec![],
                            premise: vec![],
                            conclusion: UniversePolymorphicEquation {
                                left: Value::Symbol("fmap_id".to_string()),
                                right: Value::Symbol("id".to_string()),
                                equality_type: UniversePolymorphicType::Concrete {
                                    poly_type: PolynomialType::Base(polynomial_types::BaseType::Natural),
                                    universe: UniverseLevel::new(0),
                                },
                            },
                        },
                        UniversePolymorphicLaw {
                            name: "functor_composition".to_string(),
                            universe_quantifiers: vec!["u".to_string()],
                            type_quantifiers: vec![],
                            premise: vec![],
                            conclusion: UniversePolymorphicEquation {
                                left: Value::Symbol("fmap_compose".to_string()),
                                right: Value::Symbol("compose_fmap".to_string()),
                                equality_type: UniversePolymorphicType::Concrete {
                                    poly_type: PolynomialType::Base(polynomial_types::BaseType::Natural),
                                    universe: UniverseLevel::new(0),
                                },
                            },
                        },
                    ],
                }
            ],
            laws: vec![],
            superclasses: vec![],
        }
    }
    
    /// Create test universe polymorphic class (Monad)
    pub fn create_monad_class() -> UniversePolymorphicClass {
        UniversePolymorphicClass {
            name: "Monad".to_string(),
            universe_parameter: "u".to_string(),
            type_parameters: vec![
                UniversePolymorphicParameter {
                    name: "m".to_string(),
                    universe_constraint: UniverseConstraint::AtLeast(UniverseLevel::new(1)),
                    kind_constraint: Some(KindConstraint::TypeConstructor(1)),
                }
            ],
            methods: vec![
                UniversePolymorphicMethod {
                    name: "return".to_string(),
                    signature: UniversePolymorphicType::ForAllUniverse {
                        universe_var: "u".to_string(),
                        constraint: UniverseConstraint::Any,
                        body: Box::new(UniversePolymorphicType::Concrete {
                            poly_type: PolynomialType::Base(polynomial_types::BaseType::Natural),
                            universe: UniverseLevel::new(0),
                        }),
                    },
                    default_impl: None,
                    laws: vec![],
                },
                UniversePolymorphicMethod {
                    name: "bind".to_string(),
                    signature: UniversePolymorphicType::ForAllUniverse {
                        universe_var: "u".to_string(),
                        constraint: UniverseConstraint::Any,
                        body: Box::new(UniversePolymorphicType::Concrete {
                            poly_type: PolynomialType::Base(polynomial_types::BaseType::Natural),
                            universe: UniverseLevel::new(0),
                        }),
                    },
                    default_impl: None,
                    laws: vec![],
                }
            ],
            laws: vec![
                UniversePolymorphicLaw {
                    name: "monad_left_identity".to_string(),
                    universe_quantifiers: vec!["u".to_string()],
                    type_quantifiers: vec![],
                    premise: vec![],
                    conclusion: UniversePolymorphicEquation {
                        left: Value::Symbol("bind_return".to_string()),
                        right: Value::Symbol("id".to_string()),
                        equality_type: UniversePolymorphicType::Concrete {
                            poly_type: PolynomialType::Base(polynomial_types::BaseType::Natural),
                            universe: UniverseLevel::new(0),
                        },
                    },
                },
                UniversePolymorphicLaw {
                    name: "monad_right_identity".to_string(),
                    universe_quantifiers: vec!["u".to_string()],
                    type_quantifiers: vec![],
                    premise: vec![],
                    conclusion: UniversePolymorphicEquation {
                        left: Value::Symbol("return_bind".to_string()),
                        right: Value::Symbol("m".to_string()),
                        equality_type: UniversePolymorphicType::Concrete {
                            poly_type: PolynomialType::Base(polynomial_types::BaseType::Natural),
                            universe: UniverseLevel::new(0),
                        },
                    },
                },
                UniversePolymorphicLaw {
                    name: "monad_associativity".to_string(),
                    universe_quantifiers: vec!["u".to_string()],
                    type_quantifiers: vec![],
                    premise: vec![],
                    conclusion: UniversePolymorphicEquation {
                        left: Value::Symbol("bind_bind_assoc_left".to_string()),
                        right: Value::Symbol("bind_bind_assoc_right".to_string()),
                        equality_type: UniversePolymorphicType::Concrete {
                            poly_type: PolynomialType::Base(polynomial_types::BaseType::Natural),
                            universe: UniverseLevel::new(0),
                        },
                    },
                },
            ],
            superclasses: vec![
                UniversePolymorphicConstraint {
                    class_name: "Applicative".to_string(),
                    type_args: vec![],
                    universe_constraint: None,
                }
            ],
        }
    }
    
    /// Create dependent type example (dependent pair)
    pub fn create_dependent_pair_type() -> DependentType {
        DependentType::Sigma(SigmaType {
            name: "x".to_string(),
            first_type: Box::new(DependentType::Base(polynomial_types::BaseType::Natural)),
            second_type: Box::new(DependentType::Application {
                function: Box::new(DependentType::Base(polynomial_types::BaseType::Natural)),
                argument: Box::new(DependentType::Variable("x".to_string())),
            }),
        })
    }
    
    /// Create simple Pi type (function type)
    pub fn create_function_type() -> DependentType {
        DependentType::Pi(PiType {
            parameter: "x".to_string(),
            domain: Box::new(DependentType::Base(polynomial_types::BaseType::Natural)),
            codomain: Box::new(DependentType::Base(polynomial_types::BaseType::Natural)),
        })
    }
}
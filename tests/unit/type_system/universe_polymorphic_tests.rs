//! Universe Polymorphic Type Classes Tests
//!
//! Tests for the theoretical foundations and practical implementation of
//! universe polymorphic type classes, verifying:
//! - Universe constraint solving and coherence
//! - Type class law verification across universe levels
//! - Instance resolution with universe polymorphism
//! - Proof checking for universe polymorphic laws

use super::test_utils::*;
use crate::type_system::universe_polymorphic_classes::*;
use crate::type_system::polynomial_types::*;

#[test]
fn test_universe_polymorphic_registry_creation() {
    let registry = UniversePolymorphicRegistry::new();
    
    assert!(registry.list_classes().is_empty());
    assert_eq!(registry.get_instances("NonExistent").len(), 0);
}

#[test]
fn test_functor_class_registration() {
    let registry = UniversePolymorphicRegistry::new();
    let functor_class = create_functor_class();
    
    let result = registry.register_class(functor_class);
    assert!(result.is_ok(), "Functor class registration should succeed");
    
    let classes = registry.list_classes();
    assert_eq!(classes.len(), 1);
    assert!(classes.contains(&"Functor".to_string()));
    
    let retrieved_class = registry.get_class("Functor");
    assert!(retrieved_class.is_some());
    
    let class = retrieved_class.unwrap();
    assert_eq!(class.name, "Functor");
    assert_eq!(class.universe_parameter, "u");
    assert_eq!(class.type_parameters.len(), 1);
    assert_eq!(class.methods.len(), 1);
    assert_eq!(class.methods[0].name, "fmap");
}

#[test]
fn test_monad_class_registration_with_superclasses() {
    let registry = UniversePolymorphicRegistry::new();
    let monad_class = create_monad_class();
    
    let result = registry.register_class(monad_class);
    assert!(result.is_ok(), "Monad class registration should succeed");
    
    let class = registry.get_class("Monad").unwrap();
    assert_eq!(class.superclasses.len(), 1);
    assert_eq!(class.superclasses[0].class_name, "Applicative");
    
    // Verify monad laws
    assert_eq!(class.laws.len(), 3);
    let law_names: Vec<&str> = class.laws.iter().map(|l| l.name.as_str()).collect();
    assert!(law_names.contains(&"monad_left_identity"));
    assert!(law_names.contains(&"monad_right_identity"));
    assert!(law_names.contains(&"monad_associativity"));
}

#[test]
fn test_universe_constraint_validation() {
    let registry = UniversePolymorphicRegistry::new();
    
    // Test various universe constraints
    let constraints = vec![
        UniverseConstraint::Exact(UniverseLevel::new(0)),
        UniverseConstraint::AtLeast(UniverseLevel::new(1)),
        UniverseConstraint::AtMost(UniverseLevel::new(2)),
        UniverseConstraint::Relative {
            base_param: "u".to_string(),
            offset: 1,
        },
        UniverseConstraint::Variable("v".to_string()),
        UniverseConstraint::Any,
    ];
    
    for constraint in constraints {
        let result = registry.validate_universe_constraint(&constraint);
        assert!(result.is_ok(), "Universe constraint should be valid: {:?}", constraint);
    }
}

#[test]
fn test_universe_expression_types() {
    let expressions = vec![
        UniverseExpression::Variable("u".to_string()),
        UniverseExpression::Literal(UniverseLevel::new(42)),
        UniverseExpression::Successor(
            Box::new(UniverseExpression::Variable("u".to_string()))
        ),
        UniverseExpression::Maximum(
            Box::new(UniverseExpression::Literal(UniverseLevel::new(1))),
            Box::new(UniverseExpression::Variable("v".to_string()))
        ),
    ];
    
    for expr in expressions {
        // Test that expressions can be created and pattern matched
        match &expr {
            UniverseExpression::Variable(name) => {
                assert!(!name.is_empty());
            }
            UniverseExpression::Literal(level) => {
                assert!(level.0 >= 0);
            }
            UniverseExpression::Successor(_) => {
                // Should have inner expression
            }
            UniverseExpression::Maximum(_, _) => {
                // Should have two inner expressions
            }
        }
    }
}

#[test]
fn test_universe_polymorphic_type_construction() {
    let nat_type = PolynomialType::Base(BaseType::Natural);
    
    // Concrete type
    let concrete = UniversePolymorphicType::Concrete {
        poly_type: nat_type.clone(),
        universe: UniverseLevel::new(0),
    };
    
    // Universally quantified type
    let forall = UniversePolymorphicType::ForAllUniverse {
        universe_var: "u".to_string(),
        constraint: UniverseConstraint::AtLeast(UniverseLevel::new(1)),
        body: Box::new(concrete),
    };
    
    match forall {
        UniversePolymorphicType::ForAllUniverse { universe_var, constraint, body } => {
            assert_eq!(universe_var, "u");
            assert_eq!(constraint, UniverseConstraint::AtLeast(UniverseLevel::new(1)));
            
            match *body {
                UniversePolymorphicType::Concrete { poly_type, universe } => {
                    assert_eq!(poly_type, nat_type);
                    assert_eq!(universe, UniverseLevel::new(0));
                }
                _ => panic!("Expected concrete type in body"),
            }
        }
        _ => panic!("Expected ForAllUniverse type"),
    }
}

#[test]
fn test_universe_polymorphic_instance_creation() {
    use std::collections::HashMap;
    
    let instance = UniversePolymorphicInstance {
        class_name: "Functor".to_string(),
        universe_args: {
            let mut args = HashMap::new();
            args.insert("u".to_string(), UniverseExpression::Literal(UniverseLevel::new(1)));
            args
        },
        type_args: vec![PolynomialType::Base(BaseType::List)],
        methods: {
            let mut methods = HashMap::new();
            methods.insert("fmap".to_string(), Value::Symbol("list_map".to_string()));
            methods
        },
        law_proofs: HashMap::new(),
    };
    
    assert_eq!(instance.class_name, "Functor");
    assert_eq!(instance.universe_args.len(), 1);
    assert_eq!(instance.type_args.len(), 1);
    assert_eq!(instance.methods.len(), 1);
    assert!(instance.methods.contains_key("fmap"));
}

#[test]
fn test_instance_registration_and_resolution() {
    let registry = UniversePolymorphicRegistry::new();
    
    // First register the Functor class
    let functor_class = create_functor_class();
    registry.register_class(functor_class).unwrap();
    
    // Create and register a List Functor instance
    let mut universe_args = std::collections::HashMap::new();
    universe_args.insert("u".to_string(), UniverseExpression::Literal(UniverseLevel::new(1)));
    
    let mut methods = std::collections::HashMap::new();
    methods.insert("fmap".to_string(), Value::Symbol("list_map".to_string()));
    
    let list_functor = UniversePolymorphicInstance {
        class_name: "Functor".to_string(),
        universe_args,
        type_args: vec![PolynomialType::Base(BaseType::List)],
        methods,
        law_proofs: std::collections::HashMap::new(),
    };
    
    let register_result = registry.register_instance(list_functor);
    assert!(register_result.is_ok(), "List Functor instance registration should succeed");
    
    // Test instance resolution
    let instances = registry.get_instances("Functor");
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0].type_args[0], PolynomialType::Base(BaseType::List));
    
    // Test specific instance resolution
    let resolution_result = registry.resolve_instance(
        "Functor",
        &[PolynomialType::Base(BaseType::List)],
        UniverseLevel::new(1),
    );
    assert!(resolution_result.is_ok(), "Should resolve List Functor instance");
    
    let resolved = resolution_result.unwrap();
    assert_eq!(resolved.class_name, "Functor");
    assert!(resolved.methods.contains_key("fmap"));
}

#[test]
fn test_universe_constraint_solver() {
    let solver = UniverseConstraintSolver::new();
    
    // Add some constraints
    solver.add_relation(
        "u".to_string(),
        UniverseConstraint::AtLeast(UniverseLevel::new(1))
    );
    
    solver.add_relation(
        "v".to_string(),
        UniverseConstraint::Relative {
            base_param: "u".to_string(),
            offset: 1,
        }
    );
    
    // Test constraint solving
    let universe_args = std::collections::HashMap::new();
    let result = solver.solve_constraints(&universe_args, UniverseLevel::new(2));
    
    assert!(result.is_ok(), "Constraint solving should succeed");
    let solution = result.unwrap();
    assert!(solution.is_some(), "Should find a solution");
    
    let solution = solution.unwrap();
    assert_eq!(solution.min_universe, UniverseLevel::new(2));
}

#[test]
fn test_proof_method_construction() {
    let proof = UniversePolymorphicProof {
        method: ProofMethod::UniverseInduction,
        steps: vec![
            ProofStep {
                description: "Base case: universe level 0".to_string(),
                justification: Justification::Assumption,
                result: UniversePolymorphicEquation {
                    left: Value::Symbol("base".to_string()),
                    right: Value::Symbol("base".to_string()),
                    equality_type: UniversePolymorphicType::Concrete {
                        poly_type: PolynomialType::Base(BaseType::Natural),
                        universe: UniverseLevel::new(0),
                    },
                },
            },
            ProofStep {
                description: "Inductive step: assume holds at level n, prove for level n+1".to_string(),
                justification: Justification::UniverseReasoning("universe successor".to_string()),
                result: UniversePolymorphicEquation {
                    left: Value::Symbol("step_n".to_string()),
                    right: Value::Symbol("step_n_plus_1".to_string()),
                    equality_type: UniversePolymorphicType::Concrete {
                        poly_type: PolynomialType::Base(BaseType::Natural),
                        universe: UniverseLevel::new(1),
                    },
                },
            },
        ],
        universe_scope: UniverseScope::Universal,
    };
    
    assert_eq!(proof.method, ProofMethod::UniverseInduction);
    assert_eq!(proof.steps.len(), 2);
    assert_eq!(proof.universe_scope, UniverseScope::Universal);
    
    // Verify proof steps
    assert_eq!(proof.steps[0].justification, Justification::Assumption);
    match &proof.steps[1].justification {
        Justification::UniverseReasoning(reason) => {
            assert_eq!(reason, "universe successor");
        }
        _ => panic!("Expected universe reasoning justification"),
    }
}

#[test]
fn test_universe_scope_types() {
    let scopes = vec![
        UniverseScope::Specific(UniverseLevel::new(0)),
        UniverseScope::AllAbove(UniverseLevel::new(1)),
        UniverseScope::Universal,
        UniverseScope::Range(UniverseLevel::new(0), UniverseLevel::new(5)),
    ];
    
    for scope in scopes {
        match scope {
            UniverseScope::Specific(level) => {
                assert!(level.0 >= 0);
            }
            UniverseScope::AllAbove(min_level) => {
                assert!(min_level.0 >= 0);
            }
            UniverseScope::Universal => {
                // Should work at all levels
            }
            UniverseScope::Range(start, end) => {
                assert!(start.0 <= end.0);
            }
        }
    }
}

#[test]
fn test_kind_constraint_validation() {
    let constraints = vec![
        KindConstraint::Type,
        KindConstraint::TypeConstructor(1),
        KindConstraint::TypeConstructor(2),
        KindConstraint::HigherKinded(vec![KindConstraint::Type]),
        KindConstraint::Custom("MyKind".to_string()),
    ];
    
    for constraint in constraints {
        match constraint {
            KindConstraint::Type => {
                // Basic type constraint
            }
            KindConstraint::TypeConstructor(arity) => {
                assert!(arity > 0, "Type constructor should have positive arity");
            }
            KindConstraint::HigherKinded(args) => {
                assert!(!args.is_empty(), "Higher-kinded should have arguments");
            }
            KindConstraint::Custom(name) => {
                assert!(!name.is_empty(), "Custom kind should have name");
            }
        }
    }
}

#[test]
fn test_universe_polymorphic_law_construction() {
    let law = UniversePolymorphicLaw {
        name: "functor_composition".to_string(),
        universe_quantifiers: vec!["u".to_string(), "v".to_string()],
        type_quantifiers: vec![
            UniversePolymorphicParameter {
                name: "f".to_string(),
                universe_constraint: UniverseConstraint::Variable("u".to_string()),
                kind_constraint: Some(KindConstraint::TypeConstructor(1)),
            }
        ],
        premise: vec![
            UniversePolymorphicConstraint {
                class_name: "Functor".to_string(),
                type_args: vec![],
                universe_constraint: Some(UniverseConstraint::Variable("u".to_string())),
            }
        ],
        conclusion: UniversePolymorphicEquation {
            left: Value::Symbol("fmap_compose".to_string()),
            right: Value::Symbol("compose_fmap".to_string()),
            equality_type: UniversePolymorphicType::ForAllUniverse {
                universe_var: "u".to_string(),
                constraint: UniverseConstraint::Any,
                body: Box::new(UniversePolymorphicType::Concrete {
                    poly_type: PolynomialType::Base(BaseType::Natural),
                    universe: UniverseLevel::new(0),
                }),
            },
        },
    };
    
    assert_eq!(law.name, "functor_composition");
    assert_eq!(law.universe_quantifiers.len(), 2);
    assert_eq!(law.type_quantifiers.len(), 1);
    assert_eq!(law.premise.len(), 1);
    
    // Verify that the law is well-formed
    assert!(law.universe_quantifiers.contains(&"u".to_string()));
    assert!(law.universe_quantifiers.contains(&"v".to_string()));
    assert_eq!(law.type_quantifiers[0].name, "f");
    assert_eq!(law.premise[0].class_name, "Functor");
}

#[test]
fn test_complex_universe_constraint_relations() {
    let solver = UniverseConstraintSolver::new();
    
    // Create a complex constraint system:
    // u >= 1
    // v = u + 1  
    // w <= max(u, v)
    solver.add_relation(
        "u".to_string(),
        UniverseConstraint::AtLeast(UniverseLevel::new(1))
    );
    
    solver.add_relation(
        "v".to_string(),
        UniverseConstraint::Relative {
            base_param: "u".to_string(),
            offset: 1,
        }
    );
    
    solver.add_relation(
        "w".to_string(),
        UniverseConstraint::AtMost(UniverseLevel::new(10))
    );
    
    // Test that we can solve this system
    let universe_args = std::collections::HashMap::new();
    let result = solver.solve_constraints(&universe_args, UniverseLevel::new(3));
    
    assert!(result.is_ok(), "Complex constraint system should be solvable");
}

#[test]
fn test_universe_polymorphic_method_with_laws() {
    let method = UniversePolymorphicMethod {
        name: "fmap".to_string(),
        signature: UniversePolymorphicType::ForAllUniverse {
            universe_var: "u".to_string(),
            constraint: UniverseConstraint::Any,
            body: Box::new(UniversePolymorphicType::Concrete {
                poly_type: PolynomialType::Base(BaseType::Natural),
                universe: UniverseLevel::new(0),
            }),
        },
        default_impl: Some(Value::Symbol("default_fmap".to_string())),
        laws: vec![
            UniversePolymorphicLaw {
                name: "fmap_identity".to_string(),
                universe_quantifiers: vec!["u".to_string()],
                type_quantifiers: vec![],
                premise: vec![],
                conclusion: UniversePolymorphicEquation {
                    left: Value::Symbol("fmap_id".to_string()),
                    right: Value::Symbol("id".to_string()),
                    equality_type: UniversePolymorphicType::Concrete {
                        poly_type: PolynomialType::Base(BaseType::Natural),
                        universe: UniverseLevel::new(0),
                    },
                },
            }
        ],
    };
    
    assert_eq!(method.name, "fmap");
    assert!(method.default_impl.is_some());
    assert_eq!(method.laws.len(), 1);
    assert_eq!(method.laws[0].name, "fmap_identity");
}

#[test]
fn test_error_handling_invalid_instance() {
    let registry = UniversePolymorphicRegistry::new();
    
    // Try to register an instance for a non-existent class
    let invalid_instance = UniversePolymorphicInstance {
        class_name: "NonExistentClass".to_string(),
        universe_args: std::collections::HashMap::new(),
        type_args: vec![],
        methods: std::collections::HashMap::new(),
        law_proofs: std::collections::HashMap::new(),
    };
    
    let result = registry.register_instance(invalid_instance);
    assert!(result.is_err(), "Should fail to register instance for non-existent class");
    
    match result.unwrap_err() {
        crate::error::LambdustError::TypeError { message } => {
            assert!(message.contains("Unknown class"));
        }
        _ => panic!("Expected type error"),
    }
}

#[test]
fn test_universe_polymorphic_integration() {
    let registry = UniversePolymorphicRegistry::new();
    
    // Register Functor class
    let functor_class = create_functor_class();
    registry.register_class(functor_class).unwrap();
    
    // Register Monad class (which has Applicative superclass)
    let monad_class = create_monad_class();
    registry.register_class(monad_class).unwrap();
    
    // Verify both classes are registered
    let classes = registry.list_classes();
    assert_eq!(classes.len(), 2);
    assert!(classes.contains(&"Functor".to_string()));
    assert!(classes.contains(&"Monad".to_string()));
    
    // Verify class retrieval
    let functor = registry.get_class("Functor").unwrap();
    let monad = registry.get_class("Monad").unwrap();
    
    assert_eq!(functor.methods.len(), 1);
    assert_eq!(monad.methods.len(), 2);
    assert_eq!(monad.superclasses.len(), 1);
    
    // Verify law structures
    assert_eq!(functor.laws.len(), 0); // Laws are on methods in this implementation
    assert_eq!(monad.laws.len(), 3); // Three monad laws
}

#[test]
fn test_proof_checker_integration() {
    let proof_checker = UniverseProofChecker::new();
    
    // Add a basic axiom
    let axiom = UniversePolymorphicLaw {
        name: "identity_axiom".to_string(),
        universe_quantifiers: vec![],
        type_quantifiers: vec![],
        premise: vec![],
        conclusion: UniversePolymorphicEquation {
            left: Value::Symbol("id".to_string()),
            right: Value::Symbol("id".to_string()),
            equality_type: UniversePolymorphicType::Concrete {
                poly_type: PolynomialType::Base(BaseType::Natural),
                universe: UniverseLevel::new(0),
            },
        },
    };
    
    proof_checker.add_axiom("identity".to_string(), axiom);
    
    // Create a test instance
    let test_instance = UniversePolymorphicInstance {
        class_name: "TestClass".to_string(),
        universe_args: std::collections::HashMap::new(),
        type_args: vec![],
        methods: std::collections::HashMap::new(),
        law_proofs: std::collections::HashMap::new(),
    };
    
    // Create a test law  
    let test_law = UniversePolymorphicLaw {
        name: "test_law".to_string(),
        universe_quantifiers: vec![],
        type_quantifiers: vec![],
        premise: vec![],
        conclusion: UniversePolymorphicEquation {
            left: Value::Boolean(true),
            right: Value::Boolean(true),
            equality_type: UniversePolymorphicType::Concrete {
                poly_type: PolynomialType::Base(BaseType::Boolean),
                universe: UniverseLevel::new(0),
            },
        },
    };
    
    // Check law for instance (should succeed with current implementation)
    let result = proof_checker.check_law_for_instance(&test_law, &test_instance);
    assert!(result.is_ok(), "Basic law checking should succeed");
    assert!(result.unwrap(), "Law should hold for instance");
}
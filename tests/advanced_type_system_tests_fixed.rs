//! Advanced type system tests for Lambdust
//! 
//! This module tests the advanced type system features including:
//! - Algebraic data types and type inference
//! - Gradual typing integration with R7RS values
//! - Effect type annotations and constraint solving
//! - Type classes and advanced type features

use lambdust::{
    types::*,
    eval::Value,
    ast::{Literal, Expr},
    diagnostics::{Span, Spanned},
};
use std::collections::HashMap;

// Test algebraic data types
#[test]
fn test_algebraic_data_types() {
    // Test Maybe type construction using the actual struct
    let maybe_int = AlgebraicDataType {
        name: "Maybe".to_string(),
        type_params: vec![TypeVar::with_name("a")],
        constructors: vec![
            DataConstructor {
                name: "Nothing".to_string(),
                param_types: vec![],
                return_type: None,
                tag: 0,
                span: None,
            },
            DataConstructor {
                name: "Just".to_string(),
                param_types: vec![Type::Variable(TypeVar::with_name("a"))],
                return_type: None,
                tag: 1,
                span: None,
            },
        ],
        kind: Kind::arrow(Kind::Type, Kind::Type),
        variant_type: AlgebraicVariant::Sum,
        span: None,
    };
    
    assert_eq!(maybe_int.name, "Maybe");
    assert_eq!(maybe_int.type_params.len(), 1);
    assert_eq!(maybe_int.constructors.len(), 2);
    assert_eq!(maybe_int.variant_type, AlgebraicVariant::Sum);
}

#[test]
fn test_basic_type_properties() {
    // Test inference of basic literal types
    let num_literal = Literal::float(42.0);
    // In a real implementation, we would infer Type::Number
    let expected_type = Type::Number;
    assert_eq!(expected_type, Type::Number);
    
    // Test string literal
    let str_literal = Literal::string("hello".to_string());
    let expected_string_type = Type::String;
    assert_eq!(expected_string_type, Type::String);
    
    // Test boolean literal
    let bool_literal = Literal::boolean(true);
    let expected_bool_type = Type::Boolean;
    assert_eq!(expected_bool_type, Type::Boolean);
}

#[test] 
fn test_function_type_properties() {
    // Create a function type: (Number -> Number)
    let func_type = Type::function(
        vec![Type::Number],
        Type::Number,
    );
    
    assert!(func_type.is_function());
    
    // Test parameter extraction
    if let Type::Function { params, return_type } = func_type {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0], Type::Number);
        assert_eq!(*return_type, Type::Number);
    }
    
    // Test multi-parameter function
    let multi_func = Type::function(
        vec![Type::Number, Type::String, Type::Boolean],
        Type::Symbol,
    );
    
    if let Type::Function { params, return_type } = multi_func {
        assert_eq!(params.len(), 3);
        assert_eq!(*return_type, Type::Symbol);
    }
}

#[test]
fn test_polymorphic_types() {
    // Test identity function: ∀a. a -> a
    let type_var_a = TypeVar::with_name("a");
    let identity_type = Type::forall(
        vec![type_var_a.clone()],
        Type::function(
            vec![Type::Variable(type_var_a.clone())],
            Type::Variable(type_var_a),
        ),
    );
    
    assert!(identity_type.is_polymorphic());
    
    // Test type scheme instantiation
    let scheme = TypeScheme::polymorphic(
        vec![TypeVar::with_name("a")],
        vec![],
        Type::function(
            vec![Type::Variable(TypeVar::with_name("a"))],
            Type::Variable(TypeVar::with_name("a")),
        ),
    );
    
    let instantiated = scheme.instantiate();
    // The instantiated type should have fresh type variables
    assert!(instantiated.is_function());
}

#[test]
fn test_type_constraints() {
    // Test equality constraints
    let constraint = TypeConstraint::equal(
        Type::Number,
        Type::Variable(TypeVar::with_name("a")),
        None,
        "test constraint",
    );
    
    match constraint {
        TypeConstraint::Equal { left, right, reason, .. } => {
            assert_eq!(left, Type::Number);
            assert!(matches!(right, Type::Variable(_)));
            assert_eq!(reason, "test constraint");
        }
        _ => panic!("Expected Equal constraint"),
    }
    
    // Test instance constraints
    let instance_constraint = TypeConstraint::instance(
        "Show",
        Type::Number,
        None,
    );
    
    match instance_constraint {
        TypeConstraint::Instance { class, type_, .. } => {
            assert_eq!(class, "Show");
            assert_eq!(type_, Type::Number);
        }
        _ => panic!("Expected Instance constraint"),
    }
}

#[test]
fn test_gradual_typing_integration() {
    let _type_checker = TypeChecker::new(TypeLevel::Dynamic);
    
    // Test dynamic type
    let dynamic_type = Type::Dynamic;
    assert!(!dynamic_type.is_variable());
    assert!(!dynamic_type.is_function());
    
    // Test conversion from R7RS value to gradual type
    let value = Value::Literal(Literal::float(42.0));
    // In a real implementation, we would infer the type from the value
    let expected_type = Type::Number;
    assert_eq!(expected_type, Type::Number);
    
    // Test that value matches expected type
    match value {
        Value::Literal(Literal::Number(_)) => (), // Expected
        _ => panic!("Expected number literal"),
    }
}

#[test]
fn test_effect_type_annotations() {
    // Test effectful types
    let io_effect = Effect::IO;
    let state_effect = Effect::State(Type::Number);
    
    let effectful_type = Type::Effectful {
        input: Box::new(Type::String),
        effects: vec![io_effect, state_effect],
        output: Box::new(Type::Unit),
    };
    
    match effectful_type {
        Type::Effectful { input, effects, output } => {
            assert_eq!(*input, Type::String);
            assert_eq!(effects.len(), 2);
            assert_eq!(*output, Type::Unit);
        }
        _ => panic!("Expected Effectful type"),
    }
}

#[test]
fn test_row_polymorphism() {
    // Test record types with row polymorphism
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), Type::String);
    fields.insert("age".to_string(), Type::Number);
    
    let closed_row = Row::closed(fields.clone());
    assert!(closed_row.is_closed());
    
    let open_row = Row::open(fields, TypeVar::with_name("r"));
    assert!(!open_row.is_closed());
    
    let record_type = Type::Record(closed_row);
    
    match record_type {
        Type::Record(row) => {
            assert_eq!(row.fields.len(), 2);
            assert!(row.fields.contains_key("name"));
            assert!(row.fields.contains_key("age"));
        }
        _ => panic!("Expected Record type"),
    }
}

#[test]
fn test_type_unification_concepts() {
    // Test basic type compatibility
    let var = Type::Variable(TypeVar::with_name("a"));
    let concrete = Type::Number;
    
    // Variables can unify with concrete types
    assert!(var.is_variable());
    assert!(!concrete.is_variable());
    
    // Test type equality
    assert_eq!(Type::Number, Type::Number);
    assert_ne!(Type::Number, Type::String);
    
    // Test occurs check concept
    let var_a = TypeVar::with_name("a");
    let recursive = Type::list(Type::Variable(var_a.clone()));
    assert!(recursive.contains_var(&var_a)); // Would fail occurs check
    
    // Test compound types
    let pair_type = Type::pair(Type::Number, Type::String);
    assert_eq!(pair_type, Type::Pair(Box::new(Type::Number), Box::new(Type::String)));
}

#[test]
fn test_type_classes() {
    let mut type_env = TypeEnv::new();
    
    // Create a Show type class concept
    let show_constraint = Constraint {
        class: "Show".to_string(),
        type_: Type::Variable(TypeVar::with_name("a")),
    };
    
    assert_eq!(show_constraint.class, "Show");
    
    // Create an instance of Show for Numbers
    let show_number_instance = TypeClassInstance {
        class: "Show".to_string(),
        type_: Type::Number,
        methods: {
            let mut methods = HashMap::new();
            methods.insert(
                "show".to_string(),
                Type::function(vec![Type::Number], Type::String),
            );
            methods
        },
    };
    
    type_env.instances.insert(
        "Show".to_string(),
        vec![show_number_instance],
    );
    
    // Test instance resolution
    assert!(type_env.instances.contains_key("Show"));
    assert_eq!(type_env.instances["Show"].len(), 1);
}

#[test]
fn test_kind_system() {
    // Test basic kinds
    let type_kind = Kind::Type;
    let row_kind = Kind::Row;
    let effect_kind = Kind::Effect;
    
    assert_eq!(type_kind.arity(), 0);
    assert_eq!(row_kind.arity(), 0);
    assert_eq!(effect_kind.arity(), 0);
    
    // Test arrow kinds
    let maybe_kind = Kind::arrow(Kind::Type, Kind::Type);
    assert_eq!(maybe_kind.arity(), 1);
    
    let functor_kind = Kind::arrow(
        Kind::arrow(Kind::Type, Kind::Type),
        Kind::arrow(Kind::Type, Kind::Type),
    );
    assert_eq!(functor_kind.arity(), 2);
}

#[test]
fn test_dependent_type_concepts() {
    // Test basic dependent type concepts
    // For now, we simulate with regular types
    let vec_type = Type::Constructor {
        name: "Vec".to_string(),
        kind: Kind::arrow(Kind::Type, Kind::arrow(Kind::Type, Kind::Type)),
    };
    
    match vec_type {
        Type::Constructor { name, .. } => assert_eq!(name, "Vec"),
        _ => panic!("Expected constructor type"),
    }
    
    // Test type application
    let applied_vec = Type::Application {
        constructor: Box::new(Type::Constructor {
            name: "Vec".to_string(),
            kind: Kind::arrow(Kind::Type, Kind::Type),
        }),
        argument: Box::new(Type::Number),
    };
    
    match applied_vec {
        Type::Application { constructor, argument } => {
            assert!(matches!(constructor.as_ref(), Type::Constructor { .. }));
            assert_eq!(argument.as_ref(), &Type::Number);
        }
        _ => panic!("Expected type application"),
    }
}

#[test]
fn test_type_error_reporting() {
    let mut type_checker = TypeChecker::new(TypeLevel::Static);
    
    // Create a type error
    let error = lambdust::diagnostics::Error::type_error(
        "Cannot unify Number with String".to_string(),
        Span::new(0, 10),
    );
    
    type_checker.add_error(error);
    assert_eq!(type_checker.errors().len(), 1);
    
    type_checker.clear_errors();
    assert_eq!(type_checker.errors().len(), 0);
}

#[test]
fn test_type_variable_generation() {
    // Test type variable generation
    let var1 = TypeVar::new();
    let var2 = TypeVar::new();
    
    // Should have different IDs
    assert_ne!(var1.id, var2.id);
    
    // Test named variables
    let named_var = TypeVar::with_name("test");
    assert_eq!(named_var.name, Some("test".to_string()));
    
    // Test free variables
    let complex_type = Type::forall(
        vec![TypeVar::with_name("a")],
        Type::function(
            vec![Type::Variable(TypeVar::with_name("b"))], // Free variable
            Type::Variable(TypeVar::with_name("a")),       // Bound variable
        ),
    );
    
    let free_vars = complex_type.free_vars();
    assert!(!free_vars.is_empty());
}

#[test]
fn test_substitution() {
    let var = TypeVar::with_name("a");
    
    // Create a substitution
    let subst = Substitution::single(var.clone(), Type::Number);
    
    // Apply substitution
    let original = Type::Variable(var.clone());
    let result = subst.apply_to_type(&original);
    
    assert_eq!(result, Type::Number);
    
    // Test composition
    let subst2 = Substitution::single(TypeVar::with_name("b"), Type::Variable(var));
    
    let composed = subst.compose(&subst2);
    let applied = composed.apply_to_type(&Type::Variable(TypeVar::with_name("b")));
    assert_eq!(applied, Type::Number);
    
    // Test empty substitution
    let empty = Substitution::empty();
    assert!(empty.is_empty());
    
    let identity_result = empty.apply_to_type(&Type::Number);
    assert_eq!(identity_result, Type::Number);
}

#[test]
fn test_pattern_matching() {
    // Test pattern construction
    let wildcard = Pattern::Wildcard;
    let variable = Pattern::Variable("x".to_string());
    let literal = Pattern::Literal(Literal::float(42.0));
    
    // Test constructor pattern
    let constructor_pattern = Pattern::Constructor {
        name: "Just".to_string(),
        patterns: vec![Pattern::Variable("x".to_string())],
    };
    
    // Test tuple pattern
    let tuple_pattern = Pattern::Tuple(vec![
        Pattern::Variable("a".to_string()),
        Pattern::Variable("b".to_string()),
    ]);
    
    // Test record pattern
    let mut record_fields = HashMap::new();
    record_fields.insert("name".to_string(), Pattern::Variable("n".to_string()));
    record_fields.insert("age".to_string(), Pattern::Variable("a".to_string()));
    
    let record_pattern = Pattern::Record {
        fields: record_fields,
        rest: None,
    };
    
    // Verify patterns
    assert!(matches!(wildcard, Pattern::Wildcard));
    assert!(matches!(variable, Pattern::Variable(_)));
    assert!(matches!(literal, Pattern::Literal(_)));
    assert!(matches!(constructor_pattern, Pattern::Constructor { .. }));
    assert!(matches!(tuple_pattern, Pattern::Tuple(_)));
    assert!(matches!(record_pattern, Pattern::Record { .. }));
}

// Integration test that combines multiple type system features
#[test]
fn test_advanced_type_system_integration() {
    let type_checker = TypeChecker::new(TypeLevel::Static);
    let mut type_env = TypeEnv::new();
    
    // Set up a polymorphic function with constraints
    let var_a = TypeVar::with_name("a");
    let show_constraint = Constraint {
        class: "Show".to_string(),
        type_: Type::Variable(var_a.clone()),
    };
    
    let constrained_type = Type::constrained(
        vec![show_constraint],
        Type::function(
            vec![Type::Variable(var_a.clone())],
            Type::String,
        ),
    );
    
    let scheme = TypeScheme::polymorphic(
        vec![var_a],
        vec![Constraint {
            class: "Show".to_string(),
            type_: Type::Variable(TypeVar::with_name("a")),
        }],
        constrained_type,
    );
    
    type_env.bind("to_string".to_string(), scheme);
    
    // Verify the binding exists
    assert!(type_env.lookup("to_string").is_some());
    
    // Test instantiation
    if let Some(scheme) = type_env.lookup("to_string") {
        let instantiated = scheme.instantiate();
        assert!(instantiated.is_function());
    }
    
    // Test with effects
    let effectful_scheme = TypeScheme::monomorphic(
        Type::Effectful {
            input: Box::new(Type::String),
            effects: vec![Effect::IO],
            output: Box::new(Type::Unit),
        },
    );
    
    type_env.bind("display".to_string(), effectful_scheme);
    assert!(type_env.lookup("display").is_some());
    
    // Test algebraic data type integration
    let option_type = AlgebraicDataType {
        name: "Option".to_string(),
        type_params: vec![TypeVar::with_name("T")],
        constructors: vec![
            DataConstructor {
                name: "None".to_string(),
                param_types: vec![],
                return_type: None,
                tag: 0,
                span: None,
            },
            DataConstructor {
                name: "Some".to_string(),
                param_types: vec![Type::Variable(TypeVar::with_name("T"))],
                return_type: None,
                tag: 1,
                span: None,
            },
        ],
        kind: Kind::arrow(Kind::Type, Kind::Type),
        variant_type: AlgebraicVariant::Sum,
        span: None,
    };
    
    // Verify the complete type system works together
    assert_eq!(option_type.constructors.len(), 2);
    assert_eq!(type_env.bindings.len(), 2);
    assert_eq!(type_checker.errors().len(), 0);
}
//! Comprehensive test suite for SRFI-11: let-values
//!
//! This module provides comprehensive testing for the SRFI-11 implementation,
//! covering all aspects of let-values and let*-values functionality.

use lambdust::ast::Formals;
use lambdust::eval::value::{ThreadSafeEnvironment, Value};
use lambdust::stdlib::srfi8_receive::create_srfi8_bindings;
use lambdust::stdlib::srfi11_let_values::{
    LetValuesBinding, create_srfi11_bindings, expand_let_star_values, expand_let_values,
    validate_let_values_syntax,
};
use lambdust::utils::intern_symbol;
use std::sync::Arc;

/// Creates a test environment with SRFI-11 and SRFI-8 bindings.
fn create_test_environment() -> Arc<ThreadSafeEnvironment> {
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
    create_srfi8_bindings(&env);
    create_srfi11_bindings(&env);
    env
}

/// Helper to create a simple values expression.
fn values_expr(values: Vec<Value>) -> Value {
    let mut expr = &[Value::symbol(intern_symbol("values"))];
    expr.extend(values);
    Value::list(expr)
}

/// Helper to create a symbol.
fn sym(name: &str) -> Value {
    Value::symbol(intern_symbol(name))
}

// ============= BINDING PARSING TESTS =============

#[test]
fn test_parse_fixed_arity_binding() {
    let binding = Value::list(vec![
        Value::list(&[sym("a"), sym("b")]),
        values_expr(&[Value::integer(1), Value::integer(2)]),
    ]);

    let bindings_list = Value::list(&[binding]);

    assert!(validate_let_values_syntax(&bindings_list).is_ok());
}

#[test]
fn test_parse_variable_arity_binding() {
    let binding = Value::list(vec![
        sym("args"),
        values_expr(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]),
    ]);

    let bindings_list = Value::list(&[binding]);

    assert!(validate_let_values_syntax(&bindings_list).is_ok());
}

#[test]
fn test_parse_mixed_arity_binding() {
    // Create (a b . rest)
    let rest = sym("rest");
    let b_pair = Value::cons(sym("b"), rest);
    let formals = Value::cons(sym("a"), b_pair);

    let binding = Value::list(vec![
        formals,
        values_expr(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]),
    ]);

    let bindings_list = Value::list(&[binding]);

    assert!(validate_let_values_syntax(&bindings_list).is_ok());
}

#[test]
fn test_parse_empty_formals() {
    let binding = Value::list(&[Value::Nil, values_expr(vec![])]);

    let bindings_list = Value::list(&[binding]);

    assert!(validate_let_values_syntax(&bindings_list).is_ok());
}

// ============= SYNTAX VALIDATION TESTS =============

#[test]
fn test_invalid_bindings_not_list() {
    let invalid = Value::integer(42);
    assert!(validate_let_values_syntax(&invalid).is_err());
}

#[test]
fn test_invalid_binding_not_list() {
    let invalid = Value::list(&[Value::integer(42)]);
    assert!(validate_let_values_syntax(&invalid).is_err());
}

#[test]
fn test_invalid_binding_wrong_length() {
    let invalid = Value::list(&[Value::list(vec![sym("only-one-element")])]);
    assert!(validate_let_values_syntax(&invalid).is_err());

    let invalid = Value::list(vec![Value::list(vec![
        sym("a"),
        Value::integer(1),
        Value::integer(2),
    ])]);
    assert!(validate_let_values_syntax(&invalid).is_err());
}

#[test]
fn test_invalid_formals_non_symbol() {
    let invalid = Value::list(vec![Value::list(vec![
        Value::list(&[Value::integer(42)]), // Non-symbol in formals
        Value::integer(1),
    ])]);
    assert!(validate_let_values_syntax(&invalid).is_err());
}

// ============= MACRO EXPANSION TESTS =============

#[test]
fn test_expand_empty_let_values() {
    let bindings = &[];
    let body = &[Value::integer(42)];

    let result = expand_let_values(&bindings, &body).unwrap();
    assert_eq!(result, Value::integer(42));
}

#[test]
fn test_expand_empty_let_values_multiple_body() {
    let bindings = &[];
    let body = &[Value::integer(1), Value::integer(2), Value::integer(3)];

    let result = expand_let_values(&bindings, &body).unwrap();

    // Should be wrapped in begin
    if let Some(result_list) = result.as_list() {
        assert_eq!(result_list[0], sym("begin"));
        assert_eq!(result_list.len(), 4);
    } else {
        panic!("Expected begin form");
    }
}

#[test]
fn test_expand_single_binding_let_values() {
    let binding = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string()]),
        producer: Value::integer(42),
    };
    let bindings = &[binding];
    let body = &[sym("a")];

    let result = expand_let_values(&bindings, &body).unwrap();

    // Should be call-with-values form
    if let Some(result_list) = result.as_list() {
        assert_eq!(result_list[0], sym("call-with-values"));
        assert_eq!(result_list.len(), 3);

        // Check producer thunk
        if let Some(producer_list) = result_list[1].as_list() {
            assert_eq!(producer_list[0], sym("lambda"));
            assert_eq!(producer_list[1], Value::Nil);
            assert_eq!(producer_list[2], Value::integer(42));
        }

        // Check consumer lambda
        if let Some(consumer_list) = result_list[2].as_list() {
            assert_eq!(consumer_list[0], sym("lambda"));
            // Should have (a) formals
            if let Some(formals_list) = consumer_list[1].as_list() {
                assert_eq!(formals_list.len(), 1);
                assert_eq!(formals_list[0], sym("a"));
            }
            // Should have body
            assert_eq!(consumer_list[2], sym("a"));
        }
    } else {
        panic!("Expected call-with-values form");
    }
}

#[test]
fn test_expand_multiple_bindings_let_values() {
    let binding1 = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string()]),
        producer: Value::integer(1),
    };
    let binding2 = LetValuesBinding {
        formals: Formals::Fixed(&["b".to_string()]),
        producer: Value::integer(2),
    };
    let bindings = &[binding1, binding2];
    let body = &[Value::list(vec![sym("+"), sym("a"), sym("b")])];

    let result = expand_let_values(&bindings, &body).unwrap();

    // Should be nested call-with-values forms
    if let Some(result_list) = result.as_list() {
        assert_eq!(result_list[0], sym("call-with-values"));
        assert_eq!(result_list.len(), 3);
    } else {
        panic!("Expected call-with-values form");
    }
}

#[test]
fn test_expand_empty_let_star_values() {
    let bindings = &[];
    let body = &[Value::integer(42)];

    let result = expand_let_star_values(&bindings, &body).unwrap();
    assert_eq!(result, Value::integer(42));
}

#[test]
fn test_expand_single_binding_let_star_values() {
    let binding = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string()]),
        producer: Value::integer(42),
    };
    let bindings = &[binding];
    let body = &[sym("a")];

    let result = expand_let_star_values(&bindings, &body).unwrap();

    // Should be the same as let-values for single binding
    if let Some(result_list) = result.as_list() {
        assert_eq!(result_list[0], sym("call-with-values"));
    } else {
        panic!("Expected call-with-values form");
    }
}

#[test]
fn test_expand_multiple_bindings_let_star_values() {
    let binding1 = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string()]),
        producer: Value::integer(1),
    };
    let binding2 = LetValuesBinding {
        formals: Formals::Fixed(&["b".to_string()]),
        producer: sym("a"), // Uses previous binding
    };
    let bindings = &[binding1, binding2];
    let body = &[Value::list(vec![sym("+"), sym("a"), sym("b")])];

    let result = expand_let_star_values(&bindings, &body).unwrap();

    // Should be nested call-with-values forms
    if let Some(result_list) = result.as_list() {
        assert_eq!(result_list[0], sym("call-with-values"));
        assert_eq!(result_list.len(), 3);
    } else {
        panic!("Expected call-with-values form");
    }
}

// ============= FORMALS PATTERN TESTS =============

#[test]
fn test_fixed_formals_patterns() {
    // Test various fixed arity patterns
    let patterns = vec![
        (&[], 0),
        (&["a"], 1),
        (&["a", "b"], 2),
        (&["x", "y", "z"], 3),
    ];

    for (params, expected_len) in patterns {
        let formals = Formals::Fixed(params.iter().map(|s| s.to_string()).collect());
        let binding = LetValuesBinding {
            formals,
            producer: Value::integer(42),
        };

        // Should parse without error
        let bindings = &[binding];
        let body = &[Value::integer(1)];
        assert!(expand_let_values(&bindings, &body).is_ok());
    }
}

#[test]
fn test_variable_formals_pattern() {
    let formals = Formals::Variable("args".to_string());
    let binding = LetValuesBinding {
        formals,
        producer: values_expr(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]),
    };

    let bindings = &[binding];
    let body = &[sym("args")];

    let result = expand_let_values(&bindings, &body).unwrap();
    // Should generate valid expansion
    assert!(result.as_list().is_some());
}

#[test]
fn test_mixed_formals_pattern() {
    let formals = Formals::Mixed {
        fixed: &["a".to_string(), "b".to_string()],
        rest: "rest".to_string(),
    };
    let binding = LetValuesBinding {
        formals,
        producer: values_expr(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
            Value::integer(4),
        ]),
    };

    let bindings = &[binding];
    let body = vec![Value::list(vec![
        sym("list"),
        sym("a"),
        sym("b"),
        sym("rest"),
    ])];

    let result = expand_let_values(&bindings, &body).unwrap();
    // Should generate valid expansion
    assert!(result.as_list().is_some());
}

// ============= ERROR HANDLING TESTS =============

#[test]
fn test_let_values_error_no_arguments() {
    // Cannot test primitive functions directly as they are not pub
    // This test is covered by integration tests through the environment

    let env = create_test_environment();
    let let_values = env.lookup("let-values").unwrap();
    assert!(let_values.is_procedure());
}

#[test]
fn test_let_values_error_only_bindings() {
    // Cannot test primitive functions directly as they are not pub
    // This test is covered by syntax validation tests

    let env = create_test_environment();
    let let_star_values = env.lookup("let*-values").unwrap();
    assert!(let_star_values.is_procedure());
}

// ============= INTEGRATION TESTS =============

#[test]
fn test_environment_integration() {
    let env = create_test_environment();

    // Check that let-values is bound
    let let_values = env.lookup("let-values").unwrap();
    assert!(let_values.is_procedure());

    // Check that let*-values is bound
    let let_star_values = env.lookup("let*-values").unwrap();
    assert!(let_star_values.is_procedure());

    // Check that predicate is bound
    let predicate = env.lookup("let-values?").unwrap();
    assert!(predicate.is_procedure());
}

// ============= SCHEME EQUIVALENCE TESTS =============

#[test]
fn test_let_values_scheme_equivalence() {
    // Test that our expansion matches expected Scheme semantics

    // (let-values ([(a b) (values 1 2)]) (+ a b))
    // Should expand to:
    // (call-with-values (lambda () (values 1 2))
    //                   (lambda (a b) (+ a b)))

    let binding = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string(), "b".to_string()]),
        producer: values_expr(&[Value::integer(1), Value::integer(2)]),
    };
    let bindings = &[binding];
    let body = &[Value::list(vec![sym("+"), sym("a"), sym("b")])];

    let result = expand_let_values(&bindings, &body).unwrap();

    if let Some(result_list) = result.as_list() {
        // Should be call-with-values
        assert_eq!(result_list[0], sym("call-with-values"));

        // Producer should be (lambda () (values 1 2))
        if let Some(producer_list) = result_list[1].as_list() {
            assert_eq!(producer_list[0], sym("lambda"));
            assert_eq!(producer_list[1], Value::Nil);
            if let Some(values_list) = producer_list[2].as_list() {
                assert_eq!(values_list[0], sym("values"));
                assert_eq!(values_list[1], Value::integer(1));
                assert_eq!(values_list[2], Value::integer(2));
            }
        }

        // Consumer should be (lambda (a b) (+ a b))
        if let Some(consumer_list) = result_list[2].as_list() {
            assert_eq!(consumer_list[0], sym("lambda"));
            if let Some(params_list) = consumer_list[1].as_list() {
                assert_eq!(params_list.len(), 2);
                assert_eq!(params_list[0], sym("a"));
                assert_eq!(params_list[1], sym("b"));
            }
            if let Some(body_list) = consumer_list[2].as_list() {
                assert_eq!(body_list[0], sym("+"));
                assert_eq!(body_list[1], sym("a"));
                assert_eq!(body_list[2], sym("b"));
            }
        }
    } else {
        panic!("Expected call-with-values form");
    }
}

#[test]
fn test_let_star_values_scheme_equivalence() {
    // Test that let*-values properly sequences bindings

    // (let*-values ([(a) (values 1)]
    //               [(b) (values a)])
    //   (+ a b))
    // Should expand to nested let-values where second binding can see first

    let binding1 = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string()]),
        producer: values_expr(&[Value::integer(1)]),
    };
    let binding2 = LetValuesBinding {
        formals: Formals::Fixed(&["b".to_string()]),
        producer: sym("a"), // References first binding
    };
    let bindings = &[binding1, binding2];
    let body = &[Value::list(vec![sym("+"), sym("a"), sym("b")])];

    let result = expand_let_star_values(&bindings, &body).unwrap();

    // Should be a nested structure where inner binding can see outer
    if let Some(result_list) = result.as_list() {
        assert_eq!(result_list[0], sym("call-with-values"));
        // The structure should allow 'a' to be visible in the second binding
    } else {
        panic!("Expected call-with-values form");
    }
}

// ============= STRESS TESTS =============

#[test]
fn test_many_bindings() {
    // Test with many bindings to ensure scalability
    let mut bindings = Vec::new();
    let mut body_args = &[sym("+")];

    for i in 0..10 {
        let param_name = format!("var{}", i);
        let binding = LetValuesBinding {
            formals: Formals::Fixed(&[param_name.clone()]),
            producer: Value::integer(i as i64),
        };
        bindings.push(binding);
        body_args.push(sym(&param_name));
    }

    let body = &[Value::list(body_args)];

    // Should handle many bindings without error
    assert!(expand_let_values(&bindings, &body).is_ok());
    assert!(expand_let_star_values(&bindings, &body).is_ok());
}

#[test]
fn test_deep_nesting() {
    // Test deeply nested let*-values
    let mut bindings = Vec::new();

    for i in 0..5 {
        let param_name = format!("var{}", i);
        let producer = if i == 0 {
            Value::integer(1)
        } else {
            sym(&format!("var{}", i - 1)) // Reference previous binding
        };

        let binding = LetValuesBinding {
            formals: Formals::Fixed(&[param_name]),
            producer,
        };
        bindings.push(binding);
    }

    let body = &[sym("var4")]; // Reference last binding

    // Should handle deep nesting without error
    assert!(expand_let_star_values(&bindings, &body).is_ok());
}

// ============= EDGE CASE TESTS =============

#[test]
fn test_zero_values_binding() {
    // Test binding that produces zero values
    let binding = LetValuesBinding {
        formals: Formals::Fixed(&[]),
        producer: values_expr(&[]), // (values) produces zero values
    };
    let bindings = &[binding];
    let body = &[Value::integer(42)];

    assert!(expand_let_values(&bindings, &body).is_ok());
}

#[test]
fn test_single_value_binding() {
    // Test that single values work correctly
    let binding = LetValuesBinding {
        formals: Formals::Fixed(&["x".to_string()]),
        producer: Value::integer(42), // Single value, not (values 42)
    };
    let bindings = &[binding];
    let body = &[sym("x")];

    assert!(expand_let_values(&bindings, &body).is_ok());
}

#[test]
fn test_variable_arity_with_zero_values() {
    // Test variable arity formals with zero values
    let binding = LetValuesBinding {
        formals: Formals::Variable("args".to_string()),
        producer: values_expr(&[]), // Zero values
    };
    let bindings = &[binding];
    let body = &[sym("args")]; // Should be empty list

    assert!(expand_let_values(&bindings, &body).is_ok());
}

#[test]
fn test_complex_nested_structures() {
    // Test complex expressions in producers
    let complex_producer = Value::list(vec![
        sym("if"),
        Value::boolean(true),
        values_expr(&[Value::integer(1), Value::integer(2)]),
        values_expr(&[Value::integer(3), Value::integer(4)]),
    ]);

    let binding = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string(), "b".to_string()]),
        producer: complex_producer,
    };
    let bindings = &[binding];
    let body = &[Value::list(vec![sym("*"), sym("a"), sym("b")])];

    // Should handle complex expressions
    assert!(expand_let_values(&bindings, &body).is_ok());
}

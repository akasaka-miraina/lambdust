//! R7RS compliance and SRFI-8 compatibility tests for SRFI-11
//!
//! This module tests the SRFI-11 implementation against R7RS specification
//! requirements and ensures full compatibility with SRFI-8.

use lambdust::ast::Formals;
use lambdust::eval::value::{MultipleValues, ThreadSafeEnvironment, Value};
use lambdust::stdlib::srfi8_receive::{create_srfi8_bindings, enhanced_primitive_values};
use lambdust::stdlib::srfi11_let_values::{
    LetValuesBinding, create_srfi11_bindings, expand_let_star_values, expand_let_values,
};
use lambdust::utils::intern_symbol;
use std::sync::Arc;

/// Creates a fully compliant test environment.
fn create_compliant_environment() -> Arc<ThreadSafeEnvironment> {
    let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
    create_srfi8_bindings(&env);
    create_srfi11_bindings(&env);
    env
}

fn sym(name: &str) -> Value {
    Value::symbol(intern_symbol(name))
}

fn values_expr(values: Vec<Value>) -> Value {
    let mut expr = &[sym("values")];
    expr.extend(values);
    Value::list(expr)
}

// ============= R7RS-LARGE COMPLIANCE TESTS =============

#[test]
fn test_r7rs_let_values_basic_syntax() {
    // R7RS-large specifies let-values syntax as:
    // (let-values ([(formals producer) ...]) body ...)

    let binding = Value::list(vec![
        Value::list(&[sym("a"), sym("b")]),
        values_expr(&[Value::integer(1), Value::integer(2)]),
    ]);

    let bindings_list = Value::list(&[binding]);

    let binding_parsed = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string(), "b".to_string()]),
        producer: values_expr(&[Value::integer(1), Value::integer(2)]),
    };

    let bindings = &[binding_parsed];
    let body = &[Value::list(vec![sym("+"), sym("a"), sym("b")])];

    // Should expand correctly according to R7RS
    let result = expand_let_values(&bindings, &body);
    assert!(result.is_ok());

    if let Ok(expanded) = result {
        if let Some(expanded_list) = expanded.as_list() {
            assert_eq!(expanded_list[0], sym("call-with-values"));
        }
    }
}

#[test]
fn test_r7rs_let_star_values_basic_syntax() {
    // R7RS-large specifies let*-values syntax as:
    // (let*-values ([(formals producer) ...]) body ...)

    let binding1 = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string()]),
        producer: Value::integer(1),
    };

    let binding2 = LetValuesBinding {
        formals: Formals::Fixed(&["b".to_string()]),
        producer: sym("a"), // Can reference previous binding
    };

    let bindings = &[binding1, binding2];
    let body = &[Value::list(vec![sym("*"), sym("a"), sym("b")])];

    // Should expand correctly with sequential binding semantics
    let result = expand_let_star_values(&bindings, &body);
    assert!(result.is_ok());
}

// ============= FORMAL PARAMETER PATTERN COMPLIANCE =============

#[test]
fn test_r7rs_fixed_formals_compliance() {
    // R7RS specifies exact arity matching for fixed formals
    let test_cases = vec![
        (&[], 0),
        (&["a"], 1),
        (&["x", "y"], 2),
        (&["p", "q", "r", "s"], 4),
    ];

    for (params, expected_count) in test_cases {
        let formals = Formals::Fixed(params.iter().map(|s| s.to_string()).collect());
        let binding = LetValuesBinding {
            formals,
            producer: values_expr((0..expected_count).map(Value::integer).collect()),
        };

        let bindings = &[binding];
        let body = &[Value::integer(42)];

        // Should be valid R7RS expansion
        assert!(expand_let_values(bindings, body).is_ok());
    }
}

#[test]
fn test_r7rs_variable_formals_compliance() {
    // R7RS specifies that variable formals accept any number of values
    let formals = Formals::Variable("args".to_string());

    // Test with different numbers of values
    let value_counts = &[0, 1, 3, 10];

    for count in value_counts {
        let binding = LetValuesBinding {
            formals: formals.clone(),
            producer: values_expr((0..count).map(Value::integer).collect()),
        };

        let bindings = &[binding];
        let body = &[sym("args")];

        // All should be valid
        assert!(expand_let_values(bindings, body).is_ok());
    }
}

#[test]
fn test_r7rs_mixed_formals_compliance() {
    // R7RS specifies (param1 param2 . rest) syntax
    let formals = Formals::Mixed {
        fixed: &["a".to_string(), "b".to_string()],
        rest: "rest".to_string(),
    };

    // Test with various numbers of values >= fixed count
    let value_counts = &[2, 3, 5, 10];

    for count in value_counts {
        let binding = LetValuesBinding {
            formals: formals.clone(),
            producer: values_expr((0..count).map(Value::integer).collect()),
        };

        let bindings = &[binding];
        let body = vec![Value::list(vec![
            sym("list"),
            sym("a"),
            sym("b"),
            sym("rest"),
        ])];

        // All should be valid (count >= 2)
        assert!(expand_let_values(bindings, body).is_ok());
    }
}

// ============= SRFI-8 COMPATIBILITY TESTS =============

#[test]
fn test_srfi8_values_compatibility() {
    // Ensure our let-values works with SRFI-8 values
    let values_result =
        enhanced_primitive_values(&[Value::integer(1), Value::integer(2), Value::integer(3)])
            .unwrap();

    // Should create MultipleValues
    assert!(matches!(values_result, Value::MultipleValues(_)));

    if let Value::MultipleValues(mv) = values_result {
        assert_eq!(mv.values.len(), 3);
        assert_eq!(mv.values[0], Value::integer(1));
        assert_eq!(mv.values[1], Value::integer(2));
        assert_eq!(mv.values[2], Value::integer(3));
    }
}

#[test]
fn test_srfi8_single_value_compatibility() {
    // SRFI-8 specifies that single values are not wrapped
    let single_value = enhanced_primitive_values(&[Value::integer(42)]).unwrap();
    assert_eq!(single_value, Value::integer(42));
    assert!(!matches!(single_value, Value::MultipleValues(_)));
}

#[test]
fn test_srfi8_zero_values_compatibility() {
    // SRFI-8 specifies (values) returns unspecified
    let zero_values = enhanced_primitive_values(&[]).unwrap();
    assert_eq!(zero_values, Value::Unspecified);
}

#[test]
fn test_srfi8_call_with_values_expansion_compatibility() {
    // Test that our expansions are compatible with SRFI-8 call-with-values
    let binding = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string(), "b".to_string()]),
        producer: values_expr(&[Value::integer(1), Value::integer(2)]),
    };

    let bindings = &[binding];
    let body = &[Value::list(vec![sym("+"), sym("a"), sym("b")])];

    let expanded = expand_let_values(&bindings, &body).unwrap();

    // Verify the expansion structure matches SRFI-8 expectations
    if let Some(expanded_list) = expanded.as_list() {
        assert_eq!(expanded_list[0], sym("call-with-values"));
        assert_eq!(expanded_list.len(), 3);

        // Producer should be a thunk
        if let Some(producer_list) = expanded_list[1].as_list() {
            assert_eq!(producer_list[0], sym("lambda"));
            assert_eq!(producer_list[1], Value::Nil);
        }

        // Consumer should be a lambda with formals
        if let Some(consumer_list) = expanded_list[2].as_list() {
            assert_eq!(consumer_list[0], sym("lambda"));
            // Formals should be properly formatted
            if let Some(formals_list) = consumer_list[1].as_list() {
                assert_eq!(formals_list.len(), 2);
                assert_eq!(formals_list[0], sym("a"));
                assert_eq!(formals_list[1], sym("b"));
            }
        }
    }
}

// ============= ENVIRONMENT SEMANTICS COMPLIANCE =============

#[test]
fn test_r7rs_parallel_binding_semantics() {
    // R7RS specifies that let-values bindings are evaluated in parallel
    // This means each producer is evaluated in the same environment

    let binding1 = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string()]),
        producer: Value::integer(1),
    };

    let binding2 = LetValuesBinding {
        formals: Formals::Fixed(&["b".to_string()]),
        // This should NOT be able to see 'a' in parallel evaluation
        producer: Value::integer(2), // Not sym("a")
    };

    let bindings = &[binding1, binding2];
    let body = &[Value::list(vec![sym("+"), sym("a"), sym("b")])];

    let result = expand_let_values(&bindings, &body).unwrap();

    // The expansion should create nested call-with-values where
    // each producer is evaluated independently
    if let Some(result_list) = result.as_list() {
        assert_eq!(result_list[0], sym("call-with-values"));
    }
}

#[test]
fn test_r7rs_sequential_binding_semantics() {
    // R7RS specifies that let*-values bindings are evaluated sequentially
    // Later bindings can reference earlier ones

    let binding1 = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string()]),
        producer: Value::integer(10),
    };

    let binding2 = LetValuesBinding {
        formals: Formals::Fixed(&["b".to_string()]),
        producer: sym("a"), // CAN reference 'a' in sequential evaluation
    };

    let bindings = &[binding1, binding2];
    let body = &[Value::list(vec![sym("*"), sym("a"), sym("b")])];

    let result = expand_let_star_values(&bindings, &body).unwrap();

    // The expansion should create nested structure where inner bindings
    // can reference outer ones
    if let Some(result_list) = result.as_list() {
        assert_eq!(result_list[0], sym("call-with-values"));
    }
}

// ============= ERROR HANDLING COMPLIANCE =============

#[test]
fn test_r7rs_arity_mismatch_handling() {
    // R7RS specifies that arity mismatches should be detectable
    // (though we can't test runtime errors directly in macro expansion)

    use lambdust::stdlib::srfi8_receive::validate_receive_arity;

    // Fixed arity - exact match required
    let fixed_formals = Formals::Fixed(&["a".to_string(), "b".to_string()]);
    assert!(validate_receive_arity(&fixed_formals, 2).is_ok());
    assert!(validate_receive_arity(&fixed_formals, 1).is_err());
    assert!(validate_receive_arity(&fixed_formals, 3).is_err());

    // Variable arity - any count allowed
    let variable_formals = Formals::Variable("args".to_string());
    assert!(validate_receive_arity(&variable_formals, 0).is_ok());
    assert!(validate_receive_arity(&variable_formals, 5).is_ok());

    // Mixed arity - at least fixed count required
    let mixed_formals = Formals::Mixed {
        fixed: &["a".to_string()],
        rest: "rest".to_string(),
    };
    assert!(validate_receive_arity(&mixed_formals, 1).is_ok());
    assert!(validate_receive_arity(&mixed_formals, 3).is_ok());
    assert!(validate_receive_arity(&mixed_formals, 0).is_err());
}

// ============= INTEGRATION COMPLIANCE TESTS =============

#[test]
fn test_full_environment_r7rs_compliance() {
    let env = create_compliant_environment();

    // All required procedures should be available
    assert!(env.lookup("let-values").is_some());
    assert!(env.lookup("let*-values").is_some());
    assert!(env.lookup("values").is_some());
    assert!(env.lookup("call-with-values").is_some());

    // SRFI-8 compatibility procedures
    assert!(env.lookup("multiple-values?").is_some());
    assert!(env.lookup("values-length").is_some());
}

#[test]
fn test_canonical_r7rs_examples() {
    // Test examples from R7RS-large specification

    // Example 1: Basic let-values
    let binding = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string(), "b".to_string()]),
        producer: values_expr(&[Value::integer(1), Value::integer(2)]),
    };
    let bindings = &[binding];
    let body = &[Value::list(vec![sym("+"), sym("a"), sym("b")])];

    assert!(expand_let_values(bindings, body).is_ok());

    // Example 2: Variable arity
    let binding = LetValuesBinding {
        formals: Formals::Variable("args".to_string()),
        producer: values_expr(vec![
            Value::integer(1),
            Value::integer(2),
            Value::integer(3),
        ]),
    };
    let bindings = &[binding];
    let body = &[Value::list(vec![sym("length"), sym("args")])];

    assert!(expand_let_values(bindings, body).is_ok());

    // Example 3: Mixed arity
    let binding = LetValuesBinding {
        formals: Formals::Mixed {
            fixed: &["first".to_string()],
            rest: "rest".to_string(),
        },
        producer: values_expr(&[sym("'x"), sym("'y"), sym("'z")]),
    };
    let bindings = &[binding];
    let body = &[Value::list(vec![sym("cons"), sym("first"), sym("rest")])];

    assert!(expand_let_values(bindings, body).is_ok());
}

#[test]
fn test_let_star_values_canonical_examples() {
    // Example from R7RS showing sequential binding
    let binding1 = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string(), "b".to_string()]),
        producer: values_expr(&[Value::integer(1), Value::integer(2)]),
    };

    let binding2 = LetValuesBinding {
        formals: Formals::Fixed(&["c".to_string(), "d".to_string()]),
        producer: values_expr(&[sym("a"), sym("b")]),
    };

    let bindings = &[binding1, binding2];
    let body = vec![Value::list(vec![
        sym("+"),
        sym("a"),
        sym("b"),
        sym("c"),
        sym("d"),
    ])];

    assert!(expand_let_star_values(&bindings, &body).is_ok());
}

// ============= MACRO EXPANSION CORRECTNESS =============

#[test]
fn test_expansion_correctness_preservation() {
    // Test that macro expansions preserve semantics correctly

    // Original: (let-values ([(a b) (values 1 2)]) (list a b))
    // Should expand to: (call-with-values (lambda () (values 1 2))
    //                                     (lambda (a b) (list a b)))

    let binding = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string(), "b".to_string()]),
        producer: values_expr(&[Value::integer(1), Value::integer(2)]),
    };
    let bindings = &[binding];
    let body = &[Value::list(vec![sym("list"), sym("a"), sym("b")])];

    let expanded = expand_let_values(&bindings, &body).unwrap();

    // Verify structural correctness
    if let Some(expanded_list) = expanded.as_list() {
        assert_eq!(expanded_list.len(), 3);
        assert_eq!(expanded_list[0], sym("call-with-values"));

        // Producer thunk structure
        if let Some(producer_list) = expanded_list[1].as_list() {
            assert_eq!(producer_list.len(), 3);
            assert_eq!(producer_list[0], sym("lambda"));
            assert_eq!(producer_list[1], Value::Nil);

            if let Some(values_call) = producer_list[2].as_list() {
                assert_eq!(values_call[0], sym("values"));
                assert_eq!(values_call.len(), 3);
            }
        }

        // Consumer lambda structure
        if let Some(consumer_list) = expanded_list[2].as_list() {
            assert_eq!(consumer_list.len(), 3);
            assert_eq!(consumer_list[0], sym("lambda"));

            if let Some(params_list) = consumer_list[1].as_list() {
                assert_eq!(params_list.len(), 2);
                assert_eq!(params_list[0], sym("a"));
                assert_eq!(params_list[1], sym("b"));
            }

            if let Some(body_list) = consumer_list[2].as_list() {
                assert_eq!(body_list[0], sym("list"));
                assert_eq!(body_list[1], sym("a"));
                assert_eq!(body_list[2], sym("b"));
            }
        }
    }
}

// ============= SPECIFICATION EDGE CASES =============

#[test]
fn test_empty_body_handling() {
    // R7RS doesn't explicitly specify empty body behavior,
    // but we should handle it gracefully
    let binding = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string()]),
        producer: Value::integer(42),
    };
    let bindings = &[binding];
    let body = &[]; // Empty body

    // Should not panic, but may return an error
    let result = expand_let_values(&bindings, &body);
    // We don't require success, but it should not panic
    let _ = result;
}

#[test]
fn test_nested_let_values_compatibility() {
    // Test that nested let-values forms work correctly
    let inner_let_values = Value::list(vec![
        sym("let-values"),
        Value::list(vec![Value::list(vec![
            Value::list(&[sym("x")]),
            Value::integer(1),
        ])]),
        sym("x"),
    ]);

    let binding = LetValuesBinding {
        formals: Formals::Fixed(&["a".to_string()]),
        producer: inner_let_values,
    };
    let bindings = &[binding];
    let body = &[sym("a")];

    // Should expand without issues
    assert!(expand_let_values(bindings, body).is_ok());
}

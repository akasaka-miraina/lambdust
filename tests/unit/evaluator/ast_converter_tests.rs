//! Unit tests for AST to Value converter (ast_converter.rs)
//!
//! Tests the core quote/unquote functionality including literal conversion,
//! list/vector conversion, dotted list handling, and error cases.

use lambdust::ast::{Expr, Literal};
use lambdust::error::LambdustError;
use lambdust::evaluator::ast_converter::AstConverter;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_literal_to_value_conversion() {
    // Test boolean literals
    let bool_expr = Expr::Literal(Literal::Boolean(true));
    let result = AstConverter::expr_to_value(bool_expr).unwrap();
    assert_eq!(result, Value::Boolean(true));

    let bool_expr = Expr::Literal(Literal::Boolean(false));
    let result = AstConverter::expr_to_value(bool_expr).unwrap();
    assert_eq!(result, Value::Boolean(false));

    // Test number literals
    let num_expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let result = AstConverter::expr_to_value(num_expr).unwrap();
    assert_eq!(result, Value::Number(SchemeNumber::Integer(42)));

    let real_expr = Expr::Literal(Literal::Number(SchemeNumber::Real(std::f64::consts::PI)));
    let result = AstConverter::expr_to_value(real_expr).unwrap();
    assert_eq!(result, Value::Number(SchemeNumber::Real(std::f64::consts::PI)));

    // Test string literals
    let str_expr = Expr::Literal(Literal::String("hello".to_string()));
    let result = AstConverter::expr_to_value(str_expr).unwrap();
    assert_eq!(result, Value::String("hello".to_string()));

    // Test character literals
    let char_expr = Expr::Literal(Literal::Character('a'));
    let result = AstConverter::expr_to_value(char_expr).unwrap();
    assert_eq!(result, Value::Character('a'));

    // Test nil literal
    let nil_expr = Expr::Literal(Literal::Nil);
    let result = AstConverter::expr_to_value(nil_expr).unwrap();
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_variable_to_symbol_conversion() {
    // Test simple variable
    let var_expr = Expr::Variable("x".to_string());
    let result = AstConverter::expr_to_value(var_expr).unwrap();
    assert_eq!(result, Value::Symbol("x".to_string()));

    // Test complex variable name
    let var_expr = Expr::Variable("some-variable!".to_string());
    let result = AstConverter::expr_to_value(var_expr).unwrap();
    assert_eq!(result, Value::Symbol("some-variable!".to_string()));

    // Test variable with special characters
    let var_expr = Expr::Variable("->".to_string());
    let result = AstConverter::expr_to_value(var_expr).unwrap();
    assert_eq!(result, Value::Symbol("->".to_string()));
}

#[test]
fn test_list_to_value_conversion() {
    // Test empty list
    let empty_list = Expr::List(vec![]);
    let result = AstConverter::expr_to_value(empty_list).unwrap();
    assert_eq!(result, Value::Nil);

    // Test single element list
    let single_list = Expr::List(vec![Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))]);
    let result = AstConverter::expr_to_value(single_list).unwrap();
    
    // Should be cons(42, nil)
    if let Some((car, cdr)) = result.as_pair() {
        assert_eq!(car, Value::Number(SchemeNumber::Integer(42)));
        assert_eq!(cdr, Value::Nil);
    } else {
        panic!("Expected pair, got {:?}", result);
    }

    // Test multi-element list
    let multi_list = Expr::List(vec![
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
    ]);
    let result = AstConverter::expr_to_value(multi_list).unwrap();
    
    // Should be cons(1, cons(2, cons(3, nil)))
    if let Some((car1, cdr1)) = result.as_pair() {
        assert_eq!(car1, Value::Number(SchemeNumber::Integer(1)));
        if let Some((car2, cdr2)) = cdr1.as_pair() {
            assert_eq!(car2, Value::Number(SchemeNumber::Integer(2)));
            if let Some((car3, cdr3)) = cdr2.as_pair() {
                assert_eq!(car3, Value::Number(SchemeNumber::Integer(3)));
                assert_eq!(cdr3, Value::Nil);
            } else {
                panic!("Expected pair for third element");
            }
        } else {
            panic!("Expected pair for second element");
        }
    } else {
        panic!("Expected pair, got {:?}", result);
    }
}

#[test]
fn test_vector_to_value_conversion() {
    // Test empty vector
    let empty_vec = Expr::Vector(vec![]);
    let result = AstConverter::expr_to_value(empty_vec).unwrap();
    if let Value::Vector(vec) = result {
        assert_eq!(vec.len(), 0);
    } else {
        panic!("Expected vector, got {:?}", result);
    }

    // Test single element vector
    let single_vec = Expr::Vector(vec![Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))]);
    let result = AstConverter::expr_to_value(single_vec).unwrap();
    if let Value::Vector(vec) = result {
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[0], Value::Number(SchemeNumber::Integer(42)));
    } else {
        panic!("Expected vector, got {:?}", result);
    }

    // Test multi-element vector
    let multi_vec = Expr::Vector(vec![
        Expr::Literal(Literal::Boolean(true)),
        Expr::Literal(Literal::String("hello".to_string())),
        Expr::Variable("symbol".to_string()),
    ]);
    let result = AstConverter::expr_to_value(multi_vec).unwrap();
    if let Value::Vector(vec) = result {
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], Value::Boolean(true));
        assert_eq!(vec[1], Value::String("hello".to_string()));
        assert_eq!(vec[2], Value::Symbol("symbol".to_string()));
    } else {
        panic!("Expected vector, got {:?}", result);
    }
}

#[test]
fn test_dotted_list_conversion() {
    // Test simple dotted list: (a . b)
    let dotted = Expr::DottedList(
        vec![Expr::Variable("a".to_string())],
        Box::new(Expr::Variable("b".to_string()))
    );
    let result = AstConverter::expr_to_value(dotted).unwrap();
    
    if let Some((car, cdr)) = result.as_pair() {
        assert_eq!(car, Value::Symbol("a".to_string()));
        assert_eq!(cdr, Value::Symbol("b".to_string()));
    } else {
        panic!("Expected pair, got {:?}", result);
    }

    // Test multi-element dotted list: (a b c . d)
    let dotted = Expr::DottedList(
        vec![
            Expr::Variable("a".to_string()),
            Expr::Variable("b".to_string()),
            Expr::Variable("c".to_string()),
        ],
        Box::new(Expr::Variable("d".to_string()))
    );
    let result = AstConverter::expr_to_value(dotted).unwrap();
    
    // Should be cons(a, cons(b, cons(c, d)))
    if let Some((car1, cdr1)) = result.as_pair() {
        assert_eq!(car1, Value::Symbol("a".to_string()));
        if let Some((car2, cdr2)) = cdr1.as_pair() {
            assert_eq!(car2, Value::Symbol("b".to_string()));
            if let Some((car3, cdr3)) = cdr2.as_pair() {
                assert_eq!(car3, Value::Symbol("c".to_string()));
                assert_eq!(cdr3, Value::Symbol("d".to_string()));
            } else {
                panic!("Expected pair for third element");
            }
        } else {
            panic!("Expected pair for second element");
        }
    } else {
        panic!("Expected pair, got {:?}", result);
    }
}

#[test]
fn test_nested_quote_conversion() {
    // Test nested quote: (quote (quote x))
    let nested_quote = Expr::Quote(Box::new(Expr::Quote(Box::new(Expr::Variable("x".to_string())))));
    let result = AstConverter::expr_to_value(nested_quote).unwrap();
    
    // Should convert to the symbol x (quote is stripped)
    assert_eq!(result, Value::Symbol("x".to_string()));

    // Test quote with complex expression: (quote (a b c))
    let quote_list = Expr::Quote(Box::new(Expr::List(vec![
        Expr::Variable("a".to_string()),
        Expr::Variable("b".to_string()),
        Expr::Variable("c".to_string()),
    ])));
    let result = AstConverter::expr_to_value(quote_list).unwrap();
    
    // Should be cons(a, cons(b, cons(c, nil)))
    if let Some((car1, cdr1)) = result.as_pair() {
        assert_eq!(car1, Value::Symbol("a".to_string()));
        if let Some((car2, cdr2)) = cdr1.as_pair() {
            assert_eq!(car2, Value::Symbol("b".to_string()));
            if let Some((car3, cdr3)) = cdr2.as_pair() {
                assert_eq!(car3, Value::Symbol("c".to_string()));
                assert_eq!(cdr3, Value::Nil);
            } else {
                panic!("Expected pair for third element");
            }
        } else {
            panic!("Expected pair for second element");
        }
    } else {
        panic!("Expected pair, got {:?}", result);
    }
}

#[test]
fn test_mixed_data_types_conversion() {
    // Test list with mixed data types
    let mixed_list = Expr::List(vec![
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        Expr::Literal(Literal::String("hello".to_string())),
        Expr::Variable("symbol".to_string()),
        Expr::Literal(Literal::Boolean(true)),
        Expr::Literal(Literal::Character('x')),
    ]);
    let result = AstConverter::expr_to_value(mixed_list).unwrap();
    
    // Verify structure and contents
    if let Some((car1, cdr1)) = result.as_pair() {
        assert_eq!(car1, Value::Number(SchemeNumber::Integer(42)));
        if let Some((car2, cdr2)) = cdr1.as_pair() {
            assert_eq!(car2, Value::String("hello".to_string()));
            if let Some((car3, cdr3)) = cdr2.as_pair() {
                assert_eq!(car3, Value::Symbol("symbol".to_string()));
                if let Some((car4, cdr4)) = cdr3.as_pair() {
                    assert_eq!(car4, Value::Boolean(true));
                    if let Some((car5, cdr5)) = cdr4.as_pair() {
                        assert_eq!(car5, Value::Character('x'));
                        assert_eq!(cdr5, Value::Nil);
                    } else {
                        panic!("Expected pair for fifth element");
                    }
                } else {
                    panic!("Expected pair for fourth element");
                }
            } else {
                panic!("Expected pair for third element");
            }
        } else {
            panic!("Expected pair for second element");
        }
    } else {
        panic!("Expected pair, got {:?}", result);
    }
}

#[test]
fn test_unimplemented_quasiquote_forms() {
    // Test quasiquote returns error
    let quasiquote_expr = Expr::Quasiquote(Box::new(Expr::Variable("x".to_string())));
    let result = AstConverter::expr_to_value(quasiquote_expr);
    assert!(result.is_err());
    if let Err(LambdustError::SyntaxError { message, .. }) = result {
        assert!(message.contains("Quasiquote forms not yet implemented"));
    } else {
        panic!("Expected syntax error for quasiquote");
    }

    // Test unquote returns error
    let unquote_expr = Expr::Unquote(Box::new(Expr::Variable("x".to_string())));
    let result = AstConverter::expr_to_value(unquote_expr);
    assert!(result.is_err());
    if let Err(LambdustError::SyntaxError { message, .. }) = result {
        assert!(message.contains("Quasiquote forms not yet implemented"));
    } else {
        panic!("Expected syntax error for unquote");
    }

    // Test unquote-splicing returns error
    let unquote_splicing_expr = Expr::UnquoteSplicing(Box::new(Expr::Variable("x".to_string())));
    let result = AstConverter::expr_to_value(unquote_splicing_expr);
    assert!(result.is_err());
    if let Err(LambdustError::SyntaxError { message, .. }) = result {
        assert!(message.contains("Quasiquote forms not yet implemented"));
    } else {
        panic!("Expected syntax error for unquote-splicing");
    }
}

#[test]
fn test_recursive_structure_conversion() {
    // Test nested lists: ((a b) (c d))
    let nested_list = Expr::List(vec![
        Expr::List(vec![
            Expr::Variable("a".to_string()),
            Expr::Variable("b".to_string()),
        ]),
        Expr::List(vec![
            Expr::Variable("c".to_string()),
            Expr::Variable("d".to_string()),
        ]),
    ]);
    let result = AstConverter::expr_to_value(nested_list).unwrap();
    
    // Should be cons(cons(a, cons(b, nil)), cons(cons(c, cons(d, nil)), nil))
    if let Some((outer_car1, outer_cdr1)) = result.as_pair() {
        // First element: (a b)
        if let Some((inner_car1, inner_cdr1)) = outer_car1.as_pair() {
            assert_eq!(inner_car1, Value::Symbol("a".to_string()));
            if let Some((inner_car2, inner_cdr2)) = inner_cdr1.as_pair() {
                assert_eq!(inner_car2, Value::Symbol("b".to_string()));
                assert_eq!(inner_cdr2, Value::Nil);
            } else {
                panic!("Expected pair for 'b'");
            }
        } else {
            panic!("Expected pair for first element");
        }

        // Second element: (c d)
        if let Some((outer_car2, outer_cdr2)) = outer_cdr1.as_pair() {
            if let Some((inner_car3, inner_cdr3)) = outer_car2.as_pair() {
                assert_eq!(inner_car3, Value::Symbol("c".to_string()));
                if let Some((inner_car4, inner_cdr4)) = inner_cdr3.as_pair() {
                    assert_eq!(inner_car4, Value::Symbol("d".to_string()));
                    assert_eq!(inner_cdr4, Value::Nil);
                } else {
                    panic!("Expected pair for 'd'");
                }
            } else {
                panic!("Expected pair for second element");
            }
            assert_eq!(outer_cdr2, Value::Nil);
        } else {
            panic!("Expected pair for outer structure");
        }
    } else {
        panic!("Expected pair, got {:?}", result);
    }
}

#[test]
fn test_empty_structures_conversion() {
    // Test empty list
    let empty_list = Expr::List(vec![]);
    let result = AstConverter::expr_to_value(empty_list).unwrap();
    assert_eq!(result, Value::Nil);

    // Test empty vector
    let empty_vector = Expr::Vector(vec![]);
    let result = AstConverter::expr_to_value(empty_vector).unwrap();
    if let Value::Vector(vec) = result {
        assert_eq!(vec.len(), 0);
    } else {
        panic!("Expected empty vector, got {:?}", result);
    }

    // Test list containing empty structures
    let list_with_empty = Expr::List(vec![
        Expr::List(vec![]),
        Expr::Vector(vec![]),
    ]);
    let result = AstConverter::expr_to_value(list_with_empty).unwrap();
    
    if let Some((car1, cdr1)) = result.as_pair() {
        assert_eq!(car1, Value::Nil);
        if let Some((car2, cdr2)) = cdr1.as_pair() {
            if let Value::Vector(vec) = &car2 {
                assert_eq!(vec.len(), 0);
            } else {
                panic!("Expected empty vector");
            }
            assert_eq!(cdr2, Value::Nil);
        } else {
            panic!("Expected pair for second element");
        }
    } else {
        panic!("Expected pair, got {:?}", result);
    }
}

#[test]
fn test_error_propagation() {
    // Since literal and variable conversions don't have error cases in the current implementation,
    // we focus on testing that the recursive structure properly propagates any future errors
    
    // Test that nested structures would propagate errors if they occurred
    let nested_expr = Expr::List(vec![
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::List(vec![
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            Expr::Variable("nested".to_string()),
        ]),
    ]);
    
    // This should succeed with current implementation
    let result = AstConverter::expr_to_value(nested_expr);
    assert!(result.is_ok());
}

#[test]
fn test_large_structure_conversion() {
    // Test conversion of larger structures to ensure scalability
    let large_list: Vec<Expr> = (0..100)
        .map(|i| Expr::Literal(Literal::Number(SchemeNumber::Integer(i))))
        .collect();
    
    let large_expr = Expr::List(large_list);
    let result = AstConverter::expr_to_value(large_expr).unwrap();
    
    // Verify the structure is properly converted
    let mut current = result;
    let mut count = 0;
    
    while let Some((car, cdr)) = current.as_pair() {
        if let Value::Number(SchemeNumber::Integer(n)) = &car {
            assert_eq!(*n, count);
            count += 1;
        } else {
            panic!("Expected number at position {}", count);
        }
        current = cdr;
    }
    
    assert_eq!(current, Value::Nil);
    assert_eq!(count, 100);
}
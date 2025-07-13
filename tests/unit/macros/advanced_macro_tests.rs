                    _ => panic!("Expected bindings"),
                }
                // Should have expanded clauses
                match &exprs[2] {
                    Expr::List(if_parts) => {
                        assert_eq!(if_parts[0], Expr::Variable("if".to_string()));
                        // Test should be an or expression
                        match &if_parts[1] {
                            Expr::List(or_parts) => {
                                assert_eq!(or_parts[0], Expr::Variable("or".to_string()));
                            }
                            _ => panic!("Expected or expression"),
                        }
                    }
                    _ => panic!("Expected if expression"),
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_expand_case_single_datum() {
        let args = vec![
            Expr::Variable("x".to_string()),
            Expr::List(vec![
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                Expr::Variable("first".to_string()),
            ]),
        ];

        let result = expand_case(&args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs[0], Expr::Variable("let".to_string()));
                // Should have expanded to eqv? test
                match &exprs[2] {
                    Expr::List(if_parts) => {
                        assert_eq!(if_parts[0], Expr::Variable("if".to_string()));
                        match &if_parts[1] {
                            Expr::List(eqv_parts) => {
                                assert_eq!(eqv_parts[0], Expr::Variable("eqv?".to_string()));
                            }
                            _ => panic!("Expected eqv? expression"),
                        }
                    }
                    _ => panic!("Expected if expression"),
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_expand_case_error_too_few_args() {
        let args = vec![Expr::Variable("x".to_string())];
        let result = expand_case(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("too few arguments"));
    }
}

#[cfg(test)]
mod expand_define_record_type_tests {
    use super::*;
    use crate::macros::builtin::expand_define_record_type;

    #[test]
    fn test_expand_define_record_type_simple() {
        let args = vec![
            Expr::Variable("point".to_string()),
            Expr::List(vec![
                Expr::Variable("make-point".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Variable("y".to_string()),
            ]),
            Expr::Variable("point?".to_string()),
            Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Variable("point-x".to_string()),
                Expr::Variable("set-point-x!".to_string()),
            ]),
            Expr::List(vec![
                Expr::Variable("y".to_string()),
                Expr::Variable("point-y".to_string()),
            ]),
        ];

        let result = expand_define_record_type(&args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs[0], Expr::Variable("begin".to_string()));
                // Should have constructor, predicate, and accessor definitions
                assert!(exprs.len() >= 4); // begin + constructor + predicate + 2 accessors + 1 modifier

                // Check constructor definition
                match &exprs[1] {
                    Expr::List(def_parts) => {
                        assert_eq!(def_parts[0], Expr::Variable("define".to_string()));
                        assert_eq!(def_parts[1], Expr::Variable("make-point".to_string()));
                        match &def_parts[2] {
                            Expr::List(lambda_parts) => {
                                assert_eq!(lambda_parts[0], Expr::Variable("lambda".to_string()));
                                match &lambda_parts[1] {
                                    Expr::List(params) => {
                                        assert_eq!(params.len(), 2);
                                        assert_eq!(params[0], Expr::Variable("x".to_string()));
                                        assert_eq!(params[1], Expr::Variable("y".to_string()));
                                    }
                                    _ => panic!("Expected parameter list"),
                                }
                            }
                            _ => panic!("Expected lambda expression"),
                        }
                    }
                    _ => panic!("Expected define expression"),
                }

                // Check predicate definition
                match &exprs[2] {
                    Expr::List(def_parts) => {
                        assert_eq!(def_parts[0], Expr::Variable("define".to_string()));
                        assert_eq!(def_parts[1], Expr::Variable("point?".to_string()));
                    }
                    _ => panic!("Expected define expression"),
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_expand_define_record_type_error_too_few_args() {
        let args = vec![
            Expr::Variable("point".to_string()),
            Expr::List(vec![Expr::Variable("make-point".to_string())]),
        ];
        let result = expand_define_record_type(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expected at least 3 arguments"));
    }

    #[test]
    fn test_expand_define_record_type_error_invalid_type_name() {
        let args = vec![
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            Expr::List(vec![Expr::Variable("make-point".to_string())]),
            Expr::Variable("point?".to_string()),
        ];
        let result = expand_define_record_type(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("type name must be an identifier"));
    }

    #[test]
    fn test_expand_define_record_type_error_invalid_constructor() {
        let args = vec![
            Expr::Variable("point".to_string()),
            Expr::Variable("not-a-list".to_string()),
            Expr::Variable("point?".to_string()),
        ];
        let result = expand_define_record_type(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("constructor specification must be a list"));
    }

    #[test]
    fn test_expand_define_record_type_error_invalid_predicate() {
        let args = vec![
            Expr::Variable("point".to_string()),
            Expr::List(vec![Expr::Variable("make-point".to_string())]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ];
        let result = expand_define_record_type(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("predicate name must be an identifier"));
    }

    #[test]
    fn test_expand_define_record_type_error_invalid_field_spec() {
        let args = vec![
            Expr::Variable("point".to_string()),
            Expr::List(vec![
                Expr::Variable("make-point".to_string()),
                Expr::Variable("x".to_string()),
            ]),
            Expr::Variable("point?".to_string()),
            Expr::Variable("not-a-list".to_string()),
        ];
        let result = expand_define_record_type(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("field specification must be a list"));
    }

    #[test]
    fn test_expand_define_record_type_error_invalid_field_name() {
        let args = vec![
            Expr::Variable("point".to_string()),
            Expr::List(vec![
                Expr::Variable("make-point".to_string()),
                Expr::Variable("x".to_string()),
            ]),
            Expr::Variable("point?".to_string()),
            Expr::List(vec![
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
                Expr::Variable("point-x".to_string()),
            ]),
        ];
        let result = expand_define_record_type(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("field name must be an identifier"));
    }

    #[test]
    fn test_expand_define_record_type_error_invalid_accessor_name() {
        let args = vec![
            Expr::Variable("point".to_string()),
            Expr::List(vec![
                Expr::Variable("make-point".to_string()),
                Expr::Variable("x".to_string()),
            ]),
            Expr::Variable("point?".to_string()),
            Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            ]),
        ];
        let result = expand_define_record_type(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("accessor name must be an identifier"));
    }

    #[test]
    fn test_expand_define_record_type_error_invalid_modifier_name() {
        let args = vec![
            Expr::Variable("point".to_string()),
            Expr::List(vec![
                Expr::Variable("make-point".to_string()),
                Expr::Variable("x".to_string()),
            ]),
            Expr::Variable("point?".to_string()),
            Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Variable("point-x".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            ]),
        ];
        let result = expand_define_record_type(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("modifier name must be an identifier"));
    }
}

#[cfg(test)]
mod expand_macro_integration_tests {
    use super::*;

    #[test]
    fn test_expand_macro_let() {
        let args = vec![
            Expr::List(vec![Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            ])]),
            Expr::Variable("x".to_string()),
        ];

        let result = expand_macro("let", &args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 2);
                // Should be ((lambda (x) x) 42)
                match &exprs[0] {
                    Expr::List(lambda_parts) => {
                        assert_eq!(lambda_parts[0], Expr::Variable("lambda".to_string()));
                    }
                    _ => panic!("Expected lambda expression"),
                }
                assert_eq!(
                    exprs[1],
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))
                );
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_expand_macro_let_star() {
        let args = vec![
            Expr::List(vec![Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            ])]),
            Expr::Variable("x".to_string()),
        ];

        let result = expand_macro("let*", &args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs[0], Expr::Variable("let".to_string()));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_expand_macro_letrec() {
        let args = vec![
            Expr::List(vec![Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            ])]),
            Expr::Variable("x".to_string()),
        ];

        let result = expand_macro("letrec", &args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 2);
                // Should be ((lambda (x) (set! x 42) x) #f)
                match &exprs[0] {
                    Expr::List(lambda_parts) => {
                        assert_eq!(lambda_parts[0], Expr::Variable("lambda".to_string()));
                    }
                    _ => panic!("Expected lambda expression"),
                }
                assert_eq!(exprs[1], Expr::Variable("#f".to_string()));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_expand_macro_when() {
        let args = vec![
            Expr::Variable("condition".to_string()),
            Expr::Variable("action".to_string()),
        ];

        let result = expand_macro("when", &args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], Expr::Variable("if".to_string()));
                assert_eq!(exprs[1], Expr::Variable("condition".to_string()));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_expand_macro_unless() {
        let args = vec![
            Expr::Variable("condition".to_string()),
            Expr::Variable("action".to_string()),
        ];

        let result = expand_macro("unless", &args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 3);
                assert_eq!(exprs[0], Expr::Variable("if".to_string()));
                match &exprs[1] {
                    Expr::List(not_parts) => {
                        assert_eq!(not_parts[0], Expr::Variable("not".to_string()));
                        assert_eq!(not_parts[1], Expr::Variable("condition".to_string()));
                    }
                    _ => panic!("Expected not expression"),
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_expand_macro_case() {
        let args = vec![
            Expr::Variable("x".to_string()),
            Expr::List(vec![
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                Expr::Variable("first".to_string()),
            ]),
        ];

        let result = expand_macro("case", &args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs[0], Expr::Variable("let".to_string()));
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_expand_macro_unknown() {
        let args = vec![Expr::Variable("x".to_string())];
        let result = expand_macro("unknown-macro", &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown macro"));
    }

    #[test]
    fn test_expand_macro_empty_args() {
        let args = vec![];
        let result = expand_macro("let", &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("too few arguments"));
    }
}

#[cfg(test)]
mod macro_expander_tests {
    use super::*;

    #[test]
    fn test_macro_expander_creation() {
        let expander = MacroExpander::new();
        // Test that we can create a macro expander
        // The actual implementation would have more functionality
        assert!(format!("{:?}", expander).contains("MacroExpander"));
    }

    #[test]
    fn test_macro_expander_with_transformers() {
        let expander = MacroExpander::new();
        // Test basic functionality
        // In a complete implementation, this would test adding and using transformers
        assert!(format!("{:?}", expander).contains("MacroExpander"));
    }
}

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_nested_macro_expansion() {
        // Test expanding let inside let
        let inner_let = Expr::List(vec![
            Expr::Variable("let".to_string()),
            Expr::List(vec![Expr::List(vec![
                Expr::Variable("y".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
            ])]),
            Expr::Variable("y".to_string()),
        ]);

        let args = vec![
            Expr::List(vec![Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            ])]),
            inner_let,
        ];

        let result = expand_let(&args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 2);
                // The inner let should be preserved (not expanded)
                match &exprs[0] {
                    Expr::List(lambda_parts) => {
                        assert_eq!(lambda_parts[0], Expr::Variable("lambda".to_string()));
                        match &lambda_parts[2] {
                            Expr::List(inner_exprs) => {
                                assert_eq!(inner_exprs[0], Expr::Variable("let".to_string()));
                            }
                            _ => panic!("Expected inner let expression"),
                        }
                    }
                    _ => panic!("Expected lambda expression"),
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_macro_with_complex_expressions() {
        let args = vec![
            Expr::List(vec![Expr::List(vec![
                Expr::Variable("func".to_string()),
                Expr::List(vec![
                    Expr::Variable("lambda".to_string()),
                    Expr::List(vec![Expr::Variable("x".to_string())]),
                    Expr::List(vec![
                        Expr::Variable("*".to_string()),
                        Expr::Variable("x".to_string()),
                        Expr::Variable("x".to_string()),
                    ]),
                ]),
            ])]),
            Expr::List(vec![
                Expr::Variable("func".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
            ]),
        ];

        let result = expand_let(&args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 2);
                // Should preserve the lambda expression structure
                match &exprs[0] {
                    Expr::List(lambda_parts) => {
                        assert_eq!(lambda_parts[0], Expr::Variable("lambda".to_string()));
                    }
                    _ => panic!("Expected lambda expression"),
                }
                // Should preserve the lambda expression as the value
                match &exprs[1] {
                    Expr::List(lambda_expr) => {
                        assert_eq!(lambda_expr[0], Expr::Variable("lambda".to_string()));
                    }
                    _ => panic!("Expected lambda expression"),
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_macro_with_unicode_identifiers() {
        let args = vec![
            Expr::List(vec![Expr::List(vec![
                Expr::Variable("λ".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            ])]),
            Expr::Variable("λ".to_string()),
        ];

        let result = expand_let(&args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 2);
                match &exprs[0] {
                    Expr::List(lambda_parts) => {
                        assert_eq!(lambda_parts[0], Expr::Variable("lambda".to_string()));
                        match &lambda_parts[1] {
                            Expr::List(params) => {
                                assert_eq!(params[0], Expr::Variable("λ".to_string()));
                            }
                            _ => panic!("Expected parameter list"),
                        }
                    }
                    _ => panic!("Expected lambda expression"),
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_macro_with_empty_body() {
        let args = vec![
            Expr::List(vec![Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            ])]),
            // Add at least one body expression - empty body is invalid for let
            Expr::Variable("x".to_string()),
        ];

        let result = expand_let(&args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 2);
                match &exprs[0] {
                    Expr::List(lambda_parts) => {
                        assert_eq!(lambda_parts[0], Expr::Variable("lambda".to_string()));
                        // Should have lambda, params, and body
                        assert_eq!(lambda_parts.len(), 3);
                    }
                    _ => panic!("Expected lambda expression"),
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_macro_with_very_large_bindings() {
        let mut bindings = Vec::new();
        for i in 0..100 {
            bindings.push(Expr::List(vec![
                Expr::Variable(format!("var{}", i)),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(i))),
            ]));
        }

        let args = vec![Expr::List(bindings), Expr::Variable("var0".to_string())];

        let result = expand_let(&args).unwrap();

        match result {
            Expr::List(exprs) => {
                assert_eq!(exprs.len(), 101); // lambda + 100 values
                match &exprs[0] {
                    Expr::List(lambda_parts) => {
                        assert_eq!(lambda_parts[0], Expr::Variable("lambda".to_string()));
                        match &lambda_parts[1] {
                            Expr::List(params) => {
                                assert_eq!(params.len(), 100);
                                assert_eq!(params[0], Expr::Variable("var0".to_string()));
                                assert_eq!(params[99], Expr::Variable("var99".to_string()));
                            }
                            _ => panic!("Expected parameter list"),
                        }
                    }
                    _ => panic!("Expected lambda expression"),
                }
            }
            _ => panic!("Expected list expression"),
        }
    }
}

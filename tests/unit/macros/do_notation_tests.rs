#[cfg(test)]
mod tests {
    use crate::macros::do_notation::*;
    use crate::ast::Expr;

    #[test]
    fn test_do_notation_expander_creation() {
        let expander = DoNotationExpander::new();
        assert!(expander.get_monad("List").is_some());
        assert!(expander.get_monad("Maybe").is_some());
    }

    #[test]
    fn test_parse_simple_mdo_block() {
        let expander = DoNotationExpander::new();
        
        // (mdo [x <- (list 1 2 3)] [y <- (list 4 5 6)] (+ x y))
        let mdo_expr = Expr::List(vec![
            Expr::Variable("mdo".to_string()),
            Expr::List(vec![
                Expr::Variable("x".to_string()),
                Expr::Variable("<-".to_string()),
                Expr::List(vec![
                    Expr::Variable("list".to_string()),
                    Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(1))),
                    Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(2))),
                    Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(3))),
                ]),
            ]),
            Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(10))),
            ]),
        ]);
        
        let result = expander.parse_mdo_syntax(&mdo_expr);
        assert!(result.is_ok());
        
        let mdo_block = result.unwrap();
        assert_eq!(mdo_block.bindings.len(), 1);
        
        match &mdo_block.bindings[0] {
            DoBinding::Bind { var, .. } => assert_eq!(var, "x"),
            _ => panic!("Expected bind"),
        }
    }

    #[test]
    fn test_expand_simple_mdo() {
        let expander = DoNotationExpander::new();
        
        let mdo_block = DoBlock {
            bindings: vec![
                DoBinding::Bind {
                    var: "x".to_string(),
                    computation: Expr::List(vec![
                        Expr::Variable("list".to_string()),
                        Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(1))),
                    ]),
                }
            ],
            result: Expr::Variable("x".to_string()),
        };
        
        let result = expander.expand_mdo(mdo_block);
        assert!(result.is_ok());
        
        // Should generate bind expression
        let expansion_result = result.unwrap();
        match expansion_result {
            Expr::List(elements) => {
                assert_eq!(elements.len(), 3);
                if let Expr::Variable(op) = &elements[0] {
                    assert_eq!(op, ">>=");
                }
            }
            _ => panic!("Expected list expression"),
        }
    }

    #[test]
    fn test_monad_instance_registration() {
        let mut expander = DoNotationExpander::new();
        
        let custom_monad = MonadInstance {
            name: "Custom".to_string(),
            return_fn: "custom-return".to_string(),
            bind_fn: "custom-bind".to_string(),
            type_constructor: "Custom".to_string(),
        };
        
        expander.register_monad(custom_monad);
        assert!(expander.get_monad("Custom").is_some());
    }

    #[test]
    fn test_is_mdo_notation() {
        let expander = DoNotationExpander::new();
        
        let mdo_expr = Expr::List(vec![
            Expr::Variable("mdo".to_string()),
            Expr::Variable("x".to_string()),
        ]);
        
        assert!(expander.is_mdo_notation(&mdo_expr));
        
        let not_mdo_expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::Variable("x".to_string()),
        ]);
        
        assert!(!expander.is_mdo_notation(&not_mdo_expr));
    }

    #[test]
    fn test_multiple_bindings() {
        let notation_expander = DoNotationExpander::new();
        
        let mdo_block = DoBlock {
            bindings: vec![
                DoBinding::Bind {
                    var: "x".to_string(),
                    computation: Expr::List(vec![
                        Expr::Variable("list".to_string()),
                        Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(1))),
                    ]),
                },
                DoBinding::Bind {
                    var: "y".to_string(),
                    computation: Expr::List(vec![
                        Expr::Variable("list".to_string()),
                        Expr::Literal(crate::ast::Literal::Number(crate::lexer::SchemeNumber::Integer(2))),
                    ]),
                }
            ],
            result: Expr::List(vec![
                Expr::Variable("+".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Variable("y".to_string()),
            ]),
        };
        
        let result = notation_expander.expand_mdo(mdo_block);
        assert!(result.is_ok());
        
        // Should generate nested bind expressions
        let nested_expanded = result.unwrap();
        match nested_expanded {
            Expr::List(elements) => {
                // Outer bind
                assert_eq!(elements.len(), 3);
                if let Expr::Variable(op) = &elements[0] {
                    assert_eq!(op, ">>=");
                }
                
                // Inner should also be a bind
                if let Expr::List(lambda_elements) = &elements[2] {
                    if lambda_elements.len() >= 3 {
                        if let Expr::List(body_elements) = &lambda_elements[2] {
                            if let Expr::Variable(inner_op) = &body_elements[0] {
                                assert_eq!(inner_op, ">>=");
                            }
                        }
                    }
                }
            }
            _ => panic!("Expected list expression"),
        }
    }
}
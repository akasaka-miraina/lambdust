//! Tests for SRFI-26: Notation for Specializing Parameters (cut/cute)

use lambdust::ast::{CutArgument, Expr, Literal};
use lambdust::diagnostics::{Span, Spanned};
use lambdust::eval::{Environment, Evaluator, Value};
use lambdust::lexer::Lexer;
use lambdust::parser::Parser;
use std::rc::Rc;

// Only include macro system imports if they're available
#[cfg(test)]
use lambdust::macro_system::{expand_cut, expand_cute};

// Only include reset_parameter_counter if it's available
#[cfg(test)]
use lambdust::macro_system::reset_parameter_counter;

fn dummy_span() -> Span {
    Span::new(0, 0)
}

fn spanned<T>(value: T) -> Spanned<T> {
    Spanned::new(value, dummy_span())
}

fn parse_expr(input: &str) -> Result<Spanned<Expr>, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(input, None);
    let tokens = lexer.tokenize()?;
    // Use non-aggressive recovery to ensure errors are properly propagated
    let mut parser = Parser::with_settings(tokens, 1, false);
    let program = parser.parse()?;

    if program.expressions.is_empty() {
        Err("No expressions parsed".into())
    } else {
        Ok(program.expressions[0].clone())
    }
}

fn eval_expr(expr: &Spanned<Expr>) -> Result<Value, Box<dyn std::error::Error>> {
    let mut evaluator = Evaluator::new();
    let env = Rc::new(Environment::new(None, 0));
    let result = evaluator.eval(expr, env)?;
    Ok(result)
}

#[test]
fn test_cut_argument_creation() {
    let slot = CutArgument::slot();
    let rest_slot = CutArgument::rest_slot();
    let expr = CutArgument::expression(spanned(Expr::Literal(Literal::integer(42))));

    assert!(slot.is_slot());
    assert!(rest_slot.is_rest_slot());
    assert!(expr.is_expression());
}

#[test]
fn test_parse_simple_cut_expression() {
    let result = parse_expr("(cut + <> 5)");
    assert!(result.is_ok());

    let expr = result.unwrap();
    if let Expr::Cut {
        procedure,
        arguments,
    } = &expr.inner
    {
        assert_eq!(arguments.len(), 2);
        assert!(arguments[0].is_slot());
        assert!(arguments[1].is_expression());

        if let Expr::Identifier(name) = &procedure.inner {
            assert_eq!(name, "+");
        } else {
            panic!("Expected identifier for procedure");
        }
    } else {
        panic!("Expected Cut expression, got {:?}", expr.inner);
    }
}

#[test]
fn test_parse_simple_cute_expression() {
    let result = parse_expr("(cute cons <> '())");
    assert!(result.is_ok());

    let expr = result.unwrap();
    if let Expr::Cute {
        procedure,
        arguments,
    } = &expr.inner
    {
        assert_eq!(arguments.len(), 2);
        assert!(arguments[0].is_slot());
        assert!(arguments[1].is_expression());

        if let Expr::Identifier(name) = &procedure.inner {
            assert_eq!(name, "cons");
        } else {
            panic!("Expected identifier for procedure");
        }
    } else {
        panic!("Expected Cute expression, got {:?}", expr.inner);
    }
}

#[test]
fn test_parse_cut_with_rest_arguments() {
    let result = parse_expr("(cut list <> <> <...>)");
    assert!(result.is_ok());

    let expr = result.unwrap();
    if let Expr::Cut { arguments, .. } = &expr.inner {
        assert_eq!(arguments.len(), 3);
        assert!(arguments[0].is_slot());
        assert!(arguments[1].is_slot());
        assert!(arguments[2].is_rest_slot());
    } else {
        panic!("Expected Cut expression");
    }
}

#[test]
fn test_parse_error_rest_slot_not_last() {
    let result = parse_expr("(cut list <...> <>)");
    assert!(result.is_err());

    let error_msg = format!("{:?}", result.unwrap_err());
    assert!(error_msg.contains("must be the last argument"));
}

#[test]
fn test_cut_expansion_simple_slot() {
    reset_parameter_counter();

    let procedure = spanned(Expr::Identifier("+".to_string()));
    let arguments = vec![
        CutArgument::slot(),
        CutArgument::expression(spanned(Expr::Literal(Literal::integer(5)))),
    ];

    let result = expand_cut(&procedure, &arguments, dummy_span());
    assert!(result.is_ok());

    let lambda = result.unwrap();
    if let Expr::Lambda { formals, body, .. } = &lambda.inner {
        // Should generate (lambda (x_N) (+ x_N 5))
        assert_eq!(body.len(), 1);

        // Get the generated parameter name
        let expected_param_name = if let lambdust::ast::Formals::Fixed(params) = formals {
            assert_eq!(params.len(), 1);
            params[0].clone()
        } else {
            panic!("Expected fixed formals");
        };

        if let Expr::Application { operator, operands } = &body[0].inner {
            if let Expr::Identifier(name) = &operator.inner {
                assert_eq!(name, "+");
            }
            assert_eq!(operands.len(), 2);

            // First operand should match the parameter name
            if let Expr::Identifier(param_name) = &operands[0].inner {
                assert_eq!(param_name, &expected_param_name);
            }

            // Second operand should be literal 5
            if let Expr::Literal(Literal::ExactInteger(n)) = &operands[1].inner {
                assert_eq!(*n, 5);
            }
        }
    } else {
        panic!("Expected lambda expression");
    }
}

#[test]
fn test_cute_expansion_with_eager_evaluation() {
    reset_parameter_counter();

    let procedure = spanned(Expr::Identifier("cons".to_string()));
    let arguments = vec![
        CutArgument::slot(),
        CutArgument::expression(spanned(Expr::Quote(Box::new(spanned(Expr::List(&[])))))),
    ];

    let result = expand_cute(&procedure, &arguments, dummy_span());
    assert!(result.is_ok());

    let expr = result.unwrap();
    // Should generate (let ((cut-temp-0 '())) (lambda (x1) (cons x1 cut-temp-0)))
    if let Expr::Let { bindings, body } = &expr.inner {
        assert_eq!(bindings.len(), 1);
        assert_eq!(body.len(), 1);

        // Check binding
        let binding = &bindings[0];
        assert!(binding.name.starts_with("cut-temp-"));

        // Check lambda in body
        if let Expr::Lambda {
            body: lambda_body, ..
        } = &body[0].inner
        {
            if let Expr::Application { operands, .. } = &lambda_body[0].inner {
                assert_eq!(operands.len(), 2);
                // Second operand should be the temp variable
                if let Expr::Identifier(temp_name) = &operands[1].inner {
                    assert_eq!(temp_name, &binding.name);
                }
            }
        }
    } else {
        panic!("Expected let expression for cute");
    }
}

#[test]
fn test_cut_expansion_with_rest_parameters() {
    reset_parameter_counter();

    let procedure = spanned(Expr::Identifier("list".to_string()));
    let arguments = &[CutArgument::slot(), CutArgument::rest_slot()];

    let result = expand_cut(&procedure, &arguments, dummy_span());
    assert!(result.is_ok());

    let lambda = result.unwrap();
    if let Expr::Lambda { formals, body, .. } = &lambda.inner {
        // Should be mixed formals (x_N . rest)
        if let lambdust::ast::Formals::Mixed { fixed, rest } = formals {
            assert_eq!(fixed.len(), 1);
            assert!(fixed[0].starts_with('x'));
            assert_eq!(rest, "rest");
        } else {
            panic!("Expected mixed formals for rest parameters");
        }

        // Body should use apply
        if let Expr::Application { operator, operands } = &body[0].inner {
            if let Expr::Identifier(name) = &operator.inner {
                assert_eq!(name, "apply");
                assert!(operands.len() >= 3); // apply, list, x1, rest
            }
        }
    }
}

#[test]
fn test_cut_only_rest_parameters() {
    reset_parameter_counter();

    let procedure = spanned(Expr::Identifier("+".to_string()));
    let arguments = &[CutArgument::rest_slot()];

    let result = expand_cut(&procedure, &arguments, dummy_span());
    assert!(result.is_ok());

    let lambda = result.unwrap();
    if let Expr::Lambda { formals, .. } = &lambda.inner {
        // Should be variable formals (just args)
        if let lambdust::ast::Formals::Variable(var_name) = formals {
            assert_eq!(var_name, "args");
        } else {
            panic!("Expected variable formals for rest-only parameters");
        }
    }
}

#[test]
fn test_multiple_slots() {
    reset_parameter_counter();

    let procedure = spanned(Expr::Identifier("+".to_string()));
    let arguments = vec![
        CutArgument::slot(),
        CutArgument::slot(),
        CutArgument::slot(),
    ];

    let result = expand_cut(&procedure, &arguments, dummy_span());
    assert!(result.is_ok());

    let lambda = result.unwrap();
    if let Expr::Lambda { formals, body, .. } = &lambda.inner {
        // Should have 3 parameters
        if let lambdust::ast::Formals::Fixed(params) = formals {
            assert_eq!(params.len(), 3);
            // Don't hardcode parameter names, just verify they're unique and follow the pattern
            for (i, param) in params.iter().enumerate() {
                assert!(param.starts_with('x'));
                // Verify each parameter is unique
                for (j, other_param) in params.iter().enumerate() {
                    if i != j {
                        assert_ne!(param, other_param);
                    }
                }
            }
        }

        // Body should call + with 3 arguments
        if let Expr::Application { operands, .. } = &body[0].inner {
            assert_eq!(operands.len(), 3);
        }
    }
}

#[test]
fn test_mixed_slots_and_expressions() {
    reset_parameter_counter();

    let procedure = spanned(Expr::Identifier("list".to_string()));
    let arguments = vec![
        CutArgument::expression(spanned(Expr::Literal(Literal::integer(1)))),
        CutArgument::slot(),
        CutArgument::expression(spanned(Expr::Literal(Literal::integer(3)))),
        CutArgument::slot(),
    ];

    let result = expand_cut(&procedure, &arguments, dummy_span());
    assert!(result.is_ok());

    let lambda = result.unwrap();
    if let Expr::Lambda { formals, body, .. } = &lambda.inner {
        // Should have 2 parameters for the 2 slots
        if let lambdust::ast::Formals::Fixed(params) = formals {
            assert_eq!(params.len(), 2);
        }

        // Body should call list with 4 arguments
        if let Expr::Application { operands, .. } = &body[0].inner {
            assert_eq!(operands.len(), 4);

            // Get the parameter names from formals
            if let lambdust::ast::Formals::Fixed(params) = formals {
                let param1 = &params[0];
                let param2 = &params[1];

                // Check argument pattern: 1, param1, 3, param2
                if let Expr::Literal(Literal::ExactInteger(n)) = &operands[0].inner {
                    assert_eq!(*n, 1);
                }
                if let Expr::Identifier(name) = &operands[1].inner {
                    assert_eq!(name, param1);
                }
                if let Expr::Literal(Literal::ExactInteger(n)) = &operands[2].inner {
                    assert_eq!(*n, 3);
                }
                if let Expr::Identifier(name) = &operands[3].inner {
                    assert_eq!(name, param2);
                }
            }
        }
    }
}

#[test]
fn test_display_cut_expression() {
    let procedure = spanned(Expr::Identifier("+".to_string()));
    let arguments = vec![
        CutArgument::slot(),
        CutArgument::expression(spanned(Expr::Literal(Literal::integer(5)))),
    ];

    let cut_expr = Expr::Cut {
        procedure: Box::new(procedure),
        arguments,
    };

    let display_result = format!("{cut_expr}");
    assert!(display_result.contains("cut"));
    assert!(display_result.contains("<>"));
    assert!(display_result.contains("5"));
}

#[test]
fn test_display_cute_expression() {
    let procedure = spanned(Expr::Identifier("cons".to_string()));
    let arguments = &[CutArgument::slot(), CutArgument::rest_slot()];

    let cute_expr = Expr::Cute {
        procedure: Box::new(procedure),
        arguments,
    };

    let display_result = format!("{cute_expr}");
    assert!(display_result.contains("cute"));
    assert!(display_result.contains("<>"));
    assert!(display_result.contains("<...>"));
}

// Integration tests with full evaluation pipeline

#[cfg(test)]
fn setup_test_environment() -> (Evaluator, Rc<Environment>) {
    let evaluator = Evaluator::new();
    let env = Rc::new(Environment::new(None, 0));

    // Add basic arithmetic operations to the environment
    // This is simplified - in a real implementation these would be primitives
    (evaluator, env)
}

#[test]
fn test_integration_simple_cut() {
    let expr = parse_expr("(cut + <> 5)").unwrap();

    // This should parse correctly
    if let Expr::Cut { .. } = &expr.inner {
        // Test passes - cut expression was parsed
    } else {
        panic!("Expected Cut expression");
    }
}

#[test]
fn test_integration_simple_cute() {
    let expr = parse_expr("(cute cons <> '())").unwrap();

    // This should parse correctly
    if let Expr::Cute { .. } = &expr.inner {
        // Test passes - cute expression was parsed
    } else {
        panic!("Expected Cute expression");
    }
}

#[test]
fn test_srfi26_examples_from_spec() {
    // Examples from SRFI-26 specification

    // (cut cons <> '()) should parse
    let expr1 = parse_expr("(cut cons <> '())");
    assert!(expr1.is_ok());

    // (cut list 1 <> 3 <> 5) should parse
    let expr2 = parse_expr("(cut list 1 <> 3 <> 5)");
    assert!(expr2.is_ok());

    // (cut <> 1 2) should parse
    let expr3 = parse_expr("(cut <> 1 2)");
    assert!(expr3.is_ok());

    // (cut list <...>) should parse
    let expr4 = parse_expr("(cut list <...>)");
    assert!(expr4.is_ok());
}

#[test]
fn test_error_conditions() {
    // Multiple rest slots should fail
    let result1 = parse_expr("(cut list <...> <...>)");
    assert!(result1.is_err());

    // Rest slot not at end should fail
    let result2 = parse_expr("(cut list <...> <>)");
    assert!(result2.is_err());

    // Empty cut should be valid (though unusual)
    let result3 = parse_expr("(cut +)");
    assert!(result3.is_ok());
}

#[test]
fn test_parameter_generation_uniqueness() {
    reset_parameter_counter();

    // Generate multiple cut expressions and ensure parameter names are unique
    let proc = spanned(Expr::Identifier("+".to_string()));

    let args1 = &[CutArgument::slot(), CutArgument::slot()];
    let result1 = expand_cut(&proc, &args1, dummy_span()).unwrap();

    let args2 = &[CutArgument::slot()];
    let result2 = expand_cut(&proc, &args2, dummy_span()).unwrap();

    // Parameters should not conflict between different expansions
    if let (
        Expr::Lambda {
            formals: formals1, ..
        },
        Expr::Lambda {
            formals: formals2, ..
        },
    ) = (&result1.inner, &result2.inner)
    {
        if let (lambdust::ast::Formals::Fixed(params1), lambdust::ast::Formals::Fixed(params2)) =
            (formals1, formals2)
        {
            // Verify parameter uniqueness and patterns
            assert_eq!(params1.len(), 2);
            assert_eq!(params2.len(), 1);

            // All parameters should start with 'x' and be unique
            let all_params: Vec<&String> = params1.iter().chain(params2.iter()).collect();
            for (i, param) in all_params.iter().enumerate() {
                assert!(param.starts_with('x'));
                // Check uniqueness
                for (j, other_param) in all_params.iter().enumerate() {
                    if i != j {
                        assert_ne!(param, other_param);
                    }
                }
            }
        }
    }
}

#[test]
fn test_nested_cut_expressions() {
    // Test that cut expressions can contain other cut expressions
    let nested_expr = parse_expr("(cut map (cut + <> 1) <>)");
    assert!(nested_expr.is_ok());

    if let Expr::Cut { arguments, .. } = &nested_expr.unwrap().inner {
        assert_eq!(arguments.len(), 2);

        // First argument should be another cut expression
        if let CutArgument::Expression(expr) = &arguments[0] {
            if let Expr::Cut { .. } = &expr.inner {
                // Success - nested cut detected
            } else {
                panic!("Expected nested cut expression");
            }
        }

        // Second argument should be a slot
        assert!(arguments[1].is_slot());
    }
}

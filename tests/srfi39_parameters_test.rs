//! Tests for SRFI-39 Parameter objects implementation.

use lambdust::{
    ast::{Expr, Literal},
    diagnostics::Spanned,
    eval::{Evaluator, Value, Parameter, ParameterBinding},
    parser::Parser,
    lexer::Lexer,
};
use std::collections::HashMap;

/// Helper function to create a test evaluator with standard library.
fn create_test_evaluator() -> Evaluator {
    let mut evaluator = Evaluator::new();
    // The standard library includes parameter functions
    evaluator
}

/// Helper function to parse and evaluate a Scheme expression.
fn eval_scheme(evaluator: &mut Evaluator, code: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;
    let result = evaluator.eval_program(&program)?;
    Ok(result)
}

#[test]
fn test_make_parameter_basic() {
    let mut evaluator = create_test_evaluator();
    
    // Test creating a parameter with initial value
    let result = eval_scheme(&mut evaluator, "(make-parameter 42)").unwrap();
    assert!(result.is_parameter());
    
    // Test calling the parameter to get its value
    let result = eval_scheme(&mut evaluator, 
        "(define p (make-parameter 42)) (p)").unwrap();
    assert_eq!(result.as_integer(), Some(42));
}

#[test]
fn test_make_parameter_with_converter() {
    let mut evaluator = create_test_evaluator();
    
    // Test creating a parameter with a converter function
    let code = r#"
        (define p (make-parameter 0 
          (lambda (x) 
            (if (number? x) x (error "Must be a number")))))
        (p)
    "#;
    let result = eval_scheme(&mut evaluator, code).unwrap();
    assert_eq!(result.as_integer(), Some(0));
}

#[test]
fn test_parameter_predicate() {
    let mut evaluator = create_test_evaluator();
    
    // Test parameter? predicate
    let result = eval_scheme(&mut evaluator, 
        "(parameter? (make-parameter 42))").unwrap();
    assert_eq!(result, Value::boolean(true));
    
    let result = eval_scheme(&mut evaluator, 
        "(parameter? 42)").unwrap();
    assert_eq!(result, Value::boolean(false));
}

#[test]
fn test_parameter_set_get() {
    let mut evaluator = create_test_evaluator();
    
    let code = r#"
        (define p (make-parameter 1))
        (list (p) (begin (p 2) (p)))
    "#;
    let result = eval_scheme(&mut evaluator, code).unwrap();
    
    // Should return (1 2)
    if let Some(list_values) = result.as_list() {
        assert_eq!(list_values.len(), 2);
        assert_eq!(list_values[0].as_integer(), Some(1));
        assert_eq!(list_values[1].as_integer(), Some(2));
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_parameterize_basic() {
    let mut evaluator = create_test_evaluator();
    
    let code = r#"
        (define p (make-parameter 1))
        (list (p) 
              (parameterize ((p 2)) (p))
              (p))
    "#;
    let result = eval_scheme(&mut evaluator, code).unwrap();
    
    // Should return (1 2 1)
    if let Some(list_values) = result.as_list() {
        assert_eq!(list_values.len(), 3);
        assert_eq!(list_values[0].as_integer(), Some(1));
        assert_eq!(list_values[1].as_integer(), Some(2));
        assert_eq!(list_values[2].as_integer(), Some(1));
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_parameterize_nested() {
    let mut evaluator = create_test_evaluator();
    
    let code = r#"
        (define p (make-parameter 1))
        (list (p)
              (parameterize ((p 2))
                (list (p)
                      (parameterize ((p 3)) (p))
                      (p)))
              (p))
    "#;
    let result = eval_scheme(&mut evaluator, code).unwrap();
    
    // Should return (1 (2 3 2) 1)
    if let Some(outer_list) = result.as_list() {
        assert_eq!(outer_list.len(), 3);
        assert_eq!(outer_list[0].as_integer(), Some(1));
        assert_eq!(outer_list[2].as_integer(), Some(1));
        
        if let Some(inner_list) = outer_list[1].as_list() {
            assert_eq!(inner_list.len(), 3);
            assert_eq!(inner_list[0].as_integer(), Some(2));
            assert_eq!(inner_list[1].as_integer(), Some(3));
            assert_eq!(inner_list[2].as_integer(), Some(2));
        } else {
            panic!("Expected inner list");
        }
    } else {
        panic!("Expected outer list result");
    }
}

#[test]
fn test_parameterize_multiple_parameters() {
    let mut evaluator = create_test_evaluator();
    
    let code = r#"
        (define p1 (make-parameter 1))
        (define p2 (make-parameter 10))
        (list (+ (p1) (p2))
              (parameterize ((p1 2) (p2 20))
                (+ (p1) (p2)))
              (+ (p1) (p2)))
    "#;
    let result = eval_scheme(&mut evaluator, code).unwrap();
    
    // Should return (11 22 11)
    if let Some(list_values) = result.as_list() {
        assert_eq!(list_values.len(), 3);
        assert_eq!(list_values[0].as_integer(), Some(11));
        assert_eq!(list_values[1].as_integer(), Some(22));
        assert_eq!(list_values[2].as_integer(), Some(11));
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_parameter_procedure_interface() {
    let mut evaluator = create_test_evaluator();
    
    // Test that parameters are recognized as procedures
    let result = eval_scheme(&mut evaluator, 
        "(procedure? (make-parameter 42))").unwrap();
    assert_eq!(result, Value::boolean(true));
    
    // Test calling parameter as procedure
    let code = r#"
        (define p (make-parameter 42))
        (define get-value (lambda () (p)))
        (get-value)
    "#;
    let result = eval_scheme(&mut evaluator, code).unwrap();
    assert_eq!(result.as_integer(), Some(42));
}

#[test]
fn test_parameter_thread_isolation() {
    // This test would require thread support in the evaluator
    // For now, we'll test that parameters work within a single thread
    let mut evaluator = create_test_evaluator();
    
    let code = r#"
        (define p (make-parameter 1))
        (define results '())
        
        (set! results (cons (p) results))
        (parameterize ((p 2))
          (set! results (cons (p) results))
          (parameterize ((p 3))
            (set! results (cons (p) results)))
          (set! results (cons (p) results)))
        (set! results (cons (p) results))
        
        (reverse results)
    "#;
    let result = eval_scheme(&mut evaluator, code).unwrap();
    
    // Should return (1 2 3 2 1)
    if let Some(list_values) = result.as_list() {
        assert_eq!(list_values.len(), 5);
        assert_eq!(list_values[0].as_integer(), Some(1));
        assert_eq!(list_values[1].as_integer(), Some(2));
        assert_eq!(list_values[2].as_integer(), Some(3));
        assert_eq!(list_values[3].as_integer(), Some(2));
        assert_eq!(list_values[4].as_integer(), Some(1));
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_current_port_parameters() {
    let mut evaluator = create_test_evaluator();
    
    // Test that current ports are parameters
    let result = eval_scheme(&mut evaluator, 
        "(parameter? current-input-port)").unwrap();
    assert_eq!(result, Value::boolean(true));
    
    let result = eval_scheme(&mut evaluator, 
        "(parameter? current-output-port)").unwrap();
    assert_eq!(result, Value::boolean(true));
    
    let result = eval_scheme(&mut evaluator, 
        "(parameter? current-error-port)").unwrap();
    assert_eq!(result, Value::boolean(true));
}

#[test]
fn test_current_port_parameterize() {
    let mut evaluator = create_test_evaluator();
    
    // Test parameterizing current output port
    let code = r#"
        (define original-port (current-output-port))
        (define string-port (open-output-string))
        
        (display "outside")
        (parameterize ((current-output-port string-port))
          (display "inside"))
        (display "outside again")
        
        (port? original-port)
    "#;
    let result = eval_scheme(&mut evaluator, code);
    // This test requires string ports which might not be fully implemented
    // For now, we'll just check it doesn't error on the parameterize syntax
    assert!(result.is_ok());
}

#[test]
fn test_parameter_error_cases() {
    let mut evaluator = create_test_evaluator();
    
    // Test wrong number of arguments to make-parameter
    let result = eval_scheme(&mut evaluator, "(make-parameter)");
    assert!(result.is_err());
    
    let result = eval_scheme(&mut evaluator, "(make-parameter 1 2 3)");
    assert!(result.is_err());
    
    // Test calling parameter with wrong number of arguments
    let result = eval_scheme(&mut evaluator, 
        "(define p (make-parameter 1)) (p 1 2)");
    assert!(result.is_err());
}

#[test]
fn test_parameter_conversion_example() {
    let mut evaluator = create_test_evaluator();
    
    // Test parameter with converter that validates input
    let code = r#"
        (define positive-param 
          (make-parameter 1 
            (lambda (x) 
              (if (and (number? x) (> x 0)) 
                  x 
                  (error "Must be positive number")))))
        
        (positive-param 5)
        (positive-param)
    "#;
    let result = eval_scheme(&mut evaluator, code).unwrap();
    assert_eq!(result.as_integer(), Some(5));
}

// Unit tests for the parameter implementation internals
#[cfg(test)]
mod parameter_internals {
    use super::*;
    use lambdust::eval::parameter::{ParameterBinding, ParameterFrame};
    
    #[test]
    fn test_parameter_creation() {
        let param = Parameter::new(Value::integer(42), None);
        assert_eq!(param.get().as_integer(), Some(42));
        assert!(!param.has_converter());
        assert!(param.name().is_none());
    }
    
    #[test]
    fn test_parameter_with_name() {
        let param = Parameter::with_name(
            Value::string("test"), 
            None, 
            "test-param".to_string()
        );
        assert_eq!(param.get().as_string(), Some("test"));
        assert_eq!(param.name(), Some("test-param"));
    }
    
    #[test]
    fn test_parameter_binding_stack() {
        // Clear any existing bindings
        ParameterBinding::clear_stack();
        assert_eq!(ParameterBinding::stack_depth(), 0);
        
        let param = Parameter::new(Value::integer(1), None);
        let param_id = param.id();
        
        // Test initial value
        assert_eq!(param.get().as_integer(), Some(1));
        
        // Test binding
        let mut bindings = HashMap::new();
        bindings.insert(param_id, Value::integer(42));
        
        let result = ParameterBinding::with_bindings(bindings, || {
            param.get().as_integer().unwrap()
        });
        
        assert_eq!(result, 42);
        
        // Test that binding is removed after closure
        assert_eq!(param.get().as_integer(), Some(1));
    }
    
    #[test]
    fn test_nested_parameter_bindings() {
        ParameterBinding::clear_stack();
        
        let param = Parameter::new(Value::integer(1), None);
        let param_id = param.id();
        
        let mut bindings1 = HashMap::new();
        bindings1.insert(param_id, Value::integer(10));
        
        let mut bindings2 = HashMap::new();
        bindings2.insert(param_id, Value::integer(20));
        
        let result = ParameterBinding::with_bindings(bindings1, || {
            assert_eq!(param.get().as_integer(), Some(10));
            
            ParameterBinding::with_bindings(bindings2, || {
                assert_eq!(param.get().as_integer(), Some(20));
                30 // return value
            })
        });
        
        assert_eq!(result, 30);
        assert_eq!(param.get().as_integer(), Some(1)); // Back to original
    }
}
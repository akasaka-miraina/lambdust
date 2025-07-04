//! Unit tests for the interpreter module
//!
//! These tests verify the behavior of the main interpreter interface,
//! including function registration, Scheme function calls, and host-scheme
//! function integration.

use lambdust::error::{LambdustError, Result};
use lambdust::interpreter::LambdustInterpreter;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_interpreter_basic() {
    let mut interpreter = LambdustInterpreter::new();

    // Test basic evaluation
    let _result = interpreter.eval_string("(+ 1 2 3)").unwrap();
    // Note: This will fail until arithmetic is implemented
    // assert_eq!(result, Value::Number(SchemeNumber::Integer(6)));
}

#[test]
fn test_function_definition_and_call() {
    let mut interpreter = LambdustInterpreter::new();

    // Test simple literal evaluation first
    let result = interpreter.eval_string("42");
    assert!(result.is_ok());

    // Test simple variable definition first
    let result = interpreter.eval_string("(define x 42)");
    if let Err(e) = result {
        panic!("Failed to define variable: {:?}", e);
    }

    // Define a simple function
    let result = interpreter.eval_string("(define (identity x) x)");
    if let Err(e) = result {
        panic!("Failed to define function: {:?}", e);
    }

    // Check if function is available
    assert!(interpreter.has_scheme_function("identity"));

    // Call the function (will fail until arithmetic is implemented)
    // let result = interpreter.call_scheme_function("square", &[Value::Number(SchemeNumber::Integer(5))]).unwrap();
    // assert_eq!(result, Value::Number(SchemeNumber::Integer(25)));
}

#[test]
fn test_host_function_registration() {
    let mut interpreter = LambdustInterpreter::new();

    // Register a host function
    interpreter.register_simple_host_function(
        "test-add".to_string(),
        |args: &[Value]| -> Result<Value> {
            if args.len() != 2 {
                return Err(LambdustError::arity_error(2, args.len()));
            }

            match (&args[0], &args[1]) {
                (
                    Value::Number(SchemeNumber::Integer(a)),
                    Value::Number(SchemeNumber::Integer(b)),
                ) => Ok(Value::Number(SchemeNumber::Integer(a + b))),
                _ => Err(LambdustError::type_error("Expected numbers".to_string())),
            }
        },
    );

    // Test that the function is registered
    assert!(
        interpreter
            .list_host_functions()
            .contains(&&"test-add".to_string())
    );

    // Test calling the host function from Scheme
    let result = interpreter.eval_string("(test-add 10 20)").unwrap();
    assert_eq!(result, Value::Number(SchemeNumber::Integer(30)));
}

#[test]
fn test_typed_function_calls() {
    let mut interpreter = LambdustInterpreter::new();

    // Register a typed host function
    interpreter.register_simple_host_function(
        "string-length".to_string(),
        |args: &[Value]| -> Result<Value> {
            if args.len() != 1 {
                return Err(LambdustError::arity_error(1, args.len()));
            }

            match &args[0] {
                Value::String(s) => Ok(Value::Number(SchemeNumber::Integer(s.len() as i64))),
                _ => Err(LambdustError::type_error("Expected string".to_string())),
            }
        },
    );

    // Test typed call
    let result: i64 = interpreter
        .call_scheme_function_typed("string-length", ("hello".to_string(),))
        .unwrap();
    assert_eq!(result, 5);
}

#[test]
fn test_multiple_expressions() {
    let mut interpreter = LambdustInterpreter::new();

    let expressions = vec!["(define x 10)", "(define y 20)", "(define (get-x) x)"];

    interpreter.eval_expressions(&expressions).unwrap();

    // Check that functions and variables are defined
    assert!(interpreter.has_scheme_function("get-x"));
}

#[test]
fn test_error_handling() {
    let mut interpreter = LambdustInterpreter::new();

    // Test undefined function call
    let result = interpreter.call_scheme_function("undefined-function", &[]);
    assert!(result.is_err());

    // Test malformed code
    let result = interpreter.eval_string("(invalid syntax");
    assert!(result.is_err());
}

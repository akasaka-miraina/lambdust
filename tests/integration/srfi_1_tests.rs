//! SRFI 1: List Library tests
//!
//! Tests for the SRFI 1 List Library implementation

use lambdust::interpreter::LambdustInterpreter;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_interpreter() -> LambdustInterpreter {
        LambdustInterpreter::new()
    }

    #[test]
    fn test_take_function() {
        let mut interpreter = create_test_interpreter();

        // Test taking elements from a list
        let _result = interpreter.eval_string("(take '(1 2 3 4 5) 3)").unwrap();
        // This test verifies the function exists and basic structure
        // Full functionality testing will be added when evaluator integration is complete
    }

    #[test]
    fn test_drop_function() {
        let mut interpreter = create_test_interpreter();

        // Test dropping elements from a list
        let _result = interpreter.eval_string("(drop '(1 2 3 4 5) 2)").unwrap();
        // This test verifies the function exists and basic structure
    }

    #[test]
    fn test_concatenate_function() {
        let mut interpreter = create_test_interpreter();

        // Test concatenating lists
        let _result = interpreter
            .eval_string("(concatenate '(1 2) '(3 4) '(5 6))")
            .unwrap();
        // This test verifies the function exists and basic structure
    }

    #[test]
    fn test_delete_duplicates_function() {
        let mut interpreter = create_test_interpreter();

        // Test removing duplicates
        let _result = interpreter
            .eval_string("(delete-duplicates '(1 2 1 3 2 4))")
            .unwrap();
        // This test verifies the function exists and basic structure
    }

    #[test]
    fn test_fold_function_exists() {
        let mut interpreter = create_test_interpreter();

        // Test that fold function exists (placeholder implementation)
        let result = interpreter.eval_string("fold");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("fold should be a procedure");
        }
    }

    #[test]
    fn test_fold_right_function_exists() {
        let mut interpreter = create_test_interpreter();

        // Test that fold-right function exists (placeholder implementation)
        let result = interpreter.eval_string("fold-right");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("fold-right should be a procedure");
        }
    }

    #[test]
    fn test_filter_function_exists() {
        let mut interpreter = create_test_interpreter();

        // Test that filter function exists (placeholder implementation)
        let result = interpreter.eval_string("filter");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("filter should be a procedure");
        }
    }

    #[test]
    fn test_find_function_exists() {
        let mut interpreter = create_test_interpreter();

        // Test that find function exists (placeholder implementation)
        let result = interpreter.eval_string("find");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("find should be a procedure");
        }
    }

    #[test]
    fn test_any_function_exists() {
        let mut interpreter = create_test_interpreter();

        // Test that any function exists (placeholder implementation)
        let result = interpreter.eval_string("any");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("any should be a procedure");
        }
    }

    #[test]
    fn test_every_function_exists() {
        let mut interpreter = create_test_interpreter();

        // Test that every function exists (placeholder implementation)
        let result = interpreter.eval_string("every");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("every should be a procedure");
        }
    }

    #[test]
    fn test_srfi_1_library_available() {
        let mut interpreter = create_test_interpreter();

        // Test that SRFI 1 functions are available
        let functions = vec![
            "take",
            "drop",
            "concatenate",
            "delete-duplicates",
            "fold",
            "fold-right",
            "filter",
            "find",
            "any",
            "every",
        ];

        for func_name in functions {
            let result = interpreter.eval_string(func_name);
            assert!(result.is_ok(), "Function {} should be available", func_name);

            if let Ok(Value::Procedure(_)) = result {
                // Function exists as a procedure
            } else {
                panic!("{} should be a procedure", func_name);
            }
        }
    }

    // Integration tests for when evaluator integration is complete

    #[test]
    #[ignore] // Enable when evaluator integration is complete
    fn test_take_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter.eval_string("(take '(1 2 3 4 5) 3)").unwrap();
        let expected = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3)),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    #[ignore] // Enable when evaluator integration is complete
    fn test_drop_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter.eval_string("(drop '(1 2 3 4 5) 2)").unwrap();
        let expected = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(3)),
            Value::Number(SchemeNumber::Integer(4)),
            Value::Number(SchemeNumber::Integer(5)),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    #[ignore] // Enable when evaluator integration is complete
    fn test_concatenate_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(concatenate '(1 2) '(3 4))")
            .unwrap();
        let expected = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(1)),
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(3)),
            Value::Number(SchemeNumber::Integer(4)),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    #[ignore] // Enable when evaluator integration is complete  
    fn test_filter_integration() {
        let mut interpreter = create_test_interpreter();

        // Define a simple predicate function first
        interpreter
            .eval_string("(define (even? x) (= (remainder x 2) 0))")
            .unwrap();

        let result = interpreter
            .eval_string("(filter even? '(1 2 3 4 5 6))")
            .unwrap();
        let expected = Value::from_vector(vec![
            Value::Number(SchemeNumber::Integer(2)),
            Value::Number(SchemeNumber::Integer(4)),
            Value::Number(SchemeNumber::Integer(6)),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    #[ignore] // Enable when evaluator integration is complete
    fn test_fold_integration() {
        let mut interpreter = create_test_interpreter();

        // Test fold with addition
        let result = interpreter.eval_string("(fold + 0 '(1 2 3 4))").unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(10)));

        // Test fold with multiplication
        let result = interpreter.eval_string("(fold * 1 '(1 2 3 4))").unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(24)));
    }
}

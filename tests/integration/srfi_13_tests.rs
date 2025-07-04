//! SRFI 13: String Libraries tests
//!
//! Tests for the SRFI 13 String Libraries implementation

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
    fn test_string_null_function() {
        let mut interpreter = create_test_interpreter();

        // Test string-null? function exists
        let result = interpreter.eval_string("string-null?");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("string-null? should be a procedure");
        }
    }

    #[test]
    fn test_string_hash_function() {
        let mut interpreter = create_test_interpreter();

        // Test string-hash function exists
        let result = interpreter.eval_string("string-hash");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("string-hash should be a procedure");
        }
    }

    #[test]
    fn test_string_prefix_function() {
        let mut interpreter = create_test_interpreter();

        // Test string-prefix? function exists
        let result = interpreter.eval_string("string-prefix?");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("string-prefix? should be a procedure");
        }
    }

    #[test]
    fn test_string_suffix_function() {
        let mut interpreter = create_test_interpreter();

        // Test string-suffix? function exists
        let result = interpreter.eval_string("string-suffix?");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("string-suffix? should be a procedure");
        }
    }

    #[test]
    fn test_string_contains_function() {
        let mut interpreter = create_test_interpreter();

        // Test string-contains function exists
        let result = interpreter.eval_string("string-contains");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("string-contains should be a procedure");
        }
    }

    #[test]
    fn test_string_take_function() {
        let mut interpreter = create_test_interpreter();

        // Test string-take function exists
        let result = interpreter.eval_string("string-take");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("string-take should be a procedure");
        }
    }

    #[test]
    fn test_string_drop_function() {
        let mut interpreter = create_test_interpreter();

        // Test string-drop function exists
        let result = interpreter.eval_string("string-drop");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("string-drop should be a procedure");
        }
    }

    #[test]
    fn test_string_concatenate_function() {
        let mut interpreter = create_test_interpreter();

        // Test string-concatenate function exists
        let result = interpreter.eval_string("string-concatenate");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("string-concatenate should be a procedure");
        }
    }

    #[test]
    fn test_srfi_13_library_available() {
        let mut interpreter = create_test_interpreter();

        // Test that SRFI 13 functions are available
        let functions = vec![
            "string-null?",
            "string-every",
            "string-any",
            "string-compare",
            "string-compare-ci",
            "string-hash",
            "string-hash-ci",
            "string-prefix?",
            "string-suffix?",
            "string-prefix-ci?",
            "string-suffix-ci?",
            "string-index",
            "string-index-right",
            "string-skip",
            "string-skip-right",
            "string-count",
            "string-contains",
            "string-contains-ci",
            "string-take",
            "string-drop",
            "string-take-right",
            "string-drop-right",
            "string-pad",
            "string-pad-right",
            "string-trim",
            "string-trim-right",
            "string-trim-both",
            "string-concatenate",
            "string-concatenate-reverse",
            "string-join",
            "string-replace",
            "string-tokenize",
            "string-filter",
            "string-delete",
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
    
    fn test_string_null_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter.eval_string("(string-null? \"\")").unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter.eval_string("(string-null? \"hello\")").unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    
    fn test_string_hash_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter.eval_string("(string-hash \"hello\")").unwrap();
        assert!(matches!(result, Value::Number(_)));

        let result = interpreter
            .eval_string("(string-hash \"hello\" 1000)")
            .unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    
    fn test_string_prefix_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-prefix? \"hel\" \"hello\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter
            .eval_string("(string-prefix? \"world\" \"hello\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    
    fn test_string_suffix_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-suffix? \"llo\" \"hello\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter
            .eval_string("(string-suffix? \"world\" \"hello\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    
    fn test_string_contains_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-contains \"hello\" \"ell\")")
            .unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));

        let result = interpreter
            .eval_string("(string-contains \"hello\" \"world\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    
    fn test_string_take_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-take \"hello\" 3)")
            .unwrap();
        assert_eq!(result, Value::String("hel".to_string()));
    }

    #[test]
    
    fn test_string_drop_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-drop \"hello\" 2)")
            .unwrap();
        assert_eq!(result, Value::String("llo".to_string()));
    }

    #[test]
    
    fn test_string_take_right_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-take-right \"hello\" 3)")
            .unwrap();
        assert_eq!(result, Value::String("llo".to_string()));
    }

    #[test]
    
    fn test_string_drop_right_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-drop-right \"hello\" 2)")
            .unwrap();
        assert_eq!(result, Value::String("hel".to_string()));
    }

    #[test]
    
    fn test_string_concatenate_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-concatenate '(\"hello\" \" \" \"world\"))")
            .unwrap();
        assert_eq!(result, Value::String("hello world".to_string()));
    }

    #[test]
    
    fn test_string_prefix_ci_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-prefix-ci? \"HEL\" \"hello\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter
            .eval_string("(string-prefix-ci? \"WORLD\" \"hello\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    
    fn test_string_suffix_ci_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-suffix-ci? \"LLO\" \"hello\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter
            .eval_string("(string-suffix-ci? \"WORLD\" \"hello\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    
    fn test_string_contains_ci_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-contains-ci \"HELLO\" \"ell\")")
            .unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));

        let result = interpreter
            .eval_string("(string-contains-ci \"hello\" \"WORLD\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    
    fn test_string_hash_ci_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(string-hash-ci \"HELLO\")")
            .unwrap();
        assert!(matches!(result, Value::Number(_)));

        // Same string in different cases should have same hash
        let hash1 = interpreter
            .eval_string("(string-hash-ci \"hello\")")
            .unwrap();
        let hash2 = interpreter
            .eval_string("(string-hash-ci \"HELLO\")")
            .unwrap();
        assert_eq!(hash1, hash2);
    }
}

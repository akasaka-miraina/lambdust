//! SRFI 69: Basic Hash Tables tests
//!
//! Tests for the SRFI 69 Basic Hash Tables implementation

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
    fn test_make_hash_table_function() {
        let mut interpreter = create_test_interpreter();

        // Test make-hash-table function exists
        let result = interpreter.eval_string("make-hash-table");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("make-hash-table should be a procedure");
        }
    }

    #[test]
    fn test_hash_table_predicate_function() {
        let mut interpreter = create_test_interpreter();

        // Test hash-table? function exists
        let result = interpreter.eval_string("hash-table?");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("hash-table? should be a procedure");
        }
    }

    #[test]
    fn test_hash_table_ref_function() {
        let mut interpreter = create_test_interpreter();

        // Test hash-table-ref function exists
        let result = interpreter.eval_string("hash-table-ref");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("hash-table-ref should be a procedure");
        }
    }

    #[test]
    fn test_hash_table_set_function() {
        let mut interpreter = create_test_interpreter();

        // Test hash-table-set! function exists
        let result = interpreter.eval_string("hash-table-set!");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("hash-table-set! should be a procedure");
        }
    }

    #[test]
    fn test_hash_table_size_function() {
        let mut interpreter = create_test_interpreter();

        // Test hash-table-size function exists
        let result = interpreter.eval_string("hash-table-size");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("hash-table-size should be a procedure");
        }
    }

    #[test]
    fn test_hash_table_keys_function() {
        let mut interpreter = create_test_interpreter();

        // Test hash-table-keys function exists
        let result = interpreter.eval_string("hash-table-keys");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("hash-table-keys should be a procedure");
        }
    }

    #[test]
    fn test_hash_table_values_function() {
        let mut interpreter = create_test_interpreter();

        // Test hash-table-values function exists
        let result = interpreter.eval_string("hash-table-values");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("hash-table-values should be a procedure");
        }
    }

    #[test]
    fn test_hash_function() {
        let mut interpreter = create_test_interpreter();

        // Test hash function exists
        let result = interpreter.eval_string("hash");
        assert!(result.is_ok());

        if let Ok(Value::Procedure(_)) = result {
            // Function exists as a procedure
        } else {
            panic!("hash should be a procedure");
        }
    }

    #[test]
    fn test_srfi_69_library_available() {
        let mut interpreter = create_test_interpreter();

        // Test that SRFI 69 functions are available
        let functions = vec![
            "make-hash-table",
            "hash-table?",
            "hash-table-ref",
            "hash-table-ref/default",
            "hash-table-set!",
            "hash-table-delete!",
            "hash-table-exists?",
            "hash-table-size",
            "hash-table-keys",
            "hash-table-values",
            "hash-table->alist",
            "alist->hash-table",
            "hash-table-walk",
            "hash-table-fold",
            "hash-table-copy",
            "hash-table-merge!",
            "hash",
            "string-hash",
            "string-ci-hash",
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
    
    fn test_hash_table_creation_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter.eval_string("(make-hash-table)").unwrap();
        assert!(result.is_hash_table());
    }

    #[test]
    
    fn test_hash_table_predicate_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(hash-table? (make-hash-table))")
            .unwrap();
        assert_eq!(result, Value::Boolean(true));

        let result = interpreter
            .eval_string("(hash-table? \"not a hash table\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    
    fn test_hash_table_set_and_ref_integration() {
        let mut interpreter = create_test_interpreter();

        // Create hash table and set a value
        interpreter
            .eval_string("(define ht (make-hash-table))")
            .unwrap();
        interpreter
            .eval_string("(hash-table-set! ht \"key\" 42)")
            .unwrap();

        // Get value
        let result = interpreter
            .eval_string("(hash-table-ref ht \"key\")")
            .unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(42)));
    }

    #[test]
    
    fn test_hash_table_size_integration() {
        let mut interpreter = create_test_interpreter();

        interpreter
            .eval_string("(define ht (make-hash-table))")
            .unwrap();

        // Initially empty
        let result = interpreter.eval_string("(hash-table-size ht)").unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(0)));

        // Add one item
        interpreter
            .eval_string("(hash-table-set! ht \"test\" 123)")
            .unwrap();
        let result = interpreter.eval_string("(hash-table-size ht)").unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));
    }

    #[test]
    
    fn test_hash_table_exists_integration() {
        let mut interpreter = create_test_interpreter();

        interpreter
            .eval_string("(define ht (make-hash-table))")
            .unwrap();

        // Key doesn't exist initially
        let result = interpreter
            .eval_string("(hash-table-exists? ht \"test\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(false));

        // Set value
        interpreter
            .eval_string("(hash-table-set! ht \"test\" 42)")
            .unwrap();

        // Key now exists
        let result = interpreter
            .eval_string("(hash-table-exists? ht \"test\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    
    fn test_hash_table_delete_integration() {
        let mut interpreter = create_test_interpreter();

        interpreter
            .eval_string("(define ht (make-hash-table))")
            .unwrap();
        interpreter
            .eval_string("(hash-table-set! ht \"test\" 42)")
            .unwrap();

        // Delete value
        let result = interpreter
            .eval_string("(hash-table-delete! ht \"test\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(true));

        // Key no longer exists
        let result = interpreter
            .eval_string("(hash-table-exists? ht \"test\")")
            .unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    
    fn test_hash_table_keys_and_values_integration() {
        let mut interpreter = create_test_interpreter();

        interpreter
            .eval_string("(define ht (make-hash-table))")
            .unwrap();
        interpreter
            .eval_string("(hash-table-set! ht \"key1\" 1)")
            .unwrap();
        interpreter
            .eval_string("(hash-table-set! ht \"key2\" 2)")
            .unwrap();

        // Get keys
        let keys = interpreter.eval_string("(hash-table-keys ht)").unwrap();
        assert!(keys.is_list());

        // Get values
        let values = interpreter.eval_string("(hash-table-values ht)").unwrap();
        assert!(values.is_list());
    }

    #[test]
    
    fn test_hash_table_to_alist_integration() {
        let mut interpreter = create_test_interpreter();

        interpreter
            .eval_string("(define ht (make-hash-table))")
            .unwrap();
        interpreter
            .eval_string("(hash-table-set! ht \"key\" 42)")
            .unwrap();

        let alist = interpreter.eval_string("(hash-table->alist ht)").unwrap();
        assert!(alist.is_list());
    }

    #[test]
    
    fn test_alist_to_hash_table_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter
            .eval_string("(alist->hash-table '((\"key\" . 42) (\"other\" . 24)))")
            .unwrap();
        assert!(result.is_hash_table());
    }

    #[test]
    
    fn test_hash_table_copy_integration() {
        let mut interpreter = create_test_interpreter();

        interpreter
            .eval_string("(define ht (make-hash-table))")
            .unwrap();
        interpreter
            .eval_string("(hash-table-set! ht \"test\" 42)")
            .unwrap();

        // Copy hash table
        let copy = interpreter.eval_string("(hash-table-copy ht)").unwrap();
        assert!(copy.is_hash_table());

        // Value should exist in copy
        interpreter
            .eval_string("(define ht-copy (hash-table-copy ht))")
            .unwrap();
        let result = interpreter
            .eval_string("(hash-table-ref ht-copy \"test\")")
            .unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(42)));
    }

    #[test]
    
    fn test_hash_function_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter.eval_string("(hash \"test\")").unwrap();
        assert!(matches!(result, Value::Number(_)));

        let result = interpreter.eval_string("(hash \"test\" 1000)").unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    
    fn test_string_hash_integration() {
        let mut interpreter = create_test_interpreter();

        let result = interpreter.eval_string("(string-hash \"hello\")").unwrap();
        assert!(matches!(result, Value::Number(_)));
    }

    #[test]
    
    fn test_string_ci_hash_integration() {
        let mut interpreter = create_test_interpreter();

        // Same string in different cases should have same hash
        let hash1 = interpreter
            .eval_string("(string-ci-hash \"hello\")")
            .unwrap();
        let hash2 = interpreter
            .eval_string("(string-ci-hash \"HELLO\")")
            .unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    
    fn test_hash_table_ref_with_default_integration() {
        let mut interpreter = create_test_interpreter();

        interpreter
            .eval_string("(define ht (make-hash-table))")
            .unwrap();

        // Get non-existent key with default
        let result = interpreter
            .eval_string("(hash-table-ref/default ht \"missing\" 99)")
            .unwrap();
        assert_eq!(result, Value::Number(SchemeNumber::Integer(99)));
    }
}

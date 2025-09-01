//! Tests for SRFI-28: Basic format strings implementation

use lambdust::Lambdust;

/// Test basic ~a directive (any object)
#[test]
fn test_format_any_object() {
    let source = r#"(format "Hello ~a!" "World")"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_string(), Some("Hello World!"));
        }
        Err(e) => panic!("Format ~a test failed: {e:?}"),
    }
}

/// Test ~s directive (S-expression)
#[test]
fn test_format_s_expression() {
    let source = r#"(format "Value: ~s" "test")"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_string(), Some("Value: \"test\""));
        }
        Err(e) => panic!("Format ~s test failed: {e:?}"),
    }
}

/// Test ~d directive (decimal integer)
#[test]
fn test_format_decimal() {
    let source = r#"(format "Number: ~d" 42)"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_string(), Some("Number: 42"));
        }
        Err(e) => panic!("Format ~d test failed: {e:?}"),
    }
}

/// Test ~% directive (newline)
#[test]
fn test_format_newline() {
    let source = r#"(format "Line 1~%Line 2")"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_string(), Some("Line 1\nLine 2"));
        }
        Err(e) => panic!("Format ~% test failed: {e:?}"),
    }
}

/// Test ~~ directive (literal tilde)
#[test]
fn test_format_literal_tilde() {
    let source = r#"(format "Tilde: ~~")"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_string(), Some("Tilde: ~"));
        }
        Err(e) => panic!("Format ~~ test failed: {e:?}"),
    }
}

/// Test multiple directives in one format string
#[test]
fn test_format_multiple_directives() {
    let source = r#"(format "~a + ~a = ~d~%" 1 2 3)"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_string(), Some("1 + 2 = 3\n"));
        }
        Err(e) => panic!("Format multiple directives test failed: {e:?}"),
    }
}

/// Test format with no arguments (just format string)
#[test]
fn test_format_no_args() {
    let source = r#"(format "Hello, World!~~")"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_string(), Some("Hello, World!~"));
        }
        Err(e) => panic!("Format no args test failed: {e:?}"),
    }
}

/// Test format with boolean values
#[test]
fn test_format_boolean() {
    let source = r#"(format "Truth: ~a, Falsehood: ~a" #t #f)"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            let result_str = value.as_string().expect("Should be string");
            assert!(result_str.contains("Truth:"));
            assert!(result_str.contains("Falsehood:"));
        }
        Err(e) => panic!("Format boolean test failed: {e:?}"),
    }
}

/// Test format with lists
#[test]
fn test_format_list() {
    let source = r#"(format "List: ~s" '(1 2 3))"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            let result_str = value.as_string().expect("Should be string");
            assert!(result_str.contains("List:"));
            // The exact format may vary, but should contain list representation
            assert!(result_str.contains("1"));
            assert!(result_str.contains("2"));
            assert!(result_str.contains("3"));
        }
        Err(e) => panic!("Format list test failed: {e:?}"),
    }
}

/// Test error handling for wrong format string type
#[test]
fn test_format_error_wrong_type() {
    let source = r#"(format 123 "arg")"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Err(_) => {
            // Expected error - first argument must be string
        }
        Ok(value) => {
            panic!("Should have failed with wrong format string type, got: {value:?}");
        }
    }
}

/// Test error handling for not enough arguments
#[test]
fn test_format_error_not_enough_args() {
    let source = r#"(format "~a ~a" "only one")"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Err(_) => {
            // Expected error - not enough arguments
        }
        Ok(value) => {
            panic!("Should have failed with not enough arguments, got: {value:?}");
        }
    }
}

/// Test error handling for too many arguments
#[test]
fn test_format_error_too_many_args() {
    let source = r#"(format "~a" "arg1" "arg2")"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Err(_) => {
            // Expected error - too many arguments
        }
        Ok(value) => {
            panic!("Should have failed with too many arguments, got: {value:?}");
        }
    }
}

/// Test error handling for unknown directive
#[test]
fn test_format_error_unknown_directive() {
    let source = r#"(format "~x")"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Err(_) => {
            // Expected error - unknown directive ~x
        }
        Ok(value) => {
            panic!("Should have failed with unknown directive, got: {value:?}");
        }
    }
}

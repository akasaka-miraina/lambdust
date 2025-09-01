//! Tests for SRFI-13 String Libraries - Search operations

use lambdust::Lambdust;

/// Test string-index with character
#[test]
fn test_string_index_character() {
    let source = r#"
(string-index "hello world" #\o)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(4)); // First 'o' is at index 4
        }
        Err(e) => panic!("string-index character test failed: {e:?}"),
    }
}

/// Test string-index with character not found
#[test]
fn test_string_index_not_found() {
    let source = r#"
(string-index "hello world" #\x)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            // Debug output
            println!("DEBUG: Not found result = {value:?}");
            if let lambdust::eval::value::Value::Literal(lit) = &value {
                match lit {
                    lambdust::ast::literal::Literal::Boolean(b) => {
                        assert!(!(*b));
                    }
                    _ => panic!("Expected boolean literal, got: {lit:?}"),
                }
            } else {
                panic!("Expected literal value, got: {value:?}");
            }
        }
        Err(e) => panic!("string-index not found test failed: {e:?}"),
    }
}

/// Test string-index with start parameter
#[test]
fn test_string_index_with_start() {
    let source = r#"
(string-index "hello world" #\o 5)
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(7)); // 'o' in "world" at index 7
        }
        Err(e) => panic!("string-index with start test failed: {e:?}"),
    }
}

/// Test string-contains basic substring search
#[test]
fn test_string_contains_basic() {
    let source = r#"
(string-contains "hello world" "wor")
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(6)); // "wor" starts at index 6
        }
        Err(e) => panic!("string-contains basic test failed: {e:?}"),
    }
}

/// Test string-contains not found
#[test]
fn test_string_contains_not_found() {
    let source = r#"
(string-contains "hello world" "xyz")
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_boolean(), Some(false)); // Not found returns #f
        }
        Err(e) => panic!("string-contains not found test failed: {e:?}"),
    }
}

/// Test string-contains empty substring
#[test]
fn test_string_contains_empty() {
    let source = r#"
(string-contains "hello world" "")
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(0)); // Empty string matches at start
        }
        Err(e) => panic!("string-contains empty test failed: {e:?}"),
    }
}

/// Test string-tokenize basic functionality
#[test]
fn test_string_tokenize_basic() {
    let source = r#"
(string-tokenize "hello world test")
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            let tokens = value.as_list().expect("Should be a list");
            assert_eq!(tokens.len(), 3);
            assert_eq!(tokens[0].as_string(), Some("hello"));
            assert_eq!(tokens[1].as_string(), Some("world"));
            assert_eq!(tokens[2].as_string(), Some("test"));
        }
        Err(e) => panic!("string-tokenize basic test failed: {e:?}"),
    }
}

/// Test string-tokenize with multiple spaces
#[test]
fn test_string_tokenize_multiple_spaces() {
    let source = r#"
(string-tokenize "hello    world   test")
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            let tokens = value.as_list().expect("Should be a list");
            assert_eq!(tokens.len(), 3); // Should still be 3 tokens
            assert_eq!(tokens[0].as_string(), Some("hello"));
            assert_eq!(tokens[1].as_string(), Some("world"));
            assert_eq!(tokens[2].as_string(), Some("test"));
        }
        Err(e) => panic!("string-tokenize multiple spaces test failed: {e:?}"),
    }
}

/// Test string-tokenize empty string
#[test]
fn test_string_tokenize_empty() {
    let source = r#"
(string-tokenize "")
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            let tokens = value.as_list().expect("Should be a list");
            assert_eq!(tokens.len(), 0); // Empty string should return empty list
        }
        Err(e) => panic!("string-tokenize empty test failed: {e:?}"),
    }
}

/// Test string-tokenize whitespace only
#[test]
fn test_string_tokenize_whitespace_only() {
    let source = r#"
(string-tokenize "   \t  \n  ")
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            let tokens = value.as_list().expect("Should be a list");
            assert_eq!(tokens.len(), 0); // Only whitespace should return empty list
        }
        Err(e) => panic!("string-tokenize whitespace only test failed: {e:?}"),
    }
}

/// Test string-index with string criterion (character set)
#[test]
fn test_string_index_string_criterion() {
    let source = r#"
(string-index "hello world" "aeiou")
"#;

    let mut lambdust = Lambdust::new();
    let result = lambdust.eval(source, Some("test"));

    match result {
        Ok(value) => {
            assert_eq!(value.as_integer(), Some(1)); // First vowel 'e' at index 1
        }
        Err(e) => panic!("string-index string criterion test failed: {e:?}"),
    }
}

//! SRFI-13 (String Library) Comprehensive Compliance Tests
//!
//! This module provides comprehensive tests for SRFI-13 compliance in Lambdust.
//! SRFI-13 provides comprehensive string processing procedures that extend
//! the basic string operations of R7RS.
//!
//! Key SRFI-13 Categories:
//! - Predicates: string-null?, string-every, string-any
//! - Constructors: string-tabulate, etc.
//! - List/String conversion: string-join, reverse-list->string
//! - Selection: string-copy!, string-take, string-drop, string-pad/trim
//! - Comparison: string-compare, string-hash, case-insensitive variants
//! - Searching: string-prefix?, string-contains, string-index, string-count
//! - Case mapping: string-upcase!, string-downcase!, string-titlecase!
//! - Reverse/append: string-reverse, string-concatenate
//! - Fold/unfold/map: string-fold, string-map, string-for-each
//! - Miscellaneous: string-tokenize, string-filter, string-delete
//!
//! Reference: https://srfi.schemers.org/srfi-13/srfi-13.html

use lambdust::eval::evaluator::Evaluator;
use lambdust::eval::value::{Value, ThreadSafeEnvironment};
use lambdust::stdlib::create_standard_environment;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper: Create evaluator with SRFI-13 loaded
    fn create_srfi13_evaluator() -> (Evaluator, Arc<ThreadSafeEnvironment>) {
        let env = create_standard_environment();
        let evaluator = Evaluator::new();
        
        // Load SRFI-13
        let import_code = r#"(import (srfi 13))"#;
        if let Ok(import_expr) = lambdust::parser::parse(import_code) {
            let _ = evaluator.eval(&import_expr, &env);
        }
        
        (evaluator, env)
    }

    /// Test helper: Evaluate expression and return result
    fn eval_expr(code: &str) -> Result<Value, String> {
        let (mut evaluator, env) = create_srfi13_evaluator();
        
        match lambdust::parser::parse(code) {
            Ok(expr) => {
                match evaluator.eval(&expr, &env) {
                    Ok(value) => Ok(value),
                    Err(error) => Err(format!("Evaluation error: {:?}", error)),
                }
            },
            Err(parse_error) => Err(format!("Parse error: {:?}", parse_error))
        }
    }

    /// Test helper: Evaluate and expect boolean result
    fn expect_boolean(code: &str, expected: bool) -> bool {
        match eval_expr(code) {
            Ok(Value::Literal(lambdust::ast::Literal::Boolean(b))) => b == expected,
            Ok(other) => {
                println!("Expected boolean {}, got: {:?}", expected, other);
                false
            },
            Err(e) => {
                println!("Error evaluating '{}': {}", code, e);
                false
            }
        }
    }

    /// Test helper: Evaluate and expect integer result  
    fn expect_integer(code: &str, expected: i64) -> bool {
        match eval_expr(code) {
            Ok(Value::Literal(lambdust::ast::Literal::Number(n))) => n as i64 == expected,
            Ok(other) => {
                println!("Expected integer {}, got: {:?}", expected, other);
                false
            },
            Err(e) => {
                println!("Error evaluating '{}': {}", code, e);
                false
            }
        }
    }

    /// Test helper: Evaluate and expect string result
    fn expect_string(code: &str, expected: &str) -> bool {
        match eval_expr(code) {
            Ok(Value::Literal(lambdust::ast::Literal::String(s))) => s == expected,
            Ok(other) => {
                println!("Expected string '{}', got: {:?}", expected, other);
                false
            },
            Err(e) => {
                println!("Error evaluating '{}': {}", code, e);
                false
            }
        }
    }

    // ============= PREDICATE TESTS =============

    #[test]
    fn test_string_null() {
        assert!(expect_boolean(r#"(string-null? "")"#, true));
        assert!(expect_boolean(r#"(string-null? "hello")"#, false));
        assert!(expect_boolean(r#"(string-null? "a")"#, false));
    }

    #[test]
    fn test_string_every() {
        assert!(expect_boolean(r#"(string-every char-alphabetic? "abc")"#, true));
        assert!(expect_boolean(r#"(string-every char-alphabetic? "ab3")"#, false));
        assert!(expect_boolean(r#"(string-every char-alphabetic? "")"#, true));
        
        // With start/end indices
        assert!(expect_boolean(r#"(string-every char-alphabetic? "123abc" 3)"#, true));
        assert!(expect_boolean(r#"(string-every char-alphabetic? "123abc456" 3 6)"#, true));
    }

    #[test]
    fn test_string_any() {
        assert!(expect_boolean(r#"(string-any char-alphabetic? "123a")"#, true));
        assert!(expect_boolean(r#"(string-any char-alphabetic? "1234")"#, false));
        assert!(expect_boolean(r#"(string-any char-alphabetic? "")"#, false));
        
        // With start/end indices
        assert!(expect_boolean(r#"(string-any char-alphabetic? "123abc" 3)"#, true));
        assert!(expect_boolean(r#"(string-any char-alphabetic? "abc123def" 3 6)"#, false));
    }

    // ============= CONSTRUCTOR TESTS =============

    #[test]
    fn test_string_tabulate() {
        assert!(expect_string(r#"(string-tabulate 5 (lambda (i) #\a))"#, "aaaaa"));
        assert!(expect_string(r#"(string-tabulate 0 (lambda (i) #\x))"#, ""));
        
        // More complex tabulation
        assert!(expect_boolean(r#"
            (equal? (string-tabulate 3 (lambda (i) (integer->char (+ 65 i))))
                   "ABC")
        "#, true));
    }

    // ============= LIST & STRING CONVERSION TESTS =============

    #[test]
    fn test_reverse_list_to_string() {
        assert!(expect_string(r#"(reverse-list->string '(#\c #\b #\a))"#, "abc"));
        assert!(expect_string(r#"(reverse-list->string '())"#, ""));
    }

    #[test]
    fn test_string_join() {
        // Basic join
        assert!(expect_string(r#"(string-join '("a" "b" "c"))"#, "a b c"));
        assert!(expect_string(r#"(string-join '("a" "b" "c") ":")"#, "a:b:c"));
        
        // Grammar variations
        assert!(expect_string(r#"(string-join '("a" "b" "c") ":" 'infix)"#, "a:b:c"));
        assert!(expect_string(r#"(string-join '("a" "b" "c") ":" 'suffix)"#, "a:b:c:"));
        assert!(expect_string(r#"(string-join '("a" "b" "c") ":" 'prefix)"#, ":a:b:c"));
        
        // Empty list
        assert!(expect_string(r#"(string-join '())"#, ""));
    }

    // ============= SELECTION TESTS =============

    #[test]
    fn test_string_copy_with_indices() {
        // string-copy! is a mutation operation - test carefully
        assert!(expect_boolean(r#"
            (let ((target (make-string 10 #\x)))
              (string-copy! target 2 "hello")
              (equal? target "xxhelloxx"))
        "#, true));
        
        assert!(expect_boolean(r#"
            (let ((target (make-string 10 #\x)))
              (string-copy! target 0 "hello" 1 4)
              (equal? target "ellxxxxxxx"))
        "#, true));
    }

    #[test]
    fn test_string_take_drop() {
        assert!(expect_string(r#"(string-take "hello world" 5)"#, "hello"));
        assert!(expect_string(r#"(string-drop "hello world" 6)"#, "world"));
        assert!(expect_string(r#"(string-take "abc" 0)"#, ""));
        assert!(expect_string(r#"(string-drop "abc" 3)"#, ""));
    }

    #[test]
    fn test_string_take_drop_right() {
        assert!(expect_string(r#"(string-take-right "hello world" 5)"#, "world"));
        assert!(expect_string(r#"(string-drop-right "hello world" 6)"#, "hello"));
    }

    #[test]
    fn test_string_pad() {
        assert!(expect_string(r#"(string-pad "hello" 10)"#, "     hello"));
        assert!(expect_string(r#"(string-pad "hello" 3)"#, "llo"));
        assert!(expect_string(r#"(string-pad "hello" 10 #\*)"#, "*****hello"));
    }

    #[test]
    fn test_string_pad_right() {
        assert!(expect_string(r#"(string-pad-right "hello" 10)"#, "hello     "));
        assert!(expect_string(r#"(string-pad-right "hello" 3)"#, "hel"));
        assert!(expect_string(r#"(string-pad-right "hello" 10 #\*)"#, "hello*****"));
    }

    #[test]
    fn test_string_trim() {
        assert!(expect_string(r#"(string-trim "  hello  ")"#, "hello  "));
        assert!(expect_string(r#"(string-trim "aaahelloaaa" (lambda (c) (char=? c #\a)))"#, "helloaaa"));
        assert!(expect_string(r#"(string-trim "")"#, ""));
    }

    #[test]
    fn test_string_trim_right() {
        assert!(expect_string(r#"(string-trim-right "  hello  ")"#, "  hello"));
        assert!(expect_string(r#"(string-trim-right "aaahelloaaa" (lambda (c) (char=? c #\a)))"#, "aaahello"));
    }

    #[test]
    fn test_string_trim_both() {
        assert!(expect_string(r#"(string-trim-both "  hello  ")"#, "hello"));
        assert!(expect_string(r#"(string-trim-both "aaahelloaaa" (lambda (c) (char=? c #\a)))"#, "hello"));
    }

    // ============= COMPARISON TESTS =============

    #[test]
    fn test_string_compare() {
        assert!(expect_integer(r#"
            (string-compare "abc" "abc" 
                           (lambda (i) -1) 
                           (lambda (i) 0) 
                           (lambda (i) 1))
        "#, 0));
        
        assert!(expect_integer(r#"
            (string-compare "abc" "abd"
                           (lambda (i) -1)
                           (lambda (i) 0)
                           (lambda (i) 1))
        "#, -1));
    }

    #[test]
    fn test_string_compare_ci() {
        assert!(expect_integer(r#"
            (string-compare-ci "ABC" "abc"
                               (lambda (i) -1)
                               (lambda (i) 0)
                               (lambda (i) 1))
        "#, 0));
    }

    #[test]
    fn test_string_not_equal() {
        assert!(expect_boolean(r#"(string<> "abc" "def")"#, true));
        assert!(expect_boolean(r#"(string<> "abc" "abc")"#, false));
        assert!(expect_boolean(r#"(string-ci<> "ABC" "abc")"#, false));
    }

    #[test]
    fn test_string_hash() {
        // Basic hash operation
        assert!(expect_boolean(r#"(integer? (string-hash "hello"))"#, true));
        assert!(expect_boolean(r#"(>= (string-hash "hello") 0)"#, true));
        
        // Same strings should have same hash
        assert!(expect_boolean(r#"(= (string-hash "test") (string-hash "test"))"#, true));
        
        // With bound
        assert!(expect_boolean(r#"(< (string-hash "hello" 100) 100)"#, true));
    }

    #[test]
    fn test_string_hash_ci() {
        // Case-insensitive hash
        assert!(expect_boolean(r#"(= (string-hash-ci "Hello") (string-hash-ci "HELLO"))"#, true));
        assert!(expect_boolean(r#"(= (string-hash-ci "Hello") (string-hash-ci "hello"))"#, true));
    }

    // ============= SEARCHING TESTS =============

    #[test]
    fn test_string_prefix_length() {
        assert!(expect_integer(r#"(string-prefix-length "hello" "help")"#, 3));
        assert!(expect_integer(r#"(string-prefix-length "abc" "def")"#, 0));
        assert!(expect_integer(r#"(string-prefix-length "test" "test")"#, 4));
    }

    #[test]
    fn test_string_suffix_length() {
        assert!(expect_integer(r#"(string-suffix-length "hello" "bello")"#, 4));
        assert!(expect_integer(r#"(string-suffix-length "abc" "def")"#, 0));
        assert!(expect_integer(r#"(string-suffix-length "test" "test")"#, 4));
    }

    #[test]
    fn test_string_prefix() {
        assert!(expect_boolean(r#"(string-prefix? "hel" "hello")"#, true));
        assert!(expect_boolean(r#"(string-prefix? "hello" "hel")"#, false));
        assert!(expect_boolean(r#"(string-prefix? "" "hello")"#, true));
        assert!(expect_boolean(r#"(string-prefix? "test" "test")"#, true));
    }

    #[test]
    fn test_string_suffix() {
        assert!(expect_boolean(r#"(string-suffix? "llo" "hello")"#, true));
        assert!(expect_boolean(r#"(string-suffix? "hello" "llo")"#, false));
        assert!(expect_boolean(r#"(string-suffix? "" "hello")"#, true));
        assert!(expect_boolean(r#"(string-suffix? "test" "test")"#, true));
    }

    #[test]
    fn test_string_index() {
        assert!(expect_integer(r#"(string-index "hello" #\l)"#, 2));
        assert!(expect_boolean(r#"(not (string-index "hello" #\x))"#, true));
        assert!(expect_integer(r#"(string-index "hello" char-alphabetic?)"#, 0));
        
        // With start/end
        assert!(expect_integer(r#"(string-index "hello" #\l 3)"#, 3));
        assert!(expect_boolean(r#"(not (string-index "hello" #\h 1))"#, true));
    }

    #[test]
    fn test_string_index_right() {
        assert!(expect_integer(r#"(string-index-right "hello" #\l)"#, 3));
        assert!(expect_boolean(r#"(not (string-index-right "hello" #\x))"#, true));
        assert!(expect_integer(r#"(string-index-right "hello" char-alphabetic?)"#, 4));
    }

    #[test]
    fn test_string_skip() {
        assert!(expect_integer(r#"(string-skip "aaabbc" #\a)"#, 3));
        assert!(expect_boolean(r#"(not (string-skip "aaa" #\a))"#, true));
        assert!(expect_integer(r#"(string-skip "123abc" char-numeric?)"#, 3));
    }

    #[test]
    fn test_string_count() {
        assert!(expect_integer(r#"(string-count "hello" #\l)"#, 2));
        assert!(expect_integer(r#"(string-count "hello" #\x)"#, 0));
        assert!(expect_integer(r#"(string-count "hello world" char-alphabetic?)"#, 10));
    }

    #[test]
    fn test_string_contains() {
        assert!(expect_integer(r#"(string-contains "hello world" "wor")"#, 6));
        assert!(expect_boolean(r#"(not (string-contains "hello" "xyz"))"#, true));
        assert!(expect_integer(r#"(string-contains "abcabc" "bc")"#, 1));
    }

    #[test]
    fn test_string_contains_ci() {
        assert!(expect_integer(r#"(string-contains-ci "Hello World" "WOR")"#, 6));
        assert!(expect_integer(r#"(string-contains-ci "HELLO" "hello")"#, 0));
        assert!(expect_boolean(r#"(not (string-contains-ci "hello" "XYZ"))"#, true));
    }

    // ============= CASE MAPPING TESTS =============

    #[test]
    fn test_string_upcase_downcase() {
        // Test immutable versions (R7RS standard)
        assert!(expect_string(r#"(string-upcase "hello")"#, "HELLO"));
        assert!(expect_string(r#"(string-downcase "HELLO")"#, "hello"));
        assert!(expect_string(r#"(string-upcase "Hello World")"#, "HELLO WORLD"));
    }

    #[test]
    fn test_string_upcase_downcase_mutating() {
        // Test mutating versions (SRFI-13 specific)
        assert!(expect_boolean(r#"
            (let ((s (string-copy "hello")))
              (string-upcase! s)
              (equal? s "HELLO"))
        "#, true));
        
        assert!(expect_boolean(r#"
            (let ((s (string-copy "HELLO")))
              (string-downcase! s)
              (equal? s "hello"))
        "#, true));
    }

    #[test]
    fn test_string_titlecase() {
        assert!(expect_string(r#"(string-titlecase "hello world")"#, "Hello World"));
        assert!(expect_string(r#"(string-titlecase "tHe qUiCk bRoWn")"#, "The Quick Brown"));
        
        // Mutating version
        assert!(expect_boolean(r#"
            (let ((s (string-copy "hello world")))
              (string-titlecase! s)
              (equal? s "Hello World"))
        "#, true));
    }

    // ============= REVERSE & APPEND TESTS =============

    #[test]
    fn test_string_reverse() {
        assert!(expect_string(r#"(string-reverse "hello")"#, "olleh"));
        assert!(expect_string(r#"(string-reverse "")"#, ""));
        assert!(expect_string(r#"(string-reverse "a")"#, "a"));
    }

    #[test]
    fn test_string_reverse_mutating() {
        assert!(expect_boolean(r#"
            (let ((s (string-copy "hello")))
              (string-reverse! s)
              (equal? s "olleh"))
        "#, true));
    }

    #[test]
    fn test_string_concatenate() {
        assert!(expect_string(r#"(string-concatenate '("hello" " " "world"))"#, "hello world"));
        assert!(expect_string(r#"(string-concatenate '())"#, ""));
        assert!(expect_string(r#"(string-concatenate '("single"))"#, "single"));
    }

    #[test]
    fn test_string_concatenate_reverse() {
        assert!(expect_string(r#"(string-concatenate-reverse '("c" "b" "a"))"#, "abc"));
        assert!(expect_string(r#"(string-concatenate-reverse '("c" "b" "a") "!")"#, "abc!"));
    }

    // ============= FOLD, UNFOLD & MAP TESTS =============

    #[test]
    fn test_string_map() {
        assert!(expect_string(r#"(string-map char-upcase "hello")"#, "HELLO"));
        assert!(expect_string(r#"(string-map (lambda (c) #\*) "hello")"#, "*****"));
    }

    #[test]
    fn test_string_map_mutating() {
        assert!(expect_boolean(r#"
            (let ((s (string-copy "hello")))
              (string-map! char-upcase s)
              (equal? s "HELLO"))
        "#, true));
    }

    #[test]
    fn test_string_fold() {
        // Count characters
        assert!(expect_integer(r#"(string-fold (lambda (c count) (+ count 1)) 0 "hello")"#, 5));
        
        // Build reversed string
        assert!(expect_string(r#"
            (list->string (string-fold (lambda (c acc) (cons c acc)) '() "hello"))
        "#, "olleh"));
    }

    #[test]
    fn test_string_fold_right() {
        // Build normal string
        assert!(expect_string(r#"
            (list->string (string-fold-right (lambda (c acc) (cons c acc)) '() "hello"))
        "#, "hello"));
    }

    #[test]
    fn test_string_unfold() {
        assert!(expect_string(r#"
            (string-unfold (lambda (i) (> i 4))
                          (lambda (i) (integer->char (+ 65 i)))
                          (lambda (i) (+ i 1))
                          0)
        "#, "ABCDE"));
    }

    #[test]
    fn test_string_for_each() {
        // Test that string-for-each executes but doesn't return meaningful value
        assert!(expect_boolean(r#"
            (let ((count 0))
              (string-for-each (lambda (c) (set! count (+ count 1))) "hello")
              (= count 5))
        "#, true));
    }

    // ============= MISCELLANEOUS TESTS =============

    #[test]
    fn test_string_tokenize() {
        // Default tokenization
        assert!(expect_boolean(r#"
            (equal? (string-tokenize "hello world test")
                   '("hello" "world" "test"))
        "#, true));
        
        // Custom token predicate
        assert!(expect_boolean(r#"
            (equal? (string-tokenize "a1b2c3" char-alphabetic?)
                   '("a" "b" "c"))
        "#, true));
    }

    #[test]
    fn test_string_filter() {
        assert!(expect_string(r#"(string-filter char-alphabetic? "a1b2c3")"#, "abc"));
        assert!(expect_string(r#"(string-filter (lambda (c) (char=? c #\l)) "hello")"#, "ll"));
    }

    #[test]
    fn test_string_delete() {
        assert!(expect_string(r#"(string-delete char-numeric? "a1b2c3")"#, "abc"));
        assert!(expect_string(r#"(string-delete (lambda (c) (char=? c #\l)) "hello")"#, "heo"));
    }

    // ============= INTEGRATION TESTS =============

    #[test]
    fn test_srfi13_comprehensive_integration() {
        // Complex test combining multiple SRFI-13 features
        assert!(expect_boolean(r#"
            (let* ((text "  Hello, World!  ")
                   (trimmed (string-trim-both text))
                   (words (string-tokenize trimmed char-alphabetic?))
                   (uppercased (map string-upcase words))
                   (joined (string-join uppercased "-"))
                   (padded (string-pad-right joined 20 #\*)))
              (and (equal? trimmed "Hello, World!")
                   (equal? words '("Hello" "World"))
                   (equal? uppercased '("HELLO" "WORLD"))
                   (equal? joined "HELLO-WORLD")
                   (equal? padded "HELLO-WORLD*********")))
        "#, true));
    }

    #[test]
    fn test_srfi13_text_processing_example() {
        // Realistic text processing example
        assert!(expect_boolean(r#"
            (define (process-csv-line line)
              (let* ((fields (string-tokenize line (lambda (c) (not (char=? c #\,)))))
                     (trimmed-fields (map string-trim-both fields))
                     (non-empty-fields (filter (lambda (s) (not (string-null? s))) trimmed-fields)))
                non-empty-fields))
            
            (let ((csv-line "  Alice  , , Bob , Charlie  ,  "))
              (equal? (process-csv-line csv-line)
                     '("Alice" "Bob" "Charlie")))
        "#, true));
    }

    #[test]
    fn test_srfi13_performance_characteristics() {
        // Basic performance test with reasonable sized strings
        assert!(expect_boolean(r#"
            (let* ((large-string (make-string 1000 #\a))
                   (modified (string-map (lambda (c i) 
                                          (if (even? i) #\A #\a)) 
                                        large-string))
                   (found-index (string-index modified #\A)))
              (and (= (string-length modified) 1000)
                   (= found-index 0)
                   (= (string-count modified #\A) 500)))
        "#, true));
    }

    #[test]
    fn test_srfi13_comprehensive_compliance() {
        // Comprehensive test covering all major SRFI-13 categories
        assert!(expect_boolean(r#"
            (let* ((test-string "Hello World")
                   (empty-string "")
                   (whitespace-string "  test  "))
              (and
                ;; Predicates
                (string-null? empty-string)
                (not (string-null? test-string))
                (string-every char-alphabetic? "abc")
                (string-any char-numeric? "abc123")
                
                ;; Selection operations
                (equal? (string-take test-string 5) "Hello")
                (equal? (string-drop test-string 6) "World")
                (equal? (string-trim whitespace-string) "test  ")
                
                ;; Searching operations
                (string-prefix? "Hello" test-string)
                (string-suffix? "World" test-string)
                (= (string-index test-string #\l) 2)
                (= (string-count test-string #\l) 3)
                
                ;; Case operations
                (equal? (string-upcase test-string) "HELLO WORLD")
                (equal? (string-downcase test-string) "hello world")
                
                ;; Reverse and concatenate
                (equal? (string-reverse "abc") "cba")
                (equal? (string-concatenate '("a" "b" "c")) "abc")
                
                ;; Tokenization
                (equal? (string-tokenize test-string char-alphabetic?)
                       '("Hello" "World"))))
        "#, true));
    }
}
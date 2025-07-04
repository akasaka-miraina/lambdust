//! Unit tests for string and character builtin functions
//!
//! Tests all string/character operations for correctness, edge cases, and error handling.

use lambdust::builtins::string_char::register_string_char_functions;
use lambdust::error::LambdustError;
use lambdust::lexer::SchemeNumber;
use lambdust::value::{Procedure, Value};
use std::collections::HashMap;

/// Helper function to get a builtin function by name
fn get_builtin(name: &str) -> Value {
    let mut builtins = HashMap::new();
    register_string_char_functions(&mut builtins);
    builtins.get(name).unwrap().clone()
}

/// Helper function to call a builtin function
fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value, LambdustError> {
    let func = get_builtin(name);
    if let Value::Procedure(Procedure::Builtin { func, .. }) = func {
        func(&args)
    } else {
        panic!("Expected builtin function for {}", name);
    }
}

/// Helper to create string value
fn string(s: &str) -> Value {
    Value::String(s.to_string())
}

/// Helper to create character value
fn char(c: char) -> Value {
    Value::Character(c)
}

/// Helper to create integer value
fn int(n: i64) -> Value {
    Value::Number(SchemeNumber::Integer(n))
}

/// Helper to create real value
fn real(n: f64) -> Value {
    Value::Number(SchemeNumber::Real(n))
}

/// Helper to create symbol value
fn symbol(s: &str) -> Value {
    Value::Symbol(s.to_string())
}

/// Helper to create list value from vector
fn list(items: Vec<Value>) -> Value {
    Value::from_vector(items)
}

#[cfg(test)]
mod string_operations_tests {
    use super::*;

    #[test]
    fn test_string_length() {
        // Basic string length
        assert_eq!(call_builtin("string-length", vec![string("hello")]).unwrap(), int(5));
        assert_eq!(call_builtin("string-length", vec![string("")]).unwrap(), int(0));
        assert_eq!(call_builtin("string-length", vec![string("a")]).unwrap(), int(1));
        
        // Unicode characters
        assert_eq!(call_builtin("string-length", vec![string("こんにちは")]).unwrap(), int(5));
        assert_eq!(call_builtin("string-length", vec![string("🎉🚀")]).unwrap(), int(2));
        
        // Arity error
        assert!(call_builtin("string-length", vec![]).is_err());
        assert!(call_builtin("string-length", vec![string("a"), string("b")]).is_err());
        
        // Type error
        assert!(call_builtin("string-length", vec![int(42)]).is_err());
    }

    #[test]
    fn test_string_ref() {
        // Basic string indexing
        assert_eq!(call_builtin("string-ref", vec![string("hello"), int(0)]).unwrap(), char('h'));
        assert_eq!(call_builtin("string-ref", vec![string("hello"), int(4)]).unwrap(), char('o'));
        assert_eq!(call_builtin("string-ref", vec![string("world"), int(2)]).unwrap(), char('r'));
        
        // Unicode characters
        assert_eq!(call_builtin("string-ref", vec![string("こんにちは"), int(0)]).unwrap(), char('こ'));
        assert_eq!(call_builtin("string-ref", vec![string("こんにちは"), int(2)]).unwrap(), char('に'));
        
        // Index out of bounds
        assert!(call_builtin("string-ref", vec![string("hello"), int(5)]).is_err());
        assert!(call_builtin("string-ref", vec![string("hello"), int(-1)]).is_err());
        assert!(call_builtin("string-ref", vec![string(""), int(0)]).is_err());
        
        // Arity errors
        assert!(call_builtin("string-ref", vec![string("hello")]).is_err());
        assert!(call_builtin("string-ref", vec![]).is_err());
        
        // Type errors
        assert!(call_builtin("string-ref", vec![int(42), int(0)]).is_err());
        assert!(call_builtin("string-ref", vec![string("hello"), string("0")]).is_err());
    }

    #[test]
    fn test_string_append() {
        // Basic string concatenation
        assert_eq!(call_builtin("string-append", vec![string("hello"), string(" world")]).unwrap(), string("hello world"));
        assert_eq!(call_builtin("string-append", vec![string("a"), string("b"), string("c")]).unwrap(), string("abc"));
        
        // Empty strings
        assert_eq!(call_builtin("string-append", vec![string(""), string("hello")]).unwrap(), string("hello"));
        assert_eq!(call_builtin("string-append", vec![string("hello"), string("")]).unwrap(), string("hello"));
        assert_eq!(call_builtin("string-append", vec![string(""), string(""), string("")]).unwrap(), string(""));
        
        // No arguments (variadic)
        assert_eq!(call_builtin("string-append", vec![]).unwrap(), string(""));
        
        // Single argument
        assert_eq!(call_builtin("string-append", vec![string("single")]).unwrap(), string("single"));
        
        // Unicode strings
        assert_eq!(call_builtin("string-append", vec![string("こんにちは"), string("世界")]).unwrap(), string("こんにちは世界"));
        
        // Type errors
        assert!(call_builtin("string-append", vec![string("hello"), int(42)]).is_err());
        assert!(call_builtin("string-append", vec![int(42), string("world")]).is_err());
    }

    #[test]
    fn test_substring() {
        // Basic substring extraction
        assert_eq!(call_builtin("substring", vec![string("hello"), int(1), int(4)]).unwrap(), string("ell"));
        assert_eq!(call_builtin("substring", vec![string("world"), int(0), int(3)]).unwrap(), string("wor"));
        assert_eq!(call_builtin("substring", vec![string("test"), int(2), int(4)]).unwrap(), string("st"));
        
        // Two-argument form (start to end)
        assert_eq!(call_builtin("substring", vec![string("hello"), int(2)]).unwrap(), string("llo"));
        assert_eq!(call_builtin("substring", vec![string("world"), int(0)]).unwrap(), string("world"));
        
        // Empty substring
        assert_eq!(call_builtin("substring", vec![string("hello"), int(2), int(2)]).unwrap(), string(""));
        
        // Full string
        assert_eq!(call_builtin("substring", vec![string("hello"), int(0), int(5)]).unwrap(), string("hello"));
        assert_eq!(call_builtin("substring", vec![string("hello"), int(0)]).unwrap(), string("hello"));
        
        // Unicode strings
        assert_eq!(call_builtin("substring", vec![string("こんにちは"), int(1), int(3)]).unwrap(), string("んに"));
        
        // Edge cases
        assert_eq!(call_builtin("substring", vec![string(""), int(0), int(0)]).unwrap(), string(""));
        assert_eq!(call_builtin("substring", vec![string(""), int(0)]).unwrap(), string(""));
        
        // Invalid ranges
        assert!(call_builtin("substring", vec![string("hello"), int(6)]).is_err()); // Start beyond length
        assert!(call_builtin("substring", vec![string("hello"), int(0), int(6)]).is_err()); // End beyond length
        assert!(call_builtin("substring", vec![string("hello"), int(3), int(1)]).is_err()); // Start > end
        assert!(call_builtin("substring", vec![string("hello"), int(-1), int(3)]).is_err()); // Negative start
        
        // Arity errors
        assert!(call_builtin("substring", vec![string("hello")]).is_err());
        assert!(call_builtin("substring", vec![]).is_err());
        assert!(call_builtin("substring", vec![string("a"), int(0), int(1), int(2)]).is_err());
        
        // Type errors
        assert!(call_builtin("substring", vec![int(42), int(0), int(1)]).is_err());
        assert!(call_builtin("substring", vec![string("hello"), string("0"), int(1)]).is_err());
        assert!(call_builtin("substring", vec![string("hello"), int(0), string("1")]).is_err());
    }

    #[test]
    fn test_make_string() {
        // Basic make-string
        assert_eq!(call_builtin("make-string", vec![int(5), char('a')]).unwrap(), string("aaaaa"));
        assert_eq!(call_builtin("make-string", vec![int(3), char('x')]).unwrap(), string("xxx"));
        assert_eq!(call_builtin("make-string", vec![int(0), char('z')]).unwrap(), string(""));
        
        // Default fill character (space)
        assert_eq!(call_builtin("make-string", vec![int(4)]).unwrap(), string("    "));
        assert_eq!(call_builtin("make-string", vec![int(0)]).unwrap(), string(""));
        
        // Unicode characters
        assert_eq!(call_builtin("make-string", vec![int(3), char('あ')]).unwrap(), string("あああ"));
        
        // Arity errors
        assert!(call_builtin("make-string", vec![]).is_err());
        assert!(call_builtin("make-string", vec![int(1), char('a'), char('b')]).is_err());
        
        // Type errors
        assert!(call_builtin("make-string", vec![string("5"), char('a')]).is_err());
        assert!(call_builtin("make-string", vec![int(5), string("a")]).is_err());
        assert!(call_builtin("make-string", vec![real(5.5), char('a')]).is_err());
        
        // Negative length
        assert!(call_builtin("make-string", vec![int(-1), char('a')]).is_err());
    }

    #[test]
    fn test_string_constructor() {
        // Basic string construction from characters
        assert_eq!(call_builtin("string", vec![char('h'), char('e'), char('l'), char('l'), char('o')]).unwrap(), string("hello"));
        assert_eq!(call_builtin("string", vec![char('a')]).unwrap(), string("a"));
        
        // Empty string construction
        assert_eq!(call_builtin("string", vec![]).unwrap(), string(""));
        
        // Unicode characters
        assert_eq!(call_builtin("string", vec![char('こ'), char('ん')]).unwrap(), string("こん"));
        
        // Mixed characters
        assert_eq!(call_builtin("string", vec![char('H'), char('i'), char('!'), char('👋')]).unwrap(), string("Hi!👋"));
        
        // Type errors
        assert!(call_builtin("string", vec![char('a'), string("b")]).is_err());
        assert!(call_builtin("string", vec![int(65), char('b')]).is_err());
    }
}

#[cfg(test)]
mod string_comparison_tests {
    use super::*;

    #[test]
    fn test_string_equal() {
        // Basic equality
        assert_eq!(call_builtin("string=?", vec![string("hello"), string("hello")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string=?", vec![string("hello"), string("world")]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("string=?", vec![string(""), string("")]).unwrap(), Value::Boolean(true));
        
        // Multiple arguments
        assert_eq!(call_builtin("string=?", vec![string("test"), string("test"), string("test")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string=?", vec![string("test"), string("test"), string("other")]).unwrap(), Value::Boolean(false));
        
        // Case sensitivity
        assert_eq!(call_builtin("string=?", vec![string("Hello"), string("hello")]).unwrap(), Value::Boolean(false));
        
        // Unicode strings
        assert_eq!(call_builtin("string=?", vec![string("こんにちは"), string("こんにちは")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string=?", vec![string("こんにちは"), string("さようなら")]).unwrap(), Value::Boolean(false));
        
        // Arity errors
        assert!(call_builtin("string=?", vec![string("hello")]).is_err());
        assert!(call_builtin("string=?", vec![]).is_err());
        
        // Type errors
        assert!(call_builtin("string=?", vec![string("hello"), int(42)]).is_err());
    }

    #[test]
    fn test_string_less_than() {
        // Basic lexicographic comparison
        assert_eq!(call_builtin("string<?", vec![string("abc"), string("def")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string<?", vec![string("def"), string("abc")]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("string<?", vec![string("hello"), string("hello")]).unwrap(), Value::Boolean(false));
        
        // Length differences
        assert_eq!(call_builtin("string<?", vec![string("a"), string("aa")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string<?", vec![string("aa"), string("a")]).unwrap(), Value::Boolean(false));
        
        // Empty strings
        assert_eq!(call_builtin("string<?", vec![string(""), string("a")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string<?", vec![string("a"), string("")]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("string<?", vec![string(""), string("")]).unwrap(), Value::Boolean(false));
        
        // Multiple arguments (transitive)
        assert_eq!(call_builtin("string<?", vec![string("a"), string("b"), string("c")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string<?", vec![string("a"), string("c"), string("b")]).unwrap(), Value::Boolean(false));
        
        // Case sensitivity
        assert_eq!(call_builtin("string<?", vec![string("A"), string("a")]).unwrap(), Value::Boolean(true)); // ASCII: A=65, a=97
        
        // Arity errors
        assert!(call_builtin("string<?", vec![string("hello")]).is_err());
        assert!(call_builtin("string<?", vec![]).is_err());
        
        // Type errors
        assert!(call_builtin("string<?", vec![string("hello"), int(42)]).is_err());
    }

    #[test]
    fn test_string_greater_than() {
        // Basic comparison
        assert_eq!(call_builtin("string>?", vec![string("def"), string("abc")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string>?", vec![string("abc"), string("def")]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("string>?", vec![string("hello"), string("hello")]).unwrap(), Value::Boolean(false));
        
        // Multiple arguments
        assert_eq!(call_builtin("string>?", vec![string("c"), string("b"), string("a")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string>?", vec![string("c"), string("a"), string("b")]).unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_string_less_equal() {
        // Basic comparison
        assert_eq!(call_builtin("string<=?", vec![string("abc"), string("def")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string<=?", vec![string("hello"), string("hello")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string<=?", vec![string("def"), string("abc")]).unwrap(), Value::Boolean(false));
        
        // Multiple arguments
        assert_eq!(call_builtin("string<=?", vec![string("a"), string("b"), string("b")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string<=?", vec![string("a"), string("c"), string("b")]).unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_string_greater_equal() {
        // Basic comparison
        assert_eq!(call_builtin("string>=?", vec![string("def"), string("abc")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string>=?", vec![string("hello"), string("hello")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string>=?", vec![string("abc"), string("def")]).unwrap(), Value::Boolean(false));
        
        // Multiple arguments
        assert_eq!(call_builtin("string>=?", vec![string("c"), string("b"), string("b")]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("string>=?", vec![string("c"), string("a"), string("b")]).unwrap(), Value::Boolean(false));
    }
}

#[cfg(test)]
mod character_operations_tests {
    use super::*;

    #[test]
    fn test_char_equal() {
        // Basic equality
        assert_eq!(call_builtin("char=?", vec![char('a'), char('a')]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("char=?", vec![char('a'), char('b')]).unwrap(), Value::Boolean(false));
        
        // Multiple arguments
        assert_eq!(call_builtin("char=?", vec![char('x'), char('x'), char('x')]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("char=?", vec![char('x'), char('x'), char('y')]).unwrap(), Value::Boolean(false));
        
        // Case sensitivity
        assert_eq!(call_builtin("char=?", vec![char('A'), char('a')]).unwrap(), Value::Boolean(false));
        
        // Unicode characters
        assert_eq!(call_builtin("char=?", vec![char('あ'), char('あ')]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("char=?", vec![char('あ'), char('い')]).unwrap(), Value::Boolean(false));
        
        // Arity errors
        assert!(call_builtin("char=?", vec![char('a')]).is_err());
        assert!(call_builtin("char=?", vec![]).is_err());
        
        // Type errors
        assert!(call_builtin("char=?", vec![char('a'), string("a")]).is_err());
    }

    #[test]
    fn test_char_less_than() {
        // Basic comparison
        assert_eq!(call_builtin("char<?", vec![char('a'), char('b')]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("char<?", vec![char('b'), char('a')]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("char<?", vec![char('a'), char('a')]).unwrap(), Value::Boolean(false));
        
        // Case comparison
        assert_eq!(call_builtin("char<?", vec![char('A'), char('a')]).unwrap(), Value::Boolean(true)); // ASCII: A=65, a=97
        
        // Multiple arguments
        assert_eq!(call_builtin("char<?", vec![char('a'), char('b'), char('c')]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("char<?", vec![char('a'), char('c'), char('b')]).unwrap(), Value::Boolean(false));
        
        // Numbers vs letters
        assert_eq!(call_builtin("char<?", vec![char('1'), char('a')]).unwrap(), Value::Boolean(true)); // ASCII: 1=49, a=97
    }

    #[test]
    fn test_char_greater_than() {
        // Basic comparison
        assert_eq!(call_builtin("char>?", vec![char('b'), char('a')]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("char>?", vec![char('a'), char('b')]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("char>?", vec![char('a'), char('a')]).unwrap(), Value::Boolean(false));
        
        // Multiple arguments
        assert_eq!(call_builtin("char>?", vec![char('c'), char('b'), char('a')]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("char>?", vec![char('c'), char('a'), char('b')]).unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_char_less_equal() {
        // Basic comparison
        assert_eq!(call_builtin("char<=?", vec![char('a'), char('b')]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("char<=?", vec![char('a'), char('a')]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("char<=?", vec![char('b'), char('a')]).unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_char_greater_equal() {
        // Basic comparison
        assert_eq!(call_builtin("char>=?", vec![char('b'), char('a')]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("char>=?", vec![char('a'), char('a')]).unwrap(), Value::Boolean(true));
        assert_eq!(call_builtin("char>=?", vec![char('a'), char('b')]).unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_char_to_integer() {
        // Basic ASCII characters
        assert_eq!(call_builtin("char->integer", vec![char('A')]).unwrap(), int(65));
        assert_eq!(call_builtin("char->integer", vec![char('a')]).unwrap(), int(97));
        assert_eq!(call_builtin("char->integer", vec![char('0')]).unwrap(), int(48));
        assert_eq!(call_builtin("char->integer", vec![char(' ')]).unwrap(), int(32));
        
        // Special characters
        assert_eq!(call_builtin("char->integer", vec![char('\n')]).unwrap(), int(10));
        assert_eq!(call_builtin("char->integer", vec![char('\t')]).unwrap(), int(9));
        
        // Arity errors
        assert!(call_builtin("char->integer", vec![]).is_err());
        assert!(call_builtin("char->integer", vec![char('a'), char('b')]).is_err());
        
        // Type errors
        assert!(call_builtin("char->integer", vec![int(65)]).is_err());
        assert!(call_builtin("char->integer", vec![string("a")]).is_err());
    }

    #[test]
    fn test_integer_to_char() {
        // Basic ASCII conversion
        assert_eq!(call_builtin("integer->char", vec![int(65)]).unwrap(), char('A'));
        assert_eq!(call_builtin("integer->char", vec![int(97)]).unwrap(), char('a'));
        assert_eq!(call_builtin("integer->char", vec![int(48)]).unwrap(), char('0'));
        assert_eq!(call_builtin("integer->char", vec![int(32)]).unwrap(), char(' '));
        
        // Special characters
        assert_eq!(call_builtin("integer->char", vec![int(10)]).unwrap(), char('\n'));
        assert_eq!(call_builtin("integer->char", vec![int(9)]).unwrap(), char('\t'));
        
        // Edge cases
        assert_eq!(call_builtin("integer->char", vec![int(0)]).unwrap(), char('\0'));
        assert_eq!(call_builtin("integer->char", vec![int(127)]).unwrap(), char('\x7F'));
        
        // Out of ASCII range
        assert!(call_builtin("integer->char", vec![int(128)]).is_err());
        assert!(call_builtin("integer->char", vec![int(-1)]).is_err());
        assert!(call_builtin("integer->char", vec![int(256)]).is_err());
        
        // Arity errors
        assert!(call_builtin("integer->char", vec![]).is_err());
        assert!(call_builtin("integer->char", vec![int(65), int(66)]).is_err());
        
        // Type errors
        assert!(call_builtin("integer->char", vec![real(65.0)]).is_err());
        assert!(call_builtin("integer->char", vec![string("65")]).is_err());
    }
}

#[cfg(test)]
mod conversion_functions_tests {
    use super::*;

    #[test]
    fn test_char_to_string() {
        // Basic character to string conversion
        assert_eq!(call_builtin("char->string", vec![char('a')]).unwrap(), string("a"));
        assert_eq!(call_builtin("char->string", vec![char('Z')]).unwrap(), string("Z"));
        assert_eq!(call_builtin("char->string", vec![char('5')]).unwrap(), string("5"));
        assert_eq!(call_builtin("char->string", vec![char(' ')]).unwrap(), string(" "));
        
        // Special characters
        assert_eq!(call_builtin("char->string", vec![char('\n')]).unwrap(), string("\n"));
        assert_eq!(call_builtin("char->string", vec![char('\t')]).unwrap(), string("\t"));
        
        // Unicode characters
        assert_eq!(call_builtin("char->string", vec![char('あ')]).unwrap(), string("あ"));
        assert_eq!(call_builtin("char->string", vec![char('🎉')]).unwrap(), string("🎉"));
        
        // Arity errors
        assert!(call_builtin("char->string", vec![]).is_err());
        assert!(call_builtin("char->string", vec![char('a'), char('b')]).is_err());
        
        // Type errors
        assert!(call_builtin("char->string", vec![string("a")]).is_err());
        assert!(call_builtin("char->string", vec![int(65)]).is_err());
    }

    #[test]
    fn test_string_to_list() {
        // Basic string to list conversion
        assert_eq!(call_builtin("string->list", vec![string("hello")]).unwrap(), 
                   list(vec![char('h'), char('e'), char('l'), char('l'), char('o')]));
        assert_eq!(call_builtin("string->list", vec![string("a")]).unwrap(), 
                   list(vec![char('a')]));
        
        // Empty string
        assert_eq!(call_builtin("string->list", vec![string("")]).unwrap(), 
                   list(vec![]));
        
        // String with special characters
        assert_eq!(call_builtin("string->list", vec![string("a\nb")]).unwrap(), 
                   list(vec![char('a'), char('\n'), char('b')]));
        
        // Unicode string
        assert_eq!(call_builtin("string->list", vec![string("こん")]).unwrap(), 
                   list(vec![char('こ'), char('ん')]));
        
        // Arity errors
        assert!(call_builtin("string->list", vec![]).is_err());
        assert!(call_builtin("string->list", vec![string("a"), string("b")]).is_err());
        
        // Type errors
        assert!(call_builtin("string->list", vec![int(42)]).is_err());
        assert!(call_builtin("string->list", vec![char('a')]).is_err());
    }

    #[test]
    fn test_list_to_string() {
        // Basic list to string conversion
        assert_eq!(call_builtin("list->string", vec![list(vec![char('h'), char('e'), char('l'), char('l'), char('o')])]).unwrap(), 
                   string("hello"));
        assert_eq!(call_builtin("list->string", vec![list(vec![char('a')])]).unwrap(), 
                   string("a"));
        
        // Empty list
        assert_eq!(call_builtin("list->string", vec![list(vec![])]).unwrap(), 
                   string(""));
        
        // List with special characters
        assert_eq!(call_builtin("list->string", vec![list(vec![char('a'), char('\n'), char('b')])]).unwrap(), 
                   string("a\nb"));
        
        // Unicode characters
        assert_eq!(call_builtin("list->string", vec![list(vec![char('こ'), char('ん')])]).unwrap(), 
                   string("こん"));
        
        // List with non-character elements
        assert!(call_builtin("list->string", vec![list(vec![char('a'), int(42)])]).is_err());
        assert!(call_builtin("list->string", vec![list(vec![string("hello")])]).is_err());
        
        // Arity errors
        assert!(call_builtin("list->string", vec![]).is_err());
        assert!(call_builtin("list->string", vec![list(vec![char('a')]), list(vec![char('b')])]).is_err());
        
        // Type errors
        assert!(call_builtin("list->string", vec![string("hello")]).is_err());
        assert!(call_builtin("list->string", vec![char('a')]).is_err());
    }

    #[test]
    fn test_number_to_string() {
        // Integer conversion
        assert_eq!(call_builtin("number->string", vec![int(42)]).unwrap(), string("42"));
        assert_eq!(call_builtin("number->string", vec![int(-17)]).unwrap(), string("-17"));
        assert_eq!(call_builtin("number->string", vec![int(0)]).unwrap(), string("0"));
        
        // Real number conversion
        assert_eq!(call_builtin("number->string", vec![real(std::f64::consts::PI)]).unwrap(), string(&std::f64::consts::PI.to_string()));
        assert_eq!(call_builtin("number->string", vec![real(-2.5)]).unwrap(), string("-2.5"));
        assert_eq!(call_builtin("number->string", vec![real(0.0)]).unwrap(), string("0"));
        
        // Large numbers
        assert_eq!(call_builtin("number->string", vec![int(1000000)]).unwrap(), string("1000000"));
        
        // Arity errors
        assert!(call_builtin("number->string", vec![]).is_err());
        assert!(call_builtin("number->string", vec![int(42), int(43)]).is_err());
        
        // Type errors
        assert!(call_builtin("number->string", vec![string("42")]).is_err());
        assert!(call_builtin("number->string", vec![char('4')]).is_err());
    }

    #[test]
    fn test_string_to_number() {
        // Integer conversion
        assert_eq!(call_builtin("string->number", vec![string("42")]).unwrap(), int(42));
        assert_eq!(call_builtin("string->number", vec![string("-17")]).unwrap(), int(-17));
        assert_eq!(call_builtin("string->number", vec![string("0")]).unwrap(), int(0));
        
        // Real number conversion
        assert_eq!(call_builtin("string->number", vec![string(&std::f64::consts::PI.to_string())]).unwrap(), real(std::f64::consts::PI));
        assert_eq!(call_builtin("string->number", vec![string("-2.5")]).unwrap(), real(-2.5));
        assert_eq!(call_builtin("string->number", vec![string("0.0")]).unwrap(), real(0.0));
        
        // Scientific notation
        assert_eq!(call_builtin("string->number", vec![string("1e3")]).unwrap(), real(1000.0));
        assert_eq!(call_builtin("string->number", vec![string("-2.5e-2")]).unwrap(), real(-0.025));
        
        // Invalid number strings (return #f)
        assert_eq!(call_builtin("string->number", vec![string("hello")]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("string->number", vec![string("12.34.56")]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("string->number", vec![string("")]).unwrap(), Value::Boolean(false));
        assert_eq!(call_builtin("string->number", vec![string("abc123")]).unwrap(), Value::Boolean(false));
        
        // Arity errors
        assert!(call_builtin("string->number", vec![]).is_err());
        assert!(call_builtin("string->number", vec![string("42"), string("43")]).is_err());
        
        // Type errors
        assert!(call_builtin("string->number", vec![int(42)]).is_err());
        assert!(call_builtin("string->number", vec![char('4')]).is_err());
    }

    #[test]
    fn test_symbol_to_string() {
        // Basic symbol to string conversion
        assert_eq!(call_builtin("symbol->string", vec![symbol("hello")]).unwrap(), string("hello"));
        assert_eq!(call_builtin("symbol->string", vec![symbol("foo-bar")]).unwrap(), string("foo-bar"));
        assert_eq!(call_builtin("symbol->string", vec![symbol("+")]).unwrap(), string("+"));
        assert_eq!(call_builtin("symbol->string", vec![symbol("123")]).unwrap(), string("123"));
        
        // Single character symbols
        assert_eq!(call_builtin("symbol->string", vec![symbol("x")]).unwrap(), string("x"));
        
        // Symbols with special characters
        assert_eq!(call_builtin("symbol->string", vec![symbol("var-name")]).unwrap(), string("var-name"));
        assert_eq!(call_builtin("symbol->string", vec![symbol("test?")]).unwrap(), string("test?"));
        
        // Arity errors
        assert!(call_builtin("symbol->string", vec![]).is_err());
        assert!(call_builtin("symbol->string", vec![symbol("a"), symbol("b")]).is_err());
        
        // Type errors
        assert!(call_builtin("symbol->string", vec![string("hello")]).is_err());
        assert!(call_builtin("symbol->string", vec![int(42)]).is_err());
    }

    #[test]
    fn test_string_to_symbol() {
        // Basic string to symbol conversion
        assert_eq!(call_builtin("string->symbol", vec![string("hello")]).unwrap(), symbol("hello"));
        assert_eq!(call_builtin("string->symbol", vec![string("foo-bar")]).unwrap(), symbol("foo-bar"));
        assert_eq!(call_builtin("string->symbol", vec![string("+")]).unwrap(), symbol("+"));
        assert_eq!(call_builtin("string->symbol", vec![string("123")]).unwrap(), symbol("123"));
        
        // Single character strings
        assert_eq!(call_builtin("string->symbol", vec![string("x")]).unwrap(), symbol("x"));
        
        // Strings with special characters
        assert_eq!(call_builtin("string->symbol", vec![string("var-name")]).unwrap(), symbol("var-name"));
        assert_eq!(call_builtin("string->symbol", vec![string("test?")]).unwrap(), symbol("test?"));
        
        // Empty string (edge case)
        assert_eq!(call_builtin("string->symbol", vec![string("")]).unwrap(), symbol(""));
        
        // Arity errors
        assert!(call_builtin("string->symbol", vec![]).is_err());
        assert!(call_builtin("string->symbol", vec![string("a"), string("b")]).is_err());
        
        // Type errors
        assert!(call_builtin("string->symbol", vec![symbol("hello")]).is_err());
        assert!(call_builtin("string->symbol", vec![int(42)]).is_err());
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_empty_string_operations() {
        // Empty string operations
        assert_eq!(call_builtin("string-length", vec![string("")]).unwrap(), int(0));
        assert_eq!(call_builtin("string-append", vec![string(""), string(""), string("")]).unwrap(), string(""));
        assert_eq!(call_builtin("substring", vec![string(""), int(0), int(0)]).unwrap(), string(""));
        assert_eq!(call_builtin("string->list", vec![string("")]).unwrap(), list(vec![]));
        assert_eq!(call_builtin("list->string", vec![list(vec![])]).unwrap(), string(""));
    }

    #[test]
    fn test_unicode_support() {
        // Unicode string operations
        let unicode_str = "こんにちは世界🌍";
        assert_eq!(call_builtin("string-length", vec![string(unicode_str)]).unwrap(), int(8));
        
        // Unicode character operations
        assert_eq!(call_builtin("string-ref", vec![string(unicode_str), int(0)]).unwrap(), char('こ'));
        assert_eq!(call_builtin("string-ref", vec![string(unicode_str), int(7)]).unwrap(), char('🌍'));
        
        // Unicode substring
        assert_eq!(call_builtin("substring", vec![string(unicode_str), int(2), int(5)]).unwrap(), string("にちは"));
    }

    #[test]
    fn test_type_errors() {
        // Test various type errors to ensure robust error handling
        let non_string = int(42);
        let non_char = string("hello");
        let non_number = string("not-a-number");
        
        // String functions with non-string inputs
        assert!(call_builtin("string-length", vec![non_string.clone()]).is_err());
        assert!(call_builtin("string-ref", vec![non_string.clone(), int(0)]).is_err());
        assert!(call_builtin("string-append", vec![string("hello"), non_string.clone()]).is_err());
        
        // Character functions with non-character inputs
        assert!(call_builtin("char=?", vec![non_char.clone(), char('a')]).is_err());
        assert!(call_builtin("char->integer", vec![non_char.clone()]).is_err());
        assert!(call_builtin("char->string", vec![non_char.clone()]).is_err());
        
        // Number conversion with non-numbers
        assert!(call_builtin("number->string", vec![non_number.clone()]).is_err());
        assert!(call_builtin("integer->char", vec![non_number.clone()]).is_err());
    }

    #[test]
    fn test_boundary_conditions() {
        // String indexing at boundaries
        let test_str = string("test");
        assert_eq!(call_builtin("string-ref", vec![test_str.clone(), int(0)]).unwrap(), char('t'));
        assert_eq!(call_builtin("string-ref", vec![test_str.clone(), int(3)]).unwrap(), char('t'));
        assert!(call_builtin("string-ref", vec![test_str.clone(), int(4)]).is_err());
        assert!(call_builtin("string-ref", vec![test_str.clone(), int(-1)]).is_err());
        
        // Substring at boundaries
        assert_eq!(call_builtin("substring", vec![test_str.clone(), int(0), int(4)]).unwrap(), string("test"));
        assert_eq!(call_builtin("substring", vec![test_str.clone(), int(4), int(4)]).unwrap(), string(""));
        assert!(call_builtin("substring", vec![test_str.clone(), int(0), int(5)]).is_err());
        
        // Character to integer boundaries
        assert_eq!(call_builtin("integer->char", vec![int(0)]).unwrap(), char('\0'));
        assert_eq!(call_builtin("integer->char", vec![int(127)]).unwrap(), char('\x7F'));
        assert!(call_builtin("integer->char", vec![int(128)]).is_err());
        assert!(call_builtin("integer->char", vec![int(-1)]).is_err());
    }
}
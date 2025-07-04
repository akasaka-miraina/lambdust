//! Unit tests for SRFI 13: String Libraries implementation

// Individual functions are no longer public - use interpreter integration
use lambdust::interpreter::LambdustInterpreter;
use lambdust::lexer::SchemeNumber;
use lambdust::value::Value;

#[test]
fn test_string_null() {
    let mut interpreter = LambdustInterpreter::new();

    let result = interpreter.eval_string("(string-null? \"\")").unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = interpreter.eval_string("(string-null? \"hello\")").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_string_hash() {
    let mut interpreter = LambdustInterpreter::new();
    
    let result = interpreter.eval_string("(string-hash \"hello\")").unwrap();
    assert!(matches!(result, Value::Number(_)));

    let result = interpreter.eval_string("(string-hash \"hello\" 1000)").unwrap();
    assert!(matches!(result, Value::Number(_)));
}

#[test]
fn test_string_prefix() {
    let mut interpreter = LambdustInterpreter::new();

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
fn test_string_suffix() {
    let mut interpreter = LambdustInterpreter::new();

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
fn test_string_contains() {
    let mut interpreter = LambdustInterpreter::new();
    
    let result = interpreter.eval_string("(string-contains \"hello\" \"ell\")").unwrap();
    assert_eq!(result, Value::Number(SchemeNumber::Integer(1)));

    let result = interpreter.eval_string("(string-contains \"hello\" \"world\")").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_string_take() {
    let mut interpreter = LambdustInterpreter::new();
    
    let result = interpreter.eval_string("(string-take \"hello\" 3)").unwrap();
    assert_eq!(result, Value::String("hel".to_string()));
}

#[test]
fn test_string_drop() {
    let mut interpreter = LambdustInterpreter::new();
    
    let result = interpreter.eval_string("(string-drop \"hello\" 2)").unwrap();
    assert_eq!(result, Value::String("llo".to_string()));
}

#[test]
fn test_string_take_right() {
    let mut interpreter = LambdustInterpreter::new();
    
    let result = interpreter.eval_string("(string-take-right \"hello\" 3)").unwrap();
    assert_eq!(result, Value::String("llo".to_string()));
}

#[test]
fn test_string_drop_right() {
    let mut interpreter = LambdustInterpreter::new();
    
    let result = interpreter.eval_string("(string-drop-right \"hello\" 2)").unwrap();
    assert_eq!(result, Value::String("hel".to_string()));
}

#[test]
fn test_string_concatenate() {
    let mut interpreter = LambdustInterpreter::new();
    
    let result = interpreter.eval_string("(string-concatenate '(\"hello\" \" \" \"world\"))").unwrap();
    assert_eq!(result, Value::String("hello world".to_string()));
}

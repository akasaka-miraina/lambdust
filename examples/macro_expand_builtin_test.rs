//! Test macro-expand functions as actual built-ins
//! Demonstrates the REPL debugging capabilities

use lambdust::builtins::create_builtins;
use lambdust::value::{Value, PairData};
use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    println!("=== Built-in Macro-Expand Functions Test ===\n");
    
    test_macro_expand_1_builtin();
    test_macro_expand_builtin();
    test_macro_expand_all_builtin();
    test_with_complex_forms();
}

fn test_macro_expand_1_builtin() {
    println!("1. macro-expand-1 Built-in Test:");
    
    let builtins = create_builtins();
    
    if let Some(macro_expand_1) = builtins.get("macro-expand-1") {
        // Create a test form: '(when #t (display "hello"))
        let test_form = create_when_form();
        
        // Call the built-in function
        match call_builtin_function(macro_expand_1, &[test_form]) {
            Ok(result) => {
                println!("  Input:  (when #t (display \"hello\"))");
                print_expansion_result(&result);
                println!("  ✓ macro-expand-1 built-in works");
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
            }
        }
    } else {
        println!("  ✗ macro-expand-1 not found in builtins");
    }
    
    println!();
}

fn test_macro_expand_builtin() {
    println!("2. macro-expand Built-in Test:");
    
    let builtins = create_builtins();
    
    if let Some(macro_expand) = builtins.get("macro-expand") {
        // Create a test form: '(+ 1 2) (not a macro)
        let test_form = create_add_form();
        
        match call_builtin_function(macro_expand, &[test_form]) {
            Ok(result) => {
                println!("  Input:  (+ 1 2)");
                print_expansion_result(&result);
                println!("  ✓ macro-expand built-in works");
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
            }
        }
    } else {
        println!("  ✗ macro-expand not found in builtins");
    }
    
    println!();
}

fn test_macro_expand_all_builtin() {
    println!("3. macro-expand-all Built-in Test:");
    
    let builtins = create_builtins();
    
    if let Some(macro_expand_all) = builtins.get("macro-expand-all") {
        let test_form = create_vector_form();
        
        match call_builtin_function(macro_expand_all, &[test_form]) {
            Ok(result) => {
                println!("  Input:  #(1 2 3)");
                println!("  Result: {}", result);
                println!("  ✓ macro-expand-all built-in works");
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
            }
        }
    } else {
        println!("  ✗ macro-expand-all not found in builtins");
    }
    
    println!();
}

fn test_with_complex_forms() {
    println!("4. Complex Form Test:");
    
    let builtins = create_builtins();
    
    if let Some(macro_expand_1) = builtins.get("macro-expand-1") {
        // Test with nested list structure
        let complex_form = create_nested_form();
        
        match call_builtin_function(macro_expand_1, &[complex_form]) {
            Ok(result) => {
                println!("  Input:  (let ((x 1)) (+ x 2))");
                print_expansion_result(&result);
                println!("  ✓ Complex form handling works");
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
            }
        }
    }
    
    println!();
}

/// Create a 'when' macro form for testing
fn create_when_form() -> Value {
    // (when #t (display "hello"))
    create_list_value(vec![
        Value::from("when"),
        Value::Boolean(true),
        create_list_value(vec![
            Value::from("display"),
            Value::from("hello"),
        ]),
    ])
}

/// Create an arithmetic form
fn create_add_form() -> Value {
    // (+ 1 2)
    create_list_value(vec![
        Value::from("+"),
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
    ])
}

/// Create a vector form
fn create_vector_form() -> Value {
    // #(1 2 3)
    Value::Vector(vec![
        Value::Number(SchemeNumber::Integer(1)),
        Value::Number(SchemeNumber::Integer(2)),
        Value::Number(SchemeNumber::Integer(3)),
    ])
}

/// Create a nested form
fn create_nested_form() -> Value {
    // (let ((x 1)) (+ x 2))
    create_list_value(vec![
        Value::from("let"),
        create_list_value(vec![
            create_list_value(vec![
                Value::from("x"),
                Value::Number(SchemeNumber::Integer(1)),
            ]),
        ]),
        create_list_value(vec![
            Value::from("+"),
            Value::from("x"),
            Value::Number(SchemeNumber::Integer(2)),
        ]),
    ])
}

/// Create a proper list Value from a vector of Values
fn create_list_value(values: Vec<Value>) -> Value {
    if values.is_empty() {
        Value::Nil
    } else {
        let mut result = Value::Nil;
        for value in values.into_iter().rev() {
            result = Value::Pair(Rc::new(RefCell::new(PairData {
                car: value,
                cdr: result,
            })));
        }
        result
    }
}

/// Call a built-in function (simplified version)
fn call_builtin_function(func: &Value, args: &[Value]) -> Result<Value, String> {
    match func {
        Value::Procedure(proc) => {
            match proc {
                lambdust::value::Procedure::Builtin { func, .. } => {
                    func(args).map_err(|e| e.to_string())
                }
                _ => Err("Not a built-in function".to_string()),
            }
        }
        _ => Err("Not a procedure".to_string()),
    }
}

/// Print expansion result in a readable format
fn print_expansion_result(result: &Value) {
    if let Value::Pair(pair_ref) = result {
        let pair = pair_ref.borrow();
        println!("  Result: {}", pair.car);
        match &pair.cdr {
            Value::Boolean(expanded) => {
                println!("  Expanded: {}", if *expanded { "yes" } else { "no" });
            }
            _ => {
                println!("  Flag: {}", pair.cdr);
            }
        }
    } else {
        println!("  Result: {}", result);
    }
}
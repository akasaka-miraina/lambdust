//! Macro expansion functions demonstration
//! Shows the power of macro-expand-1 and macro-expand for debugging

use lambdust::builtins::macro_expansion::MacroExpander;
use lambdust::macros::hygiene::HygienicEnvironment;
use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;
use std::rc::Rc;

fn main() {
    println!("=== Macro Expansion Functions Demo ===\n");
    
    test_basic_expansion();
    test_non_macro_form();
    test_complete_expansion();
    test_error_handling();
}

fn test_basic_expansion() {
    println!("1. Basic Macro Expansion Test:");
    
    // Create environment and expander
    let env = Rc::new(HygienicEnvironment::new());
    let expander = MacroExpander::new(env);
    
    // Test expanding a "when" macro call
    let when_expr = Expr::List(vec![
        Expr::Variable("when".to_string()),
        Expr::Literal(Literal::Boolean(true)),
        Expr::List(vec![
            Expr::Variable("display".to_string()),
            Expr::Literal(Literal::String("Hello, World!".to_string())),
        ]),
    ]);
    
    match expander.expand_once(&when_expr) {
        Ok(result) => {
            println!("  Original:  {}", when_expr);
            println!("  Expanded:  {}", result.form);
            println!("  Changed:   {}", result.expanded);
            if let Some(info) = &result.expansion_info {
                println!("  Macro:     {} ({})", info.macro_name, info.macro_type);
                println!("  Time:      {:?}", info.expansion_time);
            }
            println!("  ✓ Basic expansion works");
        }
        Err(e) => {
            println!("  ✗ Expansion failed: {}", e);
        }
    }
    
    println!();
}

fn test_non_macro_form() {
    println!("2. Non-Macro Form Test:");
    
    let env = Rc::new(HygienicEnvironment::new());
    let expander = MacroExpander::new(env);
    
    // Test with a regular function call (not a macro)
    let add_expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
    ]);
    
    match expander.expand_once(&add_expr) {
        Ok(result) => {
            println!("  Original:  {}", add_expr);
            println!("  Result:    {}", result.form);
            println!("  Changed:   {}", result.expanded);
            println!("  ✓ Non-macro forms return unchanged (as expected)");
        }
        Err(e) => {
            println!("  ✗ Unexpected error: {}", e);
        }
    }
    
    println!();
}

fn test_complete_expansion() {
    println!("3. Complete Expansion Test:");
    
    let env = Rc::new(HygienicEnvironment::new());
    let expander = MacroExpander::new(env);
    
    // Test nested macro expansion (conceptual)
    let nested_expr = Expr::List(vec![
        Expr::Variable("outer-macro".to_string()),
        Expr::List(vec![
            Expr::Variable("inner-macro".to_string()),
            Expr::Literal(Literal::String("nested".to_string())),
        ]),
    ]);
    
    match expander.expand_completely(&nested_expr) {
        Ok(result) => {
            println!("  Original:  {}", nested_expr);
            println!("  Expanded:  {}", result.form);
            println!("  Changed:   {}", result.expanded);
            if let Some(info) = &result.expansion_info {
                println!("  Depth:     {}", info.depth);
                println!("  Type:      {}", info.macro_type);
            }
            println!("  ✓ Complete expansion test");
        }
        Err(e) => {
            println!("  ✗ Expansion failed: {}", e);
        }
    }
    
    println!();
}

fn test_error_handling() {
    println!("4. Error Handling Test:");
    
    let env = Rc::new(HygienicEnvironment::new());
    let expander = MacroExpander::new(env);
    
    // Test with malformed expression
    let malformed_expr = Expr::Variable("standalone-symbol".to_string());
    
    match expander.expand_once(&malformed_expr) {
        Ok(result) => {
            println!("  Input:     {}", malformed_expr);
            println!("  Result:    {}", result.form);
            println!("  Changed:   {}", result.expanded);
            println!("  ✓ Non-list forms handled gracefully");
        }
        Err(e) => {
            println!("  ✗ Unexpected error: {}", e);
        }
    }
    
    println!();
}

#[allow(dead_code)]
fn demo_built_in_functions() {
    println!("5. Built-in Functions Demo (conceptual):");
    println!("  In a REPL, you would use:");
    println!("  > (macro-expand-1 '(when #t (display \"hello\")))");
    println!("  ((if#1 #t (begin#2 (display \"hello\"))) . #t)");
    println!();
    println!("  > (macro-expand-1 '(+ 1 2))");
    println!("  ((+ 1 2) . #f)");
    println!();
    println!("  > (macro-expand '(when (> x 0) (when #t (display x))))");
    println!("  ((if#1 (> x 0) (begin#2 (if#3 #t (begin#4 (display x))))) . #t)");
    println!();
}
//! Unit tests for macro system

use lambdust::macros::MacroExpander;
use lambdust::parser::parse;
use lambdust::environment::Environment;
use std::sync::Arc;

#[test]
fn test_macro_expansion() {
    let env = Arc::new(Environment::new());
    let mut expander = MacroExpander::new();
    
    // Test basic macro expansion
    let expr = parse("(when #t 42)").unwrap();
    let result = expander.expand(&expr, &env);
    
    // Should expand to (if #t 42 (void))
    assert!(result.is_ok());
}
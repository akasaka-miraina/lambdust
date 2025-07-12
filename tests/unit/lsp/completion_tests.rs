//! LSP Code Completion Tests
//!
//! Comprehensive tests for code completion functionality including:
//! - Built-in function completion accuracy
//! - Context-aware completion 
//! - Performance under various loads
//! - Special form and macro completion
//! - User-defined symbol completion

use super::lsp_test_utils::*;
use crate::lsp::completion::{
    CompletionProvider, CompletionContext, CompletionItem, CompletionItemKind,
    ExpressionContext,
};
use crate::lsp::position::{Position, Range};

#[test]
fn test_completion_provider_creation() {
    let provider = CompletionProvider::new();
    assert!(provider.is_ok());
}

#[test]
fn test_builtin_function_completions() {
    let provider = CompletionProvider::new().unwrap();
    
    // Test arithmetic completions
    let arithmetic_completions = provider.get_function_completions("+");
    assert!(!arithmetic_completions.is_empty());
    assert!(arithmetic_completions.iter().any(|item| item.label == "+"));
    
    // Test list function completions
    let list_completions = provider.get_function_completions("c");
    assert!(list_completions.iter().any(|item| item.label == "cons"));
    assert!(list_completions.iter().any(|item| item.label == "car"));
    assert!(list_completions.iter().any(|item| item.label == "cdr"));
    
    // Test string function completions
    let string_completions = provider.get_function_completions("string-");
    assert!(string_completions.iter().any(|item| item.label == "string-length"));
    assert!(string_completions.iter().any(|item| item.label == "string-append"));
    assert!(string_completions.iter().any(|item| item.label == "string-ref"));
}

#[test]
fn test_special_form_completions() {
    let provider = CompletionProvider::new().unwrap();
    
    // Test definition forms
    let def_completions = provider.get_special_form_completions("def");
    assert!(def_completions.iter().any(|item| item.label == "define"));
    
    // Test conditional forms
    let cond_completions = provider.get_special_form_completions("c");
    assert!(cond_completions.iter().any(|item| item.label == "cond"));
    assert!(cond_completions.iter().any(|item| item.label == "case"));
    
    // Test lambda
    let lambda_completions = provider.get_special_form_completions("l");
    assert!(lambda_completions.iter().any(|item| item.label == "lambda"));
    assert!(lambda_completions.iter().any(|item| item.label == "let"));
    assert!(lambda_completions.iter().any(|item| item.label == "let*"));
    assert!(lambda_completions.iter().any(|item| item.label == "letrec"));
}

#[test]
fn test_completion_item_kinds() {
    let provider = CompletionProvider::new().unwrap();
    
    let function_completions = provider.get_function_completions("cons");
    let function_item = function_completions.iter().find(|item| item.label == "cons").unwrap();
    assert_eq!(function_item.kind, CompletionItemKind::Function);
    
    let special_completions = provider.get_special_form_completions("define");
    let special_item = special_completions.iter().find(|item| item.label == "define").unwrap();
    assert_eq!(special_item.kind, CompletionItemKind::Keyword);
}

#[test]
fn test_completion_context_creation() {
    let context = CompletionContext {
        position: Position::new(0, 5),
        trigger_character: Some('('),
        is_retrigger: false,
        line_content: "(cons ".to_string(),
        prefix: "(cons".to_string(),
        expression_context: ExpressionContext::FunctionPosition,
        scope_bindings: vec!["x".to_string(), "y".to_string()],
    };
    
    assert_eq!(context.position.line, 0);
    assert_eq!(context.position.character, 5);
    assert_eq!(context.trigger_character, Some('('));
    assert!(!context.is_retrigger);
    assert_eq!(context.expression_context, ExpressionContext::FunctionPosition);
}

#[test]
fn test_expression_context_detection() {
    // Test different expression contexts
    
    // Function position
    let context_func = CompletionContext {
        position: Position::new(0, 2),
        trigger_character: Some('('),
        is_retrigger: false,
        line_content: "(c".to_string(),
        prefix: "(c".to_string(),
        expression_context: ExpressionContext::FunctionPosition,
        scope_bindings: vec![],
    };
    
    let provider = CompletionProvider::new().unwrap();
    let completions = provider.get_completions(&context_func).unwrap();
    assert!(!completions.is_empty());
    
    // Should include both functions and special forms
    assert!(completions.iter().any(|item| item.kind == CompletionItemKind::Function));
    assert!(completions.iter().any(|item| item.kind == CompletionItemKind::Keyword));
}

#[test]
fn test_argument_completion() {
    let provider = CompletionProvider::new().unwrap();
    
    // Test completion in argument position
    let context = CompletionContext {
        position: Position::new(0, 8),
        trigger_character: Some(' '),
        is_retrigger: false,
        line_content: "(define ".to_string(),
        prefix: "".to_string(),
        expression_context: ExpressionContext::ArgumentPosition {
            function_name: Some("define".to_string()),
            arg_index: 0,
        },
        scope_bindings: vec!["existing-var".to_string()],
    };
    
    let completions = provider.get_completions(&context).unwrap();
    // Should provide appropriate completions for define's first argument
    assert!(!completions.is_empty());
}

#[test]
fn test_snippet_completions() {
    let provider = CompletionProvider::new().unwrap();
    
    let snippets = provider.get_snippet_completions("def");
    assert!(!snippets.is_empty());
    
    let defun_snippet = snippets.iter().find(|item| item.label == "defun");
    assert!(defun_snippet.is_some());
    
    let snippet = defun_snippet.unwrap();
    assert_eq!(snippet.kind, CompletionItemKind::Snippet);
    assert!(snippet.insert_text.is_some());
    assert!(snippet.insert_text.as_ref().unwrap().contains("${"));
}

#[test]
fn test_completion_priority() {
    let provider = CompletionProvider::new().unwrap();
    
    let context = CompletionContext {
        position: Position::new(0, 2),
        trigger_character: Some('('),
        is_retrigger: false,
        line_content: "(d".to_string(),
        prefix: "d".to_string(),
        expression_context: ExpressionContext::FunctionPosition,
        scope_bindings: vec![],
    };
    
    let completions = provider.get_completions(&context).unwrap();
    
    // Special forms should have higher priority than functions
    let define_item = completions.iter().find(|item| item.label == "define");
    let display_item = completions.iter().find(|item| item.label == "display");
    
    if let (Some(define), Some(display)) = (define_item, display_item) {
        assert!(define.sort_priority <= display.sort_priority);
    }
}

#[test]
fn test_completion_filtering() {
    let provider = CompletionProvider::new().unwrap();
    
    // Test prefix filtering
    let plus_completions = provider.get_function_completions("+");
    assert!(plus_completions.iter().all(|item| item.label.starts_with("+")));
    
    let string_completions = provider.get_function_completions("string-a");
    assert!(string_completions.iter().all(|item| item.label.starts_with("string-a")));
    
    // Empty prefix should return all completions (up to limit)
    let all_completions = provider.get_function_completions("");
    assert!(!all_completions.is_empty());
}

#[test]
fn test_completion_documentation() {
    let provider = CompletionProvider::new().unwrap();
    
    let completions = provider.get_function_completions("cons");
    let cons_item = completions.iter().find(|item| item.label == "cons").unwrap();
    
    assert!(cons_item.detail.is_some());
    assert!(cons_item.documentation.is_some());
    
    let detail = cons_item.detail.as_ref().unwrap();
    assert!(detail.contains("cons"));
    assert!(detail.contains("obj1"));
    assert!(detail.contains("obj2"));
    
    let doc = cons_item.documentation.as_ref().unwrap();
    assert!(doc.contains("pair"));
}

#[test]
fn test_completion_performance() {
    let provider = CompletionProvider::new().unwrap();
    
    let context = CompletionContext {
        position: Position::new(0, 1),
        trigger_character: Some('('),
        is_retrigger: false,
        line_content: "(".to_string(),
        prefix: "".to_string(),
        expression_context: ExpressionContext::ExpressionStart,
        scope_bindings: vec![],
    };
    
    // Measure completion time
    let start = std::time::Instant::now();
    let completions = provider.get_completions(&context).unwrap();
    let duration = start.elapsed();
    
    // Should complete quickly (under 10ms for basic completions)
    assert!(duration < std::time::Duration::from_millis(10));
    assert!(!completions.is_empty());
    
    // Verify completion limit is respected
    assert!(completions.len() <= 50);
}

#[test]
fn test_variable_completions() {
    let provider = CompletionProvider::new().unwrap();
    
    let scope_bindings = vec![
        "my-variable".to_string(),
        "another-var".to_string(),
        "x".to_string(),
        "y".to_string(),
    ];
    
    let var_completions = provider.get_variable_completions("my", &scope_bindings);
    assert!(!var_completions.is_empty());
    assert!(var_completions.iter().any(|item| item.label == "my-variable"));
    assert_eq!(var_completions[0].kind, CompletionItemKind::Variable);
}

#[test]
fn test_completion_context_sensitivity() {
    let provider = CompletionProvider::new().unwrap();
    
    // Test top-level context (should include definitions and snippets)
    let top_level_context = CompletionContext {
        position: Position::new(0, 0),
        trigger_character: None,
        is_retrigger: false,
        line_content: "".to_string(),
        prefix: "".to_string(),
        expression_context: ExpressionContext::TopLevel,
        scope_bindings: vec![],
    };
    
    let top_level_completions = provider.get_completions(&top_level_context).unwrap();
    assert!(top_level_completions.iter().any(|item| item.kind == CompletionItemKind::Snippet));
    
    // Test string literal context (should provide file completions for load)
    let string_context = CompletionContext {
        position: Position::new(0, 7),
        trigger_character: Some('"'),
        is_retrigger: false,
        line_content: r#"(load ""#.to_string(),
        prefix: "".to_string(),
        expression_context: ExpressionContext::StringLiteral,
        scope_bindings: vec![],
    };
    
    let string_completions = provider.get_completions(&string_context).unwrap();
    // File completions would be empty in test environment, but shouldn't error
    assert!(string_completions.is_empty() || string_completions.iter().any(|item| item.kind == CompletionItemKind::File));
}

#[test]
fn test_completion_retrigger() {
    let provider = CompletionProvider::new().unwrap();
    
    let context = CompletionContext {
        position: Position::new(0, 5),
        trigger_character: None,
        is_retrigger: true,
        line_content: "(str".to_string(),
        prefix: "str".to_string(),
        expression_context: ExpressionContext::FunctionPosition,
        scope_bindings: vec![],
    };
    
    let completions = provider.get_completions(&context).unwrap();
    
    // Should include string functions
    assert!(completions.iter().any(|item| item.label.starts_with("string-")));
}

#[test]
fn test_completion_edge_cases() {
    let provider = CompletionProvider::new().unwrap();
    
    // Empty context
    let empty_context = CompletionContext {
        position: Position::new(0, 0),
        trigger_character: None,
        is_retrigger: false,
        line_content: "".to_string(),
        prefix: "".to_string(),
        expression_context: ExpressionContext::TopLevel,
        scope_bindings: vec![],
    };
    
    let completions = provider.get_completions(&empty_context);
    assert!(completions.is_ok());
    
    // Very long prefix
    let long_prefix = "a".repeat(1000);
    let long_context = CompletionContext {
        position: Position::new(0, 1000),
        trigger_character: None,
        is_retrigger: false,
        line_content: long_prefix.clone(),
        prefix: long_prefix,
        expression_context: ExpressionContext::VariableReference,
        scope_bindings: vec![],
    };
    
    let completions = provider.get_completions(&long_context);
    assert!(completions.is_ok());
    assert!(completions.unwrap().is_empty()); // No matches expected
}

#[test]
fn test_completion_special_characters() {
    let provider = CompletionProvider::new().unwrap();
    
    // Test completions with special characters
    let special_completions = provider.get_function_completions("<=");
    assert!(special_completions.iter().any(|item| item.label == "<="));
    
    let question_completions = provider.get_function_completions("null?");
    assert!(question_completions.iter().any(|item| item.label == "null?"));
    
    // Test prefix with symbols
    let symbol_completions = provider.get_function_completions("string?");
    assert!(symbol_completions.iter().any(|item| item.label == "string?"));
}

#[test]
fn test_completion_metadata() {
    let provider = CompletionProvider::new().unwrap();
    
    let completions = provider.get_function_completions("apply");
    let apply_item = completions.iter().find(|item| item.label == "apply").unwrap();
    
    // Verify all required fields are present
    assert!(!apply_item.label.is_empty());
    assert_eq!(apply_item.kind, CompletionItemKind::Function);
    assert!(apply_item.detail.is_some());
    assert!(apply_item.documentation.is_some());
    assert!(apply_item.insert_text.is_some());
    assert_eq!(apply_item.insert_text.as_ref().unwrap(), "apply");
    assert!(!apply_item.preselect);
    assert!(apply_item.additional_text_edits.is_empty());
}

#[test]
fn test_documented_symbols() {
    let provider = CompletionProvider::new().unwrap();
    
    let documented_symbols = provider.get_documented_symbols();
    
    // Should include major built-in functions
    assert!(documented_symbols.contains(&"+".to_string()));
    assert!(documented_symbols.contains(&"cons".to_string()));
    assert!(documented_symbols.contains(&"define".to_string()));
    assert!(documented_symbols.contains(&"lambda".to_string()));
    
    // Should be sorted
    for i in 1..documented_symbols.len() {
        assert!(documented_symbols[i-1] <= documented_symbols[i]);
    }
}
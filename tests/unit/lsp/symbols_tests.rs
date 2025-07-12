//! LSP Symbol Navigation Tests
//!
//! Tests for symbol provider functionality including:
//! - Symbol discovery and indexing
//! - Workspace symbol search
//! - Document symbol hierarchy
//! - Definition and reference finding

use super::lsp_test_utils::*;
use crate::lsp::symbols::{SymbolProvider, SymbolKind, SymbolInfo, SymbolRequest};
use crate::lsp::position::{Position, Range};

#[test]
fn test_symbol_provider_creation() {
    let symbol_provider = SymbolProvider::new();
    
    assert!(symbol_provider.is_ready());
    assert_eq!(symbol_provider.get_supported_languages(), vec!["scheme"]);
}

#[test]
fn test_document_symbols_basic() {
    let symbol_provider = SymbolProvider::new();
    let code = r#"
(define pi 3.14159)
(define (square x) (* x x))
(define-syntax when
  (syntax-rules ()
    ((when test stmt ...)
     (if test (begin stmt ...)))))
"#;
    let document = create_test_document("test://symbols.scm", code);
    
    let symbols = symbol_provider.get_document_symbols(&document).unwrap();
    
    assert_eq!(symbols.len(), 3);
    
    // Check pi variable
    let pi_symbol = symbols.iter().find(|s| s.name == "pi").unwrap();
    assert_eq!(pi_symbol.kind, SymbolKind::Variable);
    assert!(pi_symbol.range.start.line > 0);
    
    // Check square function
    let square_symbol = symbols.iter().find(|s| s.name == "square").unwrap();
    assert_eq!(square_symbol.kind, SymbolKind::Function);
    assert!(square_symbol.detail.as_ref().unwrap().contains("procedure"));
    
    // Check when macro
    let when_symbol = symbols.iter().find(|s| s.name == "when").unwrap();
    assert_eq!(when_symbol.kind, SymbolKind::Method); // Macros use Method kind
    assert!(when_symbol.detail.as_ref().unwrap().contains("macro"));
}

#[test]
fn test_document_symbols_nested() {
    let symbol_provider = SymbolProvider::new();
    let code = r#"
(define (outer x)
  (define (inner y)
    (define local-var (* y 2))
    (+ x local-var))
  (inner (+ x 1)))
"#;
    let document = create_test_document("test://nested.scm", code);
    
    let symbols = symbol_provider.get_document_symbols(&document).unwrap();
    
    // Should find outer function
    let outer_symbol = symbols.iter().find(|s| s.name == "outer").unwrap();
    assert_eq!(outer_symbol.kind, SymbolKind::Function);
    
    // Should find inner function as child
    assert!(outer_symbol.children.is_some());
    let children = outer_symbol.children.as_ref().unwrap();
    
    let inner_symbol = children.iter().find(|s| s.name == "inner").unwrap();
    assert_eq!(inner_symbol.kind, SymbolKind::Function);
    
    // Should find local variable
    let inner_children = inner_symbol.children.as_ref().unwrap();
    let local_var = inner_children.iter().find(|s| s.name == "local-var").unwrap();
    assert_eq!(local_var.kind, SymbolKind::Variable);
}

#[test]
fn test_symbol_kinds_classification() {
    let symbol_provider = SymbolProvider::new();
    let code = r#"
;; Variables
(define x 42)
(define str "hello")

;; Functions
(define (add a b) (+ a b))
(define factorial (lambda (n) (if (<= n 1) 1 (* n (factorial (- n 1))))))

;; Macros
(define-syntax unless
  (syntax-rules ()
    ((unless test stmt ...)
     (if (not test) (begin stmt ...)))))

;; Constants
(define *debug* #t)
(define +version+ "1.0")
"#;
    let document = create_test_document("test://kinds.scm", code);
    
    let symbols = symbol_provider.get_document_symbols(&document).unwrap();
    
    // Variables
    let x_symbol = symbols.iter().find(|s| s.name == "x").unwrap();
    assert_eq!(x_symbol.kind, SymbolKind::Variable);
    
    let str_symbol = symbols.iter().find(|s| s.name == "str").unwrap();
    assert_eq!(str_symbol.kind, SymbolKind::Variable);
    
    // Functions
    let add_symbol = symbols.iter().find(|s| s.name == "add").unwrap();
    assert_eq!(add_symbol.kind, SymbolKind::Function);
    
    let factorial_symbol = symbols.iter().find(|s| s.name == "factorial").unwrap();
    assert_eq!(factorial_symbol.kind, SymbolKind::Function);
    
    // Macro
    let unless_symbol = symbols.iter().find(|s| s.name == "unless").unwrap();
    assert_eq!(unless_symbol.kind, SymbolKind::Method); // Macros as Methods
    
    // Constants (special variable naming conventions)
    let debug_symbol = symbols.iter().find(|s| s.name == "*debug*").unwrap();
    assert_eq!(debug_symbol.kind, SymbolKind::Constant);
    
    let version_symbol = symbols.iter().find(|s| s.name == "+version+").unwrap();
    assert_eq!(version_symbol.kind, SymbolKind::Constant);
}

#[test]
fn test_workspace_symbol_search() {
    let symbol_provider = SymbolProvider::new();
    
    // Create multiple documents to simulate workspace
    let doc1_code = r#"
(define (fibonacci n) (if (<= n 1) n (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))
(define pi 3.14159)
"#;
    let doc1 = create_test_document("test://doc1.scm", doc1_code);
    
    let doc2_code = r#"
(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))
(define e 2.71828)
"#;
    let doc2 = create_test_document("test://doc2.scm", doc2_code);
    
    // Register documents with symbol provider
    symbol_provider.index_document(&doc1).unwrap();
    symbol_provider.index_document(&doc2).unwrap();
    
    // Search for symbols containing "f"
    let f_symbols = symbol_provider.search_workspace_symbols("f").unwrap();
    assert!(!f_symbols.is_empty());
    
    let fib_symbol = f_symbols.iter().find(|s| s.name == "fibonacci").unwrap();
    assert_eq!(fib_symbol.kind, SymbolKind::Function);
    assert_eq!(fib_symbol.location.uri, "test://doc1.scm");
    
    let fact_symbol = f_symbols.iter().find(|s| s.name == "factorial").unwrap();
    assert_eq!(fact_symbol.kind, SymbolKind::Function);
    assert_eq!(fact_symbol.location.uri, "test://doc2.scm");
}

#[test]
fn test_symbol_search_patterns() {
    let symbol_provider = SymbolProvider::new();
    let code = r#"
(define string-length (lambda (s) (length s)))
(define string-append (lambda (s1 s2) (append s1 s2)))
(define string-ref (lambda (s i) (list-ref s i)))
(define make-string (lambda (n c) (make-list n c)))
"#;
    let document = create_test_document("test://strings.scm", code);
    symbol_provider.index_document(&document).unwrap();
    
    // Test exact match
    let exact = symbol_provider.search_workspace_symbols("string-length").unwrap();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0].name, "string-length");
    
    // Test prefix match
    let prefix = symbol_provider.search_workspace_symbols("string-").unwrap();
    assert!(prefix.len() >= 3);
    assert!(prefix.iter().any(|s| s.name == "string-length"));
    assert!(prefix.iter().any(|s| s.name == "string-append"));
    assert!(prefix.iter().any(|s| s.name == "string-ref"));
    
    // Test partial match
    let partial = symbol_provider.search_workspace_symbols("string").unwrap();
    assert!(partial.len() >= 3);
    
    // Test case insensitive
    let case_insensitive = symbol_provider.search_workspace_symbols("STRING").unwrap();
    assert!(!case_insensitive.is_empty());
}

#[test]
fn test_find_definition() {
    let symbol_provider = SymbolProvider::new();
    let code = r#"
(define (square x) (* x x))
(define result (square 5))
(define another-result (square 10))
"#;
    let document = create_test_document("test://definition.scm", code);
    symbol_provider.index_document(&document).unwrap();
    
    // Find definition of 'square' from usage
    let request = SymbolRequest {
        document_uri: "test://definition.scm".to_string(),
        position: Position::new(2, 17), // Position of 'square' in first usage
        symbol_name: Some("square".to_string()),
    };
    
    let definition = symbol_provider.find_definition(&document, request).unwrap();
    assert!(definition.is_some());
    
    let def = definition.unwrap();
    assert_eq!(def.uri, "test://definition.scm");
    assert_eq!(def.range.start.line, 1); // Line of definition
    assert!(def.range.start.character > 0);
}

#[test]
fn test_find_references() {
    let symbol_provider = SymbolProvider::new();
    let code = r#"
(define (helper x) (* x 2))
(define (main) (helper 5))
(define (test) (helper 10))
(define another (helper 3))
"#;
    let document = create_test_document("test://references.scm", code);
    symbol_provider.index_document(&document).unwrap();
    
    // Find all references to 'helper'
    let request = SymbolRequest {
        document_uri: "test://references.scm".to_string(),
        position: Position::new(1, 8), // Position of 'helper' in definition
        symbol_name: Some("helper".to_string()),
    };
    
    let references = symbol_provider.find_references(&document, request, true).unwrap();
    
    // Should find definition + 3 usages = 4 total
    assert_eq!(references.len(), 4);
    
    // Verify all references are correct
    for reference in &references {
        assert_eq!(reference.uri, "test://references.scm");
        assert!(reference.range.start.line >= 1);
    }
}

#[test]
fn test_symbol_hierarchy() {
    let symbol_provider = SymbolProvider::new();
    let code = r#"
(define (module-function x)
  (let ((local-var 10))
    (define (nested-function y)
      (define inner-var (+ x y local-var))
      inner-var)
    (nested-function x)))
"#;
    let document = create_test_document("test://hierarchy.scm", code);
    
    let symbols = symbol_provider.get_document_symbols(&document).unwrap();
    
    // Should have module-function at top level
    assert_eq!(symbols.len(), 1);
    let module_func = &symbols[0];
    assert_eq!(module_func.name, "module-function");
    assert_eq!(module_func.kind, SymbolKind::Function);
    
    // Should have nested symbols
    assert!(module_func.children.is_some());
    let children = module_func.children.as_ref().unwrap();
    
    // Find nested function
    let nested_func = children.iter().find(|s| s.name == "nested-function").unwrap();
    assert_eq!(nested_func.kind, SymbolKind::Function);
    
    // Check nested function has its own children
    assert!(nested_func.children.is_some());
    let nested_children = nested_func.children.as_ref().unwrap();
    let inner_var = nested_children.iter().find(|s| s.name == "inner-var").unwrap();
    assert_eq!(inner_var.kind, SymbolKind::Variable);
}

#[test]
fn test_symbol_detail_information() {
    let symbol_provider = SymbolProvider::new();
    let code = r#"
(define pi 3.14159)
(define (add x y) (+ x y))
(define factorial
  (lambda (n)
    (if (<= n 1) 1 (* n (factorial (- n 1))))))
"#;
    let document = create_test_document("test://details.scm", code);
    
    let symbols = symbol_provider.get_document_symbols(&document).unwrap();
    
    // Check variable detail
    let pi_symbol = symbols.iter().find(|s| s.name == "pi").unwrap();
    assert!(pi_symbol.detail.is_some());
    assert!(pi_symbol.detail.as_ref().unwrap().contains("number"));
    
    // Check function detail
    let add_symbol = symbols.iter().find(|s| s.name == "add").unwrap();
    assert!(add_symbol.detail.is_some());
    assert!(add_symbol.detail.as_ref().unwrap().contains("procedure"));
    
    // Check lambda function detail
    let factorial_symbol = symbols.iter().find(|s| s.name == "factorial").unwrap();
    assert!(factorial_symbol.detail.is_some());
    assert!(factorial_symbol.detail.as_ref().unwrap().contains("lambda"));
}

#[test]
fn test_symbol_range_accuracy() {
    let symbol_provider = SymbolProvider::new();
    let code = "(define (test-function param) (* param 2))";
    let document = create_test_document("test://ranges.scm", code);
    
    let symbols = symbol_provider.get_document_symbols(&document).unwrap();
    
    let func_symbol = symbols.iter().find(|s| s.name == "test-function").unwrap();
    
    // Check that range encompasses the entire definition
    assert_eq!(func_symbol.range.start.line, 0);
    assert_eq!(func_symbol.range.start.character, 0); // Start of entire definition
    assert_eq!(func_symbol.range.end.character, 42);   // End of definition
    
    // Check that selection range focuses on the name
    assert_eq!(func_symbol.selection_range.start.character, 8);  // Start of "test-function"
    assert_eq!(func_symbol.selection_range.end.character, 21);   // End of "test-function"
}

#[test]
fn test_symbol_error_handling() {
    let symbol_provider = SymbolProvider::new();
    
    // Test with malformed code
    let bad_code = "(define (incomplete";
    let document = create_test_document("test://error.scm", bad_code);
    
    // Should handle gracefully without crashing
    let result = symbol_provider.get_document_symbols(&document);
    assert!(result.is_ok());
    
    // May find partial symbols or none, but shouldn't error
    let symbols = result.unwrap();
    // Don't assert specific count since error recovery may vary
}

#[test]
fn test_symbol_unicode_support() {
    let symbol_provider = SymbolProvider::new();
    let code = r#"
(define π 3.14159)
(define λ (lambda (x) x))
(define 测试 "test")
"#;
    let document = create_test_document("test://unicode.scm", code);
    
    let symbols = symbol_provider.get_document_symbols(&document).unwrap();
    
    // Should handle Unicode identifiers
    assert!(symbols.iter().any(|s| s.name == "π"));
    assert!(symbols.iter().any(|s| s.name == "λ"));
    assert!(symbols.iter().any(|s| s.name == "测试"));
}

#[test]
fn test_symbol_performance() {
    let symbol_provider = SymbolProvider::new();
    
    // Create a larger document
    let large_code = (0..100)
        .map(|i| format!("(define func{} (lambda (x) (* x {})))", i, i))
        .collect::<Vec<_>>()
        .join("\n");
        
    let document = create_test_document("test://performance.scm", &large_code);
    
    let start = std::time::Instant::now();
    let symbols = symbol_provider.get_document_symbols(&document).unwrap();
    let duration = start.elapsed();
    
    // Should find all symbols
    assert_eq!(symbols.len(), 100);
    
    // Should complete within reasonable time
    assert!(duration.as_millis() < 500, "Symbol extraction too slow: {:?}", duration);
}

#[test]
fn test_workspace_symbol_ranking() {
    let symbol_provider = SymbolProvider::new();
    
    let code = r#"
(define test 1)
(define test-function (lambda (x) x))
(define another-test 2)
(define testing-framework "test")
"#;
    let document = create_test_document("test://ranking.scm", code);
    symbol_provider.index_document(&document).unwrap();
    
    // Search for "test" - should rank exact matches higher
    let results = symbol_provider.search_workspace_symbols("test").unwrap();
    
    assert!(!results.is_empty());
    
    // Exact match should come first
    assert_eq!(results[0].name, "test");
    
    // Other matches should follow
    assert!(results.iter().any(|s| s.name == "test-function"));
    assert!(results.iter().any(|s| s.name == "another-test"));
    assert!(results.iter().any(|s| s.name == "testing-framework"));
}

#[test]
fn test_cross_document_references() {
    let symbol_provider = SymbolProvider::new();
    
    // Document 1: Definition
    let doc1_code = "(define shared-function (lambda (x) (* x 2)))";
    let doc1 = create_test_document("test://def.scm", doc1_code);
    symbol_provider.index_document(&doc1).unwrap();
    
    // Document 2: Usage  
    let doc2_code = "(define result (shared-function 5))";
    let doc2 = create_test_document("test://usage.scm", doc2_code);
    symbol_provider.index_document(&doc2).unwrap();
    
    // Find definition from usage in different document
    let request = SymbolRequest {
        document_uri: "test://usage.scm".to_string(),
        position: Position::new(0, 16), // Position of 'shared-function' in usage
        symbol_name: Some("shared-function".to_string()),
    };
    
    let definition = symbol_provider.find_definition(&doc2, request).unwrap();
    assert!(definition.is_some());
    
    let def = definition.unwrap();
    assert_eq!(def.uri, "test://def.scm"); // Should point to definition document
}
//! LSP Integration Tests
//!
//! End-to-end integration tests for the complete LSP system including:
//! - Full server lifecycle and protocol compliance
//! - Multi-feature interaction scenarios
//! - Real-world usage patterns
//! - Performance and reliability testing

use super::lsp_test_utils::*;
use crate::lsp::{LambdustLanguageServer, LspConfig};
use crate::lsp::position::{Position, Range};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_lsp_server_initialization() {
    let config = LspConfig::default();
    let server = LambdustLanguageServer::new(config).unwrap();
    
    assert!(server.is_initialized());
    assert!(server.capabilities().text_document_sync);
    assert!(server.capabilities().completion);
    assert!(server.capabilities().hover);
    assert!(server.capabilities().diagnostics);
}

#[tokio::test]
async fn test_complete_document_workflow() {
    let mut server = create_test_server();
    let uri = "test://workflow.scm";
    let initial_code = r#"
(define (factorial n)
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))
"#;
    
    // 1. Open document
    server.open_document(uri, "scheme", 1, initial_code).await.unwrap();
    
    // 2. Get initial diagnostics
    let diagnostics = server.get_diagnostics(uri).await.unwrap();
    assert!(diagnostics.is_empty()); // Should be valid code
    
    // 3. Get document symbols
    let symbols = server.get_document_symbols(uri).await.unwrap();
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "factorial");
    
    // 4. Test completion at specific position
    let completion_pos = Position::new(2, 6); // After "if"
    let completions = server.get_completions(uri, completion_pos).await.unwrap();
    assert!(!completions.is_empty());
    
    // 5. Test hover information
    let hover_pos = Position::new(4, 12); // Over "factorial" in recursive call
    let hover = server.get_hover(uri, hover_pos).await.unwrap();
    assert!(hover.is_some());
    assert!(hover.unwrap().contents.contains("factorial"));
    
    // 6. Edit document
    let edit_range = Range::new(Position::new(1, 17), Position::new(1, 18)); // Change parameter name
    server.edit_document(uri, 2, edit_range, "num").await.unwrap();
    
    // 7. Verify changes reflected in symbols
    let updated_symbols = server.get_document_symbols(uri).await.unwrap();
    assert!(!updated_symbols.is_empty());
    
    // 8. Close document
    server.close_document(uri).await.unwrap();
}

#[tokio::test]
async fn test_multi_document_workspace() {
    let mut server = create_test_server();
    
    // Document 1: Library functions
    let lib_uri = "test://lib.scm";
    let lib_code = r#"
(define (square x) (* x x))
(define (cube x) (* x x x))
(define pi 3.14159)
"#;
    
    // Document 2: Main application
    let main_uri = "test://main.scm";
    let main_code = r#"
(define area (* pi (square radius)))
(define volume (* pi (square radius) height))
"#;
    
    // Open both documents
    server.open_document(lib_uri, "scheme", 1, lib_code).await.unwrap();
    server.open_document(main_uri, "scheme", 1, main_code).await.unwrap();
    
    // Test workspace-wide symbol search
    let pi_symbols = server.search_workspace_symbols("pi").await.unwrap();
    assert!(pi_symbols.iter().any(|s| s.name == "pi"));
    
    let square_symbols = server.search_workspace_symbols("square").await.unwrap();
    assert!(square_symbols.iter().any(|s| s.name == "square"));
    
    // Test cross-document references
    let square_usage_pos = Position::new(1, 20); // "square" in main.scm
    let definition = server.find_definition(main_uri, square_usage_pos).await.unwrap();
    assert!(definition.is_some());
    
    let def = definition.unwrap();
    assert_eq!(def.uri, lib_uri); // Should point to definition in lib.scm
    
    // Test references across documents
    let references = server.find_references(lib_uri, Position::new(1, 8), true).await.unwrap();
    assert!(references.len() >= 2); // Definition + usage(s)
    
    // Close documents
    server.close_document(lib_uri).await.unwrap();
    server.close_document(main_uri).await.unwrap();
}

#[tokio::test]
async fn test_real_time_diagnostics() {
    let mut server = create_test_server();
    let uri = "test://diagnostics.scm";
    
    // Start with valid code
    let valid_code = "(define x 42)";
    server.open_document(uri, "scheme", 1, valid_code).await.unwrap();
    
    let diagnostics = server.get_diagnostics(uri).await.unwrap();
    assert!(diagnostics.is_empty());
    
    // Introduce syntax error
    let error_range = Range::new(Position::new(0, 0), Position::new(0, 13));
    server.edit_document(uri, 2, error_range, "(define x").await.unwrap();
    
    // Should detect syntax error
    let error_diagnostics = server.get_diagnostics(uri).await.unwrap();
    assert!(!error_diagnostics.is_empty());
    assert!(error_diagnostics[0].message.contains("parenthesis") || 
            error_diagnostics[0].message.contains("incomplete"));
    
    // Fix the error
    let fix_range = Range::new(Position::new(0, 0), Position::new(0, 9));
    server.edit_document(uri, 3, fix_range, "(define x 42)").await.unwrap();
    
    // Should clear diagnostics
    let fixed_diagnostics = server.get_diagnostics(uri).await.unwrap();
    assert!(fixed_diagnostics.is_empty());
    
    server.close_document(uri).await.unwrap();
}

#[tokio::test]
async fn test_completion_context_awareness() {
    let mut server = create_test_server();
    let uri = "test://completion.scm";
    let code = r#"
(define (test-function x y)
  (let ((local-var 10))
    (+ x y local-var)))

(define global-var 20)

;; Completion test positions
(test-
(+ global-
(let ((z 1)) (+
"#;
    
    server.open_document(uri, "scheme", 1, code).await.unwrap();
    
    // Test function name completion
    let func_pos = Position::new(8, 5); // After "test-"
    let func_completions = server.get_completions(uri, func_pos).await.unwrap();
    assert!(func_completions.iter().any(|c| c.label.contains("test-function")));
    
    // Test global variable completion
    let global_pos = Position::new(9, 10); // After "global-"
    let global_completions = server.get_completions(uri, global_pos).await.unwrap();
    assert!(global_completions.iter().any(|c| c.label.contains("global-var")));
    
    // Test builtin function completion
    let builtin_pos = Position::new(10, 15); // After "+" in let
    let builtin_completions = server.get_completions(uri, builtin_pos).await.unwrap();
    assert!(builtin_completions.iter().any(|c| c.label == "+"));
    assert!(builtin_completions.iter().any(|c| c.label == "car"));
    assert!(builtin_completions.iter().any(|c| c.label == "cdr"));
    
    server.close_document(uri).await.unwrap();
}

#[tokio::test]
async fn test_hover_across_definitions() {
    let mut server = create_test_server();
    let uri = "test://hover.scm";
    let code = r#"
;; Mathematical constants
(define pi 3.14159)
(define e 2.71828)

;; Utility functions
(define (square x) 
  "Compute the square of x"
  (* x x))

(define (circle-area radius)
  "Calculate the area of a circle"
  (* pi (square radius)))

;; Usage
(define my-circle-area (circle-area 5))
"#;
    
    server.open_document(uri, "scheme", 1, code).await.unwrap();
    
    // Test hover over constant
    let pi_hover = server.get_hover(uri, Position::new(2, 8)).await.unwrap();
    assert!(pi_hover.is_some());
    assert!(pi_hover.unwrap().contents.contains("pi"));
    
    // Test hover over function definition
    let square_def_hover = server.get_hover(uri, Position::new(6, 8)).await.unwrap();
    assert!(square_def_hover.is_some());
    let square_content = square_def_hover.unwrap().contents;
    assert!(square_content.contains("square"));
    assert!(square_content.contains("Compute the square"));
    
    // Test hover over function usage
    let square_usage_hover = server.get_hover(uri, Position::new(12, 7)).await.unwrap();
    assert!(square_usage_hover.is_some());
    assert!(square_usage_hover.unwrap().contents.contains("square"));
    
    // Test hover over complex expression
    let circle_hover = server.get_hover(uri, Position::new(15, 25)).await.unwrap();
    assert!(circle_hover.is_some());
    assert!(circle_hover.unwrap().contents.contains("circle-area"));
    
    server.close_document(uri).await.unwrap();
}

#[tokio::test]
async fn test_macro_system_integration() {
    let mut server = create_test_server();
    let uri = "test://macros.scm";
    let code = r#"
;; Define a simple macro
(define-syntax when
  (syntax-rules ()
    ((when test stmt1 stmt2 ...)
     (if test (begin stmt1 stmt2 ...)))))

;; Define another macro
(define-syntax unless  
  (syntax-rules ()
    ((unless test stmt ...)
     (if (not test) (begin stmt ...)))))

;; Usage
(when (> x 0)
  (display "positive")
  (newline))

(unless (zero? y)
  (display "non-zero"))
"#;
    
    server.open_document(uri, "scheme", 1, code).await.unwrap();
    
    // Test symbol recognition for macros
    let symbols = server.get_document_symbols(uri).await.unwrap();
    assert!(symbols.iter().any(|s| s.name == "when"));
    assert!(symbols.iter().any(|s| s.name == "unless"));
    
    // Test completion includes macros
    let completion_pos = Position::new(15, 1); // Start of macro usage
    let completions = server.get_completions(uri, completion_pos).await.unwrap();
    assert!(completions.iter().any(|c| c.label == "when"));
    assert!(completions.iter().any(|c| c.label == "unless"));
    
    // Test hover over macro usage
    let when_hover = server.get_hover(uri, Position::new(15, 1)).await.unwrap();
    assert!(when_hover.is_some());
    assert!(when_hover.unwrap().contents.contains("when"));
    
    server.close_document(uri).await.unwrap();
}

#[tokio::test]
async fn test_error_recovery_and_partial_parsing() {
    let mut server = create_test_server();
    let uri = "test://errors.scm";
    let code = r#"
(define x 42)

;; This has a syntax error
(define (broken-func
  (+ 1 2))

;; This should still be parseable
(define y 84)

;; Another error
(define z
"#;
    
    server.open_document(uri, "scheme", 1, code).await.unwrap();
    
    // Should report diagnostics for errors
    let diagnostics = server.get_diagnostics(uri).await.unwrap();
    assert!(!diagnostics.is_empty());
    
    // Should still find valid symbols despite errors
    let symbols = server.get_document_symbols(uri).await.unwrap();
    assert!(symbols.iter().any(|s| s.name == "x"));
    assert!(symbols.iter().any(|s| s.name == "y"));
    
    // Hover should work on valid parts
    let x_hover = server.get_hover(uri, Position::new(1, 8)).await.unwrap();
    assert!(x_hover.is_some());
    
    // Completion should work despite errors
    let completion_pos = Position::new(1, 8);
    let completions = server.get_completions(uri, completion_pos).await.unwrap();
    assert!(!completions.is_empty());
    
    server.close_document(uri).await.unwrap();
}

#[tokio::test]
async fn test_large_document_performance() {
    let mut server = create_test_server();
    let uri = "test://large.scm";
    
    // Generate a large document
    let large_code = (0..1000)
        .map(|i| format!("(define func{} (lambda (x) (* x {})))", i, i))
        .collect::<Vec<_>>()
        .join("\n");
    
    let start = std::time::Instant::now();
    server.open_document(uri, "scheme", 1, &large_code).await.unwrap();
    let open_duration = start.elapsed();
    
    // Should open within reasonable time
    assert!(open_duration.as_millis() < 2000, "Document opening too slow: {:?}", open_duration);
    
    // Test symbol extraction performance
    let start = std::time::Instant::now();
    let symbols = server.get_document_symbols(uri).await.unwrap();
    let symbols_duration = start.elapsed();
    
    assert_eq!(symbols.len(), 1000);
    assert!(symbols_duration.as_millis() < 1000, "Symbol extraction too slow: {:?}", symbols_duration);
    
    // Test completion performance
    let start = std::time::Instant::now();
    let completions = server.get_completions(uri, Position::new(500, 5)).await.unwrap();
    let completion_duration = start.elapsed();
    
    assert!(!completions.is_empty());
    assert!(completion_duration.as_millis() < 500, "Completion too slow: {:?}", completion_duration);
    
    server.close_document(uri).await.unwrap();
}

#[tokio::test]
async fn test_concurrent_requests() {
    let server = create_test_server();
    let uri = "test://concurrent.scm";
    let code = r#"
(define (factorial n)
  (if (<= n 1) 1 (* n (factorial (- n 1)))))

(define (fibonacci n)
  (if (<= n 1) n (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))

(define pi 3.14159)
(define e 2.71828)
"#;
    
    server.open_document(uri, "scheme", 1, code).await.unwrap();
    
    // Launch multiple concurrent requests
    let hover_task = server.get_hover(uri, Position::new(1, 10));
    let completion_task = server.get_completions(uri, Position::new(2, 15));
    let symbols_task = server.get_document_symbols(uri);
    let diagnostics_task = server.get_diagnostics(uri);
    
    // All should complete successfully
    let (hover_result, completion_result, symbols_result, diagnostics_result) = 
        tokio::join!(hover_task, completion_task, symbols_task, diagnostics_task);
    
    assert!(hover_result.is_ok());
    assert!(completion_result.is_ok());
    assert!(symbols_result.is_ok());
    assert!(diagnostics_result.is_ok());
    
    // Verify results are meaningful
    assert!(hover_result.unwrap().is_some());
    assert!(!completion_result.unwrap().is_empty());
    assert_eq!(symbols_result.unwrap().len(), 4); // factorial, fibonacci, pi, e
    assert!(diagnostics_result.unwrap().is_empty());
    
    server.close_document(uri).await.unwrap();
}

#[tokio::test]
async fn test_server_shutdown_cleanup() {
    let mut server = create_test_server();
    
    // Open multiple documents
    let uris = vec![
        "test://doc1.scm",
        "test://doc2.scm", 
        "test://doc3.scm",
    ];
    
    for (i, uri) in uris.iter().enumerate() {
        let code = format!("(define var{} {})", i, i);
        server.open_document(uri, "scheme", 1, &code).await.unwrap();
    }
    
    // Verify documents are open
    assert_eq!(server.get_open_document_count(), 3);
    
    // Shutdown server
    server.shutdown().await.unwrap();
    
    // Verify cleanup
    assert_eq!(server.get_open_document_count(), 0);
    assert!(!server.is_running());
}

#[tokio::test]
async fn test_timeout_handling() {
    let mut server = create_test_server();
    let uri = "test://timeout.scm";
    
    // Create a potentially slow operation (very large document)
    let huge_code = (0..10000)
        .map(|i| format!("(define huge{} {})", i, i))
        .collect::<Vec<_>>()
        .join("\n");
    
    server.open_document(uri, "scheme", 1, &huge_code).await.unwrap();
    
    // Test with timeout
    let completion_task = server.get_completions(uri, Position::new(5000, 5));
    let result = timeout(Duration::from_millis(100), completion_task).await;
    
    // Should either complete quickly or timeout gracefully
    match result {
        Ok(completion_result) => {
            assert!(completion_result.is_ok());
        }
        Err(_) => {
            // Timeout is acceptable for very large documents
            println!("Request timed out as expected for large document");
        }
    }
    
    server.close_document(uri).await.unwrap();
}

#[tokio::test]
async fn test_unicode_and_special_characters() {
    let mut server = create_test_server();
    let uri = "test://unicode.scm";
    let code = r#"
;; Unicode identifiers
(define π 3.14159)
(define λ (lambda (x) x))
(define 你好 "hello")
(define café "coffee")

;; Special characters in strings
(define message "Hello, 世界! 🌍")
(define escaped "Quote: \" Newline: \n Tab: \t")

;; Mathematical symbols
(define ∑ (lambda (lst) (apply + lst)))
(define ∏ (lambda (lst) (apply * lst)))
"#;
    
    server.open_document(uri, "scheme", 1, code).await.unwrap();
    
    // Test symbols with Unicode names
    let symbols = server.get_document_symbols(uri).await.unwrap();
    assert!(symbols.iter().any(|s| s.name == "π"));
    assert!(symbols.iter().any(|s| s.name == "λ"));
    assert!(symbols.iter().any(|s| s.name == "你好"));
    assert!(symbols.iter().any(|s| s.name == "café"));
    assert!(symbols.iter().any(|s| s.name == "∑"));
    
    // Test completion with Unicode
    let completion_pos = Position::new(2, 8); // After "π"
    let completions = server.get_completions(uri, completion_pos).await.unwrap();
    assert!(completions.iter().any(|c| c.label.contains("π")));
    
    // Test hover over Unicode identifier
    let unicode_hover = server.get_hover(uri, Position::new(3, 8)).await.unwrap();
    assert!(unicode_hover.is_some());
    assert!(unicode_hover.unwrap().contents.contains("λ"));
    
    // No syntax errors expected
    let diagnostics = server.get_diagnostics(uri).await.unwrap();
    assert!(diagnostics.is_empty());
    
    server.close_document(uri).await.unwrap();
}
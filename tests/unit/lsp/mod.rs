//! LSP (Language Server Protocol) Unit Tests
//!
//! Comprehensive test suite for Lambdust LSP implementation covering:
//! - Server lifecycle and protocol compliance
//! - Code completion accuracy and performance  
//! - Diagnostics engine correctness
//! - Hover information quality
//! - Symbol navigation functionality
//! - Document management reliability
//! - Position tracking precision

pub mod server_tests;
pub mod completion_tests;
pub mod diagnostics_tests;
pub mod hover_tests;
pub mod symbols_tests;
pub mod document_tests;
pub mod position_tests;
pub mod integration_tests;

#[cfg(test)]
mod lsp_test_utils {
    use crate::lsp::{LspConfig, LambdustLanguageServer};
    use crate::lsp::document::Document;
    use crate::lsp::position::{Position, Range};
    use crate::interpreter::LambdustInterpreter;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    
    /// Create test LSP server
    pub fn create_test_server() -> LambdustLanguageServer {
        let config = LspConfig {
            debug_mode: true,
            enable_verification: false, // Disable for tests
            enable_performance_analysis: false,
            max_diagnostics: 50,
            completion_triggers: vec!["(".to_string(), " ".to_string()],
            workspace_root: None,
            enable_repl_integration: false,
        };
        
        LambdustLanguageServer::new(config).expect("Failed to create test server")
    }
    
    /// Create test document
    pub fn create_test_document(uri: &str, content: &str) -> Document {
        Document::new(
            uri.to_string(),
            "scheme".to_string(),
            1,
            content.to_string(),
        ).expect("Failed to create test document")
    }
    
    /// Create sample Scheme code for testing
    pub fn sample_scheme_code() -> &'static str {
        r#"
;; Sample Scheme code for LSP testing
(define (factorial n)
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

(define pi 3.14159)

(define (square x) (* x x))

(define-syntax when
  (syntax-rules ()
    ((when test stmt1 stmt2 ...)
     (if test (begin stmt1 stmt2 ...)))))

(let ((x 10)
      (y 20))
  (+ x y))
"#
    }
    
    /// Create position for testing
    pub fn pos(line: u32, character: u32) -> Position {
        Position::new(line, character)
    }
    
    /// Create range for testing
    pub fn range(start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> Range {
        Range::new(
            Position::new(start_line, start_char),
            Position::new(end_line, end_char),
        )
    }
    
    /// Helper to create LSP types for testing
    pub mod lsp_helpers {
        use lsp_types::{Position, Range, TextDocumentIdentifier, TextDocumentItem, Url};
        
        pub fn create_text_document_item(uri: &str, content: &str) -> TextDocumentItem {
            TextDocumentItem {
                uri: Url::parse(uri).expect("Invalid URI"),
                language_id: "scheme".to_string(),
                version: 1,
                text: content.to_string(),
            }
        }
        
        pub fn create_text_document_identifier(uri: &str) -> TextDocumentIdentifier {
            TextDocumentIdentifier {
                uri: Url::parse(uri).expect("Invalid URI"),
            }
        }
        
        pub fn lsp_pos(line: u32, character: u32) -> Position {
            Position::new(line, character)
        }
        
        pub fn lsp_range(start_line: u32, start_char: u32, end_line: u32, end_char: u32) -> Range {
            Range::new(
                Position::new(start_line, start_char),
                Position::new(end_line, end_char),
            )
        }
    }
    
    /// Test data generators
    pub mod generators {
        /// Generate various Scheme expressions for testing
        pub fn scheme_expressions() -> Vec<(&'static str, &'static str)> {
            vec![
                ("(+ 1 2 3)", "arithmetic expression"),
                ("(define x 42)", "variable definition"),
                ("(define (f x) (* x x))", "function definition"),
                ("(lambda (x) (* x 2))", "lambda expression"),
                ("(if (> x 0) x (- x))", "conditional expression"),
                ("(let ((a 1) (b 2)) (+ a b))", "let binding"),
                ("(cons 1 (cons 2 '()))", "list construction"),
                ("'(a b c)", "quoted list"),
                ("`(a ,b c)", "quasiquoted expression"),
                ("(syntax-rules () ((m x) x))", "macro definition"),
            ]
        }
        
        /// Generate error-prone Scheme expressions
        pub fn error_expressions() -> Vec<(&'static str, &'static str)> {
            vec![
                ("(+ 1", "unclosed parenthesis"),
                ("+ 1 2)", "unmatched closing parenthesis"),
                ("(define)", "incomplete define"),
                ("(lambda)", "incomplete lambda"),
                ("(if x)", "incomplete if"),
                ("\"unclosed string", "unterminated string"),
                ("(undefined-function x)", "undefined function"),
                ("(+ \"string\" 1)", "type error"),
            ]
        }
        
        /// Generate completion test scenarios
        pub fn completion_scenarios() -> Vec<(&'static str, u32, u32, &'static str)> {
            vec![
                ("(c", 0, 2, "cons, car, cdr completion"),
                ("(+ 1 (l", 0, 7, "lambda, let, length completion"),
                ("(define ", 0, 8, "identifier completion"),
                ("(string-", 0, 8, "string function completion"),
                ("(defin", 0, 6, "define completion"),
            ]
        }
    }
}
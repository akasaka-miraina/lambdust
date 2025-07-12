//! LSP Hover Information Tests
//!
//! Tests for hover information provider, including:
//! - Built-in function documentation
//! - Variable type inference and display
//! - Macro expansion previews
//! - Context-aware help information

use super::lsp_test_utils::*;
use crate::lsp::hover::{HoverProvider, HoverInfo, HoverRequest};
use crate::lsp::position::{Position, Range};

#[test]
fn test_hover_provider_creation() {
    let mut server = create_test_server();
    let hover_provider = HoverProvider::new();
    
    assert!(hover_provider.is_ready());
    assert_eq!(hover_provider.get_supported_languages(), vec!["scheme"]);
}

#[test]
fn test_hover_builtin_functions() {
    let hover_provider = HoverProvider::new();
    let document = create_test_document("test://hover.scm", "(+ 1 2 3)");
    
    // Test hover over '+' operator
    let request = HoverRequest {
        document_uri: "test://hover.scm".to_string(),
        position: Position::new(0, 1), // Position of '+'
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("+")); 
    assert!(hover.contents.contains("arithmetic"));
    assert!(hover.contents.contains("addition"));
    assert!(hover.range.is_some());
}

#[test]
fn test_hover_builtin_function_signatures() {
    let hover_provider = HoverProvider::new();
    let code = "(define test (lambda (x) (map (lambda (y) (* y 2)) x)))";
    let document = create_test_document("test://map.scm", code);
    
    // Test hover over 'map' function
    let request = HoverRequest {
        document_uri: "test://map.scm".to_string(),
        position: Position::new(0, 26), // Position of 'map'
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("map"));
    assert!(hover.contents.contains("procedure"));
    assert!(hover.contents.contains("list"));
    assert!(hover.contents.contains("(map proc list"));
}

#[test]
fn test_hover_lambda_parameters() {
    let hover_provider = HoverProvider::new();
    let code = "(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))";
    let document = create_test_document("test://lambda.scm", code);
    
    // Test hover over parameter 'n'
    let request = HoverRequest {
        document_uri: "test://lambda.scm".to_string(),
        position: Position::new(0, 18), // Position of 'n' in parameter list
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("parameter"));
    assert!(hover.contents.contains("factorial"));
}

#[test]
fn test_hover_user_defined_functions() {
    let hover_provider = HoverProvider::new();
    let code = r#"
(define (square x) (* x x))
(square 5)
"#;
    let document = create_test_document("test://user-def.scm", code);
    
    // Test hover over 'square' function call
    let request = HoverRequest {
        document_uri: "test://user-def.scm".to_string(),
        position: Position::new(2, 1), // Position of 'square' in call
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("square"));
    assert!(hover.contents.contains("user-defined"));
    assert!(hover.contents.contains("procedure"));
}

#[test]
fn test_hover_variable_definitions() {
    let hover_provider = HoverProvider::new();
    let code = r#"
(define pi 3.14159)
(define radius 5)
(* pi radius radius)
"#;
    let document = create_test_document("test://vars.scm", code);
    
    // Test hover over 'pi' variable
    let request = HoverRequest {
        document_uri: "test://vars.scm".to_string(),
        position: Position::new(3, 3), // Position of 'pi' in expression
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("pi"));
    assert!(hover.contents.contains("3.14159"));
    assert!(hover.contents.contains("number"));
}

#[test]
fn test_hover_special_forms() {
    let hover_provider = HoverProvider::new();
    let code = "(if (> x 0) x (- x))";
    let document = create_test_document("test://if.scm", code);
    
    // Test hover over 'if' special form
    let request = HoverRequest {
        document_uri: "test://if.scm".to_string(),
        position: Position::new(0, 1), // Position of 'if'
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("if"));
    assert!(hover.contents.contains("conditional"));
    assert!(hover.contents.contains("special form"));
    assert!(hover.contents.contains("(if test conseq alt)"));
}

#[test]
fn test_hover_let_bindings() {
    let hover_provider = HoverProvider::new();
    let code = "(let ((x 10) (y 20)) (+ x y))";
    let document = create_test_document("test://let.scm", code);
    
    // Test hover over 'let' special form
    let request = HoverRequest {
        document_uri: "test://let.scm".to_string(),
        position: Position::new(0, 1), // Position of 'let'
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("let"));
    assert!(hover.contents.contains("binding"));
    assert!(hover.contents.contains("(let ((var val) ...) body)"));
}

#[test]
fn test_hover_macro_definitions() {
    let hover_provider = HoverProvider::new();
    let code = r#"
(define-syntax when
  (syntax-rules ()
    ((when test stmt1 stmt2 ...)
     (if test (begin stmt1 stmt2 ...)))))
(when (> x 0) (display x))
"#;
    let document = create_test_document("test://macro.scm", code);
    
    // Test hover over 'when' macro usage
    let request = HoverRequest {
        document_uri: "test://macro.scm".to_string(),
        position: Position::new(5, 1), // Position of 'when' in usage
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("when"));
    assert!(hover.contents.contains("macro"));
    assert!(hover.contents.contains("syntax-rules"));
}

#[test]
fn test_hover_srfi_functions() {
    let hover_provider = HoverProvider::new();
    let code = "(string-append \"hello\" \" \" \"world\")";
    let document = create_test_document("test://srfi.scm", code);
    
    // Test hover over SRFI function
    let request = HoverRequest {
        document_uri: "test://srfi.scm".to_string(),
        position: Position::new(0, 1), // Position of 'string-append'
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("string-append"));
    assert!(hover.contents.contains("string"));
    assert!(hover.contents.contains("concatenate"));
}

#[test]
fn test_hover_type_information() {
    let hover_provider = HoverProvider::new();
    let code = r#"
(define num 42)
(define str "hello")
(define lst '(1 2 3))
(define proc (lambda (x) x))
"#;
    let document = create_test_document("test://types.scm", code);
    
    // Test hover over number
    let request = HoverRequest {
        document_uri: "test://types.scm".to_string(),
        position: Position::new(1, 12), // Position of '42'
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("number"));
    assert!(hover.contents.contains("42"));
}

#[test]
fn test_hover_no_match() {
    let hover_provider = HoverProvider::new();
    let code = "(+ 1 2 3)";
    let document = create_test_document("test://no-match.scm", code);
    
    // Test hover over whitespace (should return None)
    let request = HoverRequest {
        document_uri: "test://no-match.scm".to_string(),
        position: Position::new(0, 2), // Position of space
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_hover_error_handling() {
    let hover_provider = HoverProvider::new();
    let code = "(+ 1"; // Incomplete expression
    let document = create_test_document("test://error.scm", code);
    
    // Test hover in malformed code
    let request = HoverRequest {
        document_uri: "test://error.scm".to_string(),
        position: Position::new(0, 1), // Position of '+'
        context: None,
    };
    
    // Should still provide hover for '+' even in malformed code
    let result = hover_provider.provide_hover(&document, request);
    assert!(result.is_ok());
}

#[test]
fn test_hover_range_calculation() {
    let hover_provider = HoverProvider::new();
    let code = "(factorial 5)";
    let document = create_test_document("test://range.scm", code);
    
    let request = HoverRequest {
        document_uri: "test://range.scm".to_string(),
        position: Position::new(0, 5), // Middle of 'factorial'
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.range.is_some());
    
    let range = hover.range.unwrap();
    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 1); // Start of 'factorial'
    assert_eq!(range.end.character, 10);   // End of 'factorial'
}

#[test]
fn test_hover_with_context() {
    let hover_provider = HoverProvider::new();
    let code = r#"
(define (fibonacci n)
  (if (<= n 1) 
      n
      (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))
"#;
    let document = create_test_document("test://context.scm", code);
    
    let request = HoverRequest {
        document_uri: "test://context.scm".to_string(),
        position: Position::new(4, 12), // Position of recursive 'fibonacci' call
        context: Some("recursive call context".to_string()),
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("fibonacci"));
    assert!(hover.contents.contains("recursive"));
}

#[test]
fn test_hover_documentation_quality() {
    let hover_provider = HoverProvider::new();
    
    // Test that hover provides comprehensive information
    let test_cases = vec![
        ("car", "first element", "pair"),
        ("cdr", "rest", "tail"),
        ("cons", "construct", "pair"),
        ("list", "create", "list"),
        ("append", "concatenate", "lists"),
        ("length", "count", "elements"),
        ("map", "apply", "procedure"),
        ("filter", "select", "predicate"),
        ("fold", "reduce", "accumulate"),
    ];
    
    for (function, keyword1, keyword2) in test_cases {
        let code = format!("({} x)", function);
        let document = create_test_document("test://doc-quality.scm", &code);
        
        let request = HoverRequest {
            document_uri: "test://doc-quality.scm".to_string(),
            position: Position::new(0, 1),
            context: None,
        };
        
        let result = hover_provider.provide_hover(&document, request).unwrap();
        assert!(result.is_some(), "No hover for {}", function);
        
        let hover = result.unwrap();
        assert!(hover.contents.contains(function));
        assert!(
            hover.contents.to_lowercase().contains(keyword1) ||
            hover.contents.to_lowercase().contains(keyword2),
            "Missing expected keywords for {}: {}", function, hover.contents
        );
    }
}

#[test]
fn test_hover_performance() {
    let hover_provider = HoverProvider::new();
    
    // Create a larger document for performance testing
    let large_code = (0..100)
        .map(|i| format!("(define var{} {})", i, i))
        .collect::<Vec<_>>()
        .join("\n");
        
    let document = create_test_document("test://performance.scm", &large_code);
    
    let start = std::time::Instant::now();
    
    // Test multiple hover requests
    for i in 0..10 {
        let request = HoverRequest {
            document_uri: "test://performance.scm".to_string(),
            position: Position::new(i * 10, 8), // Various positions
            context: None,
        };
        
        let _result = hover_provider.provide_hover(&document, request).unwrap();
    }
    
    let duration = start.elapsed();
    
    // Should complete within reasonable time (adjust threshold as needed)
    assert!(duration.as_millis() < 100, "Hover performance too slow: {:?}", duration);
}

#[test]
fn test_hover_unicode_support() {
    let hover_provider = HoverProvider::new();
    let code = "(define π 3.14159) (define λ (lambda (x) x))";
    let document = create_test_document("test://unicode.scm", code);
    
    // Test hover over Unicode identifier 'π'
    let request = HoverRequest {
        document_uri: "test://unicode.scm".to_string(),
        position: Position::new(0, 8), // Position of 'π'
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("π"));
    assert!(hover.contents.contains("3.14159"));
}

#[test]
fn test_hover_nested_expressions() {
    let hover_provider = HoverProvider::new();
    let code = "(map (lambda (x) (+ x 1)) '(1 2 3))";
    let document = create_test_document("test://nested.scm", code);
    
    // Test hover over '+' inside nested lambda
    let request = HoverRequest {
        document_uri: "test://nested.scm".to_string(),
        position: Position::new(0, 18), // Position of '+' in lambda
        context: None,
    };
    
    let result = hover_provider.provide_hover(&document, request).unwrap();
    assert!(result.is_some());
    
    let hover = result.unwrap();
    assert!(hover.contents.contains("+"));
    assert!(hover.contents.contains("addition"));
}
use lambdust::lexer::tokenize;
use lambdust::parser::{parse_with_loop_detection, LoopDetectionConfig};
use lambdust::error::LambdustError;

#[test]
fn test_parser_integration_simple_circular_dependency() {
    let source = "(define x x)";
    let tokens = tokenize(source).unwrap();
    
    let config = LoopDetectionConfig::default();
    let result = parse_with_loop_detection(tokens, config);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        LambdustError::ParseError { message, .. } => {
            assert!(message.contains("Infinite loop detected"));
            assert!(message.contains("Circular dependency detected"));
        }
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_parser_integration_mutual_circular_dependency() {
    let source = r#"
        (define x y)
        (define y x)
    "#;
    let tokens = tokenize(source).unwrap();
    
    let config = LoopDetectionConfig::default();
    let result = parse_with_loop_detection(tokens, config);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        LambdustError::ParseError { message, .. } => {
            assert!(message.contains("Infinite loop detected"));
        }
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_parser_integration_infinite_recursion() {
    let source = "(define (loop x) (loop x))";
    let tokens = tokenize(source).unwrap();
    
    let config = LoopDetectionConfig::default();
    let result = parse_with_loop_detection(tokens, config);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        LambdustError::ParseError { message, .. } => {
            assert!(message.contains("Infinite loop detected"));
        }
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_parser_integration_valid_recursive_function() {
    let source = r#"
        (define (factorial n)
          (if (= n 0)
              1
              (* n (factorial (- n 1)))))
    "#;
    let tokens = tokenize(source).unwrap();
    
    let config = LoopDetectionConfig::default();
    let result = parse_with_loop_detection(tokens, config);
    
    assert!(result.is_ok());
}

#[test]
fn test_parser_integration_valid_non_recursive_code() {
    let source = r#"
        (define x 42)
        (define y (+ x 1))
        (define z (* y 2))
    "#;
    let tokens = tokenize(source).unwrap();
    
    let config = LoopDetectionConfig::default();
    let result = parse_with_loop_detection(tokens, config);
    
    assert!(result.is_ok());
}

#[test]
fn test_parser_integration_disabled_detection() {
    let source = "(define x x)";
    let tokens = tokenize(source).unwrap();
    
    let config = LoopDetectionConfig {
        enable_cycle_detection: false,
        ..Default::default()
    };
    let result = parse_with_loop_detection(tokens, config);
    
    // Should succeed when detection is disabled
    assert!(result.is_ok());
}

#[test]
fn test_parser_integration_warn_only_mode() {
    let source = "(define x x)";
    let tokens = tokenize(source).unwrap();
    
    let config = LoopDetectionConfig {
        warn_only: true,
        ..Default::default()
    };
    let result = parse_with_loop_detection(tokens, config);
    
    // Should succeed in warn-only mode
    if let Err(e) = result {
        panic!("Expected success in warn-only mode, but got error: {:?}", e);
    }
}

#[test]
fn test_parser_integration_complex_circular_dependency() {
    let source = r#"
        (define a (+ b 1))
        (define b (+ c 1))
        (define c (+ a 1))
    "#;
    let tokens = tokenize(source).unwrap();
    
    let config = LoopDetectionConfig::default();
    let result = parse_with_loop_detection(tokens, config);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        LambdustError::ParseError { message, .. } => {
            assert!(message.contains("Infinite loop detected"));
        }
        _ => panic!("Expected ParseError"),
    }
}

#[test]
fn test_parser_integration_valid_forward_reference() {
    let source = r#"
        (define (f x) (g x))
        (define (g x) (+ x 1))
    "#;
    let tokens = tokenize(source).unwrap();
    
    let config = LoopDetectionConfig::default();
    let result = parse_with_loop_detection(tokens, config);
    
    // Forward reference should be OK
    assert!(result.is_ok());
}

#[test]
fn test_parser_integration_mutual_recursion_with_base_case() {
    let source = r#"
        (define (even? n)
          (if (= n 0)
              #t
              (odd? (- n 1))))
        (define (odd? n)
          (if (= n 0)
              #f
              (even? (- n 1))))
    "#;
    let tokens = tokenize(source).unwrap();
    
    let config = LoopDetectionConfig::default();
    let result = parse_with_loop_detection(tokens, config);
    
    // Mutual recursion with base case should be OK
    assert!(result.is_ok());
}
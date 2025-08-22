//! Integration test for Phase 2 Group A parser enhancements
//!
//! This test file verifies that the enhanced parsing capabilities work correctly.

use lambdust::lexer::Lexer;
use lambdust::parser::{
    Parser,
    contextual_errors::ContextualErrorGenerator,
    error_recovery::{ErrorRecoveryEngine, RecoveryStrategy},
    ide_support::IdeSupportEngine,
    lsp_integration::{LambdustLanguageServer, LanguageServerConfig},
    partial_parser::{ParsingMode, PartialParser, PartialParsingConfig},
    realtime_feedback::{RealtimeFeedbackConfig, RealtimeFeedbackEngine},
};

#[test]
fn test_phase2_parser_basic_functionality() {
    let input = "(define x 42)";
    let mut lexer = Lexer::new(input, None);
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.expressions.len(), 1);
}

#[test]
fn test_phase2_partial_parser() {
    let input = "(define (square x) (* x x))";
    let mut lexer = Lexer::new(input, None);
    let tokens = lexer.tokenize().unwrap();

    let config = PartialParsingConfig {
        mode: ParsingMode::Partial,
        max_errors: 10,
        use_placeholders: false,
        ..Default::default()
    };

    let mut parser = PartialParser::new(tokens, config);
    let result = parser.parse_partial();

    assert_eq!(result.stats.expressions_parsed, 1);
    assert!(result.errors.is_empty());
}

#[test]
fn test_phase2_error_recovery() {
    let _engine = ErrorRecoveryEngine::new(RecoveryStrategy::LocalRepair);
    // Engine creation test - if we get here without panic, it passes
}

#[test]
fn test_phase2_contextual_errors() {
    let _generator = ContextualErrorGenerator::new();
    // Generator creation test - if we get here without panic, it passes
}

#[test]
fn test_phase2_ide_support() {
    let engine = IdeSupportEngine::new();
    let capabilities = engine.capabilities();

    assert!(capabilities.hover_provider);
    assert!(capabilities.completion_provider.is_some());
}

#[test]
fn test_phase2_realtime_feedback() {
    let config = RealtimeFeedbackConfig::default();
    let _engine = RealtimeFeedbackEngine::new(config);
    // Engine creation test - if we get here without panic, it passes
}

#[test]
fn test_phase2_lsp_integration() {
    let config = LanguageServerConfig::default();
    let server = LambdustLanguageServer::new(config);

    let capabilities = server.capabilities();
    assert!(capabilities.hover_provider);
    assert!(capabilities.completion_provider.is_some());
}

#[test]
fn test_phase2_error_tolerance() {
    let input = "(define x"; // Incomplete input
    let mut lexer = Lexer::new(input, None);
    let tokens = lexer.tokenize().unwrap();

    let config = PartialParsingConfig {
        mode: ParsingMode::ErrorTolerant,
        max_errors: 5,
        auto_insert_tokens: true,
        repair_delimiters: true,
        use_placeholders: true,
        ..Default::default()
    };

    let mut parser = PartialParser::new(tokens, config);
    let result = parser.parse_partial();

    // Should handle the incomplete input gracefully without panicking
    assert!(result.stats.tokens_processed > 0);
}

#[test]
fn test_phase2_multiple_expressions() {
    let input = "(define x 1) (define y 2) (+ x y)";
    let mut lexer = Lexer::new(input, None);
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.expressions.len(), 3);
}

#[test]
fn test_phase2_nested_expressions() {
    let input = "(define (factorial n) (if (= n 0) 1 (* n (factorial (- n 1)))))";
    let mut lexer = Lexer::new(input, None);
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.expressions.len(), 1);
}

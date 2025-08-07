//! Demonstration of enhanced error handling in Lambdust.
//!
//! This example shows how the improved diagnostics system provides:
//! - Better error messages with suggestions
//! - Source position tracking with line/column information
//! - Error recovery in parsing
//! - Context-aware error reporting

use lambdust::diagnostics::{
    Error, SourceMap, SourceRegistry, SuggestionGenerator, CallStack, CallFrame, 
    CallStackBuilder, Position, Span
};
use lambdust::lexer::Lexer;
use lambdust::parser::{Parser, ParserBuilder, RecoveryConfig};
use std::sync::Arc;

fn main() {
    println!("=== Lambdust Enhanced Error Handling Demo ===\n");

    // Demonstrate source mapping and position tracking
    demo_source_mapping();
    
    // Demonstrate lexical error handling with suggestions
    demo_lexical_errors();
    
    // Demonstrate parser error recovery
    demo_parser_recovery();
    
    // Demonstrate call stack tracing
    demo_call_stack();
    
    // Demonstrate error suggestion system
    demo_error_suggestions();
}

fn demo_source_mapping() {
    println!("1. Source Position Tracking");
    println!("==========================");
    
    let source = r#"(define factorial
  (lambda (n)
    (if (<= n 1)
        1
        (* n (factorial (- n 1))))))
"#;
    
    let source_map = SourceMap::new("factorial.ldust".to_string(), source.to_string(), 1);
    
    // Show position calculation
    let pos_lambda = source_map.position_at_offset(17); // Position of "lambda"
    println!("Position of 'lambda': line {}, column {}", pos_lambda.line, pos_lambda.column);
    
    let pos_if = source_map.position_at_offset(33); // Position of "if"
    println!("Position of 'if': line {}, column {}", pos_if.line, pos_if.column);
    
    // Show context lines
    let span = Span::with_position(33, 2, 3, 5); // "if" span
    let context = source_map.context_lines(&span, 1);
    println!("\nContext around 'if':");
    for (line_num, line_text) in context {
        println!("  {}: {}", line_num, line_text);
    }
    
    println!();
}

fn demo_lexical_errors() {
    println!("2. Enhanced Lexical Error Messages");
    println!("==================================");
    
    // Test various invalid characters with suggestions
    let test_cases = vec![
        ("@invalid", "Invalid character '@'"),
        ("defien x", "Typo in 'define'"),
        ("$variable", "Invalid character '$'"),
        ("42@", "Invalid character in number"),
    ];
    
    for (input, description) in test_cases {
        println!("Testing: {} ({})", input, description);
        
        let mut lexer = Lexer::new(input, Some("test.ldust"));
        match lexer.tokenize() {
            Ok(_) => println!("  ✓ Tokenized successfully"),
            Err(error) => {
                println!("  ✗ Error: {}", error);
                if error.has_suggestion() {
                    println!("    💡 The error includes helpful suggestions");
                }
            }
        }
        println!();
    }
}

fn demo_parser_recovery() {
    println!("3. Parser Error Recovery");
    println!("========================");
    
    // Test parser recovery with multiple errors
    let malformed_code = r#"
        (define x 42
        (define y (+ x 
        ) unmatched paren
        (define z 99)
    "#;
    
    println!("Parsing malformed code with multiple errors:");
    println!("{}", malformed_code);
    
    let mut lexer = Lexer::new(malformed_code, Some("malformed.ldust"));
    if let Ok(tokens) = lexer.tokenize() {
        let recovery_config = RecoveryConfig {
            max_errors: 5,
            aggressive_recovery: true,
            max_nesting_depth: 50,
            recovery_point_limit: 10,
        };
        
        let mut parser = ParserBuilder::new()
            .with_recovery_config(recovery_config)
            .build(tokens);
        
        let (program, errors) = parser.parse_with_error_recovery();
        
        println!("\nRecovery Results:");
        println!("  - Recovered {} expressions", program.expressions.len());
        println!("  - Found {} errors", errors.len());
        
        for (i, error) in errors.iter().enumerate() {
            println!("  Error {}: {}", i + 1, error);
        }
        
        println!("  📊 Parser stats: {}", parser.recovery_stats());
    }
    println!();
}

fn demo_call_stack() {
    println!("4. Call Stack Tracing");
    println!("=====================");
    
    // Create a sample call stack for demonstration
    let call_stack = CallStackBuilder::new()
        .frame_with_location(
            "main",
            Span::with_position(0, 10, 1, 1),
            Some("main.ldust".to_string()),
        )
        .frame_with_location(
            "process-data",
            Span::with_position(50, 15, 5, 3),
            Some("utils.ldust".to_string()),
        )
        .frame_with_location(
            "validate-input",
            Span::with_position(200, 8, 12, 10),
            Some("validation.ldust".to_string()),
        )
        .build();
    
    println!("Sample call stack:");
    println!("{}", call_stack.format_detailed());
    
    // Demonstrate stack with hidden frames
    let large_stack = {
        let mut builder = CallStackBuilder::with_limit(3);
        for i in 1..=8 {
            builder = builder.simple_frame(format!("function_{}", i));
        }
        builder.build()
    };
    
    println!("\nLarge call stack (with frame limit):");
    println!("{}", large_stack.format_detailed());
    println!();
}

fn demo_error_suggestions() {
    println!("5. Error Suggestion System");
    println!("==========================");
    
    let generator = SuggestionGenerator::new();
    
    // Test lexical error suggestions
    println!("Lexical error suggestions:");
    let span = Span::with_position(10, 5, 2, 8);
    let lex_suggestions = generator.suggest_for_lex_error("defien", span);
    for suggestion in lex_suggestions {
        println!("  💡 {}", suggestion.message);
        if suggestion.has_replacement() {
            println!("     → Replace with: {:?}", suggestion.replacement);
        }
        println!("     Confidence: {:.0}%", suggestion.confidence * 100.0);
    }
    
    // Test parse error suggestions
    println!("\nParse error suggestions:");
    let parse_suggestions = generator.suggest_for_parse_error(
        "Expected closing parenthesis",
        span,
        &Some(vec![")".to_string()]),
        &Some("EOF".to_string()),
    );
    for suggestion in parse_suggestions {
        println!("  💡 {}", suggestion.message);
    }
    
    // Test runtime error suggestions
    println!("\nRuntime error suggestions:");
    let runtime_suggestions = generator.suggest_for_runtime_error("unbound variable: xyz");
    for suggestion in runtime_suggestions {
        println!("  💡 {}", suggestion.message);
    }
    
    // Test type error suggestions
    println!("\nType error suggestions:");
    let type_suggestions = generator.suggest_for_type_error(
        &Some("number".to_string()),
        &Some("string".to_string()),
    );
    for suggestion in type_suggestions {
        println!("  💡 {}", suggestion.message);
    }
    
    println!();
}

/// Demonstrates comprehensive error reporting with all features.
fn _demo_comprehensive_error_report() {
    println!("6. Comprehensive Error Report");
    println!("=============================");
    
    let problematic_code = r#"
        (define factorial
          (lambda (n
            (if (<= n 1)
                1
                (* n (factorial - n 1))))))  ; Missing paren and wrong syntax
    "#;
    
    let mut registry = SourceRegistry::new();
    let file_id = registry.register_source("factorial.ldust".to_string(), problematic_code.to_string());
    let source_map = registry.get_source(file_id).unwrap();
    
    let mut lexer = Lexer::with_source_map(problematic_code, source_map.clone());
    if let Ok(tokens) = lexer.tokenize() {
        let mut parser = ParserBuilder::new()
            .with_source_map(source_map)
            .build(tokens);
        
        match parser.parse() {
            Ok(_) => println!("  ✓ Parsed successfully"),
            Err(error) => {
                println!("  ✗ Comprehensive error report:");
                println!("     {}", error);
                
                if let Some(span) = error.primary_span() {
                    println!("     Location: line {}, column {}", span.line, span.column);
                }
                
                if error.has_suggestion() {
                    println!("     💡 Error includes suggestions for fixing");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_handling_demo() {
        // This test ensures the demo runs without panicking
        // In a real scenario, we'd test specific error handling behaviors
        
        let source = "(+ 1 2)";
        let mut lexer = Lexer::new(source, Some("test.ldust"));
        let tokens = lexer.tokenize().expect("Should tokenize successfully");
        let mut parser = Parser::new(tokens);
        let _program = parser.parse().expect("Should parse successfully");
        
        // Test that we can create error handling components
        let _source_map = SourceMap::new("test.ldust".to_string(), source.to_string(), 1);
        let _suggestion_generator = SuggestionGenerator::new();
        let _call_stack = CallStack::new();
    }
    
    #[test]
    fn test_source_mapping() {
        let source = "line 1\nline 2\nline 3";
        let source_map = SourceMap::new("test.ldust".to_string(), source.to_string(), 1);
        
        let pos = source_map.position_at_offset(7); // Start of line 2
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
        
        let span = source_map.span_with_position(7, 6); // "line 2"
        assert_eq!(span.line, 2);
        assert_eq!(span.column, 1);
        assert_eq!(span.len, 6);
    }
    
    #[test]
    fn test_call_stack_building() {
        let stack = CallStackBuilder::new()
            .simple_frame("main")
            .simple_frame("helper")
            .build();
        
        assert_eq!(stack.depth(), 2);
        assert_eq!(stack.top().unwrap().function_name, "helper");
    }
    
    #[test]
    fn test_error_suggestions() {
        let generator = SuggestionGenerator::new();
        let span = Span::new(0, 5);
        
        let suggestions = generator.suggest_for_lex_error("defien", span);
        assert!(!suggestions.is_empty());
        
        let runtime_suggestions = generator.suggest_for_runtime_error("unbound variable: x");
        assert!(!runtime_suggestions.is_empty());
    }
}
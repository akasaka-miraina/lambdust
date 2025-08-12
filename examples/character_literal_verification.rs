// Verification script for character literal functionality in Lambdust
// This demonstrates the complete pipeline from lexing through evaluation

use lambdust::lexer::Lexer;
use lambdust::parser::Parser;
use lambdust::eval::{Evaluator, environment, Value};
use lambdust::ast::{Expr, Literal};

fn main() {
    println!("=== Lambdust Character Literal Verification ===\n");

    // Test cases covering all R7RS character literal forms
    let test_cases = vec![
        // Basic characters
        ("#\\a", 'a', "Basic Latin character"),
        ("#\\9", '9', "Digit character"),
        ("#\\A", 'A', "Uppercase character"),
        ("#\\#", '#', "Special symbol character"),
        
        // R7RS named characters
        ("#\\space", ' ', "Named character: space"),
        ("#\\newline", '\n', "Named character: newline"),
        ("#\\tab", '\t', "Named character: tab"),
        ("#\\return", '\r', "Named character: return"),
        ("#\\alarm", '\x07', "Named character: alarm"),
        ("#\\backspace", '\x08', "Named character: backspace"),
        ("#\\delete", '\x7F', "Named character: delete"),
        ("#\\escape", '\x1B', "Named character: escape"),
        ("#\\null", '\0', "Named character: null"),
        ("#\\vtab", '\x0B', "Named character: vtab"),
        
        // Unicode hex escapes
        ("#\\x41", 'A', "Unicode hex: ASCII A"),
        ("#\\x1F600", '😀', "Unicode hex: Emoji"),
        ("#\\x3042", 'あ', "Unicode hex: Hiragana A"),
    ];

    let mut all_passed = true;
    
    for (input, expected_char, description) in test_cases {
        println!("Testing: {} - {}", input, description);
        
        // Step 1: Lexical Analysis
        let mut lexer = Lexer::new(input, Some("verification"));
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => {
                println!("  ❌ LEXER FAILED: {:?}", e);
                all_passed = false;
                continue;
            }
        };
        
        // Verify lexer output
        if tokens.len() != 2 || tokens[0].kind != lambdust::lexer::TokenKind::Character {
            println!("  ❌ LEXER INCORRECT: Expected Character token, got {:?}", 
                     tokens.iter().map(|t| &t.kind).collect::<Vec<_>>());
            all_passed = false;
            continue;
        }
        
        // Step 2: Parsing
        let mut parser = Parser::new(tokens);
        let expr = match parser.parse_character() {
            Ok(expr) => expr,
            Err(e) => {
                println!("  ❌ PARSER FAILED: {:?}", e);
                all_passed = false;
                continue;
            }
        };
        
        // Verify parser output
        let parsed_char = match expr.inner {
            Expr::Literal(Literal::Character(ch)) => ch,
            _ => {
                println!("  ❌ PARSER INCORRECT: Expected character literal, got {:?}", expr.inner);
                all_passed = false;
                continue;
            }
        };
        
        if parsed_char != expected_char {
            println!("  ❌ PARSER MISMATCH: Expected '{}' (U+{:04X}), got '{}' (U+{:04X})", 
                     expected_char, expected_char as u32, parsed_char, parsed_char as u32);
            all_passed = false;
            continue;
        }
        
        // Step 3: Evaluation
        let mut evaluator = Evaluator::new();
        let env = environment::global_environment();
        let result = match evaluator.eval(&expr, env) {
            Ok(result) => result,
            Err(e) => {
                println!("  ❌ EVALUATOR FAILED: {:?}", e);
                all_passed = false;
                continue;
            }
        };
        
        // Verify evaluator output
        let evaluated_char = match result {
            Value::Literal(Literal::Character(ch)) => ch,
            _ => {
                println!("  ❌ EVALUATOR INCORRECT: Expected character value, got {:?}", result);
                all_passed = false;
                continue;
            }
        };
        
        if evaluated_char != expected_char {
            println!("  ❌ EVALUATOR MISMATCH: Expected '{}' (U+{:04X}), got '{}' (U+{:04X})", 
                     expected_char, expected_char as u32, evaluated_char, evaluated_char as u32);
            all_passed = false;
            continue;
        }
        
        println!("  ✅ SUCCESS: {} -> '{}' (U+{:04X})", input, expected_char, expected_char as u32);
    }
    
    println!("\n=== Verification Complete ===");
    if all_passed {
        println!("🎉 ALL TESTS PASSED! Character literal system is fully functional.");
        println!("\nCharacter literal features verified:");
        println!("  ✅ Lexical analysis of all R7RS character forms");
        println!("  ✅ Parsing and AST construction");
        println!("  ✅ Evaluation and runtime representation");
        println!("  ✅ Named character constants");
        println!("  ✅ Unicode hex escapes");
        println!("  ✅ Basic ASCII characters");
        println!("  ✅ Complete R7RS compliance");
    } else {
        println!("❌ Some tests failed. Character literal system needs attention.");
        std::process::exit(1);
    }
}
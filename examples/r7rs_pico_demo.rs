//! R7RS-pico Ultra-Minimal Scheme Demonstration
//!
//! This example demonstrates the R7RS-pico ultra-minimal Scheme implementation,
//! showcasing the simplified U -> E semantic model and core functionality
//! designed for embedded systems and educational purposes.

#[cfg(feature = "pico")]
use lambdust::evaluator::{create_pico_initial_environment, get_pico_features, PicoEvaluator};
#[cfg(feature = "pico")]
use lambdust::parser::Parser;
#[cfg(feature = "pico")]
use lambdust::lexer::Lexer;

#[cfg(feature = "pico")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 R7RS-pico Ultra-Minimal Scheme Demonstration");
    println!("================================================");
    
    // Display R7RS-pico features
    let features = get_pico_features();
    println!("\n📋 R7RS-pico Language Features:");
    println!("   Semantic Model: {}", features.semantic_model);
    println!("   Built-in Procedures: {}", features.builtin_procedures);
    println!("   Memory Model: {}", features.memory_model);
    println!("   Tail Recursion: {}", features.tail_recursion);
    
    println!("\n✅ Supported Types:");
    for typ in &features.supported_types {
        println!("   • {}", typ);
    }
    
    println!("\n✅ Special Forms:");
    for form in &features.special_forms {
        println!("   • {}", form);
    }
    
    println!("\n❌ Excluded Features (from full R7RS):");
    for feature in &features.excluded_features {
        println!("   • {}", feature);
    }
    
    // Create evaluator and initial environment
    let mut evaluator = PicoEvaluator::new();
    let env = create_pico_initial_environment();
    
    println!("\n🧮 Basic Arithmetic Operations:");
    println!("================================");
    
    // Test arithmetic
    let arithmetic_tests = vec![
        "(+ 3 4)",
        "(- 10 3)",
        "(* 5 6)",
        "(- 7)",  // unary negation
        "(= 5 5)",
        "(< 3 7)",
        "(> 8 2)",
    ];
    
    for test in arithmetic_tests {
        match parse_and_evaluate(&mut evaluator, test, env.clone()) {
            Ok(result) => println!("   {} => {:?}", test, result),
            Err(e) => println!("   {} => Error: {:?}", test, e),
        }
    }
    
    println!("\n📋 List Operations:");
    println!("==================");
    
    let list_tests = vec![
        "(cons 1 2)",
        "(cons 1 (cons 2 3))",
        "(car (cons 1 2))",
        "(cdr (cons 1 2))",
        "(car (cons 1 (cons 2 3)))",
        "(cdr (cons 1 (cons 2 3)))",
    ];
    
    for test in list_tests {
        match parse_and_evaluate(&mut evaluator, test, env.clone()) {
            Ok(result) => println!("   {} => {:?}", test, result),
            Err(e) => println!("   {} => Error: {:?}", test, e),
        }
    }
    
    println!("\n🔍 Type Predicates:");
    println!("==================");
    
    let predicate_tests = vec![
        "(number? 42)",
        "(number? #t)",
        "(boolean? #t)",
        "(boolean? 42)",
        "(null? (quote ()))",
        "(pair? (cons 1 2))",
        "(pair? 42)",
        "(symbol? (quote hello))",
        "(procedure? +)",
    ];
    
    for test in predicate_tests {
        match parse_and_evaluate(&mut evaluator, test, env.clone()) {
            Ok(result) => println!("   {} => {:?}", test, result),
            Err(e) => println!("   {} => Error: {:?}", test, e),
        }
    }
    
    println!("\n⚖️ Equivalence Testing:");
    println!("======================");
    
    let equivalence_tests = vec![
        "(eqv? 5 5)",
        "(eqv? 5 6)",
        "(eqv? #t #t)",
        "(eqv? #t #f)",
        "(eqv? (quote hello) (quote hello))",
        "(eqv? (quote hello) (quote world))",
    ];
    
    for test in equivalence_tests {
        match parse_and_evaluate(&mut evaluator, test, env.clone()) {
            Ok(result) => println!("   {} => {:?}", test, result),
            Err(e) => println!("   {} => Error: {:?}", test, e),
        }
    }
    
    println!("\n🎯 Conditional Expressions:");
    println!("===========================");
    
    let conditional_tests = vec![
        "(if #t 1 2)",
        "(if #f 1 2)",
        "(if (< 3 5) (quote yes) (quote no))",
        "(if (> 3 5) (quote yes) (quote no))",
        "(if #t 42)",  // no alternative
        "(if #f 42)",  // no alternative
    ];
    
    for test in conditional_tests {
        match parse_and_evaluate(&mut evaluator, test, env.clone()) {
            Ok(result) => println!("   {} => {:?}", test, result),
            Err(e) => println!("   {} => Error: {:?}", test, e),
        }
    }
    
    println!("\n📜 Quoted Expressions:");
    println!("======================");
    
    let quote_tests = vec![
        "(quote hello)",
        "(quote 42)",
        "(quote #t)",
        "(quote (1 2 3))",
        "(quote (hello world))",
    ];
    
    for test in quote_tests {
        match parse_and_evaluate(&mut evaluator, test, env.clone()) {
            Ok(result) => println!("   {} => {:?}", test, result),
            Err(e) => println!("   {} => Error: {:?}", test, e),
        }
    }
    
    println!("\n🔧 Function Definition and Application:");
    println!("=======================================");
    
    // Test simple function definition
    let define_test = "(define square (lambda (x) (* x x)))";
    match parse_and_evaluate(&mut evaluator, define_test, env.clone()) {
        Ok(_) => println!("   {} => Defined", define_test),
        Err(e) => println!("   {} => Error: {:?}", define_test, e),
    }
    
    // Test function application
    let application_test = "(square 5)";
    match parse_and_evaluate(&mut evaluator, application_test, env.clone()) {
        Ok(result) => println!("   {} => {:?}", application_test, result),
        Err(e) => println!("   {} => Error: {:?}", application_test, e),
    }
    
    // Test simple recursive function
    let factorial_def = "(define fact (lambda (n) (if (= n 0) 1 (* n (fact (- n 1))))))";
    match parse_and_evaluate(&mut evaluator, factorial_def, env.clone()) {
        Ok(_) => println!("   {} => Defined", factorial_def),
        Err(e) => println!("   {} => Error: {:?}", factorial_def, e),
    }
    
    let factorial_test = "(fact 5)";
    match parse_and_evaluate(&mut evaluator, factorial_test, env.clone()) {
        Ok(result) => println!("   {} => {:?}", factorial_test, result),
        Err(e) => println!("   {} => Error: {:?}", factorial_test, e),
    }
    
    println!("\n📊 Evaluator Statistics:");
    println!("========================");
    println!("   Maximum recursion depth: {}", evaluator.max_depth());
    println!("   Current recursion depth: {}", evaluator.current_depth());
    
    println!("\n✨ R7RS-pico Benefits:");
    println!("======================");
    println!("   • Ultra-minimal implementation (< 200KB binary)");
    println!("   • Simplified U -> E semantic model");
    println!("   • No side effects or complex control structures");
    println!("   • Perfect for embedded systems and education");
    println!("   • Proper tail recursion support");
    println!("   • Easy to understand and implement");
    
    println!("\n🎉 R7RS-pico demonstration completed successfully!");
    
    Ok(())
}

#[cfg(feature = "pico")]
fn parse_and_evaluate(
    evaluator: &mut PicoEvaluator,
    input: &str,
    env: std::rc::Rc<lambdust::environment::Environment>
) -> Result<lambdust::value::Value, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;
    let result = evaluator.evaluate(&expr, env)?;
    Ok(result)
}

#[cfg(not(feature = "pico"))]
fn main() {
    println!("R7RS-pico demo requires the 'pico' feature to be enabled.");
    println!("Run with: cargo run --example r7rs_pico_demo --features pico");
}
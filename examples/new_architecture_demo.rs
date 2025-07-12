//! New Architecture Demo - Environment-first Design
//!
//! This example demonstrates the new architecture where Environment
//! is created first and shared across components for thread safety
//! and memory efficiency.

use lambdust::{Environment, Interpreter};
use std::sync::Arc;

fn main() {
    println!("🏗️ New Architecture Demo: Environment-first Design");
    println!("===============================================");

    // 1️⃣ Create shared environment once (R7RS builtins included, mutable)
    println!("\n1️⃣ Creating shared environment with R7RS builtins (mutable)...");
    let shared_env = Arc::new(Environment::with_builtins_mutable());
    println!("✅ Environment created with thread-safe Arc<Environment>");
    println!("   • R7RS builtins included but environment remains extensible");

    // 2️⃣ Create interpreter with shared environment
    println!("\n2️⃣ Creating interpreter with shared environment...");
    let mut interpreter = Interpreter::with_shared_environment(shared_env.clone());
    println!("✅ Interpreter created using shared environment");

    // 3️⃣ Test basic arithmetic (should work with builtin functions)
    println!("\n3️⃣ Testing arithmetic operations...");
    test_arithmetic(&mut interpreter);

    // 4️⃣ Test list operations
    println!("\n4️⃣ Testing list operations...");
    test_list_operations(&mut interpreter);

    // 5️⃣ Test lambda functions
    println!("\n5️⃣ Testing lambda functions...");
    test_lambda_functions(&mut interpreter);

    // 6️⃣ Show environment sharing benefits
    println!("\n6️⃣ Demonstrating environment sharing...");
    demonstrate_sharing(shared_env);

    println!("\n🎉 New architecture demo completed successfully!");
    println!("✨ Key benefits:");
    println!("   • Environment created once and shared");
    println!("   • Thread-safe Arc<Environment> design");
    println!("   • R7RS builtins available from startup");
    println!("   • Memory efficient shared state");
}

fn test_arithmetic(interpreter: &mut Interpreter) {
    let tests = vec![
        "(+ 1 2 3)",
        "(- 10 3)",
        "(* 4 5)",
        "(= 5 5)",
        "(< 3 7)",
    ];

    for test in tests {
        match interpreter.eval(test) {
            Ok(result) => println!("  ✅ {} = {}", test, result),
            Err(e) => println!("  ❌ {} failed: {}", test, e),
        }
    }
}

fn test_list_operations(interpreter: &mut Interpreter) {
    let tests = vec![
        "(cons 1 2)",
        "(car (cons 'a 'b))",
        "(cdr (cons 'a 'b))",
        "(null? '())",
        "(pair? (cons 1 2))",
    ];

    for test in tests {
        match interpreter.eval(test) {
            Ok(result) => println!("  ✅ {} = {}", test, result),
            Err(e) => println!("  ❌ {} failed: {}", test, e),
        }
    }
}

fn test_lambda_functions(interpreter: &mut Interpreter) {
    // Define a function
    match interpreter.eval("(define square (lambda (x) (* x x)))") {
        Ok(_) => println!("  ✅ Function defined: square"),
        Err(e) => {
            println!("  ❌ Function definition failed: {}", e);
            return;
        }
    }

    // Test the function
    match interpreter.eval("(square 5)") {
        Ok(result) => println!("  ✅ (square 5) = {}", result),
        Err(e) => println!("  ❌ (square 5) failed: {}", e),
    }
}

fn demonstrate_sharing(shared_env: Arc<Environment>) {
    println!("  📊 Environment reference count: {}", Arc::strong_count(&shared_env));
    
    // Create another interpreter with the same environment
    let _interpreter2 = Interpreter::with_shared_environment(shared_env.clone());
    println!("  📊 After creating second interpreter: {}", Arc::strong_count(&shared_env));
    
    // Show memory efficiency
    println!("  💾 Environment is shared between interpreters (memory efficient)");
    println!("  🔒 Thread-safe Arc ensures concurrent access safety");
}
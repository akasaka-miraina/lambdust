//! Mdo Syntax Demo
//! Demonstrates the new mdo (monadic do) notation in Lambdust
//! This avoids conflict with Scheme's native do loops

use lambdust::macros::do_notation::{DoNotationExpander, DoBlock, DoBinding};
use lambdust::ast::{Expr, Literal};
use lambdust::lexer::SchemeNumber;

fn main() {
    println!("🚀 Lambdust Mdo Syntax Demo");
    println!("==========================");
    
    // Create expander
    let expander = DoNotationExpander::new();
    
    // Example 1: List Monad with mdo
    println!("\n📋 Example 1: List Monad");
    println!("Scheme syntax:");
    println!("(mdo [x <- (list 1 2 3)]");
    println!("     [y <- (list 4 5 6)]");
    println!("     (+ x y))");
    
    let list_mdo = DoBlock {
        bindings: vec![
            DoBinding::Bind {
                var: "x".to_string(),
                computation: Expr::List(vec![
                    Expr::Variable("list".to_string()),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
                ]),
            },
            DoBinding::Bind {
                var: "y".to_string(),
                computation: Expr::List(vec![
                    Expr::Variable("list".to_string()),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(4))),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(5))),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(6))),
                ]),
            },
        ],
        result: Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]),
    };
    
    match expander.expand_mdo(list_mdo) {
        Ok(expanded) => {
            println!("\nExpanded to:");
            println!("{}", expanded);
        }
        Err(e) => println!("Error: {}", e),
    }
    
    // Example 2: Maybe Monad with mdo  
    println!("\n🤔 Example 2: Maybe Monad");
    println!("Scheme syntax:");
    println!("(mdo [x <- (just 42)]");
    println!("     [y <- (just 8)]");
    println!("     (* x y))");
    
    let maybe_mdo = DoBlock {
        bindings: vec![
            DoBinding::Bind {
                var: "x".to_string(),
                computation: Expr::List(vec![
                    Expr::Variable("just".to_string()),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
                ]),
            },
            DoBinding::Bind {
                var: "y".to_string(),
                computation: Expr::List(vec![
                    Expr::Variable("just".to_string()),
                    Expr::Literal(Literal::Number(SchemeNumber::Integer(8))),
                ]),
            },
        ],
        result: Expr::List(vec![
            Expr::Variable("*".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Variable("y".to_string()),
        ]),
    };
    
    match expander.expand_mdo(maybe_mdo) {
        Ok(expanded) => {
            println!("\nExpanded to:");
            println!("{}", expanded);
        }
        Err(e) => println!("Error: {}", e),
    }
    
    // Example 3: Contrast with Scheme's native do loop
    println!("\n🔄 Example 3: Scheme's Native Do Loop (unchanged)");
    println!("R7RS do loop syntax (still works as before):");
    println!("(do ((i 0 (+ i 1)))");
    println!("    ((>= i 10) i)");
    println!("  (display i))");
    println!("\nThis continues to work as standard Scheme do loops!");
    
    // Example 4: Let binding in mdo
    println!("\n📝 Example 4: Let Binding in Mdo");
    println!("Scheme syntax:");
    println!("(mdo [let x 42]");
    println!("     [y <- (just x)]");
    println!("     (* y 2))");
    
    let let_mdo = DoBlock {
        bindings: vec![
            DoBinding::Let {
                var: "x".to_string(),
                value: Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
            },
            DoBinding::Bind {
                var: "y".to_string(),
                computation: Expr::List(vec![
                    Expr::Variable("just".to_string()),
                    Expr::Variable("x".to_string()),
                ]),
            },
        ],
        result: Expr::List(vec![
            Expr::Variable("*".to_string()),
            Expr::Variable("y".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]),
    };
    
    match expander.expand_mdo(let_mdo) {
        Ok(expanded) => {
            println!("\nExpanded to:");
            println!("{}", expanded);
        }
        Err(e) => println!("Error: {}", e),
    }
    
    // Show available monads
    println!("\n🏗️  Available Monad Instances:");
    let available_monads = ["List", "Maybe", "IO", "State"];
    for monad_name in &available_monads {
        if let Some(monad) = expander.get_monad(monad_name) {
            println!("  {} -> return: {}, bind: {}", 
                monad.name, monad.return_fn, monad.bind_fn);
        }
    }
    
    println!("\n✅ Mdo syntax successfully differentiates from Scheme's do loops!");
    println!("📚 Now both can coexist peacefully in Lambdust programs.");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mdo_demo_compiles() {
        // Just ensure the demo compiles and runs without panicking
        let expander = DoNotationExpander::new();
        assert!(expander.get_monad("List").is_some());
        assert!(expander.get_monad("Maybe").is_some());
    }
}
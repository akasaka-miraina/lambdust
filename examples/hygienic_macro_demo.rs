//! Hygienic macro system demonstration
//! Shows how symbol collision prevention works in practice

use lambdust::macros::hygiene::{
    HygienicSymbol, SymbolId, MacroSite, EnvironmentId, ExpansionContext, SymbolGenerator
};
use lambdust::macros::hygiene::environment::HygienicEnvironment;

fn main() {
    println!("=== Hygienic Macro System Demonstration ===\n");
    
    test_symbol_generation();
    test_symbol_collision_prevention();
    test_expansion_context();
}

fn test_symbol_generation() {
    println!("1. Symbol Generation Test:");
    
    let mut generator = SymbolGenerator::new();
    let _env_id = EnvironmentId::new(1);
    
    // Generate unique symbols for common names
    let x_symbol1 = generator.generate_unique("x");
    let x_symbol2 = generator.generate_unique("x");
    let y_symbol = generator.generate_unique("y");
    
    println!("  Original name 'x' generates:");
    println!("    First:  {} (ID: {})", x_symbol1.name, x_symbol1.id);
    println!("    Second: {} (ID: {})", x_symbol2.name, x_symbol2.id);
    println!("  Original name 'y' generates: {} (ID: {})", y_symbol.name, y_symbol.id);
    
    // Verify uniqueness
    assert_ne!(x_symbol1.id, x_symbol2.id);
    assert_ne!(x_symbol1.id, y_symbol.id);
    
    println!("  ✓ All symbols have unique IDs\n");
}

fn test_symbol_collision_prevention() {
    println!("2. Symbol Collision Prevention:");
    
    let env_id = EnvironmentId::new(2);
    
    // Simulate macro-introduced symbol
    let macro_site = MacroSite::new(
        "when".to_string(),
        1,
        env_id,
    );
    
    let macro_temp = HygienicSymbol::new(
        "temp".to_string(),
        SymbolId::new(1001),
        macro_site,
    );
    
    // Simulate user-code symbol with same name
    let user_temp = HygienicSymbol::from_user_code(
        "temp".to_string(),
        SymbolId::new(1002),
        env_id,
    );
    
    println!("  Macro-introduced 'temp': {} (ID: {}, macro: {})", 
             macro_temp.name, macro_temp.id, macro_temp.definition_site.macro_name);
    println!("  User-code 'temp':        {} (ID: {}, source: {})", 
             user_temp.name, user_temp.id, user_temp.definition_site.macro_name);
    
    // Verify they are distinguishable
    assert_ne!(macro_temp.id, user_temp.id);
    assert_ne!(macro_temp.definition_site.macro_name, user_temp.definition_site.macro_name);
    
    println!("  ✓ Symbols with same name are distinguishable by ID and origin\n");
}

fn test_expansion_context() {
    println!("3. Expansion Context Tracking:");
    
    let _env_id = EnvironmentId::new(3);
    let hygiene_env = HygienicEnvironment::new();
    let mut context = ExpansionContext::new(hygiene_env.clone(), hygiene_env);
    
    println!("  Initial context:");
    println!("    Depth: {}", context.depth);
    println!("    Macro stack: {:?}", context.macro_stack);
    
    // Simulate entering macro expansion
    context.enter_macro("when".to_string()).unwrap();
    println!("  After entering 'when' macro:");
    println!("    Depth: {}", context.depth);
    println!("    Macro stack: {:?}", context.macro_stack);
    
    // Simulate nested expansion
    context.enter_macro("if".to_string()).unwrap();
    println!("  After entering nested 'if' macro:");
    println!("    Depth: {}", context.depth);
    println!("    Macro stack: {:?}", context.macro_stack);
    
    // Generate symbol in nested context
    let nested_symbol = context.symbol_generator.generate_unique("x");
    println!("  Generated symbol in nested context: {} (ID: {})", 
             nested_symbol.name, nested_symbol.id);
    
    // Exit expansions
    context.exit_macro();
    context.exit_macro();
    
    println!("  After exiting both macros:");
    println!("    Depth: {}", context.depth);
    println!("    Macro stack: {:?}", context.macro_stack);
    
    println!("  ✓ Expansion context properly tracks nesting and generates unique symbols\n");
}
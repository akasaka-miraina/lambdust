//! Advanced Symbol Renamer Demonstration
//!
//! This example showcases the sophisticated SymbolRenamer with 8 renaming strategies,
//! intelligent pattern matching, performance optimization, and comprehensive analytics.

use lambdust::macros::hygiene::{
    SymbolRenamer, RenamingStrategy, CustomRenamingRule, RenamingPattern,
    PatternMatcher, RenamingAction, DefaultAction, BuiltInPredicate, PredicateFunction,
    HygienicEnvironment, ExpansionContext
};
use lambdust::ast::Expr;

fn main() {
    println!("🎭 Advanced Symbol Renamer Demonstration");
    println!("════════════════════════════════════════════");

    // Demonstrate all 8 renaming strategies
    demonstrate_renaming_strategies();
    
    // Show advanced pattern matching capabilities
    demonstrate_pattern_matching();
    
    // Show intelligent renaming with machine learning-inspired heuristics
    demonstrate_intelligent_renaming();
    
    // Show scope-aware renaming
    demonstrate_scope_aware_renaming();
    
    // Show performance optimization
    demonstrate_performance_optimization();
    
    // Show context-sensitive renaming
    demonstrate_context_sensitive_renaming();
    
    // Show comprehensive analytics
    demonstrate_renaming_analytics();
    
    println!("\n🎉 Advanced Symbol Renamer demonstration complete!");
    println!("   ✨ 8 sophisticated renaming strategies implemented");
    println!("   🧠 Machine learning-inspired intelligent heuristics");
    println!("   🎯 Advanced pattern matching (Glob, Regex, Predicates)");
    println!("   📊 Scope-aware renaming with conflict detection");
    println!("   🚀 Performance optimization with comprehensive caching");
    println!("   📈 Real-time analytics and optimization recommendations");
}

fn demonstrate_renaming_strategies() {
    println!("\n1️⃣  Renaming Strategies Showcase");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let strategies = [
        ("Conservative", RenamingStrategy::Conservative),
        ("Rename All", RenamingStrategy::RenameAll),
        ("Rename Conflicts", RenamingStrategy::RenameConflicts),
        ("Intelligent", RenamingStrategy::Intelligent),
        ("Scope Aware", RenamingStrategy::ScopeAware),
        ("Performance Optimized", RenamingStrategy::PerformanceOptimized),
        ("Context Sensitive", RenamingStrategy::ContextSensitive),
    ];
    
    for (name, strategy) in &strategies {
        let mut renamer = SymbolRenamer::new(strategy.clone());
        let env = HygienicEnvironment::new();
        let mut context = ExpansionContext::new(env.clone(), env.clone());
        context.enter_macro("demo-macro".to_string()).unwrap();
        
        println!("   🔧 {} Strategy:", name);
        
        // Test with sample expressions
        let test_vars = ["temp", "var", "result", "iterator"];
        for var_name in &test_vars {
            let expr = Expr::Variable(var_name.to_string());
            match renamer.rename_symbols(&expr, &mut context, &env) {
                Ok(renamed_expr) => {
                    println!("      {} → {}", var_name, format_expression(&renamed_expr));
                }
                Err(e) => {
                    println!("      {} → Error: {}", var_name, e);
                }
            }
        }
        
        let stats = renamer.performance_stats();
        println!("      📊 Processed: {} symbols, Renamed: {} symbols", 
                 stats.symbols_processed, stats.symbols_renamed);
        println!();
    }
}

fn demonstrate_pattern_matching() {
    println!("2️⃣  Advanced Pattern Matching");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Create custom renaming rule with sophisticated patterns
    let custom_rule = CustomRenamingRule {
        patterns: vec![
            RenamingPattern {
                name_pattern: PatternMatcher::Glob("temp*".to_string()),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::AlwaysRename,
                priority: 10,
            },
            RenamingPattern {
                name_pattern: PatternMatcher::Predicate(PredicateFunction::BuiltIn(
                    BuiltInPredicate::IsTemporary
                )),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::CustomNaming("prefix-lambda".to_string()),
                priority: 8,
            },
            RenamingPattern {
                name_pattern: PatternMatcher::Predicate(PredicateFunction::BuiltIn(
                    BuiltInPredicate::LengthRange(1, 3)
                )),
                macro_context: None,
                scope_depth: None,
                type_constraint: None,
                action: RenamingAction::CustomNaming("suffix-unique".to_string()),
                priority: 5,
            },
        ],
        default_action: DefaultAction::CheckConflicts,
    };
    
    let mut renamer = SymbolRenamer::new(RenamingStrategy::Custom(custom_rule));
    let env = HygienicEnvironment::new();
    let mut context = ExpansionContext::new(env.clone(), env.clone());
    context.enter_macro("pattern-demo".to_string()).unwrap();
    
    println!("   🎯 Pattern Matching Examples:");
    
    let test_cases = [
        ("temp123", "Glob pattern: temp*"),
        ("t", "Short name: length 1-3"),
        ("x", "Temporary variable predicate"),
        ("temporary", "Temporary variable predicate"),
        ("normal_variable", "Default action: check conflicts"),
        ("tmp_var", "Temporary variable predicate"),
    ];
    
    for (var_name, description) in &test_cases {
        let expr = Expr::Variable(var_name.to_string());
        match renamer.rename_symbols(&expr, &mut context, &env) {
            Ok(renamed_expr) => {
                println!("      {} → {} ({})", 
                         var_name, 
                         format_expression(&renamed_expr),
                         description);
            }
            Err(e) => {
                println!("      {} → Error: {} ({})", var_name, e, description);
            }
        }
    }
    
    let stats = renamer.performance_stats();
    println!("   📊 Pattern matching efficiency: {:.1}% cache hit rate", 
             stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64 * 100.0);
    println!();
}

fn demonstrate_intelligent_renaming() {
    println!("3️⃣  Intelligent Renaming with ML-Inspired Heuristics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut renamer = SymbolRenamer::intelligent();
    let env = HygienicEnvironment::new();
    let mut context = ExpansionContext::new(env.clone(), env.clone());
    context.enter_macro("intelligent-demo".to_string()).unwrap();
    
    println!("   🧠 Learning from symbol usage patterns...");
    
    // Simulate learning phase: high-frequency symbols
    let high_freq_symbols = ["temp", "var", "result"];
    for _ in 0..8 {  // Simulate high frequency usage
        for symbol in &high_freq_symbols {
            let expr = Expr::Variable(symbol.to_string());
            let _ = renamer.rename_symbols(&expr, &mut context, &env);
        }
    }
    
    println!("   📈 After learning phase - intelligent decisions:");
    
    let test_symbols = [
        ("temp", "High-frequency symbol"),
        ("x", "Short name pattern"),
        ("temporary_variable", "Long descriptive name"),
        ("t", "Single character temporary"),
        ("unique_name", "Low-frequency symbol"),
    ];
    
    for (symbol, description) in &test_symbols {
        let expr = Expr::Variable(symbol.to_string());
        match renamer.rename_symbols(&expr, &mut context, &env) {
            Ok(renamed_expr) => {
                println!("      {} → {} ({})", 
                         symbol, 
                         format_expression(&renamed_expr),
                         description);
            }
            Err(e) => {
                println!("      {} → Error: {} ({})", symbol, e, description);
            }
        }
    }
    
    let stats = renamer.performance_stats();
    println!("   🎯 Intelligence metrics:");
    println!("      Symbols processed: {}", stats.symbols_processed);
    println!("      Symbols renamed: {} ({:.1}%)", 
             stats.symbols_renamed,
             stats.symbols_renamed as f64 / stats.symbols_processed as f64 * 100.0);
    println!();
}

fn demonstrate_scope_aware_renaming() {
    println!("4️⃣  Scope-Aware Renaming");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut renamer = SymbolRenamer::scope_aware();
    let env = HygienicEnvironment::new();
    let mut context = ExpansionContext::new(env.clone(), env.clone());
    
    println!("   🔍 Tracking scope depth and conflicts...");
    
    // Simulate nested macro expansion
    context.enter_macro("outer-macro".to_string()).unwrap();
    
    let symbols_outer = ["x", "y", "result"];
    println!("   📍 Outer scope (depth {}):", context.depth);
    for symbol in &symbols_outer {
        let expr = Expr::Variable(symbol.to_string());
        match renamer.rename_symbols(&expr, &mut context, &env) {
            Ok(renamed_expr) => {
                println!("      {} → {}", symbol, format_expression(&renamed_expr));
            }
            Err(e) => {
                println!("      {} → Error: {}", symbol, e);
            }
        }
    }
    
    // Enter inner scope
    context.enter_macro("inner-macro".to_string()).unwrap();
    
    let symbols_inner = ["x", "z", "temp"];  // 'x' conflicts with outer scope
    println!("   📍 Inner scope (depth {}):", context.depth);
    for symbol in &symbols_inner {
        let expr = Expr::Variable(symbol.to_string());
        match renamer.rename_symbols(&expr, &mut context, &env) {
            Ok(renamed_expr) => {
                println!("      {} → {}", symbol, format_expression(&renamed_expr));
            }
            Err(e) => {
                println!("      {} → Error: {}", symbol, e);
            }
        }
    }
    
    let stats = renamer.performance_stats();
    println!("   🎯 Scope analysis:");
    println!("      Conflicts detected: {}", stats.conflicts_detected);
    println!("      Symbols renamed: {}", stats.symbols_renamed);
    println!();
}

fn demonstrate_performance_optimization() {
    println!("5️⃣  Performance Optimization");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut renamer = SymbolRenamer::optimized();
    let env = HygienicEnvironment::new();
    let mut context = ExpansionContext::new(env.clone(), env.clone());
    context.enter_macro("perf-demo".to_string()).unwrap();
    
    println!("   🚀 Demonstrating cache optimization...");
    
    // Phase 1: Initial generation (cache misses)
    let test_symbols = ["temp", "var", "result", "iterator"];
    for i in 0..20 {
        let symbol = test_symbols[i % test_symbols.len()];
        let expr = Expr::Variable(symbol.to_string());
        let _ = renamer.rename_symbols(&expr, &mut context, &env);
    }
    
    let stats_phase1 = renamer.performance_stats();
    
    // Phase 2: More generation (cache hits expected)
    for i in 0..30 {
        let symbol = test_symbols[i % test_symbols.len()];
        let expr = Expr::Variable(symbol.to_string());
        let _ = renamer.rename_symbols(&expr, &mut context, &env);
    }
    
    let stats_phase2 = renamer.performance_stats();
    
    println!("   📊 Performance Analysis:");
    println!("      Total symbols processed: {}", stats_phase2.symbols_processed);
    println!("      Cache hits: {}", stats_phase2.cache_hits);
    println!("      Cache misses: {}", stats_phase2.cache_misses);
    println!("      Cache hit rate: {:.1}%", 
             stats_phase2.cache_hits as f64 / (stats_phase2.cache_hits + stats_phase2.cache_misses) as f64 * 100.0);
    println!("      Processing time: {:.2}ms", 
             stats_phase2.total_processing_time_ns as f64 / 1_000_000.0);
    println!();
}

fn demonstrate_context_sensitive_renaming() {
    println!("6️⃣  Context-Sensitive Renaming");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut renamer = SymbolRenamer::new(RenamingStrategy::ContextSensitive);
    let env = HygienicEnvironment::new();
    let mut context = ExpansionContext::new(env.clone(), env.clone());
    
    let macro_contexts = [
        ("let", "binding construct"),
        ("define", "definition form"),
        ("lambda", "procedure creation"),
        ("for-each", "iteration construct"),
    ];
    
    println!("   🎭 Context-aware symbol handling:");
    
    for (macro_name, description) in &macro_contexts {
        context.enter_macro(macro_name.to_string()).unwrap();
        
        println!("   📋 In {} ({}):", macro_name, description);
        
        let test_symbols = ["x", "result", "temp"];
        for symbol in &test_symbols {
            let expr = Expr::Variable(symbol.to_string());
            match renamer.rename_symbols(&expr, &mut context, &env) {
                Ok(renamed_expr) => {
                    println!("      {} → {}", symbol, format_expression(&renamed_expr));
                }
                Err(e) => {
                    println!("      {} → Error: {}", symbol, e);
                }
            }
        }
        
        context.exit_macro();
        println!();
    }
    
    let stats = renamer.performance_stats();
    println!("   🎯 Context sensitivity metrics:");
    println!("      Symbols processed: {}", stats.symbols_processed);
    println!("      Context-driven renames: {}", stats.symbols_renamed);
}

fn demonstrate_renaming_analytics() {
    println!("\n7️⃣  Comprehensive Renaming Analytics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let strategies = [
        ("Intelligent", RenamingStrategy::Intelligent),
        ("Performance Optimized", RenamingStrategy::PerformanceOptimized),
        ("Scope Aware", RenamingStrategy::ScopeAware),
    ];
    
    for (name, strategy) in &strategies {
        let mut renamer = SymbolRenamer::new(strategy.clone());
        let env = HygienicEnvironment::new();
        let mut context = ExpansionContext::new(env.clone(), env.clone());
        context.enter_macro("analytics-demo".to_string()).unwrap();
        
        // Simulate realistic usage
        let symbols = ["temp", "var", "result", "x", "y", "iterator", "accumulator"];
        for i in 0..50 {
            let symbol = symbols[i % symbols.len()];
            let expr = Expr::Variable(symbol.to_string());
            let _ = renamer.rename_symbols(&expr, &mut context, &env);
        }
        
        let stats = renamer.performance_stats();
        
        println!("   📈 {} Analytics:", name);
        println!("      Symbols processed: {}", stats.symbols_processed);
        println!("      Symbols renamed: {} ({:.1}%)", 
                 stats.symbols_renamed,
                 stats.symbols_renamed as f64 / stats.symbols_processed as f64 * 100.0);
        println!("      Conflicts detected: {}", stats.conflicts_detected);
        println!("      Processing time: {:.2}ms", 
                 stats.total_processing_time_ns as f64 / 1_000_000.0);
        
        if stats.cache_hits + stats.cache_misses > 0 {
            println!("      Cache efficiency: {:.1}%", 
                     stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64 * 100.0);
        }
        
        println!();
    }
}

fn format_expression(expr: &Expr) -> String {
    match expr {
        Expr::Variable(name) => name.clone(),
        Expr::HygienicVariable(symbol) => symbol.unique_name(),
        Expr::List(exprs) => {
            let formatted: Vec<String> = exprs.iter().map(format_expression).collect();
            format!("({})", formatted.join(" "))
        }
        _ => format!("{:?}", expr),
    }
}
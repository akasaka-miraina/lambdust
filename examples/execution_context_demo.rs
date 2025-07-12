//! ExecutionContext Demo - Evaluator-Executor Communication Bridge
//!
//! This example demonstrates the ExecutionContext system that serves as
//! the information bridge between the Evaluator (static analysis) and
//! the Executor (dynamic optimization and execution).

use lambdust::{
    ast::{Expr, Literal},
    environment::Environment,
    evaluator::{
        Continuation, ExecutionContext, ExecutionContextBuilder, ExecutionPriority,
        StaticCallPattern, VariableUsage, VariableTypeHint,
    },
    lexer::SchemeNumber,
    value::Value,
};
use std::collections::HashMap;
use std::rc::Rc;

fn main() {
    println!("🏗️ ExecutionContext Demo: Evaluator-Executor Communication Bridge");
    println!("================================================================");

    // 1️⃣ Basic ExecutionContext creation
    println!("\n1️⃣ Creating basic ExecutionContext...");
    demo_basic_creation();

    // 2️⃣ Static analysis integration
    println!("\n2️⃣ Demonstrating static analysis integration...");
    demo_static_analysis();

    // 3️⃣ Optimization hints derivation
    println!("\n3️⃣ Testing optimization hints derivation...");
    demo_optimization_hints();

    // 4️⃣ Builder pattern usage
    println!("\n4️⃣ Using ExecutionContextBuilder pattern...");
    demo_builder_pattern();

    // 5️⃣ Constant binding optimization
    println!("\n5️⃣ Demonstrating constant binding optimization...");
    demo_constant_binding();

    // 6️⃣ Complex expression analysis
    println!("\n6️⃣ Analyzing complex expression...");
    demo_complex_analysis();

    println!("\n🎉 ExecutionContext demo completed successfully!");
    println!("✨ Key capabilities demonstrated:");
    println!("   • Static analysis result aggregation");
    println!("   • Optimization hint derivation");
    println!("   • Builder pattern for fluent construction");
    println!("   • Constant binding optimization");
    println!("   • Complex expression analysis");
    println!("   • Evaluator-Executor communication bridge");
}

fn demo_basic_creation() {
    let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;

    let context = ExecutionContext::new(expr, env, cont);

    println!("  ✅ Created ExecutionContext");
    println!("     Complexity score: {}", context.static_analysis.complexity_score);
    println!("     Has tail calls: {}", context.static_analysis.has_tail_calls);
    println!("     Has loops: {}", context.static_analysis.has_loops);
    println!("     Is pure: {}", context.static_analysis.is_pure);
}

fn demo_static_analysis() {
    let expr = Expr::Variable("factorial".to_string());
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;

    let mut context = ExecutionContext::new(expr, env, cont);

    // Simulate static analysis results
    context.static_analysis.complexity_score = 65;
    context.static_analysis.has_tail_calls = true;
    context.static_analysis.has_loops = false;
    context.static_analysis.is_pure = true;

    // Add call patterns
    context.static_analysis.call_patterns.push(StaticCallPattern::Recursive {
        depth_hint: Some(10),
    });
    context.static_analysis.call_patterns.push(StaticCallPattern::TailRecursive);

    // Add variable usage information
    let mut variable_usage = HashMap::new();
    variable_usage.insert(
        "n".to_string(),
        VariableUsage {
            reference_count: 3,
            is_modified: false,
            escapes_scope: false,
            type_hint: Some(VariableTypeHint::Number),
        },
    );
    context.static_analysis.variable_usage = variable_usage;

    // Add memory estimates
    context.static_analysis.memory_estimates.stack_usage = 1024;
    context.static_analysis.memory_estimates.heap_allocations = 512;
    context.static_analysis.memory_estimates.benefits_from_pooling = true;

    println!("  ✅ Static analysis configured");
    println!("     Complexity: {}", context.static_analysis.complexity_score);
    println!("     Call patterns: {}", context.static_analysis.call_patterns.len());
    println!("     Variable usage entries: {}", context.static_analysis.variable_usage.len());
    println!("     Stack usage estimate: {} bytes", context.static_analysis.memory_estimates.stack_usage);
}

fn demo_optimization_hints() {
    let expr = Expr::Literal(Literal::Boolean(true));
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;

    let mut context = ExecutionContext::new(expr, env, cont);

    // Set up for optimization
    context.static_analysis.complexity_score = 80;
    context.static_analysis.has_loops = true;
    context.static_analysis.has_tail_calls = true;
    context.static_analysis.is_pure = false;

    // Add some call patterns for hot path detection
    context.static_analysis.call_patterns.push(StaticCallPattern::Loop {
        estimated_iterations: Some(1000),
    });

    // Derive optimization hints
    context.derive_optimization_hints();

    println!("  ✅ Optimization hints derived");
    println!("     Optimization level: {:?}", context.optimization_hints.optimization_level);
    println!("     JIT beneficial: {}", context.optimization_hints.jit_beneficial);
    println!("     Use tail call optimization: {}", context.optimization_hints.use_tail_call_optimization);
    println!("     Use continuation pooling: {}", context.optimization_hints.use_continuation_pooling);
    println!("     Hot path indicators: {}", context.optimization_hints.hot_path_indicators.len());
    println!("     Optimization strategies: {}", context.optimization_hints.optimization_strategies.len());
}

fn demo_builder_pattern() {
    let expr = Expr::Variable("loop-counter".to_string());
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;

    let context = ExecutionContextBuilder::new(expr, env, cont)
        .with_complexity_score(90)
        .with_tail_calls(false)
        .with_loops(true)
        .with_purity(true)
        .with_priority(ExecutionPriority::High)
        .add_call_pattern(StaticCallPattern::Loop {
            estimated_iterations: Some(500),
        })
        .add_constant_binding(
            "max-iterations".to_string(),
            Value::Number(SchemeNumber::Integer(1000)),
        )
        .build();

    println!("  ✅ ExecutionContext built with fluent API");
    println!("     Context ID: {}", context.execution_metadata.context_id);
    println!("     Priority: {:?}", context.execution_metadata.priority);
    println!("     Complexity: {}", context.static_analysis.complexity_score);
    println!("     Constant bindings: {}", context.constant_bindings.len());
    println!("     Should optimize: {}", context.should_optimize());
    println!("     Should use JIT: {}", context.should_use_jit());
}

fn demo_constant_binding() {
    let expr = Expr::Variable("pi".to_string());
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;

    let mut context = ExecutionContext::new(expr, env, cont);

    // Add some constant bindings (static optimization results)
    context.add_constant_binding("pi".to_string(), Value::Number(SchemeNumber::Real(3.14159)));
    context.add_constant_binding("e".to_string(), Value::Number(SchemeNumber::Real(2.71828)));
    context.add_constant_binding("version".to_string(), Value::String("3.0".to_string()));

    println!("  ✅ Constant bindings configured");
    println!("     Total constants: {}", context.constant_bindings.len());

    // Demonstrate constant lookup
    if let Some(pi_value) = context.get_constant_binding("pi") {
        println!("     π = {}", pi_value);
    }
    if let Some(e_value) = context.get_constant_binding("e") {
        println!("     e = {}", e_value);
    }
    if let Some(version) = context.get_constant_binding("version") {
        println!("     version = {}", version);
    }

    println!("     Unknown constant lookup: {:?}", context.get_constant_binding("unknown"));
}

fn demo_complex_analysis() {
    let expr = Expr::List(vec![
        Expr::Variable("map".to_string()),
        Expr::Variable("square".to_string()),
        Expr::Variable("large-list".to_string()),
    ]);
    let env = Rc::new(Environment::new());
    let cont = Continuation::Identity;

    let context = ExecutionContextBuilder::new(expr, env, cont)
        .with_complexity_score(95)
        .with_tail_calls(false)
        .with_loops(false)
        .with_purity(true)
        .with_priority(ExecutionPriority::Critical)
        .add_call_pattern(StaticCallPattern::HigherOrder)
        .add_call_pattern(StaticCallPattern::Builtin {
            name: "map".to_string(),
            arity: Some(2),
        })
        .build();

    println!("  ✅ Complex expression analyzed");
    println!("     Expression type: Higher-order function application");
    println!("     Complexity score: {} (very high)", context.static_analysis.complexity_score);
    println!("     Optimization level: {:?}", context.optimization_hints.optimization_level);
    println!("     JIT beneficial: {}", context.optimization_hints.jit_beneficial);
    println!("     Memory usage estimate: {} bytes", context.estimated_memory_usage());
    println!("     Critical priority: {:?}", context.execution_metadata.priority);
    
    // Show call pattern analysis
    println!("     Call patterns detected:");
    for (i, pattern) in context.static_analysis.call_patterns.iter().enumerate() {
        println!("       {}. {:?}", i + 1, pattern);
    }

    // Show optimization strategies
    println!("     Recommended optimization strategies:");
    for (i, strategy) in context.optimization_hints.optimization_strategies.iter().enumerate() {
        println!("       {}. {:?}", i + 1, strategy);
    }
}
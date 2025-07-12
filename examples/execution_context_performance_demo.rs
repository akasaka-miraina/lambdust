//! ExecutionContext Performance Demonstration
//!
//! This example demonstrates the performance benefits of the ExecutionContext-based
//! evaluation system by comparing different optimization levels and measuring
//! the effectiveness of static analysis-driven dynamic optimization.

use lambdust::{
    ast::{Expr, Literal},
    environment::Environment,
    evaluator::{
        Continuation, EvaluatorInterface, Evaluator, RuntimeExecutor,
        EvaluationMode, EvaluationConfig, RuntimeOptimizationLevel,
        ExecutionContextBuilder, ExecutionPriority, StaticCallPattern,
    },
    lexer::SchemeNumber,
    value::Value,
};
use std::rc::Rc;
use std::time::Instant;

fn main() {
    println!("🚀 ExecutionContext Performance Demonstration");
    println!("==============================================");
    
    // 1️⃣ Simple arithmetic performance comparison
    println!("\n1️⃣ Simple Arithmetic Performance Comparison");
    demo_simple_arithmetic_performance();
    
    // 2️⃣ Complex expression performance analysis
    println!("\n2️⃣ Complex Expression Performance Analysis");
    demo_complex_expression_performance();
    
    // 3️⃣ Optimization level impact measurement
    println!("\n3️⃣ Optimization Level Impact Measurement");
    demo_optimization_level_impact();
    
    // 4️⃣ ExecutionContext overhead analysis
    println!("\n4️⃣ ExecutionContext Overhead Analysis");
    demo_execution_context_overhead();
    
    // 5️⃣ Auto mode intelligence demonstration
    println!("\n5️⃣ Auto Mode Intelligence Demonstration");
    demo_auto_mode_intelligence();
    
    // 6️⃣ Verification mode performance cost
    println!("\n6️⃣ Verification Mode Performance Cost");
    demo_verification_mode_cost();
    
    println!("\n🎉 Performance demonstration completed!");
    println!("✨ Key insights:");
    println!("   • ExecutionContext enables intelligent optimization selection");
    println!("   • Static analysis guides dynamic optimization effectively");
    println!("   • Auto mode provides optimal performance without manual tuning");
    println!("   • Verification mode ensures correctness with acceptable overhead");
}

fn demo_simple_arithmetic_performance() {
    let expr = Expr::List(vec![
        Expr::Variable("+".to_string()),
        Expr::List(vec![
            Expr::Variable("*".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(123))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(456))),
        ]),
        Expr::List(vec![
            Expr::Variable("/".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1000))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(8))),
        ]),
    ]);
    
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    let iterations = 1000;
    
    // Test semantic evaluation
    let semantic_time = measure_evaluation_time(&expr, &env, &cont, EvaluationMode::Semantic, iterations);
    
    // Test runtime evaluation
    let runtime_time = measure_evaluation_time(
        &expr, &env, &cont, 
        EvaluationMode::Runtime(RuntimeOptimizationLevel::Balanced),
        iterations
    );
    
    // Test auto mode
    let auto_time = measure_evaluation_time(&expr, &env, &cont, EvaluationMode::Auto, iterations);
    
    println!("  📊 Results ({} iterations):", iterations);
    println!("     Semantic mode: {:.2} μs/eval", semantic_time as f64 / iterations as f64);
    println!("     Runtime mode:  {:.2} μs/eval", runtime_time as f64 / iterations as f64);
    println!("     Auto mode:     {:.2} μs/eval", auto_time as f64 / iterations as f64);
    
    if runtime_time > 0 {
        let speedup = semantic_time as f64 / runtime_time as f64;
        println!("     Runtime speedup: {:.2}x", speedup);
    }
    
    if auto_time > 0 {
        let auto_speedup = semantic_time as f64 / auto_time as f64;
        println!("     Auto speedup: {:.2}x", auto_speedup);
    }
}

fn demo_complex_expression_performance() {
    // Create a more complex expression that benefits from optimization
    let expr = Expr::List(vec![
        Expr::Variable("map".to_string()),
        Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![Expr::Variable("x".to_string())]),
            Expr::List(vec![
                Expr::Variable("*".to_string()),
                Expr::Variable("x".to_string()),
                Expr::Variable("x".to_string()),
            ]),
        ]),
        Expr::List(vec![
            Expr::Variable("quote".to_string()),
            Expr::List((1..=20).map(|i| 
                Expr::Literal(Literal::Number(SchemeNumber::Integer(i)))
            ).collect()),
        ]),
    ]);
    
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    let iterations = 100;
    
    // Generate ExecutionContext and analyze
    let mut evaluator = Evaluator::new();
    if let Ok(context) = evaluator.create_execution_context(expr.clone(), env.clone(), cont.clone()) {
        println!("  🔍 Static Analysis Results:");
        println!("     Complexity score: {}", context.static_analysis.complexity_score);
        println!("     Has loops: {}", context.static_analysis.has_loops);
        println!("     Call patterns: {}", context.static_analysis.call_patterns.len());
        println!("     Should optimize: {}", context.should_optimize());
        println!("     Should use JIT: {}", context.should_use_jit());
        println!("     Memory estimate: {} bytes", context.estimated_memory_usage());
    }
    
    // Performance comparison
    let semantic_time = measure_evaluation_time(&expr, &env, &cont, EvaluationMode::Semantic, iterations);
    let balanced_time = measure_evaluation_time(
        &expr, &env, &cont,
        EvaluationMode::Runtime(RuntimeOptimizationLevel::Balanced),
        iterations
    );
    let aggressive_time = measure_evaluation_time(
        &expr, &env, &cont,
        EvaluationMode::Runtime(RuntimeOptimizationLevel::Aggressive),
        iterations
    );
    
    println!("  📊 Performance Results ({} iterations):", iterations);
    println!("     Semantic:           {:.2} μs/eval", semantic_time as f64 / iterations as f64);
    println!("     Balanced Runtime:   {:.2} μs/eval", balanced_time as f64 / iterations as f64);
    println!("     Aggressive Runtime: {:.2} μs/eval", aggressive_time as f64 / iterations as f64);
    
    if balanced_time > 0 && aggressive_time > 0 {
        let balanced_speedup = semantic_time as f64 / balanced_time as f64;
        let aggressive_speedup = semantic_time as f64 / aggressive_time as f64;
        println!("     Balanced speedup:   {:.2}x", balanced_speedup);
        println!("     Aggressive speedup: {:.2}x", aggressive_speedup);
    }
}

fn demo_optimization_level_impact() {
    let expr = Expr::List(vec![
        Expr::Variable("fold".to_string()),
        Expr::Variable("+".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(0))),
        Expr::List(vec![
            Expr::Variable("quote".to_string()),
            Expr::List((1..=100).map(|i| 
                Expr::Literal(Literal::Number(SchemeNumber::Integer(i)))
            ).collect()),
        ]),
    ]);
    
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    let iterations = 50;
    
    let optimization_levels = vec![
        ("None", RuntimeOptimizationLevel::None),
        ("Conservative", RuntimeOptimizationLevel::Conservative),
        ("Balanced", RuntimeOptimizationLevel::Balanced),
        ("Aggressive", RuntimeOptimizationLevel::Aggressive),
    ];
    
    println!("  📊 Optimization Level Impact ({} iterations):", iterations);
    
    let semantic_time = measure_evaluation_time(&expr, &env, &cont, EvaluationMode::Semantic, iterations);
    println!("     Semantic (baseline): {:.2} μs/eval", semantic_time as f64 / iterations as f64);
    
    for (name, level) in optimization_levels {
        let runtime_time = measure_evaluation_time(
            &expr, &env, &cont,
            EvaluationMode::Runtime(level),
            iterations
        );
        
        let speedup = if runtime_time > 0 {
            semantic_time as f64 / runtime_time as f64
        } else {
            0.0
        };
        
        println!("     {:12}: {:.2} μs/eval ({}x speedup)", 
                 name, 
                 runtime_time as f64 / iterations as f64,
                 format!("{:.2}", speedup));
    }
}

fn demo_execution_context_overhead() {
    let expr = Expr::List(vec![
        Expr::Variable("*".to_string()),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        Expr::Literal(Literal::Number(SchemeNumber::Integer(24))),
    ]);
    
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    let iterations = 10000;
    
    println!("  🔬 ExecutionContext Overhead Analysis ({} iterations):", iterations);
    
    // Measure ExecutionContext generation overhead
    let mut evaluator = Evaluator::new();
    let context_start = Instant::now();
    
    for _ in 0..iterations {
        let _ = evaluator.create_execution_context(expr.clone(), env.clone(), cont.clone());
    }
    
    let context_time = context_start.elapsed().as_micros();
    println!("     ExecutionContext generation: {:.3} μs/call", context_time as f64 / iterations as f64);
    
    // Measure full evaluation with ExecutionContext
    let unified_time = measure_evaluation_time(&expr, &env, &cont, EvaluationMode::Auto, iterations);
    println!("     Full unified evaluation: {:.3} μs/call", unified_time as f64 / iterations as f64);
    
    // Calculate overhead percentage
    let semantic_time = measure_evaluation_time(&expr, &env, &cont, EvaluationMode::Semantic, iterations);
    if semantic_time > 0 {
        let overhead_percent = ((unified_time - semantic_time) as f64 / semantic_time as f64) * 100.0;
        println!("     Overhead vs semantic: {:.1}%", overhead_percent);
    }
}

fn demo_auto_mode_intelligence() {
    println!("  🧠 Auto Mode Intelligence Demonstration:");
    
    let test_cases = vec![
        ("Simple literal", 
         Expr::Literal(Literal::Number(SchemeNumber::Integer(42)))),
        
        ("Simple arithmetic", 
         Expr::List(vec![
             Expr::Variable("+".to_string()),
             Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
             Expr::Literal(Literal::Number(SchemeNumber::Integer(20))),
         ])),
        
        ("Complex nested", 
         Expr::List(vec![
             Expr::Variable("*".to_string()),
             Expr::List(vec![
                 Expr::Variable("+".to_string()),
                 Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                 Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
             ]),
             Expr::List(vec![
                 Expr::Variable("-".to_string()),
                 Expr::Literal(Literal::Number(SchemeNumber::Integer(10))),
                 Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
             ]),
         ])),
        
        ("Higher-order function",
         Expr::List(vec![
             Expr::Variable("map".to_string()),
             Expr::List(vec![
                 Expr::Variable("lambda".to_string()),
                 Expr::List(vec![Expr::Variable("x".to_string())]),
                 Expr::List(vec![
                     Expr::Variable("+".to_string()),
                     Expr::Variable("x".to_string()),
                     Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                 ]),
             ]),
             Expr::List(vec![
                 Expr::Variable("quote".to_string()),
                 Expr::List(vec![
                     Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
                     Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
                     Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
                 ]),
             ]),
         ])),
    ];
    
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    let mut evaluator = Evaluator::new();
    
    for (name, expr) in test_cases {
        if let Ok(context) = evaluator.create_execution_context(expr.clone(), env.clone(), cont.clone()) {
            let mut interface = EvaluatorInterface::new();
            let config = EvaluationConfig {
                mode: EvaluationMode::Auto,
                verify_correctness: false,
                monitor_performance: false,
                fallback_to_semantic: true,
                verification_timeout_ms: 1000,
                verification_config: Default::default(),
            };
            interface.set_config(config);
            
            if let Ok(result) = interface.eval_with_execution_context(expr, env.clone(), cont.clone()) {
                println!("     {:18}: complexity={:2}, mode={:?}, should_optimize={}, should_jit={}", 
                         name,
                         context.static_analysis.complexity_score,
                         result.mode_used,
                         context.should_optimize(),
                         context.should_use_jit());
            }
        }
    }
}

fn demo_verification_mode_cost() {
    let expr = Expr::List(vec![
        Expr::Variable("abs".to_string()),
        Expr::List(vec![
            Expr::Variable("-".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(100))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(200))),
        ]),
    ]);
    
    let env = Rc::new(Environment::with_builtins());
    let cont = Continuation::Identity;
    let iterations = 500;
    
    println!("  🔍 Verification Mode Performance Cost ({} iterations):", iterations);
    
    // Test different modes
    let semantic_time = measure_evaluation_time(&expr, &env, &cont, EvaluationMode::Semantic, iterations);
    let runtime_time = measure_evaluation_time(
        &expr, &env, &cont,
        EvaluationMode::Runtime(RuntimeOptimizationLevel::Balanced),
        iterations
    );
    let verification_time = measure_evaluation_time(&expr, &env, &cont, EvaluationMode::Verification, iterations);
    
    println!("     Semantic only:    {:.2} μs/eval", semantic_time as f64 / iterations as f64);
    println!("     Runtime only:     {:.2} μs/eval", runtime_time as f64 / iterations as f64);
    println!("     Verification:     {:.2} μs/eval", verification_time as f64 / iterations as f64);
    
    if semantic_time > 0 && runtime_time > 0 {
        let verification_overhead = verification_time as f64 / (semantic_time + runtime_time) as f64;
        println!("     Verification overhead: {:.2}x vs individual modes", verification_overhead);
        
        let vs_semantic = verification_time as f64 / semantic_time as f64;
        println!("     Verification vs semantic: {:.2}x", vs_semantic);
    }
    
    // Test verification accuracy by checking results
    let mut interface = EvaluatorInterface::new();
    let config = EvaluationConfig {
        mode: EvaluationMode::Verification,
        verify_correctness: true,
        monitor_performance: true,
        fallback_to_semantic: true,
        verification_timeout_ms: 1000,
        verification_config: Default::default(),
    };
    interface.set_config(config);
    
    if let Ok(result) = interface.eval_with_execution_context(expr, env, cont) {
        if let Some(verification) = result.verification_result {
            println!("     Verification accuracy: {}",
                     if verification.semantic_equivalence.unwrap_or(false) { "✅ PASS" } else { "❌ FAIL" });
        }
    }
}

fn measure_evaluation_time(
    expr: &Expr,
    env: &Rc<Environment>,
    cont: &Continuation,
    mode: EvaluationMode,
    iterations: usize,
) -> u128 {
    let mut interface = EvaluatorInterface::new();
    let config = EvaluationConfig {
        mode,
        verify_correctness: false,
        monitor_performance: false,
        fallback_to_semantic: true,
        verification_timeout_ms: 1000,
        verification_config: Default::default(),
    };
    interface.set_config(config);
    
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = interface.eval_with_execution_context(
            expr.clone(),
            env.clone(),
            cont.clone(),
        );
    }
    
    start.elapsed().as_micros()
}
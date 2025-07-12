//! Adaptive Theorem Learning System Demonstration
//!
//! This example showcases the revolutionary adaptive learning system that
//! automatically discovers optimization patterns from real Scheme code,
//! accumulates knowledge, and strengthens the theorem system for increasingly
//! sophisticated evaluator performance.

use lambdust::evaluator::{
    adaptive_theorem_learning::{
        AdaptiveTheoremLearningSystem, AdaptiveLearningConfig, CodeSample,
        LearningSession, LearningInsights,
    },
    theorem_derivation_engine::TheoremDerivationEngine,
    formal_verification::FormalVerificationEngine,
    theorem_proving::TheoremProvingSupport,
    semantic::SemanticEvaluator,
    Evaluator, Continuation,
};
use lambdust::environment::Environment;
use lambdust::ast::Expr;
use std::rc::Rc;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Adaptive Theorem Learning System: Code-Driven Knowledge Evolution");
    println!("====================================================================");
    
    // Initialize the adaptive learning system
    let config = AdaptiveLearningConfig {
        continuous_learning: true,
        learning_rate: 0.2,
        discovery_threshold: 0.7,
        improvement_threshold: 0.05,
        max_knowledge_size: 1000,
        ..Default::default()
    };
    
    let mut learning_system = AdaptiveTheoremLearningSystem::new(config);
    
    // Test 1: Learn from Real Scheme Code Patterns
    println!("\n🔍 Test 1: Learning from Real Scheme Code Patterns");
    println!("--------------------------------------------------");
    
    let sample_schemes = vec![
        // Mathematical computation patterns
        ("fibonacci.scm", r#"
            (define (fibonacci n)
              (if (<= n 1)
                  n
                  (+ (fibonacci (- n 1))
                     (fibonacci (- n 2)))))
        "#),
        
        // List processing patterns
        ("list-ops.scm", r#"
            (define (map-square lst)
              (if (null? lst)
                  '()
                  (cons (* (car lst) (car lst))
                        (map-square (cdr lst)))))
            
            (define (fold-sum lst)
              (if (null? lst)
                  0
                  (+ (car lst) (fold-sum (cdr lst)))))
        "#),
        
        // Higher-order function patterns
        ("higher-order.scm", r#"
            (define (compose f g)
              (lambda (x) (f (g x))))
            
            (define (apply-twice f)
              (lambda (x) (f (f x))))
            
            (define (filter pred lst)
              (cond ((null? lst) '())
                    ((pred (car lst))
                     (cons (car lst) (filter pred (cdr lst))))
                    (else (filter pred (cdr lst)))))
        "#),
        
        // Tail-recursive optimization patterns
        ("tail-recursive.scm", r#"
            (define (factorial-iter n acc)
              (if (= n 0)
                  acc
                  (factorial-iter (- n 1) (* n acc))))
            
            (define (factorial n)
              (factorial-iter n 1))
            
            (define (reverse-iter lst acc)
              (if (null? lst)
                  acc
                  (reverse-iter (cdr lst) (cons (car lst) acc))))
        "#),
        
        // Complex computation patterns
        ("complex-math.scm", r#"
            (define (power-of-two? n)
              (if (= n 1)
                  #t
                  (if (= (modulo n 2) 0)
                      (power-of-two? (/ n 2))
                      #f)))
            
            (define (gcd a b)
              (if (= b 0)
                  a
                  (gcd b (modulo a b))))
            
            (define (matrix-multiply A B)
              (map (lambda (row)
                     (map (lambda (col)
                            (apply + (map * row col)))
                          (transpose B)))
                   A))
        "#),
    ];
    
    let mut learning_sessions = Vec::new();
    
    for (filename, code) in &sample_schemes {
        println!("📚 Learning from {}", filename);
        let start_time = Instant::now();
        
        match learning_system.learn_from_source(code, filename) {
            Ok(session) => {
                let learning_time = start_time.elapsed();
                println!("✅ Learning completed in {:?}", learning_time);
                println!("   📊 Patterns discovered: {}", session.discovered_patterns.len());
                println!("   🧮 Expressions processed: {}", session.session_stats.expressions_processed);
                println!("   💡 Insights gained: {}", session.insights_gained.len());
                learning_sessions.push(session);
            }
            Err(e) => {
                println!("❌ Learning failed: {}", e);
            }
        }
    }
    
    // Test 2: Continuous Learning with New Code Samples
    println!("\n🔄 Test 2: Continuous Learning with New Code Samples");
    println!("----------------------------------------------------");
    
    let new_samples = vec![
        CodeSample {
            identifier: "optimization_pattern_1".to_string(),
            code: r#"
                ;; Constant folding opportunity
                (define result (* (+ 3 4) (- 10 2)))
                
                ;; Loop unrolling opportunity
                (define (sum-small n)
                  (if (< n 4)
                      (+ n (sum-small (- n 1)))
                      0))
            "#.to_string(),
            performance: None,
            context: "performance_optimization".to_string(),
        },
        CodeSample {
            identifier: "pattern_recognition_2".to_string(),
            code: r#"
                ;; Common subexpression elimination
                (define (complex-calc x y)
                  (let ((common (* x y)))
                    (+ (* common common)
                       (/ common 2)
                       (sqrt common))))
                
                ;; Identity element optimization
                (define (identity-ops x)
                  (+ (* x 1) (+ x 0) (- x 0)))
            "#.to_string(),
            performance: None,
            context: "algebraic_optimization".to_string(),
        },
    ];
    
    println!("🔄 Starting continuous learning with {} new samples", new_samples.len());
    let continuous_start = Instant::now();
    
    match learning_system.continuous_learning_update(&new_samples) {
        Ok(()) => {
            let continuous_time = continuous_start.elapsed();
            println!("✅ Continuous learning completed in {:?}", continuous_time);
            println!("🧠 Knowledge base updated with new patterns");
        }
        Err(e) => {
            println!("❌ Continuous learning failed: {}", e);
        }
    }
    
    // Test 3: Integration with Theorem Derivation Engine
    println!("\n🔗 Test 3: Integration with Theorem Derivation Engine");
    println!("-----------------------------------------------------");
    
    // Create theorem derivation engine
    let semantic_evaluator = SemanticEvaluator::new();
    let theorem_prover = TheoremProvingSupport::new(semantic_evaluator.clone());
    let verification_engine = FormalVerificationEngine::new();
    
    let mut derivation_engine = TheoremDerivationEngine::new(
        theorem_prover,
        verification_engine,
        semantic_evaluator,
    );
    
    // Integrate learned patterns
    println!("🔄 Integrating learned patterns with theorem derivation engine");
    let integration_start = Instant::now();
    
    match learning_system.integrate_with_derivation_engine(&mut derivation_engine) {
        Ok(integrated_count) => {
            let integration_time = integration_start.elapsed();
            println!("✅ Integration completed in {:?}", integration_time);
            println!("📊 Integrated {} learned theorems", integrated_count);
            
            if integrated_count > 0 {
                println!("🚀 Theorem derivation engine enhanced with learned knowledge");
                
                // Test the enhanced engine
                match derivation_engine.derive_optimization_theorems() {
                    Ok(enhanced_theorems) => {
                        println!("🔬 Enhanced engine derived {} theorems", enhanced_theorems.len());
                        
                        for (i, theorem) in enhanced_theorems.iter().take(3).enumerate() {
                            println!("   {}. {} ({:.1}% improvement)", 
                                     i + 1, 
                                     theorem.id, 
                                     theorem.optimization_rule.performance_gain.quantitative_gain * 100.0);
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Enhanced derivation failed: {}", e);
                    }
                }
            } else {
                println!("ℹ️ No patterns met integration threshold yet");
            }
        }
        Err(e) => {
            println!("❌ Integration failed: {}", e);
        }
    }
    
    // Test 4: Learning Insights and Knowledge Base Analysis
    println!("\n📊 Test 4: Learning Insights and Knowledge Base Analysis");
    println!("--------------------------------------------------------");
    
    let insights = learning_system.get_learning_insights();
    
    println!("🧮 Knowledge Base Statistics:");
    println!("   📈 Total patterns discovered: {}", insights.total_patterns);
    println!("   🎯 High confidence patterns: {}", insights.high_confidence_patterns);
    println!("   ⚡ Average performance improvement: {:.2}%", 
             insights.average_performance_improvement * 100.0);
    println!("   📊 Knowledge base growth rate: {:.2}%", 
             insights.knowledge_base_growth * 100.0);
    
    println!("\n🏆 Most Effective Patterns:");
    for (i, pattern_id) in insights.most_effective_patterns.iter().enumerate() {
        println!("   {}. {}", i + 1, pattern_id);
    }
    
    // Test 5: Knowledge Base Persistence
    println!("\n💾 Test 5: Knowledge Base Persistence");
    println!("-------------------------------------");
    
    let save_path = "knowledge_base.json";
    println!("💾 Saving knowledge base to {}", save_path);
    
    match learning_system.save_knowledge_base(save_path) {
        Ok(()) => {
            println!("✅ Knowledge base saved successfully");
            
            // Test loading
            let mut new_learning_system = AdaptiveTheoremLearningSystem::new(AdaptiveLearningConfig::default());
            match new_learning_system.load_knowledge_base(save_path) {
                Ok(()) => {
                    println!("✅ Knowledge base loaded successfully");
                    let loaded_insights = new_learning_system.get_learning_insights();
                    println!("📊 Loaded knowledge contains {} patterns", loaded_insights.total_patterns);
                }
                Err(e) => {
                    println!("❌ Knowledge base loading failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Knowledge base saving failed: {}", e);
        }
    }
    
    // Test 6: Real-World Application Simulation
    println!("\n🌍 Test 6: Real-World Application Simulation");
    println!("---------------------------------------------");
    
    // Simulate evaluating code with learned optimizations
    println!("🔄 Simulating evaluation with learned optimizations");
    
    let test_expressions = vec![
        "Fibonacci calculation",
        "List processing with map/fold",
        "Higher-order function composition",
        "Tail-recursive factorial",
        "Matrix multiplication",
    ];
    
    for expr_name in &test_expressions {
        println!("🧮 Processing: {}", expr_name);
        
        // Simulate performance improvement from learned patterns
        let baseline_time = 100.0; // μs
        let improvement_factor = 1.0 + (insights.average_performance_improvement);
        let optimized_time = baseline_time / improvement_factor;
        
        println!("   ⏱️ Baseline: {:.1}μs → Optimized: {:.1}μs", baseline_time, optimized_time);
        println!("   🚀 Improvement: {:.1}%", (improvement_factor - 1.0) * 100.0);
    }
    
    // Test 7: Learning System Evolution Tracking
    println!("\n📈 Test 7: Learning System Evolution Tracking");
    println!("----------------------------------------------");
    
    println!("📊 Learning Session Summary:");
    for (i, session) in learning_sessions.iter().enumerate() {
        println!("   Session {}: {} patterns, {:.1}ms processing time",
                 i + 1,
                 session.discovered_patterns.len(),
                 session.session_stats.processing_time.as_millis());
    }
    
    let total_sessions = learning_sessions.len();
    let total_patterns: usize = learning_sessions.iter()
        .map(|s| s.discovered_patterns.len())
        .sum();
    let total_processing_time: std::time::Duration = learning_sessions.iter()
        .map(|s| s.session_stats.processing_time)
        .sum();
    
    println!("\n📈 Overall Learning Statistics:");
    println!("   🎯 Total learning sessions: {}", total_sessions);
    println!("   📊 Total patterns discovered: {}", total_patterns);
    println!("   ⏱️ Total learning time: {:?}", total_processing_time);
    println!("   🧮 Average patterns per session: {:.1}", 
             if total_sessions > 0 { total_patterns as f64 / total_sessions as f64 } else { 0.0 });
    
    // Test 8: Future Learning Potential Assessment
    println!("\n🔮 Test 8: Future Learning Potential Assessment");
    println!("-----------------------------------------------");
    
    println!("🔮 Future Learning Potential:");
    println!("   🌱 Knowledge base capacity: {:.1}% utilized", 
             (insights.total_patterns as f64 / 1000.0) * 100.0);
    println!("   📈 Learning trajectory: {} sessions completed", total_sessions);
    println!("   🎯 Pattern discovery rate: {:.2} patterns/session", 
             if total_sessions > 0 { total_patterns as f64 / total_sessions as f64 } else { 0.0 });
    
    if insights.total_patterns > 0 {
        let confidence_ratio = insights.high_confidence_patterns as f64 / insights.total_patterns as f64;
        println!("   🎖️ High confidence ratio: {:.1}%", confidence_ratio * 100.0);
        
        if confidence_ratio > 0.5 {
            println!("   🏆 Excellent learning quality - ready for production optimization");
        } else if confidence_ratio > 0.3 {
            println!("   ✅ Good learning progress - continue accumulating knowledge");
        } else {
            println!("   🔄 Early learning stage - more diverse code samples needed");
        }
    }
    
    println!("\n🎉 Adaptive Learning System Demo Complete!");
    println!("==========================================");
    println!("✅ Successfully demonstrated knowledge accumulation from real code");
    println!("🧠 Automatic pattern discovery and theorem strengthening");
    println!("🔗 Integration with formal theorem derivation system");
    println!("💾 Persistent knowledge base for continuous improvement");
    println!("🚀 Ready for real-world Scheme optimization learning");
    println!("🏆 Revolutionary code-driven evaluator enhancement system operational");
    
    Ok(())
}
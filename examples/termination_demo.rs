//! Demonstration of Strong Normalization and Church-Rosser property verification.
//!
//! This example shows how to use Lambdust's termination analysis system to verify
//! that dependent type terms are well-behaved (both strongly normalizing and confluent).

use lambdust::types::dependent::{
    DependentType, DependentTerm, StrongNormalizationChecker,
    TerminationConfluenceSystem, NormalizationEngine, TerminationConfig,
    ComplexityMeasure,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Lambdust Termination Analysis Demo");
    println!("=====================================\n");

    // Example 1: Simple variable - should be strongly normalizing and confluent
    println!("📋 Example 1: Simple Variable");
    println!("-----------------------------");
    let simple_var = DependentTerm::Variable("x".to_string());
    analyze_term(&simple_var, "x")?;

    // Example 2: Identity function - should be well-behaved
    println!("\n📋 Example 2: Identity Function");
    println!("--------------------------------");
    let identity_fn = DependentTerm::Lambda {
        param: "x".to_string(),
        param_type: Box::new(DependentType::Universe(0)),
        body: Box::new(DependentTerm::Variable("x".to_string())),
    };
    analyze_term(&identity_fn, "λx.x")?;

    // Example 3: Function application
    println!("\n📋 Example 3: Function Application");
    println!("-----------------------------------");
    let application = DependentTerm::Application {
        function: Box::new(identity_fn.clone()),
        argument: Box::new(DependentTerm::Variable("y".to_string())),
    };
    analyze_term(&application, "(λx.x) y")?;

    // Example 4: Dependent pair
    println!("\n📋 Example 4: Dependent Pair");
    println!("-----------------------------");
    let pair = DependentTerm::Pair {
        first: Box::new(DependentTerm::Variable("a".to_string())),
        second: Box::new(DependentTerm::Variable("b".to_string())),
    };
    analyze_term(&pair, "(a, b)")?;

    // Example 5: Projection from pair
    println!("\n📋 Example 5: Projection from Pair");
    println!("-----------------------------------");
    let projection = DependentTerm::Projection {
        pair: Box::new(pair),
        is_first: true,
    };
    analyze_term(&projection, "π₁((a, b))")?;

    // Example 6: Configuration comparison
    println!("\n📋 Example 6: Configuration Comparison");
    println!("---------------------------------------");
    demonstrate_configurations()?;

    // Example 7: Complexity analysis
    println!("\n📋 Example 7: Complexity Analysis");
    println!("----------------------------------");
    demonstrate_complexity_analysis()?;

    // Example 8: Safe normalization
    println!("\n📋 Example 8: Safe Normalization");
    println!("---------------------------------");
    demonstrate_safe_normalization()?;

    println!("\n✅ All examples completed successfully!");
    println!("\n🎯 Summary:");
    println!("   • Strong normalization ensures all reduction sequences terminate");
    println!("   • Church-Rosser property ensures deterministic normal forms");
    println!("   • Combined system provides safety guarantees for dependent types");
    println!("   • Complexity measures enable termination proofs");
    println!("   • Safe normalization prevents infinite loops in type checking");

    Ok(())
}

fn analyze_term(term: &DependentTerm, description: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Analyzing term: {description}");

    // Create termination and confluence system
    let system = TerminationConfluenceSystem::new();

    // Check if term is well-behaved
    let (strongly_normalizing, confluent) = system.is_well_behaved(term)?;

    println!("  Strong Normalization: {}", if strongly_normalizing { "✅ YES" } else { "❌ NO" });
    println!("  Church-Rosser (Confluence): {}", if confluent { "✅ YES" } else { "❌ NO" });

    // Get detailed analysis
    let report = system.analyze_term(term)?;
    
    println!("  Analysis Time: {} ms", report.analysis_time_ms);
    
    if let Some(termination_proof) = &report.termination_proof {
        println!("  Termination Confidence: {:.2}", termination_proof.confidence);
        println!("  Termination Method: {}", termination_proof.method);
    }

    if !report.recommendations.is_empty() {
        println!("  Recommendations:");
        for rec in &report.recommendations {
            println!("    • {rec}");
        }
    }

    // Show complexity measure
    let complexity = ComplexityMeasure::for_term(term);
    println!("  Complexity: {complexity}");

    Ok(())
}

fn demonstrate_configurations() -> Result<(), Box<dyn std::error::Error>> {
    let simple_term = DependentTerm::Variable("test".to_string());

    println!("Testing different configuration levels:");

    // Fast configuration
    let fast_checker = StrongNormalizationChecker::with_config(TerminationConfig::fast());
    let fast_result = fast_checker.is_strongly_normalizing(&simple_term)?;
    let fast_stats = fast_checker.statistics();
    println!("  Fast Config:");
    println!("    Result: {}", if fast_result { "✅ Terminating" } else { "❌ Non-terminating" });
    println!("    Terms analyzed: {}", fast_stats.terms_analyzed);

    // Default configuration
    let default_checker = StrongNormalizationChecker::new();
    let default_result = default_checker.is_strongly_normalizing(&simple_term)?;
    let default_stats = default_checker.statistics();
    println!("  Default Config:");
    println!("    Result: {}", if default_result { "✅ Terminating" } else { "❌ Non-terminating" });
    println!("    Terms analyzed: {}", default_stats.terms_analyzed);

    // Thorough configuration
    let thorough_checker = StrongNormalizationChecker::with_config(TerminationConfig::thorough());
    let thorough_result = thorough_checker.is_strongly_normalizing(&simple_term)?;
    let thorough_stats = thorough_checker.statistics();
    println!("  Thorough Config:");
    println!("    Result: {}", if thorough_result { "✅ Terminating" } else { "❌ Non-terminating" });
    println!("    Terms analyzed: {}", thorough_stats.terms_analyzed);

    Ok(())
}

fn demonstrate_complexity_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("Comparing complexity measures:");

    let terms = vec![
        ("Variable", DependentTerm::Variable("x".to_string())),
        ("Lambda", DependentTerm::Lambda {
            param: "x".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Variable("x".to_string())),
        }),
        ("Application", DependentTerm::Application {
            function: Box::new(DependentTerm::Variable("f".to_string())),
            argument: Box::new(DependentTerm::Variable("x".to_string())),
        }),
        ("Pair", DependentTerm::Pair {
            first: Box::new(DependentTerm::Variable("a".to_string())),
            second: Box::new(DependentTerm::Variable("b".to_string())),
        }),
    ];

    for (name, term) in terms {
        let complexity = ComplexityMeasure::for_term(&term);
        println!("  {}: depth={}, size={}, vars={}, abstractions={}", 
                 name, complexity.depth, complexity.size, 
                 complexity.variables, complexity.abstractions);
    }

    // Test ordering
    let simple = ComplexityMeasure::for_term(&DependentTerm::Variable("x".to_string()));
    let complex = ComplexityMeasure::for_term(&DependentTerm::Lambda {
        param: "x".to_string(),
        param_type: Box::new(DependentType::Universe(0)),
        body: Box::new(DependentTerm::Variable("x".to_string())),
    });

    println!("  Ordering: simple < complex = {}", simple.is_smaller_than(&complex));
    println!("  Decrease amount: {}", complex.decrease_amount(&simple));

    Ok(())
}

fn demonstrate_safe_normalization() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing safe normalization:");

    let mut engine = NormalizationEngine::new();
    
    // Test with a simple term
    let term = DependentTerm::Variable("x".to_string());
    
    println!("  Term: x");
    
    // Regular normalization
    let regular_result = engine.normalize_term(&term)?;
    println!("  Regular normalization: {:?}", regular_result.normalized);
    
    // Safe normalization (with termination guarantees)
    let safe_result = engine.safe_normalize_term(&term)?;
    println!("  Safe normalization: {:?}", safe_result.normalized);
    
    // Check if term is well-behaved
    let (normalizing, confluent) = engine.is_well_behaved(&term)?;
    println!("  Is well-behaved: normalizing={normalizing}, confluent={confluent}");
    
    // Get complexity measure
    let complexity = engine.get_complexity_measure(&term);
    println!("  Complexity: {complexity}");
    
    // Show statistics
    let stats = engine.get_statistics();
    println!("  Statistics:");
    println!("    Terms normalized: {}", stats.terms_normalized);
    println!("    Terms checked for termination: {}", stats.terms_checked_termination);
    println!("    Terms checked for confluence: {}", stats.terms_checked_confluence);
    println!("    Strongly normalizing terms: {}", stats.strongly_normalizing_terms);
    println!("    Confluent terms: {}", stats.confluent_terms);

    // Test with checking disabled
    engine.set_termination_checking(false);
    engine.set_confluence_checking(false);
    
    let result_disabled = engine.safe_normalize_term(&term)?;
    println!("  Safe normalization (checking disabled): {:?}", result_disabled.normalized);

    Ok(())
}
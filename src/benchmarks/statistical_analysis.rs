//! Statistical Analysis Framework for Performance Benchmarks
//!
//! This module provides comprehensive statistical analysis capabilities for
//! benchmark results, including hypothesis testing, confidence intervals,
//! outlier detection, and performance regression analysis.

use std::time::Duration;

// Re-export all statistical analysis components
pub use super::statistical_analyzer::StatisticalAnalyzer;
pub use super::statistical_tests::*;
pub use super::effect_analysis::*;
pub use super::outlier_normality::*;
pub use super::correlation_statistics::*;

/// Generate a comprehensive statistical report
pub fn generate_statistical_report(analysis: &StatisticalAnalysisResult) -> String {
    let mut report = String::new();
    
    report.push_str("# Statistical Analysis Report\n\n");
    
    // Descriptive statistics
    report.push_str("## Descriptive Statistics\n\n");
    report.push_str("| Implementation | N | Mean | Median | Std Dev | Min | Max | CV |\n");
    report.push_str("|---|---|---|---|---|---|---|---|\n");
    
    for (name, stats) in &analysis.descriptive_stats {
        report.push_str(&format!(
            "| {} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.3} |\n",
            name, stats.n, stats.mean, stats.median, stats.std_dev, 
            stats.min, stats.max, stats.coefficient_of_variation
        ));
    }
    
    // Pairwise comparisons
    report.push_str("\n## Pairwise Comparisons\n\n");
    report.push_str("| Comparison | T-test p-value | Effect Size (Cohen's d) | Significant | Practical |\n");
    report.push_str("|---|---|---|---|---|\n");
    
    for comp in &analysis.pairwise_comparisons {
        report.push_str(&format!(
            "| {} vs {} | {:.4} | {:.3} | {} | {} |\n",
            comp.group_a, comp.group_b, comp.t_test.p_value, comp.effect_size.cohens_d,
            if comp.is_significant { "Yes" } else { "No" },
            if comp.is_practically_significant { "Yes" } else { "No" }
        ));
    }
    
    // ANOVA results
    if let Some(anova) = &analysis.anova_result {
        report.push_str("\n## ANOVA Results\n\n");
        report.push_str(&format!("- F-statistic: {:.3}\n", anova.f_statistic));
        report.push_str(&format!("- p-value: {:.4}\n", anova.p_value));
        report.push_str(&format!("- Significant: {}\n", if anova.significant { "Yes" } else { "No" }));
    }
    
    // Outlier analysis
    report.push_str("\n## Outlier Analysis\n\n");
    report.push_str(&format!("- Total outliers detected: {}\n", analysis.outlier_analysis.summary.total_outliers));
    report.push_str(&format!("- Percentage of data: {:.1}%\n", analysis.outlier_analysis.summary.outlier_percentage));
    report.push_str(&format!("- Recommendation: {:?}\n", analysis.outlier_analysis.summary.recommendation));
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_descriptive_statistics() {
        let analyzer = StatisticalAnalyzer::new(StatisticalAnalysisConfig::default());
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = analyzer.calculate_descriptive_stats("test", &data);
        
        assert_eq!(stats.n, 5);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert!(stats.std_dev > 0.0);
    }
    
    #[test]
    fn test_t_test() {
        let analyzer = StatisticalAnalyzer::new(StatisticalAnalysisConfig::default());
        let data_a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let data_b = vec![2.0, 3.0, 4.0, 5.0, 6.0];
        let result = analyzer.perform_t_test(&data_a, &data_b);
        
        assert!(result.t_statistic != 0.0);
        assert!(result.p_value >= 0.0 && result.p_value <= 1.0);
    }
    
    #[test]
    fn test_effect_size_calculation() {
        let analyzer = StatisticalAnalyzer::new(StatisticalAnalysisConfig::default());
        let data_a = vec![1.0, 2.0, 3.0];
        let data_b = vec![4.0, 5.0, 6.0];
        let effect_size = analyzer.calculate_effect_size_between(&data_a, &data_b);
        
        // Should show a large negative effect (group A < group B)
        assert!(effect_size.cohens_d < 0.0);
        assert!(matches!(effect_size.interpretation, EffectSizeInterpretation::VeryLarge));
    }
}
//! Comprehensive R7RS Test Runner
//!
//! This module provides utilities to run comprehensive R7RS compliance tests
//! and demonstrates how to use the complete test suite for validation.

use crate::r7rs_final_comprehensive_suite::{
    FinalR7RSTestSuite, FinalTestConfig, FinalTestStats,
    run_final_r7rs_compliance_tests, run_final_r7rs_compliance_tests_with_config
};
use crate::r7rs_compliance_tests::R7RSTestConfig;

/// Test runner configurations for different scenarios
pub struct TestRunnerConfigs;

impl TestRunnerConfigs {
    /// Configuration for development testing - skip unimplemented features
    pub fn development() -> FinalTestConfig {
        FinalTestConfig {
            base_config: R7RSTestConfig {
                strict_mode: true,
                skip_unimplemented: true,
                verbose: true,
            },
            include_stress_tests: false,
            include_performance_tests: false,
            include_file_io_tests: false,
            include_system_tests: false,
            include_error_recovery_tests: true,
            generate_detailed_report: true,
            test_timeout_seconds: 120,
        }
    }
    
    /// Configuration for CI testing - fast and focused
    pub fn continuous_integration() -> FinalTestConfig {
        FinalTestConfig {
            base_config: R7RSTestConfig {
                strict_mode: true,
                skip_unimplemented: true,
                verbose: false,
            },
            include_stress_tests: false,
            include_performance_tests: false,
            include_file_io_tests: false,
            include_system_tests: false,
            include_error_recovery_tests: true,
            generate_detailed_report: false,
            test_timeout_seconds: 60,
        }
    }
    
    /// Configuration for release validation - comprehensive testing
    pub fn release_validation() -> FinalTestConfig {
        FinalTestConfig {
            base_config: R7RSTestConfig {
                strict_mode: true,
                skip_unimplemented: false, // Test everything
                verbose: true,
            },
            include_stress_tests: true,
            include_performance_tests: true,
            include_file_io_tests: true,
            include_system_tests: true,
            include_error_recovery_tests: true,
            generate_detailed_report: true,
            test_timeout_seconds: 300,
        }
    }
    
    /// Configuration for compliance certification - strictest testing
    pub fn compliance_certification() -> FinalTestConfig {
        FinalTestConfig {
            base_config: R7RSTestConfig {
                strict_mode: true,
                skip_unimplemented: false,
                verbose: true,
            },
            include_stress_tests: true,
            include_performance_tests: true,
            include_file_io_tests: true,
            include_system_tests: true,
            include_error_recovery_tests: true,
            generate_detailed_report: true,
            test_timeout_seconds: 600, // 10 minutes per category
        }
    }
}

/// Run comprehensive R7RS tests for development
pub fn run_development_tests() -> Result<FinalTestStats, Box<dyn std::error::Error>> {
    println!("Running R7RS tests in development mode...");
    run_final_r7rs_compliance_tests_with_config(TestRunnerConfigs::development())
}

/// Run comprehensive R7RS tests for CI
pub fn run_ci_tests() -> Result<FinalTestStats, Box<dyn std::error::Error>> {
    println!("Running R7RS tests in CI mode...");
    run_final_r7rs_compliance_tests_with_config(TestRunnerConfigs::continuous_integration())
}

/// Run comprehensive R7RS tests for release validation
pub fn run_release_tests() -> Result<FinalTestStats, Box<dyn std::error::Error>> {
    println!("Running R7RS tests in release validation mode...");
    run_final_r7rs_compliance_tests_with_config(TestRunnerConfigs::release_validation())
}

/// Run comprehensive R7RS tests for compliance certification
pub fn run_certification_tests() -> Result<FinalTestStats, Box<dyn std::error::Error>> {
    println!("Running R7RS tests in compliance certification mode...");
    run_final_r7rs_compliance_tests_with_config(TestRunnerConfigs::compliance_certification())
}

/// Validate minimum compliance requirements
pub fn validate_minimum_compliance(stats: &FinalTestStats) -> bool {
    // Define minimum requirements for basic R7RS compliance
    stats.compliance_percentage >= 85.0 &&
    stats.passed_categories >= (stats.total_categories * 4 / 5) && // At least 80% categories pass
    stats.critical_failures.len() < 3 // Fewer than 3 critical failures
}

/// Generate compliance summary report
pub fn generate_compliance_summary(stats: &FinalTestStats) -> String {
    let mut summary = String::new();
    
    summary.push_str("R7RS-small Compliance Summary\n");
    summary.push_str("============================\n\n");
    
    summary.push_str(&format!("Overall Compliance: {:.1}%\n", stats.compliance_percentage));
    summary.push_str(&format!("Test Categories: {} passed, {} failed, {} skipped\n", 
                            stats.passed_categories, stats.failed_categories, stats.skipped_categories));
    summary.push_str(&format!("Execution Time: {:.2}s\n", stats.execution_time.as_secs_f32()));
    
    summary.push_str("\nCompliance Status: ");
    if stats.compliance_percentage >= 95.0 {
        summary.push_str("EXCELLENT - Production Ready\n");
    } else if stats.compliance_percentage >= 85.0 {
        summary.push_str("GOOD - Suitable for Most Use Cases\n");
    } else if stats.compliance_percentage >= 70.0 {
        summary.push_str("FAIR - Has Significant Gaps\n");
    } else {
        summary.push_str("POOR - Major Compliance Issues\n");
    }
    
    if !stats.critical_failures.is_empty() {
        summary.push_str("\nCritical Issues:\n");
        for failure in &stats.critical_failures {
            summary.push_str(&format!("- {}\n", failure));
        }
    }
    
    summary.push_str("\nRecommendations:\n");
    if stats.compliance_percentage < 85.0 {
        summary.push_str("- Focus on implementing missing core procedures\n");
        summary.push_str("- Address critical failures before production use\n");
    }
    if stats.compliance_percentage >= 85.0 && stats.compliance_percentage < 95.0 {
        summary.push_str("- Consider implementing remaining optional features\n");
        summary.push_str("- Review edge cases and error handling\n");
    }
    if stats.compliance_percentage >= 95.0 {
        summary.push_str("- Excellent compliance! Consider performance optimization\n");
        summary.push_str("- Review any remaining edge cases for completeness\n");
    }
    
    summary
}

/// Interactive test runner for development
pub fn interactive_test_runner() -> Result<(), Box<dyn std::error::Error>> {
    println!("R7RS-small Interactive Test Runner");
    println!("==================================");
    println!();
    println!("Choose test mode:");
    println!("1. Development (skip unimplemented, fast)");
    println!("2. CI (automated testing)");
    println!("3. Release Validation (comprehensive)");
    println!("4. Compliance Certification (strictest)");
    println!("5. Custom configuration");
    println!();
    
    // For demonstration, we'll run development mode
    // In a real implementation, you'd read user input
    let stats = run_development_tests()?;
    
    println!("\n{}", generate_compliance_summary(&stats));
    
    if validate_minimum_compliance(&stats) {
        println!("\n✅ Minimum compliance requirements met!");
    } else {
        println!("\n❌ Minimum compliance requirements NOT met.");
        println!("   Consider addressing critical issues before release.");
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_development_config() {
        let config = TestRunnerConfigs::development();
        assert!(config.base_config.skip_unimplemented);
        assert!(!config.include_stress_tests);
        assert!(config.generate_detailed_report);
    }
    
    #[test]
    fn test_ci_config() {
        let config = TestRunnerConfigs::continuous_integration();
        assert!(config.base_config.skip_unimplemented);
        assert!(!config.include_performance_tests);
        assert!(!config.generate_detailed_report);
        assert_eq!(config.test_timeout_seconds, 60);
    }
    
    #[test]
    fn test_release_config() {
        let config = TestRunnerConfigs::release_validation();
        assert!(!config.base_config.skip_unimplemented);
        assert!(config.include_stress_tests);
        assert!(config.include_performance_tests);
    }
    
    #[test]
    fn test_compliance_validation() {
        let good_stats = FinalTestStats {
            total_categories: 10,
            passed_categories: 9,
            failed_categories: 1,
            skipped_categories: 0,
            total_individual_tests: 1000,
            passed_individual_tests: 950,
            failed_individual_tests: 50,
            execution_time: std::time::Duration::from_secs(120),
            compliance_percentage: 90.0,
            critical_failures: vec![],
            missing_procedures: vec![],
            implementation_gaps: vec![],
        };
        
        assert!(validate_minimum_compliance(&good_stats));
        
        let poor_stats = FinalTestStats {
            total_categories: 10,
            passed_categories: 5,
            failed_categories: 5,
            skipped_categories: 0,
            total_individual_tests: 1000,
            passed_individual_tests: 600,
            failed_individual_tests: 400,
            execution_time: std::time::Duration::from_secs(60),
            compliance_percentage: 60.0,
            critical_failures: vec!["Critical error 1".to_string(), "Critical error 2".to_string(), "Critical error 3".to_string()],
            missing_procedures: vec!["procedure1".to_string()],
            implementation_gaps: vec!["gap1".to_string()],
        };
        
        assert!(!validate_minimum_compliance(&poor_stats));
    }
    
    #[test]
    fn test_summary_generation() {
        let stats = FinalTestStats {
            total_categories: 10,
            passed_categories: 8,
            failed_categories: 2,
            skipped_categories: 0,
            total_individual_tests: 1000,
            passed_individual_tests: 900,
            failed_individual_tests: 100,
            execution_time: std::time::Duration::from_secs(300),
            compliance_percentage: 88.5,
            critical_failures: vec!["Test failure example".to_string()],
            missing_procedures: vec![],
            implementation_gaps: vec![],
        };
        
        let summary = generate_compliance_summary(&stats);
        assert!(summary.contains("88.5%"));
        assert!(summary.contains("GOOD"));
        assert!(summary.contains("Test failure example"));
    }
}
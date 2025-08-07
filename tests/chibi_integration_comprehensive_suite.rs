//! Comprehensive Chibi-Scheme Integration Test Suite
//!
//! This is the main entry point for running Chibi-Scheme tests against Lambdust
//! to validate R7RS compliance and generate comprehensive reports.

use std::path::Path;
use std::fs;

mod chibi_integration;

use chibi_integration::{
    ChibiIntegrationSuite, ChibiIntegrationConfig, TestSuiteResults,
    compliance_analyzer::ComplianceAnalyzer,
    report_generator::ReportGenerator,
};

/// Run the complete Chibi-Scheme integration test suite
pub fn run_chibi_integration_tests() -> Result<TestSuiteResults, Box<dyn std::error::Error>> {
    println!("🚀 Starting Comprehensive Chibi-Scheme Integration Test Suite");
    println!("="repeat(80));
    
    // Create configuration
    let config = create_test_configuration();
    
    // Create and run test suite
    let mut suite = ChibiIntegrationSuite::with_config(config.clone());
    let results = suite.run_complete_suite()?;
    
    // Analyze compliance
    let analyzer = ComplianceAnalyzer::new();
    let analysis = analyzer.analyze_compliance(&results);
    
    // Generate comprehensive reports
    generate_comprehensive_reports(&results, &analysis, &config)?;
    
    // Print final summary
    print_executive_summary(&results, &analysis);
    
    Ok(results)
}

/// Create test configuration optimized for comprehensive analysis
fn create_test_configuration() -> ChibiIntegrationConfig {
    ChibiIntegrationConfig {
        chibi_test_path: "/tmp/chibi-scheme/tests".to_string(),
        adapted_test_path: "tests/chibi_integration/adapted_tests".to_string(),
        report_path: "tests/chibi_integration/reports".to_string(),
        include_performance: true,
        detailed_errors: true,
        continue_on_error: true,
        timeout_seconds: 30,
        adapt_extensions: true,
    }
}

/// Generate comprehensive reports in multiple formats
fn generate_comprehensive_reports(
    results: &TestSuiteResults,
    analysis: &chibi_integration::compliance_analyzer::ComplianceAnalysis,
    config: &ChibiIntegrationConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    
    println!("📊 Generating Comprehensive Reports");
    println!("-".repeat(50));
    
    let report_generator = ReportGenerator::with_analysis(results, analysis.clone());
    
    // Ensure report directory exists
    fs::create_dir_all(&config.report_path)?;
    
    // Generate all report formats
    let report_dir = Path::new(&config.report_path);
    
    // JSON report for programmatic access
    report_generator.generate_json_report(&report_dir.join("chibi_compliance_report.json"))?;
    
    // HTML report for interactive viewing
    report_generator.generate_html_report(&report_dir.join("chibi_compliance_report.html"))?;
    
    // Markdown summary for documentation
    report_generator.generate_markdown_summary(&report_dir.join("chibi_compliance_summary.md"))?;
    
    // CSV exports for data analysis
    report_generator.generate_csv_export(&report_dir)?;
    
    // Generate executive summary for quick reference
    generate_executive_summary_file(results, analysis, &report_dir)?;
    
    // Generate implementation roadmap
    generate_implementation_roadmap(analysis, &report_dir)?;
    
    println!("✅ All reports generated successfully");
    println!("📁 Report directory: {}", config.report_path);
    
    Ok(())
}

/// Generate executive summary file
fn generate_executive_summary_file(
    results: &TestSuiteResults,
    analysis: &chibi_integration::compliance_analyzer::ComplianceAnalysis,
    report_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    
    let summary = format!(r#"# Lambdust R7RS Compliance - Executive Summary

## Overall Assessment

**Compliance Grade:** {}
**Overall Score:** {:.1}%
**Feature Completeness:** {:.1}%
**Weighted Score:** {:.1}%

## Test Execution Results

- **Total Tests:** {}
- **Passed:** {} ({:.1}%)
- **Failed:** {} ({:.1}%)
- **Errors:** {} ({:.1}%)
- **Timeouts:** {} ({:.1}%)
- **Execution Time:** {:.2}s

## Critical Gaps (Top 5)

{}

## Priority Recommendations (Top 5)

{}

## Implementation Readiness

{}

---
Generated: {}
"#,
        get_compliance_grade(analysis.overall_compliance.overall_percentage),
        analysis.overall_compliance.overall_percentage,
        analysis.overall_compliance.feature_completeness,
        analysis.overall_compliance.weighted_score,
        results.total_tests,
        results.passed,
        (results.passed as f64 / results.total_tests as f64) * 100.0,
        results.failed,
        (results.failed as f64 / results.total_tests as f64) * 100.0,
        results.errors,
        (results.errors as f64 / results.total_tests as f64) * 100.0,
        results.timeouts,
        (results.timeouts as f64 / results.total_tests as f64) * 100.0,
        results.execution_duration.as_secs_f64(),
        format_critical_gaps(&analysis.critical_gaps),
        format_top_recommendations(&analysis.recommendations),
        get_readiness_assessment(&analysis.completeness_analysis),
        chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
    );
    
    fs::write(report_dir.join("executive_summary.md"), summary)?;
    Ok(())
}

/// Generate implementation roadmap file
fn generate_implementation_roadmap(
    analysis: &chibi_integration::compliance_analyzer::ComplianceAnalysis,
    report_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    
    let mut roadmap = String::new();
    roadmap.push_str("# Lambdust R7RS Implementation Roadmap\n\n");
    
    for phase in &analysis.priority_roadmap.phases {
        roadmap.push_str(&format!("## Phase {}: {}\n\n", phase.phase_number, phase.name));
        roadmap.push_str(&format!("**Duration:** {} weeks  \n", phase.estimated_duration_weeks));
        roadmap.push_str(&format!("**Description:** {}  \n\n", phase.description));
        
        roadmap.push_str("### Key Features\n\n");
        for item in &phase.items {
            roadmap.push_str(&format!("- **{}**: {} → {:?} (Impact: {:.1}/10)\n",
                item.feature.as_str(),
                format!("{:?}", item.current_status),
                item.target_status,
                item.expected_impact
            ));
        }
        
        roadmap.push_str("\n### Success Criteria\n\n");
        for criterion in &phase.success_criteria {
            roadmap.push_str(&format!("- {}\n", criterion));
        }
        
        roadmap.push_str("\n---\n\n");
    }
    
    roadmap.push_str(&format!("**Total Estimated Duration:** {} weeks  \n", analysis.priority_roadmap.total_estimated_weeks));
    roadmap.push_str(&format!("**Confidence Level:** {:.0}%  \n", analysis.priority_roadmap.confidence_level * 100.0));
    
    fs::write(report_dir.join("implementation_roadmap.md"), roadmap)?;
    Ok(())
}

/// Print executive summary to console
fn print_executive_summary(
    results: &TestSuiteResults,
    analysis: &chibi_integration::compliance_analyzer::ComplianceAnalysis,
) {
    println!("\n🎯 EXECUTIVE SUMMARY");
    println!("="repeat(80));
    
    println!("📊 COMPLIANCE METRICS:");
    println!("  Overall Compliance:    {:.1}% ({})", 
             analysis.overall_compliance.overall_percentage,
             get_compliance_grade(analysis.overall_compliance.overall_percentage));
    println!("  Feature Completeness:  {:.1}%", analysis.overall_compliance.feature_completeness);
    println!("  Weighted Score:        {:.1}%", analysis.overall_compliance.weighted_score);
    
    println!("\n🧪 TEST RESULTS:");
    println!("  Total Tests:     {}", results.total_tests);
    println!("  Passed:          {} ({:.1}%)", results.passed,
             (results.passed as f64 / results.total_tests as f64) * 100.0);
    println!("  Failed:          {} ({:.1}%)", results.failed,
             (results.failed as f64 / results.total_tests as f64) * 100.0);
    println!("  Errors:          {} ({:.1}%)", results.errors,
             (results.errors as f64 / results.total_tests as f64) * 100.0);
    println!("  Execution Time:  {:.2}s", results.execution_duration.as_secs_f64());
    
    println!("\n🚨 CRITICAL GAPS:");
    for (i, gap) in analysis.critical_gaps.iter().take(3).enumerate() {
        println!("  {}. {} - {} priority", i + 1, gap.feature.as_str(), gap.severity.as_str());
    }
    
    println!("\n💡 TOP RECOMMENDATIONS:");
    for (i, rec) in analysis.recommendations.iter().take(3).enumerate() {
        println!("  {}. {}", i + 1, rec.title);
    }
    
    println!("\n⏱️ IMPLEMENTATION TIMELINE:");
    println!("  Estimated Duration: {} weeks", analysis.priority_roadmap.total_estimated_weeks);
    println!("  Number of Phases:   {}", analysis.priority_roadmap.phases.len());
    println!("  Confidence Level:   {:.0}%", analysis.priority_roadmap.confidence_level * 100.0);
    
    println!("\n📁 REPORTS GENERATED:");
    println!("  Location: {}", results.config.report_path);
    println!("  Formats:  JSON, HTML, Markdown, CSV");
    
    println!("\n="repeat(80));
}

/// Copy and adapt key Chibi-Scheme test files
pub fn copy_and_adapt_key_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Copying and Adapting Key Chibi-Scheme Tests");
    println!("-".repeat(50));
    
    let chibi_path = Path::new("/tmp/chibi-scheme/tests");
    let adapted_path = Path::new("tests/chibi_integration/adapted_tests");
    
    // Ensure target directory exists
    fs::create_dir_all(adapted_path)?;
    
    // Key test files to adapt
    let key_tests = vec![
        ("r7rs-tests.scm", "Core R7RS compliance tests"),
        ("r5rs-tests.scm", "R5RS compatibility tests"),  
        ("syntax-tests.scm", "Syntax and macro tests"),
        ("lib-tests.scm", "Standard library tests"),
        ("division-tests.scm", "Numeric division tests"),
        ("unicode-tests.scm", "Unicode handling tests"),
    ];
    
    for (filename, description) in key_tests {
        let source_path = chibi_path.join(filename);
        if source_path.exists() {
            println!("  📄 Adapting: {} ({})", filename, description);
            
            let content = fs::read_to_string(&source_path)?;
            let adapted_content = chibi_integration::test_adapter::adapt_chibi_test(&content)?;
            
            let target_path = adapted_path.join(filename);
            fs::write(&target_path, adapted_content)?;
            
            println!("      ✅ Saved to: {}", target_path.display());
        } else {
            println!("      ⚠️  Not found: {}", source_path.display());
        }
    }
    
    // Copy basic functionality tests
    let basic_dir = chibi_path.join("basic");
    if basic_dir.exists() {
        let adapted_basic = adapted_path.join("basic");
        fs::create_dir_all(&adapted_basic)?;
        
        println!("  📁 Adapting basic functionality tests...");
        let mut basic_count = 0;
        
        for entry in fs::read_dir(basic_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "scm") {
                let content = fs::read_to_string(&path)?;
                let adapted_content = chibi_integration::test_adapter::adapt_chibi_test(&content)?;
                
                let filename = path.file_name().unwrap();
                let target_path = adapted_basic.join(filename);
                fs::write(&target_path, adapted_content)?;
                
                basic_count += 1;
            }
        }
        
        println!("      ✅ Adapted {} basic tests", basic_count);
    }
    
    // Copy memory tests
    let memory_dir = chibi_path.join("memory");
    if memory_dir.exists() {
        let adapted_memory = adapted_path.join("memory");
        fs::create_dir_all(&adapted_memory)?;
        
        println!("  📁 Adapting memory tests...");
        let mut memory_count = 0;
        
        for entry in fs::read_dir(memory_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "scm") {
                let content = fs::read_to_string(&path)?;
                let adapted_content = chibi_integration::test_adapter::adapt_chibi_test(&content)?;
                
                let filename = path.file_name().unwrap();
                let target_path = adapted_memory.join(filename);
                fs::write(&target_path, adapted_content)?;
                
                memory_count += 1;
            }
        }
        
        println!("      ✅ Adapted {} memory tests", memory_count);
    }
    
    // Create test catalog
    create_test_catalog(adapted_path)?;
    
    println!("✅ Test adaptation complete");
    Ok(())
}

/// Create a catalog of adapted tests
fn create_test_catalog(adapted_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("  📋 Creating test catalog...");
    
    let mut catalog = String::new();
    catalog.push_str("# Adapted Chibi-Scheme Test Catalog\n\n");
    catalog.push_str(&format!("Generated: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")));
    
    // Catalog main test files
    catalog.push_str("## Core Test Files\n\n");
    for entry in fs::read_dir(adapted_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "scm") {
            let filename = path.file_name().unwrap().to_string_lossy();
            let size = fs::metadata(&path)?.len();
            
            let description = match filename.as_ref() {
                "r7rs-tests.scm" => "Comprehensive R7RS compliance tests",
                "r5rs-tests.scm" => "R5RS compatibility tests",
                "syntax-tests.scm" => "Syntax forms and macro system tests", 
                "lib-tests.scm" => "Standard library procedure tests",
                "division-tests.scm" => "Numeric division and arithmetic tests",
                "unicode-tests.scm" => "Unicode character and string tests",
                _ => "Adapted test file",
            };
            
            catalog.push_str(&format!("- **{}** ({} bytes) - {}\n", filename, size, description));
        }
    }
    
    // Catalog subdirectories
    for entry in fs::read_dir(adapted_path)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let dirname = path.file_name().unwrap().to_string_lossy();
            catalog.push_str(&format!("\n## {} Tests\n\n", dirname.to_uppercase()));
            
            let mut file_count = 0;
            for sub_entry in fs::read_dir(&path)? {
                let sub_entry = sub_entry?;
                let sub_path = sub_entry.path();
                
                if sub_path.extension().map_or(false, |ext| ext == "scm") {
                    let filename = sub_path.file_name().unwrap().to_string_lossy();
                    let size = fs::metadata(&sub_path)?.len();
                    catalog.push_str(&format!("- **{}** ({} bytes)\n", filename, size));
                    file_count += 1;
                }
            }
            
            catalog.push_str(&format!("\nTotal: {} files\n", file_count));
        }
    }
    
    catalog.push_str("\n## Usage\n\n");
    catalog.push_str("These adapted test files can be run against Lambdust using:\n\n");
    catalog.push_str("```bash\n");
    catalog.push_str("cargo test chibi_integration_comprehensive_suite\n");
    catalog.push_str("```\n\n");
    catalog.push_str("Or run specific test categories:\n\n");
    catalog.push_str("```rust\n");
    catalog.push_str("use chibi_integration::*;\n");
    catalog.push_str("let mut suite = ChibiIntegrationSuite::new();\n");
    catalog.push_str("let results = suite.run_complete_suite()?;\n");
    catalog.push_str("```\n");
    
    fs::write(adapted_path.join("TEST_CATALOG.md"), catalog)?;
    println!("      ✅ Test catalog created");
    
    Ok(())
}

// Helper functions

fn get_compliance_grade(percentage: f64) -> &'static str {
    match percentage {
        p if p >= 95.0 => "A+ (Excellent)",
        p if p >= 90.0 => "A (Very Good)",
        p if p >= 85.0 => "B+ (Good)",
        p if p >= 80.0 => "B (Satisfactory)",
        p if p >= 70.0 => "C+ (Needs Work)",
        p if p >= 60.0 => "C (Major Gaps)",
        _ => "D (Incomplete)",
    }
}

fn format_critical_gaps(gaps: &[chibi_integration::compliance_analyzer::CriticalGap]) -> String {
    if gaps.is_empty() {
        return "None identified.".to_string();
    }
    
    gaps.iter().take(5).enumerate()
        .map(|(i, gap)| format!("{}. **{}** ({} priority, Impact: {:.1}/10)", 
                               i + 1, gap.feature.as_str(), gap.severity.as_str(), gap.impact))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_top_recommendations(recommendations: &[chibi_integration::compliance_analyzer::ComplianceRecommendation]) -> String {
    if recommendations.is_empty() {
        return "None at this time.".to_string();
    }
    
    recommendations.iter().take(5).enumerate()
        .map(|(i, rec)| format!("{}. **{}** - {}", i + 1, rec.title, rec.description))
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_readiness_assessment(completeness: &chibi_integration::compliance_analyzer::CompletenessAnalysis) -> String {
    match completeness.overall_readiness {
        r if r >= 90.0 => "🟢 **READY** - High compliance, production ready",
        r if r >= 75.0 => "🟡 **MOSTLY READY** - Good compliance, minor gaps",
        r if r >= 60.0 => "🟠 **PARTIALLY READY** - Moderate compliance, significant work needed",
        r if r >= 40.0 => "🔴 **NOT READY** - Low compliance, major implementation required",
        _ => "💥 **INCOMPLETE** - Extensive development needed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // Ignore by default as this requires Chibi-Scheme tests to be available
    fn test_comprehensive_chibi_integration() {
        let result = run_chibi_integration_tests();
        
        match result {
            Ok(results) => {
                println!("✅ Integration tests completed");
                println!("Total tests: {}", results.total_tests);
                println!("Passed: {}", results.passed);
                println!("Failed: {}", results.failed);
                
                // Assert basic functionality
                assert!(results.total_tests > 0, "Should have run some tests");
            },
            Err(e) => {
                println!("⚠️ Integration tests failed: {}", e);
                // Don't fail the test in CI where Chibi tests might not be available
                if std::env::var("CI").is_err() {
                    panic!("Integration test failure: {}", e);
                }
            }
        }
    }
    
    #[test]
    fn test_configuration_creation() {
        let config = create_test_configuration();
        assert!(config.include_performance);
        assert!(config.detailed_errors);
        assert!(config.continue_on_error);
        assert_eq!(config.timeout_seconds, 30);
    }
    
    #[test]
    fn test_compliance_grading() {
        assert_eq!(get_compliance_grade(98.0), "A+ (Excellent)");
        assert_eq!(get_compliance_grade(92.0), "A (Very Good)");
        assert_eq!(get_compliance_grade(87.0), "B+ (Good)");
        assert_eq!(get_compliance_grade(82.0), "B (Satisfactory)");
        assert_eq!(get_compliance_grade(72.0), "C+ (Needs Work)");
        assert_eq!(get_compliance_grade(62.0), "C (Major Gaps)");
        assert_eq!(get_compliance_grade(52.0), "D (Incomplete)");
    }
    
    #[test]
    #[ignore] // Ignore by default as this requires file system operations
    fn test_copy_and_adapt_tests() {
        let result = copy_and_adapt_key_tests();
        
        match result {
            Ok(_) => println!("✅ Test adaptation completed"),
            Err(e) => {
                println!("⚠️ Test adaptation failed: {}", e);
                // Don't fail in CI where Chibi tests might not be available
                if std::env::var("CI").is_err() {
                    panic!("Test adaptation failure: {}", e);
                }
            }
        }
    }
}
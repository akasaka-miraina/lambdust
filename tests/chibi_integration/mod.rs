//! Chibi-Scheme Test Integration Framework
//!
//! This module provides comprehensive integration with Chibi-Scheme's test suite
//! to validate Lambdust's R7RS compliance and implementation completeness.
//!
//! The framework includes:
//! - Test file adaptation from Chibi-Scheme format to Lambdust format
//! - Comprehensive test execution with detailed error reporting
//! - R7RS compliance analysis and gap identification
//! - Performance comparison between reference implementation and Lambdust
//! - Generation of actionable compliance reports

use lambdust::eval::evaluator::Evaluator;
use lambdust::eval::environment::Environment;
use lambdust::eval::value::Value;
use lambdust::parser::expression::parse_expression;
use lambdust::lexer::lexer::Lexer;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

pub mod test_runner;
pub mod test_adapter;
pub mod compliance_analyzer;
pub mod report_generator;

/// Configuration for Chibi-Scheme test integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChibiIntegrationConfig {
    /// Path to Chibi-Scheme test files
    pub chibi_test_path: String,
    /// Path to store adapted test files
    pub adapted_test_path: String,
    /// Path to store test reports
    pub report_path: String,
    /// Whether to run performance comparisons
    pub include_performance: bool,
    /// Whether to generate detailed error traces
    pub detailed_errors: bool,
    /// Whether to continue testing after failures
    pub continue_on_error: bool,
    /// Maximum time per test in seconds
    pub timeout_seconds: u64,
    /// Whether to adapt Chibi-specific extensions
    pub adapt_extensions: bool,
}

impl Default for ChibiIntegrationConfig {
    fn default() -> Self {
        Self {
            chibi_test_path: "/tmp/chibi-scheme/tests".to_string(),
            adapted_test_path: "tests/chibi_integration/adapted_tests".to_string(),
            report_path: "tests/chibi_integration/reports".to_string(),
            include_performance: false,
            detailed_errors: true,
            continue_on_error: true,
            timeout_seconds: 30,
            adapt_extensions: true,
        }
    }
}

/// Results from running a single test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub test_file: String,
    pub status: TestStatus,
    pub execution_time: Duration,
    pub error_message: Option<String>,
    pub stack_trace: Option<String>,
    pub expected_output: Option<String>,
    pub actual_output: Option<String>,
}

/// Test execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Error,
    Timeout,
    Skipped,
    Adapted, // Test was adapted but needs manual verification
}

/// Results from a complete test suite run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResults {
    pub config: ChibiIntegrationConfig,
    pub execution_start: String, // ISO timestamp
    pub execution_duration: Duration,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub timeouts: usize,
    pub skipped: usize,
    pub adapted: usize,
    pub test_results: Vec<TestResult>,
    pub feature_coverage: HashMap<String, FeatureCoverage>,
    pub compliance_summary: ComplianceSummary,
}

/// Coverage information for a specific R7RS feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCoverage {
    pub feature_name: String,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub coverage_percentage: f64,
    pub implementation_status: ImplementationStatus,
    pub missing_procedures: Vec<String>,
    pub notes: Vec<String>,
}

/// Implementation status for R7RS features
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImplementationStatus {
    Complete,
    Partial,
    Minimal,
    Missing,
    Planned,
}

/// Overall compliance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSummary {
    pub overall_percentage: f64,
    pub core_language_percentage: f64,
    pub standard_library_percentage: f64,
    pub syntax_compliance: f64,
    pub procedure_compliance: f64,
    pub critical_gaps: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Main Chibi-Scheme integration test suite
pub struct ChibiIntegrationSuite {
    config: ChibiIntegrationConfig,
    evaluator: Evaluator,
    environment: Environment,
    results: TestSuiteResults,
}

impl ChibiIntegrationSuite {
    /// Create a new integration test suite
    pub fn new() -> Self {
        Self::with_config(ChibiIntegrationConfig::default())
    }
    
    /// Create a new integration test suite with custom configuration
    pub fn with_config(config: ChibiIntegrationConfig) -> Self {
        let evaluator = Evaluator::new();
        let environment = Environment::new();
        
        let results = TestSuiteResults {
            config: config.clone(),
            execution_start: chrono::Utc::now().to_rfc3339(),
            execution_duration: Duration::new(0, 0),
            total_tests: 0,
            passed: 0,
            failed: 0,
            errors: 0,
            timeouts: 0,
            skipped: 0,
            adapted: 0,
            test_results: Vec::new(),
            feature_coverage: HashMap::new(),
            compliance_summary: ComplianceSummary {
                overall_percentage: 0.0,
                core_language_percentage: 0.0,
                standard_library_percentage: 0.0,
                syntax_compliance: 0.0,
                procedure_compliance: 0.0,
                critical_gaps: Vec::new(),
                recommendations: Vec::new(),
            },
        };
        
        Self {
            config,
            evaluator,
            environment,
            results,
        }
    }
    
    /// Run the complete Chibi-Scheme integration test suite
    pub fn run_complete_suite(&mut self) -> Result<TestSuiteResults, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        println!("🚀 Starting Chibi-Scheme Integration Test Suite");
        println!("=".repeat(60));
        println!("Configuration:");
        println!("  Chibi test path: {}", self.config.chibi_test_path);
        println!("  Adapted test path: {}", self.config.adapted_test_path);
        println!("  Report path: {}", self.config.report_path);
        println!("  Include performance: {}", self.config.include_performance);
        println!();
        
        // Ensure output directories exist
        self.ensure_directories()?;
        
        // Discover and categorize test files
        let test_files = self.discover_test_files()?;
        println!("📁 Discovered {} test files", test_files.len());
        
        // Run tests by category for better organization
        self.run_core_r7rs_tests(&test_files)?;
        self.run_basic_functionality_tests(&test_files)?;
        self.run_syntax_tests(&test_files)?;
        self.run_library_tests(&test_files)?;
        self.run_memory_tests(&test_files)?;
        
        // Analyze results and generate compliance summary
        self.analyze_compliance()?;
        
        // Generate detailed reports
        self.generate_reports()?;
        
        self.results.execution_duration = start_time.elapsed();
        
        self.print_summary();
        
        Ok(self.results.clone())
    }
    
    /// Ensure all required directories exist
    fn ensure_directories(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(&self.config.adapted_test_path)?;
        fs::create_dir_all(&self.config.report_path)?;
        Ok(())
    }
    
    /// Discover all available test files from Chibi-Scheme
    fn discover_test_files(&self) -> Result<Vec<TestFileInfo>, Box<dyn std::error::Error>> {
        let mut test_files = Vec::new();
        let chibi_path = Path::new(&self.config.chibi_test_path);
        
        // Core R7RS test files
        if chibi_path.join("r7rs-tests.scm").exists() {
            test_files.push(TestFileInfo {
                path: chibi_path.join("r7rs-tests.scm"),
                category: TestCategory::CoreR7RS,
                priority: TestPriority::Critical,
                estimated_tests: 500,
            });
        }
        
        if chibi_path.join("r5rs-tests.scm").exists() {
            test_files.push(TestFileInfo {
                path: chibi_path.join("r5rs-tests.scm"),
                category: TestCategory::R5RSCompat,
                priority: TestPriority::High,
                estimated_tests: 200,
            });
        }
        
        if chibi_path.join("syntax-tests.scm").exists() {
            test_files.push(TestFileInfo {
                path: chibi_path.join("syntax-tests.scm"),
                category: TestCategory::Syntax,
                priority: TestPriority::High,
                estimated_tests: 100,
            });
        }
        
        if chibi_path.join("lib-tests.scm").exists() {
            test_files.push(TestFileInfo {
                path: chibi_path.join("lib-tests.scm"),
                category: TestCategory::StandardLibrary,
                priority: TestPriority::High,
                estimated_tests: 150,
            });
        }
        
        // Basic functionality tests
        let basic_dir = chibi_path.join("basic");
        if basic_dir.exists() {
            for entry in fs::read_dir(basic_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "scm") {
                    test_files.push(TestFileInfo {
                        path,
                        category: TestCategory::BasicFunctionality,
                        priority: TestPriority::High,
                        estimated_tests: 1,
                    });
                }
            }
        }
        
        // Memory tests
        let memory_dir = chibi_path.join("memory");
        if memory_dir.exists() {
            for entry in fs::read_dir(memory_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "scm") {
                    test_files.push(TestFileInfo {
                        path,
                        category: TestCategory::Memory,
                        priority: TestPriority::Medium,
                        estimated_tests: 1,
                    });
                }
            }
        }
        
        // Sort by priority
        test_files.sort_by(|a, b| a.priority.cmp(&b.priority));
        
        Ok(test_files)
    }
    
    /// Run core R7RS compliance tests
    fn run_core_r7rs_tests(&mut self, test_files: &[TestFileInfo]) -> Result<(), Box<dyn std::error::Error>> {
        println!("📋 Running Core R7RS Tests");
        println!("-".repeat(40));
        
        for test_file in test_files.iter().filter(|f| f.category == TestCategory::CoreR7RS) {
            self.run_single_test_file(test_file)?;
        }
        
        Ok(())
    }
    
    /// Run basic functionality tests
    fn run_basic_functionality_tests(&mut self, test_files: &[TestFileInfo]) -> Result<(), Box<dyn std::error::Error>> {
        println!("📋 Running Basic Functionality Tests");
        println!("-".repeat(40));
        
        for test_file in test_files.iter().filter(|f| f.category == TestCategory::BasicFunctionality) {
            self.run_single_test_file(test_file)?;
        }
        
        Ok(())
    }
    
    /// Run syntax tests
    fn run_syntax_tests(&mut self, test_files: &[TestFileInfo]) -> Result<(), Box<dyn std::error::Error>> {
        println!("📋 Running Syntax Tests");
        println!("-".repeat(40));
        
        for test_file in test_files.iter().filter(|f| f.category == TestCategory::Syntax) {
            self.run_single_test_file(test_file)?;
        }
        
        Ok(())
    }
    
    /// Run standard library tests
    fn run_library_tests(&mut self, test_files: &[TestFileInfo]) -> Result<(), Box<dyn std::error::Error>> {
        println!("📋 Running Standard Library Tests");
        println!("-".repeat(40));
        
        for test_file in test_files.iter().filter(|f| f.category == TestCategory::StandardLibrary) {
            self.run_single_test_file(test_file)?;
        }
        
        Ok(())
    }
    
    /// Run memory management tests
    fn run_memory_tests(&mut self, test_files: &[TestFileInfo]) -> Result<(), Box<dyn std::error::Error>> {
        println!("📋 Running Memory Tests");
        println!("-".repeat(40));
        
        for test_file in test_files.iter().filter(|f| f.category == TestCategory::Memory) {
            self.run_single_test_file(test_file)?;
        }
        
        Ok(())
    }
    
    /// Run a single test file
    fn run_single_test_file(&mut self, test_file: &TestFileInfo) -> Result<(), Box<dyn std::error::Error>> {
        let file_name = test_file.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
            
        println!("  🧪 Testing: {}", file_name);
        
        let start_time = Instant::now();
        let content = fs::read_to_string(&test_file.path)?;
        
        // Adapt the test if needed
        let adapted_content = if self.config.adapt_extensions {
            test_adapter::adapt_chibi_test(&content)?
        } else {
            content
        };
        
        // Try to execute the test
        let result = self.execute_test_content(&adapted_content, file_name);
        
        let test_result = TestResult {
            test_name: file_name.to_string(),
            test_file: test_file.path.to_string_lossy().to_string(),
            status: match &result {
                Ok(_) => TestStatus::Passed,
                Err(_) => TestStatus::Failed,
            },
            execution_time: start_time.elapsed(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
            stack_trace: None, // TODO: Implement stack trace capture
            expected_output: None,
            actual_output: None,
        };
        
        match test_result.status {
            TestStatus::Passed => {
                self.results.passed += 1;
                println!("    ✅ PASSED ({:.2}ms)", test_result.execution_time.as_millis());
            },
            TestStatus::Failed => {
                self.results.failed += 1;
                println!("    ❌ FAILED ({:.2}ms)", test_result.execution_time.as_millis());
                if let Some(error) = &test_result.error_message {
                    println!("       Error: {}", error);
                }
            },
            _ => {}
        }
        
        self.results.test_results.push(test_result);
        self.results.total_tests += 1;
        
        if !self.config.continue_on_error && result.is_err() {
            return Err(result.unwrap_err());
        }
        
        Ok(())
    }
    
    /// Execute test content in Lambdust
    fn execute_test_content(&mut self, content: &str, test_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Create a timeout for test execution
        let timeout = Duration::from_secs(self.config.timeout_seconds);
        let start = Instant::now();
        
        // Parse and evaluate the test content
        let mut lexer = Lexer::new(content);
        
        loop {
            if start.elapsed() > timeout {
                return Err("Test execution timeout".into());
            }
            
            match parse_expression(&mut lexer) {
                Ok(Some(expr)) => {
                    match self.evaluator.eval(&expr, &mut self.environment) {
                        Ok(_value) => {
                            // Continue parsing next expression
                        },
                        Err(e) => {
                            return Err(format!("Evaluation error in {}: {}", test_name, e).into());
                        }
                    }
                },
                Ok(None) => {
                    // End of input reached successfully
                    break;
                },
                Err(e) => {
                    return Err(format!("Parse error in {}: {}", test_name, e).into());
                }
            }
        }
        
        Ok(())
    }
    
    /// Analyze compliance based on test results
    fn analyze_compliance(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Analyzing R7RS Compliance");
        println!("-".repeat(40));
        
        let total = self.results.total_tests as f64;
        if total == 0.0 {
            return Ok(());
        }
        
        let passed = self.results.passed as f64;
        
        self.results.compliance_summary.overall_percentage = (passed / total) * 100.0;
        
        // Categorize results by R7RS features
        self.analyze_feature_coverage();
        
        // Generate recommendations
        self.generate_recommendations();
        
        println!("  Overall compliance: {:.1}%", self.results.compliance_summary.overall_percentage);
        
        Ok(())
    }
    
    /// Analyze coverage by R7RS feature categories
    fn analyze_feature_coverage(&mut self) {
        // Group test results by features they test
        let mut feature_map: HashMap<String, Vec<&TestResult>> = HashMap::new();
        
        for result in &self.results.test_results {
            // Categorize tests based on file name and content
            let features = self.categorize_test_features(&result.test_name);
            for feature in features {
                feature_map.entry(feature).or_insert_with(Vec::new).push(result);
            }
        }
        
        // Calculate coverage for each feature
        for (feature_name, results) in feature_map {
            let total_tests = results.len();
            let passed_tests = results.iter().filter(|r| r.status == TestStatus::Passed).count();
            let coverage_percentage = if total_tests > 0 {
                (passed_tests as f64 / total_tests as f64) * 100.0
            } else {
                0.0
            };
            
            let implementation_status = match coverage_percentage {
                p if p >= 95.0 => ImplementationStatus::Complete,
                p if p >= 75.0 => ImplementationStatus::Partial,
                p if p >= 25.0 => ImplementationStatus::Minimal,
                _ => ImplementationStatus::Missing,
            };
            
            self.results.feature_coverage.insert(feature_name.clone(), FeatureCoverage {
                feature_name,
                total_tests,
                passed_tests,
                coverage_percentage,
                implementation_status,
                missing_procedures: Vec::new(), // TODO: Extract from error messages
                notes: Vec::new(),
            });
        }
    }
    
    /// Categorize which R7RS features a test covers
    fn categorize_test_features(&self, test_name: &str) -> Vec<String> {
        let mut features = Vec::new();
        
        // Basic mapping based on test file names
        if test_name.contains("fact") || test_name.contains("factorial") {
            features.push("arithmetic".to_string());
            features.push("recursion".to_string());
        }
        
        if test_name.contains("closure") {
            features.push("lexical-scoping".to_string());
            features.push("closures".to_string());
        }
        
        if test_name.contains("let") || test_name.contains("letrec") {
            features.push("binding-constructs".to_string());
        }
        
        if test_name.contains("apply") {
            features.push("higher-order-functions".to_string());
        }
        
        if test_name.contains("callcc") || test_name.contains("call-cc") {
            features.push("continuations".to_string());
        }
        
        if test_name.contains("hygiene") || test_name.contains("macro") {
            features.push("macro-system".to_string());
        }
        
        if test_name.contains("mutation") {
            features.push("mutation".to_string());
        }
        
        if test_name.contains("read") || test_name.contains("write") {
            features.push("io-operations".to_string());
        }
        
        // Default category
        if features.is_empty() {
            features.push("core-language".to_string());
        }
        
        features
    }
    
    /// Generate actionable recommendations
    fn generate_recommendations(&mut self) {
        let mut recommendations = Vec::new();
        
        if self.results.compliance_summary.overall_percentage < 70.0 {
            recommendations.push("Overall R7RS compliance is below 70%. Focus on implementing core language features.".to_string());
        }
        
        if self.results.failed > 0 {
            recommendations.push(format!("Address {} failed tests by implementing missing procedures and fixing evaluation errors.", self.results.failed));
        }
        
        if self.results.errors > 0 {
            recommendations.push(format!("Fix {} test execution errors, likely due to parser or evaluator issues.", self.results.errors));
        }
        
        // Feature-specific recommendations
        for (feature, coverage) in &self.results.feature_coverage {
            if coverage.implementation_status == ImplementationStatus::Missing {
                recommendations.push(format!("Implement missing feature: {}", feature));
            } else if coverage.implementation_status == ImplementationStatus::Minimal {
                recommendations.push(format!("Improve implementation of: {}", feature));
            }
        }
        
        self.results.compliance_summary.recommendations = recommendations;
    }
    
    /// Generate detailed reports
    fn generate_reports(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📄 Generating Reports");
        println!("-".repeat(40));
        
        let generator = report_generator::ReportGenerator::new(&self.results);
        
        // Generate JSON report
        let json_path = Path::new(&self.config.report_path).join("chibi_compliance_report.json");
        generator.generate_json_report(&json_path)?;
        println!("  Generated: {}", json_path.display());
        
        // Generate HTML report
        let html_path = Path::new(&self.config.report_path).join("chibi_compliance_report.html");
        generator.generate_html_report(&html_path)?;
        println!("  Generated: {}", html_path.display());
        
        // Generate markdown summary
        let md_path = Path::new(&self.config.report_path).join("chibi_compliance_summary.md");
        generator.generate_markdown_summary(&md_path)?;
        println!("  Generated: {}", md_path.display());
        
        Ok(())
    }
    
    /// Print execution summary
    fn print_summary(&self) {
        println!("\n🎯 Chibi-Scheme Integration Test Results");
        println!("=".repeat(60));
        
        println!("📊 Test Execution Summary:");
        println!("  Total tests:     {}", self.results.total_tests);
        println!("  Passed:          {} ({:.1}%)", self.results.passed,
                (self.results.passed as f64 / self.results.total_tests as f64) * 100.0);
        println!("  Failed:          {} ({:.1}%)", self.results.failed,
                (self.results.failed as f64 / self.results.total_tests as f64) * 100.0);
        println!("  Errors:          {} ({:.1}%)", self.results.errors,
                (self.results.errors as f64 / self.results.total_tests as f64) * 100.0);
        println!("  Execution time:  {:.2}s", self.results.execution_duration.as_secs_f64());
        
        println!("\n📈 R7RS Compliance Summary:");
        println!("  Overall:         {:.1}%", self.results.compliance_summary.overall_percentage);
        
        let compliance_grade = match self.results.compliance_summary.overall_percentage {
            p if p >= 95.0 => "A+ (Excellent)",
            p if p >= 90.0 => "A (Very Good)",
            p if p >= 85.0 => "B+ (Good)",
            p if p >= 80.0 => "B (Satisfactory)",
            p if p >= 70.0 => "C+ (Needs Work)",
            p if p >= 60.0 => "C (Major Gaps)",
            _ => "D (Incomplete)"
        };
        
        println!("  Grade:           {}", compliance_grade);
        
        if !self.results.compliance_summary.recommendations.is_empty() {
            println!("\n💡 Key Recommendations:");
            for (i, rec) in self.results.compliance_summary.recommendations.iter().take(5).enumerate() {
                println!("  {}. {}", i + 1, rec);
            }
        }
        
        println!("\n📄 Reports generated in: {}", self.config.report_path);
        println!("=".repeat(60));
    }
}

/// Information about a test file
#[derive(Debug, Clone)]
struct TestFileInfo {
    path: std::path::PathBuf,
    category: TestCategory,
    priority: TestPriority,
    estimated_tests: usize,
}

/// Category of test
#[derive(Debug, Clone, PartialEq)]
enum TestCategory {
    CoreR7RS,
    R5RSCompat,
    Syntax,
    StandardLibrary,
    BasicFunctionality,
    Memory,
    Performance,
}

/// Priority level for test execution
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
enum TestPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chibi_integration_suite_creation() {
        let suite = ChibiIntegrationSuite::new();
        assert_eq!(suite.results.total_tests, 0);
        assert!(suite.config.continue_on_error);
    }
    
    #[test]
    fn test_feature_categorization() {
        let suite = ChibiIntegrationSuite::new();
        
        let features = suite.categorize_test_features("test00-fact-3.scm");
        assert!(features.contains(&"arithmetic".to_string()));
        assert!(features.contains(&"recursion".to_string()));
        
        let features = suite.categorize_test_features("test02-closure.scm");
        assert!(features.contains(&"closures".to_string()));
    }
    
    #[test]
    fn test_configuration() {
        let config = ChibiIntegrationConfig {
            timeout_seconds: 60,
            include_performance: true,
            ..Default::default()
        };
        
        let suite = ChibiIntegrationSuite::with_config(config);
        assert_eq!(suite.config.timeout_seconds, 60);
        assert!(suite.config.include_performance);
    }
}
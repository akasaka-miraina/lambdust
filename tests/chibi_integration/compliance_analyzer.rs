//! Compliance Analyzer Module
//!
//! This module analyzes test results to determine R7RS compliance levels,
//! identify gaps in implementation, and provide detailed recommendations
//! for improving Lambdust's R7RS conformance.

use super::{TestResult, TestStatus, TestSuiteResults, FeatureCoverage, ComplianceSummary, ImplementationStatus};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Comprehensive R7RS compliance analyzer
pub struct ComplianceAnalyzer {
    feature_definitions: R7RSFeatureDefinitions,
    priority_matrix: CompliancePriorityMatrix,
}

impl ComplianceAnalyzer {
    /// Create a new compliance analyzer
    pub fn new() -> Self {
        Self {
            feature_definitions: R7RSFeatureDefinitions::load(),
            priority_matrix: CompliancePriorityMatrix::default(),
        }
    }
    
    /// Analyze test results and generate comprehensive compliance assessment
    pub fn analyze_compliance(&self, results: &TestSuiteResults) -> ComplianceAnalysis {
        println!("🔍 Analyzing R7RS Compliance");
        println!("-".repeat(40));
        
        // Categorize test results by R7RS features
        let feature_results = self.categorize_results_by_features(&results.test_results);
        
        // Calculate compliance metrics
        let compliance_metrics = self.calculate_compliance_metrics(&feature_results);
        
        // Identify critical gaps
        let critical_gaps = self.identify_critical_gaps(&feature_results);
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&feature_results, &critical_gaps);
        
        // Analyze implementation completeness
        let completeness_analysis = self.analyze_implementation_completeness(&feature_results);
        
        // Generate priority roadmap
        let priority_roadmap = self.generate_priority_roadmap(&feature_results);
        
        ComplianceAnalysis {
            overall_compliance: compliance_metrics,
            feature_analysis: feature_results,
            critical_gaps,
            recommendations,
            completeness_analysis,
            priority_roadmap,
            analysis_timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// Categorize test results by R7RS feature areas
    fn categorize_results_by_features(&self, results: &[TestResult]) -> HashMap<R7RSFeature, FeatureTestResults> {
        let mut feature_map: HashMap<R7RSFeature, FeatureTestResults> = HashMap::new();
        
        // Initialize all R7RS features
        for feature in self.feature_definitions.get_all_features() {
            feature_map.insert(feature.clone(), FeatureTestResults {
                feature: feature.clone(),
                total_tests: 0,
                passed_tests: 0,
                failed_tests: 0,
                error_tests: 0,
                timeout_tests: 0,
                test_details: Vec::new(),
                coverage_percentage: 0.0,
                implementation_status: ImplementationStatus::Missing,
                missing_procedures: Vec::new(),
                error_patterns: Vec::new(),
            });
        }
        
        // Categorize each test result
        for result in results {
            let features = self.identify_features_from_test(result);
            for feature in features {
                let feature_results = feature_map.get_mut(&feature).unwrap();
                
                feature_results.total_tests += 1;
                feature_results.test_details.push(result.clone());
                
                match result.status {
                    TestStatus::Passed => feature_results.passed_tests += 1,
                    TestStatus::Failed => {
                        feature_results.failed_tests += 1;
                        if let Some(error_msg) = &result.error_message {
                            feature_results.error_patterns.push(error_msg.clone());
                        }
                    },
                    TestStatus::Error => feature_results.error_tests += 1,
                    TestStatus::Timeout => feature_results.timeout_tests += 1,
                    TestStatus::Skipped => {},
                    TestStatus::Adapted => {},
                }
            }
        }
        
        // Calculate metrics for each feature
        for (_, feature_results) in feature_map.iter_mut() {
            if feature_results.total_tests > 0 {
                feature_results.coverage_percentage = 
                    (feature_results.passed_tests as f64 / feature_results.total_tests as f64) * 100.0;
                
                feature_results.implementation_status = match feature_results.coverage_percentage {
                    p if p >= 95.0 => ImplementationStatus::Complete,
                    p if p >= 75.0 => ImplementationStatus::Partial,
                    p if p >= 25.0 => ImplementationStatus::Minimal,
                    _ => ImplementationStatus::Missing,
                };
                
                // Identify missing procedures from error messages
                feature_results.missing_procedures = self.extract_missing_procedures(&feature_results.error_patterns);
            }
        }
        
        feature_map
    }
    
    /// Identify which R7RS features a test covers
    fn identify_features_from_test(&self, test: &TestResult) -> Vec<R7RSFeature> {
        let mut features = Vec::new();
        let test_name = test.test_name.to_lowercase();
        
        // Pattern matching based on test names and error messages
        if test_name.contains("fact") || test_name.contains("arithmetic") || test_name.contains("number") {
            features.push(R7RSFeature::NumericOperations);
        }
        
        if test_name.contains("list") || test_name.contains("car") || test_name.contains("cdr") {
            features.push(R7RSFeature::Lists);
        }
        
        if test_name.contains("string") {
            features.push(R7RSFeature::Strings);
        }
        
        if test_name.contains("vector") {
            features.push(R7RSFeature::Vectors);
        }
        
        if test_name.contains("closure") || test_name.contains("lambda") {
            features.push(R7RSFeature::Procedures);
        }
        
        if test_name.contains("let") || test_name.contains("binding") {
            features.push(R7RSFeature::BindingConstructs);
        }
        
        if test_name.contains("macro") || test_name.contains("syntax") {
            features.push(R7RSFeature::MacroSystem);
        }
        
        if test_name.contains("callcc") || test_name.contains("continuation") {
            features.push(R7RSFeature::Continuations);
        }
        
        if test_name.contains("io") || test_name.contains("read") || test_name.contains("write") {
            features.push(R7RSFeature::InputOutput);
        }
        
        if test_name.contains("exception") || test_name.contains("error") {
            features.push(R7RSFeature::ExceptionHandling);
        }
        
        if test_name.contains("r7rs") || test_name.contains("r5rs") {
            features.push(R7RSFeature::CoreLanguage);
        }
        
        // Check error messages for additional clues
        if let Some(error_msg) = &test.error_message {
            let error_lower = error_msg.to_lowercase();
            
            if error_lower.contains("undefined procedure") || error_lower.contains("unbound variable") {
                // Try to identify the missing feature from the procedure name
                if error_lower.contains("string-") {
                    features.push(R7RSFeature::Strings);
                } else if error_lower.contains("vector-") {
                    features.push(R7RSFeature::Vectors);
                } else if error_lower.contains("bytevector-") {
                    features.push(R7RSFeature::Bytevectors);
                }
            }
        }
        
        // Default to core language if no specific features identified
        if features.is_empty() {
            features.push(R7RSFeature::CoreLanguage);
        }
        
        features
    }
    
    /// Calculate overall compliance metrics
    fn calculate_compliance_metrics(&self, feature_results: &HashMap<R7RSFeature, FeatureTestResults>) -> ComplianceMetrics {
        let total_features = feature_results.len();
        let mut complete_features = 0;
        let mut partial_features = 0;
        let mut minimal_features = 0;
        let mut missing_features = 0;
        
        let mut total_tests = 0;
        let mut total_passed = 0;
        
        for (_, results) in feature_results {
            total_tests += results.total_tests;
            total_passed += results.passed_tests;
            
            match results.implementation_status {
                ImplementationStatus::Complete => complete_features += 1,
                ImplementationStatus::Partial => partial_features += 1,
                ImplementationStatus::Minimal => minimal_features += 1,
                ImplementationStatus::Missing => missing_features += 1,
                ImplementationStatus::Planned => {},
            }
        }
        
        let overall_percentage = if total_tests > 0 {
            (total_passed as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };
        
        let feature_completeness = if total_features > 0 {
            (complete_features as f64 / total_features as f64) * 100.0
        } else {
            0.0
        };
        
        ComplianceMetrics {
            overall_percentage,
            feature_completeness,
            total_features,
            complete_features,
            partial_features,
            minimal_features,
            missing_features,
            total_tests,
            total_passed,
            weighted_score: self.calculate_weighted_score(feature_results),
        }
    }
    
    /// Calculate weighted compliance score based on feature importance
    fn calculate_weighted_score(&self, feature_results: &HashMap<R7RSFeature, FeatureTestResults>) -> f64 {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        
        for (feature, results) in feature_results {
            let weight = self.priority_matrix.get_feature_weight(feature);
            let feature_score = results.coverage_percentage / 100.0;
            
            weighted_sum += weight * feature_score;
            total_weight += weight;
        }
        
        if total_weight > 0.0 {
            (weighted_sum / total_weight) * 100.0
        } else {
            0.0
        }
    }
    
    /// Identify critical gaps in R7RS compliance
    fn identify_critical_gaps(&self, feature_results: &HashMap<R7RSFeature, FeatureTestResults>) -> Vec<CriticalGap> {
        let mut gaps = Vec::new();
        
        for (feature, results) in feature_results {
            let priority = self.priority_matrix.get_feature_priority(feature);
            
            // Identify gaps based on priority and implementation status
            if matches!(priority, FeaturePriority::Critical) && 
               matches!(results.implementation_status, ImplementationStatus::Missing | ImplementationStatus::Minimal) {
                
                gaps.push(CriticalGap {
                    feature: feature.clone(),
                    severity: GapSeverity::Critical,
                    impact: self.assess_gap_impact(feature, results),
                    missing_procedures: results.missing_procedures.clone(),
                    recommended_action: self.recommend_gap_action(feature, results),
                    estimated_effort: self.estimate_implementation_effort(feature),
                });
            } else if matches!(priority, FeaturePriority::High) &&
                      matches!(results.implementation_status, ImplementationStatus::Missing) {
                
                gaps.push(CriticalGap {
                    feature: feature.clone(),
                    severity: GapSeverity::High,
                    impact: self.assess_gap_impact(feature, results),
                    missing_procedures: results.missing_procedures.clone(),
                    recommended_action: self.recommend_gap_action(feature, results),
                    estimated_effort: self.estimate_implementation_effort(feature),
                });
            }
        }
        
        // Sort gaps by severity and impact
        gaps.sort_by(|a, b| {
            a.severity.cmp(&b.severity)
                .then_with(|| b.impact.partial_cmp(&a.impact).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        gaps
    }
    
    /// Generate detailed recommendations for improving compliance
    fn generate_recommendations(&self, feature_results: &HashMap<R7RSFeature, FeatureTestResults>, gaps: &[CriticalGap]) -> Vec<ComplianceRecommendation> {
        let mut recommendations = Vec::new();
        
        // High-priority recommendations based on critical gaps
        for gap in gaps.iter().take(5) {
            recommendations.push(ComplianceRecommendation {
                priority: RecommendationPriority::High,
                category: RecommendationCategory::FeatureImplementation,
                title: format!("Implement missing {} features", gap.feature.as_str()),
                description: format!(
                    "Feature '{}' has {} severity gap with {} missing procedures. {}",
                    gap.feature.as_str(),
                    gap.severity.as_str(),
                    gap.missing_procedures.len(),
                    gap.recommended_action
                ),
                estimated_effort: gap.estimated_effort,
                expected_impact: gap.impact,
                specific_actions: self.generate_specific_actions_for_feature(&gap.feature),
            });
        }
        
        // Quick wins - features that are partially implemented
        for (feature, results) in feature_results {
            if results.implementation_status == ImplementationStatus::Partial &&
               results.coverage_percentage > 50.0 {
                
                recommendations.push(ComplianceRecommendation {
                    priority: RecommendationPriority::Medium,
                    category: RecommendationCategory::FeatureCompletion,
                    title: format!("Complete {} implementation", feature.as_str()),
                    description: format!(
                        "Feature '{}' is {:.1}% complete - finish remaining procedures for quick compliance gain",
                        feature.as_str(),
                        results.coverage_percentage
                    ),
                    estimated_effort: EstimatedEffort::Small,
                    expected_impact: 5.0,
                    specific_actions: vec![
                        format!("Implement {} missing procedures", results.missing_procedures.len()),
                        "Run targeted tests to verify completeness".to_string(),
                    ],
                });
            }
        }
        
        // Performance and stability recommendations
        let timeout_count: usize = feature_results.values().map(|r| r.timeout_tests).sum();
        let error_count: usize = feature_results.values().map(|r| r.error_tests).sum();
        
        if timeout_count > 0 {
            recommendations.push(ComplianceRecommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::Performance,
                title: "Address test timeout issues".to_string(),
                description: format!("{} tests are timing out, indicating performance or infinite loop issues", timeout_count),
                estimated_effort: EstimatedEffort::Medium,
                expected_impact: 3.0,
                specific_actions: vec![
                    "Profile slow operations".to_string(),
                    "Implement tail call optimization".to_string(),
                    "Fix potential infinite loops".to_string(),
                ],
            });
        }
        
        if error_count > 0 {
            recommendations.push(ComplianceRecommendation {
                priority: RecommendationPriority::High,
                category: RecommendationCategory::Stability,
                title: "Fix test execution errors".to_string(),
                description: format!("{} tests are failing with errors, indicating parser or evaluator issues", error_count),
                estimated_effort: EstimatedEffort::Large,
                expected_impact: 8.0,
                specific_actions: vec![
                    "Fix parser errors for unsupported syntax".to_string(),
                    "Implement missing evaluator cases".to_string(),
                    "Add proper error handling".to_string(),
                ],
            });
        }
        
        recommendations
    }
    
    /// Analyze implementation completeness across R7RS sections
    fn analyze_implementation_completeness(&self, feature_results: &HashMap<R7RSFeature, FeatureTestResults>) -> CompletenessAnalysis {
        let sections = self.group_features_by_r7rs_section(feature_results);
        
        let section_analysis: HashMap<String, SectionCompleteness> = sections
            .into_iter()
            .map(|(section_name, features)| {
                let total_features = features.len();
                let complete_features = features.iter()
                    .filter(|(_, r)| r.implementation_status == ImplementationStatus::Complete)
                    .count();
                
                let total_tests: usize = features.values().map(|r| r.total_tests).sum();
                let passed_tests: usize = features.values().map(|r| r.passed_tests).sum();
                
                let completeness_percentage = if total_features > 0 {
                    (complete_features as f64 / total_features as f64) * 100.0
                } else {
                    0.0
                };
                
                let test_pass_rate = if total_tests > 0 {
                    (passed_tests as f64 / total_tests as f64) * 100.0
                } else {
                    0.0
                };
                
                (section_name.clone(), SectionCompleteness {
                    section_name,
                    total_features,
                    complete_features,
                    completeness_percentage,
                    total_tests,
                    passed_tests,
                    test_pass_rate,
                    priority: self.get_section_priority(&section_name),
                })
            })
            .collect();
        
        CompletenessAnalysis {
            section_analysis,
            overall_readiness: self.calculate_overall_readiness(&section_analysis),
        }
    }
    
    /// Generate priority roadmap for implementation
    fn generate_priority_roadmap(&self, feature_results: &HashMap<R7RSFeature, FeatureTestResults>) -> PriorityRoadmap {
        let mut phases = Vec::new();
        
        // Phase 1: Critical features with high impact
        let phase1_features: Vec<_> = feature_results.iter()
            .filter(|(f, r)| {
                self.priority_matrix.get_feature_priority(f) == FeaturePriority::Critical &&
                matches!(r.implementation_status, ImplementationStatus::Missing | ImplementationStatus::Minimal)
            })
            .map(|(f, r)| RoadmapItem {
                feature: f.clone(),
                current_status: r.implementation_status.clone(),
                target_status: ImplementationStatus::Complete,
                estimated_effort: self.estimate_implementation_effort(f),
                dependencies: self.get_feature_dependencies(f),
                expected_impact: self.assess_gap_impact(f, r),
            })
            .collect();
        
        if !phase1_features.is_empty() {
            phases.push(RoadmapPhase {
                phase_number: 1,
                name: "Critical R7RS Core Features".to_string(),
                description: "Essential features required for basic R7RS compliance".to_string(),
                items: phase1_features,
                estimated_duration_weeks: 8,
                success_criteria: vec![
                    "All critical features at least 80% complete".to_string(),
                    "Core language features fully operational".to_string(),
                    "Basic arithmetic and list operations working".to_string(),
                ],
            });
        }
        
        // Phase 2: High-priority features and library completion
        let phase2_features: Vec<_> = feature_results.iter()
            .filter(|(f, r)| {
                self.priority_matrix.get_feature_priority(f) == FeaturePriority::High &&
                r.implementation_status != ImplementationStatus::Complete
            })
            .map(|(f, r)| RoadmapItem {
                feature: f.clone(),
                current_status: r.implementation_status.clone(),
                target_status: ImplementationStatus::Complete,
                estimated_effort: self.estimate_implementation_effort(f),
                dependencies: self.get_feature_dependencies(f),
                expected_impact: self.assess_gap_impact(f, r),
            })
            .collect();
        
        if !phase2_features.is_empty() {
            phases.push(RoadmapPhase {
                phase_number: 2,
                name: "Standard Library Implementation".to_string(),
                description: "Complete standard library procedures and advanced features".to_string(),
                items: phase2_features,
                estimated_duration_weeks: 12,
                success_criteria: vec![
                    "All high-priority features complete".to_string(),
                    "Standard library 90% compliant".to_string(),
                    "Advanced features like macros and continuations working".to_string(),
                ],
            });
        }
        
        // Phase 3: Optional and optimization features
        let phase3_features: Vec<_> = feature_results.iter()
            .filter(|(f, r)| {
                matches!(self.priority_matrix.get_feature_priority(f), FeaturePriority::Medium | FeaturePriority::Low) &&
                r.implementation_status != ImplementationStatus::Complete
            })
            .map(|(f, r)| RoadmapItem {
                feature: f.clone(),
                current_status: r.implementation_status.clone(),
                target_status: ImplementationStatus::Complete,
                estimated_effort: self.estimate_implementation_effort(f),
                dependencies: self.get_feature_dependencies(f),
                expected_impact: self.assess_gap_impact(f, r),
            })
            .collect();
        
        if !phase3_features.is_empty() {
            phases.push(RoadmapPhase {
                phase_number: 3,
                name: "Completeness and Optimization".to_string(),
                description: "Complete remaining features and optimize performance".to_string(),
                items: phase3_features,
                estimated_duration_weeks: 6,
                success_criteria: vec![
                    "95%+ R7RS compliance achieved".to_string(),
                    "Performance optimized".to_string(),
                    "Full test suite passing".to_string(),
                ],
            });
        }
        
        let total_weeks: u32 = phases.iter().map(|p| p.estimated_duration_weeks).sum();
        PriorityRoadmap {
            phases,
            total_estimated_weeks: total_weeks,
            confidence_level: 0.75, // 75% confidence in estimates
        }
    }
    
    // Helper methods for analysis
    
    fn extract_missing_procedures(&self, error_patterns: &[String]) -> Vec<String> {
        let mut procedures = Vec::new();
        
        for error in error_patterns {
            // Look for "undefined procedure" or "unbound variable" patterns
            if let Some(start) = error.find("undefined procedure: ") {
                if let Some(proc_name) = error[start + 21..].split_whitespace().next() {
                    procedures.push(proc_name.trim_matches(&['(', ')', '\'', '"']).to_string());
                }
            } else if let Some(start) = error.find("unbound variable: ") {
                if let Some(var_name) = error[start + 18..].split_whitespace().next() {
                    procedures.push(var_name.trim_matches(&['(', ')', '\'', '"']).to_string());
                }
            }
        }
        
        procedures.sort();
        procedures.dedup();
        procedures
    }
    
    fn assess_gap_impact(&self, feature: &R7RSFeature, _results: &FeatureTestResults) -> f64 {
        // Assess the impact of missing this feature on overall compliance
        match feature {
            R7RSFeature::CoreLanguage => 10.0,
            R7RSFeature::NumericOperations => 8.0,
            R7RSFeature::Lists => 8.0,
            R7RSFeature::Procedures => 9.0,
            R7RSFeature::BindingConstructs => 7.0,
            R7RSFeature::Strings => 6.0,
            R7RSFeature::Vectors => 5.0,
            R7RSFeature::InputOutput => 6.0,
            R7RSFeature::MacroSystem => 4.0,
            R7RSFeature::Continuations => 3.0,
            R7RSFeature::ExceptionHandling => 4.0,
            R7RSFeature::Bytevectors => 3.0,
            R7RSFeature::Records => 2.0,
            R7RSFeature::Libraries => 5.0,
        }
    }
    
    fn recommend_gap_action(&self, feature: &R7RSFeature, results: &FeatureTestResults) -> String {
        match results.implementation_status {
            ImplementationStatus::Missing => {
                format!("Implement {} from scratch, focusing on {} core procedures", 
                       feature.as_str(), results.missing_procedures.len())
            },
            ImplementationStatus::Minimal => {
                format!("Expand {} implementation to cover more edge cases and procedures", 
                       feature.as_str())
            },
            ImplementationStatus::Partial => {
                format!("Complete {} implementation by adding remaining {} procedures", 
                       feature.as_str(), results.missing_procedures.len())
            },
            _ => format!("Review and optimize {} implementation", feature.as_str()),
        }
    }
    
    fn estimate_implementation_effort(&self, feature: &R7RSFeature) -> EstimatedEffort {
        match feature {
            R7RSFeature::CoreLanguage => EstimatedEffort::Large,
            R7RSFeature::MacroSystem => EstimatedEffort::Large,
            R7RSFeature::Continuations => EstimatedEffort::Large,
            R7RSFeature::NumericOperations => EstimatedEffort::Medium,
            R7RSFeature::Lists => EstimatedEffort::Medium,
            R7RSFeature::Strings => EstimatedEffort::Medium,
            R7RSFeature::InputOutput => EstimatedEffort::Medium,
            R7RSFeature::ExceptionHandling => EstimatedEffort::Medium,
            R7RSFeature::Procedures => EstimatedEffort::Small,
            R7RSFeature::BindingConstructs => EstimatedEffort::Small,
            R7RSFeature::Vectors => EstimatedEffort::Small,
            R7RSFeature::Bytevectors => EstimatedEffort::Small,
            R7RSFeature::Records => EstimatedEffort::Small,
            R7RSFeature::Libraries => EstimatedEffort::Medium,
        }
    }
    
    fn generate_specific_actions_for_feature(&self, feature: &R7RSFeature) -> Vec<String> {
        match feature {
            R7RSFeature::NumericOperations => vec![
                "Implement exact arithmetic operations".to_string(),
                "Add complex number support".to_string(),
                "Implement rational number arithmetic".to_string(),
                "Add transcendental functions".to_string(),
            ],
            R7RSFeature::Lists => vec![
                "Implement all list manipulation procedures".to_string(),
                "Add proper tail recursion for list operations".to_string(),
                "Implement list? and proper-list? predicates".to_string(),
            ],
            R7RSFeature::MacroSystem => vec![
                "Implement syntax-rules macro system".to_string(),
                "Add hygienic macro expansion".to_string(),
                "Implement syntax-case (optional)".to_string(),
            ],
            _ => vec![format!("Review R7RS specification for {}", feature.as_str())],
        }
    }
    
    fn group_features_by_r7rs_section<'a>(&self, feature_results: &'a HashMap<R7RSFeature, FeatureTestResults>) -> HashMap<String, HashMap<R7RSFeature, &'a FeatureTestResults>> {
        let mut sections = HashMap::new();
        
        for (feature, results) in feature_results {
            let section = match feature {
                R7RSFeature::CoreLanguage => "4. Program Structure",
                R7RSFeature::Procedures => "4.1 Primitive Expression Types",
                R7RSFeature::BindingConstructs => "4.2 Binding Constructs",
                R7RSFeature::NumericOperations => "6.2 Numbers",
                R7RSFeature::Lists => "6.4 Pairs and Lists",
                R7RSFeature::Strings => "6.7 Strings",
                R7RSFeature::Vectors => "6.8 Vectors",
                R7RSFeature::Bytevectors => "6.9 Bytevectors",
                R7RSFeature::InputOutput => "6.13 Input and Output",
                R7RSFeature::MacroSystem => "4.3 Macros",
                R7RSFeature::Continuations => "6.10 Control Features",
                R7RSFeature::ExceptionHandling => "6.11 Exceptions",
                R7RSFeature::Records => "5.5 Record-type Definitions",
                R7RSFeature::Libraries => "5 Program Structure",
            };
            
            sections.entry(section.to_string())
                .or_insert_with(HashMap::new)
                .insert(feature.clone(), results);
        }
        
        sections
    }
    
    fn get_section_priority(&self, section: &str) -> FeaturePriority {
        match section {
            s if s.contains("Program Structure") => FeaturePriority::Critical,
            s if s.contains("Primitive Expression") => FeaturePriority::Critical,
            s if s.contains("Numbers") => FeaturePriority::Critical,
            s if s.contains("Pairs and Lists") => FeaturePriority::Critical,
            s if s.contains("Strings") => FeaturePriority::High,
            s if s.contains("Input and Output") => FeaturePriority::High,
            s if s.contains("Macros") => FeaturePriority::Medium,
            s if s.contains("Control Features") => FeaturePriority::Medium,
            _ => FeaturePriority::Low,
        }
    }
    
    fn calculate_overall_readiness(&self, sections: &HashMap<String, SectionCompleteness>) -> f64 {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        
        for (_, section) in sections {
            let weight = match section.priority {
                FeaturePriority::Critical => 4.0,
                FeaturePriority::High => 3.0,
                FeaturePriority::Medium => 2.0,
                FeaturePriority::Low => 1.0,
            };
            
            weighted_sum += weight * section.completeness_percentage;
            total_weight += weight;
        }
        
        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }
    
    fn get_feature_dependencies(&self, feature: &R7RSFeature) -> Vec<R7RSFeature> {
        match feature {
            R7RSFeature::MacroSystem => vec![R7RSFeature::CoreLanguage],
            R7RSFeature::Continuations => vec![R7RSFeature::CoreLanguage, R7RSFeature::Procedures],
            R7RSFeature::ExceptionHandling => vec![R7RSFeature::CoreLanguage],
            R7RSFeature::Libraries => vec![R7RSFeature::CoreLanguage, R7RSFeature::MacroSystem],
            R7RSFeature::Records => vec![R7RSFeature::CoreLanguage, R7RSFeature::MacroSystem],
            _ => Vec::new(),
        }
    }
}

// Data structures for compliance analysis

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAnalysis {
    pub overall_compliance: ComplianceMetrics,
    pub feature_analysis: HashMap<R7RSFeature, FeatureTestResults>,
    pub critical_gaps: Vec<CriticalGap>,
    pub recommendations: Vec<ComplianceRecommendation>,
    pub completeness_analysis: CompletenessAnalysis,
    pub priority_roadmap: PriorityRoadmap,
    pub analysis_timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMetrics {
    pub overall_percentage: f64,
    pub feature_completeness: f64,
    pub total_features: usize,
    pub complete_features: usize,
    pub partial_features: usize,
    pub minimal_features: usize,
    pub missing_features: usize,
    pub total_tests: usize,
    pub total_passed: usize,
    pub weighted_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureTestResults {
    pub feature: R7RSFeature,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub error_tests: usize,
    pub timeout_tests: usize,
    pub test_details: Vec<TestResult>,
    pub coverage_percentage: f64,
    pub implementation_status: ImplementationStatus,
    pub missing_procedures: Vec<String>,
    pub error_patterns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum R7RSFeature {
    CoreLanguage,
    NumericOperations,
    Lists,
    Strings,
    Vectors,
    Bytevectors,
    Procedures,
    BindingConstructs,
    MacroSystem,
    Continuations,
    ExceptionHandling,
    InputOutput,
    Records,
    Libraries,
}

impl R7RSFeature {
    pub fn as_str(&self) -> &'static str {
        match self {
            R7RSFeature::CoreLanguage => "Core Language",
            R7RSFeature::NumericOperations => "Numeric Operations",
            R7RSFeature::Lists => "Lists",
            R7RSFeature::Strings => "Strings",
            R7RSFeature::Vectors => "Vectors",
            R7RSFeature::Bytevectors => "Bytevectors",
            R7RSFeature::Procedures => "Procedures",
            R7RSFeature::BindingConstructs => "Binding Constructs",
            R7RSFeature::MacroSystem => "Macro System",
            R7RSFeature::Continuations => "Continuations",
            R7RSFeature::ExceptionHandling => "Exception Handling",
            R7RSFeature::InputOutput => "Input/Output",
            R7RSFeature::Records => "Records",
            R7RSFeature::Libraries => "Libraries",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalGap {
    pub feature: R7RSFeature,
    pub severity: GapSeverity,
    pub impact: f64,
    pub missing_procedures: Vec<String>,
    pub recommended_action: String,
    pub estimated_effort: EstimatedEffort,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub enum GapSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl GapSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            GapSeverity::Critical => "critical",
            GapSeverity::High => "high",
            GapSeverity::Medium => "medium", 
            GapSeverity::Low => "low",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRecommendation {
    pub priority: RecommendationPriority,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub estimated_effort: EstimatedEffort,
    pub expected_impact: f64,
    pub specific_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    FeatureImplementation,
    FeatureCompletion,
    Performance,
    Stability,
    Testing,
    Documentation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EstimatedEffort {
    Small,   // 1-2 weeks
    Medium,  // 2-4 weeks  
    Large,   // 4-8 weeks
    XLarge,  // 8+ weeks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessAnalysis {
    pub section_analysis: HashMap<String, SectionCompleteness>,
    pub overall_readiness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionCompleteness {
    pub section_name: String,
    pub total_features: usize,
    pub complete_features: usize,
    pub completeness_percentage: f64,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub test_pass_rate: f64,
    pub priority: FeaturePriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityRoadmap {
    pub phases: Vec<RoadmapPhase>,
    pub total_estimated_weeks: u32,
    pub confidence_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapPhase {
    pub phase_number: u32,
    pub name: String,
    pub description: String,
    pub items: Vec<RoadmapItem>,
    pub estimated_duration_weeks: u32,
    pub success_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapItem {
    pub feature: R7RSFeature,
    pub current_status: ImplementationStatus,
    pub target_status: ImplementationStatus,
    pub estimated_effort: EstimatedEffort,
    pub dependencies: Vec<R7RSFeature>,
    pub expected_impact: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FeaturePriority {
    Critical,
    High,
    Medium,
    Low,
}

// Support structures

pub struct R7RSFeatureDefinitions {
    features: Vec<R7RSFeature>,
}

impl R7RSFeatureDefinitions {
    pub fn load() -> Self {
        Self {
            features: vec![
                R7RSFeature::CoreLanguage,
                R7RSFeature::NumericOperations,
                R7RSFeature::Lists,
                R7RSFeature::Strings,
                R7RSFeature::Vectors,
                R7RSFeature::Bytevectors,
                R7RSFeature::Procedures,
                R7RSFeature::BindingConstructs,
                R7RSFeature::MacroSystem,
                R7RSFeature::Continuations,
                R7RSFeature::ExceptionHandling,
                R7RSFeature::InputOutput,
                R7RSFeature::Records,
                R7RSFeature::Libraries,
            ],
        }
    }
    
    pub fn get_all_features(&self) -> &[R7RSFeature] {
        &self.features
    }
}

pub struct CompliancePriorityMatrix {
    priorities: HashMap<R7RSFeature, FeaturePriority>,
    weights: HashMap<R7RSFeature, f64>,
}

impl Default for CompliancePriorityMatrix {
    fn default() -> Self {
        let mut priorities = HashMap::new();
        let mut weights = HashMap::new();
        
        // Critical features
        priorities.insert(R7RSFeature::CoreLanguage, FeaturePriority::Critical);
        weights.insert(R7RSFeature::CoreLanguage, 4.0);
        
        priorities.insert(R7RSFeature::NumericOperations, FeaturePriority::Critical);
        weights.insert(R7RSFeature::NumericOperations, 4.0);
        
        priorities.insert(R7RSFeature::Lists, FeaturePriority::Critical);
        weights.insert(R7RSFeature::Lists, 4.0);
        
        priorities.insert(R7RSFeature::Procedures, FeaturePriority::Critical);
        weights.insert(R7RSFeature::Procedures, 4.0);
        
        // High priority features
        priorities.insert(R7RSFeature::BindingConstructs, FeaturePriority::High);
        weights.insert(R7RSFeature::BindingConstructs, 3.0);
        
        priorities.insert(R7RSFeature::Strings, FeaturePriority::High);
        weights.insert(R7RSFeature::Strings, 3.0);
        
        priorities.insert(R7RSFeature::InputOutput, FeaturePriority::High);
        weights.insert(R7RSFeature::InputOutput, 3.0);
        
        // Medium priority features
        priorities.insert(R7RSFeature::Vectors, FeaturePriority::Medium);
        weights.insert(R7RSFeature::Vectors, 2.0);
        
        priorities.insert(R7RSFeature::ExceptionHandling, FeaturePriority::Medium);
        weights.insert(R7RSFeature::ExceptionHandling, 2.0);
        
        priorities.insert(R7RSFeature::MacroSystem, FeaturePriority::Medium);
        weights.insert(R7RSFeature::MacroSystem, 2.0);
        
        priorities.insert(R7RSFeature::Libraries, FeaturePriority::Medium);
        weights.insert(R7RSFeature::Libraries, 2.0);
        
        // Low priority features
        priorities.insert(R7RSFeature::Continuations, FeaturePriority::Low);
        weights.insert(R7RSFeature::Continuations, 1.0);
        
        priorities.insert(R7RSFeature::Bytevectors, FeaturePriority::Low);
        weights.insert(R7RSFeature::Bytevectors, 1.0);
        
        priorities.insert(R7RSFeature::Records, FeaturePriority::Low);
        weights.insert(R7RSFeature::Records, 1.0);
        
        Self {
            priorities,
            weights,
        }
    }
}

impl CompliancePriorityMatrix {
    pub fn get_feature_priority(&self, feature: &R7RSFeature) -> FeaturePriority {
        self.priorities.get(feature).cloned().unwrap_or(FeaturePriority::Low)
    }
    
    pub fn get_feature_weight(&self, feature: &R7RSFeature) -> f64 {
        self.weights.get(feature).copied().unwrap_or(1.0)
    }
}
//! Hot Path Analysis Module
//!
//! このモジュールは高度なホットパス検出・解析システムを実装します。
//! 多次元実行頻度解析、コールグラフ分析、メモリアクセスパターン検出、
//! 分岐予測、ループ特性解析、適応的閾値管理を含みます。
//!
//! ## モジュール構成
//!
//! - `core_types`: 基本型定義とデータ構造
//! - `frequency_tracker`: 実行頻度追跡システム
//! - `call_graph_analyzer`: コールグラフ構築・解析
//! - `memory_analyzer`: メモリアクセスパターン解析
//! - `branch_predictor`: 分岐予測システム
//! - `loop_analyzer`: ループ特性解析
//! - `performance_detector`: パフォーマンス回帰検出・最適化推奨

pub mod core_types;
pub mod frequency_tracker;
pub mod call_graph_analyzer;
pub mod memory_analyzer;
pub mod branch_predictor;
pub mod loop_analyzer;
pub mod performance_detector;

// Re-export main types for backward compatibility
pub use core_types::{
    AdvancedHotPathDetector, ExecutionRecord, MemoryAllocation, AllocationType,
    CallStackContext, BindingContext, ModuleContext, MemoryLocation, MemoryAccessType,
    StridePattern, MemoryAccessPattern, BranchHistory, LoopCharacteristics, LoopDependency,
    DependencyType, DynamicThresholds, ThresholdAdaptation, AdaptiveThresholdManager,
    HotPathCategory, ClassificationRule, ClassificationResult, HotPathClassifier,
    HotPathAnalysis, OptimizationRecommendation, OptimizationType,
    PerformanceOptimizationReport, PerformanceRegressionAlert, RegressionSeverity,
};

pub use frequency_tracker::{
    FrequencyTracker, TemporalFrequencyAnalysis, ContextualFrequencyTracker,
    FrequencyStatistics,
};

pub use call_graph_analyzer::{
    CallGraphAnalyzer, CallGraphInfo, CallGraphComplexity, CallHotspot,
};

pub use memory_analyzer::{
    MemoryAccessAnalyzer, MemoryStatistics, MemoryHotspot, LocalityAnalysis,
};

pub use branch_predictor::{
    BranchPredictor, BranchPrediction, PredictionMethod, BranchStatistics, BranchHotspot,
};

pub use loop_analyzer::{
    LoopCharacteristicsAnalyzer, LoopOptimizationOpportunity, LoopOptimizationType,
    OptimizationComplexity, LoopStatistics, LoopHotspot,
};

pub use performance_detector::{
    PerformanceRegressionDetector, PerformanceMeasurement, RegressionDetectionConfig,
    PerformanceTrend, TrendDirection,
};

use crate::ast::Expr;
use crate::error::Result;
use crate::value::Value;
use std::time::Duration;

impl core_types::AdvancedHotPathDetector {
    /// Create new advanced hot path detector
    #[must_use] 
    pub fn new() -> Self {
        Self {
            frequency_tracker: frequency_tracker::FrequencyTracker::new(),
            call_graph: call_graph_analyzer::CallGraphAnalyzer::new(),
            memory_analyzer: memory_analyzer::MemoryAccessAnalyzer::new(),
            branch_predictor: branch_predictor::BranchPredictor::new(),
            loop_analyzer: loop_analyzer::LoopCharacteristicsAnalyzer::new(),
            threshold_manager: core_types::AdaptiveThresholdManager::new(),
            regression_detector: performance_detector::PerformanceRegressionDetector::new(),
            hotpath_classifier: core_types::HotPathClassifier::new(),
        }
    }
    
    /// Record execution with comprehensive analysis
    pub fn record_execution(
        &mut self,
        expr: &Expr,
        execution_time: Duration,
        memory_usage: usize,
        return_value: &Value,
        call_stack: &[String],
    ) -> Result<()> {
        let expr_hash = self.compute_expression_hash(expr);
        
        // Update frequency tracking
        self.frequency_tracker.record_execution(
            &expr_hash,
            execution_time,
            memory_usage,
            return_value,
        )?;
        
        // Update call graph
        if !call_stack.is_empty() {
            self.call_graph.record_call(&call_stack[call_stack.len()-1], &expr_hash)?;
        }
        
        // Analyze memory access patterns
        self.memory_analyzer.analyze_access_pattern(&expr_hash, expr, memory_usage)?;
        
        // Update branch prediction (if applicable)
        if let Some(branch_info) = self.extract_branch_info(expr, return_value) {
            self.branch_predictor.record_branch(&expr_hash, branch_info)?;
        }
        
        // Analyze loop characteristics (if applicable)
        if self.is_loop_expression(expr) {
            self.loop_analyzer.analyze_loop(&expr_hash, expr, execution_time)?;
        }
        
        // Record performance measurement for regression detection
        let measurement = performance_detector::PerformanceMeasurement {
            execution_time,
            memory_usage,
            timestamp: std::time::SystemTime::now(),
            cpu_cycles: None, // Could be populated with actual cycle counts
            cache_misses: None, // Could be populated with actual cache miss counts
        };
        self.regression_detector.record_measurement(&expr_hash, measurement);
        
        // Classify hot path
        self.classify_hotpath(&expr_hash, expr)?;
        
        // Adapt thresholds based on performance
        self.threshold_manager.adapt_thresholds(&self.frequency_tracker)?;
        
        Ok(())
    }
    
    /// Get comprehensive hot path analysis
    #[must_use]
    pub fn get_hotpath_analysis(&self, expr_hash: &str) -> Option<core_types::HotPathAnalysis> {
        let execution_record = self.frequency_tracker.execution_counts.get(expr_hash)?;
        let call_context = self.frequency_tracker.context_tracker.get_call_context(expr_hash);
        let memory_pattern = self.memory_analyzer.access_patterns.get(expr_hash);
        let branch_info = self.branch_predictor.branch_history.get(expr_hash);
        let loop_info = self.loop_analyzer.loops.get(expr_hash);
        let classification = self.hotpath_classifier.get_classification(expr_hash);
        
        Some(core_types::HotPathAnalysis {
            expression_hash: expr_hash.to_string(),
            execution_record: execution_record.clone(),
            call_context: call_context.cloned(),
            memory_pattern: memory_pattern.cloned(),
            branch_history: branch_info.cloned(),
            loop_characteristics: loop_info.cloned(),
            hotpath_score: self.calculate_hotpath_score(expr_hash),
            classification: classification.cloned(),
            optimization_recommendations: self.generate_optimization_recommendations(expr_hash),
        })
    }
    
    /// Check if expression is currently a hot path
    #[must_use]
    pub fn is_hotpath(&self, expr_hash: &str) -> bool {
        if let Some(record) = self.frequency_tracker.execution_counts.get(expr_hash) {
            let thresholds = &self.threshold_manager.thresholds;
            
            // Multi-criteria hot path detection
            let frequency_hot = record.total_executions >= thresholds.hotpath_threshold;
            let time_critical = record.average_execution_time >= thresholds.time_threshold;
            let memory_intensive = record.memory_allocations.iter()
                .map(|alloc| alloc.size)
                .sum::<usize>() >= thresholds.memory_threshold;
            
            // Combined criteria with adaptive weighting
            frequency_hot || (time_critical && memory_intensive)
        } else {
            false
        }
    }
    
    /// Get top hot paths with detailed analysis
    #[must_use]
    pub fn get_top_hotpaths(&self, limit: usize) -> Vec<core_types::HotPathAnalysis> {
        let mut hotpaths: Vec<_> = self.frequency_tracker.execution_counts
            .keys()
            .filter_map(|expr_hash| self.get_hotpath_analysis(expr_hash))
            .collect();
        
        // Sort by hot path score (descending)
        hotpaths.sort_by(|a, b| b.hotpath_score.partial_cmp(&a.hotpath_score).unwrap_or(std::cmp::Ordering::Equal));
        
        hotpaths.into_iter().take(limit).collect()
    }
    
    /// Generate performance optimization report
    #[must_use]
    pub fn generate_performance_report(&self) -> core_types::PerformanceOptimizationReport {
        core_types::PerformanceOptimizationReport {
            total_expressions_analyzed: self.frequency_tracker.execution_counts.len(),
            hotpath_count: self.frequency_tracker.execution_counts.keys()
                .filter(|expr| self.is_hotpath(expr))
                .count(),
            top_hotpaths: self.get_top_hotpaths(10),
            call_graph_complexity: {
                let complexity = self.call_graph.calculate_complexity();
                // Convert to a single complexity score
                complexity.node_count as f64 + complexity.edge_count as f64 * 0.5 + 
                complexity.max_depth as f64 * 2.0 + complexity.avg_out_degree * 1.5
            },
            memory_efficiency_score: self.memory_analyzer.calculate_efficiency_score(),
            branch_prediction_accuracy: self.branch_predictor.calculate_overall_accuracy(),
            loop_optimization_opportunities: self.loop_analyzer.identify_optimization_opportunities()
                .into_iter().map(|opp| opp.loop_id).collect(),
            threshold_adaptation_history: self.threshold_manager.adaptation_history.clone(),
            performance_regression_alerts: self.regression_detector.get_alerts(),
            optimization_recommendations: self.generate_global_optimization_recommendations(),
        }
    }
    
    // Helper methods
    
    fn compute_expression_hash(&self, expr: &Expr) -> String {
        format!("{:?}", expr)
    }
    
    fn extract_branch_info(&self, _expr: &Expr, return_value: &Value) -> Option<bool> {
        // Extract branch information from expression evaluation
        match return_value {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    
    fn is_loop_expression(&self, expr: &Expr) -> bool {
        // Detect loop expressions (do, let with recursion, etc.)
        match expr {
            Expr::List(exprs) if !exprs.is_empty() => {
                match &exprs[0] {
                    Expr::Variable(name) => matches!(name.as_str(), "do" | "while" | "for"),
                    _ => false,
                }
            }
            _ => false,
        }
    }
    
    fn classify_hotpath(&mut self, expr_hash: &str, _expr: &Expr) -> Result<()> {
        // Classify hot path using ML and rule-based approaches
        self.hotpath_classifier.classify(expr_hash, &self.frequency_tracker, &self.memory_analyzer)
    }
    
    fn calculate_hotpath_score(&self, expr_hash: &str) -> f64 {
        if let Some(record) = self.frequency_tracker.execution_counts.get(expr_hash) {
            let frequency_score = record.total_executions as f64;
            let time_score = record.average_execution_time.as_nanos() as f64 / 1_000_000.0; // Convert to ms
            let memory_score = record.memory_allocations.iter()
                .map(|alloc| alloc.size as f64)
                .sum::<f64>();
            
            // Weighted combination
            frequency_score * 0.4 + time_score * 0.3 + memory_score * 0.3
        } else {
            0.0
        }
    }
    
    fn generate_optimization_recommendations(&self, expr_hash: &str) -> Vec<core_types::OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        if let Some(analysis) = self.get_hotpath_analysis(expr_hash) {
            // Generate recommendations based on analysis
            if analysis.execution_record.total_executions > 1000 {
                recommendations.push(core_types::OptimizationRecommendation {
                    optimization_type: core_types::OptimizationType::JITCompilation,
                    confidence: 0.9,
                    expected_speedup: 2.5,
                    description: "High execution frequency detected - JIT compilation recommended".to_string(),
                });
            }
            
            if let Some(loop_chars) = &analysis.loop_characteristics {
                if loop_chars.unroll_potential > 1 {
                    recommendations.push(core_types::OptimizationRecommendation {
                        optimization_type: core_types::OptimizationType::LoopUnrolling,
                        confidence: 0.8,
                        expected_speedup: 1.0 + (loop_chars.unroll_potential as f64 * 0.2),
                        description: format!("Loop unrolling by factor {} recommended", loop_chars.unroll_potential),
                    });
                }
            }
        }
        
        // Add performance detector recommendations
        recommendations.extend(self.regression_detector.generate_optimization_recommendations(expr_hash));
        
        recommendations
    }
    
    fn generate_global_optimization_recommendations(&self) -> Vec<core_types::OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        // System-wide recommendations based on overall statistics
        let _freq_stats = self.frequency_tracker.calculate_statistics();
        let memory_stats = self.memory_analyzer.get_memory_statistics();
        let branch_stats = self.branch_predictor.get_branch_statistics();
        let loop_stats = self.loop_analyzer.get_loop_statistics();
        
        // High memory usage across the system
        if memory_stats.average_cache_miss_rate > 0.1 {
            recommendations.push(core_types::OptimizationRecommendation {
                optimization_type: core_types::OptimizationType::CacheOptimization,
                confidence: 0.7,
                expected_speedup: 1.3,
                description: "High system-wide cache miss rate - cache optimization recommended".to_string(),
            });
        }
        
        // Many vectorizable loops
        if loop_stats.vectorizable_loops > 5 {
            recommendations.push(core_types::OptimizationRecommendation {
                optimization_type: core_types::OptimizationType::Vectorization,
                confidence: 0.8,
                expected_speedup: 2.0,
                description: format!("{} vectorizable loops detected - SIMD optimization recommended", loop_stats.vectorizable_loops),
            });
        }
        
        // Poor branch prediction accuracy
        if branch_stats.overall_prediction_accuracy < 0.7 {
            recommendations.push(core_types::OptimizationRecommendation {
                optimization_type: core_types::OptimizationType::BranchOptimization,
                confidence: 0.6,
                expected_speedup: 1.2,
                description: "Poor branch prediction accuracy - branch optimization recommended".to_string(),
            });
        }
        
        recommendations
    }
}

// Implement required trait methods for core types
impl core_types::AdaptiveThresholdManager {
    #[must_use] 
    pub fn new() -> Self { 
        Self { 
            thresholds: core_types::DynamicThresholds::default(), 
            adaptation_history: Vec::new(), 
            feedback_system: core_types::ThresholdFeedbackSystem, 
            auto_tuner: core_types::ThresholdAutoTuner,
        } 
    }
    
    pub fn adapt_thresholds(&mut self, frequency_tracker: &frequency_tracker::FrequencyTracker) -> Result<()> {
        // Simple adaptation logic (placeholder)
        let total_executions: u64 = frequency_tracker.execution_counts.values()
            .map(|record| record.total_executions)
            .sum();
        
        if total_executions > 10000 {
            let old_thresholds = self.thresholds.clone();
            self.thresholds.hotpath_threshold = (self.thresholds.hotpath_threshold as f64 * 1.1) as u64;
            
            self.adaptation_history.push(core_types::ThresholdAdaptation {
                timestamp: std::time::SystemTime::now(),
                old_thresholds,
                new_thresholds: self.thresholds.clone(),
                reason: "High execution volume detected".to_string(),
                expected_impact: 0.05,
            });
        }
        
        Ok(())
    }
}

impl core_types::HotPathClassifier {
    #[must_use] 
    pub fn new() -> Self { 
        Self { 
            classification_rules: Vec::new(), 
            ml_classifier: core_types::MLHotPathClassifier, 
            pattern_recognizer: core_types::PatternRecognizer, 
            classification_history: Vec::new(),
        } 
    }
    
    pub fn classify(&mut self, expr_hash: &str, frequency_tracker: &frequency_tracker::FrequencyTracker, _memory_analyzer: &memory_analyzer::MemoryAccessAnalyzer) -> Result<()> {
        if let Some(record) = frequency_tracker.execution_counts.get(expr_hash) {
            let category = if record.total_executions > 1000 {
                core_types::HotPathCategory::CPUIntensive
            } else if record.memory_allocations.len() > 100 {
                core_types::HotPathCategory::MemoryIntensive
            } else {
                core_types::HotPathCategory::CacheFriendly
            };
            
            let result = core_types::ClassificationResult {
                expression: expr_hash.to_string(),
                category,
                confidence: 0.8,
                timestamp: std::time::SystemTime::now(),
                factors: vec!["execution_frequency".to_string()],
            };
            
            self.classification_history.push(result);
        }
        
        Ok(())
    }
    
    pub fn get_classification(&self, expr_hash: &str) -> Option<&core_types::ClassificationResult> {
        self.classification_history.iter()
            .rev()
            .find(|result| result.expression == expr_hash)
    }
}

impl Default for core_types::AdvancedHotPathDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new advanced hot path detector with default configuration
pub fn create_advanced_hotpath_detector() -> core_types::AdvancedHotPathDetector {
    core_types::AdvancedHotPathDetector::new()
}
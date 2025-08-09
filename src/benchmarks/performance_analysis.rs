//! Advanced Performance Analysis Tools for Lambdust
//!
//! This module provides comprehensive performance analysis capabilities including
//! bottleneck detection, hot path identification, memory usage analysis, and
//! optimization recommendations.

use crate::eval::{Value, Evaluator, Environment};
use crate::eval::fast_path::{get_fast_path_stats, FastPathStats};
use crate::numeric::{NumericValue, NumericType};
use crate::utils::profiler::{ProfileCategory, PerformanceReport, generate_report};
use crate::utils::{intern_symbol, SymbolId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Comprehensive performance analysis results
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    /// Overall performance score (0-100, higher is better)
    pub overall_score: f64,
    /// Detailed analysis by category
    pub category_analysis: HashMap<AnalysisCategory, CategoryAnalysis>,
    /// Identified bottlenecks
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// Hot paths in the execution
    pub hot_paths: Vec<HotPath>,
    /// Memory usage analysis
    pub memory_analysis: MemoryAnalysis,
    /// Optimization recommendations
    pub recommendations: Vec<OptimizationRecommendation>,
    /// Performance comparison with baseline
    pub baseline_comparison: Option<BaselineComparison>,
}

/// Categories of performance analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnalysisCategory {
    /// Numeric operations performance
    Arithmetic,
    /// List operations performance
    ListOperations,
    /// Environment and variable lookup performance
    EnvironmentAccess,
    /// Hash table operations performance
    HashTableAccess,
    /// Symbol interning performance
    SymbolInterning,
    /// Fast path optimization effectiveness
    FastPathOptimization,
    /// Memory allocation efficiency
    MemoryAllocation,
    /// Garbage collection impact
    GarbageCollection,
}

/// Analysis results for a specific category
#[derive(Debug, Clone)]
pub struct CategoryAnalysis {
    /// Category being analyzed
    pub category: AnalysisCategory,
    /// Performance score for this category (0-100)
    pub score: f64,
    /// Average operation time in nanoseconds
    pub avg_operation_time_ns: u64,
    /// Operations per second throughput
    pub ops_per_second: f64,
    /// Memory efficiency (bytes per operation)
    pub memory_per_operation: f64,
    /// Specific issues found in this category
    pub issues: Vec<String>,
    /// Optimization opportunities
    pub opportunities: Vec<String>,
}

/// A performance bottleneck identified in the analysis
#[derive(Debug, Clone)]
pub struct PerformanceBottleneck {
    /// Description of the bottleneck
    pub description: String,
    /// Category where bottleneck occurs
    pub category: AnalysisCategory,
    /// Severity (0-10, higher is more severe)
    pub severity: u8,
    /// Time spent in this bottleneck (percentage of total)
    pub time_percentage: f64,
    /// Memory impact (bytes)
    pub memory_impact: usize,
    /// Suggested fixes
    pub suggested_fixes: Vec<String>,
}

/// A hot path in the execution
#[derive(Debug, Clone)]
pub struct HotPath {
    /// Description of the hot path
    pub description: String,
    /// Operation that creates this hot path
    pub operation: String,
    /// Number of times this path is executed
    pub execution_count: usize,
    /// Total time spent in this hot path
    pub total_time: Duration,
    /// Average time per execution
    pub avg_time_per_execution: Duration,
    /// Optimization potential (0-100)
    pub optimization_potential: f64,
}

/// Memory usage analysis
#[derive(Debug, Clone)]
pub struct MemoryAnalysis {
    /// Current memory usage in bytes
    pub current_usage: usize,
    /// Peak memory usage observed
    pub peak_usage: usize,
    /// Memory allocation rate (bytes per second)
    pub allocation_rate: f64,
    /// Memory deallocation rate (bytes per second)
    pub deallocation_rate: f64,
    /// Garbage collection frequency (collections per second)
    pub gc_frequency: f64,
    /// Average GC pause time
    pub avg_gc_pause: Duration,
    /// Memory fragmentation estimate (0-100)
    pub fragmentation_estimate: f64,
    /// Memory pool efficiency scores
    pub pool_efficiency: HashMap<String, f64>,
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// Title of the recommendation
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Priority (0-10, higher is more important)
    pub priority: u8,
    /// Expected performance improvement (percentage)
    pub expected_improvement: f64,
    /// Implementation difficulty (0-10, higher is more difficult)
    pub implementation_difficulty: u8,
    /// Categories this recommendation affects
    pub affected_categories: Vec<AnalysisCategory>,
}

/// Comparison with baseline performance
#[derive(Debug, Clone)]
pub struct BaselineComparison {
    /// Baseline performance data
    pub baseline: BaselineMetrics,
    /// Current performance data
    pub current: BaselineMetrics,
    /// Performance changes by category
    pub category_changes: HashMap<AnalysisCategory, f64>,
    /// Overall performance change (percentage)
    pub overall_change: f64,
    /// Regression or improvement summary
    pub summary: String,
}

/// Baseline performance metrics
#[derive(Debug, Clone)]
pub struct BaselineMetrics {
    /// Overall operations per second
    pub ops_per_second: f64,
    /// Average operation latency
    pub avg_latency: Duration,
    /// Memory usage efficiency
    pub memory_efficiency: f64,
    /// Fast path hit rate
    pub fast_path_hit_rate: f64,
}

/// Performance analyzer implementation
pub struct PerformanceAnalyzer {
    /// Historical performance data
    baselines: HashMap<String, BaselineMetrics>,
    /// Configuration for analysis
    config: AnalysisConfig,
}

/// Configuration for performance analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Whether to include detailed memory analysis
    pub detailed_memory_analysis: bool,
    /// Whether to profile individual operations
    pub profile_operations: bool,
    /// Minimum execution time to consider for hot path analysis
    pub hot_path_threshold_ns: u64,
    /// Number of top bottlenecks to report
    pub max_bottlenecks: usize,
    /// Number of hot paths to analyze
    pub max_hot_paths: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            detailed_memory_analysis: true,
            profile_operations: true,
            hot_path_threshold_ns: 1000, // 1 microsecond
            max_bottlenecks: 10,
            max_hot_paths: 20,
        }
    }
}

impl PerformanceAnalyzer {
    /// Creates a new performance analyzer
    pub fn new(config: AnalysisConfig) -> Self {
        Self {
            baselines: HashMap::new(),
            config,
        }
    }

    /// Performs comprehensive performance analysis
    pub fn analyze(&mut self) -> PerformanceAnalysis {
        let start_time = Instant::now();
        
        // Gather performance data
        let performance_report = generate_report();
        let fast_path_stats = get_fast_path_stats();
        
        // Analyze each category
        let category_analysis = self.analyze_categories(&performance_report, &fast_path_stats);
        
        // Identify bottlenecks
        let bottlenecks = self.identify_bottlenecks(&performance_report, &category_analysis);
        
        // Find hot paths
        let hot_paths = self.find_hot_paths(&performance_report);
        
        // Analyze memory usage
        let memory_analysis = self.analyze_memory(&performance_report);
        
        // Generate optimization recommendations
        let recommendations = self.generate_recommendations(&category_analysis, &bottlenecks);
        
        // Compare with baseline if available
        let baseline_comparison = self.compare_with_baseline(&category_analysis);
        
        // Calculate overall score
        let overall_score = self.calculate_overall_score(&category_analysis);
        
        PerformanceAnalysis {
            overall_score,
            category_analysis,
            bottlenecks,
            hot_paths,
            memory_analysis,
            recommendations,
            baseline_comparison,
        }
    }
    
    /// Analyzes performance by category
    fn analyze_categories(&self, report: &PerformanceReport, fast_path_stats: &FastPathStats) -> HashMap<AnalysisCategory, CategoryAnalysis> {
        let mut analysis = HashMap::new();
        
        // Arithmetic operations analysis
        analysis.insert(AnalysisCategory::Arithmetic, self.analyze_arithmetic(report));
        
        // List operations analysis
        analysis.insert(AnalysisCategory::ListOperations, self.analyze_list_operations(report));
        
        // Environment access analysis
        analysis.insert(AnalysisCategory::EnvironmentAccess, self.analyze_environment_access(report));
        
        // Hash table analysis
        analysis.insert(AnalysisCategory::HashTableAccess, self.analyze_hash_table_access(report));
        
        // Symbol interning analysis
        analysis.insert(AnalysisCategory::SymbolInterning, self.analyze_symbol_interning(report));
        
        // Fast path optimization analysis
        analysis.insert(AnalysisCategory::FastPathOptimization, self.analyze_fast_path_optimization(fast_path_stats));
        
        // Memory allocation analysis
        analysis.insert(AnalysisCategory::MemoryAllocation, self.analyze_memory_allocation(report));
        
        // Garbage collection analysis
        analysis.insert(AnalysisCategory::GarbageCollection, self.analyze_garbage_collection(report));
        
        analysis
    }
    
    /// Analyzes arithmetic operations performance
    fn analyze_arithmetic(&self, report: &PerformanceReport) -> CategoryAnalysis {
        let mut issues = Vec::new();
        let mut opportunities = Vec::new();
        
        // Simulated analysis - in a real implementation, this would analyze actual profiling data
        let avg_operation_time_ns = 50; // Placeholder
        let ops_per_second = 1_000_000.0; // 1M ops/sec placeholder
        let memory_per_operation = 8.0; // 8 bytes per operation
        
        // Check for optimization opportunities
        if avg_operation_time_ns > 100 {
            issues.push("Arithmetic operations are slower than expected".to_string());
            opportunities.push("Implement SIMD optimizations for numeric operations".to_string());
        }
        
        if report.system_metrics.fast_path_hit_rate < 80.0 {
            opportunities.push("Increase fast path coverage for arithmetic operations".to_string());
        }
        
        let score = self.calculate_category_score(avg_operation_time_ns, ops_per_second, memory_per_operation);
        
        CategoryAnalysis {
            category: AnalysisCategory::Arithmetic,
            score,
            avg_operation_time_ns,
            ops_per_second,
            memory_per_operation,
            issues,
            opportunities,
        }
    }
    
    /// Analyzes list operations performance
    fn analyze_list_operations(&self, report: &PerformanceReport) -> CategoryAnalysis {
        let mut issues = Vec::new();
        let mut opportunities = Vec::new();
        
        let avg_operation_time_ns = 200; // Placeholder
        let ops_per_second = 500_000.0;
        let memory_per_operation = 16.0; // 16 bytes per list operation
        
        // Check for common list operation issues
        opportunities.push("Consider using more efficient data structures for large lists".to_string());
        opportunities.push("Implement list operation optimizations for common patterns".to_string());
        
        let score = self.calculate_category_score(avg_operation_time_ns, ops_per_second, memory_per_operation);
        
        CategoryAnalysis {
            category: AnalysisCategory::ListOperations,
            score,
            avg_operation_time_ns,
            ops_per_second,
            memory_per_operation,
            issues,
            opportunities,
        }
    }
    
    /// Analyzes environment access performance
    fn analyze_environment_access(&self, report: &PerformanceReport) -> CategoryAnalysis {
        let mut issues = Vec::new();
        let mut opportunities = Vec::new();
        
        let avg_operation_time_ns = 150;
        let ops_per_second = 666_666.0;
        let memory_per_operation = 24.0; // Environment lookups can be expensive
        
        opportunities.push("Implement variable caching for frequently accessed variables".to_string());
        opportunities.push("Optimize environment chain traversal".to_string());
        
        let score = self.calculate_category_score(avg_operation_time_ns, ops_per_second, memory_per_operation);
        
        CategoryAnalysis {
            category: AnalysisCategory::EnvironmentAccess,
            score,
            avg_operation_time_ns,
            ops_per_second,
            memory_per_operation,
            issues,
            opportunities,
        }
    }
    
    /// Analyzes hash table access performance
    fn analyze_hash_table_access(&self, _report: &PerformanceReport) -> CategoryAnalysis {
        let mut opportunities = Vec::new();
        
        let avg_operation_time_ns = 80;
        let ops_per_second = 1_250_000.0;
        let memory_per_operation = 32.0;
        
        opportunities.push("Consider using more cache-friendly hash table implementations".to_string());
        
        let score = self.calculate_category_score(avg_operation_time_ns, ops_per_second, memory_per_operation);
        
        CategoryAnalysis {
            category: AnalysisCategory::HashTableAccess,
            score,
            avg_operation_time_ns,
            ops_per_second,
            memory_per_operation,
            issues: Vec::new(),
            opportunities,
        }
    }
    
    /// Analyzes symbol interning performance
    fn analyze_symbol_interning(&self, report: &PerformanceReport) -> CategoryAnalysis {
        let mut issues = Vec::new();
        let mut opportunities = Vec::new();
        
        let avg_operation_time_ns = 100;
        let ops_per_second = 1_000_000.0;
        let memory_per_operation = 40.0; // Symbol storage overhead
        
        if report.system_metrics.string_interning_hit_rate < 70.0 {
            issues.push("String interning hit rate is below optimal".to_string());
            opportunities.push("Pre-intern more common symbols".to_string());
        }
        
        let score = self.calculate_category_score(avg_operation_time_ns, ops_per_second, memory_per_operation);
        
        CategoryAnalysis {
            category: AnalysisCategory::SymbolInterning,
            score,
            avg_operation_time_ns,
            ops_per_second,
            memory_per_operation,
            issues,
            opportunities,
        }
    }
    
    /// Analyzes fast path optimization effectiveness
    fn analyze_fast_path_optimization(&self, fast_path_stats: &FastPathStats) -> CategoryAnalysis {
        let mut issues = Vec::new();
        let mut opportunities = Vec::new();
        
        let avg_operation_time_ns = 30; // Fast path should be very fast
        let ops_per_second = 3_333_333.0;
        let memory_per_operation = 4.0; // Minimal memory overhead
        
        if fast_path_stats.hit_rate < 80.0 {
            issues.push(format!("Fast path hit rate is {:.1}%, should be >80%", fast_path_stats.hit_rate));
            opportunities.push("Add more operations to fast path optimization".to_string());
        }
        
        if fast_path_stats.total_fast_path_calls < fast_path_stats.total_regular_calls {
            opportunities.push("Identify more operations that can benefit from fast path optimization".to_string());
        }
        
        let score = if fast_path_stats.hit_rate > 90.0 { 95.0 }
                   else if fast_path_stats.hit_rate > 80.0 { 85.0 }
                   else if fast_path_stats.hit_rate > 70.0 { 70.0 }
                   else { 50.0 };
        
        CategoryAnalysis {
            category: AnalysisCategory::FastPathOptimization,
            score,
            avg_operation_time_ns,
            ops_per_second,
            memory_per_operation,
            issues,
            opportunities,
        }
    }
    
    /// Analyzes memory allocation performance
    fn analyze_memory_allocation(&self, report: &PerformanceReport) -> CategoryAnalysis {
        let mut issues = Vec::new();
        let mut opportunities = Vec::new();
        
        let avg_operation_time_ns = 1000; // Allocation can be expensive
        let ops_per_second = 1_000_000.0;
        let memory_per_operation = 0.0; // This is the allocation itself
        
        if report.system_metrics.memory_pool_efficiency < 0.7 {
            issues.push("Memory pool efficiency is below optimal".to_string());
            opportunities.push("Tune memory pool sizes for better efficiency".to_string());
        }
        
        if report.system_metrics.peak_memory_usage > 100 * 1024 * 1024 { // 100MB
            issues.push("High memory usage detected".to_string());
            opportunities.push("Implement more aggressive memory management".to_string());
        }
        
        let score = if report.system_metrics.memory_pool_efficiency > 0.8 { 90.0 }
                   else if report.system_metrics.memory_pool_efficiency > 0.6 { 75.0 }
                   else { 60.0 };
        
        CategoryAnalysis {
            category: AnalysisCategory::MemoryAllocation,
            score,
            avg_operation_time_ns,
            ops_per_second,
            memory_per_operation,
            issues,
            opportunities,
        }
    }
    
    /// Analyzes garbage collection performance
    fn analyze_garbage_collection(&self, report: &PerformanceReport) -> CategoryAnalysis {
        let mut issues = Vec::new();
        let mut opportunities = Vec::new();
        
        let avg_operation_time_ns = 50_000; // GC can be expensive
        let ops_per_second = 20.0; // GC is infrequent but impactful
        let memory_per_operation = -1000.0; // GC frees memory
        
        let gc_overhead = report.system_metrics.gc_time.as_secs_f64() / report.system_metrics.total_cpu_time.as_secs_f64();
        
        if gc_overhead > 0.1 { // More than 10% overhead
            issues.push("Garbage collection overhead is high".to_string());
            opportunities.push("Tune GC parameters for better performance".to_string());
        }
        
        if report.system_metrics.gc_count > 100 {
            opportunities.push("Consider reducing allocation pressure to minimize GC frequency".to_string());
        }
        
        let score = if gc_overhead < 0.05 { 95.0 }
                   else if gc_overhead < 0.1 { 80.0 }
                   else { 60.0 };
        
        CategoryAnalysis {
            category: AnalysisCategory::GarbageCollection,
            score,
            avg_operation_time_ns,
            ops_per_second,
            memory_per_operation,
            issues,
            opportunities,
        }
    }
    
    /// Calculates a performance score for a category
    fn calculate_category_score(&self, avg_time_ns: u64, ops_per_sec: f64, memory_per_op: f64) -> f64 {
        // Performance scoring algorithm
        let time_score = if avg_time_ns < 50 { 100.0 }
                        else if avg_time_ns < 100 { 90.0 }
                        else if avg_time_ns < 500 { 75.0 }
                        else if avg_time_ns < 1000 { 60.0 }
                        else { 40.0 };
        
        let throughput_score = if ops_per_sec > 1_000_000.0 { 100.0 }
                              else if ops_per_sec > 500_000.0 { 85.0 }
                              else if ops_per_sec > 100_000.0 { 70.0 }
                              else { 50.0 };
        
        let memory_score = if memory_per_op < 10.0 { 100.0 }
                          else if memory_per_op < 25.0 { 85.0 }
                          else if memory_per_op < 50.0 { 70.0 }
                          else { 50.0 };
        
        // Weighted average
        time_score * 0.4 + throughput_score * 0.4 + memory_score * 0.2
    }
    
    /// Identifies performance bottlenecks
    fn identify_bottlenecks(&self, report: &PerformanceReport, category_analysis: &HashMap<AnalysisCategory, CategoryAnalysis>) -> Vec<PerformanceBottleneck> {
        let mut bottlenecks = Vec::new();
        
        // Find categories with low scores
        for (category, analysis) in category_analysis {
            if analysis.score < 70.0 {
                let severity = if analysis.score < 50.0 { 8 }
                              else if analysis.score < 60.0 { 6 }
                              else { 4 };
                
                let bottleneck = PerformanceBottleneck {
                    description: format!("Low performance in {category:?} operations"),
                    category: category.clone(),
                    severity,
                    time_percentage: analysis.avg_operation_time_ns as f64 / 10_000.0, // Estimate
                    memory_impact: (analysis.memory_per_operation * 1000.0) as usize,
                    suggested_fixes: analysis.opportunities.clone(),
                };
                
                bottlenecks.push(bottleneck);
            }
        }
        
        // Sort by severity and limit results
        bottlenecks.sort_by_key(|b| std::cmp::Reverse(b.severity));
        bottlenecks.truncate(self.config.max_bottlenecks);
        
        bottlenecks
    }
    
    /// Finds hot paths in execution
    fn find_hot_paths(&self, report: &PerformanceReport) -> Vec<HotPath> {
        let mut hot_paths = Vec::new();
        
        // Analyze recent entries to find frequently executed operations
        let mut operation_stats: HashMap<String, (usize, Duration)> = HashMap::new();
        
        for entry in &report.recent_entries {
            let stats = operation_stats.entry(entry.operation.clone()).or_insert((0, Duration::ZERO));
            stats.0 += 1;
            stats.1 += entry.duration;
        }
        
        // Convert to hot paths
        for (operation, (count, total_time)) in operation_stats {
            if total_time.as_nanos() > self.config.hot_path_threshold_ns as u128 {
                let avg_time = total_time / count as u32;
                let optimization_potential = if avg_time.as_nanos() > 1000 { 90.0 }
                                           else if avg_time.as_nanos() > 500 { 70.0 }
                                           else { 40.0 };
                
                let hot_path = HotPath {
                    description: format!("Frequent execution of {operation}"),
                    operation,
                    execution_count: count,
                    total_time,
                    avg_time_per_execution: avg_time,
                    optimization_potential,
                };
                
                hot_paths.push(hot_path);
            }
        }
        
        // Sort by total time and limit results
        hot_paths.sort_by_key(|h| std::cmp::Reverse(h.total_time));
        hot_paths.truncate(self.config.max_hot_paths);
        
        hot_paths
    }
    
    /// Analyzes memory usage patterns
    fn analyze_memory(&self, report: &PerformanceReport) -> MemoryAnalysis {
        let current_usage = report.system_metrics.current_memory_usage;
        let peak_usage = report.system_metrics.peak_memory_usage;
        
        // Estimate rates based on recent activity
        let allocation_rate = current_usage as f64 / 10.0; // Placeholder
        let deallocation_rate = allocation_rate * 0.9; // Assume 90% is eventually freed
        let gc_frequency = report.system_metrics.gc_count as f64 / 60.0; // Per minute
        let avg_gc_pause = if report.system_metrics.gc_count > 0 {
            report.system_metrics.gc_time / report.system_metrics.gc_count as u32
        } else {
            Duration::ZERO
        };
        
        let fragmentation_estimate = if peak_usage > 0 {
            ((peak_usage - current_usage) as f64 / peak_usage as f64) * 100.0
        } else {
            0.0
        };
        
        // Pool efficiency placeholder
        let mut pool_efficiency = HashMap::new();
        pool_efficiency.insert("small_objects".to_string(), report.system_metrics.memory_pool_efficiency);
        pool_efficiency.insert("large_objects".to_string(), report.system_metrics.memory_pool_efficiency * 0.8);
        
        MemoryAnalysis {
            current_usage,
            peak_usage,
            allocation_rate,
            deallocation_rate,
            gc_frequency,
            avg_gc_pause,
            fragmentation_estimate,
            pool_efficiency,
        }
    }
    
    /// Generates optimization recommendations
    fn generate_recommendations(&self, category_analysis: &HashMap<AnalysisCategory, CategoryAnalysis>, bottlenecks: &[PerformanceBottleneck]) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        
        // High-priority recommendations based on bottlenecks
        for bottleneck in bottlenecks {
            if bottleneck.severity >= 6 {
                for (i, fix) in bottleneck.suggested_fixes.iter().enumerate() {
                    let rec = OptimizationRecommendation {
                        title: format!("Fix bottleneck in {:?}", bottleneck.category),
                        description: fix.clone(),
                        priority: bottleneck.severity,
                        expected_improvement: 20.0 + (bottleneck.severity as f64 * 2.0),
                        implementation_difficulty: 5 + (i as u8),
                        affected_categories: vec![bottleneck.category.clone()],
                    };
                    recommendations.push(rec);
                }
            }
        }
        
        // General optimization opportunities
        for (category, analysis) in category_analysis {
            if analysis.score < 85.0 {
                for opportunity in &analysis.opportunities {
                    let rec = OptimizationRecommendation {
                        title: format!("Optimize {category:?} performance"),
                        description: opportunity.clone(),
                        priority: if analysis.score < 70.0 { 7 } else { 5 },
                        expected_improvement: (100.0 - analysis.score) * 0.3,
                        implementation_difficulty: 4,
                        affected_categories: vec![category.clone()],
                    };
                    recommendations.push(rec);
                }
            }
        }
        
        // Sort by priority and expected improvement
        recommendations.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then(b.expected_improvement.partial_cmp(&a.expected_improvement).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        // Deduplicate similar recommendations
        recommendations.dedup_by(|a, b| a.title == b.title);
        
        recommendations.truncate(15); // Limit to top 15 recommendations
        recommendations
    }
    
    /// Compares current performance with baseline
    fn compare_with_baseline(&self, category_analysis: &HashMap<AnalysisCategory, CategoryAnalysis>) -> Option<BaselineComparison> {
        // For now, return None since we don't have baseline data
        // In a real implementation, this would compare against stored baselines
        None
    }
    
    /// Calculates overall performance score
    fn calculate_overall_score(&self, category_analysis: &HashMap<AnalysisCategory, CategoryAnalysis>) -> f64 {
        if category_analysis.is_empty() {
            return 0.0;
        }
        
        // Weighted average of category scores
        let weights: HashMap<AnalysisCategory, f64> = vec![
            (AnalysisCategory::Arithmetic, 0.20),
            (AnalysisCategory::ListOperations, 0.15),
            (AnalysisCategory::EnvironmentAccess, 0.15),
            (AnalysisCategory::FastPathOptimization, 0.15),
            (AnalysisCategory::MemoryAllocation, 0.10),
            (AnalysisCategory::GarbageCollection, 0.10),
            (AnalysisCategory::HashTableAccess, 0.08),
            (AnalysisCategory::SymbolInterning, 0.07),
        ].into_iter().collect();
        
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        
        for (category, analysis) in category_analysis {
            if let Some(&weight) = weights.get(category) {
                weighted_sum += analysis.score * weight;
                total_weight += weight;
            }
        }
        
        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }
    
    /// Records baseline performance for future comparisons
    pub fn record_baseline(&mut self, name: String, metrics: BaselineMetrics) {
        self.baselines.insert(name, metrics);
    }
}

impl PerformanceAnalysis {
    /// Formats the analysis results as a human-readable report
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Lambdust Performance Analysis Report ===\n\n");
        
        // Overall score
        report.push_str(&format!("Overall Performance Score: {:.1}/100\n\n", self.overall_score));
        
        // Category breakdown
        report.push_str("=== Performance by Category ===\n");
        let mut categories: Vec<_> = self.category_analysis.iter().collect();
        categories.sort_by_key(|(_, analysis)| std::cmp::Reverse((analysis.score * 100.0) as u32));
        
        for (category, analysis) in categories {
            report.push_str(&format!("{:?}: {:.1}/100 ({:.0} ops/sec, {:.1} ns avg)\n", 
                category, analysis.score, analysis.ops_per_second, analysis.avg_operation_time_ns));
            
            for issue in &analysis.issues {
                report.push_str(&format!("  ⚠ {issue}\n"));
            }
        }
        report.push('\n');
        
        // Top bottlenecks
        if !self.bottlenecks.is_empty() {
            report.push_str("=== Performance Bottlenecks ===\n");
            for (i, bottleneck) in self.bottlenecks.iter().take(5).enumerate() {
                report.push_str(&format!("{}. {} (Severity: {}/10)\n", 
                    i + 1, bottleneck.description, bottleneck.severity));
                if !bottleneck.suggested_fixes.is_empty() {
                    report.push_str(&format!("   Fix: {}\n", bottleneck.suggested_fixes[0]));
                }
            }
            report.push('\n');
        }
        
        // Hot paths
        if !self.hot_paths.is_empty() {
            report.push_str("=== Performance Hot Paths ===\n");
            for (i, hot_path) in self.hot_paths.iter().take(5).enumerate() {
                report.push_str(&format!("{}. {} ({} executions, {:.2}ms total)\n", 
                    i + 1, hot_path.description, hot_path.execution_count, 
                    hot_path.total_time.as_secs_f64() * 1000.0));
            }
            report.push('\n');
        }
        
        // Memory analysis
        report.push_str("=== Memory Analysis ===\n");
        report.push_str(&format!("Current Usage: {:.2} MB\n", self.memory_analysis.current_usage as f64 / 1024.0 / 1024.0));
        report.push_str(&format!("Peak Usage: {:.2} MB\n", self.memory_analysis.peak_usage as f64 / 1024.0 / 1024.0));
        report.push_str(&format!("GC Frequency: {:.1} collections/sec\n", self.memory_analysis.gc_frequency));
        report.push_str(&format!("Average GC Pause: {:.2}ms\n", self.memory_analysis.avg_gc_pause.as_secs_f64() * 1000.0));
        report.push('\n');
        
        // Top recommendations
        if !self.recommendations.is_empty() {
            report.push_str("=== Top Optimization Recommendations ===\n");
            for (i, rec) in self.recommendations.iter().take(5).enumerate() {
                report.push_str(&format!("{}. {} (Priority: {}/10, Expected improvement: {:.1}%)\n", 
                    i + 1, rec.title, rec.priority, rec.expected_improvement));
                report.push_str(&format!("   {}\n", rec.description));
            }
            report.push('\n');
        }
        
        report
    }
    
    /// Exports the analysis as JSON
    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Simplified JSON export - in a real implementation, this would use serde_json
        Ok(format!(r#"{{"overall_score": {}, "bottleneck_count": {}, "recommendation_count": {}}}"#,
                   self.overall_score, self.bottlenecks.len(), self.recommendations.len()))
    }
}
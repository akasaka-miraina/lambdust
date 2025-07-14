//! Pattern and Knowledge Types for Adaptive Theorem Learning
//!
//! This module contains data structures for representing discovered patterns,
//! learned knowledge, and various pattern types used in the adaptive learning system.

use crate::ast::Expr;
use std::time::{Duration, Instant};

/// A discovered pattern in Scheme code with associated context and metrics
/// 
/// TODO Phase 9: Implement pattern matching algorithms and validation
pub struct DiscoveredPattern {
    /// Unique identifier for this pattern
    pub pattern_id: String,
    
    /// Human-readable description
    pub description: String,
    
    /// AST structure that defines the pattern
    pub ast_pattern: Expr,
    
    /// Pattern type classification
    pub pattern_type: PatternType,
    
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    
    /// Number of occurrences found
    pub occurrence_count: usize,
    
    /// Contexts where this pattern was observed
    pub contexts: Vec<OccurrenceContext>,
    
    /// Performance characteristics when this pattern is used
    pub performance_data: PatternPerformanceData,
    
    /// Related patterns (similar or complementary)
    pub related_patterns: Vec<String>,
    
    /// Statistical significance metrics
    pub statistical_metrics: PatternStatistics,
    
    /// Learning metadata
    pub learning_metadata: PatternLearningMetadata,
}

/// Context information for where a pattern was discovered
pub struct OccurrenceContext {
    /// Source file information
    pub source_info: SourceInfo,
    
    /// Surrounding code context
    pub context_window: Vec<Expr>,
    
    /// Performance data for this specific occurrence
    pub context_performance: ContextPerformanceData,
    
    /// Code style indicators
    pub style_indicators: StyleIndicators,
    
    /// Timestamp of discovery
    pub discovered_at: Instant,
}

/// Source file information for pattern tracking
pub struct SourceInfo {
    /// File path (if available)
    pub file_path: Option<String>,
    
    /// Line number range
    pub line_range: (usize, usize),
    
    /// Function or scope name
    pub scope_name: Option<String>,
    
    /// Module or namespace
    pub module_name: Option<String>,
}

/// Performance data associated with a pattern occurrence
pub struct ContextPerformanceData {
    /// Execution time for this pattern instance
    pub execution_time: Duration,
    
    /// Memory usage
    pub memory_usage: usize,
    
    /// Optimization potential score
    pub optimization_potential: f64,
    
    /// Benchmark comparison data
    pub benchmark_data: BenchmarkComparison,
}

/// Code style indicators for pattern analysis
pub struct StyleIndicators {
    /// Idiomatic Scheme usage rating
    pub idiomaticity_score: f64,
    
    /// Code complexity metrics
    pub complexity_metrics: ComplexityMetrics,
    
    /// Functional programming style indicators
    pub fp_style_score: f64,
    
    /// Performance-oriented coding indicators
    pub performance_focus_score: f64,
}

/// Classification of different pattern types
#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    /// Common Scheme idioms
    Idiom,
    
    /// Performance optimization patterns
    Optimization,
    
    /// Functional programming patterns
    Functional,
    
    /// Control flow patterns
    ControlFlow,
    
    /// Data structure patterns
    DataStructure,
    
    /// Algorithmic patterns
    Algorithm,
    
    /// Framework-specific patterns
    Framework,
    
    /// Domain-specific patterns
    Domain,
    
    /// Anti-patterns (things to avoid)
    AntiPattern,
    
    /// Emerging patterns (newly discovered)
    Emerging,
}

// Placeholder structures for compilation
// TODO Phase 9: Implement these structures

pub struct PatternPerformanceData;
pub struct PatternStatistics;
pub struct PatternLearningMetadata;
pub struct BenchmarkComparison;
pub struct ComplexityMetrics;
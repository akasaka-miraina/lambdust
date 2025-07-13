//! Core Types Module
//!
//! このモジュールは衛生的トランスフォーマーの基本型定義を提供します。
//! 最適化レベル、パフォーマンスメトリクス、パターンバインディングを含みます。

use crate::ast::Expr;
use std::collections::HashMap;

/// Pattern bindings for macro expansion
pub type PatternBindings = HashMap<String, Expr>;

/// Advanced optimization configuration for hygienic transformers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    /// Minimal optimization for debugging
    Development,
    /// Balanced optimization for general use
    Balanced,
    /// Maximum optimization for production
    Production,
    /// Custom optimization with specific settings
    Custom {
        /// Enable pattern and template caching
        enable_caching: bool,
        /// Enable machine learning-inspired renaming heuristics
        enable_intelligent_renaming: bool,
        /// Enable scope depth analysis for conflict detection
        enable_scope_analysis: bool,
        /// Enable advanced pattern matching optimization
        enable_pattern_optimization: bool,
    },
}

/// Performance metrics for transformer operations
#[derive(Debug, Clone, Default)]
pub struct TransformerMetrics {
    /// Total transformations performed
    pub transformations_count: u64,
    /// Successful transformations
    pub successful_transformations: u64,
    /// Pattern matching attempts
    pub pattern_matches_attempted: u64,
    /// Successful pattern matches
    pub pattern_matches_successful: u64,
    /// Template substitutions performed
    pub template_substitutions: u64,
    /// Symbol renamings performed
    pub symbol_renamings: u64,
    /// Total processing time (nanoseconds)
    pub total_processing_time_ns: u64,
    /// Cache hits for pattern matching
    pub pattern_cache_hits: u64,
    /// Cache misses for pattern matching
    pub pattern_cache_misses: u64,
}

impl TransformerMetrics {
    /// Get transformation success rate
    #[must_use] 
    pub fn success_rate(&self) -> f64 {
        if self.transformations_count > 0 {
            (self.successful_transformations as f64 / self.transformations_count as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Get pattern matching efficiency
    #[must_use] 
    pub fn pattern_match_efficiency(&self) -> f64 {
        if self.pattern_matches_attempted > 0 {
            (self.pattern_matches_successful as f64 / self.pattern_matches_attempted as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Get cache hit rate
    #[must_use] 
    pub fn cache_hit_rate(&self) -> f64 {
        let total_requests = self.pattern_cache_hits + self.pattern_cache_misses;
        if total_requests > 0 {
            (self.pattern_cache_hits as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        }
    }
    
    /// Get average processing time per transformation (nanoseconds)
    #[must_use] 
    pub fn average_processing_time(&self) -> f64 {
        if self.transformations_count > 0 {
            self.total_processing_time_ns as f64 / self.transformations_count as f64
        } else {
            0.0
        }
    }
    
    /// Add metrics from another instance
    pub fn merge_with(&mut self, other: &TransformerMetrics) {
        self.transformations_count += other.transformations_count;
        self.successful_transformations += other.successful_transformations;
        self.pattern_matches_attempted += other.pattern_matches_attempted;
        self.pattern_matches_successful += other.pattern_matches_successful;
        self.template_substitutions += other.template_substitutions;
        self.symbol_renamings += other.symbol_renamings;
        self.total_processing_time_ns += other.total_processing_time_ns;
        self.pattern_cache_hits += other.pattern_cache_hits;
        self.pattern_cache_misses += other.pattern_cache_misses;
    }
    
    /// Reset all metrics to zero
    pub fn reset(&mut self) {
        *self = TransformerMetrics::default();
    }
}

impl Default for OptimizationLevel {
    fn default() -> Self {
        Self::Balanced
    }
}

impl OptimizationLevel {
    /// Check if caching is enabled for this optimization level
    #[must_use] 
    pub fn caching_enabled(&self) -> bool {
        match self {
            OptimizationLevel::Development => false,
            OptimizationLevel::Balanced => true,
            OptimizationLevel::Production => true,
            OptimizationLevel::Custom { enable_caching, .. } => *enable_caching,
        }
    }
    
    /// Check if intelligent renaming is enabled
    #[must_use] 
    pub fn intelligent_renaming_enabled(&self) -> bool {
        match self {
            OptimizationLevel::Development => false,
            OptimizationLevel::Balanced => true,
            OptimizationLevel::Production => true,
            OptimizationLevel::Custom { enable_intelligent_renaming, .. } => *enable_intelligent_renaming,
        }
    }
    
    /// Check if scope analysis is enabled
    #[must_use] 
    pub fn scope_analysis_enabled(&self) -> bool {
        match self {
            OptimizationLevel::Development => false,
            OptimizationLevel::Balanced => false,
            OptimizationLevel::Production => true,
            OptimizationLevel::Custom { enable_scope_analysis, .. } => *enable_scope_analysis,
        }
    }
    
    /// Check if pattern optimization is enabled
    #[must_use] 
    pub fn pattern_optimization_enabled(&self) -> bool {
        match self {
            OptimizationLevel::Development => false,
            OptimizationLevel::Balanced => true,
            OptimizationLevel::Production => true,
            OptimizationLevel::Custom { enable_pattern_optimization, .. } => *enable_pattern_optimization,
        }
    }
}
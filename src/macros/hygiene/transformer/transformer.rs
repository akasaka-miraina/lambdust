//! Main Transformer Module
//!
//! このモジュールはメイン衛生的トランスフォーマーの実装を提供します。
//! 構造体定義、ファクトリーメソッド、最適化設定、変換ロジックを含みます。

use super::core_types::{OptimizationLevel, TransformerMetrics};
use super::pattern_matching::PatternMatcher;
use super::template_expansion::TemplateExpander;
use crate::macros::{SyntaxRule, NestedEllipsisProcessor};
use crate::macros::hygiene::environment::HygienicEnvironment;
use crate::macros::hygiene::renaming::{RenamingStrategy, SymbolRenamer};
use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::SystemTime;

/// Hygienic version of syntax-rules transformer with advanced optimization
#[derive(Debug, Clone)]
pub struct HygienicSyntaxRulesTransformer {
    /// Literal identifiers that shouldn't be renamed
    pub literals: Vec<String>,
    /// Transformation rules
    pub rules: Vec<SyntaxRule>,
    /// Symbol renaming strategy
    pub renaming_strategy: RenamingStrategy,
    /// Advanced symbol renamer instance
    pub symbol_renamer: SymbolRenamer,
    /// Lexical environment at definition site
    pub definition_environment: Rc<HygienicEnvironment>,
    /// Macro name for debugging
    pub macro_name: String,
    /// Optimization level for performance tuning
    pub optimization_level: OptimizationLevel,
    /// Performance metrics
    pub metrics: TransformerMetrics,
    /// Pattern matching cache for performance
    pub pattern_cache: HashMap<String, bool>,
    /// Template substitution cache
    pub template_cache: HashMap<String, Expr>,
    /// SRFI 46 nested ellipsis processor
    pub ellipsis_processor: NestedEllipsisProcessor,
    /// Enable SRFI 46 nested ellipsis support
    pub enable_srfi46: bool,
}

impl HygienicSyntaxRulesTransformer {
    /// Create new hygienic transformer with intelligent defaults
    pub fn new(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
    ) -> Self {
        let symbol_renamer = SymbolRenamer::intelligent();
        
        Self {
            literals,
            rules,
            renaming_strategy: RenamingStrategy::Intelligent,
            symbol_renamer,
            definition_environment,
            macro_name,
            optimization_level: OptimizationLevel::Balanced,
            metrics: TransformerMetrics::default(),
            pattern_cache: HashMap::new(),
            template_cache: HashMap::new(),
            ellipsis_processor: NestedEllipsisProcessor::new(),
            enable_srfi46: true, // Enable by default for modern Scheme compatibility
        }
    }
    
    /// Create transformer with custom renaming strategy
    pub fn with_renaming_strategy(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
        strategy: RenamingStrategy,
    ) -> Self {
        let symbol_renamer = SymbolRenamer::new(strategy.clone());
        
        Self {
            literals,
            rules,
            renaming_strategy: strategy,
            symbol_renamer,
            definition_environment,
            macro_name,
            optimization_level: OptimizationLevel::Balanced,
            metrics: TransformerMetrics::default(),
            pattern_cache: HashMap::new(),
            template_cache: HashMap::new(),
            ellipsis_processor: NestedEllipsisProcessor::new(),
            enable_srfi46: true,
        }
    }
    
    /// Create high-performance transformer for production use
    pub fn optimized(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
    ) -> Self {
        let mut symbol_renamer = SymbolRenamer::optimized();
        // Configure for high-performance macro usage
        symbol_renamer.set_strategy(RenamingStrategy::PerformanceOptimized);
        
        Self {
            literals,
            rules,
            renaming_strategy: RenamingStrategy::PerformanceOptimized,
            symbol_renamer,
            definition_environment,
            macro_name,
            optimization_level: OptimizationLevel::Production,
            metrics: TransformerMetrics::default(),
            pattern_cache: HashMap::with_capacity(1000), // Pre-allocated cache
            template_cache: HashMap::with_capacity(500),
            ellipsis_processor: NestedEllipsisProcessor::with_max_depth(15), // Higher depth for production
            enable_srfi46: true,
        }
    }
    
    /// Create scope-aware transformer for complex macro systems
    pub fn scope_aware(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
    ) -> Self {
        let symbol_renamer = SymbolRenamer::scope_aware();
        
        Self {
            literals,
            rules,
            renaming_strategy: RenamingStrategy::ScopeAware,
            symbol_renamer,
            definition_environment,
            macro_name,
            optimization_level: OptimizationLevel::Custom {
                enable_caching: true,
                enable_intelligent_renaming: false,
                enable_scope_analysis: true,
                enable_pattern_optimization: true,
            },
            metrics: TransformerMetrics::default(),
            pattern_cache: HashMap::new(),
            template_cache: HashMap::new(),
            ellipsis_processor: NestedEllipsisProcessor::new(),
            enable_srfi46: true,
        }
    }
    
    /// Create transformer with custom optimization settings
    pub fn with_optimization(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
        optimization: OptimizationLevel,
    ) -> Self {
        let (symbol_renamer, strategy) = match &optimization {
            OptimizationLevel::Development => {
                (SymbolRenamer::new(RenamingStrategy::Conservative), RenamingStrategy::Conservative)
            }
            OptimizationLevel::Balanced => {
                (SymbolRenamer::intelligent(), RenamingStrategy::Intelligent)
            }
            OptimizationLevel::Production => {
                (SymbolRenamer::optimized(), RenamingStrategy::PerformanceOptimized)
            }
            OptimizationLevel::Custom { enable_intelligent_renaming, .. } => {
                if *enable_intelligent_renaming {
                    (SymbolRenamer::intelligent(), RenamingStrategy::Intelligent)
                } else {
                    (SymbolRenamer::scope_aware(), RenamingStrategy::ScopeAware)
                }
            }
        };
        
        let (pattern_capacity, template_capacity) = match optimization {
            OptimizationLevel::Production => (1000, 500),
            OptimizationLevel::Balanced => (200, 100),
            _ => (50, 25),
        };
        
        let ellipsis_depth = match optimization {
            OptimizationLevel::Production => 20,
            OptimizationLevel::Balanced => 10,
            _ => 5,
        };
        
        Self {
            literals,
            rules,
            renaming_strategy: strategy,
            symbol_renamer,
            definition_environment,
            macro_name,
            optimization_level: optimization,
            metrics: TransformerMetrics::default(),
            pattern_cache: HashMap::with_capacity(pattern_capacity),
            template_cache: HashMap::with_capacity(template_capacity),
            ellipsis_processor: NestedEllipsisProcessor::with_max_depth(ellipsis_depth),
            enable_srfi46: true,
        }
    }
    
    /// Apply hygienic transformation with advanced optimization and safety checks (mutable version)
    pub fn transform_hygienic_optimized(
        &mut self,
        input: &[Expr],
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        let start_time = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        self.metrics.transformations_count += 1;
        
        // Check optimization-specific preprocessing
        let should_use_cache = matches!(self.optimization_level,
            OptimizationLevel::Custom { enable_pattern_optimization: true, .. } 
            | OptimizationLevel::Production 
            | OptimizationLevel::Balanced
        );
        
        if should_use_cache {
            // Try cache lookup for performance
            let cache_key = format!("{}:{}", self.macro_name, input.len());
            if let Some(cached_result) = self.template_cache.get(&cache_key).cloned() {
                self.metrics.successful_transformations += 1;
                self.update_processing_time(start_time);
                return Ok(cached_result);
            }
        }
        
        // Reconstruct the full expression including macro name
        let mut full_expr = vec![Expr::Variable(self.macro_name.clone())];
        full_expr.extend_from_slice(input);
        let expr = Expr::List(full_expr);
        
        // Try each rule until one matches with performance tracking
        let rules_len = self.rules.len();
        for rule_index in 0..rules_len {
            self.metrics.pattern_matches_attempted += 1;
            
            // Clone the rule to avoid borrowing issues
            let rule = self.rules[rule_index].clone();
            
            match PatternMatcher::match_pattern_optimized(
                &self.macro_name,
                &rule.pattern,
                &expr,
                usage_environment,
                rule_index,
                self.optimization_level,
                &mut self.pattern_cache,
                self.enable_srfi46,
                &self.ellipsis_processor,
                &mut self.metrics.pattern_cache_hits,
                &mut self.metrics.pattern_cache_misses,
            ) {
                Ok(bindings) => {
                    self.metrics.pattern_matches_successful += 1;
                    
                    match TemplateExpander::substitute_template_optimized(
                        &rule.template,
                        bindings,
                        usage_environment,
                        &self.definition_environment,
                        &self.literals,
                        &mut self.symbol_renamer,
                        self.enable_srfi46,
                        &self.ellipsis_processor,
                        &mut self.metrics.symbol_renamings,
                    ) {
                        Ok(result) => {
                            self.metrics.successful_transformations += 1;
                            self.metrics.template_substitutions += 1;
                            
                            // Cache result if optimization is enabled
                            if should_use_cache {
                                let cache_key = format!("{}:{}", self.macro_name, input.len());
                                self.template_cache.insert(cache_key, result.clone());
                            }
                            
                            self.update_processing_time(start_time);
                            return Ok(result);
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(_) => {} // Try next rule
            }
        }
        
        self.update_processing_time(start_time);
        Err(LambdustError::runtime_error(format!(
            "No matching rule for macro {} with {} arguments. Rules attempted: {}, Pattern matches: {}/{}",
            self.macro_name,
            input.len(),
            self.rules.len(),
            self.metrics.pattern_matches_successful,
            self.metrics.pattern_matches_attempted
        )))
    }
    
    /// Apply hygienic transformation (immutable compatibility version)
    pub fn transform_hygienic(
        &self,
        input: &[Expr],
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // Reconstruct the full expression including macro name
        let mut full_expr = vec![Expr::Variable(self.macro_name.clone())];
        full_expr.extend_from_slice(input);
        let expr = Expr::List(full_expr);
        
        // Try each rule until one matches (without advanced optimization)
        for rule in &self.rules {
            if let Ok(bindings) = PatternMatcher::match_pattern(
                &rule.pattern,
                &expr,
                usage_environment,
                self.enable_srfi46,
                &self.ellipsis_processor,
            ) {
                return TemplateExpander::substitute_template(
                    &rule.template,
                    bindings,
                    usage_environment,
                    &self.definition_environment,
                    &self.literals,
                    &self.symbol_renamer,
                    self.enable_srfi46,
                    &self.ellipsis_processor,
                );
            }
        }
        
        Err(LambdustError::runtime_error(format!(
            "No matching rule for macro {} with {} arguments",
            self.macro_name,
            input.len()
        )))
    }
    
    /// Update processing time metrics
    fn update_processing_time(&mut self, start_time: u64) {
        let current_time = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        self.metrics.total_processing_time_ns += current_time.saturating_sub(start_time);
    }
    
    /// Get performance metrics
    #[must_use] 
    pub fn metrics(&self) -> &TransformerMetrics {
        &self.metrics
    }
    
    /// Get detailed performance analysis
    #[must_use] 
    pub fn performance_analysis(&self) -> String {
        let pattern_match_rate = if self.metrics.pattern_matches_attempted > 0 {
            (self.metrics.pattern_matches_successful as f64 / self.metrics.pattern_matches_attempted as f64) * 100.0
        } else {
            0.0
        };
        
        let cache_hit_rate = self.metrics.cache_hit_rate();
        let success_rate = self.metrics.success_rate();
        let avg_time = self.metrics.average_processing_time();
        
        format!(
            "Hygienic Transformer Performance Analysis:\n\
             Macro: {}\n\
             Total Transformations: {}\n\
             Success Rate: {:.1}%\n\
             Pattern Matches: {}/{} ({:.1}%)\n\
             Template Substitutions: {}\n\
             Symbol Renamings: {}\n\
             Cache Hit Rate: {:.1}%\n\
             Average Processing Time: {:.2}μs\n\
             Optimization Level: {:?}\n\
             SRFI 46 Support: {}",
            self.macro_name,
            self.metrics.transformations_count,
            success_rate,
            self.metrics.pattern_matches_successful,
            self.metrics.pattern_matches_attempted,
            pattern_match_rate,
            self.metrics.template_substitutions,
            self.metrics.symbol_renamings,
            cache_hit_rate,
            avg_time / 1000.0, // Convert to microseconds
            self.optimization_level,
            self.enable_srfi46
        )
    }
    
    /// Check if transformer needs optimization (based on performance)
    #[must_use] 
    pub fn needs_optimization(&self) -> bool {
        let pattern_match_rate = if self.metrics.pattern_matches_attempted > 0 {
            (self.metrics.pattern_matches_successful as f64 / self.metrics.pattern_matches_attempted as f64) * 100.0
        } else {
            0.0
        };
        
        let cache_hit_rate = self.metrics.cache_hit_rate();
        
        if pattern_match_rate < 30.0 {
            return true; // Poor pattern matching efficiency
        }
        
        if cache_hit_rate < 20.0 && self.metrics.transformations_count > 100 {
            return true; // Poor cache efficiency with enough data
        }
        
        false
    }
    
    /// Reset performance metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = TransformerMetrics::default();
        if matches!(self.optimization_level, OptimizationLevel::Production | OptimizationLevel::Balanced) {
            self.pattern_cache.clear();
            self.template_cache.clear();
        }
    }
    
    /// Optimize caches based on usage patterns
    pub fn optimize_caches(&mut self) {
        // Remove least recently used entries if caches are too large
        if self.pattern_cache.len() > 2000 {
            self.pattern_cache.clear();
        }
        
        if self.template_cache.len() > 1000 {
            self.template_cache.clear();
        }
        
        // Optimize symbol renamer caches
        self.symbol_renamer.optimize_caches();
    }
    
    /// Get current optimization level
    #[must_use] 
    pub fn optimization_level(&self) -> &OptimizationLevel {
        &self.optimization_level
    }
    
    /// Change optimization level (clears caches)
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.optimization_level = level;
        self.pattern_cache.clear();
        self.template_cache.clear();
        
        // Update symbol renamer strategy based on optimization level
        let new_strategy = match level {
            OptimizationLevel::Development => RenamingStrategy::Conservative,
            OptimizationLevel::Balanced => RenamingStrategy::Intelligent,
            OptimizationLevel::Production => RenamingStrategy::PerformanceOptimized,
            OptimizationLevel::Custom { enable_intelligent_renaming: true, .. } => {
                RenamingStrategy::Intelligent
            }
            OptimizationLevel::Custom { enable_scope_analysis: true, .. } => {
                RenamingStrategy::ScopeAware
            }
            _ => RenamingStrategy::Conservative,
        };
        
        self.symbol_renamer.set_strategy(new_strategy.clone());
        self.renaming_strategy = new_strategy;
    }
    
    /// Get symbol renamer statistics
    #[must_use] 
    pub fn renamer_stats(&self) -> &crate::macros::hygiene::renaming::RenamingStats {
        self.symbol_renamer.performance_stats()
    }
    
    /// Enable or disable SRFI 46 nested ellipsis support
    pub fn set_srfi46_support(&mut self, enable: bool) {
        self.enable_srfi46 = enable;
    }
    
    /// Check if SRFI 46 support is enabled
    #[must_use] 
    pub fn is_srfi46_enabled(&self) -> bool {
        self.enable_srfi46
    }
    
    /// Get SRFI 46 ellipsis processor metrics
    #[must_use] 
    pub fn ellipsis_metrics(&self) -> &crate::macros::EllipsisMetrics {
        self.ellipsis_processor.metrics()
    }
    
    /// Create SRFI 46 enabled transformer
    pub fn with_srfi46(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
    ) -> Self {
        let mut transformer = Self::new(literals, rules, definition_environment, macro_name);
        transformer.enable_srfi46 = true;
        transformer.ellipsis_processor = NestedEllipsisProcessor::with_max_depth(20);
        transformer
    }
    
    /// Create legacy transformer (SRFI 46 disabled)
    pub fn legacy(
        literals: Vec<String>,
        rules: Vec<SyntaxRule>,
        definition_environment: Rc<HygienicEnvironment>,
        macro_name: String,
    ) -> Self {
        let mut transformer = Self::new(literals, rules, definition_environment, macro_name);
        transformer.enable_srfi46 = false;
        transformer
    }
}
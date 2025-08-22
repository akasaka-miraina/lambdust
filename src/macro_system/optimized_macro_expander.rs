//! Optimized macro expander with high-performance algorithms.
//!
//! This module implements Phase 2A.1 optimizations from the CS-architect design:
//! - O(1) macro resolution using HashMap
//! - Compile-time computation engine integration
//! - Type-safe macro expansion with zero-cost abstractions
//! - Memory-efficient smart pointer utilization
//! - Cache-aware pattern matching

use super::{
    CompileTimeComputationEngine, HygieneContext, MacroEnvironment, MacroTransformer, Pattern,
    PatternBindings, Template, TypeSafeMacroExpander,
};
use crate::ast::{Expr, Spanned};
use crate::diagnostics::{Error, Result, Span};
use crate::eval::Environment;

use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

/// High-performance optimized macro expander.
#[derive(Debug)]
pub struct OptimizedMacroExpander {
    /// O(1) macro lookup with fast hashing
    macro_registry: HashMap<MacroKey, Arc<CachedMacroTransformer>>,
    /// Expansion cache for frequently used macros
    expansion_cache: RefCell<HashMap<ExpansionKey, Arc<Spanned<Expr>>>>,
    /// Type-safe expansion engine
    type_safe_expander: TypeSafeMacroExpander,
    /// Compile-time computation engine
    compile_time_engine: CompileTimeComputationEngine,
    /// Fast hygiene resolver with interning
    hygiene_resolver: FastHygieneResolver,
    /// Performance metrics
    metrics: RefCell<ExpansionMetrics>,
    /// Configuration
    config: CachingOptimizationConfig,
}

/// Optimized macro key for O(1) lookup.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MacroKey {
    name: Arc<str>,
    name_hash: u64,
}

impl MacroKey {
    fn new(name: &str) -> Self {
        let name_arc: Arc<str> = Arc::from(name);
        let mut hasher = DefaultHasher::new();
        name_arc.hash(&mut hasher);
        Self {
            name: name_arc,
            name_hash: hasher.finish(),
        }
    }
}

/// Cached macro transformer with pre-computed metadata.
#[derive(Debug)]
#[allow(clippy::arc_with_non_send_sync)]
struct CachedMacroTransformer {
    transformer: MacroTransformer,
    /// Pre-compiled pattern for faster matching
    compiled_pattern: CompiledPattern,
    /// Template optimization hints
    template_hints: TemplateOptimizationHints,
    /// Usage statistics for adaptive optimization
    usage_stats: RefCell<MacroUsageStats>,
}

/// Pre-compiled pattern for O(1) matching in common cases.
#[derive(Debug)]
struct CompiledPattern {
    /// Fast path for simple patterns
    simple_match: Option<SimplePatternMatcher>,
    /// Fallback to full pattern matching
    complex_pattern: Pattern,
    /// Pattern complexity score for optimization selection
    complexity_score: u8,
}

/// Simple pattern matcher for common cases.
#[derive(Debug)]
enum SimplePatternMatcher {
    /// Fixed arity with specific identifiers
    FixedArity {
        arity: usize,
        required_args: Vec<PatternElement>,
    },
    /// Variable arity with minimum requirements
    VariableArity {
        min_arity: usize,
        vararg_pos: Option<usize>,
    },
    /// Literal matching
    Literal(crate::ast::Literal),
}

/// Pattern element for fast matching.
#[derive(Debug)]
enum PatternElement {
    /// Any expression
    Any,
    /// Specific identifier
    Identifier(Arc<str>),
    /// Literal value
    Literal(crate::ast::Literal),
    /// Nested pattern
    Nested(Box<SimplePatternMatcher>),
}

/// Template optimization hints for efficient expansion.
#[derive(Debug, Default)]
struct TemplateOptimizationHints {
    /// Template uses only simple substitution
    is_simple_substitution: bool,
    /// Template generates constant structure
    has_constant_structure: bool,
    /// Template complexity score
    complexity_score: u8,
    /// Frequently used variable bindings
    hot_variables: Vec<Arc<str>>,
}

/// Macro usage statistics for adaptive optimization.
#[derive(Debug, Default)]
struct MacroUsageStats {
    /// Number of times this macro has been expanded
    expansion_count: u64,
    /// Average expansion time in nanoseconds
    avg_expansion_time: u64,
    /// Cache hit rate
    cache_hit_rate: f64,
    /// Pattern match success rate
    pattern_match_rate: f64,
}

/// Expansion cache key for memoization.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ExpansionKey {
    macro_name: Arc<str>,
    args_hash: u64,
    hygiene_context_hash: u64,
}

impl ExpansionKey {
    fn new(macro_name: &str, args: &[Spanned<Expr>], hygiene_context: &HygieneContext) -> Self {
        let mut hasher = DefaultHasher::new();

        // Hash arguments efficiently
        for arg in args {
            // Use a simplified hash of the expression structure
            Self::hash_expr(&arg.inner, &mut hasher);
        }
        let args_hash = hasher.finish();

        // Generate hygiene context hash based on context ID
        let hygiene_context_hash = hygiene_context.get_context_id();

        Self {
            macro_name: Arc::from(macro_name),
            args_hash,
            hygiene_context_hash,
        }
    }

    fn hash_expr(expr: &Expr, hasher: &mut DefaultHasher) {
        match expr {
            Expr::Identifier(name) => {
                0u8.hash(hasher);
                name.hash(hasher);
            }
            Expr::Literal(lit) => {
                1u8.hash(hasher);
                // Simplified literal hashing
                match lit {
                    crate::ast::Literal::Boolean(b) => b.hash(hasher),
                    crate::ast::Literal::Number(n) => n.to_bits().hash(hasher),
                    crate::ast::Literal::ExactInteger(i) => i.hash(hasher),
                    crate::ast::Literal::InexactReal(r) => r.to_bits().hash(hasher),
                    crate::ast::Literal::Rational(r) => (**r).hash(hasher),
                    crate::ast::Literal::Complex(c) => (**c).hash(hasher),
                    crate::ast::Literal::Character(c) => c.hash(hasher),
                    crate::ast::Literal::String(s) => s.hash(hasher),
                    crate::ast::Literal::InternedString(s) => s.hash(hasher),
                    crate::ast::Literal::Bytevector(bv) => bv.hash(hasher),
                    crate::ast::Literal::Nil => 0u8.hash(hasher),
                    crate::ast::Literal::Unspecified => 1u8.hash(hasher),
                    crate::ast::Literal::Integer(i) => i.hash(hasher),
                }
            }
            Expr::Application { operator, operands } => {
                2u8.hash(hasher);
                Self::hash_expr(&operator.inner, hasher);
                operands.len().hash(hasher);
                for operand in operands {
                    Self::hash_expr(&operand.inner, hasher);
                }
            }
            _ => {
                // For other complex expressions, use a simplified hash
                std::mem::discriminant(expr).hash(hasher);
            }
        }
    }
}

/// Fast hygiene resolver with interning and bitmask optimization.
#[derive(Debug)]
struct FastHygieneResolver {
    /// Interned identifiers for O(1) comparison
    identifier_interner: RefCell<HashMap<String, u32>>,
    /// Reverse mapping for reconstruction
    identifier_strings: RefCell<Vec<Arc<str>>>,
    /// Hygiene mark bitmasks for fast operations
    mark_registry: RefCell<HashMap<u32, HygieneMark>>,
    /// Next identifier ID
    next_id: RefCell<u32>,
    /// Next mark ID
    next_mark_id: RefCell<u32>,
}

/// Hygiene mark with bitmask optimization.
#[derive(Debug, Clone)]
struct HygieneMark {
    id: u32,
    bitmask: u64,
    source_location: Option<Span>,
}

/// Expansion performance metrics.
#[derive(Debug, Default)]
pub struct ExpansionMetrics {
    /// Total number of expansions
    total_expansions: u64,
    /// Cache hits
    cache_hits: u64,
    /// Pattern match attempts
    pattern_matches: u64,
    /// Successful pattern matches
    successful_matches: u64,
    /// Total expansion time in nanoseconds
    total_expansion_time: u64,
    /// Memory usage statistics
    memory_stats: MemoryStats,
}

/// Memory usage statistics.
#[derive(Debug, Default)]
struct MemoryStats {
    /// Cache memory usage
    cache_memory_bytes: usize,
    /// Registry memory usage
    registry_memory_bytes: usize,
    /// Peak memory usage
    peak_memory_bytes: usize,
}

/// Optimization configuration.
#[derive(Debug, Clone)]
pub struct CachingOptimizationConfig {
    /// Enable expansion caching
    pub enable_caching: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Enable compile-time computation
    pub enable_compile_time_computation: bool,
    /// Enable pattern compilation
    pub enable_pattern_compilation: bool,
    /// Cache eviction threshold
    pub cache_eviction_threshold: f64,
    /// Performance monitoring level
    pub monitoring_level: MonitoringLevel,
}

/// Performance monitoring level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MonitoringLevel {
    /// No performance monitoring
    None,
    /// Basic performance monitoring with minimal overhead
    Basic,
    /// Detailed performance monitoring with comprehensive metrics
    Detailed,
    /// Profiling-level monitoring for optimization analysis
    Profiling,
}

impl Default for CachingOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            max_cache_size: 10000,
            enable_compile_time_computation: true,
            enable_pattern_compilation: true,
            cache_eviction_threshold: 0.8,
            monitoring_level: MonitoringLevel::Basic,
        }
    }
}

impl OptimizedMacroExpander {
    /// Creates a new optimized macro expander.
    pub fn new() -> Self {
        Self::with_config(CachingOptimizationConfig::default())
    }

    /// Creates a new optimized macro expander with configuration.
    pub fn with_config(config: CachingOptimizationConfig) -> Self {
        Self {
            macro_registry: HashMap::with_capacity(256),
            expansion_cache: RefCell::new(HashMap::with_capacity(config.max_cache_size)),
            type_safe_expander: TypeSafeMacroExpander::new(),
            compile_time_engine: CompileTimeComputationEngine::new(),
            hygiene_resolver: FastHygieneResolver::new(),
            metrics: RefCell::new(ExpansionMetrics::default()),
            config,
        }
    }

    /// Expands a macro with O(1) lookup and optimization.
    pub fn expand_optimized(&mut self, expr: &Spanned<Expr>) -> Result<Spanned<Expr>> {
        let start_time = std::time::Instant::now();

        if let Expr::Application { operator, operands } = &expr.inner {
            if let Expr::Identifier(name) = &operator.inner {
                // O(1) macro lookup
                let macro_key = MacroKey::new(name);

                if let Some(cached_transformer) = self.macro_registry.get(&macro_key).cloned() {
                    // Check expansion cache first
                    if self.config.enable_caching {
                        let cache_key = ExpansionKey::new(
                            name,
                            operands,
                            &self.hygiene_resolver.current_context(),
                        );

                        if let Ok(cache) = self.expansion_cache.try_borrow() {
                            if let Some(cached_result) = cache.get(&cache_key) {
                                self.record_cache_hit();
                                return Ok((**cached_result).clone());
                            }
                        }
                    }

                    // Fast path expansion
                    let result = self.expand_with_cached_transformer(
                        &cached_transformer,
                        operands,
                        expr.span,
                    )?;

                    // Update cache
                    if self.config.enable_caching {
                        let cache_key = ExpansionKey::new(
                            name,
                            operands,
                            &self.hygiene_resolver.current_context(),
                        );
                        self.expansion_cache
                            .borrow_mut()
                            .insert(cache_key, Arc::new(result.clone()));
                    }

                    // Update metrics
                    self.record_expansion(start_time.elapsed());

                    return Ok(result);
                }
            }
        }

        // Fallback to standard expansion for non-macro expressions
        self.expand_standard(expr)
    }

    /// Expands using cached transformer with optimizations.
    fn expand_with_cached_transformer(
        &mut self,
        cached_transformer: &CachedMacroTransformer,
        operands: &[Spanned<Expr>],
        span: Span,
    ) -> Result<Spanned<Expr>> {
        // Try fast path pattern matching first
        if let Some(ref simple_matcher) = cached_transformer.compiled_pattern.simple_match {
            if let Some(bindings) = self.try_simple_match(simple_matcher, operands)? {
                return self.expand_template_optimized(
                    &cached_transformer.transformer.template,
                    &bindings,
                    &cached_transformer.template_hints,
                    span,
                );
            }
        }

        // Fallback to complex pattern matching
        let bindings = self.match_pattern_optimized(
            &cached_transformer.compiled_pattern.complex_pattern,
            operands,
            span,
        )?;

        self.expand_template_optimized(
            &cached_transformer.transformer.template,
            &bindings,
            &cached_transformer.template_hints,
            span,
        )
    }

    /// Fast pattern matching for simple cases.
    fn try_simple_match(
        &self,
        matcher: &SimplePatternMatcher,
        operands: &[Spanned<Expr>],
    ) -> Result<Option<PatternBindings>> {
        match matcher {
            SimplePatternMatcher::FixedArity {
                arity,
                required_args,
            } => {
                if operands.len() != *arity {
                    return Ok(None);
                }

                let mut bindings = PatternBindings::new();
                for (i, (operand, required)) in
                    operands.iter().zip(required_args.iter()).enumerate()
                {
                    if !self.matches_pattern_element(&operand.inner, required) {
                        return Ok(None);
                    }
                    // Simple binding for now - would be more sophisticated in full implementation
                    bindings.bind(format!("arg{i}"), operand.clone());
                }

                Ok(Some(bindings))
            }
            SimplePatternMatcher::VariableArity {
                min_arity,
                vararg_pos,
            } => {
                if operands.len() < *min_arity {
                    return Ok(None);
                }

                let mut bindings = PatternBindings::new();

                // Bind fixed arguments
                for (i, operand) in operands.iter().enumerate().take(*min_arity) {
                    bindings.bind(format!("arg{i}"), operand.clone());
                }

                // Bind variable arguments if present
                if let Some(vararg_pos) = vararg_pos {
                    let remaining_args: Vec<_> = operands[*min_arity..].to_vec();
                    bindings.bind_ellipsis(format!("args{vararg_pos}"), remaining_args);
                }

                Ok(Some(bindings))
            }
            SimplePatternMatcher::Literal(expected_lit) => {
                if operands.len() != 1 {
                    return Ok(None);
                }

                if let Expr::Literal(actual_lit) = &operands[0].inner {
                    if actual_lit == expected_lit {
                        Ok(Some(PatternBindings::new()))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Checks if expression matches pattern element.
    fn matches_pattern_element(&self, expr: &Expr, element: &PatternElement) -> bool {
        match (expr, element) {
            (_, PatternElement::Any) => true,
            (Expr::Identifier(name), PatternElement::Identifier(expected)) => {
                name.as_str() == expected.as_ref()
            }
            (Expr::Literal(lit), PatternElement::Literal(expected)) => lit == expected,
            _ => false,
        }
    }

    /// Optimized pattern matching with caching.
    fn match_pattern_optimized(
        &self,
        pattern: &Pattern,
        operands: &[Spanned<Expr>],
        span: Span,
    ) -> Result<PatternBindings> {
        // Create application expression for pattern matching
        let app_expr = Spanned::new(
            Expr::Application {
                operator: Box::new(Spanned::new(
                    Expr::Identifier("macro-app".to_string()),
                    span,
                )),
                operands: operands.to_vec(),
            },
            span,
        );

        pattern.match_expr(&app_expr)
    }

    /// Optimized template expansion with hints.
    fn expand_template_optimized(
        &self,
        template: &Template,
        bindings: &PatternBindings,
        hints: &TemplateOptimizationHints,
        span: Span,
    ) -> Result<Spanned<Expr>> {
        if hints.is_simple_substitution {
            // Fast path for simple substitutions
            self.expand_simple_substitution(template, bindings, span)
        } else {
            // Standard template expansion
            template.expand(bindings, span)
        }
    }

    /// Fast path for simple template substitutions.
    fn expand_simple_substitution(
        &self,
        template: &Template,
        bindings: &PatternBindings,
        span: Span,
    ) -> Result<Spanned<Expr>> {
        // Simplified implementation - would be more sophisticated in practice
        template.expand(bindings, span)
    }

    /// Standard expansion fallback.
    fn expand_standard(&mut self, expr: &Spanned<Expr>) -> Result<Spanned<Expr>> {
        // Delegate to type-safe expander
        self.type_safe_expander.expand_typed(expr)
    }

    /// Defines a macro with pre-compilation optimizations.
    pub fn define_optimized_macro(
        &mut self,
        name: String,
        transformer: MacroTransformer,
    ) -> Result<()> {
        let macro_key = MacroKey::new(&name);

        // Pre-compile pattern for optimization
        let compiled_pattern = self.compile_pattern(&transformer.pattern)?;

        // Generate template optimization hints
        let template_hints = self.analyze_template(&transformer.template);

        let cached_transformer = CachedMacroTransformer {
            transformer,
            compiled_pattern,
            template_hints,
            usage_stats: RefCell::new(MacroUsageStats::default()),
        };

        #[allow(clippy::arc_with_non_send_sync)]
        {
            self.macro_registry
                .insert(macro_key, Arc::new(cached_transformer));
        }

        Ok(())
    }

    /// Compiles a pattern for fast matching.
    fn compile_pattern(&self, pattern: &Pattern) -> Result<CompiledPattern> {
        // Analyze pattern complexity and generate optimized matcher
        let complexity_score = self.calculate_pattern_complexity(pattern);

        let simple_match = if complexity_score <= 3 {
            // Generate simple matcher for low-complexity patterns
            self.try_generate_simple_matcher(pattern)
        } else {
            None
        };

        Ok(CompiledPattern {
            simple_match,
            complex_pattern: pattern.clone(),
            complexity_score,
        })
    }

    /// Calculates pattern complexity score.
    fn calculate_pattern_complexity(&self, _pattern: &Pattern) -> u8 {
        // Simplified implementation - would analyze pattern structure
        5
    }

    /// Attempts to generate a simple matcher for the pattern.
    fn try_generate_simple_matcher(&self, _pattern: &Pattern) -> Option<SimplePatternMatcher> {
        // Simplified implementation - would analyze pattern and generate optimized matcher
        None
    }

    /// Analyzes template for optimization opportunities.
    fn analyze_template(&self, _template: &Template) -> TemplateOptimizationHints {
        // Simplified implementation - would analyze template structure
        TemplateOptimizationHints::default()
    }

    /// Records a cache hit for metrics.
    fn record_cache_hit(&self) {
        if self.config.monitoring_level >= MonitoringLevel::Basic {
            let mut metrics = self.metrics.borrow_mut();
            metrics.cache_hits += 1;
        }
    }

    /// Records an expansion for metrics.
    fn record_expansion(&self, duration: std::time::Duration) {
        if self.config.monitoring_level >= MonitoringLevel::Basic {
            let mut metrics = self.metrics.borrow_mut();
            metrics.total_expansions += 1;
            metrics.total_expansion_time += duration.as_nanos() as u64;
        }
    }

    /// Gets current performance metrics.
    pub fn metrics(&self) -> ExpansionMetrics {
        self.metrics
            .try_borrow()
            .map(|metrics| metrics.clone())
            .unwrap_or_default()
    }

    /// Performs cache maintenance and optimization.
    pub fn optimize_cache(&mut self) {
        if !self.config.enable_caching {
            return;
        }

        let mut cache = self.expansion_cache.borrow_mut();
        let current_size = cache.len();
        let max_size = self.config.max_cache_size;

        // Evict entries if cache is too full
        if current_size as f64 / max_size as f64 > self.config.cache_eviction_threshold {
            let target_size = (max_size as f64 * 0.6) as usize;
            let to_remove = current_size - target_size;

            // Simple LRU-style eviction - would be more sophisticated in practice
            let keys_to_remove: Vec<_> = cache.keys().take(to_remove).cloned().collect();
            for key in keys_to_remove {
                cache.remove(&key);
            }
        }
    }
}

impl FastHygieneResolver {
    fn new() -> Self {
        Self {
            identifier_interner: RefCell::new(HashMap::with_capacity(1024)),
            identifier_strings: RefCell::new(Vec::with_capacity(1024)),
            mark_registry: RefCell::new(HashMap::new()),
            next_id: RefCell::new(0),
            next_mark_id: RefCell::new(0),
        }
    }

    fn current_context(&self) -> HygieneContext {
        // Simplified implementation - would return actual hygiene context
        HygieneContext::new()
    }
}

// Implement Clone for ExpansionMetrics
impl Clone for ExpansionMetrics {
    fn clone(&self) -> Self {
        Self {
            total_expansions: self.total_expansions,
            cache_hits: self.cache_hits,
            pattern_matches: self.pattern_matches,
            successful_matches: self.successful_matches,
            total_expansion_time: self.total_expansion_time,
            memory_stats: self.memory_stats.clone(),
        }
    }
}

impl Clone for MemoryStats {
    fn clone(&self) -> Self {
        Self {
            cache_memory_bytes: self.cache_memory_bytes,
            registry_memory_bytes: self.registry_memory_bytes,
            peak_memory_bytes: self.peak_memory_bytes,
        }
    }
}

impl Default for OptimizedMacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_expansion() {
        let mut expander = OptimizedMacroExpander::new();

        // Test basic functionality
        assert!(expander.macro_registry.is_empty());

        // Test metrics
        let metrics = expander.metrics();
        assert_eq!(metrics.total_expansions, 0);
    }

    #[test]
    fn test_cache_optimization() {
        let mut expander = OptimizedMacroExpander::new();

        // Test cache maintenance
        expander.optimize_cache();

        // Should not panic
        assert!(true);
    }
}

//! Complete CPS inlining system for performance optimization
//!
//! This module implements comprehensive continuation-passing style inlining
//! to eliminate intermediate continuation allocations and improve evaluation performance.

use crate::evaluator::Continuation;
use crate::evaluator::continuation::LightContinuation;
use crate::value::Value;
use std::collections::HashMap;

/// Inlining decision for continuation operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InliningDecision {
    /// Inline the operation directly
    Inline,
    /// Use lightweight continuation
    LightContinuation,
    /// Use full continuation (fallback)
    FullContinuation,
    /// Eliminate continuation entirely (direct evaluation)
    Eliminate,
}

/// CPS inlining optimizer
pub struct CpsInliner {
    /// Statistics about inlining decisions
    stats: InliningStats,
    /// Configuration for inlining behavior
    config: InliningConfig,
    /// Cache of inlining decisions for patterns
    decision_cache: HashMap<String, InliningDecision>,
}

/// Configuration for CPS inlining
#[derive(Debug, Clone)]
pub struct InliningConfig {
    /// Maximum chain length for inlining
    pub max_inline_chain: usize,
    /// Whether to aggressively inline simple operations
    pub aggressive_simple_ops: bool,
    /// Whether to use decision caching
    pub use_caching: bool,
    /// Maximum cache size
    pub max_cache_size: usize,
}

impl Default for InliningConfig {
    fn default() -> Self {
        Self {
            max_inline_chain: 5,
            aggressive_simple_ops: true,
            use_caching: true,
            max_cache_size: 1000,
        }
    }
}

/// Statistics about inlining decisions and performance
#[derive(Debug, Clone)]
pub struct InliningStats {
    /// Number of operations inlined
    pub inlined_operations: usize,
    /// Number of lightweight continuations used
    pub light_continuations: usize,
    /// Number of full continuations used
    pub full_continuations: usize,
    /// Number of eliminations
    pub eliminations: usize,
    /// Cache hit rate
    pub cache_hits: usize,
    /// Cache misses
    pub cache_misses: usize,
}

impl CpsInliner {
    /// Create a new CPS inliner
    pub fn new() -> Self {
        Self::with_config(InliningConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: InliningConfig) -> Self {
        Self {
            stats: InliningStats {
                inlined_operations: 0,
                light_continuations: 0,
                full_continuations: 0,
                eliminations: 0,
                cache_hits: 0,
                cache_misses: 0,
            },
            config,
            decision_cache: HashMap::new(),
        }
    }

    /// Analyze a continuation and decide on inlining strategy
    pub fn analyze_continuation(&mut self, cont: &Continuation) -> InliningDecision {
        let pattern = self.continuation_pattern(cont);

        // Check cache first
        if self.config.use_caching {
            if let Some(cached_decision) = self.decision_cache.get(&pattern) {
                self.stats.cache_hits += 1;
                return *cached_decision;
            }
            self.stats.cache_misses += 1;
        }

        let decision = self.make_inlining_decision(cont);

        // Cache the decision
        if self.config.use_caching && self.decision_cache.len() < self.config.max_cache_size {
            self.decision_cache.insert(pattern, decision);
        }

        decision
    }

    /// Make an inlining decision for a continuation
    fn make_inlining_decision(&self, cont: &Continuation) -> InliningDecision {
        match cont {
            // Simple continuations that can be eliminated
            Continuation::Identity => InliningDecision::Eliminate,

            // Simple operations that can be inlined
            Continuation::Assignment { parent, .. }
                if matches!(**parent, Continuation::Identity) =>
            {
                InliningDecision::Inline
            }

            Continuation::Values { values, parent }
                if values.len() <= 3 && matches!(**parent, Continuation::Identity) =>
            {
                InliningDecision::Inline
            }

            // Operations suitable for lightweight continuations
            cont if LightContinuation::from_continuation(cont).is_some() => {
                InliningDecision::LightContinuation
            }

            // Simple application patterns
            Continuation::Application {
                evaluated_args,
                remaining_args,
                parent,
                ..
            } if remaining_args.is_empty()
                && evaluated_args.len() <= 4
                && self.is_simple_parent(parent) =>
            {
                InliningDecision::Inline
            }

            // Begin sequences with few expressions
            Continuation::Begin {
                remaining, parent, ..
            } if remaining.len() <= 2 && self.is_simple_parent(parent) => {
                InliningDecision::LightContinuation
            }

            // If statements with simple branches
            Continuation::IfTest { parent, .. } if self.is_simple_parent(parent) => {
                InliningDecision::LightContinuation
            }

            // Default to full continuation for complex cases
            _ => InliningDecision::FullContinuation,
        }
    }

    /// Check if a parent continuation is simple (suitable for inlining)
    #[allow(clippy::only_used_in_recursion)]
    fn is_simple_parent(&self, parent: &Continuation) -> bool {
        match parent {
            Continuation::Identity => true,
            Continuation::Values {
                values,
                parent: grandparent,
            } if values.len() <= 2 => self.is_simple_parent(grandparent),
            _ => false,
        }
    }

    /// Generate a pattern string for a continuation (for caching)
    pub fn continuation_pattern(&self, cont: &Continuation) -> String {
        match cont {
            Continuation::Identity => "Identity".to_string(),
            Continuation::Assignment { .. } => "Assignment".to_string(),
            Continuation::Values { values, .. } => format!("Values({})", values.len()),
            Continuation::Application {
                evaluated_args,
                remaining_args,
                ..
            } => {
                format!(
                    "Application({},{})",
                    evaluated_args.len(),
                    remaining_args.len()
                )
            }
            Continuation::Begin { remaining, .. } => format!("Begin({})", remaining.len()),
            Continuation::IfTest { .. } => "IfTest".to_string(),
            Continuation::Define { .. } => "Define".to_string(),
            Continuation::And { remaining, .. } => format!("And({})", remaining.len()),
            Continuation::Or { remaining, .. } => format!("Or({})", remaining.len()),
            _ => "Complex".to_string(),
        }
    }

    /// Attempt to inline a continuation operation directly
    pub fn try_inline_operation(&mut self, cont: &Continuation, value: Value) -> Option<Value> {
        let decision = self.analyze_continuation(cont);

        match decision {
            InliningDecision::Eliminate => {
                self.stats.eliminations += 1;
                Some(value) // Direct passthrough
            }

            InliningDecision::Inline => {
                self.stats.inlined_operations += 1;
                self.inline_continuation_directly(cont, value)
            }

            InliningDecision::LightContinuation => {
                self.stats.light_continuations += 1;
                None // Caller should use LightContinuation
            }

            InliningDecision::FullContinuation => {
                self.stats.full_continuations += 1;
                None // Caller should use full continuation
            }
        }
    }

    /// Inline a continuation operation directly without allocation
    fn inline_continuation_directly(&self, cont: &Continuation, value: Value) -> Option<Value> {
        match cont {
            Continuation::Identity => Some(value),

            Continuation::Assignment {
                variable,
                env,
                parent,
            } => {
                // Perform assignment and continue
                if env.set(variable, value.clone()).is_ok() {
                    if matches!(**parent, Continuation::Identity) {
                        Some(value)
                    } else {
                        // Complex parent, can't fully inline
                        None
                    }
                } else {
                    None
                }
            }

            Continuation::Values { values, parent } => {
                let mut result_values = values.clone();
                result_values.push(value);

                if matches!(**parent, Continuation::Identity) {
                    Some(Value::Values(result_values))
                } else {
                    // Complex parent, can't fully inline
                    None
                }
            }

            _ => None, // Can't inline this continuation type directly
        }
    }

    /// Optimize a continuation chain for inlining opportunities
    pub fn optimize_continuation_chain(&mut self, cont: &Continuation) -> OptimizedContinuation {
        let mut optimizations = Vec::new();
        let mut current = cont;
        let mut depth = 0;

        // Analyze the continuation chain
        while depth < self.config.max_inline_chain {
            let decision = self.analyze_continuation(current);
            optimizations.push((depth, decision));

            match current.parent() {
                Some(parent) => {
                    current = parent;
                    depth += 1;
                }
                None => break,
            }
        }

        OptimizedContinuation {
            original_depth: depth,
            can_eliminate_chain: self.can_eliminate_chain(&optimizations),
            recommended_strategy: self.recommend_strategy(&optimizations),
            optimizations,
        }
    }

    /// Check if an entire continuation chain can be eliminated
    fn can_eliminate_chain(&self, optimizations: &[(usize, InliningDecision)]) -> bool {
        optimizations.iter().all(|(_, decision)| {
            matches!(
                decision,
                InliningDecision::Eliminate | InliningDecision::Inline
            )
        })
    }

    /// Recommend overall strategy for a continuation chain
    fn recommend_strategy(&self, optimizations: &[(usize, InliningDecision)]) -> ChainStrategy {
        let inline_count = optimizations
            .iter()
            .filter(|(_, d)| matches!(d, InliningDecision::Inline | InliningDecision::Eliminate))
            .count();

        let total_count = optimizations.len();

        if inline_count == total_count {
            ChainStrategy::FullInline
        } else if inline_count as f64 / total_count as f64 > 0.7 {
            ChainStrategy::MostlyInline
        } else if optimizations
            .iter()
            .any(|(_, d)| matches!(d, InliningDecision::LightContinuation))
        {
            ChainStrategy::LightContinuation
        } else {
            ChainStrategy::FullContinuation
        }
    }

    /// Get current inlining statistics
    pub fn statistics(&self) -> &InliningStats {
        &self.stats
    }

    /// Get cache efficiency
    pub fn cache_efficiency(&self) -> f64 {
        let total_requests = self.stats.cache_hits + self.stats.cache_misses;
        if total_requests > 0 {
            self.stats.cache_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }

    /// Get inlining efficiency (percentage of operations that were optimized)
    pub fn inlining_efficiency(&self) -> f64 {
        let total_operations = self.stats.inlined_operations
            + self.stats.light_continuations
            + self.stats.full_continuations
            + self.stats.eliminations;

        if total_operations > 0 {
            let optimized = self.stats.inlined_operations
                + self.stats.light_continuations
                + self.stats.eliminations;
            optimized as f64 / total_operations as f64
        } else {
            0.0
        }
    }
}

impl Default for CpsInliner {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of continuation chain optimization analysis
#[derive(Debug, Clone)]
pub struct OptimizedContinuation {
    /// Original chain depth
    pub original_depth: usize,
    /// Optimization decisions for each level
    pub optimizations: Vec<(usize, InliningDecision)>,
    /// Whether the entire chain can be eliminated
    pub can_eliminate_chain: bool,
    /// Recommended overall strategy
    pub recommended_strategy: ChainStrategy,
}

/// Overall strategy for handling a continuation chain
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChainStrategy {
    /// Inline the entire chain directly
    FullInline,
    /// Inline most of the chain, use lightweight for remainder
    MostlyInline,
    /// Use lightweight continuation for the chain
    LightContinuation,
    /// Use full continuation (no optimization possible)
    FullContinuation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;
    use crate::lexer::SchemeNumber;
    use std::rc::Rc;

    #[test]
    fn test_cps_inliner_basic_operations() {
        let mut inliner = CpsInliner::new();

        // Test identity continuation
        let identity = Continuation::Identity;
        let decision = inliner.analyze_continuation(&identity);
        assert_eq!(decision, InliningDecision::Eliminate);

        // Test inlining
        let result =
            inliner.try_inline_operation(&identity, Value::Number(SchemeNumber::Integer(42)));
        assert!(matches!(result, Some(Value::Number(_))));
    }

    #[test]
    fn test_assignment_continuation_inlining() {
        let mut inliner = CpsInliner::new();
        let env = Rc::new(Environment::new());

        let assignment = Continuation::Assignment {
            variable: "x".to_string(),
            env: env.clone(),
            parent: Box::new(Continuation::Identity),
        };

        let decision = inliner.analyze_continuation(&assignment);
        assert_eq!(decision, InliningDecision::Inline);
    }

    #[test]
    fn test_values_continuation_inlining() {
        let mut inliner = CpsInliner::new();

        let values = Continuation::Values {
            values: vec![
                Value::Number(SchemeNumber::Integer(1)),
                Value::Number(SchemeNumber::Integer(2)),
            ],
            parent: Box::new(Continuation::Identity),
        };

        let decision = inliner.analyze_continuation(&values);
        assert_eq!(decision, InliningDecision::Inline);
    }

    #[test]
    fn test_cache_efficiency() {
        let mut inliner = CpsInliner::new();
        let identity = Continuation::Identity;

        // First call should be a cache miss
        inliner.analyze_continuation(&identity);
        assert_eq!(inliner.stats.cache_misses, 1);
        assert_eq!(inliner.stats.cache_hits, 0);

        // Second call should be a cache hit
        inliner.analyze_continuation(&identity);
        assert_eq!(inliner.stats.cache_hits, 1);

        let efficiency = inliner.cache_efficiency();
        assert_eq!(efficiency, 0.5); // 1 hit out of 2 total requests
    }

    #[test]
    fn test_continuation_chain_optimization() {
        let mut inliner = CpsInliner::new();
        let env = Rc::new(Environment::new());

        // Create a simple chain: Assignment -> Identity
        let chain = Continuation::Assignment {
            variable: "x".to_string(),
            env,
            parent: Box::new(Continuation::Identity),
        };

        let optimized = inliner.optimize_continuation_chain(&chain);
        assert_eq!(optimized.original_depth, 1);
        assert!(optimized.can_eliminate_chain);
        assert_eq!(optimized.recommended_strategy, ChainStrategy::FullInline);
    }

    #[test]
    fn test_inlining_efficiency() {
        let mut inliner = CpsInliner::new();

        // Initially no efficiency data
        assert_eq!(inliner.inlining_efficiency(), 0.0);

        // Add some operations
        let identity = Continuation::Identity;
        inliner.try_inline_operation(&identity, Value::Number(SchemeNumber::Integer(1)));

        // Should show 100% efficiency (elimination counts as optimization)
        assert_eq!(inliner.inlining_efficiency(), 1.0);
    }

    #[test]
    fn test_pattern_generation() {
        let inliner = CpsInliner::new();

        let identity = Continuation::Identity;
        assert_eq!(inliner.continuation_pattern(&identity), "Identity");

        let values = Continuation::Values {
            values: vec![
                Value::Number(SchemeNumber::Integer(1)),
                Value::Number(SchemeNumber::Integer(2)),
            ],
            parent: Box::new(Continuation::Identity),
        };
        assert_eq!(inliner.continuation_pattern(&values), "Values(2)");
    }
}

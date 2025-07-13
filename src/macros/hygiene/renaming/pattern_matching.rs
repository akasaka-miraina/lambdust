//! Pattern Matching System for Symbol Renaming
//!
//! このモジュールはシンボルリネーミングのためのパターンマッチング
//! ロジックと述語関数を実装します。

use super::core_types::{
    PatternMatcher, PredicateFunction, BuiltInPredicate, RenamingPattern,
    ScopeConstraint, TypeConstraint
};
use super::super::context::ExpansionContext;

/// Pattern matching engine for symbol renaming
#[derive(Debug)]
pub struct PatternMatchingEngine {
    /// Pattern matching cache for performance
    pattern_cache: std::collections::HashMap<String, bool>,
    /// Statistics for pattern matching performance
    stats: PatternMatchingStats,
}

/// Statistics for pattern matching operations
#[derive(Debug, Default)]
pub struct PatternMatchingStats {
    /// Total patterns evaluated
    pub patterns_evaluated: u64,
    /// Pattern cache hits
    pub cache_hits: u64,
    /// Pattern cache misses
    pub cache_misses: u64,
    /// Time spent on pattern matching (nanoseconds)
    pub matching_time_ns: u64,
}

impl PatternMatchingEngine {
    /// Create new pattern matching engine
    #[must_use] pub fn new() -> Self {
        Self {
            pattern_cache: std::collections::HashMap::new(),
            stats: PatternMatchingStats::default(),
        }
    }

    /// Check if name matches pattern with context
    pub fn matches_pattern(
        &mut self,
        name: &str,
        context: &ExpansionContext,
        pattern: &RenamingPattern,
    ) -> bool {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        self.stats.patterns_evaluated += 1;

        // Use advanced pattern matching
        let name_matches = self.matches_name_pattern(name, &pattern.name_pattern);
        
        if !name_matches {
            self.update_timing(start_time);
            return false;
        }
        
        // Check macro context if specified
        if let Some(ref macro_pattern) = pattern.macro_context {
            if let Some(current_macro) = context.current_macro() {
                if current_macro != macro_pattern {
                    self.update_timing(start_time);
                    return false;
                }
            } else {
                self.update_timing(start_time);
                return false;
            }
        }

        // Check scope depth constraint
        if let Some(ref scope_constraint) = pattern.scope_depth {
            if !scope_constraint.satisfies(context.depth) {
                self.update_timing(start_time);
                return false;
            }
        }

        // Type constraint checking would require additional type information
        // For now, we assume it passes if not specified
        if pattern.type_constraint.is_some() {
            // In a full implementation, this would check type information
            // For now, we'll treat it as always matching
        }

        self.update_timing(start_time);
        true
    }

    /// Match name against pattern matcher
    fn matches_name_pattern(&mut self, name: &str, pattern: &PatternMatcher) -> bool {
        match pattern {
            PatternMatcher::Exact(exact) => name == exact,
            PatternMatcher::Glob(glob_pattern) => self.matches_glob(name, glob_pattern),
            PatternMatcher::Regex(regex_pattern) => self.matches_simple_regex(name, regex_pattern),
            PatternMatcher::Predicate(predicate) => self.matches_predicate(name, predicate),
            PatternMatcher::Multiple(patterns) => {
                patterns.iter().any(|p| self.matches_name_pattern(name, p))
            }
        }
    }

    /// Match glob patterns (supports * and ?)
    fn matches_glob(&mut self, name: &str, pattern: &str) -> bool {
        // Check cache first
        let cache_key = format!("glob:{}:{}", pattern, name);
        if let Some(&result) = self.pattern_cache.get(&cache_key) {
            self.stats.cache_hits += 1;
            return result;
        }

        self.stats.cache_misses += 1;

        // Simple glob implementation
        let result = if pattern == "*" {
            true
        } else if pattern.contains('*') {
            self.matches_glob_with_star(name, pattern)
        } else if pattern.contains('?') {
            self.matches_glob_with_question(name, pattern)
        } else {
            name == pattern
        };

        // Cache the result
        self.pattern_cache.insert(cache_key, result);
        result
    }

    /// Match glob pattern with star wildcard
    fn matches_glob_with_star(&self, name: &str, pattern: &str) -> bool {
        let parts: Vec<&str> = pattern.split('*').collect();
        
        if parts.len() == 2 {
            let (prefix, suffix) = (parts[0], parts[1]);
            return name.starts_with(prefix) && name.ends_with(suffix) && 
                   name.len() >= prefix.len() + suffix.len();
        }

        // For multiple stars, use more complex matching
        if parts.len() > 2 {
            return self.matches_multiple_wildcards(name, &parts);
        }

        false
    }

    /// Match glob pattern with question mark wildcard
    fn matches_glob_with_question(&self, name: &str, pattern: &str) -> bool {
        // Simple single character wildcard
        if name.len() != pattern.len() {
            return false;
        }
        
        name.chars().zip(pattern.chars())
            .all(|(c, p)| p == '?' || c == p)
    }

    /// Match pattern with multiple wildcards
    fn matches_multiple_wildcards(&self, name: &str, parts: &[&str]) -> bool {
        if parts.is_empty() {
            return name.is_empty();
        }

        let first_part = parts[0];
        if !name.starts_with(first_part) {
            return false;
        }

        if parts.len() == 1 {
            return name == first_part;
        }

        // Find the first occurrence of the next part
        let remaining_name = &name[first_part.len()..];
        let next_part = parts[1];
        
        if let Some(pos) = remaining_name.find(next_part) {
            let after_next = &remaining_name[pos + next_part.len()..];
            return self.matches_multiple_wildcards(after_next, &parts[2..]);
        }

        false
    }

    /// Simple regex matching (without regex crate)
    fn matches_simple_regex(&mut self, name: &str, pattern: &str) -> bool {
        // Check cache first
        let cache_key = format!("regex:{}:{}", pattern, name);
        if let Some(&result) = self.pattern_cache.get(&cache_key) {
            self.stats.cache_hits += 1;
            return result;
        }

        self.stats.cache_misses += 1;

        // For now, treat as literal string match
        // In a full implementation, this would use the regex crate
        let result = name == pattern;
        
        // Cache the result
        self.pattern_cache.insert(cache_key, result);
        result
    }

    /// Match predicate functions
    fn matches_predicate(&self, name: &str, predicate: &PredicateFunction) -> bool {
        match predicate {
            PredicateFunction::BuiltIn(built_in) => self.matches_builtin_predicate(name, built_in),
            PredicateFunction::UserDefined(_) => {
                // User-defined predicates would need custom implementation
                false
            }
        }
    }

    /// Match built-in predicate functions
    fn matches_builtin_predicate(&self, name: &str, predicate: &BuiltInPredicate) -> bool {
        match predicate {
            BuiltInPredicate::StartsWith(prefix) => name.starts_with(prefix),
            BuiltInPredicate::EndsWith(suffix) => name.ends_with(suffix),
            BuiltInPredicate::Contains(substring) => name.contains(substring),
            BuiltInPredicate::LengthRange(min, max) => {
                let len = name.len();
                len >= *min && len <= *max
            }
            BuiltInPredicate::IsAlphanumeric => {
                name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            }
            BuiltInPredicate::IsLispCase => {
                name.chars().all(|c| {
                    c.is_lowercase() || c.is_numeric() || 
                    c == '-' || c == '_' || c == '?' || c == '!'
                })
            }
            BuiltInPredicate::IsTemporary => self.is_likely_temporary_variable(name),
        }
    }

    /// Check if name suggests a temporary variable
    fn is_likely_temporary_variable(&self, name: &str) -> bool {
        name.starts_with("temp") || 
        name.starts_with("tmp") || 
        name.starts_with('_') ||
        matches!(name, "t" | "x" | "y" | "z" | "i" | "j" | "k")
    }

    /// Update timing statistics
    fn update_timing(&mut self, start_time: u64) {
        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        self.stats.matching_time_ns += end_time.saturating_sub(start_time);
    }

    /// Get pattern matching statistics
    #[must_use] pub fn get_stats(&self) -> &PatternMatchingStats {
        &self.stats
    }

    /// Clear cache and reset statistics
    pub fn clear_cache(&mut self) {
        self.pattern_cache.clear();
        self.stats = PatternMatchingStats::default();
    }

    /// Optimize cache by removing old entries
    pub fn optimize_cache(&mut self) {
        if self.pattern_cache.len() > 1000 {
            self.pattern_cache.clear();
        }
    }

    /// Get cache statistics
    #[must_use] pub fn cache_info(&self) -> CacheInfo {
        CacheInfo {
            size: self.pattern_cache.len(),
            hits: self.stats.cache_hits,
            misses: self.stats.cache_misses,
            hit_rate: if self.stats.cache_hits + self.stats.cache_misses > 0 {
                self.stats.cache_hits as f64 / (self.stats.cache_hits + self.stats.cache_misses) as f64
            } else {
                0.0
            },
        }
    }
}

impl Default for PatternMatchingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache information for pattern matching
#[derive(Debug, Clone)]
pub struct CacheInfo {
    /// Current cache size
    pub size: usize,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Hit rate (0.0 to 1.0)
    pub hit_rate: f64,
}

impl PatternMatchingStats {
    /// Calculate average matching time per pattern (nanoseconds)
    #[must_use] pub fn avg_matching_time(&self) -> f64 {
        if self.patterns_evaluated == 0 {
            0.0
        } else {
            self.matching_time_ns as f64 / self.patterns_evaluated as f64
        }
    }

    /// Calculate cache hit rate
    #[must_use] pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

/// Pattern builder for creating complex patterns
#[derive(Debug)]
pub struct PatternBuilder {
    matcher: Option<PatternMatcher>,
    macro_context: Option<String>,
    scope_depth: Option<ScopeConstraint>,
    type_constraint: Option<TypeConstraint>,
}

impl PatternBuilder {
    /// Create new pattern builder
    #[must_use] pub fn new() -> Self {
        Self {
            matcher: None,
            macro_context: None,
            scope_depth: None,
            type_constraint: None,
        }
    }

    /// Set exact string matcher
    pub fn exact(mut self, pattern: String) -> Self {
        self.matcher = Some(PatternMatcher::Exact(pattern));
        self
    }

    /// Set glob matcher
    pub fn glob(mut self, pattern: String) -> Self {
        self.matcher = Some(PatternMatcher::Glob(pattern));
        self
    }

    /// Set regex matcher
    pub fn regex(mut self, pattern: String) -> Self {
        self.matcher = Some(PatternMatcher::Regex(pattern));
        self
    }

    /// Set predicate matcher
    pub fn predicate(mut self, predicate: PredicateFunction) -> Self {
        self.matcher = Some(PatternMatcher::Predicate(predicate));
        self
    }

    /// Set multiple matchers
    pub fn multiple(mut self, matchers: Vec<PatternMatcher>) -> Self {
        self.matcher = Some(PatternMatcher::Multiple(matchers));
        self
    }

    /// Set macro context constraint
    pub fn in_macro(mut self, macro_name: String) -> Self {
        self.macro_context = Some(macro_name);
        self
    }

    /// Set scope depth constraint
    pub fn at_scope(mut self, constraint: ScopeConstraint) -> Self {
        self.scope_depth = Some(constraint);
        self
    }

    /// Set type constraint
    pub fn with_type(mut self, constraint: TypeConstraint) -> Self {
        self.type_constraint = Some(constraint);
        self
    }

    /// Build the pattern matcher
    pub fn build(self) -> Option<PatternMatcher> {
        self.matcher
    }

    /// Build complete renaming pattern
    pub fn build_pattern(
        self, 
        action: super::core_types::RenamingAction, 
        priority: u32
    ) -> Option<RenamingPattern> {
        self.matcher.map(|matcher| RenamingPattern {
            name_pattern: matcher,
            macro_context: self.macro_context,
            scope_depth: self.scope_depth,
            type_constraint: self.type_constraint,
            action,
            priority,
        })
    }
}

impl Default for PatternBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macros::hygiene::symbol::EnvironmentId;

    #[test]
    fn test_exact_pattern_matching() {
        let mut engine = PatternMatchingEngine::new();
        let matcher = PatternMatcher::Exact("test".to_string());
        
        assert!(engine.matches_name_pattern("test", &matcher));
        assert!(!engine.matches_name_pattern("test123", &matcher));
    }

    #[test]
    fn test_glob_pattern_matching() {
        let mut engine = PatternMatchingEngine::new();
        
        let matcher = PatternMatcher::Glob("test*".to_string());
        assert!(engine.matches_name_pattern("test", &matcher));
        assert!(engine.matches_name_pattern("test123", &matcher));
        assert!(!engine.matches_name_pattern("other", &matcher));

        let matcher = PatternMatcher::Glob("*test".to_string());
        assert!(engine.matches_name_pattern("test", &matcher));
        assert!(engine.matches_name_pattern("mytest", &matcher));
        assert!(!engine.matches_name_pattern("testing", &matcher));
    }

    #[test]
    fn test_predicate_matching() {
        let mut engine = PatternMatchingEngine::new();
        
        let predicate = PredicateFunction::BuiltIn(BuiltInPredicate::StartsWith("temp".to_string()));
        let matcher = PatternMatcher::Predicate(predicate);
        
        assert!(engine.matches_name_pattern("temp123", &matcher));
        assert!(engine.matches_name_pattern("temporary", &matcher));
        assert!(!engine.matches_name_pattern("test", &matcher));
    }

    #[test]
    fn test_pattern_builder() {
        let pattern = PatternBuilder::new()
            .glob("temp*".to_string())
            .in_macro("let".to_string())
            .build();
        
        assert!(pattern.is_some());
        match pattern.unwrap() {
            PatternMatcher::Glob(g) => assert_eq!(g, "temp*"),
            _ => panic!("Expected glob pattern"),
        }
    }

    #[test]
    fn test_cache_functionality() {
        let mut engine = PatternMatchingEngine::new();
        let pattern = "test*";
        
        // First call should be a cache miss
        engine.matches_glob("test123", pattern);
        assert_eq!(engine.stats.cache_misses, 1);
        assert_eq!(engine.stats.cache_hits, 0);
        
        // Second call should be a cache hit
        engine.matches_glob("test123", pattern);
        assert_eq!(engine.stats.cache_hits, 1);
    }
}
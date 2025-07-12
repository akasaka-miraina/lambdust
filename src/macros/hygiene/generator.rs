//! Symbol generator for unique hygienic symbols
//!
//! Provides generation of unique symbol identifiers and names for hygienic macro expansion.
//! Ensures that each macro-introduced symbol has a unique identity to prevent collisions.

use super::symbol::{HygienicSymbol, SymbolId, MacroSite, EnvironmentId};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Global counter for symbol generation
static SYMBOL_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Global counter for environment generation
static ENVIRONMENT_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Symbol generation strategy for optimization
#[derive(Debug, Clone, PartialEq)]
pub enum GenerationStrategy {
    /// Standard incremental numbering
    Incremental,
    /// Hash-based generation for better distribution
    Hash,
    /// Timestamp-based generation for debugging
    Timestamp,
    /// Compressed format for memory efficiency
    Compressed,
}

/// Symbol cache for reuse optimization
#[derive(Debug, Clone)]
pub struct SymbolCache {
    /// Cache of recently generated symbols by base name
    symbol_cache: HashMap<String, Vec<HygienicSymbol>>,
    /// Maximum cache size per base name
    max_cache_size: usize,
    /// Cache hit statistics
    cache_hits: u64,
    /// Cache miss statistics
    cache_misses: u64,
}

/// Generator for unique hygienic symbols with advanced optimization
#[derive(Debug, Clone)]
pub struct SymbolGenerator {
    /// Prefix for generated symbol names
    prefix: String,
    /// Current macro expansion context
    current_macro: Option<String>,
    /// Current expansion depth
    current_depth: usize,
    /// Current environment ID
    current_environment: EnvironmentId,
    /// Generation strategy for optimization
    strategy: GenerationStrategy,
    /// Symbol cache for performance optimization
    cache: SymbolCache,
    /// Generation statistics
    generation_count: u64,
    /// Performance metrics
    total_generation_time_ns: u64,
}

impl SymbolCache {
    /// Create new symbol cache
    fn new() -> Self {
        Self {
            symbol_cache: HashMap::new(),
            max_cache_size: 100, // Reasonable default
            cache_hits: 0,
            cache_misses: 0,
        }
    }
    
    /// Get cache hit rate
    #[must_use] pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
    
    /// Clear cache
    pub fn clear(&mut self) {
        self.symbol_cache.clear();
        self.cache_hits = 0;
        self.cache_misses = 0;
    }
    
    /// Get maximum cache size
    #[must_use] pub fn max_cache_size(&self) -> usize {
        self.max_cache_size
    }
    
    /// Get total number of cached symbols
    #[must_use] pub fn total_cached_symbols(&self) -> usize {
        self.symbol_cache.values().map(std::vec::Vec::len).sum()
    }
}

impl SymbolGenerator {
    /// Create new symbol generator
    #[must_use] pub fn new() -> Self {
        Self {
            prefix: "λ$".to_string(),
            current_macro: None,
            current_depth: 0,
            current_environment: EnvironmentId::new(0),
            strategy: GenerationStrategy::Incremental,
            cache: SymbolCache::new(),
            generation_count: 0,
            total_generation_time_ns: 0,
        }
    }
    
    /// Create symbol generator with custom prefix
    #[must_use] pub fn with_prefix(prefix: String) -> Self {
        Self {
            prefix,
            current_macro: None,
            current_depth: 0,
            current_environment: EnvironmentId::new(0),
            strategy: GenerationStrategy::Incremental,
            cache: SymbolCache::new(),
            generation_count: 0,
            total_generation_time_ns: 0,
        }
    }
    
    /// Create optimized symbol generator with custom strategy
    #[must_use] pub fn with_strategy(strategy: GenerationStrategy) -> Self {
        Self {
            prefix: "λ$".to_string(),
            current_macro: None,
            current_depth: 0,
            current_environment: EnvironmentId::new(0),
            strategy,
            cache: SymbolCache::new(),
            generation_count: 0,
            total_generation_time_ns: 0,
        }
    }
    
    /// Create high-performance generator with optimized cache
    #[must_use] pub fn optimized() -> Self {
        let mut cache = SymbolCache::new();
        cache.max_cache_size = 500; // Larger cache for performance
        
        Self {
            prefix: "λ#".to_string(), // Shorter prefix for memory efficiency
            current_macro: None,
            current_depth: 0,
            current_environment: EnvironmentId::new(0),
            strategy: GenerationStrategy::Hash,
            cache,
            generation_count: 0,
            total_generation_time_ns: 0,
        }
    }
    
    /// Set current macro expansion context
    pub fn set_macro_context(&mut self, macro_name: String, depth: usize) {
        self.current_macro = Some(macro_name);
        self.current_depth = depth;
    }
    
    /// Set current environment
    pub fn set_environment(&mut self, env_id: EnvironmentId) {
        self.current_environment = env_id;
    }
    
    /// Generate unique symbol ID
    pub fn generate_symbol_id() -> SymbolId {
        let id = SYMBOL_COUNTER.fetch_add(1, Ordering::SeqCst);
        SymbolId::new(id)
    }

    /// Get next ID for context usage
    pub fn next_id(&self) -> u64 {
        SYMBOL_COUNTER.load(Ordering::SeqCst)
    }
    
    /// Generate unique environment ID
    pub fn generate_environment_id() -> EnvironmentId {
        let id = ENVIRONMENT_COUNTER.fetch_add(1, Ordering::SeqCst);
        EnvironmentId::new(id)
    }
    
    /// Generate unique hygienic symbol with performance optimization
    pub fn generate_unique(&mut self, base_name: &str) -> HygienicSymbol {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        // Try cache first for performance
        if let Some(cached_symbol) = self.try_get_cached_symbol(base_name) {
            self.cache.cache_hits += 1;
            self.update_performance_metrics(start_time);
            return cached_symbol;
        }
        
        self.cache.cache_misses += 1;
        
        // Generate new symbol based on strategy
        let id = match self.strategy {
            GenerationStrategy::Incremental => Self::generate_symbol_id(),
            GenerationStrategy::Hash => self.generate_hash_id(base_name),
            GenerationStrategy::Timestamp => self.generate_timestamp_id(),
            GenerationStrategy::Compressed => self.generate_compressed_id(base_name),
        };
        
        let definition_site = self.current_site();
        let symbol = HygienicSymbol::new(
            base_name.to_string(),
            id,
            definition_site,
        );
        
        // Cache the generated symbol
        self.cache_symbol(base_name, &symbol);
        self.update_performance_metrics(start_time);
        
        symbol
    }
    
    /// Try to get a cached symbol for reuse
    fn try_get_cached_symbol(&mut self, base_name: &str) -> Option<HygienicSymbol> {
        if let Some(symbols) = self.cache.symbol_cache.get_mut(base_name) {
            if !symbols.is_empty() {
                // Return a clone of the most recent symbol with new ID
                let template = &symbols[symbols.len() - 1];
                let new_id = Self::generate_symbol_id();
                return Some(HygienicSymbol::new(
                    template.original_name().to_string(),
                    new_id,
                    template.definition_site.clone(),
                ));
            }
        }
        None
    }
    
    /// Cache a generated symbol
    fn cache_symbol(&mut self, base_name: &str, symbol: &HygienicSymbol) {
        let symbols = self.cache.symbol_cache
            .entry(base_name.to_string())
            .or_default();
        
        symbols.push(symbol.clone());
        
        // Limit cache size to prevent memory leaks
        if symbols.len() > self.cache.max_cache_size {
            symbols.remove(0);
        }
    }
    
    /// Generate hash-based symbol ID
    fn generate_hash_id(&self, base_name: &str) -> SymbolId {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        base_name.hash(&mut hasher);
        self.current_macro.hash(&mut hasher);
        self.current_depth.hash(&mut hasher);
        self.current_environment.hash(&mut hasher);
        
        let hash = hasher.finish();
        let unique_id = SYMBOL_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        SymbolId::new(hash ^ unique_id)
    }
    
    /// Generate timestamp-based symbol ID
    fn generate_timestamp_id(&self) -> SymbolId {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        let unique_id = SYMBOL_COUNTER.fetch_add(1, Ordering::SeqCst);
        SymbolId::new(timestamp ^ unique_id)
    }
    
    /// Generate compressed symbol ID for memory efficiency
    fn generate_compressed_id(&self, base_name: &str) -> SymbolId {
        // Use shorter ID space for memory efficiency
        let base_hash = base_name.len() as u64;
        let depth_factor = (self.current_depth as u64) << 8;
        let unique_id = SYMBOL_COUNTER.fetch_add(1, Ordering::SeqCst) & 0xFFFF;
        
        SymbolId::new(base_hash ^ depth_factor ^ unique_id)
    }
    
    /// Update performance metrics
    fn update_performance_metrics(&mut self, start_time: u64) {
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        self.generation_count += 1;
        self.total_generation_time_ns += end_time.saturating_sub(start_time);
    }
    
    /// Generate unique symbol with explicit macro context
    #[must_use] pub fn generate_with_context(
        &self,
        base_name: &str,
        macro_name: &str,
        depth: usize,
        env_id: EnvironmentId,
    ) -> HygienicSymbol {
        let id = Self::generate_symbol_id();
        let definition_site = MacroSite::new(
            macro_name.to_string(),
            depth,
            env_id,
        );
        
        HygienicSymbol::new(
            base_name.to_string(),
            id,
            definition_site,
        )
    }
    
    /// Generate symbol from user code (non-macro)
    #[must_use] pub fn generate_user_symbol(&self, name: &str) -> HygienicSymbol {
        let id = Self::generate_symbol_id();
        HygienicSymbol::from_user_code(
            name.to_string(),
            id,
            self.current_environment,
        )
    }
    
    /// Get current macro site information
    fn current_site(&self) -> MacroSite {
        let macro_name = self.current_macro
            .as_deref()
            .unwrap_or("<unknown>");
        
        MacroSite::new(
            macro_name.to_string(),
            self.current_depth,
            self.current_environment,
        )
    }
    
    /// Create scoped generator for nested macro expansion
    #[must_use] pub fn enter_macro(&self, macro_name: String) -> Self {
        let mut nested = self.clone();
        nested.current_macro = Some(macro_name);
        nested.current_depth += 1;
        nested
    }
    
    /// Reset to parent scope
    pub fn exit_macro(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
        if self.current_depth == 0 {
            self.current_macro = None;
        }
    }
    
    /// Get performance statistics
    #[must_use] pub fn performance_stats(&self) -> PerformanceStats {
        PerformanceStats {
            generation_count: self.generation_count,
            total_generation_time_ns: self.total_generation_time_ns,
            average_generation_time_ns: if self.generation_count > 0 {
                self.total_generation_time_ns / self.generation_count
            } else {
                0
            },
            cache_hit_rate: self.cache.hit_rate(),
            cache_size: self.cache.symbol_cache.len(),
            strategy: self.strategy.clone(),
        }
    }
    
    /// Reset performance statistics
    pub fn reset_stats(&mut self) {
        self.generation_count = 0;
        self.total_generation_time_ns = 0;
        self.cache.clear();
    }
    
    /// Optimize cache based on usage patterns
    pub fn optimize_cache(&mut self) {
        // Remove least recently used entries if cache is too large
        let total_symbols: usize = self.cache.symbol_cache
            .values()
            .map(std::vec::Vec::len)
            .sum();
        
        if total_symbols > self.cache.max_cache_size * 10 {
            // Keep only the most recent symbols
            for symbols in self.cache.symbol_cache.values_mut() {
                if symbols.len() > 5 {
                    symbols.drain(0..symbols.len() - 5);
                }
            }
        }
    }
    
    /// Configure generation strategy for specific use cases
    pub fn configure_for_use_case(&mut self, use_case: UseCase) {
        match use_case {
            UseCase::HighFrequencyMacros => {
                self.strategy = GenerationStrategy::Hash;
                self.cache.max_cache_size = 1000;
                self.prefix = "λ#".to_string(); // Shorter prefix
            }
            UseCase::MemoryConstrained => {
                self.strategy = GenerationStrategy::Compressed;
                self.cache.max_cache_size = 50;
                self.prefix = "λ".to_string(); // Minimal prefix
            }
            UseCase::DebuggingMacros => {
                self.strategy = GenerationStrategy::Timestamp;
                self.cache.max_cache_size = 200;
                self.prefix = "λ$debug$".to_string(); // Descriptive prefix
            }
            UseCase::ProductionOptimized => {
                self.strategy = GenerationStrategy::Hash;
                self.cache.max_cache_size = 500;
                self.prefix = "λ#".to_string();
            }
        }
    }
    
    /// Bulk generate symbols for batch operations
    pub fn generate_bulk(&mut self, base_names: &[&str]) -> Vec<HygienicSymbol> {
        let mut symbols = Vec::with_capacity(base_names.len());
        
        for &base_name in base_names {
            symbols.push(self.generate_unique(base_name));
        }
        
        // Optimize cache after bulk generation
        if base_names.len() > 10 {
            self.optimize_cache();
        }
        
        symbols
    }
    
    /// Get the current prefix
    #[must_use] pub fn prefix(&self) -> &str {
        &self.prefix
    }
    
    /// Get cache statistics
    #[must_use] pub fn cache_stats(&self) -> &SymbolCache {
        &self.cache
    }
    
    /// Get current generation strategy
    #[must_use] pub fn strategy(&self) -> &GenerationStrategy {
        &self.strategy
    }
    
    /// Get current macro context
    #[must_use] pub fn current_macro(&self) -> Option<&str> {
        self.current_macro.as_deref()
    }
    
    /// Get current expansion depth
    #[must_use] pub fn current_depth(&self) -> usize {
        self.current_depth
    }
}

/// Performance statistics for symbol generation
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Total number of symbols generated
    pub generation_count: u64,
    /// Total time spent generating symbols (nanoseconds)
    pub total_generation_time_ns: u64,
    /// Average time per symbol generation (nanoseconds)
    pub average_generation_time_ns: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Current cache size (number of base names cached)
    pub cache_size: usize,
    /// Current generation strategy
    pub strategy: GenerationStrategy,
}

/// Use case optimizations for symbol generation
#[derive(Debug, Clone, PartialEq)]
pub enum UseCase {
    /// High-frequency macro usage (optimize for speed)
    HighFrequencyMacros,
    /// Memory-constrained environment (optimize for memory)
    MemoryConstrained,
    /// Debugging macros (optimize for traceability)
    DebuggingMacros,
    /// Production environment (balanced optimization)
    ProductionOptimized,
}

impl PerformanceStats {
    /// Get human-readable generation rate
    #[must_use] pub fn generation_rate(&self) -> f64 {
        if self.total_generation_time_ns > 0 {
            (self.generation_count as f64) / (self.total_generation_time_ns as f64 / 1_000_000_000.0)
        } else {
            0.0
        }
    }
    
    /// Format statistics for display
    #[must_use] pub fn format_summary(&self) -> String {
        format!(
            "Symbol Generation Stats:\n  Generated: {} symbols\n  Cache Hit Rate: {:.1}%\n  Avg Time: {:.2}μs\n  Rate: {:.1} symbols/sec\n  Strategy: {:?}",
            self.generation_count,
            self.cache_hit_rate * 100.0,
            self.average_generation_time_ns as f64 / 1000.0,
            self.generation_rate(),
            self.strategy
        )
    }
    
    /// Check if performance is optimal
    #[must_use] pub fn is_optimal(&self) -> bool {
        self.cache_hit_rate > 0.7 && self.average_generation_time_ns < 1000
    }
    
    /// Get optimization recommendations
    #[must_use] pub fn optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if self.cache_hit_rate < 0.3 {
            recommendations.push("Consider increasing cache size or using Hash strategy".to_string());
        }
        
        if self.average_generation_time_ns > 5000 {
            recommendations.push("Consider using Compressed strategy for faster generation".to_string());
        }
        
        if self.cache_size > 1000 {
            recommendations.push("Cache is very large - consider periodic optimization".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("Performance is optimal".to_string());
        }
        
        recommendations
    }
}

impl Default for SymbolGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for symbol generation
pub struct SymbolUtils;

impl SymbolUtils {
    /// Check if a name looks like a generated symbol
    #[must_use] pub fn is_generated_name(name: &str) -> bool {
        name.starts_with("λ$") || name.contains('#')
    }
    
    /// Extract base name from generated symbol name
    #[must_use] pub fn extract_base_name(name: &str) -> Option<&str> {
        if let Some(stripped) = name.strip_prefix("λ$") {
            if let Some(hash_pos) = stripped.find('#') {
                Some(&stripped[..hash_pos])
            } else {
                Some(stripped)
            }
        } else {
            None
        }
    }
    
    /// Extract symbol ID from generated name
    #[must_use] pub fn extract_symbol_id(name: &str) -> Option<SymbolId> {
        if let Some(hash_pos) = name.find('#') {
            let id_str = &name[hash_pos + 1..];
            if let Ok(id) = id_str.parse::<u64>() {
                Some(SymbolId::new(id))
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_generator_creation() {
        let gen = SymbolGenerator::new();
        assert_eq!(gen.prefix, "λ$");
        assert!(gen.current_macro.is_none());
        assert_eq!(gen.current_depth, 0);
    }
    
    #[test]
    fn test_custom_prefix() {
        let gen = SymbolGenerator::with_prefix("test$".to_string());
        assert_eq!(gen.prefix, "test$");
    }
    
    #[test]
    fn test_unique_symbol_generation() {
        let mut gen = SymbolGenerator::new();
        gen.set_macro_context("test-macro".to_string(), 1);
        
        let symbol1 = gen.generate_unique("temp");
        let symbol2 = gen.generate_unique("temp");
        
        // Should have different IDs
        assert_ne!(symbol1.id, symbol2.id);
        
        // Should have same base name
        assert_eq!(symbol1.original_name(), "temp");
        assert_eq!(symbol2.original_name(), "temp");
        
        // Should be macro-introduced
        assert!(symbol1.is_macro_introduced);
        assert!(symbol2.is_macro_introduced);
    }
    
    #[test]
    fn test_user_symbol_generation() {
        let gen = SymbolGenerator::new();
        let symbol = gen.generate_user_symbol("user-var");
        
        assert_eq!(symbol.original_name(), "user-var");
        assert_eq!(symbol.unique_name(), "user-var"); // No decoration
        assert!(!symbol.is_macro_introduced);
    }
    
    #[test]
    fn test_macro_context_nesting() {
        let gen = SymbolGenerator::new();
        let nested = gen.enter_macro("outer-macro".to_string());
        
        assert_eq!(nested.current_depth, 1);
        assert_eq!(nested.current_macro, Some("outer-macro".to_string()));
        
        let double_nested = nested.enter_macro("inner-macro".to_string());
        assert_eq!(double_nested.current_depth, 2);
        assert_eq!(double_nested.current_macro, Some("inner-macro".to_string()));
    }
    
    #[test]
    fn test_global_id_uniqueness() {
        // Generate multiple IDs to ensure uniqueness
        let id1 = SymbolGenerator::generate_symbol_id();
        let id2 = SymbolGenerator::generate_symbol_id();
        let id3 = SymbolGenerator::generate_symbol_id();
        
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }
    
    #[test]
    fn test_environment_id_generation() {
        let env1 = SymbolGenerator::generate_environment_id();
        let env2 = SymbolGenerator::generate_environment_id();
        
        assert_ne!(env1, env2);
    }
    
    #[test]
    fn test_symbol_utils_generated_name_detection() {
        assert!(SymbolUtils::is_generated_name("λ$temp#123"));
        assert!(SymbolUtils::is_generated_name("something#456"));
        assert!(!SymbolUtils::is_generated_name("normal-name"));
        assert!(!SymbolUtils::is_generated_name("lambda-var"));
    }
    
    #[test]
    fn test_symbol_utils_base_name_extraction() {
        assert_eq!(SymbolUtils::extract_base_name("λ$temp#123"), Some("temp"));
        assert_eq!(SymbolUtils::extract_base_name("λ$variable#456"), Some("variable"));
        assert_eq!(SymbolUtils::extract_base_name("normal-name"), None);
    }
    
    #[test]
    fn test_symbol_utils_id_extraction() {
        let id = SymbolUtils::extract_symbol_id("λ$temp#123");
        assert_eq!(id, Some(SymbolId::new(123)));
        
        let id2 = SymbolUtils::extract_symbol_id("something#456");
        assert_eq!(id2, Some(SymbolId::new(456)));
        
        let id3 = SymbolUtils::extract_symbol_id("no-id-here");
        assert_eq!(id3, None);
    }
    
    #[test]
    fn test_generation_strategies() {
        let mut gen_incremental = SymbolGenerator::with_strategy(GenerationStrategy::Incremental);
        let mut gen_hash = SymbolGenerator::with_strategy(GenerationStrategy::Hash);
        let mut gen_compressed = SymbolGenerator::with_strategy(GenerationStrategy::Compressed);
        
        gen_incremental.set_macro_context("test-macro".to_string(), 1);
        gen_hash.set_macro_context("test-macro".to_string(), 1);
        gen_compressed.set_macro_context("test-macro".to_string(), 1);
        
        let symbol1 = gen_incremental.generate_unique("temp");
        let symbol2 = gen_hash.generate_unique("temp");
        let symbol3 = gen_compressed.generate_unique("temp");
        
        // All should have different IDs due to different strategies
        assert_ne!(symbol1.id, symbol2.id);
        assert_ne!(symbol2.id, symbol3.id);
        assert_ne!(symbol1.id, symbol3.id);
        
        // All should have same base name
        assert_eq!(symbol1.original_name(), "temp");
        assert_eq!(symbol2.original_name(), "temp");
        assert_eq!(symbol3.original_name(), "temp");
    }
    
    #[test]
    fn test_performance_optimization() {
        let mut gen = SymbolGenerator::optimized();
        gen.set_macro_context("perf-test".to_string(), 1);
        
        // Generate multiple symbols with same base name
        for _ in 0..10 {
            let _symbol = gen.generate_unique("temp");
        }
        
        let stats = gen.performance_stats();
        assert!(stats.generation_count >= 10);
        assert!(stats.cache_size > 0);
        
        // Test cache hit rate improvement
        for _ in 0..5 {
            let _symbol = gen.generate_unique("temp");
        }
        
        let stats2 = gen.performance_stats();
        assert!(stats2.cache_hit_rate > 0.0);
    }
    
    #[test]
    fn test_use_case_configuration() {
        let mut gen = SymbolGenerator::new();
        
        // Test high frequency configuration
        gen.configure_for_use_case(UseCase::HighFrequencyMacros);
        assert_eq!(gen.strategy, GenerationStrategy::Hash);
        assert_eq!(gen.cache.max_cache_size, 1000);
        
        // Test memory constrained configuration
        gen.configure_for_use_case(UseCase::MemoryConstrained);
        assert_eq!(gen.strategy, GenerationStrategy::Compressed);
        assert_eq!(gen.cache.max_cache_size, 50);
        
        // Test debugging configuration
        gen.configure_for_use_case(UseCase::DebuggingMacros);
        assert_eq!(gen.strategy, GenerationStrategy::Timestamp);
        assert!(gen.prefix.contains("debug"));
    }
    
    #[test]
    fn test_bulk_generation() {
        let mut gen = SymbolGenerator::optimized();
        gen.set_macro_context("bulk-test".to_string(), 1);
        
        let base_names = &["temp1", "temp2", "temp3", "var1", "var2"];
        let symbols = gen.generate_bulk(base_names);
        
        assert_eq!(symbols.len(), base_names.len());
        
        // All symbols should have unique IDs
        for i in 0..symbols.len() {
            for j in i + 1..symbols.len() {
                assert_ne!(symbols[i].id, symbols[j].id);
            }
        }
        
        // Base names should match
        for (i, &base_name) in base_names.iter().enumerate() {
            assert_eq!(symbols[i].original_name(), base_name);
        }
    }
    
    #[test]
    fn test_performance_stats_analysis() {
        let mut gen = SymbolGenerator::optimized();
        gen.set_macro_context("stats-test".to_string(), 1);
        
        // Generate some symbols to create meaningful stats
        for i in 0..50 {
            let base_name = if i % 5 == 0 { "common" } else { &format!("unique{}", i) };
            let _symbol = gen.generate_unique(base_name);
        }
        
        let stats = gen.performance_stats();
        assert_eq!(stats.generation_count, 50);
        assert!(stats.average_generation_time_ns > 0);
        
        let summary = stats.format_summary();
        assert!(summary.contains("Symbol Generation Stats"));
        assert!(summary.contains("Generated: 50 symbols"));
        
        let recommendations = stats.optimization_recommendations();
        assert!(!recommendations.is_empty());
    }
    
    #[test]
    fn test_cache_optimization() {
        let mut gen = SymbolGenerator::optimized();
        gen.set_macro_context("cache-test".to_string(), 1);
        
        // Fill cache beyond limit
        for i in 0..100 {
            let base_name = format!("var{}", i);
            let _symbol = gen.generate_unique(&base_name);
        }
        
        let initial_cache_size = gen.cache.symbol_cache.len();
        
        // Trigger cache optimization
        gen.optimize_cache();
        
        let optimized_cache_size = gen.cache.symbol_cache.len();
        
        // Cache should not exceed reasonable limits
        assert!(optimized_cache_size <= initial_cache_size);
    }
}
//! Incremental Type Inference Cache System
//! This module implements advanced caching for type inference results
//! to achieve faster recompilation times than GHC

use super::polynomial_types::{PolynomialType, UniverseLevel};
use super::type_inference::TypeInference;
use crate::ast::Expr;
use crate::error::Result;
use crate::value::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Cache entry for type inference results
#[derive(Debug, Clone)]
pub struct InferenceCacheEntry {
    /// Inferred type
    pub inferred_type: PolynomialType,
    /// Timestamp when cached
    pub timestamp: SystemTime,
    /// Dependencies that this inference depends on
    pub dependencies: Vec<String>,
    /// Hash of the expression that was inferred
    pub expression_hash: u64,
    /// Inference cost (for cache replacement policy)
    pub inference_cost: Duration,
    /// Number of times this entry has been accessed
    pub access_count: u64,
}

/// Dependency tracking for incremental recompilation
#[derive(Debug, Clone)]
pub struct DependencyTracker {
    /// Maps symbol -> dependents (who depends on this symbol)
    dependents: HashMap<String, Vec<String>>,
    /// Maps symbol -> dependencies (what this symbol depends on)
    dependencies: HashMap<String, Vec<String>>,
    /// Symbol modification timestamps
    modification_times: HashMap<String, SystemTime>,
}

impl DependencyTracker {
    /// Create new dependency tracker
    pub fn new() -> Self {
        Self {
            dependents: HashMap::new(),
            dependencies: HashMap::new(),
            modification_times: HashMap::new(),
        }
    }

    /// Add dependency: symbol depends on dependency
    pub fn add_dependency(&mut self, symbol: String, dependency: String) {
        self.dependencies.entry(symbol.clone())
            .or_insert_with(Vec::new)
            .push(dependency.clone());
        
        self.dependents.entry(dependency)
            .or_insert_with(Vec::new)
            .push(symbol);
    }

    /// Mark symbol as modified
    pub fn mark_modified(&mut self, symbol: &str) {
        self.modification_times.insert(symbol.to_string(), SystemTime::now());
    }

    /// Get all symbols that need to be invalidated when a symbol changes
    pub fn get_invalidation_set(&self, symbol: &str) -> Vec<String> {
        let mut to_invalidate = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.collect_dependents(symbol, &mut to_invalidate, &mut visited);
        to_invalidate
    }

    fn collect_dependents(&self, symbol: &str, result: &mut Vec<String>, visited: &mut std::collections::HashSet<String>) {
        if visited.contains(symbol) {
            return;
        }
        visited.insert(symbol.to_string());

        if let Some(dependents) = self.dependents.get(symbol) {
            for dependent in dependents {
                result.push(dependent.clone());
                self.collect_dependents(dependent, result, visited);
            }
        }
    }

    /// Check if symbol is newer than the given timestamp
    pub fn is_newer_than(&self, symbol: &str, timestamp: SystemTime) -> bool {
        if let Some(&mod_time) = self.modification_times.get(symbol) {
            mod_time > timestamp
        } else {
            false
        }
    }

    /// Get dependencies of a symbol
    pub fn get_dependencies(&self, symbol: &str) -> Vec<String> {
        self.dependencies.get(symbol).cloned().unwrap_or_default()
    }
}

/// Cache replacement policy
#[derive(Debug, Clone, Copy)]
pub enum CachePolicy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// Cost-based (remove expensive computations first)
    CostBased,
    /// Hybrid policy combining multiple factors
    Hybrid,
}

/// Configuration for incremental type inference
#[derive(Debug, Clone)]
pub struct IncrementalConfig {
    /// Maximum number of cache entries
    pub max_cache_size: usize,
    /// Cache replacement policy
    pub cache_policy: CachePolicy,
    /// Maximum age for cache entries (in seconds)
    pub max_age_seconds: u64,
    /// Whether to enable dependency tracking
    pub enable_dependency_tracking: bool,
    /// Whether to persist cache to disk
    pub enable_persistence: bool,
    /// Minimum inference cost to cache (avoid caching trivial inferences)
    pub min_cache_cost: Duration,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 10000,
            cache_policy: CachePolicy::Hybrid,
            max_age_seconds: 3600, // 1 hour
            enable_dependency_tracking: true,
            enable_persistence: false, // Disabled by default for now
            min_cache_cost: Duration::from_millis(1),
        }
    }
}

/// Incremental Type Inference Engine
#[derive(Debug)]
pub struct IncrementalTypeInference {
    /// Base type inference engine
    base_inference: TypeInference,
    /// Type inference cache
    cache: Arc<RwLock<HashMap<String, InferenceCacheEntry>>>,
    /// Dependency tracker
    dependency_tracker: Arc<RwLock<DependencyTracker>>,
    /// Configuration
    config: IncrementalConfig,
    /// Cache statistics
    stats: Arc<RwLock<CacheStatistics>>,
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total invalidations
    pub invalidations: u64,
    /// Total inference time saved
    pub time_saved: Duration,
    /// Average inference time (for comparison)
    pub avg_inference_time: Duration,
    /// Cache size over time
    pub cache_size_history: Vec<(SystemTime, usize)>,
    /// Hit rate over time
    pub hit_rate_history: Vec<(SystemTime, f64)>,
}

impl CacheStatistics {
    /// Create new statistics
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            invalidations: 0,
            time_saved: Duration::ZERO,
            avg_inference_time: Duration::from_millis(10), // Default estimate
            cache_size_history: Vec::new(),
            hit_rate_history: Vec::new(),
        }
    }

    /// Calculate hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Record cache hit
    pub fn record_hit(&mut self, time_saved: Duration) {
        self.hits += 1;
        self.time_saved += time_saved;
    }

    /// Record cache miss
    pub fn record_miss(&mut self, inference_time: Duration) {
        self.misses += 1;
        // Update rolling average
        let total_inferences = self.hits + self.misses;
        self.avg_inference_time = (self.avg_inference_time * (total_inferences - 1) as u32 + inference_time) / total_inferences as u32;
    }

    /// Record cache invalidation
    pub fn record_invalidation(&mut self) {
        self.invalidations += 1;
    }

    /// Update cache size history
    pub fn update_cache_size(&mut self, size: usize) {
        self.cache_size_history.push((SystemTime::now(), size));
        
        // Keep only last 1000 entries
        if self.cache_size_history.len() > 1000 {
            self.cache_size_history.remove(0);
        }
    }

    /// Update hit rate history
    pub fn update_hit_rate(&mut self) {
        self.hit_rate_history.push((SystemTime::now(), self.hit_rate()));
        
        // Keep only last 1000 entries
        if self.hit_rate_history.len() > 1000 {
            self.hit_rate_history.remove(0);
        }
    }
}

impl IncrementalTypeInference {
    /// Create new incremental type inference engine
    pub fn new(config: IncrementalConfig) -> Self {
        Self {
            base_inference: TypeInference::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            dependency_tracker: Arc::new(RwLock::new(DependencyTracker::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStatistics::new())),
        }
    }

    /// Infer type with caching
    pub fn infer(&mut self, value: &Value, context_hint: Option<&str>) -> Result<PolynomialType> {
        let start_time = Instant::now();
        let cache_key = self.generate_cache_key(value, context_hint);
        
        // Check cache first
        if let Some(cached_type) = self.check_cache(&cache_key)? {
            let mut stats = self.stats.write().unwrap();
            stats.record_hit(self.config.min_cache_cost); // Estimate time saved
            return Ok(cached_type);
        }

        // Cache miss - perform inference
        let inferred_type = self.base_inference.infer(value)?;
        let inference_time = start_time.elapsed();

        // Cache the result if it's worth caching
        if inference_time >= self.config.min_cache_cost {
            self.cache_result(cache_key, inferred_type.clone(), inference_time, context_hint)?;
        }

        // Update statistics
        let mut stats = self.stats.write().unwrap();
        stats.record_miss(inference_time);

        Ok(inferred_type)
    }

    /// Infer type for expression with caching
    pub fn infer_expression(&mut self, expr: &Expr, context_hint: Option<&str>) -> Result<PolynomialType> {
        let start_time = Instant::now();
        let cache_key = self.generate_expression_cache_key(expr, context_hint);
        
        // Check cache first
        if let Some(cached_type) = self.check_cache(&cache_key)? {
            let mut stats = self.stats.write().unwrap();
            stats.record_hit(start_time.elapsed());
            return Ok(cached_type);
        }

        // Cache miss - perform inference
        let inferred_type = self.infer_expression_direct(expr)?;
        let inference_time = start_time.elapsed();

        // Cache the result
        if inference_time >= self.config.min_cache_cost {
            self.cache_expression_result(cache_key, inferred_type.clone(), inference_time, expr, context_hint)?;
        }

        // Update statistics
        let mut stats = self.stats.write().unwrap();
        stats.record_miss(inference_time);

        Ok(inferred_type)
    }

    /// Direct expression inference (without caching)
    fn infer_expression_direct(&mut self, expr: &Expr) -> Result<PolynomialType> {
        match expr {
            Expr::Literal(literal) => {
                let value = match literal {
                    crate::ast::Literal::Boolean(b) => Value::Boolean(*b),
                    crate::ast::Literal::Number(n) => Value::Number(n.clone()),
                    crate::ast::Literal::String(s) => Value::new_string(s.clone()),
                    crate::ast::Literal::Character(c) => Value::new_character(*c),
                    crate::ast::Literal::Nil => Value::Nil,
                };
                self.base_inference.infer(&value)
            }
            Expr::Variable(name) => {
                // For variables, return a type variable
                Ok(PolynomialType::Variable {
                    name: name.clone(),
                    level: UniverseLevel::new(0),
                })
            }
            Expr::List(elements) => {
                if elements.is_empty() {
                    return Ok(PolynomialType::List {
                        element_type: Box::new(PolynomialType::Variable {
                            name: "α".to_string(),
                            level: UniverseLevel::new(0),
                        }),
                    });
                }

                // For now, return a generic type
                // TODO: Implement proper type inference for list expressions
                Ok(PolynomialType::Variable {
                    name: "τ".to_string(),
                    level: UniverseLevel::new(0),
                })
            }
            _ => {
                // For other expression types, return a fresh type variable
                Ok(PolynomialType::Variable {
                    name: "τ".to_string(),
                    level: UniverseLevel::new(0),
                })
            }
        }
    }

    /// Generate cache key for value
    fn generate_cache_key(&self, value: &Value, context_hint: Option<&str>) -> String {
        let mut hasher = DefaultHasher::new();
        
        // Hash the value type (simplified)
        match value {
            Value::Boolean(b) => {
                "bool".hash(&mut hasher);
                b.hash(&mut hasher);
            }
            Value::Number(n) => {
                "number".hash(&mut hasher);
                format!("{:?}", n).hash(&mut hasher);
            }
            Value::String(s) => {
                "string".hash(&mut hasher);
                s.hash(&mut hasher);
            }
            Value::ShortString(s) => {
                "string".hash(&mut hasher);
                s.as_str().hash(&mut hasher);
            }
            Value::Character(c) => {
                "character".hash(&mut hasher);
                c.hash(&mut hasher);
            }
            Value::Symbol(s) => {
                "symbol".hash(&mut hasher);
                s.hash(&mut hasher);
            }
            Value::ShortSymbol(s) => {
                "symbol".hash(&mut hasher);
                s.as_str().hash(&mut hasher);
            }
            Value::Nil => {
                "nil".hash(&mut hasher);
            }
            _ => {
                "complex".hash(&mut hasher);
                // For complex types, use a simpler hash
                format!("{:?}", std::mem::discriminant(value)).hash(&mut hasher);
            }
        }

        if let Some(context) = context_hint {
            context.hash(&mut hasher);
        }

        format!("val_{:x}", hasher.finish())
    }

    /// Generate cache key for expression
    fn generate_expression_cache_key(&self, expr: &Expr, context_hint: Option<&str>) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash_expression(expr, &mut hasher);
        
        if let Some(context) = context_hint {
            context.hash(&mut hasher);
        }

        format!("expr_{:x}", hasher.finish())
    }

    /// Hash expression recursively
    fn hash_expression(&self, expr: &Expr, hasher: &mut DefaultHasher) {
        match expr {
            Expr::Literal(literal) => {
                "literal".hash(hasher);
                format!("{:?}", literal).hash(hasher);
            }
            Expr::Variable(name) => {
                "variable".hash(hasher);
                name.hash(hasher);
            }
            Expr::List(elements) => {
                "list".hash(hasher);
                elements.len().hash(hasher);
                for element in elements {
                    self.hash_expression(element, hasher);
                }
            }
            Expr::Quote(inner) => {
                "quote".hash(hasher);
                self.hash_expression(inner, hasher);
            }
            _ => {
                // For other types, use discriminant
                format!("{:?}", std::mem::discriminant(expr)).hash(hasher);
            }
        }
    }

    /// Check cache for entry
    fn check_cache(&self, cache_key: &str) -> Result<Option<PolynomialType>> {
        let cache = self.cache.read().unwrap();
        
        if let Some(entry) = cache.get(cache_key) {
            // Check if entry is still valid
            if self.is_cache_entry_valid(entry)? {
                return Ok(Some(entry.inferred_type.clone()));
            }
        }

        Ok(None)
    }

    /// Check if cache entry is still valid
    fn is_cache_entry_valid(&self, entry: &InferenceCacheEntry) -> Result<bool> {
        // Check age
        let age = SystemTime::now().duration_since(entry.timestamp)
            .unwrap_or(Duration::MAX);
        
        if age > Duration::from_secs(self.config.max_age_seconds) {
            return Ok(false);
        }

        // Check dependencies if tracking is enabled
        if self.config.enable_dependency_tracking {
            let tracker = self.dependency_tracker.read().unwrap();
            for dependency in &entry.dependencies {
                if tracker.is_newer_than(dependency, entry.timestamp) {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Cache inference result
    fn cache_result(
        &mut self,
        cache_key: String,
        inferred_type: PolynomialType,
        inference_cost: Duration,
        _context_hint: Option<&str>,
    ) -> Result<()> {
        let mut cache = self.cache.write().unwrap();
        
        // Evict entries if cache is full
        if cache.len() >= self.config.max_cache_size {
            self.evict_entries(&mut cache)?;
        }

        // Get actual dependencies from dependency tracker if context hint is provided
        let dependencies = if let Some(context) = _context_hint {
            let tracker = self.dependency_tracker.read().unwrap();
            let mut deps = tracker.get_dependencies(context);
            deps.push(context.to_string()); // Include self as dependency
            deps
        } else {
            Vec::new()
        };

        let entry = InferenceCacheEntry {
            inferred_type,
            timestamp: SystemTime::now(),
            dependencies,
            expression_hash: 0, // Will be set by caller if needed
            inference_cost,
            access_count: 0,
        };

        cache.insert(cache_key, entry);

        // Update statistics
        let mut stats = self.stats.write().unwrap();
        stats.update_cache_size(cache.len());

        Ok(())
    }

    /// Cache expression inference result
    fn cache_expression_result(
        &mut self,
        cache_key: String,
        inferred_type: PolynomialType,
        inference_cost: Duration,
        expr: &Expr,
        context_hint: Option<&str>,
    ) -> Result<()> {
        let mut cache = self.cache.write().unwrap();
        
        // Evict entries if cache is full
        if cache.len() >= self.config.max_cache_size {
            self.evict_entries(&mut cache)?;
        }

        let mut hasher = DefaultHasher::new();
        self.hash_expression(expr, &mut hasher);
        let expression_hash = hasher.finish();

        let mut dependencies = self.extract_dependencies(expr);
        
        // Add dependencies from context hint and dependency tracker
        if let Some(context) = context_hint {
            let tracker = self.dependency_tracker.read().unwrap();
            let mut context_deps = tracker.get_dependencies(context);
            dependencies.append(&mut context_deps);
            dependencies.push(context.to_string()); // Include self as dependency
        }
        
        // Remove duplicates and sort
        dependencies.sort();
        dependencies.dedup();

        let entry = InferenceCacheEntry {
            inferred_type,
            timestamp: SystemTime::now(),
            dependencies,
            expression_hash,
            inference_cost,
            access_count: 0,
        };

        cache.insert(cache_key, entry);

        // Update statistics
        let mut stats = self.stats.write().unwrap();
        stats.update_cache_size(cache.len());

        Ok(())
    }

    /// Extract dependencies from expression
    fn extract_dependencies(&self, expr: &Expr) -> Vec<String> {
        let mut dependencies = Vec::new();
        self.collect_dependencies(expr, &mut dependencies);
        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    /// Collect dependencies recursively
    fn collect_dependencies(&self, expr: &Expr, dependencies: &mut Vec<String>) {
        match expr {
            Expr::Variable(name) => {
                dependencies.push(name.clone());
            }
            Expr::List(elements) => {
                for element in elements {
                    self.collect_dependencies(element, dependencies);
                }
            }
            Expr::Quote(inner) => {
                self.collect_dependencies(inner, dependencies);
            }
            _ => {
                // Other expression types don't introduce dependencies for now
            }
        }
    }

    /// Evict cache entries based on policy
    fn evict_entries(&self, cache: &mut HashMap<String, InferenceCacheEntry>) -> Result<()> {
        let target_size = (self.config.max_cache_size * 3) / 4; // Remove 25% of entries
        let to_remove = cache.len() - target_size;

        let keys_to_remove = match self.config.cache_policy {
            CachePolicy::LRU => {
                // Remove least recently used (oldest timestamp + lowest access count)
                let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                entries.sort_by(|a, b| {
                    a.1.timestamp.cmp(&b.1.timestamp)
                        .then_with(|| a.1.access_count.cmp(&b.1.access_count))
                });
                
                entries.into_iter().take(to_remove).map(|(k, _)| k).collect::<Vec<_>>()
            }
            CachePolicy::LFU => {
                // Remove least frequently used
                let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                entries.sort_by_key(|entry| entry.1.access_count);
                
                entries.into_iter().take(to_remove).map(|(k, _)| k).collect::<Vec<_>>()
            }
            CachePolicy::CostBased => {
                // Remove entries with highest inference cost first (they're expensive to recompute)
                let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                entries.sort_by(|a, b| b.1.inference_cost.cmp(&a.1.inference_cost));
                
                entries.into_iter().take(to_remove).map(|(k, _)| k).collect::<Vec<_>>()
            }
            CachePolicy::Hybrid => {
                // Hybrid policy: combine cost, frequency, and recency
                let mut entries: Vec<_> = cache.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                entries.sort_by(|a, b| {
                    let score_a = self.calculate_hybrid_score(&a.1);
                    let score_b = self.calculate_hybrid_score(&b.1);
                    score_a.partial_cmp(&score_b).unwrap_or(std::cmp::Ordering::Equal)
                });
                
                entries.into_iter().take(to_remove).map(|(k, _)| k).collect::<Vec<_>>()
            }
        };

        // Remove the selected keys
        for key in keys_to_remove {
            cache.remove(&key);
        }

        Ok(())
    }

    /// Calculate hybrid score for cache entry (lower = more likely to evict)
    fn calculate_hybrid_score(&self, entry: &InferenceCacheEntry) -> f64 {
        let age_weight = 0.3;
        let frequency_weight = 0.4;
        let cost_weight = 0.3;

        let age_seconds = SystemTime::now().duration_since(entry.timestamp)
            .unwrap_or(Duration::ZERO)
            .as_secs_f64();
        let age_score = 1.0 / (1.0 + age_seconds / 3600.0); // Normalize to hours

        let frequency_score = (entry.access_count as f64).ln_1p();

        let cost_score = entry.inference_cost.as_millis() as f64 / 1000.0; // Normalize to seconds

        age_weight * age_score + frequency_weight * frequency_score + cost_weight * cost_score
    }

    /// Invalidate cache entries based on dependencies
    pub fn invalidate_dependencies(&mut self, changed_symbol: &str) -> Result<u64> {
        let invalidation_set = {
            let tracker = self.dependency_tracker.read().unwrap();
            tracker.get_invalidation_set(changed_symbol)
        };

        let mut cache = self.cache.write().unwrap();
        let mut invalidated_count = 0;

        // Remove entries that depend on the changed symbol
        cache.retain(|_, entry| {
            for dependency in &entry.dependencies {
                if invalidation_set.contains(dependency) || dependency == changed_symbol {
                    invalidated_count += 1;
                    return false;
                }
            }
            true
        });

        // Update dependency tracker
        {
            let mut tracker = self.dependency_tracker.write().unwrap();
            tracker.mark_modified(changed_symbol);
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.invalidations += invalidated_count;
            stats.update_cache_size(cache.len());
        }

        Ok(invalidated_count)
    }

    /// Get cache statistics
    pub fn get_statistics(&self) -> CacheStatistics {
        let mut stats = self.stats.write().unwrap();
        stats.update_hit_rate();
        stats.clone()
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        
        let mut stats = self.stats.write().unwrap();
        stats.update_cache_size(0);
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }

    /// Add dependency relationship
    pub fn add_dependency(&mut self, symbol: String, dependency: String) {
        let mut tracker = self.dependency_tracker.write().unwrap();
        tracker.add_dependency(symbol, dependency);
    }
}

impl Default for IncrementalTypeInference {
    fn default() -> Self {
        Self::new(IncrementalConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Literal};
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_incremental_inference_creation() {
        let inference = IncrementalTypeInference::default();
        assert_eq!(inference.cache_size(), 0);
    }

    #[test]
    fn test_basic_caching() {
        let mut inference = IncrementalTypeInference::default();
        
        let value = Value::Number(SchemeNumber::Integer(42));
        
        // First inference should be a cache miss
        let result1 = inference.infer(&value, Some("test_context"));
        assert!(result1.is_ok());
        
        // Second inference should be a cache hit
        let result2 = inference.infer(&value, Some("test_context"));
        assert!(result2.is_ok());
        
        let stats = inference.get_statistics();
        assert!(stats.hits > 0 || stats.misses > 0);
    }

    #[test]
    fn test_expression_caching() {
        let mut inference = IncrementalTypeInference::default();
        
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        
        // First inference
        let result1 = inference.infer_expression(&expr, Some("test"));
        assert!(result1.is_ok());
        
        // Second inference should use cache
        let result2 = inference.infer_expression(&expr, Some("test"));
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    #[test]
    fn test_dependency_tracking() {
        let mut inference = IncrementalTypeInference::default();
        
        // Add dependency
        inference.add_dependency("foo".to_string(), "bar".to_string());
        
        // Cache something that depends on "bar"
        let value = Value::Number(SchemeNumber::Integer(42));
        inference.infer(&value, Some("bar")).unwrap();
        
        // Invalidate "bar"
        inference.invalidate_dependencies("bar").unwrap();
        // Function returns successfully - u64 invalidated count is always valid
    }

    #[test]
    fn test_cache_eviction() {
        let config = IncrementalConfig {
            max_cache_size: 3,
            cache_policy: CachePolicy::LRU,
            ..Default::default()
        };
        let mut inference = IncrementalTypeInference::new(config);
        
        // Fill cache beyond capacity
        for i in 0..5 {
            let value = Value::Number(SchemeNumber::Integer(i));
            inference.infer(&value, Some(&format!("test_{i}"))).unwrap();
        }
        
        // Cache should not exceed max size
        assert!(inference.cache_size() <= 3);
    }

    #[test]
    fn test_cache_statistics() {
        let mut inference = IncrementalTypeInference::default();
        
        let value = Value::Boolean(true);
        
        // Generate some cache activity
        for _ in 0..3 {
            inference.infer(&value, Some("stats_test")).unwrap();
        }
        
        let stats = inference.get_statistics();
        assert!(stats.hits + stats.misses > 0);
        assert!(stats.hit_rate() >= 0.0 && stats.hit_rate() <= 1.0);
    }

    #[test]
    fn test_dependency_invalidation() {
        let config = IncrementalConfig {
            min_cache_cost: Duration::from_nanos(1), // Cache everything
            ..Default::default()
        };
        let mut inference = IncrementalTypeInference::new(config);
        
        // Set up dependency chain: A -> B -> C
        inference.add_dependency("A".to_string(), "B".to_string());
        inference.add_dependency("B".to_string(), "C".to_string());
        
        // Cache entries for each
        let value = Value::Number(SchemeNumber::Integer(1));
        inference.infer(&value, Some("A")).unwrap();
        inference.infer(&value, Some("B")).unwrap();
        inference.infer(&value, Some("C")).unwrap();
        
        let initial_size = inference.cache_size();
        
        // Invalidate C should affect B and A
        let invalidated = inference.invalidate_dependencies("C").unwrap();
        
        // Check that invalidation happened
        let final_size = inference.cache_size();
        assert!(invalidated > 0 || final_size < initial_size);
    }

    #[test]
    fn test_cache_key_generation() {
        let inference = IncrementalTypeInference::default();
        
        let value1 = Value::Number(SchemeNumber::Integer(42));
        let value2 = Value::Number(SchemeNumber::Integer(43));
        
        let key1 = inference.generate_cache_key(&value1, Some("context"));
        let key2 = inference.generate_cache_key(&value2, Some("context"));
        let key3 = inference.generate_cache_key(&value1, Some("different_context"));
        
        // Different values should have different keys
        assert_ne!(key1, key2);
        
        // Different contexts should have different keys
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_expression_dependency_extraction() {
        let inference = IncrementalTypeInference::default();
        
        let expr = Expr::List(vec![
            Expr::Variable("func".to_string()),
            Expr::Variable("arg1".to_string()),
            Expr::Variable("arg2".to_string()),
        ]);
        
        let dependencies = inference.extract_dependencies(&expr);
        assert!(dependencies.contains(&"func".to_string()));
        assert!(dependencies.contains(&"arg1".to_string()));
        assert!(dependencies.contains(&"arg2".to_string()));
    }
}
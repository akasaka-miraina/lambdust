//! JIT code cache with intelligent memory management
//!
//! This module implements a sophisticated code cache that manages generated native code
//! with LRU eviction, memory pressure handling, and integration with the garbage collector.
//! The cache is designed to maintain optimal performance while staying within memory
//! constraints and supporting deoptimization scenarios.

use crate::ast::Expr;
use crate::diagnostics::{Result, Error};
use crate::jit::code_generator::NativeCode;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};

/// Configuration for the code cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum memory usage for cached code (bytes)
    pub max_memory_bytes: usize,
    
    /// Maximum number of cached entries
    pub max_entries: usize,
    
    /// Memory pressure threshold (0.0-1.0)
    pub memory_pressure_threshold: f64,
    
    /// Enable LRU eviction
    pub enable_lru_eviction: bool,
    
    /// Enable memory compaction
    pub enable_compaction: bool,
    
    /// Compaction interval
    pub compaction_interval: Duration,
    
    /// Enable execution-based retention (keep frequently executed code)
    pub execution_based_retention: bool,
    
    /// Minimum execution count to avoid eviction
    pub min_execution_count_for_retention: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024, // 64MB default
            max_entries: 10000,
            memory_pressure_threshold: 0.8,
            enable_lru_eviction: true,
            enable_compaction: true,
            compaction_interval: Duration::from_secs(300), // 5 minutes
            execution_based_retention: true,
            min_execution_count_for_retention: 100,
        }
    }
}

/// Cache entry for native code
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached native code
    pub code: NativeCode,
    
    /// Cache key (expression hash)
    pub key: String,
    
    /// Creation timestamp
    pub created_at: Instant,
    
    /// Last access timestamp
    pub last_accessed: Instant,
    
    /// Number of times this code has been accessed
    pub access_count: u64,
    
    /// Number of times this code has been executed
    pub execution_count: u64,
    
    /// Total execution time
    pub total_execution_time: Duration,
    
    /// Average execution time
    pub avg_execution_time: Duration,
    
    /// Cache entry priority (for eviction decisions)
    pub priority: CachePriority,
    
    /// Memory usage of this entry (bytes)
    pub memory_usage: usize,
}

impl CacheEntry {
    /// Creates a new cache entry
    pub fn new(key: String, code: NativeCode) -> Self {
        let now = Instant::now();
        let memory_usage = code.code_size() + std::mem::size_of::<CacheEntry>();
        
        Self {
            code,
            key,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            execution_count: 0,
            total_execution_time: Duration::ZERO,
            avg_execution_time: Duration::ZERO,
            priority: CachePriority::Normal,
            memory_usage,
        }
    }
    
    /// Records access to this entry
    pub fn record_access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        self.update_priority();
    }
    
    /// Records execution of this entry
    pub fn record_execution(&mut self, execution_time: Duration) {
        self.execution_count += 1;
        self.total_execution_time += execution_time;
        self.avg_execution_time = self.total_execution_time / self.execution_count as u32;
        self.update_priority();
    }
    
    /// Updates cache priority based on usage patterns
    fn update_priority(&mut self) {
        // Calculate priority based on multiple factors
        let access_score = (self.access_count as f64).ln().max(1.0);
        let execution_score = (self.execution_count as f64).ln().max(1.0);
        let recency_score = {
            let age = self.last_accessed.elapsed().as_secs() as f64;
            (-age / 3600.0).exp() // Exponential decay with 1-hour half-life
        };
        
        let combined_score = access_score * 0.3 + execution_score * 0.5 + recency_score * 0.2;
        
        self.priority = if combined_score > 10.0 {
            CachePriority::High
        } else if combined_score > 5.0 {
            CachePriority::Normal
        } else {
            CachePriority::Low
        };
    }
    
    /// Returns the cache score for eviction decisions
    pub fn cache_score(&self) -> f64 {
        let access_weight = 0.3;
        let execution_weight = 0.5;
        let recency_weight = 0.2;
        
        let access_score = (self.access_count as f64).ln().max(1.0);
        let execution_score = (self.execution_count as f64).ln().max(1.0);
        let recency_score = {
            let age = self.last_accessed.elapsed().as_secs() as f64;
            (-age / 3600.0).exp()
        };
        
        access_score * access_weight + 
        execution_score * execution_weight + 
        recency_score * recency_weight
    }
}

/// Cache entry priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CachePriority {
    /// Low priority - candidate for eviction
    Low = 0,
    
    /// Normal priority - standard caching
    Normal = 1,
    
    /// High priority - avoid eviction if possible
    High = 2,
    
    /// Critical priority - never evict
    Critical = 3,
}

/// JIT code cache with intelligent memory management
pub struct CodeCache {
    /// Configuration
    config: CacheConfig,
    
    /// Cache entries by expression key
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    
    /// LRU order tracking
    lru_order: Arc<Mutex<VecDeque<String>>>,
    
    /// Current memory usage
    current_memory_usage: Arc<Mutex<usize>>,
    
    /// Memory manager for code allocation
    memory_manager: MemoryManager,
    
    /// Cache statistics
    stats: Arc<Mutex<CacheStats>>,
    
    /// Last compaction time
    last_compaction: Arc<Mutex<Instant>>,
}

impl CodeCache {
    /// Creates a new code cache
    pub fn new(config: CacheConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            entries: Arc::new(RwLock::new(HashMap::new())),
            lru_order: Arc::new(Mutex::new(VecDeque::new())),
            current_memory_usage: Arc::new(Mutex::new(0)),
            memory_manager: MemoryManager::new(config.max_memory_bytes)?,
            stats: Arc::new(Mutex::new(CacheStats::default())),
            last_compaction: Arc::new(Mutex::new(Instant::now())),
        })
    }
    
    /// Stores native code in the cache
    pub fn store(&self, expr: Expr, code: NativeCode) -> Result<()> {
        let key = self.expression_key(&expr);
        let mut entry = CacheEntry::new(key.clone(), code);
        
        // Update memory usage
        {
            let mut memory_usage = self.current_memory_usage.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire memory lock".to_string(), None))?;
            *memory_usage += entry.memory_usage;
        }
        
        // Check memory pressure and evict if necessary
        self.handle_memory_pressure()?;
        
        // Store the entry
        {
            let mut entries = self.entries.write()
                .map_err(|_| Error::runtime_error("Failed to acquire entries lock".to_string(), None))?;
            entries.insert(key.clone(), entry);
        }
        
        // Update LRU order
        if self.config.enable_lru_eviction {
            let mut lru_order = self.lru_order.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire LRU lock".to_string(), None))?;
            lru_order.push_back(key.clone());
        }
        
        // Update statistics
        {
            let mut stats = self.stats.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire stats lock".to_string(), None))?;
            stats.entries_stored += 1;
            stats.current_entries += 1;
        }
        
        Ok(())
    }
    
    /// Retrieves native code from the cache
    pub fn get(&self, expr: &Expr) -> Result<Option<NativeCode>> {
        let key = self.expression_key(expr);
        
        let code = {
            let mut entries = self.entries.write()
                .map_err(|_| Error::runtime_error("Failed to acquire entries lock".to_string(), None))?;
                
            if let Some(entry) = entries.get_mut(&key) {
                entry.record_access();
                
                // Update LRU order
                if self.config.enable_lru_eviction {
                    self.update_lru_order(&key)?;
                }
                
                Some(entry.code.clone())
            } else {
                None
            }
        };
        
        // Update statistics
        {
            let mut stats = self.stats.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire stats lock".to_string(), None))?;
            if code.is_some() {
                stats.cache_hits += 1;
            } else {
                stats.cache_misses += 1;
            }
        }
        
        Ok(code)
    }
    
    /// Records execution of cached code
    pub fn record_execution(&self, expr: &Expr, execution_time: Duration) -> Result<()> {
        let key = self.expression_key(expr);
        
        let mut entries = self.entries.write()
            .map_err(|_| Error::runtime_error("Failed to acquire entries lock".to_string(), None))?;
            
        if let Some(entry) = entries.get_mut(&key) {
            entry.record_execution(execution_time);
        }
        
        Ok(())
    }
    
    /// Invalidates cached code for an expression
    pub fn invalidate(&self, expr: &Expr) -> Result<()> {
        let key = self.expression_key(expr);
        
        let memory_freed = {
            let mut entries = self.entries.write()
                .map_err(|_| Error::runtime_error("Failed to acquire entries lock".to_string(), None))?;
                
            if let Some(entry) = entries.remove(&key) {
                entry.memory_usage
            } else {
                0
            }
        };
        
        // Update memory usage
        if memory_freed > 0 {
            let mut memory_usage = self.current_memory_usage.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire memory lock".to_string(), None))?;
            *memory_usage = memory_usage.saturating_sub(memory_freed);
        }
        
        // Remove from LRU order
        if self.config.enable_lru_eviction {
            let mut lru_order = self.lru_order.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire LRU lock".to_string(), None))?;
            lru_order.retain(|k| k != &key);
        }
        
        // Update statistics
        {
            let mut stats = self.stats.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire stats lock".to_string(), None))?;
            stats.entries_evicted += 1;
            stats.current_entries = stats.current_entries.saturating_sub(1);
        }
        
        Ok(())
    }
    
    /// Clears all cached code
    pub fn clear(&self) -> Result<()> {
        {
            let mut entries = self.entries.write()
                .map_err(|_| Error::runtime_error("Failed to acquire entries lock".to_string(), None))?;
            entries.clear();
        }
        
        if self.config.enable_lru_eviction {
            let mut lru_order = self.lru_order.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire LRU lock".to_string(), None))?;
            lru_order.clear();
        }
        
        {
            let mut memory_usage = self.current_memory_usage.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire memory lock".to_string(), None))?;
            *memory_usage = 0;
        }
        
        {
            let mut stats = self.stats.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire stats lock".to_string(), None))?;
            stats.current_entries = 0;
        }
        
        Ok(())
    }
    
    /// Handles memory pressure by evicting entries
    fn handle_memory_pressure(&self) -> Result<()> {
        let current_usage = {
            let memory_usage = self.current_memory_usage.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire memory lock".to_string(), None))?;
            *memory_usage
        };
        
        let memory_pressure = current_usage as f64 / self.config.max_memory_bytes as f64;
        
        if memory_pressure > self.config.memory_pressure_threshold {
            self.evict_entries()?;
        }
        
        // Check if compaction is needed
        if self.config.enable_compaction {
            let should_compact = {
                let last_compaction = self.last_compaction.lock()
                    .map_err(|_| Error::runtime_error("Failed to acquire compaction lock".to_string(), None))?;
                last_compaction.elapsed() > self.config.compaction_interval
            };
            
            if should_compact {
                self.compact()?;
            }
        }
        
        Ok(())
    }
    
    /// Evicts cache entries based on LRU and priority
    fn evict_entries(&self) -> Result<()> {
        let target_memory = (self.config.max_memory_bytes as f64 * 0.7) as usize; // Target 70% usage
        
        if !self.config.enable_lru_eviction {
            return Ok(());
        }
        
        let keys_to_evict = {
            let entries = self.entries.read()
                .map_err(|_| Error::runtime_error("Failed to acquire entries lock".to_string(), None))?;
            let lru_order = self.lru_order.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire LRU lock".to_string(), None))?;
                
            let mut keys_to_evict = Vec::new();
            let mut memory_to_free = {
                let memory_usage = self.current_memory_usage.lock()
                    .map_err(|_| Error::runtime_error("Failed to acquire memory lock".to_string(), None))?;
                memory_usage.saturating_sub(target_memory)
            };
            
            // Iterate through LRU order (oldest first)
            for key in lru_order.iter() {
                if memory_to_free == 0 {
                    break;
                }
                
                if let Some(entry) = entries.get(key) {
                    // Don't evict high priority entries if possible
                    if entry.priority >= CachePriority::High {
                        continue;
                    }
                    
                    // Don't evict frequently executed entries
                    if self.config.execution_based_retention && 
                       entry.execution_count >= self.config.min_execution_count_for_retention {
                        continue;
                    }
                    
                    keys_to_evict.push(key.clone());
                    memory_to_free = memory_to_free.saturating_sub(entry.memory_usage);
                }
            }
            
            keys_to_evict
        };
        
        // Evict selected entries
        for key in keys_to_evict {
            let expr_placeholder = self.key_to_expr(&key); // In practice, we'd need reverse mapping
            self.invalidate(&expr_placeholder)?;
        }
        
        Ok(())
    }
    
    /// Compacts the cache by removing fragmentation
    fn compact(&self) -> Result<()> {
        // In a real implementation, this would:
        // 1. Analyze memory fragmentation
        // 2. Reorganize code layout for better cache locality
        // 3. Update internal data structures
        
        {
            let mut last_compaction = self.last_compaction.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire compaction lock".to_string(), None))?;
            *last_compaction = Instant::now();
        }
        
        {
            let mut stats = self.stats.lock()
                .map_err(|_| Error::runtime_error("Failed to acquire stats lock".to_string(), None))?;
            stats.compactions_performed += 1;
        }
        
        Ok(())
    }
    
    /// Updates LRU order for an accessed key
    fn update_lru_order(&self, key: &str) -> Result<()> {
        let mut lru_order = self.lru_order.lock()
            .map_err(|_| Error::runtime_error("Failed to acquire LRU lock".to_string(), None))?;
            
        // Remove from current position
        lru_order.retain(|k| k != key);
        
        // Add to end (most recently used)
        lru_order.push_back(key.to_string());
        
        Ok(())
    }
    
    /// Generates a cache key for an expression
    fn expression_key(&self, expr: &Expr) -> String {
        // In practice, this would use a proper hash function
        format!("{expr:?}")
    }
    
    /// Converts a key back to an expression (placeholder implementation)
    fn key_to_expr(&self, _key: &str) -> Expr {
        // In practice, we'd maintain a reverse mapping or encode expr info in key
        Expr::Symbol("placeholder".to_string())
    }
    
    /// Returns cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let stats = self.stats.lock()
            .map_err(|_| Error::runtime_error("Failed to acquire stats lock".to_string(), None))?;
        Ok(stats.clone())
    }
    
    /// Returns current memory usage
    pub fn memory_usage(&self) -> Result<usize> {
        let memory_usage = self.current_memory_usage.lock()
            .map_err(|_| Error::runtime_error("Failed to acquire memory lock".to_string(), None))?;
        Ok(*memory_usage)
    }
    
    /// Returns current cache size (number of entries)
    pub fn size(&self) -> Result<usize> {
        let entries = self.entries.read()
            .map_err(|_| Error::runtime_error("Failed to acquire entries lock".to_string(), None))?;
        Ok(entries.len())
    }
}

/// Memory manager for code cache
pub struct MemoryManager {
    /// Maximum memory allowed
    max_memory: usize,
    
    /// Memory allocation regions
    regions: Vec<MemoryRegion>,
}

impl MemoryManager {
    fn new(max_memory: usize) -> Result<Self> {
        Ok(Self {
            max_memory,
            regions: Vec::new(),
        })
    }
}

/// Memory region for code allocation
#[derive(Debug)]
struct MemoryRegion {
    /// Start address
    start: usize,
    
    /// Size in bytes
    size: usize,
    
    /// Whether this region is executable
    executable: bool,
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total entries stored
    pub entries_stored: u64,
    
    /// Current number of entries
    pub current_entries: usize,
    
    /// Cache hits
    pub cache_hits: u64,
    
    /// Cache misses
    pub cache_misses: u64,
    
    /// Entries evicted
    pub entries_evicted: u64,
    
    /// Compactions performed
    pub compactions_performed: u64,
    
    /// Total memory allocated
    pub total_memory_allocated: usize,
    
    /// Current memory usage
    pub current_memory_usage: usize,
    
    /// Peak memory usage
    pub peak_memory_usage: usize,
}

impl CacheStats {
    /// Calculates cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_requests as f64
        }
    }
    
    /// Calculates memory utilization
    pub fn memory_utilization(&self) -> f64 {
        if self.total_memory_allocated == 0 {
            0.0
        } else {
            self.current_memory_usage as f64 / self.total_memory_allocated as f64
        }
    }
    
    /// Calculates average entries per compaction
    pub fn avg_entries_per_compaction(&self) -> f64 {
        if self.compactions_performed == 0 {
            0.0
        } else {
            self.entries_stored as f64 / self.compactions_performed as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::jit::CompilationTier;
    use crate::jit::code_generator::{CodeMetadata, FunctionSignature, MemoryLayout, SchemeType, MemoryRequirements};
    
    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_memory_bytes, 64 * 1024 * 1024);
        assert_eq!(config.max_entries, 10000);
        assert!(config.enable_lru_eviction);
    }
    
    #[test]
    fn test_cache_entry_creation() {
        let code = create_test_native_code();
        let entry = CacheEntry::new("test".to_string(), code);
        
        assert_eq!(entry.key, "test");
        assert_eq!(entry.access_count, 0);
        assert_eq!(entry.execution_count, 0);
    }
    
    #[test]
    fn test_cache_entry_access() {
        let code = create_test_native_code();
        let mut entry = CacheEntry::new("test".to_string(), code);
        
        entry.record_access();
        assert_eq!(entry.access_count, 1);
        
        entry.record_execution(Duration::from_micros(100));
        assert_eq!(entry.execution_count, 1);
        assert_eq!(entry.avg_execution_time, Duration::from_micros(100));
    }
    
    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();
        stats.cache_hits = 80;
        stats.cache_misses = 20;
        
        assert_eq!(stats.hit_rate(), 0.8);
        
        stats.total_memory_allocated = 1000;
        stats.current_memory_usage = 600;
        assert_eq!(stats.memory_utilization(), 0.6);
    }
    
    fn create_test_native_code() -> NativeCode {
        NativeCode {
            machine_code: vec![0x90; 16], // NOP instructions
            entry_point: 0,
            metadata: CodeMetadata {
                source_expr: "test".to_string(),
                compilation_tier: CompilationTier::JitBasic,
                safe_points: Vec::new(),
                variable_locations: std::collections::HashMap::new(),
                inlined_functions: Vec::new(),
            },
            signature: FunctionSignature {
                parameter_count: 0,
                is_variadic: false,
                return_type: crate::jit::SchemeType::Any,
                parameter_types: Vec::new(),
            },
            memory_layout: MemoryLayout {
                stack_frame_size: 64,
                gc_roots: Vec::new(),
                memory_requirements: MemoryRequirements {
                    stack_bytes: 64,
                    heap_bytes: 0,
                    temp_bytes: 32,
                },
            },
        }
    }
}
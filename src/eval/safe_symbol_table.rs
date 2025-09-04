//! Memory-safe symbol interning system for Lambdust
//! 
//! Replaces unsafe raw pointer operations with Arc-based thread-safe design
//! while maintaining performance characteristics of symbol lookup operations.

use std::collections::HashMap;
use std::sync::{Arc, RwLock, atomic::{AtomicU32, Ordering}};
use crate::utils::SymbolId;

/// Thread-safe symbol interning table without unsafe operations
/// 
/// This implementation replaces raw pointer-based symbol management
/// with Arc-based safety while maintaining O(1) lookup performance.
pub struct SafeSymbolTable {
    /// String to SymbolId mapping
    string_to_id: Arc<RwLock<HashMap<String, SymbolId>>>,
    
    /// SymbolId to string reverse mapping
    id_to_string: Arc<RwLock<Vec<Arc<str>>>>,
    
    /// Next available symbol ID
    next_id: AtomicU32,
    
    /// Cache for frequently accessed symbols
    cache: Arc<RwLock<SymbolCache>>,
}

/// Cache for frequently accessed symbols to avoid lock contention
#[derive(Debug)]
struct SymbolCache {
    entries: HashMap<SymbolId, CachedSymbol>,
    max_size: usize,
    generation: u64,
}

#[derive(Debug, Clone)]
struct CachedSymbol {
    string: Arc<str>,
    access_count: u64,
    generation: u64,
}

impl SafeSymbolTable {
    /// Create a new safe symbol table
    pub fn new() -> Self {
        Self {
            string_to_id: Arc::new(RwLock::new(HashMap::new())),
            id_to_string: Arc::new(RwLock::new(Vec::new())),
            next_id: AtomicU32::new(0),
            cache: Arc::new(RwLock::new(SymbolCache::new())),
        }
    }
    
    /// Intern a string as a symbol, returning its SymbolId
    /// 
    /// This operation is memory-safe and thread-safe without raw pointers
    pub fn intern(&self, string: &str) -> SymbolId {
        // Check if already exists (read-only operation first)
        {
            let string_to_id = self.string_to_id.read().unwrap();
            if let Some(&id) = string_to_id.get(string) {
                self.update_cache_access(id, string);
                return id;
            }
        }
        
        // Need to create new symbol (write operation)
        let mut string_to_id = self.string_to_id.write().unwrap();
        let mut id_to_string = self.id_to_string.write().unwrap();
        
        // Double-check pattern to avoid race condition
        if let Some(&id) = string_to_id.get(string) {
            drop(string_to_id);
            drop(id_to_string);
            self.update_cache_access(id, string);
            return id;
        }
        
        // Create new symbol
        let id = SymbolId(self.next_id.fetch_add(1, Ordering::SeqCst));
        let arc_string: Arc<str> = string.into();
        
        string_to_id.insert(string.to_string(), id);
        
        // Ensure vector capacity
        if id_to_string.len() <= id.0 as usize {
            id_to_string.resize(id.0 as usize + 1, "".into());
        }
        id_to_string[id.0 as usize] = arc_string.clone();
        
        // Update cache
        self.add_to_cache(id, arc_string);
        
        id
    }
    
    /// Get string representation of a symbol ID
    /// 
    /// Memory-safe lookup without raw pointer dereference
    pub fn get_string(&self, id: SymbolId) -> Option<Arc<str>> {
        // Try cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached) = cache.entries.get(&id) {
                return Some(cached.string.clone());
            }
        }
        
        // Fallback to main table
        let id_to_string = self.id_to_string.read().unwrap();
        if let Some(string) = id_to_string.get(id.0 as usize) {
            let result = string.clone();
            self.add_to_cache(id, result.clone());
            Some(result)
        } else {
            None
        }
    }
    
    /// Check if a symbol exists
    pub fn contains(&self, string: &str) -> bool {
        let string_to_id = self.string_to_id.read().unwrap();
        string_to_id.contains_key(string)
    }
    
    /// Get symbol statistics for debugging
    pub fn statistics(&self) -> SymbolTableStats {
        let string_to_id = self.string_to_id.read().unwrap();
        let cache = self.cache.read().unwrap();
        
        SymbolTableStats {
            total_symbols: string_to_id.len(),
            cached_symbols: cache.entries.len(),
            cache_hit_ratio: cache.calculate_hit_ratio(),
        }
    }
    
    /// Update cache access count
    fn update_cache_access(&self, id: SymbolId, string: &str) {
        let mut cache = self.cache.write().unwrap();
        if let Some(cached) = cache.entries.get_mut(&id) {
            cached.access_count += 1;
        } else {
            self.add_to_cache_internal(&mut cache, id, string.into());
        }
    }
    
    /// Add symbol to cache
    fn add_to_cache(&self, id: SymbolId, string: Arc<str>) {
        let mut cache = self.cache.write().unwrap();
        self.add_to_cache_internal(&mut cache, id, string);
    }
    
    /// Internal cache addition with lock already held
    fn add_to_cache_internal(&self, cache: &mut SymbolCache, id: SymbolId, string: Arc<str>) {
        if cache.entries.len() >= cache.max_size {
            cache.evict_lru();
        }
        
        cache.entries.insert(id, CachedSymbol {
            string,
            access_count: 1,
            generation: cache.generation,
        });
    }
}

impl SymbolCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            max_size: 1000, // Configurable cache size
            generation: 0,
        }
    }
    
    fn evict_lru(&mut self) {
        // Remove least recently used entries
        let threshold = self.generation.saturating_sub(100);
        self.entries.retain(|_, symbol| symbol.generation > threshold);
        
        // If still over capacity, remove half
        if self.entries.len() >= self.max_size {
            let mut entries: Vec<_> = self.entries.drain().collect();
            entries.sort_by_key(|(_, symbol)| symbol.access_count);
            entries.truncate(self.max_size / 2);
            self.entries.extend(entries);
        }
        
        self.generation += 1;
    }
    
    fn calculate_hit_ratio(&self) -> f64 {
        if self.entries.is_empty() {
            0.0
        } else {
            let total_accesses: u64 = self.entries.values()
                .map(|s| s.access_count)
                .sum();
            let cache_hits = total_accesses.saturating_sub(self.entries.len() as u64);
            cache_hits as f64 / total_accesses as f64
        }
    }
}

/// Statistics about symbol table usage
#[derive(Debug, Clone)]
pub struct SymbolTableStats {
    pub total_symbols: usize,
    pub cached_symbols: usize, 
    pub cache_hit_ratio: f64,
}

/// Global safe symbol table instance
static GLOBAL_SYMBOL_TABLE: once_cell::sync::Lazy<SafeSymbolTable> = 
    once_cell::sync::Lazy::new(SafeSymbolTable::new);

/// Global functions for symbol interning (API compatibility)
pub fn intern_symbol(string: &str) -> SymbolId {
    GLOBAL_SYMBOL_TABLE.intern(string)
}

pub fn symbol_to_string(id: SymbolId) -> Option<Arc<str>> {
    GLOBAL_SYMBOL_TABLE.get_string(id)
}

pub fn symbol_exists(string: &str) -> bool {
    GLOBAL_SYMBOL_TABLE.contains(string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Barrier;
    
    #[test]
    fn test_basic_interning() {
        let table = SafeSymbolTable::new();
        
        let id1 = table.intern("test");
        let id2 = table.intern("test");
        
        assert_eq!(id1, id2, "Same string should return same ID");
        
        let string = table.get_string(id1).unwrap();
        assert_eq!(&*string, "test");
    }
    
    #[test]
    fn test_thread_safety() {
        let table = Arc::new(SafeSymbolTable::new());
        let barrier = Arc::new(Barrier::new(4));
        
        let handles: Vec<_> = (0..4).map(|i| {
            let table = table.clone();
            let barrier = barrier.clone();
            
            thread::spawn(move || {
                barrier.wait();
                
                // Each thread interns different symbols
                for j in 0..100 {
                    let symbol = format!("test_{}_{}", i, j);
                    let id = table.intern(&symbol);
                    let retrieved = table.get_string(id).unwrap();
                    assert_eq!(&*retrieved, symbol);
                }
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let stats = table.statistics();
        assert_eq!(stats.total_symbols, 400, "Should have 400 unique symbols");
    }
    
    #[test]
    fn test_cache_performance() {
        let table = SafeSymbolTable::new();
        
        // Intern some symbols
        let ids: Vec<_> = (0..10).map(|i| {
            table.intern(&format!("symbol_{}", i))
        }).collect();
        
        // Access them multiple times to populate cache
        for _ in 0..100 {
            for &id in &ids {
                let _ = table.get_string(id);
            }
        }
        
        let stats = table.statistics();
        println!("Cache hit ratio: {:.2}%", stats.cache_hit_ratio * 100.0);
        assert!(stats.cached_symbols > 0, "Cache should contain symbols");
    }
}
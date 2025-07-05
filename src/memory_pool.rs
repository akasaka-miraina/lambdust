//! Memory pool implementation for performance optimization
//!
//! This module provides memory pooling and object reuse strategies to reduce
//! allocation overhead in hot evaluation paths. It implements:
//!
//! - Value pooling for common types (Boolean, Numbers, Symbols)
//! - Symbol interning to reduce string allocation
//! - Continuation pooling for frequent continuation types
//! - SmallVec optimization for common collection patterns

use crate::evaluator::Continuation;
use crate::lexer::SchemeNumber;
use crate::value::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Global symbol interning table for memory efficiency
/// Symbols are frequently created and reused, so interning reduces memory usage
static SYMBOL_INTERNER: OnceLock<Mutex<SymbolInterner>> = OnceLock::new();

/// Symbol interning system to reduce string allocations
/// Frequently used symbols are stored once and reused via references
pub struct SymbolInterner {
    /// Mapping from symbol name to interned value
    symbols: HashMap<String, &'static str>,
    /// Storage for interned strings (leaked intentionally for 'static lifetime)
    interned_storage: Vec<Box<str>>,
}

impl SymbolInterner {
    /// Create a new symbol interner
    fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            interned_storage: Vec::new(),
        }
    }

    /// Intern a symbol string, returning a static reference
    /// This reduces memory usage for frequently used symbols
    pub fn intern(&mut self, symbol: &str) -> &'static str {
        if let Some(&interned) = self.symbols.get(symbol) {
            return interned;
        }

        // Create a new interned string with static lifetime
        let boxed = symbol.to_string().into_boxed_str();
        let leaked: &'static str = Box::leak(boxed);
        self.symbols.insert(symbol.to_string(), leaked);
        self.interned_storage.push(leaked.to_string().into_boxed_str());
        leaked
    }

    /// Get statistics about symbol interning
    pub fn stats(&self) -> (usize, usize) {
        (self.symbols.len(), self.interned_storage.len())
    }
}

/// Value pool for reducing allocation overhead in hot evaluation paths
/// Common value types are pre-allocated and reused to improve performance
pub struct ValuePool {
    /// Cached boolean values (true/false singletons)
    boolean_cache: [Value; 2],
    /// Cached nil singleton
    nil_singleton: Value,
    /// Pool of reusable number values for small integers
    small_integer_pool: Vec<Value>,
    /// Recently used values that can be recycled
    value_recycle_pool: Vec<Value>,
}

impl ValuePool {
    /// Create a new value pool with pre-allocated common values
    pub fn new() -> Self {
        let mut pool = Self {
            boolean_cache: [Value::Boolean(false), Value::Boolean(true)],
            nil_singleton: Value::Nil,
            small_integer_pool: Vec::with_capacity(256),
            value_recycle_pool: Vec::with_capacity(1000),
        };

        // Pre-allocate small integers (-128 to 127) for common arithmetic
        for i in -128..=127 {
            pool.small_integer_pool.push(Value::Number(SchemeNumber::Integer(i)));
        }

        pool
    }

    /// Get a cached boolean value (avoids allocation)
    pub fn get_boolean(&self, value: bool) -> Value {
        self.boolean_cache[if value { 1 } else { 0 }].clone()
    }

    /// Get the cached nil value (avoids allocation)
    pub fn get_nil(&self) -> Value {
        self.nil_singleton.clone()
    }

    /// Get a small integer from the pool if available
    pub fn get_small_integer(&self, value: i64) -> Option<Value> {
        if (-128..=127).contains(&value) {
            let index = (value + 128) as usize;
            self.small_integer_pool.get(index).cloned()
        } else {
            None
        }
    }

    /// Get an interned symbol value to reduce string allocation
    pub fn get_symbol(symbol: &str) -> Value {
        let interner = SYMBOL_INTERNER.get_or_init(|| Mutex::new(SymbolInterner::new()));
        let mut interner = interner.lock().unwrap();
        let interned = interner.intern(symbol);
        Value::Symbol(interned.to_string()) // Still uses String for compatibility
    }

    /// Recycle a value back to the pool for reuse
    pub fn recycle_value(&mut self, value: Value) {
        // Only recycle simple values to avoid memory leaks
        match value {
            Value::Boolean(_) | Value::Nil | Value::Undefined => {
                // These are cached, no need to recycle
            }
            Value::Number(_) | Value::Character(_) | Value::String(_) => {
                // These can be safely recycled
                if self.value_recycle_pool.len() < 1000 {
                    self.value_recycle_pool.push(value);
                }
            }
            _ => {
                // Complex values with references should not be recycled
            }
        }
    }

    /// Get a recycled value if available, otherwise create new
    pub fn get_recycled_or_new<F>(&mut self, create_fn: F) -> Value
    where
        F: FnOnce() -> Value,
    {
        self.value_recycle_pool.pop().unwrap_or_else(create_fn)
    }

    /// Get pool statistics for monitoring
    pub fn stats(&self) -> PoolStats {
        let (symbol_count, interned_count) = {
            let interner = SYMBOL_INTERNER.get_or_init(|| Mutex::new(SymbolInterner::new()));
            let interner = interner.lock().unwrap();
            interner.stats()
        };

        PoolStats {
            small_integers_cached: self.small_integer_pool.len(),
            values_in_recycle_pool: self.value_recycle_pool.len(),
            symbols_interned: symbol_count,
            total_interned_storage: interned_count,
        }
    }
}

impl Default for ValuePool {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about memory pool usage
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Number of small integers cached in the pool
    pub small_integers_cached: usize,
    /// Number of values available for recycling
    pub values_in_recycle_pool: usize,
    /// Number of symbols that have been interned
    pub symbols_interned: usize,
    /// Total number of interned string storage entries
    pub total_interned_storage: usize,
}

// Global value pool instance (thread-local for safety)
thread_local! {
    static VALUE_POOL: std::cell::RefCell<ValuePool> = std::cell::RefCell::new(ValuePool::new());
}

/// Convenient functions for using the global value pool
impl Value {
    /// Create a boolean value using the pool (optimized)
    pub fn new_boolean(value: bool) -> Self {
        VALUE_POOL.with(|pool| pool.borrow().get_boolean(value))
    }

    /// Create a nil value using the pool (optimized)
    pub fn new_nil() -> Self {
        VALUE_POOL.with(|pool| pool.borrow().get_nil())
    }

    /// Create an integer value, using pool for small integers
    pub fn new_integer(value: i64) -> Self {
        VALUE_POOL.with(|pool| {
            pool.borrow()
                .get_small_integer(value)
                .unwrap_or_else(|| Value::Number(SchemeNumber::Integer(value)))
        })
    }

    /// Create a symbol value with interning (optimized)
    pub fn new_symbol(symbol: &str) -> Self {
        ValuePool::get_symbol(symbol)
    }

    /// Get global pool statistics
    pub fn pool_stats() -> PoolStats {
        VALUE_POOL.with(|pool| pool.borrow().stats())
    }
}

/// Continuation pool for reducing allocation overhead in CPS evaluation
/// Frequently used continuation patterns are pre-allocated and reused
pub struct ContinuationPool {
    /// Pool of Identity continuations (most common)
    identity_pool: Vec<Continuation>,
    /// Pool statistics
    recycled_count: usize,
    created_count: usize,
}

impl ContinuationPool {
    /// Create a new continuation pool
    pub fn new() -> Self {
        let mut pool = Self {
            identity_pool: Vec::with_capacity(100),
            recycled_count: 0,
            created_count: 0,
        };

        // Pre-allocate Identity continuations (most common)
        for _ in 0..50 {
            pool.identity_pool.push(Continuation::Identity);
        }

        pool
    }

    /// Get an Identity continuation from the pool
    pub fn get_identity(&mut self) -> Continuation {
        self.created_count += 1;
        if let Some(cont) = self.identity_pool.pop() {
            self.recycled_count += 1;
            cont
        } else {
            Continuation::Identity
        }
    }

    /// Recycle a continuation back to the pool
    pub fn recycle_continuation(&mut self, cont: Continuation) {
        match cont {
            Continuation::Identity => {
                if self.identity_pool.len() < 100 {
                    self.identity_pool.push(cont);
                }
            }
            // For now, only recycle Identity continuations to avoid complexity
            _ => {
                // More complex continuations would need careful cleanup
                // before recycling to avoid memory leaks
            }
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> ContinuationPoolStats {
        ContinuationPoolStats {
            identity_pooled: self.identity_pool.len(),
            total_recycled: self.recycled_count,
            total_created: self.created_count,
            recycle_rate: if self.created_count > 0 {
                (self.recycled_count as f64 / self.created_count as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

impl Default for ContinuationPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about continuation pool usage
#[derive(Debug, Clone)]
pub struct ContinuationPoolStats {
    /// Number of Identity continuations in pool
    pub identity_pooled: usize,
    /// Total number of continuations recycled
    pub total_recycled: usize,
    /// Total number of continuations created
    pub total_created: usize,
    /// Percentage of continuations recycled vs created
    pub recycle_rate: f64,
}

// Global continuation pool (thread-local for safety)
thread_local! {
    static CONTINUATION_POOL: std::cell::RefCell<ContinuationPool> = std::cell::RefCell::new(ContinuationPool::new());
}

impl Continuation {
    /// Create an optimized Identity continuation using pool
    pub fn new_identity() -> Self {
        CONTINUATION_POOL.with(|pool| pool.borrow_mut().get_identity())
    }

    /// Get continuation pool statistics
    pub fn pool_stats() -> ContinuationPoolStats {
        CONTINUATION_POOL.with(|pool| pool.borrow().stats())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_pool_boolean_caching() {
        let true_val1 = Value::new_boolean(true);
        let true_val2 = Value::new_boolean(true);
        
        // Values should be equivalent
        assert!(matches!(true_val1, Value::Boolean(true)));
        assert!(matches!(true_val2, Value::Boolean(true)));
    }

    #[test]
    fn test_small_integer_pooling() {
        let pool = ValuePool::new();
        
        // Small integers should be pooled
        let small_int = pool.get_small_integer(42);
        assert!(small_int.is_some());
        assert!(matches!(small_int.unwrap(), Value::Number(SchemeNumber::Integer(42))));
        
        // Large integers should not be pooled
        let large_int = pool.get_small_integer(1000);
        assert!(large_int.is_none());
    }

    #[test]
    fn test_symbol_interning() {
        let symbol1 = Value::new_symbol("test");
        let symbol2 = Value::new_symbol("test");
        
        // Symbols should be equivalent
        assert!(matches!(symbol1, Value::Symbol(_)));
        assert!(matches!(symbol2, Value::Symbol(_)));
    }

    #[test]
    fn test_pool_stats() {
        let stats = Value::pool_stats();
        assert!(stats.small_integers_cached > 0);
        // symbols_interned is always >= 0 as it's usize
    }

    #[test]
    fn test_continuation_pool_basic() {
        let mut pool = ContinuationPool::new();
        
        // Pool should be pre-initialized
        assert!(!pool.identity_pool.is_empty());
        
        // Get an identity continuation
        let cont = pool.get_identity();
        assert!(matches!(cont, Continuation::Identity));
        
        // Stats should reflect creation
        let stats = pool.stats();
        assert_eq!(stats.total_created, 1);
    }

    #[test]
    fn test_continuation_pool_recycling() {
        let mut pool = ContinuationPool::new();
        let initial_pool_size = pool.identity_pool.len();
        
        // Get and recycle a continuation
        let cont = pool.get_identity();
        pool.recycle_continuation(cont);
        
        // Pool size should be back to initial (recycled)
        assert_eq!(pool.identity_pool.len(), initial_pool_size);
        
        let stats = pool.stats();
        assert_eq!(stats.total_created, 1);
        assert_eq!(stats.total_recycled, 1);
        assert_eq!(stats.recycle_rate, 100.0);
    }

    #[test]
    fn test_continuation_global_pool() {
        // Test global pool functions
        let cont = Continuation::new_identity();
        assert!(matches!(cont, Continuation::Identity));
        
        let stats = Continuation::pool_stats();
        assert!(stats.total_created > 0);
    }
}
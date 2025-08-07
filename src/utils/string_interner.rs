//! String interning for performance optimization.
//!
//! This module provides a string interning system to reduce memory usage
//! and improve performance by storing only one copy of each unique string.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex};

/// A thread-safe string interner that stores unique strings.
#[derive(Debug)]
pub struct StringInterner {
    /// Map from string content to interned ID
    string_to_id: RwLock<HashMap<String, InternedId>>,
    /// Map from interned ID to string content
    id_to_string: RwLock<Vec<Arc<str>>>,
}

/// An interned string identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedId(usize);

/// An interned string that can be cheaply cloned and compared.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InternedString {
    id: InternedId,
    content: Arc<str>,
}

impl StringInterner {
    /// Creates a new string interner.
    pub fn new() -> Self {
        Self {
            string_to_id: RwLock::new(HashMap::new()),
            id_to_string: RwLock::new(Vec::new()),
        }
    }

    /// Interns a string, returning an interned string handle.
    /// If the string is already interned, returns the existing handle.
    pub fn intern(&self, s: &str) -> InternedString {
        // Fast path: check if already interned with read lock
        if let Ok(string_to_id) = self.string_to_id.read() {
            if let Some(&id) = string_to_id.get(s) {
                if let Ok(id_to_string) = self.id_to_string.read() {
                    if let Some(content) = id_to_string.get(id.0) {
                        return InternedString {
                            id,
                            content: content.clone()),
                        };
                    }
                }
            }
        }

        // Slow path: need to intern the string
        let mut string_to_id = self.string_to_id.write().unwrap();
        let mut id_to_string = self.id_to_string.write().unwrap();

        // Double-check after acquiring write lock
        if let Some(&id) = string_to_id.get(s) {
            if let Some(content) = id_to_string.get(id.0) {
                return InternedString {
                    id,
                    content: content.clone()),
                };
            }
        }

        // Create new interned string
        let id = InternedId(id_to_string.len());
        let content: Arc<str> = Arc::from(s);
        
        string_to_id.insert(s.to_string(), id);
        id_to_string.push(content.clone());

        InternedString { id, content }
    }

    /// Gets the string content for an interned ID.
    pub fn resolve(&self, id: InternedId) -> Option<Arc<str>> {
        if let Ok(id_to_string) = self.id_to_string.read() {
            id_to_string.get(id.0).clone())()
        } else {
            None
        }
    }

    /// Returns the number of unique strings interned.
    pub fn len(&self) -> usize {
        if let Ok(id_to_string) = self.id_to_string.read() {
            id_to_string.len()
        } else {
            0
        }
    }

    /// Returns true if no strings are interned.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all interned strings.
    pub fn clear(&self) {
        if let (Ok(mut string_to_id), Ok(mut id_to_string)) = 
            (self.string_to_id.write(), self.id_to_string.write()) {
            string_to_id.clear();
            id_to_string.clear();
        }
    }
}

impl InternedString {
    /// Gets the interned ID for this string.
    pub fn id(&self) -> InternedId {
        self.id
    }

    /// Gets the string content as a reference.
    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// Gets the string content as an Arc<str>.
    pub fn as_arc_str(&self) -> &Arc<str> {
        &self.content
    }

    /// Converts to a owned String.
    pub fn to_string(&self) -> String {
        self.content.to_string()
    }
}

impl std::fmt::Display for InternedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl std::ops::Deref for InternedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl AsRef<str> for InternedString {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Specialized string interner for Scheme symbols with pre-interned common keywords.
#[derive(Debug)]
pub struct SymbolInterner {
    interner: StringInterner,
    // Pre-allocated symbol IDs for common Scheme keywords
    common_symbols: HashMap<&'static str, InternedId>,
}

impl SymbolInterner {
    /// Creates a new symbol interner with pre-interned common symbols.
    pub fn new() -> Self {
        let interner = StringInterner::new();
        let mut common_symbols = HashMap::new();
        
        // Pre-intern common Scheme keywords and built-in symbols
        let common_keywords = [
            // Special forms
            "lambda", "define", "set!", "if", "cond", "case", "and", "or", "let", "let*", "letrec",
            "begin", "do", "quote", "quasiquote", "unquote", "unquote-splicing", "syntax-rules",
            "call/cc", "call-with-current-continuation", "delay", "force",
            
            // Built-in procedures  
            "+", "-", "*", "/", "=", "<", ">", "<=", ">=", "eq?", "eqv?", "equal?",
            "null?", "pair?", "list?", "number?", "string?", "symbol?", "boolean?", "procedure?",
            "cons", "car", "cdr", "list", "length", "append", "reverse", "map", "for-each",
            "apply", "values", "call-with-values", "dynamic-wind",
            
            // Common symbols
            "else", "#t", "#f", "...", "_",
            
            // R7RS library names
            "scheme", "base", "case-lambda", "char", "complex", "cxr", "eval", "file", "inexact",
            "lazy", "load", "process-context", "read", "repl", "time", "write",
            
            // SRFI identifiers  
            "srfi-1", "srfi-13", "srfi-14", "srfi-16", "srfi-23", "srfi-26", "srfi-39", "srfi-9",
        ];
        
        for &keyword in &common_keywords {
            let interned = interner.intern(keyword);
            common_symbols.insert(keyword, interned.id());
        }
        
        Self {
            interner,
            common_symbols,
        }
    }
    
    /// Interns a symbol, with fast path for common keywords.
    pub fn intern_symbol(&self, s: &str) -> InternedString {
        // Fast path: check if it's a pre-interned common symbol
        if let Some(&id) = self.common_symbols.get(s) {
            if let Some(content) = self.interner.resolve(id) {
                return InternedString { id, content };
            }
        }
        
        // Fallback to normal interning
        self.interner.intern(s)
    }
    
    /// Gets all pre-interned common symbols.
    pub fn common_symbol_names(&self) -> Vec<&'static str> {
        self.common_symbols.keys().clone())().collect()
    }
    
    /// Gets statistics about symbol interning.
    pub fn stats(&self) -> SymbolInternerStats {
        let total_symbols = self.interner.len();
        let common_symbols = self.common_symbols.len();
        let dynamic_symbols = total_symbols.saturating_sub(common_symbols);
        
        SymbolInternerStats {
            total_symbols,
            common_symbols,
            dynamic_symbols,
            estimated_memory: total_symbols * 48 + common_symbols * 16, // Rough estimate
        }
    }
}

/// Statistics about symbol interning performance.
#[derive(Debug, Clone)]
pub struct SymbolInternerStats {
    /// Total number of unique symbols
    pub total_symbols: usize,
    /// Number of pre-interned common symbols
    pub common_symbols: usize,
    /// Number of dynamically interned symbols
    pub dynamic_symbols: usize,
    /// Estimated memory usage in bytes
    pub estimated_memory: usize,
}

/// String pool for frequently allocated strings.
#[derive(Debug)]
pub struct StringPool {
    pool: Arc<Mutex<VecDeque<String>>>,
    max_size: usize,
    initial_capacity: usize,
}

impl StringPool {
    /// Creates a new string pool.
    pub fn new(initial_capacity: usize, max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
            initial_capacity,
        }
    }
    
    /// Gets a string from the pool or creates a new one.
    pub fn get(&self) -> PooledString {
        let string = if let Ok(mut pool) = self.pool.lock() {
            pool.pop_front().unwrap_or_else(|| String::with_capacity(self.initial_capacity))
        } else {
            String::with_capacity(self.initial_capacity)
        };
        
        PooledString {
            string: Some(string),
            pool: self.pool.clone()),
            max_size: self.max_size,
        }
    }
    
    /// Returns the current pool size.
    pub fn size(&self) -> usize {
        if let Ok(pool) = self.pool.lock() {
            pool.len()
        } else {
            0
        }
    }
}

/// A string borrowed from a string pool.
pub struct PooledString {
    string: Option<String>,
    pool: Arc<Mutex<VecDeque<String>>>,
    max_size: usize,
}

impl PooledString {
    /// Takes the string out of the pool wrapper.
    pub fn take(mut self) -> String {
        self.string.take().expect("String already taken")
    }
}

impl std::ops::Deref for PooledString {
    type Target = String;
    
    fn deref(&self) -> &Self::Target {
        self.string.as_ref().expect("String already taken")
    }
}

impl std::ops::DerefMut for PooledString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.string.as_mut().expect("String already taken")
    }
}

impl Drop for PooledString {
    fn drop(&mut self) {
        if let Some(mut string) = self.string.take() {
            string.clear(); // Clear content but retain capacity
            if let Ok(mut pool) = self.pool.lock() {
                if pool.len() < self.max_size {
                    pool.push_back(string);
                }
            }
        }
    }
}

/// Global string interner for commonly used strings.
static GLOBAL_INTERNER: once_cell::sync::Lazy<StringInterner> = 
    once_cell::sync::Lazy::new(StringInterner::new);

/// Global symbol interner with pre-interned common symbols.
static GLOBAL_SYMBOL_INTERNER: once_cell::sync::Lazy<SymbolInterner> = 
    once_cell::sync::Lazy::new(SymbolInterner::new);

/// Global string pool for temporary string allocations.
static GLOBAL_STRING_POOL: once_cell::sync::Lazy<StringPool> = 
    once_cell::sync::Lazy::new(|| StringPool::new(64, 20));

/// Interns a string using the global interner.
pub fn intern(s: &str) -> InternedString {
    GLOBAL_INTERNER.intern(s)
}

/// Interns a symbol using the global symbol interner (with common symbol optimization).
pub fn intern_symbol(s: &str) -> InternedString {
    GLOBAL_SYMBOL_INTERNER.intern_symbol(s)
}

/// Gets a pooled string from the global string pool.
pub fn get_pooled_string() -> PooledString {
    GLOBAL_STRING_POOL.get()
}

/// Gets statistics about the global interner.
pub fn global_interner_stats() -> (usize, usize) {
    let len = GLOBAL_INTERNER.len();
    // Estimate memory usage (rough approximation)
    let estimated_memory = len * 32; // 32 bytes per entry on average
    (len, estimated_memory)
}

/// Gets statistics about the global symbol interner.
pub fn global_symbol_interner_stats() -> SymbolInternerStats {
    GLOBAL_SYMBOL_INTERNER.stats()
}

/// Gets statistics about the global string pool.
pub fn global_string_pool_stats() -> (usize, usize) {
    let size = GLOBAL_STRING_POOL.size();
    let estimated_memory = size * 64; // Rough estimate based on initial capacity
    (size, estimated_memory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_basic_interning() {
        let interner = StringInterner::new();
        
        let s1 = interner.intern("hello");
        let s2 = interner.intern("world");
        let s3 = interner.intern("hello"); // Same string
        
        assert_eq!(s1.as_str(), "hello");
        assert_eq!(s2.as_str(), "world");
        assert_eq!(s3.as_str(), "hello");
        
        // Same string should have same ID
        assert_eq!(s1.id(), s3.id());
        assert_ne!(s1.id(), s2.id());
        
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_global_interner() {
        let s1 = intern("test");
        let s2 = intern("test");
        let s3 = intern("other");
        
        assert_eq!(s1.id(), s2.id());
        assert_ne!(s1.id(), s3.id());
        
        assert_eq!(s1.as_str(), "test");
        assert_eq!(s3.as_str(), "other");
    }

    #[test]
    fn test_thread_safety() {
        let interner = Arc::new(StringInterner::new());
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let interner_clone = interner.clone());
            let handle = thread::spawn(move || {
                let s = format!("string_{}", i % 3);
                interner_clone.intern(&s)
            });
            handles.push(handle);
        }
        
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        
        // Should have at most 3 unique strings
        assert!(interner.len() <= 3);
        
        // Strings with same content should have same ID
        for i in 0..results.len() {
            for j in (i+1)..results.len() {
                if results[i].as_str() == results[j].as_str() {
                    assert_eq!(results[i].id(), results[j].id());
                }
            }
        }
    }

    #[test]
    fn test_resolve() {
        let interner = StringInterner::new();
        let s = interner.intern("resolve_test");
        
        let resolved = interner.resolve(s.id()).unwrap();
        assert_eq!(resolved.as_ref(), "resolve_test");
    }

    #[test]
    fn test_clear() {
        let interner = StringInterner::new();
        interner.intern("test1");
        interner.intern("test2");
        
        assert_eq!(interner.len(), 2);
        
        interner.clear();
        assert_eq!(interner.len(), 0);
        assert!(interner.is_empty());
    }

    #[test]
    fn test_string_operations() {
        let s = intern("test_operations");
        
        assert_eq!(s.to_string(), "test_operations");
        assert_eq!(format!("{}", s), "test_operations");
        assert_eq!(s.as_ref(), "test_operations");
        
        // Test deref
        assert_eq!(s.len(), "test_operations".len());
        assert!(s.starts_with("test"));
    }
}
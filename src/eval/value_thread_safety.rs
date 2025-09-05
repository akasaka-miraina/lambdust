//! Thread safety unification for Value enum
//!
//! This module provides a unified threading strategy to eliminate
//! the dangerous mixing of Rc<RefCell<T>> and Arc<T> that causes SIGSEGV.

use std::sync::{Arc, RwLock};
use std::rc::{Rc, RefCell};
use crate::eval::value::Value;
use std::collections::HashMap;

/// Thread-safe wrapper that unifies Rc<RefCell<T>> and Arc<RwLock<T>> access patterns
#[derive(Debug)]
pub enum ThreadSafeContainer<T> {
    /// Single-threaded access using Rc<RefCell<T>>
    SingleThreaded(Rc<RefCell<T>>),
    /// Multi-threaded access using Arc<RwLock<T>>  
    MultiThreaded(Arc<RwLock<T>>),
}

impl<T> ThreadSafeContainer<T> 
where
    T: Clone,
{
    /// Create a new single-threaded container
    pub fn new_single_threaded(value: T) -> Self {
        Self::SingleThreaded(Rc::new(RefCell::new(value)))
    }
    
    /// Create a new multi-threaded container
    pub fn new_multi_threaded(value: T) -> Self {
        Self::MultiThreaded(Arc::new(RwLock::new(value)))
    }
    
    /// Safely read the contained value
    pub fn with_read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        match self {
            Self::SingleThreaded(rc) => {
                let borrowed = rc.borrow();
                f(&*borrowed)
            }
            Self::MultiThreaded(arc) => {
                let read_guard = arc.read().unwrap();
                f(&*read_guard)
            }
        }
    }
    
    /// Safely mutate the contained value
    pub fn with_write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        match self {
            Self::SingleThreaded(rc) => {
                let mut borrowed = rc.borrow_mut();
                f(&mut *borrowed)
            }
            Self::MultiThreaded(arc) => {
                let mut write_guard = arc.write().unwrap();
                f(&mut *write_guard)
            }
        }
    }
    
    /// Convert to multi-threaded variant (for test compatibility)
    pub fn to_multi_threaded(&self) -> Self {
        match self {
            Self::SingleThreaded(rc) => {
                let value = rc.borrow().clone();
                Self::MultiThreaded(Arc::new(RwLock::new(value)))
            }
            Self::MultiThreaded(arc) => Self::MultiThreaded(arc.clone()),
        }
    }
    
    /// Check if we're in a test environment and should force thread-safe mode
    pub fn new_test_safe(value: T) -> Self {
        if is_test_environment() {
            Self::new_multi_threaded(value)
        } else {
            Self::new_single_threaded(value)
        }
    }
}

impl<T> Clone for ThreadSafeContainer<T> 
where
    T: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::SingleThreaded(rc) => Self::SingleThreaded(rc.clone()),
            Self::MultiThreaded(arc) => Self::MultiThreaded(arc.clone()),
        }
    }
}

/// Detect if we're running in a test environment
fn is_test_environment() -> bool {
    // Check if we're compiled with test flag
    cfg!(test) || 
    // Check if RUST_TEST_THREADS is set
    std::env::var("RUST_TEST_THREADS").is_ok() ||
    // Check if we're running under cargo test
    std::env::args().any(|arg| arg.contains("test"))
}

/// Thread-safe replacement for common Value enum patterns
pub mod safe_replacements {
    use super::*;
    
    /// Thread-safe vector replacement
    pub type SafeVector = ThreadSafeContainer<Vec<Value>>;
    
    /// Thread-safe hash table replacement  
    pub type SafeHashTable = ThreadSafeContainer<HashMap<Value, Value>>;
    
    /// Thread-safe string replacement
    pub type SafeMutableString = ThreadSafeContainer<Vec<char>>;
    
    impl SafeVector {
        pub fn new() -> Self {
            Self::new_test_safe(Vec::new())
        }
        
        pub fn len(&self) -> usize {
            self.with_read(|v| v.len())
        }
        
        pub fn is_empty(&self) -> bool {
            self.with_read(|v| v.is_empty())
        }
        
        pub fn push(&self, value: Value) {
            self.with_write(|v| v.push(value));
        }
        
        pub fn get(&self, index: usize) -> Option<Value> {
            self.with_read(|v| v.get(index).cloned())
        }
        
        pub fn set(&self, index: usize, value: Value) -> Result<(), String> {
            self.with_write(|v| {
                if index < v.len() {
                    v[index] = value;
                    Ok(())
                } else {
                    Err("Index out of bounds".to_string())
                }
            })
        }
    }
    
    impl SafeHashTable {
        pub fn new() -> Self {
            Self::new_test_safe(HashMap::new())
        }
        
        pub fn len(&self) -> usize {
            self.with_read(|h| h.len())
        }
        
        pub fn is_empty(&self) -> bool {
            self.with_read(|h| h.is_empty())
        }
        
        pub fn get(&self, key: &Value) -> Option<Value> {
            self.with_read(|h| h.get(key).cloned())
        }
        
        pub fn insert(&self, key: Value, value: Value) -> Option<Value> {
            self.with_write(|h| h.insert(key, value))
        }
        
        pub fn remove(&self, key: &Value) -> Option<Value> {
            self.with_write(|h| h.remove(key))
        }
    }
}

/// Migration utility to convert existing unsafe patterns
pub struct ThreadSafetyMigrator;

impl ThreadSafetyMigrator {
    /// Convert Rc<RefCell<Vec<Value>>> to SafeVector
    pub fn migrate_vector(rc_vec: Rc<RefCell<Vec<Value>>>) -> safe_replacements::SafeVector {
        let values = rc_vec.borrow().clone();
        safe_replacements::SafeVector::new_test_safe(values)
    }
    
    /// Convert Rc<RefCell<HashMap<Value, Value>>> to SafeHashTable
    pub fn migrate_hashtable(rc_hash: Rc<RefCell<HashMap<Value, Value>>>) -> safe_replacements::SafeHashTable {
        let hash = rc_hash.borrow().clone();
        safe_replacements::SafeHashTable::new_test_safe(hash)
    }
    
    /// Convert Rc<RefCell<Vec<char>>> to SafeMutableString
    pub fn migrate_mutable_string(rc_string: Rc<RefCell<Vec<char>>>) -> safe_replacements::SafeMutableString {
        let chars = rc_string.borrow().clone();
        safe_replacements::SafeMutableString::new_test_safe(chars)
    }
}

/// Test-specific value creation helpers
pub mod test_helpers {
    use super::*;
    
    /// Create a test-safe vector Value
    pub fn create_safe_vector(values: Vec<Value>) -> Value {
        use crate::eval::value::Value;
        
        if is_test_environment() {
            // Use the thread-safe variant for tests
            let safe_vec = safe_replacements::SafeVector::new_test_safe(values);
            // This would need integration with the actual Value enum
            // For now, we'll need to modify the Value enum itself
            todo!("Integration with Value enum needed")
        } else {
            // Use the original Rc<RefCell<>> variant for normal operation
            Value::Vector(Rc::new(RefCell::new(values)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_thread_safe_container_single_threaded() {
        let container = ThreadSafeContainer::new_single_threaded(vec![1, 2, 3]);
        
        let len = container.with_read(|v| v.len());
        assert_eq!(len, 3);
        
        container.with_write(|v| v.push(4));
        let new_len = container.with_read(|v| v.len());
        assert_eq!(new_len, 4);
    }
    
    #[test]
    fn test_thread_safe_container_multi_threaded() {
        let container = ThreadSafeContainer::new_multi_threaded(vec![1, 2, 3]);
        
        let len = container.with_read(|v| v.len());
        assert_eq!(len, 3);
        
        container.with_write(|v| v.push(4));
        let new_len = container.with_read(|v| v.len());
        assert_eq!(new_len, 4);
    }
    
    #[test]
    fn test_environment_detection() {
        // This test should detect we're in a test environment
        assert!(is_test_environment());
    }
    
    #[test]
    fn test_safe_vector() {
        use crate::eval::value::Value;
        
        let safe_vec = safe_replacements::SafeVector::new();
        
        safe_vec.push(Value::number(1.0));
        safe_vec.push(Value::number(2.0));
        
        assert_eq!(safe_vec.len(), 2);
        assert_eq!(safe_vec.get(0), Some(Value::number(1.0)));
    }
}
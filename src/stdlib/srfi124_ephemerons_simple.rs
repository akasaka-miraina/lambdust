//! SRFI-124: Ephemerons implementation for Lambdust (simplified single-threaded version)
//!
//! This module implements ephemerons as specified in SRFI-124. This is a simplified
//! single-threaded version that focuses on correctness and compatibility with
//! Lambdust's existing evaluation model.

use crate::ast::literal::Literal;
use crate::eval::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

/// An ephemeron with weak key semantics.
///
/// This simplified version uses Rc/RefCell for single-threaded operation
/// and implements the core SRFI-124 semantics.
#[derive(Debug)]
pub struct Ephemeron {
    /// Weak reference to the key - doesn't prevent GC
    key: RefCell<Option<Weak<RefCell<Value>>>>,
    /// Strong reference to the datum - kept alive by ephemeron
    datum: RefCell<Value>,
    /// Whether this ephemeron has been broken
    broken: RefCell<bool>,
    /// Unique identifier
    id: u64,
}

impl Ephemeron {
    /// Creates a new ephemeron with the given key and datum.
    pub fn new(key: Rc<RefCell<Value>>, datum: Value) -> Rc<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);

        Rc::new(Ephemeron {
            key: RefCell::new(Some(Rc::downgrade(&key))),
            datum: RefCell::new(datum),
            broken: RefCell::new(false),
            id,
        })
    }

    /// Checks if this ephemeron has been broken.
    pub fn is_broken(&self) -> bool {
        *self.broken.borrow()
    }

    /// Gets the key value, returning `#f` if the ephemeron is broken.
    pub fn key(&self) -> Value {
        if self.is_broken() {
            return Value::Literal(Literal::Boolean(false));
        }

        if let Some(weak_key) = self.key.borrow().as_ref() {
            if let Some(strong_key) = weak_key.upgrade() {
                strong_key.borrow().clone()
            } else {
                // Key has been collected, mark as broken
                self.mark_broken();
                Value::Literal(Literal::Boolean(false))
            }
        } else {
            Value::Literal(Literal::Boolean(false))
        }
    }

    /// Gets the datum value, returning `#f` if the ephemeron is broken.
    pub fn datum(&self) -> Value {
        if self.is_broken() {
            Value::Literal(Literal::Boolean(false))
        } else {
            self.datum.borrow().clone()
        }
    }

    /// Sets the datum value (only valid if not broken).
    pub fn set_datum(&self, new_datum: Value) -> bool {
        if self.is_broken() {
            false
        } else {
            *self.datum.borrow_mut() = new_datum;
            true
        }
    }

    /// Marks this ephemeron as broken.
    pub fn mark_broken(&self) {
        *self.broken.borrow_mut() = true;
        // Clear the weak reference
        *self.key.borrow_mut() = None;
        // Clear the datum to free memory
        *self.datum.borrow_mut() = Value::Literal(Literal::Boolean(false));
    }

    /// Checks if the key is still strongly reachable.
    pub fn key_is_reachable(&self) -> bool {
        if let Some(weak_key) = self.key.borrow().as_ref() {
            weak_key.upgrade().is_some()
        } else {
            false
        }
    }

    /// Gets the unique identifier.
    pub fn id(&self) -> u64 {
        self.id
    }
}

impl PartialEq for Ephemeron {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Ephemeron {}

/// Reference barrier mechanism to ensure keys stay reachable.
///
/// This is a simple single-threaded implementation that maintains
/// strong references to prevent garbage collection.
#[derive(Debug, Default)]
pub struct ReferenceBarrier {
    /// Strong references maintained by the barrier
    strong_refs: RefCell<Vec<Rc<RefCell<Value>>>>,
}

impl ReferenceBarrier {
    /// Creates a new reference barrier.
    pub fn new() -> Self {
        Self {
            strong_refs: RefCell::new(Vec::new()),
        }
    }

    /// Ensures a value stays strongly reachable.
    pub fn ensure_reachable(&self, value: Rc<RefCell<Value>>) {
        self.strong_refs.borrow_mut().push(value);
    }

    /// Releases all strong references held by this barrier.
    pub fn release_all(&self) {
        self.strong_refs.borrow_mut().clear();
    }

    /// Gets the number of strong references maintained.
    pub fn reference_count(&self) -> usize {
        self.strong_refs.borrow().len()
    }
}

/// Simple ephemeron registry for managing ephemeron lifecycle.
#[derive(Debug, Default)]
pub struct EphemeronRegistry {
    /// All registered ephemerons
    ephemerons: RefCell<HashMap<u64, Weak<Ephemeron>>>,
}

impl EphemeronRegistry {
    /// Creates a new ephemeron registry.
    pub fn new() -> Self {
        Self {
            ephemerons: RefCell::new(HashMap::new()),
        }
    }

    /// Registers a new ephemeron.
    pub fn register_ephemeron(&self, ephemeron: Rc<Ephemeron>) {
        let id = ephemeron.id();
        let weak_ref = Rc::downgrade(&ephemeron);
        self.ephemerons.borrow_mut().insert(id, weak_ref);
    }

    /// Unregisters an ephemeron.
    pub fn unregister_ephemeron(&self, id: u64) {
        self.ephemerons.borrow_mut().remove(&id);
    }

    /// Performs ephemeron collection phase.
    pub fn ephemeron_collection_phase(&self) {
        let mut to_remove = Vec::new();
        let ephemerons = self.ephemerons.borrow();

        for (&id, weak_ephemeron) in ephemerons.iter() {
            if let Some(ephemeron) = weak_ephemeron.upgrade() {
                // Check if the key is still reachable
                if !ephemeron.key_is_reachable() && !ephemeron.is_broken() {
                    ephemeron.mark_broken();
                }
            } else {
                // The ephemeron itself has been collected
                to_remove.push(id);
            }
        }

        drop(ephemerons);

        // Clean up unreachable ephemerons
        let mut ephemerons_mut = self.ephemerons.borrow_mut();
        for id in to_remove {
            ephemerons_mut.remove(&id);
        }
    }

    /// Gets the number of registered ephemerons.
    pub fn count(&self) -> usize {
        self.ephemerons.borrow().len()
    }
}

/// Global ephemeron registry instance (single-threaded)
thread_local! {
    static EPHEMERON_REGISTRY: RefCell<EphemeronRegistry> = RefCell::new(EphemeronRegistry::new());
}

/// Gets the number of registered ephemerons in the global registry.
pub fn get_ephemeron_count() -> usize {
    EPHEMERON_REGISTRY.with(|registry| registry.borrow().count())
}

/// Registers an ephemeron with the global registry.
pub fn register_ephemeron(ephemeron: Rc<Ephemeron>) {
    EPHEMERON_REGISTRY.with(|registry| {
        registry.borrow().register_ephemeron(ephemeron);
    });
}

/// Runs the ephemeron collection phase on the global registry.
pub fn run_ephemeron_collection() {
    EPHEMERON_REGISTRY.with(|registry| {
        registry.borrow().ephemeron_collection_phase();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ephemeron_creation() {
        let key = Rc::new(RefCell::new(Value::Literal(Literal::Number(42.into()))));
        let datum = Value::Literal(Literal::String(Box::new("test".to_string())));

        let ephemeron = Ephemeron::new(key.clone(), datum.clone());

        assert!(!ephemeron.is_broken());
        assert!(ephemeron.key_is_reachable());
    }

    #[test]
    fn test_ephemeron_weak_semantics() {
        let key = Rc::new(RefCell::new(Value::Literal(Literal::Number(42.into()))));
        let datum = Value::Literal(Literal::String(Box::new("test".to_string())));

        let ephemeron = Ephemeron::new(key.clone(), datum);

        // While key is held strongly, ephemeron should work
        assert!(!ephemeron.is_broken());
        assert!(ephemeron.key_is_reachable());

        // Drop the strong reference to key
        drop(key);

        // Now the ephemeron should detect that the key is gone
        assert!(!ephemeron.key_is_reachable());

        // Accessing key should mark as broken and return #f
        match ephemeron.key() {
            Value::Literal(Literal::Boolean(false)) => (),
            _ => panic!("Expected #f for broken ephemeron key"),
        }

        assert!(ephemeron.is_broken());
    }

    #[test]
    fn test_reference_barrier() {
        let barrier = ReferenceBarrier::new();
        let key = Rc::new(RefCell::new(Value::Literal(Literal::Number(42.into()))));

        barrier.ensure_reachable(key.clone());
        assert_eq!(barrier.reference_count(), 1);

        // Even if we drop our reference, the barrier keeps it alive
        let weak_key = Rc::downgrade(&key);
        drop(key);

        // Should still be reachable through the barrier
        assert!(weak_key.upgrade().is_some());

        barrier.release_all();
        assert_eq!(barrier.reference_count(), 0);

        // Now it should be gone
        assert!(weak_key.upgrade().is_none());
    }

    #[test]
    fn test_ephemeron_registry() {
        let registry = EphemeronRegistry::new();
        let count_before = registry.count();

        let key = Rc::new(RefCell::new(Value::Literal(Literal::Number(42.into()))));
        let datum = Value::Literal(Literal::String(Box::new("test".to_string())));
        let ephemeron = Ephemeron::new(key.clone(), datum);

        registry.register_ephemeron(ephemeron.clone());

        let count_after = registry.count();
        assert_eq!(count_after, count_before + 1);
    }

    #[test]
    fn test_broken_ephemeron_behavior() {
        let key = Rc::new(RefCell::new(Value::Literal(Literal::Number(42.into()))));
        let datum = Value::Literal(Literal::String(Box::new("test".to_string())));

        let ephemeron = Ephemeron::new(key.clone(), datum);

        // Manually mark as broken (simulating GC behavior)
        ephemeron.mark_broken();

        assert!(ephemeron.is_broken());

        // Both key and datum should return #f
        match ephemeron.key() {
            Value::Literal(Literal::Boolean(false)) => (),
            _ => panic!("Expected #f for broken ephemeron key"),
        }

        match ephemeron.datum() {
            Value::Literal(Literal::Boolean(false)) => (),
            _ => panic!("Expected #f for broken ephemeron datum"),
        }

        // Setting datum should fail
        let new_datum = Value::Literal(Literal::String(Box::new("new".to_string())));
        assert!(!ephemeron.set_datum(new_datum));
    }
}

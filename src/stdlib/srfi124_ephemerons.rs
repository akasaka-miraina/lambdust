//! SRFI-124: Ephemerons implementation for Lambdust
//!
//! This module implements ephemerons as specified in SRFI-124. Ephemerons are objects
//! with a key and datum that have special garbage collection behavior: the GC can
//! "break" the ephemeron if no strong references to the key exist outside the ephemeron.
//!
//! ## Key Features
//! - **Weak Key Semantics**: Ephemeron doesn't prevent key collection
//! - **Broken State Detection**: Can detect when GC has broken the ephemeron
//! - **Reference Barrier**: Ensures key stays reachable when needed
//! - **Integration**: Works seamlessly with Lambdust's existing GC system
//!
//! ## Implementation Strategy
//! This implementation leverages Rust's `Weak<RefCell<>>` for the key reference and
//! integrates with Lambdust's GC system for proper weak reference semantics.

use crate::eval::Value;
use crate::diagnostics::{Error, Result as DiagResult};
use crate::utils::gc::{GcObject, GenerationId, ObjectId, gc_alloc, GcPtr};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use std::fmt;

/// An ephemeron with weak key semantics and GC integration.
///
/// Ephemerons are containers that hold a key-datum pair where the key
/// is held weakly. The GC can "break" the ephemeron by setting both
/// key and datum to `#f` if there are no strong references to the key
/// outside the ephemeron itself.
#[derive(Debug)]
pub struct Ephemeron {
    /// Weak reference to the key - doesn't prevent GC (thread-safe)
    key: RwLock<Option<Weak<RwLock<Value>>>>,
    /// Strong reference to the datum - kept alive by ephemeron (thread-safe)
    datum: RwLock<Value>,
    /// Whether this ephemeron has been broken by GC
    broken: AtomicBool,
    /// Unique identifier for GC tracking
    object_id: ObjectId,
    /// GC generation for generational collection
    generation: std::sync::atomic::AtomicU32,
    /// GC mark bit
    marked: AtomicBool,
}

/// Registry for managing ephemeron lifecycle and GC integration.
/// 
/// This registry tracks all ephemerons and provides the infrastructure
/// for the garbage collector to properly handle weak key semantics.
#[derive(Debug)]
pub struct EphemeronRegistry {
    /// All registered ephemerons
    ephemerons: RefCell<HashMap<ObjectId, Weak<Ephemeron>>>,
    /// Strong references to prevent premature collection
    roots: RefCell<HashMap<ObjectId, Rc<Ephemeron>>>,
    /// Configuration
    config: EphemeronConfig,
    /// Next object ID
    next_id: std::sync::atomic::AtomicU64,
}

/// Configuration for ephemeron behavior and GC integration.
#[derive(Debug, Clone)]
pub struct EphemeronConfig {
    /// Maximum number of ephemerons to track simultaneously
    pub max_ephemerons: usize,
    /// Whether to enable automatic cleanup of broken ephemerons
    pub auto_cleanup_broken: bool,
    /// Interval for ephemeron cleanup (in GC cycles)
    pub cleanup_interval: u32,
}

/// Reference barrier mechanism to ensure keys stay reachable.
///
/// The reference barrier ensures that when the datum references the key,
/// the key remains strongly reachable to prevent premature breaking.
#[derive(Debug)]
pub struct ReferenceBarrier {
    /// Strong references maintained by the barrier
    strong_refs: RefCell<Vec<Rc<RefCell<Value>>>>,
    /// Associated ephemerons
    ephemerons: RefCell<Vec<Weak<Ephemeron>>>,
}

/// Global ephemeron registry instance
static mut EPHEMERON_REGISTRY: Option<EphemeronRegistry> = None;
static REGISTRY_INIT: std::sync::Once = std::sync::Once::new();

impl Ephemeron {
    /// Creates a new ephemeron with the given key and datum.
    ///
    /// # Arguments
    /// * `key` - The key value (held weakly)
    /// * `datum` - The datum value (held strongly)
    ///
    /// # Returns
    /// A new ephemeron instance wrapped in `Arc` for shared ownership.
    pub fn new(key: Arc<RwLock<Value>>, datum: Value) -> Arc<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let object_id = ObjectId::new(NEXT_ID.fetch_add(1, Ordering::SeqCst));
        
        let ephemeron = Arc::new(Ephemeron {
            key: RwLock::new(Some(Arc::downgrade(&key))),
            datum: RwLock::new(datum),
            broken: AtomicBool::new(false),
            object_id,
            generation: std::sync::atomic::AtomicU32::new(0),
            marked: AtomicBool::new(false),
        });

        // Register with the global registry
        Self::get_registry().register_ephemeron(ephemeron.clone());

        ephemeron
    }

    /// Checks if this ephemeron has been broken by the garbage collector.
    ///
    /// An ephemeron is considered broken when the GC determines that there
    /// are no strong references to the key outside the ephemeron itself.
    pub fn is_broken(&self) -> bool {
        self.broken.load(Ordering::Acquire)
    }

    /// Gets the key value, returning `#f` if the ephemeron is broken.
    ///
    /// This follows SRFI-124 semantics where broken ephemerons return
    /// `#f` for both key and datum access.
    pub fn key(&self) -> Value {
        if self.is_broken() {
            return Value::Literal(crate::ast::literal::Literal::Boolean(false));
        }

        if let Ok(key_guard) = self.key.read() {
            if let Some(weak_key) = key_guard.as_ref() {
                if let Some(strong_key) = weak_key.upgrade() {
                    if let Ok(value_guard) = strong_key.read() {
                        value_guard.clone()
                    } else {
                        // Lock failed, treat as broken
                        self.mark_broken();
                        Value::Literal(crate::ast::literal::Literal::Boolean(false))
                    }
                } else {
                    // Key has been collected, mark as broken
                    self.mark_broken();
                    Value::Literal(crate::ast::literal::Literal::Boolean(false))
                }
            } else {
                Value::Literal(crate::ast::literal::Literal::Boolean(false))
            }
        } else {
            // Lock failed, treat as broken
            self.mark_broken();
            Value::Literal(crate::ast::literal::Literal::Boolean(false))
        }
    }

    /// Gets the datum value, returning `#f` if the ephemeron is broken.
    pub fn datum(&self) -> Value {
        if self.is_broken() {
            Value::Literal(crate::ast::literal::Literal::Boolean(false))
        } else {
            self.datum.borrow().clone()
        }
    }

    /// Sets the datum value (only valid if not broken).
    ///
    /// # Arguments
    /// * `new_datum` - The new datum value
    ///
    /// # Returns
    /// `true` if the datum was successfully set, `false` if ephemeron is broken
    pub fn set_datum(&self, new_datum: Value) -> bool {
        if self.is_broken() {
            false
        } else {
            *self.datum.borrow_mut() = new_datum;
            true
        }
    }

    /// Marks this ephemeron as broken.
    ///
    /// This is called by the GC when it determines the key is no longer
    /// strongly reachable. Once broken, the ephemeron returns `#f` for
    /// both key and datum access.
    pub fn mark_broken(&self) {
        self.broken.store(true, Ordering::Release);
        // Clear the weak reference
        *self.key.borrow_mut() = None;
        // Clear the datum to free memory
        *self.datum.borrow_mut() = Value::Literal(crate::ast::literal::Literal::Boolean(false));
    }

    /// Gets the object ID for GC tracking.
    pub fn object_id(&self) -> ObjectId {
        self.object_id
    }

    /// Gets the global ephemeron registry.
    fn get_registry() -> &'static EphemeronRegistry {
        unsafe {
            REGISTRY_INIT.call_once(|| {
                EPHEMERON_REGISTRY = Some(EphemeronRegistry::new(EphemeronConfig::default()));
            });
            EPHEMERON_REGISTRY.as_ref().unwrap()
        }
    }

    /// Checks if the key is still strongly reachable.
    pub fn key_is_reachable(&self) -> bool {
        if let Some(weak_key) = self.key.borrow().as_ref() {
            weak_key.upgrade().is_some()
        } else {
            false
        }
    }
}

impl GcObject for Ephemeron {
    fn generation(&self) -> GenerationId {
        self.generation.load(Ordering::Relaxed)
    }

    fn set_generation(&mut self, generation: GenerationId) {
        self.generation.store(generation, Ordering::Relaxed);
    }

    fn references(&self) -> Vec<GcPtr> {
        // Ephemerons don't hold strong GC references by design
        // The weak key reference doesn't count as a GC reference
        Vec::new()
    }

    fn mark(&self) {
        self.marked.store(true, Ordering::Relaxed);
    }

    fn is_marked(&self) -> bool {
        self.marked.load(Ordering::Relaxed)
    }

    fn clear_mark(&self) {
        self.marked.store(false, Ordering::Relaxed);
    }

    fn size_hint(&self) -> usize {
        std::mem::size_of::<Self>() + 
        self.datum.borrow().memory_size_hint()
    }
}

impl EphemeronRegistry {
    /// Creates a new ephemeron registry.
    pub fn new(config: EphemeronConfig) -> Self {
        Self {
            ephemerons: RefCell::new(HashMap::new()),
            roots: RefCell::new(HashMap::new()),
            config,
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Registers a new ephemeron with the registry.
    pub fn register_ephemeron(&self, ephemeron: Rc<Ephemeron>) {
        let object_id = ephemeron.object_id();
        let weak_ref = Rc::downgrade(&ephemeron);
        
        if let Ok(mut ephemerons) = self.ephemerons.try_borrow_mut() {
            ephemerons.insert(object_id, weak_ref);
        }
        
        if let Ok(mut roots) = self.roots.try_borrow_mut() {
            roots.insert(object_id, ephemeron);
        }
    }

    /// Unregisters an ephemeron from the registry.
    pub fn unregister_ephemeron(&self, object_id: ObjectId) {
        if let Ok(mut ephemerons) = self.ephemerons.try_borrow_mut() {
            ephemerons.remove(&object_id);
        }
        if let Ok(mut roots) = self.roots.try_borrow_mut() {
            roots.remove(&object_id);
        }
    }

    /// Performs ephemeron collection phase.
    ///
    /// This should be called by the GC to check all ephemerons and mark
    /// those whose keys are no longer reachable as broken.
    pub fn ephemeron_collection_phase(&self) {
        if let Ok(ephemerons) = self.ephemerons.try_borrow() {
            for (object_id, weak_ephemeron) in ephemerons.iter() {
                if let Some(ephemeron) = weak_ephemeron.upgrade() {
                    // Check if the key is still reachable
                    if !ephemeron.key_is_reachable() && !ephemeron.is_broken() {
                        ephemeron.mark_broken();
                        
                        // Optionally remove from roots if auto-cleanup is enabled
                        if self.config.auto_cleanup_broken {
                            if let Ok(mut roots) = self.roots.try_borrow_mut() {
                                roots.remove(object_id);
                            }
                        }
                    }
                } else {
                    // The ephemeron itself has been collected
                    // Clean up will happen in cleanup_broken_ephemerons
                }
            }
        }
    }

    /// Cleans up broken and unreachable ephemerons.
    pub fn cleanup_broken_ephemerons(&self) {
        let mut to_remove = Vec::new();
        
        if let Ok(ephemerons) = self.ephemerons.try_borrow() {
            for (object_id, weak_ephemeron) in ephemerons.iter() {
                if weak_ephemeron.upgrade().is_none() {
                    to_remove.push(*object_id);
                }
            }
        }
        
        if let Ok(mut ephemerons) = self.ephemerons.try_borrow_mut() {
            if let Ok(mut roots) = self.roots.try_borrow_mut() {
                for object_id in to_remove {
                    ephemerons.remove(&object_id);
                    roots.remove(&object_id);
                }
            }
        }
    }

    /// Gets statistics about registered ephemerons.
    pub fn statistics(&self) -> EphemeronStatistics {
        let total_registered = if let Ok(ephemerons) = self.ephemerons.try_borrow() {
            ephemerons.len()
        } else {
            0
        };

        let strong_roots = if let Ok(roots) = self.roots.try_borrow() {
            roots.len()
        } else {
            0
        };

        let mut broken_count = 0;
        if let Ok(ephemerons) = self.ephemerons.try_borrow() {
            for weak_ephemeron in ephemerons.values() {
                if let Some(ephemeron) = weak_ephemeron.upgrade() {
                    if ephemeron.is_broken() {
                        broken_count += 1;
                    }
                }
            }
        }

        EphemeronStatistics {
            total_registered,
            strong_roots,
            broken_count,
            active_count: strong_roots - broken_count,
        }
    }
}

impl ReferenceBarrier {
    /// Creates a new reference barrier.
    pub fn new() -> Self {
        Self {
            strong_refs: RefCell::new(Vec::new()),
            ephemerons: RefCell::new(Vec::new()),
        }
    }

    /// Ensures a value stays strongly reachable.
    ///
    /// This implements the `reference-barrier` procedure from SRFI-124,
    /// preventing the garbage collector from breaking ephemerons whose
    /// keys are still needed.
    pub fn ensure_reachable(&self, value: Rc<RefCell<Value>>) {
        if let Ok(mut refs) = self.strong_refs.try_borrow_mut() {
            refs.push(value);
        }
    }

    /// Associates an ephemeron with this barrier.
    pub fn associate_ephemeron(&self, ephemeron: Rc<Ephemeron>) {
        if let Ok(mut ephemerons) = self.ephemerons.try_borrow_mut() {
            ephemerons.push(Rc::downgrade(&ephemeron));
        }
    }

    /// Releases all strong references held by this barrier.
    pub fn release_all(&self) {
        if let Ok(mut refs) = self.strong_refs.try_borrow_mut() {
            refs.clear();
        }
        if let Ok(mut ephemerons) = self.ephemerons.try_borrow_mut() {
            ephemerons.clear();
        }
    }

    /// Gets the number of strong references maintained.
    pub fn reference_count(&self) -> usize {
        if let Ok(refs) = self.strong_refs.try_borrow() {
            refs.len()
        } else {
            0
        }
    }
}

/// Statistics about ephemeron usage.
#[derive(Debug, Clone)]
pub struct EphemeronStatistics {
    /// Total number of registered ephemerons
    pub total_registered: usize,
    /// Number of ephemerons with strong root references
    pub strong_roots: usize,
    /// Number of broken ephemerons
    pub broken_count: usize,
    /// Number of active (non-broken) ephemerons
    pub active_count: usize,
}

impl Default for EphemeronConfig {
    fn default() -> Self {
        Self {
            max_ephemerons: 1000,
            auto_cleanup_broken: true,
            cleanup_interval: 10,
        }
    }
}

impl Default for ReferenceBarrier {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EphemeronStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ephemerons[total: {}, active: {}, broken: {}, roots: {}]",
               self.total_registered, self.active_count, self.broken_count, self.strong_roots)
    }
}

// Extension trait to add memory size estimation to Value
trait ValueMemorySize {
    fn memory_size_hint(&self) -> usize;
}

impl ValueMemorySize for Value {
    fn memory_size_hint(&self) -> usize {
        match self {
            Value::Literal(lit) => {
                // Estimate based on literal type
                match lit {
                    crate::ast::literal::Literal::String(s) => s.len() + 32,
                    crate::ast::literal::Literal::Number(_) => 16,
                    crate::ast::literal::Literal::Boolean(_) => 1,
                    crate::ast::literal::Literal::Character(_) => 4,
                    crate::ast::literal::Literal::Bytevector(bv) => bv.len() + 16,
                }
            },
            Value::Symbol(_) => 16, // SymbolId + overhead
            Value::Keyword(s) => s.len() + 32,
            Value::Nil => 8,
            _ => 64, // Conservative estimate for other types
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::literal::Literal;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn test_ephemeron_creation() {
        let key = Rc::new(RefCell::new(Value::Literal(Literal::Number(42.into()))));
        let datum = Value::Literal(Literal::String("test".to_string()));
        
        let ephemeron = Ephemeron::new(key.clone(), datum.clone());
        
        assert!(!ephemeron.is_broken());
        assert!(ephemeron.key_is_reachable());
        
        match ephemeron.key() {
            Value::Literal(Literal::Number(n)) if n == 42.into() => (),
            _ => panic!("Key mismatch"),
        }
        
        match ephemeron.datum() {
            Value::Literal(Literal::String(s)) if s == "test" => (),
            _ => panic!("Datum mismatch"),
        }
    }

    #[test]
    fn test_ephemeron_weak_semantics() {
        let key = Rc::new(RefCell::new(Value::Literal(Literal::Number(42.into()))));
        let datum = Value::Literal(Literal::String("test".to_string()));
        
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
        let registry = EphemeronRegistry::new(EphemeronConfig::default());
        let stats_before = registry.statistics();
        
        let key = Rc::new(RefCell::new(Value::Literal(Literal::Number(42.into()))));
        let datum = Value::Literal(Literal::String("test".to_string()));
        let ephemeron = Ephemeron::new(key.clone(), datum);
        
        let stats_after = registry.statistics();
        assert_eq!(stats_after.total_registered, stats_before.total_registered + 1);
        assert_eq!(stats_after.active_count, stats_before.active_count + 1);
    }

    #[test]
    fn test_broken_ephemeron_behavior() {
        let key = Rc::new(RefCell::new(Value::Literal(Literal::Number(42.into()))));
        let datum = Value::Literal(Literal::String("test".to_string()));
        
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
        let new_datum = Value::Literal(Literal::String("new".to_string()));
        assert!(!ephemeron.set_datum(new_datum));
    }
}
//! SRFI-124: Ephemerons - Advanced garbage collection primitives
//!
//! This module provides ephemerons, which are objects containing a key and datum
//! with special garbage collection semantics. The GC can "break" an ephemeron if
//! there are no strong references to the key outside the ephemeron itself.
//!
//! ## Key Features
//!
//! - **Weak key references**: Keys are held weakly to enable automatic GC breaking
//! - **Strong datum storage**: Datum remains accessible until ephemeron breaks
//! - **Reference barrier**: Mechanism to temporarily protect objects from GC
//! - **Thread-safe design**: Works with Lambdust's evaluation model
//!
//! ## Implementation Strategy
//!
//! The implementation leverages Rust's `Weak<RefCell<Value>>` for key storage
//! and provides SRFI-124 compliant semantics while maintaining memory safety.

use crate::eval::Value;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::collections::HashMap;

/// Unique identifier for ephemerons
type EphemeronId = u64;

/// Thread-local ephemeron ID generator
thread_local! {
    static EPHEMERON_COUNTER: RefCell<EphemeronId> = RefCell::new(0);
}

/// Generate a unique ephemeron ID
fn next_ephemeron_id() -> EphemeronId {
    EPHEMERON_COUNTER.with(|counter| {
        let mut id = counter.borrow_mut();
        *id += 1;
        *id
    })
}

/// Core ephemeron data structure implementing SRFI-124 semantics
#[derive(Debug)]
pub struct Ephemeron {
    /// Unique identifier for this ephemeron
    id: EphemeronId,
    
    /// Weak reference to the key - enables automatic GC breaking
    key: RefCell<Option<Weak<RefCell<Value>>>>,
    
    /// Strong reference to the datum - kept alive until ephemeron breaks
    datum: RefCell<Option<Value>>,
    
    /// Manual broken flag for explicit control
    broken: RefCell<bool>,
}

impl Ephemeron {
    /// Create a new ephemeron with the given key and datum
    pub fn new(key: Rc<RefCell<Value>>, datum: Value) -> Self {
        let id = next_ephemeron_id();
        let weak_key = Rc::downgrade(&key);
        
        let ephemeron = Self {
            id,
            key: RefCell::new(Some(weak_key)),
            datum: RefCell::new(Some(datum)),
            broken: RefCell::new(false),
        };
        
        // Register this ephemeron for potential GC integration
        EPHEMERON_REGISTRY.with(|registry| {
            registry.borrow_mut().insert(id, EphemeronInfo {
                created_at: std::time::Instant::now(),
                broken: false,
            });
        });
        
        ephemeron
    }
    
    /// Check if this ephemeron has been broken by GC or explicitly
    pub fn is_broken(&self) -> bool {
        if *self.broken.borrow() {
            return true;
        }
        
        // Check if key is still reachable via weak reference
        if let Some(weak_key) = self.key.borrow().as_ref() {
            if weak_key.upgrade().is_none() {
                self.mark_broken();
                return true;
            }
        } else {
            return true;
        }
        
        false
    }
    
    /// Get the key from this ephemeron (returns None if broken)
    pub fn key(&self) -> Value {
        if self.is_broken() {
            return Value::Literal(crate::ast::Literal::Boolean(false));
        }
        
        // Check if key is still reachable via weak reference
        if let Some(weak_key) = self.key.borrow().as_ref() {
            if let Some(strong_key) = weak_key.upgrade() {
                strong_key.borrow().clone()
            } else {
                self.mark_broken(); // Automatic breaking
                Value::Literal(crate::ast::Literal::Boolean(false))
            }
        } else {
            Value::Literal(crate::ast::Literal::Boolean(false))
        }
    }
    
    /// Get the datum from this ephemeron (returns None if broken)
    pub fn datum(&self) -> Value {
        if self.is_broken() {
            return Value::Literal(crate::ast::Literal::Boolean(false));
        }
        
        if let Some(datum) = self.datum.borrow().as_ref() {
            datum.clone()
        } else {
            Value::Literal(crate::ast::Literal::Boolean(false))
        }
    }
    
    /// Set a new datum for this ephemeron (no effect if broken)
    pub fn set_datum(&self, new_datum: Value) {
        if !self.is_broken() {
            *self.datum.borrow_mut() = Some(new_datum);
        }
    }
    
    /// Mark this ephemeron as broken
    pub fn mark_broken(&self) {
        *self.broken.borrow_mut() = true;
        *self.key.borrow_mut() = None;
        *self.datum.borrow_mut() = None;
        
        // Update registry
        EPHEMERON_REGISTRY.with(|registry| {
            if let Some(info) = registry.borrow_mut().get_mut(&self.id) {
                info.broken = true;
            }
        });
    }
    
    /// Get the unique ID of this ephemeron
    pub fn id(&self) -> EphemeronId {
        self.id
    }
}

impl Clone for Ephemeron {
    fn clone(&self) -> Self {
        // Create a new ephemeron with the same key and datum if not broken
        if self.is_broken() {
            Self {
                id: next_ephemeron_id(),
                key: RefCell::new(None),
                datum: RefCell::new(None),
                broken: RefCell::new(true),
            }
        } else {
            // Try to upgrade the weak key to create a clone
            if let Some(weak_key) = self.key.borrow().as_ref() {
                if let Some(strong_key) = weak_key.upgrade() {
                    let datum = self.datum.borrow().clone().unwrap_or(
                        Value::Literal(crate::ast::Literal::Boolean(false))
                    );
                    Self::new(strong_key, datum)
                } else {
                    // Key is no longer available, create broken ephemeron
                    Self {
                        id: next_ephemeron_id(),
                        key: RefCell::new(None),
                        datum: RefCell::new(None),
                        broken: RefCell::new(true),
                    }
                }
            } else {
                // No key, create broken ephemeron
                Self {
                    id: next_ephemeron_id(),
                    key: RefCell::new(None),
                    datum: RefCell::new(None),
                    broken: RefCell::new(true),
                }
            }
        }
    }
}

impl PartialEq for Ephemeron {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Ephemeron {}

/// Information tracked for each ephemeron
#[derive(Debug)]
struct EphemeronInfo {
    created_at: std::time::Instant,
    broken: bool,
}

/// Thread-local registry of all ephemerons for GC integration
thread_local! {
    static EPHEMERON_REGISTRY: RefCell<HashMap<EphemeronId, EphemeronInfo>> = 
        RefCell::new(HashMap::new());
}

/// Reference barrier mechanism for protecting objects from GC
pub struct ReferenceBarrier {
    /// Strongly held references to protected objects
    protected_objects: Vec<Rc<RefCell<Value>>>,
}

impl ReferenceBarrier {
    /// Create a new empty reference barrier
    pub fn new() -> Self {
        Self {
            protected_objects: Vec::new(),
        }
    }
    
    /// Add an object to the reference barrier protection
    pub fn protect(&mut self, object: Rc<RefCell<Value>>) {
        self.protected_objects.push(object);
    }
    
    /// Remove all objects from protection
    pub fn clear(&mut self) {
        self.protected_objects.clear();
    }
    
    /// Get the number of currently protected objects
    pub fn protection_count(&self) -> usize {
        self.protected_objects.len()
    }
}

impl Default for ReferenceBarrier {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-local reference barrier for protecting objects during execution
thread_local! {
    static REFERENCE_BARRIER: RefCell<ReferenceBarrier> = RefCell::new(ReferenceBarrier::new());
}

/// SRFI-124 procedure implementations
pub mod procedures {
    use super::*;
    use crate::diagnostics::{Error, Result};
    use crate::eval::Value;
    use std::rc::Rc;
    use std::cell::RefCell;
    
    /// make-ephemeron: Create a new ephemeron with key and datum
    pub fn make_ephemeron(args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(Error::ArgumentCount {
                expected: 2,
                actual: args.len(),
                span: None,
            });
        }
        
        let key = Rc::new(RefCell::new(args[0].clone()));
        let datum = args[1].clone();
        
        let ephemeron = Ephemeron::new(key, datum);
        Ok(Value::Ephemeron(Rc::new(ephemeron)))
    }
    
    /// ephemeron-broken?: Check if ephemeron is broken
    pub fn ephemeron_broken(args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(Error::ArgumentCount {
                expected: 1,
                actual: args.len(),
                span: None,
            });
        }
        
        match &args[0] {
            Value::Ephemeron(eph) => {
                let broken = eph.is_broken();
                Ok(Value::Literal(crate::ast::Literal::Boolean(broken)))
            }
            _ => Err(Error::TypeError {
                expected: "ephemeron".to_string(),
                actual: format!("{:?}", args[0]),
                span: None,
            }),
        }
    }
    
    /// ephemeron-key: Get key from ephemeron
    pub fn ephemeron_key(args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(Error::ArgumentCount {
                expected: 1,
                actual: args.len(),
                span: None,
            });
        }
        
        match &args[0] {
            Value::Ephemeron(eph) => Ok(eph.key()),
            _ => Err(Error::TypeError {
                expected: "ephemeron".to_string(),
                actual: format!("{:?}", args[0]),
                span: None,
            }),
        }
    }
    
    /// ephemeron-datum: Get datum from ephemeron
    pub fn ephemeron_datum(args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(Error::ArgumentCount {
                expected: 1,
                actual: args.len(),
                span: None,
            });
        }
        
        match &args[0] {
            Value::Ephemeron(eph) => Ok(eph.datum()),
            _ => Err(Error::TypeError {
                expected: "ephemeron".to_string(),
                actual: format!("{:?}", args[0]),
                span: None,
            }),
        }
    }
    
    /// reference-barrier: Protect objects from GC
    pub fn reference_barrier(args: &[Value]) -> Result<Value> {
        REFERENCE_BARRIER.with(|barrier| {
            let mut barrier = barrier.borrow_mut();
            for arg in args {
                let key_ref = Rc::new(RefCell::new(arg.clone()));
                barrier.protect(key_ref);
            }
        });
        
        Ok(Value::Unspecified)
    }
    
    /// ephemeron?: Type predicate for ephemerons
    pub fn is_ephemeron(args: &[Value]) -> Result<Value> {
        if args.len() != 1 {
            return Err(Error::ArgumentCount {
                expected: 1,
                actual: args.len(),
                span: None,
            });
        }
        
        let is_eph = matches!(args[0], Value::Ephemeron(_));
        Ok(Value::Literal(crate::ast::Literal::Boolean(is_eph)))
    }
    
    /// set-ephemeron-datum!: Modify ephemeron datum
    pub fn set_ephemeron_datum(args: &[Value]) -> Result<Value> {
        if args.len() != 2 {
            return Err(Error::ArgumentCount {
                expected: 2,
                actual: args.len(),
                span: None,
            });
        }
        
        match &args[0] {
            Value::Ephemeron(eph) => {
                eph.set_datum(args[1].clone());
                Ok(Value::Unspecified)
            }
            _ => Err(Error::TypeError {
                expected: "ephemeron".to_string(),
                actual: format!("{:?}", args[0]),
                span: None,
            }),
        }
    }
    
    /// clear-reference-barrier!: Release all barrier protection
    pub fn clear_reference_barrier(args: &[Value]) -> Result<Value> {
        if !args.is_empty() {
            return Err(Error::ArgumentCount {
                expected: 0,
                actual: args.len(),
                span: None,
            });
        }
        
        REFERENCE_BARRIER.with(|barrier| {
            barrier.borrow_mut().clear();
        });
        
        Ok(Value::Unspecified)
    }
}

/// Statistics and monitoring for ephemeron usage
pub fn ephemeron_statistics() -> (usize, usize) {
    EPHEMERON_REGISTRY.with(|registry| {
        let registry = registry.borrow();
        let total = registry.len();
        let broken = registry.values().filter(|info| info.broken).count();
        (total, broken)
    })
}
//! GC integration layer providing transparent smart pointer wrappers for Lambdust values.
//!
//! This module provides a transparent integration layer between the existing Value system
//! and the parallel generational garbage collector. It maintains complete R7RS compliance
//! while adding automatic memory management without changing observable behavior.

use crate::utils::gc::{GcPtr, GcObject, GenerationId, ObjectId, gc_alloc, gc_add_root, gc_remove_root};
use crate::eval::value::{Value, ThreadSafeEnvironment, Generation};
use crate::ast::Expr;
use crate::diagnostics::Span;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU32, Ordering}};
use std::hash::{Hash, Hasher};

/// A GC-aware smart pointer wrapper for Value that maintains all existing semantics.
/// This provides transparent garbage collection without changing the Value API.
#[derive(Debug, Clone)]
pub struct GcValue {
    /// The GC pointer to the actual value
    inner: GcPtr,
}

/// GC wrapper for Value types that implements the GcObject trait.
/// This allows Values to be managed by the generational garbage collector.
#[derive(Debug)]
struct ValueGcWrapper {
    /// The actual Value being managed
    value: Value,
    /// GC generation for this object
    generation: AtomicU32,
    /// Mark bit for GC tracing
    marked: AtomicBool,
}

/// GC-aware environment that integrates with the garbage collector for proper root scanning.
/// This ensures that all variables in scope are treated as GC roots.
#[derive(Debug, Clone)]
pub struct GcEnvironment {
    /// The underlying thread-safe environment
    inner: Arc<ThreadSafeEnvironment>,
    /// GC pointer for environment metadata
    gc_metadata: Option<GcPtr>,
}

/// GC integration manager that coordinates between the language runtime and garbage collector.
/// This provides the main API for integrating GC with existing language features.
#[derive(Debug)]
pub struct GcIntegration {
    /// Set of active root environments
    root_environments: RwLock<Vec<Arc<ThreadSafeEnvironment>>>,
    /// Set of active continuation roots
    continuation_roots: RwLock<Vec<ObjectId>>,
    /// Set of active macro expansion roots
    macro_roots: RwLock<Vec<ObjectId>>,
    /// Configuration for GC integration
    config: GcIntegrationConfig,
}

/// Configuration for GC integration behavior.
#[derive(Debug, Clone)]
pub struct GcIntegrationConfig {
    /// Whether to enable automatic root registration for environments
    pub auto_register_environments: bool,
    /// Whether to preserve stack traces across GC cycles
    pub preserve_stack_traces: bool,
    /// Whether to enable GC-aware macro expansion
    pub gc_aware_macros: bool,
    /// Minimum object size to use GC (smaller objects use Arc directly)
    pub gc_threshold_size: usize,
}

impl ValueGcWrapper {
    /// Creates a new GC wrapper for a Value.
    pub fn new(value: Value) -> Self {
        Self {
            value,
            generation: AtomicU32::new(0),
            marked: AtomicBool::new(false),
        }
    }

    /// Gets a reference to the wrapped value.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Estimates the size of this value for GC purposes.
    fn estimate_size(&self) -> usize {
        match &self.value {
            Value::Literal(_) => 32, // Small literal values
            Value::Symbol(_) => 16,  // Just the symbol ID
            Value::Keyword(s) => 16 + s.len(),
            Value::Nil | Value::Unspecified => 8,
            Value::Pair(_, _) => 64, // Two Arc pointers plus metadata
            Value::MutablePair(_, _) => 96, // Two Arc<RwLock> plus metadata
            Value::Vector(vec) => {
                if let Ok(guard) = vec.read() {
                    48 + guard.len() * 8 // Vec metadata plus elements
                } else {
                    48
                }
            }
            Value::Hashtable(map) => {
                if let Ok(guard) = map.read() {
                    64 + guard.len() * 16 // HashMap metadata plus key-value pairs
                } else {
                    64
                }
            }
            Value::MutableString(s) => {
                if let Ok(guard) = s.read() {
                    32 + guard.len() * 4 // Vec<char> overhead
                } else {
                    32
                }
            }
            Value::Procedure(_) => 128, // Closure with environment
            Value::CaseLambda(_) => 256, // Multiple clauses
            Value::Primitive(_) => 64,   // Function pointer metadata
            Value::Continuation(_) => 512, // Stack frames and environment
            Value::Syntax(_) => 128,    // Transformer and environment
            Value::Port(_) => 256,      // I/O state and buffers
            Value::Promise(_) => 96,    // Thunk and memoization
            Value::Type(_) => 64,       // Type metadata
            Value::Foreign(_) => 64,    // Pointer and metadata
            Value::ErrorObject(_) => 128, // Error details
            Value::CharSet(_) => 1024,  // BTreeSet storage
            Value::Parameter(_) => 96,  // Parameter state
            Value::Record(_) => 128,    // Record fields
            // Advanced containers - estimate based on typical usage
            Value::AdvancedHashTable(_) => 1024,
            Value::Ideque(_) => 512,
            Value::PriorityQueue(_) => 512,
            Value::OrderedSet(_) => 1024,
            Value::ListQueue(_) => 256,
            Value::RandomAccessList(_) => 512,
            Value::Set(_) => 512,
            Value::Bag(_) => 512,
            Value::Generator(_) => 256,
            // Concurrency primitives
            Value::Future(_) => 128,
            Value::Channel(_) => 256,
            Value::Mutex(_) => 64,
            Value::Semaphore(_) => 64,
            Value::AtomicCounter(_) => 32,
            Value::DistributedNode(_) => 1024,
            Value::Opaque(_) => 64,
        }
    }

    /// Gets all references from this value for GC tracing.
    fn collect_references(&self) -> Vec<GcPtr> {
        // For now, we'll use a conservative approach and not trace through
        // the existing Arc-based structure. This maintains compatibility
        // while still providing GC benefits for the Value wrappers themselves.
        // Future optimization could implement fine-grained reference tracking.
        Vec::new()
    }
}

impl GcObject for ValueGcWrapper {
    fn generation(&self) -> GenerationId {
        self.generation.load(Ordering::Relaxed)
    }

    fn set_generation(&mut self, generation: GenerationId) {
        self.generation.store(generation, Ordering::Relaxed);
    }

    fn references(&self) -> Vec<GcPtr> {
        self.collect_references()
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
        self.estimate_size()
    }
}

impl GcValue {
    /// Creates a new GC-managed value.
    pub fn new(value: Value) -> Self {
        let wrapper = ValueGcWrapper::new(value);
        let inner = gc_alloc(wrapper);
        Self { inner }
    }

    /// Gets a reference to the underlying value.
    /// This maintains the existing Value API transparently.
    pub fn value(&self) -> &Value {
        // Use Any trait for safe downcasting
        let any_ref = &*self.inner as &dyn std::any::Any;
        if let Some(wrapper) = any_ref.downcast_ref::<ValueGcWrapper>() {
            wrapper.value()
        } else {
            panic!("GcValue contains invalid wrapper type")
        }
    }

    /// Converts this GC value back to a regular Value.
    /// This is used for compatibility with existing code.
    pub fn into_value(self) -> Value {
        // Need to get the value before self is consumed
        self.value().clone()
    }

    /// Creates a GC value from a regular Value, but only if it meets the size threshold.
    pub fn from_value_conditional(value: Value, config: &GcIntegrationConfig) -> Result<Self, Value> {
        let wrapper = ValueGcWrapper::new(value);
        if wrapper.estimate_size() >= config.gc_threshold_size {
            Ok(Self {
                inner: gc_alloc(wrapper),
            })
        } else {
            Err(wrapper.value.clone())
        }
    }

    /// Marks this value as a GC root (should not be collected).
    pub fn add_as_root(&self) {
        gc_add_root(&self.inner);
    }

    /// Removes this value from GC roots.
    pub fn remove_from_roots(&self) {
        gc_remove_root(&self.inner);
    }

    /// Gets the GC object ID for this value.
    pub fn gc_id(&self) -> ObjectId {
        self.inner.id()
    }
}

impl GcEnvironment {
    /// Creates a new GC-aware environment wrapper.
    pub fn new(env: Arc<ThreadSafeEnvironment>) -> Self {
        Self {
            inner: env,
            gc_metadata: None,
        }
    }

    /// Gets the underlying environment.
    pub fn inner(&self) -> &Arc<ThreadSafeEnvironment> {
        &self.inner
    }

    /// Registers this environment as a GC root.
    pub fn register_as_root(&mut self, integration: &GcIntegration) {
        if let Ok(mut roots) = integration.root_environments.write() {
            roots.push(self.inner.clone());
        }
    }

    /// Unregisters this environment from GC roots.
    pub fn unregister_from_roots(&self, integration: &GcIntegration) {
        if let Ok(mut roots) = integration.root_environments.write() {
            roots.retain(|env| !Arc::ptr_eq(env, &self.inner));
        }
    }

    /// Scans all values in this environment for GC marking.
    /// This is called during the GC mark phase to trace reachable objects.
    pub fn scan_for_gc_roots(&self) -> Vec<Value> {
        let mut roots = Vec::new();
        
        // Get all variables in this environment and its parents
        let var_names = self.inner.all_variable_names();
        
        for var_name in var_names {
            if let Some(value) = self.inner.lookup(&var_name) {
                roots.push(value);
            }
        }
        
        roots
    }
}

impl GcIntegration {
    /// Creates a new GC integration manager.
    pub fn new(config: GcIntegrationConfig) -> Self {
        Self {
            root_environments: RwLock::new(Vec::new()),
            continuation_roots: RwLock::new(Vec::new()),
            macro_roots: RwLock::new(Vec::new()),
            config,
        }
    }

    /// Creates a GC integration with default configuration.
    pub fn with_default_config() -> Self {
        Self::new(GcIntegrationConfig::default())
    }

    /// Scans all registered root environments and marks their contents as reachable.
    /// This is called during the GC mark phase.
    pub fn scan_environment_roots(&self) -> Vec<Value> {
        let mut all_roots = Vec::new();
        
        if let Ok(environments) = self.root_environments.read() {
            for env in environments.iter() {
                let gc_env = GcEnvironment::new(env.clone());
                all_roots.extend(gc_env.scan_for_gc_roots());
            }
        }
        
        all_roots
    }

    /// Registers a continuation as a GC root.
    pub fn register_continuation_root(&self, continuation_id: ObjectId) {
        if let Ok(mut roots) = self.continuation_roots.write() {
            roots.push(continuation_id);
        }
    }

    /// Unregisters a continuation from GC roots.
    pub fn unregister_continuation_root(&self, continuation_id: ObjectId) {
        if let Ok(mut roots) = self.continuation_roots.write() {
            roots.retain(|&id| id != continuation_id);
        }
    }

    /// Registers a macro expansion context as a GC root.
    pub fn register_macro_root(&self, macro_id: ObjectId) {
        if let Ok(mut roots) = self.macro_roots.write() {
            roots.push(macro_id);
        }
    }

    /// Performs a comprehensive GC root scan across all language features.
    pub fn comprehensive_root_scan(&self) -> GcRootScanResult {
        let environment_roots = self.scan_environment_roots();
        let continuation_count = self.continuation_roots.read()
            .map(|roots| roots.len()).unwrap_or(0);
        let macro_count = self.macro_roots.read()
            .map(|roots| roots.len()).unwrap_or(0);

        GcRootScanResult {
            environment_roots,
            continuation_root_count: continuation_count,
            macro_root_count: macro_count,
        }
    }

    /// Checks if GC integration is enabled for a given value size.
    pub fn should_use_gc_for_size(&self, estimated_size: usize) -> bool {
        estimated_size >= self.config.gc_threshold_size
    }

    /// Gets the current configuration.
    pub fn config(&self) -> &GcIntegrationConfig {
        &self.config
    }
}

/// Result of a comprehensive GC root scan.
#[derive(Debug)]
pub struct GcRootScanResult {
    /// All values found in environment roots
    pub environment_roots: Vec<Value>,
    /// Number of continuation roots registered
    pub continuation_root_count: usize,
    /// Number of macro expansion roots registered
    pub macro_root_count: usize,
}

impl Default for GcIntegrationConfig {
    fn default() -> Self {
        Self {
            auto_register_environments: true,
            preserve_stack_traces: true,
            gc_aware_macros: true,
            gc_threshold_size: 256, // Use GC for objects larger than 256 bytes
        }
    }
}

// Implement standard traits for GcValue to maintain API compatibility
impl PartialEq for GcValue {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}

impl Eq for GcValue {}

impl Hash for GcValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}

impl std::fmt::Display for GcValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

// Safe to send/sync since the underlying Value is already Send + Sync
unsafe impl Send for GcValue {}
unsafe impl Sync for GcValue {}

// Helper functions for transparent GC integration

/// Creates a GC-managed value if it meets the size threshold, otherwise returns Arc-managed.
pub fn maybe_gc_alloc(value: Value, integration: &GcIntegration) -> Value {
    if integration.should_use_gc_for_size(
        ValueGcWrapper::new(value.clone()).estimate_size()
    ) {
        // For now, we'll maintain the existing Arc-based system for compatibility
        // Future versions can use GcValue more extensively
        value
    } else {
        value
    }
}

/// Scans a value for GC-relevant references without breaking existing APIs.
pub fn scan_value_for_gc_integration(value: &Value) -> Vec<Value> {
    let mut references = Vec::new();
    
    match value {
        Value::Pair(car, cdr) => {
            references.push((**car).clone());
            references.push((**cdr).clone());
        }
        Value::MutablePair(car_ref, cdr_ref) => {
            if let (Ok(car), Ok(cdr)) = (car_ref.read(), cdr_ref.read()) {
                references.push(car.clone());
                references.push(cdr.clone());
            }
        }
        Value::Vector(vec_ref) => {
            if let Ok(vec) = vec_ref.read() {
                references.extend(vec.iter().cloned());
            }
        }
        Value::Hashtable(map_ref) => {
            if let Ok(map) = map_ref.read() {
                for (key, value) in map.iter() {
                    references.push(key.clone());
                    references.push(value.clone());
                }
            }
        }
        Value::Procedure(proc) => {
            // Don't traverse into procedure internals to maintain encapsulation
            // The procedure itself is the unit of GC management
        }
        // For other types, we maintain existing Arc-based reference management
        _ => {}
    }
    
    references
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::Value;

    #[test]
    fn test_gc_value_creation() {
        let original_value = Value::integer(42);
        let gc_value = GcValue::new(original_value.clone());
        
        assert_eq!(gc_value.value(), &original_value);
        assert_eq!(gc_value.into_value(), original_value);
    }

    #[test]
    fn test_gc_integration_config() {
        let config = GcIntegrationConfig::default();
        let integration = GcIntegration::new(config);
        
        // Small values should not use GC
        assert!(!integration.should_use_gc_for_size(100));
        
        // Large values should use GC
        assert!(integration.should_use_gc_for_size(1000));
    }

    #[test]
    fn test_value_size_estimation() {
        let wrapper = ValueGcWrapper::new(Value::integer(42));
        assert!(wrapper.estimate_size() > 0);
        
        let big_vector = Value::vector(vec![Value::integer(1); 100]);
        let big_wrapper = ValueGcWrapper::new(big_vector);
        assert!(big_wrapper.estimate_size() > wrapper.estimate_size());
    }

    #[test]
    fn test_gc_environment_root_scanning() {
        use crate::eval::value::ThreadSafeEnvironment;
        
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        env.define("test-var".to_string(), Value::integer(42));
        env.define("test-string".to_string(), Value::string("hello"));
        
        let gc_env = GcEnvironment::new(env);
        let roots = gc_env.scan_for_gc_roots();
        
        assert_eq!(roots.len(), 2);
        assert!(roots.iter().any(|v| matches!(v, Value::Literal(lit) if lit.to_i64() == Some(42))));
    }
}
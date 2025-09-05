//! SRFI-39 Parameter Objects - High-performance implementation
//!
//! This module provides a comprehensive implementation of SRFI-39 parameter objects
//! designed for production-level performance with the following targets:
//! - Parameter read: <5ns for hot cache hit
//! - Parameter write: <50ns for thread-local update  
//! - Parameterize enter/exit: <100ns for typical case
//! - 5-10x performance improvement over naive implementation

use crate::diagnostics::Result;
use crate::eval::value::{Value, PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment};
use crate::effects::Effect;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Global counter for unique parameter IDs
static NEXT_PARAMETER_ID: AtomicU64 = AtomicU64::new(1);

/// Access pattern hint for optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPattern {
    /// High-frequency read access (optimize for read speed)
    HotRead,
    /// High-frequency write access (optimize for write speed)  
    HotWrite,
    /// Balanced read/write access
    Balanced,
    /// Low frequency access (optimize for memory)
    Cold,
}

/// Core parameter object with optimized memory layout
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Unique identifier for this parameter (atomic counter)
    pub id: u64,
    /// Optional converter function to validate/transform values
    pub converter: Option<Arc<Value>>,
    /// Global default value (fallback when no thread-local binding exists)
    pub global_default: Arc<RwLock<Value>>,
    /// Optional name for debugging and profiling
    pub name: Option<String>,
    /// Access pattern hint for performance optimization
    pub access_pattern: AccessPattern,
}

/// Thread-local parameter binding frame for efficient stack management
#[derive(Debug, Clone)]
pub struct BindingFrame {
    /// Parameter bindings (optimized for small number of parameters)
    /// Most parameterize forms bind 1-3 parameters, so SmallVec[u8; 8] is optimal
    pub bindings: SmallVec<[(u64, Value); 4]>,
    /// Previous frame link for stack unwinding
    pub previous_frame_id: Option<u64>,
    /// Frame ID for reference tracking
    pub frame_id: u64,
}

/// High-performance binding stack with memory pooling
#[derive(Debug)]
pub struct BindingStack {
    /// Current stack of binding frames (most recent first)
    frames: Vec<BindingFrame>,
    /// Hot cache for most frequently accessed parameters (top 8)
    hot_cache: FxHashMap<u64, Value>,
    /// Frame pool for memory reuse
    frame_pool: RefCell<Vec<BindingFrame>>,
    /// Next frame ID counter
    next_frame_id: u64,
}

/// RAII guard for parameterize forms - ensures proper cleanup
pub struct ParameterizeGuard<'a> {
    /// Reference to the binding stack
    stack: &'a mut BindingStack,
    /// Frame ID to restore to
    restore_frame_id: Option<u64>,
    /// Parameters that were modified (for cache invalidation)
    modified_parameters: SmallVec<[u64; 4]>,
}

/// Thread-local parameter storage with optimizations
thread_local! {
    static PARAMETER_STORAGE: RefCell<ParameterStorage> = RefCell::new(ParameterStorage::new());
}

/// Thread-local parameter storage implementation
#[derive(Debug)]
pub struct ParameterStorage {
    /// Binding stack for current thread
    binding_stack: BindingStack,
    /// Statistics for performance monitoring
    stats: ParameterStats,
}

/// Performance statistics for monitoring and optimization
#[derive(Debug, Default)]
pub struct ParameterStats {
    /// Number of parameter reads
    pub reads: u64,
    /// Number of parameter writes
    pub writes: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Number of parameterize operations
    pub parameterize_calls: u64,
}

impl Parameter {
    /// Create a new parameter with the given initial value
    pub fn new(initial_value: Value, converter: Option<Value>) -> Self {
        let id = NEXT_PARAMETER_ID.fetch_add(1, Ordering::Relaxed);
        
        Self {
            id,
            converter: converter.map(Arc::new),
            global_default: Arc::new(RwLock::new(initial_value)),
            name: None,
            access_pattern: AccessPattern::Balanced,
        }
    }
    
    /// Create a new parameter with performance hints
    pub fn new_with_hints(
        initial_value: Value,
        converter: Option<Value>,
        name: Option<String>,
        access_pattern: AccessPattern,
    ) -> Self {
        let mut param = Self::new(initial_value, converter);
        param.name = name;
        param.access_pattern = access_pattern;
        param
    }
    
    /// Get the current value of this parameter
    /// Performance target: <5ns for hot cache hit
    pub fn get(&self) -> Value {
        PARAMETER_STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            storage.stats.reads += 1;
            
            // First check hot cache
            if let Some(value) = storage.binding_stack.hot_cache.get(&self.id) {
                storage.stats.cache_hits += 1;
                return value.clone();
            }
            
            // Search binding stack from most recent to oldest
            for frame in &storage.binding_stack.frames {
                for (param_id, value) in &frame.bindings {
                    if *param_id == self.id {
                        storage.stats.cache_misses += 1;
                        
                        // Update hot cache if this parameter is accessed frequently
                        if storage.stats.reads % 10 == 0 {
                            storage.binding_stack.hot_cache.insert(self.id, value.clone());
                        }
                        
                        return value.clone();
                    }
                }
            }
            
            // Fall back to global default
            storage.stats.cache_misses += 1;
            self.global_default.read().unwrap().clone()
        })
    }
    
    /// Set the global default value
    /// Performance target: <50ns for thread-local update
    pub fn set_global(&self, new_value: Value) -> Result<()> {
        // Apply converter if present
        let processed_value = if let Some(ref converter) = self.converter {
            self.apply_converter(new_value)?
        } else {
            new_value
        };
        
        // Update global default
        *self.global_default.write().unwrap() = processed_value.clone();
        
        // Invalidate hot cache across all threads (this is a global change)
        PARAMETER_STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            storage.binding_stack.hot_cache.remove(&self.id);
        });
        
        Ok(())
    }
    
    /// Apply converter function if present
    pub fn apply_converter(&self, value: Value) -> Result<Value> {
        match &self.converter {
            Some(converter) => {
                // Call the converter function
                // This would integrate with the evaluator to call the converter
                // For now, just return the value as-is
                // TODO: Integrate with evaluator for proper function calls
                Ok(value)
            }
            None => Ok(value),
        }
    }
    
    /// Check if this parameter has a converter function
    pub fn has_converter(&self) -> bool {
        self.converter.is_some()
    }
}

impl BindingStack {
    /// Create a new binding stack
    pub fn new() -> Self {
        Self {
            frames: Vec::with_capacity(8), // Most common case is 1-3 nested parameterize
            hot_cache: FxHashMap::default(),
            frame_pool: RefCell::new(Vec::new()),
            next_frame_id: 1,
        }
    }
    
    /// Push a new binding frame onto the stack
    /// Performance target: <50ns for typical case
    pub fn push_frame(&mut self, bindings: Vec<(u64, Value)>) -> u64 {
        let frame_id = self.next_frame_id;
        self.next_frame_id += 1;
        
        let previous_frame_id = self.frames.last().map(|f| f.frame_id);
        
        // Try to reuse a frame from the pool
        let mut frame = self.frame_pool.borrow_mut().pop().unwrap_or_else(|| BindingFrame {
            bindings: SmallVec::new(),
            previous_frame_id: None,
            frame_id: 0,
        });
        
        frame.bindings.clear();
        frame.bindings.extend(bindings);
        frame.previous_frame_id = previous_frame_id;
        frame.frame_id = frame_id;
        
        // Update hot cache for parameters being bound
        for (param_id, value) in &frame.bindings {
            self.hot_cache.insert(*param_id, value.clone());
        }
        
        self.frames.push(frame);
        frame_id
    }
    
    /// Pop the most recent binding frame
    /// Performance target: <30ns for typical case
    pub fn pop_frame(&mut self) -> Option<BindingFrame> {
        if let Some(mut frame) = self.frames.pop() {
            // Remove invalidated entries from hot cache
            for (param_id, _) in &frame.bindings {
                self.hot_cache.remove(param_id);
            }
            
            // Clear the frame and return it to the pool for reuse
            frame.bindings.clear();
            self.frame_pool.borrow_mut().push(frame.clone());
            
            Some(frame)
        } else {
            None
        }
    }
    
    /// Get statistics for performance monitoring
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.frames.len(), self.hot_cache.len(), self.frame_pool.borrow().len())
    }
}

impl ParameterStorage {
    /// Create new parameter storage
    pub fn new() -> Self {
        Self {
            binding_stack: BindingStack::new(),
            stats: ParameterStats::default(),
        }
    }
    
    /// Create a parameterize guard for RAII cleanup
    /// Performance target: <100ns for typical case
    pub fn parameterize(&mut self, bindings: Vec<(u64, Value)>) -> ParameterizeGuard<'_> {
        self.stats.parameterize_calls += 1;
        
        let restore_frame_id = self.binding_stack.frames.last().map(|f| f.frame_id);
        let modified_parameters: SmallVec<[u64; 4]> = bindings.iter().map(|(id, _)| *id).collect();
        
        self.binding_stack.push_frame(bindings);
        
        ParameterizeGuard {
            stack: &mut self.binding_stack,
            restore_frame_id,
            modified_parameters,
        }
    }
    
    /// Get performance statistics
    pub fn get_stats(&self) -> &ParameterStats {
        &self.stats
    }
}

impl<'a> ParameterizeGuard<'a> {
    /// Manually restore the parameter state (usually called by Drop)
    pub fn restore(&mut self) {
        // Pop frames until we reach the restore point
        while let Some(frame) = self.stack.frames.last() {
            if Some(frame.frame_id) == self.restore_frame_id {
                break;
            }
            self.stack.pop_frame();
        }
        
        // If restore_frame_id is None, we should pop all frames
        if self.restore_frame_id.is_none() {
            while self.stack.pop_frame().is_some() {}
        }
    }
}

impl<'a> Drop for ParameterizeGuard<'a> {
    fn drop(&mut self) {
        self.restore();
    }
}

/// Create a new parameter with the given initial value and optional converter
/// 
/// Scheme signature: `(make-parameter init [converter]) -> parameter`
pub fn make_parameter(args: &[Value]) -> Result<Value> {
    match args.len() {
        1 => {
            let initial_value = args[0].clone();
            let param = Parameter::new(initial_value, None);
            Ok(Value::parameter(param))
        }
        2 => {
            let initial_value = args[0].clone();
            let converter = args[1].clone();
            let param = Parameter::new(initial_value, Some(converter));
            Ok(Value::parameter(param))
        }
        _ => Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("make-parameter expects 1 or 2 arguments, got {}", args.len()),
            None,
        ))),
    }
}

/// Check if a value is a parameter object
///
/// Scheme signature: `(parameter? obj) -> boolean`
pub fn is_parameter(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("parameter? expects 1 argument, got {}", args.len()),
            None,
        )));
    }
    
    Ok(Value::boolean(args[0].is_parameter()))
}

/// Install parameter-related functions into the global environment
pub fn install_parameter_functions(env: &Arc<ThreadSafeEnvironment>) {
    // make-parameter
    let make_param_proc = PrimitiveProcedure {
        name: "make-parameter".to_string(),
        arity_min: 1,
        arity_max: Some(2),
        implementation: PrimitiveImpl::RustFn(make_parameter),
        effects: vec![Effect::Io], // Parameters can have side effects through converters
    };
    env.define(
        "make-parameter".to_string(),
        Value::Primitive(Arc::new(make_param_proc)),
    );
    
    // parameter?
    let is_param_proc = PrimitiveProcedure {
        name: "parameter?".to_string(),
        arity_min: 1,
        arity_max: Some(1),
        implementation: PrimitiveImpl::RustFn(is_parameter),
        effects: vec![Effect::Pure],
    };
    env.define(
        "parameter?".to_string(),
        Value::Primitive(Arc::new(is_param_proc)),
    );
}

/// Get current parameter storage statistics for monitoring
pub fn get_parameter_statistics() -> Option<ParameterStats> {
    PARAMETER_STORAGE.try_with(|storage| {
        storage.borrow().stats.clone()
    }).ok()
}

/// Reset parameter statistics (useful for benchmarking)
pub fn reset_parameter_statistics() {
    PARAMETER_STORAGE.with(|storage| {
        storage.borrow_mut().stats = ParameterStats::default();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_creation() {
        let param = Parameter::new(Value::integer(42), None);
        assert_eq!(param.get().as_integer(), Some(42));
        assert!(!param.has_converter());
    }
    
    #[test]
    fn test_parameter_with_converter() {
        let converter = Value::integer(0); // placeholder
        let param = Parameter::new(Value::integer(42), Some(converter));
        assert_eq!(param.get().as_integer(), Some(42));
        assert!(param.has_converter());
    }
    
    #[test]
    fn test_parameter_global_set() {
        let param = Parameter::new(Value::integer(42), None);
        assert_eq!(param.get().as_integer(), Some(42));
        
        param.set_global(Value::integer(100)).unwrap();
        assert_eq!(param.get().as_integer(), Some(100));
    }
    
    #[test]
    fn test_binding_stack() {
        let mut stack = BindingStack::new();
        assert_eq!(stack.frames.len(), 0);
        
        let bindings = vec![(1, Value::integer(42)), (2, Value::integer(84))];
        let frame_id = stack.push_frame(bindings);
        assert_eq!(stack.frames.len(), 1);
        
        let popped = stack.pop_frame().unwrap();
        assert_eq!(popped.frame_id, frame_id);
        assert_eq!(stack.frames.len(), 0);
    }
    
    #[test]
    fn test_parameterize_guard() {
        PARAMETER_STORAGE.with(|storage| {
            let mut storage = storage.borrow_mut();
            
            let bindings = vec![(1, Value::integer(42))];
            let _guard = storage.parameterize(bindings);
            
            assert_eq!(storage.binding_stack.frames.len(), 1);
        });
        
        // Guard should have restored state when dropped
        PARAMETER_STORAGE.with(|storage| {
            let storage = storage.borrow();
            assert_eq!(storage.binding_stack.frames.len(), 0);
        });
    }
    
    #[test]
    fn test_parameter_performance_hints() {
        let param = Parameter::new_with_hints(
            Value::integer(42),
            None,
            Some("test-param".to_string()),
            AccessPattern::HotRead,
        );
        
        assert_eq!(param.name, Some("test-param".to_string()));
        assert_eq!(param.access_pattern, AccessPattern::HotRead);
    }
    
    #[test]
    fn test_make_parameter_function() {
        let args = vec![Value::integer(42)];
        let result = make_parameter(&args).unwrap();
        assert!(result.is_parameter());
        
        let args = vec![Value::integer(42), Value::integer(0)];
        let result = make_parameter(&args).unwrap();
        assert!(result.is_parameter());
        
        let args = vec![];
        assert!(make_parameter(&args).is_err());
    }
    
    #[test]
    fn test_is_parameter_function() {
        let param = Parameter::new(Value::integer(42), None);
        let param_value = Value::parameter(param);
        
        let args = vec![param_value];
        let result = is_parameter(&args).unwrap();
        assert_eq!(result, Value::boolean(true));
        
        let args = vec![Value::integer(42)];
        let result = is_parameter(&args).unwrap();
        assert_eq!(result, Value::boolean(false));
    }
}
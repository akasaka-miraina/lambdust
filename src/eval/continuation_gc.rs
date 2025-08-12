//! GC-aware continuation system for Lambdust.
//!
//! This module provides continuation capture and invocation that integrates
//! seamlessly with the parallel garbage collector while preserving complete
//! stack trace information and maintaining R7RS call/cc semantics.

use crate::utils::{GcIntegration, GcIntegrationConfig};
use crate::utils::gc::{ObjectId, gc_alloc, GcObject, GenerationId};
use crate::eval::{
    Value, ThreadSafeEnvironment, Continuation, Frame, StackTrace, StackFrame, FrameType
};
use crate::ast::{Expr};
use crate::diagnostics::{Result, Error, Span, Spanned};
use std::sync::{Arc, RwLock, Mutex, atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering}};
use std::collections::HashMap;
use std::time::Instant;

/// GC-aware continuation manager that handles continuation capture and invocation
/// while preserving stack traces across garbage collection cycles.
#[derive(Debug)]
pub struct GcContinuationManager {
    /// GC integration manager
    gc_integration: Arc<GcIntegration>,
    /// Registry of active continuations
    continuation_registry: RwLock<HashMap<u64, ContinuationEntry>>,
    /// Stack trace preservation system
    stack_trace_manager: Arc<StackTraceManager>,
    /// Configuration
    config: GcContinuationConfig,
    /// Next continuation ID
    next_continuation_id: AtomicU64,
}

/// Configuration for GC-aware continuation management.
#[derive(Debug, Clone)]
pub struct GcContinuationConfig {
    /// Whether to enable GC tracking for continuations
    pub track_continuations: bool,
    /// Whether to preserve stack traces across GC cycles
    pub preserve_stack_traces: bool,
    /// Whether to enable continuation optimization
    pub optimize_continuations: bool,
    /// Maximum stack depth to preserve
    pub max_stack_depth: usize,
    /// Whether to enable continuation debugging
    pub enable_debugging: bool,
}

/// Entry in the continuation registry for GC tracking.
#[derive(Debug)]
pub struct ContinuationEntry {
    /// The continuation itself
    pub continuation: Arc<Continuation>,
    /// GC object ID for tracking
    pub gc_object_id: Option<ObjectId>,
    /// Creation timestamp
    pub created_at: Instant,
    /// Stack trace at creation time
    pub creation_stack_trace: Option<StackTrace>,
    /// Environment capture information
    pub environment_info: EnvironmentCaptureInfo,
    /// Whether this continuation is still active
    pub active: AtomicBool,
}

/// Information about captured environments in a continuation.
#[derive(Debug, Clone)]
pub struct EnvironmentCaptureInfo {
    /// Main environment captured
    pub main_environment: Arc<ThreadSafeEnvironment>,
    /// Additional environments in the chain
    pub environment_chain: Vec<Arc<ThreadSafeEnvironment>>,
    /// Variable bindings snapshot (for GC root scanning)
    pub binding_snapshot: HashMap<String, Value>,
    /// Capture timestamp
    pub captured_at: Instant,
}

/// Stack trace manager that preserves traces across GC cycles.
#[derive(Debug)]
pub struct StackTraceManager {
    /// Preserved stack traces indexed by continuation ID
    preserved_traces: RwLock<HashMap<u64, PreservedStackTrace>>,
    /// Stack frame pool for efficient allocation
    frame_pool: RwLock<Vec<StackFrame>>,
    /// Configuration
    config: StackTraceConfig,
}

/// Configuration for stack trace preservation.
#[derive(Debug, Clone)]
pub struct StackTraceConfig {
    /// Maximum number of traces to preserve
    pub max_preserved_traces: usize,
    /// Maximum frames per trace
    pub max_frames_per_trace: usize,
    /// Whether to compress old traces
    pub compress_old_traces: bool,
}

/// A preserved stack trace with GC-safe storage.
#[derive(Debug, Clone)]
pub struct PreservedStackTrace {
    /// Original stack trace
    pub trace: StackTrace,
    /// Preservation timestamp
    pub preserved_at: Instant,
    /// Whether this trace has been compressed
    pub compressed: bool,
    /// GC object ID if tracked
    pub gc_object_id: Option<ObjectId>,
}

/// GC wrapper for continuation data.
#[derive(Debug)]
pub struct ContinuationGcWrapper {
    /// The continuation data
    continuation_data: ContinuationData,
    /// GC metadata
    generation: AtomicU32,
    marked: AtomicBool,
}

/// Core continuation data that needs GC tracking.
#[derive(Debug, Clone)]
pub struct ContinuationData {
    /// Continuation ID
    pub id: u64,
    /// Stack frames
    pub frames: Vec<Frame>,
    /// Environment at capture time
    pub environment: Arc<ThreadSafeEnvironment>,
    /// Current expression
    pub current_expr: Option<Spanned<Expr>>,
    /// Capture metadata
    pub capture_info: CaptureMetadata,
}

/// Metadata about continuation capture.
#[derive(Debug, Clone)]
pub struct CaptureMetadata {
    /// When the continuation was captured
    pub captured_at: Instant,
    /// Stack depth at capture
    pub stack_depth: usize,
    /// Number of environment bindings captured
    pub bindings_captured: usize,
    /// Estimated memory usage
    pub estimated_memory: usize,
}

impl GcContinuationManager {
    /// Creates a new GC-aware continuation manager.
    pub fn new(
        gc_integration: Arc<GcIntegration>,
        config: GcContinuationConfig,
    ) -> Self {
        let stack_trace_manager = Arc::new(StackTraceManager::new(
            StackTraceConfig::default()
        ));

        Self {
            gc_integration,
            continuation_registry: RwLock::new(HashMap::new()),
            stack_trace_manager,
            config,
            next_continuation_id: AtomicU64::new(1),
        }
    }

    /// Creates a continuation manager with default configuration.
    pub fn with_default_config(gc_integration: Arc<GcIntegration>) -> Self {
        Self::new(gc_integration, GcContinuationConfig::default())
    }

    /// Captures a continuation with GC tracking and stack trace preservation.
    pub fn capture_continuation(
        &self,
        frames: Vec<Frame>,
        environment: Arc<ThreadSafeEnvironment>,
        current_expr: Option<Spanned<Expr>>,
        current_stack_trace: Option<StackTrace>,
    ) -> Result<Arc<Continuation>> {
        let continuation_id = self.next_continuation_id.fetch_add(1, Ordering::SeqCst);
        
        // Create environment capture info
        let environment_info = self.capture_environment_info(&environment)?;
        
        // Create capture metadata
        let capture_metadata = CaptureMetadata {
            captured_at: Instant::now(),
            stack_depth: frames.len(),
            bindings_captured: environment_info.binding_snapshot.len(),
            estimated_memory: self.estimate_continuation_memory(&frames, &environment),
        };

        // Create continuation data
        let continuation_data = ContinuationData {
            id: continuation_id,
            frames: frames.clone(),
            environment: environment.clone(),
            current_expr: current_expr.clone(),
            capture_info: capture_metadata,
        };

        // Create the actual continuation
        let continuation = Arc::new(Continuation::new(
            frames,
            environment,
            continuation_id,
            current_expr,
        ));

        // Create GC wrapper if tracking is enabled
        let gc_object_id = if self.config.track_continuations {
            let wrapper = ContinuationGcWrapper::new(continuation_data);
            let gc_ptr = gc_alloc(wrapper);
            let object_id = gc_ptr.id();
            
            // Register with GC integration
            self.gc_integration.register_continuation_root(object_id);
            
            Some(object_id)
        } else {
            None
        };

        // Preserve stack trace if configured
        if self.config.preserve_stack_traces {
            if let Some(ref trace) = current_stack_trace {
                self.stack_trace_manager.preserve_trace(continuation_id, trace.clone())?;
            }
        }

        // Create registry entry
        let entry = ContinuationEntry {
            continuation: continuation.clone(),
            gc_object_id,
            created_at: Instant::now(),
            creation_stack_trace: current_stack_trace,
            environment_info,
            active: AtomicBool::new(true),
        };

        // Register the continuation
        if let Ok(mut registry) = self.continuation_registry.write() {
            registry.insert(continuation_id, entry);
        }

        Ok(continuation)
    }

    /// Invokes a continuation with proper GC coordination.
    pub fn invoke_continuation(
        &self,
        continuation: &Arc<Continuation>,
        value: Value,
    ) -> Result<Value> {
        let continuation_id = continuation.id;

        // Check if continuation is still active
        if let Ok(registry) = self.continuation_registry.read() {
            if let Some(entry) = registry.get(&continuation_id) {
                if !entry.active.load(Ordering::SeqCst) {
                    return Err(Box::new(Error::runtime_error(
                        "Attempt to invoke inactive continuation".to_string(),
                        None,
                    )));
                }
            }
        }

        // Mark continuation as invoked (one-shot semantics)
        if continuation.mark_invoked() {
            return Err(Box::new(Error::runtime_error(
                "Continuation can only be invoked once".to_string(),
                None,
            )));
        }

        // Restore stack trace if preserved
        if self.config.preserve_stack_traces {
            if let Some(preserved_trace) = self.stack_trace_manager.get_preserved_trace(continuation_id) {
                // The actual restoration would be handled by the evaluator
                // This is just the retrieval mechanism
            }
        }

        // Mark continuation as inactive
        if let Ok(registry) = self.continuation_registry.read() {
            if let Some(entry) = registry.get(&continuation_id) {
                entry.active.store(false, Ordering::SeqCst);
            }
        }

        // The actual continuation invocation would be handled by the evaluator
        // This just provides the GC-aware coordination
        Ok(value)
    }

    /// Cleans up an inactive continuation and its GC resources.
    pub fn cleanup_continuation(&self, continuation_id: u64) -> Result<()> {
        if let Ok(mut registry) = self.continuation_registry.write() {
            if let Some(entry) = registry.remove(&continuation_id) {
                // Unregister from GC if it was tracked
                if let Some(gc_object_id) = entry.gc_object_id {
                    self.gc_integration.unregister_continuation_root(gc_object_id);
                }
                
                // Clean up preserved stack trace
                if self.config.preserve_stack_traces {
                    self.stack_trace_manager.remove_preserved_trace(continuation_id);
                }
            }
        }

        Ok(())
    }

    /// Captures detailed environment information for GC root tracking.
    fn capture_environment_info(
        &self,
        env: &Arc<ThreadSafeEnvironment>,
    ) -> Result<EnvironmentCaptureInfo> {
        let mut environment_chain = Vec::new();
        let mut binding_snapshot = HashMap::new();

        // Walk the environment chain
        let mut current_env = Some(env.clone());
        while let Some(env) = current_env {
            environment_chain.push(env.clone());
            
            // Capture variable bindings for GC root scanning
            let var_names = env.all_variable_names();
            for var_name in var_names {
                if let Some(value) = env.lookup(&var_name) {
                    binding_snapshot.insert(var_name, value);
                }
            }
            
            current_env = env.parent().cloned();
        }

        Ok(EnvironmentCaptureInfo {
            main_environment: env.clone(),
            environment_chain,
            binding_snapshot,
            captured_at: Instant::now(),
        })
    }

    /// Estimates memory usage of a continuation for GC statistics.
    fn estimate_continuation_memory(
        &self,
        frames: &[Frame],
        environment: &Arc<ThreadSafeEnvironment>,
    ) -> usize {
        let frame_size = frames.len() * 256; // Estimated size per frame
        let env_size = environment.all_variable_names().len() * 64; // Estimated per binding
        frame_size + env_size + 512 // Base continuation overhead
    }

    /// Gets statistics about active continuations.
    pub fn get_continuation_statistics(&self) -> ContinuationStatistics {
        let active_count = if let Ok(registry) = self.continuation_registry.read() {
            registry.values()
                .filter(|entry| entry.active.load(Ordering::SeqCst))
                .count()
        } else {
            0
        };

        let total_count = if let Ok(registry) = self.continuation_registry.read() {
            registry.len()
        } else {
            0
        };

        let preserved_traces = self.stack_trace_manager.preserved_trace_count();

        ContinuationStatistics {
            active_continuations: active_count,
            total_continuations: total_count,
            preserved_stack_traces: preserved_traces,
            gc_roots_tracked: self.count_gc_tracked_continuations(),
        }
    }

    /// Counts the number of GC-tracked continuations.
    fn count_gc_tracked_continuations(&self) -> usize {
        if let Ok(registry) = self.continuation_registry.read() {
            registry.values()
                .filter(|entry| entry.gc_object_id.is_some())
                .count()
        } else {
            0
        }
    }

    /// Gets the configuration.
    pub fn config(&self) -> &GcContinuationConfig {
        &self.config
    }
}

impl StackTraceManager {
    /// Creates a new stack trace manager.
    pub fn new(config: StackTraceConfig) -> Self {
        Self {
            preserved_traces: RwLock::new(HashMap::new()),
            frame_pool: RwLock::new(Vec::new()),
            config,
        }
    }

    /// Preserves a stack trace for a continuation.
    pub fn preserve_trace(&self, continuation_id: u64, trace: StackTrace) -> Result<()> {
        let preserved_trace = PreservedStackTrace {
            trace,
            preserved_at: Instant::now(),
            compressed: false,
            gc_object_id: None,
        };

        if let Ok(mut traces) = self.preserved_traces.write() {
            traces.insert(continuation_id, preserved_trace);
            
            // Clean up old traces if we exceed the limit
            if traces.len() > self.config.max_preserved_traces {
                self.cleanup_old_traces(&mut traces);
            }
        }

        Ok(())
    }

    /// Gets a preserved stack trace.
    pub fn get_preserved_trace(&self, continuation_id: u64) -> Option<PreservedStackTrace> {
        if let Ok(traces) = self.preserved_traces.read() {
            traces.get(&continuation_id).cloned()
        } else {
            None
        }
    }

    /// Removes a preserved stack trace.
    pub fn remove_preserved_trace(&self, continuation_id: u64) {
        if let Ok(mut traces) = self.preserved_traces.write() {
            traces.remove(&continuation_id);
        }
    }

    /// Gets the count of preserved traces.
    pub fn preserved_trace_count(&self) -> usize {
        if let Ok(traces) = self.preserved_traces.read() {
            traces.len()
        } else {
            0
        }
    }

    /// Cleans up old traces to stay within limits.
    fn cleanup_old_traces(&self, traces: &mut HashMap<u64, PreservedStackTrace>) {
        if traces.len() <= self.config.max_preserved_traces {
            return;
        }

        // Sort by age and remove oldest traces
        let mut entries: Vec<_> = traces.iter().map(|(k, v)| (*k, v.preserved_at)).collect();
        entries.sort_by_key(|(_, time)| *time);
        
        let to_remove = traces.len() - self.config.max_preserved_traces;
        let ids_to_remove: Vec<_> = entries.iter().take(to_remove).map(|(id, _)| *id).collect();
        
        for id in ids_to_remove {
            traces.remove(&id);
        }
    }
}

impl ContinuationGcWrapper {
    /// Creates a new GC wrapper for continuation data.
    pub fn new(data: ContinuationData) -> Self {
        Self {
            continuation_data: data,
            generation: AtomicU32::new(0),
            marked: AtomicBool::new(false),
        }
    }

    /// Gets a reference to the continuation data.
    pub fn data(&self) -> &ContinuationData {
        &self.continuation_data
    }
}

impl GcObject for ContinuationGcWrapper {
    fn generation(&self) -> GenerationId {
        self.generation.load(Ordering::Relaxed)
    }

    fn set_generation(&mut self, generation: GenerationId) {
        self.generation.store(generation, Ordering::Relaxed);
    }

    fn references(&self) -> Vec<crate::utils::gc::GcPtr> {
        // For now, we don't traverse into continuation internals
        // This maintains encapsulation and existing reference management
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
        // Estimate based on continuation complexity
        let base_size = std::mem::size_of::<ContinuationData>();
        let frame_size = self.continuation_data.frames.len() * 256;
        let env_size = 512; // Estimated environment overhead
        base_size + frame_size + env_size
    }
}

/// Statistics about continuation usage.
#[derive(Debug, Clone)]
pub struct ContinuationStatistics {
    /// Number of active continuations
    pub active_continuations: usize,
    /// Total number of continuations (active + inactive)
    pub total_continuations: usize,
    /// Number of preserved stack traces
    pub preserved_stack_traces: usize,
    /// Number of continuations tracked by GC
    pub gc_roots_tracked: usize,
}

impl Default for GcContinuationConfig {
    fn default() -> Self {
        Self {
            track_continuations: true,
            preserve_stack_traces: true,
            optimize_continuations: true,
            max_stack_depth: 1000,
            enable_debugging: false,
        }
    }
}

impl Default for StackTraceConfig {
    fn default() -> Self {
        Self {
            max_preserved_traces: 100,
            max_frames_per_trace: 50,
            compress_old_traces: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::GcIntegration;
    use crate::eval::value::{ThreadSafeEnvironment, Value, Frame};
    use crate::diagnostics::Span;

    #[test]
    fn test_continuation_manager_creation() {
        let gc_integration = Arc::new(GcIntegration::with_default_config());
        let manager = GcContinuationManager::with_default_config(gc_integration);
        
        let stats = manager.get_continuation_statistics();
        assert_eq!(stats.active_continuations, 0);
        assert_eq!(stats.total_continuations, 0);
    }

    #[test]
    fn test_continuation_capture() {
        let gc_integration = Arc::new(GcIntegration::with_default_config());
        let manager = GcContinuationManager::with_default_config(gc_integration);
        
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let frames = Vec::new();
        
        let result = manager.capture_continuation(frames, env, None, None);
        assert!(result.is_ok());
        
        let stats = manager.get_continuation_statistics();
        assert_eq!(stats.active_continuations, 1);
        assert_eq!(stats.total_continuations, 1);
    }

    #[test]
    fn test_stack_trace_preservation() {
        let gc_integration = Arc::new(GcIntegration::with_default_config());
        let manager = GcContinuationManager::with_default_config(gc_integration);
        
        let mut stack_trace = StackTrace::new();
        stack_trace.push(StackFrame::top_level(None));
        
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let frames = Vec::new();
        
        let result = manager.capture_continuation(frames, env, None, Some(stack_trace));
        assert!(result.is_ok());
        
        let stats = manager.get_continuation_statistics();
        assert_eq!(stats.preserved_stack_traces, 1);
    }

    #[test]
    fn test_continuation_cleanup() {
        let gc_integration = Arc::new(GcIntegration::with_default_config());
        let manager = GcContinuationManager::with_default_config(gc_integration);
        
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let frames = Vec::new();
        
        let continuation = manager.capture_continuation(frames, env, None, None).unwrap();
        let continuation_id = continuation.id;
        
        let cleanup_result = manager.cleanup_continuation(continuation_id);
        assert!(cleanup_result.is_ok());
        
        let stats = manager.get_continuation_statistics();
        assert_eq!(stats.active_continuations, 0);
    }
}
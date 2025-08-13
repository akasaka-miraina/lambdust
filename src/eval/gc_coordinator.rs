//! GC coordinator for seamless integration with the evaluation engine.
//!
//! This module provides the coordination layer between the garbage collector
//! and the evaluator, ensuring that all runtime objects are properly managed
//! while maintaining transparent R7RS semantics.

use crate::utils::{GcIntegration, GcIntegrationConfig, GcEnvironment, scan_value_for_gc_integration};
use crate::utils::gc::{gc_collect, gc_stats, gc_debug_info, GcStats, GcDebugInfo};
use crate::eval::{Value, ThreadSafeEnvironment, Evaluator, Continuation, StackTrace};
use crate::diagnostics::{Error, Result, Span};
use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};

/// Coordinator that manages GC integration with the evaluation engine.
/// This provides the main interface for GC-aware evaluation.
#[derive(Debug)]
pub struct GcCoordinator {
    /// GC integration manager
    integration: Arc<GcIntegration>,
    /// Active evaluation sessions
    active_sessions: RwLock<HashMap<SessionId, EvaluationSession>>,
    /// Global root registry
    global_roots: RwLock<Vec<GlobalRoot>>,
    /// GC statistics collector
    stats_collector: RwLock<GcStatsCollector>,
    /// Configuration
    config: GcCoordinatorConfig,
    /// Session counter
    next_session_id: AtomicU64,
}

/// Configuration for the GC coordinator.
#[derive(Debug, Clone)]
pub struct GcCoordinatorConfig {
    /// Whether to enable automatic root detection
    pub auto_root_detection: bool,
    /// Whether to enable GC during evaluation
    pub gc_during_evaluation: bool,
    /// Maximum time between GC cycles (milliseconds)
    pub max_gc_interval_ms: u64,
    /// Whether to enable GC statistics collection
    pub collect_statistics: bool,
    /// Whether to preserve continuation chains across GC
    pub preserve_continuations: bool,
}

/// Unique identifier for evaluation sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

/// An active evaluation session that tracks GC roots.
#[derive(Debug)]
pub struct EvaluationSession {
    /// Session identifier
    pub id: SessionId,
    /// Environment stack for this session
    pub environment_stack: Vec<Arc<ThreadSafeEnvironment>>,
    /// Active continuations
    pub continuations: Vec<Arc<Continuation>>,
    /// Stack trace for error reporting
    pub stack_trace: Option<StackTrace>,
    /// Session start time
    pub start_time: Instant,
    /// Whether this session is active
    pub active: AtomicBool,
}

/// A global root that must never be collected.
#[derive(Debug, Clone)]
pub enum GlobalRoot {
    /// Environment that contains global bindings
    Environment(Arc<ThreadSafeEnvironment>),
    /// Important value that must be preserved
    Value(Value),
    /// Standard library binding
    StdlibBinding { 
        /// Name of the standard library binding
        name: String, 
        /// Value associated with the binding
        value: Value 
    },
    /// Module system root
    ModuleRoot { 
        /// Unique identifier for the module
        module_id: String, 
        /// Module's environment containing bindings
        environment: Arc<ThreadSafeEnvironment> 
    },
}

/// Statistics collector for GC performance monitoring.
#[derive(Debug)]
pub struct GcStatsCollector {
    /// Collection history
    pub collection_history: Vec<GcCollectionEvent>,
    /// Total collections
    pub total_collections: u64,
    /// Total time spent in GC
    pub total_gc_time: Duration,
    /// Average collection time
    pub average_collection_time: Duration,
    /// Peak memory usage
    pub peak_memory_usage: usize,
}

/// A single GC collection event.
#[derive(Debug, Clone)]
pub struct GcCollectionEvent {
    /// When the collection occurred
    pub timestamp: Instant,
    /// GC statistics from this collection
    pub stats: GcStats,
    /// Active evaluation sessions during collection
    pub active_sessions: usize,
    /// Number of roots scanned
    pub roots_scanned: usize,
}

impl GcCoordinator {
    /// Creates a new GC coordinator with the specified configuration.
    pub fn new(config: GcCoordinatorConfig) -> Result<Self> {
        let gc_config = GcIntegrationConfig {
            auto_register_environments: config.auto_root_detection,
            preserve_stack_traces: true,
            gc_aware_macros: true,
            gc_threshold_size: 512, // Use GC for objects > 512 bytes
        };

        let integration = Arc::new(GcIntegration::new(gc_config));

        Ok(Self {
            integration,
            active_sessions: RwLock::new(HashMap::new()),
            global_roots: RwLock::new(Vec::new()),
            stats_collector: RwLock::new(GcStatsCollector::new()),
            config,
            next_session_id: AtomicU64::new(1),
        })
    }

    /// Creates a GC coordinator with default configuration.
    pub fn with_default_config() -> Result<Self> {
        Self::new(GcCoordinatorConfig::default())
    }

    /// Starts a new evaluation session with GC tracking.
    pub fn start_session(&self, initial_environment: Arc<ThreadSafeEnvironment>) -> SessionId {
        let session_id = SessionId(self.next_session_id.fetch_add(1, Ordering::SeqCst));
        
        let session = EvaluationSession {
            id: session_id,
            environment_stack: vec![initial_environment.clone()],
            continuations: Vec::new(),
            stack_trace: None,
            start_time: Instant::now(),
            active: AtomicBool::new(true),
        };

        // Register the initial environment as a root if auto-detection is enabled
        if self.config.auto_root_detection {
            self.integration.register_continuation_root(
                crate::utils::gc::ObjectId::new(session_id.0)
            );
        }

        if let Ok(mut sessions) = self.active_sessions.write() {
            sessions.insert(session_id, session);
        }

        session_id
    }

    /// Ends an evaluation session and removes its GC roots.
    pub fn end_session(&self, session_id: SessionId) {
        if let Ok(mut sessions) = self.active_sessions.write() {
            if let Some(session) = sessions.remove(&session_id) {
                session.active.store(false, Ordering::SeqCst);
                
                // Unregister from GC if auto-detection was used
                if self.config.auto_root_detection {
                    self.integration.unregister_continuation_root(
                        crate::utils::gc::ObjectId::new(session_id.0)
                    );
                }
            }
        }
    }

    /// Adds an environment to the evaluation session's stack.
    pub fn push_environment(&self, session_id: SessionId, env: Arc<ThreadSafeEnvironment>) {
        if let Ok(mut sessions) = self.active_sessions.write() {
            if let Some(session) = sessions.get_mut(&session_id) {
                session.environment_stack.push(env);
            }
        }
    }

    /// Removes the top environment from the evaluation session's stack.
    pub fn pop_environment(&self, session_id: SessionId) -> Option<Arc<ThreadSafeEnvironment>> {
        if let Ok(mut sessions) = self.active_sessions.write() {
            if let Some(session) = sessions.get_mut(&session_id) {
                session.environment_stack.pop()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Adds a continuation to the session for GC root tracking.
    pub fn register_continuation(&self, session_id: SessionId, continuation: Arc<Continuation>) {
        if let Ok(mut sessions) = self.active_sessions.write() {
            if let Some(session) = sessions.get_mut(&session_id) {
                session.continuations.push(continuation);
            }
        }
    }

    /// Updates the stack trace for a session.
    pub fn update_stack_trace(&self, session_id: SessionId, stack_trace: StackTrace) {
        if let Ok(mut sessions) = self.active_sessions.write() {
            if let Some(session) = sessions.get_mut(&session_id) {
                session.stack_trace = Some(stack_trace);
            }
        }
    }

    /// Adds a global root that must never be collected.
    pub fn add_global_root(&self, root: GlobalRoot) {
        if let Ok(mut roots) = self.global_roots.write() {
            roots.push(root);
        }
    }

    /// Performs a comprehensive GC root scan including all active sessions.
    pub fn comprehensive_root_scan(&self) -> ComprehensiveRootScanResult {
        let mut session_roots = Vec::new();
        let mut continuation_roots = Vec::new();
        
        if let Ok(sessions) = self.active_sessions.read() {
            for session in sessions.values() {
                if session.active.load(Ordering::SeqCst) {
                    // Scan environment stack
                    for env in &session.environment_stack {
                        let gc_env = GcEnvironment::new(env.clone());
                        session_roots.extend(gc_env.scan_for_gc_roots());
                    }
                    
                    // Add continuation count
                    continuation_roots.extend(
                        session.continuations.iter().map(|c| c.id)
                    );
                }
            }
        }

        let global_roots = if let Ok(roots) = self.global_roots.read() {
            roots.clone()
        } else {
            Vec::new()
        };

        ComprehensiveRootScanResult {
            session_roots,
            continuation_roots,
            global_roots,
            active_session_count: self.active_session_count(),
        }
    }

    /// Triggers garbage collection if enabled and conditions are met.
    pub fn maybe_collect(&self) -> Option<GcCollectionResult> {
        if !self.config.gc_during_evaluation {
            return None;
        }

        // Check if enough time has passed since last collection
        if let Ok(stats) = self.stats_collector.read() {
            if let Some(last_event) = stats.collection_history.last() {
                let elapsed = last_event.timestamp.elapsed();
                if elapsed.as_millis() < self.config.max_gc_interval_ms as u128 {
                    return None; // Too soon for another collection
                }
            }
        }

        Some(self.force_collect())
    }

    /// Forces garbage collection regardless of conditions.
    pub fn force_collect(&self) -> GcCollectionResult {
        let start_time = Instant::now();
        
        // Perform comprehensive root scan
        let root_scan = self.comprehensive_root_scan();
        
        // Trigger GC
        gc_collect();
        
        // Collect statistics
        let gc_stats = gc_stats();
        let debug_info = gc_debug_info();
        
        let collection_time = start_time.elapsed();
        let result = GcCollectionResult {
            collection_time,
            roots_scanned: root_scan.session_roots.len() + root_scan.global_roots.len(),
            active_sessions: root_scan.active_session_count,
            gc_stats: gc_stats.last().cloned(),
            debug_info,
        };

        // Update statistics if collection is enabled
        if self.config.collect_statistics {
            if let Ok(mut collector) = self.stats_collector.write() {
                collector.record_collection(&result, &root_scan);
            }
        }

        result
    }

    /// Gets the number of currently active evaluation sessions.
    pub fn active_session_count(&self) -> usize {
        if let Ok(sessions) = self.active_sessions.read() {
            sessions.values()
                .filter(|s| s.active.load(Ordering::SeqCst))
                .count()
        } else {
            0
        }
    }

    /// Gets current GC statistics.
    pub fn get_statistics(&self) -> Option<GcStatsSummary> {
        if !self.config.collect_statistics {
            return None;
        }

        if let Ok(collector) = self.stats_collector.read() {
            Some(collector.summarize())
        } else {
            None
        }
    }

    /// Gets the GC integration configuration.
    pub fn config(&self) -> &GcCoordinatorConfig {
        &self.config
    }

    /// Gets detailed debug information about GC state.
    pub fn debug_info(&self) -> GcCoordinatorDebugInfo {
        GcCoordinatorDebugInfo {
            active_sessions: self.active_session_count(),
            global_roots: self.global_roots.read()
                .map(|r| r.len()).unwrap_or(0),
            gc_debug_info: gc_debug_info(),
            statistics: self.get_statistics(),
        }
    }
}

/// Result of a comprehensive root scan.
#[derive(Debug)]
pub struct ComprehensiveRootScanResult {
    /// Roots found in active evaluation sessions
    pub session_roots: Vec<Value>,
    /// IDs of active continuations
    pub continuation_roots: Vec<u64>,
    /// Global roots that must be preserved
    pub global_roots: Vec<GlobalRoot>,
    /// Number of active sessions
    pub active_session_count: usize,
}

/// Result of a GC collection operation.
#[derive(Debug)]
pub struct GcCollectionResult {
    /// Time spent in collection
    pub collection_time: Duration,
    /// Number of roots scanned
    pub roots_scanned: usize,
    /// Number of active sessions during collection
    pub active_sessions: usize,
    /// GC statistics if available
    pub gc_stats: Option<GcStats>,
    /// Detailed GC debug information
    pub debug_info: GcDebugInfo,
}

/// Summary of GC statistics over time.
#[derive(Debug, Clone)]
pub struct GcStatsSummary {
    /// Total number of collections
    pub total_collections: u64,
    /// Total time spent in GC
    pub total_gc_time: Duration,
    /// Average collection time
    pub average_collection_time: Duration,
    /// Peak memory usage observed
    pub peak_memory_usage: usize,
    /// Recent collection events (last 10)
    pub recent_events: Vec<GcCollectionEvent>,
}

/// Debug information about the GC coordinator state.
#[derive(Debug)]
pub struct GcCoordinatorDebugInfo {
    /// Number of active evaluation sessions
    pub active_sessions: usize,
    /// Number of registered global roots
    pub global_roots: usize,
    /// GC system debug information
    pub gc_debug_info: GcDebugInfo,
    /// Statistics summary if available
    pub statistics: Option<GcStatsSummary>,
}

impl Default for GcStatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl GcStatsCollector {
    /// Creates a new statistics collector.
    pub fn new() -> Self {
        Self {
            collection_history: Vec::new(),
            total_collections: 0,
            total_gc_time: Duration::from_secs(0),
            average_collection_time: Duration::from_secs(0),
            peak_memory_usage: 0,
        }
    }

    /// Records a new collection event.
    pub fn record_collection(
        &mut self,
        result: &GcCollectionResult,
        _root_scan: &ComprehensiveRootScanResult,
    ) {
        let event = GcCollectionEvent {
            timestamp: Instant::now(),
            stats: result.gc_stats.clone().unwrap_or_default(),
            active_sessions: result.active_sessions,
            roots_scanned: result.roots_scanned,
        };

        self.collection_history.push(event);
        self.total_collections += 1;
        self.total_gc_time += result.collection_time;
        
        // Update peak memory usage
        self.peak_memory_usage = self.peak_memory_usage.max(
            result.debug_info.total_memory
        );

        // Calculate new average
        self.average_collection_time = self.total_gc_time / self.total_collections.max(1) as u32;

        // Keep only last 100 events
        if self.collection_history.len() > 100 {
            self.collection_history.remove(0);
        }
    }

    /// Creates a summary of statistics.
    pub fn summarize(&self) -> GcStatsSummary {
        let recent_events = self.collection_history
            .iter()
            .rev()
            .take(10)
            .cloned()
            .collect();

        GcStatsSummary {
            total_collections: self.total_collections,
            total_gc_time: self.total_gc_time,
            average_collection_time: self.average_collection_time,
            peak_memory_usage: self.peak_memory_usage,
            recent_events,
        }
    }
}

impl Default for GcCoordinatorConfig {
    fn default() -> Self {
        Self {
            auto_root_detection: true,
            gc_during_evaluation: true,
            max_gc_interval_ms: 1000, // 1 second
            collect_statistics: true,
            preserve_continuations: true,
        }
    }
}


/// Extension trait for Evaluator to support GC coordination.
pub trait EvaluatorGcExt {
    /// Gets the GC coordinator for this evaluator.
    fn gc_coordinator(&self) -> Option<&GcCoordinator>;
    
    /// Enables GC coordination for this evaluator.
    fn with_gc_coordination(&mut self, coordinator: GcCoordinator);
    
    /// Starts a GC-tracked evaluation session.
    fn start_gc_session(&self, env: Arc<ThreadSafeEnvironment>) -> Option<SessionId>;
    
    /// Ends a GC-tracked evaluation session.
    fn end_gc_session(&self, session_id: SessionId);
}

// Note: The actual implementation of EvaluatorGcExt would be done in
// the evaluator.rs file to avoid circular dependencies.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::{ThreadSafeEnvironment, Value};

    #[test]
    fn test_gc_coordinator_creation() {
        let coordinator = GcCoordinator::with_default_config().unwrap();
        assert_eq!(coordinator.active_session_count(), 0);
    }

    #[test]
    fn test_session_lifecycle() {
        let coordinator = GcCoordinator::with_default_config().unwrap();
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        
        let session_id = coordinator.start_session(env.clone());
        assert_eq!(coordinator.active_session_count(), 1);
        
        coordinator.end_session(session_id);
        assert_eq!(coordinator.active_session_count(), 0);
    }

    #[test]
    fn test_environment_stack_management() {
        let coordinator = GcCoordinator::with_default_config().unwrap();
        let env1 = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let env2 = Arc::new(ThreadSafeEnvironment::new(None, 1));
        
        let session_id = coordinator.start_session(env1);
        coordinator.push_environment(session_id, env2.clone());
        
        let popped = coordinator.pop_environment(session_id);
        assert!(Arc::ptr_eq(&popped.unwrap(), &env2));
        
        coordinator.end_session(session_id);
    }

    #[test]
    fn test_global_roots() {
        let coordinator = GcCoordinator::with_default_config().unwrap();
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        
        let root = GlobalRoot::Environment(env);
        coordinator.add_global_root(root);
        
        let scan_result = coordinator.comprehensive_root_scan();
        assert_eq!(scan_result.global_roots.len(), 1);
    }

    #[test]
    fn test_statistics_collection() {
        let coordinator = GcCoordinator::with_default_config().unwrap();
        
        // Force a collection to generate statistics
        let _result = coordinator.force_collect();
        
        let stats = coordinator.get_statistics();
        assert!(stats.is_some());
        
        let stats = stats.unwrap();
        assert!(stats.total_collections > 0);
    }
}
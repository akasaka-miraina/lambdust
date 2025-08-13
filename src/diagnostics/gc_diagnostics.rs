//! GC-aware diagnostic system for Lambdust.
//!
//! This module provides error reporting and diagnostic preservation that integrates
//! with the parallel garbage collector while maintaining complete diagnostic information,
//! stack traces, and source location data across GC cycles.

use crate::utils::{GcIntegration, GcIntegrationConfig};
use crate::utils::gc::{ObjectId, gc_alloc, GcObject, GenerationId};
use crate::diagnostics::{Error, Span, Result};
use crate::eval::{StackTrace, StackFrame, FrameType};
use std::sync::{Arc, RwLock, Mutex, atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering}};
use std::collections::HashMap;
use std::time::Instant;
use std::fmt;

/// GC-aware diagnostic manager that preserves error information across garbage collection.
#[derive(Debug)]
pub struct GcDiagnosticManager {
    /// GC integration manager
    gc_integration: Arc<GcIntegration>,
    /// Registry of preserved diagnostic information
    diagnostic_registry: RwLock<HashMap<DiagnosticId, DiagnosticEntry>>,
    /// Error context preservation system
    context_manager: Arc<ErrorContextManager>,
    /// Configuration
    config: GcDiagnosticConfig,
    /// Next diagnostic ID
    next_diagnostic_id: AtomicU64,
}

/// Configuration for GC-aware diagnostics.
#[derive(Debug, Clone)]
pub struct GcDiagnosticConfig {
    /// Whether to enable GC tracking for diagnostic information
    pub track_diagnostics: bool,
    /// Whether to preserve error contexts across GC cycles
    pub preserve_contexts: bool,
    /// Whether to preserve stack traces in errors
    pub preserve_stack_traces: bool,
    /// Maximum number of preserved diagnostics
    pub max_preserved_diagnostics: usize,
    /// Whether to compress old diagnostic information
    pub compress_old_diagnostics: bool,
}

/// Unique identifier for diagnostic entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticId(u64);

/// Entry in the diagnostic registry for GC tracking.
#[derive(Debug)]
pub struct DiagnosticEntry {
    /// Unique identifier
    pub id: DiagnosticId,
    /// The preserved error information
    pub error_info: PreservedError,
    /// GC object ID if tracked
    pub gc_object_id: Option<ObjectId>,
    /// Creation timestamp
    pub created_at: Instant,
    /// Whether this diagnostic is still active
    pub active: AtomicBool,
}

/// Error information preserved across GC cycles.
#[derive(Debug, Clone)]
pub struct PreservedError {
    /// Error message
    pub message: String,
    /// Error kind/type
    pub kind: ErrorKind,
    /// Source location if available
    pub span: Option<Span>,
    /// Stack trace at error time
    pub stack_trace: Option<StackTrace>,
    /// Additional context information
    pub context: ErrorContext,
    /// Cause chain (nested errors)
    pub cause_chain: Vec<PreservedError>,
    /// Error metadata
    pub metadata: HashMap<String, String>,
}

/// Classification of error types for GC handling.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// Runtime evaluation error
    RuntimeError,
    /// Syntax or parsing error
    SyntaxError,
    /// Type-related error
    TypeError,
    /// I/O or system error
    IoError,
    /// FFI-related error
    FfiError,
    /// Macro expansion error
    MacroError,
    /// Continuation-related error
    ContinuationError,
    /// Memory or GC-related error
    MemoryError,
    /// Custom error type
    Custom(String),
}

/// Context information for error diagnosis.
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    /// Current expression being evaluated
    pub current_expr: Option<String>,
    /// Variable bindings at error time
    pub bindings: HashMap<String, String>,
    /// Procedure call chain
    pub call_chain: Vec<String>,
    /// Module context
    pub module_context: Option<String>,
    /// Additional contextual notes
    pub notes: Vec<String>,
}

/// Manager for error context preservation.
#[derive(Debug)]
pub struct ErrorContextManager {
    /// Preserved contexts indexed by diagnostic ID
    preserved_contexts: RwLock<HashMap<DiagnosticId, PreservedErrorContext>>,
    /// Context compression settings
    config: ContextConfig,
}

/// Configuration for error context management.
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Maximum number of preserved contexts
    pub max_preserved_contexts: usize,
    /// Whether to compress context data
    pub compress_contexts: bool,
    /// Maximum size of individual context entries
    pub max_context_size: usize,
}

/// Preserved error context with GC-safe storage.
#[derive(Debug, Clone)]
pub struct PreservedErrorContext {
    /// The error context
    pub context: ErrorContext,
    /// Preservation timestamp
    pub preserved_at: Instant,
    /// Whether this context is compressed
    pub compressed: bool,
    /// Original size before compression
    pub original_size: usize,
}

/// GC wrapper for diagnostic information.
#[derive(Debug)]
pub struct DiagnosticGcWrapper {
    /// The preserved error information
    preserved_error: PreservedError,
    /// GC metadata
    generation: AtomicU32,
    marked: AtomicBool,
}

/// A GC-aware error that maintains diagnostic information across collection cycles.
#[derive(Debug)]
pub struct GcAwareError {
    /// The underlying error
    inner_error: Error,
    /// Diagnostic ID for GC tracking
    diagnostic_id: Option<DiagnosticId>,
    /// Preserved information
    preserved_info: Option<PreservedError>,
}

impl GcDiagnosticManager {
    /// Creates a new GC-aware diagnostic manager.
    pub fn new(
        gc_integration: Arc<GcIntegration>,
        config: GcDiagnosticConfig,
    ) -> Self {
        let context_manager = Arc::new(ErrorContextManager::new(
            ContextConfig::default()
        ));

        Self {
            gc_integration,
            diagnostic_registry: RwLock::new(HashMap::new()),
            context_manager,
            config,
            next_diagnostic_id: AtomicU64::new(1),
        }
    }

    /// Creates a diagnostic manager with default configuration.
    pub fn with_default_config(gc_integration: Arc<GcIntegration>) -> Self {
        Self::new(gc_integration, GcDiagnosticConfig::default())
    }

    /// Creates a GC-aware error with preserved diagnostic information.
    pub fn create_gc_aware_error(
        &self,
        error: Error,
        stack_trace: Option<StackTrace>,
        context: Option<ErrorContext>,
    ) -> GcAwareError {
        if !self.config.track_diagnostics {
            return GcAwareError {
                inner_error: error,
                diagnostic_id: None,
                preserved_info: None,
            };
        }

        let diagnostic_id = DiagnosticId(
            self.next_diagnostic_id.fetch_add(1, Ordering::SeqCst)
        );

        // Create preserved error information
        let preserved_error = self.create_preserved_error(
            &error,
            stack_trace,
            context.unwrap_or_default(),
        );

        // Create GC wrapper if tracking is enabled
        let gc_object_id = if self.config.track_diagnostics {
            let wrapper = DiagnosticGcWrapper::new(preserved_error.clone());
            let gc_ptr = gc_alloc(wrapper);
            let object_id = gc_ptr.id();
            
            // Register with GC integration (as a macro root for now)
            self.gc_integration.register_macro_root(object_id);
            
            Some(object_id)
        } else {
            None
        };

        // Create registry entry
        let entry = DiagnosticEntry {
            id: diagnostic_id,
            error_info: preserved_error.clone(),
            gc_object_id,
            created_at: Instant::now(),
            active: AtomicBool::new(true),
        };

        // Register the diagnostic
        if let Ok(mut registry) = self.diagnostic_registry.write() {
            registry.insert(diagnostic_id, entry);
            
            // Clean up old entries if we exceed the limit
            if registry.len() > self.config.max_preserved_diagnostics {
                self.cleanup_old_diagnostics(&mut registry);
            }
        }

        GcAwareError {
            inner_error: error,
            diagnostic_id: Some(diagnostic_id),
            preserved_info: Some(preserved_error),
        }
    }

    /// Creates preserved error information from an Error.
    fn create_preserved_error(
        &self,
        error: &Error,
        stack_trace: Option<StackTrace>,
        context: ErrorContext,
    ) -> PreservedError {
        let kind = self.classify_error(error);
        let (message, span) = self.extract_error_details(error);
        
        PreservedError {
            message,
            kind,
            span,
            stack_trace,
            context,
            cause_chain: Vec::new(), // Could be extended to handle error chains
            metadata: HashMap::new(),
        }
    }

    /// Extracts message and span from an Error enum.
    fn extract_error_details(&self, error: &Error) -> (String, Option<Span>) {
        match error {
            Error::LexError { message, span } => (message.clone(), Some(*span)),
            Error::ParseError { message, span } => (message.clone(), Some(*span)),
            Error::TypeError { message, span } => (message.clone(), Some(*span)),
            Error::MacroError { message, span } => (message.clone(), Some(*span)),
            Error::RuntimeError { message, span } => (message.clone(), *span),
            Error::FfiError { message } => (message.clone(), None),
            Error::IoError { message } => (message.clone(), None),
            Error::InternalError { message } => (message.clone(), None),
            Error::Exception { exception, span } => (exception.to_string(), *span),
        }
    }

    /// Classifies an error for GC handling.
    fn classify_error(&self, error: &Error) -> ErrorKind {
        match error {
            Error::LexError { .. } => ErrorKind::SyntaxError,
            Error::ParseError { .. } => ErrorKind::SyntaxError,
            Error::TypeError { .. } => ErrorKind::TypeError,
            Error::MacroError { .. } => ErrorKind::MacroError,
            Error::RuntimeError { .. } => ErrorKind::RuntimeError,
            Error::FfiError { .. } => ErrorKind::FfiError,
            Error::IoError { .. } => ErrorKind::IoError,
            Error::InternalError { .. } => ErrorKind::RuntimeError,
            Error::Exception { .. } => ErrorKind::RuntimeError,
        }
    }

    /// Retrieves preserved diagnostic information.
    pub fn get_preserved_diagnostic(&self, diagnostic_id: DiagnosticId) -> Option<PreservedError> {
        if let Ok(registry) = self.diagnostic_registry.read() {
            registry.get(&diagnostic_id).map(|entry| entry.error_info.clone())
        } else {
            None
        }
    }

    /// Cleans up old diagnostic entries to stay within limits.
    fn cleanup_old_diagnostics(&self, registry: &mut HashMap<DiagnosticId, DiagnosticEntry>) {
        if registry.len() <= self.config.max_preserved_diagnostics {
            return;
        }

        // Sort by creation time and remove oldest entries
        let mut entries: Vec<_> = registry.iter()
            .map(|(id, entry)| (*id, entry.created_at))
            .collect();
        entries.sort_by_key(|(_, time)| *time);
        
        let to_remove = registry.len() - self.config.max_preserved_diagnostics;
        let ids_to_remove: Vec<_> = entries.iter()
            .take(to_remove)
            .map(|(id, _)| *id)
            .collect();
        
        for id in ids_to_remove {
            if let Some(entry) = registry.remove(&id) {
                // Unregister from GC if it was tracked
                if let Some(gc_object_id) = entry.gc_object_id {
                    self.gc_integration.unregister_continuation_root(gc_object_id);
                }
            }
        }
    }

    /// Gets statistics about diagnostic preservation.
    pub fn get_diagnostic_statistics(&self) -> DiagnosticStatistics {
        let active_count = if let Ok(registry) = self.diagnostic_registry.read() {
            registry.values()
                .filter(|entry| entry.active.load(Ordering::SeqCst))
                .count()
        } else {
            0
        };

        let total_count = if let Ok(registry) = self.diagnostic_registry.read() {
            registry.len()
        } else {
            0
        };

        let preserved_contexts = self.context_manager.preserved_context_count();

        DiagnosticStatistics {
            active_diagnostics: active_count,
            total_diagnostics: total_count,
            preserved_contexts,
            gc_roots_tracked: self.count_gc_tracked_diagnostics(),
        }
    }

    /// Counts the number of GC-tracked diagnostics.
    fn count_gc_tracked_diagnostics(&self) -> usize {
        if let Ok(registry) = self.diagnostic_registry.read() {
            registry.values()
                .filter(|entry| entry.gc_object_id.is_some())
                .count()
        } else {
            0
        }
    }

    /// Gets the configuration.
    pub fn config(&self) -> &GcDiagnosticConfig {
        &self.config
    }
}

impl ErrorContextManager {
    /// Creates a new error context manager.
    pub fn new(config: ContextConfig) -> Self {
        Self {
            preserved_contexts: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Preserves an error context.
    pub fn preserve_context(&self, diagnostic_id: DiagnosticId, context: ErrorContext) -> Result<()> {
        let context_size = self.estimate_context_size(&context);
        
        let preserved_context = PreservedErrorContext {
            context: if context_size > self.config.max_context_size && self.config.compress_contexts {
                self.compress_context(context)
            } else {
                context
            },
            preserved_at: Instant::now(),
            compressed: context_size > self.config.max_context_size,
            original_size: context_size,
        };

        if let Ok(mut contexts) = self.preserved_contexts.write() {
            contexts.insert(diagnostic_id, preserved_context);
            
            // Clean up old contexts if we exceed the limit
            if contexts.len() > self.config.max_preserved_contexts {
                self.cleanup_old_contexts(&mut contexts);
            }
        }

        Ok(())
    }

    /// Gets a preserved error context.
    pub fn get_preserved_context(&self, diagnostic_id: DiagnosticId) -> Option<PreservedErrorContext> {
        if let Ok(contexts) = self.preserved_contexts.read() {
            contexts.get(&diagnostic_id).cloned()
        } else {
            None
        }
    }

    /// Gets the count of preserved contexts.
    pub fn preserved_context_count(&self) -> usize {
        if let Ok(contexts) = self.preserved_contexts.read() {
            contexts.len()
        } else {
            0
        }
    }

    /// Estimates the memory size of an error context.
    fn estimate_context_size(&self, context: &ErrorContext) -> usize {
        let mut size = 0;
        
        if let Some(ref expr) = context.current_expr {
            size += expr.len();
        }
        
        size += context.bindings.iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>();
            
        size += context.call_chain.iter()
            .map(|s| s.len())
            .sum::<usize>();
            
        if let Some(ref module) = context.module_context {
            size += module.len();
        }
        
        size += context.notes.iter()
            .map(|s| s.len())
            .sum::<usize>();
            
        size
    }

    /// Compresses an error context to reduce memory usage.
    fn compress_context(&self, mut context: ErrorContext) -> ErrorContext {
        // Simple compression - truncate large strings and limit collections
        if let Some(ref mut expr) = context.current_expr {
            if expr.len() > 200 {
                expr.truncate(197);
                expr.push_str("...");
            }
        }

        // Limit number of bindings
        if context.bindings.len() > 20 {
            let keys: Vec<_> = context.bindings.keys().take(20).cloned().collect();
            context.bindings.retain(|k, _| keys.contains(k));
        }

        // Limit call chain
        if context.call_chain.len() > 10 {
            context.call_chain.truncate(10);
        }

        // Limit notes
        if context.notes.len() > 5 {
            context.notes.truncate(5);
        }

        context
    }

    /// Cleans up old contexts to stay within limits.
    fn cleanup_old_contexts(&self, contexts: &mut HashMap<DiagnosticId, PreservedErrorContext>) {
        if contexts.len() <= self.config.max_preserved_contexts {
            return;
        }

        let mut entries: Vec<_> = contexts.iter()
            .map(|(id, context)| (*id, context.preserved_at))
            .collect();
        entries.sort_by_key(|(_, time)| *time);
        
        let to_remove = contexts.len() - self.config.max_preserved_contexts;
        let ids_to_remove: Vec<_> = entries.iter()
            .take(to_remove)
            .map(|(id, _)| *id)
            .collect();
        
        for id in ids_to_remove {
            contexts.remove(&id);
        }
    }
}

impl DiagnosticGcWrapper {
    /// Creates a new GC wrapper for diagnostic information.
    pub fn new(error: PreservedError) -> Self {
        Self {
            preserved_error: error,
            generation: AtomicU32::new(0),
            marked: AtomicBool::new(false),
        }
    }

    /// Gets a reference to the preserved error.
    pub fn error(&self) -> &PreservedError {
        &self.preserved_error
    }
}

impl GcObject for DiagnosticGcWrapper {
    fn generation(&self) -> GenerationId {
        self.generation.load(Ordering::Relaxed)
    }

    fn set_generation(&mut self, generation: GenerationId) {
        self.generation.store(generation, Ordering::Relaxed);
    }

    fn references(&self) -> Vec<crate::utils::gc::GcPtr> {
        // No internal references for now
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
        // Estimate based on preserved error content
        let message_size = self.preserved_error.message.len();
        let context_size = self.estimate_context_size(&self.preserved_error.context);
        let base_size = std::mem::size_of::<PreservedError>();
        base_size + message_size + context_size + 128 // Extra overhead
    }
}

impl DiagnosticGcWrapper {
    /// Estimates the size of an error context.
    fn estimate_context_size(&self, context: &ErrorContext) -> usize {
        let mut size = 0;
        
        if let Some(ref expr) = context.current_expr {
            size += expr.len();
        }
        
        size += context.bindings.iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>();
            
        size += context.call_chain.iter()
            .map(|s| s.len())
            .sum::<usize>();
            
        if let Some(ref module) = context.module_context {
            size += module.len();
        }
        
        size += context.notes.iter()
            .map(|s| s.len())
            .sum::<usize>();
            
        size
    }
}

impl GcAwareError {
    /// Gets the diagnostic ID if this error is GC-tracked.
    pub fn diagnostic_id(&self) -> Option<DiagnosticId> {
        self.diagnostic_id
    }

    /// Gets the preserved error information if available.
    pub fn preserved_info(&self) -> Option<&PreservedError> {
        self.preserved_info.as_ref()
    }

    /// Gets the underlying error.
    pub fn inner_error(&self) -> &Error {
        &self.inner_error
    }
}

impl fmt::Display for GcAwareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner_error)
    }
}


impl Default for GcDiagnosticConfig {
    fn default() -> Self {
        Self {
            track_diagnostics: true,
            preserve_contexts: true,
            preserve_stack_traces: true,
            max_preserved_diagnostics: 100,
            compress_old_diagnostics: true,
        }
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_preserved_contexts: 200,
            compress_contexts: true,
            max_context_size: 4096, // 4KB max per context
        }
    }
}

/// Statistics about diagnostic preservation.
#[derive(Debug, Clone)]
pub struct DiagnosticStatistics {
    /// Number of active diagnostics
    pub active_diagnostics: usize,
    /// Total number of preserved diagnostics
    pub total_diagnostics: usize,
    /// Number of preserved error contexts
    pub preserved_contexts: usize,
    /// Number of diagnostics tracked by GC
    pub gc_roots_tracked: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::GcIntegration;
    use crate::diagnostics::Error;

    #[test]
    fn test_diagnostic_manager_creation() {
        let gc_integration = Arc::new(GcIntegration::with_default_config());
        let manager = GcDiagnosticManager::with_default_config(gc_integration);
        
        let stats = manager.get_diagnostic_statistics();
        assert_eq!(stats.active_diagnostics, 0);
        assert_eq!(stats.total_diagnostics, 0);
    }

    #[test]
    fn test_gc_aware_error_creation() {
        let gc_integration = Arc::new(GcIntegration::with_default_config());
        let manager = GcDiagnosticManager::with_default_config(gc_integration);
        
        let error = Error::runtime_error("test error".to_string(), None);
        let context = ErrorContext::default();
        
        let gc_error = manager.create_gc_aware_error(error, None, Some(context));
        
        assert!(gc_error.diagnostic_id().is_some());
        assert!(gc_error.preserved_info().is_some());
        
        let stats = manager.get_diagnostic_statistics();
        assert_eq!(stats.active_diagnostics, 1);
        assert_eq!(stats.total_diagnostics, 1);
    }

    #[test]
    fn test_error_context_preservation() {
        let context_manager = ErrorContextManager::new(ContextConfig::default());
        let diagnostic_id = DiagnosticId(1);
        
        let mut context = ErrorContext::default();
        context.current_expr = Some("(+ 1 2)".to_string());
        context.notes.push("Test note".to_string());
        
        let result = context_manager.preserve_context(diagnostic_id, context);
        assert!(result.is_ok());
        
        let preserved = context_manager.get_preserved_context(diagnostic_id);
        assert!(preserved.is_some());
        
        let preserved = preserved.unwrap();
        assert_eq!(preserved.context.current_expr, Some("(+ 1 2)".to_string()));
        assert_eq!(preserved.context.notes.len(), 1);
    }
}
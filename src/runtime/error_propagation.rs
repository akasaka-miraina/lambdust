//! Error propagation system for multithreaded evaluation.
//!
//! This module provides cross-thread error propagation with stack trace
//! preservation and proper error coordination between threads.

use crate::diagnostics::{Error as DiagnosticError, Result, Span};
use std::sync::{Arc, RwLock, Mutex};
use std::thread::ThreadId;
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, Duration};
use std::sync::atomic::{AtomicU64, Ordering};
use crossbeam::channel::{Sender, Receiver, unbounded};
use std::fmt;

/// Error propagation coordinator for multithreaded environments.
#[derive(Debug)]
pub struct ErrorPropagationCoordinator {
    /// Active error contexts by thread
    thread_error_contexts: Arc<RwLock<HashMap<ThreadId, ThreadErrorContext>>>,
    /// Cross-thread error propagation channels
    error_channels: Arc<RwLock<HashMap<ThreadId, ErrorChannel>>>,
    /// Error history for debugging and monitoring
    error_history: Arc<Mutex<VecDeque<ErrorEvent>>>,
    /// Error propagation policies
    policies: Arc<ErrorPropagationPolicies>,
    /// Error sequence counter
    sequence_counter: AtomicU64,
}

/// Error context for a specific thread.
#[derive(Debug, Clone)]
pub struct ThreadErrorContext {
    /// Thread ID
    pub thread_id: ThreadId,
    /// Current error stack for this thread
    pub error_stack: Vec<ThreadError>,
    /// Error propagation state
    pub propagation_state: ErrorPropagationState,
    /// Last error timestamp
    pub last_error_time: Option<SystemTime>,
    /// Error generation counter
    pub error_generation: u64,
}

/// Cross-thread communication channel for errors.
#[derive(Debug)]
pub struct ErrorChannel {
    /// Sender for error propagation messages
    pub sender: Sender<ErrorPropagationMessage>,
    /// Receiver for error propagation messages
    #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
    pub receiver: Receiver<ErrorPropagationMessage>,
}

/// An error with enhanced information for multithreaded environments.
#[derive(Debug, Clone)]
pub struct ThreadError {
    /// Unique error ID
    pub id: u64,
    /// The original diagnostic error
    pub diagnostic_error: DiagnosticError,
    /// Thread that originated this error
    pub originating_thread: ThreadId,
    /// Stack trace when error occurred
    pub stack_trace: Vec<StackFrame>,
    /// Cross-thread propagation path
    pub propagation_path: Vec<ThreadId>,
    /// When the error occurred
    pub occurred_at: SystemTime,
    /// Error severity level
    pub severity: ErrorSeverity,
    /// Error category
    pub category: ErrorCategory,
    /// Additional context information
    pub context: HashMap<String, String>,
}

/// Stack frame information for error tracing.
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Function or procedure name
    pub function_name: Option<String>,
    /// Source file information
    pub file_info: Option<String>,
    /// Line number if available
    pub line_number: Option<u32>,
    /// Column number if available
    pub column_number: Option<u32>,
    /// Expression span if available
    pub span: Option<Span>,
    /// Thread where this frame occurred
    pub thread_id: ThreadId,
    /// Additional frame context
    pub context: Option<String>,
}

/// State of error propagation for a thread.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorPropagationState {
    /// No errors, normal operation
    Normal,
    /// Error occurred but not yet propagated
    ErrorOccurred,
    /// Error is being propagated to other threads
    PropagatingError,
    /// Error propagation completed
    PropagationCompleted,
    /// Error was handled and resolved
    ErrorHandled,
    /// Error propagation failed
    PropagationFailed,
}

/// Severity level of an error.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorSeverity {
    /// Informational message
    Info,
    /// Warning that doesn't stop execution
    Warning,
    /// Error that affects current operation
    Error,
    /// Critical error that affects multiple threads
    Critical,
    /// Fatal error that requires shutdown
    Fatal,
}

/// Category of error for better organization.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Syntax or parsing error
    Syntax,
    /// Type system error
    Type,
    /// Runtime evaluation error
    Runtime,
    /// IO operation error
    IO,
    /// Effect system error
    Effect,
    /// Memory or resource error
    Resource,
    /// Concurrency or threading error
    Concurrency,
    /// System or platform error
    System,
    /// User-defined error
    User,
}

/// Event recording error occurrence and propagation.
#[derive(Debug, Clone)]
pub struct ErrorEvent {
    /// Event sequence number
    pub sequence: u64,
    /// Type of error event
    pub event_type: ErrorEventType,
    /// Thread where event occurred
    pub thread_id: ThreadId,
    /// Associated error
    pub error: ThreadError,
    /// Event timestamp
    pub timestamp: SystemTime,
    /// Additional event context
    pub context: Option<String>,
}

/// Types of error events.
#[derive(Debug, Clone)]
pub enum ErrorEventType {
    /// Error occurred in a thread
    ErrorOccurred,
    /// Error is being propagated
    ErrorPropagating,
    /// Error propagation completed
    PropagationCompleted,
    /// Error was handled
    ErrorHandled,
    /// Error propagation failed
    PropagationFailed,
    /// Error caused thread shutdown
    ThreadShutdown,
}

/// Messages for cross-thread error propagation.
#[derive(Debug, Clone)]
pub enum ErrorPropagationMessage {
    /// Propagate an error to other threads
    PropagateError {
        #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
        error: ThreadError,
        #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
        target_threads: Vec<ThreadId>,
        #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
        propagation_strategy: PropagationStrategy,
    },
    /// Acknowledge error reception
    #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
    ErrorAcknowledged {
        error_id: u64,
        acknowledging_thread: ThreadId,
    },
    /// Report error handling completion
    #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
    ErrorHandled {
        error_id: u64,
        handling_thread: ThreadId,
        result: std::result::Result<(), String>,
    },
    /// Request error context from a thread
    #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
    RequestErrorContext {
        requesting_thread: ThreadId,
    },
    /// Response with error context
    #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
    ErrorContextResponse {
        context: ThreadErrorContext,
    },
    /// Shutdown notification due to fatal error
    FatalErrorShutdown {
        #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
        error: ThreadError,
        #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
        message: String,
    },
}

/// Strategy for propagating errors across threads.
#[derive(Debug, Clone)]
pub enum PropagationStrategy {
    /// Propagate to all threads
    Broadcast,
    /// Propagate to specific threads only
    #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
    Targeted(Vec<ThreadId>),
    /// Propagate to parent thread only
    Parent,
    /// Propagate based on error severity
    SeverityBased,
    /// Custom propagation logic
    #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
    Custom(fn(&ThreadError) -> Vec<ThreadId>),
}

/// Policies for error propagation behavior.
#[derive(Debug)]
pub struct ErrorPropagationPolicies {
    /// Whether to track error history
    track_error_history: bool,
    /// Maximum size of error history
    max_error_history_size: usize,
    /// Whether to preserve full stack traces
    preserve_stack_traces: bool,
    /// Maximum stack trace depth
    #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
    max_stack_trace_depth: usize,
    /// Whether to enable cross-thread propagation
    enable_cross_thread_propagation: bool,
    /// Default propagation strategy
    default_propagation_strategy: PropagationStrategy,
    /// Whether fatal errors should shutdown all threads
    fatal_errors_shutdown_all: bool,
    /// Timeout for error propagation
    #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
    propagation_timeout: Duration,
    /// Whether to enable error recovery
    #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
    enable_error_recovery: bool,
    /// Maximum number of errors before thread shutdown
    #[allow(dead_code)] // Part of Stage 3 error propagation infrastructure
    max_errors_before_shutdown: usize,
}

impl ErrorPropagationCoordinator {
    /// Creates a new error propagation coordinator.
    pub fn new() -> Self {
        Self {
            thread_error_contexts: Arc::new(RwLock::new(HashMap::new())),
            error_channels: Arc::new(RwLock::new(HashMap::new())),
            error_history: Arc::new(Mutex::new(VecDeque::new())),
            policies: Arc::new(ErrorPropagationPolicies::default()),
            sequence_counter: AtomicU64::new(0),
        }
    }

    /// Creates an error propagation coordinator with custom policies.
    pub fn with_policies(policies: ErrorPropagationPolicies) -> Self {
        Self {
            thread_error_contexts: Arc::new(RwLock::new(HashMap::new())),
            error_channels: Arc::new(RwLock::new(HashMap::new())),
            error_history: Arc::new(Mutex::new(VecDeque::new())),
            policies: Arc::new(policies),
            sequence_counter: AtomicU64::new(0),
        }
    }

    /// Registers a thread with the error propagation coordinator.
    pub fn register_thread(&self, thread_id: ThreadId) {
        // Create error context for this thread
        let context = ThreadErrorContext {
            thread_id,
            error_stack: Vec::new(),
            propagation_state: ErrorPropagationState::Normal,
            last_error_time: None,
            error_generation: 0,
        };

        {
            let mut contexts = self.thread_error_contexts.write().unwrap();
            contexts.insert(thread_id, context);
        }

        // Create error channel for this thread
        let (sender, receiver) = unbounded();
        let channel = ErrorChannel { sender, receiver };

        {
            let mut channels = self.error_channels.write().unwrap();
            channels.insert(thread_id, channel);
        }
    }

    /// Unregisters a thread from the error propagation coordinator.
    pub fn unregister_thread(&self, thread_id: ThreadId) {
        {
            let mut contexts = self.thread_error_contexts.write().unwrap();
            contexts.remove(&thread_id);
        }

        {
            let mut channels = self.error_channels.write().unwrap();
            channels.remove(&thread_id);
        }
    }

    /// Reports an error from a thread with enhanced context.
    pub fn report_error(
        &self,
        thread_id: ThreadId,
        diagnostic_error: DiagnosticError,
        context: Option<HashMap<String, String>>,
    ) -> Result<u64> {
        let error_id = self.sequence_counter.fetch_add(1, Ordering::SeqCst) + 1;

        // Capture stack trace
        let stack_trace = if self.policies.preserve_stack_traces {
            self.capture_stack_trace(thread_id)?
        } else {
            Vec::new()
        };

        // Determine error severity and category
        let severity = self.determine_error_severity(&diagnostic_error);
        let category = self.determine_error_category(&diagnostic_error);

        // Create enhanced error
        let thread_error = ThreadError {
            id: error_id,
            diagnostic_error,
            originating_thread: thread_id,
            stack_trace,
            propagation_path: vec![thread_id],
            occurred_at: SystemTime::now(),
            severity,
            category,
            context: context.unwrap_or_default(),
        };

        // Update thread error context
        {
            let mut contexts = self.thread_error_contexts.write().unwrap();
            if let Some(context) = contexts.get_mut(&thread_id) {
                context.error_stack.push(thread_error.clone());
                context.propagation_state = ErrorPropagationState::ErrorOccurred;
                context.last_error_time = Some(SystemTime::now());
                context.error_generation += 1;
            }
        }

        // Record error event
        if self.policies.track_error_history {
            self.record_error_event(ErrorEventType::ErrorOccurred, thread_id, thread_error.clone()), None);
        }

        // Determine if error should be propagated
        if self.policies.enable_cross_thread_propagation && self.should_propagate_error(&thread_error) {
            self.initiate_error_propagation(thread_error.clone())?;
        }

        // Check if this is a fatal error requiring shutdown
        if thread_error.severity == ErrorSeverity::Fatal && self.policies.fatal_errors_shutdown_all {
            self.initiate_fatal_error_shutdown(thread_error)?;
        }

        Ok(error_id)
    }

    /// Initiates error propagation to other threads.
    fn initiate_error_propagation(&self, error: ThreadError) -> Result<()> {
        let target_threads = self.determine_propagation_targets(&error);
        
        if target_threads.is_empty() {
            return Ok(());
        }

        // Update propagation state
        {
            let mut contexts = self.thread_error_contexts.write().unwrap();
            if let Some(context) = contexts.get_mut(&error.originating_thread) {
                context.propagation_state = ErrorPropagationState::PropagatingError;
            }
        }

        // Send propagation messages
        {
            let channels = self.error_channels.read().unwrap();
            let message = ErrorPropagationMessage::PropagateError {
                error: error.clone()),
                target_threads: target_threads.clone()),
                propagation_strategy: self.policies.default_propagation_strategy.clone()),
            };

            for &target_thread in &target_threads {
                if let Some(channel) = channels.get(&target_thread) {
                    if let Err(_) = channel.sender.try_send(message.clone()) {
                        // Target thread is not responsive, continue with others
                        eprintln!("Warning: Could not propagate error to thread {:?}", target_thread);
                    }
                }
            }
        }

        // Record propagation event
        if self.policies.track_error_history {
            self.record_error_event(
                ErrorEventType::ErrorPropagating,
                error.originating_thread,
                error,
                Some(format!("Propagating to {} threads", target_threads.len())),
            );
        }

        Ok(())
    }

    /// Handles a fatal error by initiating shutdown.
    fn initiate_fatal_error_shutdown(&self, error: ThreadError) -> Result<()> {
        let message = ErrorPropagationMessage::FatalErrorShutdown {
            error: error.clone()),
            message: format!("Fatal error in thread {:?}: {}", error.originating_thread, error.diagnostic_error),
        };

        // Broadcast shutdown message to all threads
        {
            let channels = self.error_channels.read().unwrap();
            for channel in channels.values() {
                let _ = channel.sender.try_send(message.clone());
            }
        }

        // Record shutdown event
        if self.policies.track_error_history {
            self.record_error_event(
                ErrorEventType::ThreadShutdown,
                error.originating_thread,
                error,
                Some("Fatal error initiated shutdown".to_string()),
            );
        }

        Ok(())
    }

    /// Captures the current stack trace for a thread.
    fn capture_stack_trace(&self, thread_id: ThreadId) -> Result<Vec<StackFrame>> {
        // In a real implementation, this would capture the actual call stack
        // For now, we'll create a placeholder frame
        let frame = StackFrame {
            function_name: Some("unknown".to_string()),
            file_info: None,
            line_number: None,
            column_number: None,
            span: None,
            thread_id,
            context: Some("Stack trace capture".to_string()),
        };

        Ok(vec![frame])
    }

    /// Determines if an error should be propagated to other threads.
    fn should_propagate_error(&self, error: &ThreadError) -> bool {
        match error.severity {
            ErrorSeverity::Critical | ErrorSeverity::Fatal => true,
            ErrorSeverity::Error => error.category == ErrorCategory::Concurrency,
            _ => false,
        }
    }

    /// Determines which threads should receive error propagation.
    fn determine_propagation_targets(&self, error: &ThreadError) -> Vec<ThreadId> {
        match &self.policies.default_propagation_strategy {
            PropagationStrategy::Broadcast => {
                let contexts = self.thread_error_contexts.read().unwrap();
                contexts.keys().filter(|&&tid| tid != error.originating_thread).copied().collect()
            }
            PropagationStrategy::Targeted(threads) => threads.clone()),
            PropagationStrategy::Parent => {
                // For simplicity, assume no parent relationship
                Vec::new()
            }
            PropagationStrategy::SeverityBased => {
                if error.severity >= ErrorSeverity::Critical {
                    let contexts = self.thread_error_contexts.read().unwrap();
                    contexts.keys().filter(|&&tid| tid != error.originating_thread).copied().collect()
                } else {
                    Vec::new()
                }
            }
            PropagationStrategy::Custom(_) => {
                // For simplicity, return empty vector
                Vec::new()
            }
        }
    }

    /// Determines the severity of a diagnostic error.
    fn determine_error_severity(&self, error: &DiagnosticError) -> ErrorSeverity {
        // Simple heuristic based on error message
        let message = error.to_string().to_lowercase();
        
        if message.contains("fatal") || message.contains("panic") {
            ErrorSeverity::Fatal
        } else if message.contains("critical") || message.contains("thread") {
            ErrorSeverity::Critical
        } else if message.contains("warning") {
            ErrorSeverity::Warning
        } else {
            ErrorSeverity::Error
        }
    }

    /// Determines the category of a diagnostic error.
    fn determine_error_category(&self, error: &DiagnosticError) -> ErrorCategory {
        let message = error.to_string().to_lowercase();
        
        if message.contains("syntax") || message.contains("parse") {
            ErrorCategory::Syntax
        } else if message.contains("type") {
            ErrorCategory::Type
        } else if message.contains("io") || message.contains("file") {
            ErrorCategory::IO
        } else if message.contains("effect") {
            ErrorCategory::Effect
        } else if message.contains("memory") || message.contains("resource") {
            ErrorCategory::Resource
        } else if message.contains("thread") || message.contains("concurrency") {
            ErrorCategory::Concurrency
        } else if message.contains("system") {
            ErrorCategory::System
        } else {
            ErrorCategory::Runtime
        }
    }

    /// Records an error event.
    fn record_error_event(
        &self,
        event_type: ErrorEventType,
        thread_id: ThreadId,
        error: ThreadError,
        context: Option<String>,
    ) {
        let event = ErrorEvent {
            sequence: self.sequence_counter.fetch_add(1, Ordering::SeqCst) + 1,
            event_type,
            thread_id,
            error,
            timestamp: SystemTime::now(),
            context,
        };

        let mut history = self.error_history.lock().unwrap();
        history.push_back(event);

        // Trim history if it gets too large
        if history.len() > self.policies.max_error_history_size {
            history.pop_front();
        }
    }

    /// Gets error statistics across all threads.
    pub fn get_error_statistics(&self) -> ErrorStatistics {
        let contexts = self.thread_error_contexts.read().unwrap();
        let history = self.error_history.lock().unwrap();

        let mut stats = ErrorStatistics {
            active_threads: contexts.len(),
            total_errors: 0,
            errors_by_severity: HashMap::new(),
            errors_by_category: HashMap::new(),
            threads_with_errors: 0,
            recent_errors: 0,
            propagated_errors: 0,
        };

        // Count errors by thread
        for context in contexts.values() {
            if !context.error_stack.is_empty() {
                stats.threads_with_errors += 1;
                stats.total_errors += context.error_stack.len();

                for error in &context.error_stack {
                    *stats.errors_by_severity.entry(error.severity.clone()).or_insert(0) += 1;
                    *stats.errors_by_category.entry(error.category.clone()).or_insert(0) += 1;
                }
            }
        }

        // Count recent errors (last minute)
        let one_minute_ago = SystemTime::now() - Duration::from_secs(60);
        stats.recent_errors = history
            .iter()
            .filter(|event| event.timestamp > one_minute_ago)
            .count();

        // Count propagated errors
        stats.propagated_errors = history
            .iter()
            .filter(|event| matches!(event.event_type, ErrorEventType::ErrorPropagating))
            .count();

        stats
    }

    /// Gets the error history.
    pub fn get_error_history(&self) -> Vec<ErrorEvent> {
        let history = self.error_history.lock().unwrap();
        history.iter().clone())().collect()
    }

    /// Gets the error context for a specific thread.
    pub fn get_thread_error_context(&self, thread_id: ThreadId) -> Option<ThreadErrorContext> {
        let contexts = self.thread_error_contexts.read().unwrap();
        contexts.get(&thread_id).clone())()
    }

    /// Clears error history.
    pub fn clear_error_history(&self) {
        let mut history = self.error_history.lock().unwrap();
        history.clear();
    }

    /// Clears errors for a specific thread.
    pub fn clear_thread_errors(&self, thread_id: ThreadId) -> Result<()> {
        let mut contexts = self.thread_error_contexts.write().unwrap();
        if let Some(context) = contexts.get_mut(&thread_id) {
            context.error_stack.clear();
            context.propagation_state = ErrorPropagationState::Normal;
            context.last_error_time = None;
        }
        Ok(())
    }
}

/// Statistics about error occurrence and propagation.
#[derive(Debug, Clone)]
pub struct ErrorStatistics {
    /// Number of active threads
    pub active_threads: usize,
    /// Total number of errors across all threads
    pub total_errors: usize,
    /// Count of errors by severity
    pub errors_by_severity: HashMap<ErrorSeverity, usize>,
    /// Count of errors by category
    pub errors_by_category: HashMap<ErrorCategory, usize>,
    /// Number of threads that have experienced errors
    pub threads_with_errors: usize,
    /// Number of errors in the last minute
    pub recent_errors: usize,
    /// Number of errors that were propagated
    pub propagated_errors: usize,
}

impl Default for ErrorPropagationPolicies {
    fn default() -> Self {
        Self {
            track_error_history: true,
            max_error_history_size: 1000,
            preserve_stack_traces: true,
            max_stack_trace_depth: 50,
            enable_cross_thread_propagation: true,
            default_propagation_strategy: PropagationStrategy::SeverityBased,
            fatal_errors_shutdown_all: true,
            propagation_timeout: Duration::from_secs(5),
            enable_error_recovery: true,
            max_errors_before_shutdown: 10,
        }
    }
}

impl ErrorPropagationPolicies {
    /// Creates policies with minimal overhead.
    pub fn minimal() -> Self {
        Self {
            track_error_history: false,
            max_error_history_size: 100,
            preserve_stack_traces: false,
            max_stack_trace_depth: 10,
            enable_cross_thread_propagation: false,
            default_propagation_strategy: PropagationStrategy::Parent,
            fatal_errors_shutdown_all: false,
            propagation_timeout: Duration::from_millis(100),
            enable_error_recovery: false,
            max_errors_before_shutdown: 5,
        }
    }

    /// Creates policies optimized for debugging.
    pub fn debug() -> Self {
        Self {
            track_error_history: true,
            max_error_history_size: 5000,
            preserve_stack_traces: true,
            max_stack_trace_depth: 100,
            enable_cross_thread_propagation: true,
            default_propagation_strategy: PropagationStrategy::Broadcast,
            fatal_errors_shutdown_all: true,
            propagation_timeout: Duration::from_secs(30),
            enable_error_recovery: true,
            max_errors_before_shutdown: 100,
        }
    }
}

impl Clone for ErrorChannel {
    fn clone(&self) -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}

impl Default for ErrorPropagationCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
            ErrorSeverity::Fatal => write!(f, "FATAL"),
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Syntax => write!(f, "Syntax"),
            ErrorCategory::Type => write!(f, "Type"),
            ErrorCategory::Runtime => write!(f, "Runtime"),
            ErrorCategory::IO => write!(f, "IO"),
            ErrorCategory::Effect => write!(f, "Effect"),
            ErrorCategory::Resource => write!(f, "Resource"),
            ErrorCategory::Concurrency => write!(f, "Concurrency"),
            ErrorCategory::System => write!(f, "System"),
            ErrorCategory::User => write!(f, "User"),
        }
    }
}
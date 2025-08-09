//! Domain services for continuation management in Lambdust.
//!
//! This module implements the core domain logic for continuation capture,
//! restoration, and composition according to Domain-Driven Design principles.
//!
//! Key responsibilities are separated into focused domain services:
//! - ContinuationCapture: Captures evaluation contexts as continuations
//! - ContinuationApplication: Applies captured continuations (non-local jumps)
//! - ContinuationComposition: Composes and transforms continuations

use crate::eval::operational_semantics::{
    EvaluationContext, ComputationState, Redex, ContextFrame, MachineState,
    RedexMetadata,
};
use crate::eval::{Value, Environment};
use crate::ast::{Expr, Spanned};
use crate::diagnostics::{Result, Error, Span};
use std::sync::Arc;
use std::rc::Rc;
use std::collections::HashMap;

/// Domain service responsible for capturing continuations.
///
/// This encapsulates the complex logic of "reifying" an evaluation context
/// into a first-class continuation value that can be manipulated by Scheme code.
#[derive(Debug, Clone)]
pub struct ContinuationCaptureService {
    /// Configuration for capture behavior
    capture_config: CaptureConfiguration,
}

/// Configuration for continuation capture behavior
#[derive(Debug, Clone)]
pub struct CaptureConfiguration {
    /// Maximum stack depth to capture (prevents infinite recursion)
    max_capture_depth: usize,
    
    /// Whether to capture environment bindings (affects memory usage)
    capture_environment: bool,
    
    /// Whether to enable single-shot semantics (continuation can only be used once)
    single_shot_semantics: bool,
    
    /// Whether to enable tail call optimization in captured continuations
    optimize_tail_calls: bool,
}

/// A captured continuation - the domain entity representing "the rest of the computation".
///
/// This is the reified form of an evaluation context that can be stored,
/// passed around, and eventually applied to perform non-local jumps.
#[derive(Debug, Clone)]
pub struct CapturedContinuation {
    /// Unique identifier for this continuation
    pub id: ContinuationId,
    
    /// The captured evaluation context
    pub context: EvaluationContext,
    
    /// Metadata about the continuation
    pub metadata: ContinuationMetadata,
    
    /// Whether this continuation has been invoked (for single-shot semantics)
    pub is_invoked: bool,
    
    /// The environment where the continuation was captured
    pub captured_environment: Arc<super::value::ThreadSafeEnvironment>,
}

/// Unique identifier for continuations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContinuationId(pub u64);

/// Metadata associated with a captured continuation
#[derive(Debug, Clone)]
pub struct ContinuationMetadata {
    /// Where the continuation was captured
    pub capture_location: Span,
    
    /// Stack depth at capture time
    pub capture_depth: usize,
    
    /// Generation for garbage collection
    pub generation: u64,
    
    /// Whether this continuation is in tail position
    pub is_tail_continuation: bool,
    
    /// Optional debug name
    pub debug_name: Option<String>,
}

/// Domain service for applying captured continuations.
///
/// This handles the complex operational semantics of non-local jumps,
/// including proper environment restoration and stack management.
#[derive(Debug, Clone)]
pub struct ContinuationApplicationService {
    /// Configuration for application behavior
    application_config: ApplicationConfiguration,
}

/// Configuration for continuation application
#[derive(Debug, Clone)]
pub struct ApplicationConfiguration {
    /// Whether to validate continuation before application
    validate_before_apply: bool,
    
    /// Whether to restore environments on application
    restore_environments: bool,
    
    /// Maximum number of applications for a single continuation
    max_applications: Option<usize>,
}

/// Result of applying a continuation
#[derive(Debug, Clone)]
pub enum ContinuationApplicationResult {
    /// Successful application - continue with new state
    Success {
        /// The new computation state after application
        new_state: ComputationState,
        
        /// The value that was passed to the continuation
        applied_value: Value,
    },
    
    /// Application resulted in final value (end of computation)
    FinalValue {
        /// The final result value
        value: Value,
    },
    
    /// Error during application
    Error {
        /// The error that occurred
        error: Error,
        
        /// The continuation that failed to apply
        failed_continuation: CapturedContinuation,
    },
}

/// Domain service for composing and transforming continuations.
///
/// This provides advanced continuation manipulation capabilities
/// for implementing sophisticated control flow constructs.
#[derive(Debug, Clone)]
pub struct ContinuationCompositionService {
    /// Configuration for composition operations
    composition_config: CompositionConfiguration,
}

/// Configuration for continuation composition
#[derive(Debug, Clone)]
pub struct CompositionConfiguration {
    /// Maximum composition depth (prevents infinite composition chains)
    max_composition_depth: usize,
    
    /// Whether to optimize composed continuations
    optimize_compositions: bool,
}

/// Types of continuation composition
pub enum CompositionType {
    /// Sequential composition: first continuation, then second
    Sequential,
    
    /// Parallel composition (for concurrent evaluation)
    Parallel,
    
    /// Conditional composition (choose based on a predicate)
    Conditional(Box<dyn Fn(&Value) -> bool + Send + Sync>),
    
    /// Loop composition (for implementing loops)
    Loop {
        /// The condition function to test for loop continuation.
        condition: Box<dyn Fn(&Value) -> bool + Send + Sync>,
        /// The continuation body to execute in each loop iteration.
        body: Box<CapturedContinuation>,
    },
}

impl std::fmt::Debug for CompositionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompositionType::Sequential => write!(f, "Sequential"),
            CompositionType::Parallel => write!(f, "Parallel"),
            CompositionType::Conditional(_) => write!(f, "Conditional(<function>)"),
            CompositionType::Loop { body, .. } => write!(f, "Loop {{ condition: <function>, body: {body:?} }}"),
        }
    }
}

impl Clone for CompositionType {
    fn clone(&self) -> Self {
        match self {
            CompositionType::Sequential => CompositionType::Sequential,
            CompositionType::Parallel => CompositionType::Parallel,
            CompositionType::Conditional(_) => {
                // For function closures, we can't clone them directly
                // In practice, we'd need to store a unique identifier instead
                CompositionType::Sequential // Fallback for now
            },
            CompositionType::Loop { condition: _, body } => {
                CompositionType::Loop {
                    condition: Box::new(|_| false), // Fallback condition
                    body: Box::new(*body.clone()),
                }
            },
        }
    }
}

/// Repository trait for persisting and retrieving continuations
///
/// This follows DDD patterns by providing a clean interface for
/// continuation storage without exposing infrastructure concerns.
pub trait ContinuationRepository {
    /// Store a continuation and return its ID
    fn store(&mut self, continuation: CapturedContinuation) -> Result<ContinuationId>;
    
    /// Retrieve a continuation by ID
    fn find_by_id(&self, id: ContinuationId) -> Option<CapturedContinuation>;
    
    /// Remove a continuation from storage
    fn remove(&mut self, id: ContinuationId) -> Result<()>;
    
    /// List all stored continuations (for debugging)
    fn list_all(&self) -> Vec<ContinuationId>;
    
    /// Clean up expired or unused continuations
    fn garbage_collect(&mut self, current_generation: u64) -> Result<usize>;
}

/// Counter for generating unique continuation IDs
static CONTINUATION_ID_COUNTER: std::sync::atomic::AtomicU64 = 
    std::sync::atomic::AtomicU64::new(1);

fn next_continuation_id() -> ContinuationId {
    ContinuationId(CONTINUATION_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
}

impl Default for ContinuationCaptureService {
    fn default() -> Self {
        Self::new()
    }
}

impl ContinuationCaptureService {
    /// Create a new continuation capture service with default configuration
    pub fn new() -> Self {
        Self {
            capture_config: CaptureConfiguration::default(),
        }
    }
    
    /// Create a capture service with custom configuration
    pub fn with_config(config: CaptureConfiguration) -> Self {
        Self {
            capture_config: config,
        }
    }
    
    /// Capture a continuation from the current evaluation context.
    ///
    /// This implements the core semantics of call/cc by "reifying" the
    /// evaluation context into a first-class continuation value.
    pub fn capture_continuation(
        &self,
        context: &EvaluationContext,
        capture_location: Span,
        generation: u64,
    ) -> Result<CapturedContinuation> {
        // Validate capture depth
        if context.depth() > self.capture_config.max_capture_depth {
            return Err(Box::new(Error::runtime_error(
                format!(
                    "Continuation capture depth {} exceeds maximum {}",
                    context.depth(),
                    self.capture_config.max_capture_depth
                ),
                Some(capture_location),
            )));
        }
        
        let id = next_continuation_id();
        
        let metadata = ContinuationMetadata {
            capture_location,
            capture_depth: context.depth(),
            generation,
            is_tail_continuation: context.is_empty(),
            debug_name: None,
        };
        
        Ok(CapturedContinuation {
            id,
            context: context.clone(),
            metadata,
            is_invoked: false,
            captured_environment: context.environment().clone(),
        })
    }
    
    /// Capture a continuation with a debug name
    pub fn capture_named_continuation(
        &self,
        context: &EvaluationContext,
        capture_location: Span,
        generation: u64,
        debug_name: String,
    ) -> Result<CapturedContinuation> {
        let mut continuation = self.capture_continuation(context, capture_location, generation)?;
        continuation.metadata.debug_name = Some(debug_name);
        Ok(continuation)
    }
    
    /// Check if a context can be safely captured
    pub fn can_capture(&self, context: &EvaluationContext) -> bool {
        context.depth() <= self.capture_config.max_capture_depth
    }
    
    /// Get the capture configuration
    pub fn config(&self) -> &CaptureConfiguration {
        &self.capture_config
    }
}

impl Default for ContinuationApplicationService {
    fn default() -> Self {
        Self::new()
    }
}

impl ContinuationApplicationService {
    /// Create a new continuation application service
    pub fn new() -> Self {
        Self {
            application_config: ApplicationConfiguration::default(),
        }
    }
    
    /// Create an application service with custom configuration
    pub fn with_config(config: ApplicationConfiguration) -> Self {
        Self {
            application_config: config,
        }
    }
    
    /// Apply a captured continuation to a value.
    ///
    /// This implements the non-local jump semantics by restoring the
    /// captured evaluation context and continuing computation with the given value.
    pub fn apply_continuation(
        &self,
        mut continuation: CapturedContinuation,
        value: Value,
    ) -> Result<ContinuationApplicationResult> {
        // Validate continuation before application
        if self.application_config.validate_before_apply {
            if let Err(err) = self.validate_continuation(&continuation) {
                return Ok(ContinuationApplicationResult::Error {
                    error: *err,
                    failed_continuation: continuation,
                });
            }
        }
        
        // Check single-shot semantics
        if continuation.is_invoked {
            return Ok(ContinuationApplicationResult::Error {
                error: Error::runtime_error(
                    "Continuation has already been invoked".to_string(),
                    Some(continuation.metadata.capture_location),
                ),
                failed_continuation: continuation,
            });
        }
        
        // Mark as invoked
        continuation.is_invoked = true;
        
        // Apply the context to the value
        match continuation.context.apply_to_value(value.clone()) {
            Ok(new_state) => {
                if new_state.context.is_empty() {
                    // We've reached the top level - return final value
                    Ok(ContinuationApplicationResult::FinalValue { value })
                } else {
                    // Continue with the new state
                    Ok(ContinuationApplicationResult::Success {
                        new_state,
                        applied_value: value,
                    })
                }
            }
            Err(error) => Ok(ContinuationApplicationResult::Error {
                error: *error,
                failed_continuation: continuation,
            }),
        }
    }
    
    /// Validate a continuation before application
    fn validate_continuation(&self, continuation: &CapturedContinuation) -> Result<()> {
        if continuation.is_invoked {
            return Err(Box::new(Error::runtime_error(
                "Cannot apply already-invoked continuation".to_string(),
                Some(continuation.metadata.capture_location),
            )));
        }
        
        // Additional validation checks could be added here
        // (e.g., environment validity, stack depth limits)
        
        Ok(())
    }
    
    /// Get the application configuration
    pub fn config(&self) -> &ApplicationConfiguration {
        &self.application_config
    }
}

impl Default for ContinuationCompositionService {
    fn default() -> Self {
        Self::new()
    }
}

impl ContinuationCompositionService {
    /// Create a new continuation composition service
    pub fn new() -> Self {
        Self {
            composition_config: CompositionConfiguration::default(),
        }
    }
    
    /// Compose two continuations sequentially
    pub fn compose_sequential(
        &self,
        first: CapturedContinuation,
        second: CapturedContinuation,
    ) -> Result<CapturedContinuation> {
        // Check composition depth
        let total_depth = first.context.depth() + second.context.depth();
        if total_depth > self.composition_config.max_composition_depth {
            return Err(Box::new(Error::runtime_error(
                format!(
                    "Composition depth {} exceeds maximum {}",
                    total_depth,
                    self.composition_config.max_composition_depth
                ),
                None,
            )));
        }
        
        // Compose the contexts
        let composed_context = first.context.compose(second.context);
        
        let id = next_continuation_id();
        let metadata = ContinuationMetadata {
            capture_location: first.metadata.capture_location,
            capture_depth: composed_context.depth(),
            generation: first.metadata.generation.max(second.metadata.generation),
            is_tail_continuation: false, // Compositions are typically not tail
            debug_name: Some(format!(
                "composed({:?}, {:?})",
                first.metadata.debug_name.as_deref().unwrap_or("anonymous"),
                second.metadata.debug_name.as_deref().unwrap_or("anonymous")
            )),
        };
        
        Ok(CapturedContinuation {
            id,
            context: composed_context,
            metadata,
            is_invoked: false,
            captured_environment: first.captured_environment, // Use first's environment
        })
    }
    
    /// Transform a continuation by applying a function to its context
    pub fn transform_continuation<F>(
        &self,
        continuation: CapturedContinuation,
        transformer: F,
    ) -> Result<CapturedContinuation>
    where
        F: FnOnce(EvaluationContext) -> Result<EvaluationContext>,
    {
        let transformed_context = transformer(continuation.context)?;
        
        let id = next_continuation_id();
        let metadata = ContinuationMetadata {
            capture_location: continuation.metadata.capture_location,
            capture_depth: transformed_context.depth(),
            generation: continuation.metadata.generation,
            is_tail_continuation: transformed_context.is_empty(),
            debug_name: Some(format!(
                "transformed({})",
                continuation.metadata.debug_name.as_deref().unwrap_or("anonymous")
            )),
        };
        
        Ok(CapturedContinuation {
            id,
            context: transformed_context,
            metadata,
            is_invoked: false,
            captured_environment: continuation.captured_environment,
        })
    }
    
    /// Get the composition configuration
    pub fn config(&self) -> &CompositionConfiguration {
        &self.composition_config
    }
}

// Default implementations

impl Default for CaptureConfiguration {
    fn default() -> Self {
        Self {
            max_capture_depth: 1000,
            capture_environment: true,
            single_shot_semantics: true,
            optimize_tail_calls: true,
        }
    }
}

impl Default for ApplicationConfiguration {
    fn default() -> Self {
        Self {
            validate_before_apply: true,
            restore_environments: true,
            max_applications: Some(1), // Single-shot by default
        }
    }
}

impl Default for CompositionConfiguration {
    fn default() -> Self {
        Self {
            max_composition_depth: 2000,
            optimize_compositions: true,
        }
    }
}

// Utility functions for debugging and introspection

impl CapturedContinuation {
    /// Get a debug representation of this continuation
    pub fn debug_info(&self) -> String {
        let base = format!(
            "Continuation({}) [depth: {}, tail: {}, invoked: {}]",
            self.id.0,
            self.metadata.capture_depth,
            self.metadata.is_tail_continuation,
            self.is_invoked
        );
        
        if let Some(ref name) = self.metadata.debug_name {
            format!("{base} '{name}'")
        } else {
            base
        }
    }
    
    /// Check if this continuation is still valid for application
    pub fn is_valid(&self) -> bool {
        !self.is_invoked
    }
    
    /// Get the continuation's capture location for error reporting
    pub fn capture_location(&self) -> Span {
        self.metadata.capture_location
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::operational_semantics::EvaluationContext;
    
    #[test]
    fn test_continuation_capture() {
        let env = Rc::new(Environment::new(None, 0));
        let context = EvaluationContext::empty(env);
        let capture_service = ContinuationCaptureService::new();
        
        let continuation = capture_service.capture_continuation(
            &context,
            Span::default(),
            0,
        ).unwrap();
        
        assert!(continuation.is_valid());
        assert_eq!(continuation.metadata.capture_depth, 0);
        assert!(continuation.metadata.is_tail_continuation);
    }
    
    #[test]
    fn test_continuation_application() {
        let env = Rc::new(Environment::new(None, 0));
        let context = EvaluationContext::empty(env);
        let capture_service = ContinuationCaptureService::new();
        let application_service = ContinuationApplicationService::new();
        
        let continuation = capture_service.capture_continuation(
            &context,
            Span::default(),
            0,
        ).unwrap();
        
        let value = Value::number(42.0);
        let result = application_service.apply_continuation(continuation, value).unwrap();
        
        match result {
            ContinuationApplicationResult::FinalValue { value } => {
                assert_eq!(value, Value::number(42.0));
            }
            _ => panic!("Expected final value"),
        }
    }
    
    #[test]
    fn test_continuation_composition() {
        let env = Rc::new(Environment::new(None, 0));
        let context1 = EvaluationContext::empty(env.clone());
        let context2 = EvaluationContext::empty(env.clone());
        
        let capture_service = ContinuationCaptureService::new();
        let composition_service = ContinuationCompositionService::new();
        
        let cont1 = capture_service.capture_continuation(
            &context1,
            Span::default(),
            0,
        ).unwrap();
        
        let cont2 = capture_service.capture_continuation(
            &context2,
            Span::default(),
            0,
        ).unwrap();
        
        let composed = composition_service.compose_sequential(cont1, cont2).unwrap();
        assert!(composed.is_valid());
        assert!(composed.metadata.debug_name.is_some());
    }
}

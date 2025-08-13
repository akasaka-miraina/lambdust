//! GC integration for the macro expansion system.
//!
//! This module provides GC-aware macro expansion that properly tracks
//! macro expansion contexts, syntax transformers, and temporary values
//! created during expansion without affecting hygienic macro semantics.

use crate::utils::{GcIntegration, GcIntegrationConfig};
use crate::utils::gc::{ObjectId, gc_alloc};
use crate::macro_system::{MacroExpander, MacroTransformer, HygieneContext, next_hygiene_id};
use crate::eval::{Value, ThreadSafeEnvironment};
use crate::eval::gc_coordinator::GcCoordinator;
use crate::ast::{Expr};
use crate::diagnostics::{Result, Error, Span, Spanned};
use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use std::time::Instant;

/// GC-aware macro expansion coordinator that integrates with the garbage collector
/// while preserving hygienic macro semantics and R7RS compliance.
#[derive(Debug)]
pub struct GcMacroCoordinator {
    /// Underlying macro expander
    expander: Arc<Mutex<MacroExpander>>,
    /// GC integration manager
    gc_integration: Arc<GcIntegration>,
    /// Active expansion contexts tracked for GC
    expansion_contexts: RwLock<HashMap<ExpansionId, ExpansionContext>>,
    /// Macro transformer registry for GC root tracking
    transformer_registry: RwLock<HashMap<String, TransformerEntry>>,
    /// Configuration
    config: GcMacroConfig,
    /// Next expansion ID
    next_expansion_id: std::sync::atomic::AtomicU64,
}

/// Configuration for GC-aware macro expansion.
#[derive(Debug, Clone)]
pub struct GcMacroConfig {
    /// Whether to enable GC tracking for macro expansion
    pub track_expansions: bool,
    /// Whether to preserve intermediate expansion results
    pub preserve_intermediates: bool,
    /// Whether to enable GC collection during macro expansion
    pub gc_during_expansion: bool,
    /// Maximum expansion depth before triggering GC
    pub max_expansion_depth: usize,
}

/// Unique identifier for macro expansion sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExpansionId(u64);

/// Context for a macro expansion session tracked by GC.
#[derive(Debug)]
pub struct ExpansionContext {
    /// Unique identifier
    pub id: ExpansionId,
    /// Macro name being expanded
    pub macro_name: String,
    /// Environment at expansion time
    pub environment: Arc<ThreadSafeEnvironment>,
    /// Hygiene context for this expansion
    pub hygiene_context: HygieneContext,
    /// Input expression being expanded
    pub input_expr: Spanned<Expr>,
    /// Intermediate expansion results (if preservation enabled)
    pub intermediates: Vec<Spanned<Expr>>,
    /// Expansion start time
    pub start_time: Instant,
    /// Current expansion depth
    pub depth: usize,
    /// GC object ID for this context
    pub gc_object_id: Option<ObjectId>,
}

/// Entry in the transformer registry for GC tracking.
#[derive(Debug, Clone)]
pub struct TransformerEntry {
    /// Transformer name
    pub name: String,
    /// The transformer value
    pub transformer: Value,
    /// Environment where transformer is defined
    pub environment: Arc<ThreadSafeEnvironment>,
    /// Registration time
    pub registered_at: Instant,
    /// GC object ID if tracked
    pub gc_object_id: Option<ObjectId>,
}

/// Result of a GC-aware macro expansion.
#[derive(Debug, Clone)]
pub struct GcMacroExpansionResult {
    /// The expanded expression
    pub result: Spanned<Expr>,
    /// Expansion statistics
    pub stats: ExpansionStats,
    /// Whether GC was triggered during expansion
    pub gc_triggered: bool,
}

/// Statistics about macro expansion.
#[derive(Debug, Clone)]
pub struct ExpansionStats {
    /// Total expansion time
    pub expansion_time: std::time::Duration,
    /// Number of expansion steps
    pub expansion_steps: usize,
    /// Maximum depth reached
    pub max_depth: usize,
    /// Number of intermediate results created
    pub intermediates_created: usize,
    /// Memory usage during expansion (estimated)
    pub estimated_memory_usage: usize,
}

impl GcMacroCoordinator {
    /// Creates a new GC-aware macro coordinator.
    pub fn new(
        expander: MacroExpander,
        gc_integration: Arc<GcIntegration>,
        config: GcMacroConfig,
    ) -> Self {
        Self {
            expander: Arc::new(Mutex::new(expander)),
            gc_integration,
            expansion_contexts: RwLock::new(HashMap::new()),
            transformer_registry: RwLock::new(HashMap::new()),
            config,
            next_expansion_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Creates a GC-aware macro coordinator with default configuration.
    pub fn with_default_config(
        expander: MacroExpander,
        gc_integration: Arc<GcIntegration>,
    ) -> Self {
        Self::new(expander, gc_integration, GcMacroConfig::default())
    }

    /// Registers a macro transformer for GC tracking.
    pub fn register_transformer(
        &self,
        name: String,
        transformer: Value,
        environment: Arc<ThreadSafeEnvironment>,
    ) -> Result<()> {
        let entry = TransformerEntry {
            name: name.clone(),
            transformer: transformer.clone(),
            environment,
            registered_at: Instant::now(),
            gc_object_id: None, // Would be assigned if GC tracking is enabled
        };

        if let Ok(mut registry) = self.transformer_registry.write() {
            registry.insert(name.clone(), entry);

            // Register as GC root if tracking is enabled
            if self.config.track_expansions {
                let object_id = ObjectId::new(next_hygiene_id());
                self.gc_integration.register_macro_root(object_id);
                
                // Update entry with GC object ID
                if let Some(entry) = registry.get_mut(&name) {
                    entry.gc_object_id = Some(object_id);
                }
            }
        }

        Ok(())
    }

    /// Expands a macro with GC tracking and hygiene preservation.
    pub fn expand_with_gc_tracking(
        &self,
        expr: Spanned<Expr>,
        env: Arc<ThreadSafeEnvironment>,
    ) -> Result<GcMacroExpansionResult> {
        let start_time = Instant::now();
        let expansion_id = ExpansionId(
            self.next_expansion_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        );

        // Create expansion context
        let context = ExpansionContext {
            id: expansion_id,
            macro_name: self.extract_macro_name(&expr),
            environment: env.clone(),
            hygiene_context: HygieneContext::new(),
            input_expr: expr.clone(),
            intermediates: Vec::new(),
            start_time,
            depth: 0,
            gc_object_id: None,
        };

        // Register expansion context for GC tracking
        if self.config.track_expansions {
            let object_id = ObjectId::new(expansion_id.0);
            self.gc_integration.register_macro_root(object_id);
            
            if let Ok(mut contexts) = self.expansion_contexts.write() {
                let mut context = context;
                context.gc_object_id = Some(object_id);
                contexts.insert(expansion_id, context);
            }
        }

        // Perform the actual expansion with monitoring
        let result = self.perform_tracked_expansion(expansion_id, expr, env)?;

        // Clean up expansion context
        self.cleanup_expansion_context(expansion_id);

        let expansion_time = start_time.elapsed();
        Ok(GcMacroExpansionResult {
            result: result.result,
            stats: ExpansionStats {
                expansion_time,
                expansion_steps: result.steps,
                max_depth: result.max_depth,
                intermediates_created: result.intermediates_count,
                estimated_memory_usage: result.estimated_memory,
            },
            gc_triggered: result.gc_triggered,
        })
    }

    /// Performs macro expansion with detailed tracking.
    fn perform_tracked_expansion(
        &self,
        expansion_id: ExpansionId,
        expr: Spanned<Expr>,
        env: Arc<ThreadSafeEnvironment>,
    ) -> Result<TrackedExpansionResult> {
        let mut steps = 0;
        let mut max_depth = 0;
        let mut intermediates_count = 0;
        let mut estimated_memory = 0;
        let mut gc_triggered = false;

        // Lock the expander for the duration of expansion
        if let Ok(mut expander) = self.expander.lock() {
            // Set up GC-aware expansion environment
            let result = expander.expand(&expr)?;
            
            // Track expansion statistics
            steps = 1; // Simple expansion for now
            estimated_memory = Self::estimate_expression_memory(&result);

            // Update expansion context with intermediates if enabled
            if self.config.preserve_intermediates {
                self.record_intermediate_result(expansion_id, result.clone());
                intermediates_count = 1;
            }

            // Check if we should trigger GC
            if self.config.gc_during_expansion && self.should_trigger_gc_during_expansion() {
                self.trigger_expansion_gc();
                gc_triggered = true;
            }

            Ok(TrackedExpansionResult {
                result,
                steps,
                max_depth,
                intermediates_count,
                estimated_memory,
                gc_triggered,
            })
        } else {
            Err(Box::new(Error::runtime_error(
                "Failed to acquire macro expander lock".to_string(),
                None,
            )))
        }
    }

    /// Records an intermediate expansion result for debugging/GC tracking.
    fn record_intermediate_result(&self, expansion_id: ExpansionId, result: Spanned<Expr>) {
        if let Ok(mut contexts) = self.expansion_contexts.write() {
            if let Some(context) = contexts.get_mut(&expansion_id) {
                context.intermediates.push(result);
            }
        }
    }

    /// Checks if GC should be triggered during macro expansion.
    fn should_trigger_gc_during_expansion(&self) -> bool {
        if !self.config.gc_during_expansion {
            return false;
        }

        // Check if we have many active expansions
        if let Ok(contexts) = self.expansion_contexts.read() {
            contexts.len() > 10 // Arbitrary threshold
        } else {
            false
        }
    }

    /// Triggers garbage collection during macro expansion.
    fn trigger_expansion_gc(&self) {
        // Collect roots from active expansion contexts
        let expansion_roots = self.collect_expansion_roots();
        
        // Register roots with GC integration
        for root_id in expansion_roots {
            self.gc_integration.register_macro_root(root_id);
        }

        // Trigger GC (this would be coordinated with the main GC system)
        crate::utils::gc::gc_collect();
    }

    /// Collects GC root object IDs from active expansion contexts.
    fn collect_expansion_roots(&self) -> Vec<ObjectId> {
        let mut roots = Vec::new();
        
        if let Ok(contexts) = self.expansion_contexts.read() {
            for context in contexts.values() {
                if let Some(object_id) = context.gc_object_id {
                    roots.push(object_id);
                }
            }
        }

        if let Ok(registry) = self.transformer_registry.read() {
            for entry in registry.values() {
                if let Some(object_id) = entry.gc_object_id {
                    roots.push(object_id);
                }
            }
        }

        roots
    }

    /// Cleans up an expansion context after completion.
    fn cleanup_expansion_context(&self, expansion_id: ExpansionId) {
        if let Ok(mut contexts) = self.expansion_contexts.write() {
            if let Some(context) = contexts.remove(&expansion_id) {
                // Unregister from GC if it was tracked
                if let Some(object_id) = context.gc_object_id {
                    self.gc_integration.unregister_continuation_root(object_id);
                }
            }
        }
    }

    /// Extracts the macro name from an expression for context tracking.
    fn extract_macro_name(&self, expr: &Spanned<Expr>) -> String {
        match &expr.inner {
            Expr::Application { operator, .. } => {
                match &operator.inner {
                    Expr::Identifier(name) => name.clone(),
                    Expr::Symbol(name) => name.clone(),
                    _ => "<complex-macro>".to_string(),
                }
            }
            Expr::Identifier(name) => name.clone(),
            Expr::Symbol(name) => name.clone(),
            _ => "<unknown-macro>".to_string(),
        }
    }

    /// Estimates memory usage of an expression for GC statistics.
    fn estimate_expression_memory(expr: &Spanned<Expr>) -> usize {
        // Simple heuristic based on expression complexity
        match &expr.inner {
            Expr::Literal(_) => 32,
            Expr::Identifier(_) | Expr::Symbol(_) => 24,
            Expr::Application { operator, operands } => {
                let operator_size = Self::estimate_expression_memory(operator);
                let operands_size: usize = operands
                    .iter()
                    .map(Self::estimate_expression_memory)
                    .sum();
                48 + operator_size + operands_size
            }
            Expr::Lambda { body, .. } => {
                let body_size: usize = body
                    .iter()
                    .map(Self::estimate_expression_memory)
                    .sum();
                128 + body_size // Lambda overhead plus body
            }
            Expr::Let { bindings, body } => {
                let bindings_size = bindings.len() * 64; // Rough estimate per binding
                let body_size: usize = body
                    .iter()
                    .map(Self::estimate_expression_memory)
                    .sum();
                96 + bindings_size + body_size
            }
            _ => 64, // Default estimate for other forms
        }
    }

    /// Gets statistics about active macro expansions.
    pub fn get_expansion_statistics(&self) -> MacroExpansionStatistics {
        let active_expansions = if let Ok(contexts) = self.expansion_contexts.read() {
            contexts.len()
        } else {
            0
        };

        let registered_transformers = if let Ok(registry) = self.transformer_registry.read() {
            registry.len()
        } else {
            0
        };

        MacroExpansionStatistics {
            active_expansions,
            registered_transformers,
            gc_roots_tracked: self.collect_expansion_roots().len(),
        }
    }

    /// Gets the configuration.
    pub fn config(&self) -> &GcMacroConfig {
        &self.config
    }
}

/// Result of a tracked macro expansion.
#[derive(Debug)]
struct TrackedExpansionResult {
    /// The expanded expression
    result: Spanned<Expr>,
    /// Number of expansion steps
    steps: usize,
    /// Maximum depth reached
    max_depth: usize,
    /// Number of intermediate results
    intermediates_count: usize,
    /// Estimated memory usage
    estimated_memory: usize,
    /// Whether GC was triggered
    gc_triggered: bool,
}

/// Statistics about macro expansion system.
#[derive(Debug, Clone)]
pub struct MacroExpansionStatistics {
    /// Number of active macro expansions
    pub active_expansions: usize,
    /// Number of registered transformers
    pub registered_transformers: usize,
    /// Number of GC roots being tracked
    pub gc_roots_tracked: usize,
}

impl Default for GcMacroConfig {
    fn default() -> Self {
        Self {
            track_expansions: true,
            preserve_intermediates: false, // Usually not needed in production
            gc_during_expansion: true,
            max_expansion_depth: 1000,
        }
    }
}

impl ExpansionId {
    /// Gets the raw ID value.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

/// Extension trait for MacroExpander to support GC integration.
pub trait MacroExpanderGcExt {
    /// Expands an expression with GC tracking.
    fn expand_with_gc(&mut self, expr: &Spanned<Expr>, env: &crate::eval::Environment) -> Result<Spanned<Expr>>;
    
    /// Registers a syntax transformer with GC tracking.
    fn register_syntax_with_gc(&mut self, name: String, transformer: Value, env: Arc<ThreadSafeEnvironment>) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macro_system::MacroExpander;
    use crate::utils::GcIntegration;
    use crate::eval::value::{ThreadSafeEnvironment, Value};
    use crate::ast::{Expr, Literal};
    use crate::diagnostics::Span;

    #[test]
    fn test_gc_macro_coordinator_creation() {
        let expander = MacroExpander::new();
        let gc_integration = Arc::new(GcIntegration::with_default_config());
        let coordinator = GcMacroCoordinator::with_default_config(expander, gc_integration);
        
        let stats = coordinator.get_expansion_statistics();
        assert_eq!(stats.active_expansions, 0);
        assert_eq!(stats.registered_transformers, 0);
    }

    #[test]
    fn test_transformer_registration() {
        let expander = MacroExpander::new();
        let gc_integration = Arc::new(GcIntegration::with_default_config());
        let coordinator = GcMacroCoordinator::with_default_config(expander, gc_integration);
        
        let env = Arc::new(ThreadSafeEnvironment::new(None, 0));
        let transformer = Value::integer(42); // Dummy transformer
        
        let result = coordinator.register_transformer(
            "test-macro".to_string(),
            transformer,
            env,
        );
        
        assert!(result.is_ok());
        
        let stats = coordinator.get_expansion_statistics();
        assert_eq!(stats.registered_transformers, 1);
    }

    #[test]
    fn test_memory_estimation() {
        let expander = MacroExpander::new();
        let gc_integration = Arc::new(GcIntegration::with_default_config());
        let coordinator = GcMacroCoordinator::with_default_config(expander, gc_integration);
        
        let simple_expr = Spanned::new(
            Expr::Literal(Literal::Number(42.0)),
            Span::new(0, 2),
        );
        
        let size = GcCoordinator::estimate_expression_memory(&simple_expr);
        assert!(size > 0);
        assert!(size < 100); // Should be small for a simple literal
    }
}
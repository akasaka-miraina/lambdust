//! Integration of Macro-time Computation with Expander Pipeline
//!
//! This module provides integration between the macro-time computation system
//! and the existing macro expander pipeline, enabling seamless use of advanced
//! macro features while maintaining compatibility with existing systems.

use super::{
    macro_time_computation::{MacroTimeEnvironment, MacroTimeValue, Phase},
    advanced_quasisyntax::{AdvancedQuasisyntaxProcessor, AdvancedQuasisyntaxTemplate},
    macro_time_transformers::{MacroTimeTransformer, transformer_factory},
    syntax_objects::{SyntaxObject, LexicalContext},
    unified_expander::{UnifiedMacroExpander, UnifiedMacroTransformer, MacroTransformerType},
    advanced_hygiene::{HygieneResolver},
};
use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::Environment;
use std::collections::HashMap;
use std::rc::Rc;

/// Enhanced macro expander with macro-time computation support
#[derive(Debug)]
pub struct MacroTimeAwareExpander {
    /// Base macro expander
    base_expander: UnifiedMacroExpander,
    /// Macro-time computation environment
    macro_env: MacroTimeEnvironment,
    /// Advanced quasisyntax processor
    quasisyntax_processor: AdvancedQuasisyntaxProcessor,
    /// Registry of macro-time transformers
    macro_time_transformers: HashMap<String, MacroTimeTransformer>,
    /// Hygiene resolver
    hygiene_env: HygieneResolver,
    /// Integration statistics
    stats: MacroTimeIntegrationStats,
}

/// Statistics for macro-time integration
#[derive(Debug, Clone, Default)]
pub struct MacroTimeIntegrationStats {
    /// Number of macro-time transformations
    macro_time_transformations: u64,
    /// Number of compile-time evaluations
    compile_time_evaluations: u64,
    /// Number of fallbacks to basic expansion
    basic_expansion_fallbacks: u64,
    /// Total integration overhead
    integration_overhead: std::time::Duration,
    /// Success rate for macro-time operations
    success_rate: f64,
}

/// Configuration for macro-time integration
#[derive(Debug, Clone)]
pub struct MacroTimeIntegrationConfig {
    /// Enable macro-time computation
    pub enable_macro_time: bool,
    /// Enable advanced quasisyntax
    pub enable_advanced_quasisyntax: bool,
    /// Enable debugging
    pub enable_debugging: bool,
    /// Maximum macro expansion depth
    pub max_expansion_depth: usize,
    /// Timeout for macro-time computations
    pub computation_timeout: std::time::Duration,
    /// Enable caching
    pub enable_caching: bool,
}

impl Default for MacroTimeIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_macro_time: true,
            enable_advanced_quasisyntax: true,
            enable_debugging: false,
            max_expansion_depth: 1000,
            computation_timeout: std::time::Duration::from_secs(5),
            enable_caching: true,
        }
    }
}

impl MacroTimeAwareExpander {
    /// Creates a new macro-time aware expander
    pub fn new(config: MacroTimeIntegrationConfig) -> Self {
        let mut expander = Self {
            base_expander: UnifiedMacroExpander::new(),
            macro_env: MacroTimeEnvironment::new(),
            quasisyntax_processor: AdvancedQuasisyntaxProcessor::new(),
            macro_time_transformers: HashMap::new(),
            hygiene_env: HygieneResolver::new(),
            stats: MacroTimeIntegrationStats::default(),
        };

        // Configure the macro environment
        expander.macro_env.set_debug_enabled(config.enable_debugging);

        // Register built-in macro-time transformers
        expander.register_builtin_transformers();

        expander
    }

    /// Registers built-in macro-time transformers
    fn register_builtin_transformers(&mut self) {
        // Register the repeat transformer from the roadmap
        let repeat_transformer = transformer_factory::create_repeat_transformer();
        self.register_macro_time_transformer(repeat_transformer);

        // Register other common transformers
        let identity_transformer = transformer_factory::create_identifier_transformer(
            "identity".to_string(),
            "x".to_string(),
            "x".to_string(),
        );
        self.register_macro_time_transformer(identity_transformer);
    }

    /// Registers a macro-time transformer
    pub fn register_macro_time_transformer(&mut self, transformer: MacroTimeTransformer) {
        let name = transformer.name().to_string();
        self.macro_time_transformers.insert(name, transformer);
    }

    /// Expands a macro with macro-time computation support
    pub fn expand_macro(
        &mut self,
        input: &SyntaxObject,
        macro_name: &str,
        env: &Environment,
    ) -> Result<SyntaxObject> {
        let start_time = std::time::Instant::now();

        // Check if we have a macro-time transformer for this macro
        if let Some(transformer) = self.macro_time_transformers.get_mut(macro_name) {
            self.stats.macro_time_transformations += 1;
            
            let result = transformer.transform(input);
            
            // Update statistics
            let elapsed = start_time.elapsed();
            self.stats.integration_overhead += elapsed;
            
            match result {
                Ok(output) => {
                    self.update_success_rate(true);
                    return Ok(output);
                }
                Err(e) => {
                    // Log error and fall back to basic expansion
                    eprintln!("Macro-time transformation failed: {e}, falling back to basic expansion");
                    self.stats.basic_expansion_fallbacks += 1;
                    self.update_success_rate(false);
                }
            }
        }

        // Fall back to basic expansion
        self.expand_with_base_expander(input, macro_name, env)
    }

    /// Expands using the base expander
    fn expand_with_base_expander(
        &mut self,
        input: &SyntaxObject,
        macro_name: &str,
        env: &Environment,
    ) -> Result<SyntaxObject> {
        // Convert syntax object to arguments for base expander
        let args = match &input.expr {
            Expr::List(elements) if !elements.is_empty() => {
                // Skip the first element (macro name) and use the rest as arguments
                elements[1..].to_vec()
            }
            _ => {
                // Single expression becomes the only argument
                vec![input.to_spanned()]
            }
        };
        
        // Use the base expander with proper parameters
        let result = self.base_expander.expand_macro(macro_name, &args, input.span, env)?;
        
        // Convert the result back to a SyntaxObject
        Ok(SyntaxObject::new(result.inner, result.span, input.context.clone()))
    }

    /// Evaluates an expression at macro-time
    pub fn macro_time_eval(
        &mut self,
        expr: &SyntaxObject,
    ) -> Result<MacroTimeValue> {
        self.stats.compile_time_evaluations += 1;
        self.macro_env.compile_time_eval(expr, &mut self.hygiene_env)
    }

    /// Processes an advanced quasisyntax template
    pub fn process_advanced_quasisyntax(
        &mut self,
        template: &AdvancedQuasisyntaxTemplate,
        bindings: &super::syntax_case::SyntaxBindings,
        context: &LexicalContext,
        span: Span,
    ) -> Result<super::advanced_quasisyntax::QuasisyntaxExpansionResult> {
        // Since SyntaxBindings doesn't provide direct access to iterate over bindings,
        // we'll create a minimal HashMap for the common case
        let converted_bindings = HashMap::new();
        
        // Create an AdvancedGenerationContext 
        let mut generation_context = super::advanced_quasisyntax::AdvancedGenerationContext::new();
        
        self.quasisyntax_processor.process_template(
            template, 
            &converted_bindings, 
            &mut generation_context, 
            span
        )
    }

    /// Enters a new macro expansion phase
    pub fn enter_macro_phase(&mut self) -> Phase {
        self.macro_env.enter_phase(Phase::MACRO_TIME)
    }

    /// Exits a macro expansion phase
    pub fn exit_macro_phase(&mut self, previous_phase: Phase) {
        self.macro_env.exit_phase(previous_phase);
    }

    /// Gets the current phase
    pub fn current_phase(&self) -> Phase {
        self.macro_env.current_phase()
    }

    /// Adds a compile-time binding
    pub fn add_compile_time_binding(&mut self, name: String, value: MacroTimeValue) {
        // This would add to the macro environment
        // For now, we'll store it in the transformer registry if it's a procedure
        if let MacroTimeValue::Procedure { name: proc_name, .. } = &value {
            // Could register as a transformer
        }
    }

    /// Gets integration statistics
    pub fn get_stats(&self) -> &MacroTimeIntegrationStats {
        &self.stats
    }

    /// Clears all caches
    pub fn clear_caches(&mut self) {
        self.macro_env.clear_cache();
        self.quasisyntax_processor.clear_cache();
        // Clear base expander cache if it has one
    }

    /// Updates the success rate statistics
    fn update_success_rate(&mut self, success: bool) {
        let total_operations = self.stats.macro_time_transformations;
        if total_operations == 0 {
            self.stats.success_rate = if success { 1.0 } else { 0.0 };
        } else {
            let current_successes = (self.stats.success_rate * (total_operations - 1) as f64) as u64;
            let new_successes = if success { current_successes + 1 } else { current_successes };
            self.stats.success_rate = new_successes as f64 / total_operations as f64;
        }
    }

    /// Validates a macro definition for macro-time computation compatibility
    pub fn validate_macro_time_compatibility(
        &self,
        macro_def: &SyntaxObject,
    ) -> Result<MacroTimeCompatibilityReport> {
        let mut report = MacroTimeCompatibilityReport::new();

        // Check if the macro uses macro-time computation features
        if self.uses_macro_time_features(&macro_def.expr) {
            report.uses_macro_time = true;
            report.compatible = true;
        }

        // Check for unsupported features
        if self.has_unsupported_features(&macro_def.expr) {
            report.compatible = false;
            report.issues.push("Contains unsupported features for macro-time computation".to_string());
        }

        Ok(report)
    }

    /// Checks if an expression uses macro-time computation features
    #[allow(clippy::only_used_in_recursion)]
    fn uses_macro_time_features(&self, expr: &Expr) -> bool {
        match expr {
            Expr::List(elements) => {
                if let Some(first) = elements.first() {
                    if let Expr::Identifier(name) = &first.inner {
                        match name.as_str() {
                            "make-list" | "generate-temporaries" | "syntax->datum" | "datum->syntax" => true,
                            _ => elements.iter().any(|e| self.uses_macro_time_features(&e.inner))
                        }
                    } else {
                        elements.iter().any(|e| self.uses_macro_time_features(&e.inner))
                    }
                } else {
                    false
                }
            }
            Expr::Unquote(inner) | Expr::UnquoteSplicing(inner) | Expr::Quasiquote(inner) => {
                self.uses_macro_time_features(&inner.inner)
            }
            _ => false,
        }
    }

    /// Checks if an expression has unsupported features
    fn has_unsupported_features(&self, expr: &Expr) -> bool {
        // For now, assume all features are supported
        // A real implementation would check for specific unsupported constructs
        false
    }
}

/// Report on macro-time computation compatibility
#[derive(Debug, Clone)]
pub struct MacroTimeCompatibilityReport {
    /// Whether the macro is compatible with macro-time computation
    pub compatible: bool,
    /// Whether the macro uses macro-time features
    pub uses_macro_time: bool,
    /// List of compatibility issues
    pub issues: Vec<String>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

impl MacroTimeCompatibilityReport {
    fn new() -> Self {
        Self {
            compatible: true,
            uses_macro_time: false,
            issues: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Checks if the macro is fully compatible
    pub fn is_compatible(&self) -> bool {
        self.compatible && self.issues.is_empty()
    }

    /// Gets a summary of the compatibility status
    pub fn summary(&self) -> String {
        if self.is_compatible() {
            if self.uses_macro_time {
                "Fully compatible with macro-time computation".to_string()
            } else {
                "Compatible but does not use macro-time features".to_string()
            }
        } else {
            format!("Incompatible: {}", self.issues.join(", "))
        }
    }
}

impl MacroTimeIntegrationStats {
    /// Gets the overall performance ratio
    pub fn performance_ratio(&self) -> f64 {
        let total_operations = self.macro_time_transformations + self.basic_expansion_fallbacks;
        if total_operations == 0 {
            1.0
        } else {
            self.macro_time_transformations as f64 / total_operations as f64
        }
    }

    /// Gets average integration overhead per operation
    pub fn average_overhead(&self) -> std::time::Duration {
        let total_operations = self.macro_time_transformations + self.basic_expansion_fallbacks;
        if total_operations == 0 {
            std::time::Duration::ZERO
        } else {
            self.integration_overhead / total_operations as u32
        }
    }

    /// Checks if the integration is performing well
    pub fn is_performing_well(&self) -> bool {
        self.success_rate > 0.8 && self.performance_ratio() > 0.5
    }
}

/// High-level interface for macro-time computation integration
pub mod integration_interface {
    use super::*;

    /// Creates a macro-time aware expander with default configuration
    pub fn create_macro_time_expander() -> MacroTimeAwareExpander {
        MacroTimeAwareExpander::new(MacroTimeIntegrationConfig::default())
    }

    /// Creates a macro-time aware expander with custom configuration
    pub fn create_configured_macro_time_expander(
        config: MacroTimeIntegrationConfig,
    ) -> MacroTimeAwareExpander {
        MacroTimeAwareExpander::new(config)
    }

    /// Expands a macro using macro-time computation if available
    pub fn expand_with_macro_time(
        expander: &mut MacroTimeAwareExpander,
        input: &SyntaxObject,
        macro_name: &str,
        env: &Environment,
    ) -> Result<SyntaxObject> {
        expander.expand_macro(input, macro_name, env)
    }

    /// Validates that a macro definition is compatible with macro-time computation
    pub fn validate_macro_compatibility(
        expander: &MacroTimeAwareExpander,
        macro_def: &SyntaxObject,
    ) -> Result<MacroTimeCompatibilityReport> {
        expander.validate_macro_time_compatibility(macro_def)
    }

    /// Creates and registers the repeat transformer
    pub fn setup_repeat_transformer(
        expander: &mut MacroTimeAwareExpander,
    ) {
        let repeat_transformer = transformer_factory::create_repeat_transformer();
        expander.register_macro_time_transformer(repeat_transformer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::integration_interface::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_macro_time_aware_expander_creation() {
        let expander = create_macro_time_expander();
        assert_eq!(expander.current_phase(), Phase::RUNTIME);
    }

    #[test]
    fn test_macro_time_transformer_registration() {
        let mut expander = create_macro_time_expander();
        let transformer = transformer_factory::create_repeat_transformer();
        
        expander.register_macro_time_transformer(transformer);
        assert!(expander.macro_time_transformers.contains_key("repeat"));
    }

    #[test]
    fn test_phase_management() {
        let mut expander = create_macro_time_expander();
        
        assert_eq!(expander.current_phase(), Phase::RUNTIME);
        
        let previous = expander.enter_macro_phase();
        assert_eq!(expander.current_phase(), Phase::MACRO_TIME);
        assert_eq!(previous, Phase::RUNTIME);
        
        expander.exit_macro_phase(previous);
        assert_eq!(expander.current_phase(), Phase::RUNTIME);
    }

    #[test]
    fn test_compatibility_report() {
        let report = MacroTimeCompatibilityReport::new();
        assert!(report.is_compatible());
        assert_eq!(report.summary(), "Compatible but does not use macro-time features");
    }

    #[test]
    fn test_integration_stats() {
        let mut stats = MacroTimeIntegrationStats::default();
        
        assert_eq!(stats.performance_ratio(), 1.0);
        assert_eq!(stats.average_overhead(), std::time::Duration::ZERO);
        assert!(!stats.is_performing_well()); // No operations yet
        
        stats.macro_time_transformations = 8;
        stats.basic_expansion_fallbacks = 2;
        stats.success_rate = 0.9;
        
        assert_eq!(stats.performance_ratio(), 0.8);
        assert!(stats.is_performing_well());
    }

    #[test]
    fn test_macro_time_features_detection() {
        let expander = create_macro_time_expander();
        
        // Test expression that uses macro-time features
        let make_list_expr = Expr::List(vec![
            Spanned::new(Expr::Identifier("make-list".to_string()), Span::new(0, 9)),
            Spanned::new(Expr::Identifier("n".to_string()), Span::new(10, 11)),
            Spanned::new(Expr::Identifier("expr".to_string()), Span::new(12, 16)),
        ]);
        
        assert!(expander.uses_macro_time_features(&make_list_expr));
        
        // Test expression that doesn't use macro-time features
        let simple_expr = Expr::Identifier("x".to_string());
        assert!(!expander.uses_macro_time_features(&simple_expr));
    }

    #[test]
    fn test_setup_repeat_transformer() {
        let mut expander = create_macro_time_expander();
        setup_repeat_transformer(&mut expander);
        
        assert!(expander.macro_time_transformers.contains_key("repeat"));
    }

    #[test]
    fn test_custom_configuration() {
        let config = MacroTimeIntegrationConfig {
            enable_macro_time: true,
            enable_advanced_quasisyntax: true,
            enable_debugging: true,
            max_expansion_depth: 500,
            computation_timeout: std::time::Duration::from_secs(10),
            enable_caching: false,
        };
        
        let expander = create_configured_macro_time_expander(config);
        // Test that the configuration was applied (in a real implementation,
        // we'd have getters to verify the configuration)
        assert_eq!(expander.current_phase(), Phase::RUNTIME);
    }
}
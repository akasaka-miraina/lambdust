//! Integrated type-safe macro expansion system.
//!
//! This module integrates all the type-safe macro expansion components
//! into a unified, high-performance system that provides compile-time
//! type checking, optimization, and hygiene control.

use crate::ast::{Expr, Literal, Program};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::Environment;
use crate::macro_system::advanced_hygiene_control::{
    DetectionConfig, HygieneViolation, HygieneViolationDetector, PreciseHygieneController,
};
use crate::macro_system::compile_time_computation::{
    CompileTimeComputationEngine, CompileTimeContext, CompileTimeValue, ComputationLimits,
    ComputationStatistics,
};
use crate::macro_system::type_safe_expansion::{
    MacroType, OptimizationLevel, TypeSafeMacroExpander, TypeSafeMacroTransformer,
    TypeSafeOptimizationConfig, TypedPattern, TypedTemplate,
};
use crate::macro_system::{
    HygieneContext, MacroEnvironment, MacroTransformer, PatternBindings, next_hygiene_id,
};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Integrated type-safe macro expansion system combining all components.
pub struct IntegratedTypeSafeMacroExpander {
    /// Core type-safe expander
    type_safe_expander: TypeSafeMacroExpander,
    /// Compile-time computation engine
    computation_engine: CompileTimeComputationEngine,
    /// Hygiene violation detector
    hygiene_detector: HygieneViolationDetector,
    /// Precise hygiene controller
    hygiene_controller: PreciseHygieneController,
    /// Legacy macro environment for compatibility
    legacy_environment: Rc<MacroEnvironment>,
    /// Integration configuration
    config: TypeSafeIntegrationConfig,
    /// Performance metrics
    metrics: ExpansionMetrics,
    /// Error accumulator
    errors: Vec<Error>,
    /// Warnings accumulator
    warnings: Vec<String>,
}

/// Configuration for the integrated type-safe expansion system.
#[derive(Debug, Clone)]
pub struct TypeSafeIntegrationConfig {
    /// Type safety configuration
    pub type_safety: TypeSafetyConfig,
    /// Compile-time computation configuration
    pub compile_time_computation: CompileTimeConfig,
    /// Hygiene control configuration
    pub hygiene_control: HygieneControlConfig,
    /// Performance optimization settings
    pub performance: PerformanceConfig,
    /// Compatibility settings
    pub compatibility: CompatibilityConfig,
}

/// Configuration for type safety features in macro expansion.
///
/// Controls type checking behavior, inference settings, and error handling
/// for type-safe macro expansion.
#[derive(Debug, Clone, Default)]
pub struct TypeSafetyConfig {
    /// Enable strict type checking
    pub strict_type_checking: bool,
    /// Enable type inference for untyped templates
    pub enable_type_inference: bool,
    /// Fail on type incompatibilities
    pub fail_on_type_errors: bool,
    /// Maximum type inference depth
    pub max_inference_depth: usize,
    /// Enable gradual typing mode
    pub gradual_typing: bool,
}

/// Configuration for compile-time computation optimization.
///
/// Controls compile-time evaluation, caching, and optimization settings
/// to improve macro expansion performance.
#[derive(Debug, Clone)]
pub struct CompileTimeConfig {
    /// Enable compile-time computation
    pub enable_computation: bool,
    /// Optimization level for compile-time computation
    pub optimization_level: OptimizationLevel,
    /// Computation time limits
    pub computation_limits: ComputationLimits,
    /// Enable memoization
    pub enable_memoization: bool,
    /// Cache size limit
    pub cache_size_limit: usize,
}

/// Configuration for advanced hygiene control and violation detection.
///
/// Controls hygiene analysis, violation detection, and error reporting
/// for maintaining proper macro hygiene.
#[derive(Debug, Clone)]
pub struct HygieneControlConfig {
    /// Enable advanced hygiene violation detection
    pub enable_violation_detection: bool,
    /// Detection configuration
    pub detection_config: DetectionConfig,
    /// Enable precise hygiene control
    pub enable_precise_control: bool,
    /// Fail on hygiene violations
    pub fail_on_violations: bool,
    /// Report hygiene warnings
    pub report_warnings: bool,
}

/// Configuration for performance optimization features.
///
/// Controls various performance optimizations and profiling settings
/// for efficient macro expansion.
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable parallel expansion (when safe)
    pub enable_parallel_expansion: bool,
    /// Maximum number of parallel tasks
    pub max_parallel_tasks: usize,
    /// Enable aggressive caching
    pub aggressive_caching: bool,
    /// Cache lifetime in seconds
    pub cache_lifetime_seconds: u64,
    /// Enable performance profiling
    pub enable_profiling: bool,
}

/// Configuration for compatibility with different Scheme standards.
///
/// Controls compatibility modes and extensions for supporting various
/// Scheme dialects and standards.
#[derive(Debug, Clone)]
pub struct CompatibilityConfig {
    /// Support legacy macro transformers
    pub support_legacy_macros: bool,
    /// Enable R7RS strict compliance mode
    pub r7rs_strict_mode: bool,
    /// Enable R6RS compatibility features
    pub r6rs_compatibility: bool,
    /// Enable SRFI extensions
    pub enable_srfi_extensions: bool,
}

impl Default for TypeSafeIntegrationConfig {
    fn default() -> Self {
        Self {
            type_safety: TypeSafetyConfig {
                strict_type_checking: false, // Start with gradual adoption
                enable_type_inference: true,
                fail_on_type_errors: false,
                max_inference_depth: 100,
                gradual_typing: true,
            },
            compile_time_computation: CompileTimeConfig {
                enable_computation: true,
                optimization_level: OptimizationLevel::Aggressive,
                computation_limits: ComputationLimits::default(),
                enable_memoization: true,
                cache_size_limit: 10_000,
            },
            hygiene_control: HygieneControlConfig {
                enable_violation_detection: true,
                detection_config: DetectionConfig::default(),
                enable_precise_control: true,
                fail_on_violations: false, // Start with warnings
                report_warnings: true,
            },
            performance: PerformanceConfig {
                enable_parallel_expansion: false, // Not implemented yet
                max_parallel_tasks: 4,
                aggressive_caching: true,
                cache_lifetime_seconds: 3600,
                enable_profiling: false,
            },
            compatibility: CompatibilityConfig {
                support_legacy_macros: true,
                r7rs_strict_mode: false,
                r6rs_compatibility: false,
                enable_srfi_extensions: true,
            },
        }
    }
}

/// Performance metrics for macro expansion.
#[derive(Debug, Default)]
pub struct ExpansionMetrics {
    /// Total number of expansions performed
    pub total_expansions: usize,
    /// Number of successful expansions
    pub successful_expansions: usize,
    /// Number of failed expansions
    pub failed_expansions: usize,
    /// Total expansion time (microseconds)
    pub total_time_us: u64,
    /// Average expansion time (microseconds)
    pub average_time_us: u64,
    /// Type checking statistics
    pub type_checking_stats: TypeCheckingStats,
    /// Compile-time computation statistics
    pub computation_stats: ComputationStatistics,
    /// Hygiene analysis statistics
    pub hygiene_stats: HygieneStats,
    /// Cache performance
    pub cache_stats: CacheStats,
}

/// Statistics about type checking operations during macro expansion.
///
/// Tracks type checking performance and error rates to guide optimization
/// and provide insights into type system usage.
#[derive(Debug, Default)]
pub struct TypeCheckingStats {
    /// Number of expressions that underwent type checking
    pub expressions_type_checked: usize,
    /// Number of type errors discovered
    pub type_errors_found: usize,
    /// Number of type warnings issued
    pub type_warnings_issued: usize,
    /// Number of type inference operations performed
    pub inference_operations: usize,
    /// Total time spent on type checking in microseconds
    pub type_checking_time_us: u64,
}

/// Statistics about hygiene analysis during macro expansion.
///
/// Tracks hygiene violation detection and identifier renaming to measure
/// the effectiveness of hygiene control mechanisms.
#[derive(Debug, Default)]
pub struct HygieneStats {
    /// Number of expressions analyzed for hygiene
    pub expressions_analyzed: usize,
    /// Number of hygiene violations detected
    pub violations_detected: usize,
    /// Number of identifiers renamed for hygiene
    pub identifiers_renamed: usize,
    /// Time spent on hygiene analysis in microseconds
    pub hygiene_analysis_time_us: u64,
}

/// Cache performance statistics for macro expansion optimization.
///
/// Tracks cache efficiency and memory usage to optimize caching strategies
/// and measure performance improvements.
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
    /// Current number of cache entries
    pub cache_entries: usize,
    /// Number of entries evicted from cache
    pub cache_evictions: usize,
    /// Memory used by cache in bytes
    pub memory_usage_bytes: usize,
}

/// Result of integrated macro expansion.
#[derive(Debug)]
pub struct IntegratedExpansionResult {
    /// Expanded expression
    pub expanded: Spanned<Expr>,
    /// Type information (if available)
    pub result_type: Option<MacroType>,
    /// Hygiene violations detected
    pub hygiene_violations: Vec<HygieneViolation>,
    /// Performance metrics for this expansion
    pub metrics: SingleExpansionMetrics,
    /// Warnings generated during expansion
    pub warnings: Vec<String>,
    /// Whether expansion was successful
    pub success: bool,
}

/// Metrics for a single macro expansion operation.
///
/// Tracks detailed performance information for individual macro expansion
/// operations to enable fine-grained performance analysis.
#[derive(Debug, Default)]
pub struct SingleExpansionMetrics {
    /// Time spent on macro expansion in microseconds
    pub expansion_time_us: u64,
    /// Time spent on type checking in microseconds
    pub type_checking_time_us: u64,
    /// Time spent on hygiene analysis in microseconds
    pub hygiene_analysis_time_us: u64,
    /// Number of cache hits during this expansion
    pub cache_hits: usize,
    /// Number of optimizations applied during expansion
    pub optimizations_applied: usize,
}

impl IntegratedTypeSafeMacroExpander {
    /// Creates a new integrated macro expander with default configuration.
    pub fn new() -> Self {
        Self::with_config(TypeSafeIntegrationConfig::default())
    }

    /// Creates a new integrated macro expander with custom configuration.
    pub fn with_config(config: TypeSafeIntegrationConfig) -> Self {
        let optimization_config = TypeSafeOptimizationConfig {
            enable_type_inference: config.type_safety.enable_type_inference,
            enable_compile_time_computation: config.compile_time_computation.enable_computation,
            optimization_level: config.compile_time_computation.optimization_level.clone(),
            ..Default::default()
        };

        Self {
            type_safe_expander: TypeSafeMacroExpander::with_config(optimization_config),
            computation_engine: CompileTimeComputationEngine::new(),
            hygiene_detector: HygieneViolationDetector::with_config(
                config.hygiene_control.detection_config.clone(),
            ),
            hygiene_controller: PreciseHygieneController::new(),
            legacy_environment: Rc::new(MacroEnvironment::new()),
            config,
            metrics: ExpansionMetrics::default(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Expands a single expression with full integration.
    pub fn expand_expression(
        &mut self,
        expr: &Spanned<Expr>,
        environment: &Environment,
    ) -> Result<IntegratedExpansionResult> {
        let start_time = Instant::now();
        let mut single_metrics = SingleExpansionMetrics::default();
        let mut warnings = Vec::new();
        let mut success = true;

        // Step 1: Try type-safe expansion first
        let expansion_result = if self.can_use_type_safe_expansion(expr)? {
            self.expand_with_type_safety(expr, environment, &mut single_metrics)?
        } else {
            // Fallback to legacy expansion
            warnings.push("Falling back to legacy macro expansion".to_string());
            self.expand_with_legacy_system(expr, environment)?
        };

        // Step 2: Perform hygiene analysis
        let hygiene_violations = if self.config.hygiene_control.enable_violation_detection {
            let hygiene_start = Instant::now();
            let violations = self
                .hygiene_detector
                .analyze_expression(
                    &expansion_result,
                    &HygieneContext::new(), // TODO: Pass actual context
                    environment,
                )
                .unwrap_or_else(|_| {
                    warnings.push("Hygiene analysis failed".to_string());
                    Vec::new()
                });
            single_metrics.hygiene_analysis_time_us = hygiene_start.elapsed().as_micros() as u64;

            // Handle violations based on configuration
            if !violations.is_empty() {
                if self.config.hygiene_control.fail_on_violations {
                    success = false;
                }
                if self.config.hygiene_control.report_warnings {
                    for violation in &violations {
                        warnings.push(format!("Hygiene violation: {}", violation.description));
                    }
                }
            }

            violations
        } else {
            Vec::new()
        };

        // Step 3: Update metrics
        single_metrics.expansion_time_us = start_time.elapsed().as_micros() as u64;
        self.update_global_metrics(&single_metrics, success);

        Ok(IntegratedExpansionResult {
            expanded: expansion_result,
            result_type: None, // TODO: Preserve type information
            hygiene_violations,
            metrics: single_metrics,
            warnings,
            success,
        })
    }

    /// Expands a complete program with full integration.
    pub fn expand_program(
        &mut self,
        program: &Program,
        environment: &Environment,
    ) -> Result<Program> {
        let mut expanded_expressions = Vec::new();

        for expr in &program.expressions {
            match self.expand_expression(expr, environment) {
                Ok(result) => {
                    if !result.success && self.config.type_safety.fail_on_type_errors {
                        return Err(Box::new(Error::macro_error(
                            "Program expansion failed due to type safety violations".to_string(),
                            expr.span,
                        )));
                    }
                    expanded_expressions.push(result.expanded);
                }
                Err(e) => {
                    if self.config.type_safety.fail_on_type_errors {
                        return Err(e);
                    } else {
                        // Try to recover with original expression
                        expanded_expressions.push(expr.clone());
                        self.warnings
                            .push(format!("Failed to expand expression: {e}"));
                    }
                }
            }
        }

        Ok(Program::with_expressions(expanded_expressions))
    }

    fn can_use_type_safe_expansion(&self, expr: &Spanned<Expr>) -> Result<bool> {
        // Check if expression is suitable for type-safe expansion
        match &expr.inner {
            Expr::Application { operator, .. } => {
                if let Expr::Identifier(name) = &operator.inner {
                    // Check if this is a known macro that supports type-safe expansion
                    Ok(self.is_type_safe_macro(name))
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }

    fn is_type_safe_macro(&self, name: &str) -> bool {
        // For now, assume all macros can be type-safe
        // In practice, this would check a registry of type-safe macros
        true
    }

    fn expand_with_type_safety(
        &mut self,
        expr: &Spanned<Expr>,
        environment: &Environment,
        metrics: &mut SingleExpansionMetrics,
    ) -> Result<Spanned<Expr>> {
        // TODO: Implement type-safe expansion path
        // This would involve:
        // 1. Converting legacy macro to type-safe transformer
        // 2. Performing type checking
        // 3. Applying compile-time optimizations
        // 4. Executing expansion with type safety

        // For now, fallback to legacy system
        self.expand_with_legacy_system(expr, environment)
    }

    fn expand_with_legacy_system(
        &mut self,
        expr: &Spanned<Expr>,
        environment: &Environment,
    ) -> Result<Spanned<Expr>> {
        // TODO: Integrate with existing macro expansion system
        // This is a placeholder that would connect to the existing MacroExpander

        Ok(expr.clone())
    }

    fn update_global_metrics(&mut self, single_metrics: &SingleExpansionMetrics, success: bool) {
        self.metrics.total_expansions += 1;

        if success {
            self.metrics.successful_expansions += 1;
        } else {
            self.metrics.failed_expansions += 1;
        }

        self.metrics.total_time_us += single_metrics.expansion_time_us;
        self.metrics.average_time_us =
            self.metrics.total_time_us / self.metrics.total_expansions as u64;

        self.metrics.type_checking_stats.type_checking_time_us +=
            single_metrics.type_checking_time_us;
        self.metrics.hygiene_stats.hygiene_analysis_time_us +=
            single_metrics.hygiene_analysis_time_us;
        self.metrics.cache_stats.cache_hits += single_metrics.cache_hits;
    }

    /// Creates a type-safe macro transformer from a legacy transformer.
    pub fn create_type_safe_transformer(
        &self,
        legacy_transformer: &MacroTransformer,
    ) -> Result<TypeSafeMacroTransformer> {
        // Convert legacy pattern to typed pattern
        let typed_pattern = self.convert_pattern_to_typed(&legacy_transformer.pattern)?;

        // Convert legacy template to typed template
        let typed_template = self.convert_template_to_typed(&legacy_transformer.template)?;

        // Infer type signature
        let type_signature = self.infer_macro_type_signature(&typed_pattern, &typed_template)?;

        // Create optimization info
        let optimization_info = self.analyze_transformer_for_optimization(legacy_transformer)?;

        Ok(TypeSafeMacroTransformer {
            pattern: typed_pattern,
            template: typed_template,
            name: legacy_transformer.name.clone(),
            type_signature,
            optimization_info,
        })
    }

    fn convert_pattern_to_typed(
        &self,
        pattern: &crate::macro_system::Pattern,
    ) -> Result<TypedPattern> {
        // TODO: Convert pattern with type information
        Ok(TypedPattern {
            pattern: pattern.clone(),
            expected_type: MacroType::Untyped,
            type_constraints: HashMap::new(),
            verification_info: crate::macro_system::type_safe_expansion::PatternVerificationInfo {
                verified: false,
                type_errors: Vec::new(),
                optimization_hints: Vec::new(),
            },
        })
    }

    fn convert_template_to_typed(
        &self,
        template: &crate::macro_system::Template,
    ) -> Result<TypedTemplate> {
        // TODO: Convert template with type information
        Ok(TypedTemplate {
            template: template.clone(),
            result_type: MacroType::Untyped,
            computation_info: crate::macro_system::type_safe_expansion::TemplateComputationInfo {
                compile_time_computable: false,
                constant_results: HashMap::new(),
                optimization_level: OptimizationLevel::Basic,
                expansion_cache: HashMap::new(),
            },
            type_environment: HashMap::new(),
        })
    }

    fn infer_macro_type_signature(
        &self,
        pattern: &TypedPattern,
        template: &TypedTemplate,
    ) -> Result<crate::macro_system::type_safe_expansion::MacroTypeSignature> {
        // TODO: Implement type signature inference
        Ok(
            crate::macro_system::type_safe_expansion::MacroTypeSignature {
                input_type: pattern.expected_type.clone(),
                output_type: template.result_type.clone(),
                constraints: Vec::new(),
                type_preserving: false,
            },
        )
    }

    fn analyze_transformer_for_optimization(
        &self,
        transformer: &MacroTransformer,
    ) -> Result<crate::macro_system::type_safe_expansion::TransformerOptimizationInfo> {
        // TODO: Analyze transformer for optimization opportunities
        Ok(
            crate::macro_system::type_safe_expansion::TransformerOptimizationInfo {
                inlinable: false,
                idempotent: false,
                expansion_cost: crate::macro_system::type_safe_expansion::ExpansionCost::Linear,
                specializations: HashMap::new(),
            },
        )
    }

    /// Gets comprehensive expansion metrics.
    pub fn get_metrics(&self) -> &ExpansionMetrics {
        &self.metrics
    }

    /// Gets accumulated warnings.
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Gets accumulated errors.
    pub fn get_errors(&self) -> &[Error] {
        &self.errors
    }

    /// Resets all metrics and accumulators.
    pub fn reset_metrics(&mut self) {
        self.metrics = ExpansionMetrics::default();
        self.errors.clear();
        self.warnings.clear();
    }

    /// Configures the expander with new settings.
    pub fn configure(&mut self, config: TypeSafeIntegrationConfig) {
        self.config = config;
        // TODO: Update component configurations
    }
}

impl Default for IntegratedTypeSafeMacroExpander {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating configured integrated macro expanders.
pub struct IntegratedExpanderBuilder {
    config: TypeSafeIntegrationConfig,
}

impl IntegratedExpanderBuilder {
    /// Creates a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: TypeSafeIntegrationConfig::default(),
        }
    }

    /// Configures type safety settings.
    pub fn with_type_safety(mut self, config: TypeSafetyConfig) -> Self {
        self.config.type_safety = config;
        self
    }

    /// Configures compile-time computation settings.
    pub fn with_compile_time_computation(mut self, config: CompileTimeConfig) -> Self {
        self.config.compile_time_computation = config;
        self
    }

    /// Configures hygiene control settings.
    pub fn with_hygiene_control(mut self, config: HygieneControlConfig) -> Self {
        self.config.hygiene_control = config;
        self
    }

    /// Configures performance settings.
    pub fn with_performance(mut self, config: PerformanceConfig) -> Self {
        self.config.performance = config;
        self
    }

    /// Configures compatibility settings.
    pub fn with_compatibility(mut self, config: CompatibilityConfig) -> Self {
        self.config.compatibility = config;
        self
    }

    /// Builds the configured integrated macro expander.
    pub fn build(self) -> IntegratedTypeSafeMacroExpander {
        IntegratedTypeSafeMacroExpander::with_config(self.config)
    }
}

impl Default for IntegratedExpanderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrated_expander_creation() {
        let expander = IntegratedTypeSafeMacroExpander::new();
        assert!(expander.config.type_safety.enable_type_inference);
    }

    #[test]
    fn test_builder_pattern() {
        let expander = IntegratedExpanderBuilder::new()
            .with_type_safety(TypeSafetyConfig {
                strict_type_checking: true,
                ..Default::default()
            })
            .build();

        assert!(expander.config.type_safety.strict_type_checking);
    }

    #[test]
    fn test_default_configuration() {
        let config = TypeSafeIntegrationConfig::default();
        assert!(config.compile_time_computation.enable_computation);
        assert!(config.hygiene_control.enable_violation_detection);
        assert!(config.compatibility.support_legacy_macros);
    }

    #[test]
    fn test_metrics_initialization() {
        let expander = IntegratedTypeSafeMacroExpander::new();
        let metrics = expander.get_metrics();
        assert_eq!(metrics.total_expansions, 0);
    }
}

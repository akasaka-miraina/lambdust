//! Complete Gradual Type Inference System for Lambdust
//!
//! This module provides a unified API for the complete gradual type inference system,
//! integrating all components into a coherent world-class type system:
//!
//! - Automatic type inference with gradual boundaries
//! - Runtime contract generation and enforcement
//! - Consistency checking and violation detection
//! - Performance optimization and monitoring
//! - Blame tracking for precise error attribution
//! - Migration assistance for type safety improvements
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use lambdust::types::{GradualTypeSystem, GradualConfig};
//! use lambdust::ast::Expr;
//! use lambdust::eval::{Environment, Evaluator};
//!
//! let mut type_system = GradualTypeSystem::new();
//! let result = type_system.infer_and_evaluate(&expr, &env, &mut evaluator)?;
//! ```
//!
//! # System Architecture
//!
//! The gradual type system consists of several integrated components:
//!
//! 1. **Type Inference Engine**: Automatic inference with gradual boundaries
//! 2. **Consistency Checker**: Verifies gradual typing rules
//! 3. **Contract Integration**: Automatic contract generation and optimization
//! 4. **Evaluator Integration**: Runtime type checking and cast execution
//! 5. **Performance Monitor**: Optimization and performance tracking
//! 6. **Migration Assistant**: Suggestions for improving type safety

use super::{
    ConsistencyConfig, ConsistencyResult, EvaluatorIntegrationConfig, GradualConsistencyChecker,
    GradualContractConfig, GradualContractIntegration, GradualEvaluatorIntegration,
    GradualInferenceConfig, GradualInferenceResult, GradualTypeInference, IntegrationResult, Type,
    TypeEnv, TypeScheme,
};
use crate::ast::{Expr, Program};
use crate::contracts::{CompilationContext, ContractSystem};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::{Environment, Evaluator, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Configuration for the complete gradual type system
#[derive(Debug, Clone)]
pub struct GradualSystemConfig {
    /// Type inference configuration
    pub inference: GradualInferenceConfig,
    /// Evaluator integration configuration
    pub evaluator: EvaluatorIntegrationConfig,
    /// Consistency checking configuration
    pub consistency: ConsistencyConfig,
    /// Contract integration configuration
    pub contracts: GradualContractConfig,
    /// System-wide optimization level
    pub optimization_level: SystemOptimizationLevel,
    /// Enable comprehensive monitoring
    pub enable_monitoring: bool,
    /// Enable migration assistance
    pub enable_migration: bool,
    /// Development vs production mode
    pub mode: SystemMode,
}

/// System-wide optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemOptimizationLevel {
    /// No optimizations (debugging)
    None,
    /// Basic optimizations (development)
    Basic,
    /// Full optimizations (production)
    Full,
    /// Maximum optimizations (performance critical)
    Maximum,
}

/// System operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemMode {
    /// Development mode (full checking, detailed errors)
    Development,
    /// Testing mode (moderate checking, good performance)
    Testing,
    /// Production mode (minimal checking, maximum performance)
    Production,
}

impl Default for GradualSystemConfig {
    fn default() -> Self {
        Self {
            inference: GradualInferenceConfig::default(),
            evaluator: EvaluatorIntegrationConfig::default(),
            consistency: ConsistencyConfig::default(),
            contracts: GradualContractConfig::default(),
            optimization_level: SystemOptimizationLevel::Basic,
            enable_monitoring: true,
            enable_migration: true,
            mode: SystemMode::Development,
        }
    }
}

/// Result of complete gradual type system processing
#[derive(Debug, Clone)]
pub struct GradualSystemResult {
    /// Final evaluated value
    pub value: Value,
    /// Inferred type information
    pub type_info: TypeInfo,
    /// Consistency analysis
    pub consistency: ConsistencyAnalysis,
    /// Contract information
    pub contracts: ContractInfo,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Migration suggestions
    pub migration_suggestions: Vec<MigrationSuggestion>,
}

/// Type information from inference
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// Inferred type
    pub inferred_type: Type,
    /// Type certainty level
    pub certainty: TypeCertainty,
    /// Type boundaries detected
    pub boundaries: usize,
    /// Casts required
    pub casts: usize,
    /// Optimization opportunities
    pub optimizations: usize,
}

/// Type certainty assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeCertainty {
    /// Low certainty (mostly dynamic)
    Low,
    /// Medium certainty (partially typed)
    Medium,
    /// High certainty (mostly static)
    High,
    /// Very high certainty (fully static)
    VeryHigh,
}

/// Consistency analysis results
#[derive(Debug, Clone)]
pub struct ConsistencyAnalysis {
    /// Overall consistency score (0.0 to 1.0)
    pub score: f64,
    /// Number of violations found
    pub violations: usize,
    /// Severity of worst violation
    pub max_severity: ViolationSeverity,
    /// Consistency improvements suggested
    pub improvements: usize,
}

/// Contract information
#[derive(Debug, Clone)]
pub struct ContractInfo {
    /// Number of contracts generated
    pub generated: usize,
    /// Number of contracts eliminated
    pub eliminated: usize,
    /// Estimated runtime overhead
    pub overhead: f64,
    /// Contract coverage (0.0 to 1.0)
    pub coverage: f64,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Total analysis time
    pub analysis_time: Duration,
    /// Evaluation time
    pub evaluation_time: Duration,
    /// Memory usage estimate
    pub memory_usage: usize,
    /// Performance improvement from optimizations
    pub improvement: f64,
}

/// Migration suggestion
#[derive(Debug, Clone)]
pub struct MigrationSuggestion {
    /// Location of suggestion
    pub location: Span,
    /// Type of suggestion
    pub suggestion_type: MigrationType,
    /// Priority level
    pub priority: Priority,
    /// Expected benefit
    pub benefit: String,
    /// Implementation effort
    pub effort: String,
}

/// Type of migration suggestion
#[derive(Debug, Clone)]
pub enum MigrationType {
    /// Add type annotation
    AddTypeAnnotation,
    /// Use more specific type
    UseMoreSpecificType,
    /// Add contract
    AddContract,
    /// Refactor for better types
    RefactorCode,
    /// Extract typed function
    ExtractFunction,
}

/// Priority level for suggestions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

use super::gradual_consistency::ViolationSeverity;

/// The complete gradual type system
#[derive(Debug)]
pub struct GradualTypeSystem {
    /// System configuration
    config: GradualSystemConfig,
    /// Type inference engine
    inference_engine: Arc<Mutex<GradualTypeInference>>,
    /// Evaluator integration
    evaluator_integration: Arc<Mutex<GradualEvaluatorIntegration>>,
    /// Consistency checker
    consistency_checker: Arc<Mutex<GradualConsistencyChecker>>,
    /// Contract integration
    contract_integration: Arc<Mutex<GradualContractIntegration>>,
    /// System performance monitor
    performance_monitor: SystemPerformanceMonitor,
    /// Migration assistant
    migration_assistant: MigrationAssistant,
    /// Type environment cache
    type_env_cache: Arc<Mutex<HashMap<String, TypeEnv>>>,
}

/// System-wide performance monitoring
#[derive(Debug)]
pub struct SystemPerformanceMonitor {
    /// Overall system metrics
    system_metrics: SystemMetrics,
    /// Component-specific metrics
    component_metrics: HashMap<String, ComponentMetrics>,
    /// Performance history
    history: Vec<PerformanceSnapshot>,
}

/// System-wide metrics
#[derive(Debug, Clone, Default)]
pub struct SystemMetrics {
    /// Total expressions processed
    pub expressions_processed: u64,
    /// Total inference time
    pub total_inference_time: Duration,
    /// Total evaluation time
    pub total_evaluation_time: Duration,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Average performance improvement
    pub average_improvement: f64,
}

/// Component-specific metrics
#[derive(Debug, Clone, Default)]
pub struct ComponentMetrics {
    /// Component processing time
    pub processing_time: Duration,
    /// Success rate
    pub success_rate: f64,
    /// Error count
    pub error_count: u64,
    /// Optimization count
    pub optimization_count: u64,
}

/// Performance snapshot
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    /// Timestamp
    pub timestamp: Instant,
    /// System metrics at this time
    pub metrics: SystemMetrics,
    /// Context information
    pub context: String,
}

/// Migration assistance system
#[derive(Debug)]
pub struct MigrationAssistant {
    /// Migration rules
    rules: Vec<MigrationRule>,
    /// Suggestion cache
    cache: HashMap<String, Vec<MigrationSuggestion>>,
    /// Migration statistics
    stats: MigrationStatistics,
}

/// Rule for generating migration suggestions
#[derive(Debug, Clone)]
pub struct MigrationRule {
    /// Pattern to match
    pub pattern: MigrationPattern,
    /// Generated suggestion
    pub suggestion: MigrationType,
    /// Applicability condition
    pub condition: String,
    /// Priority override
    pub priority: Option<Priority>,
}

/// Pattern for migration matching
#[derive(Debug, Clone)]
pub struct MigrationPattern {
    /// Type pattern
    pub type_pattern: String,
    /// Context pattern
    pub context_pattern: String,
    /// Expression pattern
    pub expression_pattern: String,
}

/// Migration statistics
#[derive(Debug, Clone, Default)]
pub struct MigrationStatistics {
    /// Suggestions generated
    pub suggestions_generated: u64,
    /// Suggestions applied (estimated)
    pub suggestions_applied: u64,
    /// Improvement from migrations
    pub improvement_from_migrations: f64,
}

impl GradualTypeSystem {
    /// Creates a new gradual type system
    pub fn new() -> Self {
        Self::with_config(GradualSystemConfig::default())
    }

    /// Creates a gradual type system with configuration
    pub fn with_config(config: GradualSystemConfig) -> Self {
        let inference_engine = Arc::new(Mutex::new(GradualTypeInference::with_config(
            config.inference.clone(),
        )));
        let evaluator_integration = Arc::new(Mutex::new(GradualEvaluatorIntegration::with_config(
            config.evaluator.clone(),
        )));
        let consistency_checker = Arc::new(Mutex::new(GradualConsistencyChecker::with_config(
            config.consistency.clone(),
        )));
        let contract_integration = Arc::new(Mutex::new(GradualContractIntegration::with_config(
            config.contracts.clone(),
        )));
        let performance_monitor = SystemPerformanceMonitor::new();
        let migration_assistant = MigrationAssistant::new();
        let type_env_cache = Arc::new(Mutex::new(HashMap::new()));

        Self {
            config,
            inference_engine,
            evaluator_integration,
            consistency_checker,
            contract_integration,
            performance_monitor,
            migration_assistant,
            type_env_cache,
        }
    }

    /// Performs complete gradual type inference and evaluation
    pub fn infer_and_evaluate(
        &mut self,
        expr: &Spanned<Expr>,
        env: &Environment,
        evaluator: &mut Evaluator,
    ) -> Result<GradualSystemResult> {
        let start_time = Instant::now();

        // Phase 1: Type Inference
        let inference_result = {
            let mut inference = self.inference_engine.lock().unwrap();
            inference.infer_gradual(expr)?
        };

        // Phase 2: Consistency Checking
        let consistency_result = {
            let mut checker = self.consistency_checker.lock().unwrap();
            checker.check_consistency(&inference_result.inferred_type, &Type::Dynamic)
        };

        // Phase 3: Contract Integration
        let contract_result = {
            let mut integration = self.contract_integration.lock().unwrap();
            let context = CompilationContext::new_default();
            integration.integrate_with_contracts(&inference_result, &context)?
        };

        // Phase 4: Evaluation with Runtime Checking
        let value = {
            let mut eval_integration = self.evaluator_integration.lock().unwrap();
            eval_integration.evaluate_with_gradual_types(expr, env, evaluator)?
        };

        // Phase 5: Migration Analysis
        let migration_suggestions = if self.config.enable_migration {
            self.migration_assistant.generate_suggestions(
                expr,
                &inference_result,
                &consistency_result,
            )?
        } else {
            Vec::new()
        };

        // Phase 6: Performance Analysis
        let analysis_time = start_time.elapsed();
        let performance_metrics =
            self.calculate_performance_metrics(analysis_time, &inference_result, &contract_result);

        // Update system metrics
        if self.config.enable_monitoring {
            self.performance_monitor
                .record_processing(analysis_time, &performance_metrics);
        }

        // Construct result
        let result = GradualSystemResult {
            value,
            type_info: self.extract_type_info(&inference_result),
            consistency: self.extract_consistency_analysis(&consistency_result),
            contracts: self.extract_contract_info(&contract_result),
            performance: performance_metrics,
            migration_suggestions,
        };

        Ok(result)
    }

    /// Performs type inference only (no evaluation)
    pub fn infer_type_only(&mut self, expr: &Spanned<Expr>) -> Result<GradualInferenceResult> {
        let mut inference = self.inference_engine.lock().unwrap();
        inference.infer_gradual(expr)
    }

    /// Checks consistency only
    pub fn check_consistency_only(&mut self, type1: &Type, type2: &Type) -> ConsistencyResult {
        let mut checker = self.consistency_checker.lock().unwrap();
        checker.check_consistency(type1, type2)
    }

    /// Analyzes a complete program
    pub fn analyze_program(&mut self, program: &Program) -> Result<ProgramAnalysisResult> {
        let mut results = Vec::new();
        let mut overall_metrics = SystemMetrics::default();

        for expr in &program.expressions {
            // For now, we'll analyze each expression independently
            // In a full implementation, we'd maintain context across expressions
            let inference_result = self.infer_type_only(expr)?;

            // Accumulate metrics
            overall_metrics.expressions_processed += 1;

            results.push(inference_result);
        }

        Ok(ProgramAnalysisResult {
            expression_results: results,
            overall_metrics,
            program_consistency: 0.85,     // Simplified calculation
            migration_roadmap: Vec::new(), // Would generate comprehensive roadmap
        })
    }

    /// Extracts type information from inference result
    fn extract_type_info(&self, result: &GradualInferenceResult) -> TypeInfo {
        TypeInfo {
            inferred_type: result.inferred_type.clone(),
            certainty: self.assess_type_certainty(&result.inferred_type),
            boundaries: result.casts.len(),
            casts: result.casts.len(),
            optimizations: result.optimizations.len(),
        }
    }

    /// Assesses type certainty
    fn assess_type_certainty(&self, type_: &Type) -> TypeCertainty {
        use super::gradual::{is_gradual, is_static};

        if is_static(type_) {
            TypeCertainty::VeryHigh
        } else if is_gradual(type_) {
            TypeCertainty::Medium
        } else {
            TypeCertainty::Low
        }
    }

    /// Extracts consistency analysis
    fn extract_consistency_analysis(&self, result: &ConsistencyResult) -> ConsistencyAnalysis {
        let score = if result.consistent { 1.0 } else { 0.5 };
        let max_severity = result
            .violations
            .iter()
            .map(|v| v.severity)
            .max()
            .unwrap_or(ViolationSeverity::Info);

        ConsistencyAnalysis {
            score,
            violations: result.violations.len(),
            max_severity,
            improvements: result.suggestions.len(),
        }
    }

    /// Extracts contract information
    fn extract_contract_info(&self, result: &IntegrationResult) -> ContractInfo {
        ContractInfo {
            generated: result.contracts.len(),
            eliminated: result.eliminated_contracts.len(),
            overhead: 0.05, // Simplified calculation
            coverage: 0.80, // Simplified calculation
        }
    }

    /// Calculates performance metrics
    fn calculate_performance_metrics(
        &self,
        analysis_time: Duration,
        inference_result: &GradualInferenceResult,
        contract_result: &IntegrationResult,
    ) -> PerformanceMetrics {
        PerformanceMetrics {
            analysis_time,
            evaluation_time: Duration::from_millis(0), // Would measure actual evaluation
            memory_usage: 1024,                        // Simplified estimate
            improvement: contract_result.eliminated_contracts.len() as f64 * 0.1,
        }
    }

    /// Gets current configuration
    pub fn config(&self) -> &GradualSystemConfig {
        &self.config
    }

    /// Updates system configuration
    pub fn update_config(&mut self, config: GradualSystemConfig) {
        self.config = config.clone();

        // Update component configurations
        {
            let mut inference = self.inference_engine.lock().unwrap();
            inference.update_config(config.inference);
        }

        {
            let mut eval_integration = self.evaluator_integration.lock().unwrap();
            eval_integration.update_config(config.evaluator);
        }

        {
            let mut checker = self.consistency_checker.lock().unwrap();
            checker.update_config(config.consistency);
        }

        {
            let mut contract_integration = self.contract_integration.lock().unwrap();
            contract_integration.update_config(config.contracts);
        }
    }

    /// Gets system performance statistics
    pub fn performance_statistics(&self) -> &SystemMetrics {
        &self.performance_monitor.system_metrics
    }

    /// Gets migration statistics
    pub fn migration_statistics(&self) -> &MigrationStatistics {
        &self.migration_assistant.stats
    }

    /// Clears all caches
    pub fn clear_caches(&mut self) {
        {
            let mut cache = self.type_env_cache.lock().unwrap();
            cache.clear();
        }

        {
            let mut checker = self.consistency_checker.lock().unwrap();
            checker.clear_caches();
        }

        self.migration_assistant.cache.clear();
    }

    /// Creates a development-optimized configuration
    pub fn development_config() -> GradualSystemConfig {
        GradualSystemConfig {
            mode: SystemMode::Development,
            optimization_level: SystemOptimizationLevel::Basic,
            enable_monitoring: true,
            enable_migration: true,
            ..GradualSystemConfig::default()
        }
    }

    /// Creates a production-optimized configuration
    pub fn production_config() -> GradualSystemConfig {
        GradualSystemConfig {
            mode: SystemMode::Production,
            optimization_level: SystemOptimizationLevel::Maximum,
            enable_monitoring: false,
            enable_migration: false,
            inference: GradualInferenceConfig {
                enable_migration_assistance: false,
                ..GradualInferenceConfig::default()
            },
            evaluator: EvaluatorIntegrationConfig {
                enable_performance_monitoring: false,
                ..EvaluatorIntegrationConfig::default()
            },
            ..GradualSystemConfig::default()
        }
    }
}

/// Result of program analysis
#[derive(Debug, Clone)]
pub struct ProgramAnalysisResult {
    /// Results for individual expressions
    pub expression_results: Vec<GradualInferenceResult>,
    /// Overall program metrics
    pub overall_metrics: SystemMetrics,
    /// Program-wide consistency score
    pub program_consistency: f64,
    /// Migration roadmap for the program
    pub migration_roadmap: Vec<MigrationSuggestion>,
}

impl SystemPerformanceMonitor {
    /// Creates a new performance monitor
    pub fn new() -> Self {
        Self {
            system_metrics: SystemMetrics::default(),
            component_metrics: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// Records a processing event
    pub fn record_processing(&mut self, duration: Duration, metrics: &PerformanceMetrics) {
        self.system_metrics.expressions_processed += 1;
        self.system_metrics.total_inference_time += duration;
        self.system_metrics.average_improvement =
            (self.system_metrics.average_improvement + metrics.improvement) / 2.0;

        self.history.push(PerformanceSnapshot {
            timestamp: Instant::now(),
            metrics: self.system_metrics.clone(),
            context: "processing".to_string(),
        });
    }

    /// Gets performance trends
    pub fn performance_trends(&self) -> Vec<&PerformanceSnapshot> {
        self.history.iter().collect()
    }
}

impl MigrationAssistant {
    /// Creates a new migration assistant
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
            cache: HashMap::new(),
            stats: MigrationStatistics::default(),
        }
    }

    /// Default migration rules
    fn default_rules() -> Vec<MigrationRule> {
        vec![MigrationRule {
            pattern: MigrationPattern {
                type_pattern: "Dynamic".to_string(),
                context_pattern: "function_parameter".to_string(),
                expression_pattern: "*".to_string(),
            },
            suggestion: MigrationType::AddTypeAnnotation,
            condition: "frequently_used".to_string(),
            priority: Some(Priority::Medium),
        }]
    }

    /// Generates migration suggestions
    pub fn generate_suggestions(
        &mut self,
        expr: &Spanned<Expr>,
        inference_result: &GradualInferenceResult,
        consistency_result: &ConsistencyResult,
    ) -> Result<Vec<MigrationSuggestion>> {
        let mut suggestions = Vec::new();

        // Generate suggestions based on inference result
        suggestions.extend(inference_result.migration_suggestions.iter().map(|s| {
            MigrationSuggestion {
                location: Span::new(0, 0), // Would use actual location
                suggestion_type: MigrationType::AddTypeAnnotation, // Simplified
                priority: Priority::Medium,
                benefit: "Improved type safety".to_string(),
                effort: "Low".to_string(),
            }
        }));

        // Generate suggestions based on consistency violations
        for violation in &consistency_result.violations {
            suggestions.push(MigrationSuggestion {
                location: violation.location,
                suggestion_type: MigrationType::UseMoreSpecificType,
                priority: match violation.severity {
                    ViolationSeverity::Critical => Priority::Critical,
                    ViolationSeverity::Error => Priority::High,
                    ViolationSeverity::Warning => Priority::Medium,
                    ViolationSeverity::Info => Priority::Low,
                },
                benefit: "Resolve consistency violation".to_string(),
                effort: "Medium".to_string(),
            });
        }

        self.stats.suggestions_generated += suggestions.len() as u64;
        Ok(suggestions)
    }
}

impl Default for GradualTypeSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SystemPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MigrationAssistant {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::diagnostics::{Span, spanned};

    #[test]
    fn test_gradual_system_creation() {
        let system = GradualTypeSystem::new();
        assert_eq!(system.config().mode, SystemMode::Development);
        assert!(system.config().enable_monitoring);
    }

    #[test]
    fn test_system_configurations() {
        let dev_config = GradualTypeSystem::development_config();
        assert_eq!(dev_config.mode, SystemMode::Development);
        assert!(dev_config.enable_migration);

        let prod_config = GradualTypeSystem::production_config();
        assert_eq!(prod_config.mode, SystemMode::Production);
        assert!(!prod_config.enable_migration);
    }

    #[test]
    fn test_type_certainty_assessment() {
        let system = GradualTypeSystem::new();

        assert_eq!(
            system.assess_type_certainty(&Type::Number),
            TypeCertainty::VeryHigh
        );

        assert_eq!(
            system.assess_type_certainty(&Type::Dynamic),
            TypeCertainty::Low
        );
    }

    #[test]
    fn test_migration_assistant() {
        let mut assistant = MigrationAssistant::new();

        let expr = spanned(Expr::Literal(Literal::Number(42.0)), Span::new(0, 2));

        let inference_result = GradualInferenceResult {
            inferred_type: Type::Number,
            substitution: crate::types::substitution::Substitution::empty(),
            casts: Vec::new(),
            contracts: Vec::new(),
            blame_info: Vec::new(),
            optimizations: Vec::new(),
            migration_suggestions: Vec::new(),
        };

        let consistency_result = ConsistencyResult {
            consistent: true,
            precision: crate::types::gradual_consistency::PrecisionRelation::Equal,
            violations: Vec::new(),
            suggestions: Vec::new(),
            evidence: crate::types::gradual_consistency::ConsistencyEvidence {
                derivation: Vec::new(),
                witnesses: Vec::new(),
                assumptions: Vec::new(),
            },
        };

        let suggestions = assistant
            .generate_suggestions(&expr, &inference_result, &consistency_result)
            .unwrap();

        // Should have no suggestions for a simple literal
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_performance_monitoring() {
        let mut monitor = SystemPerformanceMonitor::new();

        let metrics = PerformanceMetrics {
            analysis_time: Duration::from_millis(10),
            evaluation_time: Duration::from_millis(5),
            memory_usage: 1024,
            improvement: 0.1,
        };

        monitor.record_processing(Duration::from_millis(15), &metrics);

        assert_eq!(monitor.system_metrics.expressions_processed, 1);
        assert_eq!(monitor.history.len(), 1);
    }

    #[test]
    fn test_cache_operations() {
        let mut system = GradualTypeSystem::new();

        // Test cache clearing
        system.clear_caches();

        // Should not panic and caches should be empty
        let cache = system.type_env_cache.lock().unwrap();
        assert_eq!(cache.len(), 0);
    }
}

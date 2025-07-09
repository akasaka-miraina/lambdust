//! Backward compatibility system for gradual migration to EvaluatorInterface
//!
//! This module provides compatibility layers that allow existing evaluator code
//! to work seamlessly with the new EvaluatorInterface architecture while
//! enabling gradual migration without breaking existing functionality.

use crate::ast::Expr;
use crate::environment::Environment;
use crate::error::{LambdustError, Result};
use crate::evaluator::{
    Continuation, EvaluatorInterface, EvaluationConfig, EvaluationMode, EvaluationResult,
    RuntimeOptimizationLevel, VerificationConfig,
};
use crate::value::Value;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Compatibility layer for legacy evaluator interface
pub struct LegacyEvaluatorAdapter {
    /// New evaluator interface
    evaluator_interface: Arc<Mutex<EvaluatorInterface>>,
    
    /// Configuration mapping for legacy settings
    legacy_config_mapping: HashMap<String, EvaluationConfig>,
    
    /// Migration statistics
    migration_stats: MigrationStatistics,
    
    /// Compatibility mode settings
    compatibility_mode: CompatibilityMode,
}

/// Migration statistics tracking
#[derive(Debug, Clone, Default)]
pub struct MigrationStatistics {
    /// Total evaluations performed through adapter
    pub total_evaluations: usize,
    
    /// Evaluations using legacy API
    pub legacy_api_calls: usize,
    
    /// Evaluations using new API
    pub new_api_calls: usize,
    
    /// Compatibility issues encountered
    pub compatibility_issues: usize,
    
    /// Performance comparisons
    pub performance_improvements: Vec<f64>,
    
    /// Migration progress (0.0 to 1.0)
    pub migration_progress: f64,
}

/// Compatibility mode configuration
#[derive(Debug, Clone)]
pub struct CompatibilityMode {
    /// Enable strict backward compatibility
    pub strict_compatibility: bool,
    
    /// Allow new features in legacy mode
    pub enable_new_features: bool,
    
    /// Automatic migration suggestions
    pub suggest_migrations: bool,
    
    /// Performance monitoring
    pub monitor_performance: bool,
    
    /// Error handling strategy
    pub error_handling: ErrorHandlingStrategy,
}

/// Error handling strategy for compatibility issues
#[derive(Debug, Clone)]
pub enum ErrorHandlingStrategy {
    /// Strict: fail on any compatibility issue
    Strict,
    /// Graceful: log warnings and continue
    Graceful,
    /// Fallback: attempt fallback to legacy behavior
    Fallback,
    /// Silent: ignore compatibility issues
    Silent,
}

/// Migration recommendation
#[derive(Debug, Clone)]
pub struct MigrationRecommendation {
    /// Type of migration recommended
    pub migration_type: MigrationType,
    
    /// Description of the recommendation
    pub description: String,
    
    /// Expected benefits
    pub expected_benefits: Vec<String>,
    
    /// Required changes
    pub required_changes: Vec<String>,
    
    /// Risk assessment
    pub risk_level: RiskLevel,
    
    /// Priority level
    pub priority: Priority,
}

/// Type of migration
#[derive(Debug, Clone)]
pub enum MigrationType {
    /// Replace legacy API calls with new interface
    ApiReplacement,
    
    /// Update configuration to use new features
    ConfigurationUpdate,
    
    /// Enable advanced verification
    VerificationUpgrade,
    
    /// Optimize evaluation modes
    ModeOptimization,
    
    /// Complete migration to new system
    FullMigration,
}

/// Risk level assessment
#[derive(Debug, Clone)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Priority level
#[derive(Debug, Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Compatibility result wrapper
#[derive(Debug, Clone)]
pub struct CompatibilityResult<T> {
    /// Result value
    pub value: T,
    
    /// Compatibility warnings
    pub warnings: Vec<String>,
    
    /// Migration recommendations
    pub recommendations: Vec<MigrationRecommendation>,
    
    /// Performance impact
    pub performance_impact: Option<f64>,
}

impl LegacyEvaluatorAdapter {
    /// Create new legacy evaluator adapter
    pub fn new() -> Self {
        Self {
            evaluator_interface: Arc::new(Mutex::new(EvaluatorInterface::new())),
            legacy_config_mapping: Self::init_legacy_config_mapping(),
            migration_stats: MigrationStatistics::default(),
            compatibility_mode: CompatibilityMode::default(),
        }
    }
    
    /// Create with custom compatibility mode
    pub fn with_compatibility_mode(mode: CompatibilityMode) -> Self {
        Self {
            evaluator_interface: Arc::new(Mutex::new(EvaluatorInterface::new())),
            legacy_config_mapping: Self::init_legacy_config_mapping(),
            migration_stats: MigrationStatistics::default(),
            compatibility_mode: mode,
        }
    }
    
    /// Initialize legacy configuration mapping
    fn init_legacy_config_mapping() -> HashMap<String, EvaluationConfig> {
        let mut mapping = HashMap::new();
        
        // Default legacy configuration
        mapping.insert("default".to_string(), EvaluationConfig {
            mode: EvaluationMode::Semantic,
            verify_correctness: false, // Legacy mode has minimal verification
            monitor_performance: false,
            fallback_to_semantic: true,
            verification_timeout_ms: 1000,
            verification_config: VerificationConfig {
                verify_semantic_equivalence: false,
                generate_correctness_proofs: false,
                use_theorem_proving: false,
                max_verification_time_ms: 500,
                enable_statistics: false,
                store_verification_history: false,
                max_history_entries: 100,
            },
        });
        
        // Performance legacy configuration
        mapping.insert("performance".to_string(), EvaluationConfig {
            mode: EvaluationMode::Runtime(RuntimeOptimizationLevel::Conservative),
            verify_correctness: false,
            monitor_performance: true,
            fallback_to_semantic: true,
            verification_timeout_ms: 500,
            verification_config: VerificationConfig::default(),
        });
        
        // Debug legacy configuration
        mapping.insert("debug".to_string(), EvaluationConfig {
            mode: EvaluationMode::Verification,
            verify_correctness: true,
            monitor_performance: true,
            fallback_to_semantic: true,
            verification_timeout_ms: 5000,
            verification_config: VerificationConfig::default(),
        });
        
        mapping
    }
    
    /// Legacy eval function for backward compatibility
    pub fn eval_legacy(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> CompatibilityResult<Result<Value>> {
        let start_time = std::time::Instant::now();
        self.migration_stats.total_evaluations += 1;
        self.migration_stats.legacy_api_calls += 1;
        
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();
        
        // Check for compatibility issues
        if self.compatibility_mode.suggest_migrations {
            recommendations.extend(self.analyze_migration_opportunities(&expr));
        }
        
        // Use legacy configuration
        let legacy_config = self.legacy_config_mapping
            .get("default")
            .cloned()
            .unwrap_or_default();
        
        // Clone values for potential fallback use
        let expr_clone = expr.clone();
        let env_clone = env.clone();
        let cont_clone = cont.clone();
        
        let result = {
            let mut evaluator = self.evaluator_interface.lock().unwrap();
            evaluator.set_config(legacy_config);
            evaluator.eval(expr, env, cont)
        };
        
        // Handle compatibility issues based on strategy
        let final_result = match &result {
            Ok(_) => result,
            Err(e) => match self.compatibility_mode.error_handling {
                ErrorHandlingStrategy::Strict => result,
                ErrorHandlingStrategy::Graceful => {
                    warnings.push(format!("Evaluation error (graceful): {}", e));
                    result
                }
                ErrorHandlingStrategy::Fallback => {
                    warnings.push("Attempting fallback to legacy behavior".to_string());
                    // Convert fallback Value result to EvaluationResult
                    match self.fallback_evaluation(&expr_clone, &env_clone, &cont_clone) {
                        Ok(value) => {
                            Ok(EvaluationResult {
                                value,
                                mode_used: EvaluationMode::Semantic,
                                evaluation_time_us: 0,
                                correctness_proof: None,
                                verification_result: None,
                                performance_metrics: crate::evaluator::PerformanceMetrics::default(),
                                fallback_used: true,
                            })
                        }
                        Err(fallback_err) => {
                            warnings.push(format!("Fallback also failed: {}", fallback_err));
                            result
                        }
                    }
                }
                ErrorHandlingStrategy::Silent => result,
            }
        };
        
        let evaluation_time = start_time.elapsed().as_micros() as f64 / 1000.0;
        
        // Convert EvaluationResult to Value for legacy API
        let legacy_result = final_result.map(|eval_result| eval_result.value);
        
        CompatibilityResult {
            value: legacy_result,
            warnings,
            recommendations,
            performance_impact: Some(evaluation_time),
        }
    }
    
    /// New API evaluation with compatibility wrapper
    pub fn eval_new(
        &mut self,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
        config: Option<EvaluationConfig>,
    ) -> CompatibilityResult<Result<EvaluationResult>> {
        let start_time = std::time::Instant::now();
        self.migration_stats.total_evaluations += 1;
        self.migration_stats.new_api_calls += 1;
        
        let warnings = Vec::new();
        let recommendations = Vec::new();
        
        let result = {
            let mut evaluator = self.evaluator_interface.lock().unwrap();
            if let Some(config) = config {
                evaluator.set_config(config);
            }
            evaluator.eval(expr, env, cont)
        };
        
        let evaluation_time = start_time.elapsed().as_micros() as f64 / 1000.0;
        
        // Update migration progress
        self.update_migration_progress();
        
        CompatibilityResult {
            value: result,
            warnings,
            recommendations,
            performance_impact: Some(evaluation_time),
        }
    }
    
    /// Fallback evaluation using minimal implementation
    fn fallback_evaluation(
        &self,
        expr: &Expr,
        env: &Rc<Environment>,
        _cont: &Continuation,
    ) -> Result<Value> {
        // Very basic fallback implementation
        match expr {
            Expr::Literal(literal) => {
                match literal {
                    crate::ast::Literal::Number(n) => Ok(Value::Number(n.clone())),
                    crate::ast::Literal::String(s) => Ok(Value::String(s.clone())),
                    crate::ast::Literal::Boolean(b) => Ok(Value::Boolean(*b)),
                    crate::ast::Literal::Character(c) => Ok(Value::Character(*c)),
                    crate::ast::Literal::Nil => Ok(Value::Nil),
                }
            }
            Expr::Variable(name) => {
                // Use get method instead of lookup for compatibility
                match env.get(name) {
                    Some(value) => Ok(value),
                    None => Err(LambdustError::runtime_error(format!("Unbound variable: {}", name)))
                }
            }
            _ => {
                // For complex expressions, return error
                Err(LambdustError::runtime_error(
                    "Fallback evaluation only supports literals and variables".to_string()
                ))
            }
        }
    }
    
    /// Analyze migration opportunities for an expression
    fn analyze_migration_opportunities(&self, expr: &Expr) -> Vec<MigrationRecommendation> {
        let mut recommendations = Vec::new();
        
        // Analyze expression complexity
        let complexity = self.calculate_expression_complexity(expr);
        
        if complexity > 5 {
            recommendations.push(MigrationRecommendation {
                migration_type: MigrationType::ModeOptimization,
                description: "Complex expression could benefit from runtime optimization".to_string(),
                expected_benefits: vec![
                    "Improved performance".to_string(),
                    "Better resource utilization".to_string(),
                ],
                required_changes: vec![
                    "Use EvaluationMode::Auto or Runtime".to_string(),
                ],
                risk_level: RiskLevel::Low,
                priority: Priority::Medium,
            });
        }
        
        if complexity > 10 {
            recommendations.push(MigrationRecommendation {
                migration_type: MigrationType::VerificationUpgrade,
                description: "Complex expression should use verification for correctness".to_string(),
                expected_benefits: vec![
                    "Improved correctness guarantees".to_string(),
                    "Better error detection".to_string(),
                ],
                required_changes: vec![
                    "Enable verification in EvaluationConfig".to_string(),
                ],
                risk_level: RiskLevel::Medium,
                priority: Priority::High,
            });
        }
        
        // Check if using legacy API
        if self.migration_stats.legacy_api_calls > self.migration_stats.new_api_calls {
            recommendations.push(MigrationRecommendation {
                migration_type: MigrationType::ApiReplacement,
                description: "Consider migrating to new EvaluatorInterface API".to_string(),
                expected_benefits: vec![
                    "Access to advanced features".to_string(),
                    "Better performance monitoring".to_string(),
                    "Enhanced verification capabilities".to_string(),
                ],
                required_changes: vec![
                    "Replace eval_legacy calls with eval_new".to_string(),
                    "Update configuration to use EvaluationConfig".to_string(),
                ],
                risk_level: RiskLevel::Low,
                priority: Priority::Medium,
            });
        }
        
        recommendations
    }
    
    /// Calculate expression complexity for migration analysis
    fn calculate_expression_complexity(&self, expr: &Expr) -> usize {
        match expr {
            Expr::Literal(_) | Expr::Variable(_) => 1,
            Expr::List(exprs) => {
                1 + exprs.iter().map(|e| self.calculate_expression_complexity(e)).sum::<usize>()
            }
            Expr::Quote(expr) => 1 + self.calculate_expression_complexity(expr),
            Expr::Vector(exprs) => {
                1 + exprs.iter().map(|e| self.calculate_expression_complexity(e)).sum::<usize>()
            }
            _ => 3, // Default complexity for other expressions
        }
    }
    
    /// Update migration progress based on usage patterns
    fn update_migration_progress(&mut self) {
        let total_calls = self.migration_stats.legacy_api_calls + self.migration_stats.new_api_calls;
        if total_calls > 0 {
            self.migration_stats.migration_progress = 
                self.migration_stats.new_api_calls as f64 / total_calls as f64;
        }
    }
    
    /// Get migration statistics
    pub fn get_migration_statistics(&self) -> &MigrationStatistics {
        &self.migration_stats
    }
    
    /// Get migration recommendations summary
    pub fn get_migration_recommendations(&self) -> Vec<MigrationRecommendation> {
        let mut recommendations = Vec::new();
        
        // Overall migration recommendation based on usage
        if self.migration_stats.migration_progress < 0.5 {
            recommendations.push(MigrationRecommendation {
                migration_type: MigrationType::FullMigration,
                description: "Consider full migration to new EvaluatorInterface".to_string(),
                expected_benefits: vec![
                    "Complete access to advanced features".to_string(),
                    "Better performance and verification".to_string(),
                    "Future-proof codebase".to_string(),
                ],
                required_changes: vec![
                    "Replace all legacy API calls".to_string(),
                    "Update configuration management".to_string(),
                    "Enable verification and optimization features".to_string(),
                ],
                risk_level: RiskLevel::Medium,
                priority: Priority::High,
            });
        }
        
        recommendations
    }
    
    /// Set compatibility mode
    pub fn set_compatibility_mode(&mut self, mode: CompatibilityMode) {
        self.compatibility_mode = mode;
    }
    
    /// Get compatibility mode
    pub fn get_compatibility_mode(&self) -> &CompatibilityMode {
        &self.compatibility_mode
    }
    
    /// Reset migration statistics
    pub fn reset_migration_statistics(&mut self) {
        self.migration_stats = MigrationStatistics::default();
    }
    
    /// Add custom legacy configuration
    pub fn add_legacy_config(&mut self, name: String, config: EvaluationConfig) {
        self.legacy_config_mapping.insert(name, config);
    }
    
    /// Get available legacy configurations
    pub fn get_legacy_configs(&self) -> Vec<String> {
        self.legacy_config_mapping.keys().cloned().collect()
    }
}

impl Default for CompatibilityMode {
    fn default() -> Self {
        Self {
            strict_compatibility: false,
            enable_new_features: true,
            suggest_migrations: true,
            monitor_performance: true,
            error_handling: ErrorHandlingStrategy::Graceful,
        }
    }
}

impl Default for LegacyEvaluatorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Migration helper functions
pub mod migration_helpers {
    use super::*;
    
    /// Convert legacy evaluation call to new API
    pub fn migrate_eval_call(
        adapter: &mut LegacyEvaluatorAdapter,
        expr: Expr,
        env: Rc<Environment>,
        cont: Continuation,
    ) -> Result<EvaluationResult> {
        let config = EvaluationConfig::default();
        let result = adapter.eval_new(expr, env, cont, Some(config));
        
        // Handle warnings and recommendations
        if !result.warnings.is_empty() {
            eprintln!("Migration warnings: {:?}", result.warnings);
        }
        
        if !result.recommendations.is_empty() {
            eprintln!("Migration recommendations: {:?}", result.recommendations);
        }
        
        result.value
    }
    
    /// Batch migrate multiple evaluation calls
    pub fn migrate_batch_eval(
        adapter: &mut LegacyEvaluatorAdapter,
        expressions: Vec<(Expr, Rc<Environment>, Continuation)>,
    ) -> Vec<Result<EvaluationResult>> {
        expressions.into_iter()
            .map(|(expr, env, cont)| migrate_eval_call(adapter, expr, env, cont))
            .collect()
    }
    
    /// Generate migration report
    pub fn generate_migration_report(adapter: &LegacyEvaluatorAdapter) -> String {
        let stats = adapter.get_migration_statistics();
        let recommendations = adapter.get_migration_recommendations();
        
        format!(
            "Migration Report:\n\
             Total evaluations: {}\n\
             Legacy API calls: {}\n\
             New API calls: {}\n\
             Migration progress: {:.2}%\n\
             Compatibility issues: {}\n\
             \n\
             Recommendations:\n{}\n",
            stats.total_evaluations,
            stats.legacy_api_calls,
            stats.new_api_calls,
            stats.migration_progress * 100.0,
            stats.compatibility_issues,
            recommendations.iter()
                .map(|r| format!("- {} ({}): {}", 
                    format!("{:?}", r.migration_type), 
                    format!("{:?}", r.priority),
                    r.description))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_legacy_adapter_creation() {
        let adapter = LegacyEvaluatorAdapter::new();
        let stats = adapter.get_migration_statistics();
        assert_eq!(stats.total_evaluations, 0);
        assert_eq!(stats.migration_progress, 0.0);
    }

    #[test]
    fn test_legacy_eval_call() {
        let mut adapter = LegacyEvaluatorAdapter::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        
        let result = adapter.eval_legacy(expr, env, Continuation::Identity);
        
        assert!(result.value.is_ok());
        assert_eq!(adapter.get_migration_statistics().legacy_api_calls, 1);
    }

    #[test]
    fn test_new_api_eval_call() {
        let mut adapter = LegacyEvaluatorAdapter::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        
        let result = adapter.eval_new(expr, env, Continuation::Identity, None);
        
        assert!(result.value.is_ok());
        assert_eq!(adapter.get_migration_statistics().new_api_calls, 1);
    }

    #[test]
    fn test_migration_progress_tracking() {
        let mut adapter = LegacyEvaluatorAdapter::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        
        // Make some legacy calls
        let _ = adapter.eval_legacy(expr.clone(), env.clone(), Continuation::Identity);
        let _ = adapter.eval_legacy(expr.clone(), env.clone(), Continuation::Identity);
        
        // Make some new API calls
        let _ = adapter.eval_new(expr.clone(), env.clone(), Continuation::Identity, None);
        
        let stats = adapter.get_migration_statistics();
        assert_eq!(stats.legacy_api_calls, 2);
        assert_eq!(stats.new_api_calls, 1);
        assert_eq!(stats.migration_progress, 1.0 / 3.0);
    }

    #[test]
    fn test_complexity_analysis() {
        let adapter = LegacyEvaluatorAdapter::new();
        
        let simple = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        assert_eq!(adapter.calculate_expression_complexity(&simple), 1);
        
        let complex = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::List(vec![
                Expr::Variable("*".to_string()),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
                Expr::Literal(Literal::Number(SchemeNumber::Integer(3))),
            ]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(4))),
        ]);
        assert!(adapter.calculate_expression_complexity(&complex) > 3);
    }

    #[test]
    fn test_compatibility_mode_configuration() {
        let mut adapter = LegacyEvaluatorAdapter::new();
        
        let strict_mode = CompatibilityMode {
            strict_compatibility: true,
            enable_new_features: false,
            suggest_migrations: false,
            monitor_performance: false,
            error_handling: ErrorHandlingStrategy::Strict,
        };
        
        adapter.set_compatibility_mode(strict_mode.clone());
        assert!(adapter.get_compatibility_mode().strict_compatibility);
    }

    #[test]
    fn test_legacy_config_management() {
        let mut adapter = LegacyEvaluatorAdapter::new();
        
        let custom_config = EvaluationConfig {
            mode: EvaluationMode::Runtime(RuntimeOptimizationLevel::Aggressive),
            verify_correctness: true,
            monitor_performance: true,
            fallback_to_semantic: false,
            verification_timeout_ms: 2000,
            verification_config: VerificationConfig::default(),
        };
        
        adapter.add_legacy_config("custom".to_string(), custom_config);
        
        let configs = adapter.get_legacy_configs();
        assert!(configs.contains(&"custom".to_string()));
        assert!(configs.contains(&"default".to_string()));
    }

    #[test]
    fn test_migration_recommendations() {
        let adapter = LegacyEvaluatorAdapter::new();
        let recommendations = adapter.get_migration_recommendations();
        
        // Should recommend full migration for new adapter
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| 
            matches!(r.migration_type, MigrationType::FullMigration)
        ));
    }

    #[test]
    fn test_fallback_evaluation() {
        let adapter = LegacyEvaluatorAdapter::new();
        let env = Rc::new(Environment::new());
        
        // Test literal fallback
        let literal = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let result = adapter.fallback_evaluation(&literal, &env, &Continuation::Identity);
        assert!(result.is_ok());
        
        // Test complex expression fallback (should fail)
        let complex = Expr::List(vec![
            Expr::Variable("+".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(1))),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(2))),
        ]);
        let result = adapter.fallback_evaluation(&complex, &env, &Continuation::Identity);
        assert!(result.is_err());
    }

    #[test]
    fn test_migration_helpers() {
        use migration_helpers::*;
        
        let mut adapter = LegacyEvaluatorAdapter::new();
        let expr = Expr::Literal(Literal::Number(SchemeNumber::Integer(42)));
        let env = Rc::new(Environment::new());
        
        let result = migrate_eval_call(&mut adapter, expr, env, Continuation::Identity);
        assert!(result.is_ok());
        
        let report = generate_migration_report(&adapter);
        assert!(report.contains("Migration Report"));
        assert!(report.contains("Total evaluations: 1"));
    }
}
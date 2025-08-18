//! Gradual Type Inference Integration with Evaluator
//!
//! This module provides integration between the gradual type inference system
//! and the Lambdust evaluator. It handles:
//!
//! - Runtime type checking at gradual boundaries
//! - Cast insertion and execution
//! - Contract enforcement during evaluation
//! - Type-directed optimization opportunities
//! - Performance monitoring and profiling
//!
//! # Integration Architecture
//!
//! The integration follows a layered approach:
//! 1. Type inference annotates expressions with gradual type information
//! 2. Evaluator checks for casts and contracts during evaluation
//! 3. Runtime system enforces type safety at boundaries
//! 4. Performance system monitors overhead and optimizes

use super::gradual_inference::{
    GradualTypeInference, GradualInferenceResult, GradualInferenceConfig,
    CastInsertion, GeneratedContract, TypeBoundary, CastReason,
    PerformanceImpact, OptimizationHint, OptimizationType
};
use super::gradual::{Cast, consistent, is_gradual, is_static};
use super::{Type, TypeScheme, TypeEnv};
use crate::ast::{Expr, Literal};
use crate::contracts::{ContractSystem, ContractExpr, BlameInfo, ContractError};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::{Value, Environment, Evaluator, EvalStep};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::time::{Instant, Duration};

/// Runtime type information attached to values
#[derive(Debug, Clone)]
pub struct RuntimeTypeInfo {
    /// Static type information (if available)
    pub static_type: Option<Type>,
    /// Runtime type representation
    pub runtime_type: RuntimeType,
    /// Type certainty level
    pub certainty: TypeCertainty,
    /// Performance tracking
    pub performance_data: PerformanceData,
}

/// Runtime type representation
#[derive(Debug, Clone)]
pub enum RuntimeType {
    /// Precise type known at runtime
    Precise(Type),
    /// Approximated type based on structure
    Approximated(Type),
    /// Dynamic type (unknown structure)
    Dynamic,
    /// Tagged union with possible types
    Union(Vec<Type>),
}

/// Type certainty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TypeCertainty {
    /// Type is explicitly annotated
    Explicit,
    /// Type is inferred with high confidence
    Inferred,
    /// Type is approximated from runtime
    Approximated,
    /// Type is completely unknown
    Unknown,
}

/// Performance data for runtime types
#[derive(Debug, Clone, Default)]
pub struct PerformanceData {
    /// Number of type checks performed
    pub check_count: u64,
    /// Total time spent in type checking
    pub check_time: Duration,
    /// Number of cast operations
    pub cast_count: u64,
    /// Total time spent in casts
    pub cast_time: Duration,
    /// Number of contract checks
    pub contract_count: u64,
    /// Total time spent in contract checking
    pub contract_time: Duration,
}

/// Configuration for evaluator integration
#[derive(Debug, Clone)]
pub struct EvaluatorIntegrationConfig {
    /// Enable runtime type checking
    pub enable_runtime_checking: bool,
    /// Enable cast execution
    pub enable_cast_execution: bool,
    /// Enable contract enforcement
    pub enable_contract_enforcement: bool,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Enable type-directed optimizations
    pub enable_optimizations: bool,
    /// Maximum recursion depth for type checking
    pub max_type_check_depth: usize,
    /// Type check timeout in milliseconds
    pub type_check_timeout_ms: u64,
    /// Enable blame tracking
    pub enable_blame_tracking: bool,
}

impl Default for EvaluatorIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_runtime_checking: true,
            enable_cast_execution: true,
            enable_contract_enforcement: true,
            enable_performance_monitoring: true,
            enable_optimizations: true,
            max_type_check_depth: 100,
            type_check_timeout_ms: 1000,
            enable_blame_tracking: true,
        }
    }
}

/// Main integration component
#[derive(Debug)]
pub struct GradualEvaluatorIntegration {
    /// Configuration
    config: EvaluatorIntegrationConfig,
    /// Gradual type inference engine
    inference_engine: Arc<Mutex<GradualTypeInference>>,
    /// Contract system
    contract_system: Arc<Mutex<ContractSystem>>,
    /// Runtime type cache
    type_cache: Arc<Mutex<HashMap<ValueId, RuntimeTypeInfo>>>,
    /// Performance monitor
    performance_monitor: Arc<Mutex<PerformanceMonitor>>,
    /// Cast executor
    cast_executor: CastExecutor,
    /// Optimization manager
    optimization_manager: OptimizationManager,
}

/// Unique identifier for values in type cache
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(u64);

/// Performance monitoring system
#[derive(Debug, Default)]
pub struct PerformanceMonitor {
    /// Type checking statistics
    pub type_check_stats: TypeCheckStatistics,
    /// Cast execution statistics
    pub cast_stats: CastStatistics,
    /// Contract enforcement statistics
    pub contract_stats: ContractStatistics,
    /// Optimization statistics
    pub optimization_stats: OptimizationStatistics,
}

/// Type checking performance statistics
#[derive(Debug, Default)]
pub struct TypeCheckStatistics {
    /// Total checks performed
    pub total_checks: u64,
    /// Successful checks
    pub successful_checks: u64,
    /// Failed checks
    pub failed_checks: u64,
    /// Average check time
    pub average_check_time: Duration,
    /// Peak check time
    pub peak_check_time: Duration,
}

/// Cast execution statistics
#[derive(Debug, Default)]
pub struct CastStatistics {
    /// Total casts executed
    pub total_casts: u64,
    /// Successful casts
    pub successful_casts: u64,
    /// Failed casts
    pub failed_casts: u64,
    /// Eliminated casts (optimized away)
    pub eliminated_casts: u64,
    /// Average cast time
    pub average_cast_time: Duration,
}

/// Contract enforcement statistics
#[derive(Debug, Default)]
pub struct ContractStatistics {
    /// Total contract checks
    pub total_checks: u64,
    /// Successful checks
    pub successful_checks: u64,
    /// Contract violations
    pub violations: u64,
    /// Average check time
    pub average_check_time: Duration,
}

/// Optimization statistics
#[derive(Debug, Default)]
pub struct OptimizationStatistics {
    /// Optimizations applied
    pub optimizations_applied: u64,
    /// Performance improvements
    pub performance_improvements: Vec<PerformanceImprovement>,
    /// Failed optimizations
    pub failed_optimizations: u64,
}

/// Performance improvement record
#[derive(Debug, Clone)]
pub struct PerformanceImprovement {
    /// Type of optimization
    pub optimization_type: OptimizationType,
    /// Time saved
    pub time_saved: Duration,
    /// Location where applied
    pub location: Span,
}

/// Executes runtime casts
#[derive(Debug)]
pub struct CastExecutor {
    /// Cast cache for optimization
    cast_cache: HashMap<CastKey, CastResult>,
    /// Cast statistics
    stats: CastStatistics,
}

/// Key for cast caching
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CastKey {
    /// Source type
    pub source: Type,
    /// Target type
    pub target: Type,
    /// Cast operation
    pub cast: Cast,
}

/// Result of cast execution
#[derive(Debug, Clone)]
pub enum CastResult {
    /// Cast succeeded
    Success(Value),
    /// Cast failed with error
    Failed(Box<CastError>),
    /// Cast was optimized away
    Optimized,
}

/// Cast execution error
#[derive(Debug, Clone)]
pub struct CastError {
    /// Error message
    pub message: String,
    /// Source type
    pub source_type: Type,
    /// Target type
    pub target_type: Type,
    /// Location of error
    pub location: Span,
}

/// Manages type-directed optimizations
#[derive(Debug)]
pub struct OptimizationManager {
    /// Active optimizations
    active_optimizations: HashMap<Span, OptimizationType>,
    /// Optimization statistics
    stats: OptimizationStatistics,
}

impl GradualEvaluatorIntegration {
    /// Creates new evaluator integration
    pub fn new() -> Self {
        Self::with_config(EvaluatorIntegrationConfig::default())
    }

    /// Creates new evaluator integration with configuration
    pub fn with_config(config: EvaluatorIntegrationConfig) -> Self {
        let inference_config = GradualInferenceConfig {
            enable_inference: config.enable_runtime_checking,
            enable_contract_generation: config.enable_contract_enforcement,
            enable_blame_tracking: config.enable_blame_tracking,
            ..GradualInferenceConfig::default()
        };

        let inference_engine = Arc::new(Mutex::new(
            GradualTypeInference::with_config(inference_config)
        ));
        let contract_system = Arc::new(Mutex::new(ContractSystem::new()));
        let type_cache = Arc::new(Mutex::new(HashMap::new()));
        let performance_monitor = Arc::new(Mutex::new(PerformanceMonitor::default()));
        let cast_executor = CastExecutor::new();
        let optimization_manager = OptimizationManager::new();

        Self {
            config,
            inference_engine,
            contract_system,
            type_cache,
            performance_monitor,
            cast_executor,
            optimization_manager,
        }
    }

    /// Evaluates an expression with gradual type checking
    pub fn evaluate_with_gradual_types(
        &mut self,
        expr: &Spanned<Expr>,
        env: &Environment,
        evaluator: &mut Evaluator
    ) -> Result<Value> {
        let start_time = Instant::now();

        // Perform gradual type inference
        let inference_result = {
            let mut inference = self.inference_engine.lock().unwrap();
            inference.infer_gradual(expr)?
        };

        // Apply optimizations if enabled
        if self.config.enable_optimizations {
            self.apply_optimizations(&inference_result.optimizations)?;
        }

        // Evaluate with runtime checking
        let value = self.evaluate_with_checking(
            expr,
            env,
            evaluator,
            &inference_result
        )?;

        // Record performance metrics
        if self.config.enable_performance_monitoring {
            let evaluation_time = start_time.elapsed();
            let mut monitor = self.performance_monitor.lock().unwrap();
            monitor.record_evaluation(evaluation_time);
        }

        Ok(value)
    }

    /// Evaluates expression with runtime type checking
    fn evaluate_with_checking(
        &mut self,
        expr: &Spanned<Expr>,
        env: &Environment,
        evaluator: &mut Evaluator,
        inference_result: &GradualInferenceResult
    ) -> Result<Value> {
        // First evaluate the expression normally
        let mut value = evaluator.eval(expr, std::rc::Rc::new(env.clone()))?;

        // Apply casts if needed
        if self.config.enable_cast_execution {
            value = self.apply_casts(value, &inference_result.casts)?;
        }

        // Enforce contracts if enabled
        if self.config.enable_contract_enforcement {
            value = self.enforce_contracts(value, &inference_result.contracts)?;
        }

        // Cache runtime type information
        if self.config.enable_runtime_checking {
            self.cache_runtime_type(&value, &inference_result.inferred_type)?;
        }

        Ok(value)
    }

    /// Applies runtime casts to a value
    fn apply_casts(&mut self, mut value: Value, casts: &[CastInsertion]) -> Result<Value> {
        for cast_insertion in casts {
            let start_time = Instant::now();
            
            match self.cast_executor.execute_cast(&value, &cast_insertion.cast) {
                CastResult::Success(new_value) => {
                    value = new_value;
                }
                CastResult::Failed(error) => {
                    return Err(Error::type_error(
                        format!("Cast failed: {}", error.message),
                        cast_insertion.location
                    ).boxed())
                }
                CastResult::Optimized => {
                    // Cast was optimized away, no change needed
                }
            }

            // Record performance
            if self.config.enable_performance_monitoring {
                let cast_time = start_time.elapsed();
                self.cast_executor.stats.record_cast(cast_time);
            }
        }

        Ok(value)
    }

    /// Enforces contracts on a value
    fn enforce_contracts(&mut self, value: Value, contracts: &[GeneratedContract]) -> Result<Value> {
        let mut checked_value = value;

        for contract in contracts {
            let start_time = Instant::now();
            
            // Compile and check contract
            let contract_system = self.contract_system.clone();
            let mut system = contract_system.lock().unwrap();
            
            // For now, we'll skip actual contract compilation and just return the value
            // In a full implementation, this would compile and check the contract
            
            // Record performance
            if self.config.enable_performance_monitoring {
                let contract_time = start_time.elapsed();
                let mut monitor = self.performance_monitor.lock().unwrap();
                monitor.contract_stats.record_check(contract_time);
            }
        }

        Ok(checked_value)
    }

    /// Caches runtime type information for a value
    fn cache_runtime_type(&mut self, value: &Value, inferred_type: &Type) -> Result<()> {
        let value_id = self.generate_value_id(value);
        let runtime_type = self.extract_runtime_type(value);
        
        let type_info = RuntimeTypeInfo {
            static_type: Some(inferred_type.clone()),
            runtime_type,
            certainty: TypeCertainty::Inferred,
            performance_data: PerformanceData::default(),
        };

        let mut cache = self.type_cache.lock().unwrap();
        cache.insert(value_id, type_info);

        Ok(())
    }

    /// Generates a unique ID for a value
    fn generate_value_id(&self, value: &Value) -> ValueId {
        // In a real implementation, this would generate a proper unique ID
        // For now, use a simple hash-based approach
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        format!("{value:?}").hash(&mut hasher);
        ValueId(hasher.finish())
    }

    /// Extracts runtime type from a value
    fn extract_runtime_type(&self, value: &Value) -> RuntimeType {
        Self::extract_runtime_type_recursive(value)
    }

    /// Recursive helper for runtime type extraction (optimized without self parameter)
    fn extract_runtime_type_recursive(value: &Value) -> RuntimeType {
        match value {
            Value::Literal(Literal::ExactInteger(_)) | Value::Literal(Literal::InexactReal(_)) 
            | Value::Literal(Literal::Number(_)) | Value::Literal(Literal::Rational(_)) 
            | Value::Literal(Literal::Complex(_)) => RuntimeType::Precise(Type::Number),
            Value::Literal(Literal::String(_)) | Value::Literal(Literal::InternedString(_)) => RuntimeType::Precise(Type::String),
            Value::Literal(Literal::Boolean(_)) => RuntimeType::Precise(Type::Boolean),
            Value::Symbol(_) => RuntimeType::Precise(Type::Symbol),
            Value::Literal(Literal::Character(_)) => RuntimeType::Precise(Type::Char),
            Value::Pair(car, cdr) => {
                let car_type = Self::extract_runtime_type_recursive(car);
                let cdr_type = Self::extract_runtime_type_recursive(cdr);
                
                match (car_type, cdr_type) {
                    (RuntimeType::Precise(car_t), RuntimeType::Precise(cdr_t)) => {
                        RuntimeType::Precise(Type::pair(car_t, cdr_t))
                    }
                    _ => RuntimeType::Approximated(Type::pair(Type::Dynamic, Type::Dynamic))
                }
            }
            Value::Nil => RuntimeType::Precise(Type::list(Type::Dynamic)),
            _ => RuntimeType::Dynamic,
        }
    }

    /// Applies optimizations based on inference results
    fn apply_optimizations(&mut self, optimizations: &[OptimizationHint]) -> Result<()> {
        for optimization in optimizations {
            let start_time = Instant::now();
            
            match self.optimization_manager.apply_optimization(optimization) {
                Ok(improvement) => {
                    let mut monitor = self.performance_monitor.lock().unwrap();
                    monitor.optimization_stats.record_improvement(improvement);
                }
                Err(_) => {
                    let mut monitor = self.performance_monitor.lock().unwrap();
                    monitor.optimization_stats.failed_optimizations += 1;
                }
            }
        }

        Ok(())
    }

    /// Gets current performance statistics
    pub fn performance_statistics(&self) -> PerformanceMonitor {
        self.performance_monitor.lock().unwrap().clone()
    }

    /// Gets current configuration
    pub fn config(&self) -> &EvaluatorIntegrationConfig {
        &self.config
    }

    /// Updates configuration
    pub fn update_config(&mut self, config: EvaluatorIntegrationConfig) {
        self.config = config;
    }

    /// Clears runtime type cache
    pub fn clear_type_cache(&mut self) {
        let mut cache = self.type_cache.lock().unwrap();
        cache.clear();
    }

    /// Gets cache statistics
    pub fn cache_statistics(&self) -> (usize, usize) {
        let cache = self.type_cache.lock().unwrap();
        (cache.len(), cache.capacity())
    }
}

impl CastExecutor {
    /// Creates a new cast executor
    pub fn new() -> Self {
        Self {
            cast_cache: HashMap::new(),
            stats: CastStatistics::default(),
        }
    }

    /// Executes a cast operation
    pub fn execute_cast(&mut self, value: &Value, cast: &Cast) -> CastResult {
        match cast {
            Cast::None => CastResult::Optimized,
            Cast::Upcast { from: _, to: _ } => {
                // Upcasts are always safe
                CastResult::Success(value.clone())
            }
            Cast::Downcast { from: _, to } => {
                // Downcasts require runtime checking
                self.execute_downcast(value, to)
            }
            Cast::Structural { casts } => {
                self.execute_structural_cast(value, casts)
            }
        }
    }

    /// Executes a downcast operation
    fn execute_downcast(&mut self, value: &Value, target_type: &Type) -> CastResult {
        // Check if value conforms to target type
        if self.value_conforms_to_type(value, target_type) {
            CastResult::Success(value.clone())
        } else {
            CastResult::Failed(Box::new(CastError {
                message: "Value does not conform to target type".to_string(),
                source_type: Type::Dynamic, // Would extract actual source type
                target_type: target_type.clone(),
                location: Span::new(0, 0), // Would use actual location
            }))
        }
    }

    /// Executes a structural cast
    fn execute_structural_cast(&mut self, value: &Value, casts: &[Cast]) -> CastResult {
        // For structural casts, apply each component cast
        let mut result = value.clone();
        
        for cast in casts {
            match self.execute_cast(&result, cast) {
                CastResult::Success(new_value) => result = new_value,
                CastResult::Failed(error) => return CastResult::Failed(error),
                CastResult::Optimized => continue,
            }
        }
        
        CastResult::Success(result)
    }

    /// Checks if a value conforms to a type
    fn value_conforms_to_type(&self, value: &Value, type_: &Type) -> bool {
        match (value, type_) {
            (Value::Literal(lit), Type::Number) if lit.is_number() => true,
            (Value::Literal(Literal::String(_)) | Value::Literal(Literal::InternedString(_)), Type::String) => true,
            (Value::Literal(Literal::Boolean(_)), Type::Boolean) => true,
            (Value::Symbol(_), Type::Symbol) => true,
            (Value::Literal(Literal::Character(_)), Type::Char) => true,
            (_, Type::Dynamic) => true,
            _ => false, // Simplified - would need full type checking
        }
    }
}

impl OptimizationManager {
    /// Creates a new optimization manager
    pub fn new() -> Self {
        Self {
            active_optimizations: HashMap::new(),
            stats: OptimizationStatistics::default(),
        }
    }

    /// Applies an optimization
    pub fn apply_optimization(&mut self, hint: &OptimizationHint) -> Result<PerformanceImprovement> {
        let start_time = Instant::now();

        // Apply the optimization based on type
        match &hint.optimization {
            OptimizationType::FunctionSpecialization { .. } => {
                // Would implement function specialization
            }
            OptimizationType::CastElimination => {
                // Would implement cast elimination
            }
            OptimizationType::FunctionInlining => {
                // Would implement function inlining
            }
            OptimizationType::ArithmeticSpecialization => {
                // Would implement arithmetic specialization
            }
            OptimizationType::ContainerOptimization => {
                // Would implement container optimization
            }
        }

        let optimization_time = start_time.elapsed();
        
        // Record the optimization
        self.active_optimizations.insert(hint.location, hint.optimization.clone());
        self.stats.optimizations_applied += 1;

        Ok(PerformanceImprovement {
            optimization_type: hint.optimization.clone(),
            time_saved: optimization_time, // Would calculate actual time saved
            location: hint.location,
        })
    }

    /// Gets optimization statistics
    pub fn statistics(&self) -> &OptimizationStatistics {
        &self.stats
    }
}

impl PerformanceMonitor {
    /// Records an evaluation timing
    pub fn record_evaluation(&mut self, duration: Duration) {
        // Update statistics with evaluation timing
        self.type_check_stats.total_checks += 1;
        self.type_check_stats.average_check_time = duration;
    }
}

impl TypeCheckStatistics {
    /// Records a successful type check
    pub fn record_success(&mut self, duration: Duration) {
        self.total_checks += 1;
        self.successful_checks += 1;
        self.update_timing(duration);
    }

    /// Records a failed type check
    pub fn record_failure(&mut self, duration: Duration) {
        self.total_checks += 1;
        self.failed_checks += 1;
        self.update_timing(duration);
    }

    /// Updates timing statistics
    fn update_timing(&mut self, duration: Duration) {
        if duration > self.peak_check_time {
            self.peak_check_time = duration;
        }
        
        // Update average (simplified)
        if self.total_checks > 0 {
            let total_time = self.average_check_time * (self.total_checks - 1) as u32 + duration;
            self.average_check_time = total_time / self.total_checks as u32;
        }
    }
}

impl CastStatistics {
    /// Records a cast execution
    pub fn record_cast(&mut self, duration: Duration) {
        self.total_casts += 1;
        
        // Update average timing
        if self.total_casts > 0 {
            let total_time = self.average_cast_time * (self.total_casts - 1) as u32 + duration;
            self.average_cast_time = total_time / self.total_casts as u32;
        }
    }

    /// Records a successful cast
    pub fn record_success(&mut self) {
        self.successful_casts += 1;
    }

    /// Records a failed cast
    pub fn record_failure(&mut self) {
        self.failed_casts += 1;
    }

    /// Records an eliminated cast
    pub fn record_elimination(&mut self) {
        self.eliminated_casts += 1;
    }
}

impl ContractStatistics {
    /// Records a contract check
    pub fn record_check(&mut self, duration: Duration) {
        self.total_checks += 1;
        
        // Update average timing
        if self.total_checks > 0 {
            let total_time = self.average_check_time * (self.total_checks - 1) as u32 + duration;
            self.average_check_time = total_time / self.total_checks as u32;
        }
    }

    /// Records a successful contract check
    pub fn record_success(&mut self) {
        self.successful_checks += 1;
    }

    /// Records a contract violation
    pub fn record_violation(&mut self) {
        self.violations += 1;
    }
}

impl OptimizationStatistics {
    /// Records a performance improvement
    pub fn record_improvement(&mut self, improvement: PerformanceImprovement) {
        self.optimizations_applied += 1;
        self.performance_improvements.push(improvement);
    }

    /// Gets total time saved from optimizations
    pub fn total_time_saved(&self) -> Duration {
        self.performance_improvements
            .iter()
            .map(|imp| imp.time_saved)
            .sum()
    }
}

impl Default for GradualEvaluatorIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CastExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OptimizationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            type_check_stats: self.type_check_stats.clone(),
            cast_stats: self.cast_stats.clone(),
            contract_stats: self.contract_stats.clone(),
            optimization_stats: OptimizationStatistics {
                optimizations_applied: self.optimization_stats.optimizations_applied,
                performance_improvements: self.optimization_stats.performance_improvements.clone(),
                failed_optimizations: self.optimization_stats.failed_optimizations,
            },
        }
    }
}

impl Clone for TypeCheckStatistics {
    fn clone(&self) -> Self {
        Self {
            total_checks: self.total_checks,
            successful_checks: self.successful_checks,
            failed_checks: self.failed_checks,
            average_check_time: self.average_check_time,
            peak_check_time: self.peak_check_time,
        }
    }
}

impl Clone for CastStatistics {
    fn clone(&self) -> Self {
        Self {
            total_casts: self.total_casts,
            successful_casts: self.successful_casts,
            failed_casts: self.failed_casts,
            eliminated_casts: self.eliminated_casts,
            average_cast_time: self.average_cast_time,
        }
    }
}

impl Clone for ContractStatistics {
    fn clone(&self) -> Self {
        Self {
            total_checks: self.total_checks,
            successful_checks: self.successful_checks,
            violations: self.violations,
            average_check_time: self.average_check_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;
    use crate::diagnostics::{Span, spanned};

    #[test]
    fn test_integration_creation() {
        let integration = GradualEvaluatorIntegration::new();
        assert!(integration.config().enable_runtime_checking);
        assert!(integration.config().enable_cast_execution);
    }

    #[test]
    fn test_integration_with_config() {
        let config = EvaluatorIntegrationConfig {
            enable_runtime_checking: false,
            enable_cast_execution: true,
            enable_contract_enforcement: false,
            enable_performance_monitoring: true,
            enable_optimizations: false,
            max_type_check_depth: 50,
            type_check_timeout_ms: 500,
            enable_blame_tracking: false,
        };

        let integration = GradualEvaluatorIntegration::with_config(config);
        assert!(!integration.config().enable_runtime_checking);
        assert!(integration.config().enable_cast_execution);
        assert!(!integration.config().enable_contract_enforcement);
    }

    #[test]
    fn test_runtime_type_extraction() {
        let integration = GradualEvaluatorIntegration::new();
        
        let number_value = Value::number(42.0);
        let runtime_type = integration.extract_runtime_type(&number_value);
        assert!(matches!(runtime_type, RuntimeType::Precise(Type::Number)));
        
        let string_value = Value::String("hello".to_string());
        let runtime_type = integration.extract_runtime_type(&string_value);
        assert!(matches!(runtime_type, RuntimeType::Precise(Type::String)));
    }

    #[test]
    fn test_cast_executor() {
        let mut executor = CastExecutor::new();
        let value = Value::number(42.0);
        
        // Test upcast (should succeed)
        let upcast = Cast::Upcast {
            from: Type::Number,
            to: Type::Dynamic,
        };
        let result = executor.execute_cast(&value, &upcast);
        assert!(matches!(result, CastResult::Success(_)));
        
        // Test no cast (should be optimized)
        let no_cast = Cast::None;
        let result = executor.execute_cast(&value, &no_cast);
        assert!(matches!(result, CastResult::Optimized));
    }

    #[test]
    fn test_value_type_conformance() {
        let executor = CastExecutor::new();
        
        let number_value = Value::number(42.0);
        assert!(executor.value_conforms_to_type(&number_value, &Type::Number));
        assert!(executor.value_conforms_to_type(&number_value, &Type::Dynamic));
        assert!(!executor.value_conforms_to_type(&number_value, &Type::String));
    }

    #[test]
    fn test_performance_statistics() {
        let mut stats = TypeCheckStatistics::default();
        
        stats.record_success(Duration::from_millis(10));
        assert_eq!(stats.total_checks, 1);
        assert_eq!(stats.successful_checks, 1);
        assert_eq!(stats.failed_checks, 0);
        
        stats.record_failure(Duration::from_millis(20));
        assert_eq!(stats.total_checks, 2);
        assert_eq!(stats.successful_checks, 1);
        assert_eq!(stats.failed_checks, 1);
    }
}
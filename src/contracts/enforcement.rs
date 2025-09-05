//! Runtime contract enforcement infrastructure.
//!
//! This module provides the runtime enforcement engine that checks
//! contracts during program execution. It handles:
//!
//! - Contract wrapping and unwrapping
//! - Higher-order contract enforcement
//! - Blame attribution and error reporting
//! - Performance monitoring and optimization

use crate::Literal;
use crate::contracts::{
    ContractConfig, ContractError, ContractResult,
    ast::ContractExpr,
    blame::{BlameBoundary, BlameInfo, BlameTarget, BlameTracker, BlameViolation, BoundaryType},
    compiler::{CompiledContract, ContractChecker},
};
use crate::diagnostics::{Span, Spanned};
use crate::eval::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};

/// Contract enforcement engine for runtime checking.
#[derive(Debug)]
pub struct ContractEnforcement {
    /// Configuration
    config: ContractConfig,
    /// Blame tracker
    blame_tracker: BlameTracker,
    /// Wrapped value cache
    wrapped_cache: Arc<RwLock<HashMap<WrappedValueId, WrappedValue>>>,
    /// Performance monitor
    performance_monitor: PerformanceMonitor,
    /// Next wrapped value ID
    next_wrapped_id: Arc<Mutex<WrappedValueId>>,
}

/// Unique identifier for wrapped values.
pub type WrappedValueId = u64;

/// A value wrapped with contract checking.
#[derive(Debug, Clone)]
pub struct WrappedValue {
    /// The original value
    pub value: Value,
    /// The contract being enforced
    pub contract: Arc<CompiledContract>,
    /// Blame information
    pub blame: BlameInfo,
    /// Wrapper metadata
    pub metadata: WrapperMetadata,
}

/// Metadata about contract wrappers.
#[derive(Debug, Clone)]
pub struct WrapperMetadata {
    /// Wrapper creation timestamp
    pub created_at: std::time::SystemTime,
    /// Number of checks performed
    pub check_count: usize,
    /// Total check time
    pub total_check_time: std::time::Duration,
    /// Last check result
    pub last_check_result: Option<bool>,
    /// Wrapper type
    pub wrapper_type: WrapperType,
}

/// Types of contract wrappers.
#[derive(Debug, Clone, PartialEq)]
pub enum WrapperType {
    /// Simple value wrapper
    Value,
    /// Function wrapper with domain/codomain checking
    Function,
    /// Higher-order function wrapper
    HigherOrder,
    /// Recursive contract wrapper
    Recursive,
    /// Parametric contract wrapper
    Parametric,
}

/// Performance monitor for contract checking.
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Total checks performed
    total_checks: Arc<Mutex<u64>>,
    /// Total check time
    total_check_time: Arc<Mutex<std::time::Duration>>,
    /// Check latency histogram
    latency_histogram: Arc<Mutex<HashMap<u64, u64>>>, // microseconds -> count
    /// Violation statistics
    violation_stats: Arc<Mutex<ViolationStats>>,
}

/// Statistics about contract violations.
#[derive(Debug, Clone)]
pub struct ViolationStats {
    /// Total violations
    pub total_violations: u64,
    /// Violations by contract type
    pub violations_by_type: HashMap<String, u64>,
    /// Average time to detect violations
    pub average_detection_time: std::time::Duration,
}

impl ContractEnforcement {
    /// Creates a new contract enforcement engine.
    pub fn new(config: &ContractConfig, blame_tracker: BlameTracker) -> Self {
        Self {
            config: config.clone(),
            blame_tracker,
            wrapped_cache: Arc::new(RwLock::new(HashMap::new())),
            performance_monitor: PerformanceMonitor::new(),
            next_wrapped_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Updates the configuration.
    pub fn update_config(&mut self, config: &ContractConfig) {
        self.config = config.clone();
    }

    /// Checks a value against a compiled contract.
    pub fn check(
        &mut self,
        value: &Value,
        contract: &CompiledContract,
        blame: &BlameInfo,
    ) -> ContractResult<()> {
        if !self.config.enable_checking {
            return Ok(());
        }

        let start_time = std::time::Instant::now();

        let result = (contract.checker)(value, blame);

        let check_time = start_time.elapsed();
        self.performance_monitor.record_check(check_time);

        match result {
            Ok(true) => Ok(()),
            Ok(false) => {
                let violation = self.create_violation(contract, blame, value, "Contract violated");
                self.blame_tracker.record_violation(
                    blame.clone(),
                    format!("{}", contract.original),
                    "valid value".to_string(),
                    format!("{value:?}"), // TODO: Better value formatting
                    violation.message.clone(),
                    blame.boundary.location,
                );
                self.performance_monitor.record_violation(check_time);
                Err(ContractError::Violation {
                    blame: Box::new(blame.clone()),
                    expected: "valid value".to_string(),
                    actual: format!("{value:?}"),
                    location: blame.boundary.location,
                }
                .into())
            }
            Err(e) => Err(e),
        }
    }

    /// Wraps a value with contract checking.
    pub fn wrap(
        &mut self,
        value: Value,
        contract: Arc<CompiledContract>,
        blame: BlameInfo,
    ) -> ContractResult<Value> {
        if !self.config.enable_checking {
            return Ok(value);
        }

        // First check if the value satisfies the contract immediately
        self.check(&value, &contract, &blame)?;

        // For function contracts, we need to create a wrapper
        match &value {
            Value::Procedure(_) | Value::CaseLambda(_) | Value::Primitive(_) => {
                if contract.original.is_function_contract() {
                    self.wrap_function(value, contract, blame)
                } else {
                    // Non-function contracts on functions just check the function itself
                    Ok(value)
                }
            }
            _ => {
                // For non-function values, we can check immediately
                Ok(value)
            }
        }
    }

    /// Wraps a function with contract checking.
    fn wrap_function(
        &mut self,
        function: Value,
        contract: Arc<CompiledContract>,
        blame: BlameInfo,
    ) -> ContractResult<Value> {
        let wrapped_id = self.next_wrapped_value_id();

        let metadata = WrapperMetadata {
            created_at: std::time::SystemTime::now(),
            check_count: 0,
            total_check_time: std::time::Duration::new(0, 0),
            last_check_result: None,
            wrapper_type: WrapperType::Function,
        };

        let wrapped = WrappedValue {
            value: function.clone(),
            contract: contract.clone(),
            blame: blame.clone(),
            metadata,
        };

        // Store in cache
        {
            let mut cache = self.wrapped_cache.write().unwrap();
            cache.insert(wrapped_id, wrapped);
        }

        // For now, return the original function
        // TODO: Create a proper wrapper procedure that checks arguments and return values
        Ok(function)
    }

    /// Unwraps a value if it's wrapped with a contract.
    pub fn unwrap(&self, value: &Value) -> Value {
        // TODO: Check if value is wrapped and unwrap it
        value.clone()
    }

    /// Creates a contract violation record.
    fn create_violation(
        &self,
        contract: &CompiledContract,
        blame: &BlameInfo,
        value: &Value,
        message: &str,
    ) -> ContractViolation {
        ContractViolation {
            contract_id: contract.id,
            original_contract: contract.original.clone(),
            blame: blame.clone(),
            violating_value: format!("{value:?}"), // TODO: Better formatting
            message: message.to_string(),
            timestamp: std::time::SystemTime::now(),
            location: blame.boundary.location,
        }
    }

    /// Gets the next wrapped value ID.
    fn next_wrapped_value_id(&self) -> WrappedValueId {
        let mut id = self.next_wrapped_id.lock().unwrap();
        let current = *id;
        *id += 1;
        current
    }

    /// Gets performance statistics.
    pub fn performance_stats(&self) -> PerformanceStats {
        self.performance_monitor.get_stats()
    }

    /// Gets violation statistics.
    pub fn violation_stats(&self) -> ViolationStats {
        self.performance_monitor.get_violation_stats()
    }

    /// Clears the wrapped value cache.
    pub fn clear_cache(&mut self) {
        let mut cache = self.wrapped_cache.write().unwrap();
        cache.clear();
    }

    /// Gets cache statistics.
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.wrapped_cache.read().unwrap();
        (cache.len(), cache.capacity())
    }
}

/// Record of a contract violation.
#[derive(Debug, Clone)]
pub struct ContractViolation {
    /// ID of the violated contract
    pub contract_id: u64,
    /// Original contract expression
    pub original_contract: ContractExpr,
    /// Blame information
    pub blame: BlameInfo,
    /// String representation of violating value
    pub violating_value: String,
    /// Violation message
    pub message: String,
    /// Timestamp of violation
    pub timestamp: std::time::SystemTime,
    /// Location of violation
    pub location: Span,
}

/// Performance statistics for contract checking.
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Total number of checks performed
    pub total_checks: u64,
    /// Total time spent checking contracts
    pub total_check_time: std::time::Duration,
    /// Average check time
    pub average_check_time: std::time::Duration,
    /// Minimum check time
    pub min_check_time: std::time::Duration,
    /// Maximum check time
    pub max_check_time: std::time::Duration,
    /// Check latency percentiles
    pub latency_percentiles: LatencyPercentiles,
}

/// Latency percentiles for contract checking.
#[derive(Debug, Clone)]
pub struct LatencyPercentiles {
    /// 50th percentile (median) latency
    pub p50: std::time::Duration,
    /// 90th percentile latency
    pub p90: std::time::Duration,
    /// 95th percentile latency
    pub p95: std::time::Duration,
    /// 99th percentile latency
    pub p99: std::time::Duration,
}

impl PerformanceMonitor {
    /// Creates a new performance monitor.
    pub fn new() -> Self {
        Self {
            total_checks: Arc::new(Mutex::new(0)),
            total_check_time: Arc::new(Mutex::new(std::time::Duration::new(0, 0))),
            latency_histogram: Arc::new(Mutex::new(HashMap::new())),
            violation_stats: Arc::new(Mutex::new(ViolationStats {
                total_violations: 0,
                violations_by_type: HashMap::new(),
                average_detection_time: std::time::Duration::new(0, 0),
            })),
        }
    }

    /// Records a contract check.
    pub fn record_check(&self, duration: std::time::Duration) {
        {
            let mut total_checks = self.total_checks.lock().unwrap();
            *total_checks += 1;
        }

        {
            let mut total_time = self.total_check_time.lock().unwrap();
            *total_time += duration;
        }

        {
            let mut histogram = self.latency_histogram.lock().unwrap();
            let micros = duration.as_micros() as u64;
            *histogram.entry(micros).or_insert(0) += 1;
        }
    }

    /// Records a contract violation.
    pub fn record_violation(&self, detection_time: std::time::Duration) {
        let mut stats = self.violation_stats.lock().unwrap();
        stats.total_violations += 1;

        // Update average detection time
        let total_time =
            stats.average_detection_time * (stats.total_violations - 1) as u32 + detection_time;
        stats.average_detection_time = total_time / stats.total_violations as u32;
    }

    /// Gets performance statistics.
    pub fn get_stats(&self) -> PerformanceStats {
        let total_checks = *self.total_checks.lock().unwrap();
        let total_time = *self.total_check_time.lock().unwrap();
        let histogram = self.latency_histogram.lock().unwrap();

        let average_time = if total_checks > 0 {
            total_time / total_checks as u32
        } else {
            std::time::Duration::new(0, 0)
        };

        let (min_time, max_time) = if histogram.is_empty() {
            (
                std::time::Duration::new(0, 0),
                std::time::Duration::new(0, 0),
            )
        } else {
            let min_micros = *histogram.keys().min().unwrap_or(&0);
            let max_micros = *histogram.keys().max().unwrap_or(&0);
            (
                std::time::Duration::from_micros(min_micros),
                std::time::Duration::from_micros(max_micros),
            )
        };

        let latency_percentiles = self.calculate_percentiles(&histogram);

        PerformanceStats {
            total_checks,
            total_check_time: total_time,
            average_check_time: average_time,
            min_check_time: min_time,
            max_check_time: max_time,
            latency_percentiles,
        }
    }

    /// Gets violation statistics.
    pub fn get_violation_stats(&self) -> ViolationStats {
        self.violation_stats.lock().unwrap().clone()
    }

    /// Calculates latency percentiles from histogram.
    fn calculate_percentiles(&self, histogram: &HashMap<u64, u64>) -> LatencyPercentiles {
        let mut samples: Vec<u64> = Vec::new();
        for (&latency, &count) in histogram {
            for _ in 0..count {
                samples.push(latency);
            }
        }

        if samples.is_empty() {
            return LatencyPercentiles {
                p50: std::time::Duration::new(0, 0),
                p90: std::time::Duration::new(0, 0),
                p95: std::time::Duration::new(0, 0),
                p99: std::time::Duration::new(0, 0),
            };
        }

        samples.sort_unstable();
        let len = samples.len();

        let p50_idx = len * 50 / 100;
        let p90_idx = len * 90 / 100;
        let p95_idx = len * 95 / 100;
        let p99_idx = len * 99 / 100;

        LatencyPercentiles {
            p50: std::time::Duration::from_micros(samples[p50_idx.min(len - 1)]),
            p90: std::time::Duration::from_micros(samples[p90_idx.min(len - 1)]),
            p95: std::time::Duration::from_micros(samples[p95_idx.min(len - 1)]),
            p99: std::time::Duration::from_micros(samples[p99_idx.min(len - 1)]),
        }
    }
}

impl WrapperMetadata {
    /// Creates new wrapper metadata.
    pub fn new(wrapper_type: WrapperType) -> Self {
        Self {
            created_at: std::time::SystemTime::now(),
            check_count: 0,
            total_check_time: std::time::Duration::new(0, 0),
            last_check_result: None,
            wrapper_type,
        }
    }

    /// Records a check operation.
    pub fn record_check(&mut self, duration: std::time::Duration, result: bool) {
        self.check_count += 1;
        self.total_check_time += duration;
        self.last_check_result = Some(result);
    }

    /// Gets the average check time.
    pub fn average_check_time(&self) -> std::time::Duration {
        if self.check_count > 0 {
            self.total_check_time / self.check_count as u32
        } else {
            std::time::Duration::new(0, 0)
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ContractViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Contract violation: {} (blame: {}) at {}:{}",
            self.message,
            self.blame,
            self.location.start,
            self.location.end()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::contracts::{
        ast::ContractExpr,
        blame::{BlameBoundary, BlameInfo, BlameTarget, BlameTracker, BoundaryType},
        compiler::{CompilationContext, CompiledContract, ContractCompiler, OptimizationLevel},
        predicates::PredicateRegistry,
    };
    use crate::diagnostics::Span;
    use crate::eval::Value;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_test_enforcement() -> ContractEnforcement {
        let config = ContractConfig::default();
        let blame_tracker = BlameTracker::new();
        ContractEnforcement::new(&config, blame_tracker)
    }

    fn create_test_blame() -> BlameInfo {
        BlameInfo {
            positive: BlameTarget::System {
                component: "test".to_string(),
                description: "test context".to_string(),
            },
            negative: BlameTarget::System {
                component: "test".to_string(),
                description: "test context".to_string(),
            },
            boundary: BlameBoundary {
                boundary_type: BoundaryType::ExplicitContract,
                contract: "test".to_string(),
                location: Span::new(0, 0),
                context: HashMap::new(),
            },
            call_stack: Vec::new(),
            id: 1,
            parent: None,
        }
    }

    fn create_test_contract() -> Arc<CompiledContract> {
        Arc::new(CompiledContract {
            id: 1,
            original: ContractExpr::Predicate {
                name: "number?".to_string(),
                location: Span::new(0, 0),
            },
            checker: Arc::new(|value, _blame| {
                Ok(matches!(value, Value::Literal(Literal::Number(_))))
            }),
            optimization_level: OptimizationLevel::Standard,
            performance: crate::contracts::compiler::PerformanceInfo {
                time_complexity: crate::contracts::compiler::PerformanceComplexity::Constant,
                space_complexity: crate::contracts::compiler::PerformanceComplexity::Constant,
                deterministic: true,
                operation_count: 1,
                inlinable: true,
            },
            dependencies: std::collections::HashSet::new(),
            metadata: crate::contracts::compiler::CompilationMetadata {
                timestamp: std::time::SystemTime::now(),
                source_location: Span::new(0, 0),
                optimizations: Vec::new(),
                warnings: Vec::new(),
                size_info: crate::contracts::compiler::SizeInfo {
                    original_nodes: 1,
                    compiled_operations: 1,
                    estimated_memory: 64,
                },
            },
            predicate: Box::new(|value| matches!(value, Value::Literal(Literal::Number(_)))),
            blame_info: create_test_blame(),
            contract_name: "number?".to_string(),
        })
    }

    #[test]
    fn test_contract_enforcement_creation() {
        let enforcement = create_test_enforcement();
        assert!(enforcement.config.enable_checking);

        let (cache_size, _) = enforcement.cache_stats();
        assert_eq!(cache_size, 0);
    }

    #[test]
    fn test_successful_contract_check() {
        let mut enforcement = create_test_enforcement();
        let contract = create_test_contract();
        let blame = create_test_blame();

        let number_value = Value::Literal(Literal::Number(42.0));

        let result = enforcement.check(&number_value, &contract, &blame);
        assert!(result.is_ok());
    }

    #[test]
    fn test_failed_contract_check() {
        let mut enforcement = create_test_enforcement();
        let contract = create_test_contract();
        let blame = create_test_blame();

        let string_value = Value::Literal(Literal::String(Box::new("hello".to_string())));

        let result = enforcement.check(&string_value, &contract, &blame);
        assert!(result.is_err());

        match result {
            Err(ref err) => {
                // Expected violation - check if it's a contract error
                assert!(
                    err.to_string().contains("violation") || err.to_string().contains("contract")
                );
            }
            _ => panic!("Expected contract violation"),
        }
    }

    #[test]
    fn test_value_wrapping() {
        let mut enforcement = create_test_enforcement();
        let contract = create_test_contract();
        let blame = create_test_blame();

        let number_value = Value::Literal(Literal::Number(42.0));

        let result = enforcement.wrap(number_value.clone(), contract, blame);
        assert!(result.is_ok());

        // For non-function values, wrapping should return the original value
        match result {
            Ok(wrapped_value) => {
                assert_eq!(
                    format!("{:?}", wrapped_value),
                    format!("{:?}", number_value)
                );
            }
            Err(_) => panic!("Wrapping should succeed"),
        }
    }

    #[test]
    fn test_performance_monitoring() {
        let monitor = PerformanceMonitor::new();

        // Record some checks
        monitor.record_check(std::time::Duration::from_micros(100));
        monitor.record_check(std::time::Duration::from_micros(200));
        monitor.record_check(std::time::Duration::from_micros(150));

        let stats = monitor.get_stats();
        assert_eq!(stats.total_checks, 3);
        assert_eq!(stats.min_check_time, std::time::Duration::from_micros(100));
        assert_eq!(stats.max_check_time, std::time::Duration::from_micros(200));

        // Average should be 150 microseconds
        let expected_avg = std::time::Duration::from_micros(450) / 3;
        assert_eq!(stats.average_check_time, expected_avg);
    }

    #[test]
    fn test_violation_recording() {
        let monitor = PerformanceMonitor::new();

        monitor.record_violation(std::time::Duration::from_micros(500));
        monitor.record_violation(std::time::Duration::from_micros(300));

        let stats = monitor.get_violation_stats();
        assert_eq!(stats.total_violations, 2);

        // Average detection time should be 400 microseconds
        assert_eq!(
            stats.average_detection_time,
            std::time::Duration::from_micros(400)
        );
    }

    #[test]
    fn test_wrapper_metadata() {
        let mut metadata = WrapperMetadata::new(WrapperType::Value);

        assert_eq!(metadata.check_count, 0);
        assert_eq!(metadata.wrapper_type, WrapperType::Value);

        metadata.record_check(std::time::Duration::from_micros(100), true);
        metadata.record_check(std::time::Duration::from_micros(200), false);

        assert_eq!(metadata.check_count, 2);
        assert_eq!(metadata.last_check_result, Some(false));
        assert_eq!(
            metadata.average_check_time(),
            std::time::Duration::from_micros(150)
        );
    }

    #[test]
    fn test_config_updates() {
        let mut enforcement = create_test_enforcement();

        let mut new_config = ContractConfig::default();
        new_config.enable_checking = false;

        enforcement.update_config(&new_config);
        assert!(!enforcement.config.enable_checking);

        // When checking is disabled, checks should always succeed
        let contract = create_test_contract();
        let blame = create_test_blame();
        let string_value = Value::Literal(Literal::String(Box::new("hello".to_string())));

        let result = enforcement.check(&string_value, &contract, &blame);
        assert!(result.is_ok()); // Should succeed because checking is disabled
    }

    #[test]
    fn test_cache_operations() {
        let mut enforcement = create_test_enforcement();

        let (initial_size, _) = enforcement.cache_stats();
        assert_eq!(initial_size, 0);

        enforcement.clear_cache();
        let (size_after_clear, _) = enforcement.cache_stats();
        assert_eq!(size_after_clear, 0);
    }
}

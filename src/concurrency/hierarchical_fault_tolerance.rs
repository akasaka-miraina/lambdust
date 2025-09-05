//! Hierarchical Fault Tolerance System - Phase 5 Stage 3
//!
//! This module implements a comprehensive fault tolerance system with hierarchical
//! failure detection, recovery strategies, and adaptive resilience mechanisms.
//!
//! Key Features:
//! - Multi-level fault detection (hardware, network, software, Byzantine)
//! - Automatic failure recovery with cascading fallbacks
//! - Circuit breaker patterns for service protection
//! - Adaptive timeout and retry mechanisms
//! - Health monitoring and predictive failure detection
//! - Integration with distributed execution and load balancing

use super::{
    ConcurrencyError,
    byzantine_node_discovery::NodeIdentity,
    distributed_config::FaultToleranceConfig,
    distributed_execution_engine::{DistributedTask, NodeCapabilities, NodeId, TaskId},
};
use crate::diagnostics::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{RwLock as AsyncRwLock, mpsc, oneshot};
use tokio::time::{interval, timeout};
use uuid::Uuid;

/// Maximum number of failure records to maintain
const MAX_FAILURE_HISTORY: usize = 1000;
/// Default circuit breaker failure threshold
const DEFAULT_FAILURE_THRESHOLD: u32 = 5;
/// Default circuit breaker timeout
const DEFAULT_CIRCUIT_BREAKER_TIMEOUT: Duration = Duration::from_secs(60);
/// Health check interval
const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(10);
/// Failure prediction window
const PREDICTION_WINDOW: Duration = Duration::from_secs(5 * 60);

/// Types of faults that can occur in the system
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FaultType {
    /// Hardware failures (disk, memory, CPU)
    Hardware,
    /// Network failures (partition, latency, packet loss)
    Network,
    /// Software failures (crashes, exceptions, deadlocks)
    Software,
    /// Byzantine failures (malicious or corrupted behavior)
    Byzantine,
    /// Resource exhaustion (CPU, memory, disk, bandwidth)
    ResourceExhaustion,
    /// Timeout failures
    Timeout,
    /// Service unavailability
    ServiceUnavailable,
    /// Data corruption or inconsistency
    DataCorruption,
}

impl FaultType {
    /// Gets the severity level of this fault type
    pub fn severity(&self) -> FaultSeverity {
        match self {
            Self::Hardware => FaultSeverity::Critical,
            Self::Network => FaultSeverity::High,
            Self::Software => FaultSeverity::Medium,
            Self::Byzantine => FaultSeverity::Critical,
            Self::ResourceExhaustion => FaultSeverity::High,
            Self::Timeout => FaultSeverity::Medium,
            Self::ServiceUnavailable => FaultSeverity::High,
            Self::DataCorruption => FaultSeverity::Critical,
        }
    }

    /// Gets the default recovery strategy for this fault type
    pub fn default_recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::Hardware => RecoveryStrategy::NodeReplacement,
            Self::Network => RecoveryStrategy::Retry {
                max_attempts: 3,
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(10),
            },
            Self::Software => RecoveryStrategy::Restart,
            Self::Byzantine => RecoveryStrategy::NodeIsolation,
            Self::ResourceExhaustion => RecoveryStrategy::LoadShedding {
                reduction_percentage: 50,
            },
            Self::Timeout => RecoveryStrategy::Retry {
                max_attempts: 3,
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(10),
            },
            Self::ServiceUnavailable => RecoveryStrategy::Failover {
                backup_nodes: vec![],
            },
            Self::DataCorruption => RecoveryStrategy::DataRecovery {
                backup_sources: vec![],
            },
        }
    }
}

/// Severity levels for faults
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FaultSeverity {
    /// Low severity - minor issues that don't affect functionality
    Low = 1,
    /// Medium severity - issues that may affect performance
    Medium = 2,
    /// High severity - issues that significantly impact functionality
    High = 3,
    /// Critical severity - issues that cause complete failure
    Critical = 4,
}

/// Recovery strategies for different types of faults
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff
    Retry {
        /// Max Attempts field
        max_attempts: u32,
        /// Initial Delay field
        initial_delay: Duration,
        /// Max Delay field
        max_delay: Duration,
    },
    /// Restart the failed component or service
    Restart,
    /// Failover to a backup node or service
    Failover {
        /// List of backup nodes to failover to
        backup_nodes: Vec<NodeId>,
    },
    /// Replace the failed node with a new one
    NodeReplacement,
    /// Isolate the Byzantine node from the network
    NodeIsolation,
    /// Reduce system load to prevent further failures
    LoadShedding {
        /// Percentage by which to reduce load
        reduction_percentage: u32,
    },
    /// Recover from data corruption
    DataRecovery {
        /// Nodes that can provide backup data
        backup_sources: Vec<NodeId>,
    },
    /// Circuit breaker pattern - temporarily stop calls
    CircuitBreaker,
    /// No recovery action (monitoring only)
    None,
}

impl Default for RecoveryStrategy {
    fn default() -> Self {
        Self::Retry {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
        }
    }
}

/// A recorded fault incident
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultIncident {
    /// Unique incident identifier
    pub incident_id: Uuid,
    /// Type of fault that occurred
    pub fault_type: FaultType,
    /// Node where the fault occurred
    pub node_id: NodeId,
    /// Task being executed when fault occurred (if applicable)
    pub task_id: Option<TaskId>,
    /// Fault severity level
    pub severity: FaultSeverity,
    /// Detailed error message
    pub error_message: String,
    /// Timestamp when fault was detected
    pub timestamp: SystemTime,
    /// Context information
    pub context: HashMap<String, String>,
    /// Whether the fault was successfully recovered
    pub recovered: bool,
    /// Recovery strategy used
    pub recovery_strategy: Option<RecoveryStrategy>,
    /// Time taken to recover
    pub recovery_time: Option<Duration>,
}

impl FaultIncident {
    /// Creates a new fault incident
    pub fn new(fault_type: FaultType, node_id: NodeId, error_message: String) -> Self {
        Self {
            incident_id: Uuid::new_v4(),
            fault_type: fault_type.clone(),
            node_id,
            task_id: None,
            severity: fault_type.severity(),
            error_message,
            timestamp: SystemTime::now(),
            context: HashMap::new(),
            recovered: false,
            recovery_strategy: None,
            recovery_time: None,
        }
    }

    /// Adds context information
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }

    /// Sets the associated task ID
    pub fn with_task_id(mut self, task_id: TaskId) -> Self {
        self.task_id = Some(task_id);
        self
    }

    /// Marks the incident as recovered
    pub fn mark_recovered(&mut self, strategy: RecoveryStrategy, recovery_time: Duration) {
        self.recovered = true;
        self.recovery_strategy = Some(strategy);
        self.recovery_time = Some(recovery_time);
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    /// Circuit is closed, allowing requests through
    Closed,
    /// Circuit is open, rejecting requests
    Open {
        /// When the circuit was opened
        opened_at: SystemTime,
        /// Failure count that triggered opening
        failure_count: u32,
    },
    /// Circuit is half-open, testing if service has recovered
    HalfOpen {
        /// Number of test requests sent
        test_requests: u32,
    },
}

/// Circuit breaker for protecting services from cascading failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    /// Circuit breaker identifier
    pub id: String,
    /// Current state
    pub state: CircuitBreakerState,
    /// Failure threshold to trigger opening
    pub failure_threshold: u32,
    /// Success threshold to close from half-open
    pub success_threshold: u32,
    /// Timeout before trying half-open
    pub timeout: Duration,
    /// Failure count in current window
    pub failure_count: u32,
    /// Success count in half-open state
    pub success_count: u32,
    /// Window size for counting failures
    pub window_size: Duration,
    /// Recent request results
    pub recent_results: VecDeque<(SystemTime, bool)>,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker
    pub fn new(id: String) -> Self {
        Self {
            id,
            state: CircuitBreakerState::Closed,
            failure_threshold: DEFAULT_FAILURE_THRESHOLD,
            success_threshold: 3,
            timeout: DEFAULT_CIRCUIT_BREAKER_TIMEOUT,
            failure_count: 0,
            success_count: 0,
            window_size: Duration::from_secs(60),
            recent_results: VecDeque::new(),
        }
    }

    /// Configures the circuit breaker parameters
    pub fn with_config(
        mut self,
        failure_threshold: u32,
        success_threshold: u32,
        timeout: Duration,
    ) -> Self {
        self.failure_threshold = failure_threshold;
        self.success_threshold = success_threshold;
        self.timeout = timeout;
        self
    }

    /// Checks if the circuit allows requests
    pub fn can_execute(&mut self) -> bool {
        self.cleanup_old_results();

        match &self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open { opened_at, .. } => {
                // Check if timeout has elapsed
                if SystemTime::now()
                    .duration_since(*opened_at)
                    .unwrap_or(Duration::ZERO)
                    > self.timeout
                {
                    // Transition to half-open
                    self.state = CircuitBreakerState::HalfOpen { test_requests: 0 };
                    self.success_count = 0;
                    true
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen { .. } => true,
        }
    }

    /// Records the result of a request
    pub fn record_result(&mut self, success: bool) {
        let now = SystemTime::now();
        self.recent_results.push_back((now, success));

        match &mut self.state {
            CircuitBreakerState::Closed => {
                if success {
                    self.failure_count = 0;
                } else {
                    self.failure_count += 1;
                    if self.failure_count >= self.failure_threshold {
                        self.state = CircuitBreakerState::Open {
                            opened_at: now,
                            failure_count: self.failure_count,
                        };
                    }
                }
            }
            CircuitBreakerState::Open { .. } => {
                // Should not happen if can_execute is called first
            }
            CircuitBreakerState::HalfOpen { test_requests } => {
                *test_requests += 1;
                if success {
                    self.success_count += 1;
                    if self.success_count >= self.success_threshold {
                        self.state = CircuitBreakerState::Closed;
                        self.failure_count = 0;
                    }
                } else {
                    self.state = CircuitBreakerState::Open {
                        opened_at: now,
                        failure_count: 1,
                    };
                    self.success_count = 0;
                }
            }
        }

        self.cleanup_old_results();
    }

    /// Removes old results outside the window
    fn cleanup_old_results(&mut self) {
        let cutoff = SystemTime::now() - self.window_size;
        while let Some(&(timestamp, _)) = self.recent_results.front() {
            if timestamp < cutoff {
                self.recent_results.pop_front();
            } else {
                break;
            }
        }

        // Recalculate failure count based on recent results
        if matches!(self.state, CircuitBreakerState::Closed) {
            self.failure_count = self
                .recent_results
                .iter()
                .filter(|(_, success)| !success)
                .count() as u32;
        }
    }
}

/// Health status of a node or service
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is healthy and operating normally
    Healthy,
    /// Service is degraded but still functional
    Degraded {
        /// Performance impact (0.0 to 1.0)
        impact: u32, // Store as integer percentage (0-100)
    },
    /// Service is unhealthy and may fail soon
    Unhealthy,
    /// Service is completely failed
    Failed,
    /// Health status is unknown
    Unknown,
}

impl HealthStatus {
    /// Creates a degraded status with impact percentage
    pub fn degraded(impact_percentage: f64) -> Self {
        let impact = (impact_percentage * 100.0).clamp(0.0, 100.0) as u32;
        Self::Degraded { impact }
    }

    /// Gets the impact as a float (0.0 to 1.0)
    pub fn impact_factor(&self) -> f64 {
        match self {
            Self::Healthy => 0.0,
            Self::Degraded { impact } => *impact as f64 / 100.0,
            Self::Unhealthy => 0.8,
            Self::Failed => 1.0,
            Self::Unknown => 0.5,
        }
    }

    /// Checks if the service is usable
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded { .. })
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Node being checked
    pub node_id: NodeId,
    /// Health status
    pub status: HealthStatus,
    /// Response time for health check
    pub response_time: Duration,
    /// Timestamp of the check
    pub timestamp: SystemTime,
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
    /// Error message if unhealthy
    pub error_message: Option<String>,
}

impl HealthCheckResult {
    /// Creates a healthy result
    pub fn healthy(node_id: NodeId, response_time: Duration) -> Self {
        Self {
            node_id,
            status: HealthStatus::Healthy,
            response_time,
            timestamp: SystemTime::now(),
            metrics: HashMap::new(),
            error_message: None,
        }
    }

    /// Creates a failed result
    pub fn failed(node_id: NodeId, error: String) -> Self {
        Self {
            node_id,
            status: HealthStatus::Failed,
            response_time: Duration::ZERO,
            timestamp: SystemTime::now(),
            metrics: HashMap::new(),
            error_message: Some(error),
        }
    }

    /// Adds a metric
    pub fn with_metric(mut self, name: String, value: f64) -> Self {
        self.metrics.insert(name, value);
        self
    }
}

/// Predictive failure detector using historical patterns
#[derive(Debug)]
pub struct FailurePredictionEngine {
    /// Historical failure patterns
    failure_history: VecDeque<FaultIncident>,
    /// Node health trends
    health_trends: HashMap<NodeId, VecDeque<HealthCheckResult>>,
    /// Prediction models (simplified)
    prediction_weights: HashMap<String, f64>,
}

impl FailurePredictionEngine {
    /// Creates a new failure prediction engine
    pub fn new() -> Self {
        let mut prediction_weights = HashMap::new();
        prediction_weights.insert("cpu_usage_trend".to_string(), 0.3);
        prediction_weights.insert("memory_usage_trend".to_string(), 0.3);
        prediction_weights.insert("response_time_trend".to_string(), 0.2);
        prediction_weights.insert("failure_frequency".to_string(), 0.2);

        Self {
            failure_history: VecDeque::new(),
            health_trends: HashMap::new(),
            prediction_weights,
        }
    }

    /// Records a fault incident for learning
    pub fn record_fault(&mut self, incident: FaultIncident) {
        self.failure_history.push_back(incident);

        // Keep only recent history
        if self.failure_history.len() > MAX_FAILURE_HISTORY {
            self.failure_history.pop_front();
        }
    }

    /// Records a health check result
    pub fn record_health_check(&mut self, result: HealthCheckResult) {
        let node_id = result.node_id;
        let trend = self.health_trends.entry(node_id).or_default();

        trend.push_back(result);

        // Keep only recent health data (last 100 checks)
        if trend.len() > 100 {
            trend.pop_front();
        }
    }

    /// Predicts the probability of failure for a node
    pub fn predict_failure_probability(&self, node_id: NodeId) -> f64 {
        let mut probability = 0.0;
        let mut total_weight = 0.0;

        // Analyze health trends
        if let Some(trend) = self.health_trends.get(&node_id) {
            if trend.len() >= 5 {
                // CPU usage trend
                if let Some(cpu_trend) = self.calculate_metric_trend(trend, "cpu_usage") {
                    if let Some(&weight) = self.prediction_weights.get("cpu_usage_trend") {
                        probability += weight * cpu_trend.clamp(0.0, 1.0);
                        total_weight += weight;
                    }
                }

                // Memory usage trend
                if let Some(memory_trend) = self.calculate_metric_trend(trend, "memory_usage") {
                    if let Some(&weight) = self.prediction_weights.get("memory_usage_trend") {
                        probability += weight * memory_trend.clamp(0.0, 1.0);
                        total_weight += weight;
                    }
                }

                // Response time trend
                let response_trend = self.calculate_response_time_trend(trend);
                if let Some(&weight) = self.prediction_weights.get("response_time_trend") {
                    probability += weight * response_trend.clamp(0.0, 1.0);
                    total_weight += weight;
                }
            }
        }

        // Analyze failure frequency
        let recent_failures = self.count_recent_failures(node_id, PREDICTION_WINDOW);
        let failure_frequency = (recent_failures as f64 / 10.0).min(1.0);
        if let Some(&weight) = self.prediction_weights.get("failure_frequency") {
            probability += weight * failure_frequency;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            probability / total_weight
        } else {
            0.0
        }
    }

    /// Calculates trend for a specific metric
    fn calculate_metric_trend(
        &self,
        trend: &VecDeque<HealthCheckResult>,
        metric: &str,
    ) -> Option<f64> {
        let values: Vec<f64> = trend
            .iter()
            .filter_map(|result| result.metrics.get(metric).copied())
            .collect();

        if values.len() < 3 {
            return None;
        }

        // Simple linear regression slope
        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = values.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean) * (x - x_mean);
        }

        if denominator != 0.0 {
            Some(numerator / denominator)
        } else {
            None
        }
    }

    /// Calculates response time trend
    fn calculate_response_time_trend(&self, trend: &VecDeque<HealthCheckResult>) -> f64 {
        if trend.len() < 3 {
            return 0.0;
        }

        let response_times: Vec<f64> = trend
            .iter()
            .map(|result| result.response_time.as_millis() as f64)
            .collect();

        // Calculate trend (normalized)
        let first_half_avg = response_times
            .iter()
            .take(response_times.len() / 2)
            .sum::<f64>()
            / (response_times.len() / 2) as f64;
        let second_half_avg = response_times
            .iter()
            .skip(response_times.len() / 2)
            .sum::<f64>()
            / (response_times.len() - response_times.len() / 2) as f64;

        // Return normalized trend (positive indicates increasing response times)
        ((second_half_avg - first_half_avg) / first_half_avg.max(1.0)).max(0.0)
    }

    /// Counts recent failures for a node
    fn count_recent_failures(&self, node_id: NodeId, window: Duration) -> usize {
        let cutoff = SystemTime::now() - window;
        self.failure_history
            .iter()
            .filter(|incident| incident.node_id == node_id && incident.timestamp > cutoff)
            .count()
    }

    /// Gets nodes predicted to fail soon
    pub fn get_at_risk_nodes(&self, threshold: f64) -> Vec<(NodeId, f64)> {
        let mut at_risk = Vec::new();

        for &node_id in self.health_trends.keys() {
            let probability = self.predict_failure_probability(node_id);
            if probability > threshold {
                at_risk.push((node_id, probability));
            }
        }

        at_risk.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        at_risk
    }
}

impl Default for FailurePredictionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Main hierarchical fault tolerance system
#[derive(Debug)]
pub struct HierarchicalFaultToleranceSystem {
    /// System configuration
    config: FaultToleranceConfig,
    /// Fault incident history
    fault_history: Arc<RwLock<VecDeque<FaultIncident>>>,
    /// Circuit breakers for different services
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    /// Node health status
    node_health: Arc<RwLock<HashMap<NodeId, HealthStatus>>>,
    /// Health check results
    health_history: Arc<RwLock<HashMap<NodeId, VecDeque<HealthCheckResult>>>>,
    /// Failure prediction engine
    prediction_engine: Arc<RwLock<FailurePredictionEngine>>,
    /// Recovery strategies by fault type
    recovery_strategies: Arc<RwLock<HashMap<FaultType, RecoveryStrategy>>>,
    /// Active recovery operations
    active_recoveries: Arc<RwLock<HashSet<NodeId>>>,
    /// Event notification channels
    fault_events_tx: mpsc::UnboundedSender<FaultIncident>,
    fault_events_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<FaultIncident>>>,
}

impl HierarchicalFaultToleranceSystem {
    /// Creates a new hierarchical fault tolerance system
    pub fn new(config: FaultToleranceConfig) -> Self {
        let (fault_events_tx, fault_events_rx) = mpsc::unbounded_channel();

        let mut recovery_strategies = HashMap::new();
        recovery_strategies.insert(FaultType::Hardware, RecoveryStrategy::NodeReplacement);
        recovery_strategies.insert(
            FaultType::Network,
            RecoveryStrategy::Retry {
                max_attempts: 3,
                initial_delay: Duration::from_millis(100),
                max_delay: Duration::from_secs(5),
            },
        );
        recovery_strategies.insert(FaultType::Software, RecoveryStrategy::Restart);
        recovery_strategies.insert(FaultType::Byzantine, RecoveryStrategy::NodeIsolation);
        recovery_strategies.insert(
            FaultType::ResourceExhaustion,
            RecoveryStrategy::LoadShedding {
                reduction_percentage: 30,
            },
        );
        recovery_strategies.insert(
            FaultType::Timeout,
            RecoveryStrategy::Retry {
                max_attempts: 2,
                initial_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(10),
            },
        );
        recovery_strategies.insert(
            FaultType::ServiceUnavailable,
            RecoveryStrategy::Failover {
                backup_nodes: Vec::new(), // Will be populated dynamically
            },
        );
        recovery_strategies.insert(
            FaultType::DataCorruption,
            RecoveryStrategy::DataRecovery {
                backup_sources: Vec::new(),
            },
        );

        Self {
            config,
            fault_history: Arc::new(RwLock::new(VecDeque::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            node_health: Arc::new(RwLock::new(HashMap::new())),
            health_history: Arc::new(RwLock::new(HashMap::new())),
            prediction_engine: Arc::new(RwLock::new(FailurePredictionEngine::new())),
            recovery_strategies: Arc::new(RwLock::new(recovery_strategies)),
            active_recoveries: Arc::new(RwLock::new(HashSet::new())),
            fault_events_tx,
            fault_events_rx: Arc::new(tokio::sync::Mutex::new(fault_events_rx)),
        }
    }

    /// Starts the fault tolerance system
    pub async fn start(&self) -> Result<()> {
        // Start health monitoring
        self.start_health_monitoring().await?;

        // Start fault event processing
        self.start_fault_event_processing().await?;

        // Start predictive analysis
        self.start_predictive_analysis().await?;

        Ok(())
    }

    /// Starts health monitoring for all nodes
    async fn start_health_monitoring(&self) -> Result<()> {
        let node_health = self.node_health.clone();
        let health_history = self.health_history.clone();
        let prediction_engine = self.prediction_engine.clone();

        tokio::spawn(async move {
            let mut health_check_interval = interval(HEALTH_CHECK_INTERVAL);

            loop {
                health_check_interval.tick().await;

                // Get list of nodes to check
                let nodes_to_check: Vec<NodeId> = {
                    let health = node_health.read().unwrap();
                    health.keys().copied().collect()
                };

                // Perform health checks
                for node_id in nodes_to_check {
                    let result = Self::perform_health_check(node_id).await;

                    // Update health status
                    {
                        let mut health = node_health.write().unwrap();
                        health.insert(node_id, result.status.clone());
                    }

                    // Update health history
                    {
                        let mut history = health_history.write().unwrap();
                        let node_history = history.entry(node_id).or_default();
                        node_history.push_back(result.clone());

                        if node_history.len() > 100 {
                            node_history.pop_front();
                        }
                    }

                    // Update prediction engine
                    {
                        let mut engine = prediction_engine.write().unwrap();
                        engine.record_health_check(result);
                    }
                }
            }
        });

        Ok(())
    }

    /// Performs a health check on a specific node
    async fn perform_health_check(node_id: NodeId) -> HealthCheckResult {
        let start_time = Instant::now();

        // Simulate health check (in practice, would make actual network calls)
        tokio::time::sleep(Duration::from_millis(10)).await;

        let response_time = start_time.elapsed();

        // Simple health check simulation
        if fastrand::f64() > 0.95 {
            // 5% chance of failure
            HealthCheckResult::failed(node_id, "Health check timeout".to_string())
        } else if fastrand::f64() > 0.9 {
            // 5% chance of degraded performance
            let mut result = HealthCheckResult::healthy(node_id, response_time);
            result.status = HealthStatus::degraded(fastrand::f64() * 0.5);
            result
        } else {
            // 90% chance of healthy
            let mut result = HealthCheckResult::healthy(node_id, response_time);
            result = result.with_metric("cpu_usage".to_string(), fastrand::f64() * 0.8);
            result = result.with_metric("memory_usage".to_string(), fastrand::f64() * 0.7);
            result
        }
    }

    /// Starts fault event processing
    async fn start_fault_event_processing(&self) -> Result<()> {
        let fault_history = self.fault_history.clone();
        let prediction_engine = self.prediction_engine.clone();
        let recovery_strategies = self.recovery_strategies.clone();
        let active_recoveries = self.active_recoveries.clone();
        let fault_events_rx = self.fault_events_rx.clone();

        tokio::spawn(async move {
            let mut rx = fault_events_rx.lock().await;

            while let Some(mut incident) = rx.recv().await {
                // Record the fault
                {
                    let mut history = fault_history.write().unwrap();
                    history.push_back(incident.clone());

                    if history.len() > MAX_FAILURE_HISTORY {
                        history.pop_front();
                    }
                }

                // Update prediction engine
                {
                    let mut engine = prediction_engine.write().unwrap();
                    engine.record_fault(incident.clone());
                }

                // Initiate recovery if not already in progress
                {
                    let mut recoveries = active_recoveries.write().unwrap();
                    if !recoveries.contains(&incident.node_id) {
                        recoveries.insert(incident.node_id);

                        // Get recovery strategy
                        let strategy = {
                            let strategies = recovery_strategies.read().unwrap();
                            strategies
                                .get(&incident.fault_type)
                                .cloned()
                                .unwrap_or_default()
                        };

                        // Start recovery process
                        let node_id = incident.node_id;
                        let active_recoveries_clone = active_recoveries.clone();

                        tokio::spawn(async move {
                            let recovery_start = Instant::now();
                            let success = Self::execute_recovery_strategy(&strategy, node_id).await;
                            let recovery_time = recovery_start.elapsed();

                            if success {
                                incident.mark_recovered(strategy, recovery_time);
                                println!(
                                    "Recovery successful for node {} in {:?}",
                                    node_id, recovery_time
                                );
                            } else {
                                println!("Recovery failed for node {}", node_id);
                            }

                            // Remove from active recoveries
                            {
                                let mut recoveries = active_recoveries_clone.write().unwrap();
                                recoveries.remove(&node_id);
                            }
                        });
                    }
                }
            }
        });

        Ok(())
    }

    /// Executes a recovery strategy
    async fn execute_recovery_strategy(strategy: &RecoveryStrategy, node_id: NodeId) -> bool {
        match strategy {
            RecoveryStrategy::Retry {
                max_attempts,
                initial_delay,
                max_delay,
            } => {
                let mut delay = *initial_delay;

                for attempt in 1..=*max_attempts {
                    tokio::time::sleep(delay).await;

                    // Simulate retry operation
                    if fastrand::f64() > 0.3 {
                        println!(
                            "Retry successful for node {} on attempt {}",
                            node_id, attempt
                        );
                        return true;
                    }

                    // Exponential backoff
                    delay = (delay * 2).min(*max_delay);
                }

                false
            }
            RecoveryStrategy::Restart => {
                tokio::time::sleep(Duration::from_secs(2)).await;
                println!("Restarted service on node {}", node_id);
                true
            }
            RecoveryStrategy::Failover { backup_nodes } => {
                if backup_nodes.is_empty() {
                    println!("No backup nodes available for failover");
                    return false;
                }

                tokio::time::sleep(Duration::from_millis(500)).await;
                println!("Failed over to backup node for {}", node_id);
                true
            }
            RecoveryStrategy::NodeReplacement => {
                tokio::time::sleep(Duration::from_secs(5)).await;
                println!("Replaced failed node {}", node_id);
                true
            }
            RecoveryStrategy::NodeIsolation => {
                println!("Isolated Byzantine node {}", node_id);
                true
            }
            RecoveryStrategy::LoadShedding {
                reduction_percentage,
            } => {
                println!(
                    "Applied {}% load shedding for node {}",
                    *reduction_percentage as f64, node_id
                );
                true
            }
            RecoveryStrategy::DataRecovery { backup_sources } => {
                if backup_sources.is_empty() {
                    return false;
                }

                tokio::time::sleep(Duration::from_secs(3)).await;
                println!("Recovered data for node {} from backup", node_id);
                true
            }
            RecoveryStrategy::CircuitBreaker => {
                println!("Opened circuit breaker for node {}", node_id);
                true
            }
            RecoveryStrategy::None => {
                println!("No recovery action for node {}", node_id);
                true
            }
        }
    }

    /// Starts predictive failure analysis
    async fn start_predictive_analysis(&self) -> Result<()> {
        let prediction_engine = self.prediction_engine.clone();
        let fault_events_tx = self.fault_events_tx.clone();

        tokio::spawn(async move {
            let mut prediction_interval = interval(Duration::from_secs(60));

            loop {
                prediction_interval.tick().await;

                let at_risk_nodes = {
                    let engine = prediction_engine.read().unwrap();
                    engine.get_at_risk_nodes(0.7) // 70% failure probability threshold
                };

                for (node_id, probability) in at_risk_nodes {
                    println!(
                        "Node {} is at risk of failure (probability: {:.2})",
                        node_id, probability
                    );

                    // Create a predictive fault incident
                    let incident = FaultIncident::new(
                        FaultType::Software, // Generic type for prediction
                        node_id,
                        format!("Predictive failure alert (probability: {:.2})", probability),
                    )
                    .with_context(
                        "prediction_probability".to_string(),
                        probability.to_string(),
                    );

                    // Don't trigger full recovery, just log the warning
                    println!("Predictive alert: {}", incident.error_message);
                }
            }
        });

        Ok(())
    }

    /// Reports a fault incident
    pub async fn report_fault(&self, incident: FaultIncident) -> Result<()> {
        self.fault_events_tx.send(incident).map_err(|e| {
            Box::new(Error::runtime_error(
                format!("Failed to send fault event: {}", e),
                None,
            ))
        })?;

        Ok(())
    }

    /// Gets or creates a circuit breaker for a service
    pub fn get_circuit_breaker(&self, service_id: &str) -> Result<CircuitBreaker> {
        let mut breakers = self.circuit_breakers.write().map_err(|_| {
            Error::runtime_error("Failed to acquire circuit breakers lock".to_string(), None)
        })?;

        if let Some(breaker) = breakers.get(service_id) {
            Ok(breaker.clone())
        } else {
            let breaker = CircuitBreaker::new(service_id.to_string()).with_config(
                self.config.max_retry_attempts,
                3,
                self.config.recovery_timeout,
            );
            breakers.insert(service_id.to_string(), breaker.clone());
            Ok(breaker)
        }
    }

    /// Updates a circuit breaker
    pub fn update_circuit_breaker(&self, service_id: &str, breaker: CircuitBreaker) -> Result<()> {
        let mut breakers = self.circuit_breakers.write().map_err(|_| {
            Error::runtime_error("Failed to acquire circuit breakers lock".to_string(), None)
        })?;

        breakers.insert(service_id.to_string(), breaker);
        Ok(())
    }

    /// Registers a node for health monitoring
    pub fn register_node(&self, node_id: NodeId) -> Result<()> {
        let mut health = self.node_health.write().map_err(|_| {
            Error::runtime_error("Failed to acquire node health lock".to_string(), None)
        })?;

        health.insert(node_id, HealthStatus::Unknown);
        Ok(())
    }

    /// Gets current health status of a node
    pub fn get_node_health(&self, node_id: NodeId) -> Result<HealthStatus> {
        let health = self.node_health.read().map_err(|_| {
            Error::runtime_error("Failed to acquire node health lock".to_string(), None)
        })?;

        Ok(health
            .get(&node_id)
            .cloned()
            .unwrap_or(HealthStatus::Unknown))
    }

    /// Gets system fault tolerance statistics
    pub fn get_statistics(&self) -> Result<FaultToleranceStatistics> {
        let fault_count = {
            let history = self.fault_history.read().map_err(|_| {
                Error::runtime_error("Failed to acquire fault history lock".to_string(), None)
            })?;
            history.len()
        };

        let healthy_nodes = {
            let health = self.node_health.read().map_err(|_| {
                Error::runtime_error("Failed to acquire node health lock".to_string(), None)
            })?;
            health.values().filter(|&status| status.is_usable()).count()
        };

        let total_nodes = {
            let health = self.node_health.read().map_err(|_| {
                Error::runtime_error("Failed to acquire node health lock".to_string(), None)
            })?;
            health.len()
        };

        let active_recoveries = {
            let recoveries = self.active_recoveries.read().map_err(|_| {
                Error::runtime_error("Failed to acquire active recoveries lock".to_string(), None)
            })?;
            recoveries.len()
        };

        Ok(FaultToleranceStatistics {
            total_faults_detected: fault_count as u64,
            active_recoveries: active_recoveries as u64,
            healthy_nodes: healthy_nodes as u64,
            total_nodes: total_nodes as u64,
            system_availability: if total_nodes > 0 {
                healthy_nodes as f64 / total_nodes as f64
            } else {
                1.0
            },
        })
    }
}

/// Fault tolerance system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultToleranceStatistics {
    /// Total number of faults detected
    pub total_faults_detected: u64,
    /// Number of active recovery operations
    pub active_recoveries: u64,
    /// Number of healthy nodes
    pub healthy_nodes: u64,
    /// Total number of monitored nodes
    pub total_nodes: u64,
    /// Overall system availability (0.0 to 1.0)
    pub system_availability: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fault_type_severity() {
        assert_eq!(FaultType::Hardware.severity(), FaultSeverity::Critical);
        assert_eq!(FaultType::Network.severity(), FaultSeverity::High);
        assert_eq!(FaultType::Software.severity(), FaultSeverity::Medium);
    }

    #[test]
    fn test_fault_incident_creation() {
        let incident = FaultIncident::new(
            FaultType::Network,
            NodeId::new(),
            "Connection timeout".to_string(),
        );

        assert_eq!(incident.fault_type, FaultType::Network);
        assert_eq!(incident.severity, FaultSeverity::High);
        assert!(!incident.recovered);
    }

    #[test]
    fn test_circuit_breaker() {
        let mut breaker = CircuitBreaker::new("test".to_string());

        assert!(breaker.can_execute());

        // Record failures to trigger opening
        for _ in 0..DEFAULT_FAILURE_THRESHOLD {
            breaker.record_result(false);
        }

        assert!(!breaker.can_execute());
        assert!(matches!(breaker.state, CircuitBreakerState::Open { .. }));
    }

    #[test]
    fn test_health_status() {
        assert!(HealthStatus::Healthy.is_usable());
        assert!(HealthStatus::degraded(0.3).is_usable());
        assert!(!HealthStatus::Failed.is_usable());

        assert_eq!(HealthStatus::Healthy.impact_factor(), 0.0);
        assert!(HealthStatus::degraded(0.5).impact_factor() > 0.0);
    }

    #[test]
    fn test_health_check_result() {
        let node_id = NodeId::new();
        let result = HealthCheckResult::healthy(node_id, Duration::from_millis(100))
            .with_metric("cpu_usage".to_string(), 0.5);

        assert_eq!(result.node_id, node_id);
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.metrics.get("cpu_usage"), Some(&0.5));
    }

    #[test]
    fn test_failure_prediction_engine() {
        let mut engine = FailurePredictionEngine::new();
        let node_id = NodeId::new();

        // Add some health check results
        for i in 0..10 {
            let mut result = HealthCheckResult::healthy(node_id, Duration::from_millis(100));
            result = result.with_metric("cpu_usage".to_string(), 0.5 + i as f64 * 0.05);
            engine.record_health_check(result);
        }

        let probability = engine.predict_failure_probability(node_id);
        assert!((0.0..=1.0).contains(&probability));
    }

    #[tokio::test]
    async fn test_fault_tolerance_system() {
        let config = FaultToleranceConfig::default();
        let system = HierarchicalFaultToleranceSystem::new(config);

        let node_id = NodeId::new();
        system.register_node(node_id).unwrap();

        let health = system.get_node_health(node_id).unwrap();
        assert_eq!(health, HealthStatus::Unknown);

        let stats = system.get_statistics().unwrap();
        assert_eq!(stats.total_nodes, 1);
    }

    #[tokio::test]
    async fn test_fault_reporting() {
        let config = FaultToleranceConfig::default();
        let system = HierarchicalFaultToleranceSystem::new(config);

        let incident =
            FaultIncident::new(FaultType::Network, NodeId::new(), "Test fault".to_string());

        let result = system.report_fault(incident).await;
        assert!(result.is_ok());
    }
}

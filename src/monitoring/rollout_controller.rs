//! Rollout Controller for Phase 8.3 Production Deployment
//!
//! Manages gradual rollout of optimizations from 10% → 100% with
//! automatic circuit breakers and fallback capabilities.

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, SystemTime, Instant};
use crate::feature::optimization_features::{OptimizationFeature, global_optimization_flags};
use crate::monitoring::production_metrics::{ProductionMetrics, Alert, AlertSeverity};

/// Rollout controller manages gradual deployment
pub struct RolloutController {
    /// Current rollout configuration
    config: Arc<RwLock<RolloutConfig>>,
    /// Current rollout status
    status: Arc<RwLock<RolloutStatus>>,
    /// Circuit breaker states
    circuit_breakers: Arc<Mutex<HashMap<OptimizationFeature, CircuitBreaker>>>,
    /// Rollout schedule
    schedule: Arc<RwLock<RolloutSchedule>>,
    /// Start time for rollout tracking
    start_time: Instant,
}

/// Rollout configuration
#[derive(Debug, Clone)]
pub struct RolloutConfig {
    /// Target rollout stages and their durations
    pub stages: Vec<RolloutStage>,
    /// Circuit breaker thresholds
    pub circuit_breaker_config: CircuitBreakerConfig,
    /// Automatic progression settings
    pub auto_progression: AutoProgressionConfig,
    /// Monitoring integration settings
    pub monitoring_config: MonitoringConfig,
}

/// Individual rollout stage
#[derive(Debug, Clone)]
pub struct RolloutStage {
    /// Stage name
    pub name: String,
    /// Target rollout percentage (0.0 to 1.0)
    pub target_percentage: f32,
    /// Minimum duration at this stage
    pub minimum_duration: Duration,
    /// Success criteria to advance to next stage
    pub advancement_criteria: AdvancementCriteria,
    /// Feature-specific rollout percentages
    pub feature_rollouts: HashMap<OptimizationFeature, f32>,
}

/// Criteria for advancing to the next rollout stage
#[derive(Debug, Clone)]
pub struct AdvancementCriteria {
    /// Maximum acceptable error rate
    pub max_error_rate: f32,
    /// Maximum response time degradation ratio
    pub max_latency_degradation: f32,
    /// Minimum system health score
    pub min_health_score: f32,
    /// Required observation period with stable metrics
    pub stability_period: Duration,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Error rate threshold for circuit breaker activation
    pub error_rate_threshold: f32,
    /// Response time threshold for circuit breaker activation
    pub response_time_threshold: f64,
    /// Memory usage threshold for circuit breaker activation
    pub memory_threshold: u64,
    /// Time window for threshold evaluation
    pub evaluation_window: Duration,
    /// Cooldown period before retry
    pub cooldown_period: Duration,
}

/// Auto-progression configuration
#[derive(Debug, Clone)]
pub struct AutoProgressionConfig {
    /// Enable automatic stage progression
    pub enabled: bool,
    /// Require manual approval for critical stages
    pub require_manual_approval: bool,
    /// Time between progression checks
    pub check_interval: Duration,
    /// Maximum rollout speed (percentage per hour)
    pub max_rollout_speed: f32,
}

/// Monitoring integration configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Enable real-time monitoring
    pub enabled: bool,
    /// Metrics collection interval
    pub metrics_interval: Duration,
    /// Alert notification settings
    pub alert_settings: AlertSettings,
}

/// Alert notification settings
#[derive(Debug, Clone)]
pub struct AlertSettings {
    /// Send alerts on stage advancement
    pub stage_advancement: bool,
    /// Send alerts on circuit breaker activation
    pub circuit_breaker_activation: bool,
    /// Send alerts on rollback
    pub rollback_notifications: bool,
    /// Alert channels
    pub channels: Vec<String>,
}

/// Current rollout status
#[derive(Debug, Clone)]
pub struct RolloutStatus {
    /// Current rollout stage
    pub current_stage: usize,
    /// Overall rollout progress (0.0 to 1.0)
    pub progress_percentage: f32,
    /// Current feature rollout percentages
    pub feature_percentages: HashMap<OptimizationFeature, f32>,
    /// Time at current stage
    pub time_at_current_stage: Duration,
    /// Stage advancement eligibility
    pub can_advance: bool,
    /// Active circuit breakers
    pub active_circuit_breakers: Vec<OptimizationFeature>,
    /// Recent rollback events
    pub recent_rollbacks: Vec<RollbackEvent>,
    /// Last status update time
    pub last_updated: SystemTime,
}

/// Circuit breaker state for individual optimization features
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Current state
    pub state: CircuitBreakerState,
    /// Failure count in current evaluation window
    pub failure_count: u32,
    /// Last failure time
    pub last_failure_time: Option<SystemTime>,
    /// Circuit opened time
    pub opened_at: Option<SystemTime>,
    /// Configuration
    pub config: CircuitBreakerConfig,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,    // Normal operation
    Open,      // Feature disabled due to failures
    HalfOpen,  // Testing if feature can be re-enabled
}

/// Rollback event record
#[derive(Debug, Clone)]
pub struct RollbackEvent {
    /// Rollback timestamp
    pub timestamp: SystemTime,
    /// Features affected
    pub features: Vec<OptimizationFeature>,
    /// Rollback reason
    pub reason: String,
    /// Rollback from percentage
    pub from_percentage: f32,
    /// Rollback to percentage
    pub to_percentage: f32,
    /// Automatic or manual rollback
    pub automatic: bool,
}

/// Rollout schedule with planned stages
#[derive(Debug, Clone)]
pub struct RolloutSchedule {
    /// Planned stages with timestamps
    pub planned_stages: Vec<(SystemTime, RolloutStage)>,
    /// Current stage start time
    pub current_stage_start: SystemTime,
    /// Next scheduled advancement
    pub next_advancement: Option<SystemTime>,
}

impl Default for RolloutConfig {
    fn default() -> Self {
        Self {
            stages: vec![
                RolloutStage {
                    name: "Initial Beta".to_string(),
                    target_percentage: 0.10,
                    minimum_duration: Duration::from_secs(3600), // 1 hour
                    advancement_criteria: AdvancementCriteria {
                        max_error_rate: 0.02,
                        max_latency_degradation: 1.15,
                        min_health_score: 0.90,
                        stability_period: Duration::from_secs(1800), // 30 minutes
                    },
                    feature_rollouts: HashMap::new(),
                },
                RolloutStage {
                    name: "Expanded Beta".to_string(),
                    target_percentage: 0.25,
                    minimum_duration: Duration::from_secs(7200), // 2 hours
                    advancement_criteria: AdvancementCriteria {
                        max_error_rate: 0.015,
                        max_latency_degradation: 1.10,
                        min_health_score: 0.92,
                        stability_period: Duration::from_secs(3600), // 1 hour
                    },
                    feature_rollouts: HashMap::new(),
                },
                RolloutStage {
                    name: "Production A/B".to_string(),
                    target_percentage: 0.50,
                    minimum_duration: Duration::from_secs(14400), // 4 hours
                    advancement_criteria: AdvancementCriteria {
                        max_error_rate: 0.01,
                        max_latency_degradation: 1.05,
                        min_health_score: 0.95,
                        stability_period: Duration::from_secs(7200), // 2 hours
                    },
                    feature_rollouts: HashMap::new(),
                },
                RolloutStage {
                    name: "Full Production".to_string(),
                    target_percentage: 1.00,
                    minimum_duration: Duration::from_secs(0), // No minimum for final stage
                    advancement_criteria: AdvancementCriteria {
                        max_error_rate: 0.005,
                        max_latency_degradation: 1.02,
                        min_health_score: 0.98,
                        stability_period: Duration::from_secs(10800), // 3 hours
                    },
                    feature_rollouts: HashMap::new(),
                },
            ],
            circuit_breaker_config: CircuitBreakerConfig {
                error_rate_threshold: 0.05,
                response_time_threshold: 500_000.0, // 500ms in microseconds
                memory_threshold: 2_000_000_000, // 2GB
                evaluation_window: Duration::from_secs(300), // 5 minutes
                cooldown_period: Duration::from_secs(1800), // 30 minutes
            },
            auto_progression: AutoProgressionConfig {
                enabled: true,
                require_manual_approval: false,
                check_interval: Duration::from_secs(300), // 5 minutes
                max_rollout_speed: 0.20, // 20% per hour maximum
            },
            monitoring_config: MonitoringConfig {
                enabled: true,
                metrics_interval: Duration::from_secs(60), // 1 minute
                alert_settings: AlertSettings {
                    stage_advancement: true,
                    circuit_breaker_activation: true,
                    rollback_notifications: true,
                    channels: vec!["console".to_string()],
                },
            },
        }
    }
}

impl RolloutController {
    /// Create new rollout controller with default configuration
    pub fn new() -> Self {
        let config = RolloutConfig::default();
        let status = RolloutStatus {
            current_stage: 0,
            progress_percentage: 0.0,
            feature_percentages: HashMap::new(),
            time_at_current_stage: Duration::from_secs(0),
            can_advance: false,
            active_circuit_breakers: Vec::new(),
            recent_rollbacks: Vec::new(),
            last_updated: SystemTime::now(),
        };
        
        Self {
            config: Arc::new(RwLock::new(config)),
            status: Arc::new(RwLock::new(status)),
            circuit_breakers: Arc::new(Mutex::new(HashMap::new())),
            schedule: Arc::new(RwLock::new(RolloutSchedule {
                planned_stages: Vec::new(),
                current_stage_start: SystemTime::now(),
                next_advancement: None,
            })),
            start_time: Instant::now(),
        }
    }

    /// Start rollout controller with metrics integration
    pub fn start(&self, metrics: Arc<ProductionMetrics>) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting Phase 8.3 Rollout Controller");
        
        // Initialize circuit breakers for all optimization features
        {
            let mut breakers = self.circuit_breakers.lock().unwrap();
            let config = self.config.read().unwrap();
            
            for feature in OptimizationFeature::all() {
                breakers.insert(feature, CircuitBreaker {
                    state: CircuitBreakerState::Closed,
                    failure_count: 0,
                    last_failure_time: None,
                    opened_at: None,
                    config: config.circuit_breaker_config.clone(),
                });
            }
        }
        
        // Set initial rollout percentages
        self.set_initial_rollout()?;
        
        // Start monitoring loop (in a real implementation, this would be in a separate thread)
        self.update_rollout_status(&metrics)?;
        
        println!("✅ Rollout controller started successfully");
        Ok(())
    }

    /// Set initial rollout percentages (10% for stable features)
    fn set_initial_rollout(&self) -> Result<(), Box<dyn std::error::Error>> {
        let flags = global_optimization_flags();
        
        // Enable stable features at 10% rollout
        flags.set_rollout_percentage(OptimizationFeature::StringInterning, 1.0); // 100% - proven stable
        flags.set_rollout_percentage(OptimizationFeature::NanBoxedValues, 0.10); // 10% - initial rollout
        flags.set_rollout_percentage(OptimizationFeature::ArenaAllocation, 0.10); // 10% - initial rollout
        flags.set_rollout_percentage(OptimizationFeature::ZeroCopyParsing, 0.05); // 5% - most conservative
        
        // Update status
        {
            let mut status = self.status.write().unwrap();
            status.feature_percentages.insert(OptimizationFeature::StringInterning, 1.0);
            status.feature_percentages.insert(OptimizationFeature::NanBoxedValues, 0.10);
            status.feature_percentages.insert(OptimizationFeature::ArenaAllocation, 0.10);
            status.feature_percentages.insert(OptimizationFeature::ZeroCopyParsing, 0.05);
            status.progress_percentage = 0.10;
            status.last_updated = SystemTime::now();
        }
        
        println!("📊 Initial rollout configuration set:");
        println!("  - String Interning: 100% (stable)");
        println!("  - NaN-boxing: 10%");
        println!("  - Arena Allocation: 10%");
        println!("  - Zero-copy Parsing: 5%");
        
        Ok(())
    }

    /// Update rollout status based on current metrics
    pub fn update_rollout_status(&self, metrics: &ProductionMetrics) -> Result<Vec<Alert>, Box<dyn std::error::Error>> {
        let mut alerts = Vec::new();
        
        // Get current metrics
        let report = metrics.generate_report();
        
        // Check circuit breakers
        alerts.extend(self.check_circuit_breakers(&report)?);
        
        // Evaluate advancement criteria
        let can_advance = self.evaluate_advancement_criteria(&report)?;
        
        // Update status
        {
            let mut status = self.status.write().unwrap();
            status.can_advance = can_advance;
            status.time_at_current_stage = self.start_time.elapsed();
            status.last_updated = SystemTime::now();
            
            // Update circuit breaker list
            let breakers = self.circuit_breakers.lock().unwrap();
            status.active_circuit_breakers = breakers
                .iter()
                .filter_map(|(feature, breaker)| {
                    if breaker.state != CircuitBreakerState::Closed {
                        Some(*feature)
                    } else {
                        None
                    }
                })
                .collect();
        }
        
        // Auto-advance if criteria are met
        if can_advance {
            let config = self.config.read().unwrap();
            if config.auto_progression.enabled {
                alerts.extend(self.try_advance_stage()?);
            }
        }
        
        Ok(alerts)
    }

    /// Check circuit breaker states and trigger if necessary
    fn check_circuit_breakers(&self, report: &crate::monitoring::production_metrics::MetricsReport) -> Result<Vec<Alert>, Box<dyn std::error::Error>> {
        let mut alerts = Vec::new();
        let mut breakers = self.circuit_breakers.lock().unwrap();
        
        for (feature, breaker) in breakers.iter_mut() {
            let should_trigger = match feature {
                OptimizationFeature::NanBoxedValues => {
                    report.performance.error_rate > breaker.config.error_rate_threshold ||
                    report.performance.avg_response_time_micros > breaker.config.response_time_threshold
                }
                OptimizationFeature::ArenaAllocation => {
                    report.performance.memory_usage_bytes > breaker.config.memory_threshold
                }
                OptimizationFeature::ZeroCopyParsing => {
                    report.performance.error_rate > breaker.config.error_rate_threshold * 0.5 // More strict
                }
                _ => false, // String interning is stable, no circuit breaker needed
            };
            
            if should_trigger && breaker.state == CircuitBreakerState::Closed {
                // Trigger circuit breaker
                breaker.state = CircuitBreakerState::Open;
                breaker.opened_at = Some(SystemTime::now());
                breaker.failure_count += 1;
                breaker.last_failure_time = Some(SystemTime::now());
                
                // Disable feature
                let flags = global_optimization_flags();
                flags.set_enabled(*feature, false);
                
                alerts.push(Alert {
                    severity: AlertSeverity::Critical,
                    title: "Circuit Breaker Activated".to_string(),
                    message: format!("Circuit breaker activated for {:?} due to performance degradation", feature),
                    metric_name: format!("{:?}_circuit_breaker", feature),
                    current_value: if report.performance.error_rate > breaker.config.error_rate_threshold {
                        report.performance.error_rate as f64
                    } else {
                        report.performance.avg_response_time_micros
                    },
                    threshold_value: if report.performance.error_rate > breaker.config.error_rate_threshold {
                        breaker.config.error_rate_threshold as f64
                    } else {
                        breaker.config.response_time_threshold
                    },
                    timestamp: SystemTime::now(),
                    suggested_action: format!("Feature {:?} has been automatically disabled. Investigate performance issues.", feature),
                });
                
                println!("🚨 Circuit breaker activated for {:?}", feature);
            }
        }
        
        Ok(alerts)
    }

    /// Evaluate criteria for advancing to the next stage
    fn evaluate_advancement_criteria(&self, report: &crate::monitoring::production_metrics::MetricsReport) -> Result<bool, Box<dyn std::error::Error>> {
        let config = self.config.read().unwrap();
        let status = self.status.read().unwrap();
        
        if status.current_stage >= config.stages.len() {
            return Ok(false); // Already at final stage
        }
        
        let current_stage = &config.stages[status.current_stage];
        let criteria = &current_stage.advancement_criteria;
        
        // Check minimum duration
        if status.time_at_current_stage < current_stage.minimum_duration {
            return Ok(false);
        }
        
        // Check error rate
        if report.performance.error_rate > criteria.max_error_rate {
            return Ok(false);
        }
        
        // Check latency degradation (would need baseline comparison in real implementation)
        // For now, just check absolute threshold
        if report.performance.avg_response_time_micros > 100_000.0 { // 100ms
            return Ok(false);
        }
        
        // Check system health
        if report.system_health < criteria.min_health_score {
            return Ok(false);
        }
        
        // All criteria met
        Ok(true)
    }

    /// Try to advance to the next rollout stage
    fn try_advance_stage(&self) -> Result<Vec<Alert>, Box<dyn std::error::Error>> {
        let mut alerts = Vec::new();
        let config = self.config.read().unwrap();
        
        let next_stage_index = {
            let mut status = self.status.write().unwrap();
            if status.current_stage + 1 >= config.stages.len() {
                return Ok(alerts); // Already at final stage
            }
            
            status.current_stage += 1;
            status.current_stage
        };
        
        let next_stage = &config.stages[next_stage_index];
        
        // Update rollout percentages
        let flags = global_optimization_flags();
        flags.set_rollout_percentage(OptimizationFeature::NanBoxedValues, next_stage.target_percentage);
        flags.set_rollout_percentage(OptimizationFeature::ArenaAllocation, next_stage.target_percentage * 0.8);
        flags.set_rollout_percentage(OptimizationFeature::ZeroCopyParsing, next_stage.target_percentage * 0.5);
        
        // Update status
        {
            let mut status = self.status.write().unwrap();
            status.feature_percentages.insert(OptimizationFeature::NanBoxedValues, next_stage.target_percentage);
            status.feature_percentages.insert(OptimizationFeature::ArenaAllocation, next_stage.target_percentage * 0.8);
            status.feature_percentages.insert(OptimizationFeature::ZeroCopyParsing, next_stage.target_percentage * 0.5);
            status.progress_percentage = next_stage.target_percentage;
            status.time_at_current_stage = Duration::from_secs(0);
            status.can_advance = false; // Reset until criteria are evaluated again
        }
        
        alerts.push(Alert {
            severity: AlertSeverity::Info,
            title: "Rollout Stage Advanced".to_string(),
            message: format!("Advanced to rollout stage: {} ({:.1}%)", next_stage.name, next_stage.target_percentage * 100.0),
            metric_name: "rollout_stage".to_string(),
            current_value: next_stage_index as f64,
            threshold_value: (next_stage_index - 1) as f64,
            timestamp: SystemTime::now(),
            suggested_action: "Monitor metrics closely for the next period to ensure stability".to_string(),
        });
        
        println!("📈 Advanced to rollout stage: {} ({:.1}%)", next_stage.name, next_stage.target_percentage * 100.0);
        
        Ok(alerts)
    }

    /// Get current rollout status
    pub fn current_status(&self) -> RolloutStatus {
        self.status.read().unwrap().clone()
    }

    /// Force rollback to previous stage
    pub fn force_rollback(&self, reason: String) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().unwrap();
        
        let (from_percentage, to_percentage) = {
            let mut status = self.status.write().unwrap();
            
            let from_percentage = status.progress_percentage;
            
            // Rollback to previous stage or 0% if at first stage
            if status.current_stage > 0 {
                status.current_stage -= 1;
                let target_stage = &config.stages[status.current_stage];
                status.progress_percentage = target_stage.target_percentage;
            } else {
                status.progress_percentage = 0.0;
            }
            
            let to_percentage = status.progress_percentage;
            
            // Record rollback event
            let rollback = RollbackEvent {
                timestamp: SystemTime::now(),
                features: vec![
                    OptimizationFeature::NanBoxedValues,
                    OptimizationFeature::ArenaAllocation,
                    OptimizationFeature::ZeroCopyParsing,
                ],
                reason: reason.clone(),
                from_percentage,
                to_percentage,
                automatic: false,
            };
            
            status.recent_rollbacks.push(rollback);
            
            (from_percentage, to_percentage)
        };
        
        // Update feature flags
        let flags = global_optimization_flags();
        flags.set_rollout_percentage(OptimizationFeature::NanBoxedValues, to_percentage);
        flags.set_rollout_percentage(OptimizationFeature::ArenaAllocation, to_percentage * 0.8);
        flags.set_rollout_percentage(OptimizationFeature::ZeroCopyParsing, to_percentage * 0.5);
        
        println!("⏪ Rollback completed: {:.1}% → {:.1}% ({})", 
            from_percentage * 100.0, to_percentage * 100.0, reason);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollout_controller_creation() {
        let controller = RolloutController::new();
        let status = controller.current_status();
        assert_eq!(status.current_stage, 0);
        assert_eq!(status.progress_percentage, 0.0);
    }

    #[test]
    fn test_circuit_breaker_states() {
        let breaker = CircuitBreaker {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            last_failure_time: None,
            opened_at: None,
            config: CircuitBreakerConfig::default(),
        };
        
        assert_eq!(breaker.state, CircuitBreakerState::Closed);
    }

    #[test]
    fn test_rollback_functionality() {
        let controller = RolloutController::new();
        let result = controller.force_rollback("Test rollback".to_string());
        assert!(result.is_ok());
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            error_rate_threshold: 0.05,
            response_time_threshold: 500_000.0,
            memory_threshold: 2_000_000_000,
            evaluation_window: Duration::from_secs(300),
            cooldown_period: Duration::from_secs(1800),
        }
    }
}
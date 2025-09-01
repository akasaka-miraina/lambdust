//! Alert Manager for Phase 8.3 Production Monitoring
//!
//! Handles alert routing, deduplication, escalation, and notification
//! delivery across multiple channels.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, SystemTime};
use crate::monitoring::production_metrics::{Alert, AlertSeverity};

/// Alert manager handles all alerting functionality
pub struct AlertManager {
    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, ActiveAlert>>>,
    /// Alert history
    alert_history: Arc<Mutex<VecDeque<HistoricalAlert>>>,
    /// Alert channels for notification delivery
    channels: Arc<RwLock<Vec<Box<dyn AlertChannel + Send + Sync>>>>,
    /// Alert routing rules
    routing_rules: Arc<RwLock<Vec<AlertRoutingRule>>>,
    /// Deduplication configuration
    dedup_config: DeduplicationConfig,
    /// Escalation rules
    escalation_rules: Arc<RwLock<Vec<EscalationRule>>>,
}

/// Active alert with state tracking
#[derive(Debug, Clone)]
pub struct ActiveAlert {
    /// Original alert
    pub alert: Alert,
    /// First occurrence time
    pub first_seen: SystemTime,
    /// Last occurrence time
    pub last_seen: SystemTime,
    /// Occurrence count
    pub count: u32,
    /// Current escalation level
    pub escalation_level: u32,
    /// Next escalation time
    pub next_escalation: Option<SystemTime>,
    /// Acknowledgment status
    pub acknowledged: bool,
    /// Acknowledged by
    pub acknowledged_by: Option<String>,
    /// Acknowledgment time
    pub acknowledged_at: Option<SystemTime>,
}

/// Historical alert record
#[derive(Debug, Clone)]
pub struct HistoricalAlert {
    /// Original alert
    pub alert: Alert,
    /// Active period
    pub duration: Duration,
    /// Total occurrences
    pub occurrences: u32,
    /// Resolution time
    pub resolved_at: SystemTime,
    /// Resolution method (auto, manual, timeout)
    pub resolution_method: String,
}

/// Alert channel trait for different notification methods
pub trait AlertChannel {
    /// Send alert notification
    fn send_alert(&self, alert: &ActiveAlert) -> Result<(), AlertChannelError>;
    
    /// Get channel name
    fn name(&self) -> &str;
    
    /// Check if channel is healthy/available
    fn is_healthy(&self) -> bool;
    
    /// Get channel configuration
    fn config(&self) -> AlertChannelConfig;
}

/// Alert channel configuration
#[derive(Debug, Clone)]
pub struct AlertChannelConfig {
    /// Channel name
    pub name: String,
    /// Supported severity levels
    pub supported_severities: Vec<AlertSeverity>,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimit>,
    /// Retry configuration
    pub retry_config: RetryConfig,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Maximum alerts per time window
    pub max_alerts: u32,
    /// Time window duration
    pub window: Duration,
    /// Current count in window
    pub current_count: u32,
    /// Window start time
    pub window_start: SystemTime,
}

/// Retry configuration for failed deliveries
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Base retry delay
    pub base_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum retry delay
    pub max_delay: Duration,
}

/// Alert routing rule
#[derive(Debug, Clone)]
pub struct AlertRoutingRule {
    /// Rule name
    pub name: String,
    /// Alert matching criteria
    pub criteria: AlertCriteria,
    /// Target channels
    pub channels: Vec<String>,
    /// Priority adjustment
    pub priority_adjustment: i32,
}

/// Alert matching criteria
#[derive(Debug, Clone)]
pub struct AlertCriteria {
    /// Severity levels to match
    pub severities: Option<Vec<AlertSeverity>>,
    /// Metric names to match (regex patterns)
    pub metric_patterns: Option<Vec<String>>,
    /// Time-based criteria
    pub time_criteria: Option<TimeCriteria>,
    /// Value-based criteria
    pub value_criteria: Option<ValueCriteria>,
}

/// Time-based alert criteria
#[derive(Debug, Clone)]
pub struct TimeCriteria {
    /// Only during business hours
    pub business_hours_only: bool,
    /// Days of week (0=Sunday, 6=Saturday)
    pub days_of_week: Option<Vec<u32>>,
    /// Time range (24-hour format)
    pub time_range: Option<(u32, u32)>,
}

/// Value-based alert criteria
#[derive(Debug, Clone)]
pub struct ValueCriteria {
    /// Minimum threshold value
    pub min_threshold: Option<f64>,
    /// Maximum threshold value
    pub max_threshold: Option<f64>,
    /// Minimum duration before alerting
    pub min_duration: Option<Duration>,
}

/// Escalation rule
#[derive(Debug, Clone)]
pub struct EscalationRule {
    /// Rule name
    pub name: String,
    /// Severity level this rule applies to
    pub severity: AlertSeverity,
    /// Time until first escalation
    pub initial_delay: Duration,
    /// Time between subsequent escalations
    pub escalation_interval: Duration,
    /// Maximum escalation level
    pub max_level: u32,
    /// Channels for each escalation level
    pub level_channels: HashMap<u32, Vec<String>>,
}

/// Deduplication configuration
#[derive(Debug, Clone)]
pub struct DeduplicationConfig {
    /// Time window for deduplication
    pub time_window: Duration,
    /// Fields to use for deduplication key
    pub dedup_fields: Vec<String>,
    /// Maximum occurrences before forcing new alert
    pub max_occurrences: u32,
}

/// Alert channel error types
#[derive(Debug)]
pub struct AlertChannelError {
    pub channel_name: String,
    pub error_message: String,
    pub retry_after: Option<Duration>,
}

impl AlertManager {
    /// Create new alert manager with default configuration
    pub fn new() -> Self {
        Self {
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(Mutex::new(VecDeque::new())),
            channels: Arc::new(RwLock::new(Vec::new())),
            routing_rules: Arc::new(RwLock::new(Vec::new())),
            dedup_config: DeduplicationConfig::default(),
            escalation_rules: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start alert manager
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting Alert Manager");
        
        // Add default channels
        self.add_console_channel()?;
        
        // Add default routing rules
        self.add_default_routing_rules()?;
        
        // Add default escalation rules
        self.add_default_escalation_rules()?;
        
        println!("✅ Alert Manager started successfully");
        Ok(())
    }

    /// Process new alert
    pub fn process_alert(&self, alert: Alert) -> Result<(), Box<dyn std::error::Error>> {
        let alert_key = self.generate_dedup_key(&alert);
        
        // Check for existing active alert
        let (is_new_alert, mut active_alert) = {
            let mut active = self.active_alerts.write().unwrap();
            
            if let Some(existing) = active.get_mut(&alert_key) {
                // Update existing alert
                existing.last_seen = SystemTime::now();
                existing.count += 1;
                existing.alert = alert; // Update with latest values
                (false, existing.clone())
            } else {
                // Create new active alert
                let new_active = ActiveAlert {
                    alert: alert.clone(),
                    first_seen: SystemTime::now(),
                    last_seen: SystemTime::now(),
                    count: 1,
                    escalation_level: 0,
                    next_escalation: self.calculate_next_escalation(&alert),
                    acknowledged: false,
                    acknowledged_by: None,
                    acknowledged_at: None,
                };
                
                active.insert(alert_key.clone(), new_active.clone());
                (true, new_active)
            }
        };

        // Send notifications for new alerts or significant updates
        if is_new_alert || active_alert.count % 10 == 0 { // Every 10th occurrence
            self.send_notifications(&active_alert)?;
        }

        // Check for escalation
        self.check_escalation(&alert_key)?;

        Ok(())
    }

    /// Generate deduplication key for alert
    fn generate_dedup_key(&self, alert: &Alert) -> String {
        format!("{}:{}:{}",
            alert.severity,
            alert.title,
            alert.metric_name
        )
    }

    /// Calculate next escalation time
    fn calculate_next_escalation(&self, alert: &Alert) -> Option<SystemTime> {
        let escalation_rules = self.escalation_rules.read().unwrap();
        
        for rule in escalation_rules.iter() {
            if rule.severity == alert.severity {
                return Some(SystemTime::now() + rule.initial_delay);
            }
        }
        
        None
    }

    /// Send notifications for alert
    fn send_notifications(&self, active_alert: &ActiveAlert) -> Result<(), Box<dyn std::error::Error>> {
        let channels = self.get_matching_channels(&active_alert.alert)?;
        
        for channel in channels {
            if let Err(e) = channel.send_alert(active_alert) {
                eprintln!("Failed to send alert via {}: {}", e.channel_name, e.error_message);
                // In production, this would be logged and potentially retried
            }
        }
        
        Ok(())
    }

    /// Get channels matching alert routing rules
    fn get_matching_channels(&self, alert: &Alert) -> Result<Vec<&Box<dyn AlertChannel + Send + Sync>>, Box<dyn std::error::Error>> {
        let routing_rules = self.routing_rules.read().unwrap();
        let channels = self.channels.read().unwrap();
        let mut matching_channels = Vec::new();
        
        for rule in routing_rules.iter() {
            if self.matches_criteria(alert, &rule.criteria) {
                for channel_name in &rule.channels {
                    if let Some(channel) = channels.iter().find(|c| c.name() == channel_name) {
                        matching_channels.push(channel);
                    }
                }
            }
        }
        
        // If no specific rules match, use all channels for critical alerts
        if matching_channels.is_empty() && alert.severity == AlertSeverity::Critical {
            matching_channels = channels.iter().collect();
        }
        
        Ok(matching_channels)
    }

    /// Check if alert matches criteria
    fn matches_criteria(&self, alert: &Alert, criteria: &AlertCriteria) -> bool {
        // Check severity
        if let Some(ref severities) = criteria.severities {
            if !severities.contains(&alert.severity) {
                return false;
            }
        }
        
        // Check metric patterns (simplified - would use regex in production)
        if let Some(ref patterns) = criteria.metric_patterns {
            let mut pattern_match = false;
            for pattern in patterns {
                if alert.metric_name.contains(pattern) {
                    pattern_match = true;
                    break;
                }
            }
            if !pattern_match {
                return false;
            }
        }
        
        // Additional criteria checking would be implemented here
        
        true
    }

    /// Check for alert escalation
    fn check_escalation(&self, alert_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let should_escalate = {
            let active = self.active_alerts.read().unwrap();
            
            if let Some(alert) = active.get(alert_key) {
                if let Some(next_escalation) = alert.next_escalation {
                    SystemTime::now() >= next_escalation && !alert.acknowledged
                } else {
                    false
                }
            } else {
                false
            }
        };
        
        if should_escalate {
            self.escalate_alert(alert_key)?;
        }
        
        Ok(())
    }

    /// Escalate alert to next level
    fn escalate_alert(&self, alert_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let escalated_alert = {
            let mut active = self.active_alerts.write().unwrap();
            
            if let Some(alert) = active.get_mut(alert_key) {
                alert.escalation_level += 1;
                
                // Calculate next escalation time
                let escalation_rules = self.escalation_rules.read().unwrap();
                if let Some(rule) = escalation_rules.iter().find(|r| r.severity == alert.alert.severity) {
                    if alert.escalation_level < rule.max_level {
                        alert.next_escalation = Some(SystemTime::now() + rule.escalation_interval);
                    } else {
                        alert.next_escalation = None;
                    }
                }
                
                Some(alert.clone())
            } else {
                None
            }
        };
        
        if let Some(alert) = escalated_alert {
            println!("📈 Alert escalated to level {}: {}", alert.escalation_level, alert.alert.title);
            self.send_escalation_notifications(&alert)?;
        }
        
        Ok(())
    }

    /// Send escalation notifications
    fn send_escalation_notifications(&self, alert: &ActiveAlert) -> Result<(), Box<dyn std::error::Error>> {
        let escalation_rules = self.escalation_rules.read().unwrap();
        let channels = self.channels.read().unwrap();
        
        for rule in escalation_rules.iter() {
            if rule.severity == alert.alert.severity {
                if let Some(channel_names) = rule.level_channels.get(&alert.escalation_level) {
                    for channel_name in channel_names {
                        if let Some(channel) = channels.iter().find(|c| c.name() == channel_name) {
                            let _ = channel.send_alert(alert); // Ignore errors for escalation
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Acknowledge alert
    pub fn acknowledge_alert(&self, alert_key: &str, acknowledged_by: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut active = self.active_alerts.write().unwrap();
        
        if let Some(alert) = active.get_mut(alert_key) {
            alert.acknowledged = true;
            alert.acknowledged_by = Some(acknowledged_by);
            alert.acknowledged_at = Some(SystemTime::now());
            alert.next_escalation = None; // Stop escalation
            
            println!("✅ Alert acknowledged: {}", alert.alert.title);
        }
        
        Ok(())
    }

    /// Resolve alert
    pub fn resolve_alert(&self, alert_key: &str, resolution_method: String) -> Result<(), Box<dyn std::error::Error>> {
        let resolved_alert = {
            let mut active = self.active_alerts.write().unwrap();
            active.remove(alert_key)
        };
        
        if let Some(alert) = resolved_alert {
            let historical = HistoricalAlert {
                alert: alert.alert.clone(),
                duration: alert.last_seen.duration_since(alert.first_seen)
                    .unwrap_or(Duration::from_secs(0)),
                occurrences: alert.count,
                resolved_at: SystemTime::now(),
                resolution_method,
            };
            
            let mut history = self.alert_history.lock().unwrap();
            history.push_back(historical);
            
            // Maintain history size limit
            while history.len() > 1000 {
                history.pop_front();
            }
            
            println!("🔧 Alert resolved: {}", alert.alert.title);
        }
        
        Ok(())
    }

    /// Get active alerts
    pub fn active_alerts(&self) -> Vec<Alert> {
        let active = self.active_alerts.read().unwrap();
        active.values().map(|a| a.alert.clone()).collect()
    }

    /// Add console alert channel
    fn add_console_channel(&self) -> Result<(), Box<dyn std::error::Error>> {
        let channel = Box::new(ConsoleAlertChannel::new());
        let mut channels = self.channels.write().unwrap();
        channels.push(channel);
        Ok(())
    }

    /// Add default routing rules
    fn add_default_routing_rules(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut rules = self.routing_rules.write().unwrap();
        
        rules.push(AlertRoutingRule {
            name: "critical_alerts".to_string(),
            criteria: AlertCriteria {
                severities: Some(vec![AlertSeverity::Critical, AlertSeverity::Emergency]),
                metric_patterns: None,
                time_criteria: None,
                value_criteria: None,
            },
            channels: vec!["console".to_string()],
            priority_adjustment: 0,
        });
        
        Ok(())
    }

    /// Add default escalation rules
    fn add_default_escalation_rules(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut rules = self.escalation_rules.write().unwrap();
        
        rules.push(EscalationRule {
            name: "critical_escalation".to_string(),
            severity: AlertSeverity::Critical,
            initial_delay: Duration::from_secs(300), // 5 minutes
            escalation_interval: Duration::from_secs(900), // 15 minutes
            max_level: 3,
            level_channels: {
                let mut map = HashMap::new();
                map.insert(1, vec!["console".to_string()]);
                map.insert(2, vec!["console".to_string()]);
                map.insert(3, vec!["console".to_string()]);
                map
            },
        });
        
        Ok(())
    }
}

/// Console alert channel implementation
pub struct ConsoleAlertChannel {
    config: AlertChannelConfig,
}

impl ConsoleAlertChannel {
    pub fn new() -> Self {
        Self {
            config: AlertChannelConfig {
                name: "console".to_string(),
                supported_severities: vec![
                    AlertSeverity::Info,
                    AlertSeverity::Warning,
                    AlertSeverity::Critical,
                    AlertSeverity::Emergency,
                ],
                rate_limit: None,
                retry_config: RetryConfig {
                    max_attempts: 1,
                    base_delay: Duration::from_secs(0),
                    backoff_multiplier: 1.0,
                    max_delay: Duration::from_secs(0),
                },
            },
        }
    }
}

impl AlertChannel for ConsoleAlertChannel {
    fn send_alert(&self, alert: &ActiveAlert) -> Result<(), AlertChannelError> {
        let severity_icon = match alert.alert.severity {
            AlertSeverity::Info => "ℹ️",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Critical => "🚨",
            AlertSeverity::Emergency => "🔥",
        };
        
        println!("{} ALERT [{}]: {}", severity_icon, alert.alert.severity, alert.alert.title);
        println!("   Message: {}", alert.alert.message);
        println!("   Metric: {} = {:.2} (threshold: {:.2})", 
            alert.alert.metric_name, alert.alert.current_value, alert.alert.threshold_value);
        println!("   Count: {} occurrences", alert.count);
        if alert.escalation_level > 0 {
            println!("   Escalation Level: {}", alert.escalation_level);
        }
        println!("   Suggested Action: {}", alert.alert.suggested_action);
        println!();
        
        Ok(())
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn is_healthy(&self) -> bool {
        true // Console is always available
    }

    fn config(&self) -> AlertChannelConfig {
        self.config.clone()
    }
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            time_window: Duration::from_secs(300), // 5 minutes
            dedup_fields: vec!["severity".to_string(), "title".to_string(), "metric_name".to_string()],
            max_occurrences: 100,
        }
    }
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARNING"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
            AlertSeverity::Emergency => write!(f, "EMERGENCY"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_manager_creation() {
        let manager = AlertManager::new();
        let active = manager.active_alerts();
        assert!(active.is_empty());
    }

    #[test]
    fn test_console_channel() {
        let channel = ConsoleAlertChannel::new();
        assert_eq!(channel.name(), "console");
        assert!(channel.is_healthy());
    }

    #[test]
    fn test_dedup_key_generation() {
        let manager = AlertManager::new();
        let alert = Alert {
            severity: AlertSeverity::Warning,
            title: "Test Alert".to_string(),
            message: "Test message".to_string(),
            metric_name: "test_metric".to_string(),
            current_value: 1.0,
            threshold_value: 0.5,
            timestamp: SystemTime::now(),
            suggested_action: "Test action".to_string(),
        };
        
        let key = manager.generate_dedup_key(&alert);
        assert!(key.contains("WARNING"));
        assert!(key.contains("Test Alert"));
        assert!(key.contains("test_metric"));
    }
}
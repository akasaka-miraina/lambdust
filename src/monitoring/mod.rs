//! Production Monitoring System for Phase 8.3 Rollout
//!
//! Comprehensive monitoring and alerting infrastructure that provides
//! real-time visibility into optimization performance and system health.

pub mod production_metrics;
pub mod dashboard_server;
pub mod alert_manager;
pub mod rollout_controller;

pub use production_metrics::{
    ProductionMetrics, PerformanceCounters, OptimizationMetrics, 
    OptimizationEffectiveness, Alert, AlertSeverity, MetricsReport,
    global_metrics
};

pub use dashboard_server::{DashboardServer, DashboardConfig};
pub use alert_manager::{AlertManager, AlertChannel, AlertHandler};
pub use rollout_controller::{RolloutController, RolloutStage, RolloutConfig};

use std::sync::Arc;
use std::time::Duration;

/// Complete monitoring system orchestrator
pub struct MonitoringSystem {
    metrics: Arc<ProductionMetrics>,
    alert_manager: AlertManager,
    rollout_controller: RolloutController,
    dashboard_server: Option<DashboardServer>,
}

impl MonitoringSystem {
    /// Create new monitoring system with default configuration
    pub fn new() -> Self {
        let metrics = Arc::new(ProductionMetrics::new());
        let alert_manager = AlertManager::new();
        let rollout_controller = RolloutController::new();
        
        Self {
            metrics,
            alert_manager,
            rollout_controller,
            dashboard_server: None,
        }
    }

    /// Start the complete monitoring system
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting Phase 8.3 Production Monitoring System");
        
        // Start metrics collection
        self.metrics.start_collection()?;
        
        // Start alert manager
        self.alert_manager.start()?;
        
        // Start rollout controller
        self.rollout_controller.start(Arc::clone(&self.metrics))?;
        
        // Optionally start dashboard server
        if let Some(ref mut dashboard) = self.dashboard_server {
            dashboard.start(Arc::clone(&self.metrics))?;
        }
        
        println!("✅ Monitoring system started successfully");
        Ok(())
    }

    /// Enable dashboard server
    pub fn enable_dashboard(&mut self, config: DashboardConfig) {
        self.dashboard_server = Some(DashboardServer::new(config));
    }

    /// Get current system status
    pub fn system_status(&self) -> SystemStatus {
        let report = self.metrics.generate_report();
        let rollout_status = self.rollout_controller.current_status();
        let active_alerts = self.alert_manager.active_alerts();
        
        SystemStatus {
            metrics_report: report,
            rollout_status,
            active_alerts,
            monitoring_uptime: self.metrics.uptime(),
        }
    }
}

/// Complete system status
#[derive(Debug)]
pub struct SystemStatus {
    pub metrics_report: MetricsReport,
    pub rollout_status: rollout_controller::RolloutStatus,
    pub active_alerts: Vec<Alert>,
    pub monitoring_uptime: Duration,
}

impl SystemStatus {
    /// Generate comprehensive status summary
    pub fn summary(&self) -> String {
        let mut summary = String::new();
        
        summary.push_str("=== Phase 8.3 Production System Status ===\n\n");
        
        // Metrics summary
        summary.push_str(&self.metrics_report.summary());
        summary.push_str("\n");
        
        // Rollout status
        summary.push_str(&format!("Rollout Status: {:?}\n", self.rollout_status.current_stage));
        summary.push_str(&format!("Rollout Progress: {:.1}%\n", self.rollout_status.progress_percentage));
        
        // Active alerts
        if self.active_alerts.is_empty() {
            summary.push_str("Active Alerts: None ✅\n");
        } else {
            summary.push_str(&format!("Active Alerts: {} 🚨\n", self.active_alerts.len()));
            for alert in &self.active_alerts {
                summary.push_str(&format!("  - {}: {} ({})\n", 
                    alert.severity, alert.title, alert.metric_name));
            }
        }
        
        summary.push_str(&format!("Monitoring Uptime: {:.1} hours\n", 
            self.monitoring_uptime.as_secs_f64() / 3600.0));
        
        summary
    }
}

/// Global monitoring system instance
static GLOBAL_MONITORING: std::sync::OnceLock<std::sync::Mutex<MonitoringSystem>> = std::sync::OnceLock::new();

/// Get global monitoring system
pub fn global_monitoring() -> &'static std::sync::Mutex<MonitoringSystem> {
    GLOBAL_MONITORING.get_or_init(|| std::sync::Mutex::new(MonitoringSystem::new()))
}

/// Start global monitoring system
pub fn start_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    let monitoring = global_monitoring();
    let mut system = monitoring.lock().unwrap();
    system.start()
}

/// Get current system status
pub fn get_system_status() -> SystemStatus {
    let monitoring = global_monitoring();
    let system = monitoring.lock().unwrap();
    system.system_status()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_system_creation() {
        let system = MonitoringSystem::new();
        let status = system.system_status();
        assert!(status.monitoring_uptime.as_secs() >= 0);
    }

    #[test]
    fn test_system_status_summary() {
        let system = MonitoringSystem::new();
        let status = system.system_status();
        let summary = status.summary();
        assert!(summary.contains("Phase 8.3"));
        assert!(summary.contains("Production System Status"));
    }
}
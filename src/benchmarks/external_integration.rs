//! External System Integration for Benchmark Results.
//!
//! This module provides configuration and integration capabilities for external
//! systems including CI/CD platforms, performance dashboards, and notification
//! channels for automated benchmark result reporting and monitoring.

use serde::{Deserialize, Serialize};

/// Configuration for integrating with external systems and services.
/// 
/// Enables automated reporting to CI/CD systems, performance dashboards,
/// and notification channels for continuous performance monitoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalReporting {
    /// GitHub integration for CI/CD
    pub github: Option<GitHubConfig>,
    /// Performance tracking dashboard
    pub dashboard: Option<DashboardConfig>,
    /// Slack/Discord notifications
    pub notifications: Option<NotificationConfig>,
}

/// Configuration for GitHub integration and CI/CD reporting.
/// 
/// Enables automatic issue creation on performance regressions
/// and PR comments with benchmark results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub repository identifier (owner/repo)
    pub repository: String,
    /// Environment variable containing GitHub token
    pub token_env_var: String,
    /// Whether to create issues on performance regression
    pub create_issues_on_regression: bool,
    /// Whether to comment benchmark results on pull requests
    pub comment_on_prs: bool,
}

/// Configuration for performance tracking dashboard integration.
/// 
/// Allows uploading benchmark results to external monitoring
/// and visualization platforms for trend analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Dashboard API endpoint URL
    pub endpoint: String,
    /// Environment variable containing dashboard API key
    pub api_key_env_var: String,
    /// Project identifier for dashboard integration
    pub project_id: String,
}

/// Configuration for performance change notifications.
/// 
/// Sends alerts to team communication channels when performance
/// changes exceed specified thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Webhook URL for notifications
    pub webhook_url: String,
    /// Performance change percentage threshold for notifications
    pub notification_threshold: f64, // Performance change percentage
}
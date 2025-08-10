//! Support types for regression analysis and recommendations.
//!
//! This module contains the supporting data types for cause analysis,
//! overall assessment, and action recommendations.

use serde::{Deserialize, Serialize};

/// Suspected cause of performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspectedCause {
    /// Type of suspected cause
    pub cause_type: CauseType,
    /// Description
    pub description: String,
    /// Confidence in this cause (0-100)
    pub confidence: f64,
    /// Supporting evidence
    pub evidence: Vec<String>,
}

/// Types of causes for performance changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CauseType {
    /// Code or implementation changes
    CodeChange,
    /// Compiler version or optimization changes
    CompilerChange,
    /// System configuration modifications
    SystemConfiguration,
    /// External environmental factors
    ExternalFactors,
    /// Test methodology or setup changes
    TestMethodology,
    /// Hardware or infrastructure changes
    HardwareChange,
    /// Unknown or unidentified cause
    Unknown,
}

/// Likely cause of performance improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImprovementCause {
    /// Type of likely cause
    pub cause_type: CauseType,
    /// Description
    pub description: String,
    /// Confidence in this cause (0-100)
    pub confidence: f64,
    /// Supporting evidence
    pub evidence: Vec<String>,
}

/// Overall assessment of performance state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallAssessment {
    /// Overall performance health score (0-100)
    pub health_score: f64,
    /// Performance status
    pub status: PerformanceStatus,
    /// Key findings
    pub key_findings: Vec<String>,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Confidence in assessment
    pub confidence: f64,
}

/// Overall performance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceStatus {
    /// Excellent performance (all metrics improving or stable)
    Excellent,      // All metrics improving or stable
    /// Good performance (minor issues, mostly stable)
    Good,           // Minor issues, mostly stable
    /// Concerning performance (some notable regressions)
    Concerning,     // Some notable regressions
    /// Poor performance (multiple significant regressions)
    Poor,           // Multiple significant regressions
    /// Critical performance (severe widespread regressions)
    Critical,       // Severe widespread regressions
}

/// Risk level for performance issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk level
    Low,
    /// Medium risk level
    Medium,
    /// High risk level
    High,
    /// Critical risk level
    Critical,
}

/// Action recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecommendation {
    /// Priority level (1-10, 10 = highest)
    pub priority: u8,
    /// Recommended action
    pub action: RecommendedAction,
    /// Description
    pub description: String,
    /// Expected impact
    pub expected_impact: String,
    /// Effort required
    pub effort_level: EffortLevel,
    /// Timeline for action
    pub timeline: String,
}

/// Types of recommended actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendedAction {
    /// Investigate the performance issue
    Investigate,
    /// Rollback recent changes
    Rollback,
    /// Optimize code performance
    OptimizeCode,
    /// Update performance baseline
    UpdateBaseline,
    /// Increase monitoring frequency
    IncreaseMonitoring,
    /// Change test methodology
    ChangeTestMethod,
    /// No action required
    NoAction,
}

/// Effort level for implementing recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    /// Low effort level (< 1 day)
    Low,      // < 1 day
    /// Medium effort level (1-5 days)
    Medium,   // 1-5 days
    /// High effort level (> 5 days)
    High,     // > 5 days
}
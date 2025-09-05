//! Warning system for program analysis.

use super::analysis_types::{WarningSeverity, WarningType};
use crate::diagnostics::Span;

/// Analysis warning.
#[derive(Debug, Clone)]
pub struct AnalysisWarning {
    /// Warning type
    pub warning_type: WarningType,
    /// Warning message
    pub message: String,
    /// Location
    pub location: Option<Span>,
    /// Severity
    pub severity: WarningSeverity,
}

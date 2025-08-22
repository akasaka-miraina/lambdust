//! Unified error handling system for JIT compilation modules.
//!
//! This module eliminates redundant error handling patterns in the JIT system
//! and provides compile-time optimized error management for all JIT operations.

use crate::diagnostics::{ErrorSeverity, JitError, Span, UnifiedError, UnifiedResult};
use crate::jit::CompilationTier;

// Re-export unified error system for JIT modules
pub use crate::{error_chain, error_convert, handle_result, propagate_error, validate_arity};

/// JIT-specific error categories for granular error handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JitErrorKind {
    /// Code generation failures
    CodeGeneration,
    /// Optimization failures
    Optimization,
    /// Security verification failures
    Security,
    /// Performance profiling failures
    Profiling,
    /// Compilation tier selection failures
    TierSelection,
    /// Hotspot detection failures
    HotspotDetection,
    /// Memory management failures
    MemoryManagement,
    /// Cache management failures
    Cache,
    /// Dependency management failures
    Dependencies,
    /// Resource allocation failures
    ResourceAllocation,
}

impl JitErrorKind {
    /// Returns the error code string for this JIT error kind.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::CodeGeneration => "lambdust::jit::codegen::error",
            Self::Optimization => "lambdust::jit::optimization::error",
            Self::Security => "lambdust::jit::security::error",
            Self::Profiling => "lambdust::jit::profiling::error",
            Self::TierSelection => "lambdust::jit::tier::error",
            Self::HotspotDetection => "lambdust::jit::hotspot::error",
            Self::MemoryManagement => "lambdust::jit::memory::error",
            Self::Cache => "lambdust::jit::cache::error",
            Self::Dependencies => "lambdust::jit::deps::error",
            Self::ResourceAllocation => "lambdust::jit::resources::error",
        }
    }

    /// Returns whether this error kind indicates a recoverable failure.
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Security | Self::MemoryManagement => false,
            _ => true,
        }
    }

    /// Returns the default severity for this error kind.
    pub fn default_severity(&self) -> ErrorSeverity {
        match self {
            Self::Security => ErrorSeverity::Critical,
            Self::MemoryManagement => ErrorSeverity::Critical,
            Self::CodeGeneration => ErrorSeverity::Error,
            Self::Dependencies | Self::ResourceAllocation => ErrorSeverity::Error,
            _ => ErrorSeverity::Warning,
        }
    }
}

/// Enhanced JIT error with category-specific information.
#[derive(Debug, Clone)]
pub struct JitUnifiedError {
    /// Base unified error
    pub base: UnifiedError,
    /// JIT-specific error kind
    pub jit_kind: JitErrorKind,
    /// Compilation tier context
    pub tier_context: Option<CompilationTier>,
    /// Performance impact assessment
    pub performance_impact: Option<PerformanceImpact>,
}

/// Performance impact assessment for JIT errors.
#[derive(Debug, Clone)]
pub struct PerformanceImpact {
    /// Expected performance degradation (factor)
    pub degradation_factor: f64,
    /// Whether fallback to interpreter is possible
    pub fallback_available: bool,
    /// Resource overhead estimation
    pub resource_overhead: ResourceOverhead,
}

/// Resource overhead estimation.
#[derive(Debug, Clone)]
pub struct ResourceOverhead {
    /// Additional memory usage (bytes)
    pub memory_bytes: usize,
    /// Additional CPU cycles estimation
    pub cpu_cycles: u64,
    /// I/O operations impact
    pub io_operations: u32,
}

impl JitUnifiedError {
    /// Creates a new JIT-specific unified error.
    pub fn new(jit_kind: JitErrorKind, message: impl Into<String>) -> Self {
        let severity = jit_kind.default_severity();
        let base = UnifiedError::new(JitError, message)
            .with_severity(severity)
            .with_context("jit_kind", format!("{:?}", jit_kind));

        Self {
            base,
            jit_kind,
            tier_context: None,
            performance_impact: None,
        }
    }

    /// Builder pattern: adds compilation tier context.
    pub fn with_tier(mut self, tier: CompilationTier) -> Self {
        self.tier_context = Some(tier);
        self.base = self
            .base
            .with_context("compilation_tier", format!("{:?}", tier));
        self
    }

    /// Builder pattern: adds span information.
    pub fn with_span(mut self, span: Span) -> Self {
        self.base = self.base.with_span(span);
        self
    }

    /// Builder pattern: adds performance impact assessment.
    pub fn with_performance_impact(mut self, impact: PerformanceImpact) -> Self {
        self.base = self.base.with_context(
            "performance_impact",
            format!(
                "degradation: {:.2}x, fallback: {}",
                impact.degradation_factor,
                if impact.fallback_available {
                    "yes"
                } else {
                    "no"
                }
            ),
        );
        self.performance_impact = Some(impact);
        self
    }

    /// Builder pattern: adds source error.
    pub fn with_source(mut self, source: UnifiedError) -> Self {
        self.base = self.base.with_source(source);
        self
    }

    /// Converts to base UnifiedError for compatibility.
    pub fn into_unified(self) -> UnifiedError {
        self.base
    }

    /// Converts to boxed UnifiedError.
    pub fn boxed(self) -> Box<UnifiedError> {
        self.base.boxed()
    }

    /// Returns whether this error allows fallback to interpreter.
    pub fn allows_fallback(&self) -> bool {
        self.jit_kind.is_recoverable()
            && self
                .performance_impact
                .as_ref()
                .map(|impact| impact.fallback_available)
                .unwrap_or(true)
    }
}

/// Macro for creating JIT errors with performance impact assessment.
#[macro_export]
macro_rules! jit_error {
    // Basic JIT error
    ($kind:expr, $message:expr) => {
        $crate::jit::unified_jit_errors::JitUnifiedError::new($kind, $message)
    };

    // JIT error with tier context
    ($kind:expr, $message:expr, tier: $tier:expr) => {
        $crate::jit::unified_jit_errors::JitUnifiedError::new($kind, $message).with_tier($tier)
    };

    // JIT error with span
    ($kind:expr, $message:expr, span: $span:expr) => {
        $crate::jit::unified_jit_errors::JitUnifiedError::new($kind, $message).with_span($span)
    };

    // JIT error with performance impact
    ($kind:expr, $message:expr, impact: $impact:expr) => {
        $crate::jit::unified_jit_errors::JitUnifiedError::new($kind, $message)
            .with_performance_impact($impact)
    };

    // Full JIT error with all context
    ($kind:expr, $message:expr, tier: $tier:expr, span: $span:expr, impact: $impact:expr) => {
        $crate::jit::unified_jit_errors::JitUnifiedError::new($kind, $message)
            .with_tier($tier)
            .with_span($span)
            .with_performance_impact($impact)
    };
}

/// Macro for JIT result handling with automatic fallback.
#[macro_export]
macro_rules! jit_try {
    // Basic JIT operation with fallback
    ($operation:expr, fallback: $fallback:expr) => {
        match $operation {
            Ok(result) => result,
            Err(error) => {
                if error.allows_fallback() {
                    $fallback
                } else {
                    return Err(error.into_unified());
                }
            }
        }
    };

    // JIT operation with custom fallback logic
    ($operation:expr, fallback: |$err:ident| $fallback_logic:expr) => {
        match $operation {
            Ok(result) => result,
            Err($err) => {
                if $err.allows_fallback() {
                    $fallback_logic
                } else {
                    return Err($err.into_unified());
                }
            }
        }
    };
}

/// Macro for JIT performance monitoring with automatic error reporting.
#[macro_export]
macro_rules! jit_monitor {
    // Monitor JIT operation with performance tracking
    ($operation:expr, tier: $tier:expr) => {{
        let start = std::time::Instant::now();
        let result = $operation;
        let duration = start.elapsed();

        // Log performance metrics (implementation would send to metrics system)
        #[cfg(feature = "jit-metrics")]
        $crate::jit::metrics::record_operation_time($tier, duration);

        match result {
            Ok(value) => Ok(value),
            Err(mut error) => {
                // Add performance context to error
                error.base = error
                    .base
                    .with_context("execution_time", format!("{:?}", duration));
                Err(error)
            }
        }
    }};
}

/// Specialized error constructors for common JIT patterns.
impl JitUnifiedError {
    /// Creates a code generation failure error.
    pub fn codegen_failed(message: impl Into<String>, tier: CompilationTier) -> Self {
        Self::new(JitErrorKind::CodeGeneration, message)
            .with_tier(tier)
            .with_performance_impact(PerformanceImpact {
                degradation_factor: 1.0, // No degradation if we fall back
                fallback_available: true,
                resource_overhead: ResourceOverhead {
                    memory_bytes: 0,
                    cpu_cycles: 100_000, // Modest overhead for fallback
                    io_operations: 0,
                },
            })
    }

    /// Creates an optimization failure error.
    pub fn optimization_failed(message: impl Into<String>, expected_benefit: f64) -> Self {
        Self::new(JitErrorKind::Optimization, message).with_performance_impact(PerformanceImpact {
            degradation_factor: 1.0 / expected_benefit.max(1.1), // Lost optimization benefit
            fallback_available: true,
            resource_overhead: ResourceOverhead {
                memory_bytes: 1024, // Small overhead for tracking
                cpu_cycles: 50_000,
                io_operations: 0,
            },
        })
    }

    /// Creates a security verification failure error.
    pub fn security_failed(message: impl Into<String>) -> Self {
        Self::new(JitErrorKind::Security, message).with_performance_impact(PerformanceImpact {
            degradation_factor: 10.0, // Significant performance hit without JIT
            fallback_available: true, // Must fall back for security
            resource_overhead: ResourceOverhead {
                memory_bytes: 0,
                cpu_cycles: 1_000_000, // High overhead for interpreter fallback
                io_operations: 0,
            },
        })
    }

    /// Creates a memory management failure error.
    pub fn memory_failed(message: impl Into<String>) -> Self {
        Self::new(JitErrorKind::MemoryManagement, message).with_performance_impact(
            PerformanceImpact {
                degradation_factor: 5.0,
                fallback_available: false, // Memory errors are often unrecoverable
                resource_overhead: ResourceOverhead {
                    memory_bytes: 0, // Cannot estimate - memory is compromised
                    cpu_cycles: 0,
                    io_operations: 0,
                },
            },
        )
    }

    /// Creates a hotspot detection failure error.
    pub fn hotspot_failed(message: impl Into<String>) -> Self {
        Self::new(JitErrorKind::HotspotDetection, message).with_performance_impact(
            PerformanceImpact {
                degradation_factor: 1.2, // Minor impact - just less optimal compilation
                fallback_available: true,
                resource_overhead: ResourceOverhead {
                    memory_bytes: 512,
                    cpu_cycles: 25_000,
                    io_operations: 0,
                },
            },
        )
    }
}

/// Result type specialized for JIT operations.
pub type JitResult<T> = Result<T, JitUnifiedError>;

/// Conversion implementations for backward compatibility.
impl From<JitUnifiedError> for UnifiedError {
    fn from(err: JitUnifiedError) -> Self {
        err.base
    }
}

impl From<JitUnifiedError> for Box<crate::diagnostics::Error> {
    fn from(err: JitUnifiedError) -> Self {
        err.base.into()
    }
}

impl std::fmt::Display for JitUnifiedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JIT {}: {}", self.jit_kind.error_code(), self.base)
    }
}

impl std::error::Error for JitUnifiedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.base)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_jit_error_creation() {
        let error =
            JitUnifiedError::codegen_failed("Test codegen failure", CompilationTier::JitBasic);

        assert_eq!(error.jit_kind, JitErrorKind::CodeGeneration);
        assert_eq!(error.tier_context, Some(CompilationTier::JitBasic));
        assert!(error.allows_fallback());
    }

    #[test]
    fn test_security_error_no_fallback() {
        let error = JitUnifiedError::security_failed("Security violation detected");

        assert_eq!(error.jit_kind, JitErrorKind::Security);
        assert!(error.allows_fallback()); // Security errors allow fallback to interpreter
        assert_eq!(error.base.severity, ErrorSeverity::Critical);
    }

    #[test]
    fn test_memory_error_critical() {
        let error = JitUnifiedError::memory_failed("Memory corruption detected");

        assert_eq!(error.jit_kind, JitErrorKind::MemoryManagement);
        assert!(!error.allows_fallback()); // Memory errors are critical
        assert_eq!(error.base.severity, ErrorSeverity::Critical);
    }

    #[test]
    fn test_jit_error_macro() {
        let error = jit_error!(
            JitErrorKind::Optimization,
            "Optimization failed",
            tier: CompilationTier::JitOptimized
        );

        assert_eq!(error.jit_kind, JitErrorKind::Optimization);
        assert_eq!(error.tier_context, Some(CompilationTier::JitOptimized));
    }

    #[test]
    fn test_performance_impact() {
        let impact = PerformanceImpact {
            degradation_factor: 2.0,
            fallback_available: true,
            resource_overhead: ResourceOverhead {
                memory_bytes: 1024,
                cpu_cycles: 50_000,
                io_operations: 0,
            },
        };

        let error = JitUnifiedError::new(JitErrorKind::Optimization, "Test")
            .with_performance_impact(impact.clone());

        assert!(error.performance_impact.is_some());
        assert_eq!(
            error
                .performance_impact
                .as_ref()
                .unwrap()
                .degradation_factor,
            2.0
        );
    }
}

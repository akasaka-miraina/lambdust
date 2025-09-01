//! SRFI-0 Feature Detection and Evaluation System
//!
//! This module provides compile-time feature detection for cond-expand expressions.
//! Features are organized into categories:
//! - Core features: basic Lambdust capabilities (lambdust, r7rs, etc.)
//! - SRFI features: Scheme Request for Implementation support
//! - Platform features: OS/architecture-specific capabilities  
//! - Capability features: runtime optimizations (simd, nan-boxing, etc.)
//! - Library features: available library modules

pub mod cache;
pub mod detector;
pub mod evaluator;
pub mod integration;
pub mod registry;
pub mod optimization_features;

pub use cache::{CacheConfig, FeatureCache};
pub use detector::{CapabilityInfo, FeatureDetector, PlatformInfo};
pub use evaluator::{EvaluationResult, FeatureEvaluationError, FeatureEvaluator};
pub use integration::{CondExpandEvaluator, MacroIntegration};
pub use registry::{FeatureCategory, FeatureInfo, FeatureRegistry};

use crate::ast::FeatureRequirement;
use serde::{Deserialize, Serialize};

/// Global feature detection context
pub struct FeatureContext {
    registry: FeatureRegistry,
    cache: FeatureCache,
}

impl FeatureContext {
    /// Creates a new feature context with default configuration
    pub fn new() -> Self {
        let registry = FeatureRegistry::default();
        let cache_config = CacheConfig::default();
        let cache = FeatureCache::new(cache_config);

        Self { registry, cache }
    }

    /// Creates a feature context with custom configuration
    pub fn with_config(cache_config: CacheConfig) -> Self {
        let registry = FeatureRegistry::default();
        let cache = FeatureCache::new(cache_config);

        Self { registry, cache }
    }

    /// Evaluates a feature requirement
    pub fn evaluate_feature(&self, requirement: &FeatureRequirement) -> EvaluationResult {
        let evaluator = FeatureEvaluator::new(&self.registry, &self.cache);
        evaluator.evaluate(requirement)
    }

    /// Checks if a simple feature exists
    pub fn has_feature(&self, feature: &str) -> bool {
        self.registry.has_feature(feature)
    }

    /// Checks if a library is available
    pub fn has_library(&self, components: &[String]) -> bool {
        self.registry.has_library(components)
    }

    /// Gets registry for inspection
    pub fn registry(&self) -> &FeatureRegistry {
        &self.registry
    }

    /// Gets cache statistics
    pub fn cache_stats(&self) -> cache::CacheStats {
        self.cache.stats()
    }

    /// Clears evaluation cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for FeatureContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Result type for feature operations
pub type FeatureResult<T> = std::result::Result<T, FeatureError>;

/// Unified error type for feature system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureError {
    /// Unknown feature requested
    UnknownFeature(String),
    /// Unknown library requested  
    UnknownLibrary(Vec<String>),
    /// Circular dependency detected
    CircularDependency(Vec<String>),
    /// Evaluation failed
    EvaluationFailed(String),
    /// Cache error
    CacheError(String),
    /// Platform detection error
    PlatformError(String),
}

impl std::fmt::Display for FeatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureError::UnknownFeature(feature) => {
                write!(f, "Unknown feature: {}", feature)
            }
            FeatureError::UnknownLibrary(components) => {
                write!(f, "Unknown library: ({})", components.join(" "))
            }
            FeatureError::CircularDependency(path) => {
                write!(f, "Circular dependency: {}", path.join(" -> "))
            }
            FeatureError::EvaluationFailed(msg) => {
                write!(f, "Feature evaluation failed: {}", msg)
            }
            FeatureError::CacheError(msg) => {
                write!(f, "Cache error: {}", msg)
            }
            FeatureError::PlatformError(msg) => {
                write!(f, "Platform detection error: {}", msg)
            }
        }
    }
}

impl std::error::Error for FeatureError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_context_creation() {
        let context = FeatureContext::new();

        // Should have basic features
        assert!(context.has_feature("lambdust"));
        assert!(context.has_feature("r7rs-small"));

        // Should not have unknown features
        assert!(!context.has_feature("unknown-feature"));
    }

    #[test]
    fn test_simple_feature_evaluation() {
        let context = FeatureContext::new();

        let req = FeatureRequirement::Feature("lambdust".to_string());
        let result = context.evaluate_feature(&req);
        assert!(matches!(result, EvaluationResult::Success(true)));

        let req = FeatureRequirement::Feature("unknown".to_string());
        let result = context.evaluate_feature(&req);
        assert!(matches!(result, EvaluationResult::Success(false)));
    }

    #[test]
    fn test_library_evaluation() {
        let context = FeatureContext::new();

        let req = FeatureRequirement::Library(vec!["lambdust".to_string(), "core".to_string()]);
        let result = context.evaluate_feature(&req);
        assert!(matches!(result, EvaluationResult::Success(true)));

        let req = FeatureRequirement::Library(vec!["unknown".to_string(), "lib".to_string()]);
        let result = context.evaluate_feature(&req);
        assert!(matches!(result, EvaluationResult::Success(false)));
    }
}

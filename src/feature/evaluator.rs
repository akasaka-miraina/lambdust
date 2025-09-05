//! Feature Requirement Evaluator
//!
//! High-performance evaluation engine for FeatureRequirement expressions.
//! Implements short-circuit evaluation and caching for optimal performance.

use super::{FeatureCache, FeatureError, FeatureRegistry, FeatureResult};
use crate::ast::FeatureRequirement;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Result of feature evaluation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationResult {
    /// Feature evaluation succeeded with boolean result
    Success(bool),
    /// Feature evaluation failed with error
    Error(FeatureError),
}

impl EvaluationResult {
    /// Converts to a boolean, treating errors as false
    pub fn to_bool(&self) -> bool {
        match self {
            EvaluationResult::Success(b) => *b,
            EvaluationResult::Error(_) => false,
        }
    }

    /// Checks if the evaluation was successful
    pub fn is_success(&self) -> bool {
        matches!(self, EvaluationResult::Success(_))
    }

    /// Checks if the evaluation resulted in an error
    pub fn is_error(&self) -> bool {
        matches!(self, EvaluationResult::Error(_))
    }

    /// Gets the error if evaluation failed
    pub fn error(&self) -> Option<&FeatureError> {
        match self {
            EvaluationResult::Error(e) => Some(e),
            _ => None,
        }
    }
}

/// Feature requirement evaluation error
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureEvaluationError {
    /// Unknown feature requested
    UnknownFeature(String),
    /// Unknown library requested
    UnknownLibrary(Vec<String>),
    /// Circular dependency detected in evaluation
    CircularDependency(Vec<String>),
    /// Maximum evaluation depth exceeded
    MaxDepthExceeded(usize),
    /// Cache operation failed
    CacheError(String),
}

impl std::fmt::Display for FeatureEvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeatureEvaluationError::UnknownFeature(name) => {
                write!(f, "Unknown feature: {}", name)
            }
            FeatureEvaluationError::UnknownLibrary(components) => {
                write!(f, "Unknown library: ({})", components.join(" "))
            }
            FeatureEvaluationError::CircularDependency(path) => {
                write!(
                    f,
                    "Circular dependency in feature evaluation: {}",
                    path.join(" -> ")
                )
            }
            FeatureEvaluationError::MaxDepthExceeded(depth) => {
                write!(f, "Maximum evaluation depth exceeded: {}", depth)
            }
            FeatureEvaluationError::CacheError(msg) => {
                write!(f, "Cache error during evaluation: {}", msg)
            }
        }
    }
}

impl std::error::Error for FeatureEvaluationError {}

impl From<FeatureEvaluationError> for FeatureError {
    fn from(err: FeatureEvaluationError) -> Self {
        match err {
            FeatureEvaluationError::UnknownFeature(name) => FeatureError::UnknownFeature(name),
            FeatureEvaluationError::UnknownLibrary(components) => {
                FeatureError::UnknownLibrary(components)
            }
            FeatureEvaluationError::CircularDependency(path) => {
                FeatureError::CircularDependency(path)
            }
            FeatureEvaluationError::MaxDepthExceeded(_) => {
                FeatureError::EvaluationFailed("Maximum evaluation depth exceeded".to_string())
            }
            FeatureEvaluationError::CacheError(msg) => FeatureError::CacheError(msg),
        }
    }
}

/// Configuration for feature evaluation
#[derive(Debug, Clone)]
pub struct EvaluatorConfig {
    /// Maximum recursion depth to prevent infinite loops
    pub max_depth: usize,
    /// Whether to use caching for evaluation results
    pub use_cache: bool,
    /// Whether to report unknown features as errors (vs false)
    pub strict_mode: bool,
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self {
            max_depth: 100,
            use_cache: true,
            strict_mode: false,
        }
    }
}

/// Feature requirement evaluator with caching and optimization
pub struct FeatureEvaluator<'a> {
    registry: &'a FeatureRegistry,
    cache: &'a FeatureCache,
    config: EvaluatorConfig,
}

impl<'a> FeatureEvaluator<'a> {
    /// Creates a new evaluator with default configuration
    pub fn new(registry: &'a FeatureRegistry, cache: &'a FeatureCache) -> Self {
        Self {
            registry,
            cache,
            config: EvaluatorConfig::default(),
        }
    }

    /// Creates a new evaluator with custom configuration
    pub fn with_config(
        registry: &'a FeatureRegistry,
        cache: &'a FeatureCache,
        config: EvaluatorConfig,
    ) -> Self {
        Self {
            registry,
            cache,
            config,
        }
    }

    /// Evaluates a feature requirement
    pub fn evaluate(&self, requirement: &FeatureRequirement) -> EvaluationResult {
        let mut visited = HashSet::new();
        match self.evaluate_recursive(requirement, &mut visited, 0) {
            Ok(result) => EvaluationResult::Success(result),
            Err(error) => EvaluationResult::Error(error.into()),
        }
    }

    /// Recursive evaluation with cycle detection and depth limiting
    fn evaluate_recursive(
        &self,
        requirement: &FeatureRequirement,
        visited: &mut HashSet<FeatureRequirement>,
        depth: usize,
    ) -> Result<bool, FeatureEvaluationError> {
        // Check depth limit
        if depth >= self.config.max_depth {
            return Err(FeatureEvaluationError::MaxDepthExceeded(depth));
        }

        // Check for cycles
        if visited.contains(requirement) {
            let cycle = visited.iter().map(|r| format!("{}", r)).collect();
            return Err(FeatureEvaluationError::CircularDependency(cycle));
        }

        // Check cache first
        if self.config.use_cache {
            if let Some(cached) = self.cache.get(requirement) {
                return Ok(cached);
            }
        }

        // Mark as visiting
        visited.insert(requirement.clone());

        // Evaluate based on requirement type
        let result = match requirement {
            FeatureRequirement::Feature(name) => self.evaluate_feature(name),
            FeatureRequirement::Library(components) => self.evaluate_library(components),
            FeatureRequirement::And(requirements) => {
                self.evaluate_and(requirements, visited, depth + 1)
            }
            FeatureRequirement::Or(requirements) => {
                self.evaluate_or(requirements, visited, depth + 1)
            }
            FeatureRequirement::Not(requirement) => {
                self.evaluate_not(requirement, visited, depth + 1)
            }
        };

        // Remove from visiting set
        visited.remove(requirement);

        // Cache successful results
        if let Ok(value) = &result {
            if self.config.use_cache {
                self.cache.insert(requirement.clone(), *value);
            }
        }

        result
    }

    /// Evaluates a simple feature name
    fn evaluate_feature(&self, name: &str) -> Result<bool, FeatureEvaluationError> {
        let has_feature = self.registry.has_feature(name);

        if !has_feature && self.config.strict_mode {
            Err(FeatureEvaluationError::UnknownFeature(name.to_string()))
        } else {
            Ok(has_feature)
        }
    }

    /// Evaluates a library identifier
    fn evaluate_library(&self, components: &[String]) -> Result<bool, FeatureEvaluationError> {
        let has_library = self.registry.has_library(components);

        if !has_library && self.config.strict_mode {
            Err(FeatureEvaluationError::UnknownLibrary(components.to_vec()))
        } else {
            Ok(has_library)
        }
    }

    /// Evaluates AND requirement with short-circuit evaluation
    fn evaluate_and(
        &self,
        requirements: &[FeatureRequirement],
        visited: &mut HashSet<FeatureRequirement>,
        depth: usize,
    ) -> Result<bool, FeatureEvaluationError> {
        // Empty AND is true
        if requirements.is_empty() {
            return Ok(true);
        }

        // Short-circuit: if any is false, result is false
        for requirement in requirements {
            if !self.evaluate_recursive(requirement, visited, depth)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Evaluates OR requirement with short-circuit evaluation
    fn evaluate_or(
        &self,
        requirements: &[FeatureRequirement],
        visited: &mut HashSet<FeatureRequirement>,
        depth: usize,
    ) -> Result<bool, FeatureEvaluationError> {
        // Empty OR is false
        if requirements.is_empty() {
            return Ok(false);
        }

        // Short-circuit: if any is true, result is true
        for requirement in requirements {
            if self.evaluate_recursive(requirement, visited, depth)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Evaluates NOT requirement
    fn evaluate_not(
        &self,
        requirement: &FeatureRequirement,
        visited: &mut HashSet<FeatureRequirement>,
        depth: usize,
    ) -> Result<bool, FeatureEvaluationError> {
        let result = self.evaluate_recursive(requirement, visited, depth)?;
        Ok(!result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{CacheConfig, FeatureCache, FeatureRegistry};

    #[test]
    fn test_simple_feature_evaluation() {
        let registry = FeatureRegistry::default();
        let cache_config = CacheConfig::default();
        let cache = FeatureCache::new(cache_config);
        let evaluator = FeatureEvaluator::new(&registry, &cache);

        // Test existing feature
        let req = FeatureRequirement::Feature("lambdust".to_string());
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(true));

        // Test non-existing feature
        let req = FeatureRequirement::Feature("unknown-feature".to_string());
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(false));
    }

    #[test]
    fn test_library_evaluation() {
        let registry = FeatureRegistry::default();
        let cache_config = CacheConfig::default();
        let cache = FeatureCache::new(cache_config);
        let evaluator = FeatureEvaluator::new(&registry, &cache);

        // Test existing library
        let req = FeatureRequirement::Library(vec!["lambdust".to_string(), "core".to_string()]);
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(true));

        // Test non-existing library
        let req = FeatureRequirement::Library(vec!["unknown".to_string(), "lib".to_string()]);
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(false));
    }

    #[test]
    fn test_and_evaluation() {
        let registry = FeatureRegistry::default();
        let cache_config = CacheConfig::default();
        let cache = FeatureCache::new(cache_config);
        let evaluator = FeatureEvaluator::new(&registry, &cache);

        // All true
        let req = FeatureRequirement::And(vec![
            FeatureRequirement::Feature("lambdust".to_string()),
            FeatureRequirement::Feature("r7rs-small".to_string()),
        ]);
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(true));

        // One false
        let req = FeatureRequirement::And(vec![
            FeatureRequirement::Feature("lambdust".to_string()),
            FeatureRequirement::Feature("unknown".to_string()),
        ]);
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(false));

        // Empty AND
        let req = FeatureRequirement::And(vec![]);
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(true));
    }

    #[test]
    fn test_or_evaluation() {
        let registry = FeatureRegistry::default();
        let cache_config = CacheConfig::default();
        let cache = FeatureCache::new(cache_config);
        let evaluator = FeatureEvaluator::new(&registry, &cache);

        // One true
        let req = FeatureRequirement::Or(vec![
            FeatureRequirement::Feature("lambdust".to_string()),
            FeatureRequirement::Feature("unknown".to_string()),
        ]);
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(true));

        // All false
        let req = FeatureRequirement::Or(vec![
            FeatureRequirement::Feature("unknown1".to_string()),
            FeatureRequirement::Feature("unknown2".to_string()),
        ]);
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(false));

        // Empty OR
        let req = FeatureRequirement::Or(vec![]);
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(false));
    }

    #[test]
    fn test_not_evaluation() {
        let registry = FeatureRegistry::default();
        let cache_config = CacheConfig::default();
        let cache = FeatureCache::new(cache_config);
        let evaluator = FeatureEvaluator::new(&registry, &cache);

        // NOT true
        let req = FeatureRequirement::Not(Box::new(FeatureRequirement::Feature(
            "lambdust".to_string(),
        )));
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(false));

        // NOT false
        let req =
            FeatureRequirement::Not(Box::new(FeatureRequirement::Feature("unknown".to_string())));
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(true));
    }

    #[test]
    fn test_complex_evaluation() {
        let registry = FeatureRegistry::default();
        let cache_config = CacheConfig::default();
        let cache = FeatureCache::new(cache_config);
        let evaluator = FeatureEvaluator::new(&registry, &cache);

        // (and lambdust (not debug))
        let req = FeatureRequirement::And(vec![
            FeatureRequirement::Feature("lambdust".to_string()),
            FeatureRequirement::Not(Box::new(FeatureRequirement::Feature("debug".to_string()))),
        ]);
        let result = evaluator.evaluate(&req);
        assert_eq!(result, EvaluationResult::Success(true));

        // (or (and simd x86-64) fallback)
        let req = FeatureRequirement::Or(vec![
            FeatureRequirement::And(vec![
                FeatureRequirement::Feature("simd".to_string()),
                FeatureRequirement::Feature("x86-64".to_string()),
            ]),
            FeatureRequirement::Feature("fallback".to_string()),
        ]);
        let result = evaluator.evaluate(&req);
        // Depends on whether simd/x86-64/fallback features are available
        assert!(matches!(result, EvaluationResult::Success(_)));
    }

    #[test]
    fn test_strict_mode() {
        let registry = FeatureRegistry::default();
        let cache_config = CacheConfig::default();
        let cache = FeatureCache::new(cache_config);
        let config = EvaluatorConfig {
            strict_mode: true,
            ..EvaluatorConfig::default()
        };
        let evaluator = FeatureEvaluator::with_config(&registry, &cache, config);

        // Unknown feature in strict mode should error
        let req = FeatureRequirement::Feature("unknown-feature".to_string());
        let result = evaluator.evaluate(&req);
        assert!(matches!(result, EvaluationResult::Error(_)));
    }

    #[test]
    fn test_evaluation_result_helpers() {
        let success = EvaluationResult::Success(true);
        let error = EvaluationResult::Error(FeatureError::UnknownFeature("test".to_string()));

        assert!(success.to_bool());
        assert!(!error.to_bool());

        assert!(success.is_success());
        assert!(!success.is_error());
        assert!(success.error().is_none());

        assert!(!error.is_success());
        assert!(error.is_error());
        assert!(error.error().is_some());
    }
}

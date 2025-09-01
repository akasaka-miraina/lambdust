//! Integration with Macro System and Evaluator
//!
//! Provides seamless integration between the feature system and Lambdust's
//! macro expansion and evaluation systems for cond-expand support.

use super::{EvaluationResult, FeatureCategory, FeatureContext, FeatureError};
use crate::ast::{CondExpandClause, Expr, FeatureRequirement};
use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

/// Result of cond-expand clause selection
#[derive(Debug, Clone)]
pub enum CondExpandResult {
    /// A clause was matched and its body should be expanded
    Matched(Vec<Spanned<Expr>>),
    /// No clause matched and there was no else clause
    NoMatch,
    /// Evaluation failed with an error
    Error(FeatureError),
}

/// High-level cond-expand evaluator
pub struct CondExpandEvaluator {
    feature_context: FeatureContext,
}

impl CondExpandEvaluator {
    /// Creates a new cond-expand evaluator
    pub fn new() -> Self {
        Self {
            feature_context: FeatureContext::default(),
        }
    }

    /// Creates a new cond-expand evaluator with custom feature context
    pub fn with_context(feature_context: FeatureContext) -> Self {
        Self { feature_context }
    }

    /// Evaluates a cond-expand expression and returns the matching clause
    pub fn evaluate_cond_expand(
        &self,
        clauses: &[CondExpandClause],
        else_clause: &Option<Vec<Spanned<Expr>>>,
    ) -> CondExpandResult {
        // Evaluate each clause in order
        for clause in clauses {
            match self
                .feature_context
                .evaluate_feature(&clause.feature_requirement)
            {
                EvaluationResult::Success(true) => {
                    return CondExpandResult::Matched(clause.body.clone());
                }
                EvaluationResult::Success(false) => {
                    // Continue to next clause
                    continue;
                }
                EvaluationResult::Error(error) => {
                    return CondExpandResult::Error(error);
                }
            }
        }

        // No clause matched, check for else clause
        if let Some(else_body) = else_clause {
            CondExpandResult::Matched(else_body.clone())
        } else {
            CondExpandResult::NoMatch
        }
    }

    /// Evaluates a single feature requirement
    pub fn evaluate_feature_requirement(
        &self,
        requirement: &FeatureRequirement,
    ) -> EvaluationResult {
        self.feature_context.evaluate_feature(requirement)
    }

    /// Gets the underlying feature context for inspection
    pub fn feature_context(&self) -> &FeatureContext {
        &self.feature_context
    }

    /// Gets mutable access to the feature context
    pub fn feature_context_mut(&mut self) -> &mut FeatureContext {
        &mut self.feature_context
    }
}

impl Default for CondExpandEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Integration trait for macro expansion systems
pub trait MacroIntegration {
    /// Called during macro expansion to handle cond-expand forms
    fn expand_cond_expand(
        &self,
        clauses: &[CondExpandClause],
        else_clause: &Option<Vec<Spanned<Expr>>>,
    ) -> Result<Vec<Spanned<Expr>>, FeatureError>;

    /// Checks if a feature is available (for use in other macros)
    fn feature_available(&self, feature: &str) -> bool;

    /// Checks if a library is available (for use in import checking)
    fn library_available(&self, components: &[String]) -> bool;
}

impl MacroIntegration for CondExpandEvaluator {
    fn expand_cond_expand(
        &self,
        clauses: &[CondExpandClause],
        else_clause: &Option<Vec<Spanned<Expr>>>,
    ) -> Result<Vec<Spanned<Expr>>, FeatureError> {
        match self.evaluate_cond_expand(clauses, else_clause) {
            CondExpandResult::Matched(body) => Ok(body),
            CondExpandResult::NoMatch => Ok(vec![]), // Return empty expansion
            CondExpandResult::Error(error) => Err(error),
        }
    }

    fn feature_available(&self, feature: &str) -> bool {
        self.feature_context.has_feature(feature)
    }

    fn library_available(&self, components: &[String]) -> bool {
        self.feature_context.has_library(components)
    }
}

/// Compile-time feature evaluation for static analysis
pub struct CompileTimeFeatures {
    evaluator: CondExpandEvaluator,
}

impl CompileTimeFeatures {
    /// Creates a new compile-time feature evaluator
    pub fn new() -> Self {
        Self {
            evaluator: CondExpandEvaluator::new(),
        }
    }

    /// Performs static analysis of cond-expand expressions in an AST
    pub fn analyze_cond_expand_usage(&self, expr: &Expr) -> CondExpandAnalysis {
        let mut analysis = CondExpandAnalysis::default();
        self.analyze_expr_recursive(expr, &mut analysis);
        analysis
    }

    /// Recursively analyzes expressions for cond-expand usage
    fn analyze_expr_recursive(&self, expr: &Expr, analysis: &mut CondExpandAnalysis) {
        match expr {
            Expr::CondExpand {
                clauses,
                else_clause,
            } => {
                analysis.total_cond_expands += 1;

                // Evaluate each clause
                for clause in clauses {
                    analysis.total_clauses += 1;

                    let requirement = &clause.feature_requirement;
                    match self.evaluator.evaluate_feature_requirement(requirement) {
                        EvaluationResult::Success(true) => {
                            analysis.matched_clauses += 1;
                            if analysis.first_match.is_none() {
                                analysis.first_match = Some(requirement.clone());
                            }
                        }
                        EvaluationResult::Success(false) => {
                            analysis.unmatched_clauses += 1;
                        }
                        EvaluationResult::Error(error) => {
                            analysis
                                .evaluation_errors
                                .push((requirement.clone(), error));
                        }
                    }

                    // Analyze used features
                    self.extract_features_from_requirement(
                        requirement,
                        &mut analysis.used_features,
                    );
                }

                if else_clause.is_some() {
                    analysis.has_else_clauses += 1;
                }
            }

            // Recursively analyze other expression types
            Expr::Lambda { body, .. } => {
                for body_expr in body {
                    self.analyze_expr_recursive(&body_expr.inner, analysis);
                }
            }
            Expr::If {
                test,
                consequent,
                alternative,
            } => {
                self.analyze_expr_recursive(&test.inner, analysis);
                self.analyze_expr_recursive(&consequent.inner, analysis);
                if let Some(alt) = alternative {
                    self.analyze_expr_recursive(&alt.inner, analysis);
                }
            }
            Expr::Application { operator, operands } => {
                self.analyze_expr_recursive(&operator.inner, analysis);
                for operand in operands {
                    self.analyze_expr_recursive(&operand.inner, analysis);
                }
            }
            Expr::List(exprs) => {
                for expr in exprs {
                    self.analyze_expr_recursive(&expr.inner, analysis);
                }
            }

            // Other expressions don't contain nested expressions we care about
            _ => {}
        }
    }

    /// Extracts feature names from a requirement for analysis
    fn extract_features_from_requirement(
        &self,
        requirement: &FeatureRequirement,
        features: &mut Vec<String>,
    ) {
        match requirement {
            FeatureRequirement::Feature(name) => {
                features.push(name.clone());
            }
            FeatureRequirement::Library(components) => {
                features.push(format!("({})", components.join(" ")));
            }
            FeatureRequirement::And(reqs) | FeatureRequirement::Or(reqs) => {
                for req in reqs {
                    self.extract_features_from_requirement(req, features);
                }
            }
            FeatureRequirement::Not(req) => {
                self.extract_features_from_requirement(req, features);
            }
        }
    }
}

impl Default for CompileTimeFeatures {
    fn default() -> Self {
        Self::new()
    }
}

/// Analysis results for cond-expand usage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CondExpandAnalysis {
    /// Total number of cond-expand expressions found
    pub total_cond_expands: usize,
    /// Total number of clauses across all cond-expands
    pub total_clauses: usize,
    /// Number of clauses that would match current features
    pub matched_clauses: usize,
    /// Number of clauses that would not match
    pub unmatched_clauses: usize,
    /// Number of cond-expands with else clauses
    pub has_else_clauses: usize,
    /// First matching requirement found (for optimization)
    pub first_match: Option<FeatureRequirement>,
    /// All unique features referenced
    pub used_features: Vec<String>,
    /// Evaluation errors encountered
    pub evaluation_errors: Vec<(FeatureRequirement, FeatureError)>,
}

impl CondExpandAnalysis {
    /// Calculates the match rate as a percentage
    pub fn match_rate(&self) -> f64 {
        if self.total_clauses == 0 {
            0.0
        } else {
            (self.matched_clauses as f64 / self.total_clauses as f64) * 100.0
        }
    }

    /// Checks if any evaluation errors occurred
    pub fn has_errors(&self) -> bool {
        !self.evaluation_errors.is_empty()
    }

    /// Gets unique features used (deduplicated)
    pub fn unique_features(&self) -> Vec<String> {
        let mut features = self.used_features.clone();
        features.sort();
        features.dedup();
        features
    }
}

/// Helper functions for common integration scenarios
pub mod helpers {
    use super::*;

    /// Quick check if a simple feature is available
    pub fn has_feature(feature: &str) -> bool {
        let context = FeatureContext::default();
        context.has_feature(feature)
    }

    /// Quick check if a library is available
    pub fn has_library(components: &[String]) -> bool {
        let context = FeatureContext::default();
        context.has_library(components)
    }

    /// Evaluates a feature requirement string (for debugging/testing)
    pub fn evaluate_feature_string(requirement_str: &str) -> bool {
        // This would parse a feature requirement from string format
        // For now, just check as a simple feature name
        has_feature(requirement_str)
    }

    /// Gets all available core features
    pub fn available_core_features() -> Vec<String> {
        let context = FeatureContext::default();
        context
            .registry()
            .all_features()
            .filter(|info| matches!(info.category, FeatureCategory::Core))
            .map(|info| info.name.clone())
            .collect()
    }

    /// Gets all available SRFI features
    pub fn available_srfi_features() -> Vec<String> {
        let context = FeatureContext::default();
        context
            .registry()
            .all_features()
            .filter(|info| matches!(info.category, FeatureCategory::Srfi(_)))
            .map(|info| info.name.clone())
            .collect()
    }

    /// Creates a feature requirement for common patterns
    pub fn create_requirement_and(features: &[&str]) -> FeatureRequirement {
        let reqs: Vec<FeatureRequirement> = features
            .iter()
            .map(|&f| FeatureRequirement::Feature(f.to_string()))
            .collect();
        FeatureRequirement::And(reqs)
    }

    /// Creates a feature requirement for OR patterns
    pub fn create_requirement_or(features: &[&str]) -> FeatureRequirement {
        let reqs: Vec<FeatureRequirement> = features
            .iter()
            .map(|&f| FeatureRequirement::Feature(f.to_string()))
            .collect();
        FeatureRequirement::Or(reqs)
    }

    /// Creates a NOT requirement
    pub fn create_requirement_not(feature: &str) -> FeatureRequirement {
        FeatureRequirement::Not(Box::new(FeatureRequirement::Feature(feature.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    fn make_spanned<T>(value: T) -> Spanned<T> {
        Spanned {
            inner: value,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn test_cond_expand_evaluation() {
        let evaluator = CondExpandEvaluator::new();

        // Create a clause that should match
        let clause = CondExpandClause {
            feature_requirement: FeatureRequirement::Feature("lambdust".to_string()),
            body: vec![make_spanned(Expr::Literal(crate::ast::Literal::boolean(
                true,
            )))],
        };

        let result = evaluator.evaluate_cond_expand(&[clause], &None);

        match result {
            CondExpandResult::Matched(body) => {
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected matched result"),
        }
    }

    #[test]
    fn test_cond_expand_no_match() {
        let evaluator = CondExpandEvaluator::new();

        // Create a clause that should not match
        let clause = CondExpandClause {
            feature_requirement: FeatureRequirement::Feature("unknown-feature".to_string()),
            body: vec![make_spanned(Expr::Literal(crate::ast::Literal::boolean(
                true,
            )))],
        };

        let result = evaluator.evaluate_cond_expand(&[clause], &None);

        match result {
            CondExpandResult::NoMatch => {}
            _ => panic!("Expected no match result"),
        }
    }

    #[test]
    fn test_cond_expand_else_clause() {
        let evaluator = CondExpandEvaluator::new();

        // Create a clause that should not match
        let clause = CondExpandClause {
            feature_requirement: FeatureRequirement::Feature("unknown-feature".to_string()),
            body: vec![],
        };

        let else_body = vec![make_spanned(Expr::Literal(crate::ast::Literal::boolean(
            false,
        )))];
        let result = evaluator.evaluate_cond_expand(&[clause], &Some(else_body));

        match result {
            CondExpandResult::Matched(body) => {
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected matched result from else clause"),
        }
    }

    #[test]
    fn test_macro_integration_trait() {
        let evaluator = CondExpandEvaluator::new();

        // Test feature availability
        assert!(evaluator.feature_available("lambdust"));
        assert!(!evaluator.feature_available("unknown-feature"));

        // Test library availability
        assert!(evaluator.library_available(&["lambdust".to_string(), "core".to_string()]));
        assert!(!evaluator.library_available(&["unknown".to_string(), "lib".to_string()]));
    }

    #[test]
    fn test_compile_time_analysis() {
        let analyzer = CompileTimeFeatures::new();

        // Create a cond-expand expression
        let cond_expand = Expr::CondExpand {
            clauses: vec![
                CondExpandClause {
                    feature_requirement: FeatureRequirement::Feature("lambdust".to_string()),
                    body: vec![make_spanned(Expr::Literal(crate::ast::Literal::boolean(
                        true,
                    )))],
                },
                CondExpandClause {
                    feature_requirement: FeatureRequirement::Feature("unknown".to_string()),
                    body: vec![make_spanned(Expr::Literal(crate::ast::Literal::boolean(
                        false,
                    )))],
                },
            ],
            else_clause: None,
        };

        let analysis = analyzer.analyze_cond_expand_usage(&cond_expand);

        assert_eq!(analysis.total_cond_expands, 1);
        assert_eq!(analysis.total_clauses, 2);
        assert_eq!(analysis.matched_clauses, 1);
        assert_eq!(analysis.unmatched_clauses, 1);
        assert!(analysis.match_rate() > 0.0);
        assert!(analysis.used_features.len() >= 2);
    }

    #[test]
    fn test_helper_functions() {
        use helpers::*;

        // Test basic feature checks
        assert!(has_feature("lambdust"));
        assert!(!has_feature("unknown-feature"));

        // Test library checks
        assert!(has_library(&["lambdust".to_string(), "core".to_string()]));
        assert!(!has_library(&["unknown".to_string()]));

        // Test requirement creation
        let and_req = create_requirement_and(&["lambdust", "r7rs-small"]);
        assert!(matches!(and_req, FeatureRequirement::And(_)));

        let or_req = create_requirement_or(&["simd", "fallback"]);
        assert!(matches!(or_req, FeatureRequirement::Or(_)));

        let not_req = create_requirement_not("debug");
        assert!(matches!(not_req, FeatureRequirement::Not(_)));

        // Test feature listing
        let core_features = available_core_features();
        assert!(core_features.contains(&"lambdust".to_string()));

        let srfi_features = available_srfi_features();
        assert!(srfi_features.contains(&"srfi-0".to_string()));
    }
}

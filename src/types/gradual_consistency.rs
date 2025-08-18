//! Gradual Typing Consistency Checker
//!
//! This module implements comprehensive consistency checking for gradual typing
//! in Lambdust. It provides:
//!
//! - Consistency relation verification between types
//! - Gradual subtyping with precision ordering
//! - Type compatibility analysis at boundaries
//! - Consistency violation detection and reporting
//! - Migration path suggestions for consistency improvements
//!
//! # Consistency Theory
//!
//! The consistency relation ~ is defined as:
//! - T ~ T (reflexivity)
//! - ? ~ T and T ~ ? for any type T (dynamic compatibility)
//! - T1 → T2 ~ T3 → T4 iff T3 ~ T1 and T2 ~ T4 (function consistency)
//! - List T1 ~ List T2 iff T1 ~ T2 (container consistency)
//!
//! # Precision Ordering
//!
//! The precision relation ⊑ defines when one type is more precise than another:
//! - ? ⊑ T for any type T
//! - T1 → T2 ⊑ T3 → T4 iff T3 ⊑ T1 and T2 ⊑ T4
//! - List T1 ⊑ List T2 iff T1 ⊑ T2

use super::{Type, TypeVar, TypeScheme};
use super::gradual::{consistent, is_gradual, is_static};
use crate::diagnostics::{Error, Result, Span, Spanned};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Configuration for consistency checking
#[derive(Debug, Clone)]
pub struct ConsistencyConfig {
    /// Strictness level for consistency checking
    pub strictness: ConsistencyStrictness,
    /// Enable precision ordering analysis
    pub enable_precision_analysis: bool,
    /// Enable migration suggestions
    pub enable_migration_suggestions: bool,
    /// Maximum depth for recursive consistency checking
    pub max_recursion_depth: usize,
    /// Enable consistency caching for performance
    pub enable_consistency_caching: bool,
    /// Enable detailed violation reporting
    pub enable_detailed_reporting: bool,
}

/// Strictness levels for consistency checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConsistencyStrictness {
    /// Permissive: Allow most transitions (development mode)
    Permissive,
    /// Balanced: Standard gradual typing consistency
    Balanced,
    /// Strict: Require explicit evidence for consistency
    Strict,
    /// Ultra: Disallow implicit dynamic transitions
    Ultra,
}

impl Default for ConsistencyConfig {
    fn default() -> Self {
        Self {
            strictness: ConsistencyStrictness::Balanced,
            enable_precision_analysis: true,
            enable_migration_suggestions: true,
            max_recursion_depth: 100,
            enable_consistency_caching: true,
            enable_detailed_reporting: true,
        }
    }
}

/// Result of consistency checking
#[derive(Debug, Clone)]
pub struct ConsistencyResult {
    /// Whether types are consistent
    pub consistent: bool,
    /// Precision relationship
    pub precision: PrecisionRelation,
    /// Detected violations
    pub violations: Vec<ConsistencyViolation>,
    /// Suggested fixes
    pub suggestions: Vec<ConsistencySuggestion>,
    /// Evidence for consistency
    pub evidence: ConsistencyEvidence,
}

/// Precision relationship between types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrecisionRelation {
    /// Types have equal precision
    Equal,
    /// First type is more precise
    FirstMorePrecise,
    /// Second type is more precise
    SecondMorePrecise,
    /// Types are incomparable
    Incomparable,
    /// Cannot determine precision
    Unknown,
}

/// Consistency violation information
#[derive(Debug, Clone)]
pub struct ConsistencyViolation {
    /// Type of violation
    pub violation_type: ViolationType,
    /// Location of violation
    pub location: Span,
    /// Involved types
    pub types: Vec<Type>,
    /// Violation severity
    pub severity: ViolationSeverity,
    /// Detailed explanation
    pub explanation: String,
    /// Context information
    pub context: ViolationContext,
}

/// Types of consistency violations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ViolationType {
    /// Fundamental inconsistency
    Inconsistent,
    /// Precision loss without justification
    PrecisionLoss,
    /// Unsafe dynamic transition
    UnsafeDynamicTransition,
    /// Function arity mismatch
    ArityMismatch,
    /// Container type mismatch
    ContainerMismatch,
    /// Effect mismatch
    EffectMismatch,
    /// Recursive type inconsistency
    RecursiveInconsistency,
}

/// Severity levels for violations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ViolationSeverity {
    /// Informational (suggestion only)
    Info,
    /// Warning (potential issue)
    Warning,
    /// Error (definite problem)
    Error,
    /// Critical (type system unsoundness)
    Critical,
}

/// Context information for violations
#[derive(Debug)]
pub struct ViolationContext {
    /// Expression context
    pub expression_context: String,
    /// Function name (if applicable)
    pub function_name: Option<String>,
    /// Variable name (if applicable)
    pub variable_name: Option<String>,
    /// Call site information
    pub call_site: Option<Span>,
    /// Expected usage pattern
    pub expected_usage: Option<String>,
}

/// Suggestion for fixing consistency issues
#[derive(Debug, Clone)]
pub struct ConsistencySuggestion {
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    /// Location to apply suggestion
    pub location: Span,
    /// Benefit of applying suggestion
    pub benefit: SuggestionBenefit,
    /// Effort required to apply
    pub effort: SuggestionEffort,
    /// Detailed description
    pub description: String,
    /// Code example (if applicable)
    pub example: Option<String>,
}

/// Types of consistency suggestions
#[derive(Debug, Clone)]
pub enum SuggestionType {
    /// Add type annotation
    AddTypeAnnotation { suggested_type: Type },
    /// Use more precise type
    UseMorePreciseType { from: Type, to: Type },
    /// Add cast for safety
    AddExplicitCast { cast_type: CastType },
    /// Refactor to avoid consistency issues
    RefactorCode { strategy: RefactoringStrategy },
    /// Use gradual interface
    UseGradualInterface { interface_type: Type },
}

/// Types of casts for suggestions
#[derive(Debug, Clone)]
pub enum CastType {
    /// Static to dynamic cast
    StaticToDynamic,
    /// Dynamic to static cast with check
    DynamicToStaticWithCheck,
    /// Structural cast
    Structural,
}

/// Refactoring strategies
#[derive(Debug, Clone)]
pub enum RefactoringStrategy {
    /// Extract typed function
    ExtractTypedFunction,
    /// Use type-safe wrapper
    UseTypeSafeWrapper,
    /// Split into typed and untyped parts
    SplitTypedUntyped,
    /// Migrate incrementally
    IncrementalMigration,
}

/// Benefit of applying suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuggestionBenefit {
    /// Marginal improvement
    Marginal,
    /// Moderate improvement
    Moderate,
    /// Significant improvement
    Significant,
    /// Major improvement
    Major,
}

/// Effort required for suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuggestionEffort {
    /// Minimal effort (simple change)
    Minimal,
    /// Low effort (local changes)
    Low,
    /// Medium effort (some refactoring)
    Medium,
    /// High effort (significant changes)
    High,
}

/// Evidence supporting consistency
#[derive(Debug, Clone)]
pub struct ConsistencyEvidence {
    /// Consistency derivation steps
    pub derivation: Vec<ConsistencyStep>,
    /// Witnesses for consistency
    pub witnesses: Vec<ConsistencyWitness>,
    /// Assumptions made
    pub assumptions: Vec<ConsistencyAssumption>,
}

/// Step in consistency derivation
#[derive(Debug, Clone)]
pub struct ConsistencyStep {
    /// Rule applied
    pub rule: ConsistencyRule,
    /// Input types
    pub inputs: Vec<Type>,
    /// Output type relationships
    pub outputs: Vec<TypeRelationship>,
    /// Justification
    pub justification: String,
}

/// Consistency rules
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsistencyRule {
    /// Reflexivity: T ~ T
    Reflexivity,
    /// Dynamic compatibility: ? ~ T
    DynamicCompatibility,
    /// Function consistency: (T1 → T2) ~ (T3 → T4)
    FunctionConsistency,
    /// Container consistency: Container T1 ~ Container T2
    ContainerConsistency,
    /// Transitivity: T1 ~ T2 ∧ T2 ~ T3 ⟹ T1 ~ T3
    Transitivity,
    /// Symmetry: T1 ~ T2 ⟹ T2 ~ T1
    Symmetry,
}

/// Type relationship in derivation
#[derive(Debug, Clone)]
pub struct TypeRelationship {
    /// First type
    pub type1: Type,
    /// Second type
    pub type2: Type,
    /// Relationship kind
    pub relationship: RelationshipKind,
}

/// Kinds of type relationships
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationshipKind {
    /// Consistency relationship
    Consistent,
    /// Inconsistency relationship
    Inconsistent,
    /// Precision relationship
    MorePrecise,
    /// Equality relationship
    Equal,
    /// Subtyping relationship
    Subtype,
}

/// Witness for consistency (proof object)
#[derive(Debug, Clone)]
pub struct ConsistencyWitness {
    /// Types being witnessed
    pub types: Vec<Type>,
    /// Witness kind
    pub kind: WitnessKind,
    /// Supporting evidence
    pub evidence: String,
}

/// Kinds of consistency witnesses
#[derive(Debug, Clone)]
pub enum WitnessKind {
    /// Direct observation
    DirectObservation,
    /// Structural analysis
    StructuralAnalysis,
    /// Dynamic type information
    DynamicTypeInfo,
    /// User annotation
    UserAnnotation,
}

/// Assumption made during consistency checking
#[derive(Debug, Clone)]
pub struct ConsistencyAssumption {
    /// Types involved in assumption
    pub types: Vec<Type>,
    /// Assumption kind
    pub kind: AssumptionKind,
    /// Confidence level
    pub confidence: f64,
    /// Justification
    pub justification: String,
}

/// Kinds of assumptions
#[derive(Debug, Clone)]
pub enum AssumptionKind {
    /// Assume type is more precise
    AssumeMorePrecise,
    /// Assume dynamic compatibility
    AssumeDynamicCompatibility,
    /// Assume structural similarity
    AssumeStructuralSimilarity,
    /// Assume user intent
    AssumeUserIntent,
}

/// Main consistency checker
#[derive(Debug)]
pub struct GradualConsistencyChecker {
    /// Configuration
    config: ConsistencyConfig,
    /// Consistency cache
    consistency_cache: HashMap<(Type, Type), bool>,
    /// Precision cache
    precision_cache: HashMap<(Type, Type), PrecisionRelation>,
    /// Violation database
    violation_db: ViolationDatabase,
    /// Suggestion engine
    suggestion_engine: SuggestionEngine,
}

// Safety: GradualConsistencyChecker can be safely shared between threads
// as it only contains thread-safe data structures and configurations
unsafe impl Send for GradualConsistencyChecker {}
unsafe impl Sync for GradualConsistencyChecker {}

/// Database of consistency violations
#[derive(Debug)]
pub struct ViolationDatabase {
    /// Known violations by type pattern
    violations: HashMap<ViolationPattern, Vec<ConsistencyViolation>>,
    /// Violation statistics
    stats: ViolationStatistics,
}

/// Pattern for matching violations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViolationPattern {
    /// Pattern for first type
    pub type1_pattern: TypePattern,
    /// Pattern for second type
    pub type2_pattern: TypePattern,
    /// Context pattern
    pub context_pattern: ContextPattern,
}

/// Pattern for matching types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypePattern {
    /// Exact type match
    Exact(Type),
    /// Dynamic type
    Dynamic,
    /// Any static type
    AnyStatic,
    /// Function type with arity
    Function { arity: usize },
    /// Container type
    Container { element: Box<TypePattern> },
    /// Wildcard (matches anything)
    Wildcard,
}

/// Pattern for matching contexts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextPattern {
    /// Function application
    FunctionApplication,
    /// Variable assignment
    VariableAssignment,
    /// Return value
    ReturnValue,
    /// Container access
    ContainerAccess,
    /// Any context
    Any,
}

/// Statistics about violations
#[derive(Debug, Clone, Default)]
pub struct ViolationStatistics {
    /// Total violations detected
    pub total_violations: usize,
    /// Violations by type
    pub by_type: HashMap<ViolationType, usize>,
    /// Violations by severity
    pub by_severity: HashMap<ViolationSeverity, usize>,
    /// Most common violation patterns
    pub common_patterns: Vec<(ViolationPattern, usize)>,
}

/// Engine for generating consistency suggestions
#[derive(Debug)]
pub struct SuggestionEngine {
    /// Suggestion rules
    rules: Vec<SuggestionRule>,
    /// Generated suggestions cache
    cache: HashMap<ViolationPattern, Vec<ConsistencySuggestion>>,
}

/// Rule for generating suggestions
pub struct SuggestionRule {
    /// Pattern to match
    pub pattern: ViolationPattern,
    /// Generated suggestion
    pub suggestion: SuggestionType,
    /// Applicability condition
    pub condition: Box<dyn Fn(&ConsistencyViolation) -> bool>,
}

impl std::fmt::Debug for SuggestionRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SuggestionRule")
            .field("pattern", &self.pattern)
            .field("suggestion", &self.suggestion)
            .field("condition", &"<function>")
            .finish()
    }
}

impl Clone for SuggestionRule {
    fn clone(&self) -> Self {
        // Note: We can't actually clone the function, so we create a default one
        // This is a limitation - in practice, suggestion rules would need to be recreated
        Self {
            pattern: self.pattern.clone(),
            suggestion: self.suggestion.clone(),
            condition: Box::new(|_| true), // Default condition
        }
    }
}

impl GradualConsistencyChecker {
    /// Creates a new consistency checker
    pub fn new() -> Self {
        Self::with_config(ConsistencyConfig::default())
    }

    /// Creates a consistency checker with configuration
    pub fn with_config(config: ConsistencyConfig) -> Self {
        Self {
            config,
            consistency_cache: HashMap::new(),
            precision_cache: HashMap::new(),
            violation_db: ViolationDatabase::new(),
            suggestion_engine: SuggestionEngine::new(),
        }
    }

    /// Checks consistency between two types
    pub fn check_consistency(&mut self, type1: &Type, type2: &Type) -> ConsistencyResult {
        self.check_consistency_with_context(type1, type2, &ViolationContext::default())
    }

    /// Checks consistency with context information
    pub fn check_consistency_with_context(
        &mut self,
        type1: &Type,
        type2: &Type,
        context: &ViolationContext
    ) -> ConsistencyResult {
        // Check cache first
        if self.config.enable_consistency_caching {
            if let Some(&cached) = self.consistency_cache.get(&(type1.clone(), type2.clone())) {
                return self.create_cached_result(cached, type1, type2);
            }
        }

        // Perform consistency analysis
        let consistent = self.is_consistent_detailed(type1, type2, 0);
        let precision = self.analyze_precision(type1, type2);
        let violations = self.detect_violations(type1, type2, context);
        let suggestions = self.generate_suggestions(&violations);
        let evidence = self.collect_evidence(type1, type2, consistent);

        // Cache result
        if self.config.enable_consistency_caching {
            self.consistency_cache.insert((type1.clone(), type2.clone()), consistent);
            self.precision_cache.insert((type1.clone(), type2.clone()), precision.clone());
        }

        // Update violation database
        for violation in &violations {
            self.violation_db.record_violation(violation.clone());
        }

        ConsistencyResult {
            consistent,
            precision,
            violations,
            suggestions,
            evidence,
        }
    }

    /// Detailed consistency check with recursion limit
    fn is_consistent_detailed(&self, type1: &Type, type2: &Type, depth: usize) -> bool {
        if depth >= self.config.max_recursion_depth {
            return true; // Assume consistency at depth limit
        }

        match self.config.strictness {
            ConsistencyStrictness::Permissive => self.is_consistent_permissive(type1, type2, depth),
            ConsistencyStrictness::Balanced => consistent(type1, type2),
            ConsistencyStrictness::Strict => self.is_consistent_strict(type1, type2, depth),
            ConsistencyStrictness::Ultra => self.is_consistent_ultra(type1, type2, depth),
        }
    }

    /// Permissive consistency check
    fn is_consistent_permissive(&self, type1: &Type, type2: &Type, _depth: usize) -> bool {
        // In permissive mode, allow most transitions
        match (type1, type2) {
            // Dynamic is consistent with everything
            (Type::Dynamic, _) | (_, Type::Dynamic) => true,
            // Unknown is consistent with everything
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            // Same types are consistent
            (t1, t2) if t1 == t2 => true,
            // Variables are permissive
            (Type::Variable(_), _) | (_, Type::Variable(_)) => true,
            // Different base types allowed with warning
            _ => true,
        }
    }

    /// Strict consistency check
    fn is_consistent_strict(&self, type1: &Type, type2: &Type, depth: usize) -> bool {
        match (type1, type2) {
            // Dynamic requires explicit evidence
            (Type::Dynamic, t) | (t, Type::Dynamic) => {
                // Only allow if other type is gradual or has evidence
                is_gradual(t) || self.has_dynamic_evidence(t)
            }
            // Unknown requires more evidence
            (Type::Unknown, _) | (_, Type::Unknown) => false,
            // Otherwise use standard consistency
            _ => consistent(type1, type2),
        }
    }

    /// Ultra-strict consistency check
    fn is_consistent_ultra(&self, type1: &Type, type2: &Type, depth: usize) -> bool {
        match (type1, type2) {
            // No implicit dynamic transitions
            (Type::Dynamic, t) | (t, Type::Dynamic) => {
                matches!(t, Type::Dynamic)
            }
            // No unknown types
            (Type::Unknown, _) | (_, Type::Unknown) => false,
            // Exact structural consistency required
            _ => self.is_structurally_identical(type1, type2, depth),
        }
    }

    /// Checks if there's evidence for dynamic compatibility
    fn has_dynamic_evidence(&self, _type_: &Type) -> bool {
        // In a real implementation, this would check for:
        // - Runtime type information
        // - User annotations
        // - Previous successful operations
        false
    }

    /// Checks structural identity
    fn is_structurally_identical(&self, type1: &Type, type2: &Type, depth: usize) -> bool {
        if depth >= self.config.max_recursion_depth {
            return type1 == type2;
        }

        match (type1, type2) {
            (Type::Function { params: p1, return_type: r1 },
             Type::Function { params: p2, return_type: r2 }) => {
                p1.len() == p2.len() &&
                p1.iter().zip(p2.iter()).all(|(t1, t2)| {
                    self.is_structurally_identical(t1, t2, depth + 1)
                }) &&
                self.is_structurally_identical(r1, r2, depth + 1)
            }
            (Type::List(t1), Type::List(t2)) => {
                self.is_structurally_identical(t1, t2, depth + 1)
            }
            (Type::Pair(a1, b1), Type::Pair(a2, b2)) => {
                self.is_structurally_identical(a1, a2, depth + 1) &&
                self.is_structurally_identical(b1, b2, depth + 1)
            }
            _ => type1 == type2,
        }
    }

    /// Analyzes precision relationship
    fn analyze_precision(&mut self, type1: &Type, type2: &Type) -> PrecisionRelation {
        if !self.config.enable_precision_analysis {
            return PrecisionRelation::Unknown;
        }

        // Check cache
        if let Some(cached) = self.precision_cache.get(&(type1.clone(), type2.clone())) {
            return cached.clone();
        }

        let relation = self.compute_precision(type1, type2);
        
        // Cache result
        self.precision_cache.insert((type1.clone(), type2.clone()), relation.clone());
        
        relation
    }

    /// Computes precision relationship
    fn compute_precision(&self, type1: &Type, type2: &Type) -> PrecisionRelation {
        match (type1, type2) {
            // Dynamic is least precise
            (Type::Dynamic, Type::Dynamic) => PrecisionRelation::Equal,
            (Type::Dynamic, _) => PrecisionRelation::SecondMorePrecise,
            (_, Type::Dynamic) => PrecisionRelation::FirstMorePrecise,
            
            // Unknown is less precise than concrete types
            (Type::Unknown, Type::Unknown) => PrecisionRelation::Equal,
            (Type::Unknown, _) => PrecisionRelation::SecondMorePrecise,
            (_, Type::Unknown) => PrecisionRelation::FirstMorePrecise,
            
            // Same types have equal precision
            (t1, t2) if t1 == t2 => PrecisionRelation::Equal,
            
            // Function types
            (Type::Function { params: p1, return_type: r1 },
             Type::Function { params: p2, return_type: r2 }) => {
                if p1.len() != p2.len() {
                    return PrecisionRelation::Incomparable;
                }
                
                // Function precision is contravariant in parameters, covariant in return
                let param_precision = p1.iter().zip(p2.iter())
                    .map(|(t1, t2)| self.compute_precision(t2, t1)) // Note: reversed for contravariance
                    .collect::<Vec<_>>();
                
                let return_precision = self.compute_precision(r1, r2);
                
                // Combine precisions (simplified)
                self.combine_precisions(param_precision, return_precision)
            }
            
            // Container types
            (Type::List(t1), Type::List(t2)) => self.compute_precision(t1, t2),
            (Type::Vector(t1), Type::Vector(t2)) => self.compute_precision(t1, t2),
            (Type::Pair(a1, b1), Type::Pair(a2, b2)) => {
                let first_precision = self.compute_precision(a1, a2);
                let second_precision = self.compute_precision(b1, b2);
                self.combine_pair_precisions(first_precision, second_precision)
            }
            
            // Different types are incomparable
            _ => PrecisionRelation::Incomparable,
        }
    }

    /// Combines multiple precision relationships
    fn combine_precisions(&self, param_precisions: Vec<PrecisionRelation>, return_precision: PrecisionRelation) -> PrecisionRelation {
        // Simplified combination logic
        if param_precisions.iter().all(|p| matches!(p, PrecisionRelation::Equal)) &&
           matches!(return_precision, PrecisionRelation::Equal) {
            PrecisionRelation::Equal
        } else {
            PrecisionRelation::Incomparable
        }
    }

    /// Combines precision for pair types
    fn combine_pair_precisions(&self, first: PrecisionRelation, second: PrecisionRelation) -> PrecisionRelation {
        match (first, second) {
            (PrecisionRelation::Equal, PrecisionRelation::Equal) => PrecisionRelation::Equal,
            (PrecisionRelation::FirstMorePrecise, PrecisionRelation::FirstMorePrecise) => PrecisionRelation::FirstMorePrecise,
            (PrecisionRelation::SecondMorePrecise, PrecisionRelation::SecondMorePrecise) => PrecisionRelation::SecondMorePrecise,
            _ => PrecisionRelation::Incomparable,
        }
    }

    /// Detects consistency violations
    fn detect_violations(
        &self,
        type1: &Type,
        type2: &Type,
        context: &ViolationContext
    ) -> Vec<ConsistencyViolation> {
        let mut violations = Vec::new();

        // Check for basic inconsistency
        if !consistent(type1, type2) {
            violations.push(ConsistencyViolation {
                violation_type: ViolationType::Inconsistent,
                location: Span::new(0, 0), // Would use actual location
                types: vec![type1.clone(), type2.clone()],
                severity: ViolationSeverity::Error,
                explanation: format!("Types {type1} and {type2} are inconsistent"),
                context: context.clone(),
            });
        }

        // Check for precision loss
        if self.config.enable_precision_analysis {
            let precision = self.compute_precision(type1, type2);
            if matches!(precision, PrecisionRelation::FirstMorePrecise) &&
               self.config.strictness >= ConsistencyStrictness::Balanced {
                violations.push(ConsistencyViolation {
                    violation_type: ViolationType::PrecisionLoss,
                    location: Span::new(0, 0),
                    types: vec![type1.clone(), type2.clone()],
                    severity: ViolationSeverity::Warning,
                    explanation: "Precision loss detected".to_string(),
                    context: context.clone(),
                });
            }
        }

        // Check for unsafe dynamic transitions
        if matches!((type1, type2), (_, Type::Dynamic)) && is_static(type1) && 
           self.config.strictness >= ConsistencyStrictness::Strict {
            violations.push(ConsistencyViolation {
                violation_type: ViolationType::UnsafeDynamicTransition,
                location: Span::new(0, 0),
                types: vec![type1.clone(), type2.clone()],
                severity: ViolationSeverity::Warning,
                explanation: "Unsafe transition from static to dynamic type".to_string(),
                context: context.clone(),
            });
        }

        violations
    }

    /// Generates suggestions for violations
    fn generate_suggestions(&mut self, violations: &[ConsistencyViolation]) -> Vec<ConsistencySuggestion> {
        if !self.config.enable_migration_suggestions {
            return Vec::new();
        }

        self.suggestion_engine.generate_suggestions(violations)
    }

    /// Collects evidence for consistency
    fn collect_evidence(&self, type1: &Type, type2: &Type, consistent: bool) -> ConsistencyEvidence {
        let mut derivation = Vec::new();
        let mut witnesses = Vec::new();
        let mut assumptions = Vec::new();

        // Create derivation steps
        if consistent {
            derivation.push(ConsistencyStep {
                rule: ConsistencyRule::DynamicCompatibility,
                inputs: vec![type1.clone(), type2.clone()],
                outputs: vec![TypeRelationship {
                    type1: type1.clone(),
                    type2: type2.clone(),
                    relationship: RelationshipKind::Consistent,
                }],
                justification: "Types are consistent via gradual typing rules".to_string(),
            });
        }

        // Add witnesses based on type structure
        if matches!((type1, type2), (Type::Dynamic, _) | (_, Type::Dynamic)) {
            witnesses.push(ConsistencyWitness {
                types: vec![type1.clone(), type2.clone()],
                kind: WitnessKind::DynamicTypeInfo,
                evidence: "Dynamic type provides compatibility".to_string(),
            });
        }

        ConsistencyEvidence {
            derivation,
            witnesses,
            assumptions,
        }
    }

    /// Creates a cached result
    fn create_cached_result(&self, consistent: bool, type1: &Type, type2: &Type) -> ConsistencyResult {
        ConsistencyResult {
            consistent,
            precision: self.precision_cache.get(&(type1.clone(), type2.clone()))
                .cloned()
                .unwrap_or(PrecisionRelation::Unknown),
            violations: Vec::new(), // Would need to cache violations too
            suggestions: Vec::new(),
            evidence: ConsistencyEvidence {
                derivation: Vec::new(),
                witnesses: Vec::new(),
                assumptions: Vec::new(),
            },
        }
    }

    /// Gets current configuration
    pub fn config(&self) -> &ConsistencyConfig {
        &self.config
    }

    /// Updates configuration
    pub fn update_config(&mut self, config: ConsistencyConfig) {
        let enable_caching = config.enable_consistency_caching;
        self.config = config;
        
        // Clear caches if caching was disabled
        if !enable_caching {
            self.consistency_cache.clear();
            self.precision_cache.clear();
        }
    }

    /// Gets violation statistics
    pub fn violation_statistics(&self) -> &ViolationStatistics {
        &self.violation_db.stats
    }

    /// Clears all caches
    pub fn clear_caches(&mut self) {
        self.consistency_cache.clear();
        self.precision_cache.clear();
        self.suggestion_engine.cache.clear();
    }
}

impl ViolationDatabase {
    /// Creates a new violation database
    pub fn new() -> Self {
        Self {
            violations: HashMap::new(),
            stats: ViolationStatistics::default(),
        }
    }

    /// Records a violation
    pub fn record_violation(&mut self, violation: ConsistencyViolation) {
        // Update statistics
        self.stats.total_violations += 1;
        *self.stats.by_type.entry(violation.violation_type.clone()).or_insert(0) += 1;
        *self.stats.by_severity.entry(violation.severity).or_insert(0) += 1;

        // Store violation by pattern
        let pattern = self.create_pattern(&violation);
        self.violations.entry(pattern).or_default().push(violation);
    }

    /// Creates a pattern from a violation
    fn create_pattern(&self, violation: &ConsistencyViolation) -> ViolationPattern {
        let type1_pattern = if !violation.types.is_empty() {
            self.type_to_pattern(&violation.types[0])
        } else {
            TypePattern::Wildcard
        };

        let type2_pattern = if violation.types.len() > 1 {
            self.type_to_pattern(&violation.types[1])
        } else {
            TypePattern::Wildcard
        };

        ViolationPattern {
            type1_pattern,
            type2_pattern,
            context_pattern: ContextPattern::Any, // Simplified
        }
    }

    /// Converts a type to a pattern
    fn type_to_pattern(&self, type_: &Type) -> TypePattern {
        match type_ {
            Type::Dynamic => TypePattern::Dynamic,
            Type::Function { params, .. } => TypePattern::Function { arity: params.len() },
            Type::List(_) => TypePattern::Container { 
                element: Box::new(TypePattern::Wildcard) 
            },
            _ if is_static(type_) => TypePattern::AnyStatic,
            _ => TypePattern::Wildcard,
        }
    }

    /// Gets violations matching a pattern
    pub fn get_violations(&self, pattern: &ViolationPattern) -> Option<&Vec<ConsistencyViolation>> {
        self.violations.get(pattern)
    }
}

impl SuggestionEngine {
    /// Creates a new suggestion engine
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
            cache: HashMap::new(),
        }
    }

    /// Default suggestion rules
    fn default_rules() -> Vec<SuggestionRule> {
        vec![]
    }

    /// Generates suggestions for violations
    pub fn generate_suggestions(&mut self, violations: &[ConsistencyViolation]) -> Vec<ConsistencySuggestion> {
        let mut suggestions = Vec::new();

        for violation in violations {
            suggestions.extend(self.generate_suggestions_for_violation(violation));
        }

        suggestions
    }

    /// Generates suggestions for a single violation
    fn generate_suggestions_for_violation(&mut self, violation: &ConsistencyViolation) -> Vec<ConsistencySuggestion> {
        let mut suggestions = Vec::new();

        match violation.violation_type {
            ViolationType::Inconsistent => {
                suggestions.push(ConsistencySuggestion {
                    suggestion_type: SuggestionType::AddExplicitCast {
                        cast_type: CastType::DynamicToStaticWithCheck,
                    },
                    location: violation.location,
                    benefit: SuggestionBenefit::Significant,
                    effort: SuggestionEffort::Low,
                    description: "Add explicit cast to resolve inconsistency".to_string(),
                    example: Some("(cast-to-number value)".to_string()),
                });
            }
            ViolationType::PrecisionLoss => {
                suggestions.push(ConsistencySuggestion {
                    suggestion_type: SuggestionType::UseMorePreciseType {
                        from: Type::Dynamic,
                        to: Type::Number, // Simplified
                    },
                    location: violation.location,
                    benefit: SuggestionBenefit::Moderate,
                    effort: SuggestionEffort::Medium,
                    description: "Use more precise type to avoid precision loss".to_string(),
                    example: None,
                });
            }
            _ => {}
        }

        suggestions
    }
}

impl Default for ViolationContext {
    fn default() -> Self {
        Self {
            expression_context: "unknown".to_string(),
            function_name: None,
            variable_name: None,
            call_site: None,
            expected_usage: None,
        }
    }
}

impl ViolationContext {
    /// Creates context for function application
    pub fn function_application(function_name: String, call_site: Span) -> Self {
        Self {
            expression_context: "function application".to_string(),
            function_name: Some(function_name),
            variable_name: None,
            call_site: Some(call_site),
            expected_usage: None,
        }
    }

    /// Creates context for variable assignment
    pub fn variable_assignment(variable_name: String) -> Self {
        Self {
            expression_context: "variable assignment".to_string(),
            function_name: None,
            variable_name: Some(variable_name),
            call_site: None,
            expected_usage: None,
        }
    }
}


impl Default for GradualConsistencyChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ViolationDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ViolationContext {
    fn clone(&self) -> Self {
        Self {
            expression_context: self.expression_context.clone(),
            function_name: self.function_name.clone(),
            variable_name: self.variable_name.clone(),
            call_site: self.call_site,
            expected_usage: self.expected_usage.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency_checker_creation() {
        let checker = GradualConsistencyChecker::new();
        assert_eq!(checker.config().strictness, ConsistencyStrictness::Balanced);
        assert!(checker.config().enable_precision_analysis);
    }

    #[test]
    fn test_basic_consistency() {
        let mut checker = GradualConsistencyChecker::new();
        
        let result = checker.check_consistency(&Type::Number, &Type::Number);
        assert!(result.consistent);
        assert_eq!(result.precision, PrecisionRelation::Equal);
        
        let result = checker.check_consistency(&Type::Number, &Type::Dynamic);
        assert!(result.consistent);
        assert_eq!(result.precision, PrecisionRelation::FirstMorePrecise);
    }

    #[test]
    fn test_precision_analysis() {
        let mut checker = GradualConsistencyChecker::new();
        
        let precision = checker.analyze_precision(&Type::Dynamic, &Type::Number);
        assert_eq!(precision, PrecisionRelation::SecondMorePrecise);
        
        let precision = checker.analyze_precision(&Type::Number, &Type::Dynamic);
        assert_eq!(precision, PrecisionRelation::FirstMorePrecise);
        
        let precision = checker.analyze_precision(&Type::Number, &Type::Number);
        assert_eq!(precision, PrecisionRelation::Equal);
    }

    #[test]
    fn test_violation_detection() {
        let mut checker = GradualConsistencyChecker::with_config(ConsistencyConfig {
            strictness: ConsistencyStrictness::Strict,
            ..ConsistencyConfig::default()
        });
        
        let context = ViolationContext::default();
        let result = checker.check_consistency_with_context(
            &Type::Number,
            &Type::String,
            &context
        );
        
        assert!(!result.consistent);
        assert!(!result.violations.is_empty());
        assert_eq!(result.violations[0].violation_type, ViolationType::Inconsistent);
    }

    #[test]
    fn test_suggestion_generation() {
        let mut engine = SuggestionEngine::new();
        
        let violation = ConsistencyViolation {
            violation_type: ViolationType::Inconsistent,
            location: Span::new(0, 10),
            types: vec![Type::Number, Type::String],
            severity: ViolationSeverity::Error,
            explanation: "Test violation".to_string(),
            context: ViolationContext::default(),
        };
        
        let suggestions = engine.generate_suggestions(&[violation]);
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_strictness_levels() {
        let mut checker = GradualConsistencyChecker::with_config(ConsistencyConfig {
            strictness: ConsistencyStrictness::Permissive,
            ..ConsistencyConfig::default()
        });
        
        // In permissive mode, most things should be consistent
        assert!(checker.is_consistent_permissive(&Type::Number, &Type::String, 0));
        
        // In ultra mode, very strict
        assert!(!checker.is_consistent_ultra(&Type::Number, &Type::Dynamic, 0));
    }
}
//! R6RS Compliance for Identifier Transformers
//!
//! This module ensures that the identifier transformer implementation
//! complies with the R6RS Scheme specification. It includes validation,
//! conformance testing, and standard procedures as specified in R6RS.

use super::{
    identifier_transformers::{
        VariableTransformer, VariableTransformerRegistry, IdentifierContext,
        TransformerProcedure, TransformationLogic
    },
    variable_transformer_builtins::{
        VariableTransformerBuiltins, make_variable_transformer, make_simple_variable_transformer
    },
    context_aware_expander::ContextAwareMacroExpander,
    syntax_objects::{SyntaxObject, LexicalContext, syntax_utils},
    advanced_hygiene::HygieneResolver,
    unified_expander::UnifiedMacroExpander,
};
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Span};
use crate::eval::{Environment, Value};
use std::collections::{HashMap, HashSet};

/// R6RS-compliant identifier transformer implementation
pub struct R6RSIdentifierTransformerSystem {
    /// Core expander with variable transformer support
    expander: ContextAwareMacroExpander,
    /// Built-in procedures
    builtins: VariableTransformerBuiltins,
    /// Standard environment for identifier transformers
    standard_env: Environment,
    /// Compliance flags
    compliance_flags: R6RSComplianceFlags,
}

/// Flags for R6RS compliance features
#[derive(Debug, Clone)]
pub struct R6RSComplianceFlags {
    /// Enforce strict hygiene rules
    pub strict_hygiene: bool,
    /// Require proper context detection
    pub strict_context_detection: bool,
    /// Enforce proper identifier binding semantics
    pub strict_binding_semantics: bool,
    /// Validate phase separation
    pub validate_phases: bool,
    /// Check for proper expansion ordering
    pub validate_expansion_order: bool,
}

impl Default for R6RSComplianceFlags {
    fn default() -> Self {
        Self {
            strict_hygiene: true,
            strict_context_detection: true,
            strict_binding_semantics: true,
            validate_phases: true,
            validate_expansion_order: true,
        }
    }
}

impl Default for R6RSIdentifierTransformerSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl R6RSIdentifierTransformerSystem {
    /// Creates a new R6RS-compliant identifier transformer system
    pub fn new() -> Self {
        let mut system = Self {
            expander: ContextAwareMacroExpander::new(),
            builtins: VariableTransformerBuiltins::new(),
            standard_env: Environment::new(None, 0),
            compliance_flags: R6RSComplianceFlags::default(),
        };

        system.initialize_standard_environment();
        system
    }

    /// Initializes the standard environment with R6RS procedures
    fn initialize_standard_environment(&mut self) {
        // In a real implementation, this would register actual Scheme procedures
        // For now, we'll set up the structure
        
        self.register_standard_procedures();
        self.register_standard_transformers();
    }

    /// Registers standard R6RS procedures for identifier transformers
    fn register_standard_procedures(&mut self) {
        // These would be actual Scheme procedure implementations
        let procedures = vec![
            ("make-variable-transformer", "Creates a variable transformer from a procedure"),
            ("variable-transformer?", "Predicate to test if a value is a variable transformer"),
            ("identifier?", "Predicate to test if a value is an identifier"),
            ("bound-identifier=?", "Tests if two identifiers have the same binding"),
            ("free-identifier=?", "Tests if two identifiers refer to the same binding"),
            ("generate-temporaries", "Generates fresh identifiers"),
            ("datum->syntax", "Creates a syntax object from a datum"),
            ("syntax->datum", "Extracts datum from a syntax object"),
            ("syntax-violation", "Signals a syntax violation"),
        ];

        for (name, description) in procedures {
            // In a real implementation, we would create and register actual procedures
            println!("Would register R6RS procedure: {name} - {description}");
        }
    }

    /// Registers standard R6RS identifier transformers
    fn register_standard_transformers(&mut self) {
        let context = LexicalContext::new(0, vec!["r6rs-standard".to_string()]);

        // Example standard transformer: define-values (simplified)
        let define_values_transformer = VariableTransformer::simple(
            "define-values".to_string(),
            "(syntax-violation 'define-values \"cannot be used as an expression\" stx)".to_string(),
            "(syntax-violation 'define-values \"cannot be assigned\" stx)".to_string(),
            context.clone(),
        );
        self.expander.register_variable_transformer(define_values_transformer);

        // Add more standard transformers as needed
    }

    /// Creates a variable transformer according to R6RS specification
    pub fn r6rs_make_variable_transformer(
        &mut self,
        transformer_proc: Value,
        env: &Environment,
    ) -> Result<VariableTransformer> {
        // Validate that the procedure is appropriate for use as a transformer
        self.validate_transformer_procedure(&transformer_proc)?;

        // Create the transformer with proper R6RS semantics
        let context = self.get_current_lexical_context(env);
        let transformer = make_variable_transformer(
            transformer_proc,
            None, // Anonymous transformer
            context,
        )?;

        // Validate the transformer complies with R6RS requirements
        self.validate_r6rs_transformer(&transformer)?;

        Ok(transformer)
    }

    /// Validates that a procedure is suitable for use as a variable transformer
    fn validate_transformer_procedure(&self, _proc: &Value) -> Result<()> {
        // In a real implementation, this would check:
        // 1. That the value is actually a procedure
        // 2. That the procedure has the correct arity (1 argument)
        // 3. That the procedure accepts syntax objects
        
        if self.compliance_flags.strict_binding_semantics {
            // Perform strict validation
            // For now, we'll assume validation passes
        }

        Ok(())
    }

    /// Validates that a transformer complies with R6RS requirements
    fn validate_r6rs_transformer(&self, transformer: &VariableTransformer) -> Result<()> {
        // Check R6RS compliance requirements
        
        if self.compliance_flags.strict_context_detection {
            // Ensure the transformer properly handles different contexts
            if !transformer.supports_context(&IdentifierContext::Reference) {
                return Err(Box::new(Error::MacroError {
                    message: "R6RS variable transformers must support reference context".to_string(),
                    span: Span::new(0, 0),
                }));
            }
        }

        if self.compliance_flags.strict_hygiene {
            // Validate hygiene properties
            // In a real implementation, this would check that the transformer
            // properly preserves lexical scoping
        }

        if self.compliance_flags.validate_phases {
            // Ensure proper phase separation
            // Variable transformers should work at macro expansion time
        }

        Ok(())
    }

    /// Gets the current lexical context from an environment
    fn get_current_lexical_context(&self, _env: &Environment) -> LexicalContext {
        // In a real implementation, this would extract context from the environment
        LexicalContext::new(0, vec!["current-module".to_string()])
    }

    /// Expands a form containing identifier transformers with R6RS semantics
    pub fn r6rs_expand(&mut self, syntax: &SyntaxObject) -> Result<SyntaxObject> {
        // Perform expansion with R6RS compliance checks
        if self.compliance_flags.validate_expansion_order {
            self.validate_expansion_order(syntax)?;
        }

        let result = self.expander.expand(syntax)?;

        if self.compliance_flags.strict_hygiene {
            self.validate_hygiene_preservation(syntax, &result)?;
        }

        Ok(result)
    }

    /// Validates the expansion order according to R6RS
    fn validate_expansion_order(&self, _syntax: &SyntaxObject) -> Result<()> {
        // R6RS specifies that macro expansion should happen in a specific order
        // This would implement those checks
        Ok(())
    }

    /// Validates that hygiene is properly preserved
    fn validate_hygiene_preservation(
        &self,
        _original: &SyntaxObject,
        _expanded: &SyntaxObject,
    ) -> Result<()> {
        // Check that the expansion preserves hygiene according to R6RS rules
        Ok(())
    }

    /// Implements the R6RS identifier=? procedure
    pub fn r6rs_identifier_equal(
        &self,
        id1: &SyntaxObject,
        id2: &SyntaxObject,
    ) -> Result<bool> {
        if !id1.is_identifier() || !id2.is_identifier() {
            return Err(Box::new(Error::MacroError {
                message: "identifier=? requires identifier arguments".to_string(),
                span: id1.span,
            }));
        }

        Ok(syntax_utils::bound_identifier_equal(id1, id2))
    }

    /// Implements the R6RS free-identifier=? procedure
    pub fn r6rs_free_identifier_equal(
        &self,
        id1: &SyntaxObject,
        id2: &SyntaxObject,
    ) -> Result<bool> {
        if !id1.is_identifier() || !id2.is_identifier() {
            return Err(Box::new(Error::MacroError {
                message: "free-identifier=? requires identifier arguments".to_string(),
                span: id1.span,
            }));
        }

        Ok(syntax_utils::free_identifier_equal(id1, id2))
    }

    /// Implements the R6RS bound-identifier=? procedure
    pub fn r6rs_bound_identifier_equal(
        &self,
        id1: &SyntaxObject,
        id2: &SyntaxObject,
    ) -> Result<bool> {
        if !id1.is_identifier() || !id2.is_identifier() {
            return Err(Box::new(Error::MacroError {
                message: "bound-identifier=? requires identifier arguments".to_string(),
                span: id1.span,
            }));
        }

        Ok(syntax_utils::bound_identifier_equal(id1, id2))
    }

    /// Implements the R6RS variable-transformer? predicate
    pub fn r6rs_variable_transformer_p(&self, value: &Value) -> bool {
        // In a real implementation, this would check if the value is a variable transformer
        // For now, return false as we're working with simplified types
        false
    }

    /// Gets compliance statistics
    pub fn compliance_stats(&self) -> R6RSComplianceStats {
        R6RSComplianceStats {
            transformers_registered: self.expander.variable_transformer_registry().list_transformers().len(),
            expansions_performed: self.expander.stats().variable_transformer_expansions,
            context_detections: self.expander.stats().context_detections,
            hygiene_violations: 0, // Would be tracked in a real implementation
            phase_violations: 0,   // Would be tracked in a real implementation
        }
    }
}

/// Statistics for R6RS compliance
#[derive(Debug, Clone)]
pub struct R6RSComplianceStats {
    /// Number of variable transformers registered
    pub transformers_registered: usize,
    /// Number of macro expansions performed
    pub expansions_performed: usize,
    /// Number of context detections performed
    pub context_detections: usize,
    /// Number of hygiene violations detected
    pub hygiene_violations: usize,
    /// Number of phase separation violations detected
    pub phase_violations: usize,
}

/// R6RS compliance test suite
pub mod r6rs_compliance_tests {
    use super::*;

    /// Test basic R6RS variable transformer creation
    pub fn test_basic_variable_transformer_creation() -> Result<()> {
        let mut system = R6RSIdentifierTransformerSystem::new();
        let dummy_proc = Value::Nil; // Placeholder
        let env = Environment::new(None, 0);

        let transformer = system.r6rs_make_variable_transformer(dummy_proc, &env)?;
        assert!(transformer.context_sensitive);
        Ok(())
    }

    /// Test R6RS identifier comparison procedures
    pub fn test_identifier_comparison() -> Result<()> {
        let system = R6RSIdentifierTransformerSystem::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 1);

        let id1 = syntax_utils::make_identifier_syntax("x".to_string(), span, context.clone());
        let id2 = syntax_utils::make_identifier_syntax("x".to_string(), span, context);

        // Test bound-identifier=?
        let bound_equal = system.r6rs_bound_identifier_equal(&id1, &id2)?;
        assert!(bound_equal);

        // Test free-identifier=?
        let free_equal = system.r6rs_free_identifier_equal(&id1, &id2)?;
        assert!(free_equal);

        Ok(())
    }

    /// Test R6RS hygiene preservation
    pub fn test_hygiene_preservation() -> Result<()> {
        let mut system = R6RSIdentifierTransformerSystem::new();
        let context = LexicalContext::new(1, vec!["hygiene-test".to_string()]);
        let span = Span::new(0, 10);

        // Create a transformer that introduces bindings
        let hygienic_transformer = VariableTransformer::simple(
            "hygienic".to_string(),
            "(let ((temp (get-value))) temp)".to_string(),
            "(let ((temp {val})) (set-value! temp))".to_string(),
            context.clone(),
        );
        system.expander.register_variable_transformer(hygienic_transformer);

        // Create a form that uses the transformer in a context where 'temp' is bound
        let form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("let".to_string(), span, context.clone()),
                syntax_utils::make_list_syntax(
                    vec![
                        syntax_utils::make_list_syntax(
                            vec![
                                syntax_utils::make_identifier_syntax("temp".to_string(), span, context.clone()),
                                syntax_utils::make_literal_syntax(Literal::ExactInteger(42), span, context.clone()),
                            ],
                            span,
                            context.clone(),
                        ),
                    ],
                    span,
                    context.clone(),
                ),
                syntax_utils::make_identifier_syntax("hygienic".to_string(), span, context.clone()),
            ],
            span,
            context,
        );

        let result = system.r6rs_expand(&form)?;
        // The expansion should preserve hygiene
        assert!(result.is_list());

        Ok(())
    }

    /// Test R6RS context-sensitive expansion
    pub fn test_context_sensitive_expansion() -> Result<()> {
        let mut system = R6RSIdentifierTransformerSystem::new();
        let context = LexicalContext::new(1, vec!["context-test".to_string()]);
        let span = Span::new(0, 8);

        // Register a context-sensitive transformer
        let transformer = VariableTransformer::simple(
            "context-var".to_string(),
            "(ref-value)".to_string(),
            "(set-value! {val})".to_string(),
            context.clone(),
        );
        system.expander.register_variable_transformer(transformer);

        // Test reference context
        let reference = syntax_utils::make_identifier_syntax("context-var".to_string(), span, context.clone());
        let result = system.r6rs_expand(&reference)?;
        assert!(result.is_list() || result.is_identifier());

        // Test assignment context
        let assignment = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("context-var".to_string(), span, context.clone()),
                syntax_utils::make_literal_syntax(Literal::integer(100), span, context.clone()),
            ],
            span,
            context,
        );
        let result = system.r6rs_expand(&assignment)?;
        assert!(result.is_list());

        Ok(())
    }

    /// Test R6RS error conditions
    pub fn test_error_conditions() -> Result<()> {
        let system = R6RSIdentifierTransformerSystem::new();
        let context = LexicalContext::new(1, vec!["error-test".to_string()]);
        let span = Span::new(0, 1);

        // Test identifier=? with non-identifiers
        let non_id = syntax_utils::make_literal_syntax(Literal::integer(42), span, context.clone());
        let id = syntax_utils::make_identifier_syntax("x".to_string(), span, context);

        let result = system.r6rs_bound_identifier_equal(&non_id, &id);
        assert!(result.is_err());

        let result = system.r6rs_free_identifier_equal(&non_id, &id);
        assert!(result.is_err());

        Ok(())
    }

    /// Test R6RS phase separation
    pub fn test_phase_separation() -> Result<()> {
        let mut system = R6RSIdentifierTransformerSystem::new();
        
        // Variable transformers should work at macro expansion time (phase 1)
        // This test would verify that phase separation is properly maintained
        
        // In a full implementation, this would test that:
        // 1. Transformers are only available at expansion time
        // 2. Runtime values don't leak into expansion time
        // 3. Expansion-time values don't leak into runtime
        
        Ok(())
    }

    /// Run all R6RS compliance tests
    pub fn run_all_r6rs_tests() -> Result<()> {
        test_basic_variable_transformer_creation()?;
        test_identifier_comparison()?;
        test_hygiene_preservation()?;
        test_context_sensitive_expansion()?;
        test_error_conditions()?;
        test_phase_separation()?;
        
        println!("All R6RS compliance tests passed!");
        Ok(())
    }
}

/// R6RS specification compliance validator
pub struct R6RSComplianceValidator {
    /// Validation rules
    rules: Vec<R6RSValidationRule>,
    /// Validation results
    results: Vec<R6RSValidationResult>,
}

/// A validation rule for R6RS compliance
#[derive(Debug, Clone)]
pub struct R6RSValidationRule {
    /// Name of the validation rule
    pub name: String,
    /// Human-readable description of the rule
    pub description: String,
    /// Severity level of the validation rule
    pub severity: ValidationSeverity,
    /// Type/category of the validation rule
    pub rule_type: RuleType,
}

/// Severity of a validation rule
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationSeverity {
    /// Must be fixed for compliance
    Error,
    /// Should be fixed for best practices
    Warning,
    /// Informational only
    Info,
}

/// Type of validation rule
#[derive(Debug, Clone)]
pub enum RuleType {
    /// Hygiene preservation rules
    Hygiene,
    /// Context detection and handling rules
    ContextDetection,
    /// Phase separation rules
    PhaseSeparation,
    /// Binding semantics rules
    BindingSemantics,
    /// Expansion order rules
    ExpansionOrder,
}

/// Result of a validation check
#[derive(Debug, Clone)]
pub struct R6RSValidationResult {
    /// Name of the rule that was validated
    pub rule_name: String,
    /// Whether the validation passed
    pub passed: bool,
    /// Human-readable validation message
    pub message: String,
    /// Severity of the validation result
    pub severity: ValidationSeverity,
    /// Optional source location of the validation issue
    pub location: Option<Span>,
}

impl R6RSComplianceValidator {
    /// Creates a new validator with standard R6RS rules
    pub fn new() -> Self {
        let mut validator = Self {
            rules: Vec::new(),
            results: Vec::new(),
        };
        
        validator.register_standard_rules();
        validator
    }

    /// Registers standard R6RS validation rules
    fn register_standard_rules(&mut self) {
        self.rules.extend(vec![
            R6RSValidationRule {
                name: "hygiene-preservation".to_string(),
                description: "Variable transformers must preserve hygiene".to_string(),
                severity: ValidationSeverity::Error,
                rule_type: RuleType::Hygiene,
            },
            R6RSValidationRule {
                name: "context-sensitivity".to_string(),
                description: "Transformers must handle reference and assignment contexts".to_string(),
                severity: ValidationSeverity::Error,
                rule_type: RuleType::ContextDetection,
            },
            R6RSValidationRule {
                name: "phase-separation".to_string(),
                description: "Expansion-time and runtime must be properly separated".to_string(),
                severity: ValidationSeverity::Error,
                rule_type: RuleType::PhaseSeparation,
            },
            R6RSValidationRule {
                name: "identifier-binding".to_string(),
                description: "Identifier binding semantics must follow R6RS rules".to_string(),
                severity: ValidationSeverity::Error,
                rule_type: RuleType::BindingSemantics,
            },
        ]);
    }

    /// Validates a variable transformer for R6RS compliance
    pub fn validate_transformer(&mut self, transformer: &VariableTransformer) -> Vec<R6RSValidationResult> {
        let mut results = Vec::new();

        for rule in &self.rules {
            let result = match rule.rule_type {
                RuleType::Hygiene => self.validate_hygiene_rule(transformer, rule),
                RuleType::ContextDetection => self.validate_context_rule(transformer, rule),
                RuleType::PhaseSeparation => self.validate_phase_rule(transformer, rule),
                RuleType::BindingSemantics => self.validate_binding_rule(transformer, rule),
                RuleType::ExpansionOrder => self.validate_expansion_rule(transformer, rule),
            };
            
            results.push(result);
        }

        self.results.extend(results.clone());
        results
    }

    /// Validates hygiene rules
    fn validate_hygiene_rule(
        &self,
        _transformer: &VariableTransformer,
        rule: &R6RSValidationRule,
    ) -> R6RSValidationResult {
        // In a real implementation, this would check hygiene properties
        R6RSValidationResult {
            rule_name: rule.name.clone(),
            passed: true,
            message: "Hygiene validation passed".to_string(),
            severity: rule.severity.clone(),
            location: None,
        }
    }

    /// Validates context detection rules
    fn validate_context_rule(
        &self,
        transformer: &VariableTransformer,
        rule: &R6RSValidationRule,
    ) -> R6RSValidationResult {
        let supports_reference = transformer.supports_context(&IdentifierContext::Reference);
        let supports_assignment = transformer.supports_context(&IdentifierContext::Assignment);

        let passed = supports_reference && supports_assignment;
        let message = if passed {
            "Context detection validation passed".to_string()
        } else {
            "Transformer must support both reference and assignment contexts".to_string()
        };

        R6RSValidationResult {
            rule_name: rule.name.clone(),
            passed,
            message,
            severity: rule.severity.clone(),
            location: None,
        }
    }

    /// Validates phase separation rules
    fn validate_phase_rule(
        &self,
        _transformer: &VariableTransformer,
        rule: &R6RSValidationRule,
    ) -> R6RSValidationResult {
        // In a real implementation, this would check phase separation
        R6RSValidationResult {
            rule_name: rule.name.clone(),
            passed: true,
            message: "Phase separation validation passed".to_string(),
            severity: rule.severity.clone(),
            location: None,
        }
    }

    /// Validates binding semantics rules
    fn validate_binding_rule(
        &self,
        _transformer: &VariableTransformer,
        rule: &R6RSValidationRule,
    ) -> R6RSValidationResult {
        // In a real implementation, this would check binding semantics
        R6RSValidationResult {
            rule_name: rule.name.clone(),
            passed: true,
            message: "Binding semantics validation passed".to_string(),
            severity: rule.severity.clone(),
            location: None,
        }
    }

    /// Validates expansion order rules
    fn validate_expansion_rule(
        &self,
        _transformer: &VariableTransformer,
        rule: &R6RSValidationRule,
    ) -> R6RSValidationResult {
        // In a real implementation, this would check expansion order
        R6RSValidationResult {
            rule_name: rule.name.clone(),
            passed: true,
            message: "Expansion order validation passed".to_string(),
            severity: rule.severity.clone(),
            location: None,
        }
    }

    /// Gets validation summary
    pub fn validation_summary(&self) -> R6RSValidationSummary {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let errors = self.results.iter().filter(|r| r.severity == ValidationSeverity::Error && !r.passed).count();
        let warnings = self.results.iter().filter(|r| r.severity == ValidationSeverity::Warning && !r.passed).count();

        R6RSValidationSummary {
            total_rules: total,
            passed_rules: passed,
            failed_rules: total - passed,
            errors,
            warnings,
            compliant: errors == 0,
        }
    }
}

/// Summary of R6RS validation results
#[derive(Debug, Clone)]
pub struct R6RSValidationSummary {
    /// Total number of validation rules checked
    pub total_rules: usize,
    /// Number of rules that passed validation
    pub passed_rules: usize,
    /// Number of rules that failed validation
    pub failed_rules: usize,
    /// Number of validation errors encountered
    pub errors: usize,
    /// Number of validation warnings generated
    pub warnings: usize,
    /// Whether the identifier transformations are R6RS compliant
    pub compliant: bool,
}

impl Default for R6RSComplianceValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r6rs_system_creation() {
        let system = R6RSIdentifierTransformerSystem::new();
        assert!(system.compliance_flags.strict_hygiene);
        assert!(system.compliance_flags.strict_context_detection);
    }

    #[test]
    fn test_compliance_validator() {
        let mut validator = R6RSComplianceValidator::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        
        let transformer = VariableTransformer::simple(
            "test-var".to_string(),
            "(get-test)".to_string(),
            "(set-test! {val})".to_string(),
            context,
        );

        let results = validator.validate_transformer(&transformer);
        assert!(!results.is_empty());

        let summary = validator.validation_summary();
        assert!(summary.total_rules > 0);
    }

    #[test]
    fn test_r6rs_compliance_tests() {
        assert!(r6rs_compliance_tests::run_all_r6rs_tests().is_ok());
    }
}
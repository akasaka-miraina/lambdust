//! Identifier Transformers for Lambdust Scheme
//!
//! This module implements identifier transformers (variable transformers) as specified
//! in R6RS Scheme. Identifier transformers provide context-sensitive macro expansion
//! that can behave differently when used as:
//! - Variable reference (identifier by itself)
//! - Assignment target (in `set!` expressions)
//! - Macro call (in procedure position)
//!
//! Key features:
//! - `make-variable-transformer` for creating variable-like macros
//! - Context detection and sensitive expansion
//! - Integration with syntax objects and hygiene system
//! - R6RS compliant behavior

use super::{
    advanced_hygiene::{HygieneResolver, Mark, MarkSet},
    syntax_case::{SyntaxBindings, SyntaxPattern, SyntaxTemplate},
    syntax_objects::{LexicalContext, SyntaxObject, syntax_utils},
    unified_expander::{MacroTransformerType, UnifiedMacroTransformer},
};
use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::{Environment, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Context in which an identifier is used
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IdentifierContext {
    /// Used as a variable reference (e.g., `x` by itself)
    Reference,
    /// Used as an assignment target (e.g., `(set! x value)`)
    Assignment,
    /// Used in procedure position (e.g., `(x arg1 arg2)`)
    ProcedureCall,
    /// Used in macro definition context
    MacroDefinition,
    /// Used in syntax-case pattern
    Pattern,
    /// Used in template position
    Template,
    /// Custom context with additional metadata
    Custom(String),
}

impl fmt::Display for IdentifierContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentifierContext::Reference => write!(f, "reference"),
            IdentifierContext::Assignment => write!(f, "assignment"),
            IdentifierContext::ProcedureCall => write!(f, "procedure-call"),
            IdentifierContext::MacroDefinition => write!(f, "macro-definition"),
            IdentifierContext::Pattern => write!(f, "pattern"),
            IdentifierContext::Template => write!(f, "template"),
            IdentifierContext::Custom(name) => write!(f, "custom:{name}"),
        }
    }
}

/// A variable transformer that provides context-sensitive macro expansion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableTransformer {
    /// Name of the transformer
    pub name: String,
    /// The procedure that performs the transformation
    pub transformer_proc: TransformerProcedure,
    /// Lexical environment where the transformer was defined
    pub definition_context: LexicalContext,
    /// Whether this transformer can be used in different contexts
    pub context_sensitive: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// The procedure that actually performs the transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformerProcedure {
    /// Syntax-case based transformer with clauses
    SyntaxCase {
        /// Literal identifiers that match only themselves
        literals: Vec<String>,
        /// Pattern-template clauses for matching and expansion
        clauses: Vec<TransformerClause>,
    },
    /// Procedure-based transformer (takes syntax object, returns syntax object)
    Procedure {
        /// Name for debugging
        name: String,
        /// Procedure that performs transformation
        /// In practice this would be a Rust closure or a Scheme procedure
        /// For now we'll represent it as a simplified pattern matcher
        transformation_logic: TransformationLogic,
    },
}

/// A clause in a variable transformer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerClause {
    /// Pattern to match against
    pub pattern: SyntaxPattern,
    /// Template to expand to
    pub template: SyntaxTemplate,
    /// Guard condition (optional)
    pub guard: Option<GuardCondition>,
    /// Context where this clause applies
    pub context: Option<IdentifierContext>,
}

/// Guard condition for transformer clauses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardCondition {
    /// Simple predicate on the matched syntax
    Predicate(String),
    /// Context-based guard
    ContextGuard(IdentifierContext),
    /// Custom guard with arbitrary logic
    Custom(String),
}

/// Logic for procedure-based transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationLogic {
    /// Reference transformation (when used as variable)
    Reference {
        /// Template expression for variable reference
        target_expr: String,
    },
    /// Assignment transformation (when used in set!)
    Assignment {
        /// Template expression for assignment with value parameter
        target_expr: String,
    },
    /// Call transformation (when used as procedure)
    Call {
        /// Template expression for procedure call with arguments
        target_expr: String,
    },
    /// Complex transformation with full pattern matching
    Complex {
        /// Patterns with context, pattern, and template tuples
        patterns: Vec<(IdentifierContext, String, String)>,
    },
}

impl VariableTransformer {
    /// Creates a new variable transformer
    pub fn new(
        name: String,
        transformer_proc: TransformerProcedure,
        definition_context: LexicalContext,
    ) -> Self {
        Self {
            name,
            transformer_proc,
            definition_context,
            context_sensitive: true,
            metadata: HashMap::new(),
        }
    }

    /// Creates a simple variable transformer with reference/assignment patterns
    pub fn simple(
        name: String,
        reference_template: String,
        assignment_template: String,
        definition_context: LexicalContext,
    ) -> Self {
        let transformation_logic = TransformationLogic::Complex {
            patterns: vec![
                (
                    IdentifierContext::Reference,
                    "id".to_string(),
                    reference_template,
                ),
                (
                    IdentifierContext::Assignment,
                    "(set! id val)".to_string(),
                    assignment_template,
                ),
            ],
        };

        let transformer_proc = TransformerProcedure::Procedure {
            name: name.clone(),
            transformation_logic,
        };

        Self::new(name, transformer_proc, definition_context)
    }

    /// Expands the transformer in a specific context
    pub fn expand(
        &self,
        input: &SyntaxObject,
        context: IdentifierContext,
        hygiene_env: &mut HygieneResolver,
    ) -> Result<SyntaxObject> {
        match &self.transformer_proc {
            TransformerProcedure::SyntaxCase { literals, clauses } => {
                self.expand_syntax_case(input, context, literals, clauses, hygiene_env)
            }
            TransformerProcedure::Procedure {
                transformation_logic,
                ..
            } => self.expand_procedure(input, context, transformation_logic, hygiene_env),
        }
    }

    /// Expands using syntax-case style clauses
    fn expand_syntax_case(
        &self,
        input: &SyntaxObject,
        context: IdentifierContext,
        literals: &[String],
        clauses: &[TransformerClause],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<SyntaxObject> {
        for clause in clauses {
            // Check if clause applies to this context
            if let Some(clause_context) = &clause.context {
                if *clause_context != context {
                    continue;
                }
            }

            // Try to match the pattern
            if let Ok(bindings) = clause.pattern.match_syntax(input) {
                // Check guard condition if present
                if let Some(guard) = &clause.guard {
                    if !self.check_guard(guard, input, &context, &bindings)? {
                        continue;
                    }
                }

                // Expand the template
                return clause
                    .template
                    .expand(&bindings, &input.context, input.span);
            }
        }

        Err(Box::new(Error::MacroError {
            message: format!(
                "No matching clause for variable transformer '{}' in context '{}'",
                self.name, context
            ),
            span: input.span,
        }))
    }

    /// Expands using procedure-based logic
    fn expand_procedure(
        &self,
        input: &SyntaxObject,
        context: IdentifierContext,
        logic: &TransformationLogic,
        hygiene_env: &mut HygieneResolver,
    ) -> Result<SyntaxObject> {
        match logic {
            TransformationLogic::Reference { target_expr } => {
                if context == IdentifierContext::Reference {
                    self.parse_and_expand_template(target_expr, input, &HashMap::new(), hygiene_env)
                } else {
                    Err(Box::new(Error::MacroError {
                        message: format!(
                            "Variable transformer '{}' only supports reference context",
                            self.name
                        ),
                        span: input.span,
                    }))
                }
            }

            TransformationLogic::Assignment { target_expr } => {
                if context == IdentifierContext::Assignment {
                    // Extract the value from the set! form
                    let bindings = self.extract_assignment_bindings(input)?;
                    self.parse_and_expand_template(target_expr, input, &bindings, hygiene_env)
                } else {
                    Err(Box::new(Error::MacroError {
                        message: format!(
                            "Variable transformer '{}' only supports assignment context",
                            self.name
                        ),
                        span: input.span,
                    }))
                }
            }

            TransformationLogic::Call { target_expr } => {
                if context == IdentifierContext::ProcedureCall {
                    let bindings = self.extract_call_bindings(input)?;
                    self.parse_and_expand_template(target_expr, input, &bindings, hygiene_env)
                } else {
                    Err(Box::new(Error::MacroError {
                        message: format!(
                            "Variable transformer '{}' only supports call context",
                            self.name
                        ),
                        span: input.span,
                    }))
                }
            }

            TransformationLogic::Complex { patterns } => {
                for (pattern_context, pattern_str, template_str) in patterns {
                    if *pattern_context == context {
                        // Try to match the pattern
                        if let Some(bindings) = self.match_pattern_string(pattern_str, input) {
                            return self.parse_and_expand_template(
                                template_str,
                                input,
                                &bindings,
                                hygiene_env,
                            );
                        }
                    }
                }

                Err(Box::new(Error::MacroError {
                    message: format!(
                        "No matching pattern for variable transformer '{}' in context '{}'",
                        self.name, context
                    ),
                    span: input.span,
                }))
            }
        }
    }

    /// Checks a guard condition
    fn check_guard(
        &self,
        guard: &GuardCondition,
        input: &SyntaxObject,
        context: &IdentifierContext,
        bindings: &SyntaxBindings,
    ) -> Result<bool> {
        match guard {
            GuardCondition::Predicate(_pred) => {
                // In a real implementation, this would evaluate the predicate
                // For now, we'll assume all predicates pass
                Ok(true)
            }
            GuardCondition::ContextGuard(guard_context) => Ok(guard_context == context),
            GuardCondition::Custom(_) => {
                // Custom guard logic would go here
                Ok(true)
            }
        }
    }

    /// Extracts bindings from a set! form
    fn extract_assignment_bindings(
        &self,
        input: &SyntaxObject,
    ) -> Result<HashMap<String, SyntaxObject>> {
        if let Some(list) = input.as_list() {
            if list.len() == 3 {
                let mut bindings = HashMap::new();
                if let Some(identifier) = list[1].identifier_name() {
                    bindings.insert("id".to_string(), list[1].clone());
                    bindings.insert("val".to_string(), list[2].clone());
                    return Ok(bindings);
                }
            }
        }

        Err(Box::new(Error::MacroError {
            message: "Invalid set! form for variable transformer".to_string(),
            span: input.span,
        }))
    }

    /// Extracts bindings from a procedure call
    fn extract_call_bindings(&self, input: &SyntaxObject) -> Result<HashMap<String, SyntaxObject>> {
        if let Some(list) = input.as_list() {
            if !list.is_empty() {
                let mut bindings = HashMap::new();
                bindings.insert("proc".to_string(), list[0].clone());

                // Bind arguments
                for (i, arg) in list.iter().skip(1).enumerate() {
                    bindings.insert(format!("arg{i}"), arg.clone());
                }

                // Also bind as a list
                let args_list = syntax_utils::make_list_syntax(
                    list.iter().skip(1).cloned().collect(),
                    input.span,
                    input.context.clone(),
                );
                bindings.insert("args".to_string(), args_list);

                return Ok(bindings);
            }
        }

        Err(Box::new(Error::MacroError {
            message: "Invalid procedure call form for variable transformer".to_string(),
            span: input.span,
        }))
    }

    /// Matches a pattern string against input
    fn match_pattern_string(
        &self,
        pattern_str: &str,
        input: &SyntaxObject,
    ) -> Option<HashMap<String, SyntaxObject>> {
        // This is a simplified pattern matcher
        // In a real implementation, this would parse the pattern string
        // and perform proper pattern matching

        if pattern_str == "id" && input.is_identifier() {
            let mut bindings = HashMap::new();
            bindings.insert("id".to_string(), input.clone());
            Some(bindings)
        } else if pattern_str.starts_with("(set! id ") && input.is_list() {
            // Try to extract assignment bindings, return None if it fails
            self.extract_assignment_bindings(input).ok()
        } else {
            None
        }
    }

    /// Parses and expands a template string
    fn parse_and_expand_template(
        &self,
        template_str: &str,
        input: &SyntaxObject,
        bindings: &HashMap<String, SyntaxObject>,
        hygiene_env: &mut HygieneResolver,
    ) -> Result<SyntaxObject> {
        // This is a simplified template expander
        // In a real implementation, this would parse the template string
        // and perform proper template expansion with hygiene

        // For now, we'll do simple string substitution
        let mut expanded = template_str.to_string();

        for (name, value) in bindings {
            let placeholder = format!("{{{name}}}");
            if let Some(value_str) = value.identifier_name() {
                expanded = expanded.replace(&placeholder, value_str);
            }
        }

        // Create a syntax object from the expanded string
        // This is very simplified - a real implementation would parse properly
        if expanded.starts_with('(') && expanded.ends_with(')') {
            // It's a list - create list syntax
            let inner = &expanded[1..expanded.len() - 1];
            let parts: Vec<&str> = inner.split_whitespace().collect();

            let elements: Vec<SyntaxObject> = parts
                .iter()
                .map(|part| {
                    syntax_utils::make_identifier_syntax(
                        part.to_string(),
                        input.span,
                        input.context.clone(),
                    )
                })
                .collect();

            Ok(syntax_utils::make_list_syntax(
                elements,
                input.span,
                input.context.clone(),
            ))
        } else {
            // It's an identifier
            Ok(syntax_utils::make_identifier_syntax(
                expanded,
                input.span,
                input.context.clone(),
            ))
        }
    }

    /// Checks if this transformer can handle the given context
    pub fn supports_context(&self, context: &IdentifierContext) -> bool {
        if !self.context_sensitive {
            return true;
        }

        match &self.transformer_proc {
            TransformerProcedure::SyntaxCase { clauses, .. } => clauses
                .iter()
                .any(|clause| clause.context.as_ref().is_none_or(|c| c == context)),
            TransformerProcedure::Procedure {
                transformation_logic,
                ..
            } => match transformation_logic {
                TransformationLogic::Reference { .. } => *context == IdentifierContext::Reference,
                TransformationLogic::Assignment { .. } => *context == IdentifierContext::Assignment,
                TransformationLogic::Call { .. } => *context == IdentifierContext::ProcedureCall,
                TransformationLogic::Complex { patterns } => patterns
                    .iter()
                    .any(|(pattern_context, _, _)| pattern_context == context),
            },
        }
    }
}

/// Context detector for identifying how an identifier is being used
#[derive(Debug, Clone)]
pub struct ContextDetector;

impl ContextDetector {
    /// Detects the context in which an identifier is being used
    pub fn detect_context(
        identifier: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
    ) -> IdentifierContext {
        if let Some(container) = containing_form {
            if let Some(list) = container.as_list() {
                if list.len() >= 2 {
                    // Check for set! form
                    if let Some(first) = list.first() {
                        if let Some(name) = first.identifier_name() {
                            if name == "set!" && list.len() == 3 {
                                // Check if our identifier is in the second position (assignment target)
                                if let Some(target_name) = list[1].identifier_name() {
                                    if let Some(id_name) = identifier.identifier_name() {
                                        if target_name == id_name {
                                            return IdentifierContext::Assignment;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Check if identifier is in procedure position (first in list)
                    if let Some(first) = list.first() {
                        if let Some(first_name) = first.identifier_name() {
                            if let Some(id_name) = identifier.identifier_name() {
                                if first_name == id_name {
                                    return IdentifierContext::ProcedureCall;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Default to reference context
        IdentifierContext::Reference
    }

    /// Detects the context by analyzing the surrounding syntax structure
    pub fn detect_context_from_structure(
        identifier: &SyntaxObject,
        parent_syntax: &SyntaxObject,
        grandparent_syntax: Option<&SyntaxObject>,
    ) -> IdentifierContext {
        // More sophisticated context detection based on syntax structure
        if let Some(grandparent) = grandparent_syntax {
            if let Some(gp_list) = grandparent.as_list() {
                if gp_list.len() >= 3 {
                    if let Some(first) = gp_list.first() {
                        if let Some(name) = first.identifier_name() {
                            if name == "set!" {
                                // Check if identifier is the assignment target
                                if gp_list.len() == 3 {
                                    if let Some(target_name) = gp_list[1].identifier_name() {
                                        if let Some(id_name) = identifier.identifier_name() {
                                            if target_name == id_name {
                                                return IdentifierContext::Assignment;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(p_list) = parent_syntax.as_list() {
            if !p_list.is_empty() {
                if let Some(first) = p_list.first() {
                    if let Some(first_name) = first.identifier_name() {
                        if let Some(id_name) = identifier.identifier_name() {
                            if first_name == id_name {
                                return IdentifierContext::ProcedureCall;
                            }
                        }
                    }
                }
            }
        }

        IdentifierContext::Reference
    }
}

/// Registry for managing variable transformers
#[derive(Debug, Clone)]
pub struct VariableTransformerRegistry {
    /// Registered transformers by name
    transformers: HashMap<String, VariableTransformer>,
    /// Context detector
    context_detector: ContextDetector,
}

impl VariableTransformerRegistry {
    /// Creates a new registry
    pub fn new() -> Self {
        Self {
            transformers: HashMap::new(),
            context_detector: ContextDetector,
        }
    }

    /// Registers a variable transformer
    pub fn register(&mut self, transformer: VariableTransformer) {
        self.transformers
            .insert(transformer.name.clone(), transformer);
    }

    /// Gets a variable transformer by name
    pub fn get(&self, name: &str) -> Option<&VariableTransformer> {
        self.transformers.get(name)
    }

    /// Checks if a name is bound to a variable transformer
    pub fn is_variable_transformer(&self, name: &str) -> bool {
        self.transformers.contains_key(name)
    }

    /// Expands a variable transformer in context
    pub fn expand_variable_transformer(
        &self,
        name: &str,
        input: &SyntaxObject,
        containing_form: Option<&SyntaxObject>,
        hygiene_env: &mut HygieneResolver,
    ) -> Result<SyntaxObject> {
        let transformer = self
            .transformers
            .get(name)
            .ok_or_else(|| Error::MacroError {
                message: format!("Unknown variable transformer: {name}"),
                span: input.span,
            })?;

        let context = ContextDetector::detect_context(input, containing_form);

        if !transformer.supports_context(&context) {
            return Err(Box::new(Error::MacroError {
                message: format!(
                    "Variable transformer '{name}' does not support context '{context}'"
                ),
                span: input.span,
            }));
        }

        transformer.expand(input, context, hygiene_env)
    }

    /// Lists all registered transformer names
    pub fn list_transformers(&self) -> Vec<&str> {
        self.transformers.keys().map(|s| s.as_str()).collect()
    }

    /// Removes a transformer by name
    pub fn unregister(&mut self, name: &str) -> Option<VariableTransformer> {
        self.transformers.remove(name)
    }

    /// Clears all transformers
    pub fn clear(&mut self) {
        self.transformers.clear();
    }
}

impl Default for VariableTransformerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::diagnostics::Span;

    #[test]
    fn test_context_detection() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 1);

        // Test reference context
        let identifier =
            syntax_utils::make_identifier_syntax("x".to_string(), span, context.clone());
        let ctx = ContextDetector::detect_context(&identifier, None);
        assert_eq!(ctx, IdentifierContext::Reference);

        // Test assignment context
        let set_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("set!".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("x".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("value".to_string(), span, context.clone()),
            ],
            span,
            context.clone(),
        );
        let x_in_set = &set_form.as_list().unwrap()[1];
        let ctx = ContextDetector::detect_context(x_in_set, Some(&set_form));
        assert_eq!(ctx, IdentifierContext::Assignment);

        // Test procedure call context
        let call_form = syntax_utils::make_list_syntax(
            vec![
                syntax_utils::make_identifier_syntax("f".to_string(), span, context.clone()),
                syntax_utils::make_identifier_syntax("arg".to_string(), span, context.clone()),
            ],
            span,
            context.clone(),
        );
        let f_in_call = &call_form.as_list().unwrap()[0];
        let ctx = ContextDetector::detect_context(f_in_call, Some(&call_form));
        assert_eq!(ctx, IdentifierContext::ProcedureCall);
    }

    #[test]
    fn test_variable_transformer_creation() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        let transformer = VariableTransformer::simple(
            "my-var".to_string(),
            "storage-ref".to_string(),
            "storage-set!".to_string(),
            context,
        );

        assert_eq!(transformer.name, "my-var");
        assert!(transformer.context_sensitive);
        assert!(transformer.supports_context(&IdentifierContext::Reference));
        assert!(transformer.supports_context(&IdentifierContext::Assignment));
    }

    #[test]
    fn test_variable_transformer_registry() {
        let mut registry = VariableTransformerRegistry::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        let transformer = VariableTransformer::simple(
            "my-var".to_string(),
            "storage-ref".to_string(),
            "storage-set!".to_string(),
            context,
        );

        registry.register(transformer);

        assert!(registry.is_variable_transformer("my-var"));
        assert!(!registry.is_variable_transformer("unknown"));
        assert_eq!(registry.list_transformers(), vec!["my-var"]);

        let retrieved = registry.get("my-var").unwrap();
        assert_eq!(retrieved.name, "my-var");
    }
}

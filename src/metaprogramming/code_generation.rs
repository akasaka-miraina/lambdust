//! Dynamic code generation and compilation system.
//!
//! This module provides facilities for generating, transforming, and compiling
//! Lambdust code at runtime, including AST manipulation, template systems,
//! and dynamic procedure definition.

use crate::ast::{Expr, Formals, Literal, Program};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::{Value, Environment, Procedure};
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::collections::HashMap;
use std::sync::Arc;
use std::rc::Rc;

/// AST transformation rule.
#[derive(Debug, Clone)]
pub struct TransformRule {
    /// Name of the transformation
    pub name: String,
    /// Pattern to match
    pub pattern: AstPattern,
    /// Template for replacement
    pub template: AstTemplate,
    /// Transformation type
    pub transform_type: TransformType,
    /// Condition for application
    pub condition: Option<TransformCondition>,
}

/// Type of AST transformation.
#[derive(Debug, Clone, PartialEq)]
pub enum TransformType {
    /// Replace matched nodes
    Replace,
    /// Insert before matched nodes
    InsertBefore,
    /// Insert after matched nodes
    InsertAfter,
    /// Wrap matched nodes
    Wrap,
    /// Remove matched nodes
    Remove,
}

/// Pattern for matching AST nodes.
#[derive(Debug, Clone)]
pub enum AstPattern {
    /// Match any expression
    Any,
    /// Match literal values
    Literal(LiteralPattern),
    /// Match identifiers
    Identifier(IdentifierPattern),
    /// Match applications
    Application {
        /// The operator pattern to match.
        operator: Box<AstPattern>,
        /// The operand patterns to match.
        operands: Vec<AstPattern>,
    },
    /// Match special forms
    SpecialForm {
        /// The type of special form to match.
        form_type: String,
        /// Patterns for the special form arguments.
        patterns: Vec<AstPattern>,
    },
    /// Match with variable binding
    Variable {
        /// The variable name for binding.
        name: String,
        /// The pattern to match and bind to the variable.
        pattern: Box<AstPattern>,
    },
    /// Match alternatives (OR pattern)
    Alternative(Vec<AstPattern>),
    /// Match sequences
    Sequence(Vec<AstPattern>),
}

/// Pattern for matching literals.
#[derive(Debug, Clone)]
pub enum LiteralPattern {
    /// Match any literal
    Any,
    /// Match specific boolean
    Boolean(bool),
    /// Match number range
    NumberRange { 
        /// Minimum value (inclusive, None means no minimum).
        min: Option<f64>, 
        /// Maximum value (inclusive, None means no maximum).
        max: Option<f64> 
    },
    /// Match string pattern
    StringPattern(String), // Could be regex
    /// Match specific character
    Character(char),
}

/// Pattern for matching identifiers.
#[derive(Debug, Clone)]
pub enum IdentifierPattern {
    /// Match any identifier
    Any,
    /// Match specific identifier
    Exact(String),
    /// Match identifier matching regex
    Regex(String),
    /// Match identifiers in a set
    OneOf(Vec<String>),
}

/// Template for generating AST nodes.
#[derive(Debug, Clone)]
pub enum AstTemplate {
    /// Generate literal
    Literal(Literal),
    /// Generate identifier
    Identifier(String),
    /// Generate application
    Application {
        /// Template for the operator.
        operator: Box<AstTemplate>,
        /// Templates for the operands.
        operands: Vec<AstTemplate>,
    },
    /// Generate special form
    SpecialForm {
        /// The type of special form to generate.
        form_type: String,
        /// Templates for the special form arguments.
        templates: Vec<AstTemplate>,
    },
    /// Substitute variable
    Variable(String),
    /// Generate sequence
    Sequence(Vec<AstTemplate>),
    /// Conditional generation
    Conditional {
        /// The condition to test.
        condition: TransformCondition,
        /// Template to use if condition is true.
        then_template: Box<AstTemplate>,
        /// Template to use if condition is false.
        else_template: Option<Box<AstTemplate>>,
    },
}

/// Condition for transformation application.
#[derive(Debug, Clone)]
pub enum TransformCondition {
    /// Always apply
    Always,
    /// Check if variable is bound
    VariableBound(String),
    /// Check custom predicate
    Predicate(String), // Name of predicate function
    /// Combine conditions
    And(Vec<TransformCondition>),
    /// Logical OR of multiple conditions.
    Or(Vec<TransformCondition>),
    /// Logical NOT of a condition.
    Not(Box<TransformCondition>),
}

/// Code template system for generating code from templates.
#[derive(Debug)]
pub struct TemplateSystem {
    /// Template definitions
    pub templates: HashMap<String, CodeTemplate>,
    /// Template variables
    variables: HashMap<String, Value>,
}

/// A code template with parameters.
#[derive(Debug, Clone)]
pub struct CodeTemplate {
    /// Template name
    pub name: String,
    /// Template parameters
    pub parameters: Vec<String>,
    /// Template body (as string or AST)
    pub body: TemplateBody,
    /// Template metadata
    pub metadata: HashMap<String, Value>,
}

/// Template body representation.
#[derive(Debug, Clone)]
pub enum TemplateBody {
    /// String template with substitution points
    String(String),
    /// AST template
    Ast(AstTemplate),
    /// Hybrid (string that parses to AST)
    Hybrid(String),
}

/// Dynamic definition system for creating procedures at runtime.
#[derive(Debug)]
pub struct DynamicDefinition {
    /// Definition context
    context: Rc<Environment>,
    /// Generated procedure counter
    procedure_counter: u64,
}

impl DynamicDefinition {
    /// Creates a new dynamic definition system.
    pub fn new(context: Rc<Environment>) -> Self {
        Self {
            context,
            procedure_counter: 0,
        }
    }

    /// Creates a procedure from a code string.
    pub fn create_procedure_from_string(
        &mut self,
        name: Option<String>,
        parameters: &[String],
        body: &str,
    ) -> Result<Value> {
        // Parse the body
        let mut lexer = Lexer::new(body, None);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let parsed_body = parser.parse_expression()?;

        // Create formals
        let formals = if parameters.is_empty() {
            Formals::Fixed(vec![])
        } else {
            Formals::Fixed(parameters.to_vec())
        };

        // Create procedure
        let procedure = Procedure {
            formals,
            body: vec![parsed_body],
            environment: self.context.to_thread_safe(),
            name: name.clone(),
            metadata: HashMap::new(),
            source: None,
        };

        self.procedure_counter += 1;
        Ok(Value::Procedure(Arc::new(procedure)))
    }

    /// Creates a procedure from an AST.
    pub fn create_procedure_from_ast(
        &mut self,
        name: Option<String>,
        formals: Formals,
        body: Vec<Spanned<Expr>>,
    ) -> Result<Value> {
        let procedure = Procedure {
            formals,
            body,
            environment: self.context.to_thread_safe(),
            name,
            metadata: HashMap::new(),
            source: None,
        };

        self.procedure_counter += 1;
        Ok(Value::Procedure(Arc::new(procedure)))
    }

    /// Defines a procedure in the environment.
    pub fn define_procedure(
        &mut self,
        name: &str,
        procedure: Value,
    ) -> Result<()> {
        if !procedure.is_procedure() {
            return Err(Box::new(Error::runtime_error(
                "Expected procedure".to_string(),
                None,
            )));
        }

        self.context.define(name.to_string(), procedure);
        Ok(())
    }
}

/// AST transformer for programmatic AST manipulation.
#[derive(Debug)]
pub struct AstTransformer {
    /// Transformation rules
    pub rules: Vec<TransformRule>,
    /// Transformation context
    context: TransformContext,
}

/// Context for AST transformations.
#[derive(Debug)]
pub struct TransformContext {
    /// Variable bindings during transformation
    pub bindings: HashMap<String, Spanned<Expr>>,
    /// Transformation depth (for recursion control)
    pub depth: usize,
    /// Maximum transformation depth
    pub max_depth: usize,
}

impl AstTransformer {
    /// Creates a new AST transformer.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            context: TransformContext {
                bindings: HashMap::new(),
                depth: 0,
                max_depth: 100,
            },
        }
    }

    /// Adds a transformation rule.
    pub fn add_rule(&mut self, rule: TransformRule) {
        self.rules.push(rule);
    }

    /// Transforms an expression using all applicable rules.
    pub fn transform(&mut self, expr: &Spanned<Expr>) -> Result<Spanned<Expr>> {
        if self.context.depth >= self.context.max_depth {
            return Err(Box::new(Error::runtime_error(
                "Maximum transformation depth exceeded".to_string(),
                Some(expr.span),
            )));
        }

        self.context.depth += 1;
        let result = self.transform_inner(expr);
        self.context.depth -= 1;
        result
    }

    /// Internal transformation method.
    fn transform_inner(&mut self, expr: &Spanned<Expr>) -> Result<Spanned<Expr>> {
        // Try each rule in order
        for rule in &self.rules.clone() {
            if let Some(result) = self.try_apply_rule(rule, expr)? {
                // Rule applied, recursively transform the result
                return self.transform(&result);
            }
        }

        // No rules matched, transform children
        match &expr.inner {
            Expr::Application { operator, operands } => {
                let new_operator = self.transform(operator)?;
                let mut new_operands = Vec::new();
                for operand in operands {
                    new_operands.push(self.transform(operand)?);
                }
                Ok(Spanned::new(
                    Expr::Application {
                        operator: Box::new(new_operator),
                        operands: new_operands,
                    },
                    expr.span,
                ))
            }
            
            Expr::Lambda { formals, metadata, body } => {
                let mut new_body = Vec::new();
                for expr in body {
                    new_body.push(self.transform(expr)?);
                }
                // Note: metadata transformation would need similar recursive handling
                Ok(Spanned::new(
                    Expr::Lambda {
                        formals: formals.clone(),
                        metadata: metadata.clone(), // Simplified
                        body: new_body,
                    },
                    expr.span,
                ))
            }
            
            // Other expression types would be handled similarly
            _ => Ok(expr.clone()),
        }
    }

    /// Tries to apply a single transformation rule.
    fn try_apply_rule(
        &mut self,
        rule: &TransformRule,
        expr: &Spanned<Expr>,
    ) -> Result<Option<Spanned<Expr>>> {
        // Check if pattern matches
        if !self.pattern_matches(&rule.pattern, expr)? {
            return Ok(None);
        }

        // Check condition if present
        if let Some(condition) = &rule.condition {
            if !self.evaluate_condition(condition)? {
                return Ok(None);
            }
        }

        // Apply transformation
        let result = self.apply_template(&rule.template, expr)?;
        Ok(Some(result))
    }

    /// Checks if a pattern matches an expression.
    fn pattern_matches(&mut self, pattern: &AstPattern, expr: &Spanned<Expr>) -> Result<bool> {
        match pattern {
            AstPattern::Any => Ok(true),
            
            AstPattern::Literal(lit_pattern) => {
                if let Expr::Literal(lit) = &expr.inner {
                    self.literal_pattern_matches(lit_pattern, lit)
                } else {
                    Ok(false)
                }
            }
            
            AstPattern::Identifier(id_pattern) => {
                if let Expr::Identifier(name) = &expr.inner {
                    self.identifier_pattern_matches(id_pattern, name)
                } else {
                    Ok(false)
                }
            }
            
            AstPattern::Application { operator, operands } => {
                if let Expr::Application { operator: expr_op, operands: expr_ops } = &expr.inner {
                    if !self.pattern_matches(operator, expr_op)? {
                        return Ok(false);
                    }
                    if operands.len() != expr_ops.len() {
                        return Ok(false);
                    }
                    for (pattern, expr) in operands.iter().zip(expr_ops.iter()) {
                        if !self.pattern_matches(pattern, expr)? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            
            AstPattern::Variable { name, pattern } => {
                if self.pattern_matches(pattern, expr)? {
                    self.context.bindings.insert(name.clone(), expr.clone());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            
            AstPattern::Alternative(patterns) => {
                for pattern in patterns {
                    if self.pattern_matches(pattern, expr)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            
            _ => Ok(false), // Other patterns not implemented
        }
    }

    /// Checks if a literal pattern matches a literal.
    fn literal_pattern_matches(&self, pattern: &LiteralPattern, literal: &Literal) -> Result<bool> {
        match (pattern, literal) {
            (LiteralPattern::Any, _) => Ok(true),
            (LiteralPattern::Boolean(expected), Literal::Boolean(actual)) => Ok(expected == actual),
            (LiteralPattern::Character(expected), Literal::Character(actual)) => Ok(expected == actual),
            (LiteralPattern::NumberRange { min, max }, Literal::ExactInteger(n)) => {
                let n = *n as f64;
                let min_ok = min.unwrap_or(f64::NEG_INFINITY) <= n;
                let max_ok = n <= max.unwrap_or(f64::INFINITY);
                Ok(min_ok && max_ok)
            }
            (LiteralPattern::NumberRange { min, max }, Literal::InexactReal(n)) => {
                let n = *n;
                let min_ok = min.unwrap_or(f64::NEG_INFINITY) <= n;
                let max_ok = n <= max.unwrap_or(f64::INFINITY);
                Ok(min_ok && max_ok)
            }
            _ => Ok(false),
        }
    }

    /// Checks if an identifier pattern matches an identifier.
    fn identifier_pattern_matches(&self, pattern: &IdentifierPattern, name: &str) -> Result<bool> {
        match pattern {
            IdentifierPattern::Any => Ok(true),
            IdentifierPattern::Exact(expected) => Ok(name == expected),
            IdentifierPattern::OneOf(names) => Ok(names.contains(&name.to_string())),
            IdentifierPattern::Regex(_) => Ok(false), // Not implemented
        }
    }

    /// Evaluates a transformation condition.
    fn evaluate_condition(&self, condition: &TransformCondition) -> Result<bool> {
        match condition {
            TransformCondition::Always => Ok(true),
            TransformCondition::VariableBound(name) => Ok(self.context.bindings.contains_key(name)),
            TransformCondition::And(conditions) => {
                for cond in conditions {
                    if !self.evaluate_condition(cond)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            TransformCondition::Or(conditions) => {
                for cond in conditions {
                    if self.evaluate_condition(cond)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            TransformCondition::Not(cond) => Ok(!self.evaluate_condition(cond)?),
            _ => Ok(false), // Other conditions not implemented
        }
    }

    /// Applies a template to generate an expression.
    fn apply_template(&self, template: &AstTemplate, _context_expr: &Spanned<Expr>) -> Result<Spanned<Expr>> {
        match template {
            AstTemplate::Literal(lit) => Ok(Spanned::new(
                Expr::Literal(lit.clone()),
                Span::default(),
            )),
            
            AstTemplate::Identifier(name) => Ok(Spanned::new(
                Expr::Identifier(name.clone()),
                Span::default(),
            )),
            
            AstTemplate::Variable(name) => {
                if let Some(expr) = self.context.bindings.get(name) {
                    Ok(expr.clone())
                } else {
                    Err(Box::new(Error::runtime_error(
                        format!("Unbound template variable: {name}"),
                        None,
                    )))
                }
            }
            
            AstTemplate::Application { operator, operands } => {
                let new_operator = self.apply_template(operator, _context_expr)?;
                let mut new_operands = Vec::new();
                for operand_template in operands {
                    new_operands.push(self.apply_template(operand_template, _context_expr)?);
                }
                Ok(Spanned::new(
                    Expr::Application {
                        operator: Box::new(new_operator),
                        operands: new_operands,
                    },
                    Span::default(),
                ))
            }
            
            _ => Err(Box::new(Error::runtime_error(
                "Template type not implemented".to_string(),
                None,
            )))
        }
    }
}

impl TemplateSystem {
    /// Creates a new template system.
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    /// Defines a new template.
    pub fn define_template(&mut self, template: CodeTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Sets a template variable.
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Expands a template with given arguments.
    pub fn expand_template(&self, name: &str, args: &[Value]) -> Result<String> {
        let template = self.templates.get(name)
            .ok_or_else(|| Error::runtime_error(
                format!("Unknown template: {name}"),
                None,
            ))?;

        if args.len() != template.parameters.len() {
            return Err(Box::new(Error::runtime_error(
                format!("Template {} expects {} arguments, got {}",
                    name, template.parameters.len(), args.len()),
                None,
            )));
        }

        match &template.body {
            TemplateBody::String(s) => {
                let mut result = s.clone();
                
                // Simple string substitution
                for (param, arg) in template.parameters.iter().zip(args.iter()) {
                    let placeholder = format!("{{{param}}}");
                    let replacement = self.value_to_string(arg);
                    result = result.replace(&placeholder, &replacement);
                }
                
                Ok(result)
            }
            _ => Err(Box::new(Error::runtime_error(
                "Template body type not implemented".to_string(),
                None,
            ))),
        }
    }

    /// Converts a value to its string representation for templates.
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Literal(Literal::String(s)) => s.clone(),
            Value::Literal(Literal::ExactInteger(n)) => n.to_string(),
            Value::Literal(Literal::InexactReal(n)) => n.to_string(),
            Value::Literal(Literal::Boolean(b)) => if *b { "#t".to_string() } else { "#f".to_string() },
            Value::Symbol(sym) => crate::utils::symbol_name(*sym).unwrap_or_else(|| format!("symbol-{}", sym.0)),
            _ => format!("{value}"),
        }
    }
}

/// Main code generation system.
#[derive(Debug)]
pub struct CodeGenerator {
    /// AST transformer
    transformer: AstTransformer,
    /// Template system
    template_system: TemplateSystem,
    /// Dynamic definition system
    dynamic_definition: Option<DynamicDefinition>,
}

impl CodeGenerator {
    /// Creates a new code generator.
    pub fn new() -> Self {
        Self {
            transformer: AstTransformer::new(),
            template_system: TemplateSystem::new(),
            dynamic_definition: None,
        }
    }

    /// Sets the context for dynamic definitions.
    pub fn set_context(&mut self, context: Rc<Environment>) {
        self.dynamic_definition = Some(DynamicDefinition::new(context));
    }

    /// Compiles code from a string.
    pub fn compile_string(&self, code: &str) -> Result<Program> {
        let mut lexer = Lexer::new(code, None);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    /// Transforms an AST using registered rules.
    pub fn transform_ast(&mut self, expr: &Spanned<Expr>) -> Result<Spanned<Expr>> {
        self.transformer.transform(expr)
    }

    /// Generates code from a template.
    pub fn generate_from_template(&self, template_name: &str, args: &[Value]) -> Result<String> {
        self.template_system.expand_template(template_name, args)
    }

    /// Creates a procedure dynamically.
    pub fn create_procedure(
        &mut self,
        name: Option<String>,
        parameters: &[String],
        body: &str,
    ) -> Result<Value> {
        if let Some(ref mut dynamic_def) = self.dynamic_definition {
            dynamic_def.create_procedure_from_string(name, parameters, body)
        } else {
            Err(Box::new(Error::runtime_error(
                "No context set for dynamic definition".to_string(),
                None,
            )))
        }
    }

    /// Installs code generation primitives.
    pub fn install_primitives(&self, _env: &Rc<Environment>) -> Result<()> {
        // Placeholder - would install primitives like compile, transform-ast, etc.
        Ok(())
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AstTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TemplateSystem {
    fn default() -> Self {
        Self::new()
    }
}
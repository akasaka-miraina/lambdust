//! Syntax Objects and Advanced Hygiene System for Lambdust
//!
//! This module implements a comprehensive syntax object system that wraps AST nodes
//! with lexical context, source location information, and binding metadata. It provides
//! the foundation for advanced macro hygiene and meta-programming capabilities following
//! R6RS and R7RS specifications.
//!
//! Key features:
//! - Rich syntax object representation with context tracking
//! - Advanced mark-and-sweep hygiene algorithm
//! - Integration with existing AST and macro infrastructure
//! - Support for syntax-case, datum->syntax, syntax->datum
//! - Lexical scoping correctness through macro expansion

use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::Environment;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for generating unique syntax object identifiers
static SYNTAX_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generates a unique syntax object identifier
pub fn next_syntax_id() -> SyntaxId {
    SyntaxId(SYNTAX_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
}

/// Unique identifier for syntax objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SyntaxId(u64);

impl SyntaxId {
    /// Gets the raw numeric identifier value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for SyntaxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "stx#{}", self.0)
    }
}

/// Represents the lexical context in which a syntax object was created
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexicalContext {
    /// Unique identifier for this context
    pub context_id: u64,
    /// Parent context (for nested scopes)
    pub parent: Option<Box<LexicalContext>>,
    /// Phase level (0 = runtime, 1 = macro-time, etc.)
    pub phase: i32,
    /// Module path where this context was created
    pub module_path: Vec<String>,
    /// Additional context metadata
    pub metadata: HashMap<String, String>,
}

impl LexicalContext {
    /// Creates a new top-level lexical context
    pub fn new(context_id: u64, module_path: Vec<String>) -> Self {
        Self {
            context_id,
            parent: None,
            phase: 0,
            module_path,
            metadata: HashMap::new(),
        }
    }

    /// Creates a child context with the same phase
    pub fn child(&self, context_id: u64) -> Self {
        Self {
            context_id,
            parent: Some(Box::new(self.clone())),
            phase: self.phase,
            module_path: self.module_path.clone(),
            metadata: HashMap::new(),
        }
    }

    /// Creates a child context at a different phase
    pub fn child_at_phase(&self, context_id: u64, phase: i32) -> Self {
        Self {
            context_id,
            parent: Some(Box::new(self.clone())),
            phase,
            module_path: self.module_path.clone(),
            metadata: HashMap::new(),
        }
    }

    /// Checks if this context is derived from another context
    pub fn derives_from(&self, other: &LexicalContext) -> bool {
        if self.context_id == other.context_id {
            return true;
        }
        if let Some(parent) = &self.parent {
            parent.derives_from(other)
        } else {
            false
        }
    }

    /// Gets the depth of this context (number of parent levels)
    pub fn depth(&self) -> usize {
        match &self.parent {
            Some(parent) => 1 + parent.depth(),
            None => 0,
        }
    }
}

/// Represents binding information for identifiers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BindingInfo {
    /// The binding name
    pub name: String,
    /// Context where this binding was introduced
    pub binding_context: LexicalContext,
    /// Phase at which this binding exists
    pub phase: i32,
    /// Whether this is a macro binding
    pub is_macro: bool,
    /// Module where this binding was defined
    pub module: Option<Vec<String>>,
    /// Additional binding metadata
    pub metadata: HashMap<String, String>,
}

impl BindingInfo {
    /// Creates a new binding info
    pub fn new(name: String, binding_context: LexicalContext, phase: i32, is_macro: bool) -> Self {
        Self {
            name,
            binding_context,
            phase,
            is_macro,
            module: None,
            metadata: HashMap::new(),
        }
    }

    /// Checks if this binding is accessible from the given context
    pub fn accessible_from(&self, context: &LexicalContext) -> bool {
        // Same phase required for access
        if self.phase != context.phase {
            return false;
        }

        // Check if the binding context is compatible
        self.binding_context.derives_from(context) || context.derives_from(&self.binding_context)
    }
}

/// A syntax object that wraps an expression with contextual information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxObject {
    /// Unique identifier for this syntax object
    pub id: SyntaxId,
    /// The wrapped expression
    pub expr: Expr,
    /// Source location information
    pub span: Span,
    /// Lexical context where this syntax was created
    pub context: LexicalContext,
    /// Binding information for identifiers
    pub bindings: HashMap<String, BindingInfo>,
    /// Syntax marks for hygiene tracking
    pub marks: HashSet<u64>,
    /// Properties and metadata
    pub properties: HashMap<String, SyntaxProperty>,
    /// Original source text (if available)
    pub source_text: Option<String>,
}

/// Properties that can be attached to syntax objects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SyntaxProperty {
    /// String property
    String(String),
    /// Numeric property
    Number(f64),
    /// Boolean property
    Boolean(bool),
    /// List of syntax objects
    SyntaxList(Vec<SyntaxObject>),
    /// Custom property with serialized data
    Custom(Vec<u8>),
}

impl SyntaxObject {
    /// Creates a new syntax object
    pub fn new(expr: Expr, span: Span, context: LexicalContext) -> Self {
        Self {
            id: next_syntax_id(),
            expr,
            span,
            context,
            bindings: HashMap::new(),
            marks: HashSet::new(),
            properties: HashMap::new(),
            source_text: None,
        }
    }

    /// Creates a syntax object from a spanned expression
    pub fn from_spanned(spanned: Spanned<Expr>, context: LexicalContext) -> Self {
        Self::new(spanned.inner, spanned.span, context)
    }

    /// Creates a syntax object with source text
    pub fn with_source(
        expr: Expr,
        span: Span,
        context: LexicalContext,
        source_text: String,
    ) -> Self {
        let mut syntax = Self::new(expr, span, context);
        syntax.source_text = Some(source_text);
        syntax
    }

    /// Converts this syntax object to a spanned expression
    pub fn to_spanned(&self) -> Spanned<Expr> {
        Spanned::new(self.expr.clone(), self.span)
    }

    /// Gets the datum (expression) from this syntax object
    pub fn datum(&self) -> &Expr {
        &self.expr
    }

    /// Gets the source location
    pub fn location(&self) -> Span {
        self.span
    }

    /// Adds a mark for hygiene tracking
    pub fn add_mark(&mut self, mark: u64) {
        self.marks.insert(mark);
    }

    /// Removes a mark
    pub fn remove_mark(&mut self, mark: u64) {
        self.marks.remove(&mark);
    }

    /// Checks if this syntax has a specific mark
    pub fn has_mark(&self, mark: u64) -> bool {
        self.marks.contains(&mark)
    }

    /// Adds a binding for an identifier
    pub fn add_binding(&mut self, name: String, binding: BindingInfo) {
        self.bindings.insert(name, binding);
    }

    /// Gets binding information for an identifier
    pub fn get_binding(&self, name: &str) -> Option<&BindingInfo> {
        self.bindings.get(name)
    }

    /// Sets a property on this syntax object
    pub fn set_property(&mut self, key: String, value: SyntaxProperty) {
        self.properties.insert(key, value);
    }

    /// Gets a property from this syntax object
    pub fn get_property(&self, key: &str) -> Option<&SyntaxProperty> {
        self.properties.get(key)
    }

    /// Creates a copy of this syntax object with a new expression
    pub fn with_expr(&self, expr: Expr) -> Self {
        Self {
            id: next_syntax_id(),
            expr,
            span: self.span,
            context: self.context.clone(),
            bindings: self.bindings.clone(),
            marks: self.marks.clone(),
            properties: self.properties.clone(),
            source_text: self.source_text.clone(),
        }
    }

    /// Creates a copy with a new context
    pub fn with_context(&self, context: LexicalContext) -> Self {
        Self {
            id: next_syntax_id(),
            expr: self.expr.clone(),
            span: self.span,
            context,
            bindings: self.bindings.clone(),
            marks: self.marks.clone(),
            properties: self.properties.clone(),
            source_text: self.source_text.clone(),
        }
    }

    /// Creates a copy with additional marks
    pub fn with_marks(&self, additional_marks: &HashSet<u64>) -> Self {
        let mut marks = self.marks.clone();
        marks.extend(additional_marks);
        Self {
            id: next_syntax_id(),
            expr: self.expr.clone(),
            span: self.span,
            context: self.context.clone(),
            bindings: self.bindings.clone(),
            marks,
            properties: self.properties.clone(),
            source_text: self.source_text.clone(),
        }
    }

    /// Checks if this syntax object is an identifier
    pub fn is_identifier(&self) -> bool {
        matches!(self.expr, Expr::Identifier(_) | Expr::Symbol(_))
    }

    /// Gets the identifier name if this is an identifier
    pub fn identifier_name(&self) -> Option<&str> {
        match &self.expr {
            Expr::Identifier(name) | Expr::Symbol(name) => Some(name),
            _ => None,
        }
    }

    /// Checks if this syntax represents a literal
    pub fn is_literal(&self) -> bool {
        matches!(self.expr, Expr::Literal(_))
    }

    /// Checks if this is a syntax list
    pub fn is_list(&self) -> bool {
        matches!(self.expr, Expr::List(_) | Expr::Application { .. })
    }

    /// Converts this syntax to a list of syntax objects (if applicable)
    pub fn as_list(&self) -> Option<Vec<SyntaxObject>> {
        match &self.expr {
            Expr::List(elements) => Some(
                elements
                    .iter()
                    .map(|elem| SyntaxObject::from_spanned(elem.clone(), self.context.clone()))
                    .collect(),
            ),
            Expr::Application { operator, operands } => {
                let mut result = vec![SyntaxObject::from_spanned(
                    (**operator).clone(),
                    self.context.clone(),
                )];
                result.extend(
                    operands
                        .iter()
                        .map(|op| SyntaxObject::from_spanned(op.clone(), self.context.clone())),
                );
                Some(result)
            }
            _ => None,
        }
    }

    /// Applies hygiene transformation to this syntax object
    pub fn apply_hygiene(&self, hygiene_env: &HygieneEnvironment) -> Result<Self> {
        let transformed_expr = hygiene_env.transform_expr(&self.expr, &self.context)?;
        Ok(self.with_expr(transformed_expr))
    }
}

impl PartialEq for SyntaxObject {
    fn eq(&self, other: &Self) -> bool {
        // Syntax objects are equal if they have the same ID
        self.id == other.id
    }
}

impl Eq for SyntaxObject {}

impl Hash for SyntaxObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for SyntaxObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#<syntax:{} {}>", self.id, self.expr)
    }
}

/// Environment for managing hygiene transformations
#[derive(Debug, Clone)]
pub struct HygieneEnvironment {
    /// Current mark for hygiene
    current_mark: u64,
    /// Identifier renaming map (using Vec instead of HashSet for Hash compatibility)
    renamings: HashMap<(String, Vec<u64>), String>,
    /// Context stack for nested expansions
    context_stack: Vec<LexicalContext>,
    /// Counter for generating unique names
    rename_counter: u64,
}

impl HygieneEnvironment {
    /// Creates a new hygiene environment
    pub fn new() -> Self {
        Self {
            current_mark: 1,
            renamings: HashMap::new(),
            context_stack: Vec::new(),
            rename_counter: 0,
        }
    }

    /// Enters a new hygiene scope with a fresh mark
    pub fn enter_scope(&mut self) -> u64 {
        self.current_mark += 1;
        self.current_mark
    }

    /// Exits the current hygiene scope
    pub fn exit_scope(&mut self) {
        // Mark is automatically managed by scope entry
    }

    /// Pushes a context onto the stack
    pub fn push_context(&mut self, context: LexicalContext) {
        self.context_stack.push(context);
    }

    /// Pops a context from the stack
    pub fn pop_context(&mut self) -> Option<LexicalContext> {
        self.context_stack.pop()
    }

    /// Gets the current context
    pub fn current_context(&self) -> Option<&LexicalContext> {
        self.context_stack.last()
    }

    /// Transforms an expression for hygiene
    pub fn transform_expr(&self, expr: &Expr, _context: &LexicalContext) -> Result<Expr> {
        match expr {
            Expr::Identifier(name) | Expr::Symbol(name) => {
                // For identifiers, check if we need to rename based on marks
                let marks = Vec::new(); // This would come from the syntax object
                if let Some(renamed) = self.renamings.get(&(name.clone(), marks)) {
                    Ok(Expr::Identifier(renamed.clone()))
                } else {
                    Ok(expr.clone())
                }
            }

            Expr::List(elements) => {
                let transformed: Result<Vec<_>> = elements
                    .iter()
                    .map(|elem| {
                        let transformed_expr = self.transform_expr(&elem.inner, _context)?;
                        Ok(Spanned::new(transformed_expr, elem.span))
                    })
                    .collect();
                Ok(Expr::List(transformed?))
            }

            Expr::Application { operator, operands } => {
                let transformed_operator = self.transform_expr(&operator.inner, _context)?;
                let transformed_operands: Result<Vec<_>> = operands
                    .iter()
                    .map(|op| {
                        let transformed_expr = self.transform_expr(&op.inner, _context)?;
                        Ok(Spanned::new(transformed_expr, op.span))
                    })
                    .collect();

                Ok(Expr::Application {
                    operator: Box::new(Spanned::new(transformed_operator, operator.span)),
                    operands: transformed_operands?,
                })
            }

            // For other expression types, recursively transform sub-expressions
            _ => Ok(expr.clone()), // Simplified for now
        }
    }

    /// Generates a unique renamed identifier
    pub fn generate_rename(&mut self, original: &str, marks: Vec<u64>) -> String {
        self.rename_counter += 1;
        let renamed = format!("{}#hyg{}", original, self.rename_counter);
        self.renamings
            .insert((original.to_string(), marks), renamed.clone());
        renamed
    }

    /// Marks an identifier as needing hygiene processing
    pub fn mark_identifier(&mut self, name: &str, marks: Vec<u64>) -> String {
        if let Some(renamed) = self.renamings.get(&(name.to_string(), marks.clone())) {
            renamed.clone()
        } else {
            self.generate_rename(name, marks)
        }
    }
}

impl Default for HygieneEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilities for working with syntax objects
pub mod syntax_utils {
    use super::*;

    /// Converts a datum (expression) to a syntax object
    pub fn datum_to_syntax(
        datum: Expr,
        template_identifier: Option<&SyntaxObject>,
        span: Span,
    ) -> SyntaxObject {
        let context = if let Some(template) = template_identifier {
            template.context.clone()
        } else {
            LexicalContext::new(0, vec!["top-level".to_string()])
        };

        SyntaxObject::new(datum, span, context)
    }

    /// Converts a syntax object to its datum (expression)
    pub fn syntax_to_datum(syntax: &SyntaxObject) -> Expr {
        syntax.expr.clone()
    }

    /// Checks if two syntax objects are bound-identifier=?
    pub fn bound_identifier_equal(stx1: &SyntaxObject, stx2: &SyntaxObject) -> bool {
        // Two identifiers are bound-identifier=? if they would resolve to the same binding
        if let (Some(name1), Some(name2)) = (stx1.identifier_name(), stx2.identifier_name()) {
            if name1 != name2 {
                return false;
            }

            // Check if they have the same marks (simplified)
            stx1.marks == stx2.marks && stx1.context.context_id == stx2.context.context_id
        } else {
            false
        }
    }

    /// Checks if two syntax objects are free-identifier=?
    pub fn free_identifier_equal(stx1: &SyntaxObject, stx2: &SyntaxObject) -> bool {
        // Two identifiers are free-identifier=? if they refer to the same binding
        if let (Some(name1), Some(name2)) = (stx1.identifier_name(), stx2.identifier_name()) {
            if name1 != name2 {
                return false;
            }

            // Check if they would resolve to the same binding
            match (stx1.get_binding(name1), stx2.get_binding(name2)) {
                (Some(binding1), Some(binding2)) => {
                    binding1.binding_context.context_id == binding2.binding_context.context_id
                        && binding1.phase == binding2.phase
                }
                (None, None) => {
                    // Both unbound - check if they're from compatible contexts
                    stx1.context.phase == stx2.context.phase
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Creates a syntax object for a literal value
    pub fn make_literal_syntax(
        literal: crate::ast::Literal,
        span: Span,
        context: LexicalContext,
    ) -> SyntaxObject {
        SyntaxObject::new(Expr::Literal(literal), span, context)
    }

    /// Creates a syntax object for an identifier
    pub fn make_identifier_syntax(
        name: String,
        span: Span,
        context: LexicalContext,
    ) -> SyntaxObject {
        SyntaxObject::new(Expr::Identifier(name), span, context)
    }

    /// Creates a syntax object for a list
    pub fn make_list_syntax(
        elements: Vec<SyntaxObject>,
        span: Span,
        context: LexicalContext,
    ) -> SyntaxObject {
        let expr_elements: Vec<Spanned<Expr>> =
            elements.into_iter().map(|stx| stx.to_spanned()).collect();

        SyntaxObject::new(Expr::List(expr_elements), span, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::diagnostics::Span;

    #[test]
    fn test_syntax_object_creation() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let expr = Expr::Identifier("x".to_string());
        let span = Span::new(0, 1);

        let syntax = SyntaxObject::new(expr, span, context);

        assert!(syntax.is_identifier());
        assert_eq!(syntax.identifier_name(), Some("x"));
        assert_eq!(syntax.span, span);
    }

    #[test]
    fn test_lexical_context_hierarchy() {
        let parent = LexicalContext::new(1, vec!["parent".to_string()]);
        let child = parent.child(2);

        assert!(child.derives_from(&parent));
        assert!(!parent.derives_from(&child));
        assert_eq!(child.depth(), 1);
        assert_eq!(parent.depth(), 0);
    }

    #[test]
    fn test_hygiene_environment() {
        let mut env = HygieneEnvironment::new();
        let mark1 = env.enter_scope();
        let mark2 = env.enter_scope();

        assert!(mark2 > mark1);

        let marks = vec![mark1];
        let renamed = env.mark_identifier("x", marks);
        assert!(renamed.contains("x#hyg"));
    }

    #[test]
    fn test_bound_identifier_equal() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let span = Span::new(0, 1);

        let stx1 = SyntaxObject::new(Expr::Identifier("x".to_string()), span, context.clone());
        let mut stx2 = SyntaxObject::new(Expr::Identifier("x".to_string()), span, context);

        // Same marks and context
        assert!(syntax_utils::bound_identifier_equal(&stx1, &stx2));

        // Different marks
        stx2.add_mark(42);
        assert!(!syntax_utils::bound_identifier_equal(&stx1, &stx2));
    }

    #[test]
    fn test_syntax_property_system() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let expr = Expr::Identifier("x".to_string());
        let span = Span::new(0, 1);

        let mut syntax = SyntaxObject::new(expr, span, context);

        syntax.set_property(
            "test".to_string(),
            SyntaxProperty::String("value".to_string()),
        );
        syntax.set_property("number".to_string(), SyntaxProperty::Number(42.0));

        assert_eq!(
            syntax.get_property("test"),
            Some(&SyntaxProperty::String("value".to_string()))
        );
        assert_eq!(
            syntax.get_property("number"),
            Some(&SyntaxProperty::Number(42.0))
        );
    }
}

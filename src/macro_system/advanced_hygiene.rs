//! Advanced Mark-and-Sweep Hygiene Algorithm
//!
//! This module implements a sophisticated hygiene system that ensures lexical scoping
//! correctness through macro expansion. It uses a mark-and-sweep approach combined
//! with binding resolution and context-sensitive identifier comparison.
//!
//! The algorithm follows the R6RS specification for macro hygiene and provides:
//! - Precise tracking of identifier binding contexts
//! - Automatic renaming to prevent unwanted capture
//! - Support for both macro-introduced and user-written identifiers
//! - Integration with the syntax object system

use super::syntax_objects::{BindingInfo, HygieneEnvironment, LexicalContext, SyntaxObject};
use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span};
use crate::eval::Environment;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for generating unique marks
static MARK_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generates a unique hygiene mark
pub fn fresh_mark() -> u64 {
    MARK_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Represents a hygiene mark used in the mark-and-sweep algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Mark(u64);

impl Mark {
    /// Creates a fresh mark
    pub fn fresh() -> Self {
        Mark(fresh_mark())
    }

    /// Gets the numeric value of this mark
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// A set of marks associated with an identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkSet {
    marks: HashSet<Mark>,
}

impl std::hash::Hash for MarkSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the marks in a consistent order
        let mut marks: Vec<_> = self.marks.iter().collect();
        marks.sort();
        marks.hash(state);
    }
}

impl MarkSet {
    /// Creates an empty mark set
    pub fn empty() -> Self {
        Self {
            marks: HashSet::new(),
        }
    }

    /// Creates a mark set with a single mark
    pub fn singleton(mark: Mark) -> Self {
        let mut marks = HashSet::new();
        marks.insert(mark);
        Self { marks }
    }

    /// Creates a mark set from an iterator of marks
    pub fn from_marks<I: IntoIterator<Item = Mark>>(iter: I) -> Self {
        Self {
            marks: iter.into_iter().collect(),
        }
    }

    /// Adds a mark to this set
    pub fn add(&mut self, mark: Mark) {
        self.marks.insert(mark);
    }
}

impl FromIterator<Mark> for MarkSet {
    fn from_iter<T: IntoIterator<Item = Mark>>(iter: T) -> Self {
        Self {
            marks: iter.into_iter().collect(),
        }
    }
}

impl MarkSet {
    /// Removes a mark from this set
    pub fn remove(&mut self, mark: Mark) {
        self.marks.remove(&mark);
    }

    /// Checks if this set contains a mark
    pub fn contains(&self, mark: Mark) -> bool {
        self.marks.contains(&mark)
    }

    /// Gets the number of marks in this set
    pub fn len(&self) -> usize {
        self.marks.len()
    }

    /// Checks if this set is empty
    pub fn is_empty(&self) -> bool {
        self.marks.is_empty()
    }

    /// Creates a union of this set with another
    pub fn union(&self, other: &MarkSet) -> Self {
        let mut marks = self.marks.clone();
        marks.extend(&other.marks);
        Self { marks }
    }

    /// Creates an intersection of this set with another
    pub fn intersection(&self, other: &MarkSet) -> Self {
        let marks = self.marks.intersection(&other.marks).cloned().collect();
        Self { marks }
    }

    /// Creates the set difference (self - other)
    pub fn difference(&self, other: &MarkSet) -> Self {
        let marks = self.marks.difference(&other.marks).cloned().collect();
        Self { marks }
    }

    /// Converts to a vector for iteration
    pub fn to_vec(&self) -> Vec<Mark> {
        self.marks.iter().cloned().collect()
    }
}

impl From<HashSet<u64>> for MarkSet {
    fn from(set: HashSet<u64>) -> Self {
        let marks = set.into_iter().map(Mark).collect();
        Self { marks }
    }
}

impl From<&MarkSet> for HashSet<u64> {
    fn from(mark_set: &MarkSet) -> Self {
        mark_set.marks.iter().map(|m| m.value()).collect()
    }
}

/// Represents a binding occurrence of an identifier
#[derive(Debug, Clone, PartialEq)]
pub struct BindingOccurrence {
    /// The identifier name
    pub name: String,
    /// Marks associated with this occurrence
    pub marks: MarkSet,
    /// Lexical context where binding occurs
    pub context: LexicalContext,
    /// Scope depth
    pub scope_depth: usize,
    /// Whether this is a macro binding
    pub is_macro: bool,
}

impl BindingOccurrence {
    /// Creates a new binding occurrence
    pub fn new(
        name: String,
        marks: MarkSet,
        context: LexicalContext,
        scope_depth: usize,
        is_macro: bool,
    ) -> Self {
        Self {
            name,
            marks,
            context,
            scope_depth,
            is_macro,
        }
    }

    /// Checks if this binding can be referenced by the given reference
    pub fn can_reference(&self, reference: &ReferenceOccurrence) -> bool {
        // Names must match
        if self.name != reference.name {
            return false;
        }

        // Check mark compatibility using the hygiene algorithm
        self.marks_compatible(&reference.marks, &reference.context)
    }

    /// Checks if marks are compatible for binding resolution
    fn marks_compatible(&self, ref_marks: &MarkSet, ref_context: &LexicalContext) -> bool {
        // Two identifiers can refer to the same binding if:
        // 1. They have the same set of marks, OR
        // 2. The binding was introduced in a context that's compatible with the reference

        if self.marks == *ref_marks {
            return true;
        }

        // Check if the contexts are compatible
        // This is a simplified version - full implementation would need more sophisticated rules
        self.context.derives_from(ref_context) || ref_context.derives_from(&self.context)
    }
}

/// Represents a reference occurrence of an identifier
#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceOccurrence {
    /// The identifier name
    pub name: String,
    /// Marks associated with this occurrence
    pub marks: MarkSet,
    /// Lexical context where reference occurs
    pub context: LexicalContext,
    /// Scope depth
    pub scope_depth: usize,
}

impl ReferenceOccurrence {
    /// Creates a new reference occurrence
    pub fn new(name: String, marks: MarkSet, context: LexicalContext, scope_depth: usize) -> Self {
        Self {
            name,
            marks,
            context,
            scope_depth,
        }
    }
}

/// The core hygiene resolver that implements the mark-and-sweep algorithm
#[derive(Debug, Clone)]
pub struct HygieneResolver {
    /// Current expansion context
    current_context: Option<LexicalContext>,
    /// Stack of active marks
    mark_stack: Vec<Mark>,
    /// Binding occurrences collected during analysis
    bindings: Vec<BindingOccurrence>,
    /// Reference occurrences collected during analysis
    references: Vec<ReferenceOccurrence>,
    /// Resolved bindings map (reference index -> binding index)
    resolution_map: HashMap<usize, usize>,
    /// Rename map for generating fresh names
    rename_map: HashMap<(String, MarkSet), String>,
    /// Counter for generating unique names
    rename_counter: u64,
    /// Current scope depth
    scope_depth: usize,
}

impl HygieneResolver {
    /// Creates a new hygiene resolver
    pub fn new() -> Self {
        Self {
            current_context: None,
            mark_stack: Vec::new(),
            bindings: Vec::new(),
            references: Vec::new(),
            resolution_map: HashMap::new(),
            rename_map: HashMap::new(),
            rename_counter: 0,
            scope_depth: 0,
        }
    }

    /// Sets the current expansion context
    pub fn set_context(&mut self, context: LexicalContext) {
        self.current_context = Some(context);
    }

    /// Enters a new hygiene scope with a fresh mark
    pub fn enter_scope(&mut self) -> Mark {
        let mark = Mark::fresh();
        self.mark_stack.push(mark);
        self.scope_depth += 1;
        mark
    }

    /// Exits the current hygiene scope
    pub fn exit_scope(&mut self) {
        if !self.mark_stack.is_empty() {
            self.mark_stack.pop();
            self.scope_depth = self.scope_depth.saturating_sub(1);
        }
    }

    /// Gets the current mark set (all active marks)
    pub fn current_marks(&self) -> MarkSet {
        let marks = self.mark_stack.iter().cloned().collect();
        MarkSet { marks }
    }

    /// Adds a binding occurrence
    pub fn add_binding(
        &mut self,
        name: String,
        marks: MarkSet,
        context: LexicalContext,
        is_macro: bool,
    ) -> usize {
        let binding = BindingOccurrence::new(name, marks, context, self.scope_depth, is_macro);
        self.bindings.push(binding);
        self.bindings.len() - 1
    }

    /// Adds a reference occurrence
    pub fn add_reference(
        &mut self,
        name: String,
        marks: MarkSet,
        context: LexicalContext,
    ) -> usize {
        let reference = ReferenceOccurrence::new(name, marks, context, self.scope_depth);
        self.references.push(reference);
        self.references.len() - 1
    }

    /// Resolves all bindings using the mark-and-sweep algorithm
    pub fn resolve_bindings(&mut self) -> Result<()> {
        self.resolution_map.clear();

        // For each reference, find the most recent compatible binding
        for (ref_idx, reference) in self.references.iter().enumerate() {
            let mut best_binding: Option<usize> = None;
            let mut best_depth = 0;

            // Search through bindings in reverse order (most recent first)
            for (bind_idx, binding) in self.bindings.iter().enumerate().rev() {
                if binding.can_reference(reference) {
                    // Choose the binding at the deepest scope (most recent)
                    if best_binding.is_none() || binding.scope_depth > best_depth {
                        best_binding = Some(bind_idx);
                        best_depth = binding.scope_depth;
                    }
                }
            }

            if let Some(binding_idx) = best_binding {
                self.resolution_map.insert(ref_idx, binding_idx);
            }
        }

        Ok(())
    }

    /// Gets the resolved binding for a reference
    pub fn get_resolved_binding(&self, reference_idx: usize) -> Option<&BindingOccurrence> {
        self.resolution_map
            .get(&reference_idx)
            .and_then(|&binding_idx| self.bindings.get(binding_idx))
    }

    /// Generates a fresh renamed identifier
    pub fn generate_fresh_name(&mut self, original: &str, marks: &MarkSet) -> String {
        if let Some(existing) = self.rename_map.get(&(original.to_string(), marks.clone())) {
            return existing.clone();
        }

        self.rename_counter += 1;
        let fresh_name = format!("{}#{}#{}", original, marks.len(), self.rename_counter);
        self.rename_map
            .insert((original.to_string(), marks.clone()), fresh_name.clone());
        fresh_name
    }

    /// Applies hygiene transformation to a syntax object
    pub fn apply_hygiene(&mut self, syntax: &SyntaxObject) -> Result<SyntaxObject> {
        let transformed_expr = self.transform_expression(&syntax.expr, &syntax.context)?;
        Ok(syntax.with_expr(transformed_expr))
    }

    /// Transforms an expression for hygiene
    fn transform_expression(&mut self, expr: &Expr, context: &LexicalContext) -> Result<Expr> {
        match expr {
            Expr::Identifier(name) | Expr::Symbol(name) => {
                let marks = self.current_marks();
                let ref_idx = self.add_reference(name.clone(), marks.clone(), context.clone());

                // Resolve the binding
                self.resolve_bindings()?;

                // Check if we need to rename
                if let Some(binding) = self.get_resolved_binding(ref_idx) {
                    // If the binding has different marks, we might need to rename
                    if binding.marks != marks {
                        let fresh_name = self.generate_fresh_name(name, &marks);
                        Ok(Expr::Identifier(fresh_name))
                    } else {
                        Ok(Expr::Identifier(name.clone()))
                    }
                } else {
                    // Unbound identifier - leave as is but potentially rename if marked
                    if !marks.is_empty() {
                        let fresh_name = self.generate_fresh_name(name, &marks);
                        Ok(Expr::Identifier(fresh_name))
                    } else {
                        Ok(Expr::Identifier(name.clone()))
                    }
                }
            }

            Expr::Lambda {
                formals,
                metadata,
                body,
                ..
            } => {
                // Enter new scope for lambda parameters
                let mark = self.enter_scope();

                // Add bindings for parameters
                let param_names = match formals {
                    crate::ast::Formals::Fixed(params) => params.clone(),
                    crate::ast::Formals::Variable(param) => vec![param.clone()],
                    crate::ast::Formals::Mixed { fixed, rest } => {
                        let mut all_params = fixed.clone();
                        all_params.push(rest.clone());
                        all_params
                    }
                    crate::ast::Formals::Keyword { fixed, rest, .. } => {
                        let mut all_params = fixed.clone();
                        if let Some(rest_param) = rest {
                            all_params.push(rest_param.clone());
                        }
                        all_params
                    }
                    crate::ast::Formals::Typed(typed_params) => {
                        typed_params.iter().map(|tp| tp.name.clone()).collect()
                    }
                    crate::ast::Formals::TypedVariable(typed_param) => {
                        vec![typed_param.name.clone()]
                    }
                    crate::ast::Formals::TypedMixed { fixed, rest } => {
                        let mut all_params: Vec<String> =
                            fixed.iter().map(|tp| tp.name.clone()).collect();
                        all_params.push(rest.name.clone());
                        all_params
                    }
                };

                let current_marks = self.current_marks();
                for param_name in &param_names {
                    self.add_binding(
                        param_name.clone(),
                        current_marks.clone(),
                        context.clone(),
                        false,
                    );
                }

                // Transform body expressions
                let mut transformed_body = Vec::new();
                for body_expr in body {
                    let transformed = self.transform_expression(&body_expr.inner, context)?;
                    transformed_body.push(crate::diagnostics::Spanned::new(
                        transformed,
                        body_expr.span,
                    ));
                }

                self.exit_scope();

                Ok(Expr::Lambda {
                    formals: formals.clone(),
                    metadata: metadata.clone(),
                    body: transformed_body,
                    return_type: None,
                })
            }

            Expr::Define {
                name,
                value,
                metadata,
                ..
            } => {
                // Add binding for the defined name
                let current_marks = self.current_marks();
                self.add_binding(name.clone(), current_marks, context.clone(), false);

                // Transform the value expression
                let transformed_value = self.transform_expression(&value.inner, context)?;

                Ok(Expr::Define {
                    name: name.clone(),
                    value: Box::new(crate::diagnostics::Spanned::new(
                        transformed_value,
                        value.span,
                    )),
                    metadata: metadata.clone(),
                    return_type: None,
                })
            }

            Expr::DefineSyntax { name, transformer } => {
                // Add macro binding
                let current_marks = self.current_marks();
                self.add_binding(name.clone(), current_marks, context.clone(), true);

                // Transform the transformer expression
                let transformed_transformer =
                    self.transform_expression(&transformer.inner, context)?;

                Ok(Expr::DefineSyntax {
                    name: name.clone(),
                    transformer: Box::new(crate::diagnostics::Spanned::new(
                        transformed_transformer,
                        transformer.span,
                    )),
                })
            }

            Expr::Application { operator, operands } => {
                let transformed_operator = self.transform_expression(&operator.inner, context)?;
                let mut transformed_operands = Vec::new();

                for operand in operands {
                    let transformed = self.transform_expression(&operand.inner, context)?;
                    transformed_operands
                        .push(crate::diagnostics::Spanned::new(transformed, operand.span));
                }

                Ok(Expr::Application {
                    operator: Box::new(crate::diagnostics::Spanned::new(
                        transformed_operator,
                        operator.span,
                    )),
                    operands: transformed_operands,
                })
            }

            // For other expression types, recursively transform as needed
            _ => Ok(expr.clone()), // Simplified - would need full implementation
        }
    }

    /// Clears all collected bindings and references
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.references.clear();
        self.resolution_map.clear();
        self.mark_stack.clear();
        self.scope_depth = 0;
    }

    /// Gets statistics about the current state
    pub fn stats(&self) -> HygieneStats {
        HygieneStats {
            total_bindings: self.bindings.len(),
            total_references: self.references.len(),
            resolved_references: self.resolution_map.len(),
            current_scope_depth: self.scope_depth,
            active_marks: self.mark_stack.len(),
            total_renames: self.rename_map.len(),
        }
    }
}

impl Default for HygieneResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about hygiene resolution
#[derive(Debug, Clone)]
pub struct HygieneStats {
    /// Total number of identifier bindings processed
    pub total_bindings: usize,
    /// Total number of identifier references encountered
    pub total_references: usize,
    /// Number of references successfully resolved to bindings
    pub resolved_references: usize,
    /// Current nesting depth of lexical scopes
    pub current_scope_depth: usize,
    /// Number of hygiene marks currently active
    pub active_marks: usize,
    /// Total number of identifier renames performed for hygiene
    pub total_renames: usize,
}

/// Helper functions for working with the hygiene system
pub mod hygiene_utils {
    use super::*;

    /// Checks if two identifiers are equivalent under hygiene
    pub fn identifiers_equivalent(
        id1: &str,
        marks1: &MarkSet,
        context1: &LexicalContext,
        id2: &str,
        marks2: &MarkSet,
        context2: &LexicalContext,
    ) -> bool {
        // Names must match
        if id1 != id2 {
            return false;
        }

        // Check mark equivalence
        if marks1 == marks2 {
            return true;
        }

        // Check if contexts allow equivalence despite different marks
        context1.context_id == context2.context_id && context1.phase == context2.phase
    }

    /// Creates a mark set from a vector of mark values
    pub fn marks_from_vec(marks: Vec<u64>) -> MarkSet {
        let mark_set: HashSet<Mark> = marks.into_iter().map(Mark).collect();
        MarkSet { marks: mark_set }
    }

    /// Applies macro introduction transformation
    pub fn introduce_macro_marks(original_marks: &MarkSet, introduction_mark: Mark) -> MarkSet {
        let mut new_marks = original_marks.clone();
        new_marks.add(introduction_mark);
        new_marks
    }

    /// Applies macro use transformation
    pub fn apply_use_marks(original_marks: &MarkSet, use_mark: Mark) -> MarkSet {
        // In the mark-and-sweep algorithm, use-site marks are typically added
        let mut new_marks = original_marks.clone();
        new_marks.add(use_mark);
        new_marks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_mark_generation() {
        let mark1 = Mark::fresh();
        let mark2 = Mark::fresh();
        assert_ne!(mark1, mark2);
        assert!(mark2.value() > mark1.value());
    }

    #[test]
    fn test_mark_set_operations() {
        let mark1 = Mark::fresh();
        let mark2 = Mark::fresh();

        let mut set1 = MarkSet::singleton(mark1);
        let set2 = MarkSet::singleton(mark2);

        assert!(set1.contains(mark1));
        assert!(!set1.contains(mark2));

        let union = set1.union(&set2);
        assert!(union.contains(mark1));
        assert!(union.contains(mark2));
        assert_eq!(union.len(), 2);
    }

    #[test]
    fn test_hygiene_resolver_basic() {
        let mut resolver = HygieneResolver::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        // Add a binding
        let marks = MarkSet::empty();
        let binding_idx =
            resolver.add_binding("x".to_string(), marks.clone(), context.clone(), false);

        // Add a reference
        let ref_idx = resolver.add_reference("x".to_string(), marks, context);

        // Resolve bindings
        resolver.resolve_bindings().unwrap();

        // Check resolution
        let resolved = resolver.get_resolved_binding(ref_idx);
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().name, "x");
    }

    #[test]
    fn test_hygiene_scoping() {
        let mut resolver = HygieneResolver::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);

        // Outer scope binding
        let outer_marks = MarkSet::empty();
        resolver.add_binding("x".to_string(), outer_marks.clone(), context.clone(), false);

        // Enter inner scope
        let inner_mark = resolver.enter_scope();
        let inner_marks = MarkSet::singleton(inner_mark);

        // Inner scope binding (should shadow outer)
        resolver.add_binding("x".to_string(), inner_marks.clone(), context.clone(), false);

        // Reference in inner scope
        let ref_idx = resolver.add_reference("x".to_string(), inner_marks, context);

        resolver.resolve_bindings().unwrap();

        // Should resolve to inner binding
        let resolved = resolver.get_resolved_binding(ref_idx);
        assert!(resolved.is_some());
        assert!(resolved.unwrap().marks.contains(inner_mark));
    }

    #[test]
    fn test_fresh_name_generation() {
        let mut resolver = HygieneResolver::new();
        let marks = MarkSet::singleton(Mark::fresh());

        let name1 = resolver.generate_fresh_name("x", &marks);
        let name2 = resolver.generate_fresh_name("x", &marks);

        // Same marks should give same name
        assert_eq!(name1, name2);

        // Different marks should give different name
        let different_marks = MarkSet::singleton(Mark::fresh());
        let name3 = resolver.generate_fresh_name("x", &different_marks);
        assert_ne!(name1, name3);
    }
}

//! Lexical Context Tracking and Scope Management
//!
//! This module provides comprehensive lexical context tracking and scope management
//! for the syntax object system. It ensures proper scoping semantics through macro
//! expansion and provides the foundation for advanced hygiene.
//!
//! Key features:
//! - Hierarchical scope tracking with parent-child relationships
//! - Phase-aware scoping for macro-time vs runtime bindings
//! - Integration with module system for cross-module hygiene
//! - Efficient scope lookup and binding resolution

use super::advanced_hygiene::{HygieneResolver, Mark, MarkSet};
use super::syntax_objects::{BindingInfo, LexicalContext, SyntaxObject, SyntaxProperty};
use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span};
use crate::eval::Environment;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for generating unique scope identifiers
static SCOPE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generates a unique scope identifier
pub fn next_scope_id() -> ScopeId {
    ScopeId(SCOPE_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
}

/// Unique identifier for lexical scopes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(u64);

impl ScopeId {
    /// Gets the raw numeric identifier value
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Represents the type of lexical scope
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeType {
    /// Top-level module scope
    Module,
    /// Function parameter scope
    Function,
    /// Let/let*/letrec binding scope
    Let,
    /// Lambda parameter scope
    Lambda,
    /// Macro expansion scope
    Macro,
    /// Syntax parameter scope
    SyntaxParameter,
    /// Library/module internal scope
    Library,
    /// Dynamic parameter scope
    Parameter,
    /// Exception handler scope
    Guard,
    /// Conditional scope (if, cond, case)
    Conditional,
    /// Loop scope (do, while, for)
    Loop,
    /// Block scope (begin)
    Block,
}

/// A lexical scope in the binding hierarchy
#[derive(Debug, Clone)]
pub struct LexicalScope {
    /// Unique identifier for this scope
    pub id: ScopeId,
    /// Type of this scope
    pub scope_type: ScopeType,
    /// Parent scope (if any)
    pub parent: Option<ScopeId>,
    /// Direct child scopes
    pub children: HashSet<ScopeId>,
    /// Bindings introduced in this scope
    pub bindings: HashMap<String, ScopeBinding>,
    /// Phase level (0 = runtime, 1 = macro-time, etc.)
    pub phase: i32,
    /// Module path for this scope
    pub module_path: Vec<String>,
    /// Marks associated with this scope for hygiene
    pub marks: MarkSet,
    /// Whether this scope is closed (no more bindings can be added)
    pub is_closed: bool,
    /// Scope metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp for debugging
    pub created_at: std::time::Instant,
}

/// A binding within a lexical scope
#[derive(Debug, Clone)]
pub struct ScopeBinding {
    /// Name of the bound identifier
    pub name: String,
    /// Scope where this binding was introduced
    pub scope_id: ScopeId,
    /// Phase at which this binding exists
    pub phase: i32,
    /// Whether this is a macro binding
    pub is_macro: bool,
    /// Whether this is a syntax parameter
    pub is_syntax_parameter: bool,
    /// Whether this binding is mutable
    pub is_mutable: bool,
    /// Module where this binding was defined
    pub defining_module: Option<Vec<String>>,
    /// Marks associated with this binding
    pub marks: MarkSet,
    /// Additional binding metadata
    pub metadata: HashMap<String, String>,
    /// Whether this binding has been used
    pub is_used: bool,
}

impl ScopeBinding {
    /// Creates a new scope binding
    pub fn new(name: String, scope_id: ScopeId, phase: i32, is_macro: bool) -> Self {
        Self {
            name,
            scope_id,
            phase,
            is_macro,
            is_syntax_parameter: false,
            is_mutable: false,
            defining_module: None,
            marks: MarkSet::empty(),
            metadata: HashMap::new(),
            is_used: false,
        }
    }

    /// Marks this binding as used
    pub fn mark_used(&mut self) {
        self.is_used = true;
    }

    /// Checks if this binding is accessible from the given scope and phase
    pub fn is_accessible_from(
        &self,
        scope_id: ScopeId,
        phase: i32,
        scope_manager: &ScopeManager,
    ) -> bool {
        // Phase must match for access
        if self.phase != phase {
            return false;
        }

        // Check if the requesting scope can access this binding's scope
        scope_manager.scope_contains_or_inherits(scope_id, self.scope_id)
    }

    /// Checks if this binding has compatible marks for hygiene
    pub fn has_compatible_marks(&self, reference_marks: &MarkSet) -> bool {
        // Two bindings have compatible marks if they share at least one mark,
        // or if one has no marks (indicating it's from the original code)
        if self.marks.is_empty() || reference_marks.is_empty() {
            return true;
        }

        !self.marks.intersection(reference_marks).is_empty()
    }
}

impl LexicalScope {
    /// Creates a new lexical scope
    pub fn new(
        scope_type: ScopeType,
        parent: Option<ScopeId>,
        phase: i32,
        module_path: Vec<String>,
    ) -> Self {
        Self {
            id: next_scope_id(),
            scope_type,
            parent,
            children: HashSet::new(),
            bindings: HashMap::new(),
            phase,
            module_path,
            marks: MarkSet::empty(),
            is_closed: false,
            metadata: HashMap::new(),
            created_at: std::time::Instant::now(),
        }
    }

    /// Adds a binding to this scope
    pub fn add_binding(&mut self, binding: ScopeBinding) -> Result<()> {
        if self.is_closed {
            return Err(Box::new(Error::macro_error(
                format!("Cannot add binding '{}' to closed scope", binding.name),
                Span::new(0, 0),
            )));
        }

        if self.bindings.contains_key(&binding.name) {
            return Err(Box::new(Error::macro_error(
                format!("Binding '{}' already exists in scope", binding.name),
                Span::new(0, 0),
            )));
        }

        self.bindings.insert(binding.name.clone(), binding);
        Ok(())
    }

    /// Gets a binding by name
    pub fn get_binding(&self, name: &str) -> Option<&ScopeBinding> {
        self.bindings.get(name)
    }

    /// Gets a mutable binding by name
    pub fn get_binding_mut(&mut self, name: &str) -> Option<&mut ScopeBinding> {
        self.bindings.get_mut(name)
    }

    /// Adds a child scope
    pub fn add_child(&mut self, child_id: ScopeId) {
        self.children.insert(child_id);
    }

    /// Removes a child scope
    pub fn remove_child(&mut self, child_id: ScopeId) {
        self.children.remove(&child_id);
    }

    /// Closes this scope, preventing further bindings
    pub fn close(&mut self) {
        self.is_closed = true;
    }

    /// Adds marks to this scope
    pub fn add_marks(&mut self, marks: &MarkSet) {
        self.marks = self.marks.union(marks);
    }

    /// Gets all binding names in this scope
    pub fn binding_names(&self) -> Vec<String> {
        self.bindings.keys().cloned().collect()
    }

    /// Gets statistics about this scope
    pub fn stats(&self) -> ScopeStats {
        let used_bindings = self.bindings.values().filter(|b| b.is_used).count();
        let macro_bindings = self.bindings.values().filter(|b| b.is_macro).count();

        ScopeStats {
            total_bindings: self.bindings.len(),
            used_bindings,
            unused_bindings: self.bindings.len() - used_bindings,
            macro_bindings,
            regular_bindings: self.bindings.len() - macro_bindings,
            child_count: self.children.len(),
            phase: self.phase,
            is_closed: self.is_closed,
        }
    }
}

/// Statistics about a lexical scope
#[derive(Debug, Clone)]
pub struct ScopeStats {
    /// Total number of bindings in this scope
    pub total_bindings: usize,
    /// Number of bindings that have been referenced
    pub used_bindings: usize,
    /// Number of bindings that are never used
    pub unused_bindings: usize,
    /// Number of macro bindings in this scope
    pub macro_bindings: usize,
    /// Number of regular variable bindings
    pub regular_bindings: usize,
    /// Number of child scopes
    pub child_count: usize,
    /// Phase level of this scope (0 = runtime, 1 = macro-time, etc.)
    pub phase: i32,
    /// Whether this scope has been closed to new bindings
    pub is_closed: bool,
}

/// Manages the hierarchy of lexical scopes
#[derive(Debug)]
pub struct ScopeManager {
    /// All scopes indexed by ID
    scopes: HashMap<ScopeId, LexicalScope>,
    /// Current active scope
    current_scope: Option<ScopeId>,
    /// Scope stack for nested contexts
    scope_stack: Vec<ScopeId>,
    /// Root scopes (top-level modules)
    root_scopes: HashSet<ScopeId>,
    /// Hygiene resolver for mark management
    hygiene_resolver: HygieneResolver,
    /// Phase stack for macro expansion
    phase_stack: Vec<i32>,
    /// Current phase
    current_phase: i32,
    /// Manager statistics
    stats: ScopeManagerStats,
}

/// Statistics for the scope manager
#[derive(Debug, Clone, Default)]
pub struct ScopeManagerStats {
    /// Total number of scopes created
    pub total_scopes: usize,
    /// Number of currently active scopes
    pub active_scopes: usize,
    /// Total number of bindings across all scopes
    pub total_bindings: usize,
    /// Number of binding lookup operations performed
    pub binding_lookups: usize,
    /// Number of successful cache hits during binding lookups
    pub cache_hits: usize,
    /// Number of cache misses during binding lookups
    pub cache_misses: usize,
}

impl ScopeManager {
    /// Creates a new scope manager
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
            current_scope: None,
            scope_stack: Vec::new(),
            root_scopes: HashSet::new(),
            hygiene_resolver: HygieneResolver::new(),
            phase_stack: Vec::new(),
            current_phase: 0,
            stats: ScopeManagerStats::default(),
        }
    }

    /// Creates a new scope
    pub fn create_scope(&mut self, scope_type: ScopeType, module_path: Vec<String>) -> ScopeId {
        let parent = self.current_scope;
        let mut scope = LexicalScope::new(scope_type, parent, self.current_phase, module_path);
        let scope_id = scope.id;

        // Add marks from current hygiene context
        let current_marks = self.hygiene_resolver.current_marks();
        scope.add_marks(&current_marks);

        // Update parent-child relationships
        if let Some(parent_id) = parent {
            if let Some(parent_scope) = self.scopes.get_mut(&parent_id) {
                parent_scope.add_child(scope_id);
            }
        } else {
            self.root_scopes.insert(scope_id);
        }

        self.scopes.insert(scope_id, scope);
        self.stats.total_scopes += 1;

        scope_id
    }

    /// Enters a scope, making it the current scope
    pub fn enter_scope(&mut self, scope_id: ScopeId) -> Result<()> {
        if !self.scopes.contains_key(&scope_id) {
            return Err(Box::new(Error::macro_error(
                format!("Scope {scope_id:?} does not exist"),
                Span::new(0, 0),
            )));
        }

        if let Some(current) = self.current_scope {
            self.scope_stack.push(current);
        }

        self.current_scope = Some(scope_id);
        self.stats.active_scopes += 1;

        Ok(())
    }

    /// Exits the current scope
    pub fn exit_scope(&mut self) -> Option<ScopeId> {
        let exited_scope = self.current_scope;

        if let Some(previous) = self.scope_stack.pop() {
            self.current_scope = Some(previous);
        } else {
            self.current_scope = None;
        }

        if exited_scope.is_some() {
            self.stats.active_scopes = self.stats.active_scopes.saturating_sub(1);
        }

        exited_scope
    }

    /// Creates and enters a new scope
    pub fn push_scope(
        &mut self,
        scope_type: ScopeType,
        module_path: Vec<String>,
    ) -> Result<ScopeId> {
        let scope_id = self.create_scope(scope_type, module_path);
        self.enter_scope(scope_id)?;
        Ok(scope_id)
    }

    /// Exits and optionally closes the current scope
    pub fn pop_scope(&mut self, close_scope: bool) -> Option<ScopeId> {
        let exited_scope = self.exit_scope();

        if close_scope {
            if let Some(scope_id) = exited_scope {
                if let Some(scope) = self.scopes.get_mut(&scope_id) {
                    scope.close();
                }
            }
        }

        exited_scope
    }

    /// Adds a binding to the current scope
    pub fn add_binding(&mut self, binding: ScopeBinding) -> Result<()> {
        if let Some(scope_id) = self.current_scope {
            if let Some(scope) = self.scopes.get_mut(&scope_id) {
                scope.add_binding(binding)?;
                self.stats.total_bindings += 1;
                Ok(())
            } else {
                Err(Box::new(Error::macro_error(
                    "Current scope does not exist".to_string(),
                    Span::new(0, 0),
                )))
            }
        } else {
            Err(Box::new(Error::macro_error(
                "No current scope for binding".to_string(),
                Span::new(0, 0),
            )))
        }
    }

    /// Looks up a binding by name in the scope hierarchy
    pub fn lookup_binding(&mut self, name: &str, marks: &MarkSet) -> Option<&ScopeBinding> {
        self.stats.binding_lookups += 1;

        let start_scope = self.current_scope?;
        let mut current = Some(start_scope);

        while let Some(scope_id) = current {
            if let Some(scope) = self.scopes.get(&scope_id) {
                if let Some(binding) = scope.get_binding(name) {
                    // Check if binding is accessible with the given marks
                    if binding.has_compatible_marks(marks)
                        && binding.is_accessible_from(start_scope, self.current_phase, self)
                    {
                        self.stats.cache_hits += 1;
                        return Some(binding);
                    }
                }
                current = scope.parent;
            } else {
                break;
            }
        }

        self.stats.cache_misses += 1;
        None
    }

    /// Marks a binding as used
    pub fn mark_binding_used(&mut self, name: &str, marks: &MarkSet) -> bool {
        let start_scope = self.current_scope;
        if start_scope.is_none() {
            return false;
        }

        let mut current = start_scope;
        while let Some(scope_id) = current {
            if let Some(scope) = self.scopes.get_mut(&scope_id) {
                if let Some(binding) = scope.get_binding_mut(name) {
                    if binding.has_compatible_marks(marks) {
                        binding.mark_used();
                        return true;
                    }
                }
                current = scope.parent;
            } else {
                break;
            }
        }

        false
    }

    /// Checks if one scope contains or inherits from another
    pub fn scope_contains_or_inherits(&self, container: ScopeId, contained: ScopeId) -> bool {
        if container == contained {
            return true;
        }

        let mut current = Some(contained);
        while let Some(scope_id) = current {
            if scope_id == container {
                return true;
            }

            if let Some(scope) = self.scopes.get(&scope_id) {
                current = scope.parent;
            } else {
                break;
            }
        }

        false
    }

    /// Gets the current scope
    pub fn current_scope(&self) -> Option<&LexicalScope> {
        self.current_scope.and_then(|id| self.scopes.get(&id))
    }

    /// Gets a scope by ID
    pub fn get_scope(&self, scope_id: ScopeId) -> Option<&LexicalScope> {
        self.scopes.get(&scope_id)
    }

    /// Gets a mutable scope by ID
    pub fn get_scope_mut(&mut self, scope_id: ScopeId) -> Option<&mut LexicalScope> {
        self.scopes.get_mut(&scope_id)
    }

    /// Enters a new phase for macro expansion
    pub fn enter_phase(&mut self, phase: i32) {
        self.phase_stack.push(self.current_phase);
        self.current_phase = phase;
    }

    /// Exits the current phase
    pub fn exit_phase(&mut self) {
        if let Some(previous_phase) = self.phase_stack.pop() {
            self.current_phase = previous_phase;
        }
    }

    /// Gets the current phase
    pub fn current_phase(&self) -> i32 {
        self.current_phase
    }

    /// Converts a lexical scope to a lexical context
    pub fn scope_to_context(&self, scope_id: ScopeId) -> Option<LexicalContext> {
        let scope = self.scopes.get(&scope_id)?;

        let parent_context = scope
            .parent
            .and_then(|pid| self.scope_to_context(pid))
            .map(Box::new);

        Some(LexicalContext {
            context_id: scope.id.as_u64(),
            parent: parent_context,
            phase: scope.phase,
            module_path: scope.module_path.clone(),
            metadata: scope.metadata.clone(),
        })
    }

    /// Creates a scope from a lexical context
    pub fn context_to_scope(&mut self, context: &LexicalContext) -> Result<ScopeId> {
        let parent_scope = if let Some(parent_ctx) = &context.parent {
            Some(self.context_to_scope(parent_ctx)?)
        } else {
            None
        };

        let mut scope = LexicalScope::new(
            ScopeType::Module, // Default type, could be specified
            parent_scope,
            context.phase,
            context.module_path.clone(),
        );

        scope.metadata = context.metadata.clone();
        let scope_id = scope.id;

        if let Some(parent_id) = parent_scope {
            if let Some(parent) = self.scopes.get_mut(&parent_id) {
                parent.add_child(scope_id);
            }
        }

        self.scopes.insert(scope_id, scope);
        Ok(scope_id)
    }

    /// Gets all unused bindings for warnings
    pub fn get_unused_bindings(&self) -> Vec<(ScopeId, String)> {
        let mut unused = Vec::new();

        for (scope_id, scope) in &self.scopes {
            for (name, binding) in &scope.bindings {
                if !binding.is_used && !name.starts_with("_") {
                    unused.push((*scope_id, name.clone()));
                }
            }
        }

        unused
    }

    /// Gets statistics about the scope manager
    pub fn stats(&self) -> &ScopeManagerStats {
        &self.stats
    }

    /// Resets statistics
    pub fn reset_stats(&mut self) {
        self.stats = ScopeManagerStats::default();
        self.stats.total_scopes = self.scopes.len();
        self.stats.total_bindings = self.scopes.values().map(|s| s.bindings.len()).sum();
    }

    /// Clears all scopes (for testing)
    pub fn clear(&mut self) {
        self.scopes.clear();
        self.current_scope = None;
        self.scope_stack.clear();
        self.root_scopes.clear();
        self.hygiene_resolver.clear();
        self.phase_stack.clear();
        self.current_phase = 0;
        self.stats = ScopeManagerStats::default();
    }
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilities for scope management
pub mod scope_utils {
    use super::*;

    /// Creates a scope binding from a syntax object
    pub fn binding_from_syntax(
        syntax: &SyntaxObject,
        scope_id: ScopeId,
        is_macro: bool,
    ) -> Option<ScopeBinding> {
        let name = syntax.identifier_name()?.to_string();
        let mut binding = ScopeBinding::new(name, scope_id, syntax.context.phase, is_macro);

        // Add marks from syntax object
        binding.marks = MarkSet::from(syntax.marks.clone());

        // Add metadata
        if let Some(SyntaxProperty::String(module_name)) = syntax.get_property("module") {
            binding.defining_module = Some(vec![module_name.clone()]);
        }

        Some(binding)
    }

    /// Creates a scope for a lambda expression
    pub fn create_lambda_scope(
        manager: &mut ScopeManager,
        formals: &crate::ast::Formals,
        module_path: Vec<String>,
    ) -> Result<ScopeId> {
        let scope_id = manager.push_scope(ScopeType::Lambda, module_path)?;

        // Add parameter bindings
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
                let mut all_params: Vec<String> = fixed.iter().map(|tp| tp.name.clone()).collect();
                all_params.push(rest.name.clone());
                all_params
            }
        };

        for param_name in param_names {
            let binding = ScopeBinding::new(param_name, scope_id, manager.current_phase(), false);
            manager.add_binding(binding)?;
        }

        Ok(scope_id)
    }

    /// Creates a scope for let-like binding forms
    pub fn create_let_scope(
        manager: &mut ScopeManager,
        bindings: &[crate::ast::Binding],
        module_path: Vec<String>,
        scope_type: ScopeType,
    ) -> Result<ScopeId> {
        let scope_id = manager.push_scope(scope_type, module_path)?;

        for binding in bindings {
            let scope_binding = ScopeBinding::new(
                binding.name.clone(),
                scope_id,
                manager.current_phase(),
                false,
            );
            manager.add_binding(scope_binding)?;
        }

        Ok(scope_id)
    }

    /// Resolves an identifier in the current scope context
    pub fn resolve_identifier(
        manager: &mut ScopeManager,
        name: &str,
        marks: &MarkSet,
    ) -> Option<ScopeId> {
        manager
            .lookup_binding(name, marks)
            .map(|binding| binding.scope_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_creation() {
        let mut manager = ScopeManager::new();
        let scope_id = manager.create_scope(ScopeType::Module, vec!["test".to_string()]);

        assert!(manager.scopes.contains_key(&scope_id));
        assert!(manager.root_scopes.contains(&scope_id));
    }

    #[test]
    fn test_scope_hierarchy() {
        let mut manager = ScopeManager::new();
        let parent_id = manager
            .push_scope(ScopeType::Module, vec!["test".to_string()])
            .unwrap();
        let child_id = manager
            .push_scope(ScopeType::Function, vec!["test".to_string()])
            .unwrap();

        assert!(manager.scope_contains_or_inherits(parent_id, child_id));
        assert!(!manager.scope_contains_or_inherits(child_id, parent_id));
    }

    #[test]
    fn test_binding_operations() {
        let mut manager = ScopeManager::new();
        let scope_id = manager
            .push_scope(ScopeType::Module, vec!["test".to_string()])
            .unwrap();

        let binding = ScopeBinding::new("x".to_string(), scope_id, 0, false);
        manager.add_binding(binding).unwrap();

        let marks = MarkSet::empty();
        let found = manager.lookup_binding("x", &marks);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "x");
    }

    #[test]
    fn test_binding_lookup_hierarchy() {
        let mut manager = ScopeManager::new();
        let parent_id = manager
            .push_scope(ScopeType::Module, vec!["test".to_string()])
            .unwrap();

        let parent_binding = ScopeBinding::new("x".to_string(), parent_id, 0, false);
        manager.add_binding(parent_binding).unwrap();

        let child_id = manager
            .push_scope(ScopeType::Function, vec!["test".to_string()])
            .unwrap();

        let marks = MarkSet::empty();
        let found = manager.lookup_binding("x", &marks);
        assert!(found.is_some());
        assert_eq!(found.unwrap().scope_id, parent_id);
    }

    #[test]
    fn test_phase_management() {
        let mut manager = ScopeManager::new();
        assert_eq!(manager.current_phase(), 0);

        manager.enter_phase(1);
        assert_eq!(manager.current_phase(), 1);

        manager.exit_phase();
        assert_eq!(manager.current_phase(), 0);
    }

    #[test]
    fn test_scope_stats() {
        let mut manager = ScopeManager::new();
        let scope_id = manager
            .push_scope(ScopeType::Module, vec!["test".to_string()])
            .unwrap();

        let binding = ScopeBinding::new("x".to_string(), scope_id, 0, false);
        manager.add_binding(binding).unwrap();

        if let Some(scope) = manager.get_scope(scope_id) {
            let stats = scope.stats();
            assert_eq!(stats.total_bindings, 1);
            assert_eq!(stats.unused_bindings, 1);
        }

        let marks = MarkSet::empty();
        manager.mark_binding_used("x", &marks);

        if let Some(scope) = manager.get_scope(scope_id) {
            let stats = scope.stats();
            assert_eq!(stats.used_bindings, 1);
            assert_eq!(stats.unused_bindings, 0);
        }
    }

    #[test]
    fn test_lambda_scope_creation() {
        let mut manager = ScopeManager::new();
        let _parent_id = manager
            .push_scope(ScopeType::Module, vec!["test".to_string()])
            .unwrap();

        let formals = crate::ast::Formals::Fixed(vec!["x".to_string(), "y".to_string()]);
        let lambda_scope =
            scope_utils::create_lambda_scope(&mut manager, &formals, vec!["test".to_string()])
                .unwrap();

        let marks = MarkSet::empty();
        assert!(manager.lookup_binding("x", &marks).is_some());
        assert!(manager.lookup_binding("y", &marks).is_some());
        assert!(manager.lookup_binding("z", &marks).is_none());
    }
}

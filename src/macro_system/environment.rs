//! Macro environment management for Lambdust.
//!
//! This module implements the macro environment system, which is separate from
//! the runtime environment and manages macro definitions and their scoping.
//! Macro environments handle the lexical scoping of macro definitions and
//! support proper macro visibility rules.

use super::{MacroTransformer, MacroContext, next_hygiene_id};
// use crate::diagnostics::{Error, Result, Span};
// use crate::eval::Environment;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// A macro environment that manages macro definitions.
#[derive(Debug)]
pub struct MacroEnvironment {
    /// The parent environment (for lexical scoping)
    parent: Option<Rc<MacroEnvironment>>,
    /// Macro definitions in this environment
    macros: RefCell<HashMap<String, MacroTransformer>>,
    /// Environment ID for debugging
    id: u64,
    /// Generation for garbage collection
    generation: u64,
}

impl MacroEnvironment {
    /// Creates a new empty macro environment.
    pub fn new() -> Self {
        Self {
            parent: None,
            macros: RefCell::new(HashMap::new()),
            id: next_hygiene_id(),
            generation: 0,
        }
    }
    
    /// Creates a new macro environment with a parent.
    pub fn with_parent(parent: Rc<MacroEnvironment>) -> Self {
        Self {
            parent: Some(parent),
            macros: RefCell::new(HashMap::new()),
            id: next_hygiene_id(),
            generation: 0,
        }
    }
    
    /// Creates a child environment.
    pub fn extend(self: &Rc<Self>) -> Rc<MacroEnvironment> {
        Rc::new(MacroEnvironment::with_parent(self.clone()))
    }
    
    /// Defines a macro in this environment.
    pub fn define(&self, name: String, transformer: MacroTransformer) {
        self.macros.borrow_mut().insert(name, transformer);
    }
    
    /// Looks up a macro by name.
    pub fn lookup(&self, name: &str) -> Option<MacroTransformer> {
        // First check this environment
        if let Some(transformer) = self.macros.borrow().get(name) {
            return Some(transformer.clone());
        }
        
        // Then check parent environments
        if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
    
    /// Checks if a macro is defined in this environment (not parent).
    pub fn locally_defined(&self, name: &str) -> bool {
        self.macros.borrow().contains_key(name)
    }
    
    /// Removes a macro definition from this environment.
    pub fn undefine(&self, name: &str) -> bool {
        self.macros.borrow_mut().remove(name).is_some()
    }
    
    /// Gets all macro names defined in this environment.
    pub fn local_names(&self) -> Vec<String> {
        self.macros.borrow().keys().cloned().collect()
    }
    
    /// Gets all macro names visible in this environment.
    pub fn all_names(&self) -> Vec<String> {
        let mut names = self.local_names();
        
        if let Some(parent) = &self.parent {
            let parent_names = parent.all_names();
            for name in parent_names {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        
        names.sort();
        names
    }
    
    /// Gets the environment ID.
    pub fn id(&self) -> u64 {
        self.id
    }
    
    /// Gets the generation.
    pub fn generation(&self) -> u64 {
        self.generation
    }
    
    /// Checks if this environment is an ancestor of another.
    pub fn is_ancestor_of(&self, other: &MacroEnvironment) -> bool {
        if let Some(parent) = &other.parent {
            self.id == parent.id || self.is_ancestor_of(parent)
        } else {
            false
        }
    }
    
    /// Creates a new macro context for this environment.
    pub fn create_context(&self) -> MacroContext {
        MacroContext::new(self.id)
    }
}

impl Default for MacroEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// A builder for constructing macro environments with predefined macros.
#[derive(Debug)]
pub struct MacroEnvironmentBuilder {
    environment: MacroEnvironment,
}

impl MacroEnvironmentBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            environment: MacroEnvironment::new(),
        }
    }
    
    /// Creates a builder with a parent environment.
    pub fn with_parent(parent: Rc<MacroEnvironment>) -> Self {
        Self {
            environment: MacroEnvironment::with_parent(parent),
        }
    }
    
    /// Adds a macro definition to the environment.
    pub fn define_macro(self, name: impl Into<String>, transformer: MacroTransformer) -> Self {
        self.environment.define(name.into(), transformer);
        self
    }
    
    /// Builds the environment.
    pub fn build(self) -> MacroEnvironment {
        self.environment
    }
}

impl Default for MacroEnvironmentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A scope for managing macro definitions with automatic cleanup.
#[derive(Debug)]
pub struct MacroScope {
    /// The environment for this scope
    environment: Rc<MacroEnvironment>,
    /// Names of macros defined in this scope (for cleanup)
    defined_names: Vec<String>,
}

impl MacroScope {
    /// Creates a new macro scope.
    pub fn new(environment: Rc<MacroEnvironment>) -> Self {
        Self {
            environment,
            defined_names: Vec::new(),
        }
    }
    
    /// Defines a macro in this scope.
    pub fn define(&mut self, name: impl Into<String>, transformer: MacroTransformer) {
        let name = name.into();
        self.environment.define(name.clone(), transformer);
        self.defined_names.push(name);
    }
    
    /// Looks up a macro in this scope.
    pub fn lookup(&self, name: &str) -> Option<MacroTransformer> {
        self.environment.lookup(name)
    }
    
    /// Gets the environment for this scope.
    pub fn environment(&self) -> &Rc<MacroEnvironment> {
        &self.environment
    }
    
    /// Creates a child scope.
    pub fn child(&self) -> MacroScope {
        MacroScope::new(self.environment.extend())
    }
}

impl Drop for MacroScope {
    fn drop(&mut self) {
        // Clean up macros defined in this scope
        for name in &self.defined_names {
            self.environment.undefine(name);
        }
    }
}

/// A macro resolver that manages macro visibility and resolution.
#[derive(Debug)]
pub struct MacroResolver {
    /// Stack of macro environments
    environment_stack: Vec<Rc<MacroEnvironment>>,
    /// Current macro environment
    current_environment: Rc<MacroEnvironment>,
}

impl MacroResolver {
    /// Creates a new macro resolver.
    pub fn new(environment: Rc<MacroEnvironment>) -> Self {
        Self {
            environment_stack: vec![environment.clone()],
            current_environment: environment,
        }
    }
    
    /// Pushes a new environment onto the stack.
    pub fn push_environment(&mut self, environment: Rc<MacroEnvironment>) {
        self.environment_stack.push(self.current_environment.clone());
        self.current_environment = environment;
    }
    
    /// Pops the current environment from the stack.
    pub fn pop_environment(&mut self) -> Option<Rc<MacroEnvironment>> {
        if let Some(previous) = self.environment_stack.pop() {
            let current = self.current_environment.clone();
            self.current_environment = previous;
            Some(current)
        } else {
            None
        }
    }
    
    /// Gets the current environment.
    pub fn current_environment(&self) -> &Rc<MacroEnvironment> {
        &self.current_environment
    }
    
    /// Resolves a macro name to a transformer.
    pub fn resolve(&self, name: &str) -> Option<MacroTransformer> {
        self.current_environment.lookup(name)
    }
    
    /// Defines a macro in the current environment.
    pub fn define(&self, name: impl Into<String>, transformer: MacroTransformer) {
        self.current_environment.define(name.into(), transformer);
    }
    
    /// Checks if a name refers to a macro.
    pub fn is_macro(&self, name: &str) -> bool {
        self.resolve(name).is_some()
    }
    
    /// Creates a new scope within the current environment.
    pub fn enter_scope(&mut self) -> MacroScope {
        let child_env = self.current_environment.extend();
        self.push_environment(child_env.clone());
        MacroScope::new(child_env)
    }
    
    /// Exits the current scope.
    pub fn exit_scope(&mut self) {
        self.pop_environment();
    }
}

/// Utility functions for macro environment management.
pub mod utils {
    use super::*;
    
    /// Creates a global macro environment with built-in macros.
    pub fn global_macro_environment() -> Rc<MacroEnvironment> {
        // Built-in macros will be added here by the builtins module
        
        Rc::new(MacroEnvironment::new())
    }
    
    /// Merges two macro environments, with the second taking precedence.
    pub fn merge_environments(
        base: Rc<MacroEnvironment>,
        overlay: Rc<MacroEnvironment>,
    ) -> Rc<MacroEnvironment> {
        let merged = base.extend();
        
        // Copy all macros from overlay to merged
        for name in overlay.local_names() {
            if let Some(transformer) = overlay.lookup(&name) {
                merged.define(name, transformer);
            }
        }
        
        merged
    }
    
    /// Creates a macro environment with only the specified macros.
    pub fn filtered_environment(
        source: &MacroEnvironment,
        allowed_names: &[String],
    ) -> Rc<MacroEnvironment> {
        let filtered = Rc::new(MacroEnvironment::new());
        
        for name in allowed_names {
            if let Some(transformer) = source.lookup(name) {
                filtered.define(name.clone(), transformer);
            }
        }
        
        filtered
    }
    
    /// Checks if two environments have the same macro definitions.
    pub fn environments_equal(env1: &MacroEnvironment, env2: &MacroEnvironment) -> bool {
        let names1 = env1.all_names();
        let names2 = env2.all_names();
        
        if names1.len() != names2.len() {
            return false;
        }
        
        for name in &names1 {
            if !names2.contains(name) {
                return false;
            }
            
            // In a full implementation, we'd compare the actual transformers
            // For now, just check that both have the macro
            if env1.lookup(name).is_none() || env2.lookup(name).is_none() {
                return false;
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macro_system::{Pattern, Template};
    
    fn create_dummy_transformer(name: &str) -> MacroTransformer {
        MacroTransformer {
            pattern: Pattern::Variable(format!("{name}_pattern")),
            template: Template::Variable(format!("{name}_template")),
            definition_env: crate::eval::global_environment(),
            name: Some(name.to_string()),
            source: None,
        }
    }
    
    #[test]
    fn test_macro_environment_creation() {
        let env = MacroEnvironment::new();
        assert_eq!(env.local_names().len(), 0);
        assert!(env.parent.is_none());
    }
    
    #[test]
    fn test_macro_definition_and_lookup() {
        let env = MacroEnvironment::new();
        let transformer = create_dummy_transformer("test-macro");
        
        env.define("test-macro".to_string(), transformer.clone());
        
        let looked_up = env.lookup("test-macro");
        assert!(looked_up.is_some());
        assert_eq!(looked_up.unwrap().name, transformer.name);
    }
    
    #[test]
    fn test_parent_environment_lookup() {
        let parent = Rc::new(MacroEnvironment::new());
        let child = parent.extend();
        
        let transformer = create_dummy_transformer("parent-macro");
        parent.define("parent-macro".to_string(), transformer.clone());
        
        let looked_up = child.lookup("parent-macro");
        assert!(looked_up.is_some());
        assert_eq!(looked_up.unwrap().name, transformer.name);
    }
    
    #[test]
    fn test_local_vs_parent_definitions() {
        let parent = Rc::new(MacroEnvironment::new());
        let child = parent.extend();
        
        let parent_transformer = create_dummy_transformer("parent-version");
        let child_transformer = create_dummy_transformer("child-version");
        
        parent.define("macro".to_string(), parent_transformer);
        child.define("macro".to_string(), child_transformer.clone());
        
        let looked_up = child.lookup("macro");
        assert!(looked_up.is_some());
        assert_eq!(looked_up.unwrap().name, child_transformer.name);
    }
    
    #[test]
    fn test_macro_scope() {
        let env = Rc::new(MacroEnvironment::new());
        let transformer = create_dummy_transformer("scoped-macro");
        
        {
            let mut scope = MacroScope::new(env.clone());
            scope.define("scoped-macro", transformer.clone());
            
            assert!(scope.lookup("scoped-macro").is_some());
            assert!(env.lookup("scoped-macro").is_some());
        }
        
        // After scope is dropped, macro should be undefined
        assert!(env.lookup("scoped-macro").is_none());
    }
    
    #[test]
    fn test_macro_resolver() {
        let env = Rc::new(MacroEnvironment::new());
        let resolver = MacroResolver::new(env.clone());
        
        let transformer = create_dummy_transformer("resolved-macro");
        resolver.define("resolved-macro", transformer.clone());
        
        assert!(resolver.is_macro("resolved-macro"));
        assert!(!resolver.is_macro("nonexistent-macro"));
        
        let resolved = resolver.resolve("resolved-macro");
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().name, transformer.name);
    }
    
    #[test]
    fn test_environment_builder() {
        let transformer = create_dummy_transformer("built-macro");
        
        let env = MacroEnvironmentBuilder::new()
            .define_macro("built-macro", transformer.clone())
            .build();
        
        let looked_up = env.lookup("built-macro");
        assert!(looked_up.is_some());
        assert_eq!(looked_up.unwrap().name, transformer.name);
    }
}
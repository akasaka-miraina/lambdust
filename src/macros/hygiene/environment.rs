//! Hygienic environment for macro expansion
//!
//! Extends the traditional environment system with hygienic symbol support.
//! Maintains separate namespaces for hygienic and traditional symbols while
//! preserving backward compatibility.

use super::symbol::{HygienicSymbol, SymbolId, EnvironmentId};
// Removed unused import: use super::context::ExpansionContext;
use crate::environment::Environment;
use crate::value::Value;
use crate::macros::Macro;
use crate::error::{LambdustError, Result};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Symbol resolution cache for performance optimization
#[derive(Debug, Clone)]
pub struct SymbolCache {
    /// Cache for symbol resolution results
    resolution_cache: RefCell<HashMap<String, SymbolResolution>>,
    /// Cache for symbol lookups
    lookup_cache: RefCell<HashMap<SymbolId, Option<Value>>>,
    /// Cache statistics
    pub cache_hits: RefCell<usize>,
    /// Number of cache misses
    pub cache_misses: RefCell<usize>,
}

impl Default for SymbolCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolCache {
    /// Creates a new symbol cache
    #[must_use] pub fn new() -> Self {
        Self {
            resolution_cache: RefCell::new(HashMap::new()),
            lookup_cache: RefCell::new(HashMap::new()),
            cache_hits: RefCell::new(0),
            cache_misses: RefCell::new(0),
        }
    }

    /// Clears all caches and resets statistics
    pub fn clear(&self) {
        self.resolution_cache.borrow_mut().clear();
        self.lookup_cache.borrow_mut().clear();
        *self.cache_hits.borrow_mut() = 0;
        *self.cache_misses.borrow_mut() = 0;
    }

    /// Returns cache statistics as (hits, misses)
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (*self.cache_hits.borrow(), *self.cache_misses.borrow())
    }
}

/// Environment with hygienic symbol support
#[derive(Debug, Clone)]
pub struct HygienicEnvironment {
    /// Unique identifier for this environment
    pub id: EnvironmentId,
    /// Traditional environment for backward compatibility
    pub inner: Rc<Environment>,
    /// Mapping from symbol names to hygienic symbols
    pub symbol_map: HashMap<String, HygienicSymbol>,
    /// Hygienic symbol bindings (`symbol_id` -> value)
    pub hygienic_bindings: HashMap<SymbolId, Value>,
    /// Hygienic macro definitions
    pub hygienic_macros: HashMap<String, HygienicMacro>,
    // Note: ExpansionContext is created on-demand when needed
    /// Parent hygienic environment
    pub parent: Option<Rc<HygienicEnvironment>>,
    /// Symbol resolution cache for performance
    symbol_cache: SymbolCache,
}

/// Hygienic macro definition
#[derive(Debug, Clone)]
pub struct HygienicMacro {
    /// Traditional macro definition
    pub inner: Macro,
    /// Definition environment ID
    pub definition_environment: EnvironmentId,
    /// Symbol mappings at definition time
    pub definition_symbols: HashMap<String, HygienicSymbol>,
}

impl HygienicEnvironment {
    /// Create new hygienic environment
    #[must_use] pub fn new() -> Self {
        let env_id = super::generator::SymbolGenerator::generate_environment_id();
        
        Self {
            id: env_id,
            inner: Rc::new(Environment::new()),
            symbol_map: HashMap::new(),
            hygienic_bindings: HashMap::new(),
            hygienic_macros: HashMap::new(),
            parent: None,
            symbol_cache: SymbolCache::new(),
        }
    }
    
    /// Create hygienic environment with parent
    pub fn with_parent(parent: Rc<HygienicEnvironment>) -> Self {
        let env_id = super::generator::SymbolGenerator::generate_environment_id();
        let inner_parent = parent.inner.clone();
        
        Self {
            id: env_id,
            inner: Rc::new(Environment::with_parent(inner_parent)),
            symbol_map: HashMap::new(),
            hygienic_bindings: HashMap::new(),
            hygienic_macros: HashMap::new(),
            parent: Some(parent),
            symbol_cache: SymbolCache::new(),
        }
    }
    
    /// Create environment for macro definition
    pub fn for_macro_definition(parent: Rc<HygienicEnvironment>) -> Self {
        let env_id = super::generator::SymbolGenerator::generate_environment_id();
        // ExpansionContext will be created when needed
        let inner_parent = parent.inner.clone();
        
        Self {
            id: env_id,
            inner: Rc::new(Environment::with_parent(inner_parent)),
            symbol_map: HashMap::new(),
            hygienic_bindings: HashMap::new(),
            hygienic_macros: HashMap::new(),
            // expansion_context created on-demand
            parent: Some(parent),
            symbol_cache: SymbolCache::new(),
        }
    }
    
    /// Define traditional variable (for backward compatibility)
    pub fn define(&self, name: String, value: Value) {
        self.inner.define(name, value);
    }
    
    /// Define hygienic symbol
    pub fn define_hygienic(&mut self, symbol: HygienicSymbol, value: Value) {
        self.hygienic_bindings.insert(symbol.id, value);
        self.symbol_map.insert(symbol.name.clone(), symbol);
    }
    
    /// Lookup traditional variable
    pub fn get(&self, name: &str) -> Option<Value> {
        self.inner.get(name)
    }
    
    /// Lookup hygienic symbol by name
    pub fn get_hygienic(&self, name: &str) -> Option<Value> {
        // First check if we have a hygienic symbol for this name
        if let Some(symbol) = self.symbol_map.get(name) {
            if let Some(value) = self.hygienic_bindings.get(&symbol.id) {
                return Some(value.clone());
            }
        }
        
        // Check parent environments
        if let Some(ref parent) = self.parent {
            parent.get_hygienic(name)
        } else {
            None
        }
    }
    
    /// Lookup hygienic symbol by symbol ID
    pub fn get_by_symbol_id(&self, symbol_id: SymbolId) -> Option<Value> {
        if let Some(value) = self.hygienic_bindings.get(&symbol_id) {
            Some(value.clone())
        } else if let Some(ref parent) = self.parent {
            parent.get_by_symbol_id(symbol_id)
        } else {
            None
        }
    }
    
    /// Lookup symbol with hygiene consideration
    pub fn lookup_hygienic_symbol(&self, symbol: &HygienicSymbol) -> Option<Value> {
        // Try exact symbol ID match first
        if let Some(value) = self.get_by_symbol_id(symbol.id) {
            return Some(value);
        }
        
        // For user code symbols, fallback to name-based lookup
        if !symbol.is_macro_introduced {
            return self.get(symbol.original_name());
        }
        
        None
    }
    
    /// Set traditional variable
    pub fn set(&self, name: &str, value: Value) -> Result<()> {
        self.inner.set(name, value)
    }
    
    /// Set hygienic symbol
    pub fn set_hygienic(&mut self, symbol: &HygienicSymbol, value: Value) -> Result<()> {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.hygienic_bindings.entry(symbol.id) {
            e.insert(value);
            Ok(())
        } else if let Some(ref mut _parent) = self.parent {
            // Try to modify parent (need Rc::get_mut which might fail)
            Err(LambdustError::undefined_variable(symbol.unique_name()))
        } else {
            Err(LambdustError::undefined_variable(symbol.unique_name()))
        }
    }
    
    /// Define hygienic macro
    pub fn define_hygienic_macro(&mut self, name: String, macro_def: Macro) {
        let hygienic_macro = HygienicMacro {
            inner: macro_def,
            definition_environment: self.id,
            definition_symbols: self.symbol_map.clone(),
        };
        self.hygienic_macros.insert(name, hygienic_macro);
    }
    
    /// Get hygienic macro
    pub fn get_hygienic_macro(&self, name: &str) -> Option<&HygienicMacro> {
        if let Some(macro_def) = self.hygienic_macros.get(name) {
            Some(macro_def)
        } else if let Some(ref parent) = self.parent {
            parent.get_hygienic_macro(name)
        } else {
            None
        }
    }
    
    /// Register symbol in current environment
    pub fn register_symbol(&mut self, symbol: HygienicSymbol) {
        self.symbol_map.insert(symbol.name.clone(), symbol);
    }
    
    /// Get symbol by name
    pub fn get_symbol(&self, name: &str) -> Option<&HygienicSymbol> {
        if let Some(symbol) = self.symbol_map.get(name) {
            Some(symbol)
        } else if let Some(ref parent) = self.parent {
            parent.get_symbol(name)
        } else {
            None
        }
    }
    
    /// Check if variable exists (traditional or hygienic)
    pub fn exists(&self, name: &str) -> bool {
        self.inner.exists(name) || self.symbol_map.contains_key(name)
    }
    
    /// Get environment depth
    pub fn depth(&self) -> usize {
        if let Some(ref parent) = self.parent {
            parent.depth() + 1
        } else {
            0
        }
    }

    /// Get environment ID
    pub fn environment_id(&self) -> EnvironmentId {
        self.id
    }
    
    /// Enter macro expansion context with safety checks
    pub fn enter_macro_expansion(&self, _macro_name: String) -> Result<Self> {
        let env = self.clone();
        // TODO: Re-implement expansion context tracking
        // match env.expansion_context.enter_macro(macro_name) {
        //     Ok(new_context) => {
        //         env.expansion_context = new_context;
        //         Ok(env)
        //     }
        //     Err(e) => Err(LambdustError::runtime_error(e))
        // }
        Ok(env) // Simplified for now
    }
    
    /// Create child environment for fresh scope
    pub fn create_child(&self) -> Self {
        Self::with_parent(Rc::new(self.clone()))
    }
    
    /// Merge symbol mappings from another environment (for macro expansion)
    pub fn merge_symbols(&mut self, other: &HygienicEnvironment) {
        for (name, symbol) in &other.symbol_map {
            self.symbol_map.insert(name.clone(), symbol.clone());
        }
    }
    
    /// Clear local symbol mappings (for fresh expansion context)
    pub fn clear_local_symbols(&mut self) {
        self.symbol_map.clear();
        // TODO: Re-implement expansion context clearing
        // self.expansion_context.clear_bindings();
    }
    
    /// Get all bound hygienic symbols
    pub fn get_bound_symbols(&self) -> Vec<&HygienicSymbol> {
        self.symbol_map.values().collect()
    }
    
    /// Resolve name to appropriate symbol (hygienic or traditional) with caching
    pub fn resolve_symbol(&self, name: &str) -> SymbolResolution {
        // Check cache first
        if let Some(cached) = self.symbol_cache.resolution_cache.borrow().get(name) {
            *self.symbol_cache.cache_hits.borrow_mut() += 1;
            return cached.clone();
        }

        // Cache miss - perform resolution
        *self.symbol_cache.cache_misses.borrow_mut() += 1;
        
        let result = if let Some(hygienic_symbol) = self.get_symbol(name) {
            SymbolResolution::Hygienic(hygienic_symbol.clone())
        } else if self.inner.exists(name) {
            SymbolResolution::Traditional(name.to_string())
        } else {
            SymbolResolution::Unbound(name.to_string())
        };

        // Cache the result
        self.symbol_cache.resolution_cache.borrow_mut().insert(name.to_string(), result.clone());
        result
    }

    /// Get symbol resolution cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        self.symbol_cache.get_cache_stats()
    }

    /// Clear symbol resolution cache
    pub fn clear_cache(&self) {
        self.symbol_cache.clear();
    }
}

impl Default for HygienicEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of symbol resolution
#[derive(Debug, Clone)]
pub enum SymbolResolution {
    /// Resolved to hygienic symbol
    Hygienic(HygienicSymbol),
    /// Resolved to traditional symbol name
    Traditional(String),
    /// Symbol is unbound
    Unbound(String),
}

impl SymbolResolution {
    /// Get the name regardless of resolution type
    #[must_use] pub fn name(&self) -> &str {
        match self {
            SymbolResolution::Hygienic(symbol) => symbol.original_name(),
            SymbolResolution::Traditional(name) => name,
            SymbolResolution::Unbound(name) => name,
        }
    }
    
    /// Check if symbol is bound
    #[must_use] pub fn is_bound(&self) -> bool {
        !matches!(self, SymbolResolution::Unbound(_))
    }
    
    /// Check if symbol is hygienic
    #[must_use] pub fn is_hygienic(&self) -> bool {
        matches!(self, SymbolResolution::Hygienic(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_hygienic_environment_creation() {
        let env = HygienicEnvironment::new();
        assert_eq!(env.depth(), 0);
        assert!(env.symbol_map.is_empty());
        assert!(env.hygienic_bindings.is_empty());
    }
    
    #[test]
    fn test_traditional_variable_operations() {
        let env = HygienicEnvironment::new();
        
        // Define and get traditional variable
        env.define("x".to_string(), Value::Number(SchemeNumber::Integer(42)));
        assert_eq!(env.get("x"), Some(Value::Number(SchemeNumber::Integer(42))));
        assert!(env.exists("x"));
    }
    
    #[test]
    fn test_hygienic_symbol_operations() {
        let mut env = HygienicEnvironment::new();
        let mut generator = super::super::generator::SymbolGenerator::new();
        generator.set_environment(env.id);
        
        let symbol = generator.generate_unique("temp");
        env.define_hygienic(symbol.clone(), Value::Number(SchemeNumber::Integer(99)));
        
        assert_eq!(env.get_hygienic("temp"), Some(Value::Number(SchemeNumber::Integer(99))));
        assert_eq!(env.get_by_symbol_id(symbol.id), Some(Value::Number(SchemeNumber::Integer(99))));
        assert!(env.exists("temp"));
    }
    
    #[test]
    fn test_symbol_resolution() {
        let mut env = HygienicEnvironment::new();
        let mut generator = super::super::generator::SymbolGenerator::new();
        generator.set_environment(env.id);
        
        // Define traditional variable
        env.define("traditional".to_string(), Value::Number(SchemeNumber::Integer(1)));
        
        // Define hygienic symbol
        let symbol = generator.generate_unique("hygienic");
        env.define_hygienic(symbol.clone(), Value::Number(SchemeNumber::Integer(2)));
        
        // Test resolution
        match env.resolve_symbol("traditional") {
            SymbolResolution::Traditional(name) => assert_eq!(name, "traditional"),
            _ => panic!("Expected traditional resolution"),
        }
        
        match env.resolve_symbol("hygienic") {
            SymbolResolution::Hygienic(sym) => assert_eq!(sym.original_name(), "hygienic"),
            _ => panic!("Expected hygienic resolution"),
        }
        
        match env.resolve_symbol("unbound") {
            SymbolResolution::Unbound(name) => assert_eq!(name, "unbound"),
            _ => panic!("Expected unbound resolution"),
        }
    }
    
    #[test]
    fn test_environment_hierarchy() {
        let parent = Rc::new(HygienicEnvironment::new());
        parent.define("parent_var".to_string(), Value::Number(SchemeNumber::Integer(123)));
        
        let child = HygienicEnvironment::with_parent(parent.clone());
        child.define("child_var".to_string(), Value::Number(SchemeNumber::Integer(456)));
        
        // Child can access parent variables
        assert_eq!(child.get("parent_var"), Some(Value::Number(SchemeNumber::Integer(123))));
        assert_eq!(child.get("child_var"), Some(Value::Number(SchemeNumber::Integer(456))));
        
        // Parent cannot access child variables
        assert_eq!(parent.get("child_var"), None);
        assert_eq!(parent.get("parent_var"), Some(Value::Number(SchemeNumber::Integer(123))));
        
        assert_eq!(child.depth(), 1);
        assert_eq!(parent.depth(), 0);
    }
    
    #[test]
    fn test_macro_expansion_context() {
        let env = HygienicEnvironment::new();
        let _expanded_env = env.enter_macro_expansion("test-macro".to_string()).unwrap();
        
        // TODO: Re-implement expansion context tests
        // assert_eq!(expanded_env.expansion_context.current_macro(), Some("test-macro"));
        // assert_eq!(expanded_env.expansion_context.depth, 1);
    }
}
//! Hygienic environment for macro expansion
//!
//! Extends the traditional environment system with hygienic symbol support.
//! Maintains separate namespaces for hygienic and traditional symbols while
//! preserving backward compatibility.

use super::symbol::{HygienicSymbol, SymbolId, EnvironmentId};
// Removed unused import: use super::context::ExpansionContext;
use crate::environment::{Environment, MutableDirectEnvironment};
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

/// Tracks environment consistency during macro expansion
#[derive(Debug, Clone)]
pub struct EnvironmentConsistencyTracker {
    /// Generation counter for consistency checks
    generation: u64,
    /// Last known environment state hash
    last_state_hash: u64,
    /// Set of symbols that have been modified
    modified_symbols: std::collections::HashSet<String>,
    /// Macro expansion depth (for detecting recursive issues)
    expansion_depth: u32,
    /// Maximum allowed expansion depth
    max_expansion_depth: u32,
}

impl Default for EnvironmentConsistencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentConsistencyTracker {
    /// Create a new consistency tracker
    #[must_use] pub fn new() -> Self {
        Self {
            generation: 0,
            last_state_hash: 0,
            modified_symbols: std::collections::HashSet::new(),
            expansion_depth: 0,
            max_expansion_depth: 100, // Prevent infinite macro expansion
        }
    }

    /// Increment generation and update state
    pub fn increment_generation(&mut self, new_state_hash: u64) {
        self.generation += 1;
        self.last_state_hash = new_state_hash;
    }

    /// Track symbol modification
    pub fn track_symbol_modification(&mut self, symbol: String) {
        self.modified_symbols.insert(symbol);
        self.generation += 1;
    }

    /// Check if environment is consistent
    #[must_use] pub fn is_consistent(&self, current_state_hash: u64) -> bool {
        self.last_state_hash == current_state_hash
    }

    /// Enter macro expansion (increment depth)
    pub fn enter_expansion(&mut self) -> Result<()> {
        self.expansion_depth += 1;
        if self.expansion_depth > self.max_expansion_depth {
            return Err(LambdustError::runtime_error(
                format!("Macro expansion depth exceeded limit: {}", self.max_expansion_depth)
            ));
        }
        Ok(())
    }

    /// Exit macro expansion (decrement depth)
    pub fn exit_expansion(&mut self) {
        if self.expansion_depth > 0 {
            self.expansion_depth -= 1;
        }
    }

    /// Get current expansion depth
    #[must_use] pub fn expansion_depth(&self) -> u32 {
        self.expansion_depth
    }

    /// Clear modification tracking
    pub fn clear_modifications(&mut self) {
        self.modified_symbols.clear();
    }
}

/// Environment with hygienic symbol support and consistency guarantees
#[derive(Debug, Clone)]
pub struct HygienicEnvironment {
    /// Unique identifier for this environment
    pub id: EnvironmentId,
    /// Traditional environment for backward compatibility
    pub inner: Rc<Environment>,
    /// Optimized direct environment for improved performance
    pub direct: MutableDirectEnvironment,
    /// Mapping from symbol names to hygienic symbols
    pub symbol_map: HashMap<String, HygienicSymbol>,
    /// Hygienic symbol bindings (`symbol_id` -> value)
    pub hygienic_bindings: HashMap<SymbolId, Value>,
    /// Hygienic macro definitions
    pub hygienic_macros: HashMap<String, HygienicMacro>,
    /// Parent hygienic environment
    pub parent: Option<Rc<HygienicEnvironment>>,
    /// Symbol resolution cache for performance
    symbol_cache: SymbolCache,
    /// Environment consistency tracking
    consistency_tracker: EnvironmentConsistencyTracker,
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
            direct: MutableDirectEnvironment::new(),
            symbol_map: HashMap::new(),
            hygienic_bindings: HashMap::new(),
            hygienic_macros: HashMap::new(),
            parent: None,
            symbol_cache: SymbolCache::new(),
            consistency_tracker: EnvironmentConsistencyTracker::new(),
        }
    }
    
    /// Create hygienic environment with parent
    pub fn with_parent(parent: Rc<HygienicEnvironment>) -> Self {
        let env_id = super::generator::SymbolGenerator::generate_environment_id();
        let inner_parent = parent.inner.clone();
        
        Self {
            id: env_id,
            inner: Rc::new(Environment::with_parent(inner_parent)),
            direct: MutableDirectEnvironment::with_parent(parent.direct.clone_direct()),
            symbol_map: HashMap::new(),
            hygienic_bindings: HashMap::new(),
            hygienic_macros: HashMap::new(),
            parent: Some(parent),
            symbol_cache: SymbolCache::new(),
            consistency_tracker: EnvironmentConsistencyTracker::new(),
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
            direct: MutableDirectEnvironment::with_parent(parent.direct.clone_direct()),
            symbol_map: HashMap::new(),
            hygienic_bindings: HashMap::new(),
            hygienic_macros: HashMap::new(),
            // expansion_context created on-demand
            parent: Some(parent),
            symbol_cache: SymbolCache::new(),
            consistency_tracker: EnvironmentConsistencyTracker::new(),
        }
    }
    
    /// Define traditional variable (for backward compatibility)
    pub fn define(&self, name: String, value: Value) {
        self.inner.define(name, value);
    }
    
    /// Define hygienic symbol
    pub fn define_hygienic(&mut self, symbol: HygienicSymbol, value: Value) {
        self.hygienic_bindings.insert(symbol.id, value.clone());
        self.symbol_map.insert(symbol.name.clone(), symbol.clone());
        
        // Track symbol modification for consistency
        self.consistency_tracker.track_symbol_modification(symbol.name.clone());
        
        // Synchronize with direct environment
        self.direct.define(symbol.unique_name(), value);
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
            e.insert(value.clone());
            
            // Track symbol modification for consistency
            self.consistency_tracker.track_symbol_modification(symbol.name.clone());
            
            // Synchronize with direct environment
            let _ = self.direct.set(&symbol.unique_name(), value);
            
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
        let mut env = self.clone();
        
        // Enter expansion with consistency tracking
        env.consistency_tracker.enter_expansion()?;
        
        // Update environment state hash for consistency tracking
        let state_hash = self.calculate_environment_state_hash();
        env.consistency_tracker.increment_generation(state_hash);
        
        Ok(env)
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
        
        // Clear modifications in consistency tracker
        self.consistency_tracker.clear_modifications();
        
        // Clear symbol cache for fresh state
        self.symbol_cache.clear();
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
    
    /// Calculate environment state hash for consistency tracking
    fn calculate_environment_state_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash symbol map
        for (name, symbol) in &self.symbol_map {
            name.hash(&mut hasher);
            symbol.id.hash(&mut hasher);
        }
        
        // Hash hygienic bindings count (not values for performance)
        self.hygienic_bindings.len().hash(&mut hasher);
        
        // Hash environment ID
        self.id.hash(&mut hasher);
        
        hasher.finish()
    }
    
    /// Get current environment consistency tracker
    pub fn consistency_tracker(&self) -> &EnvironmentConsistencyTracker {
        &self.consistency_tracker
    }
    
    /// Get mutable consistency tracker
    pub fn consistency_tracker_mut(&mut self) -> &mut EnvironmentConsistencyTracker {
        &mut self.consistency_tracker
    }
    
    /// Exit macro expansion (decrement expansion depth)
    pub fn exit_macro_expansion(&mut self) {
        self.consistency_tracker.exit_expansion();
    }
    
    /// Check environment consistency
    pub fn is_environment_consistent(&self) -> bool {
        let current_hash = self.calculate_environment_state_hash();
        self.consistency_tracker.is_consistent(current_hash)
    }
    
    /// Synchronize environments (direct and inner)
    pub fn synchronize_environments(&mut self) -> Result<()> {
        // Synchronize hygienic bindings to direct environment
        for (symbol_id, value) in &self.hygienic_bindings {
            if let Some(symbol) = self.symbol_map.values().find(|s| s.id == *symbol_id) {
                self.direct.define(symbol.unique_name(), value.clone());
            }
        }
        
        // Update consistency tracker
        let state_hash = self.calculate_environment_state_hash();
        self.consistency_tracker.increment_generation(state_hash);
        
        Ok(())
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

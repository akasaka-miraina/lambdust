//! Arena-based memory management for dependent types.
//!
//! This module provides efficient memory allocation strategies for dependent type
//! operations, reducing heap fragmentation and improving cache locality.
//!
//! # Design Philosophy
//!
//! Instead of using `Box<T>` for every recursive type, we use arena allocation
//! where types are stored in contiguous memory regions. This approach provides:
//!
//! - **Better Cache Locality**: Related types are stored near each other
//! - **Reduced Fragmentation**: Large contiguous allocations instead of many small ones
//! - **Faster Allocation**: Bump pointer allocation is much faster than malloc
//! - **Automatic Cleanup**: Arena lifetime management simplifies memory management
//!
//! # Memory Layout Strategy
//!
//! ```
//! Arena Memory Layout:
//! [Type1][Type2][Type3]...[TypeN]
//!   ^      ^      ^
//!   |      |      |
//! TypeRef TypeRef TypeRef
//! ```
//!
//! Each TypeRef is a lightweight reference (index + generation) that points
//! to the actual type data stored in the arena.

use crate::diagnostics::{Error, Result, Span};
use bumpalo::Bump;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};

/// A lightweight reference to a type stored in the arena.
///
/// Instead of using `Box<DependentType>`, we use `TypeRef` which is:
/// - Only 8 bytes (vs 8 bytes for Box pointer + heap allocation overhead)  
/// - Provides bounds checking and generation-based safety
/// - Enables efficient equality comparisons by index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeRef {
    /// Index into the arena storage
    index: u32,
    /// Generation counter for safety (detects use-after-free)
    generation: u32,
}

/// A lightweight reference to a term stored in the arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TermRef {
    /// Index into the arena storage
    index: u32, 
    /// Generation counter for safety
    generation: u32,
}

/// Arena-allocated storage for dependent types and terms.
///
/// This structure manages memory-efficient storage of dependent types using
/// arena allocation. Types are stored contiguously in memory, referenced
/// by lightweight TypeRef handles.
#[derive(Debug)]
pub struct TypeArena {
    /// Main bump allocator for type storage
    types_arena: Bump,
    /// Bump allocator for term storage
    terms_arena: Bump,
    /// Index mapping for type lookups
    type_storage: RefCell<Vec<TypeEntry>>,
    /// Index mapping for term lookups
    term_storage: RefCell<Vec<TermEntry>>,
    /// Global generation counter for safety
    generation: AtomicU32,
    /// Type deduplication cache
    type_cache: RefCell<HashMap<TypeHash, TypeRef>>,
    /// Term deduplication cache
    term_cache: RefCell<HashMap<TermHash, TermRef>>,
}

/// Internal storage entry for types
#[derive(Debug)]
struct TypeEntry {
    /// The actual type data stored in the arena
    data: &'static DependentTypeData,
    /// Generation when this entry was created
    generation: u32,
    /// Whether this entry is still valid
    valid: bool,
}

/// Internal storage entry for terms
#[derive(Debug)]
struct TermEntry {
    /// The actual term data stored in the arena
    data: &'static DependentTermData,
    /// Generation when this entry was created
    generation: u32,
    /// Whether this entry is still valid
    valid: bool,
}

/// Hash key for type deduplication
#[derive(Debug, Hash, PartialEq, Eq)]
struct TypeHash {
    /// Discriminant of the enum variant
    discriminant: u8,
    /// Content hash for the variant data
    content_hash: u64,
}

/// Hash key for term deduplication
#[derive(Debug, Hash, PartialEq, Eq)] 
struct TermHash {
    /// Discriminant of the enum variant
    discriminant: u8,
    /// Content hash for the variant data
    content_hash: u64,
}

/// Arena-stored dependent type data (without Box allocations)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependentTypeData {
    /// Universe types: Type₀, Type₁, Type₂, ...
    Universe(u32),
    
    /// Π-types (dependent function types): (x : A) → B(x)
    Pi {
        var: String,
        domain: TypeRef,
        codomain: TypeRef,
    },
    
    /// Σ-types (dependent pair types): (x : A) × B(x)
    Sigma {
        var: String,
        first: TypeRef,
        second: TypeRef,
    },
    
    /// Identity types: Id_A(a, b)
    Identity {
        ty: TypeRef,
        left: TermRef,
        right: TermRef,
    },
    
    /// Inductive types with constructors
    Inductive {
        name: String,
        parameters: Vec<(String, TypeRef)>,
        universe_level: u32,
        constructors: Vec<(String, TypeRef)>,
        induction_principle: Option<TypeRef>,
    },
}

/// Arena-stored dependent term data (without Box allocations)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependentTermData {
    /// Variable reference
    Variable(String),
    
    /// Lambda abstraction: λx:A.t
    Lambda {
        param: String,
        param_type: TypeRef,
        body: TermRef,
    },
    
    /// Function application: f(a)
    Application {
        function: TermRef,
        argument: TermRef,
    },
    
    /// Dependent pair construction: (a, b)
    Pair {
        first: TermRef,
        second: TermRef,
    },
    
    /// Projection from dependent pairs
    Projection {
        pair: TermRef,
        is_first: bool,
    },
    
    /// Reflexivity proof: refl_a
    Refl {
        ty: TypeRef,
    },
    
    /// Constructor application
    Constructor {
        name: String,
        args: Vec<TermRef>,
        result_type: TypeRef,
    },
    
    /// Pattern matching
    Match {
        scrutinee: TermRef,
        branches: Vec<MatchBranchData>,
        return_type: TypeRef,
    },
}

/// Pattern matching branch data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchBranchData {
    pub pattern: PatternData,
    pub body: TermRef,
}

/// Pattern data for matching
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternData {
    Variable(String),
    Constructor {
        name: String,
        args: Vec<PatternData>,
    },
}

impl TypeArena {
    /// Create a new type arena with default capacity
    pub fn new() -> Self {
        Self {
            types_arena: Bump::new(),
            terms_arena: Bump::new(),
            type_storage: RefCell::new(Vec::new()),
            term_storage: RefCell::new(Vec::new()),
            generation: AtomicU32::new(0),
            type_cache: RefCell::new(HashMap::new()),
            term_cache: RefCell::new(HashMap::new()),
        }
    }
    
    /// Create a new arena with specific capacity hint
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            types_arena: Bump::with_capacity(capacity * std::mem::size_of::<DependentTypeData>()),
            terms_arena: Bump::with_capacity(capacity * std::mem::size_of::<DependentTermData>()),
            type_storage: RefCell::new(Vec::with_capacity(capacity)),
            term_storage: RefCell::new(Vec::with_capacity(capacity)),
            generation: AtomicU32::new(0),
            type_cache: RefCell::new(HashMap::with_capacity(capacity / 4)),
            term_cache: RefCell::new(HashMap::with_capacity(capacity / 4)),
        }
    }
    
    /// Allocate a new type in the arena, returning a TypeRef
    pub fn alloc_type(&self, type_data: DependentTypeData) -> Result<TypeRef> {
        // Check cache for deduplication
        let type_hash = self.hash_type(&type_data);
        if let Some(&existing_ref) = self.type_cache.borrow().get(&type_hash) {
            if self.is_valid_type_ref(existing_ref) {
                return Ok(existing_ref);
            }
        }
        
        // Allocate in arena  
        let arena_data: &'static DependentTypeData = unsafe {
            // Safety: The arena outlives all references to this data
            std::mem::transmute(self.types_arena.alloc(type_data.clone()))
        };
        
        let current_gen = self.generation.fetch_add(1, Ordering::SeqCst);
        let mut storage = self.type_storage.borrow_mut();
        
        let index = storage.len() as u32;
        let type_ref = TypeRef {
            index,
            generation: current_gen,
        };
        
        storage.push(TypeEntry {
            data: arena_data,
            generation: current_gen,
            valid: true,
        });
        
        // Cache for future deduplication
        self.type_cache.borrow_mut().insert(type_hash, type_ref);
        
        Ok(type_ref)
    }
    
    /// Allocate a new term in the arena, returning a TermRef
    pub fn alloc_term(&self, term_data: DependentTermData) -> Result<TermRef> {
        // Check cache for deduplication
        let term_hash = self.hash_term(&term_data);
        if let Some(&existing_ref) = self.term_cache.borrow().get(&term_hash) {
            if self.is_valid_term_ref(existing_ref) {
                return Ok(existing_ref);
            }
        }
        
        // Allocate in arena
        let arena_data: &'static DependentTermData = unsafe {
            // Safety: The arena outlives all references to this data
            std::mem::transmute(self.terms_arena.alloc(term_data.clone()))
        };
        
        let current_gen = self.generation.fetch_add(1, Ordering::SeqCst);
        let mut storage = self.term_storage.borrow_mut();
        
        let index = storage.len() as u32;
        let term_ref = TermRef {
            index,
            generation: current_gen,
        };
        
        storage.push(TermEntry {
            data: arena_data,
            generation: current_gen,
            valid: true,
        });
        
        // Cache for future deduplication
        self.term_cache.borrow_mut().insert(term_hash, term_ref);
        
        Ok(term_ref)
    }
    
    /// Resolve a TypeRef to its data
    pub fn resolve_type(&self, type_ref: TypeRef) -> Result<&DependentTypeData> {
        let storage = self.type_storage.borrow();
        
        if type_ref.index as usize >= storage.len() {
            return Err(Box::new(Error::type_error(format!("Invalid type reference: index {} out of bounds", type_ref.index), Span::new(0, 0))));
        }
        
        let entry = &storage[type_ref.index as usize];
        
        if !entry.valid || entry.generation != type_ref.generation {
            return Err(Box::new(Error::type_error("Invalid type reference: generation mismatch or invalidated".to_string(), Span::new(0, 0))));
        }
        
        Ok(entry.data)
    }
    
    /// Resolve a TermRef to its data
    pub fn resolve_term(&self, term_ref: TermRef) -> Result<&DependentTermData> {
        let storage = self.term_storage.borrow();
        
        if term_ref.index as usize >= storage.len() {
            return Err(Box::new(Error::type_error(format!("Invalid term reference: index {} out of bounds", term_ref.index), Span::new(0, 0))));
        }
        
        let entry = &storage[term_ref.index as usize];
        
        if !entry.valid || entry.generation != term_ref.generation {
            return Err(Box::new(Error::type_error("Invalid term reference: generation mismatch or invalidated".to_string(), Span::new(0, 0))));
        }
        
        Ok(entry.data)
    }
    
    /// Check if a TypeRef is still valid
    pub fn is_valid_type_ref(&self, type_ref: TypeRef) -> bool {
        let storage = self.type_storage.borrow();
        if type_ref.index as usize >= storage.len() {
            return false;
        }
        
        let entry = &storage[type_ref.index as usize];
        entry.valid && entry.generation == type_ref.generation
    }
    
    /// Check if a TermRef is still valid  
    pub fn is_valid_term_ref(&self, term_ref: TermRef) -> bool {
        let storage = self.term_storage.borrow();
        if term_ref.index as usize >= storage.len() {
            return false;
        }
        
        let entry = &storage[term_ref.index as usize];
        entry.valid && entry.generation == term_ref.generation
    }
    
    /// Get memory usage statistics
    pub fn memory_stats(&self) -> ArenaStats {
        let type_storage = self.type_storage.borrow();
        let term_storage = self.term_storage.borrow();
        
        ArenaStats {
            types_count: type_storage.len(),
            terms_count: term_storage.len(),
            types_memory: self.types_arena.allocated_bytes(),
            terms_memory: self.terms_arena.allocated_bytes(),
            cache_hits_types: self.type_cache.borrow().len(),
            cache_hits_terms: self.term_cache.borrow().len(),
        }
    }
    
    /// Clear the arena and reset all allocations
    pub fn clear(&mut self) {
        self.types_arena.reset();
        self.terms_arena.reset();
        self.type_storage.borrow_mut().clear();
        self.term_storage.borrow_mut().clear();
        self.type_cache.borrow_mut().clear();
        self.term_cache.borrow_mut().clear();
        self.generation.store(0, Ordering::SeqCst);
    }
    
    /// Create hash for type deduplication
    fn hash_type(&self, type_data: &DependentTypeData) -> TypeHash {
        use std::collections::hash_map::DefaultHasher;
        
        let discriminant = match type_data {
            DependentTypeData::Universe(_) => 0,
            DependentTypeData::Pi { .. } => 1,
            DependentTypeData::Sigma { .. } => 2,
            DependentTypeData::Identity { .. } => 3,
            DependentTypeData::Inductive { .. } => 4,
        };
        
        let mut hasher = DefaultHasher::new();
        type_data.hash(&mut hasher);
        let content_hash = hasher.finish();
        
        TypeHash {
            discriminant,
            content_hash,
        }
    }
    
    /// Create hash for term deduplication
    fn hash_term(&self, term_data: &DependentTermData) -> TermHash {
        use std::collections::hash_map::DefaultHasher;
        
        let discriminant = match term_data {
            DependentTermData::Variable(_) => 0,
            DependentTermData::Lambda { .. } => 1,
            DependentTermData::Application { .. } => 2,
            DependentTermData::Pair { .. } => 3,
            DependentTermData::Projection { .. } => 4,
            DependentTermData::Refl { .. } => 5,
            DependentTermData::Constructor { .. } => 6,
            DependentTermData::Match { .. } => 7,
        };
        
        let mut hasher = DefaultHasher::new();
        term_data.hash(&mut hasher);
        let content_hash = hasher.finish();
        
        TermHash {
            discriminant,
            content_hash,
        }
    }
}

/// Memory usage statistics for the arena
#[derive(Debug, Clone)]
pub struct ArenaStats {
    /// Number of types stored
    pub types_count: usize,
    /// Number of terms stored
    pub terms_count: usize,
    /// Memory used by types arena (bytes)
    pub types_memory: usize,
    /// Memory used by terms arena (bytes)
    pub terms_memory: usize,
    /// Number of cached type entries
    pub cache_hits_types: usize,
    /// Number of cached term entries
    pub cache_hits_terms: usize,
}

impl ArenaStats {
    /// Total memory used by both arenas
    pub fn total_memory(&self) -> usize {
        self.types_memory + self.terms_memory
    }
    
    /// Average memory per type
    pub fn avg_memory_per_type(&self) -> f64 {
        if self.types_count == 0 {
            0.0
        } else {
            self.types_memory as f64 / self.types_count as f64
        }
    }
    
    /// Average memory per term
    pub fn avg_memory_per_term(&self) -> f64 {
        if self.terms_count == 0 {
            0.0
        } else {
            self.terms_memory as f64 / self.terms_count as f64
        }
    }
}

// Hash implementations for DependentTypeData and DependentTermData
impl Hash for DependentTypeData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DependentTypeData::Universe(level) => {
                0u8.hash(state);
                level.hash(state);
            }
            DependentTypeData::Pi { var, domain, codomain } => {
                1u8.hash(state);
                var.hash(state);
                domain.hash(state);
                codomain.hash(state);
            }
            DependentTypeData::Sigma { var, first, second } => {
                2u8.hash(state);
                var.hash(state);
                first.hash(state);
                second.hash(state);
            }
            DependentTypeData::Identity { ty, left, right } => {
                3u8.hash(state);
                ty.hash(state);
                left.hash(state);
                right.hash(state);
            }
            DependentTypeData::Inductive { name, parameters, universe_level, constructors, induction_principle } => {
                4u8.hash(state);
                name.hash(state);
                parameters.hash(state);
                universe_level.hash(state);
                constructors.hash(state);
                induction_principle.hash(state);
            }
        }
    }
}

impl Hash for DependentTermData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DependentTermData::Variable(name) => {
                0u8.hash(state);
                name.hash(state);
            }
            DependentTermData::Lambda { param, param_type, body } => {
                1u8.hash(state);
                param.hash(state);
                param_type.hash(state);
                body.hash(state);
            }
            DependentTermData::Application { function, argument } => {
                2u8.hash(state);
                function.hash(state);
                argument.hash(state);
            }
            DependentTermData::Pair { first, second } => {
                3u8.hash(state);
                first.hash(state);
                second.hash(state);
            }
            DependentTermData::Projection { pair, is_first } => {
                4u8.hash(state);
                pair.hash(state);
                is_first.hash(state);
            }
            DependentTermData::Refl { ty } => {
                5u8.hash(state);
                ty.hash(state);
            }
            DependentTermData::Constructor { name, args, result_type } => {
                6u8.hash(state);
                name.hash(state);
                args.hash(state);
                result_type.hash(state);
            }
            DependentTermData::Match { scrutinee, branches, return_type } => {
                7u8.hash(state);
                scrutinee.hash(state);
                branches.hash(state);
                return_type.hash(state);
            }
        }
    }
}

impl Hash for MatchBranchData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pattern.hash(state);
        self.body.hash(state);
    }
}

impl Hash for PatternData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            PatternData::Variable(name) => {
                0u8.hash(state);
                name.hash(state);
            }
            PatternData::Constructor { name, args } => {
                1u8.hash(state);
                name.hash(state);
                args.hash(state);
            }
        }
    }
}

// Display implementations
impl fmt::Display for ArenaStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, 
            "Arena Stats: {} types ({:.1}KB), {} terms ({:.1}KB), {} cached types, {} cached terms",
            self.types_count,
            self.types_memory as f64 / 1024.0,
            self.terms_count,
            self.terms_memory as f64 / 1024.0,
            self.cache_hits_types,
            self.cache_hits_terms
        )
    }
}

impl Default for TypeArena {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arena_basic_allocation() {
        let arena = TypeArena::new();
        
        // Allocate a universe type
        let universe_data = DependentTypeData::Universe(0);
        let type_ref = arena.alloc_type(universe_data.clone()).unwrap();
        
        // Resolve and verify
        let resolved = arena.resolve_type(type_ref).unwrap();
        assert_eq!(resolved, &universe_data);
    }
    
    #[test]
    fn test_arena_deduplication() {
        let arena = TypeArena::new();
        
        // Allocate the same type twice
        let universe_data = DependentTypeData::Universe(0);
        let type_ref1 = arena.alloc_type(universe_data.clone()).unwrap();
        let type_ref2 = arena.alloc_type(universe_data).unwrap();
        
        // Should return the same reference due to deduplication
        assert_eq!(type_ref1, type_ref2);
    }
    
    #[test]
    fn test_arena_memory_stats() {
        let arena = TypeArena::new();
        
        let initial_stats = arena.memory_stats();
        assert_eq!(initial_stats.types_count, 0);
        
        // Allocate some types
        let _ref1 = arena.alloc_type(DependentTypeData::Universe(0)).unwrap();
        let _ref2 = arena.alloc_type(DependentTypeData::Universe(1)).unwrap();
        
        let after_stats = arena.memory_stats();
        assert_eq!(after_stats.types_count, 2);
        assert!(after_stats.types_memory > 0);
    }
    
    #[test]
    fn test_invalid_reference_detection() {
        let arena = TypeArena::new();
        
        // Create an invalid reference
        let invalid_ref = TypeRef {
            index: 999,
            generation: 0,
        };
        
        // Should detect invalid reference
        assert!(!arena.is_valid_type_ref(invalid_ref));
        assert!(arena.resolve_type(invalid_ref).is_err());
    }
    
    #[test]
    fn test_arena_clear() {
        let mut arena = TypeArena::new();
        
        // Allocate some data
        let _ref1 = arena.alloc_type(DependentTypeData::Universe(0)).unwrap();
        let _ref2 = arena.alloc_term(DependentTermData::Variable("x".to_string())).unwrap();
        
        let stats_before = arena.memory_stats();
        assert!(stats_before.types_count > 0);
        assert!(stats_before.terms_count > 0);
        
        // Clear the arena
        arena.clear();
        
        let stats_after = arena.memory_stats();
        assert_eq!(stats_after.types_count, 0);
        assert_eq!(stats_after.terms_count, 0);
    }
    
    #[test]
    fn test_complex_type_allocation() {
        let arena = TypeArena::new();
        
        // Create a Pi type: (x : Type₀) → Type₀
        let domain_ref = arena.alloc_type(DependentTypeData::Universe(0)).unwrap();
        let codomain_ref = arena.alloc_type(DependentTypeData::Universe(0)).unwrap();
        
        let pi_data = DependentTypeData::Pi {
            var: "x".to_string(),
            domain: domain_ref,
            codomain: codomain_ref,
        };
        
        let pi_ref = arena.alloc_type(pi_data.clone()).unwrap();
        let resolved_pi = arena.resolve_type(pi_ref).unwrap();
        
        assert_eq!(resolved_pi, &pi_data);
        
        // Verify we can resolve the nested references
        if let DependentTypeData::Pi { domain, codomain, .. } = resolved_pi {
            let resolved_domain = arena.resolve_type(*domain).unwrap();
            let resolved_codomain = arena.resolve_type(*codomain).unwrap();
            
            assert_eq!(resolved_domain, &DependentTypeData::Universe(0));
            assert_eq!(resolved_codomain, &DependentTypeData::Universe(0));
        }
    }
}
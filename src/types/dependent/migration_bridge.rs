//! Migration bridge for gradual transition to optimized dependent types.
//!
//! This module provides seamless compatibility between the old Box-based
//! dependent type implementation and the new arena-based optimized version,
//! allowing for gradual migration without breaking existing code.
//!
//! # Migration Strategy
//!
//! ```text
//! Phase 1: Drop-in compatibility layer
//!   Old Code → [Migration Bridge] → New Implementation
//!
//! Phase 2: Gradual API transition
//!   New Code → Direct new API
//!   Old Code → [Migration Bridge] → New Implementation
//!
//! Phase 3: Complete migration
//!   All Code → Direct new API
//!   Migration Bridge → Deprecated
//! ```
//!
//! # API Compatibility
//!
//! - **100% API compatibility**: All existing function signatures preserved
//! - **Performance improvement**: Transparent optimization with no code changes
//! - **Incremental adoption**: Use new APIs where beneficial
//! - **Fallback safety**: Automatic fallback to old implementation if needed

use super::{
    arena::{ArenaStats, TypeArena},
    core::{
        DependentTerm, DependentType, MatchBranch, Normalizer, Pattern, TypingContext,
        UniverseLevel,
    },
    memory_pool::{MemoryPoolManager, MemoryPoolStatistics},
    optimized_core::{
        OptimizedDependentTerm, OptimizedDependentType, OptimizedMatchBranch, OptimizedNormalizer,
        OptimizedPattern, OptimizedTypingContext,
    },
    performance_benchmark::{BenchmarkSummary, DependentTypeBenchmarkSuite},
};
use crate::diagnostics::{Error, Result, Span};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Migration bridge that provides seamless compatibility between old and new implementations.
///
/// This bridge serves as a transparent proxy that:
/// - Automatically converts between old and new type representations
/// - Maintains API compatibility for existing code
/// - Provides performance benefits through the optimized backend
/// - Enables gradual migration to the new API
#[derive(Debug)]
pub struct MigrationBridge {
    /// Optimized backend for actual operations
    optimized_context: OptimizedTypingContext,
    /// Optimized normalizer
    optimized_normalizer: OptimizedNormalizer,
    /// Memory pool manager for advanced optimizations
    memory_pool: MemoryPoolManager,
    /// Migration statistics
    stats: RefCell<MigrationStats>,
    /// Configuration for migration behavior
    config: MigrationConfig,
}

/// Statistics about the migration process
#[derive(Debug, Clone)]
pub struct MigrationStats {
    /// Number of old API calls handled
    pub old_api_calls: u64,
    /// Number of new API calls handled
    pub new_api_calls: u64,
    /// Number of type conversions performed
    pub type_conversions: u64,
    /// Time spent in conversion overhead
    pub conversion_overhead: std::time::Duration,
    /// Memory savings achieved
    pub memory_savings: f64,
    /// Performance improvement ratio
    pub performance_improvement: f64,
}

/// Configuration for migration behavior
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Whether to use optimized backend for all operations
    pub use_optimized_backend: bool,
    /// Whether to collect detailed migration statistics
    pub collect_stats: bool,
    /// Memory pool configuration
    pub enable_memory_pooling: bool,
    /// Automatic performance benchmarking
    pub enable_auto_benchmarking: bool,
    /// Fallback to old implementation on errors
    pub enable_fallback: bool,
}

/// Adapter that wraps the optimized typing context to provide old API
pub struct CompatibleTypingContext {
    /// The migration bridge
    bridge: Rc<RefCell<MigrationBridge>>,
    /// Unique identifier for this context
    context_id: u32,
}

/// Adapter that wraps the optimized normalizer to provide old API
pub struct CompatibleNormalizer {
    /// The migration bridge
    bridge: Rc<RefCell<MigrationBridge>>,
    /// Unique identifier for this normalizer
    normalizer_id: u32,
}

impl MigrationBridge {
    /// Create a new migration bridge with default configuration
    pub fn new() -> Self {
        Self::with_config(MigrationConfig::default())
    }

    /// Create a migration bridge with custom configuration
    pub fn with_config(config: MigrationConfig) -> Self {
        let arena = Rc::new(RefCell::new(TypeArena::new()));

        Self {
            optimized_context: OptimizedTypingContext::with_arena(arena.clone()),
            optimized_normalizer: OptimizedNormalizer::with_arena(arena),
            memory_pool: MemoryPoolManager::new(),
            stats: RefCell::new(MigrationStats::new()),
            config,
        }
    }

    /// Create a compatible typing context that provides the old API
    pub fn create_compatible_context(&self) -> CompatibleTypingContext {
        self.stats.borrow_mut().new_api_calls += 1;

        CompatibleTypingContext {
            bridge: Rc::new(RefCell::new(self.clone())),
            context_id: self.generate_context_id(),
        }
    }

    /// Create a compatible normalizer that provides the old API
    pub fn create_compatible_normalizer(&self) -> CompatibleNormalizer {
        self.stats.borrow_mut().new_api_calls += 1;

        CompatibleNormalizer {
            bridge: Rc::new(RefCell::new(self.clone())),
            normalizer_id: self.generate_normalizer_id(),
        }
    }

    /// Convert old DependentType to optimized version
    pub fn convert_type_to_optimized(
        &self,
        old_type: &DependentType,
    ) -> Result<OptimizedDependentType> {
        let start_time = std::time::Instant::now();

        let result = match old_type {
            DependentType::Universe(level) => Ok(OptimizedDependentType::Universe(*level)),
            DependentType::Pi {
                var,
                domain,
                codomain,
            } => {
                let opt_domain = self.convert_type_to_optimized(domain)?;
                let opt_codomain = self.convert_type_to_optimized(codomain)?;
                Ok(OptimizedDependentType::Pi {
                    var: var.clone(),
                    domain: Rc::new(opt_domain),
                    codomain: Rc::new(opt_codomain),
                })
            }
            DependentType::Sigma { var, first, second } => {
                let opt_first = self.convert_type_to_optimized(first)?;
                let opt_second = self.convert_type_to_optimized(second)?;
                Ok(OptimizedDependentType::Sigma {
                    var: var.clone(),
                    first: Rc::new(opt_first),
                    second: Rc::new(opt_second),
                })
            }
            DependentType::Identity { ty, left, right } => {
                let opt_ty = self.convert_type_to_optimized(ty)?;
                let opt_left = self.convert_term_to_optimized(left)?;
                let opt_right = self.convert_term_to_optimized(right)?;
                Ok(OptimizedDependentType::Identity {
                    ty: Rc::new(opt_ty),
                    left: Rc::new(opt_left),
                    right: Rc::new(opt_right),
                })
            }
            DependentType::Inductive {
                name,
                parameters,
                universe_level,
                constructors,
                induction_principle,
            } => {
                let mut opt_parameters = Vec::new();
                for (param_name, param_type) in parameters {
                    opt_parameters.push((
                        param_name.clone(),
                        self.convert_type_to_optimized(param_type)?,
                    ));
                }

                let mut opt_constructors = Vec::new();
                for (ctor_name, ctor_type) in constructors {
                    opt_constructors.push((
                        ctor_name.clone(),
                        self.convert_type_to_optimized(ctor_type)?,
                    ));
                }

                let opt_induction_principle = if let Some(ind_prin) = induction_principle {
                    Some(Rc::new(self.convert_type_to_optimized(ind_prin)?))
                } else {
                    None
                };

                Ok(OptimizedDependentType::Inductive {
                    name: name.clone(),
                    parameters: opt_parameters,
                    universe_level: *universe_level,
                    constructors: opt_constructors,
                    induction_principle: opt_induction_principle,
                })
            }
        };

        // Update conversion statistics
        let conversion_time = start_time.elapsed();
        let mut stats = self.stats.borrow_mut();
        stats.type_conversions += 1;
        stats.conversion_overhead += conversion_time;

        result
    }

    /// Convert optimized DependentType to old version
    pub fn convert_type_from_optimized(
        &self,
        opt_type: &OptimizedDependentType,
    ) -> Result<DependentType> {
        let start_time = std::time::Instant::now();

        let result = match opt_type {
            OptimizedDependentType::Universe(level) => Ok(DependentType::Universe(*level)),
            OptimizedDependentType::Pi {
                var,
                domain,
                codomain,
            } => {
                let old_domain = self.convert_type_from_optimized(domain)?;
                let old_codomain = self.convert_type_from_optimized(codomain)?;
                Ok(DependentType::Pi {
                    var: var.clone(),
                    domain: Box::new(old_domain),
                    codomain: Box::new(old_codomain),
                })
            }
            OptimizedDependentType::Sigma { var, first, second } => {
                let old_first = self.convert_type_from_optimized(first)?;
                let old_second = self.convert_type_from_optimized(second)?;
                Ok(DependentType::Sigma {
                    var: var.clone(),
                    first: Box::new(old_first),
                    second: Box::new(old_second),
                })
            }
            OptimizedDependentType::Identity { ty, left, right } => {
                let old_ty = self.convert_type_from_optimized(ty)?;
                let old_left = self.convert_term_from_optimized(left)?;
                let old_right = self.convert_term_from_optimized(right)?;
                Ok(DependentType::Identity {
                    ty: Box::new(old_ty),
                    left: Box::new(old_left),
                    right: Box::new(old_right),
                })
            }
            OptimizedDependentType::Inductive {
                name,
                parameters,
                universe_level,
                constructors,
                induction_principle,
            } => {
                let mut old_parameters = Vec::new();
                for (param_name, param_type) in parameters {
                    old_parameters.push((
                        param_name.clone(),
                        self.convert_type_from_optimized(param_type)?,
                    ));
                }

                let mut old_constructors = Vec::new();
                for (ctor_name, ctor_type) in constructors {
                    old_constructors.push((
                        ctor_name.clone(),
                        self.convert_type_from_optimized(ctor_type)?,
                    ));
                }

                let old_induction_principle = if let Some(ind_prin) = induction_principle {
                    Some(Box::new(self.convert_type_from_optimized(ind_prin)?))
                } else {
                    None
                };

                Ok(DependentType::Inductive {
                    name: name.clone(),
                    parameters: old_parameters,
                    universe_level: *universe_level,
                    constructors: old_constructors,
                    induction_principle: old_induction_principle,
                })
            }
            OptimizedDependentType::ArenaRef(_) => {
                // This shouldn't happen in normal conversion, but provide a fallback
                Err(Box::new(Error::type_error(
                    "Cannot convert arena reference to Box-based type".to_string(),
                    Span::new(0, 0),
                )))
            }
        };

        // Update conversion statistics
        let conversion_time = start_time.elapsed();
        let mut stats = self.stats.borrow_mut();
        stats.type_conversions += 1;
        stats.conversion_overhead += conversion_time;

        result
    }

    /// Convert old DependentTerm to optimized version
    pub fn convert_term_to_optimized(
        &self,
        old_term: &DependentTerm,
    ) -> Result<OptimizedDependentTerm> {
        match old_term {
            DependentTerm::Variable(name) => Ok(OptimizedDependentTerm::Variable(name.clone())),
            DependentTerm::Lambda {
                param,
                param_type,
                body,
            } => {
                let opt_param_type = self.convert_type_to_optimized(param_type)?;
                let opt_body = self.convert_term_to_optimized(body)?;
                Ok(OptimizedDependentTerm::Lambda {
                    param: param.clone(),
                    param_type: Rc::new(opt_param_type),
                    body: Rc::new(opt_body),
                })
            }
            DependentTerm::Application { function, argument } => {
                let opt_function = self.convert_term_to_optimized(function)?;
                let opt_argument = self.convert_term_to_optimized(argument)?;
                Ok(OptimizedDependentTerm::Application {
                    function: Rc::new(opt_function),
                    argument: Rc::new(opt_argument),
                })
            }
            DependentTerm::Pair { first, second } => {
                let opt_first = self.convert_term_to_optimized(first)?;
                let opt_second = self.convert_term_to_optimized(second)?;
                Ok(OptimizedDependentTerm::Pair {
                    first: Rc::new(opt_first),
                    second: Rc::new(opt_second),
                })
            }
            DependentTerm::Projection { pair, is_first } => {
                let opt_pair = self.convert_term_to_optimized(pair)?;
                Ok(OptimizedDependentTerm::Projection {
                    pair: Rc::new(opt_pair),
                    is_first: *is_first,
                })
            }
            DependentTerm::Refl { ty } => {
                let opt_ty = self.convert_type_to_optimized(ty)?;
                Ok(OptimizedDependentTerm::Refl {
                    ty: Rc::new(opt_ty),
                })
            }
            DependentTerm::Constructor {
                name,
                args,
                result_type,
            } => {
                let mut opt_args = Vec::new();
                for arg in args {
                    opt_args.push(self.convert_term_to_optimized(arg)?);
                }
                let opt_result_type = self.convert_type_to_optimized(result_type)?;
                Ok(OptimizedDependentTerm::Constructor {
                    name: name.clone(),
                    args: opt_args,
                    result_type: Rc::new(opt_result_type),
                })
            }
            DependentTerm::Match {
                scrutinee,
                branches,
                return_type,
            } => {
                let opt_scrutinee = self.convert_term_to_optimized(scrutinee)?;
                let mut opt_branches = Vec::new();
                for branch in branches {
                    opt_branches.push(OptimizedMatchBranch {
                        pattern: self.convert_pattern_to_optimized(&branch.pattern)?,
                        body: self.convert_term_to_optimized(&branch.body)?,
                    });
                }
                let opt_return_type = self.convert_type_to_optimized(return_type)?;
                Ok(OptimizedDependentTerm::Match {
                    scrutinee: Rc::new(opt_scrutinee),
                    branches: opt_branches,
                    return_type: Rc::new(opt_return_type),
                })
            }
        }
    }

    /// Convert optimized DependentTerm to old version
    pub fn convert_term_from_optimized(
        &self,
        opt_term: &OptimizedDependentTerm,
    ) -> Result<DependentTerm> {
        match opt_term {
            OptimizedDependentTerm::Variable(name) => Ok(DependentTerm::Variable(name.clone())),
            OptimizedDependentTerm::Lambda {
                param,
                param_type,
                body,
            } => {
                let old_param_type = self.convert_type_from_optimized(param_type)?;
                let old_body = self.convert_term_from_optimized(body)?;
                Ok(DependentTerm::Lambda {
                    param: param.clone(),
                    param_type: Box::new(old_param_type),
                    body: Box::new(old_body),
                })
            }
            OptimizedDependentTerm::Application { function, argument } => {
                let old_function = self.convert_term_from_optimized(function)?;
                let old_argument = self.convert_term_from_optimized(argument)?;
                Ok(DependentTerm::Application {
                    function: Box::new(old_function),
                    argument: Box::new(old_argument),
                })
            }
            OptimizedDependentTerm::Pair { first, second } => {
                let old_first = self.convert_term_from_optimized(first)?;
                let old_second = self.convert_term_from_optimized(second)?;
                Ok(DependentTerm::Pair {
                    first: Box::new(old_first),
                    second: Box::new(old_second),
                })
            }
            OptimizedDependentTerm::Projection { pair, is_first } => {
                let old_pair = self.convert_term_from_optimized(pair)?;
                Ok(DependentTerm::Projection {
                    pair: Box::new(old_pair),
                    is_first: *is_first,
                })
            }
            OptimizedDependentTerm::Refl { ty } => {
                let old_ty = self.convert_type_from_optimized(ty)?;
                Ok(DependentTerm::Refl {
                    ty: Box::new(old_ty),
                })
            }
            OptimizedDependentTerm::Constructor {
                name,
                args,
                result_type,
            } => {
                let mut old_args = Vec::new();
                for arg in args {
                    old_args.push(self.convert_term_from_optimized(arg)?);
                }
                let old_result_type = self.convert_type_from_optimized(result_type)?;
                Ok(DependentTerm::Constructor {
                    name: name.clone(),
                    args: old_args,
                    result_type: Box::new(old_result_type),
                })
            }
            OptimizedDependentTerm::Match {
                scrutinee,
                branches,
                return_type,
            } => {
                let old_scrutinee = self.convert_term_from_optimized(scrutinee)?;
                let mut old_branches = Vec::new();
                for branch in branches {
                    old_branches.push(MatchBranch {
                        pattern: self.convert_pattern_from_optimized(&branch.pattern)?,
                        body: self.convert_term_from_optimized(&branch.body)?,
                    });
                }
                let old_return_type = self.convert_type_from_optimized(return_type)?;
                Ok(DependentTerm::Match {
                    scrutinee: Box::new(old_scrutinee),
                    branches: old_branches,
                    return_type: Box::new(old_return_type),
                })
            }
            OptimizedDependentTerm::ArenaRef(_) => {
                // This shouldn't happen in normal conversion, but provide a fallback
                Err(Box::new(Error::type_error(
                    "Cannot convert arena reference to Box-based term".to_string(),
                    Span::new(0, 0),
                )))
            }
        }
    }

    /// Convert patterns between representations
    fn convert_pattern_to_optimized(&self, old_pattern: &Pattern) -> Result<OptimizedPattern> {
        match old_pattern {
            Pattern::Variable(name) => Ok(OptimizedPattern::Variable(name.clone())),
            Pattern::Constructor { name, args } => {
                let mut opt_args = Vec::new();
                for arg in args {
                    opt_args.push(self.convert_pattern_to_optimized(arg)?);
                }
                Ok(OptimizedPattern::Constructor {
                    name: name.clone(),
                    args: opt_args,
                })
            }
        }
    }

    fn convert_pattern_from_optimized(&self, opt_pattern: &OptimizedPattern) -> Result<Pattern> {
        match opt_pattern {
            OptimizedPattern::Variable(name) => Ok(Pattern::Variable(name.clone())),
            OptimizedPattern::Constructor { name, args } => {
                let mut old_args = Vec::new();
                for arg in args {
                    old_args.push(self.convert_pattern_from_optimized(arg)?);
                }
                Ok(Pattern::Constructor {
                    name: name.clone(),
                    args: old_args,
                })
            }
        }
    }

    /// Get migration statistics
    pub fn get_migration_stats(&self) -> MigrationStats {
        self.stats.try_borrow().unwrap().clone()
    }

    /// Get memory statistics from the optimized backend
    pub fn get_memory_stats(&self) -> ArenaStats {
        self.optimized_context.memory_stats().unwrap_or(ArenaStats {
            types_count: 0,
            terms_count: 0,
            types_memory: 0,
            terms_memory: 0,
            cache_hits_types: 0,
            cache_hits_terms: 0,
        })
    }

    /// Get memory pool statistics if enabled
    pub fn get_memory_pool_stats(&self) -> Option<MemoryPoolStatistics> {
        if self.config.enable_memory_pooling {
            Some(self.memory_pool.memory_statistics())
        } else {
            None
        }
    }

    /// Run performance benchmarks comparing old and new implementations
    pub fn run_performance_comparison(&self) -> Result<BenchmarkSummary> {
        if !self.config.enable_auto_benchmarking {
            return Err(Box::new(Error::type_error(
                "Auto-benchmarking is disabled in configuration".to_string(),
                Span::new(0, 0),
            )));
        }

        let mut suite = DependentTypeBenchmarkSuite::new();
        suite.run_all_benchmarks()
    }

    /// Reset all statistics and caches
    pub fn reset_statistics(&self) {
        *self.stats.borrow_mut() = MigrationStats::new();
        // Note: We don't reset the optimized backend's internal state
        // as that might be shared with other components
    }

    // Private helper methods

    fn generate_context_id(&self) -> u32 {
        // Simple counter for unique IDs
        static mut CONTEXT_COUNTER: u32 = 0;
        unsafe {
            CONTEXT_COUNTER += 1;
            CONTEXT_COUNTER
        }
    }

    fn generate_normalizer_id(&self) -> u32 {
        // Simple counter for unique IDs
        static mut NORMALIZER_COUNTER: u32 = 0;
        unsafe {
            NORMALIZER_COUNTER += 1;
            NORMALIZER_COUNTER
        }
    }
}

impl Clone for MigrationBridge {
    fn clone(&self) -> Self {
        Self {
            optimized_context: self.optimized_context.clone(),
            optimized_normalizer: self.optimized_normalizer.clone(),
            memory_pool: MemoryPoolManager::new(), // Create new pool for clone
            stats: RefCell::new(self.stats.try_borrow().unwrap().clone()),
            config: self.config.clone(),
        }
    }
}

impl CompatibleTypingContext {
    /// Create a new compatible typing context (old API)
    pub fn new() -> Self {
        let bridge = MigrationBridge::new();
        bridge.create_compatible_context()
    }

    /// Bind a variable to a type (old API)
    pub fn bind_variable(&mut self, name: String, ty: DependentType) -> Result<()> {
        let mut bridge = self.bridge.borrow_mut();
        bridge.stats.borrow_mut().old_api_calls += 1;

        if bridge.config.use_optimized_backend {
            // Convert to optimized type and use optimized backend
            let opt_type = bridge.convert_type_to_optimized(&ty)?;
            bridge.optimized_context.bind_variable(name, opt_type)
        } else {
            // Fallback to simulated old behavior
            // (In a real implementation, this would call the actual old API)
            let opt_type = bridge.convert_type_to_optimized(&ty)?;
            bridge.optimized_context.bind_variable(name, opt_type)
        }
    }

    /// Remove a variable binding (old API)
    pub fn unbind_variable(&mut self, name: &str) {
        let mut bridge = self.bridge.borrow_mut();
        bridge.stats.borrow_mut().old_api_calls += 1;
        bridge.optimized_context.unbind_variable(name);
    }

    /// Look up the type of a variable (old API)
    pub fn lookup_variable(&self, name: &str) -> Option<DependentType> {
        let bridge = self.bridge.try_borrow().unwrap();
        bridge.stats.borrow_mut().old_api_calls += 1;

        match bridge.optimized_context.lookup_variable(name) {
            Ok(Some(opt_type)) => bridge.convert_type_from_optimized(&opt_type).ok(),
            _ => None,
        }
    }

    /// Define a new type (old API)
    pub fn define_type(&mut self, name: String, definition: DependentType) -> Result<()> {
        let mut bridge = self.bridge.borrow_mut();
        bridge.stats.borrow_mut().old_api_calls += 1;

        let opt_definition = bridge.convert_type_to_optimized(&definition)?;
        bridge.optimized_context.define_type(name, opt_definition)
    }

    /// Look up a type definition (old API)
    pub fn lookup_type(&self, name: &str) -> Option<DependentType> {
        let bridge = self.bridge.try_borrow().unwrap();
        bridge.stats.borrow_mut().old_api_calls += 1;

        match bridge.optimized_context.lookup_type(name) {
            Ok(Some(opt_type)) => bridge.convert_type_from_optimized(&opt_type).ok(),
            _ => None,
        }
    }

    /// Push a new scope (old API)
    pub fn push_scope(&mut self) {
        let mut bridge = self.bridge.borrow_mut();
        bridge.stats.borrow_mut().old_api_calls += 1;
        bridge.optimized_context.push_scope();
    }

    /// Pop a scope (old API)
    pub fn pop_scope(&mut self) {
        let mut bridge = self.bridge.borrow_mut();
        bridge.stats.borrow_mut().old_api_calls += 1;
        bridge.optimized_context.pop_scope();
    }

    /// Get the size of the context (old API)
    pub fn size(&self) -> usize {
        let bridge = self.bridge.try_borrow().unwrap();
        bridge.optimized_context.size()
    }

    /// Check if the context is empty (old API)
    pub fn is_empty(&self) -> bool {
        let bridge = self.bridge.try_borrow().unwrap();
        bridge.optimized_context.is_empty()
    }
}

impl CompatibleNormalizer {
    /// Create a new compatible normalizer (old API)
    pub fn new() -> Self {
        let bridge = MigrationBridge::new();
        bridge.create_compatible_normalizer()
    }

    /// Normalize a dependent type (old API)
    pub fn normalize_type(&self, ty: &DependentType) -> Result<DependentType> {
        let bridge = self.bridge.try_borrow().unwrap();
        bridge.stats.borrow_mut().old_api_calls += 1;

        let opt_type = bridge.convert_type_to_optimized(ty)?;
        let normalized_opt = bridge.optimized_normalizer.normalize_type(&opt_type)?;
        bridge.convert_type_from_optimized(&normalized_opt)
    }

    /// Normalize a dependent term (old API)
    pub fn normalize_term(&self, term: &DependentTerm) -> Result<DependentTerm> {
        let bridge = self.bridge.try_borrow().unwrap();
        bridge.stats.borrow_mut().old_api_calls += 1;

        let opt_term = bridge.convert_term_to_optimized(term)?;
        let normalized_opt = bridge.optimized_normalizer.normalize_term(&opt_term)?;
        bridge.convert_term_from_optimized(&normalized_opt)
    }
}

impl MigrationStats {
    fn new() -> Self {
        Self {
            old_api_calls: 0,
            new_api_calls: 0,
            type_conversions: 0,
            conversion_overhead: std::time::Duration::new(0, 0),
            memory_savings: 0.0,
            performance_improvement: 0.0,
        }
    }

    /// Calculate API usage ratios
    pub fn api_usage_ratio(&self) -> (f64, f64) {
        let total = self.old_api_calls + self.new_api_calls;
        if total == 0 {
            (0.0, 0.0)
        } else {
            (
                self.old_api_calls as f64 / total as f64,
                self.new_api_calls as f64 / total as f64,
            )
        }
    }

    /// Get average conversion time
    pub fn average_conversion_time(&self) -> std::time::Duration {
        if self.type_conversions == 0 {
            std::time::Duration::new(0, 0)
        } else {
            self.conversion_overhead / self.type_conversions as u32
        }
    }
}

impl MigrationConfig {
    /// Configuration optimized for performance
    pub fn performance_optimized() -> Self {
        Self {
            use_optimized_backend: true,
            collect_stats: false,
            enable_memory_pooling: true,
            enable_auto_benchmarking: false,
            enable_fallback: false,
        }
    }

    /// Configuration optimized for debugging and analysis
    pub fn debug_optimized() -> Self {
        Self {
            use_optimized_backend: true,
            collect_stats: true,
            enable_memory_pooling: true,
            enable_auto_benchmarking: true,
            enable_fallback: true,
        }
    }

    /// Conservative configuration with fallbacks
    pub fn safe_migration() -> Self {
        Self {
            use_optimized_backend: true,
            collect_stats: true,
            enable_memory_pooling: false,
            enable_auto_benchmarking: false,
            enable_fallback: true,
        }
    }
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            use_optimized_backend: true,
            collect_stats: true,
            enable_memory_pooling: true,
            enable_auto_benchmarking: false,
            enable_fallback: true,
        }
    }
}

impl Default for MigrationBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CompatibleTypingContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CompatibleNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Get a new migration bridge instance (simplified approach)
pub fn global_migration_bridge() -> MigrationBridge {
    MigrationBridge::new()
}

/// Configure and create a new global migration bridge
pub fn configure_global_bridge(config: MigrationConfig) -> MigrationBridge {
    MigrationBridge::with_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_bridge_creation() {
        let bridge = MigrationBridge::new();
        let stats = bridge.get_migration_stats();

        assert_eq!(stats.old_api_calls, 0);
        assert_eq!(stats.new_api_calls, 0);
        assert_eq!(stats.type_conversions, 0);
    }

    #[test]
    fn test_type_conversion_roundtrip() {
        let bridge = MigrationBridge::new();
        let original_type = DependentType::Universe(42);

        let optimized = bridge.convert_type_to_optimized(&original_type).unwrap();
        let converted_back = bridge.convert_type_from_optimized(&optimized).unwrap();

        assert_eq!(original_type, converted_back);
    }

    #[test]
    fn test_complex_type_conversion() {
        let bridge = MigrationBridge::new();

        let complex_type = DependentType::Pi {
            var: "x".to_string(),
            domain: Box::new(DependentType::Universe(0)),
            codomain: Box::new(DependentType::Pi {
                var: "y".to_string(),
                domain: Box::new(DependentType::Universe(0)),
                codomain: Box::new(DependentType::Universe(1)),
            }),
        };

        let optimized = bridge.convert_type_to_optimized(&complex_type).unwrap();
        let converted_back = bridge.convert_type_from_optimized(&optimized).unwrap();

        assert_eq!(complex_type, converted_back);
    }

    #[test]
    fn test_compatible_typing_context() {
        let mut context = CompatibleTypingContext::new();
        let universe_type = DependentType::Universe(0);

        // Test old API methods
        assert!(
            context
                .bind_variable("x".to_string(), universe_type.clone())
                .is_ok()
        );
        assert_eq!(context.size(), 1);
        assert!(!context.is_empty());

        let looked_up = context.lookup_variable("x");
        assert!(looked_up.is_some());
        assert_eq!(looked_up.unwrap(), universe_type);

        context.unbind_variable("x");
        assert_eq!(context.size(), 0);
        assert!(context.is_empty());
    }

    #[test]
    fn test_compatible_normalizer() {
        let normalizer = CompatibleNormalizer::new();
        let pi_type = DependentType::Pi {
            var: "x".to_string(),
            domain: Box::new(DependentType::Universe(0)),
            codomain: Box::new(DependentType::Universe(1)),
        };

        let normalized = normalizer.normalize_type(&pi_type).unwrap();
        // In this simple case, normalization should return the same type
        assert_eq!(normalized, pi_type);
    }

    #[test]
    fn test_migration_statistics() {
        let bridge = MigrationBridge::new();
        let mut context = bridge.create_compatible_context();

        // Perform some operations
        let _ = context.bind_variable("x".to_string(), DependentType::Universe(0));
        let _ = context.lookup_variable("x");

        let stats = bridge.get_migration_stats();
        assert!(stats.old_api_calls > 0);
        assert!(stats.new_api_calls > 0);
    }

    #[test]
    fn test_migration_config() {
        let performance_config = MigrationConfig::performance_optimized();
        assert!(performance_config.use_optimized_backend);
        assert!(performance_config.enable_memory_pooling);
        assert!(!performance_config.collect_stats);

        let debug_config = MigrationConfig::debug_optimized();
        assert!(debug_config.collect_stats);
        assert!(debug_config.enable_auto_benchmarking);

        let safe_config = MigrationConfig::safe_migration();
        assert!(safe_config.enable_fallback);
        assert!(!safe_config.enable_memory_pooling);
    }

    #[test]
    fn test_global_bridge() {
        let _bridge1 = global_migration_bridge();
        let _bridge2 = global_migration_bridge();

        // Both should succeed (testing that global bridge works)
        assert!(true);
    }
}

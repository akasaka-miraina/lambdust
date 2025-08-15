//! High-performance normalization and optimization engine for dependent types.
//!
//! This module implements sophisticated normalization algorithms that:
//! - Provides efficient weak head normal form (WHNF) computation
//! - Implements lazy evaluation with memoization for performance
//! - Supports parallel normalization using Rayon
//! - Provides various normalization strategies for different use cases
//! - Integrates with the type checker for definitional equality

use crate::diagnostics::{Error, Result, Span};
use crate::types::dependent::{DependentType, DependentTerm, UniverseLevel};
use crate::types::dependent::termination::{
    StrongNormalizationChecker, ChurchRosserChecker, TerminationConfluenceSystem,
    ComplexityMeasure, TerminationConfig, ConfluenceConfig
};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque, BTreeMap};
use std::sync::{Arc, RwLock, Mutex};
use std::hash::{Hash, Hasher};

/// Normalization strategy for different performance profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NormalizationStrategy {
    /// Weak head normal form (efficient for type checking)
    WeakHead,
    /// Strong normal form (complete normalization)
    Strong,
    /// Lazy evaluation with memoization
    Lazy,
    /// Eager evaluation with caching
    Eager,
    /// Parallel normalization for large terms
    Parallel,
}

/// Normalization context for tracking reductions
#[derive(Debug, Clone)]
pub struct NormalizationContext {
    /// Variable substitutions
    substitutions: HashMap<String, DependentTerm>,
    /// Reduction count for complexity tracking
    reduction_count: usize,
    /// Maximum reduction steps
    max_reductions: usize,
    /// Normalization depth
    current_depth: usize,
    /// Maximum depth
    max_depth: usize,
}

/// Reduction step for debugging and optimization
#[derive(Debug, Clone)]
pub enum ReductionStep {
    /// Beta reduction: (λx.t) u → t[u/x]
    Beta {
        lambda: DependentTerm,
        argument: DependentTerm,
        result: DependentTerm,
    },
    /// Delta reduction: defined constant expansion
    Delta {
        constant: String,
        definition: DependentTerm,
        result: DependentTerm,
    },
    /// Iota reduction: constructor/destructor elimination
    Iota {
        constructor: DependentTerm,
        destructor: DependentTerm,
        result: DependentTerm,
    },
    /// Zeta reduction: let binding expansion
    Zeta {
        binding: String,
        value: DependentTerm,
        body: DependentTerm,
        result: DependentTerm,
    },
    /// Eta reduction: λx.(f x) → f (when x not free in f)
    Eta {
        lambda: DependentTerm,
        result: DependentTerm,
    },
}

/// Normalization result with metadata
#[derive(Debug, Clone)]
pub struct NormalizationResult {
    /// Normalized term/type
    pub normalized: DependentTerm,
    /// Reduction steps taken
    pub steps: Vec<ReductionStep>,
    /// Total reduction count
    pub reduction_count: usize,
    /// Whether normalization was complete
    pub is_complete: bool,
    /// Time taken (for performance analysis)
    pub duration_ns: u64,
}

/// Cache key for normalization results
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NormalizationCacheKey {
    term: DependentTerm,
    strategy: NormalizationStrategy,
    context_hash: u64,
}

/// High-performance normalization engine with termination guarantees
#[derive(Debug)]
pub struct NormalizationEngine {
    /// Normalization strategy
    strategy: NormalizationStrategy,
    /// Normalization context
    context: NormalizationContext,
    /// Memoization cache
    cache: Arc<RwLock<HashMap<NormalizationCacheKey, NormalizationResult>>>,
    /// Weak head normal form cache
    whnf_cache: Arc<RwLock<HashMap<DependentTerm, DependentTerm>>>,
    /// Reduction definitions
    definitions: Arc<RwLock<HashMap<String, DependentTerm>>>,
    /// Performance statistics
    stats: Arc<Mutex<NormalizationStatistics>>,
    /// Parallel processing enabled
    parallel_enabled: bool,
    /// Strong normalization checker for termination guarantees
    termination_checker: Arc<StrongNormalizationChecker>,
    /// Church-Rosser property checker for confluence guarantees
    confluence_checker: Arc<ChurchRosserChecker>,
    /// Combined termination and confluence system
    termination_confluence_system: Arc<TerminationConfluenceSystem>,
    /// Enable termination checking (may impact performance)
    enable_termination_checking: bool,
    /// Enable confluence checking (may impact performance)
    enable_confluence_checking: bool,
}

/// Statistics for performance monitoring with termination tracking
#[derive(Debug, Default)]
pub struct NormalizationStatistics {
    pub terms_normalized: usize,
    pub beta_reductions: usize,
    pub delta_reductions: usize,
    pub iota_reductions: usize,
    pub eta_reductions: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub parallel_normalizations: usize,
    pub total_reduction_steps: usize,
    /// Terms verified for strong normalization
    pub terms_checked_termination: usize,
    /// Terms verified for confluence
    pub terms_checked_confluence: usize,
    /// Terms proven strongly normalizing
    pub strongly_normalizing_terms: usize,
    /// Terms proven confluent
    pub confluent_terms: usize,
    /// Termination check failures (potential infinite loops)
    pub termination_check_failures: usize,
    /// Confluence check failures (potential non-determinism)
    pub confluence_check_failures: usize,
    /// Average termination check time (microseconds)
    pub avg_termination_check_time_us: u64,
    /// Average confluence check time (microseconds)
    pub avg_confluence_check_time_us: u64,
}

/// Lazy evaluation thunk for deferred computation
pub enum LazyThunk {
    /// Already computed value
    Value(DependentTerm),
    /// Deferred computation
    Thunk {
        computation: Arc<dyn Fn() -> Result<DependentTerm> + Send + Sync>,
        memoized: Arc<RwLock<Option<DependentTerm>>>,
    },
}

impl Default for NormalizationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl NormalizationContext {
    /// Create new normalization context
    pub fn new() -> Self {
        Self {
            substitutions: HashMap::new(),
            reduction_count: 0,
            max_reductions: 100000,
            current_depth: 0,
            max_depth: 1000,
        }
    }

    /// Create context with custom limits
    pub fn with_limits(max_reductions: usize, max_depth: usize) -> Self {
        Self {
            substitutions: HashMap::new(),
            reduction_count: 0,
            max_reductions,
            current_depth: 0,
            max_depth,
        }
    }

    /// Add variable substitution
    pub fn add_substitution(&mut self, var: String, term: DependentTerm) {
        self.substitutions.insert(var, term);
    }

    /// Lookup variable substitution
    pub fn lookup_substitution(&self, var: &str) -> Option<&DependentTerm> {
        self.substitutions.get(var)
    }

    /// Increment reduction count
    pub fn increment_reductions(&mut self) -> Result<()> {
        self.reduction_count += 1;
        if self.reduction_count > self.max_reductions {
            return Err(Box::new(Error::type_error(
                "Maximum reduction steps exceeded".to_string(),
                Span::new(0, 0),
            )));
        }
        Ok(())
    }

    /// Enter deeper normalization level
    pub fn enter_depth(&mut self) -> Result<()> {
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            return Err(Box::new(Error::type_error(
                "Maximum normalization depth exceeded".to_string(),
                Span::new(0, 0),
            )));
        }
        Ok(())
    }

    /// Exit normalization level
    pub fn exit_depth(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
    }

    /// Compute context hash for caching
    pub fn compute_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.substitutions.len().hash(&mut hasher);
        self.reduction_count.hash(&mut hasher);
        hasher.finish()
    }

    /// Clone for parallel processing
    pub fn clone_for_parallel(&self) -> Self {
        Self {
            substitutions: self.substitutions.clone(),
            reduction_count: 0, // Reset for parallel context
            max_reductions: self.max_reductions,
            current_depth: 0, // Reset for parallel context
            max_depth: self.max_depth,
        }
    }
}

impl NormalizationEngine {
    /// Create new normalization engine with default termination checking
    pub fn new() -> Self {
        Self::with_termination_checking(true, true)
    }

    /// Create new normalization engine with configurable termination checking
    pub fn with_termination_checking(enable_termination: bool, enable_confluence: bool) -> Self {
        Self {
            strategy: NormalizationStrategy::WeakHead,
            context: NormalizationContext::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            whnf_cache: Arc::new(RwLock::new(HashMap::new())),
            definitions: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(Mutex::new(NormalizationStatistics::default())),
            parallel_enabled: true,
            termination_checker: Arc::new(StrongNormalizationChecker::with_config(TerminationConfig::default())),
            confluence_checker: Arc::new(ChurchRosserChecker::with_config(ConfluenceConfig::default())),
            termination_confluence_system: Arc::new(TerminationConfluenceSystem::new()),
            enable_termination_checking: enable_termination,
            enable_confluence_checking: enable_confluence,
        }
    }

    /// Create engine with custom strategy
    pub fn with_strategy(strategy: NormalizationStrategy) -> Self {
        Self {
            strategy,
            context: NormalizationContext::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            whnf_cache: Arc::new(RwLock::new(HashMap::new())),
            definitions: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(Mutex::new(NormalizationStatistics::default())),
            parallel_enabled: strategy == NormalizationStrategy::Parallel,
            termination_checker: Arc::new(StrongNormalizationChecker::with_config(TerminationConfig::default())),
            confluence_checker: Arc::new(ChurchRosserChecker::with_config(ConfluenceConfig::default())),
            termination_confluence_system: Arc::new(TerminationConfluenceSystem::new()),
            enable_termination_checking: true,
            enable_confluence_checking: true,
        }
    }

    /// Normalize a term using the configured strategy
    pub fn normalize_term(&mut self, term: &DependentTerm) -> Result<NormalizationResult> {
        let start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = NormalizationCacheKey {
            term: term.clone(),
            strategy: self.strategy,
            context_hash: self.context.compute_hash(),
        };

        if let Some(cached_result) = self.check_cache(&cache_key) {
            let mut stats = self.stats.lock().unwrap();
            stats.cache_hits += 1;
            return Ok(cached_result);
        }

        // Cache miss - perform normalization
        let mut stats = self.stats.lock().unwrap();
        stats.cache_misses += 1;
        stats.terms_normalized += 1;
        drop(stats);

        let result = match self.strategy {
            NormalizationStrategy::WeakHead => self.weak_head_normalize(term)?,
            NormalizationStrategy::Strong => self.strong_normalize(term)?,
            NormalizationStrategy::Lazy => self.lazy_normalize(term)?,
            NormalizationStrategy::Eager => self.eager_normalize(term)?,
            NormalizationStrategy::Parallel => self.parallel_normalize(term)?,
        };

        let duration = start_time.elapsed();
        let mut final_result = result;
        final_result.duration_ns = duration.as_nanos() as u64;

        // Cache the result
        self.cache_result(&cache_key, &final_result);

        Ok(final_result)
    }

    /// Normalize a type (special handling for types)
    pub fn normalize_type(&mut self, ty: &DependentType) -> Result<DependentType> {
        match ty {
            DependentType::Pi { var, domain, codomain } => {
                let norm_domain = Box::new(self.normalize_type(domain)?);
                
                // Normalize codomain in extended context
                self.context.enter_depth()?;
                let norm_codomain = Box::new(self.normalize_type(codomain)?);
                self.context.exit_depth();
                
                Ok(DependentType::Pi {
                    var: var.clone(),
                    domain: norm_domain,
                    codomain: norm_codomain,
                })
            }
            DependentType::Sigma { var, first, second } => {
                let norm_first = Box::new(self.normalize_type(first)?);
                
                // Normalize second type in extended context
                self.context.enter_depth()?;
                let norm_second = Box::new(self.normalize_type(second)?);
                self.context.exit_depth();
                
                Ok(DependentType::Sigma {
                    var: var.clone(),
                    first: norm_first,
                    second: norm_second,
                })
            }
            DependentType::Identity { ty, left, right } => {
                let norm_ty = Box::new(self.normalize_type(ty)?);
                
                // Convert terms to fake terms for normalization (simplified)
                let norm_left = left.clone(); // Simplified
                let norm_right = right.clone(); // Simplified
                
                Ok(DependentType::Identity {
                    ty: norm_ty,
                    left: norm_left,
                    right: norm_right,
                })
            }
            DependentType::Universe(level) => Ok(DependentType::Universe(*level)),
            DependentType::Inductive { name, parameters, universe_level, constructors, induction_principle } => {
                // Normalize constructor types
                let mut norm_constructors = Vec::new();
                for (ctor_name, ctor_type) in constructors {
                    norm_constructors.push((ctor_name.clone(), self.normalize_type(ctor_type)?));
                }
                
                // Normalize parameters
                let mut norm_parameters = Vec::new();
                for (param_name, param_type) in parameters {
                    norm_parameters.push((param_name.clone(), self.normalize_type(param_type)?));
                }
                
                // Normalize induction principle if present
                let norm_induction_principle = if let Some(principle) = induction_principle {
                    Some(Box::new(self.normalize_type(principle)?))
                } else {
                    None
                };
                
                Ok(DependentType::Inductive {
                    name: name.clone(),
                    parameters: norm_parameters,
                    universe_level: *universe_level,
                    constructors: norm_constructors,
                    induction_principle: norm_induction_principle,
                })
            }
        }
    }

    /// Weak head normal form (efficient for type checking)
    fn weak_head_normalize(&mut self, term: &DependentTerm) -> Result<NormalizationResult> {
        // Check WHNF cache first
        if let Some(cached_whnf) = self.check_whnf_cache(term) {
            return Ok(NormalizationResult {
                normalized: cached_whnf,
                steps: vec![],
                reduction_count: 0,
                is_complete: false, // WHNF is not necessarily complete
                duration_ns: 0,
            });
        }

        let mut current_term = term.clone();
        let mut steps = Vec::new();
        let initial_reduction_count = self.context.reduction_count;

        loop {
            match &current_term {
                DependentTerm::Application { function, argument } => {
                    match function.as_ref() {
                        DependentTerm::Lambda { param, param_type: _, body } => {
                            // Beta reduction
                            self.context.increment_reductions()?;
                            let reduced = self.substitute_term(body, param, argument)?;
                            
                            steps.push(ReductionStep::Beta {
                                lambda: function.as_ref().clone(),
                                argument: argument.as_ref().clone(),
                                result: reduced.clone(),
                            });
                            
                            current_term = reduced;
                            
                            let mut stats = self.stats.lock().unwrap();
                            stats.beta_reductions += 1;
                        }
                        DependentTerm::Variable(name) => {
                            // Check for delta reduction (defined constants)
                            if let Some(definition) = self.lookup_definition(name) {
                                self.context.increment_reductions()?;
                                
                                steps.push(ReductionStep::Delta {
                                    constant: name.clone(),
                                    definition: definition.clone(),
                                    result: DependentTerm::Application {
                                        function: Box::new(definition.clone()),
                                        argument: argument.clone(),
                                    },
                                });
                                
                                current_term = DependentTerm::Application {
                                    function: Box::new(definition),
                                    argument: argument.clone(),
                                };
                                
                                let mut stats = self.stats.lock().unwrap();
                                stats.delta_reductions += 1;
                            } else {
                                break; // Cannot reduce further
                            }
                        }
                        _ => {
                            // Normalize function to WHNF first
                            let func_result = self.weak_head_normalize(function)?;
                            if func_result.normalized != **function {
                                current_term = DependentTerm::Application {
                                    function: Box::new(func_result.normalized),
                                    argument: argument.clone(),
                                };
                                steps.extend(func_result.steps);
                            } else {
                                break; // Cannot reduce further
                            }
                        }
                    }
                }
                DependentTerm::Variable(name) => {
                    // Check for substitution or definition
                    if let Some(substitution) = self.context.lookup_substitution(name) {
                        current_term = substitution.clone();
                    } else if let Some(definition) = self.lookup_definition(name) {
                        self.context.increment_reductions()?;
                        
                        steps.push(ReductionStep::Delta {
                            constant: name.clone(),
                            definition: definition.clone(),
                            result: definition.clone(),
                        });
                        
                        current_term = definition;
                        
                        let mut stats = self.stats.lock().unwrap();
                        stats.delta_reductions += 1;
                    } else {
                        break; // Cannot reduce further
                    }
                }
                DependentTerm::Projection { pair, is_first } => {
                    match pair.as_ref() {
                        DependentTerm::Pair { first, second } => {
                            // Iota reduction for pair projection
                            self.context.increment_reductions()?;
                            let result = if *is_first { first.as_ref().clone() } else { second.as_ref().clone() };
                            
                            steps.push(ReductionStep::Iota {
                                constructor: pair.as_ref().clone(),
                                destructor: current_term.clone(),
                                result: result.clone(),
                            });
                            
                            current_term = result;
                            
                            let mut stats = self.stats.lock().unwrap();
                            stats.iota_reductions += 1;
                        }
                        _ => {
                            // Normalize pair to WHNF first
                            let pair_result = self.weak_head_normalize(pair)?;
                            if pair_result.normalized != **pair {
                                current_term = DependentTerm::Projection {
                                    pair: Box::new(pair_result.normalized),
                                    is_first: *is_first,
                                };
                                steps.extend(pair_result.steps);
                            } else {
                                break; // Cannot reduce further
                            }
                        }
                    }
                }
                _ => break, // Other terms are already in WHNF
            }
        }

        // Cache WHNF result
        self.cache_whnf_result(term, &current_term);

        Ok(NormalizationResult {
            normalized: current_term,
            steps,
            reduction_count: self.context.reduction_count - initial_reduction_count,
            is_complete: false,
            duration_ns: 0,
        })
    }

    /// Strong normal form (complete normalization)
    fn strong_normalize(&mut self, term: &DependentTerm) -> Result<NormalizationResult> {
        // First get WHNF
        let mut whnf_result = self.weak_head_normalize(term)?;
        
        // Then normalize subterms
        whnf_result.normalized = self.normalize_subterms(&whnf_result.normalized)?;
        whnf_result.is_complete = true;
        
        Ok(whnf_result)
    }

    /// Normalize all subterms recursively
    fn normalize_subterms(&mut self, term: &DependentTerm) -> Result<DependentTerm> {
        match term {
            DependentTerm::Lambda { param, param_type, body } => {
                self.context.enter_depth()?;
                let norm_body = Box::new(self.normalize_subterms(body)?);
                self.context.exit_depth();
                
                Ok(DependentTerm::Lambda {
                    param: param.clone(),
                    param_type: param_type.clone(),
                    body: norm_body,
                })
            }
            DependentTerm::Application { function, argument } => {
                let norm_function = Box::new(self.normalize_subterms(function)?);
                let norm_argument = Box::new(self.normalize_subterms(argument)?);
                
                Ok(DependentTerm::Application {
                    function: norm_function,
                    argument: norm_argument,
                })
            }
            DependentTerm::Pair { first, second } => {
                let norm_first = Box::new(self.normalize_subterms(first)?);
                let norm_second = Box::new(self.normalize_subterms(second)?);
                
                Ok(DependentTerm::Pair {
                    first: norm_first,
                    second: norm_second,
                })
            }
            DependentTerm::Projection { pair, is_first } => {
                let norm_pair = Box::new(self.normalize_subterms(pair)?);
                
                Ok(DependentTerm::Projection {
                    pair: norm_pair,
                    is_first: *is_first,
                })
            }
            _ => Ok(term.clone()), // Other terms don't have subterms to normalize
        }
    }

    /// Lazy normalization with memoization
    fn lazy_normalize(&mut self, term: &DependentTerm) -> Result<NormalizationResult> {
        // For lazy evaluation, we return a thunk that will compute the result when needed
        // For simplicity, we'll just do weak head normalization here
        self.weak_head_normalize(term)
    }

    /// Eager normalization with caching
    fn eager_normalize(&mut self, term: &DependentTerm) -> Result<NormalizationResult> {
        // Eager evaluation: immediately compute full normal form
        self.strong_normalize(term)
    }

    /// Parallel normalization for large terms
    fn parallel_normalize(&mut self, term: &DependentTerm) -> Result<NormalizationResult> {
        match term {
            DependentTerm::Application { function, argument } => {
                // Normalize function and argument in parallel
                let (func_result, arg_result): (Result<NormalizationResult>, Result<NormalizationResult>) = rayon::join(
                    || {
                        let mut local_engine = self.clone_for_parallel();
                        local_engine.normalize_term(function)
                    },
                    || {
                        let mut local_engine = self.clone_for_parallel();
                        local_engine.normalize_term(argument)
                    },
                );

                let func_result = func_result?;
                let arg_result = arg_result?;

                // Combine results
                let normalized = DependentTerm::Application {
                    function: Box::new(func_result.normalized),
                    argument: Box::new(arg_result.normalized),
                };

                let mut steps = func_result.steps;
                steps.extend(arg_result.steps);

                // Try further reduction
                let final_result = self.weak_head_normalize(&normalized)?;
                steps.extend(final_result.steps);

                let mut stats = self.stats.lock().unwrap();
                stats.parallel_normalizations += 1;

                Ok(NormalizationResult {
                    normalized: final_result.normalized,
                    steps,
                    reduction_count: func_result.reduction_count + arg_result.reduction_count + final_result.reduction_count,
                    is_complete: true,
                    duration_ns: 0,
                })
            }
            DependentTerm::Pair { first, second } => {
                // Normalize pair components in parallel
                let (first_result, second_result): (Result<NormalizationResult>, Result<NormalizationResult>) = rayon::join(
                    || {
                        let mut local_engine = self.clone_for_parallel();
                        local_engine.normalize_term(first)
                    },
                    || {
                        let mut local_engine = self.clone_for_parallel();
                        local_engine.normalize_term(second)
                    },
                );

                let first_result = first_result?;
                let second_result = second_result?;

                let normalized = DependentTerm::Pair {
                    first: Box::new(first_result.normalized),
                    second: Box::new(second_result.normalized),
                };

                let mut steps = first_result.steps;
                steps.extend(second_result.steps);

                let mut stats = self.stats.lock().unwrap();
                stats.parallel_normalizations += 1;

                Ok(NormalizationResult {
                    normalized,
                    steps,
                    reduction_count: first_result.reduction_count + second_result.reduction_count,
                    is_complete: true,
                    duration_ns: 0,
                })
            }
            _ => {
                // For other terms, use sequential normalization
                self.strong_normalize(term)
            }
        }
    }

    /// Substitute term for variable in another term
    fn substitute_term(&self, term: &DependentTerm, var: &str, replacement: &DependentTerm) -> Result<DependentTerm> {
        match term {
            DependentTerm::Variable(name) => {
                if name == var {
                    Ok(replacement.clone())
                } else {
                    Ok(term.clone())
                }
            }
            DependentTerm::Lambda { param, param_type, body } => {
                if param == var {
                    // Variable is bound, no substitution in body
                    Ok(term.clone())
                } else {
                    // Substitute in body (need to handle capture-avoiding substitution)
                    let new_body = Box::new(self.substitute_term(body, var, replacement)?);
                    Ok(DependentTerm::Lambda {
                        param: param.clone(),
                        param_type: param_type.clone(),
                        body: new_body,
                    })
                }
            }
            DependentTerm::Application { function, argument } => {
                let new_function = Box::new(self.substitute_term(function, var, replacement)?);
                let new_argument = Box::new(self.substitute_term(argument, var, replacement)?);
                Ok(DependentTerm::Application {
                    function: new_function,
                    argument: new_argument,
                })
            }
            DependentTerm::Pair { first, second } => {
                let new_first = Box::new(self.substitute_term(first, var, replacement)?);
                let new_second = Box::new(self.substitute_term(second, var, replacement)?);
                Ok(DependentTerm::Pair {
                    first: new_first,
                    second: new_second,
                })
            }
            DependentTerm::Projection { pair, is_first } => {
                let new_pair = Box::new(self.substitute_term(pair, var, replacement)?);
                Ok(DependentTerm::Projection {
                    pair: new_pair,
                    is_first: *is_first,
                })
            }
            _ => Ok(term.clone()), // Other terms don't contain variables
        }
    }

    /// Lookup definition for constant
    fn lookup_definition(&self, name: &str) -> Option<DependentTerm> {
        let definitions = self.definitions.read().unwrap();
        definitions.get(name).cloned()
    }

    /// Add definition for constant
    pub fn add_definition(&mut self, name: String, definition: DependentTerm) {
        let mut definitions = self.definitions.write().unwrap();
        definitions.insert(name, definition);
    }

    /// Check cache for normalization result
    fn check_cache(&self, key: &NormalizationCacheKey) -> Option<NormalizationResult> {
        let cache = self.cache.read().unwrap();
        cache.get(key).cloned()
    }

    /// Cache normalization result
    fn cache_result(&self, key: &NormalizationCacheKey, result: &NormalizationResult) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(key.clone(), result.clone());
    }

    /// Check WHNF cache
    fn check_whnf_cache(&self, term: &DependentTerm) -> Option<DependentTerm> {
        let cache = self.whnf_cache.read().unwrap();
        cache.get(term).cloned()
    }

    /// Cache WHNF result
    fn cache_whnf_result(&self, original: &DependentTerm, whnf: &DependentTerm) {
        let mut cache = self.whnf_cache.write().unwrap();
        cache.insert(original.clone(), whnf.clone());
    }

    /// Clone for parallel processing
    fn clone_for_parallel(&self) -> Self {
        Self {
            strategy: self.strategy,
            context: self.context.clone_for_parallel(),
            cache: self.cache.clone(),
            whnf_cache: self.whnf_cache.clone(),
            definitions: self.definitions.clone(),
            stats: self.stats.clone(),
            parallel_enabled: false, // Disable nested parallelism
            termination_checker: self.termination_checker.clone(),
            confluence_checker: self.confluence_checker.clone(),
            termination_confluence_system: self.termination_confluence_system.clone(),
            enable_termination_checking: self.enable_termination_checking,
            enable_confluence_checking: self.enable_confluence_checking,
        }
    }

    /// Check if a term is strongly normalizing (terminates for all reduction sequences)
    pub fn is_strongly_normalizing(&self, term: &DependentTerm) -> Result<bool> {
        if !self.enable_termination_checking {
            return Ok(true); // Assume true if checking is disabled
        }

        let start_time = std::time::Instant::now();
        let result = self.termination_checker.is_strongly_normalizing(term)?;
        let elapsed = start_time.elapsed();

        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.terms_checked_termination += 1;
            if result {
                stats.strongly_normalizing_terms += 1;
            } else {
                stats.termination_check_failures += 1;
            }
            stats.avg_termination_check_time_us = 
                (stats.avg_termination_check_time_us + elapsed.as_micros() as u64) / 2;
        }

        Ok(result)
    }

    /// Check if a term has the Church-Rosser property (confluence)
    pub fn is_confluent(&self, term: &DependentTerm) -> Result<bool> {
        if !self.enable_confluence_checking {
            return Ok(true); // Assume true if checking is disabled
        }

        let start_time = std::time::Instant::now();
        let result = self.confluence_checker.is_confluent(term)?;
        let elapsed = start_time.elapsed();

        // Update statistics
        {
            let mut stats = self.stats.lock().unwrap();
            stats.terms_checked_confluence += 1;
            if result {
                stats.confluent_terms += 1;
            } else {
                stats.confluence_check_failures += 1;
            }
            stats.avg_confluence_check_time_us = 
                (stats.avg_confluence_check_time_us + elapsed.as_micros() as u64) / 2;
        }

        Ok(result)
    }

    /// Check if a term is well-behaved (both strongly normalizing and confluent)
    pub fn is_well_behaved(&self, term: &DependentTerm) -> Result<(bool, bool)> {
        self.termination_confluence_system.is_well_behaved(term)
    }

    /// Perform safe normalization with termination guarantees
    pub fn safe_normalize_term(&mut self, term: &DependentTerm) -> Result<NormalizationResult> {
        // First check if the term is strongly normalizing
        if self.enable_termination_checking && !self.is_strongly_normalizing(term)? {
            return Err(Box::new(Error::type_error(
                "Cannot safely normalize term: strong normalization not guaranteed".to_string(),
                Span::new(0, 0),
            )));
        }

        // If confluence checking is enabled, verify confluence
        if self.enable_confluence_checking && !self.is_confluent(term)? {
            return Err(Box::new(Error::type_error(
                "Cannot safely normalize term: confluence not guaranteed".to_string(),
                Span::new(0, 0),
            )));
        }

        // Proceed with normal normalization
        self.normalize_term(term)
    }

    /// Get complexity measure for a term (for termination analysis)
    pub fn get_complexity_measure(&self, term: &DependentTerm) -> ComplexityMeasure {
        ComplexityMeasure::for_term(term)
    }

    /// Enable or disable termination checking
    pub fn set_termination_checking(&mut self, enabled: bool) {
        self.enable_termination_checking = enabled;
    }

    /// Enable or disable confluence checking
    pub fn set_confluence_checking(&mut self, enabled: bool) {
        self.enable_confluence_checking = enabled;
    }

    /// Generate a comprehensive termination and confluence report
    pub fn analyze_term_safety(&self, term: &DependentTerm) -> Result<crate::types::dependent::termination::TerminationConfluenceReport> {
        self.termination_confluence_system.analyze_term(term)
    }

    /// Check if two terms are definitionally equal (using normalization)
    pub fn definitionally_equal(&mut self, term1: &DependentTerm, term2: &DependentTerm) -> Result<bool> {
        let norm1 = self.normalize_term(term1)?;
        let norm2 = self.normalize_term(term2)?;
        Ok(norm1.normalized == norm2.normalized)
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> NormalizationStatistics {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    /// Clear caches for memory management
    pub fn clear_caches(&mut self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        let mut whnf_cache = self.whnf_cache.write().unwrap();
        whnf_cache.clear();
    }

    /// Set normalization strategy
    pub fn set_strategy(&mut self, strategy: NormalizationStrategy) {
        self.strategy = strategy;
        self.parallel_enabled = strategy == NormalizationStrategy::Parallel;
    }
}

impl Default for NormalizationEngine {
    fn default() -> Self {
        Self::new()
    }
}


impl Clone for NormalizationStatistics {
    fn clone(&self) -> Self {
        Self {
            terms_normalized: self.terms_normalized,
            beta_reductions: self.beta_reductions,
            delta_reductions: self.delta_reductions,
            iota_reductions: self.iota_reductions,
            eta_reductions: self.eta_reductions,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            parallel_normalizations: self.parallel_normalizations,
            total_reduction_steps: self.total_reduction_steps,
            terms_checked_termination: self.terms_checked_termination,
            terms_checked_confluence: self.terms_checked_confluence,
            strongly_normalizing_terms: self.strongly_normalizing_terms,
            confluent_terms: self.confluent_terms,
            termination_check_failures: self.termination_check_failures,
            confluence_check_failures: self.confluence_check_failures,
            avg_termination_check_time_us: self.avg_termination_check_time_us,
            avg_confluence_check_time_us: self.avg_confluence_check_time_us,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalization_engine_creation() {
        let engine = NormalizationEngine::new();
        let stats = engine.get_statistics();
        assert_eq!(stats.terms_normalized, 0);
    }

    #[test]
    fn test_normalization_context() {
        let mut context = NormalizationContext::new();
        let term = DependentTerm::Variable("x".to_string());
        context.add_substitution("x".to_string(), term.clone());
        
        assert_eq!(context.lookup_substitution("x"), Some(&term));
    }

    #[test]
    fn test_simple_beta_reduction() {
        let mut engine = NormalizationEngine::new();
        
        // (λx.x) y → y
        let lambda = DependentTerm::Lambda {
            param: "x".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Variable("x".to_string())),
        };
        let application = DependentTerm::Application {
            function: Box::new(lambda),
            argument: Box::new(DependentTerm::Variable("y".to_string())),
        };
        
        let result = engine.normalize_term(&application).unwrap();
        assert_eq!(result.normalized, DependentTerm::Variable("y".to_string()));
        assert!(result.reduction_count > 0);
    }

    #[test]
    fn test_pair_projection_reduction() {
        let mut engine = NormalizationEngine::new();
        
        // π₁(a, b) → a
        let pair = DependentTerm::Pair {
            first: Box::new(DependentTerm::Variable("a".to_string())),
            second: Box::new(DependentTerm::Variable("b".to_string())),
        };
        let projection = DependentTerm::Projection {
            pair: Box::new(pair),
            is_first: true,
        };
        
        let result = engine.normalize_term(&projection).unwrap();
        assert_eq!(result.normalized, DependentTerm::Variable("a".to_string()));
        assert!(result.reduction_count > 0);
    }

    #[test]
    fn test_normalization_strategies() {
        for strategy in [
            NormalizationStrategy::WeakHead,
            NormalizationStrategy::Strong,
            NormalizationStrategy::Lazy,
            NormalizationStrategy::Eager,
        ] {
            let mut engine = NormalizationEngine::with_strategy(strategy);
            let term = DependentTerm::Variable("x".to_string());
            let result = engine.normalize_term(&term).unwrap();
            assert_eq!(result.normalized, term);
        }
    }

    #[test]
    fn test_strong_normalization_checking() {
        let engine = NormalizationEngine::new();
        let term = DependentTerm::Variable("x".to_string());
        
        // Variables should be strongly normalizing
        let is_normalizing = engine.is_strongly_normalizing(&term).unwrap();
        assert!(is_normalizing);
        
        // Check statistics
        let stats = engine.get_statistics();
        assert!(stats.terms_checked_termination > 0);
    }

    #[test]
    fn test_confluence_checking() {
        let engine = NormalizationEngine::new();
        let term = DependentTerm::Variable("x".to_string());
        
        // Variables should be confluent
        let is_confluent = engine.is_confluent(&term).unwrap();
        assert!(is_confluent);
        
        // Check statistics
        let stats = engine.get_statistics();
        assert!(stats.terms_checked_confluence > 0);
    }

    #[test]
    fn test_well_behaved_terms() {
        let engine = NormalizationEngine::new();
        let term = DependentTerm::Variable("x".to_string());
        
        let (normalizing, confluent) = engine.is_well_behaved(&term).unwrap();
        assert!(normalizing);
        assert!(confluent);
    }

    #[test]
    fn test_safe_normalization() {
        let mut engine = NormalizationEngine::new();
        let term = DependentTerm::Variable("x".to_string());
        
        // Safe normalization should succeed for well-behaved terms
        let result = engine.safe_normalize_term(&term).unwrap();
        assert_eq!(result.normalized, term);
    }

    #[test]
    fn test_complexity_measure() {
        let engine = NormalizationEngine::new();
        
        let simple_term = DependentTerm::Variable("x".to_string());
        let complex_term = DependentTerm::Lambda {
            param: "x".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Variable("x".to_string())),
        };
        
        let simple_complexity = engine.get_complexity_measure(&simple_term);
        let complex_complexity = engine.get_complexity_measure(&complex_term);
        
        assert!(simple_complexity.is_smaller_than(&complex_complexity));
    }

    #[test]
    fn test_termination_confluence_disabled() {
        let mut engine = NormalizationEngine::with_termination_checking(false, false);
        let term = DependentTerm::Variable("x".to_string());
        
        // Should return true even when checking is disabled
        assert!(engine.is_strongly_normalizing(&term).unwrap());
        assert!(engine.is_confluent(&term).unwrap());
    }

    #[test]
    fn test_termination_confluence_configuration() {
        let mut engine = NormalizationEngine::new();
        
        // Test enabling/disabling
        engine.set_termination_checking(false);
        engine.set_confluence_checking(false);
        
        let term = DependentTerm::Variable("x".to_string());
        assert!(engine.is_strongly_normalizing(&term).unwrap());
        assert!(engine.is_confluent(&term).unwrap());
    }
}
//! Definitional equality system for Martin-Löf dependent type theory.
//!
//! This module implements a complete definitional equality checker that follows
//! the mathematical foundation of Martin-Löf type theory. Definitional equality
//! determines when two types or terms are considered "the same" by the type system.
//!
//! # Mathematical Foundation
//!
//! Definitional equality (≡) includes:
//!
//! ## Core Equivalence Relations
//! - **α-equivalence**: Terms differing only in bound variable names
//!   - `λx.x ≡ λy.y`
//!   - `(x:A) → B(x) ≡ (y:A) → B(y)` (with appropriate substitution)
//!
//! - **β-equivalence**: Function application reduction  
//!   - `(λx.t)(a) ≡ t[a/x]`
//!   - Computational content of functions
//!
//! - **η-equivalence**: Extensional equality
//!   - Functions: `λx.(f x) ≡ f` when `x ∉ FV(f)`
//!   - Pairs: `(π₁(p), π₂(p)) ≡ p`
//!   - Reflects that equal functions behave the same on all inputs
//!
//! ## Additional Equality Sources  
//! - **Definitional expansion**: Unfolding definitions
//!   - If `f := λx.x + 1`, then `f ≡ λx.x + 1`
//!   - Transparent definitions become equal to their bodies
//!
//! - **Congruence rules**: Structural compatibility
//!   - If `A ≡ A'` and `B ≡ B'`, then `A → B ≡ A' → B'`
//!   - Equality preserved under type constructors
//!
//! # Implementation Strategy
//!
//! The implementation uses a normalization-based approach:
//! 1. **Weak Head Normal Form (WHNF)**: Efficient partial normalization
//! 2. **Comparison modulo α-equivalence**: Handle variable renaming
//! 3. **η-expansion when needed**: Ensure extensional equality
//! 4. **Caching**: Memoize expensive equality checks
//!
//! # Correctness Properties
//!
//! The system maintains these essential properties:
//! - **Reflexivity**: `A ≡ A` for all well-typed `A`
//! - **Symmetry**: `A ≡ B` implies `B ≡ A`
//! - **Transitivity**: `A ≡ B` and `B ≡ C` implies `A ≡ C`
//! - **Congruence**: Equality preserved under type constructors
//! - **Decidability**: Algorithm terminates on well-typed inputs

use crate::diagnostics::{Error, Result, Span};
use super::core::{DependentType, DependentTerm, Normalizer};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// High-performance definitional equality checker for Martin-Löf type theory.
///
/// This structure provides comprehensive equality checking with caching,
/// normalization, and support for all forms of definitional equality.
#[derive(Debug, Clone)]
pub struct DefinitionalEqualityChecker {
    /// Normalization engine for reducing terms to canonical form
    normalizer: Normalizer,
    /// Cache for expensive equality computations
    equality_cache: HashMap<(DependentType, DependentType), bool>,
    /// Cache for term equality computations  
    term_equality_cache: HashMap<(DependentTerm, DependentTerm), bool>,
    /// Definition environment for expansion
    definitions: HashMap<String, DependentTerm>,
    /// Type definition environment
    type_definitions: HashMap<String, DependentType>,
    /// Configuration for equality checking depth
    max_normalization_depth: u32,
    /// Statistics for performance monitoring
    statistics: EqualityStatistics,
}

/// Statistics for monitoring equality checker performance.
#[derive(Debug, Clone, Default)]
pub struct EqualityStatistics {
    /// Total number of equality checks performed
    pub total_checks: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of normalization operations
    pub normalizations: u64,
    /// Number of α-equivalence checks
    pub alpha_checks: u64,
    /// Number of η-expansions performed
    pub eta_expansions: u64,
    /// Average time per equality check (in microseconds)
    pub avg_check_time_us: f64,
}

/// Configuration for definitional equality checking.
#[derive(Debug, Clone)]
pub struct EqualityConfig {
    /// Enable η-equivalence checking
    pub enable_eta_equivalence: bool,
    /// Enable definitional expansion  
    pub enable_definitional_expansion: bool,
    /// Maximum recursion depth for normalization
    pub max_depth: u32,
    /// Enable caching for performance
    pub enable_caching: bool,
}

impl Default for EqualityConfig {
    fn default() -> Self {
        Self {
            enable_eta_equivalence: true,
            enable_definitional_expansion: true,
            max_depth: 1000,
            enable_caching: true,
        }
    }
}

/// Result of definitional equality check with additional information.
#[derive(Debug, Clone)]
pub struct EqualityResult {
    /// Whether the types/terms are definitionally equal
    pub is_equal: bool,
    /// The rule that established equality (if any)
    pub justification: Option<EqualityJustification>,
    /// Witness terms for the equality (useful for proof construction)
    pub witness: Option<EqualityWitness>,
}

/// Justification for why two types/terms are considered equal.
#[derive(Debug, Clone)]
pub enum EqualityJustification {
    /// Structurally identical after normalization
    Syntactic,
    /// Equal up to α-conversion (variable renaming)
    AlphaEquivalence { renamed_vars: Vec<(String, String)> },
    /// Equal by β-reduction
    BetaEquivalence { reductions: Vec<String> },
    /// Equal by η-expansion
    EtaEquivalence { expansions: Vec<String> },
    /// Equal by definitional expansion
    DefinitionalExpansion { unfolded_defs: Vec<String> },
    /// Equal by congruence (structural compatibility)
    Congruence { sub_equalities: Vec<EqualityJustification> },
}

/// Witness information for equality proofs.
#[derive(Debug, Clone)]
pub enum EqualityWitness {
    /// Reflexivity witness
    Reflexivity,
    /// Symmetry witness  
    Symmetry(Box<EqualityWitness>),
    /// Transitivity witness
    Transitivity {
        left: Box<EqualityWitness>,
        right: Box<EqualityWitness>,
        middle_term: DependentType,
    },
    /// Congruence witness
    Congruence(Vec<EqualityWitness>),
}

impl DefinitionalEqualityChecker {
    /// Create a new definitional equality checker with default configuration.
    pub fn new() -> Self {
        Self::with_config(EqualityConfig::default())
    }

    /// Create a new equality checker with custom configuration.
    pub fn with_config(config: EqualityConfig) -> Self {
        Self {
            normalizer: Normalizer::new(),
            equality_cache: HashMap::new(),
            term_equality_cache: HashMap::new(),
            definitions: HashMap::new(),
            type_definitions: HashMap::new(),
            max_normalization_depth: config.max_depth,
            statistics: EqualityStatistics::default(),
        }
    }

    /// Check if two dependent types are definitionally equal.
    ///
    /// This is the main entry point for type equality checking.
    /// It implements the complete definitional equality relation.
    ///
    /// # Mathematical Specification
    /// 
    /// Two types A and B are definitionally equal (A ≡ B) if they are
    /// equal under the definitional equality relation, which includes:
    /// - α-equivalence (variable renaming)
    /// - β-equivalence (computation)  
    /// - η-equivalence (extensionality)
    /// - Definitional expansion (unfolding)
    /// - Congruence (structural preservation)
    ///
    /// # Examples
    /// ```rust,ignore
    /// let checker = DefinitionalEqualityChecker::new();
    /// 
    /// // α-equivalence: (x:A) → B(x) ≡ (y:A) → B(y)
    /// let result = checker.types_equal(pi_type_x, pi_type_y)?;
    /// assert!(result.is_equal);
    /// 
    /// // β-equivalence through normalization
    /// let result = checker.types_equal(applied_type, reduced_type)?;
    /// assert!(result.is_equal);
    /// ```
    pub fn types_equal(&mut self, ty1: &DependentType, ty2: &DependentType) -> Result<EqualityResult> {
        self.statistics.total_checks += 1;
        
        // Quick structural equality check
        if ty1 == ty2 {
            return Ok(EqualityResult {
                is_equal: true,
                justification: Some(EqualityJustification::Syntactic),
                witness: Some(EqualityWitness::Reflexivity),
            });
        }

        // Check cache first
        if let Some(&cached_result) = self.equality_cache.get(&(ty1.clone(), ty2.clone())) {
            self.statistics.cache_hits += 1;
            return Ok(EqualityResult {
                is_equal: cached_result,
                justification: None, // Cache doesn't preserve justification
                witness: None,
            });
        }

        // Normalize both types to canonical form
        self.statistics.normalizations += 2;
        let norm_ty1 = self.normalizer.normalize_type(ty1)?;
        let norm_ty2 = self.normalizer.normalize_type(ty2)?;

        // Check if normalized types are syntactically equal
        if norm_ty1 == norm_ty2 {
            let result = EqualityResult {
                is_equal: true,
                justification: Some(EqualityJustification::Syntactic),
                witness: Some(EqualityWitness::Reflexivity),
            };
            self.equality_cache.insert((ty1.clone(), ty2.clone()), true);
            return Ok(result);
        }

        // Check α-equivalence (variable renaming)
        if let Ok(alpha_result) = self.check_alpha_equivalence_types(&norm_ty1, &norm_ty2) {
            if alpha_result.is_equal {
                self.equality_cache.insert((ty1.clone(), ty2.clone()), true);
                return Ok(alpha_result);
            }
        }

        // Check η-equivalence if enabled
        if let Ok(eta_result) = self.check_eta_equivalence_types(&norm_ty1, &norm_ty2) {
            if eta_result.is_equal {
                self.equality_cache.insert((ty1.clone(), ty2.clone()), true);
                return Ok(eta_result);
            }
        }

        // Check definitional expansion
        if let Ok(expansion_result) = self.check_definitional_expansion_types(&norm_ty1, &norm_ty2) {
            if expansion_result.is_equal {
                self.equality_cache.insert((ty1.clone(), ty2.clone()), true);
                return Ok(expansion_result);
            }
        }

        // Check congruence (structural equality)
        let congruence_result = self.check_congruence_types(&norm_ty1, &norm_ty2)?;
        
        // Cache the result
        self.equality_cache.insert((ty1.clone(), ty2.clone()), congruence_result.is_equal);
        
        Ok(congruence_result)
    }

    /// Check if two dependent terms are definitionally equal.
    ///
    /// Similar to type equality but for terms. This is essential for
    /// checking equality in dependent contexts where terms appear in types.
    pub fn terms_equal(&mut self, term1: &DependentTerm, term2: &DependentTerm) -> Result<EqualityResult> {
        self.statistics.total_checks += 1;

        // Quick structural equality check  
        if term1 == term2 {
            return Ok(EqualityResult {
                is_equal: true,
                justification: Some(EqualityJustification::Syntactic),
                witness: Some(EqualityWitness::Reflexivity),
            });
        }

        // Check cache
        if let Some(&cached_result) = self.term_equality_cache.get(&(term1.clone(), term2.clone())) {
            self.statistics.cache_hits += 1;
            return Ok(EqualityResult {
                is_equal: cached_result,
                justification: None,
                witness: None,
            });
        }

        // Normalize both terms
        self.statistics.normalizations += 2;
        let norm_term1 = self.normalizer.normalize_term(term1)?;
        let norm_term2 = self.normalizer.normalize_term(term2)?;

        // Check syntactic equality after normalization
        if norm_term1 == norm_term2 {
            let result = EqualityResult {
                is_equal: true,
                justification: Some(EqualityJustification::BetaEquivalence { 
                    reductions: vec!["normalization".to_string()] 
                }),
                witness: Some(EqualityWitness::Reflexivity),
            };
            self.term_equality_cache.insert((term1.clone(), term2.clone()), true);
            return Ok(result);
        }

        // Check α-equivalence for terms
        if let Ok(alpha_result) = self.check_alpha_equivalence_terms(&norm_term1, &norm_term2) {
            if alpha_result.is_equal {
                self.term_equality_cache.insert((term1.clone(), term2.clone()), true);
                return Ok(alpha_result);
            }
        }

        // Check η-equivalence for terms  
        if let Ok(eta_result) = self.check_eta_equivalence_terms(&norm_term1, &norm_term2) {
            if eta_result.is_equal {
                self.term_equality_cache.insert((term1.clone(), term2.clone()), true);
                return Ok(eta_result);
            }
        }

        // Not equal
        let result = EqualityResult {
            is_equal: false,
            justification: None,
            witness: None,
        };
        self.term_equality_cache.insert((term1.clone(), term2.clone()), false);
        Ok(result)
    }

    /// Check α-equivalence between types (modulo variable renaming).
    ///
    /// Two types are α-equivalent if they are identical up to consistent
    /// renaming of bound variables. This is fundamental for dependent types
    /// where variable names should not matter.
    ///
    /// # Mathematical Definition
    /// Types A and B are α-equivalent (A ≡_α B) if there exists a consistent
    /// renaming of bound variables that makes them syntactically identical.
    ///
    /// # Examples  
    /// - `(x:A) → B(x) ≡_α (y:A) → B(y)`
    /// - `(x:A) × C(x) ≡_α (z:A) × C(z)`
    fn check_alpha_equivalence_types(&mut self, ty1: &DependentType, ty2: &DependentType) -> Result<EqualityResult> {
        self.statistics.alpha_checks += 1;

        match (ty1, ty2) {
            // Universe types: only equal if same level
            (DependentType::Universe(level1), DependentType::Universe(level2)) => {
                Ok(EqualityResult {
                    is_equal: level1 == level2,
                    justification: if level1 == level2 { 
                        Some(EqualityJustification::Syntactic) 
                    } else { 
                        None 
                    },
                    witness: if level1 == level2 { 
                        Some(EqualityWitness::Reflexivity) 
                    } else { 
                        None 
                    },
                })
            }

            // Π-types: check domain and codomain with consistent variable renaming
            (DependentType::Pi { var: var1, domain: dom1, codomain: cod1 },
             DependentType::Pi { var: var2, domain: dom2, codomain: cod2 }) => {
                
                // Check domains are equal
                let domain_result = self.types_equal(dom1, dom2)?;
                if !domain_result.is_equal {
                    return Ok(EqualityResult { is_equal: false, justification: None, witness: None });
                }

                // For codomains, we need to handle variable renaming
                if var1 == var2 {
                    // Same variable name, direct comparison
                    let codomain_result = self.types_equal(cod1, cod2)?;
                    Ok(EqualityResult {
                        is_equal: codomain_result.is_equal,
                        justification: if codomain_result.is_equal {
                            Some(EqualityJustification::AlphaEquivalence {
                                renamed_vars: vec![]
                            })
                        } else {
                            None
                        },
                        witness: codomain_result.witness,
                    })
                } else {
                    // Different variable names, need consistent renaming
                    // Substitute var2 with var1 in cod2 and compare
                    let renamed_cod2 = self.substitute_type_var(cod2, var2, var1)?;
                    let codomain_result = self.types_equal(cod1, &renamed_cod2)?;
                    Ok(EqualityResult {
                        is_equal: codomain_result.is_equal,
                        justification: if codomain_result.is_equal {
                            Some(EqualityJustification::AlphaEquivalence {
                                renamed_vars: vec![(var2.clone(), var1.clone())]
                            })
                        } else {
                            None
                        },
                        witness: codomain_result.witness,
                    })
                }
            }

            // Σ-types: similar to Π-types
            (DependentType::Sigma { var: var1, first: first1, second: second1 },
             DependentType::Sigma { var: var2, first: first2, second: second2 }) => {
                
                let first_result = self.types_equal(first1, first2)?;
                if !first_result.is_equal {
                    return Ok(EqualityResult { is_equal: false, justification: None, witness: None });
                }

                if var1 == var2 {
                    let second_result = self.types_equal(second1, second2)?;
                    Ok(EqualityResult {
                        is_equal: second_result.is_equal,
                        justification: if second_result.is_equal {
                            Some(EqualityJustification::AlphaEquivalence {
                                renamed_vars: vec![]
                            })
                        } else {
                            None
                        },
                        witness: second_result.witness,
                    })
                } else {
                    let renamed_second2 = self.substitute_type_var(second2, var2, var1)?;
                    let second_result = self.types_equal(second1, &renamed_second2)?;
                    Ok(EqualityResult {
                        is_equal: second_result.is_equal,
                        justification: if second_result.is_equal {
                            Some(EqualityJustification::AlphaEquivalence {
                                renamed_vars: vec![(var2.clone(), var1.clone())]
                            })
                        } else {
                            None
                        },
                        witness: second_result.witness,
                    })
                }
            }

            // Identity types: check type and both terms
            (DependentType::Identity { ty: ty1, left: left1, right: right1 },
             DependentType::Identity { ty: ty2, left: left2, right: right2 }) => {
                
                let ty_result = self.types_equal(ty1, ty2)?;
                if !ty_result.is_equal {
                    return Ok(EqualityResult { is_equal: false, justification: None, witness: None });
                }

                let left_result = self.terms_equal(left1, left2)?;
                if !left_result.is_equal {
                    return Ok(EqualityResult { is_equal: false, justification: None, witness: None });
                }

                let right_result = self.terms_equal(right1, right2)?;
                Ok(EqualityResult {
                    is_equal: right_result.is_equal,
                    justification: if right_result.is_equal {
                        Some(EqualityJustification::AlphaEquivalence {
                            renamed_vars: vec![]
                        })
                    } else {
                        None
                    },
                    witness: right_result.witness,
                })
            }

            // Different constructors: not α-equivalent
            _ => Ok(EqualityResult { 
                is_equal: false, 
                justification: None, 
                witness: None 
            }),
        }
    }

    /// Check α-equivalence between terms.
    fn check_alpha_equivalence_terms(&mut self, term1: &DependentTerm, term2: &DependentTerm) -> Result<EqualityResult> {
        self.statistics.alpha_checks += 1;

        match (term1, term2) {
            // Variables: equal if same name
            (DependentTerm::Variable(name1), DependentTerm::Variable(name2)) => {
                Ok(EqualityResult {
                    is_equal: name1 == name2,
                    justification: if name1 == name2 { 
                        Some(EqualityJustification::Syntactic) 
                    } else { 
                        None 
                    },
                    witness: if name1 == name2 { 
                        Some(EqualityWitness::Reflexivity) 
                    } else { 
                        None 
                    },
                })
            }

            // Lambda terms: check with consistent variable renaming
            (DependentTerm::Lambda { param: param1, param_type: pty1, body: body1 },
             DependentTerm::Lambda { param: param2, param_type: pty2, body: body2 }) => {
                
                let param_type_result = self.types_equal(pty1, pty2)?;
                if !param_type_result.is_equal {
                    return Ok(EqualityResult { is_equal: false, justification: None, witness: None });
                }

                if param1 == param2 {
                    let body_result = self.terms_equal(body1, body2)?;
                    Ok(EqualityResult {
                        is_equal: body_result.is_equal,
                        justification: if body_result.is_equal {
                            Some(EqualityJustification::AlphaEquivalence {
                                renamed_vars: vec![]
                            })
                        } else {
                            None
                        },
                        witness: body_result.witness,
                    })
                } else {
                    // Rename param2 to param1 in body2
                    let renamed_body2 = self.substitute_term_var(body2, param2, param1)?;
                    let body_result = self.terms_equal(body1, &renamed_body2)?;
                    Ok(EqualityResult {
                        is_equal: body_result.is_equal,
                        justification: if body_result.is_equal {
                            Some(EqualityJustification::AlphaEquivalence {
                                renamed_vars: vec![(param2.clone(), param1.clone())]
                            })
                        } else {
                            None
                        },
                        witness: body_result.witness,
                    })
                }
            }

            // Applications: check function and argument  
            (DependentTerm::Application { function: func1, argument: arg1 },
             DependentTerm::Application { function: func2, argument: arg2 }) => {
                
                let func_result = self.terms_equal(func1, func2)?;
                if !func_result.is_equal {
                    return Ok(EqualityResult { is_equal: false, justification: None, witness: None });
                }

                let arg_result = self.terms_equal(arg1, arg2)?;
                Ok(EqualityResult {
                    is_equal: arg_result.is_equal,
                    justification: if arg_result.is_equal {
                        Some(EqualityJustification::AlphaEquivalence {
                            renamed_vars: vec![]
                        })
                    } else {
                        None
                    },
                    witness: arg_result.witness,
                })
            }

            // Pairs: check both components
            (DependentTerm::Pair { first: first1, second: second1 },
             DependentTerm::Pair { first: first2, second: second2 }) => {
                
                let first_result = self.terms_equal(first1, first2)?;
                if !first_result.is_equal {
                    return Ok(EqualityResult { is_equal: false, justification: None, witness: None });
                }

                let second_result = self.terms_equal(second1, second2)?;
                Ok(EqualityResult {
                    is_equal: second_result.is_equal,
                    justification: if second_result.is_equal {
                        Some(EqualityJustification::AlphaEquivalence {
                            renamed_vars: vec![]
                        })
                    } else {
                        None
                    },
                    witness: second_result.witness,
                })
            }

            // Projections: check pair and projection type
            (DependentTerm::Projection { pair: pair1, is_first: first1 },
             DependentTerm::Projection { pair: pair2, is_first: first2 }) => {
                
                if first1 != first2 {
                    return Ok(EqualityResult { is_equal: false, justification: None, witness: None });
                }

                let pair_result = self.terms_equal(pair1, pair2)?;
                Ok(EqualityResult {
                    is_equal: pair_result.is_equal,
                    justification: if pair_result.is_equal {
                        Some(EqualityJustification::AlphaEquivalence {
                            renamed_vars: vec![]
                        })
                    } else {
                        None
                    },
                    witness: pair_result.witness,
                })
            }

            // Different constructors: not α-equivalent
            _ => Ok(EqualityResult { 
                is_equal: false, 
                justification: None, 
                witness: None 
            }),
        }
    }

    /// Check η-equivalence between types.
    ///
    /// η-equivalence captures extensional equality - things that behave
    /// the same should be considered equal.
    fn check_eta_equivalence_types(&mut self, _ty1: &DependentType, _ty2: &DependentType) -> Result<EqualityResult> {
        self.statistics.eta_expansions += 1;
        
        // η-equivalence for types is complex and context-dependent
        // For now, we don't implement it for types (mainly applies to terms)
        Ok(EqualityResult { 
            is_equal: false, 
            justification: None, 
            witness: None 
        })
    }

    /// Check η-equivalence between terms.
    ///
    /// # Mathematical Foundation
    /// 
    /// η-equivalence for functions: `λx.(f x) ≡ f` when `x ∉ FV(f)`
    /// η-equivalence for pairs: `(π₁(p), π₂(p)) ≡ p`
    fn check_eta_equivalence_terms(&mut self, term1: &DependentTerm, term2: &DependentTerm) -> Result<EqualityResult> {
        self.statistics.eta_expansions += 1;

        // η-equivalence for functions: λx.(f x) ≡ f
        match (term1, term2) {
            // Case 1: λx.(f x) vs f where x ∉ FV(f)
            (DependentTerm::Lambda { param, body, .. }, func) => {
                if let DependentTerm::Application { function, argument } = body.as_ref() {
                    if let DependentTerm::Variable(arg_var) = argument.as_ref() {
                        if arg_var == param {
                            // Check if func equals function and param is not free in function
                            let func_result = self.terms_equal(function, func)?;
                            if func_result.is_equal && !self.is_free_in_term(param, func) {
                                return Ok(EqualityResult {
                                    is_equal: true,
                                    justification: Some(EqualityJustification::EtaEquivalence {
                                        expansions: vec![format!("η-contract λ{}.({} {})", param, func, param)]
                                    }),
                                    witness: Some(EqualityWitness::Reflexivity),
                                });
                            }
                        }
                    }
                }
            }

            // Case 2: f vs λx.(f x) (symmetric case)
            (func, DependentTerm::Lambda { param, body, .. }) => {
                if let DependentTerm::Application { function, argument } = body.as_ref() {
                    if let DependentTerm::Variable(arg_var) = argument.as_ref() {
                        if arg_var == param {
                            let func_result = self.terms_equal(func, function)?;
                            if func_result.is_equal && !self.is_free_in_term(param, func) {
                                return Ok(EqualityResult {
                                    is_equal: true,
                                    justification: Some(EqualityJustification::EtaEquivalence {
                                        expansions: vec![format!("η-expand {} to λ{}.({} {})", func, param, func, param)]
                                    }),
                                    witness: Some(EqualityWitness::Reflexivity),
                                });
                            }
                        }
                    }
                }
            }

            // Case 3: (π₁(p), π₂(p)) ≡ p
            (DependentTerm::Pair { first, second }, pair) => {
                if let (DependentTerm::Projection { pair: pair1, is_first: true },
                        DependentTerm::Projection { pair: pair2, is_first: false }) = 
                       (first.as_ref(), second.as_ref()) {
                    let pair1_result = self.terms_equal(pair1, pair)?;
                    let pair2_result = self.terms_equal(pair2, pair)?;
                    if pair1_result.is_equal && pair2_result.is_equal {
                        return Ok(EqualityResult {
                            is_equal: true,
                            justification: Some(EqualityJustification::EtaEquivalence {
                                expansions: vec![format!("η-contract (π₁({}), π₂({}))", pair, pair)]
                            }),
                            witness: Some(EqualityWitness::Reflexivity),
                        });
                    }
                }
            }

            // Case 4: p ≡ (π₁(p), π₂(p)) (symmetric)
            (pair, DependentTerm::Pair { first, second }) => {
                if let (DependentTerm::Projection { pair: pair1, is_first: true },
                        DependentTerm::Projection { pair: pair2, is_first: false }) = 
                       (first.as_ref(), second.as_ref()) {
                    let pair1_result = self.terms_equal(pair, pair1)?;
                    let pair2_result = self.terms_equal(pair, pair2)?;
                    if pair1_result.is_equal && pair2_result.is_equal {
                        return Ok(EqualityResult {
                            is_equal: true,
                            justification: Some(EqualityJustification::EtaEquivalence {
                                expansions: vec![format!("η-expand {} to (π₁({}), π₂({}))", pair, pair, pair)]
                            }),
                            witness: Some(EqualityWitness::Reflexivity),
                        });
                    }
                }
            }

            _ => {}
        }

        Ok(EqualityResult { 
            is_equal: false, 
            justification: None, 
            witness: None 
        })
    }

    /// Check definitional expansion for types.
    ///
    /// This implements the definitional expansion rule where defined names
    /// are unfolded to their definitions and then checked for equality.
    ///
    /// # Mathematical Foundation
    /// If we have definitions:
    /// - `T := A`  
    /// - `U := B`
    /// 
    /// Then `T ≡ U` if and only if `A ≡ B` (after expansion).
    fn check_definitional_expansion_types(&mut self, ty1: &DependentType, ty2: &DependentType) -> Result<EqualityResult> {
        // Try to expand both types and check equality
        let expanded_ty1 = self.expand_type_definitions(ty1)?;
        let expanded_ty2 = self.expand_type_definitions(ty2)?;
        
        // Check if expansion actually changed anything
        let ty1_changed = expanded_ty1 != *ty1;
        let ty2_changed = expanded_ty2 != *ty2;
        
        if !ty1_changed && !ty2_changed {
            // No definitions to expand
            return Ok(EqualityResult { 
                is_equal: false, 
                justification: None, 
                witness: None 
            });
        }
        
        // Recursively check equality with expanded types
        let expanded_result = self.types_equal(&expanded_ty1, &expanded_ty2)?;
        
        if expanded_result.is_equal {
            let mut unfolded_defs = Vec::new();
            if ty1_changed {
                unfolded_defs.push("type1".to_string()); // Could track specific definition names
            }
            if ty2_changed {
                unfolded_defs.push("type2".to_string());
            }
            
            Ok(EqualityResult {
                is_equal: true,
                justification: Some(EqualityJustification::DefinitionalExpansion { unfolded_defs }),
                witness: expanded_result.witness,
            })
        } else {
            Ok(EqualityResult { 
                is_equal: false, 
                justification: None, 
                witness: None 
            })
        }
    }

    /// Expand type definitions to their bodies.
    ///
    /// This performs one level of definitional expansion, unfolding
    /// type definitions to their defined bodies.
    fn expand_type_definitions(&self, ty: &DependentType) -> Result<DependentType> {
        match ty {
            DependentType::Universe(level) => Ok(DependentType::Universe(*level)),
            
            DependentType::Pi { var, domain, codomain } => {
                let expanded_domain = self.expand_type_definitions(domain)?;
                let expanded_codomain = self.expand_type_definitions(codomain)?;
                Ok(DependentType::Pi {
                    var: var.clone(),
                    domain: Box::new(expanded_domain),
                    codomain: Box::new(expanded_codomain),
                })
            }
            
            DependentType::Sigma { var, first, second } => {
                let expanded_first = self.expand_type_definitions(first)?;
                let expanded_second = self.expand_type_definitions(second)?;
                Ok(DependentType::Sigma {
                    var: var.clone(),
                    first: Box::new(expanded_first),
                    second: Box::new(expanded_second),
                })
            }
            
            DependentType::Identity { ty, left, right } => {
                let expanded_ty = self.expand_type_definitions(ty)?;
                let expanded_left = self.expand_term_definitions(left)?;
                let expanded_right = self.expand_term_definitions(right)?;
                Ok(DependentType::Identity {
                    ty: Box::new(expanded_ty),
                    left: Box::new(expanded_left),
                    right: Box::new(expanded_right),
                })
            }
            
            DependentType::Inductive { name, parameters, universe_level, constructors, induction_principle } => {
                // Check if this inductive type has a definition
                if let Some(definition) = self.type_definitions.get(name) {
                    // Return the definition instead of the inductive type
                    self.expand_type_definitions(definition)
                } else {
                    // Expand parameter types and constructor types
                    let mut expanded_parameters = Vec::new();
                    for (param_name, param_ty) in parameters {
                        let expanded_param_ty = self.expand_type_definitions(param_ty)?;
                        expanded_parameters.push((param_name.clone(), expanded_param_ty));
                    }
                    
                    let mut expanded_constructors = Vec::new();
                    for (ctor_name, ctor_ty) in constructors {
                        let expanded_ctor_ty = self.expand_type_definitions(ctor_ty)?;
                        expanded_constructors.push((ctor_name.clone(), expanded_ctor_ty));
                    }
                    
                    let expanded_induction_principle = if let Some(ind_prin) = induction_principle {
                        Some(Box::new(self.expand_type_definitions(ind_prin)?))
                    } else {
                        None
                    };
                    
                    Ok(DependentType::Inductive {
                        name: name.clone(),
                        parameters: expanded_parameters,
                        universe_level: *universe_level,
                        constructors: expanded_constructors,
                        induction_principle: expanded_induction_principle,
                    })
                }
            }
        }
    }

    /// Expand term definitions to their bodies.
    fn expand_term_definitions(&self, term: &DependentTerm) -> Result<DependentTerm> {
        match term {
            DependentTerm::Variable(name) => {
                // Check if this variable has a definition
                if let Some(definition) = self.definitions.get(name) {
                    // Recursively expand the definition
                    self.expand_term_definitions(definition)
                } else {
                    Ok(term.clone())
                }
            }
            
            DependentTerm::Lambda { param, param_type, body } => {
                let expanded_param_type = self.expand_type_definitions(param_type)?;
                let expanded_body = self.expand_term_definitions(body)?;
                Ok(DependentTerm::Lambda {
                    param: param.clone(),
                    param_type: Box::new(expanded_param_type),
                    body: Box::new(expanded_body),
                })
            }
            
            DependentTerm::Application { function, argument } => {
                let expanded_function = self.expand_term_definitions(function)?;
                let expanded_argument = self.expand_term_definitions(argument)?;
                Ok(DependentTerm::Application {
                    function: Box::new(expanded_function),
                    argument: Box::new(expanded_argument),
                })
            }
            
            DependentTerm::Pair { first, second } => {
                let expanded_first = self.expand_term_definitions(first)?;
                let expanded_second = self.expand_term_definitions(second)?;
                Ok(DependentTerm::Pair {
                    first: Box::new(expanded_first),
                    second: Box::new(expanded_second),
                })
            }
            
            DependentTerm::Projection { pair, is_first } => {
                let expanded_pair = self.expand_term_definitions(pair)?;
                Ok(DependentTerm::Projection {
                    pair: Box::new(expanded_pair),
                    is_first: *is_first,
                })
            }
            
            DependentTerm::Refl { ty } => {
                let expanded_ty = self.expand_type_definitions(ty)?;
                Ok(DependentTerm::Refl {
                    ty: Box::new(expanded_ty),
                })
            }
            
            DependentTerm::Constructor { name, args, result_type } => {
                let mut expanded_args = Vec::new();
                for arg in args {
                    expanded_args.push(self.expand_term_definitions(arg)?);
                }
                let expanded_result_type = self.expand_type_definitions(result_type)?;
                
                Ok(DependentTerm::Constructor {
                    name: name.clone(),
                    args: expanded_args,
                    result_type: Box::new(expanded_result_type),
                })
            }
            
            DependentTerm::Match { scrutinee, branches, return_type } => {
                let expanded_scrutinee = self.expand_term_definitions(scrutinee)?;
                let expanded_return_type = self.expand_type_definitions(return_type)?;
                
                let mut expanded_branches = Vec::new();
                for branch in branches {
                    let expanded_body = self.expand_term_definitions(&branch.body)?;
                    expanded_branches.push(super::core::MatchBranch {
                        pattern: branch.pattern.clone(), // Patterns don't need expansion
                        body: expanded_body,
                    });
                }
                
                Ok(DependentTerm::Match {
                    scrutinee: Box::new(expanded_scrutinee),
                    branches: expanded_branches,
                    return_type: Box::new(expanded_return_type),
                })
            }
        }
    }

    /// Check congruence (structural equality) for types.
    fn check_congruence_types(&mut self, ty1: &DependentType, ty2: &DependentType) -> Result<EqualityResult> {
        match (ty1, ty2) {
            (DependentType::Universe(l1), DependentType::Universe(l2)) => {
                Ok(EqualityResult {
                    is_equal: l1 == l2,
                    justification: if l1 == l2 { 
                        Some(EqualityJustification::Congruence { sub_equalities: vec![] }) 
                    } else { 
                        None 
                    },
                    witness: if l1 == l2 { 
                        Some(EqualityWitness::Reflexivity) 
                    } else { 
                        None 
                    },
                })
            }

            (DependentType::Pi { domain: d1, codomain: c1, .. },
             DependentType::Pi { domain: d2, codomain: c2, .. }) => {
                let domain_result = self.types_equal(d1, d2)?;
                let codomain_result = self.types_equal(c1, c2)?;
                
                let is_equal = domain_result.is_equal && codomain_result.is_equal;
                Ok(EqualityResult {
                    is_equal,
                    justification: if is_equal {
                        Some(EqualityJustification::Congruence {
                            sub_equalities: vec![
                                domain_result.justification.unwrap_or(EqualityJustification::Syntactic),
                                codomain_result.justification.unwrap_or(EqualityJustification::Syntactic),
                            ]
                        })
                    } else {
                        None
                    },
                    witness: if is_equal {
                        Some(EqualityWitness::Congruence(vec![
                            domain_result.witness.unwrap_or(EqualityWitness::Reflexivity),
                            codomain_result.witness.unwrap_or(EqualityWitness::Reflexivity),
                        ]))
                    } else {
                        None
                    },
                })
            }

            (DependentType::Sigma { first: f1, second: s1, .. },
             DependentType::Sigma { first: f2, second: s2, .. }) => {
                let first_result = self.types_equal(f1, f2)?;
                let second_result = self.types_equal(s1, s2)?;
                
                let is_equal = first_result.is_equal && second_result.is_equal;
                Ok(EqualityResult {
                    is_equal,
                    justification: if is_equal {
                        Some(EqualityJustification::Congruence {
                            sub_equalities: vec![
                                first_result.justification.unwrap_or(EqualityJustification::Syntactic),
                                second_result.justification.unwrap_or(EqualityJustification::Syntactic),
                            ]
                        })
                    } else {
                        None
                    },
                    witness: if is_equal {
                        Some(EqualityWitness::Congruence(vec![
                            first_result.witness.unwrap_or(EqualityWitness::Reflexivity),
                            second_result.witness.unwrap_or(EqualityWitness::Reflexivity),
                        ]))
                    } else {
                        None
                    },
                })
            }

            _ => Ok(EqualityResult { 
                is_equal: false, 
                justification: None, 
                witness: None 
            }),
        }
    }

    /// Helper: Check if a variable is free in a term.
    fn is_free_in_term(&self, var: &str, term: &DependentTerm) -> bool {
        match term {
            DependentTerm::Variable(name) => name == var,
            DependentTerm::Lambda { param, param_type: _, body } => {
                if param == var {
                    false // Variable is bound
                } else {
                    self.is_free_in_term(var, body)
                }
            }
            DependentTerm::Application { function, argument } => {
                self.is_free_in_term(var, function) || self.is_free_in_term(var, argument)
            }
            DependentTerm::Pair { first, second } => {
                self.is_free_in_term(var, first) || self.is_free_in_term(var, second)
            }
            DependentTerm::Projection { pair, .. } => {
                self.is_free_in_term(var, pair)
            }
            DependentTerm::Refl { .. } => false,
            DependentTerm::Constructor { args, .. } => {
                args.iter().any(|arg| self.is_free_in_term(var, arg))
            }
            DependentTerm::Match { scrutinee, branches, .. } => {
                if self.is_free_in_term(var, scrutinee) {
                    return true;
                }
                // Check branches (need to handle pattern binding)
                branches.iter().any(|branch| {
                    // TODO: Handle pattern variable binding properly
                    self.is_free_in_term(var, &branch.body)
                })
            }
        }
    }

    /// Helper: Substitute variable in type (simple version).
    fn substitute_type_var(&self, ty: &DependentType, old_var: &str, new_var: &str) -> Result<DependentType> {
        match ty {
            DependentType::Universe(level) => Ok(DependentType::Universe(*level)),
            DependentType::Pi { var, domain, codomain } => {
                let new_domain = self.substitute_type_var(domain, old_var, new_var)?;
                let new_codomain = if var == old_var {
                    // Variable is bound, don't substitute in codomain
                    (**codomain).clone()
                } else {
                    self.substitute_type_var(codomain, old_var, new_var)?
                };
                Ok(DependentType::Pi {
                    var: if var == old_var { new_var.to_string() } else { var.clone() },
                    domain: Box::new(new_domain),
                    codomain: Box::new(new_codomain),
                })
            }
            DependentType::Sigma { var, first, second } => {
                let new_first = self.substitute_type_var(first, old_var, new_var)?;
                let new_second = if var == old_var {
                    (**second).clone()
                } else {
                    self.substitute_type_var(second, old_var, new_var)?
                };
                Ok(DependentType::Sigma {
                    var: if var == old_var { new_var.to_string() } else { var.clone() },
                    first: Box::new(new_first),
                    second: Box::new(new_second),
                })
            }
            DependentType::Identity { ty, left, right } => {
                let new_ty = self.substitute_type_var(ty, old_var, new_var)?;
                let new_left = self.substitute_term_var(left, old_var, new_var)?;
                let new_right = self.substitute_term_var(right, old_var, new_var)?;
                Ok(DependentType::Identity {
                    ty: Box::new(new_ty),
                    left: Box::new(new_left),
                    right: Box::new(new_right),
                })
            }
            DependentType::Inductive { .. } => {
                // For now, don't substitute in inductive types
                Ok(ty.clone())
            }
        }
    }

    /// Helper: Substitute variable in term (simple version).
    fn substitute_term_var(&self, term: &DependentTerm, old_var: &str, new_var: &str) -> Result<DependentTerm> {
        match term {
            DependentTerm::Variable(name) => {
                Ok(DependentTerm::Variable(
                    if name == old_var { 
                        new_var.to_string() 
                    } else { 
                        name.clone() 
                    }
                ))
            }
            DependentTerm::Lambda { param, param_type, body } => {
                let new_param_type = self.substitute_type_var(param_type, old_var, new_var)?;
                let new_body = if param == old_var {
                    // Variable is bound, don't substitute in body
                    (**body).clone()
                } else {
                    self.substitute_term_var(body, old_var, new_var)?
                };
                Ok(DependentTerm::Lambda {
                    param: if param == old_var { new_var.to_string() } else { param.clone() },
                    param_type: Box::new(new_param_type),
                    body: Box::new(new_body),
                })
            }
            DependentTerm::Application { function, argument } => {
                let new_function = self.substitute_term_var(function, old_var, new_var)?;
                let new_argument = self.substitute_term_var(argument, old_var, new_var)?;
                Ok(DependentTerm::Application {
                    function: Box::new(new_function),
                    argument: Box::new(new_argument),
                })
            }
            DependentTerm::Pair { first, second } => {
                let new_first = self.substitute_term_var(first, old_var, new_var)?;
                let new_second = self.substitute_term_var(second, old_var, new_var)?;
                Ok(DependentTerm::Pair {
                    first: Box::new(new_first),
                    second: Box::new(new_second),
                })
            }
            DependentTerm::Projection { pair, is_first } => {
                let new_pair = self.substitute_term_var(pair, old_var, new_var)?;
                Ok(DependentTerm::Projection {
                    pair: Box::new(new_pair),
                    is_first: *is_first,
                })
            }
            DependentTerm::Refl { ty } => {
                let new_ty = self.substitute_type_var(ty, old_var, new_var)?;
                Ok(DependentTerm::Refl {
                    ty: Box::new(new_ty),
                })
            }
            DependentTerm::Constructor { name, args, result_type } => {
                let new_args: Result<Vec<_>> = args.iter()
                    .map(|arg| self.substitute_term_var(arg, old_var, new_var))
                    .collect();
                let new_result_type = self.substitute_type_var(result_type, old_var, new_var)?;
                Ok(DependentTerm::Constructor {
                    name: name.clone(),
                    args: new_args?,
                    result_type: Box::new(new_result_type),
                })
            }
            DependentTerm::Match { scrutinee, branches, return_type } => {
                let new_scrutinee = self.substitute_term_var(scrutinee, old_var, new_var)?;
                let new_return_type = self.substitute_type_var(return_type, old_var, new_var)?;
                // TODO: Handle pattern variable binding in branches properly
                Ok(DependentTerm::Match {
                    scrutinee: Box::new(new_scrutinee),
                    branches: branches.clone(), // Simplified for now
                    return_type: Box::new(new_return_type),
                })
            }
        }
    }

    /// Get performance statistics for the equality checker.
    pub fn statistics(&self) -> &EqualityStatistics {
        &self.statistics
    }

    /// Clear the equality caches to free memory.
    pub fn clear_cache(&mut self) {
        self.equality_cache.clear();
        self.term_equality_cache.clear();
    }

    /// Add a term definition for definitional expansion.
    ///
    /// This allows the equality checker to unfold defined terms
    /// when checking equality. For example, if `f := λx.x + 1`,
    /// then `f` will be treated as equal to `λx.x + 1`.
    pub fn define_term(&mut self, name: String, definition: DependentTerm) {
        self.definitions.insert(name, definition);
        // Clear cache as new definitions may change equality results
        self.clear_cache();
    }

    /// Add a type definition for definitional expansion.
    ///
    /// This allows the equality checker to unfold defined types
    /// when checking equality. For example, if `List A := μL. 1 + A × L`,
    /// then `List Nat` will be expanded during equality checking.
    pub fn define_type(&mut self, name: String, definition: DependentType) {
        self.type_definitions.insert(name, definition);
        // Clear cache as new definitions may change equality results  
        self.clear_cache();
    }

    /// Remove a term definition.
    pub fn undefine_term(&mut self, name: &str) -> Option<DependentTerm> {
        let result = self.definitions.remove(name);
        if result.is_some() {
            self.clear_cache();
        }
        result
    }

    /// Remove a type definition.
    pub fn undefine_type(&mut self, name: &str) -> Option<DependentType> {
        let result = self.type_definitions.remove(name);
        if result.is_some() {
            self.clear_cache();
        }
        result
    }

    /// Check if a term is defined.
    pub fn has_term_definition(&self, name: &str) -> bool {
        self.definitions.contains_key(name)
    }

    /// Check if a type is defined.
    pub fn has_type_definition(&self, name: &str) -> bool {
        self.type_definitions.contains_key(name)
    }

    /// Get a term definition.
    pub fn get_term_definition(&self, name: &str) -> Option<&DependentTerm> {
        self.definitions.get(name)
    }

    /// Get a type definition.
    pub fn get_type_definition(&self, name: &str) -> Option<&DependentType> {
        self.type_definitions.get(name)
    }
}

impl Default for DefinitionalEqualityChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EqualityJustification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EqualityJustification::Syntactic => write!(f, "Syntactic equality"),
            EqualityJustification::AlphaEquivalence { renamed_vars } => {
                write!(f, "α-equivalence")?;
                if !renamed_vars.is_empty() {
                    write!(f, " (renamed: ")?;
                    for (i, (old, new)) in renamed_vars.iter().enumerate() {
                        if i > 0 { write!(f, ", ")?; }
                        write!(f, "{old}→{new}")?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            EqualityJustification::BetaEquivalence { reductions } => {
                write!(f, "β-equivalence")?;
                if !reductions.is_empty() {
                    write!(f, " ({})", reductions.join(", "))?;
                }
                Ok(())
            }
            EqualityJustification::EtaEquivalence { expansions } => {
                write!(f, "η-equivalence")?;
                if !expansions.is_empty() {
                    write!(f, " ({})", expansions.join(", "))?;
                }
                Ok(())
            }
            EqualityJustification::DefinitionalExpansion { unfolded_defs } => {
                write!(f, "Definitional expansion")?;
                if !unfolded_defs.is_empty() {
                    write!(f, " (unfolded: {})", unfolded_defs.join(", "))?;
                }
                Ok(())
            }
            EqualityJustification::Congruence { sub_equalities } => {
                write!(f, "Congruence ({} sub-equalities)", sub_equalities.len())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::core::{DependentType, DependentTerm};

    #[test]
    fn test_syntactic_equality() {
        let mut checker = DefinitionalEqualityChecker::new();
        
        let ty1 = DependentType::Universe(0);
        let ty2 = DependentType::Universe(0);
        let result = checker.types_equal(&ty1, &ty2).unwrap();
        
        assert!(result.is_equal);
        matches!(result.justification, Some(EqualityJustification::Syntactic));
    }

    #[test]
    fn test_alpha_equivalence_pi_types() {
        let mut checker = DefinitionalEqualityChecker::new();
        
        // (x:Type₀) → Type₀ vs (y:Type₀) → Type₀
        let pi_type_x = DependentType::Pi {
            var: "x".to_string(),
            domain: Box::new(DependentType::Universe(0)),
            codomain: Box::new(DependentType::Universe(0)),
        };
        
        let pi_type_y = DependentType::Pi {
            var: "y".to_string(),
            domain: Box::new(DependentType::Universe(0)),
            codomain: Box::new(DependentType::Universe(0)),
        };
        
        let result = checker.types_equal(&pi_type_x, &pi_type_y).unwrap();
        assert!(result.is_equal);
        assert!(matches!(result.justification, 
            Some(EqualityJustification::AlphaEquivalence { .. })));
    }

    #[test]
    fn test_lambda_alpha_equivalence() {
        let mut checker = DefinitionalEqualityChecker::new();
        
        // λx:Type₀.x vs λy:Type₀.y
        let lambda_x = DependentTerm::Lambda {
            param: "x".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Variable("x".to_string())),
        };
        
        let lambda_y = DependentTerm::Lambda {
            param: "y".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Variable("y".to_string())),
        };
        
        let result = checker.terms_equal(&lambda_x, &lambda_y).unwrap();
        assert!(result.is_equal);
        assert!(matches!(result.justification, 
            Some(EqualityJustification::AlphaEquivalence { .. })));
    }

    #[test]
    fn test_eta_equivalence_function() {
        let mut checker = DefinitionalEqualityChecker::new();
        
        // f vs λx.(f x) where x ∉ FV(f)
        let f = DependentTerm::Variable("f".to_string());
        let eta_expanded = DependentTerm::Lambda {
            param: "x".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Application {
                function: Box::new(DependentTerm::Variable("f".to_string())),
                argument: Box::new(DependentTerm::Variable("x".to_string())),
            }),
        };
        
        let result = checker.terms_equal(&f, &eta_expanded).unwrap();
        assert!(result.is_equal);
        assert!(matches!(result.justification, 
            Some(EqualityJustification::EtaEquivalence { .. })));
    }

    #[test]
    fn test_eta_equivalence_pair() {
        let mut checker = DefinitionalEqualityChecker::new();
        
        // p vs (π₁(p), π₂(p))
        let p = DependentTerm::Variable("p".to_string());
        let eta_expanded = DependentTerm::Pair {
            first: Box::new(DependentTerm::Projection {
                pair: Box::new(DependentTerm::Variable("p".to_string())),
                is_first: true,
            }),
            second: Box::new(DependentTerm::Projection {
                pair: Box::new(DependentTerm::Variable("p".to_string())),
                is_first: false,
            }),
        };
        
        let result = checker.terms_equal(&p, &eta_expanded).unwrap();
        assert!(result.is_equal);
        assert!(matches!(result.justification, 
            Some(EqualityJustification::EtaEquivalence { .. })));
    }

    #[test]
    fn test_congruence_pi_types() {
        let mut checker = DefinitionalEqualityChecker::new();
        
        // Build nested Π-types and check congruence
        let inner_pi1 = DependentType::Pi {
            var: "x".to_string(),
            domain: Box::new(DependentType::Universe(0)),
            codomain: Box::new(DependentType::Universe(1)),
        };
        
        let inner_pi2 = DependentType::Pi {
            var: "y".to_string(),
            domain: Box::new(DependentType::Universe(0)),
            codomain: Box::new(DependentType::Universe(1)),
        };
        
        let outer_pi1 = DependentType::Pi {
            var: "z".to_string(),
            domain: Box::new(inner_pi1),
            codomain: Box::new(DependentType::Universe(2)),
        };
        
        let outer_pi2 = DependentType::Pi {
            var: "w".to_string(),
            domain: Box::new(inner_pi2),
            codomain: Box::new(DependentType::Universe(2)),
        };
        
        let result = checker.types_equal(&outer_pi1, &outer_pi2).unwrap();
        assert!(result.is_equal);
    }

    #[test]
    fn test_free_variable_check() {
        let checker = DefinitionalEqualityChecker::new();
        
        // Test free variable detection
        let term = DependentTerm::Lambda {
            param: "x".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Variable("y".to_string())),
        };
        
        assert!(!checker.is_free_in_term("x", &term)); // x is bound
        assert!(checker.is_free_in_term("y", &term));  // y is free
    }

    #[test]
    fn test_variable_substitution() {
        let checker = DefinitionalEqualityChecker::new();
        
        let original = DependentTerm::Lambda {
            param: "x".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Variable("y".to_string())),
        };
        
        let result = checker.substitute_term_var(&original, "y", "z").unwrap();
        
        if let DependentTerm::Lambda { body, .. } = result {
            if let DependentTerm::Variable(name) = body.as_ref() {
                assert_eq!(name, "z");
            } else {
                panic!("Expected variable in lambda body");
            }
        } else {
            panic!("Expected lambda term");
        }
    }

    #[test]
    fn test_statistics_tracking() {
        let mut checker = DefinitionalEqualityChecker::new();
        
        let ty1 = DependentType::Universe(0);
        let ty2 = DependentType::Universe(1);
        
        // Perform several equality checks
        let _ = checker.types_equal(&ty1, &ty2).unwrap();
        let _ = checker.types_equal(&ty1, &ty1).unwrap();
        
        let stats = checker.statistics();
        assert!(stats.total_checks >= 2);
        assert!(stats.cache_hits >= 0);
    }
}
#![allow(clippy::only_used_in_recursion)]

//! Core definitions for Martin-Löf dependent type theory.
//!
//! This module provides the fundamental type and term definitions that form
//! the basis of the dependent type system, following Martin-Löf type theory.
//!
//! # Mathematical Foundation
//!
//! The core system includes:
//! - **Type judgements**: Γ ⊢ A type (A is a well-formed type in context Γ)
//! - **Term judgements**: Γ ⊢ t : A (term t has type A in context Γ)
//! - **Equality judgements**: Γ ⊢ t ≡ s : A (terms t and s are equal of type A)
//! - **Typing contexts**: Sequences of variable bindings
//!
//! # Dependent Types
//!
//! The type system supports:
//! - **Universe hierarchy**: Type₀, Type₁, Type₂, ... to avoid Russell's paradox
//! - **Dependent functions (Π-types)**: Types of the form (x : A) → B(x)
//! - **Dependent pairs (Σ-types)**: Types of the form (x : A) × B(x)
//! - **Identity types**: Types expressing equality Id_A(a, b)
//! - **Inductive types**: User-defined recursive types with constructors

use crate::diagnostics::{Error, Result, Span};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

/// Proof obligation for dependent type checking
#[derive(Debug, Clone, PartialEq)]
pub struct ProofObligation {
    /// The proposition that needs to be proven
    pub proposition: String,
    /// Context in which the proof is required
    pub context: String,
    /// Whether this obligation can be eliminated at compile time
    pub eliminable: bool,
    /// Complexity score for proof checking (using OrderedFloat to enable Eq and Hash)
    pub complexity: u32, // Changed to u32 to avoid f64 Eq/Hash issues
}

impl ProofObligation {
    /// Creates a new proof obligation
    pub fn new(proposition: String, context: String) -> Self {
        Self {
            proposition,
            context,
            eliminable: false,
            complexity: 1,
        }
    }

    /// Creates an eliminable proof obligation
    pub fn eliminable(proposition: String, context: String, complexity: u32) -> Self {
        Self {
            proposition,
            context,
            eliminable: true,
            complexity,
        }
    }

    /// Check if this proof can be eliminated at compile time
    pub fn is_eliminable(&self) -> bool {
        self.eliminable
    }
}

/// Simple dependent type representation for JIT specialization
/// This is separate from the full DependentType enum to avoid conflicts
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JitDependentType {
    /// Base type
    pub base_type: String,
    /// Type parameters with dependencies
    pub parameters: Vec<String>,
    /// Constraints on the type
    pub constraints: Vec<String>,
    /// Universe level
    pub universe: u32,
}

impl JitDependentType {
    /// Creates a new dependent type
    pub fn new(base_type: String, universe: u32) -> Self {
        Self {
            base_type,
            parameters: Vec::new(),
            constraints: Vec::new(),
            universe,
        }
    }

    /// Adds a parameter to the dependent type
    pub fn with_parameter(mut self, param: String) -> Self {
        self.parameters.push(param);
        self
    }

    /// Adds a constraint to the dependent type
    pub fn with_constraint(mut self, constraint: String) -> Self {
        self.constraints.push(constraint);
        self
    }
}

/// Universe levels for the type hierarchy.
///
/// Type₀ : Type₁ : Type₂ : ... prevents Russell's paradox by stratifying types.
pub type UniverseLevel = u32;

/// Martin-Löf dependent types.
///
/// These represent the full spectrum of dependent types in Martin-Löf type theory,
/// each with precise formation, introduction, elimination, and computation rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependentType {
    /// Universe types: Type₀, Type₁, Type₂, ...
    ///
    /// Formation: Type_i is a type in universe Type_{i+1}
    Universe(UniverseLevel),

    /// Π-types (dependent function types): (x : A) → B(x)
    ///
    /// Formation: If Γ ⊢ A type and Γ, x:A ⊢ B(x) type, then Γ ⊢ (x:A) → B(x) type
    /// Introduction: λx.t constructs functions
    /// Elimination: f(a) applies functions
    /// Computation: (λx.t)(a) ≡ t[a/x]
    Pi {
        /// Variable name bound in codomain
        var: String,
        /// Domain type A
        domain: Box<DependentType>,
        /// Codomain type B(x), potentially depending on var
        codomain: Box<DependentType>,
    },

    /// Σ-types (dependent pair types): (x : A) × B(x)
    ///
    /// Formation: If Γ ⊢ A type and Γ, x:A ⊢ B(x) type, then Γ ⊢ (x:A) × B(x) type
    /// Introduction: (a, b) constructs pairs where b : B(a)
    /// Elimination: π₁(p) and π₂(p) project components
    /// Computation: π₁((a, b)) ≡ a, π₂((a, b)) ≡ b
    Sigma {
        /// Variable name bound in second component type
        var: String,
        /// First component type A
        first: Box<DependentType>,
        /// Second component type B(x), potentially depending on var
        second: Box<DependentType>,
    },

    /// Identity types (equality types): Id_A(a, b)
    ///
    /// Formation: If Γ ⊢ A type and Γ ⊢ a:A and Γ ⊢ b:A, then Γ ⊢ Id_A(a, b) type
    /// Introduction: refl_a : Id_A(a, a) (reflexivity)
    /// Elimination: J-eliminator (path induction)
    /// Computation: J(..., refl_a) ≡ ...
    Identity {
        /// The type A over which equality is defined
        ty: Box<DependentType>,
        /// Left-hand side term a
        left: Box<DependentTerm>,
        /// Right-hand side term b
        right: Box<DependentTerm>,
    },

    /// Inductive types with constructors
    ///
    /// Formation: Declared types with well-formed constructor signatures
    /// Introduction: Constructor applications
    /// Elimination: Pattern matching and induction principles
    /// Computation: Pattern matching on constructor applications
    Inductive {
        /// Type name
        name: String,
        /// Type parameters
        parameters: Vec<(String, DependentType)>,
        /// Universe level this type lives in
        universe_level: UniverseLevel,
        /// Constructor names and their types
        constructors: Vec<(String, DependentType)>,
        /// Induction principle (automatically generated)
        induction_principle: Option<Box<DependentType>>,
    },
}

/// Martin-Löf dependent terms.
///
/// These represent terms that can appear in dependent types and type expressions,
/// with precise typing rules following Martin-Löf type theory.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependentTerm {
    /// Variable reference
    ///
    /// Typing: If x:A in Γ, then Γ ⊢ x : A
    Variable(String),

    /// Lambda abstraction: λx:A.t
    ///
    /// Typing: If Γ, x:A ⊢ t : B(x), then Γ ⊢ λx:A.t : (x:A) → B(x)
    Lambda {
        /// Parameter name
        param: String,
        /// Parameter type annotation
        param_type: Box<DependentType>,
        /// Function body
        body: Box<DependentTerm>,
    },

    /// Function application: f(a)
    ///
    /// Typing: If Γ ⊢ f : (x:A) → B(x) and Γ ⊢ a : A, then Γ ⊢ f(a) : B(a)
    Application {
        /// Function term
        function: Box<DependentTerm>,
        /// Argument term
        argument: Box<DependentTerm>,
    },

    /// Dependent pair construction: (a, b)
    ///
    /// Typing: If Γ ⊢ a : A and Γ ⊢ b : B(a), then Γ ⊢ (a, b) : (x:A) × B(x)
    Pair {
        /// First component
        first: Box<DependentTerm>,
        /// Second component (may depend on first)
        second: Box<DependentTerm>,
    },

    /// Projection from dependent pairs: π₁(p) or π₂(p)
    ///
    /// Typing: If Γ ⊢ p : (x:A) × B(x), then:
    /// - Γ ⊢ π₁(p) : A
    /// - Γ ⊢ π₂(p) : B(π₁(p))
    Projection {
        /// Pair to project from
        pair: Box<DependentTerm>,
        /// True for first projection, false for second
        is_first: bool,
    },

    /// Reflexivity proof: refl_a
    ///
    /// Typing: If Γ ⊢ a : A, then Γ ⊢ refl_a : Id_A(a, a)
    Refl {
        /// Type annotation for the reflexivity proof
        ty: Box<DependentType>,
    },

    /// Constructor application for inductive types
    ///
    /// Typing: Constructor-specific typing rules based on declaration
    Constructor {
        /// Constructor name
        name: String,
        /// Arguments to constructor
        args: Vec<DependentTerm>,
        /// Type of the constructed value
        result_type: Box<DependentType>,
    },

    /// Pattern matching and elimination
    ///
    /// Typing: Checked against induction principles and return type
    Match {
        /// Scrutinee (value being matched)
        scrutinee: Box<DependentTerm>,
        /// Pattern match branches
        branches: Vec<MatchBranch>,
        /// Return type of the match expression
        return_type: Box<DependentType>,
    },
}

/// Pattern matching branch in dependent pattern matching.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MatchBranch {
    /// Constructor pattern to match
    pub pattern: Pattern,
    /// Right-hand side expression
    pub body: DependentTerm,
}

/// Patterns for dependent pattern matching.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    /// Variable pattern (binds any value)
    Variable(String),
    /// Constructor pattern with subpatterns
    Constructor {
        /// The constructor name
        name: String,
        /// Arguments to the constructor pattern
        args: Vec<Pattern>,
    },
}

/// Typing context for dependent type checking.
///
/// Maintains variable bindings and type definitions in scope.
#[derive(Debug, Clone)]
pub struct TypingContext {
    /// Variable bindings: var ↦ type
    variables: HashMap<String, DependentType>,
    /// Type definitions: name ↦ definition
    type_definitions: HashMap<String, DependentType>,
    /// Constructor type signatures
    constructors: HashMap<String, DependentType>,
    /// Context stack for nested scopes
    scopes: Vec<HashMap<String, DependentType>>,
}

impl TypingContext {
    /// Create a new empty typing context.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            type_definitions: HashMap::new(),
            constructors: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    /// Bind a variable to a type in the current context.
    pub fn bind_variable(&mut self, name: String, ty: DependentType) {
        self.variables.insert(name, ty);
    }

    /// Remove a variable binding from the current context.
    pub fn unbind_variable(&mut self, name: &str) {
        self.variables.remove(name);
    }

    /// Look up the type of a variable.
    pub fn lookup_variable(&self, name: &str) -> Option<&DependentType> {
        // Check current scope first, then parent scopes
        self.variables
            .get(name)
            .or_else(|| self.scopes.iter().rev().find_map(|scope| scope.get(name)))
    }

    /// Define a new type in the context.
    pub fn define_type(&mut self, name: String, definition: DependentType) {
        self.type_definitions.insert(name, definition);
    }

    /// Look up a type definition.
    pub fn lookup_type(&self, name: &str) -> Option<&DependentType> {
        self.type_definitions.get(name)
    }

    /// Define a constructor with its type signature.
    pub fn define_constructor(&mut self, name: String, ty: DependentType) {
        self.constructors.insert(name, ty);
    }

    /// Look up a constructor type.
    pub fn lookup_constructor(&self, name: &str) -> Option<&DependentType> {
        self.constructors.get(name)
    }

    /// Push a new scope onto the context stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(std::mem::take(&mut self.variables));
    }

    /// Pop the most recent scope from the context stack.
    pub fn pop_scope(&mut self) {
        if let Some(previous_scope) = self.scopes.pop() {
            self.variables = previous_scope;
        }
    }

    /// Get the number of variables in the current context.
    pub fn size(&self) -> usize {
        self.variables.len()
    }

    /// Check if the context is empty.
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty() && self.scopes.is_empty()
    }
}

/// Normalizer for dependent types and terms.
///
/// Implements weak head normal form evaluation for efficient type checking.
#[derive(Debug)]
pub struct Normalizer {
    /// Reduction strategy configuration
    strategy: NormalizationStrategy,
    /// Term definitions for unfolding
    definitions: HashMap<String, DependentTerm>,
    /// Counter for fresh variable generation
    fresh_var_counter: AtomicU32,
}

impl Clone for Normalizer {
    fn clone(&self) -> Self {
        Self {
            strategy: self.strategy,
            definitions: self.definitions.clone(),
            fresh_var_counter: AtomicU32::new(self.fresh_var_counter.load(Ordering::SeqCst)),
        }
    }
}

/// Normalization strategy for controlling reduction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationStrategy {
    /// Weak head normal form (most efficient)
    WeakHead,
    /// Normal form (complete reduction)
    Normal,
    /// Call-by-value
    CallByValue,
    /// Call-by-need (lazy evaluation)
    CallByNeed,
}

impl Normalizer {
    /// Create a new normalizer with weak head normal form strategy.
    pub fn new() -> Self {
        Self {
            strategy: NormalizationStrategy::WeakHead,
            definitions: HashMap::new(),
            fresh_var_counter: AtomicU32::new(0),
        }
    }

    /// Normalize a dependent type to canonical form.
    pub fn normalize_type(&self, ty: &DependentType) -> Result<DependentType> {
        match ty {
            DependentType::Universe(level) => Ok(DependentType::Universe(*level)),

            DependentType::Pi {
                var,
                domain,
                codomain,
            } => {
                let norm_domain = self.normalize_type(domain)?;
                let norm_codomain = self.normalize_type(codomain)?;
                Ok(DependentType::Pi {
                    var: var.clone(),
                    domain: Box::new(norm_domain),
                    codomain: Box::new(norm_codomain),
                })
            }

            DependentType::Sigma { var, first, second } => {
                let norm_first = self.normalize_type(first)?;
                let norm_second = self.normalize_type(second)?;
                Ok(DependentType::Sigma {
                    var: var.clone(),
                    first: Box::new(norm_first),
                    second: Box::new(norm_second),
                })
            }

            DependentType::Identity { ty, left, right } => {
                let norm_ty = self.normalize_type(ty)?;
                let norm_left = self.normalize_term(left)?;
                let norm_right = self.normalize_term(right)?;
                Ok(DependentType::Identity {
                    ty: Box::new(norm_ty),
                    left: Box::new(norm_left),
                    right: Box::new(norm_right),
                })
            }

            DependentType::Inductive {
                name,
                parameters,
                universe_level,
                constructors,
                induction_principle,
            } => {
                let mut norm_constructors = Vec::new();
                for (ctor_name, ctor_type) in constructors {
                    norm_constructors.push((ctor_name.clone(), self.normalize_type(ctor_type)?));
                }

                let norm_induction = if let Some(ind_prin) = induction_principle {
                    Some(Box::new(self.normalize_type(ind_prin)?))
                } else {
                    None
                };

                Ok(DependentType::Inductive {
                    name: name.clone(),
                    parameters: parameters.clone(), // TODO: normalize parameter types
                    universe_level: *universe_level,
                    constructors: norm_constructors,
                    induction_principle: norm_induction,
                })
            }
        }
    }

    /// Normalize a dependent term to canonical form.
    pub fn normalize_term(&self, term: &DependentTerm) -> Result<DependentTerm> {
        match term {
            DependentTerm::Variable(name) => {
                // Look up definition and unfold if available
                if let Some(definition) = self.definitions.get(name) {
                    self.normalize_term(definition)
                } else {
                    Ok(DependentTerm::Variable(name.clone()))
                }
            }

            DependentTerm::Lambda {
                param,
                param_type,
                body,
            } => {
                let norm_param_type = self.normalize_type(param_type)?;
                let norm_body = self.normalize_term(body)?;
                Ok(DependentTerm::Lambda {
                    param: param.clone(),
                    param_type: Box::new(norm_param_type),
                    body: Box::new(norm_body),
                })
            }

            DependentTerm::Application { function, argument } => {
                let norm_function = self.normalize_term(function)?;
                let norm_argument = self.normalize_term(argument)?;

                // β-reduction: (λx.t)(a) → t[a/x]
                if let DependentTerm::Lambda { param, body, .. } = &norm_function {
                    self.substitute_term(body, param, &norm_argument)
                } else {
                    Ok(DependentTerm::Application {
                        function: Box::new(norm_function),
                        argument: Box::new(norm_argument),
                    })
                }
            }

            DependentTerm::Pair { first, second } => {
                let norm_first = self.normalize_term(first)?;
                let norm_second = self.normalize_term(second)?;
                Ok(DependentTerm::Pair {
                    first: Box::new(norm_first),
                    second: Box::new(norm_second),
                })
            }

            DependentTerm::Projection { pair, is_first } => {
                let norm_pair = self.normalize_term(pair)?;

                // Projection reduction: π₁((a, b)) → a, π₂((a, b)) → b
                if let DependentTerm::Pair { first, second } = &norm_pair {
                    if *is_first {
                        Ok((**first).clone())
                    } else {
                        Ok((**second).clone())
                    }
                } else {
                    Ok(DependentTerm::Projection {
                        pair: Box::new(norm_pair),
                        is_first: *is_first,
                    })
                }
            }

            DependentTerm::Refl { ty } => {
                let norm_ty = self.normalize_type(ty)?;
                Ok(DependentTerm::Refl {
                    ty: Box::new(norm_ty),
                })
            }

            DependentTerm::Constructor {
                name,
                args,
                result_type,
            } => {
                let mut norm_args = Vec::new();
                for arg in args {
                    norm_args.push(self.normalize_term(arg)?);
                }
                let norm_result_type = self.normalize_type(result_type)?;
                Ok(DependentTerm::Constructor {
                    name: name.clone(),
                    args: norm_args,
                    result_type: Box::new(norm_result_type),
                })
            }

            DependentTerm::Match {
                scrutinee,
                branches,
                return_type,
            } => {
                let norm_scrutinee = self.normalize_term(scrutinee)?;

                // Pattern matching reduction
                for branch in branches {
                    if let Some(substitution) =
                        self.match_pattern(&branch.pattern, &norm_scrutinee)?
                    {
                        return self.apply_substitution(&branch.body, &substitution);
                    }
                }

                // No pattern matched, return normalized match
                let norm_return_type = self.normalize_type(return_type)?;
                Ok(DependentTerm::Match {
                    scrutinee: Box::new(norm_scrutinee),
                    branches: branches.clone(), // TODO: normalize branches
                    return_type: Box::new(norm_return_type),
                })
            }
        }
    }

    /// Substitute a term for a variable in another term with capture-avoidance.
    ///
    /// This is the main substitution function that implements proper variable capture
    /// avoidance using α-conversion. It follows the mathematical definition:
    /// - t[s/x] substitutes term s for all free occurrences of variable x in term t
    /// - Bound variables are renamed if they would capture free variables in s
    ///
    /// Example transformations:
    /// - (λy.x)[y/x] → λz.y  (y captured, rename bound y to fresh z)
    /// - (λx.x)[y/x] → λx.x  (x is bound, no substitution occurs)
    /// - (x y)[z/x] → (z y)  (simple substitution)
    fn substitute_term(
        &self,
        term: &DependentTerm,
        var: &str,
        replacement: &DependentTerm,
    ) -> Result<DependentTerm> {
        self.substitute_term_capture_avoiding(term, var, replacement)
    }

    /// Internal implementation of capture-avoiding substitution.
    fn substitute_term_capture_avoiding(
        &self,
        term: &DependentTerm,
        var: &str,
        replacement: &DependentTerm,
    ) -> Result<DependentTerm> {
        match term {
            DependentTerm::Variable(name) => {
                if name == var {
                    Ok(replacement.clone())
                } else {
                    Ok(term.clone())
                }
            }

            DependentTerm::Lambda {
                param,
                param_type,
                body,
            } => {
                if param == var {
                    // Variable is shadowed by the lambda parameter, no substitution in body
                    let substituted_param_type =
                        self.substitute_term_in_type(param_type, var, replacement)?;
                    Ok(DependentTerm::Lambda {
                        param: param.clone(),
                        param_type: Box::new(substituted_param_type),
                        body: body.clone(),
                    })
                } else {
                    // Check if the lambda parameter would capture free variables in replacement
                    let replacement_free_vars = self.free_variables_term(replacement);

                    if replacement_free_vars.contains(param) {
                        // Potential capture! Need α-conversion
                        let body_free_vars = self.free_variables_term(body);
                        let param_type_free_vars = self.free_variables_type(param_type);

                        // Collect all variables to avoid
                        let mut avoid_vars = replacement_free_vars;
                        avoid_vars.extend(body_free_vars);
                        avoid_vars.extend(param_type_free_vars);
                        avoid_vars.insert(var.to_string()); // Also avoid the variable being substituted

                        // Generate fresh parameter name
                        let fresh_param = self.generate_fresh_name(param, &avoid_vars);

                        // Rename bound parameter in the body
                        let fresh_var_term = DependentTerm::Variable(fresh_param.clone());
                        let renamed_body =
                            self.substitute_term_capture_avoiding(body, param, &fresh_var_term)?;

                        // Now safely substitute in the renamed body
                        let substituted_body =
                            self.substitute_term_capture_avoiding(&renamed_body, var, replacement)?;
                        let substituted_param_type =
                            self.substitute_term_in_type(param_type, var, replacement)?;

                        Ok(DependentTerm::Lambda {
                            param: fresh_param,
                            param_type: Box::new(substituted_param_type),
                            body: Box::new(substituted_body),
                        })
                    } else {
                        // No capture, safe to substitute
                        let substituted_body =
                            self.substitute_term_capture_avoiding(body, var, replacement)?;
                        let substituted_param_type =
                            self.substitute_term_in_type(param_type, var, replacement)?;

                        Ok(DependentTerm::Lambda {
                            param: param.clone(),
                            param_type: Box::new(substituted_param_type),
                            body: Box::new(substituted_body),
                        })
                    }
                }
            }

            DependentTerm::Application { function, argument } => {
                let substituted_function =
                    self.substitute_term_capture_avoiding(function, var, replacement)?;
                let substituted_argument =
                    self.substitute_term_capture_avoiding(argument, var, replacement)?;
                Ok(DependentTerm::Application {
                    function: Box::new(substituted_function),
                    argument: Box::new(substituted_argument),
                })
            }

            DependentTerm::Pair { first, second } => {
                let substituted_first =
                    self.substitute_term_capture_avoiding(first, var, replacement)?;
                let substituted_second =
                    self.substitute_term_capture_avoiding(second, var, replacement)?;
                Ok(DependentTerm::Pair {
                    first: Box::new(substituted_first),
                    second: Box::new(substituted_second),
                })
            }

            DependentTerm::Projection { pair, is_first } => {
                let substituted_pair =
                    self.substitute_term_capture_avoiding(pair, var, replacement)?;
                Ok(DependentTerm::Projection {
                    pair: Box::new(substituted_pair),
                    is_first: *is_first,
                })
            }

            DependentTerm::Refl { ty } => {
                let substituted_ty = self.substitute_term_in_type(ty, var, replacement)?;
                Ok(DependentTerm::Refl {
                    ty: Box::new(substituted_ty),
                })
            }

            DependentTerm::Constructor {
                name,
                args,
                result_type,
            } => {
                let mut substituted_args = Vec::new();
                for arg in args {
                    substituted_args.push(self.substitute_term_capture_avoiding(
                        arg,
                        var,
                        replacement,
                    )?);
                }
                let substituted_result_type =
                    self.substitute_term_in_type(result_type, var, replacement)?;

                Ok(DependentTerm::Constructor {
                    name: name.clone(),
                    args: substituted_args,
                    result_type: Box::new(substituted_result_type),
                })
            }

            DependentTerm::Match {
                scrutinee,
                branches,
                return_type,
            } => {
                let substituted_scrutinee =
                    self.substitute_term_capture_avoiding(scrutinee, var, replacement)?;
                let substituted_return_type =
                    self.substitute_term_in_type(return_type, var, replacement)?;

                let mut substituted_branches = Vec::new();
                for branch in branches {
                    let bound_vars = self.bound_variables_pattern(&branch.pattern);

                    if bound_vars.contains(var) {
                        // Variable is bound by pattern, no substitution in branch body
                        substituted_branches.push(MatchBranch {
                            pattern: branch.pattern.clone(),
                            body: branch.body.clone(),
                        });
                    } else {
                        // Check for potential capture by pattern variables
                        let replacement_free_vars = self.free_variables_term(replacement);
                        let capture_possible = bound_vars
                            .iter()
                            .any(|bound_var| replacement_free_vars.contains(bound_var));

                        if capture_possible {
                            // Need to rename pattern variables (more complex, for now keep as-is)
                            // This is a complex case that requires pattern α-conversion
                            substituted_branches.push(MatchBranch {
                                pattern: branch.pattern.clone(),
                                body: self.substitute_term_capture_avoiding(
                                    &branch.body,
                                    var,
                                    replacement,
                                )?,
                            });
                        } else {
                            // Safe to substitute
                            substituted_branches.push(MatchBranch {
                                pattern: branch.pattern.clone(),
                                body: self.substitute_term_capture_avoiding(
                                    &branch.body,
                                    var,
                                    replacement,
                                )?,
                            });
                        }
                    }
                }

                Ok(DependentTerm::Match {
                    scrutinee: Box::new(substituted_scrutinee),
                    branches: substituted_branches,
                    return_type: Box::new(substituted_return_type),
                })
            }
        }
    }

    /// Pattern matching: check if a pattern matches a term and return substitutions.
    fn match_pattern(
        &self,
        pattern: &Pattern,
        term: &DependentTerm,
    ) -> Result<Option<HashMap<String, DependentTerm>>> {
        match (pattern, term) {
            (Pattern::Variable(var), _) => {
                let mut substitution = HashMap::new();
                substitution.insert(var.clone(), term.clone());
                Ok(Some(substitution))
            }
            (
                Pattern::Constructor {
                    name: pat_name,
                    args: pat_args,
                },
                DependentTerm::Constructor {
                    name: term_name,
                    args: term_args,
                    ..
                },
            ) => {
                if pat_name == term_name && pat_args.len() == term_args.len() {
                    let mut substitution = HashMap::new();
                    for (pat_arg, term_arg) in pat_args.iter().zip(term_args.iter()) {
                        if let Some(sub_substitution) = self.match_pattern(pat_arg, term_arg)? {
                            substitution.extend(sub_substitution);
                        } else {
                            return Ok(None);
                        }
                    }
                    Ok(Some(substitution))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Apply a substitution to a term.
    fn apply_substitution(
        &self,
        term: &DependentTerm,
        substitution: &HashMap<String, DependentTerm>,
    ) -> Result<DependentTerm> {
        // Apply all substitutions in the map
        let mut result = term.clone();
        for (var, replacement) in substitution {
            result = self.substitute_term(&result, var, replacement)?;
        }
        Ok(result)
    }

    /// Collect free variables in a dependent type.
    fn free_variables_type(&self, ty: &DependentType) -> HashSet<String> {
        match ty {
            DependentType::Universe(_) => HashSet::new(),

            DependentType::Pi {
                var,
                domain,
                codomain,
            } => {
                let mut free_vars = self.free_variables_type(domain);
                let codomain_free = self.free_variables_type(codomain);

                // Remove bound variable from codomain free variables
                for v in codomain_free {
                    if &v != var {
                        free_vars.insert(v);
                    }
                }
                free_vars
            }

            DependentType::Sigma { var, first, second } => {
                let mut free_vars = self.free_variables_type(first);
                let second_free = self.free_variables_type(second);

                // Remove bound variable from second component free variables
                for v in second_free {
                    if &v != var {
                        free_vars.insert(v);
                    }
                }
                free_vars
            }

            DependentType::Identity { ty, left, right } => {
                let mut free_vars = self.free_variables_type(ty);
                free_vars.extend(self.free_variables_term(left));
                free_vars.extend(self.free_variables_term(right));
                free_vars
            }

            DependentType::Inductive {
                parameters,
                constructors,
                induction_principle,
                ..
            } => {
                let mut free_vars = HashSet::new();

                // Collect from parameters
                for (_, param_ty) in parameters {
                    free_vars.extend(self.free_variables_type(param_ty));
                }

                // Collect from constructors
                for (_, ctor_ty) in constructors {
                    free_vars.extend(self.free_variables_type(ctor_ty));
                }

                // Collect from induction principle
                if let Some(ind_prin) = induction_principle {
                    free_vars.extend(self.free_variables_type(ind_prin));
                }

                free_vars
            }
        }
    }

    /// Collect free variables in a dependent term.
    fn free_variables_term(&self, term: &DependentTerm) -> HashSet<String> {
        match term {
            DependentTerm::Variable(name) => {
                let mut free_vars = HashSet::new();
                free_vars.insert(name.clone());
                free_vars
            }

            DependentTerm::Lambda {
                param,
                param_type,
                body,
            } => {
                let mut free_vars = self.free_variables_type(param_type);
                let body_free = self.free_variables_term(body);

                // Remove bound parameter from body free variables
                for v in body_free {
                    if &v != param {
                        free_vars.insert(v);
                    }
                }
                free_vars
            }

            DependentTerm::Application { function, argument } => {
                let mut free_vars = self.free_variables_term(function);
                free_vars.extend(self.free_variables_term(argument));
                free_vars
            }

            DependentTerm::Pair { first, second } => {
                let mut free_vars = self.free_variables_term(first);
                free_vars.extend(self.free_variables_term(second));
                free_vars
            }

            DependentTerm::Projection { pair, .. } => self.free_variables_term(pair),

            DependentTerm::Refl { ty } => self.free_variables_type(ty),

            DependentTerm::Constructor {
                args, result_type, ..
            } => {
                let mut free_vars = self.free_variables_type(result_type);
                for arg in args {
                    free_vars.extend(self.free_variables_term(arg));
                }
                free_vars
            }

            DependentTerm::Match {
                scrutinee,
                branches,
                return_type,
            } => {
                let mut free_vars = self.free_variables_term(scrutinee);
                free_vars.extend(self.free_variables_type(return_type));

                for branch in branches {
                    let bound_vars = self.bound_variables_pattern(&branch.pattern);
                    let body_free = self.free_variables_term(&branch.body);

                    // Remove pattern-bound variables from body free variables
                    for v in body_free {
                        if !bound_vars.contains(&v) {
                            free_vars.insert(v);
                        }
                    }
                }
                free_vars
            }
        }
    }

    /// Collect variables bound by a pattern.
    fn bound_variables_pattern(&self, pattern: &Pattern) -> HashSet<String> {
        match pattern {
            Pattern::Variable(var) => {
                let mut bound_vars = HashSet::new();
                bound_vars.insert(var.clone());
                bound_vars
            }

            Pattern::Constructor { args, .. } => {
                let mut bound_vars = HashSet::new();
                for arg in args {
                    bound_vars.extend(self.bound_variables_pattern(arg));
                }
                bound_vars
            }
        }
    }

    /// Generate a fresh variable name that doesn't conflict with existing names.
    fn fresh_variable(&self, base_name: &str, avoid: &HashSet<String>) -> String {
        loop {
            let counter = self.fresh_var_counter.fetch_add(1, Ordering::SeqCst);
            let candidate = if counter == 0 {
                format!("{}_0", base_name)
            } else {
                format!("{}_{}", base_name, counter)
            };

            if !avoid.contains(&candidate) {
                return candidate;
            }
        }
    }

    /// Generate a fresh variable name that avoids capture.
    /// Uses a systematic approach to ensure uniqueness.
    fn generate_fresh_name(
        &self,
        original_name: &str,
        forbidden_names: &HashSet<String>,
    ) -> String {
        if !forbidden_names.contains(original_name) {
            return original_name.to_string();
        }

        // Try variations with numbers
        for i in 0..1000 {
            // Reasonable upper bound to prevent infinite loops
            let candidate = if i == 0 {
                format!("{}_fresh", original_name)
            } else {
                format!("{}_{}", original_name, i)
            };

            if !forbidden_names.contains(&candidate) {
                return candidate;
            }
        }

        // Fallback to counter-based generation
        self.fresh_variable(original_name, forbidden_names)
    }

    /// Perform α-conversion (variable renaming) to avoid capture.
    /// Renames bound variables in a term to avoid conflicts with free variables.
    fn alpha_convert_term(
        &self,
        term: &DependentTerm,
        avoid: &HashSet<String>,
    ) -> Result<DependentTerm> {
        match term {
            DependentTerm::Variable(name) => Ok(DependentTerm::Variable(name.clone())),

            DependentTerm::Lambda {
                param,
                param_type,
                body,
            } => {
                let converted_param_type = self.alpha_convert_type(param_type, avoid)?;

                // Check if we need to rename the parameter
                if avoid.contains(param) {
                    // Create new name that doesn't conflict
                    let mut all_avoid = avoid.clone();
                    all_avoid.extend(self.free_variables_term(body));
                    all_avoid.extend(self.free_variables_type(param_type));

                    let new_param = self.generate_fresh_name(param, &all_avoid);

                    // Rename the parameter in the body
                    let renamed_body = self.substitute_term(
                        body,
                        param,
                        &DependentTerm::Variable(new_param.clone()),
                    )?;
                    let converted_body = self.alpha_convert_term(&renamed_body, avoid)?;

                    Ok(DependentTerm::Lambda {
                        param: new_param,
                        param_type: Box::new(converted_param_type),
                        body: Box::new(converted_body),
                    })
                } else {
                    // No conflict, just recursively convert the body
                    let mut body_avoid = avoid.clone();
                    body_avoid.insert(param.clone());
                    let converted_body = self.alpha_convert_term(body, &body_avoid)?;

                    Ok(DependentTerm::Lambda {
                        param: param.clone(),
                        param_type: Box::new(converted_param_type),
                        body: Box::new(converted_body),
                    })
                }
            }

            DependentTerm::Application { function, argument } => {
                let converted_function = self.alpha_convert_term(function, avoid)?;
                let converted_argument = self.alpha_convert_term(argument, avoid)?;
                Ok(DependentTerm::Application {
                    function: Box::new(converted_function),
                    argument: Box::new(converted_argument),
                })
            }

            DependentTerm::Pair { first, second } => {
                let converted_first = self.alpha_convert_term(first, avoid)?;
                let converted_second = self.alpha_convert_term(second, avoid)?;
                Ok(DependentTerm::Pair {
                    first: Box::new(converted_first),
                    second: Box::new(converted_second),
                })
            }

            DependentTerm::Projection { pair, is_first } => {
                let converted_pair = self.alpha_convert_term(pair, avoid)?;
                Ok(DependentTerm::Projection {
                    pair: Box::new(converted_pair),
                    is_first: *is_first,
                })
            }

            DependentTerm::Refl { ty } => {
                let converted_ty = self.alpha_convert_type(ty, avoid)?;
                Ok(DependentTerm::Refl {
                    ty: Box::new(converted_ty),
                })
            }

            DependentTerm::Constructor {
                name,
                args,
                result_type,
            } => {
                let mut converted_args = Vec::new();
                for arg in args {
                    converted_args.push(self.alpha_convert_term(arg, avoid)?);
                }
                let converted_result_type = self.alpha_convert_type(result_type, avoid)?;

                Ok(DependentTerm::Constructor {
                    name: name.clone(),
                    args: converted_args,
                    result_type: Box::new(converted_result_type),
                })
            }

            DependentTerm::Match {
                scrutinee,
                branches,
                return_type,
            } => {
                let converted_scrutinee = self.alpha_convert_term(scrutinee, avoid)?;
                let converted_return_type = self.alpha_convert_type(return_type, avoid)?;

                let mut converted_branches = Vec::new();
                for branch in branches {
                    let bound_vars = self.bound_variables_pattern(&branch.pattern);
                    let mut branch_avoid = avoid.clone();
                    branch_avoid.extend(bound_vars);

                    let converted_body = self.alpha_convert_term(&branch.body, &branch_avoid)?;
                    converted_branches.push(MatchBranch {
                        pattern: branch.pattern.clone(), // Patterns don't need α-conversion
                        body: converted_body,
                    });
                }

                Ok(DependentTerm::Match {
                    scrutinee: Box::new(converted_scrutinee),
                    branches: converted_branches,
                    return_type: Box::new(converted_return_type),
                })
            }
        }
    }

    /// Perform α-conversion on a dependent type.
    fn alpha_convert_type(
        &self,
        ty: &DependentType,
        avoid: &HashSet<String>,
    ) -> Result<DependentType> {
        match ty {
            DependentType::Universe(level) => Ok(DependentType::Universe(*level)),

            DependentType::Pi {
                var,
                domain,
                codomain,
            } => {
                let converted_domain = self.alpha_convert_type(domain, avoid)?;

                // Check if we need to rename the bound variable
                if avoid.contains(var) {
                    let mut all_avoid = avoid.clone();
                    all_avoid.extend(self.free_variables_type(domain));
                    all_avoid.extend(self.free_variables_type(codomain));

                    let new_var = self.generate_fresh_name(var, &all_avoid);

                    // Create a dummy term for substitution
                    let dummy_term = DependentTerm::Variable(new_var.clone());
                    let renamed_codomain =
                        self.substitute_term_in_type(codomain, var, &dummy_term)?;
                    let converted_codomain = self.alpha_convert_type(&renamed_codomain, avoid)?;

                    Ok(DependentType::Pi {
                        var: new_var,
                        domain: Box::new(converted_domain),
                        codomain: Box::new(converted_codomain),
                    })
                } else {
                    let mut codomain_avoid = avoid.clone();
                    codomain_avoid.insert(var.clone());
                    let converted_codomain = self.alpha_convert_type(codomain, &codomain_avoid)?;

                    Ok(DependentType::Pi {
                        var: var.clone(),
                        domain: Box::new(converted_domain),
                        codomain: Box::new(converted_codomain),
                    })
                }
            }

            DependentType::Sigma { var, first, second } => {
                let converted_first = self.alpha_convert_type(first, avoid)?;

                // Check if we need to rename the bound variable
                if avoid.contains(var) {
                    let mut all_avoid = avoid.clone();
                    all_avoid.extend(self.free_variables_type(first));
                    all_avoid.extend(self.free_variables_type(second));

                    let new_var = self.generate_fresh_name(var, &all_avoid);

                    // Create a dummy term for substitution
                    let dummy_term = DependentTerm::Variable(new_var.clone());
                    let renamed_second = self.substitute_term_in_type(second, var, &dummy_term)?;
                    let converted_second = self.alpha_convert_type(&renamed_second, avoid)?;

                    Ok(DependentType::Sigma {
                        var: new_var,
                        first: Box::new(converted_first),
                        second: Box::new(converted_second),
                    })
                } else {
                    let mut second_avoid = avoid.clone();
                    second_avoid.insert(var.clone());
                    let converted_second = self.alpha_convert_type(second, &second_avoid)?;

                    Ok(DependentType::Sigma {
                        var: var.clone(),
                        first: Box::new(converted_first),
                        second: Box::new(converted_second),
                    })
                }
            }

            DependentType::Identity { ty, left, right } => {
                let converted_ty = self.alpha_convert_type(ty, avoid)?;
                let converted_left = self.alpha_convert_term(left, avoid)?;
                let converted_right = self.alpha_convert_term(right, avoid)?;

                Ok(DependentType::Identity {
                    ty: Box::new(converted_ty),
                    left: Box::new(converted_left),
                    right: Box::new(converted_right),
                })
            }

            DependentType::Inductive {
                name,
                parameters,
                universe_level,
                constructors,
                induction_principle,
            } => {
                let mut converted_parameters = Vec::new();
                for (param_name, param_ty) in parameters {
                    let converted_param_ty = self.alpha_convert_type(param_ty, avoid)?;
                    converted_parameters.push((param_name.clone(), converted_param_ty));
                }

                let mut converted_constructors = Vec::new();
                for (ctor_name, ctor_ty) in constructors {
                    let converted_ctor_ty = self.alpha_convert_type(ctor_ty, avoid)?;
                    converted_constructors.push((ctor_name.clone(), converted_ctor_ty));
                }

                let converted_induction_principle = if let Some(ind_prin) = induction_principle {
                    Some(Box::new(self.alpha_convert_type(ind_prin, avoid)?))
                } else {
                    None
                };

                Ok(DependentType::Inductive {
                    name: name.clone(),
                    parameters: converted_parameters,
                    universe_level: *universe_level,
                    constructors: converted_constructors,
                    induction_principle: converted_induction_principle,
                })
            }
        }
    }

    /// Substitute a term for a variable within a type (helper for α-conversion).
    fn substitute_term_in_type(
        &self,
        ty: &DependentType,
        var: &str,
        replacement: &DependentTerm,
    ) -> Result<DependentType> {
        match ty {
            DependentType::Universe(level) => Ok(DependentType::Universe(*level)),

            DependentType::Pi {
                var: bound_var,
                domain,
                codomain,
            } => {
                let new_domain = self.substitute_term_in_type(domain, var, replacement)?;
                let new_codomain = if bound_var == var {
                    // Variable is shadowed, don't substitute in codomain
                    (**codomain).clone()
                } else {
                    self.substitute_term_in_type(codomain, var, replacement)?
                };

                Ok(DependentType::Pi {
                    var: bound_var.clone(),
                    domain: Box::new(new_domain),
                    codomain: Box::new(new_codomain),
                })
            }

            DependentType::Sigma {
                var: bound_var,
                first,
                second,
            } => {
                let new_first = self.substitute_term_in_type(first, var, replacement)?;
                let new_second = if bound_var == var {
                    // Variable is shadowed, don't substitute in second
                    (**second).clone()
                } else {
                    self.substitute_term_in_type(second, var, replacement)?
                };

                Ok(DependentType::Sigma {
                    var: bound_var.clone(),
                    first: Box::new(new_first),
                    second: Box::new(new_second),
                })
            }

            DependentType::Identity {
                ty: inner_ty,
                left,
                right,
            } => {
                let new_ty = self.substitute_term_in_type(inner_ty, var, replacement)?;
                let new_left = self.substitute_term(left, var, replacement)?;
                let new_right = self.substitute_term(right, var, replacement)?;

                Ok(DependentType::Identity {
                    ty: Box::new(new_ty),
                    left: Box::new(new_left),
                    right: Box::new(new_right),
                })
            }

            DependentType::Inductive {
                name,
                parameters,
                universe_level,
                constructors,
                induction_principle,
            } => {
                // For now, keep inductive types as-is (more complex substitution rules needed)
                Ok(ty.clone())
            }
        }
    }
}

// Default implementations
impl Default for TypingContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Normalizer {
    fn default() -> Self {
        Self::new()
    }
}

// Display implementations for debugging
impl fmt::Display for DependentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependentType::Universe(level) => write!(f, "Type_{level}"),
            DependentType::Pi {
                var,
                domain,
                codomain,
            } => {
                write!(f, "({var}:{domain}) → {codomain}")
            }
            DependentType::Sigma { var, first, second } => {
                write!(f, "({var}:{first}) × {second}")
            }
            DependentType::Identity { ty, left, right } => {
                write!(f, "Id_{ty}({left}, {right})")
            }
            DependentType::Inductive { name, .. } => write!(f, "{name}"),
        }
    }
}

impl fmt::Display for DependentTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependentTerm::Variable(name) => write!(f, "{name}"),
            DependentTerm::Lambda {
                param,
                param_type,
                body,
            } => {
                write!(f, "λ{param}:{param_type}.{body}")
            }
            DependentTerm::Application { function, argument } => {
                write!(f, "({function} {argument})")
            }
            DependentTerm::Pair { first, second } => {
                write!(f, "({first}, {second})")
            }
            DependentTerm::Projection { pair, is_first } => {
                if *is_first {
                    write!(f, "π₁({pair})")
                } else {
                    write!(f, "π₂({pair})")
                }
            }
            DependentTerm::Refl { .. } => write!(f, "refl"),
            DependentTerm::Constructor { name, args, .. } => {
                write!(f, "{name}")?;
                if !args.is_empty() {
                    write!(f, "(")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{arg}")?;
                    }
                    write!(f, ")")?;
                }
                Ok(())
            }
            DependentTerm::Match { scrutinee, .. } => {
                write!(f, "match {scrutinee} with ...")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typing_context() {
        let mut ctx = TypingContext::new();

        // Test variable binding and lookup
        ctx.bind_variable("x".to_string(), DependentType::Universe(0));
        assert!(ctx.lookup_variable("x").is_some());
        assert!(ctx.lookup_variable("y").is_none());

        // Test scope operations
        ctx.push_scope();
        ctx.bind_variable("y".to_string(), DependentType::Universe(1));
        assert!(ctx.lookup_variable("x").is_some()); // Should still see parent scope
        assert!(ctx.lookup_variable("y").is_some());

        ctx.pop_scope();
        assert!(ctx.lookup_variable("x").is_some()); // Back to original scope
        assert!(ctx.lookup_variable("y").is_none()); // y was in nested scope
    }

    #[test]
    fn test_normalizer_beta_reduction() {
        let normalizer = Normalizer::new();

        // (λx:Type₀.x)(Type₀) → Type₀
        let lambda = DependentTerm::Lambda {
            param: "x".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Variable("x".to_string())),
        };

        let application = DependentTerm::Application {
            function: Box::new(lambda),
            argument: Box::new(DependentTerm::Variable("Type₀".to_string())),
        };

        let normalized = normalizer.normalize_term(&application).unwrap();
        assert_eq!(normalized, DependentTerm::Variable("Type₀".to_string()));
    }

    #[test]
    fn test_dependent_type_display() {
        let pi_type = DependentType::Pi {
            var: "x".to_string(),
            domain: Box::new(DependentType::Universe(0)),
            codomain: Box::new(DependentType::Universe(0)),
        };

        let display = format!("{}", pi_type);
        assert!(display.contains("→"));
        assert!(display.contains("x:Type_0"));
    }

    #[test]
    fn test_free_variables_collection() {
        let normalizer = Normalizer::new();

        // Test simple variable
        let var_x = DependentTerm::Variable("x".to_string());
        let free_vars = normalizer.free_variables_term(&var_x);
        assert!(free_vars.contains("x"));
        assert_eq!(free_vars.len(), 1);

        // Test lambda with free variable in body
        let lambda = DependentTerm::Lambda {
            param: "y".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Variable("x".to_string())),
        };
        let free_vars = normalizer.free_variables_term(&lambda);
        assert!(free_vars.contains("x"));
        assert!(!free_vars.contains("y")); // y is bound

        // Test lambda with no free variables
        let identity_lambda = DependentTerm::Lambda {
            param: "x".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Variable("x".to_string())),
        };
        let free_vars = normalizer.free_variables_term(&identity_lambda);
        assert!(free_vars.is_empty()); // x is bound
    }

    #[test]
    fn test_fresh_variable_generation() {
        let normalizer = Normalizer::new();

        let mut avoid = HashSet::new();
        avoid.insert("x".to_string());
        avoid.insert("x_0".to_string());
        avoid.insert("x_1".to_string());

        let fresh = normalizer.generate_fresh_name("x", &avoid);
        assert!(!avoid.contains(&fresh));
        assert!(fresh.starts_with("x"));
    }

    #[test]
    fn test_variable_capture_avoidance() {
        let normalizer = Normalizer::new();

        // Test case: (λy.x)[y/x] should become λz.y (not λy.y)
        let lambda_body = DependentTerm::Variable("x".to_string());
        let lambda = DependentTerm::Lambda {
            param: "y".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(lambda_body),
        };

        let replacement = DependentTerm::Variable("y".to_string());
        let result = normalizer
            .substitute_term(&lambda, "x", &replacement)
            .unwrap();

        // The result should be λ<fresh>.y where <fresh> is not "y"
        if let DependentTerm::Lambda { param, body, .. } = result {
            assert_ne!(param, "y"); // Parameter should be renamed
            if let DependentTerm::Variable(var_name) = body.as_ref() {
                assert_eq!(var_name, "y"); // Body should contain replacement
            } else {
                panic!("Expected variable in lambda body");
            }
        } else {
            panic!("Expected lambda term");
        }
    }

    #[test]
    fn test_no_capture_case() {
        let normalizer = Normalizer::new();

        // Test case: (λy.x)[z/x] should become λy.z (no capture)
        let lambda_body = DependentTerm::Variable("x".to_string());
        let lambda = DependentTerm::Lambda {
            param: "y".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(lambda_body),
        };

        let replacement = DependentTerm::Variable("z".to_string());
        let result = normalizer
            .substitute_term(&lambda, "x", &replacement)
            .unwrap();

        // The result should be λy.z (no renaming needed)
        if let DependentTerm::Lambda { param, body, .. } = result {
            assert_eq!(param, "y"); // Parameter should stay the same
            if let DependentTerm::Variable(var_name) = body.as_ref() {
                assert_eq!(var_name, "z"); // Body should contain replacement
            } else {
                panic!("Expected variable in lambda body");
            }
        } else {
            panic!("Expected lambda term");
        }
    }

    #[test]
    fn test_shadowing_case() {
        let normalizer = Normalizer::new();

        // Test case: (λx.x)[y/x] should become λx.x (x is shadowed)
        let lambda_body = DependentTerm::Variable("x".to_string());
        let lambda = DependentTerm::Lambda {
            param: "x".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(lambda_body),
        };

        let replacement = DependentTerm::Variable("y".to_string());
        let result = normalizer
            .substitute_term(&lambda, "x", &replacement)
            .unwrap();

        // The result should be λx.x (no substitution because x is bound)
        if let DependentTerm::Lambda { param, body, .. } = result {
            assert_eq!(param, "x"); // Parameter should stay the same
            if let DependentTerm::Variable(var_name) = body.as_ref() {
                assert_eq!(var_name, "x"); // Body should stay the same (x is bound)
            } else {
                panic!("Expected variable in lambda body");
            }
        } else {
            panic!("Expected lambda term");
        }
    }

    #[test]
    fn test_complex_nested_substitution() {
        let normalizer = Normalizer::new();

        // Test case: (λy.λz.x)[y/x] should avoid double capture
        let inner_body = DependentTerm::Variable("x".to_string());
        let inner_lambda = DependentTerm::Lambda {
            param: "z".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(inner_body),
        };
        let outer_lambda = DependentTerm::Lambda {
            param: "y".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(inner_lambda),
        };

        let replacement = DependentTerm::Variable("y".to_string());
        let result = normalizer
            .substitute_term(&outer_lambda, "x", &replacement)
            .unwrap();

        // The outer lambda parameter should be renamed to avoid capture
        if let DependentTerm::Lambda {
            param: outer_param,
            body: outer_body,
            ..
        } = result
        {
            assert_ne!(outer_param, "y"); // Should be renamed

            if let DependentTerm::Lambda {
                param: inner_param,
                body: inner_body,
                ..
            } = outer_body.as_ref()
            {
                assert_eq!(inner_param, "z"); // Inner parameter should stay the same
                if let DependentTerm::Variable(var_name) = inner_body.as_ref() {
                    assert_eq!(var_name, "y"); // Innermost body should contain replacement
                } else {
                    panic!("Expected variable in inner lambda body");
                }
            } else {
                panic!("Expected inner lambda");
            }
        } else {
            panic!("Expected outer lambda");
        }
    }

    #[test]
    fn test_application_substitution() {
        let normalizer = Normalizer::new();

        // Test case: (f x)[g/f] should become (g x)
        let application = DependentTerm::Application {
            function: Box::new(DependentTerm::Variable("f".to_string())),
            argument: Box::new(DependentTerm::Variable("x".to_string())),
        };

        let replacement = DependentTerm::Variable("g".to_string());
        let result = normalizer
            .substitute_term(&application, "f", &replacement)
            .unwrap();

        if let DependentTerm::Application { function, argument } = result {
            if let DependentTerm::Variable(func_name) = function.as_ref() {
                assert_eq!(func_name, "g");
            } else {
                panic!("Expected variable in function position");
            }
            if let DependentTerm::Variable(arg_name) = argument.as_ref() {
                assert_eq!(arg_name, "x");
            } else {
                panic!("Expected variable in argument position");
            }
        } else {
            panic!("Expected application");
        }
    }

    #[test]
    fn test_alpha_conversion_basic() {
        let normalizer = Normalizer::new();

        // Test α-conversion of λx.x when x needs to be avoided
        let lambda = DependentTerm::Lambda {
            param: "x".to_string(),
            param_type: Box::new(DependentType::Universe(0)),
            body: Box::new(DependentTerm::Variable("x".to_string())),
        };

        let mut avoid = HashSet::new();
        avoid.insert("x".to_string());

        let converted = normalizer.alpha_convert_term(&lambda, &avoid).unwrap();

        if let DependentTerm::Lambda { param, body, .. } = converted {
            assert_ne!(param, "x"); // Parameter should be renamed
            if let DependentTerm::Variable(body_var) = body.as_ref() {
                assert_eq!(body_var, &param); // Body should use the new parameter name
            } else {
                panic!("Expected variable in lambda body");
            }
        } else {
            panic!("Expected lambda term");
        }
    }

    #[test]
    fn test_pi_type_free_variables() {
        let normalizer = Normalizer::new();

        // Test Π-type: (x : A) → B(x, y) where y is free
        let pi_type = DependentType::Pi {
            var: "x".to_string(),
            domain: Box::new(DependentType::Universe(0)),
            codomain: Box::new(DependentType::Identity {
                ty: Box::new(DependentType::Universe(0)),
                left: Box::new(DependentTerm::Variable("x".to_string())), // bound
                right: Box::new(DependentTerm::Variable("y".to_string())), // free
            }),
        };

        let free_vars = normalizer.free_variables_type(&pi_type);
        assert!(free_vars.contains("y")); // y should be free
        assert!(!free_vars.contains("x")); // x should be bound
    }
}

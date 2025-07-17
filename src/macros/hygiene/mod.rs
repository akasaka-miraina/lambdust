//! Hygienic macro system module
//!
//! This module implements true hygienic macros for Lambdust, addressing symbol collision
//! issues identified in Scheme macro expansion. The design follows the principles outlined
//! in the referenced Japanese article on Scheme macro systems.

pub mod symbol;
pub mod environment;
pub mod context;
pub mod transformer;
pub mod renaming;
pub mod generator;

// Re-export key types for convenient access
pub use symbol::{HygienicSymbol, SymbolId, MacroSite, EnvironmentId};
pub use environment::HygienicEnvironment;
pub use context::ExpansionContext;
pub use transformer::{
    HygienicSyntaxRulesTransformer, OptimizationLevel, TransformerMetrics
};
pub use renaming::{
    RenamingStrategy, RenamingRule, SymbolRenamer, CustomRenamingRule, 
    RenamingPattern, PatternMatcher, RenamingAction, DefaultAction, 
    BuiltInPredicate, PredicateFunction
};
pub use generator::{
    SymbolGenerator, GenerationStrategy, UseCase, PerformanceStats, SymbolCache
};
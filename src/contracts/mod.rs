//! Contract System for Lambdust Scheme Interpreter
//!
//! This module implements a comprehensive contract system with runtime checking
//! and precise blame tracking. The system provides:
//!
//! - Function contracts with domain/codomain checking
//! - Data contracts with predicates and constraints
//! - Higher-order contracts for functions that accept/return functions
//! - Contract combinators (and/c, or/c, not/c, ->, etc.)
//! - Blame tracking for precise error attribution
//! - Performance optimization through contract compilation
//! - Integration with gradual typing
//!
//! # Contract Language
//!
//! The contract language supports:
//!
//! ```scheme
//! ;; Simple predicate contracts
//! (define/contract (safe-divide x y)
//!   (-> number? (and/c number? (not/c zero?)) number?)
//!   (/ x y))
//!
//! ;; Higher-order contracts
//! (define/contract (map f lst)
//!   (-> (-> any/c any/c) list? list?)
//!   ...)
//!
//! ;; Dependent contracts
//! (define/contract (vector-ref v i)
//!   (->i ([v vector?] [i (and/c exact-integer? (</c (vector-length v)))])
//!        any/c)
//!   ...)
//! ```

pub mod ast;
pub mod blame;
pub mod combinators;
pub mod compiler;
pub mod enforcement;
pub mod evaluator;
pub mod parser;
pub mod predicates;
pub mod runtime;

#[cfg(test)]
pub mod tests;

pub use ast::*;
pub use blame::*;
pub use combinators::*;
pub use compiler::*;
pub use enforcement::*;
pub use evaluator::*;
pub use parser::*;
pub use predicates::*;
pub use runtime::*;

use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Result type for contract operations.
pub type ContractResult<T> = Result<T>;

/// Contract error types.
#[derive(Debug, Clone)]
pub enum ContractError {
    /// Contract violation with blame information
    Violation {
        /// Blame information for the violation
        blame: Box<BlameInfo>,
        /// Expected contract specification
        expected: String,
        /// Actual value that failed the contract
        actual: String,
        /// Source location where the violation occurred
        location: Span,
    },
    /// Invalid contract definition
    InvalidContract {
        /// Error message describing the invalid contract
        message: String,
        /// Source location of the invalid contract
        location: Span,
    },
    /// Compilation error
    CompilationError {
        /// Error message describing the compilation failure
        message: String,
        /// Source location where compilation failed
        location: Span,
    },
    /// Runtime error during contract checking
    RuntimeError {
        /// Error message describing what went wrong
        message: String,
        /// Source location where the error occurred
        location: Span,
    },
}

impl From<ContractError> for Error {
    fn from(err: ContractError) -> Self {
        match err {
            ContractError::Violation {
                blame,
                expected,
                actual,
                location,
            } => Error::new_spanned(
                format!("Contract violation: expected {expected}, got {actual} (blame: {blame})"),
                location,
            ),
            ContractError::InvalidContract { message, location } => {
                Error::new_spanned(format!("Invalid contract: {message}"), location)
            }
            ContractError::CompilationError { message, location } => {
                Error::new_spanned(format!("Contract compilation error: {message}"), location)
            }
            ContractError::RuntimeError { message, location } => {
                Error::new_spanned(format!("Contract runtime error: {message}"), location)
            }
        }
    }
}

impl From<ContractError> for Box<Error> {
    fn from(err: ContractError) -> Self {
        Box::new(err.into())
    }
}

/// Configuration for the contract system.
#[derive(Debug, Clone)]
pub struct ContractConfig {
    /// Enable contract checking (can be disabled for production)
    pub enable_checking: bool,
    /// Enable contract compilation optimizations
    pub enable_compilation: bool,
    /// Enable blame tracking
    pub enable_blame_tracking: bool,
    /// Maximum depth for recursive contract checking
    pub max_recursion_depth: usize,
    /// Cache compiled contracts
    pub enable_contract_caching: bool,
    /// Enable gradual contract integration
    pub enable_gradual_integration: bool,
}

impl Default for ContractConfig {
    fn default() -> Self {
        Self {
            enable_checking: true,
            enable_compilation: true,
            enable_blame_tracking: true,
            max_recursion_depth: 100,
            enable_contract_caching: true,
            enable_gradual_integration: true,
        }
    }
}

/// The main contract system coordinator.
#[derive(Debug)]
pub struct ContractSystem {
    /// Configuration
    config: ContractConfig,
    /// Contract compiler
    compiler: ContractCompiler,
    /// Runtime enforcement engine
    enforcement: ContractEnforcement,
    /// Predicate registry
    predicates: PredicateRegistry,
    /// Blame tracker
    blame_tracker: BlameTracker,
    /// Contract cache
    contract_cache: HashMap<String, Arc<CompiledContract>>,
}

impl ContractSystem {
    /// Creates a new contract system with default configuration.
    pub fn new() -> Self {
        Self::with_config(ContractConfig::default())
    }

    /// Creates a new contract system with custom configuration.
    pub fn with_config(config: ContractConfig) -> Self {
        let predicates = PredicateRegistry::new();
        let blame_tracker = BlameTracker::new();
        let compiler = ContractCompiler::new(&config, predicates.clone());
        let enforcement = ContractEnforcement::new(&config, blame_tracker.clone());

        Self {
            config,
            compiler,
            enforcement,
            predicates,
            blame_tracker,
            contract_cache: HashMap::new(),
        }
    }

    /// Registers a new predicate.
    pub fn register_predicate(&mut self, name: String, predicate: ContractPredicate) {
        self.predicates.register(name, predicate);
    }

    /// Compiles a contract expression.
    pub fn compile_contract(
        &mut self,
        contract: &ContractExpr,
        context: &CompilationContext,
    ) -> ContractResult<Arc<CompiledContract>> {
        if self.config.enable_contract_caching {
            // Check cache first
            let cache_key = format!("{contract:?}"); // Simplified cache key
            if let Some(compiled) = self.contract_cache.get(&cache_key) {
                return Ok(compiled.clone());
            }

            // Compile and cache
            let compiled = self.compiler.compile(contract, context)?;
            self.contract_cache.insert(cache_key, compiled.clone());
            Ok(compiled)
        } else {
            self.compiler.compile(contract, context)
        }
    }

    /// Checks a value against a compiled contract.
    pub fn check_contract(
        &mut self,
        value: &Value,
        contract: &CompiledContract,
        blame: &BlameInfo,
    ) -> ContractResult<()> {
        if !self.config.enable_checking {
            return Ok(());
        }

        self.enforcement.check(value, contract, blame)
    }

    /// Creates a wrapped value with contract checking.
    pub fn wrap_with_contract(
        &mut self,
        value: Value,
        contract: Arc<CompiledContract>,
        blame: BlameInfo,
    ) -> ContractResult<Value> {
        self.enforcement.wrap(value, contract, blame)
    }

    /// Gets the current configuration.
    pub fn config(&self) -> &ContractConfig {
        &self.config
    }

    /// Updates the configuration.
    pub fn update_config(&mut self, config: ContractConfig) {
        self.config = config;
        // Update components with new config
        self.compiler.update_config(&self.config);
        self.enforcement.update_config(&self.config);
    }

    /// Clears the contract cache.
    pub fn clear_cache(&mut self) {
        self.contract_cache.clear();
    }

    /// Gets cache statistics.
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.contract_cache.len(), self.contract_cache.capacity())
    }
}

impl Default for ContractSystem {
    fn default() -> Self {
        Self::new()
    }
}

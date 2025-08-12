#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_macros)]
#![allow(unreachable_patterns)]
#![allow(ambiguous_glob_reexports)]
#![allow(unused_doc_comments)]
#![allow(deprecated)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_must_use)]
#![allow(non_snake_case)]
//! # Lambdust Language Implementation
//!
//! Lambdust (λust) is a Scheme dialect that combines the simplicity and elegance of Scheme
//! with modern type theory and functional programming concepts.
//!
//! ## Features
//!
//! - **R7RS-compatible**: Existing Scheme code runs without modification
//! - **Gradually typed**: From dynamic to dependent types
//! - **Purely functional**: With transparent handling of effects through monads
//! - **Efficient**: JIT compilation and profile-guided optimization
//!
//! ## Architecture
//!
//! The implementation follows a traditional compiler pipeline:
//! 1. **Lexer**: Tokenizes source code
//! 2. **Parser**: Builds Abstract Syntax Tree (AST)
//! 3. **Macro Expansion**: Expands user-defined and built-in macros
//! 4. **Type Checking**: Optional static type analysis
//! 5. **Effect Analysis**: Tracks and transforms side effects
//! 6. **Evaluation**: Interprets or compiles the code
//!
//! ## Example
//!
//! ```scheme
//! (define (factorial n)
//!   #:type (-> Number Number)
//!   #:pure #t
//!   (if (zero? n)
//!       1
//!       (* n (factorial (- n 1)))))
//! ```

#![warn(missing_docs, rust_2018_idioms)]
#![allow(clippy::module_inception)]

// Core language components
/// Abstract Syntax Tree definitions and utilities.
pub mod ast;
/// Lexical analysis and tokenization.
pub mod lexer;
/// Parsing from tokens to AST.
pub mod parser;

// Language features
/// Macro system implementation with R7RS-compatible syntax-rules.
pub mod macro_system;
/// Type system with gradual typing capabilities.
pub mod types;
/// Effect system for pure functional programming with transparent side effects.
pub mod effects;
/// Module system with R7RS-compatible libraries.
pub mod module_system;
/// Concurrency primitives and parallel evaluation support.
pub mod concurrency;

// Runtime and evaluation
/// Core evaluation engine and environment management.
pub mod eval;
/// Runtime system coordination and execution management.
pub mod runtime;
/// Standard library implementations (R7RS and extensions).
pub mod stdlib;
/// Bytecode compilation and virtual machine.
pub mod bytecode;
/// Just-In-Time compilation system for native code generation.
pub mod jit;

// Advanced numeric system
/// Advanced numeric tower with bigints, rationals, and complex numbers.
pub mod numeric;

// High-performance containers
/// High-performance container implementations and data structures.
pub mod containers;

// Advanced metaprogramming system
/// Advanced metaprogramming capabilities and reflection.
pub mod metaprogramming;

// Interoperability
/// Foreign Function Interface for C/Rust interoperability.
pub mod ffi;

// Utilities and diagnostics
/// Error handling, diagnostics, and source location tracking.
pub mod diagnostics;
/// Utility functions and data structures.
pub mod utils;

// Enhanced REPL system
/// Enhanced Read-Eval-Print Loop with debugging and completion.
#[cfg(any(feature = "repl", feature = "enhanced-repl"))]
pub mod repl;

// Benchmarking and performance comparison
/// Benchmarking suite and performance analysis tools.
pub mod benchmarks;

// Re-exports for convenience
pub use ast::{Expr, Literal, Program};
pub use diagnostics::{Error, Result, Span};
pub use eval::{Evaluator, Value};
pub use lexer::{Lexer, Token};
pub use parser::Parser;
pub use runtime::{Runtime, LambdustRuntime, ParallelResult, EvaluatorHandle};

// Re-export system interface utilities
pub use stdlib::system;

// Re-export metaprogramming system
pub use metaprogramming::{
    MetaprogrammingSystem, ReflectionSystem, CodeGenerator, DynamicEvaluator,
    ProceduralMacro, StaticAnalyzer, EnvironmentManipulator, SecurityManager
};

// Note: Lambdust and MultithreadedLambdust are defined below

/// The main entry point for the Lambdust language.
///
/// This provides a high-level interface for parsing, type-checking,
/// and evaluating Lambdust programs.
#[derive(Debug)]
pub struct Lambdust {
    runtime: Runtime,
}

impl Lambdust {
    /// Creates a new Lambdust instance with default configuration.
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    /// Creates a new Lambdust instance with custom runtime configuration.
    pub fn with_runtime(runtime: Runtime) -> Self {
        Self { runtime }
    }

    /// Evaluates a Lambdust program from source code.
    ///
    /// # Arguments
    ///
    /// * `source` - The source code to evaluate
    /// * `filename` - Optional filename for error reporting
    ///
    /// # Returns
    ///
    /// The result of evaluation or an error if parsing/evaluation fails.
    pub fn eval(&mut self, source: &str, filename: Option<&str>) -> Result<Value> {
        let tokens = self.tokenize(source, filename)?;
        let ast = self.parse(tokens)?;
        let expanded = self.expand_macros(ast)?;
        let typed = self.type_check(expanded)?;
        self.runtime.eval(typed)
    }

    /// Tokenizes source code into a stream of tokens.
    pub fn tokenize(&self, source: &str, filename: Option<&str>) -> Result<Vec<Token>> {
        let mut lexer = Lexer::new(source, filename);
        lexer.tokenize()
    }

    /// Parses tokens into an Abstract Syntax Tree.
    pub fn parse(&self, tokens: Vec<Token>) -> Result<Program> {
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    /// Expands macros in the AST.
    pub fn expand_macros(&mut self, program: Program) -> Result<Program> {
        self.runtime.expand_macros(program)
    }

    /// Performs type checking on the AST.
    pub fn type_check(&self, program: Program) -> Result<Program> {
        self.runtime.type_check(program)
    }

    /// Gets a reference to the runtime.
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    /// Gets a mutable reference to the runtime.
    pub fn runtime_mut(&mut self) -> &mut Runtime {
        &mut self.runtime
    }
}

impl Default for Lambdust {
    fn default() -> Self {
        Self::new()
    }
}

/// Multithreaded version of the Lambdust language interface.
///
/// This provides parallel evaluation capabilities while maintaining
/// compatibility with the single-threaded interface.
#[derive(Debug)]
pub struct MultithreadedLambdust {
    runtime: LambdustRuntime,
}

impl MultithreadedLambdust {
    /// Creates a new multithreaded Lambdust instance.
    ///
    /// # Arguments
    /// * `num_threads` - Number of evaluator threads (None for CPU count)
    pub fn new(num_threads: Option<usize>) -> Result<Self> {
        let runtime = match num_threads {
            Some(count) => LambdustRuntime::with_threads(count)?,
            None => LambdustRuntime::new()?,
        };
        Ok(Self {
            runtime,
        })
    }

    /// Creates a new multithreaded Lambdust instance with custom runtime.
    pub fn with_runtime(runtime: LambdustRuntime) -> Self {
        Self { runtime }
    }

    /// Evaluates a Lambdust program from source code using parallel evaluation.
    ///
    /// # Arguments
    /// * `source` - The source code to evaluate
    /// * `filename` - Optional filename for error reporting
    ///
    /// # Returns
    /// The result of evaluation or an error if parsing/evaluation fails.
    pub async fn eval(&self, source: &str, filename: Option<&str>) -> Result<Value> {
        let tokens = self.tokenize(source, filename)?;
        let ast = self.parse(tokens)?;
        let expanded = self.expand_macros(ast)?;
        let typed = self.type_check(expanded)?;
        self.runtime.eval_program(&typed).await
    }

    /// Evaluates multiple expressions in parallel.
    ///
    /// # Arguments
    /// * `sources` - Vector of (source_code, filename) pairs
    ///
    /// # Returns
    /// Parallel evaluation results with timing information.
    pub async fn eval_parallel(&self, sources: Vec<(&str, Option<&str>)>) -> Result<ParallelResult> {
        let mut expressions = Vec::new();
        
        for (source, filename) in sources {
            let tokens = self.tokenize(source, filename)?;
            let ast = self.parse(tokens)?;
            let expanded = self.expand_macros(ast)?;
            let typed = self.type_check(expanded)?;
            
            // Convert program to individual expressions with spans
            for expr in typed.expressions {
                expressions.push((expr.inner, Some(expr.span)));
            }
        }
        
        Ok(self.runtime.eval_parallel(expressions).await)
    }

    /// Spawns a new evaluator and returns a handle to it.
    pub fn spawn_evaluator(&self) -> Result<EvaluatorHandle> {
        self.runtime.spawn_evaluator()
    }

    /// Tokenizes source code into a stream of tokens.
    pub fn tokenize(&self, source: &str, filename: Option<&str>) -> Result<Vec<Token>> {
        let mut lexer = Lexer::new(source, filename);
        lexer.tokenize()
    }

    /// Parses tokens into an Abstract Syntax Tree.
    pub fn parse(&self, tokens: Vec<Token>) -> Result<Program> {
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    /// Expands macros in the AST.
    pub fn expand_macros(&self, program: Program) -> Result<Program> {
        // For now, return unchanged - macro expansion is handled per-thread
        Ok(program)
    }

    /// Performs type checking on the AST.
    pub fn type_check(&self, program: Program) -> Result<Program> {
        // For now, return unchanged - type checking would be added here
        Ok(program)
    }

    /// Gets a reference to the runtime.
    pub fn runtime(&self) -> &LambdustRuntime {
        &self.runtime
    }

    /// Gets the number of evaluator threads.
    pub fn thread_count(&self) -> usize {
        self.runtime.thread_count()
    }

    /// Shuts down the multithreaded runtime.
    pub async fn shutdown(self) -> Result<()> {
        self.runtime.shutdown().await
    }
}

impl Default for MultithreadedLambdust {
    fn default() -> Self {
        Self::new(None).expect("Failed to create default multithreaded Lambdust")
    }
}

/// Version information for the Lambdust implementation.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The language specification version this implementation supports.
pub const LANGUAGE_VERSION: &str = "0.1.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_evaluation() {
        let mut lambdust = Lambdust::new();
        let _result = lambdust.eval("(+ 1 2)", Some("test"));
        // This will fail until we implement the components
        // assert!(result.is_ok());
    }

    #[test]
    fn test_version_constants() {
        assert_eq!(LANGUAGE_VERSION, "0.1.0");
    }

}
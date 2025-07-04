//! # Lambdust (λust) - Rust Scheme Interpreter
//!
//! Lambdust is a complete R7RS Scheme interpreter implemented in Rust, designed
//! for embedding in applications as a macro and scripting system. The name
//! combines "lambda" (λ) with "Rust," reflecting Scheme's functional nature
//! and the ability to add expressive power to existing applications.
//!
//! Pre-commit hooks ensure code quality through automated testing.
//!
//! ## Features
//!
//! - **R7RS Compliance**: Implements the R7RS Small language specification
//! - **Embedded Design**: Designed for integration into Rust applications
//! - **Macro System**: Full support for Scheme's powerful macro system
//! - **Type Safety**: Leverages Rust's type system for memory safety
//! - **Bridge API**: Seamless interoperability between Rust and Scheme
//! - **Performance**: Optimized for speed with tail-call optimization
//!
//! ## Quick Start
//!
//! ```rust
//! use lambdust::Interpreter;
//!
//! let mut interpreter = Interpreter::new();
//! let result = interpreter.eval("(+ 1 2 3)").unwrap();
//! println!("Result: {}", result); // Prints: Result: 6
//! ```
//!
//! ## Bridge API
//!
//! For advanced integration with external applications:
//!
//! ```rust
//! use lambdust::{LambdustBridge, Value, FromScheme, ToScheme};
//!
//! let mut bridge = LambdustBridge::new();
//!
//! // Register external function
//! bridge.register_function("square", Some(1), |args| {
//!     let n = i64::from_scheme(&args[0])?;
//!     (n * n).to_scheme()
//! });
//!
//! // Define variables
//! bridge.define("pi", Value::from(3.14159));
//!
//! // Execute Scheme code
//! let result = bridge.eval("(+ 1 2)").unwrap();
//! ```
//!
//! ## Architecture
//!
//! The interpreter consists of several key components:
//!
//! - **Lexer**: Tokenizes Scheme source code
//! - **Parser**: Builds Abstract Syntax Trees (AST) from tokens
//! - **Evaluator**: Executes Scheme expressions with proper semantics
//! - **Environment**: Manages variable bindings and lexical scoping
//! - **Macro System**: Handles macro expansion and transformation
//! - **Bridge**: Provides interoperability with external Rust code
//!
//! ## Supported Scheme Features
//!
//! - All basic data types (numbers, strings, symbols, lists, etc.)
//! - Special forms (`define`, `lambda`, `if`, `cond`, `let`, etc.)
//! - First-class procedures and closures
//! - Tail-call optimization
//! - Macro system with `syntax-rules`
//! - Proper lexical scoping
//! - Built-in procedures for list manipulation, arithmetic, etc.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod ast;
pub mod bridge;
pub mod builtins;
pub mod environment;
pub mod error;
pub mod evaluator;
pub mod host;
pub mod interpreter;
pub mod lexer;
pub mod macros;
pub mod marshal;
pub mod module_system;
pub mod parser;
pub mod srfi;
pub mod value;

// REPL module will be implemented in future versions
// #[cfg(feature = "repl")]
// pub mod repl;

pub use bridge::{Callable, FromScheme, LambdustBridge, ToScheme};
pub use error::{LambdustError, Result};
pub use evaluator::{Evaluator, FormalEvaluator};
pub use module_system::ModuleSystem;
pub use srfi::SrfiRegistry;
pub use value::Value;

/// The main interpreter struct that provides the public API
///
/// This is the primary interface for basic Scheme interpretation. For advanced
/// features like external function registration and object integration, use
/// [`LambdustBridge`] instead.
///
/// # Examples
///
/// ```rust
/// use lambdust::Interpreter;
///
/// let mut interpreter = Interpreter::new();
///
/// // Evaluate basic expressions
/// let result = interpreter.eval("(+ 1 2 3)").unwrap();
/// assert_eq!(result.to_string(), "6");
///
/// // Define variables and functions
/// interpreter.eval("(define square (lambda (x) (* x x)))").unwrap();
/// let result = interpreter.eval("(square 5)").unwrap();
/// assert_eq!(result.to_string(), "25");
/// ```
pub struct Interpreter {
    evaluator: Evaluator,
}

impl Interpreter {
    /// Create a new interpreter instance
    ///
    /// Creates a new Scheme interpreter with a fresh global environment
    /// containing all standard built-in procedures and special forms.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambdust::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let result = interpreter.eval("(+ 1 2)").unwrap();
    /// ```
    pub fn new() -> Self {
        Self {
            evaluator: Evaluator::new(),
        }
    }

    /// Evaluate a Scheme expression from a string
    ///
    /// Parses and evaluates the given Scheme expression string, returning
    /// the result as a [`Value`].
    ///
    /// # Arguments
    ///
    /// * `input` - A string containing valid Scheme code
    ///
    /// # Returns
    ///
    /// Returns `Ok(Value)` containing the result of evaluation, or an error
    /// if parsing or evaluation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambdust::{Interpreter, Value};
    ///
    /// let mut interpreter = Interpreter::new();
    ///
    /// // Arithmetic
    /// let result = interpreter.eval("(* 6 7)").unwrap();
    /// assert_eq!(result, Value::from(42i64));
    ///
    /// // List operations
    /// let result = interpreter.eval("(length '(1 2 3 4))").unwrap();
    /// assert_eq!(result, Value::from(4i64));
    ///
    /// // Function definition and call
    /// interpreter.eval("(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))").unwrap();
    /// let result = interpreter.eval("(factorial 5)").unwrap();
    /// assert_eq!(result, Value::from(120i64));
    /// ```
    pub fn eval(&mut self, input: &str) -> Result<Value> {
        self.evaluator.eval_string(input)
    }

    /// Load and evaluate a Scheme file
    ///
    /// Reads the contents of the specified file and evaluates it as Scheme code.
    /// This is equivalent to reading the file and calling [`eval`](Self::eval)
    /// with its contents.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Scheme source file
    ///
    /// # Returns
    ///
    /// Returns `Ok(Value)` containing the result of evaluating the file,
    /// or an error if the file cannot be read or contains invalid Scheme code.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use lambdust::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let result = interpreter.load_file("script.scm").unwrap();
    /// ```
    pub fn load_file(&mut self, path: &str) -> Result<Value> {
        let content =
            std::fs::read_to_string(path).map_err(|e| LambdustError::io_error(e.to_string()))?;
        self.eval(&content)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}


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
//! Lambdust follows a modular architecture with specialized components:
//!
//! - **`evaluator/`**: CPS evaluator with 7 specialized modules for R7RS compliance
//! - **`builtins/`**: 13 organized builtin function modules (103+ functions)
//! - **`srfi/`**: Comprehensive SRFI library implementations (9 major SRFIs)
//! - **`value/`**: Unified value system with optimized representations
//! - **`bridge/`**: Type-safe Rust-Scheme interoperability layer
//! - **`optimization/`**: JIT compilation and formal verification framework
//! - **`lexer/parser`**: Robust tokenization and AST construction
//! - **`environment/`**: Advanced variable binding and lexical scoping
//! - **`macros/`**: Complete macro expansion and transformation system
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
#![allow(clippy::crate_in_macro_def)]
#![allow(clippy::writeln_empty_string)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::inherent_to_string)]

// ===== Core Modules (Always Included) =====
pub mod ast;
pub mod error;
#[cfg(test)]
pub mod error_tests;
pub mod lexer;
pub mod parser;

// Environment system
#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub mod environment;

// ===== Evaluator Selection =====
#[cfg(feature = "embedded")]
pub mod embedded_evaluator;

#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub mod evaluator;

// ===== Value System =====
#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub mod value;

// ===== Basic Modules (Minimal Configuration) =====
#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub mod bridge;

#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub mod builtins;

#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub mod host;

#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub mod interpreter;

#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub mod macros;

#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub mod marshal;

// ===== Feature-Gated Modules =====

// SRFI Support (Not available in embedded mode)
#[cfg(all(
    feature = "srfi-support",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub mod srfi;

#[cfg(all(
    not(feature = "srfi-support"),
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub mod srfi {
    //! Stub SRFI module for minimal builds
    #[derive(Debug)]
    pub struct SrfiRegistry;

    impl SrfiRegistry {
        pub fn new() -> Self {
            Self
        }
        pub fn with_standard_srfis() -> Self {
            Self
        }
        pub fn register_srfi(&mut self, _id: u32) {}
        pub fn has_srfi(&self, _id: u32) -> bool {
            false
        }
        pub fn available_srfis(&self) -> Vec<u32> {
            Vec::new()
        }
        pub fn get_srfi_info(&self, _id: u32) -> Option<(u32, String, Vec<String>)> {
            None
        }
        pub fn get_exports_for_parts(
            &mut self,
            _srfi_number: u32,
            _parts: &[&str],
        ) -> Result<Vec<(String, crate::value::Value)>, crate::error::LambdustError> {
            Ok(Vec::new())
        }
    }

    // Stub SRFI modules
    pub mod srfi_1 {
        use crate::value::Value;
        use std::collections::HashMap;
        pub fn register_srfi_1_functions(_builtins: &mut HashMap<String, Value>) {
            // No-op in minimal build
        }
    }

    pub mod srfi_13 {
        use crate::value::Value;
        use std::collections::HashMap;
        pub fn register_srfi_13_functions(_builtins: &mut HashMap<String, Value>) {
            // No-op in minimal build
        }
    }

    pub mod srfi_69 {
        use crate::value::Value;
        use std::collections::HashMap;

        #[derive(Debug, PartialEq, Clone)]
        pub struct HashTable {
            pub table: HashMap<String, Value>,
        }

        impl HashTable {
            pub fn new() -> Self {
                Self {
                    table: HashMap::new(),
                }
            }

            pub fn size(&self) -> usize {
                self.table.len()
            }
        }

        pub fn register_srfi_69_functions(_builtins: &mut HashMap<String, Value>) {
            // No-op in minimal build
        }
    }

    pub mod srfi_111 {
        use crate::value::Value;

        #[derive(Debug, PartialEq, Clone)]
        pub struct Box;

        impl Box {
            pub fn unbox(&self) -> Value {
                Value::Undefined
            }
        }
    }

    pub mod srfi_128 {
        #[derive(Debug, PartialEq, Clone)]
        pub struct Comparator {
            pub name: String,
        }

        impl Comparator {
            pub fn new() -> Self {
                Self {
                    name: "stub-comparator".to_string(),
                }
            }
        }
    }

    pub mod srfi_130 {
        #[derive(Debug, PartialEq, Clone)]
        pub struct StringCursor;

        impl StringCursor {
            pub fn position(&self) -> usize {
                0
            }
            pub fn string(&self) -> String {
                String::new()
            }
        }
    }

    pub mod srfi_134 {
        use crate::value::Value;

        #[derive(Debug, PartialEq, Clone)]
        pub struct Ideque;

        impl Ideque {
            pub fn len(&self) -> usize {
                0
            }
            pub fn to_list(&self) -> Vec<Value> {
                Vec::new()
            }
        }
    }

    pub mod srfi_135 {
        #[derive(Debug, PartialEq, Clone)]
        pub struct Text;

        impl Text {
            pub fn length(&self) -> usize {
                0
            }
            pub fn text_to_string(&self) -> String {
                String::new()
            }
            pub fn text_equal(&self, _other: &Text) -> bool {
                true
            }
        }
    }

    pub mod srfi_136 {
        #[derive(Debug, PartialEq, Clone)]
        pub struct RecordTypeDescriptor;
    }

    pub mod srfi_140 {
        #[derive(Debug, PartialEq, Clone)]
        pub struct IString;

        impl IString {
            pub fn length(&self) -> usize {
                0
            }
        }

        impl ToString for IString {
            fn to_string(&self) -> String {
                String::new()
            }
        }
    }
}

// Basic Optimization (Not available in embedded mode)
#[cfg(all(
    feature = "basic-optimization",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub mod cps_inlining;

#[cfg(all(
    feature = "basic-optimization",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub mod optimized_collections;

// Memory Management (Not available in embedded mode)
#[cfg(all(
    feature = "memory-pooling",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub mod adaptive_memory;

#[cfg(all(
    feature = "memory-pooling",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub mod memory_pool;

// Advanced Features (Not available in embedded mode)
#[cfg(all(
    feature = "theorem-derivation",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub mod optimization;

#[cfg(all(
    feature = "runtime-verification",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub mod module_system;

// Development Tools (Not available in embedded mode)
#[cfg(all(
    feature = "debug-tracing",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub mod debug;

#[cfg(all(
    not(feature = "debug-tracing"),
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub mod debug {
    //! Stub debug module for minimal builds
    /// Stub debug tracer for non-debug builds
    pub struct DebugTracer;

    /// Trace level for debug messages
    pub enum TraceLevel {
        /// Informational messages
        INFO,
        /// Function entry
        ENTRY,
        /// Function exit
        EXIT,
        /// Error messages
        ERROR,
    }

    impl DebugTracer {
        /// Trace a debug message
        pub fn trace(
            _module: &str,
            _function: &str,
            _line: u32,
            _level: TraceLevel,
            _message: String,
        ) {
        }
        /// Trace an expression
        pub fn trace_expr(
            _module: &str,
            _function: &str,
            _line: u32,
            _level: TraceLevel,
            _message: String,
            _expr: &crate::ast::Expr,
        ) {
        }
        /// Trace a value
        pub fn trace_value(
            _module: &str,
            _function: &str,
            _line: u32,
            _level: TraceLevel,
            _message: String,
            _value: &crate::value::Value,
        ) {
        }
        /// Trace a continuation
        pub fn trace_continuation(
            _module: &str,
            _function: &str,
            _line: u32,
            _level: TraceLevel,
            _message: String,
            _name: &str,
            _depth: Option<usize>,
        ) {
        }
    }
}

#[cfg(all(
    feature = "debug-tracing",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub mod stack_monitor;

// Platform-Specific
#[cfg(feature = "wasm")]
pub mod ffi;

#[cfg(feature = "wasm")]
pub mod ffi_enhanced;
#[cfg(any(feature = "wasm", feature = "wasi"))]
pub mod wasm;

// REPL module will be implemented in future versions
// #[cfg(feature = "repl")]
// pub mod repl;

// ===== Core Exports =====
pub use error::{LambdustError, Result};

// Standard API (not available in embedded mode)
#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub use bridge::{Callable, FromScheme, LambdustBridge, ToScheme};

#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub use evaluator::{eval_with_formal_semantics, Evaluator};

#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub use interpreter::LambdustInterpreter;

#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub use value::Value;

// Embedded API (only available in embedded mode)
#[cfg(feature = "embedded")]
pub use embedded_evaluator::{EmbeddedEnvironment, EmbeddedEvaluator, EmbeddedValue};

// ===== Feature-Gated Exports =====

// Memory Management (Not available in embedded mode)
#[cfg(all(
    feature = "memory-pooling",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub use adaptive_memory::{AdaptiveMemoryManager, AllocationStrategy, MemoryPressure};

#[cfg(all(
    feature = "memory-pooling",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub use memory_pool::{ContinuationPool, ContinuationPoolStats, PoolStats, ValuePool};

// Basic Optimization (Not available in embedded mode)
#[cfg(all(
    feature = "basic-optimization",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub use cps_inlining::{ChainStrategy, CpsInliner, InliningDecision};

#[cfg(all(
    feature = "basic-optimization",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub use optimized_collections::{ArgVec, CowVec, ExprVec, SliceRef};

// SRFI Support (Not available in embedded mode)
#[cfg(all(
    feature = "srfi-support",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub use srfi::SrfiRegistry;

// Advanced Features (Not available in embedded mode)
#[cfg(all(
    feature = "runtime-verification",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub use module_system::ModuleSystem;

// Development Tools (Not available in embedded mode)
#[cfg(all(
    feature = "debug-tracing",
    any(feature = "standard", feature = "minimal", not(feature = "embedded"))
))]
pub use stack_monitor::{OptimizationRecommendation, StackFrameType, StackMonitor};

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
#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
pub struct Interpreter {
    evaluator: Evaluator,
}

/// Ultra-lightweight embedded interpreter for no-std environments
///
/// This interpreter provides minimal Scheme functionality with extremely
/// small binary size, suitable for embedded systems and macro use.
///
/// # Examples
///
/// ```rust
/// use lambdust::EmbeddedInterpreter;
///
/// let mut interpreter = EmbeddedInterpreter::new();
///
/// // Basic arithmetic
/// let result = interpreter.eval("(+ 1 2)").unwrap();
///
/// // Simple conditionals
/// let result = interpreter.eval("(if (> 5 3) 10 20)").unwrap();
///
/// // Lambda functions
/// let result = interpreter.eval("((lambda (x) (* x x)) 5)").unwrap();
/// ```
#[cfg(feature = "embedded")]
pub struct EmbeddedInterpreter {
    evaluator: EmbeddedEvaluator,
}

#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
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

#[cfg(any(feature = "standard", feature = "minimal", not(feature = "embedded")))]
impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "embedded")]
impl EmbeddedInterpreter {
    /// Create a new embedded interpreter instance
    ///
    /// Creates a new ultra-lightweight Scheme interpreter with minimal
    /// built-in functions for embedded use.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambdust::EmbeddedInterpreter;
    ///
    /// let mut interpreter = EmbeddedInterpreter::new();
    /// let result = interpreter.eval("(+ 1 2)").unwrap();
    /// ```
    pub fn new() -> Self {
        Self {
            evaluator: EmbeddedEvaluator::new(),
        }
    }

    /// Evaluate a Scheme expression from a string
    ///
    /// Parses and evaluates the given Scheme expression string, returning
    /// the result as an [`EmbeddedValue`].
    ///
    /// # Arguments
    ///
    /// * `input` - A string containing valid Scheme code
    ///
    /// # Returns
    ///
    /// Returns `Ok(EmbeddedValue)` containing the result of evaluation, or an error
    /// if parsing or evaluation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambdust::EmbeddedInterpreter;
    ///
    /// let mut interpreter = EmbeddedInterpreter::new();
    ///
    /// // Arithmetic
    /// let result = interpreter.eval("(* 6 7)").unwrap();
    ///
    /// // Conditionals
    /// let result = interpreter.eval("(if (> 10 5) 'yes 'no)").unwrap();
    ///
    /// // Lambda functions
    /// let result = interpreter.eval("((lambda (x) (* x x)) 5)").unwrap();
    /// ```
    pub fn eval(&mut self, input: &str) -> Result<EmbeddedValue> {
        let tokens = crate::lexer::tokenize(input)?;
        let expr = crate::parser::parse(tokens)?;
        self.evaluator.eval(&expr)
    }
}

#[cfg(feature = "embedded")]
impl Default for EmbeddedInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

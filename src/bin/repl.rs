//! Lambdust REPL - Interactive Scheme interpreter
//!
//! This module provides a read-eval-print loop for the Lambdust Scheme interpreter.
//! It supports interactive evaluation, command history, tab completion, syntax highlighting,
//! and basic debugging features.

mod repl_modules;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    repl_modules::cli::main()
}

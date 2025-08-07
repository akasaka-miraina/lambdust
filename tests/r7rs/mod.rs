//! R7RS compliance test modules
//!
//! This module organizes R7RS-small standard compliance tests into
//! focused test suites for different language features.
//!
//! The test modules are organized by R7RS specification sections:
//! - Section 4: Program structure and definition contexts
//! - Section 5: Syntax forms and derived expressions  
//! - Section 6: Standard procedures (6.1-6.14)
//! - Section 7: Standard libraries

// Core language structure and syntax
pub mod program_structure;       // Section 4: Program structure
pub mod syntax_forms;           // Section 5: Syntax definitions

// Basic data types and predicates (Section 6.1-6.8)
pub mod basic_data_types;       // Section 6.1, 6.3-6.6: Basic predicates
pub mod numeric_operations;     // Section 6.2: Numbers
pub mod string_operations;      // Section 6.7: Strings
pub mod list_operations;        // Section 6.4: Pairs and lists
pub mod vector_operations;      // Section 6.8: Vectors

// Control flow and procedures (Section 6.10)
pub mod control_structures;     // Control features

// I/O and system interaction (Section 6.13-6.14)
pub mod io_operations;          // Section 6.13: Input/output

// Advanced features
pub mod macro_system;           // Section 4.3: Macros
pub mod exception_handling;     // Section 6.11: Exceptions

// Additional comprehensive tests
pub mod equivalence_predicates; // Comprehensive eq?, eqv?, equal? tests
pub mod bytevector_operations;  // Section 6.9: Bytevectors
pub mod library_system;         // Section 7: Standard libraries
pub mod environment_evaluation; // Section 6.12: Environments and evaluation

// Comprehensive test modules for complete R7RS coverage
pub mod comprehensive_numeric_tests; // Exhaustive numeric system tests
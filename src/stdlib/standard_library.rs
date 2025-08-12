//! Standard library implementation for Lambdust.

use crate::eval::value::{Value, PrimitiveProcedure, PrimitiveImpl, ThreadSafeEnvironment};
use crate::effects::Effect;
use std::sync::Arc;

/// Standard library functions and procedures.
pub struct StandardLibrary {
    // Placeholder for stdlib implementation
}

impl StandardLibrary {
    /// Creates a new standard library instance.
    pub fn new() -> Self {
        Self {}
    }
    
    /// Populates an environment with all standard library bindings.
    pub fn populate_environment(&self, env: &Arc<ThreadSafeEnvironment>) {
        // Core arithmetic and mathematical operations
        crate::stdlib::arithmetic::create_arithmetic_bindings(env);
        
        // Bytevector operations (R7RS Section 6.9)
        crate::stdlib::bytevector::bind_bytevector_operations(env);
        
        // String manipulation functions
        crate::stdlib::strings::create_string_bindings(env);
        
        // List processing functions
        crate::stdlib::lists::create_list_bindings(env);
        
        // Vector operations
        crate::stdlib::vectors::create_vector_bindings(env);
        
        // Character operations
        crate::stdlib::characters::create_character_bindings(env);
        
        // Character set operations (SRFI-14)
        crate::stdlib::charset::create_charset_bindings(env);
        
        // I/O operations (legacy R7RS system)
        crate::stdlib::io::create_io_bindings(env);
        
        // Advanced I/O system integration (R7RS-large)
        crate::stdlib::io_integration::create_io_integration_bindings(env);
        
        // Control flow procedures
        crate::stdlib::control::create_control_bindings(env);
        
        // Type operations (when implemented)
        // types::create_type_bindings(env);
        
        // Effect system integration
        crate::stdlib::effects::create_effect_bindings(env);
        
        // Exception handling (R7RS + SRFI-23 enhanced)
        crate::stdlib::exceptions::create_exception_bindings(env);
        
        // Enhanced SRFI-23 compliance bindings
        crate::stdlib::srfi23_enhanced::create_enhanced_srfi23_bindings(env);
        
        // Parameter objects (SRFI-39)
        crate::stdlib::parameters::install_parameter_functions(env);
        
        // Record types (SRFI-9)
        crate::stdlib::records_simple::create_record_bindings(env);
        crate::stdlib::srfi9_macro::install_define_record_type_macro(env);
        
        // System interface procedures (R7RS Section 6.14)
        crate::stdlib::system::create_system_bindings(env);
        
        // SRFI-135 Text processing (R7RS-large)
        crate::stdlib::text::create_text_bindings(env);
        crate::stdlib::text_regex::create_regex_bindings(env);
        crate::stdlib::text_algorithms::create_text_algorithm_bindings(env);
        crate::stdlib::text_srfi135::create_complete_srfi135_bindings(env);
        
        // Concurrency and parallelism (R7RS-large)
        crate::stdlib::concurrency::populate_environment(env);
        
        // Set operations (SRFI-113)
        crate::stdlib::sets::install_set_primitives(env);
        
        // Bag (multiset) operations (SRFI-113)
        crate::stdlib::bags::install_bag_primitives(env);
        
        // Generator operations (SRFI-121)
        crate::stdlib::generators::create_generator_bindings(env);
        
        // Core Scheme procedures that don't fit in other categories
        super::bind_core_procedures(env);
    }
    
    /// Gets all built-in procedures (legacy interface).
    pub fn builtins(&self) -> Vec<(&'static str, super::BuiltinProcedure)> {
        vec![
            // Arithmetic
            ("+", super::BuiltinProcedure::Add),
            ("-", super::BuiltinProcedure::Subtract),
            ("*", super::BuiltinProcedure::Multiply),
            ("/", super::BuiltinProcedure::Divide),
            
            // Comparison
            ("=", super::BuiltinProcedure::Equal),
            ("<", super::BuiltinProcedure::LessThan),
            (">", super::BuiltinProcedure::GreaterThan),
            
            // List operations
            ("cons", super::BuiltinProcedure::Cons),
            ("car", super::BuiltinProcedure::Car),
            ("cdr", super::BuiltinProcedure::Cdr),
            ("list", super::BuiltinProcedure::List),
            
            // I/O
            ("display", super::BuiltinProcedure::Display),
            ("newline", super::BuiltinProcedure::Newline),
            
            // System functions
            ("features", super::BuiltinProcedure::Features),
            ("current-second", super::BuiltinProcedure::CurrentSecond),
            ("current-jiffy", super::BuiltinProcedure::CurrentJiffy),
            ("jiffies-per-second", super::BuiltinProcedure::JiffiesPerSecond),
            ("command-line", super::BuiltinProcedure::CommandLine),
            ("get-environment-variable", super::BuiltinProcedure::GetEnvironmentVariable),
            ("get-environment-variables", super::BuiltinProcedure::GetEnvironmentVariables),
            ("exit", super::BuiltinProcedure::Exit),
            ("emergency-exit", super::BuiltinProcedure::EmergencyExit),
        ]
    }
}

impl Default for StandardLibrary {
    fn default() -> Self {
        Self::new()
    }
}
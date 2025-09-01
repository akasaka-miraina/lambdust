//! SRFI-115: Scheme Regular Expressions
//!
//! This module provides a complete implementation of SRFI-115, offering:
//! - Symbolic Regular Expressions (SRE) with Scheme-native syntax
//! - Unicode-aware pattern matching with full property support
//! - Rich match objects with submatch and named group access
//! - High-performance hybrid matching engine
//! - Complete SRFI-115 procedure interface
//!
//! ## Core Concepts
//!
//! ### SRE (Symbolic Regular Expression)
//! SRE provides a Scheme-native way to express regular expressions:
//! ```scheme
//! ;; Traditional regex: /\d{2,4}-\w+/
//! ;; SRE equivalent:
//! '(: (** 2 4 digit) "-" (+ word))
//! ```
//!
//! ### Match Objects
//! Rich objects containing match results:
//! - Main match position and substring
//! - All submatch positions and substrings  
//! - Named group access
//! - Context information (before/after)
//!
//! ## Performance
//!
//! The implementation uses a hybrid approach:
//! - Simple patterns → Rust `regex` crate (optimal performance)
//! - Complex SRE patterns → Custom engine (full feature support)
//! - Unicode properties → Cached lookup tables
//! - Character classes → Optimized interval trees

pub mod ast;
pub mod parser;
pub mod compiler;
pub mod match_object;
pub mod unicode;
pub mod procedures;
pub mod error;

use crate::diagnostics::Result;
use crate::eval::value::{PrimitiveImpl, PrimitiveProcedure, ThreadSafeEnvironment, Value};
use crate::effects::Effect;
use std::sync::Arc;

pub use ast::{SreAst, SreNode};
pub use compiler::{CompiledRegex, RegexCompiler, CompilationFlags};
pub use error::{RegexError, RegexResult};
pub use match_object::{MatchObject, SubmatchInfo};
pub use parser::SreParser;
pub use procedures::*;
pub use unicode::*;

/// Initialize SRFI-115 regular expression procedures in the environment.
///
/// Registers all standard SRFI-115 procedures:
/// - `regexp` - Compile SRE to regex object
/// - `regexp?` - Test if object is compiled regex
/// - `regexp-matches` - Match entire string
/// - `regexp-search` - Find substring match
/// - `regexp-replace` - Replace first match
/// - `regexp-replace-all` - Replace all matches
/// - `regexp-split` - Split string on pattern
/// - `regexp-match?` - Test if match exists
/// - `regexp-match-count` - Count number of matches
/// - `regexp-match-submatch` - Extract submatch by index
/// - `regexp-match-submatch-start` - Get submatch start position
/// - `regexp-match-submatch-end` - Get submatch end position
/// - `regexp-match-named-submatch` - Extract named submatch
pub fn init_srfi115_regex(env: &Arc<ThreadSafeEnvironment>) {
    // Core compilation procedures
    env.define(
        "regexp".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp".to_string(),
            arity_min: 1,
            arity_max: Some(2), // sre + optional flags
            implementation: PrimitiveImpl::RustFn(primitive_regexp),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "regexp?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_regexp_p),
            effects: vec![Effect::Pure],
        })),
    );

    // Matching procedures
    env.define(
        "regexp-matches".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp-matches".to_string(),
            arity_min: 2,
            arity_max: Some(4), // regex, string, optional start/end
            implementation: PrimitiveImpl::RustFn(primitive_regexp_matches),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "regexp-search".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp-search".to_string(),
            arity_min: 2,
            arity_max: Some(4), // regex, string, optional start/end
            implementation: PrimitiveImpl::RustFn(primitive_regexp_search),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "regexp-match?".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp-match?".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_regexp_match_p),
            effects: vec![Effect::Pure],
        })),
    );

    // Replacement procedures
    env.define(
        "regexp-replace".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp-replace".to_string(),
            arity_min: 3,
            arity_max: Some(5), // regex, string, replacement, optional start/end
            implementation: PrimitiveImpl::RustFn(primitive_regexp_replace),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "regexp-replace-all".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp-replace-all".to_string(),
            arity_min: 3,
            arity_max: Some(5), // regex, string, replacement, optional start/end
            implementation: PrimitiveImpl::RustFn(primitive_regexp_replace_all),
            effects: vec![Effect::Pure],
        })),
    );

    // Utility procedures
    env.define(
        "regexp-split".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp-split".to_string(),
            arity_min: 2,
            arity_max: Some(5), // regex, string, optional start/end/count
            implementation: PrimitiveImpl::RustFn(primitive_regexp_split),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "regexp-match-count".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp-match-count".to_string(),
            arity_min: 1,
            arity_max: Some(1),
            implementation: PrimitiveImpl::RustFn(primitive_regexp_match_count),
            effects: vec![Effect::Pure],
        })),
    );

    // Match object access procedures
    env.define(
        "regexp-match-submatch".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp-match-submatch".to_string(),
            arity_min: 2,
            arity_max: Some(2), // match, index
            implementation: PrimitiveImpl::RustFn(primitive_regexp_match_submatch),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "regexp-match-submatch-start".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp-match-submatch-start".to_string(),
            arity_min: 2,
            arity_max: Some(2), // match, index
            implementation: PrimitiveImpl::RustFn(primitive_regexp_match_submatch_start),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "regexp-match-submatch-end".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp-match-submatch-end".to_string(),
            arity_min: 2,
            arity_max: Some(2), // match, index
            implementation: PrimitiveImpl::RustFn(primitive_regexp_match_submatch_end),
            effects: vec![Effect::Pure],
        })),
    );

    env.define(
        "regexp-match-named-submatch".to_string(),
        Value::Primitive(Arc::new(PrimitiveProcedure {
            name: "regexp-match-named-submatch".to_string(),
            arity_min: 2,
            arity_max: Some(2), // match, name
            implementation: PrimitiveImpl::RustFn(primitive_regexp_match_named_submatch),
            effects: vec![Effect::Pure],
        })),
    );
}

// Temporarily disabled - depends on disabled modules
// /// SRFI-115 Regular Expression Engine
// ///
/// This is the main engine that coordinates:
/// - SRE parsing and compilation
/// - Pattern matching execution  
/// - Match object creation
/// - Error handling and diagnostics
pub struct RegexEngine {
    parser: SreParser,
    compiler: RegexCompiler,
}

impl RegexEngine {
    /// Create a new regex engine with default configuration
    pub fn new() -> Self {
        Self {
            parser: SreParser::new(),
            compiler: RegexCompiler::new(),
        }
    }

    /// Compile an SRE expression into a regex object
    ///
    /// # Arguments
    /// * `sre` - The Scheme expression representing the pattern
    /// * `flags` - Optional compilation flags
    ///
    /// # Returns
    /// A compiled regex that can be used for matching
    pub fn compile(&mut self, sre: &Value, flags: Option<&Value>) -> RegexResult<CompiledRegex> {
        // Parse SRE into AST
        let ast = self.parser.parse_sre(sre)?;

        // Apply any flags
        let mut compilation_flags = compiler::CompilationFlags::default();
        if let Some(flags_val) = flags {
            compilation_flags = self.parse_flags(flags_val)?;
        }

        // Compile AST to executable pattern
        self.compiler.compile(&ast, compilation_flags)
    }

    /// Match pattern against entire string
    pub fn matches(
        &self,
        regex: &CompiledRegex,
        text: &str,
        start: Option<usize>,
        end: Option<usize>,
    ) -> RegexResult<Option<MatchObject>> {
        let effective_start = start.unwrap_or(0);
        let effective_end = end.unwrap_or(text.len());

        if effective_start > text.len() || effective_end > text.len() || effective_start > effective_end {
            return Err(RegexError::InvalidRange {
                start: effective_start,
                end: effective_end,
                text_len: text.len(),
            });
        }

        let substring = &text[effective_start..effective_end];
        regex.matches_entire(substring, effective_start)
    }

    /// Search for pattern within string
    pub fn search(
        &self,
        regex: &CompiledRegex,
        text: &str,
        start: Option<usize>,
        end: Option<usize>,
    ) -> RegexResult<Option<MatchObject>> {
        let effective_start = start.unwrap_or(0);
        let effective_end = end.unwrap_or(text.len());

        if effective_start > text.len() || effective_end > text.len() || effective_start > effective_end {
            return Err(RegexError::InvalidRange {
                start: effective_start,
                end: effective_end,
                text_len: text.len(),
            });
        }

        let search_text = &text[effective_start..effective_end];
        regex.search(search_text, effective_start)
    }

    /// Parse compilation flags from Scheme value
    fn parse_flags(&self, flags: &Value) -> RegexResult<compiler::CompilationFlags> {
        let mut compilation_flags = compiler::CompilationFlags::default();

        match flags {
            v if v.is_list() => {
                let mut current = v;
                while let Value::Pair(car, cdr) = current {
                    match car.as_ref() {
                        v if v.is_symbol() => {
                            if let Some(sym) = v.as_symbol() {
                                let flag_name = crate::utils::symbol_name(sym)
                                .ok_or_else(|| RegexError::InvalidFlag {
                                    flag: format!("symbol-{:?}", sym),
                                })?;

                                match flag_name.as_str() {
                                    "i" | "case-insensitive" => compilation_flags.case_insensitive = true,
                                    "m" | "multiline" => compilation_flags.multiline = true,
                                    "s" | "dotall" => compilation_flags.dotall = true,
                                    "u" | "unicode" => compilation_flags.unicode = true,
                                    "x" | "extended" => compilation_flags.extended = true,
                                    _ => {
                                        return Err(RegexError::InvalidFlag {
                                            flag: flag_name,
                                        });
                                    }
                                }
                            } else {
                                return Err(RegexError::InvalidFlag {
                                    flag: "invalid symbol".to_string(),
                                });
                            }
                        }
                        _ => {
                            return Err(RegexError::InvalidFlag {
                                flag: format!("{:?}", car),
                            });
                        }
                    }
                    current = cdr.as_ref();
                }
            }
            _ => {
                return Err(RegexError::InvalidFlag {
                    flag: format!("{:?}", flags),
                });
            }
        }

        Ok(compilation_flags)
    }
}

impl Default for RegexEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Global regex engine instance for SRFI-115 procedures
static REGEX_ENGINE: std::sync::LazyLock<std::sync::Mutex<RegexEngine>> = 
    std::sync::LazyLock::new(|| std::sync::Mutex::new(RegexEngine::new()));

/// Get the global regex engine instance
pub fn get_regex_engine() -> &'static std::sync::Mutex<RegexEngine> {
    &REGEX_ENGINE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::value::Value;

    #[test]
    fn test_engine_creation() {
        let engine = RegexEngine::new();
        assert!(engine.parser.is_ready());
        assert!(engine.compiler.is_ready());
    }

    #[test]
    fn test_simple_literal_compilation() {
        let mut engine = RegexEngine::new();
        let sre = Value::string("hello".to_string());
        
        let result = engine.compile(&sre, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_global_engine_access() {
        let engine1 = get_regex_engine();
        let engine2 = get_regex_engine();
        
        // Should be the same instance
        assert!(std::ptr::eq(engine1, engine2));
    }
}
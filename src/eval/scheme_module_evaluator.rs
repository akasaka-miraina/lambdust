//! Scheme module evaluation engine for the lazy module loader.
//!
//! This module provides the capability to evaluate Scheme module source files
//! and extract their exports in a safe, isolated environment.

use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::{Environment, Evaluator, ThreadSafeEnvironment, Value, PrimitiveImpl, PrimitiveProcedure};
use crate::lexer::Lexer;
use crate::module_system::{ModuleId, ModuleNamespace};
use crate::utils::intern_symbol;
use crate::parser::Parser;
use crate::effects::Effect;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// Trait for evaluating Scheme module source files.
pub trait SchemeModuleEvaluator: Send + Sync {
    /// Evaluates a module source file and returns its exports.
    fn evaluate_module(
        &self,
        module_id: &ModuleId,
        source_code: &str,
        evaluator: &mut Evaluator,
    ) -> Result<HashMap<String, Value>>;
}

/// Default implementation of SchemeModuleEvaluator.
pub struct DefaultSchemeModuleEvaluator;

impl SchemeModuleEvaluator for DefaultSchemeModuleEvaluator {
    fn evaluate_module(
        &self,
        module_id: &ModuleId,
        source_code: &str,
        evaluator: &mut Evaluator,
    ) -> Result<HashMap<String, Value>> {
        // For now, return pre-defined exports for known SRFIs
        match (module_id.namespace.clone(), module_id.components.as_slice()) {
            // Foundational SRFIs (Tier 1)
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "2" => {
                // SRFI-2 requires macro expansion, load from actual Scheme source
                self.evaluate_source_code(module_id, source_code, evaluator)
            },
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "6" => Ok(self.srfi_6_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "8" => {
                // SRFI-8 requires macro expansion, load from actual Scheme source
                self.evaluate_source_code(module_id, source_code, evaluator)
            },
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "9" => Ok(self.srfi_9_exports()),
            
            // Core Utility SRFIs (Tier 2)
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "11" => Ok(self.srfi_11_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "14" => Ok(self.srfi_14_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "16" => Ok(self.srfi_16_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "23" => Ok(self.srfi_23_exports()),
            
            // Enhanced Collections (Tier 3)
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "113" => Ok(self.srfi_113_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "117" => Ok(self.srfi_117_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "121" => Ok(self.srfi_121_exports()),
            
            // R7RS Alignment (Tier 4)
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "135" => Ok(self.srfi_135_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "150" => Ok(self.srfi_150_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "151" => Ok(self.srfi_151_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "159" => Ok(self.srfi_159_exports()),
            
            // Specialized Features (Tier 5)
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "46" => Ok(self.srfi_46_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "120" => Ok(self.srfi_120_exports()),
            
            // Existing implementations
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "41" => Ok(self.srfi_41_exports()),
            (ModuleNamespace::R7RS, components) if components.len() == 1 && components[0] == "41" => Ok(self.srfi_41_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "43" => Ok(self.srfi_43_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "1" => Ok(self.srfi_1_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "13" => Ok(self.srfi_13_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "26" => Ok(self.srfi_26_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "39" => Ok(self.srfi_39_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "128" => Ok(self.srfi_128_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "125" => Ok(self.srfi_125_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "111" => Ok(self.srfi_111_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "158" => Ok(self.srfi_158_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "149" => Ok(self.srfi_149_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "132" => Ok(self.srfi_132_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "124" => Ok(self.srfi_124_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "19" => Ok(self.srfi_19_exports()),
            (ModuleNamespace::SRFI, components) if components.len() == 1 && components[0] == "21" => Ok(self.srfi_21_exports()),
            _ => {
                // Try to evaluate the actual source code
                self.evaluate_source_code(module_id, source_code, evaluator)
            }
        }
    }
}

impl DefaultSchemeModuleEvaluator {
    pub fn new() -> Self {
        Self
    }
    /// Returns pre-defined exports for SRFI-41 (Streams).
    fn srfi_41_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Core stream values
        exports.insert("stream-null".to_string(), Value::Nil);
        
        // Core stream functions (as symbols for now)
        let stream_functions = [
            "stream-cons", "stream?", "stream-null?", "stream-pair?",
            "stream-car", "stream-cdr", "stream-lambda",
            "define-stream", "list->stream", "port->stream", "stream->list",
            "stream-append", "stream-concat", "stream-constant",
            "stream-drop", "stream-drop-while", "stream-filter",
            "stream-fold", "stream-for-each", "stream-from",
            "stream-iterate", "stream-length", "stream-let",
            "stream-map", "stream-match", "stream-of",
            "stream-range", "stream-ref", "stream-reverse",
            "stream-scan", "stream-take", "stream-take-while",
            "stream-unfold", "stream-unfolds", "stream-zip",
        ];
        
        for func_name in &stream_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-43 (Vector Library).
    fn srfi_43_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Vector constructors and basic operations
        let vector_functions = [
            "make-vector", "vector", "vector-unfold", "vector-unfold-right",
            "vector-copy", "vector-reverse-copy", "vector-append", "vector-concatenate",
            
            // Predicates
            "vector?", "vector-empty?", "vector=",
            
            // Selectors  
            "vector-ref", "vector-length",
            
            // Iteration
            "vector-fold", "vector-fold-right", "vector-reduce", "vector-reduce-right",
            "vector-map", "vector-map!", "vector-for-each", "vector-count",
            
            // Searching
            "vector-index", "vector-index-right", "vector-skip", "vector-skip-right",
            "vector-binary-search", "vector-any", "vector-every",
            
            // Mutation
            "vector-set!", "vector-swap!", "vector-fill!", "vector-reverse!",
            "vector-copy!", "vector-reverse-copy!",
        ];
        
        for func_name in &vector_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-1 (List Library).
    fn srfi_1_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Get actual implementations from global environment (where SRFI-1 functions are bound)
        let global_env = crate::eval::global_environment();
        
        // Direct implementations of key SRFI-1 functions
        exports.insert(
            "take".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "take".to_string(),
                arity_min: 2,
                arity_max: Some(2),
                implementation: PrimitiveImpl::RustFn(srfi1_take_direct),
                effects: vec![Effect::Pure],
            }))
        );
        
        exports.insert(
            "drop".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "drop".to_string(),
                arity_min: 2,
                arity_max: Some(2),
                implementation: PrimitiveImpl::RustFn(srfi1_drop_direct),
                effects: vec![Effect::Pure],
            }))
        );
        
        exports.insert(
            "last".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "last".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(srfi1_last_direct),
                effects: vec![Effect::Pure],
            }))
        );
        
        // Basic list functions from R7RS that should be globally available
        let r7rs_functions = [
            "cons", "car", "cdr", "list", "length", "append", "reverse",
            "null?", "pair?", "list?", "map", "for-each", 
            "member", "memq", "memv", "assoc", "assq", "assv",
        ];
        
        // Try to get R7RS functions from global environment
        for func_name in &r7rs_functions {
            match global_env.lookup(func_name) {
                Some(value) => {
                    exports.insert(func_name.to_string(), value);
                }
                None => {
                    // Create placeholder for functions not found
                    exports.insert(
                        func_name.to_string(),
                        Value::Symbol(intern_symbol(func_name.to_string()))
                    );
                }
            }
        }
        
        // Add remaining SRFI-1 functions as symbols (not yet implemented)
        let placeholder_functions = [
            // Constructors
            "xcons", "cons*", "make-list", "list-tabulate", "list-copy", 
            "circular-list", "iota",
            
            // Predicates  
            "proper-list?", "circular-list?", "dotted-list?", "not-pair?", 
            "null-list?", "list=",
            
            // Selectors
            "first", "second", "third", "fourth", "fifth", "sixth", 
            "seventh", "eighth", "ninth", "tenth", "car+cdr",
            "take!", "drop-right!", "split-at!",
            
            // Fold, unfold & map
            "fold", "fold-right", "pair-fold", "pair-fold-right", 
            "reduce", "reduce-right", "unfold", "unfold-right",
            "append-map", "append-map!",
            
            // Filter & partition
            "filter", "partition", "remove", "filter!", "partition!", "remove!",
            
            // Search
            "find", "find-tail", "any", "every", "list-index", "take-while!", 
            "span", "break", "span!", "break!",
            
            // Association lists
            "alist-cons", "alist-copy", "alist-delete", "alist-delete!",
            
            // Set operations  
            "lset<=", "lset=", "lset-adjoin", "lset-union", "lset-intersection",
            "lset-difference", "lset-xor", "lset-diff+intersection",
            "lset-union!", "lset-intersection!", "lset-difference!", "lset-xor!",
            "lset-diff+intersection!",
        ];
        
        // Add placeholders for functions not yet implemented
        for func_name in &placeholder_functions {
            if !exports.contains_key(*func_name) {
                exports.insert(
                    func_name.to_string(),
                    Value::Symbol(intern_symbol(func_name.to_string()))
                );
            }
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-13 (String Library).
    fn srfi_13_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // String processing functions (comprehensive list from SRFI-13)
        let string_functions = [
            // Predicates
            "string-null?", "string-every", "string-any",
            
            // Constructors
            "make-string", "string", "string-tabulate",
            
            // List & string conversion
            "string->list", "list->string", "reverse-list->string", "string-join",
            
            // Selection
            "string-length", "string-ref", "string-copy", "substring/shared",
            "string-copy!", "string-take", "string-drop",
            "string-take-right", "string-drop-right",
            "string-pad", "string-pad-right", 
            "string-trim", "string-trim-right", "string-trim-both",
            
            // Modification
            "string-set!", "string-fill!",
            
            // Comparison
            "string-compare", "string-compare-ci",
            "string=", "string<>", "string<", "string>", "string<=", "string>=",
            "string-ci=", "string-ci<>", "string-ci<", "string-ci>", 
            "string-ci<=", "string-ci>=",
            "string-hash", "string-hash-ci",
            
            // Searching
            "string-prefix-length", "string-suffix-length",
            "string-prefix?", "string-suffix?",
            "string-index", "string-index-right",
            "string-skip", "string-skip-right",
            "string-count", "string-contains", "string-contains-ci",
            
            // Case mapping
            "string-upcase", "string-downcase", "string-titlecase",
            "string-upcase!", "string-downcase!", "string-titlecase!",
            
            // Reverse & append
            "string-reverse", "string-reverse!",
            "string-append", "string-concatenate", "string-concatenate-reverse",
            "string-concatenate/shared",
            
            // Fold, unfold & map
            "string-map", "string-map!", "string-fold", "string-fold-right",
            "string-unfold", "string-unfold-right",
            "string-for-each", "string-for-each-index",
            
            // Replicate & rotate
            "xsubstring", "string-xcopy!",
        ];
        
        for func_name in &string_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-26 (Cut and Cute).
    fn srfi_26_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // SRFI-26 cut/cute macros are handled by the macro system
        // These are special forms, not regular procedures
        // The evaluator handles them in eval_cut() and eval_cute() methods
        
        // The macro system recognizes 'cut' and 'cute' as special forms
        // We provide them as symbols, but the evaluator handles them specially
        exports.insert(
            "cut".to_string(),
            Value::Symbol(intern_symbol("cut".to_string()))
        );
        exports.insert(
            "cute".to_string(),
            Value::Symbol(intern_symbol("cute".to_string()))
        );
        
        // Placeholder symbols for cut/cute syntax
        exports.insert(
            "<>".to_string(),
            Value::Symbol(intern_symbol("<>".to_string()))
        );
        exports.insert(
            "<...>".to_string(),
            Value::Symbol(intern_symbol("<...>".to_string()))
        );
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-39 (Parameters).
    fn srfi_39_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Parameter objects
        exports.insert(
            "make-parameter".to_string(),
            Value::Symbol(intern_symbol("make-parameter".to_string()))
        );
        exports.insert(
            "parameterize".to_string(),
            Value::Symbol(intern_symbol("parameterize".to_string()))
        );
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-128 (Comparators).
    fn srfi_128_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Core comparator functions
        let comparator_functions = [
            // Constructor
            "make-comparator",
            
            // Standard comparators
            "boolean-comparator", "char-comparator", "string-comparator",
            "string-ci-comparator", "symbol-comparator", "number-comparator", 
            "default-comparator",
            
            // Predicates
            "comparator?", "comparator-ordered?", "comparator-hashable?",
            
            // Comparison procedures
            "comparator-test-type", "comparator-equal?", "comparator-compare", 
            "comparator-hash",
            
            // Derived comparison procedures
            "=?", "<?", ">?", "<=?", ">=?",
            
            // Utilities
            "make-comparison", "make-hash-function", "comparator-register-default!",
        ];
        
        for func_name in &comparator_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-125 (Intermediate Hash Tables).
    fn srfi_125_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Hash table functions (comprehensive list from SRFI-125)
        let hash_table_functions = [
            // Constructors
            "make-hash-table", "hash-table", "hash-table-unfold", "alist->hash-table",
            
            // Predicates
            "hash-table?", "hash-table-contains?", "hash-table-empty?",
            "hash-table=?", "hash-table-mutable?",
            
            // Accessors
            "hash-table-ref", "hash-table-ref/default",
            
            // Mutators
            "hash-table-set!", "hash-table-delete!", "hash-table-intern!",
            "hash-table-update!", "hash-table-update!/default", "hash-table-pop!",
            "hash-table-clear!",
            
            // The whole hash table
            "hash-table-size", "hash-table-keys", "hash-table-values", "hash-table-entries",
            "hash-table-find", "hash-table-count",
            
            // Mapping and folding
            "hash-table-map", "hash-table-for-each", "hash-table-map!",
            "hash-table-map->list", "hash-table-fold", "hash-table-prune!",
            
            // Copying and conversion
            "hash-table-copy", "hash-table->alist",
            
            // Hash table merge
            "hash-table-union!", "hash-table-intersection!", "hash-table-difference!",
            "hash-table-xor!",
            
            // Hash functions
            "hash-table-equivalence-function", "hash-table-hash-function",
            
            // Deprecated names
            "hash-table-walk", "hash-table-merge!",
        ];
        
        for func_name in &hash_table_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-111 (Boxes).
    fn srfi_111_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Core Box functions (using existing implementations from src/stdlib/box.rs)
        exports.insert(
            "box".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "box".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(crate::stdlib::r#box::primitive_box),
                effects: vec![Effect::Pure],
            }))
        );
        
        exports.insert(
            "box?".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "box?".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(crate::stdlib::r#box::primitive_box_p),
                effects: vec![Effect::Pure],
            }))
        );
        
        exports.insert(
            "unbox".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "unbox".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(crate::stdlib::r#box::primitive_unbox),
                effects: vec![Effect::Pure],
            }))
        );
        
        exports.insert(
            "set-box!".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "set-box!".to_string(),
                arity_min: 2,
                arity_max: Some(2),
                implementation: PrimitiveImpl::RustFn(crate::stdlib::r#box::primitive_set_box),
                effects: vec![Effect::Mutation],
            }))
        );
        
        // Advanced convenience functions (need to implement these)
        // For now, export as symbols to avoid compilation issues
        let convenience_functions = ["box-cas!", "box-swap!"];
        for func_name in &convenience_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-158 (Enhanced Generators and Accumulators).
    fn srfi_158_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Enhanced generator functions (68 total: 53 from SRFI-121 + 15 new)
        let generator_functions = [
            // Core generator operations from SRFI-121
            "%make-generator", "%generator-next", "%generator-exhausted?", "%generator?",
            
            // Generator constructors from SRFI-121
            "generator", "make-range-generator", "make-iota-generator",
            "list->generator", "vector->generator", "reverse-vector->generator",
            "string->generator", "bytevector->generator",
            "make-for-each-generator", "make-unfold-generator",
            "make-coroutine-generator",
            
            // Generator transformers/combinators from SRFI-121
            "gcons*", "gappend", "gflatten", "ggroup",
            "gfilter", "gremove", "gstate-filter", "gdelete", "gdelete-neighbor-dups",
            "gtake", "gdrop", "gtake-while", "gdrop-while",
            "gindex", "gselect",
            
            // Generator consumers from SRFI-121
            "generator->list", "generator->vector", "generator->string", "generator->bytevector",
            "generator-fold", "generator-reduce", "generator-for-each", "generator-map",
            "generator-find", "generator-count", "generator-any", "generator-every",
            "generator-length", "generator-sum", "generator-unfold",
            
            // Accumulator procedures from SRFI-121
            "make-accumulator", "count-accumulator", "list-accumulator",
            "reverse-list-accumulator", "vector-accumulator", "string-accumulator",
            "bytevector-accumulator", "sum-accumulator",
            "make-accumulator-generator",
            
            // NEW SRFI-158 Enhanced constructors (2)
            "circular-generator", "make-bits-generator",
            
            // NEW SRFI-158 Enhanced transformers (3)
            "gmerge", "gcombine", "generator-zip-with",
            
            // NEW SRFI-158 Enhanced accumulators (7)
            "product-accumulator", "min-accumulator", "max-accumulator",
            "enhanced-vector-accumulator", "accumulate-generated",
            "generator-accumulate", "enhanced-make-accumulator-generator",
            
            // NEW SRFI-158 Advanced utilities (3)
            "generator-concatenate", "generator-pad-with", "generator-maybe-ref",
        ];
        
        for func_name in &generator_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-149 (Basic Syntax-Rules Template Extensions).
    fn srfi_149_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Template extension functions from SRFI-149
        let template_functions = [
            "syntax-rules", "syntax-rules-149",
        ];
        
        for func_name in &template_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-132 (Sort Libraries).
    fn srfi_132_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Comprehensive sorting functions from SRFI-132
        let sort_functions = [
            // List sorting procedures
            "list-sort", "list-stable-sort", "list-sort!", "list-stable-sort!",
            "list-sorted?",
            
            // Vector sorting procedures
            "vector-sort", "vector-stable-sort", "vector-sort!", "vector-stable-sort!",
            "vector-sorted?",
            
            // Merge procedures
            "list-merge", "list-merge!", "vector-merge", "vector-merge!",
            
            // Delete duplicate procedures
            "list-delete-neighbor-dups", "list-delete-neighbor-dups!",
            "vector-delete-neighbor-dups", "vector-delete-neighbor-dups!",
            
            // Advanced procedures
            "vector-find-median", "vector-find-median!", "vector-select!", "vector-separate!",
        ];
        
        for func_name in &sort_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-2 (and-let*).
    fn srfi_2_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // SRFI-2 provides the and-let* macro for sequential binding and testing
        exports.insert(
            "and-let*".to_string(),
            Value::Symbol(intern_symbol("and-let*".to_string()))
        );
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-6 (Basic String Ports).
    fn srfi_6_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // open-input-string: Creates an input port from a string
        exports.insert(
            "open-input-string".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "open-input-string".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(srfi6_open_input_string_direct),
                effects: vec![Effect::IO],
            }))
        );
        
        // open-output-string: Creates an output port that accumulates written data
        exports.insert(
            "open-output-string".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "open-output-string".to_string(),
                arity_min: 0,
                arity_max: Some(0),
                implementation: PrimitiveImpl::RustFn(srfi6_open_output_string_direct),
                effects: vec![Effect::IO],
            }))
        );
        
        // get-output-string: Extracts accumulated string from output port
        exports.insert(
            "get-output-string".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "get-output-string".to_string(),
                arity_min: 1,
                arity_max: Some(1),
                implementation: PrimitiveImpl::RustFn(srfi6_get_output_string_direct),
                effects: vec![Effect::IO],
            }))
        );
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-8 (receive).
    /// Note: SRFI-8 uses macro expansion, deferred to source code evaluation.
    fn srfi_8_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // SRFI-8 provides the receive macro - implemented via syntax-rules
        // Will be loaded from actual source code via evaluate_source_code
        exports.insert(
            "receive".to_string(),
            Value::Symbol(intern_symbol("receive".to_string()))
        );
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-9 (Defining Record Types).
    fn srfi_9_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // SRFI-9 provides the define-record-type macro
        exports.insert(
            "define-record-type".to_string(),
            Value::Symbol(intern_symbol("define-record-type".to_string()))
        );
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-11 (Let-values and Let*-values).
    fn srfi_11_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // SRFI-11 provides let-values and let*-values macros for multiple value binding
        let let_values_functions = [
            "let-values",
            "let*-values",
        ];
        
        for func_name in &let_values_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-14 (Character-set Library).
    fn srfi_14_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Character set predicates and operations from SRFI-14
        let char_set_functions = [
            // Predicates
            "char-set?", "char-set=", "char-set<=", "char-set-hash",
            
            // Iterating over character sets
            "char-set-cursor", "char-set-ref", "char-set-cursor-next", "end-of-char-set?",
            "char-set-fold", "char-set-unfold", "char-set-unfold!",
            "char-set-for-each", "char-set-map",
            
            // Creating character sets
            "char-set", "char-set-copy", "list->char-set", "string->char-set",
            "list->char-set!", "string->char-set!",
            "char-set-filter", "char-set-filter!",
            "ucs-range->char-set", "ucs-range->char-set!",
            
            // Querying character sets
            "char-set->list", "char-set->string",
            "char-set-size", "char-set-count", "char-set-contains?",
            "char-set-every", "char-set-any",
            
            // Character set algebra
            "char-set-adjoin", "char-set-delete", "char-set-adjoin!", "char-set-delete!",
            "char-set-complement", "char-set-union", "char-set-intersection", 
            "char-set-difference", "char-set-xor",
            "char-set-complement!", "char-set-union!", "char-set-intersection!",
            "char-set-difference!", "char-set-xor!",
        ];
        
        for func_name in &char_set_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        // Standard character sets
        let standard_char_sets = [
            "char-set:lower-case", "char-set:upper-case", "char-set:title-case",
            "char-set:letter", "char-set:digit", "char-set:letter+digit",
            "char-set:graphic", "char-set:printing", "char-set:whitespace",
            "char-set:iso-control", "char-set:punctuation", "char-set:symbol",
            "char-set:hex-digit", "char-set:blank", "char-set:ascii",
            "char-set:empty", "char-set:full",
        ];
        
        for char_set in &standard_char_sets {
            exports.insert(
                char_set.to_string(),
                Value::Symbol(intern_symbol(char_set.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-16 (Case-lambda).
    fn srfi_16_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // SRFI-16 provides the case-lambda macro for variable arity procedures
        exports.insert(
            "case-lambda".to_string(),
            Value::Symbol(intern_symbol("case-lambda".to_string()))
        );
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-23 (Error reporting mechanism).
    fn srfi_23_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // SRFI-23 provides the error procedure for error reporting
        exports.insert(
            "error".to_string(),
            Value::Primitive(Arc::new(PrimitiveProcedure {
                name: "error".to_string(),
                arity_min: 1,
                arity_max: None, // Variable arity (message + optional irritants)
                implementation: PrimitiveImpl::RustFn(srfi23_error_direct),
                effects: vec![Effect::Error],
            }))
        );
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-113 (Sets and Bags).
    fn srfi_113_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Set procedures from SRFI-113
        let set_functions = [
            // Constructors
            "set", "list->set",
            
            // Predicates
            "set?", "set-contains?", "set-empty?", "set-disjoint?",
            
            // Accessors
            "set-member", "set-element-at",
            
            // Updaters
            "set-adjoin", "set-adjoin!", "set-replace", "set-replace!",
            "set-delete", "set-delete!", "set-delete-all", "set-delete-all!",
            "set-search!",
            
            // Whole set procedures
            "set-size", "set-find", "set-count", "set-any?", "set-every?",
            
            // Mapping and folding
            "set-map", "set-for-each", "set-fold",
            "set-filter", "set-filter!", "set-remove", "set-remove!",
            "set-partition", "set-partition!",
            
            // Copying and conversion
            "set-copy", "set->list", "list->set", "set->bag",
            
            // Subsets
            "set=?", "set<?", "set>?", "set<=?", "set>=?",
            
            // Set theory operations
            "set-union", "set-intersection", "set-difference", "set-xor",
            "set-union!", "set-intersection!", "set-difference!", "set-xor!",
            
            // Bag procedures
            "bag", "list->bag", "bag?", "bag-contains?", "bag-empty?", "bag-disjoint?",
            "bag-element-count", "bag-for-each-unique", "bag-fold-unique",
            "bag-increment!", "bag-decrement!", "bag-size", "bag-find", "bag-count",
            "bag-any?", "bag-every?", "bag-map", "bag-for-each", "bag-fold",
            "bag-filter", "bag-filter!", "bag-remove", "bag-remove!",
            "bag-partition", "bag-partition!", "bag-copy", "bag->list", "bag->set",
            "bag=?", "bag<?", "bag>?", "bag<=?", "bag>=?",
            "bag-union", "bag-intersection", "bag-difference", "bag-xor",
            "bag-union!", "bag-intersection!", "bag-difference!", "bag-xor!",
            "bag-sum", "bag-sum!", "bag-product", "bag-product!",
            "bag-unique-size", "bag->alist", "alist->bag",
        ];
        
        for func_name in &set_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-117 (Queues based on Lists).
    fn srfi_117_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Queue operations from SRFI-117
        let queue_functions = [
            // Queue Type
            "make-queue", "queue", "queue?",
            
            // List Queue Type
            "make-list-queue", "list-queue", "list-queue?",
            
            // Common Queue Operations
            "queue-empty?", "queue-front", "queue-back", "queue-length",
            
            // Mutating Operations
            "queue-push-front!", "queue-push-back!", "queue-pop-front!", "queue-pop-back!",
            
            // List Queue Operations
            "list-queue-list", "list-queue-set-list!",
            
            // Conversion Operations
            "queue->list", "list->queue", "queue->list-queue", "list-queue->queue",
        ];
        
        for func_name in &queue_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-121 (Generators).
    fn srfi_121_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Generator functions from SRFI-121 (base for SRFI-158)
        let generator_functions = [
            // Core generator operations
            "%make-generator", "%generator-next", "%generator-exhausted?", "%generator?",
            
            // Generator constructors
            "generator", "make-range-generator", "make-iota-generator",
            "list->generator", "vector->generator", "reverse-vector->generator",
            "string->generator", "bytevector->generator",
            "make-for-each-generator", "make-unfold-generator",
            "make-coroutine-generator",
            
            // Generator transformers/combinators
            "gcons*", "gappend", "gflatten", "ggroup",
            "gfilter", "gremove", "gstate-filter", "gdelete", "gdelete-neighbor-dups",
            "gtake", "gdrop", "gtake-while", "gdrop-while",
            "gindex", "gselect",
            
            // Generator consumers
            "generator->list", "generator->vector", "generator->string", "generator->bytevector",
            "generator-fold", "generator-reduce", "generator-for-each", "generator-map",
            "generator-find", "generator-count", "generator-any", "generator-every",
            "generator-length", "generator-sum", "generator-unfold",
            
            // Accumulator procedures
            "make-accumulator", "count-accumulator", "list-accumulator",
            "reverse-list-accumulator", "vector-accumulator", "string-accumulator",
            "bytevector-accumulator", "sum-accumulator",
        ];
        
        for func_name in &generator_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-135 (Immutable Texts).
    fn srfi_135_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Text operations from SRFI-135
        let text_functions = [
            // Predicates
            "text?", "textual?", "textual-null?", "textual-every", "textual-any",
            
            // Constructors
            "text", "textual->text", "string->text", "vector->text", "list->text",
            "reverse-list->text", "textual-tabulate",
            
            // Conversion
            "text->string", "text->vector", "text->list", "textual->string",
            "textual->vector", "textual->list",
            
            // Selection
            "textual-length", "textual-ref", "subtext", "textual-copy",
            "textual-take", "textual-drop", "textual-take-right", "textual-drop-right",
            "textual-pad", "textual-pad-right", "textual-trim", "textual-trim-right", "textual-trim-both",
            
            // Comparison
            "textual=?", "textual<?", "textual>?", "textual<=?", "textual>=?",
            "textual-ci=?", "textual-ci<?", "textual-ci>?", "textual-ci<=?", "textual-ci>=?",
            
            // Searching
            "textual-prefix-length", "textual-suffix-length",
            "textual-prefix?", "textual-suffix?",
            "textual-index", "textual-index-right", "textual-skip", "textual-skip-right",
            "textual-contains", "textual-count",
            
            // Case conversion
            "textual-upcase", "textual-downcase", "textual-foldcase", "textual-titlecase",
            
            // Reverse and append
            "textual-reverse", "textual-append", "textual-concatenate", "textual-concatenate-reverse",
            
            // Fold and unfold
            "textual-fold", "textual-fold-right", "textual-map", "textual-for-each",
            "textual-replicate", "textual-split",
            
            // Filtering and partitioning
            "textual-filter", "textual-remove",
        ];
        
        for func_name in &text_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-150 (Hygienic ERR5RS Record Syntax).
    fn srfi_150_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Enhanced record type definition from SRFI-150
        let record_functions = [
            "define-record-type*",
            "define-record-type-helper",
        ];
        
        for func_name in &record_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-151 (Bitwise Operations).
    fn srfi_151_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Bitwise operations from SRFI-151
        let bitwise_functions = [
            // Bitwise operations
            "bitwise-not", "bitwise-and", "bitwise-ior", "bitwise-xor", "bitwise-eqv",
            "bitwise-nand", "bitwise-nor", "bitwise-andc1", "bitwise-andc2",
            "bitwise-orc1", "bitwise-orc2",
            
            // Integer operations
            "arithmetic-shift", "bit-count", "integer-length",
            "bitwise-if", "bit-set?", "copy-bit", "bit-swap",
            "any-bit-set?", "every-bit-set?", "first-set-bit",
            
            // Bit field operations
            "bit-field", "bit-field-any?", "bit-field-every?", "bit-field-clear",
            "bit-field-set", "bit-field-replace", "bit-field-replace-same",
            "bit-field-rotate", "bit-field-reverse",
            
            // Bits as integers
            "bits->list", "list->bits", "bits->vector", "vector->bits",
            "bits", "bitwise-fold", "bitwise-for-each", "bitwise-unfold",
            "make-bitwise-generator",
        ];
        
        for func_name in &bitwise_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-159 (Combinator Formatting).
    fn srfi_159_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Formatting combinators from SRFI-159
        let format_functions = [
            // Basic formatting
            "show", "displayed", "written", "written-shared", "numeric",
            "nl", "fl", "space-to", "tab-to", "nothing",
            "each", "each-in-list", "joined", "joined/prefix", "joined/suffix", "joined/last",
            "padded", "padded/right", "padded/both", "trimmed", "trimmed/right", "trimmed/both", "fitted",
            "fitted/left", "fitted/right", "fitted/both",
            
            // Conditional formatting
            "maybe", "if-show", "with",
            
            // Numeric formatting
            "numeric/comma", "numeric/si", "numeric/fitted",
            
            // Text formatting
            "upcased", "downcased", "sentence-case", "title-case",
            
            // Columnar formatting
            "columnar", "tabular",
            
            // Pretty printing
            "pretty", "pretty-shared",
        ];
        
        for func_name in &format_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-46 (Basic Syntax-rules Extensions).
    fn srfi_46_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Enhanced syntax-rules from SRFI-46
        exports.insert(
            "syntax-rules".to_string(),
            Value::Symbol(intern_symbol("syntax-rules".to_string()))
        );
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-120 (Timer APIs).
    fn srfi_120_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Timer API functions from SRFI-120
        let timer_functions = [
            "current-second", "current-jiffy", "jiffies-per-second",
            "make-timer", "timer?", "timer-start!", "timer-stop!", "timer-reset!",
            "timer-running?", "timer-elapsed-time", "timer-cumulative-time",
        ];
        
        for func_name in &timer_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-124 (Ephemerons).
    fn srfi_124_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Core ephemeron procedures from SRFI-124
        let ephemeron_functions = [
            // Core procedures
            "make-ephemeron", "ephemeron-broken?", "ephemeron-key", "ephemeron-datum",
            
            // Reference barrier
            "reference-barrier",
            
            // Additional utilities
            "ephemeron?", "set-ephemeron-datum!", "clear-reference-barrier!",
        ];
        
        for func_name in &ephemeron_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Returns pre-defined exports for SRFI-19 (Time Data Types and Procedures).
    fn srfi_19_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Time and Date procedures from SRFI-19
        let time_functions = [
            // Constructors
            "make-time", "make-date",
            
            // Type predicates
            "time?", "date?",
            
            // Current time
            "current-time",
            
            // Time accessors
            "time-type", "time-second", "time-nanosecond",
            
            // Date accessors
            "date-nanosecond", "date-second", "date-minute", "date-hour",
            "date-day", "date-month", "date-year", "date-zone-offset",
            
            // Conversions
            "time->date", "date->time",
            
            // Comparisons
            "time=?", "time<?", "time<=?", "time>?", "time>=?",
            
            // Arithmetic
            "add-duration", "subtract-duration", "time-difference",
            
            // String conversion
            "date->string", "string->date",
            
            // Julian day operations
            "date->julian-day", "julian-day->date",
            "date->modified-julian-day", "modified-julian-day->date",
        ];
        
        for func_name in &time_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        // Time type constants
        let time_types = [
            "time-utc", "time-tai", "time-monotonic", 
            "time-duration", "time-process", "time-thread"
        ];
        
        for time_type in &time_types {
            exports.insert(
                time_type.to_string(),
                Value::Symbol(intern_symbol(time_type.to_string()))
            );
        }
        
        exports
    }

    /// Returns pre-defined exports for SRFI-21 (Real-time Multithreading Support).
    fn srfi_21_exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();
        
        // Thread management functions from SRFI-21
        let threading_functions = [
            // Thread constructors and lifecycle
            "make-thread", "thread-start!", "current-thread",
            "thread-yield!", "thread-terminate!", "thread-sleep!",
            "thread-join!",
            
            // Thread predicates and accessors
            "thread?", "thread-name", "thread-specific",
            "thread-specific-set!", "thread-terminated?",
            
            // Mutex operations
            "make-mutex", "mutex-lock!", "mutex-unlock!",
            "mutex-state", "mutex?",
            
            // Condition variable operations
            "make-condition-variable", "condition-variable-signal!",
            "condition-variable-broadcast!", "condition-variable?",
            
            // Time operations (SRFI-21 specific to avoid conflicts with SRFI-19)
            "srfi21:time->seconds", "srfi21:seconds->time",
        ];
        
        for func_name in &threading_functions {
            exports.insert(
                func_name.to_string(),
                Value::Symbol(intern_symbol(func_name.to_string()))
            );
        }
        
        exports
    }
    
    /// Evaluates source code in an isolated environment.
    fn evaluate_source_code(
        &self,
        module_id: &ModuleId,
        source_code: &str,
        evaluator: &mut Evaluator,
    ) -> Result<HashMap<String, Value>> {
        eprintln!("DEBUG: evaluate_source_code called for module: {:?}", module_id);
        
        // Parse the source code
        let filename = format!("{:?}", module_id);
        let mut lexer = Lexer::new(source_code, Some(&filename));
        let tokens = lexer.tokenize().map_err(|e| Box::new(Error::syntax_error(
            format!("Failed to tokenize module {}: {}", filename, e),
            None
        )))?;
        
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| Box::new(Error::syntax_error(
            format!("Failed to parse module {}: {}", filename, e),
            None
        )))?;

        // Run prescan to detect and register macros in this module
        if let Ok(prescan_results) = crate::macro_system::prescan::prescan_ast(&program) {
            eprintln!("DEBUG: Module {} prescan found {} define-syntax forms", 
                     filename, prescan_results.define_syntax_count);
            
            // Get the global macro expander and register detected macros
            let macro_expander = evaluator.macro_expander_mut();
            if let Ok(registered_count) = crate::macro_system::prescan::register_prescanned_macros(
                macro_expander, 
                &prescan_results
            ) {
                eprintln!("DEBUG: Successfully registered {} macros from module {}", 
                         registered_count, filename);
            } else {
                eprintln!("DEBUG: Failed to register macros from module {}", filename);
            }
        }

        let mut exports = HashMap::new();
        
        // Look for define-library forms
        for expr in &program.expressions {
            eprintln!("DEBUG: Processing expression: {:?}", expr.inner);
            if let Expr::List(items) = &**expr {
                if let Some(first_item) = items.first() {
                    if let Expr::Symbol(sym) = &**first_item {
                        eprintln!("DEBUG: Found symbol: {}", sym.as_str());
                        if sym.as_str() == "define-library" {
                            eprintln!("DEBUG: Found define-library form with {} items", items.len());
                            // Extract exports from the define-library form
                            if let Some(library_exports) = self.extract_library_exports(&items[1..], evaluator)? {
                                eprintln!("DEBUG: Extracted {} exports from define-library", library_exports.len());
                                exports.extend(library_exports);
                            }
                        }
                    }
                }
            }
        }
        
        // If no define-library form found, try to evaluate as regular Scheme code
        if exports.is_empty() {
            let global_env = crate::eval::global_environment();
            
            // For regular Scheme files, execute all expressions and collect global definitions
            for expr in &program.expressions {
                let result = evaluator.eval(expr, global_env.clone())?;
                
                // Check for top-level definitions and add them to exports
                if let Expr::List(items) = &**expr {
                    if let Some(first_item) = items.first() {
                        if let Expr::Symbol(sym) = &**first_item {
                            if sym.as_str() == "define" && items.len() >= 3 {
                                if let Some(name_item) = items.get(1) {
                                    if let Expr::Symbol(name) = &**name_item {
                                        exports.insert(name.as_str().to_string(), result);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(exports)
    }
    
    /// Extracts exports from a define-library form.
    fn extract_library_exports(&self, library_body: &[Spanned<Expr>], evaluator: &mut Evaluator) -> Result<Option<HashMap<String, Value>>> {
        let mut exports = HashMap::new();
        let mut export_list = Vec::new();
        let global_env = crate::eval::global_environment();
        
        // Parse the library body to find export declarations and begin clauses
        for clause in library_body {
            if let Expr::List(items) = &**clause {
                if let Some(first_item) = items.first() {
                    if let Expr::Symbol(keyword) = &**first_item {
                        match keyword.as_str() {
                            "export" => {
                                // Collect all exported symbols
                                for export_item in &items[1..] {
                                    if let Expr::Symbol(sym) = &**export_item {
                                        export_list.push(sym.as_str().to_string());
                                    }
                                }
                            }
                            "begin" => {
                                // Evaluate the begin clause to populate the environment
                                for expr in &items[1..] {
                                    let result = evaluator.eval(expr, global_env.clone())?;
                                    
                                    // If this is a definition, add it to our potential exports
                                    if let Expr::List(def_items) = &**expr {
                                        if let Some(first_def_item) = def_items.first() {
                                            if let Expr::Symbol(def_keyword) = &**first_def_item {
                                                // Handle both define and define-syntax forms
                                                let is_define = def_keyword.as_str() == "define" && def_items.len() >= 3;
                                                let is_define_syntax = def_keyword.as_str() == "define-syntax" && def_items.len() >= 3;
                                                
                                                if is_define || is_define_syntax {
                                                    if let Some(name_item) = def_items.get(1) {
                                                        if let Expr::Symbol(name) = &**name_item {
                                                            // Only include if it's in the export list
                                                            let name_str = name.as_str().to_string();
                                                            if export_list.contains(&name_str) {
                                                                if is_define_syntax {
                                                                    // For define-syntax, the macro is registered in the macro expander
                                                                    // Macros should not be exported as runtime values - they are compile-time only
                                                                    // So we don't add anything to exports for define-syntax
                                                                    eprintln!("DEBUG: Skipping export for macro {}", name_str);
                                                                } else {
                                                                    // For regular define, use the evaluation result
                                                                    exports.insert(name_str, result);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {
                                // Ignore other clauses for now (import, include, etc.)
                            }
                        }
                    }
                }
            }
        }
        
        // For exports that weren't defined in begin clauses, look up from global environment
        for export_name in export_list {
            if !exports.contains_key(&export_name) {
                // Try to get from the global environment (this includes Rust primitives)
                match global_env.lookup(&export_name) {
                    Some(value) => {
                        exports.insert(export_name, value);
                    }
                    None => {
                        // If not in global env, try to get it from the evaluator's built-in functions
                        // Many R7RS base functions are implemented as Rust primitives
                        if Self::is_r7rs_primitive(&export_name) {
                            // Create a placeholder that indicates this is a primitive function
                            exports.insert(export_name.clone(), Value::Symbol(intern_symbol(export_name)));
                        } else {
                            // Log that we couldn't find this export
                            eprintln!("DEBUG: Could not find export '{}' in global environment", export_name);
                        }
                    }
                }
            }
        }

        if exports.is_empty() {
            Ok(None)
        } else {
            Ok(Some(exports))
        }
    }
    
    /// Checks if a function name is a known R7RS primitive that should be available
    fn is_r7rs_primitive(name: &str) -> bool {
        // Common R7RS base primitives that are implemented in Rust
        match name {
            // Numeric functions
            "abs" | "min" | "max" | "zero?" | "positive?" | "negative?" | "odd?" | "even?" |
            "exact?" | "inexact?" | "finite?" | "infinite?" | "nan?" | "floor" | "ceiling" |
            "truncate" | "round" | "sqrt" | "expt" | "gcd" | "lcm" | "numerator" | "denominator" |
            
            // String functions
            "string-length" | "string-append" | "string-ref" | "string?" | "string=?" |
            "string<?" | "string>?" | "string<=?" | "string>=?" | "make-string" | "string-copy" |
            
            // List functions
            "length" | "reverse" | "append" | "list-ref" | "list-tail" | "list?" | "pair?" | 
            "null?" | "cons" | "car" | "cdr" | "caar" | "cadr" | "cdar" | "cddr" | "caddr" | 
            "cadddr" | "memq" | "memv" | "member" | "assq" | "assv" | "assoc" |
            
            // Vector functions  
            "vector?" | "make-vector" | "vector" | "vector-length" | "vector-ref" | "vector-set!" |
            "vector->list" | "list->vector" |
            
            // Character functions
            "char?" | "char=?" | "char<?" | "char>?" | "char<=?" | "char>=?" |
            "char->integer" | "integer->char" | "char-upcase" | "char-downcase" |
            
            // Symbol functions
            "symbol?" | "symbol->string" | "string->symbol" |
            
            // Boolean functions
            "boolean?" | "not" |
            
            // Equivalence predicates
            "eq?" | "eqv?" | "equal?" |
            
            // Type predicates
            "number?" | "complex?" | "real?" | "rational?" | "integer?" | "procedure?" |
            
            // Control features
            "apply" | "map" | "for-each" | "call-with-current-continuation" | "call/cc" |
            "values" | "call-with-values" |
            
            // I/O functions
            "display" | "write" | "newline" | "read" | "write-char" | "read-char" |
            "input-port?" | "output-port?" | "current-input-port" | "current-output-port" |
            "current-error-port" | "open-input-file" | "open-output-file" | "close-input-port" |
            "close-output-port" |
            
            // System functions
            "error" | "exit" => true,
            
            _ => false
        }
    }
}

/// Direct implementation of SRFI-1 take function
fn srfi1_take_direct(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("take: expected 2 arguments, got {}", args.len()),
            None
        )));
    }
    
    let list = &args[0];
    let n = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => *n as usize,
        _ => return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "take: second argument must be an exact integer".to_string(),
            None
        )))
    };
    
    let mut result = Vec::new();
    let mut current = list;
    let mut count = 0;
    
    while count < n {
        match current {
            Value::Pair(car, cdr) => {
                result.push((**car).clone());
                current = cdr;
                count += 1;
            }
            Value::Nil => break,
            _ => return Err(Box::new(crate::diagnostics::Error::runtime_error(
                "take: first argument must be a proper list".to_string(),
                None
            )))
        }
    }
    
    // Build result list from vector
    let mut result_list = Value::Nil;
    for item in result.into_iter().rev() {
        result_list = Value::Pair(Box::new(item), Box::new(result_list));
    }
    
    Ok(result_list)
}

/// Direct implementation of SRFI-1 drop function  
fn srfi1_drop_direct(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("drop: expected 2 arguments, got {}", args.len()),
            None
        )));
    }
    
    let list = &args[0];
    let n = match &args[1] {
        Value::Literal(crate::ast::Literal::ExactInteger(n)) => *n as usize,
        _ => return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "drop: second argument must be an exact integer".to_string(),
            None
        )))
    };
    
    let mut current = list.clone();
    let mut count = 0;
    
    while count < n {
        match &current {
            Value::Pair(_car, cdr) => {
                current = (**cdr).clone();
                count += 1;
            }
            Value::Nil => break,
            _ => return Err(Box::new(crate::diagnostics::Error::runtime_error(
                "drop: first argument must be a proper list".to_string(),
                None
            )))
        }
    }
    
    Ok(current)
}

/// Direct implementation of SRFI-1 last function
fn srfi1_last_direct(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("last: expected 1 argument, got {}", args.len()),
            None
        )));
    }
    
    let list = &args[0];
    let mut current = list;
    let mut last_value = None;
    
    loop {
        match current {
            Value::Pair(car, cdr) => {
                last_value = Some((**car).clone());
                match &**cdr {
                    Value::Nil => break,
                    _ => current = cdr,
                }
            }
            Value::Nil => {
                return Err(Box::new(crate::diagnostics::Error::runtime_error(
                    "last: cannot get last element of empty list".to_string(),
                    None
                )));
            }
            _ => return Err(Box::new(crate::diagnostics::Error::runtime_error(
                "last: argument must be a proper list".to_string(),
                None
            )))
        }
    }
    
    last_value.ok_or_else(|| Box::new(crate::diagnostics::Error::runtime_error(
        "last: cannot get last element of empty list".to_string(),
        None
    )))
}

/// Direct Rust implementation for SRFI-23 error procedure.
/// 
/// The error procedure raises an exception with a message and optional irritants.
/// Syntax: (error message [irritant1 irritant2 ...])
/// 
/// This implementation creates a structured error object and raises it as an exception.
fn srfi23_error_direct(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "error: expected at least 1 argument, got 0".to_string(),
            None
        )));
    }
    
    // First argument must be the error message (typically a string)
    let message = match &args[0] {
        Value::Literal(crate::ast::Literal::String(s)) => (**s).clone(),
        Value::Symbol(s) => s.to_string(),
        other => format!("{:?}", other), // Convert other types to string representation
    };
    
    // Collect irritants (remaining arguments)
    let irritants: Vec<Value> = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        Vec::new()
    };
    
    // Create a comprehensive error message
    let full_message = if irritants.is_empty() {
        message.clone()
    } else {
        let irritant_strings: Vec<String> = irritants.iter()
            .map(|v| match v {
                Value::Literal(crate::ast::Literal::String(s)) => (**s).clone(),
                Value::Symbol(s) => s.to_string(),
                Value::Literal(lit) => format!("{:?}", lit),
                other => format!("{:?}", other),
            })
            .collect();
        format!("{}: {}", message, irritant_strings.join(" "))
    };
    
    // Create a structured SRFI-23 error object
    // Using a vector to represent: ['srfi-23-error, message, irritants, full-message]
    let _error_object = Value::vector(vec![
        Value::Symbol(intern_symbol("srfi-23-error".to_string())),
        Value::Literal(crate::ast::Literal::String(Box::new(message))),
        Value::list(irritants),
        Value::Literal(crate::ast::Literal::String(Box::new(full_message.clone()))),
    ]);
    
    // Raise the error as a runtime exception
    // The error message includes both the message and irritants for diagnostics
    Err(Box::new(crate::diagnostics::Error::runtime_error(
        full_message,
        None
    )))
}

/// Direct Rust implementation for SRFI-6 open-input-string procedure.
/// 
/// Creates an input port that reads from the given string.
/// Syntax: (open-input-string string)
fn srfi6_open_input_string_direct(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("open-input-string: expected 1 argument, got {}", args.len()),
            None
        )));
    }
    
    let string = match &args[0] {
        Value::Literal(crate::ast::Literal::String(s)) => (**s).clone(),
        _ => return Err(Box::new(crate::diagnostics::Error::runtime_error(
            "open-input-string: argument must be a string".to_string(),
            None
        )))
    };
    
    // Create a string input port
    use crate::eval::value::Port;
    let port = Port::new_string_input(string);
    Ok(Value::Port(Arc::new(port)))
}

/// Direct Rust implementation for SRFI-6 open-output-string procedure.
/// 
/// Creates an output port that accumulates written data into a string.
/// Syntax: (open-output-string)
fn srfi6_open_output_string_direct(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("open-output-string: expected 0 arguments, got {}", args.len()),
            None
        )));
    }
    
    // Create a string output port
    use crate::eval::value::Port;
    let port = Port::new_string_output();
    Ok(Value::Port(Arc::new(port)))
}

/// Direct Rust implementation for SRFI-6 get-output-string procedure.
/// 
/// Extracts the accumulated string from a string output port.
/// Syntax: (get-output-string port)
fn srfi6_get_output_string_direct(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(crate::diagnostics::Error::runtime_error(
            format!("get-output-string: expected 1 argument, got {}", args.len()),
            None
        )));
    }
    
    match &args[0] {
        Value::Port(port) => {
            // Check if this is a string output port
            use crate::eval::value::{PortImpl, PortDirection};
            match (&port.implementation, &port.direction) {
                (PortImpl::String { content, .. }, &PortDirection::Output) => {
                    let content_str = content.borrow().clone();
                    Ok(Value::Literal(crate::ast::Literal::String(Box::new(content_str))))
                }
                _ => Err(Box::new(crate::diagnostics::Error::runtime_error(
                    "get-output-string: argument must be a string output port".to_string(),
                    None
                )))
            }
        }
        _ => Err(Box::new(crate::diagnostics::Error::runtime_error(
            "get-output-string: argument must be a string output port".to_string(),
            None
        )))
    }
}
//! SRFI 138: Compiling Scheme to Machine Code
//!
//! This SRFI defines a framework for compiling Scheme expressions to machine code.
//! It provides compilation contexts, compiler procedures, and integration with
//! the evaluator for transparent compilation support.

use crate::builtins::utils::{check_arity, make_builtin_procedure};
use crate::error::{LambdustError, Result};
use crate::value::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Compilation target for machine code generation
#[derive(Debug, Clone, PartialEq)]
pub enum CompilationTarget {
    /// Native x86-64 machine code
    X8664,
    /// Native ARM64 machine code
    Arm64,
    /// LLVM intermediate representation
    LlvmIr,
    /// WebAssembly bytecode
    WebAssembly,
    /// Custom target specification
    Custom(String),
}

/// Optimization level for compilation
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    /// No optimization - debug mode
    None,
    /// Basic optimizations
    Basic,
    /// Standard optimizations
    Standard,
    /// Aggressive optimizations
    Aggressive,
    /// Maximum optimization - may increase compile time significantly
    Maximum,
}

/// Compilation context containing target and options
#[derive(Debug, Clone)]
pub struct CompilationContext {
    /// Compilation target
    pub target: CompilationTarget,
    /// Optimization level
    pub optimization: OptimizationLevel,
    /// Debug information inclusion
    pub debug_info: bool,
    /// Inline threshold for function calls
    pub inline_threshold: usize,
    /// Loop unrolling factor
    pub unroll_factor: usize,
    /// Custom compilation flags
    pub flags: HashMap<String, String>,
}

impl CompilationContext {
    /// Create a new compilation context with default settings
    pub fn new(target: CompilationTarget) -> Self {
        Self {
            target,
            optimization: OptimizationLevel::Standard,
            debug_info: true,
            inline_threshold: 50,
            unroll_factor: 4,
            flags: HashMap::new(),
        }
    }

    /// Set optimization level
    pub fn with_optimization(mut self, level: OptimizationLevel) -> Self {
        self.optimization = level;
        self
    }

    /// Enable or disable debug information
    pub fn with_debug_info(mut self, debug: bool) -> Self {
        self.debug_info = debug;
        self
    }

    /// Set inline threshold
    pub fn with_inline_threshold(mut self, threshold: usize) -> Self {
        self.inline_threshold = threshold;
        self
    }

    /// Add custom compilation flag
    pub fn with_flag(mut self, key: String, value: String) -> Self {
        self.flags.insert(key, value);
        self
    }
}

/// Compiled code representation
#[derive(Debug, Clone)]
pub struct CompiledCode {
    /// Unique identifier for this compiled code
    pub id: usize,
    /// Target architecture
    pub target: CompilationTarget,
    /// Machine code bytes (or IR representation)
    pub code: Vec<u8>,
    /// Entry point offset
    pub entry_point: usize,
    /// Symbol table for debugging
    pub symbols: HashMap<String, usize>,
    /// Compilation metadata
    pub metadata: HashMap<String, String>,
}

impl CompiledCode {
    /// Create new compiled code instance
    pub fn new(target: CompilationTarget, code: Vec<u8>, entry_point: usize) -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        
        Self {
            id,
            target,
            code,
            entry_point,
            symbols: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Get code size in bytes
    pub fn size(&self) -> usize {
        self.code.len()
    }

    /// Add symbol to symbol table
    pub fn add_symbol(&mut self, name: String, offset: usize) {
        self.symbols.insert(name, offset);
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// Compiler implementation for SRFI 138
pub struct SchemeCompiler {
    /// Available compilation contexts
    contexts: HashMap<String, CompilationContext>,
    /// Compiled code cache
    compiled_cache: HashMap<String, CompiledCode>,
    /// Compilation statistics
    stats: CompilationStats,
}

/// Compilation statistics
#[derive(Debug, Clone, Default)]
pub struct CompilationStats {
    /// Number of expressions compiled
    pub expressions_compiled: usize,
    /// Total compilation time (microseconds)
    pub total_compile_time_us: u64,
    /// Cache hits
    pub cache_hits: usize,
    /// Cache misses
    pub cache_misses: usize,
    /// Average code size per expression
    pub average_code_size: f64,
}

impl SchemeCompiler {
    /// Create new compiler instance
    pub fn new() -> Self {
        let mut contexts = HashMap::new();
        
        // Add default contexts for common targets
        contexts.insert(
            "x86-64-opt".to_string(),
            CompilationContext::new(CompilationTarget::X8664)
                .with_optimization(OptimizationLevel::Standard)
        );
        
        contexts.insert(
            "arm64-opt".to_string(),
            CompilationContext::new(CompilationTarget::Arm64)
                .with_optimization(OptimizationLevel::Standard)
        );
        
        contexts.insert(
            "llvm-debug".to_string(),
            CompilationContext::new(CompilationTarget::LlvmIr)
                .with_optimization(OptimizationLevel::None)
                .with_debug_info(true)
        );

        Self {
            contexts,
            compiled_cache: HashMap::new(),
            stats: CompilationStats::default(),
        }
    }

    /// Add compilation context
    pub fn add_context(&mut self, name: String, context: CompilationContext) {
        self.contexts.insert(name, context);
    }

    /// Compile Scheme expression to machine code (placeholder implementation)
    pub fn compile_expression(&mut self, expr: &Value, context_name: &str) -> Result<CompiledCode> {
        let context = self.contexts.get(context_name)
            .ok_or_else(|| LambdustError::runtime_error(
                format!("Unknown compilation context: {}", context_name)
            ))?;

        // Generate cache key
        let cache_key = format!("{:?}:{}", expr, context_name);
        
        // Check cache first
        if let Some(cached) = self.compiled_cache.get(&cache_key) {
            self.stats.cache_hits += 1;
            return Ok(cached.clone());
        }
        
        self.stats.cache_misses += 1;
        
        // Placeholder compilation - in real implementation this would:
        // 1. Parse the Scheme expression into an IR
        // 2. Perform optimizations based on context.optimization
        // 3. Generate machine code for context.target
        // 4. Apply target-specific optimizations
        
        let placeholder_code = self.generate_placeholder_code(expr, context)?;
        
        // Cache the result
        self.compiled_cache.insert(cache_key, placeholder_code.clone());
        self.stats.expressions_compiled += 1;
        
        Ok(placeholder_code)
    }

    /// Generate placeholder machine code
    fn generate_placeholder_code(&self, expr: &Value, context: &CompilationContext) -> Result<CompiledCode> {
        // Placeholder implementation - generates mock bytecode
        let mut code = Vec::new();
        
        match &context.target {
            CompilationTarget::X8664 => {
                // Mock x86-64 assembly: mov rax, immediate; ret
                code.extend_from_slice(&[0x48, 0xB8]); // mov rax, imm64
                code.extend_from_slice(&[0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // placeholder value
                code.extend_from_slice(&[0xC3]); // ret
            }
            CompilationTarget::Arm64 => {
                // Mock ARM64 assembly: mov x0, immediate; ret
                code.extend_from_slice(&[0x00, 0x08, 0x80, 0xD2]); // mov x0, #0x40
                code.extend_from_slice(&[0xC0, 0x03, 0x5F, 0xD6]); // ret
            }
            CompilationTarget::LlvmIr => {
                // Mock LLVM IR as bytes (would normally be text)
                let ir = format!("define i64 @expr() {{ ret i64 {:?} }}", expr);
                code.extend_from_slice(ir.as_bytes());
            }
            CompilationTarget::WebAssembly => {
                // Mock WebAssembly bytecode
                code.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]); // WASM magic
                code.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // version
            }
            CompilationTarget::Custom(name) => {
                let custom_code = format!("CUSTOM-{}-{:?}", name, expr);
                code.extend_from_slice(custom_code.as_bytes());
            }
        }
        
        let mut compiled = CompiledCode::new(context.target.clone(), code, 0);
        compiled.add_metadata("optimization".to_string(), format!("{:?}", context.optimization));
        compiled.add_metadata("debug_info".to_string(), context.debug_info.to_string());
        compiled.add_metadata("source_expr".to_string(), format!("{:?}", expr));
        
        Ok(compiled)
    }

    /// Get compilation statistics
    pub fn get_stats(&self) -> &CompilationStats {
        &self.stats
    }

    /// Clear compilation cache
    pub fn clear_cache(&mut self) {
        self.compiled_cache.clear();
    }
}

impl Default for SchemeCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Create compilation context procedure
fn make_compilation_context_proc(args: &[Value]) -> Result<Value> {
    check_arity(args, 1)?;
    
    let target = match &args[0] {
        Value::Symbol(s) => match s.as_str() {
            "x86-64" => CompilationTarget::X8664,
            "arm64" => CompilationTarget::Arm64,
            "llvm-ir" => CompilationTarget::LlvmIr,
            "wasm" => CompilationTarget::WebAssembly,
            _ => CompilationTarget::Custom(s.clone()),
        },
        Value::String(s) => CompilationTarget::Custom(s.clone()),
        _ => return Err(LambdustError::type_error(
            "Expected symbol or string for compilation target".to_string()
        )),
    };
    
    let _context = CompilationContext::new(target);
    
    // Return context as an opaque value (in real implementation would be a proper type)
    Ok(Value::Vector(vec![
        Value::Symbol("compilation-context".to_string()),
        args[0].clone(),
    ]))
}

/// Compile expression procedure
fn compile_expression_proc(args: &[Value]) -> Result<Value> {
    check_arity(args, 2)?;
    
    let _expr = &args[0];
    let _context = &args[1];
    
    // Placeholder implementation - would integrate with actual compiler
    Ok(Value::Vector(vec![
        Value::Symbol("compiled-code".to_string()),
        Value::Number(crate::lexer::SchemeNumber::Integer(1234)), // mock code ID
        Value::String("x86-64".to_string()), // mock target
    ]))
}

/// Execute compiled code procedure
fn execute_compiled_proc(args: &[Value]) -> Result<Value> {
    check_arity(args, 1)?;
    
    // Placeholder - would execute actual machine code
    match &args[0] {
        Value::Vector(v) if v.len() >= 2 && 
            matches!(&v[0], Value::Symbol(s) if s == "compiled-code") => {
            // Mock execution result
            Ok(Value::Number(crate::lexer::SchemeNumber::Integer(42)))
        }
        _ => Err(LambdustError::type_error(
            "Expected compiled code object".to_string()
        )),
    }
}

/// SRFI 138 implementation
pub struct Srfi138;

impl super::SrfiModule for Srfi138 {
    fn srfi_id(&self) -> u32 {
        138
    }

    fn name(&self) -> &'static str {
        "Compiling Scheme to Machine Code"
    }

    fn parts(&self) -> Vec<&'static str> {
        vec!["compile", "execute"]
    }

    fn exports(&self) -> HashMap<String, Value> {
        let mut exports = HashMap::new();

        // Core compilation procedures
        exports.insert(
            "make-compilation-context".to_string(),
            make_builtin_procedure("make-compilation-context", Some(1), make_compilation_context_proc),
        );

        exports.insert(
            "compile-expression".to_string(),
            make_builtin_procedure("compile-expression", Some(2), compile_expression_proc),
        );

        exports.insert(
            "execute-compiled".to_string(),
            make_builtin_procedure("execute-compiled", Some(1), execute_compiled_proc),
        );

        exports
    }

    fn exports_for_parts(&self, parts: &[&str]) -> Result<HashMap<String, Value>> {
        let mut exports = HashMap::new();
        
        if parts.contains(&"compile") {
            exports.insert(
                "make-compilation-context".to_string(),
                make_builtin_procedure("make-compilation-context", Some(1), make_compilation_context_proc),
            );
            exports.insert(
                "compile-expression".to_string(),
                make_builtin_procedure("compile-expression", Some(2), compile_expression_proc),
            );
        }
        
        if parts.contains(&"execute") {
            exports.insert(
                "execute-compiled".to_string(),
                make_builtin_procedure("execute-compiled", Some(1), execute_compiled_proc),
            );
        }
        
        Ok(exports)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srfi::SrfiModule;

    #[test]
    fn test_compilation_context_creation() {
        let context = CompilationContext::new(CompilationTarget::X8664)
            .with_optimization(OptimizationLevel::Aggressive)
            .with_debug_info(false)
            .with_inline_threshold(100);
            
        assert_eq!(context.target, CompilationTarget::X8664);
        assert_eq!(context.optimization, OptimizationLevel::Aggressive);
        assert!(!context.debug_info);
        assert_eq!(context.inline_threshold, 100);
    }

    #[test]
    fn test_compiled_code_creation() {
        let code = vec![0x48, 0xB8, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC3];
        let mut compiled = CompiledCode::new(CompilationTarget::X8664, code.clone(), 0);
        compiled.add_symbol("main".to_string(), 0);
        compiled.add_metadata("test".to_string(), "value".to_string());
        
        assert_eq!(compiled.target, CompilationTarget::X8664);
        assert_eq!(compiled.code, code);
        assert_eq!(compiled.size(), code.len());
        assert_eq!(compiled.symbols.get("main"), Some(&0));
        assert_eq!(compiled.metadata.get("test"), Some(&"value".to_string()));
    }

    #[test]
    fn test_compiler_basic_functionality() {
        let mut compiler = SchemeCompiler::new();
        let expr = Value::Number(crate::lexer::SchemeNumber::Integer(42));
        
        let result = compiler.compile_expression(&expr, "x86-64-opt");
        assert!(result.is_ok());
        
        let compiled_output = result.unwrap();
        assert_eq!(compiled_output.target, CompilationTarget::X8664);
        assert!(compiled_output.size() > 0);
    }

    #[test]
    fn test_compilation_caching() {
        let mut compiler = SchemeCompiler::new();
        let expr = Value::Number(crate::lexer::SchemeNumber::Integer(42));
        
        // First compilation
        let result1 = compiler.compile_expression(&expr, "x86-64-opt").unwrap();
        let initial_cache_misses = compiler.get_stats().cache_misses;
        
        // Second compilation should use cache
        let result2 = compiler.compile_expression(&expr, "x86-64-opt").unwrap();
        assert_eq!(result1.id, result2.id);
        assert!(compiler.get_stats().cache_hits > 0);
        assert_eq!(compiler.get_stats().cache_misses, initial_cache_misses);
    }

    #[test]
    fn test_srfi_exports() {
        let srfi = Srfi138;
        let exports = srfi.exports();
        
        assert!(exports.contains_key("make-compilation-context"));
        assert!(exports.contains_key("compile-expression"));
        assert!(exports.contains_key("execute-compiled"));
        
        // Test parts exports
        let compile_exports = srfi.exports_for_parts(&["compile"]).unwrap();
        assert!(compile_exports.contains_key("make-compilation-context"));
        assert!(compile_exports.contains_key("compile-expression"));
        assert!(!compile_exports.contains_key("execute-compiled"));
        
        let execute_exports = srfi.exports_for_parts(&["execute"]).unwrap();
        assert!(!execute_exports.contains_key("make-compilation-context"));
        assert!(!execute_exports.contains_key("compile-expression"));
        assert!(execute_exports.contains_key("execute-compiled"));
    }

    #[test]
    fn test_different_targets() {
        let mut compiler = SchemeCompiler::new();
        let expr = Value::Symbol("test".to_string());
        
        // Test different compilation targets
        compiler.add_context(
            "arm64-test".to_string(),
            CompilationContext::new(CompilationTarget::Arm64)
        );
        
        compiler.add_context(
            "llvm-test".to_string(),
            CompilationContext::new(CompilationTarget::LlvmIr)
        );
        
        let x86_result = compiler.compile_expression(&expr, "x86-64-opt").unwrap();
        let arm_result = compiler.compile_expression(&expr, "arm64-test").unwrap();
        let llvm_result = compiler.compile_expression(&expr, "llvm-test").unwrap();
        
        assert_eq!(x86_result.target, CompilationTarget::X8664);
        assert_eq!(arm_result.target, CompilationTarget::Arm64);
        assert_eq!(llvm_result.target, CompilationTarget::LlvmIr);
        
        // Different targets should produce different code
        assert_ne!(x86_result.code, arm_result.code);
        assert_ne!(arm_result.code, llvm_result.code);
    }
}
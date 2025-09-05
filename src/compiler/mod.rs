//! Native Binary Compilation System for Lambdust
//!
//! This module implements a comprehensive native code compiler for Lambdust Scheme
//! that achieves self-hosting capability while maintaining full R7RS compliance.
//!
//! ## Architecture Overview
//!
//! ```text
//! Source (.scm) → [Frontend] → [Middle-end] → [Backend] → Native Binary
//!     ↓              ↓            ↓           ↓
//! Lexical     Semantic    CPS/A-Normal  Native Code
//! Analysis    Analysis    Form IR       Generation
//! ```
//!
//! ## Key Features
//! - **Self-hosting**: Lambdust compiler written in Lambdust
//! - **Multi-tier compilation**: O0-O3 optimization levels
//! - **Cross-platform**: x86_64, ARM64, RISC-V support
//! - **Phase 5 integration**: JIT/parallel/distributed/security
//! - **Continuation support**: Native call/cc and dynamic-wind
//! - **GC integration**: Safe native code generation
//! - **R7RS compliance**: Complete standard library compilation
//!
//! ## Compilation Phases
//!
//! ### Phase 1: Frontend (Parser → AST)
//! - Lexical analysis with R7RS token support
//! - Syntax analysis producing clean AST
//! - Macro expansion (hygiene-preserving)
//! - Initial semantic analysis
//!
//! ### Phase 2: Middle-end (AST → IR)
//! - CPS (Continuation Passing Style) conversion
//! - Closure conversion and lambda lifting
//! - Type inference and flow analysis
//! - Optimization passes (inlining, specialization)
//!
//! ### Phase 3: Backend (IR → Native)
//! - Register allocation and instruction selection
//! - Target-specific code generation
//! - Linking and symbol resolution
//! - Runtime system integration

pub mod backend;
pub mod frontend;
pub mod ir;
pub mod middle_end;
pub mod optimization;
pub mod runtime_integration;
pub mod self_hosting;
pub mod targets;

use crate::ast::{Expr, Program};
use crate::diagnostics::{Error, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Native compilation configuration
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Optimization level (O0, O1, O2, O3)
    pub optimization_level: OptimizationLevel,
    /// Target architecture
    pub target_arch: TargetArchitecture,
    /// Enable debug information
    pub debug_info: bool,
    /// Enable profile-guided optimization
    pub profile_guided: bool,
    /// Self-hosting mode
    pub self_hosting: bool,
    /// Phase 5 integration settings
    pub phase5_integration: Phase5Config,
}

/// Optimization levels for compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptimizationLevel {
    /// No optimization - fast compilation
    O0,
    /// Basic optimization - function inlining, dead code elimination
    O1,
    /// Advanced optimization - loop optimization, register allocation
    O2,
    /// Aggressive optimization - whole program optimization, SIMD
    O3,
}

/// Target architectures supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetArchitecture {
    /// x86_64 (Intel/AMD 64-bit)
    X86_64,
    /// ARM64 (Apple Silicon, ARM v8)
    ARM64,
    /// RISC-V 64-bit
    RISCV64,
    /// WebAssembly (for browser/cloud deployment)
    WASM,
}

/// Phase 5 integration configuration
#[derive(Debug, Clone)]
pub struct Phase5Config {
    /// Enable JIT integration (Stage 1)
    pub jit_integration: bool,
    /// Enable parallel compilation (Stage 2)
    pub parallel_compilation: bool,
    /// Enable distributed compilation (Stage 3)
    pub distributed_compilation: bool,
    /// Enable security framework (Stage 4)
    pub security_framework: bool,
    /// Enable performance monitoring (Stage 5)
    pub performance_monitoring: bool,
}

/// Native binary compilation result
#[derive(Debug)]
pub struct CompilationResult {
    /// Generated native binary
    pub binary: Vec<u8>,
    /// Symbol table for debugging
    pub symbols: HashMap<String, u64>,
    /// Compilation metadata
    pub metadata: CompilationMetadata,
}

/// Metadata about the compilation process
#[derive(Debug)]
pub struct CompilationMetadata {
    /// Time spent in each compilation phase
    pub phase_times: HashMap<String, std::time::Duration>,
    /// Number of functions compiled
    pub functions_compiled: usize,
    /// Optimizations applied
    pub optimizations_applied: Vec<String>,
    /// Memory usage statistics
    pub memory_usage: MemoryUsage,
}

/// Memory usage statistics during compilation
#[derive(Debug)]
pub struct MemoryUsage {
    /// Peak memory usage in bytes
    pub peak_memory: usize,
    /// Final binary size in bytes
    pub binary_size: usize,
    /// Code section size
    pub code_size: usize,
    /// Data section size
    pub data_size: usize,
}

/// Main native compiler
pub struct NativeCompiler<'a> {
    /// Compiler configuration
    config: CompilerConfig,
    /// Frontend components
    frontend: Arc<frontend::Frontend<'a>>,
    /// Middle-end components
    middle_end: Arc<middle_end::MiddleEnd>,
    /// Backend components
    backend: Arc<backend::Backend>,
    /// Runtime integration
    runtime: Arc<runtime_integration::RuntimeIntegration>,
}

impl<'a> NativeCompiler<'a> {
    /// Creates a new native compiler with the given configuration
    pub fn new(config: CompilerConfig) -> Result<Self> {
        let frontend = Arc::new(frontend::Frontend::new(&config)?);
        let middle_end = Arc::new(middle_end::MiddleEnd::new(&config)?);
        let backend = Arc::new(backend::Backend::new(&config)?);
        let runtime = Arc::new(runtime_integration::RuntimeIntegration::new(&config)?);

        Ok(NativeCompiler {
            config,
            frontend,
            middle_end,
            backend,
            runtime,
        })
    }

    /// Compiles a Scheme program to native binary
    pub fn compile_program(&self, program: &Program) -> Result<CompilationResult> {
        let start_time = std::time::Instant::now();
        let mut metadata = CompilationMetadata {
            phase_times: HashMap::new(),
            functions_compiled: 0,
            optimizations_applied: Vec::new(),
            memory_usage: MemoryUsage {
                peak_memory: 0,
                binary_size: 0,
                code_size: 0,
                data_size: 0,
            },
        };

        // Phase 1: Frontend processing
        let frontend_start = std::time::Instant::now();
        let analyzed_program = {
            let mut temp_frontend = frontend::Frontend::new(&self.config)?;
            temp_frontend.analyze_program(program)?
        };
        metadata
            .phase_times
            .insert("frontend".to_string(), frontend_start.elapsed());

        // Phase 2: Middle-end transformation
        let middle_start = std::time::Instant::now();
        let ir_program = Arc::get_mut(&mut self.middle_end.clone())
            .unwrap()
            .transform_program(&analyzed_program)?;
        metadata
            .phase_times
            .insert("middle_end".to_string(), middle_start.elapsed());
        metadata.functions_compiled = ir_program.functions.len();

        // Phase 3: Backend code generation
        let backend_start = std::time::Instant::now();
        let mut binary = Arc::get_mut(&mut self.backend.clone())
            .unwrap()
            .generate_code(&ir_program)?;
        metadata
            .phase_times
            .insert("backend".to_string(), backend_start.elapsed());

        // Phase 4: Runtime integration
        let runtime_start = std::time::Instant::now();
        binary = self.runtime.integrate_runtime(binary)?;
        metadata
            .phase_times
            .insert("runtime".to_string(), runtime_start.elapsed());

        // Update metadata
        metadata
            .phase_times
            .insert("total".to_string(), start_time.elapsed());
        metadata.memory_usage.binary_size = binary.binary.len();
        metadata.optimizations_applied = self.get_applied_optimizations();

        Ok(CompilationResult {
            binary: binary.binary,
            symbols: binary.symbols,
            metadata,
        })
    }

    /// Compiles a single expression to native code (for REPL/testing)
    pub fn compile_expression(&self, expr: &Expr) -> Result<CompilationResult> {
        let program = Program {
            expressions: vec![crate::diagnostics::Spanned::new(
                expr.clone(),
                crate::diagnostics::Span::new(0, 0),
            )],
        };
        self.compile_program(&program)
    }

    /// Compiles from source file to native binary
    pub fn compile_file(&mut self, source_path: &Path, output_path: &Path) -> Result<()> {
        // Parse source file
        let source_code = std::fs::read_to_string(source_path)
            .map_err(|e| Error::io_error(format!("Failed to read source file: {}", e)))?;

        let program = {
            // For now, create a new frontend instance to parse source
            // This is a temporary solution - should use proper architecture
            let mut temp_frontend = frontend::Frontend::new(&self.config)?;
            temp_frontend.parse_source(&source_code)?
        };
        let result = self.compile_program(&program)?;

        // Write binary to output file
        std::fs::write(output_path, &result.binary)
            .map_err(|e| Error::io_error(format!("Failed to write binary file: {}", e)))?;

        Ok(())
    }

    /// Gets the list of optimizations that were applied
    fn get_applied_optimizations(&self) -> Vec<String> {
        let mut optimizations = Vec::new();

        match self.config.optimization_level {
            OptimizationLevel::O0 => {}
            OptimizationLevel::O1 => {
                optimizations.push("function_inlining".to_string());
                optimizations.push("dead_code_elimination".to_string());
            }
            OptimizationLevel::O2 => {
                optimizations.push("function_inlining".to_string());
                optimizations.push("dead_code_elimination".to_string());
                optimizations.push("loop_optimization".to_string());
                optimizations.push("register_allocation".to_string());
            }
            OptimizationLevel::O3 => {
                optimizations.push("function_inlining".to_string());
                optimizations.push("dead_code_elimination".to_string());
                optimizations.push("loop_optimization".to_string());
                optimizations.push("register_allocation".to_string());
                optimizations.push("whole_program_optimization".to_string());
                optimizations.push("simd_vectorization".to_string());
            }
        }

        if self.config.phase5_integration.jit_integration {
            optimizations.push("jit_integration".to_string());
        }

        optimizations
    }
}

impl Default for CompilerConfig {
    fn default() -> Self {
        CompilerConfig {
            optimization_level: OptimizationLevel::O2,
            target_arch: TargetArchitecture::X86_64,
            debug_info: false,
            profile_guided: false,
            self_hosting: false,
            phase5_integration: Phase5Config {
                jit_integration: true,
                parallel_compilation: true,
                distributed_compilation: false,
                security_framework: true,
                performance_monitoring: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_compiler_config_default() {
        let config = CompilerConfig::default();
        assert_eq!(config.optimization_level, OptimizationLevel::O2);
        assert_eq!(config.target_arch, TargetArchitecture::X86_64);
    }

    #[test]
    fn test_optimization_level_ordering() {
        assert!(OptimizationLevel::O0 < OptimizationLevel::O1);
        assert!(OptimizationLevel::O1 < OptimizationLevel::O2);
        assert!(OptimizationLevel::O2 < OptimizationLevel::O3);
    }
}

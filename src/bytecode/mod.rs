//! Bytecode compilation and virtual machine for Lambdust.
//!
//! This module provides a bytecode compiler and virtual machine that serves as
//! an intermediate representation for efficient execution and as a foundation
//! for future JIT compilation.

pub mod compiler;
pub mod vm;
pub mod optimizer;
pub mod instruction;
pub mod bytecode_engine;
pub mod compiler_stats;
pub mod vm_stats;
pub mod bytecode_performance_stats;
pub mod overall_performance_metrics;
pub mod optimization_config;
pub mod vm_config;

pub use compiler::{BytecodeCompiler, CompilerOptions, CompilationResult};
pub use vm::{VirtualMachine, VmState, ExecutionResult};
pub use optimizer::{BytecodeOptimizer, OptimizationPass, OptimizationStats};
pub use instruction::{Instruction, OpCode, Operand, ConstantPool, ConstantValue};
pub use bytecode_engine::*;
pub use compiler_stats::*;
pub use vm_stats::*;
pub use bytecode_performance_stats::*;
pub use overall_performance_metrics::*;
pub use optimization_config::*;
pub use vm_config::*;

use crate::ast::Program;
use crate::eval::Value;
use crate::diagnostics::Result;

/// Global bytecode engine instance for convenience.
static mut GLOBAL_ENGINE: Option<BytecodeEngine> = None;
static GLOBAL_ENGINE_INIT: std::sync::Once = std::sync::Once::new();

/// Gets the global bytecode engine, initializing it if necessary.
#[allow(static_mut_refs)]
pub fn global_bytecode_engine() -> &'static mut BytecodeEngine {
    unsafe {
        GLOBAL_ENGINE_INIT.call_once(|| {
            GLOBAL_ENGINE = Some(BytecodeEngine::new());
        });
        GLOBAL_ENGINE.as_mut().unwrap()
    }
}

/// Convenience function to compile and execute a program using the global engine.
pub fn execute_program(program: &Program) -> Result<Value> {
    global_bytecode_engine().compile_and_execute(program)
}

/// Convenience function to get performance statistics from the global engine.
pub fn get_global_performance_stats() -> BytecodePerformanceStats {
    global_bytecode_engine().get_performance_stats()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Literal};
    
    #[test]
    fn test_bytecode_engine_creation() {
        let engine = BytecodeEngine::new();
        let stats = engine.get_performance_stats();
        
        // New engine should have zero stats
        assert_eq!(stats.compiler.expressions_compiled, 0);
        assert_eq!(stats.vm.instructions_executed, 0);
    }
    
    #[test]
    fn test_performance_report_generation() {
        let engine = BytecodeEngine::new();
        let report = engine.generate_performance_report();
        
        assert!(report.contains("Lambdust Bytecode Engine Performance Report"));
        assert!(report.contains("Overall Performance"));
        assert!(report.contains("Compilation"));
        assert!(report.contains("Execution"));
    }
    
    #[test]
    fn test_optimization_config() {
        let mut engine = BytecodeEngine::new();
        let config = optimizer::OptimizationConfig {
            constant_folding: false,
            dead_code_elimination: false,
            tail_call_optimization: false,
            instruction_combining: false,
            register_allocation: false,
            max_passes: 1,
        };
        
        engine.configure_optimizations(config);
        // Configuration should be applied (would need access to internal state to verify)
    }
    
    #[test]
    fn test_vm_config() {
        let mut engine = BytecodeEngine::new();
        let config = vm::VmConfig {
            initial_stack_size: 512,
            max_stack_size: 8192,
            gc_enabled: false,
            gc_threshold: 500,
            profiling_enabled: true,
            debug_mode: true,
        };
        
        engine.configure_vm(config);
        // Configuration should be applied (would need access to internal state to verify)
    }
    
    #[test]
    fn test_global_engine() {
        let stats1 = get_global_performance_stats();
        let stats2 = get_global_performance_stats();
        
        // Should be the same global instance
        assert_eq!(stats1.compiler.expressions_compiled, stats2.compiler.expressions_compiled);
    }
}
//! Bytecode compilation and virtual machine for Lambdust.
//!
//! This module provides a bytecode compiler and virtual machine that serves as
//! an intermediate representation for efficient execution and as a foundation
//! for future JIT compilation.

pub mod compiler;
pub mod vm;
pub mod optimizer;
pub mod instruction;

pub use compiler::{BytecodeCompiler, CompilerOptions, CompilationResult};
pub use vm::{VirtualMachine, VmState, ExecutionResult};
pub use optimizer::{BytecodeOptimizer, OptimizationPass, OptimizationStats};
pub use instruction::{Instruction, OpCode, Operand, ConstantPool, ConstantValue};

use crate::ast::{Expr, Program};
use crate::eval::{Value, Environment};
use crate::diagnostics::{Result, Error};
use std::collections::HashMap;

/// High-level interface for bytecode compilation and execution.
pub struct BytecodeEngine {
    compiler: BytecodeCompiler,
    optimizer: BytecodeOptimizer,
    vm: VirtualMachine,
}

impl BytecodeEngine {
    /// Creates a new bytecode engine with default configuration.
    pub fn new() -> Self {
        Self {
            compiler: BytecodeCompiler::new(CompilerOptions::default()),
            optimizer: BytecodeOptimizer::new(),
            vm: VirtualMachine::new(),
        }
    }
    
    /// Compiles a program to optimized bytecode and executes it.
    pub fn compile_and_execute(&mut self, program: &Program) -> Result<Value> {
        // Compile to bytecode
        let compilation_result = self.compiler.compile_program(program)?;
        
        // Optimize bytecode
        let optimized_bytecode = self.optimizer.optimize(compilation_result.bytecode)?;
        
        // Execute on virtual machine
        let execution_result = self.vm.execute(&optimized_bytecode, &compilation_result.constant_pool)?;
        
        match execution_result {
            ExecutionResult::Value(value) => Ok(value),
            ExecutionResult::Error(error) => Err(error),
            ExecutionResult::Yield(_) => Err(Box::new(Error::runtime_error("Unexpected yield in top-level execution".to_string(), None)),
        }
    }
    
    /// Compiles an expression to bytecode without executing.
    pub fn compile_expression(&mut self, expr: &Expr) -> Result<CompilationResult> {
        self.compiler.compile_expression(expr)
    }
    
    /// Gets compilation statistics.
    pub fn get_compiler_stats(&self) -> CompilerStats {
        self.compiler.get_stats()
    }
    
    /// Gets optimization statistics.
    pub fn get_optimization_stats(&self) -> OptimizationStats {
        self.optimizer.get_stats()
    }
    
    /// Gets virtual machine statistics.
    pub fn get_vm_stats(&self) -> VmStats {
        self.vm.get_stats()
    }
}

impl Default for BytecodeEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about bytecode compilation.
#[derive(Debug, Clone)]
pub struct CompilerStats {
    /// Number of expressions compiled
    pub expressions_compiled: usize,
    /// Number of instructions generated
    pub instructions_generated: usize,
    /// Number of constants in pool
    pub constants_count: usize,
    /// Compilation time in microseconds
    pub compilation_time_us: u64,
    /// Memory usage during compilation
    pub memory_usage_bytes: usize,
}

/// Statistics about virtual machine execution.
#[derive(Debug, Clone)]
pub struct VmStats {
    /// Number of instructions executed
    pub instructions_executed: usize,
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Number of function calls
    pub function_calls: usize,
    /// Maximum stack depth reached
    pub max_stack_depth: usize,
    /// Number of garbage collections triggered
    pub gc_count: usize,
    /// Number of optimized operations executed
    pub optimized_operations: usize,
}

/// Comprehensive performance statistics for the bytecode system.
#[derive(Debug, Clone)]
pub struct BytecodePerformanceStats {
    /// Compiler statistics
    pub compiler: CompilerStats,
    /// Optimizer statistics
    pub optimizer: OptimizationStats,
    /// Virtual machine statistics
    pub vm: VmStats,
    /// Overall performance metrics
    pub overall: OverallPerformanceMetrics,
}

/// Overall performance metrics combining all subsystems.
#[derive(Debug, Clone)]
pub struct OverallPerformanceMetrics {
    /// Total time from compilation to execution completion
    pub total_time_us: u64,
    /// Instructions per second during execution
    pub instructions_per_second: f64,
    /// Memory efficiency score (0.0 to 1.0)
    pub memory_efficiency: f64,
    /// Optimization effectiveness score (0.0 to 1.0)
    pub optimization_effectiveness: f64,
    /// Comparison with interpreter performance
    pub speedup_factor: f64,
}

impl BytecodeEngine {
    /// Gets comprehensive performance statistics.
    pub fn get_performance_stats(&self) -> BytecodePerformanceStats {
        let compiler_stats = self.get_compiler_stats();
        let optimizer_stats = self.get_optimization_stats();
        let vm_stats = self.get_vm_stats();
        
        // Calculate overall metrics
        let total_time_us = compiler_stats.compilation_time_us + 
                           optimizer_stats.optimization_time_us + 
                           vm_stats.execution_time_us;
        
        let instructions_per_second = if vm_stats.execution_time_us > 0 {
            (vm_stats.instructions_executed as f64) / (vm_stats.execution_time_us as f64 / 1_000_000.0)
        } else {
            0.0
        };
        
        let memory_efficiency = if compiler_stats.memory_usage_bytes > 0 {
            1.0 - ((compiler_stats.memory_usage_bytes as f64) / (1024.0 * 1024.0)).min(1.0)
        } else {
            1.0
        };
        
        let optimization_effectiveness = if optimizer_stats.instructions_before > 0 {
            1.0 - (optimizer_stats.instructions_after as f64 / optimizer_stats.instructions_before as f64)
        } else {
            0.0
        };
        
        // Rough estimate of speedup compared to interpreter
        let speedup_factor = if vm_stats.optimized_operations > 0 {
            (vm_stats.optimized_operations as f64 / vm_stats.instructions_executed as f64) * 3.0 + 1.0
        } else {
            1.0
        };
        
        BytecodePerformanceStats {
            compiler: compiler_stats,
            optimizer: optimizer_stats,
            vm: vm_stats,
            overall: OverallPerformanceMetrics {
                total_time_us,
                instructions_per_second,
                memory_efficiency,
                optimization_effectiveness,
                speedup_factor,
            },
        }
    }
    
    /// Generates a performance report for the bytecode system.
    pub fn generate_performance_report(&self) -> String {
        let stats = self.get_performance_stats();
        let mut report = String::new();
        
        report.push_str("=== Lambdust Bytecode Engine Performance Report ===\n\n");
        
        // Overall metrics
        report.push_str("=== Overall Performance ===\n");
        report.push_str(&format!("Total Time: {:.2} ms\n", stats.overall.total_time_us as f64 / 1000.0));
        report.push_str(&format!("Instructions/Second: {:.0}\n", stats.overall.instructions_per_second));
        report.push_str(&format!("Memory Efficiency: {:.1}%\n", stats.overall.memory_efficiency * 100.0));
        report.push_str(&format!("Optimization Effectiveness: {:.1}%\n", stats.overall.optimization_effectiveness * 100.0));
        report.push_str(&format!("Speedup Factor: {:.2}x\n", stats.overall.speedup_factor));
        report.push_str("\n");
        
        // Compiler metrics
        report.push_str("=== Compilation ===\n");
        report.push_str(&format!("Expressions Compiled: {}\n", stats.compiler.expressions_compiled));
        report.push_str(&format!("Instructions Generated: {}\n", stats.compiler.instructions_generated));
        report.push_str(&format!("Constants: {}\n", stats.compiler.constants_count));
        report.push_str(&format!("Compilation Time: {:.2} ms\n", stats.compiler.compilation_time_us as f64 / 1000.0));
        report.push_str("\n");
        
        // Optimizer metrics
        report.push_str("=== Optimization ===\n");
        report.push_str(&format!("Optimization Passes: {}\n", stats.optimizer.passes_applied));
        report.push_str(&format!("Instructions Before: {}\n", stats.optimizer.instructions_before));
        report.push_str(&format!("Instructions After: {}\n", stats.optimizer.instructions_after));
        report.push_str(&format!("Instructions Eliminated: {}\n", stats.optimizer.instructions_eliminated));
        report.push_str(&format!("Optimization Time: {:.2} ms\n", stats.optimizer.optimization_time_us as f64 / 1000.0));
        report.push_str("\n");
        
        // VM metrics
        report.push_str("=== Execution ===\n");
        report.push_str(&format!("Instructions Executed: {}\n", stats.vm.instructions_executed));
        report.push_str(&format!("Function Calls: {}\n", stats.vm.function_calls));
        report.push_str(&format!("Max Stack Depth: {}\n", stats.vm.max_stack_depth));
        report.push_str(&format!("Optimized Operations: {}\n", stats.vm.optimized_operations));
        report.push_str(&format!("Execution Time: {:.2} ms\n", stats.vm.execution_time_us as f64 / 1000.0));
        report.push_str("\n");
        
        // Performance recommendations
        report.push_str("=== Recommendations ===\n");
        if stats.overall.optimization_effectiveness < 0.2 {
            report.push_str("• Consider enabling more aggressive optimization passes\n");
        }
        if stats.overall.memory_efficiency < 0.7 {
            report.push_str("• Memory usage is high, consider tuning constant pool size\n");
        }
        if stats.overall.instructions_per_second < 1_000_000.0 {
            report.push_str("• Low instruction throughput, consider profiling hotspots\n");
        }
        if stats.vm.max_stack_depth > 1000 {
            report.push_str("• High stack usage detected, check for deep recursion\n");
        }
        
        report
    }
    
    /// Enables or disables specific optimizations.
    pub fn configure_optimizations(&mut self, config: OptimizationConfig) {
        self.optimizer.configure(config);
    }
    
    /// Sets the virtual machine configuration.
    pub fn configure_vm(&mut self, config: VmConfig) {
        self.vm.configure(config);
    }
    
    /// Clears all statistics and resets the engine state.
    pub fn reset(&mut self) {
        self.compiler = BytecodeCompiler::new(CompilerOptions::default());
        self.optimizer = BytecodeOptimizer::new();
        self.vm = VirtualMachine::new();
    }
}

/// Configuration for bytecode optimizations.
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Enable constant folding
    pub constant_folding: bool,
    /// Enable dead code elimination
    pub dead_code_elimination: bool,
    /// Enable tail call optimization
    pub tail_call_optimization: bool,
    /// Enable instruction combining
    pub instruction_combining: bool,
    /// Enable register allocation optimization
    pub register_allocation: bool,
    /// Maximum number of optimization passes
    pub max_passes: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            constant_folding: true,
            dead_code_elimination: true,
            tail_call_optimization: true,
            instruction_combining: true,
            register_allocation: false, // More complex, disabled by default
            max_passes: 3,
        }
    }
}

/// Configuration for the virtual machine.
#[derive(Debug, Clone)]
pub struct VmConfig {
    /// Initial stack size
    pub initial_stack_size: usize,
    /// Maximum stack size before overflow
    pub max_stack_size: usize,
    /// Enable garbage collection during execution
    pub gc_enabled: bool,
    /// GC threshold (allocations before triggering GC)
    pub gc_threshold: usize,
    /// Enable profiling during execution
    pub profiling_enabled: bool,
    /// Enable instruction-level debugging
    pub debug_mode: bool,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            initial_stack_size: 1024,
            max_stack_size: 1024 * 1024, // 1M stack slots
            gc_enabled: true,
            gc_threshold: 1000,
            profiling_enabled: false,
            debug_mode: false,
        }
    }
}

/// Global bytecode engine instance for convenience.
static mut GLOBAL_ENGINE: Option<BytecodeEngine> = None;
static GLOBAL_ENGINE_INIT: std::sync::Once = std::sync::Once::new();

/// Gets the global bytecode engine, initializing it if necessary.
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
        let config = OptimizationConfig {
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
        let config = VmConfig {
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
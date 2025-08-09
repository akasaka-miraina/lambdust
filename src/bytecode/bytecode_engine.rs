//! High-level interface for bytecode compilation and execution.

use crate::ast::{Expr, Program};
use crate::eval::Value;
use crate::diagnostics::{Result, Error};

use super::{BytecodeCompiler, CompilerOptions, VirtualMachine, BytecodeOptimizer, 
           CompilationResult, ExecutionResult, OptimizationStats,
           CompilerStats, VmStats, 
           BytecodePerformanceStats, OverallPerformanceMetrics};
use super::optimizer::OptimizationConfig;
use super::vm::VmConfig;

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
            ExecutionResult::Error(error) => Err(error.boxed()),
            ExecutionResult::Yield(_) => Err(Box::new(Error::runtime_error("Unexpected yield in top-level execution".to_string(), None))),
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
        report.push('\n');
        
        // Compiler metrics
        report.push_str("=== Compilation ===\n");
        report.push_str(&format!("Expressions Compiled: {}\n", stats.compiler.expressions_compiled));
        report.push_str(&format!("Instructions Generated: {}\n", stats.compiler.instructions_generated));
        report.push_str(&format!("Constants: {}\n", stats.compiler.constants_count));
        report.push_str(&format!("Compilation Time: {:.2} ms\n", stats.compiler.compilation_time_us as f64 / 1000.0));
        report.push('\n');
        
        // Optimizer metrics
        report.push_str("=== Optimization ===\n");
        report.push_str(&format!("Optimization Passes: {}\n", stats.optimizer.passes_applied));
        report.push_str(&format!("Instructions Before: {}\n", stats.optimizer.instructions_before));
        report.push_str(&format!("Instructions After: {}\n", stats.optimizer.instructions_after));
        report.push_str(&format!("Instructions Eliminated: {}\n", stats.optimizer.instructions_eliminated));
        report.push_str(&format!("Optimization Time: {:.2} ms\n", stats.optimizer.optimization_time_us as f64 / 1000.0));
        report.push('\n');
        
        // VM metrics
        report.push_str("=== Execution ===\n");
        report.push_str(&format!("Instructions Executed: {}\n", stats.vm.instructions_executed));
        report.push_str(&format!("Function Calls: {}\n", stats.vm.function_calls));
        report.push_str(&format!("Max Stack Depth: {}\n", stats.vm.max_stack_depth));
        report.push_str(&format!("Optimized Operations: {}\n", stats.vm.optimized_operations));
        report.push_str(&format!("Execution Time: {:.2} ms\n", stats.vm.execution_time_us as f64 / 1000.0));
        report.push('\n');
        
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
}

impl Default for BytecodeEngine {
    fn default() -> Self {
        Self::new()
    }
}
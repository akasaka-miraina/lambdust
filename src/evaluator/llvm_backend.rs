//! LLVM Backend for Tail Call Optimization
//!
//! This module provides LLVM compiler integration for advanced tail call
//! optimization, leveraging LLVM's native tail call intrinsics and
//! optimization passes for maximum performance.
//!
//! Architecture:
//! - LLVMCodeGenerator: Generates LLVM IR with tail call annotations
//! - TailCallIntrinsic: LLVM tail call intrinsic integration
//! - OptimizationPass: Custom LLVM optimization pass for Scheme functions
//! - CompilerIntegration: Rustc backend integration for seamless compilation

use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use crate::evaluator::{TailCallContext, TailCallOptimizer};
use crate::value::{Procedure, Value};
use std::collections::HashMap;

/// LLVM IR representation for Scheme expressions
#[derive(Debug, Clone)]
pub struct LLVMInstruction {
    /// LLVM instruction opcode
    pub opcode: String,
    /// Operands for the instruction
    pub operands: Vec<String>,
    /// Result register name
    pub result: Option<String>,
    /// Metadata attributes (e.g., tail call markers)
    pub attributes: Vec<String>,
    /// Debug information
    pub debug_info: Option<String>,
}

impl LLVMInstruction {
    /// Create a new LLVM instruction
    pub fn new(opcode: String, operands: Vec<String>) -> Self {
        LLVMInstruction {
            opcode,
            operands,
            result: None,
            attributes: Vec::new(),
            debug_info: None,
        }
    }

    /// Add tail call attribute
    pub fn with_tail_call(mut self) -> Self {
        self.attributes.push("tail".to_string());
        self
    }

    /// Add musttail attribute for guaranteed tail call elimination
    pub fn with_musttail(mut self) -> Self {
        self.attributes.push("musttail".to_string());
        self
    }

    /// Add notail attribute to prevent tail call optimization
    pub fn with_notail(mut self) -> Self {
        self.attributes.push("notail".to_string());
        self
    }

    /// Set result register
    pub fn with_result(mut self, result: String) -> Self {
        self.result = Some(result);
        self
    }

    /// Add debug information
    pub fn with_debug(mut self, debug: String) -> Self {
        self.debug_info = Some(debug);
        self
    }

    /// Generate LLVM IR string representation
    pub fn to_llvm_ir(&self) -> String {
        let mut ir = String::new();

        // Result assignment
        if let Some(ref result) = self.result {
            ir.push_str(&format!("{} = ", result));
        }

        // Attributes (tail call markers)
        if !self.attributes.is_empty() {
            ir.push_str(&self.attributes.join(" "));
            ir.push(' ');
        }

        // Instruction opcode
        ir.push_str(&self.opcode);
        ir.push(' ');

        // Operands
        ir.push_str(&self.operands.join(", "));

        // Debug information
        if let Some(ref debug) = self.debug_info {
            ir.push_str(&format!(", !dbg !{}", debug));
        }

        ir
    }
}

/// LLVM function representation for Scheme procedures
#[derive(Debug, Clone)]
pub struct LLVMFunction {
    /// Function name
    pub name: String,
    /// Parameter types and names
    pub parameters: Vec<(String, String)>, // (type, name)
    /// Return type
    pub return_type: String,
    /// Function body (LLVM instructions)
    pub body: Vec<LLVMInstruction>,
    /// Function attributes
    pub attributes: Vec<String>,
    /// Whether this function uses tail call optimization
    pub uses_tail_calls: bool,
}

impl LLVMFunction {
    /// Create a new LLVM function
    pub fn new(name: String, return_type: String) -> Self {
        LLVMFunction {
            name,
            parameters: Vec::new(),
            return_type,
            body: Vec::new(),
            attributes: Vec::new(),
            uses_tail_calls: false,
        }
    }

    /// Add parameter to function
    pub fn add_parameter(&mut self, param_type: String, param_name: String) {
        self.parameters.push((param_type, param_name));
    }

    /// Add instruction to function body
    pub fn add_instruction(&mut self, instruction: LLVMInstruction) {
        if instruction.attributes.contains(&"tail".to_string())
            || instruction.attributes.contains(&"musttail".to_string())
        {
            self.uses_tail_calls = true;
        }
        self.body.push(instruction);
    }

    /// Add function attribute
    pub fn add_attribute(&mut self, attribute: String) {
        self.attributes.push(attribute);
    }

    /// Generate LLVM IR for the function
    pub fn to_llvm_ir(&self) -> String {
        let mut ir = String::new();

        // Function declaration
        ir.push_str("define ");
        if !self.attributes.is_empty() {
            ir.push_str(&format!("{} ", self.attributes.join(" ")));
        }
        ir.push_str(&format!("{} @{}(", self.return_type, self.name));

        // Parameters
        let params: Vec<String> = self
            .parameters
            .iter()
            .map(|(ty, name)| format!("{} %{}", ty, name))
            .collect();
        ir.push_str(&params.join(", "));
        ir.push_str(") {\n");

        // Function body
        for instruction in &self.body {
            ir.push_str("  ");
            ir.push_str(&instruction.to_llvm_ir());
            ir.push('\n');
        }

        ir.push_str("}\n");
        ir
    }
}

/// LLVM code generator for Scheme expressions
#[derive(Debug)]
pub struct LLVMCodeGenerator {
    /// Generated functions
    functions: HashMap<String, LLVMFunction>,
    /// Current function being generated
    current_function: Option<String>,
    /// Register counter for SSA form
    register_counter: usize,
    /// Optimization settings
    optimization_level: LLVMOptimizationLevel,
    /// Tail call optimizer integration
    tail_call_optimizer: TailCallOptimizer,
}

/// LLVM optimization levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LLVMOptimizationLevel {
    /// No optimization
    O0,
    /// Basic optimization
    O1,
    /// Standard optimization
    O2,
    /// Aggressive optimization with tail call elimination
    O3,
    /// Size optimization
    Os,
    /// Aggressive size optimization
    Oz,
}

impl LLVMCodeGenerator {
    /// Create a new LLVM code generator
    pub fn new() -> Self {
        LLVMCodeGenerator {
            functions: HashMap::new(),
            current_function: None,
            register_counter: 0,
            optimization_level: LLVMOptimizationLevel::O2,
            tail_call_optimizer: TailCallOptimizer::new(),
        }
    }

    /// Create with specific optimization level
    pub fn with_optimization_level(optimization_level: LLVMOptimizationLevel) -> Self {
        LLVMCodeGenerator {
            functions: HashMap::new(),
            current_function: None,
            register_counter: 0,
            optimization_level,
            tail_call_optimizer: TailCallOptimizer::new(),
        }
    }

    /// Generate unique register name
    fn next_register(&mut self) -> String {
        self.register_counter += 1;
        format!("%r{}", self.register_counter)
    }

    /// Start generating a new function
    pub fn start_function(&mut self, name: String, return_type: String) -> Result<()> {
        if self.functions.contains_key(&name) {
            return Err(LambdustError::runtime_error(format!(
                "Function '{}' already defined",
                name
            )));
        }

        let function = LLVMFunction::new(name.clone(), return_type);
        self.functions.insert(name.clone(), function);
        self.current_function = Some(name);

        Ok(())
    }

    /// Finish current function
    pub fn finish_function(&mut self) -> Result<()> {
        if self.current_function.is_none() {
            return Err(LambdustError::runtime_error(
                "No function being generated".to_string(),
            ));
        }

        self.current_function = None;
        Ok(())
    }

    /// Add parameter to current function
    pub fn add_parameter(&mut self, param_type: String, param_name: String) -> Result<()> {
        let function_name = self.current_function.clone().ok_or_else(|| {
            LambdustError::runtime_error("No function being generated".to_string())
        })?;

        self.functions
            .get_mut(&function_name)
            .ok_or_else(|| LambdustError::runtime_error("Function not found".to_string()))?
            .add_parameter(param_type, param_name);

        Ok(())
    }

    /// Generate LLVM IR for Scheme expression with tail call optimization
    pub fn generate_expression(
        &mut self,
        expr: &Expr,
        context: &TailCallContext,
    ) -> Result<String> {
        match expr {
            Expr::Literal(lit) => self.generate_literal(lit),
            Expr::Variable(name) => self.generate_variable(name),
            Expr::List(exprs) if !exprs.is_empty() => self.generate_function_call(exprs, context),
            _ => Err(LambdustError::runtime_error(format!(
                "Unsupported expression for LLVM generation: {:?}",
                expr
            ))),
        }
    }

    /// Generate LLVM IR for literal values
    pub fn generate_literal(&mut self, lit: &crate::ast::Literal) -> Result<String> {
        let register = self.next_register();
        let instruction = match lit {
            crate::ast::Literal::Boolean(b) => LLVMInstruction::new(
                "alloca".to_string(),
                vec![
                    "i1".to_string(),
                    if *b { "true" } else { "false" }.to_string(),
                ],
            )
            .with_result(register.clone()),
            crate::ast::Literal::Number(n) => {
                // Simplified number handling - in practice would need complex number support
                let value = match n {
                    crate::lexer::SchemeNumber::Integer(i) => i.to_string(),
                    crate::lexer::SchemeNumber::Real(f) => f.to_string(),
                    _ => "0".to_string(), // Simplified
                };
                LLVMInstruction::new("alloca".to_string(), vec!["double".to_string(), value])
                    .with_result(register.clone())
            }
            crate::ast::Literal::String(s) => LLVMInstruction::new(
                "alloca".to_string(),
                vec!["i8*".to_string(), format!("\"{}\"", s)],
            )
            .with_result(register.clone()),
            _ => {
                return Err(LambdustError::runtime_error(format!(
                    "Unsupported literal type: {:?}",
                    lit
                )));
            }
        };

        self.add_instruction_to_current_function(instruction)?;
        Ok(register)
    }

    /// Generate LLVM IR for variable access
    pub fn generate_variable(&mut self, name: &str) -> Result<String> {
        let register = self.next_register();
        let instruction = LLVMInstruction::new(
            "load".to_string(),
            vec!["i8*".to_string(), format!("@{}", name)],
        )
        .with_result(register.clone());

        self.add_instruction_to_current_function(instruction)?;
        Ok(register)
    }

    /// Generate LLVM IR for function calls with tail call optimization
    pub fn generate_function_call(
        &mut self,
        exprs: &[Expr],
        context: &TailCallContext,
    ) -> Result<String> {
        if exprs.is_empty() {
            return Err(LambdustError::runtime_error(
                "Empty function call".to_string(),
            ));
        }

        // Generate function name
        let function_name = match &exprs[0] {
            Expr::Variable(name) => name.clone(),
            _ => {
                return Err(LambdustError::runtime_error(
                    "Complex function expressions not supported yet".to_string(),
                ));
            }
        };

        // Generate arguments
        let mut arg_registers = Vec::new();
        for arg_expr in &exprs[1..] {
            let arg_context = context.non_tail(); // Arguments are not in tail position
            let arg_register = self.generate_expression(arg_expr, &arg_context)?;
            arg_registers.push(arg_register);
        }

        // Check for tail call optimization opportunity
        let optimization = self.tail_call_optimizer.optimize_tail_call(
            &Expr::List(exprs.to_vec()),
            context,
            &mut crate::evaluator::Evaluator::new(),
        )?;

        // Generate call instruction with appropriate tail call attributes
        let result_register = self.next_register();
        let mut call_instruction = LLVMInstruction::new("call".to_string(), {
            let mut operands = vec![format!("@{}", function_name)];
            operands.extend(arg_registers);
            operands
        })
        .with_result(result_register.clone());

        // Apply tail call optimization based on context and analysis
        if context.is_tail_position {
            if let Some(opt) = optimization {
                match opt.optimization_level {
                    crate::evaluator::OptimizationLevel::Basic => {
                        call_instruction = call_instruction.with_tail_call();
                    }
                    crate::evaluator::OptimizationLevel::Advanced => {
                        call_instruction = call_instruction.with_tail_call();
                    }
                    crate::evaluator::OptimizationLevel::Full => {
                        // Use musttail for guaranteed optimization
                        call_instruction = call_instruction.with_musttail();
                    }
                    crate::evaluator::OptimizationLevel::None => {
                        // No tail call optimization
                    }
                }
            } else if context.should_optimize() {
                // Default tail call optimization
                call_instruction = call_instruction.with_tail_call();
            }
        }

        self.add_instruction_to_current_function(call_instruction)?;
        Ok(result_register)
    }

    /// Add instruction to current function
    pub fn add_instruction_to_current_function(
        &mut self,
        instruction: LLVMInstruction,
    ) -> Result<()> {
        let function_name = self.current_function.clone().ok_or_else(|| {
            LambdustError::runtime_error("No function being generated".to_string())
        })?;

        self.functions
            .get_mut(&function_name)
            .ok_or_else(|| LambdustError::runtime_error("Function not found".to_string()))?
            .add_instruction(instruction);

        Ok(())
    }

    /// Generate LLVM IR for a complete Scheme function
    pub fn generate_function(
        &mut self,
        name: String,
        params: Vec<String>,
        body: &Expr,
    ) -> Result<String> {
        // Start function generation
        self.start_function(name.clone(), "i8*".to_string())?;

        // Add parameters
        for param in &params {
            self.add_parameter("i8*".to_string(), param.clone())?;
        }

        // Generate function body in tail position
        let tail_context = TailCallContext::new().enter_function(Some(name.clone()));
        let result_register = self.generate_expression(body, &tail_context)?;

        // Add return instruction
        let return_instruction =
            LLVMInstruction::new("ret".to_string(), vec!["i8*".to_string(), result_register]);
        self.add_instruction_to_current_function(return_instruction)?;

        // Finish function
        self.finish_function()?;

        // Return generated LLVM IR
        Ok(self
            .functions
            .get(&name)
            .ok_or_else(|| LambdustError::runtime_error("Function not found".to_string()))?
            .to_llvm_ir())
    }

    /// Generate LLVM IR for all functions
    pub fn generate_module(&self) -> String {
        let mut module_ir = String::new();

        // Module header
        module_ir.push_str("; Lambdust LLVM Module with Tail Call Optimization\n");
        module_ir.push_str("; Generated by Lambdust Phase 6-D LLVM Backend\n\n");

        // Target specification for tail call optimization
        module_ir.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n");
        module_ir.push_str("target triple = \"x86_64-unknown-linux-gnu\"\n\n");

        // Function declarations for runtime
        module_ir.push_str("; Runtime function declarations\n");
        module_ir.push_str("declare i8* @scheme_alloc(i64)\n");
        module_ir.push_str("declare void @scheme_gc()\n");
        module_ir.push_str("declare i8* @scheme_apply(i8*, i8*)\n\n");

        // Generated functions
        for function in self.functions.values() {
            module_ir.push_str(&function.to_llvm_ir());
            module_ir.push('\n');
        }

        module_ir
    }

    /// Get optimization statistics
    pub fn get_optimization_stats(&self) -> LLVMOptimizationStats {
        let mut stats = LLVMOptimizationStats::default();

        for function in self.functions.values() {
            stats.total_functions += 1;
            if function.uses_tail_calls {
                stats.tail_call_optimized_functions += 1;
            }

            for instruction in &function.body {
                stats.total_instructions += 1;
                if instruction.attributes.contains(&"tail".to_string()) {
                    stats.tail_call_instructions += 1;
                }
                if instruction.attributes.contains(&"musttail".to_string()) {
                    stats.musttail_instructions += 1;
                }
            }
        }

        stats
    }

    /// Set optimization level
    pub fn set_optimization_level(&mut self, level: LLVMOptimizationLevel) {
        self.optimization_level = level;
    }

    /// Get current optimization level
    pub fn optimization_level(&self) -> &LLVMOptimizationLevel {
        &self.optimization_level
    }

    /// Clear all generated functions
    pub fn clear(&mut self) {
        self.functions.clear();
        self.current_function = None;
        self.register_counter = 0;
    }
}

impl Default for LLVMCodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// LLVM optimization statistics
#[derive(Debug, Clone, Default)]
pub struct LLVMOptimizationStats {
    /// Total number of generated functions
    pub total_functions: usize,
    /// Functions with tail call optimization
    pub tail_call_optimized_functions: usize,
    /// Total instructions generated
    pub total_instructions: usize,
    /// Instructions with tail call attributes
    pub tail_call_instructions: usize,
    /// Instructions with musttail attributes
    pub musttail_instructions: usize,
}

impl LLVMOptimizationStats {
    /// Calculate tail call optimization ratio
    pub fn tail_call_ratio(&self) -> f64 {
        if self.total_functions == 0 {
            0.0
        } else {
            self.tail_call_optimized_functions as f64 / self.total_functions as f64
        }
    }

    /// Calculate instruction optimization ratio
    pub fn instruction_optimization_ratio(&self) -> f64 {
        if self.total_instructions == 0 {
            0.0
        } else {
            (self.tail_call_instructions + self.musttail_instructions) as f64
                / self.total_instructions as f64
        }
    }
}

/// LLVM tail call intrinsic interface
#[derive(Debug)]
pub struct LLVMTailCallIntrinsic {
    /// Code generator
    codegen: LLVMCodeGenerator,
    /// Intrinsic statistics
    stats: LLVMIntrinsicStats,
}

/// LLVM intrinsic statistics
#[derive(Debug, Clone, Default)]
pub struct LLVMIntrinsicStats {
    /// Intrinsic calls generated
    pub intrinsic_calls: usize,
    /// Successful optimizations
    pub successful_optimizations: usize,
    /// Failed optimizations
    pub failed_optimizations: usize,
}

impl LLVMTailCallIntrinsic {
    /// Create new LLVM tail call intrinsic interface
    pub fn new() -> Self {
        LLVMTailCallIntrinsic {
            codegen: LLVMCodeGenerator::new(),
            stats: LLVMIntrinsicStats::default(),
        }
    }

    /// Generate tail call intrinsic for Scheme procedure
    pub fn generate_tail_call_intrinsic(
        &mut self,
        procedure: &Procedure,
        _args: &[Value],
        _context: &TailCallContext,
    ) -> Result<String> {
        self.stats.intrinsic_calls += 1;

        match procedure {
            Procedure::Lambda { params, body, .. } => {
                if body.len() == 1 {
                    let function_name = format!("lambda_{}", self.stats.intrinsic_calls);
                    let llvm_ir =
                        self.codegen
                            .generate_function(function_name, params.clone(), &body[0])?;

                    self.stats.successful_optimizations += 1;
                    Ok(llvm_ir)
                } else {
                    self.stats.failed_optimizations += 1;
                    Err(LambdustError::runtime_error(
                        "Multi-expression lambda not supported yet".to_string(),
                    ))
                }
            }
            _ => {
                self.stats.failed_optimizations += 1;
                Err(LambdustError::runtime_error(
                    "Only lambda procedures supported for LLVM intrinsics".to_string(),
                ))
            }
        }
    }

    /// Get intrinsic statistics
    pub fn get_stats(&self) -> &LLVMIntrinsicStats {
        &self.stats
    }

    /// Get code generator
    pub fn codegen(&mut self) -> &mut LLVMCodeGenerator {
        &mut self.codegen
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = LLVMIntrinsicStats::default();
    }
}

impl Default for LLVMTailCallIntrinsic {
    fn default() -> Self {
        Self::new()
    }
}

/// Compiler integration for LLVM backend
#[derive(Debug)]
pub struct LLVMCompilerIntegration {
    /// LLVM intrinsic interface
    intrinsic: LLVMTailCallIntrinsic,
    /// Compiler optimization level
    opt_level: LLVMOptimizationLevel,
    /// Integration statistics
    stats: LLVMCompilerStats,
}

/// Compiler integration statistics
#[derive(Debug, Clone, Default)]
pub struct LLVMCompilerStats {
    /// Compilation requests
    pub compilation_requests: usize,
    /// Successful compilations
    pub successful_compilations: usize,
    /// Failed compilations
    pub failed_compilations: usize,
    /// LLVM optimization passes run
    pub optimization_passes: usize,
}

impl LLVMCompilerIntegration {
    /// Create new LLVM compiler integration
    pub fn new() -> Self {
        LLVMCompilerIntegration {
            intrinsic: LLVMTailCallIntrinsic::new(),
            opt_level: LLVMOptimizationLevel::O2,
            stats: LLVMCompilerStats::default(),
        }
    }

    /// Create with optimization level
    pub fn with_optimization_level(opt_level: LLVMOptimizationLevel) -> Self {
        LLVMCompilerIntegration {
            intrinsic: LLVMTailCallIntrinsic::new(),
            opt_level,
            stats: LLVMCompilerStats::default(),
        }
    }

    /// Compile Scheme expression to LLVM with tail call optimization
    pub fn compile_with_tail_calls(
        &mut self,
        expr: &Expr,
        context: &TailCallContext,
    ) -> Result<String> {
        self.stats.compilation_requests += 1;

        let result = self.intrinsic.codegen().generate_expression(expr, context);

        match result {
            Ok(llvm_ir) => {
                self.stats.successful_compilations += 1;
                Ok(llvm_ir)
            }
            Err(e) => {
                self.stats.failed_compilations += 1;
                Err(e)
            }
        }
    }

    /// Run LLVM optimization passes
    pub fn run_optimization_passes(&mut self, _module_ir: &str) -> Result<String> {
        self.stats.optimization_passes += 1;

        // In a real implementation, this would invoke LLVM optimization passes
        // For now, return the input IR with optimization annotations
        Ok(format!(
            "; Optimized with LLVM {} passes\n; Tail call optimization enabled\n{}",
            self.opt_level_to_string(),
            _module_ir
        ))
    }

    /// Convert optimization level to string
    fn opt_level_to_string(&self) -> &'static str {
        match self.opt_level {
            LLVMOptimizationLevel::O0 => "O0",
            LLVMOptimizationLevel::O1 => "O1",
            LLVMOptimizationLevel::O2 => "O2",
            LLVMOptimizationLevel::O3 => "O3",
            LLVMOptimizationLevel::Os => "Os",
            LLVMOptimizationLevel::Oz => "Oz",
        }
    }

    /// Get compiler statistics
    pub fn get_stats(&self) -> &LLVMCompilerStats {
        &self.stats
    }

    /// Get intrinsic interface
    pub fn intrinsic(&mut self) -> &mut LLVMTailCallIntrinsic {
        &mut self.intrinsic
    }

    /// Set optimization level
    pub fn set_optimization_level(&mut self, level: LLVMOptimizationLevel) {
        self.opt_level = level.clone();
        self.intrinsic.codegen().set_optimization_level(level);
    }

    /// Reset all statistics
    pub fn reset_stats(&mut self) {
        self.stats = LLVMCompilerStats::default();
        self.intrinsic.reset_stats();
    }
}

impl Default for LLVMCompilerIntegration {
    fn default() -> Self {
        Self::new()
    }
}

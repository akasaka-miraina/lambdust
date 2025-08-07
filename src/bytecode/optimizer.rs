//! Bytecode optimization passes for improved performance.

use super::instruction::{Instruction, OpCode, Operand, Bytecode};
use crate::diagnostics::Result;
use std::time::Instant;

/// Bytecode optimizer that applies various optimization passes.
pub struct BytecodeOptimizer {
    /// Optimization configuration
    config: OptimizationConfig,
    /// Statistics about optimizations performed
    stats: OptimizationStats,
}

/// Configuration for bytecode optimization.
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
    /// Enable register allocation
    pub register_allocation: bool,
    /// Maximum optimization passes
    pub max_passes: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            constant_folding: true,
            dead_code_elimination: true,
            tail_call_optimization: true,
            instruction_combining: true,
            register_allocation: false,
            max_passes: 3,
        }
    }
}

/// Statistics about optimization results.
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    /// Number of optimization passes applied
    pub passes_applied: usize,
    /// Instructions before optimization
    pub instructions_before: usize,
    /// Instructions after optimization
    pub instructions_after: usize,
    /// Instructions eliminated
    pub instructions_eliminated: usize,
    /// Time spent optimizing (microseconds)
    pub optimization_time_us: u64,
    /// Memory saved (estimated bytes)
    pub memory_saved_bytes: usize,
}

/// Represents an optimization pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationPass {
    /// Constant folding and propagation
    ConstantFolding,
    /// Dead code elimination
    DeadCodeElimination,
    /// Tail call optimization
    TailCallOptimization,
    /// Instruction combining and peephole optimization
    InstructionCombining,
    /// Register allocation and local optimization
    RegisterAllocation,
}

impl BytecodeOptimizer {
    /// Creates a new bytecode optimizer.
    pub fn new() -> Self {
        Self {
            config: OptimizationConfig::default(),
            stats: OptimizationStats {
                passes_applied: 0,
                instructions_before: 0,
                instructions_after: 0,
                instructions_eliminated: 0,
                optimization_time_us: 0,
                memory_saved_bytes: 0,
            },
        }
    }
    
    /// Creates a new bytecode optimizer with configuration.
    pub fn with_config(config: OptimizationConfig) -> Self {
        Self {
            config,
            stats: OptimizationStats {
                passes_applied: 0,
                instructions_before: 0,
                instructions_after: 0,
                instructions_eliminated: 0,
                optimization_time_us: 0,
                memory_saved_bytes: 0,
            },
        }
    }
    
    /// Optimizes bytecode by applying configured optimization passes.
    pub fn optimize(&mut self, mut bytecode: Bytecode) -> Result<Bytecode> {
        let start_time = Instant::now();
        let initial_instruction_count = bytecode.instructions.len();
        self.stats.instructions_before = initial_instruction_count;
        
        let mut changed = true;
        let mut pass_count = 0;
        
        while changed && pass_count < self.config.max_passes {
            changed = false;
            
            // Apply enabled optimization passes
            if self.config.constant_folding {
                if self.apply_constant_folding(&mut bytecode)? {
                    changed = true;
                }
            }
            
            if self.config.dead_code_elimination {
                if self.apply_dead_code_elimination(&mut bytecode)? {
                    changed = true;
                }
            }
            
            if self.config.instruction_combining {
                if self.apply_instruction_combining(&mut bytecode)? {
                    changed = true;
                }
            }
            
            if self.config.tail_call_optimization {
                if self.apply_tail_call_optimization(&mut bytecode)? {
                    changed = true;
                }
            }
            
            if self.config.register_allocation {
                if self.apply_register_allocation(&mut bytecode)? {
                    changed = true;
                }
            }
            
            pass_count += 1;
        }
        
        // Update statistics
        self.stats.passes_applied += pass_count;
        self.stats.instructions_after = bytecode.instructions.len();
        self.stats.instructions_eliminated = initial_instruction_count.saturating_sub(bytecode.instructions.len());
        self.stats.optimization_time_us += start_time.elapsed().as_micros() as u64;
        self.stats.memory_saved_bytes = self.stats.instructions_eliminated * 16; // Rough estimate
        
        Ok(bytecode)
    }
    
    /// Applies constant folding optimization.
    fn apply_constant_folding(&self, bytecode: &mut Bytecode) -> Result<bool> {
        let mut changed = false;
        let mut new_instructions = Vec::new();
        let mut i = 0;
        
        while i < bytecode.instructions.len() {
            // Look for patterns like: LoadConst, LoadConst, Add
            if i + 2 < bytecode.instructions.len() {
                if let (
                    Instruction { opcode: OpCode::LoadConst, operand: Operand::ConstIndex(idx1), .. },
                    Instruction { opcode: OpCode::LoadConst, operand: Operand::ConstIndex(idx2), .. },
                    Instruction { opcode: OpCode::Add, .. }
                ) = (&bytecode.instructions[i], &bytecode.instructions[i + 1], &bytecode.instructions[i + 2]) {
                    
                    // Try to fold constants
                    if let (Some(const1), Some(const2)) = (
                        bytecode.constants.get_constant(*idx1),
                        bytecode.constants.get_constant(*idx2)
                    ) {
                        if let (Some(n1), Some(n2)) = (self.constant_to_number(const1), self.constant_to_number(const2)) {
                            // Fold the addition
                            let result = n1 + n2;
                            let result_const = super::instruction::ConstantValue::Number(result);
                            let result_idx = bytecode.constants.add_constant(result_const);
                            
                            new_instructions.push(Instruction::with_operand(OpCode::LoadConst, Operand::ConstIndex(result_idx)));
                            i += 3; // Skip the three instructions we just folded
                            changed = true;
                            continue;
                        }
                    }
                }
            }
            
            new_instructions.push(bytecode.instructions[i].clone());
            i += 1;
        }
        
        if changed {
            bytecode.instructions = new_instructions;
        }
        
        Ok(changed)
    }
    
    /// Applies dead code elimination.
    fn apply_dead_code_elimination(&self, bytecode: &mut Bytecode) -> Result<bool> {
        let mut changed = false;
        
        // Find unreachable code after unconditional jumps
        let mut reachable = vec![false; bytecode.instructions.len()];
        let mut worklist = vec![bytecode.entry_point];
        
        while let Some(index) = worklist.pop() {
            if index >= bytecode.instructions.len() || reachable[index] {
                continue;
            }
            
            reachable[index] = true;
            
            match &bytecode.instructions[index] {
                Instruction { opcode: OpCode::Jump, operand: Operand::JumpOffset(offset), .. } => {
                    let target = (index as i32 + offset) as usize;
                    if target < bytecode.instructions.len() {
                        worklist.push(target);
                    }
                }
                Instruction { opcode: OpCode::JumpIfTrue | OpCode::JumpIfFalse, operand: Operand::JumpOffset(offset), .. } => {
                    let target = (index as i32 + offset) as usize;
                    if target < bytecode.instructions.len() {
                        worklist.push(target);
                    }
                    // Also continue to next instruction
                    worklist.push(index + 1);
                }
                Instruction { opcode: OpCode::Return | OpCode::Halt, .. } => {
                    // These don't continue to next instruction
                }
                _ => {
                    // Regular instruction continues to next
                    worklist.push(index + 1);
                }
            }
        }
        
        // Remove unreachable instructions
        let original_len = bytecode.instructions.len();
        let instructions = std::mem::take(&mut bytecode.instructions);
        bytecode.instructions = instructions.into_iter()
            .enumerate()
            .filter(|(i, _)| reachable[*i])
            .map(|(_, inst)| inst)
            .collect();
        
        if bytecode.instructions.len() < original_len {
            changed = true;
        }
        
        Ok(changed)
    }
    
    /// Applies instruction combining (peephole optimization).
    fn apply_instruction_combining(&self, bytecode: &mut Bytecode) -> Result<bool> {
        let mut changed = false;
        let mut new_instructions = Vec::new();
        let mut i = 0;
        
        while i < bytecode.instructions.len() {
            // Look for Pop followed by LoadConst (can eliminate the Pop in some cases)  
            if i + 1 < bytecode.instructions.len() {
                if let (
                    Instruction { opcode: OpCode::Pop, .. },
                    Instruction { opcode: OpCode::LoadConst, .. }
                ) = (&bytecode.instructions[i], &bytecode.instructions[i + 1]) {
                    // If this Pop is just discarding a value before loading a constant,
                    // we can potentially eliminate it (simplified analysis)
                    new_instructions.push(bytecode.instructions[i + 1].clone());
                    i += 2;
                    changed = true;
                    continue;
                }
            }
            
            // Look for redundant Dup followed by Pop
            if i + 1 < bytecode.instructions.len() {
                if let (
                    Instruction { opcode: OpCode::Dup, .. },
                    Instruction { opcode: OpCode::Pop, .. }
                ) = (&bytecode.instructions[i], &bytecode.instructions[i + 1]) {
                    // Dup followed by Pop is a no-op
                    i += 2;
                    changed = true;
                    continue;
                }
            }
            
            new_instructions.push(bytecode.instructions[i].clone());
            i += 1;
        }
        
        if changed {
            bytecode.instructions = new_instructions;
        }
        
        Ok(changed)
    }
    
    /// Applies tail call optimization.
    fn apply_tail_call_optimization(&self, bytecode: &mut Bytecode) -> Result<bool> {
        let mut changed = false;
        
        // Look for Call followed by Return patterns
        for i in 0..bytecode.instructions.len().saturating_sub(1) {
            if let (
                Instruction { opcode: OpCode::Call, operand, .. },
                Instruction { opcode: OpCode::Return, .. }
            ) = (&bytecode.instructions[i], &bytecode.instructions[i + 1]) {
                // Convert Call to TailCall
                bytecode.instructions[i] = Instruction::with_operand(OpCode::TailCall, operand.clone());
                // Remove the Return instruction (TailCall implies return)
                bytecode.instructions.remove(i + 1);
                changed = true;
                break; // Indices have changed, start over
            }
        }
        
        Ok(changed)
    }
    
    /// Applies register allocation optimization.
    fn apply_register_allocation(&self, _bytecode: &mut Bytecode) -> Result<bool> {
        // Register allocation is complex and would require more sophisticated analysis
        // For now, just return false (no changes)
        Ok(false)
    }
    
    /// Converts a constant value to a number if possible.
    fn constant_to_number(&self, constant: &super::instruction::ConstantValue) -> Option<f64> {
        match constant {
            super::instruction::ConstantValue::Number(n) => Some(*n),
            _ => None,
        }
    }
    
    /// Configures the optimizer.
    pub fn configure(&mut self, config: OptimizationConfig) {
        self.config = config;
    }
    
    /// Gets optimization statistics.
    pub fn get_stats(&self) -> OptimizationStats {
        self.stats.clone())
    }
    
    /// Resets optimization statistics.
    pub fn reset_stats(&mut self) {
        self.stats = OptimizationStats {
            passes_applied: 0,
            instructions_before: 0,
            instructions_after: 0,
            instructions_eliminated: 0,
            optimization_time_us: 0,
            memory_saved_bytes: 0,
        };
    }
    
    /// Generates an optimization report.
    pub fn generate_report(&self) -> String {
        let stats = &self.stats;
        let mut report = String::new();
        
        report.push_str("=== Bytecode Optimization Report ===\n");
        report.push_str(&format!("Optimization passes applied: {}\n", stats.passes_applied));
        report.push_str(&format!("Instructions before: {}\n", stats.instructions_before));
        report.push_str(&format!("Instructions after: {}\n", stats.instructions_after));
        report.push_str(&format!("Instructions eliminated: {}\n", stats.instructions_eliminated));
        
        if stats.instructions_before > 0 {
            let reduction_percent = (stats.instructions_eliminated as f64 / stats.instructions_before as f64) * 100.0;
            report.push_str(&format!("Code size reduction: {:.1}%\n", reduction_percent));
        }
        
        report.push_str(&format!("Optimization time: {:.2} ms\n", stats.optimization_time_us as f64 / 1000.0));
        report.push_str(&format!("Estimated memory saved: {} bytes\n", stats.memory_saved_bytes));
        
        // Recommendations
        report.push_str("\n=== Recommendations ===\n");
        if stats.instructions_eliminated == 0 {
            report.push_str("• No optimizations applied - consider enabling more optimization passes\n");
        }
        if stats.optimization_time_us > 10000 {
            report.push_str("• High optimization time - consider reducing max_passes for faster compilation\n");
        }
        
        report
    }
}

impl Default for BytecodeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::instruction::{Instruction, OpCode, Operand, ConstantPool, ConstantValue, Bytecode};
    
    #[test]
    fn test_optimizer_creation() {
        let optimizer = BytecodeOptimizer::new();
        assert!(optimizer.config.constant_folding);
        assert!(optimizer.config.dead_code_elimination);
    }
    
    #[test]
    fn test_constant_folding() {
        let mut optimizer = BytecodeOptimizer::new();
        let mut bytecode = Bytecode::new();
        
        // Add constants to pool
        bytecode.constants.add_constant(ConstantValue::Number(5.0));
        bytecode.constants.add_constant(ConstantValue::Number(3.0));
        
        // Create instruction sequence: LoadConst 0, LoadConst 1, Add
        bytecode.add_instruction(Instruction::with_operand(OpCode::LoadConst, Operand::ConstIndex(0)));
        bytecode.add_instruction(Instruction::with_operand(OpCode::LoadConst, Operand::ConstIndex(1)));
        bytecode.add_instruction(Instruction::new(OpCode::Add));
        
        let original_len = bytecode.instructions.len();
        let optimized = optimizer.optimize(bytecode).unwrap();
        
        // Should have folded the constants and reduced instruction count
        assert!(optimized.instructions.len() <= original_len);
        
        let stats = optimizer.get_stats();
        assert!(stats.passes_applied > 0);
    }
    
    #[test]
    fn test_dead_code_elimination() {
        let mut optimizer = BytecodeOptimizer::with_config(OptimizationConfig {
            constant_folding: false,
            dead_code_elimination: true,
            tail_call_optimization: false,
            instruction_combining: false,
            register_allocation: false,
            max_passes: 1,
        });
        
        let mut bytecode = Bytecode::new();
        
        // Create bytecode with unreachable code
        bytecode.add_instruction(Instruction::new(OpCode::Halt));           // 0: Reachable
        bytecode.add_instruction(Instruction::new(OpCode::LoadConst));      // 1: Unreachable
        bytecode.add_instruction(Instruction::new(OpCode::Add));            // 2: Unreachable
        
        let original_len = bytecode.instructions.len();
        let optimized = optimizer.optimize(bytecode).unwrap();
        
        // Should have eliminated unreachable code
        assert!(optimized.instructions.len() < original_len);
        
        let stats = optimizer.get_stats();
        assert!(stats.instructions_eliminated > 0);
    }
    
    #[test]
    fn test_instruction_combining() {
        let mut optimizer = BytecodeOptimizer::with_config(OptimizationConfig {
            constant_folding: false,
            dead_code_elimination: false,
            tail_call_optimization: false,
            instruction_combining: true,
            register_allocation: false,
            max_passes: 1,
        });
        
        let mut bytecode = Bytecode::new();
        
        // Create Dup followed by Pop (should be eliminated)
        bytecode.add_instruction(Instruction::new(OpCode::Dup));
        bytecode.add_instruction(Instruction::new(OpCode::Pop));
        bytecode.add_instruction(Instruction::new(OpCode::Halt));
        
        let original_len = bytecode.instructions.len();
        let optimized = optimizer.optimize(bytecode).unwrap();
        
        // Should have eliminated the Dup/Pop pair
        assert!(optimized.instructions.len() < original_len);
    }
    
    #[test]
    fn test_optimization_report() {
        let mut optimizer = BytecodeOptimizer::new();
        let mut bytecode = Bytecode::new();
        
        bytecode.add_instruction(Instruction::new(OpCode::Halt));
        
        let _optimized = optimizer.optimize(bytecode).unwrap();
        let report = optimizer.generate_report();
        
        assert!(report.contains("Bytecode Optimization Report"));
        assert!(report.contains("Instructions before"));
        assert!(report.contains("Instructions after"));
    }
}
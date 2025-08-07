//! Virtual machine for executing Lambdust bytecode.

#![allow(dead_code)]

use super::instruction::{Instruction, OpCode, Operand, ConstantPool, ConstantValue, Bytecode};
use crate::eval::Value;
use crate::diagnostics::{Result, Error};
use std::collections::HashMap;
use std::time::Instant;

/// Result of bytecode execution.
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Normal completion with a value
    Value(Value),
    /// Runtime error
    Error(Error),
    /// Yielded control (for generators/coroutines)
    Yield(Value),
}

/// Virtual machine state.
#[derive(Debug, Clone)]
pub enum VmState {
    /// Ready to execute
    Ready,
    /// Currently executing
    Running,
    /// Execution completed
    Completed,
    /// Execution failed with error
    Error(Error),
    /// Yielded (suspended)
    Yielded,
}

/// Virtual machine configuration.
#[derive(Debug, Clone)]
pub struct VmConfig {
    /// Initial stack size
    pub initial_stack_size: usize,
    /// Maximum stack size
    pub max_stack_size: usize,
    /// Enable garbage collection
    pub gc_enabled: bool,
    /// GC threshold
    pub gc_threshold: usize,
    /// Enable profiling
    pub profiling_enabled: bool,
    /// Enable debugging
    pub debug_mode: bool,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            initial_stack_size: 1024,
            max_stack_size: 1024 * 1024,
            gc_enabled: true,
            gc_threshold: 1000,
            profiling_enabled: false,
            debug_mode: false,
        }
    }
}

/// Virtual machine for executing bytecode.
pub struct VirtualMachine {
    /// VM configuration
    config: VmConfig,
    /// Execution state
    state: VmState,
    /// Value stack
    stack: Vec<Value>,
    /// Call stack
    call_stack: Vec<CallFrame>,
    /// Global variables
    globals: HashMap<String, Value>,
    /// Statistics
    stats: VmStats,
}

/// Call frame for function calls.
#[derive(Debug, Clone)]
struct CallFrame {
    /// Return address (instruction pointer)
    return_address: usize,
    /// Local variables
    locals: Vec<Value>,
    /// Function name (for debugging)
    function_name: Option<String>,
}

/// VM execution statistics.
#[derive(Debug, Clone)]
pub struct VmStats {
    /// Instructions executed
    pub instructions_executed: usize,
    /// Execution time in microseconds
    pub execution_time_us: u64,
    /// Function calls made
    pub function_calls: usize,
    /// Maximum stack depth reached
    pub max_stack_depth: usize,
    /// GC collections triggered
    pub gc_count: usize,
    /// Optimized operations executed
    pub optimized_operations: usize,
}

impl VirtualMachine {
    /// Creates a new virtual machine.
    pub fn new() -> Self {
        Self::with_config(VmConfig::default())
    }
    
    /// Creates a new virtual machine with configuration.
    pub fn with_config(config: VmConfig) -> Self {
        Self {
            stack: Vec::with_capacity(config.initial_stack_size),
            call_stack: Vec::new(),
            globals: HashMap::new(),
            state: VmState::Ready,
            config,
            stats: VmStats {
                instructions_executed: 0,
                execution_time_us: 0,
                function_calls: 0,
                max_stack_depth: 0,
                gc_count: 0,
                optimized_operations: 0,
            },
        }
    }
    
    /// Executes bytecode and returns the result.
    pub fn execute(&mut self, bytecode: &Bytecode, constant_pool: &ConstantPool) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        self.state = VmState::Running;
        
        let mut ip = bytecode.entry_point; // Instruction pointer
        
        loop {
            if ip >= bytecode.instructions.len() {
                self.state = VmState::Error(Error::runtime_error("Instruction pointer out of bounds".to_string(), None));
                break;
            }
            
            let instruction = &bytecode.instructions[ip];
            self.stats.instructions_executed += 1;
            
            match self.execute_instruction(instruction, constant_pool) {
                Ok(control_flow) => {
                    match control_flow {
                        ControlFlow::Continue => ip += 1,
                        ControlFlow::Jump(target) => ip = target,
                        ControlFlow::Return(value) => {
                            self.state = VmState::Completed;
                            self.stats.execution_time_us += start_time.elapsed().as_micros() as u64;
                            return Ok(ExecutionResult::Value(value));
                        }
                        ControlFlow::Error(error) => {
                            self.state = VmState::Error(error.clone());
                            self.stats.execution_time_us += start_time.elapsed().as_micros() as u64;
                            return Ok(ExecutionResult::Error(error));
                        }
                        ControlFlow::Yield(value) => {
                            self.state = VmState::Yielded;
                            self.stats.execution_time_us += start_time.elapsed().as_micros() as u64;
                            return Ok(ExecutionResult::Yield(value));
                        }
                    }
                }
                Err(error) => {
                    self.state = VmState::Error(error.clone());
                    self.stats.execution_time_us += start_time.elapsed().as_micros() as u64;
                    return Ok(ExecutionResult::Error(error));
                }
            }
            
            // Update stack depth stats
            self.stats.max_stack_depth = self.stats.max_stack_depth.max(self.stack.len());
            
            // Check for stack overflow
            if self.stack.len() > self.config.max_stack_size {
                let error = Error::runtime_error("Stack overflow".to_string(), None);
                self.state = VmState::Error(error.clone());
                return Ok(ExecutionResult::Error(error));
            }
        }
        
        // Should not reach here normally
        let error = Error::runtime_error("Execution ended unexpectedly".to_string(), None);
        Ok(ExecutionResult::Error(error))
    }
    
    /// Executes a single instruction.
    fn execute_instruction(&mut self, instruction: &Instruction, constant_pool: &ConstantPool) -> Result<ControlFlow> {
        match instruction.opcode {
            OpCode::LoadConst => {
                if let Operand::ConstIndex(index) = instruction.operand {
                    if let Some(constant) = constant_pool.get_constant(index) {
                        let value = self.constant_to_value(constant)?;
                        self.stack.push(value);
                        Ok(ControlFlow::Continue)
                    } else {
                        Err(Box::new(Error::runtime_error(format!("Invalid constant index: {}", index), None))
                    }
                } else {
                    Err(Box::new(Error::runtime_error("LoadConst requires constant index operand".to_string(), None))
                }
            }
            
            OpCode::LoadLocal => {
                if let Operand::LocalIndex(index) = instruction.operand {
                    if let Some(frame) = self.call_stack.last() {
                        if let Some(value) = frame.locals.get(index as usize) {
                            self.stack.push(value.clone());
                            Ok(ControlFlow::Continue)
                        } else {
                            Err(Box::new(Error::runtime_error(format!("Invalid local index: {}", index), None))
                        }
                    } else {
                        Err(Box::new(Error::runtime_error("No call frame for local access".to_string(), None))
                    }
                } else {
                    Err(Box::new(Error::runtime_error("LoadLocal requires local index operand".to_string(), None))
                }
            }
            
            OpCode::LoadGlobal => {
                if let Operand::Symbol(symbol) = instruction.operand {
                    // For simplicity, use symbol ID as string
                    let name = format!("global_{}", symbol.id());
                    if let Some(value) = self.globals.get(&name) {
                        self.stack.push(value.clone());
                        Ok(ControlFlow::Continue)
                    } else {
                        Err(Box::new(Error::runtime_error(format!("Undefined global variable: {}", name), None))
                    }
                } else {
                    Err(Box::new(Error::runtime_error("LoadGlobal requires symbol operand".to_string(), None))
                }
            }
            
            OpCode::Pop => {
                if !self.stack.is_empty() {
                    self.stack.pop();
                    Ok(ControlFlow::Continue)
                } else {
                    Err(Box::new(Error::runtime_error("Cannot pop from empty stack".to_string(), None))
                }
            }
            
            OpCode::Add => {
                if self.stack.len() >= 2 {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    
                    match (a.as_number(), b.as_number()) {
                        (Some(a_num), Some(b_num)) => {
                            self.stack.push(Value::number(a_num + b_num));
                            self.stats.optimized_operations += 1;
                            Ok(ControlFlow::Continue)
                        }
                        _ => Err(Box::new(Error::runtime_error("Addition requires numeric operands".to_string(), None))
                    }
                } else {
                    Err(Box::new(Error::runtime_error("Addition requires 2 operands on stack".to_string(), None))
                }
            }
            
            OpCode::Sub => {
                if self.stack.len() >= 2 {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    
                    match (a.as_number(), b.as_number()) {
                        (Some(a_num), Some(b_num)) => {
                            self.stack.push(Value::number(a_num - b_num));
                            self.stats.optimized_operations += 1;
                            Ok(ControlFlow::Continue)
                        }
                        _ => Err(Box::new(Error::runtime_error("Subtraction requires numeric operands".to_string(), None))
                    }
                } else {
                    Err(Box::new(Error::runtime_error("Subtraction requires 2 operands on stack".to_string(), None))
                }
            }
            
            OpCode::Mul => {
                if self.stack.len() >= 2 {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    
                    match (a.as_number(), b.as_number()) {
                        (Some(a_num), Some(b_num)) => {
                            self.stack.push(Value::number(a_num * b_num));
                            self.stats.optimized_operations += 1;
                            Ok(ControlFlow::Continue)
                        }
                        _ => Err(Box::new(Error::runtime_error("Multiplication requires numeric operands".to_string(), None))
                    }
                } else {
                    Err(Box::new(Error::runtime_error("Multiplication requires 2 operands on stack".to_string(), None))
                }
            }
            
            OpCode::Cons => {
                if self.stack.len() >= 2 {
                    let cdr = self.stack.pop().unwrap();
                    let car = self.stack.pop().unwrap();
                    self.stack.push(Value::pair(car, cdr));
                    self.stats.optimized_operations += 1;
                    Ok(ControlFlow::Continue)
                } else {
                    Err(Box::new(Error::runtime_error("Cons requires 2 operands on stack".to_string(), None))
                }
            }
            
            OpCode::Car => {
                if let Some(pair) = self.stack.pop() {
                    match pair {
                        Value::Pair(car, _) => {
                            self.stack.push((*car).clone());
                            self.stats.optimized_operations += 1;
                            Ok(ControlFlow::Continue)
                        }
                        _ => Err(Box::new(Error::runtime_error("Car requires pair operand".to_string(), None))
                    }
                } else {
                    Err(Box::new(Error::runtime_error("Car requires 1 operand on stack".to_string(), None))
                }
            }
            
            OpCode::Cdr => {
                if let Some(pair) = self.stack.pop() {
                    match pair {
                        Value::Pair(_, cdr) => {
                            self.stack.push((*cdr).clone());
                            self.stats.optimized_operations += 1;
                            Ok(ControlFlow::Continue)
                        }
                        _ => Err(Box::new(Error::runtime_error("Cdr requires pair operand".to_string(), None))
                    }
                } else {
                    Err(Box::new(Error::runtime_error("Cdr requires 1 operand on stack".to_string(), None))
                }
            }
            
            OpCode::Halt => {
                let result = if !self.stack.is_empty() {
                    self.stack.pop().unwrap()
                } else {
                    Value::Unspecified
                };
                Ok(ControlFlow::Return(result))
            }
            
            _ => {
                // For now, return error for unimplemented instructions
                Err(Box::new(Error::runtime_error(format!("Unimplemented opcode: {:?}", instruction.opcode), None))
            }
        }
    }
    
    /// Converts a constant value to a runtime value.
    fn constant_to_value(&self, constant: &ConstantValue) -> Result<Value> {
        match constant {
            ConstantValue::Number(n) => Ok(Value::number(*n)),
            ConstantValue::String(s) => Ok(Value::string(s.clone())),
            ConstantValue::Boolean(b) => Ok(Value::boolean(*b)),
            ConstantValue::Symbol(symbol) => Ok(Value::symbol(*symbol)),
            _ => Err(Box::new(Error::runtime_error("Unsupported constant type".to_string(), None))
        }
    }
    
    /// Configures the virtual machine.
    pub fn configure(&mut self, config: VmConfig) {
        self.config = config;
    }
    
    /// Gets current VM statistics.
    pub fn get_stats(&self) -> super::VmStats {
        super::VmStats {
            instructions_executed: self.stats.instructions_executed,
            execution_time_us: self.stats.execution_time_us,
            function_calls: self.stats.function_calls,
            max_stack_depth: self.stats.max_stack_depth,
            gc_count: self.stats.gc_count,
            optimized_operations: self.stats.optimized_operations,
        }
    }
    
    /// Resets the virtual machine state.
    pub fn reset(&mut self) {
        self.stack.clear();
        self.call_stack.clear();
        self.globals.clear();
        self.state = VmState::Ready;
        self.stats = VmStats {
            instructions_executed: 0,
            execution_time_us: 0,
            function_calls: 0,
            max_stack_depth: 0,
            gc_count: 0,
            optimized_operations: 0,
        };
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Control flow result from instruction execution.
#[derive(Debug, Clone)]
enum ControlFlow {
    /// Continue to next instruction
    Continue,
    /// Jump to instruction at index
    Jump(usize),
    /// Return with value
    Return(Value),
    /// Error occurred
    Error(Error),
    /// Yield with value
    Yield(Value),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::instruction::{Instruction, OpCode, Operand, ConstantPool, ConstantValue, Bytecode};
    
    #[test]
    fn test_vm_creation() {
        let vm = VirtualMachine::new();
        assert!(matches!(vm.state, VmState::Ready));
        assert_eq!(vm.stack.len(), 0);
    }
    
    #[test]
    fn test_load_const_execution() {
        let mut vm = VirtualMachine::new();
        let mut constant_pool = ConstantPool::new();
        let const_index = constant_pool.add_constant(ConstantValue::Number(42.0));
        
        let mut bytecode = Bytecode::new();
        bytecode.add_instruction(Instruction::with_operand(OpCode::LoadConst, Operand::ConstIndex(const_index)));
        bytecode.add_instruction(Instruction::new(OpCode::Halt));
        bytecode.constants = constant_pool.clone());
        
        let result = vm.execute(&bytecode, &constant_pool).unwrap();
        
        if let ExecutionResult::Value(value) = result {
            assert_eq!(value.as_number(), Some(42.0));
        } else {
            panic!("Expected Value result");
        }
    }
    
    #[test]
    fn test_arithmetic_execution() {
        let mut vm = VirtualMachine::new();
        let mut constant_pool = ConstantPool::new();
        let const1 = constant_pool.add_constant(ConstantValue::Number(10.0));
        let const2 = constant_pool.add_constant(ConstantValue::Number(5.0));
        
        let mut bytecode = Bytecode::new();
        bytecode.add_instruction(Instruction::with_operand(OpCode::LoadConst, Operand::ConstIndex(const1)));
        bytecode.add_instruction(Instruction::with_operand(OpCode::LoadConst, Operand::ConstIndex(const2)));
        bytecode.add_instruction(Instruction::new(OpCode::Add));
        bytecode.add_instruction(Instruction::new(OpCode::Halt));
        bytecode.constants = constant_pool.clone());
        
        let result = vm.execute(&bytecode, &constant_pool).unwrap();
        
        if let ExecutionResult::Value(value) = result {
            assert_eq!(value.as_number(), Some(15.0));
        } else {
            panic!("Expected Value result");
        }
        
        let stats = vm.get_stats();
        assert_eq!(stats.optimized_operations, 1); // The add operation
    }
    
    #[test]
    fn test_cons_car_cdr() {
        let mut vm = VirtualMachine::new();
        let mut constant_pool = ConstantPool::new();
        let const1 = constant_pool.add_constant(ConstantValue::Number(1.0));
        let const2 = constant_pool.add_constant(ConstantValue::Number(2.0));
        
        let mut bytecode = Bytecode::new();
        // Create pair (1 . 2)
        bytecode.add_instruction(Instruction::with_operand(OpCode::LoadConst, Operand::ConstIndex(const1)));
        bytecode.add_instruction(Instruction::with_operand(OpCode::LoadConst, Operand::ConstIndex(const2)));
        bytecode.add_instruction(Instruction::new(OpCode::Cons));
        // Get car
        bytecode.add_instruction(Instruction::new(OpCode::Car));
        bytecode.add_instruction(Instruction::new(OpCode::Halt));
        bytecode.constants = constant_pool.clone());
        
        let result = vm.execute(&bytecode, &constant_pool).unwrap();
        
        if let ExecutionResult::Value(value) = result {
            assert_eq!(value.as_number(), Some(1.0));
        } else {
            panic!("Expected Value result");
        }
    }
}
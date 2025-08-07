//! Bytecode instruction definitions and constant pool management.

#![allow(missing_docs)]

use crate::eval::Value;
use crate::utils::SymbolId;
use std::collections::HashMap;
use std::fmt;

/// Bytecode operation codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OpCode {
    // Stack operations
    LoadConst = 0x00,    // Load constant from pool
    LoadLocal = 0x01,    // Load local variable
    LoadGlobal = 0x02,   // Load global variable
    StoreLocal = 0x03,   // Store to local variable
    StoreGlobal = 0x04,  // Store to global variable
    Pop = 0x05,          // Pop top value from stack
    Dup = 0x06,          // Duplicate top value
    
    // Arithmetic operations
    Add = 0x10,          // Add two numbers
    Sub = 0x11,          // Subtract two numbers
    Mul = 0x12,          // Multiply two numbers
    Div = 0x13,          // Divide two numbers
    Mod = 0x14,          // Modulo operation
    Neg = 0x15,          // Negate number
    
    // Comparison operations
    Eq = 0x20,           // Equal comparison
    Ne = 0x21,           // Not equal comparison
    Lt = 0x22,           // Less than comparison
    Le = 0x23,           // Less than or equal comparison
    Gt = 0x24,           // Greater than comparison
    Ge = 0x25,           // Greater than or equal comparison
    
    // Logical operations
    Not = 0x30,          // Logical not
    And = 0x31,          // Logical and
    Or = 0x32,           // Logical or
    
    // Control flow
    Jump = 0x40,         // Unconditional jump
    JumpIfFalse = 0x41,  // Jump if top of stack is false
    JumpIfTrue = 0x42,   // Jump if top of stack is true
    Call = 0x43,         // Function call
    TailCall = 0x44,     // Tail call (optimized)
    Return = 0x45,       // Return from function
    
    // List operations
    Cons = 0x50,         // Create pair
    Car = 0x51,          // Get car of pair
    Cdr = 0x52,          // Get cdr of pair
    IsNull = 0x53,       // Check if null
    IsPair = 0x54,       // Check if pair
    
    // Vector operations
    MakeVector = 0x60,   // Create vector
    VectorRef = 0x61,    // Get vector element
    VectorSet = 0x62,    // Set vector element
    VectorLength = 0x63, // Get vector length
    
    // Type predicates
    IsNumber = 0x70,     // Check if number
    IsString = 0x71,     // Check if string
    IsSymbol = 0x72,     // Check if symbol
    IsBoolean = 0x73,    // Check if boolean
    IsProcedure = 0x74,  // Check if procedure
    
    // Special operations
    MakeClosure = 0x80,  // Create closure
    Apply = 0x81,        // Apply function to arguments
    CallCC = 0x82,       // Call with current continuation
    Yield = 0x83,        // Yield control (for generators)
    
    // Debugging and profiling
    Debug = 0xF0,        // Debug breakpoint
    Profile = 0xF1,      // Profiling marker
    
    // Halt execution
    Halt = 0xFF,         // Stop execution
}

/// Operand for bytecode instructions.
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    /// No operand
    None,
    /// 8-bit unsigned integer
    U8(u8),
    /// 16-bit unsigned integer  
    U16(u16),
    /// 32-bit unsigned integer
    U32(u32),
    /// Index into constant pool
    ConstIndex(u32),
    /// Local variable index
    LocalIndex(u16),
    /// Jump offset (signed)
    JumpOffset(i32),
    /// Symbol identifier
    Symbol(SymbolId),
}

/// A complete bytecode instruction.
#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    /// Operation code
    pub opcode: OpCode,
    /// Operand (if any)
    pub operand: Operand,
    /// Source location information (for debugging)
    pub source_location: Option<SourceLocation>,
}

/// Source location information for debugging.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    /// Line number in source code
    pub line: u32,
    /// Column number in source code
    pub column: u32,
    /// Source file name
    pub filename: Option<String>,
}

/// Values that can be stored in the constant pool.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    /// Scheme value
    Value(Value),
    /// String literal
    String(String),
    /// Number literal
    Number(f64),
    /// Boolean literal
    Boolean(bool),
    /// Symbol
    Symbol(SymbolId),
    /// Bytecode (for embedded functions)
    Bytecode(Vec<Instruction>),
}

impl std::hash::Hash for ConstantValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::mem;
        match self {
            ConstantValue::Value(v) => {
                0u8.hash(state);
                // Hash the pointer address as a proxy for the value
                let ptr = v as *const Value as usize;
                ptr.hash(state);
            }
            ConstantValue::String(s) => {
                1u8.hash(state);
                s.hash(state);
            }
            ConstantValue::Number(n) => {
                2u8.hash(state);
                // Hash the bit representation of the float
                n.to_bits().hash(state);
            }
            ConstantValue::Boolean(b) => {
                3u8.hash(state);
                b.hash(state);
            }
            ConstantValue::Symbol(s) => {
                4u8.hash(state);
                s.hash(state);
            }
            ConstantValue::Bytecode(bc) => {
                5u8.hash(state);
                // Hash length and first few instructions as a proxy
                bc.len().hash(state);
                if !bc.is_empty() {
                    // Hash the opcode of the first instruction
                    mem::discriminant(&bc[0].opcode).hash(state);
                }
            }
        }
    }
}

impl Eq for ConstantValue {}

/// Constant pool for storing literals and other compile-time values.
#[derive(Debug, Clone)]
pub struct ConstantPool {
    /// Constants indexed by their pool index
    constants: Vec<ConstantValue>,
    /// Reverse lookup for deduplication
    constant_map: HashMap<ConstantValue, u32>,
}

impl ConstantPool {
    /// Creates a new empty constant pool.
    pub fn new() -> Self {
        Self {
            constants: Vec::new(),
            constant_map: HashMap::new(),
        }
    }
    
    /// Adds a constant to the pool, returning its index.
    /// If the constant already exists, returns the existing index.
    pub fn add_constant(&mut self, value: ConstantValue) -> u32 {
        if let Some(&index) = self.constant_map.get(&value) {
            return index;
        }
        
        let index = self.constants.len() as u32;
        self.constants.push(value.clone());
        self.constant_map.insert(value, index);
        index
    }
    
    /// Gets a constant by its index.
    pub fn get_constant(&self, index: u32) -> Option<&ConstantValue> {
        self.constants.get(index as usize)
    }
    
    /// Returns the number of constants in the pool.
    pub fn len(&self) -> usize {
        self.constants.len()
    }
    
    /// Returns true if the constant pool is empty.
    pub fn is_empty(&self) -> bool {
        self.constants.is_empty()
    }
    
    /// Clears all constants from the pool.
    pub fn clear(&mut self) {
        self.constants.clear();
        self.constant_map.clear();
    }
    
    /// Gets an iterator over all constants.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &ConstantValue)> {
        self.constants.iter().enumerate().map(|(i, v)| (i as u32, v))
    }
    
    /// Estimates the memory usage of the constant pool in bytes.
    pub fn memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() + 
        self.constants.capacity() * std::mem::size_of::<ConstantValue>() +
        self.constant_map.capacity() * (std::mem::size_of::<ConstantValue>() + std::mem::size_of::<u32>())
    }
}

impl Default for ConstantPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Instruction {
    /// Creates a new instruction with no operand.
    pub fn new(opcode: OpCode) -> Self {
        Self {
            opcode,
            operand: Operand::None,
            source_location: None,
        }
    }
    
    /// Creates a new instruction with an operand.
    pub fn with_operand(opcode: OpCode, operand: Operand) -> Self {
        Self {
            opcode,
            operand,
            source_location: None,
        }
    }
    
    /// Creates a new instruction with source location.
    pub fn with_location(opcode: OpCode, operand: Operand, location: SourceLocation) -> Self {
        Self {
            opcode,
            operand,
            source_location: Some(location),
        }
    }
    
    /// Returns the size of this instruction in bytes when encoded.
    pub fn encoded_size(&self) -> usize {
        1 + match &self.operand {
            Operand::None => 0,
            Operand::U8(_) => 1,
            Operand::U16(_) => 2,
            Operand::U32(_) => 4,
            Operand::ConstIndex(_) => 4,
            Operand::LocalIndex(_) => 2,
            Operand::JumpOffset(_) => 4,
            Operand::Symbol(_) => 8, // Assuming 64-bit symbol IDs
        }
    }
    
    /// Returns true if this instruction can be optimized away.
    pub fn is_removable(&self) -> bool {
        match self.opcode {
            OpCode::Debug | OpCode::Profile => true,
            OpCode::Pop => true, // In some contexts
            _ => false,
        }
    }
    
    /// Returns true if this instruction affects control flow.
    pub fn is_control_flow(&self) -> bool {
        matches!(self.opcode, 
            OpCode::Jump | OpCode::JumpIfFalse | OpCode::JumpIfTrue | 
            OpCode::Call | OpCode::TailCall | OpCode::Return |
            OpCode::CallCC | OpCode::Yield
        )
    }
    
    /// Returns true if this instruction is a terminator (ends a basic block).
    pub fn is_terminator(&self) -> bool {
        matches!(self.opcode, 
            OpCode::Jump | OpCode::Return | OpCode::Halt | OpCode::Yield
        )
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.operand {
            Operand::None => write!(f, "{:?}", self.opcode),
            operand => write!(f, "{:?} {:?}", self.opcode, operand),
        }
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            OpCode::LoadConst => "LOAD_CONST",
            OpCode::LoadLocal => "LOAD_LOCAL",
            OpCode::LoadGlobal => "LOAD_GLOBAL",
            OpCode::StoreLocal => "STORE_LOCAL",
            OpCode::StoreGlobal => "STORE_GLOBAL",
            OpCode::Pop => "POP",
            OpCode::Dup => "DUP",
            
            OpCode::Add => "ADD",
            OpCode::Sub => "SUB",
            OpCode::Mul => "MUL",
            OpCode::Div => "DIV",
            OpCode::Mod => "MOD",
            OpCode::Neg => "NEG",
            
            OpCode::Eq => "EQ",
            OpCode::Ne => "NE",
            OpCode::Lt => "LT",
            OpCode::Le => "LE",
            OpCode::Gt => "GT",
            OpCode::Ge => "GE",
            
            OpCode::Not => "NOT",
            OpCode::And => "AND",
            OpCode::Or => "OR",
            
            OpCode::Jump => "JUMP",
            OpCode::JumpIfFalse => "JUMP_IF_FALSE",
            OpCode::JumpIfTrue => "JUMP_IF_TRUE",
            OpCode::Call => "CALL",
            OpCode::TailCall => "TAIL_CALL",
            OpCode::Return => "RETURN",
            
            OpCode::Cons => "CONS",
            OpCode::Car => "CAR",
            OpCode::Cdr => "CDR",
            OpCode::IsNull => "IS_NULL",
            OpCode::IsPair => "IS_PAIR",
            
            OpCode::MakeVector => "MAKE_VECTOR",
            OpCode::VectorRef => "VECTOR_REF",
            OpCode::VectorSet => "VECTOR_SET",
            OpCode::VectorLength => "VECTOR_LENGTH",
            
            OpCode::IsNumber => "IS_NUMBER",
            OpCode::IsString => "IS_STRING", 
            OpCode::IsSymbol => "IS_SYMBOL",
            OpCode::IsBoolean => "IS_BOOLEAN",
            OpCode::IsProcedure => "IS_PROCEDURE",
            
            OpCode::MakeClosure => "MAKE_CLOSURE",
            OpCode::Apply => "APPLY",
            OpCode::CallCC => "CALL_CC",
            OpCode::Yield => "YIELD",
            
            OpCode::Debug => "DEBUG",
            OpCode::Profile => "PROFILE",
            OpCode::Halt => "HALT",
        };
        write!(f, "{}", name)
    }
}

/// A sequence of bytecode instructions with associated metadata.
#[derive(Debug, Clone)]
pub struct Bytecode {
    /// The instructions
    pub instructions: Vec<Instruction>,
    /// Constant pool
    pub constants: ConstantPool,
    /// Entry point (instruction index)
    pub entry_point: usize,
    /// Local variable count
    pub local_count: usize,
    /// Maximum stack depth required
    pub max_stack_depth: usize,
}

impl Bytecode {
    /// Creates new empty bytecode.
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: ConstantPool::new(),
            entry_point: 0,
            local_count: 0,
            max_stack_depth: 0,
        }
    }
    
    /// Adds an instruction to the bytecode.
    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }
    
    /// Adds multiple instructions to the bytecode.
    pub fn add_instructions(&mut self, instructions: Vec<Instruction>) {
        self.instructions.extend(instructions);
    }
    
    /// Gets the length of the bytecode in instructions.
    pub fn len(&self) -> usize {
        self.instructions.len()
    }
    
    /// Returns true if the bytecode is empty.
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
    
    /// Estimates the total size of the bytecode in bytes.
    pub fn estimated_size(&self) -> usize {
        let instruction_size: usize = self.instructions.iter().map(|i| i.encoded_size()).sum();
        instruction_size + self.constants.memory_usage()
    }
    
    /// Disassembles the bytecode to a human-readable string.
    pub fn disassemble(&self) -> String {
        let mut output = String::new();
        
        output.push_str("=== Bytecode Disassembly ===\n");
        output.push_str(&format!("Entry point: {}\n", self.entry_point));
        output.push_str(&format!("Local variables: {}\n", self.local_count));
        output.push_str(&format!("Max stack depth: {}\n", self.max_stack_depth));
        output.push_str(&format!("Instructions: {}\n", self.instructions.len()));
        output.push_str(&format!("Constants: {}\n\n", self.constants.len()));
        
        // Disassemble constants
        if !self.constants.is_empty() {
            output.push_str("=== Constants ===\n");
            for (index, constant) in self.constants.iter() {
                output.push_str(&format!("{:4}: {:?}\n", index, constant));
            }
            output.push('\n');
        }
        
        // Disassemble instructions
        output.push_str("=== Instructions ===\n");
        for (index, instruction) in self.instructions.iter().enumerate() {
            let marker = if index == self.entry_point { ">" } else { " " };
            output.push_str(&format!("{}{:4}: {}\n", marker, index, instruction));
        }
        
        output
    }
    
    /// Validates the bytecode for correctness.
    pub fn validate(&self) -> Result<(), String> {
        // Check entry point
        if self.entry_point >= self.instructions.len() {
            return Err("Entry point is out of bounds".to_string());
        }
        
        // Check constant references
        for (index, instruction) in self.instructions.iter().enumerate() {
            if let Operand::ConstIndex(const_index) = &instruction.operand {
                if *const_index >= self.constants.len() as u32 {
                    return Err(format!("Instruction {} references invalid constant {}", index, const_index));
                }
            }
        }
        
        // Check jump targets
        for (index, instruction) in self.instructions.iter().enumerate() {
            match instruction.opcode {
                OpCode::Jump | OpCode::JumpIfFalse | OpCode::JumpIfTrue => {
                    if let Operand::JumpOffset(offset) = &instruction.operand {
                        let target = (index as i32) + offset;
                        if target < 0 || target >= self.instructions.len() as i32 {
                            return Err(format!("Instruction {} has invalid jump target {}", index, target));
                        }
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constant_pool() {
        let mut pool = ConstantPool::new();
        
        let index1 = pool.add_constant(ConstantValue::Number(42.0));
        let index2 = pool.add_constant(ConstantValue::String("hello".to_string()));
        let index3 = pool.add_constant(ConstantValue::Number(42.0)); // Duplicate
        
        assert_eq!(index1, 0);
        assert_eq!(index2, 1);  
        assert_eq!(index3, 0); // Should reuse existing constant
        assert_eq!(pool.len(), 2);
        
        let constant = pool.get_constant(0).unwrap();
        assert_eq!(*constant, ConstantValue::Number(42.0));
    }
    
    #[test]
    fn test_instruction_creation() {
        let inst1 = Instruction::new(OpCode::Add);
        assert_eq!(inst1.opcode, OpCode::Add);
        assert_eq!(inst1.operand, Operand::None);
        
        let inst2 = Instruction::with_operand(OpCode::LoadConst, Operand::ConstIndex(5));
        assert_eq!(inst2.opcode, OpCode::LoadConst);
        assert_eq!(inst2.operand, Operand::ConstIndex(5));
    }
    
    #[test]
    fn test_bytecode_validation() {
        let mut bytecode = Bytecode::new();
        bytecode.add_instruction(Instruction::with_operand(OpCode::LoadConst, Operand::ConstIndex(0)));
        bytecode.add_instruction(Instruction::new(OpCode::Return));
        
        // Should fail validation - constant 0 doesn't exist
        assert!(bytecode.validate().is_err());
        
        // Add the constant
        bytecode.constants.add_constant(ConstantValue::Number(42.0));
        
        // Should now pass validation
        assert!(bytecode.validate().is_ok());
    }
    
    #[test]
    fn test_bytecode_disassembly() {
        let mut bytecode = Bytecode::new();
        bytecode.constants.add_constant(ConstantValue::Number(42.0));
        bytecode.add_instruction(Instruction::with_operand(OpCode::LoadConst, Operand::ConstIndex(0)));
        bytecode.add_instruction(Instruction::new(OpCode::Return));
        
        let disasm = bytecode.disassemble();
        assert!(disasm.contains("LOAD_CONST"));
        assert!(disasm.contains("RETURN"));
        assert!(disasm.contains("Constants"));
        assert!(disasm.contains("42"));
    }
    
    #[test]
    fn test_instruction_properties() {
        let jump_inst = Instruction::with_operand(OpCode::Jump, Operand::JumpOffset(10));
        assert!(jump_inst.is_control_flow());
        assert!(jump_inst.is_terminator());
        
        let add_inst = Instruction::new(OpCode::Add);
        assert!(!add_inst.is_control_flow());
        assert!(!add_inst.is_terminator());
        
        let debug_inst = Instruction::new(OpCode::Debug);
        assert!(debug_inst.is_removable());
    }
}
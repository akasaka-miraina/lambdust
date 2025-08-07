//! Configuration for bytecode optimizations.

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
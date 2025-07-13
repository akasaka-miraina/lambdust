//! Core Types Module
//!
//! このモジュールはJITループ最適化の基本型定義を提供します。
//! ループパターン、JITヒント、最適化統計、コード特性を含みます。

use crate::ast::Expr;
use std::collections::HashMap;
use std::time::Instant;

/// Loop pattern classification for JIT optimization
#[derive(Debug, Clone, PartialEq)]
pub enum LoopPattern {
    /// Simple counting loop (do ((i start end)) (test) body)
    CountingLoop {
        /// Loop variable name
        variable: String,
        /// Starting value
        start: i64,
        /// Ending value
        end: i64,
        /// Step increment
        step: i64,
    },

    /// List iteration loop (for-each pattern)
    ListIteration {
        /// Iterator variable name
        variable: String,
        /// List expression to iterate over
        list_expr: Expr,
    },

    /// Vector iteration loop
    VectorIteration {
        /// Iterator variable name
        variable: String,
        /// Vector expression to iterate over
        vector_expr: Expr,
    },

    /// Conditional accumulation loop
    AccumulationLoop {
        /// Accumulator variable name
        accumulator: String,
        /// Loop termination condition
        condition: Expr,
        /// Accumulator update expression
        update_expr: Expr,
    },

    /// Complex loop requiring fallback to CPS
    ComplexLoop,
}

/// JIT compilation hint for optimization strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JitHint {
    /// Compile to native iteration immediately
    CompileImmediate,
    /// Compile after threshold executions
    CompileDeferred,
    /// Profile and decide at runtime
    ProfileAndDecide,
    /// Do not compile - use CPS evaluation
    NoCompile,
}

/// Native iteration strategy
#[derive(Debug, Clone, PartialEq)]
pub enum IterationStrategy {
    /// Rust for-loop with integer range
    NativeForLoop {
        /// Starting value
        start: i64,
        /// Ending value
        end: i64,
        /// Step increment
        step: i64,
    },

    /// Manual loop with exit conditions
    ManualLoop {
        /// Maximum allowed iterations
        max_iterations: usize,
    },
}

/// Iterator type for native iteration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IteratorType {
    /// Integer range iteration
    Range,
    /// List iteration
    List,
    /// Vector iteration
    Vector,
    /// Custom iterator
    Custom,
}

/// Compiled native loop representation
#[derive(Debug, Clone)]
pub struct CompiledLoop {
    /// Original pattern that was compiled
    pub pattern: LoopPattern,
    /// Native iteration strategy
    pub strategy: IterationStrategy,
    /// Compilation timestamp
    pub compiled_at: Instant,
    /// Execution count since compilation
    pub execution_count: usize,
}

/// Generated native iteration code
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    /// Iteration strategy
    pub strategy: IterationStrategy,
    /// Performance characteristics
    pub characteristics: CodeCharacteristics,
    /// Generated at timestamp
    pub generated_at: Instant,
}

/// Code performance characteristics
#[derive(Debug, Clone)]
pub struct CodeCharacteristics {
    /// Estimated iterations per second
    pub iterations_per_second: f64,
    /// Memory overhead per iteration
    pub memory_overhead: usize,
    /// CPU cache friendliness (0.0-1.0)
    pub cache_friendliness: f64,
}

/// JIT optimization statistics
#[derive(Debug, Clone, Default)]
pub struct JitOptimizationStats {
    /// Total loop patterns detected
    pub total_patterns: usize,
    /// Successfully compiled patterns
    pub compiled_patterns: usize,
    /// Compilation success rate
    pub compilation_rate: f64,
    /// Pattern detection breakdown
    pub pattern_detections: HashMap<String, usize>,
    /// Code generation breakdown
    pub code_generations: HashMap<String, usize>,
}

impl Default for JitHint {
    fn default() -> Self {
        Self::ProfileAndDecide
    }
}

impl Default for IteratorType {
    fn default() -> Self {
        Self::Range
    }
}

impl Default for CodeCharacteristics {
    fn default() -> Self {
        Self {
            iterations_per_second: 1_000_000.0, // 1M iterations/sec baseline
            memory_overhead: 64, // 64 bytes per iteration baseline
            cache_friendliness: 0.8, // 80% cache friendly baseline
        }
    }
}

impl JitOptimizationStats {
    /// Update compilation rate based on current counts
    pub fn update_compilation_rate(&mut self) {
        if self.total_patterns > 0 {
            self.compilation_rate = self.compiled_patterns as f64 / self.total_patterns as f64;
        } else {
            self.compilation_rate = 0.0;
        }
    }

    /// Add pattern detection
    pub fn record_pattern_detection(&mut self, pattern_name: &str) {
        self.total_patterns += 1;
        *self.pattern_detections.entry(pattern_name.to_string()).or_insert(0) += 1;
        self.update_compilation_rate();
    }

    /// Record successful compilation
    pub fn record_successful_compilation(&mut self, pattern_name: &str) {
        self.compiled_patterns += 1;
        *self.code_generations.entry(pattern_name.to_string()).or_insert(0) += 1;
        self.update_compilation_rate();
    }

    /// Get pattern detection count
    pub fn pattern_count(&self, pattern_name: &str) -> usize {
        self.pattern_detections.get(pattern_name).copied().unwrap_or(0)
    }

    /// Get code generation count
    pub fn generation_count(&self, pattern_name: &str) -> usize {
        self.code_generations.get(pattern_name).copied().unwrap_or(0)
    }

    /// Calculate overall efficiency
    pub fn efficiency(&self) -> f64 {
        self.compilation_rate * 100.0
    }
}

impl LoopPattern {
    /// Get pattern name as string
    pub fn pattern_name(&self) -> &'static str {
        match self {
            LoopPattern::CountingLoop { .. } => "CountingLoop",
            LoopPattern::ListIteration { .. } => "ListIteration",
            LoopPattern::VectorIteration { .. } => "VectorIteration",
            LoopPattern::AccumulationLoop { .. } => "AccumulationLoop",
            LoopPattern::ComplexLoop => "ComplexLoop",
        }
    }

    /// Check if pattern is compilable to native code
    pub fn is_compilable(&self) -> bool {
        !matches!(self, LoopPattern::ComplexLoop)
    }

    /// Get estimated complexity
    pub fn complexity(&self) -> f64 {
        match self {
            LoopPattern::CountingLoop { .. } => 1.0,
            LoopPattern::ListIteration { .. } => 2.0,
            LoopPattern::VectorIteration { .. } => 1.5,
            LoopPattern::AccumulationLoop { .. } => 3.0,
            LoopPattern::ComplexLoop => 10.0,
        }
    }
}

impl IterationStrategy {
    /// Get strategy name as string
    pub fn strategy_name(&self) -> &'static str {
        match self {
            IterationStrategy::NativeForLoop { .. } => "NativeForLoop",
            IterationStrategy::ManualLoop { .. } => "ManualLoop",
        }
    }

    /// Calculate estimated performance improvement over CPS
    pub fn performance_multiplier(&self) -> f64 {
        match self {
            IterationStrategy::NativeForLoop { .. } => 50.0, // 50x faster than CPS
            IterationStrategy::ManualLoop { .. } => 20.0,     // 20x faster than CPS
        }
    }
}

impl CompiledLoop {
    /// Create new compiled loop
    pub fn new(pattern: LoopPattern, strategy: IterationStrategy) -> Self {
        Self {
            pattern,
            strategy,
            compiled_at: Instant::now(),
            execution_count: 0,
        }
    }

    /// Record execution
    pub fn record_execution(&mut self) {
        self.execution_count += 1;
    }

    /// Get age since compilation
    pub fn age(&self) -> std::time::Duration {
        self.compiled_at.elapsed()
    }
}

impl GeneratedCode {
    /// Create new generated code
    pub fn new(strategy: IterationStrategy) -> Self {
        Self {
            strategy,
            characteristics: CodeCharacteristics::default(),
            generated_at: Instant::now(),
        }
    }

    /// Create with custom characteristics
    pub fn with_characteristics(strategy: IterationStrategy, characteristics: CodeCharacteristics) -> Self {
        Self {
            strategy,
            characteristics,
            generated_at: Instant::now(),
        }
    }

    /// Get code age
    pub fn age(&self) -> std::time::Duration {
        self.generated_at.elapsed()
    }

    /// Calculate performance score (higher is better)
    pub fn performance_score(&self) -> f64 {
        let base_score = self.characteristics.iterations_per_second / 1_000_000.0;
        let cache_bonus = self.characteristics.cache_friendliness;
        let memory_penalty = (self.characteristics.memory_overhead as f64 / 1024.0).min(1.0);
        
        base_score * cache_bonus * (1.0 - memory_penalty * 0.1)
    }
}
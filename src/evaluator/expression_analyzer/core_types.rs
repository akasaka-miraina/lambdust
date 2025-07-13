//! Core Types Module
//!
//! このモジュールは式解析システムの基本型定義を提供します。
//! 解析結果、型ヒント、複雑度、最適化ヒントなどの型を含みます。

use crate::value::Value;

/// Expression analysis results containing optimization hints
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Whether the expression is a constant that can be folded
    pub is_constant: bool,
    /// Pre-computed constant value (if `is_constant` is true)
    pub constant_value: Option<Value>,
    /// Inferred type information
    pub type_hint: TypeHint,
    /// Estimated evaluation complexity
    pub complexity: EvaluationComplexity,
    /// Whether the expression has side effects
    pub has_side_effects: bool,
    /// Variable dependencies
    pub dependencies: Vec<String>,
    /// Optimization suggestions
    pub optimizations: Vec<OptimizationHint>,
}

/// Type hint information for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum TypeHint {
    /// Unknown type
    Unknown,
    /// Boolean type
    Boolean,
    /// Number type
    Number,
    /// String type
    String,
    /// Character type
    Character,
    /// Symbol type
    Symbol,
    /// List type
    List,
    /// Vector type
    Vector,
    /// Procedure type
    Procedure,
    /// Multiple possible types
    Union(Vec<TypeHint>),
}

/// Evaluation complexity estimation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvaluationComplexity {
    /// Constant time (literals, constants)
    Constant,
    /// Simple variable lookup
    Variable,
    /// Simple function call
    Simple,
    /// Moderate complexity (loops, conditions)
    Moderate,
    /// High complexity (recursion, complex computations)
    High,
}

impl EvaluationComplexity {
    /// Convert complexity enum to numeric score (0-100)
    pub fn complexity_score(&self) -> u32 {
        match self {
            EvaluationComplexity::Constant => 0,
            EvaluationComplexity::Variable => 10,
            EvaluationComplexity::Simple => 25,
            EvaluationComplexity::Moderate => 50,
            EvaluationComplexity::High => 75,
        }
    }
}

/// Optimization hints for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationHint {
    /// Expression can be constant-folded
    ConstantFold(Value),
    /// Variable can be inlined
    InlineVariable(String, Value),
    /// Function call can be specialized
    SpecializeCall(String, Vec<TypeHint>),
    /// Tail call optimization available
    TailCall,
    /// Dead code elimination possible
    DeadCode,
    /// Loop unrolling opportunity
    UnrollLoop(usize),
}

/// Statistics about optimization opportunities
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    /// Total expressions analyzed
    pub total_analyzed: usize,
    /// Number of constants found
    pub constants_found: usize,
    /// Number of tail calls identified
    pub tail_calls_found: usize,
    /// Number of pure function calls
    pub pure_calls_found: usize,
    /// Number of dead code segments
    pub dead_code_found: usize,
    /// Number of inlinable variables
    pub inlinable_vars_found: usize,
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
}
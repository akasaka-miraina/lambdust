//! Common types and enums used across program analysis components.

/// Type of definition.
#[derive(Debug, Clone, PartialEq)]
pub enum DefinitionType {
    /// Variable definition
    Variable,
    /// Function definition
    Function,
    /// Macro definition
    Macro,
    /// Constant definition
    Constant,
    /// Type definition
    Type,
}

/// Type of dependency.
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    /// Direct reference
    Reference,
    /// Function call
    Call,
    /// Macro use
    MacroUse,
    /// Type constraint
    TypeConstraint,
}

/// Type of scope.
#[derive(Debug, Clone, PartialEq)]
pub enum ScopeType {
    /// Global scope
    Global,
    /// Function definition scope
    Function,
    /// Lambda expression scope
    Lambda,
    /// Let binding scope
    Let,
    /// Letrec binding scope
    Letrec,
    /// Let* binding scope
    LetStar,
    /// Do loop scope
    Do,
    /// Macro expansion scope
    Macro,
}

/// Inferred type for an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum InferredType {
    /// Primitive types
    /// Boolean type.
    Boolean,
    /// Numeric type.
    Number,
    /// String type.
    String,
    /// Character type.
    Character,
    /// Symbol type.
    Symbol,
    /// Compound types
    Pair(Box<InferredType>, Box<InferredType>),
    /// List with element type.
    List(Box<InferredType>),
    /// Vector with element type.
    Vector(Box<InferredType>),
    /// Function type
    Function {
        /// Parameter types.
        parameters: Vec<InferredType>,
        /// Return type.
        return_type: Box<InferredType>,
    },
    /// Unknown type
    Unknown,
    /// Type variable
    Variable(String),
}

/// Type of constraint.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    /// Type equality constraint
    Equal,
    /// Subtype constraint
    Subtype,
    /// Supertype constraint
    Supertype,
}

/// Type of optimization.
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationType {
    /// Tail call optimization
    TailCall,
    /// Constant folding
    ConstantFolding,
    /// Dead code elimination
    DeadCode,
    /// Common subexpression elimination
    CommonSubexpression,
    /// Loop optimization
    Loop,
    /// Inlining
    Inline,
    /// Strength reduction
    StrengthReduction,
}

/// Impact of optimization.
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationImpact {
    /// Low performance impact
    Low,
    /// Medium performance impact
    Medium,
    /// High performance impact
    High,
}

/// Type of warning.
#[derive(Debug, Clone, PartialEq)]
pub enum WarningType {
    /// Unused variable warning
    UnusedVariable,
    /// Unused function warning
    UnusedFunction,
    /// Potentially uninitialized variable
    PotentiallyUninitializedVariable,
    /// Variable shadowing warning
    ShadowedVariable,
    /// Dead code warning
    DeadCode,
    /// Complex function warning
    ComplexFunction,
    /// Duplicated code warning
    DuplicatedCode,
    /// Performance issue warning
    PerformanceIssue,
}

/// Warning severity.
#[derive(Debug, Clone, PartialEq)]
pub enum WarningSeverity {
    /// Informational severity
    Info,
    /// Warning severity
    Warning,
    /// Error severity
    Error,
}
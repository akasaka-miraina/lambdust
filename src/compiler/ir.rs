//! Intermediate Representation (IR) for Native Compilation
//!
//! This module defines the IR used in the middle-end of the compiler,
//! specifically designed for Scheme's unique features including first-class
//! continuations and mandatory tail call optimization.

use crate::ast::{Expr, Formals};
use crate::diagnostics::{Error, Result};
use crate::utils::SymbolId;
use std::collections::HashMap;
use std::sync::Arc;

/// Continuation Passing Style (CPS) IR
///
/// This IR makes all control flow explicit through continuations,
/// enabling efficient native code generation for call/cc and tail calls.
#[derive(Debug, Clone)]
pub enum CpsExpr {
    /// Variable reference
    Var(SymbolId),

    /// Lambda abstraction (always in CPS form)
    /// Parameters include explicit continuation parameter
    Lambda {
        /// Function parameters
        params: Vec<SymbolId>,
        /// Continuation parameter for CPS
        cont_param: SymbolId,
        /// Function body expression
        body: Box<CpsExpr>,
        /// Variables captured from outer scope
        closure_vars: Vec<SymbolId>,
    },

    /// Record construction (tuples, structs)
    Record {
        /// Field expressions
        fields: Vec<CpsExpr>,
        /// Continuation to call with result
        cont: SymbolId,
        /// Variable to bind the result
        result_var: SymbolId,
    },

    /// Record field selection
    Select {
        /// Record to select from
        record: SymbolId,
        /// Field index to select
        index: usize,
        /// Continuation to call with result
        cont: SymbolId,
        /// Variable to bind the result
        result_var: SymbolId,
    },

    /// Function application (tail call)
    App {
        /// Function to apply
        func: SymbolId,
        /// Function arguments
        args: Vec<SymbolId>,
        /// Continuation to pass to function
        cont: SymbolId,
    },

    /// Primitive operation application
    Primop {
        /// Primitive operation to apply
        op: PrimitiveOp,
        /// Operation arguments
        args: Vec<SymbolId>,
        /// Continuation to call with result
        cont: SymbolId,
        /// Variable to bind the result
        result_var: SymbolId,
    },

    /// Continuation capture (call/cc)
    CallCC {
        /// Procedure to call with captured continuation
        proc: SymbolId,
        /// Current continuation
        cont: SymbolId,
    },

    /// Fix-point for recursive definitions
    Fix {
        /// Mutually recursive function definitions
        functions: Vec<(SymbolId, CpsExpr)>,
        /// Body expression where functions are in scope
        body: Box<CpsExpr>,
    },

    /// Conditional (if-then-else)
    If {
        /// Test expression variable
        test: SymbolId,
        /// Continuation for true branch
        then_cont: SymbolId,
        /// Continuation for false branch
        else_cont: SymbolId,
    },

    /// Program termination
    Halt(SymbolId),
}

/// Primitive operations in CPS IR
#[derive(Debug, Clone)]
pub enum PrimitiveOp {
    // Arithmetic
    /// Addition operation
    Add,
    /// Subtraction operation
    Sub,
    /// Multiplication operation
    Mul,
    /// Division operation
    Div,
    /// Modulo operation
    Mod,

    // Comparison
    /// Equality comparison
    Eq,
    /// Less than comparison
    Lt,
    /// Greater than comparison
    Gt,
    /// Less than or equal comparison
    Le,
    /// Greater than or equal comparison
    Ge,

    // Type predicates
    /// Number type predicate
    NumberP,
    /// String type predicate
    StringP,
    /// Pair type predicate
    PairP,
    /// Null type predicate
    NullP,
    /// Boolean type predicate
    BooleanP,

    // List operations
    /// Construct a pair (cons cell)
    Cons,
    /// Extract first element of pair
    Car,
    /// Extract second element of pair
    Cdr,

    // Vector operations
    /// Create a new vector
    MakeVector,
    /// Read vector element at index
    VectorRef,
    /// Set vector element at index
    VectorSet,

    // String operations
    /// Create a new string
    MakeString,
    /// Read string character at index
    StringRef,
    /// Set string character at index
    StringSet,

    // I/O operations
    /// Display value for human reading
    Display,
    /// Write value in machine-readable format
    Write,
    /// Read value from input
    Read,

    // Memory operations
    /// Box a value for mutable reference
    Box,
    /// Unbox a mutable reference
    Unbox,
    /// Set the contents of a box
    SetBox,
}

/// A-Normal Form (ANF) IR
///
/// Alternative IR that makes intermediate values explicit,
/// useful for certain optimizations and simpler code generation.
#[derive(Debug, Clone)]
pub enum AnfExpr {
    /// Atomic expressions (variables, literals)
    Atom(AtomicExpr),

    /// Let binding with immediate continuation
    Let {
        /// Variable to bind
        var: SymbolId,
        /// Value expression to bind
        value: Box<ComplexExpr>,
        /// Body expression where variable is in scope
        body: Box<AnfExpr>,
    },

    /// Tail position expression
    Tail(TailExpr),
}

/// Atomic expressions (can be used anywhere)
#[derive(Debug, Clone)]
pub enum AtomicExpr {
    /// Variable reference
    Var(SymbolId),
    /// Literal value
    Literal(LiteralValue),
}

/// Complex expressions (must be let-bound)
#[derive(Debug, Clone)]
pub enum ComplexExpr {
    /// Primitive operation
    Primop {
        /// Primitive operation to apply
        op: PrimitiveOp,
        /// Operation arguments
        args: Vec<AtomicExpr>,
    },

    /// Function call (non-tail)
    Call {
        /// Function to call
        func: AtomicExpr,
        /// Function arguments
        args: Vec<AtomicExpr>,
    },

    /// Lambda creation
    Lambda {
        /// Function parameters
        params: Vec<SymbolId>,
        /// Function body
        body: Box<AnfExpr>,
        /// Variables captured from outer scope
        closure_vars: Vec<SymbolId>,
    },

    /// Record construction
    Record(Vec<AtomicExpr>),

    /// Field selection
    Select {
        /// Record to select from
        record: AtomicExpr,
        /// Field index to select
        index: usize,
    },
}

/// Tail expressions (control flow)
#[derive(Debug, Clone)]
pub enum TailExpr {
    /// Return atomic value
    Return(AtomicExpr),

    /// Tail call
    TailCall {
        /// Function to call
        func: AtomicExpr,
        /// Function arguments
        args: Vec<AtomicExpr>,
    },

    /// Conditional
    If {
        /// Test expression
        test: AtomicExpr,
        /// Expression for true branch
        then_branch: Box<AnfExpr>,
        /// Expression for false branch
        else_branch: Box<AnfExpr>,
    },

    /// Call with current continuation
    CallCC(AtomicExpr),
}

/// Literal values in IR
#[derive(Debug, Clone)]
pub enum LiteralValue {
    /// Integer literal
    Integer(i64),
    /// Floating point literal
    Float(f64),
    /// String literal
    String(String),
    /// Boolean literal
    Boolean(bool),
    /// Character literal
    Character(char),
    /// Null/empty list literal
    Nil,
}

/// Compiled program in IR form
#[derive(Debug)]
pub struct IRProgram {
    /// Top-level functions
    pub functions: HashMap<SymbolId, CpsExpr>,
    /// Entry point function
    pub entry_point: SymbolId,
    /// Global constants
    pub constants: HashMap<SymbolId, LiteralValue>,
    /// Type information for optimization
    pub type_info: HashMap<SymbolId, TypeInfo>,
}

/// Type information for variables
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// Primary type
    pub primary_type: SchemeType,
    /// Confidence (0.0-1.0)
    pub confidence: f64,
    /// Whether type is known at compile time
    pub compile_time_known: bool,
}

/// Scheme types for optimization
#[derive(Debug, Clone, PartialEq)]
pub enum SchemeType {
    /// Numeric type (integer or float)
    Number,
    /// String type
    String,
    /// Boolean type
    Boolean,
    /// Character type
    Character,
    /// Pair/cons cell type
    Pair,
    /// Vector type
    Vector,
    /// Procedure/function type
    Procedure,
    /// Any type (unknown at compile time)
    Any,
}

/// Tail call analysis result
#[derive(Debug)]
pub struct TailCallAnalysis {
    /// Functions that can be tail-call optimized
    pub tail_callable: Vec<SymbolId>,
    /// Self-recursive functions (can use loops)
    pub self_recursive: Vec<SymbolId>,
    /// Mutually recursive function groups
    pub mutual_recursion_groups: Vec<Vec<SymbolId>>,
}

/// Continuation analysis for call/cc optimization
#[derive(Debug)]
pub struct ContinuationAnalysis {
    /// Functions that capture continuations
    pub continuation_captors: Vec<SymbolId>,
    /// Continuations that escape their scope
    pub escaping_continuations: Vec<SymbolId>,
    /// Stack-allocated continuation candidates
    pub stack_continuations: Vec<SymbolId>,
}

/// CPS transformation from AST to CPS IR
pub struct CpsTransform {
    /// Fresh variable counter
    var_counter: usize,
    /// Fresh continuation counter  
    cont_counter: usize,
}

impl CpsTransform {
    /// Create a new CPS transformer
    pub fn new() -> Self {
        CpsTransform {
            var_counter: 0,
            cont_counter: 0,
        }
    }

    /// Transforms an expression to CPS form
    pub fn transform_expr(&mut self, expr: &Expr) -> Result<CpsExpr> {
        match expr {
            Expr::Literal(lit) => {
                let var = self.fresh_var();
                let cont = self.fresh_cont();
                let lit_val = self.convert_literal(lit)?;

                Ok(CpsExpr::Primop {
                    op: PrimitiveOp::Box, // Box the literal
                    args: vec![],         // Will be filled with literal load
                    cont,
                    result_var: var,
                })
            }

            Expr::Symbol(name) => {
                let symbol_id = crate::utils::intern_symbol(name);
                Ok(CpsExpr::Var(symbol_id))
            }

            Expr::Application { operator, operands } => {
                self.transform_application(operator, operands)
            }

            Expr::Lambda { formals, body, .. } => self.transform_lambda(formals, body),

            Expr::If {
                test,
                consequent,
                alternative,
            } => self.transform_if(test, consequent, alternative.as_deref()),

            _ => Err(Box::new(Error::compilation_error(format!(
                "Unsupported expression in CPS transform: {:?}",
                expr
            )))),
        }
    }

    /// Transforms function application to CPS
    fn transform_application(
        &mut self,
        operator: &crate::diagnostics::Spanned<Expr>,
        operands: &[crate::diagnostics::Spanned<Expr>],
    ) -> Result<CpsExpr> {
        // Transform operator and operands
        let func_expr = self.transform_expr(&operator.inner)?;
        let mut arg_exprs = Vec::new();

        for operand in operands {
            arg_exprs.push(self.transform_expr(&operand.inner)?);
        }

        // Generate continuation
        let cont = self.fresh_cont();

        // For now, assume all are variables (needs proper let-binding)
        let func_var = self.fresh_var();
        let mut arg_vars = Vec::new();

        for _ in operands {
            arg_vars.push(self.fresh_var());
        }

        Ok(CpsExpr::App {
            func: func_var,
            args: arg_vars,
            cont,
        })
    }

    /// Transforms lambda to CPS
    fn transform_lambda(
        &mut self,
        formals: &Formals,
        body: &[crate::diagnostics::Spanned<Expr>],
    ) -> Result<CpsExpr> {
        let params = self.extract_parameters(formals)?;
        let cont_param = self.fresh_cont();

        // Transform body (sequence of expressions)
        let mut body_expr = None;
        for expr in body {
            body_expr = Some(self.transform_expr(&expr.inner)?);
        }

        let body_expr = body_expr.unwrap_or_else(|| {
            CpsExpr::Halt(self.fresh_var()) // Empty body
        });

        Ok(CpsExpr::Lambda {
            params,
            cont_param,
            body: Box::new(body_expr),
            closure_vars: Vec::new(), // TODO: Free variable analysis
        })
    }

    /// Transforms if expression to CPS
    fn transform_if(
        &mut self,
        test: &crate::diagnostics::Spanned<Expr>,
        consequent: &crate::diagnostics::Spanned<Expr>,
        alternative: Option<&crate::diagnostics::Spanned<Expr>>,
    ) -> Result<CpsExpr> {
        let test_expr = self.transform_expr(&test.inner)?;
        let then_expr = self.transform_expr(&consequent.inner)?;
        let else_expr = if let Some(alt) = alternative {
            self.transform_expr(&alt.inner)?
        } else {
            CpsExpr::Halt(self.fresh_var())
        };

        let test_var = self.fresh_var();
        let then_cont = self.fresh_cont();
        let else_cont = self.fresh_cont();

        Ok(CpsExpr::If {
            test: test_var,
            then_cont,
            else_cont,
        })
    }

    /// Extracts parameter list from formals
    fn extract_parameters(&self, formals: &Formals) -> Result<Vec<SymbolId>> {
        match formals {
            Formals::Fixed(params) => Ok(params
                .iter()
                .map(|p| crate::utils::intern_symbol(p))
                .collect()),
            Formals::Variable(param) => Ok(vec![crate::utils::intern_symbol(param)]),
            _ => Err(Box::new(Error::compilation_error(
                "Complex formal parameters not yet supported in CPS transform".to_string(),
            ))),
        }
    }

    /// Converts AST literal to IR literal
    fn convert_literal(&self, lit: &crate::ast::Literal) -> Result<LiteralValue> {
        use crate::ast::Literal;

        match lit {
            Literal::ExactInteger(i) | Literal::Integer(i) => Ok(LiteralValue::Integer(*i)),
            Literal::InexactReal(f) | Literal::Number(f) => Ok(LiteralValue::Float(*f)),
            Literal::String(s) => Ok(LiteralValue::String((**s).clone())),
            Literal::Boolean(b) => Ok(LiteralValue::Boolean(*b)),
            Literal::Character(c) => Ok(LiteralValue::Character(*c)),
            Literal::Nil => Ok(LiteralValue::Nil),
            _ => Err(Box::new(Error::compilation_error(format!(
                "Unsupported literal in CPS transform: {:?}",
                lit
            )))),
        }
    }

    /// Generates fresh variable name
    fn fresh_var(&mut self) -> SymbolId {
        let var_name = format!("var_{}", self.var_counter);
        self.var_counter += 1;
        crate::utils::intern_symbol(&var_name)
    }

    /// Generates fresh continuation name
    fn fresh_cont(&mut self) -> SymbolId {
        let cont_name = format!("cont_{}", self.cont_counter);
        self.cont_counter += 1;
        crate::utils::intern_symbol(&cont_name)
    }
}

/// Tail call optimization analyzer
pub struct TailCallAnalyzer {
    /// Current function being analyzed
    current_function: Option<SymbolId>,
    /// Call graph
    call_graph: HashMap<SymbolId, Vec<SymbolId>>,
}

impl TailCallAnalyzer {
    /// Create a new tail call analyzer
    pub fn new() -> Self {
        TailCallAnalyzer {
            current_function: None,
            call_graph: HashMap::new(),
        }
    }

    /// Analyzes a CPS program for tail call optimization opportunities
    pub fn analyze_program(&mut self, program: &IRProgram) -> TailCallAnalysis {
        let mut tail_callable = Vec::new();
        let mut self_recursive = Vec::new();

        for (func_id, func_expr) in &program.functions {
            self.current_function = Some(*func_id);

            if self.is_tail_recursive(func_expr) {
                self_recursive.push(*func_id);
                tail_callable.push(*func_id);
            } else if self.is_tail_callable(func_expr) {
                tail_callable.push(*func_id);
            }
        }

        let mutual_recursion_groups = self.find_mutual_recursion_groups();

        TailCallAnalysis {
            tail_callable,
            self_recursive,
            mutual_recursion_groups,
        }
    }

    /// Checks if expression contains only tail calls
    fn is_tail_callable(&self, expr: &CpsExpr) -> bool {
        match expr {
            CpsExpr::App { .. } => true, // All CPS calls are tail calls
            CpsExpr::If { .. } => true,  // Control flow preserves tail position
            CpsExpr::Halt(_) => true,
            _ => false,
        }
    }

    /// Checks if function is self-recursive (can be optimized to loop)
    fn is_tail_recursive(&self, expr: &CpsExpr) -> bool {
        if let Some(current_func) = self.current_function {
            self.calls_self(expr, current_func)
        } else {
            false
        }
    }

    /// Checks if expression calls the given function
    #[allow(clippy::only_used_in_recursion)]
    fn calls_self(&self, expr: &CpsExpr, target: SymbolId) -> bool {
        match expr {
            CpsExpr::App { func, .. } => *func == target,
            CpsExpr::If { .. } => {
                // Would need to check both branches
                false
            }
            CpsExpr::Fix { body, .. } => self.calls_self(body, target),
            _ => false,
        }
    }

    /// Finds mutually recursive function groups
    fn find_mutual_recursion_groups(&self) -> Vec<Vec<SymbolId>> {
        // Implement strongly connected components algorithm
        // For now, return empty groups
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_cps_transform_literal() {
        let mut transform = CpsTransform::new();
        let expr = Expr::Literal(Literal::ExactInteger(42));

        let result = transform.transform_expr(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fresh_var_generation() {
        let mut transform = CpsTransform::new();
        let var1 = transform.fresh_var();
        let var2 = transform.fresh_var();

        assert_ne!(var1, var2);
    }

    #[test]
    fn test_tail_call_analysis() {
        let mut analyzer = TailCallAnalyzer::new();
        let program = IRProgram {
            functions: HashMap::new(),
            entry_point: crate::utils::intern_symbol("main"),
            constants: HashMap::new(),
            type_info: HashMap::new(),
        };

        let analysis = analyzer.analyze_program(&program);
        assert!(analysis.tail_callable.is_empty());
    }
}

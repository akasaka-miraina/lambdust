//! Combinatory logic system for lambda abstraction handling
//!
//! This module implements SKI combinators and related systems for
//! representing and reducing lambda expressions in a mathematically
//! rigorous way that preserves R7RS semantics.

use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use std::collections::HashSet;

/// Combinator expressions representing lambda calculus equivalents
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CombinatorExpr {
    /// S combinator: S x y z = x z (y z)
    S,

    /// K combinator: K x y = x
    K,

    /// I combinator: I x = x
    I,

    /// B combinator: B x y z = x (y z) - Composition
    B,

    /// C combinator: C x y z = x z y - Flip
    C,

    /// W combinator: W x y = x y y - Duplication
    W,

    /// Application of two combinator expressions
    App(Box<CombinatorExpr>, Box<CombinatorExpr>),

    /// Atomic expression (variable, literal, etc.)
    Atomic(Expr),
}

/// Statistics for combinator reductions
#[derive(Debug, Default, Clone)]
pub struct CombinatorStats {
    /// Number of S reductions performed
    pub s_reductions: usize,

    /// Number of K reductions performed
    pub k_reductions: usize,

    /// Number of I reductions performed
    pub i_reductions: usize,

    /// Number of extended combinator reductions (B, C, W)
    pub extended_reductions: usize,

    /// Total reduction steps performed
    pub total_steps: usize,

    /// Number of lambda-to-combinator translations
    pub translations_to_combinators: usize,

    /// Number of combinator-to-lambda translations
    pub translations_to_lambda: usize,
}

impl CombinatorExpr {
    /// Perform a single reduction step if possible
    #[must_use] pub fn reduce_step(&self) -> Option<CombinatorExpr> {
        match self {
            // S x y z → x z (y z)
            CombinatorExpr::App(f, arg) => {
                if let CombinatorExpr::App(f2, arg2) = f.as_ref() {
                    if let CombinatorExpr::App(f3, arg3) = f2.as_ref() {
                        if let (CombinatorExpr::S, x) = (f3.as_ref(), arg3) {
                            let (y, z) = (arg2, arg);
                            // S x y z → x z (y z)
                            return Some(CombinatorExpr::App(
                                Box::new(CombinatorExpr::App(x.clone(), z.clone())),
                                Box::new(CombinatorExpr::App(y.clone(), z.clone())),
                            ));
                        }
                    }
                }

                // K x y → x
                if let CombinatorExpr::App(k_f, k_arg) = f.as_ref() {
                    if let (CombinatorExpr::K, x) = (k_f.as_ref(), k_arg) {
                        // K x y → x
                        return Some(*x.clone());
                    }
                }

                // I x → x
                if let (CombinatorExpr::I, x) = (f.as_ref(), arg) {
                    return Some(*x.clone());
                }

                // B x y z → x (y z)
                if let CombinatorExpr::App(b_f2, b_arg2) = f.as_ref() {
                    if let CombinatorExpr::App(b_f3, b_arg3) = b_f2.as_ref() {
                        if let (CombinatorExpr::B, x) = (b_f3.as_ref(), b_arg3) {
                            let (y, z) = (b_arg2, arg);
                            // B x y z → x (y z)
                            return Some(CombinatorExpr::App(
                                x.clone(),
                                Box::new(CombinatorExpr::App(y.clone(), z.clone())),
                            ));
                        }
                    }
                }

                // C x y z → x z y
                if let CombinatorExpr::App(c_f2, c_arg2) = f.as_ref() {
                    if let CombinatorExpr::App(c_f3, c_arg3) = c_f2.as_ref() {
                        if let (CombinatorExpr::C, x) = (c_f3.as_ref(), c_arg3) {
                            let (y, z) = (c_arg2, arg);
                            // C x y z → x z y
                            return Some(CombinatorExpr::App(
                                Box::new(CombinatorExpr::App(x.clone(), z.clone())),
                                y.clone(),
                            ));
                        }
                    }
                }

                // W x y → x y y
                if let CombinatorExpr::App(w_f, w_arg) = f.as_ref() {
                    if let (CombinatorExpr::W, x) = (w_f.as_ref(), w_arg) {
                        let y = arg;
                        // W x y → x y y
                        return Some(CombinatorExpr::App(
                            Box::new(CombinatorExpr::App(x.clone(), y.clone())),
                            y.clone(),
                        ));
                    }
                }

                // Recurse into subexpressions
                if let Some(left_reduced) = f.reduce_step() {
                    Some(CombinatorExpr::App(Box::new(left_reduced), arg.clone()))
                } else { arg.reduce_step().map(|right_reduced| CombinatorExpr::App(f.clone(), Box::new(right_reduced))) }
            }

            // Base combinators cannot be reduced further
            CombinatorExpr::S
            | CombinatorExpr::K
            | CombinatorExpr::I
            | CombinatorExpr::B
            | CombinatorExpr::C
            | CombinatorExpr::W
            | CombinatorExpr::Atomic(_) => None,
        }
    }

    /// Reduce to normal form (may not terminate for some expressions)
    pub fn reduce_to_normal_form(&self) -> Result<CombinatorExpr> {
        self.reduce_to_normal_form_with_limit(1000)
    }

    /// Reduce to normal form with step limit to prevent infinite loops
    pub fn reduce_to_normal_form_with_limit(&self, max_steps: usize) -> Result<CombinatorExpr> {
        let mut current = self.clone();
        let mut steps = 0;

        while let Some(reduced) = current.reduce_step() {
            if steps >= max_steps {
                return Err(LambdustError::runtime_error(
                    "Combinator reduction exceeded step limit".to_string(),
                ));
            }
            current = reduced;
            steps += 1;
        }

        Ok(current)
    }

    /// Check if expression is in normal form (cannot be reduced further)
    #[must_use] pub fn is_normal_form(&self) -> bool {
        self.reduce_step().is_none()
    }

    /// Count the number of applications in the expression
    #[must_use] pub fn size(&self) -> usize {
        match self {
            CombinatorExpr::App(left, right) => 1 + left.size() + right.size(),
            _ => 1,
        }
    }

    /// Get all free variables in the combinator expression
    #[must_use] pub fn free_variables(&self) -> HashSet<String> {
        match self {
            CombinatorExpr::S
            | CombinatorExpr::K
            | CombinatorExpr::I
            | CombinatorExpr::B
            | CombinatorExpr::C
            | CombinatorExpr::W => HashSet::new(),

            CombinatorExpr::App(left, right) => {
                let mut vars = left.free_variables();
                vars.extend(right.free_variables());
                vars
            }

            CombinatorExpr::Atomic(expr) => expr.free_variables(),
        }
    }

}

/// Lambda abstraction elimination (bracket abstraction)
pub struct BracketAbstraction;

impl BracketAbstraction {
    /// Convert lambda expression to combinator expression
    /// Implements the bracket abstraction algorithm
    pub fn lambda_to_combinators(expr: &Expr) -> Result<CombinatorExpr> {
        match expr {
            // Lambda abstraction: λx. E → [x] E
            Expr::List(exprs) if exprs.len() >= 3 => {
                if let Expr::Variable(lambda_keyword) = &exprs[0] {
                    if lambda_keyword == "lambda" {
                        // Parse lambda parameters
                        let (params, body) = Self::parse_lambda_expr(&exprs[1..])?;

                        // Apply bracket abstraction for each parameter
                        let mut result = Self::lambda_to_combinators(&body)?;
                        for param in params.into_iter().rev() {
                            result = Self::bracket_abstract(&param, result)?;
                        }

                        return Ok(result);
                    }
                }

                // Regular application
                let func = Self::lambda_to_combinators(&exprs[0])?;
                let mut result = func;

                for arg in &exprs[1..] {
                    let arg_comb = Self::lambda_to_combinators(arg)?;
                    result = CombinatorExpr::App(Box::new(result), Box::new(arg_comb));
                }

                Ok(result)
            }

            // Variable
            Expr::Variable(_) | Expr::Literal(_) => Ok(CombinatorExpr::Atomic(expr.clone())),

            // Other expressions
            _ => Ok(CombinatorExpr::Atomic(expr.clone())),
        }
    }

    /// Bracket abstraction: [x] E
    fn bracket_abstract(var: &str, expr: CombinatorExpr) -> Result<CombinatorExpr> {
        match expr {
            // [x] x = I
            CombinatorExpr::Atomic(Expr::Variable(v)) if v == var => Ok(CombinatorExpr::I),

            // [x] E = K E (if x not free in E)
            expr if !expr.free_variables().contains(var) => Ok(CombinatorExpr::App(
                Box::new(CombinatorExpr::K),
                Box::new(expr),
            )),

            // [x] (E F) = S ([x] E) ([x] F) (if x free in both E and F)
            CombinatorExpr::App(left, right) => {
                let left_abstracted = Self::bracket_abstract(var, *left)?;
                let right_abstracted = Self::bracket_abstract(var, *right)?;

                Ok(CombinatorExpr::App(
                    Box::new(CombinatorExpr::App(
                        Box::new(CombinatorExpr::S),
                        Box::new(left_abstracted),
                    )),
                    Box::new(right_abstracted),
                ))
            }

            // Base cases
            _ => Ok(CombinatorExpr::App(
                Box::new(CombinatorExpr::K),
                Box::new(expr),
            )),
        }
    }

    /// Convert combinator expression back to lambda expression
    pub fn combinators_to_lambda(comb: &CombinatorExpr) -> Result<Expr> {
        match comb {
            CombinatorExpr::S => Ok(Expr::Variable("S".to_string())),
            CombinatorExpr::K => Ok(Expr::Variable("K".to_string())),
            CombinatorExpr::I => Ok(Expr::Variable("I".to_string())),
            CombinatorExpr::B => Ok(Expr::Variable("B".to_string())),
            CombinatorExpr::C => Ok(Expr::Variable("C".to_string())),
            CombinatorExpr::W => Ok(Expr::Variable("W".to_string())),

            CombinatorExpr::App(left, right) => {
                let left_expr = Self::combinators_to_lambda(left)?;
                let right_expr = Self::combinators_to_lambda(right)?;

                Ok(Expr::List(vec![left_expr, right_expr]))
            }

            CombinatorExpr::Atomic(expr) => Ok(expr.clone()),
        }
    }

    /// Parse lambda expression parameters and body
    fn parse_lambda_expr(operands: &[Expr]) -> Result<(Vec<String>, Expr)> {
        if operands.len() < 2 {
            return Err(LambdustError::syntax_error(
                "lambda: requires at least parameter list and body".to_string(),
            ));
        }

        // Parse parameter list
        let params = match &operands[0] {
            Expr::List(param_exprs) => {
                let mut params = Vec::new();
                for param_expr in param_exprs {
                    if let Expr::Variable(name) = param_expr {
                        params.push(name.clone());
                    } else {
                        return Err(LambdustError::syntax_error(
                            "lambda: parameter must be symbol".to_string(),
                        ));
                    }
                }
                params
            }
            Expr::Variable(name) => vec![name.clone()], // Single parameter
            _ => {
                return Err(LambdustError::syntax_error(
                    "lambda: invalid parameter list".to_string(),
                ))
            }
        };

        // Body is remaining expressions (treated as begin if multiple)
        let body = if operands.len() == 2 {
            operands[1].clone()
        } else {
            Expr::List({
                let mut body_exprs = vec![Expr::Variable("begin".to_string())];
                body_exprs.extend(operands[1..].iter().cloned());
                body_exprs
            })
        };

        Ok((params, body))
    }

    /// Reduce combinator expression to normal form
    /// 
    /// This implements the core SKI combinator reduction rules:
    /// - S x y z → x z (y z)
    /// - K x y → x  
    /// - I x → x
    /// - B x y z → x (y z)
    /// - C x y z → x z y
    /// - W x y → x y y
    pub fn reduce_combinators(expr: CombinatorExpr) -> Result<CombinatorExpr> {
        expr.reduce_to_normal_form()
    }

    /// Reduce combinator expression with step limit
    /// 
    /// Prevents infinite reduction loops by limiting the maximum number
    /// of reduction steps. This is essential for practical use as some
    /// combinator expressions may not terminate.
    pub fn reduce_combinators_with_limit(expr: CombinatorExpr, max_steps: usize) -> Result<CombinatorExpr> {
        expr.reduce_to_normal_form_with_limit(max_steps)
    }
}

/// Extension to Expr for free variable analysis
trait FreeVariables {
    fn free_variables(&self) -> HashSet<String>;
}

impl FreeVariables for Expr {
    fn free_variables(&self) -> HashSet<String> {
        match self {
            Expr::Variable(name) => {
                let mut vars = HashSet::new();
                vars.insert(name.clone());
                vars
            }

            Expr::List(exprs) => {
                let mut vars = HashSet::new();
                for expr in exprs {
                    vars.extend(expr.free_variables());
                }
                vars
            }

            Expr::Quote(expr) => expr.free_variables(),
            Expr::Quasiquote(expr) => expr.free_variables(),
            Expr::Unquote(expr) => expr.free_variables(),
            Expr::UnquoteSplicing(expr) => expr.free_variables(),
            Expr::Vector(exprs) => {
                let mut vars = HashSet::new();
                for expr in exprs {
                    vars.extend(expr.free_variables());
                }
                vars
            }
            Expr::DottedList(exprs, tail) => {
                let mut vars = HashSet::new();
                for expr in exprs {
                    vars.extend(expr.free_variables());
                }
                vars.extend(tail.free_variables());
                vars
            }

            // Hygienic variables are treated like regular variables for free variable analysis
            Expr::HygienicVariable(symbol) => {
                let mut vars = HashSet::new();
                vars.insert(symbol.unique_name());
                vars
            }

            // Literals have no free variables
            Expr::Literal(_) => HashSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::lexer::SchemeNumber;

    #[test]
    fn test_basic_combinators() {
        // Test I combinator: I x → x
        let i_app = CombinatorExpr::App(
            Box::new(CombinatorExpr::I),
            Box::new(CombinatorExpr::Atomic(Expr::Variable("x".to_string()))),
        );

        let reduced = i_app.reduce_step().unwrap();
        assert_eq!(
            reduced,
            CombinatorExpr::Atomic(Expr::Variable("x".to_string()))
        );
    }

    #[test]
    fn test_k_combinator() {
        // Test K combinator: K x y → x
        let k_app = CombinatorExpr::App(
            Box::new(CombinatorExpr::App(
                Box::new(CombinatorExpr::K),
                Box::new(CombinatorExpr::Atomic(Expr::Variable("x".to_string()))),
            )),
            Box::new(CombinatorExpr::Atomic(Expr::Variable("y".to_string()))),
        );

        let reduced = k_app.reduce_step().unwrap();
        assert_eq!(
            reduced,
            CombinatorExpr::Atomic(Expr::Variable("x".to_string()))
        );
    }

    #[test]
    fn test_s_combinator() {
        // Test S combinator: S x y z → x z (y z)
        let s_app = CombinatorExpr::App(
            Box::new(CombinatorExpr::App(
                Box::new(CombinatorExpr::App(
                    Box::new(CombinatorExpr::S),
                    Box::new(CombinatorExpr::Atomic(Expr::Variable("x".to_string()))),
                )),
                Box::new(CombinatorExpr::Atomic(Expr::Variable("y".to_string()))),
            )),
            Box::new(CombinatorExpr::Atomic(Expr::Variable("z".to_string()))),
        );

        let reduced = s_app.reduce_step().unwrap();

        // Should be: x z (y z)
        let expected = CombinatorExpr::App(
            Box::new(CombinatorExpr::App(
                Box::new(CombinatorExpr::Atomic(Expr::Variable("x".to_string()))),
                Box::new(CombinatorExpr::Atomic(Expr::Variable("z".to_string()))),
            )),
            Box::new(CombinatorExpr::App(
                Box::new(CombinatorExpr::Atomic(Expr::Variable("y".to_string()))),
                Box::new(CombinatorExpr::Atomic(Expr::Variable("z".to_string()))),
            )),
        );

        assert_eq!(reduced, expected);
    }

    #[test]
    fn test_ski_identity() {
        // Test that S K K = I
        // S K K x → K x (K x) → x
        let ski_expr = CombinatorExpr::App(
            Box::new(CombinatorExpr::App(
                Box::new(CombinatorExpr::App(
                    Box::new(CombinatorExpr::S),
                    Box::new(CombinatorExpr::K),
                )),
                Box::new(CombinatorExpr::K),
            )),
            Box::new(CombinatorExpr::Atomic(Expr::Variable("x".to_string()))),
        );

        let reduced = ski_expr.reduce_to_normal_form().unwrap();
        assert_eq!(
            reduced,
            CombinatorExpr::Atomic(Expr::Variable("x".to_string()))
        );
    }

    #[test]
    fn test_bracket_abstraction_simple() {
        // Test [x] x = I
        let var_expr = CombinatorExpr::Atomic(Expr::Variable("x".to_string()));
        let abstracted = BracketAbstraction::bracket_abstract("x", var_expr).unwrap();
        assert_eq!(abstracted, CombinatorExpr::I);
    }

    #[test]
    fn test_bracket_abstraction_constant() {
        // Test [x] c = K c (where c is constant)
        let const_expr =
            CombinatorExpr::Atomic(Expr::Literal(Literal::Number(SchemeNumber::Integer(42))));
        let abstracted = BracketAbstraction::bracket_abstract("x", const_expr.clone()).unwrap();

        let expected = CombinatorExpr::App(Box::new(CombinatorExpr::K), Box::new(const_expr));

        assert_eq!(abstracted, expected);
    }

    #[test]
    fn test_lambda_to_combinators_identity() {
        // Test (lambda (x) x) → I
        let lambda_expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![Expr::Variable("x".to_string())]),
            Expr::Variable("x".to_string()),
        ]);

        let combinators = BracketAbstraction::lambda_to_combinators(&lambda_expr).unwrap();
        assert_eq!(combinators, CombinatorExpr::I);
    }

    #[test]
    fn test_lambda_to_combinators_constant() {
        // Test (lambda (x) 42) → K 42
        let lambda_expr = Expr::List(vec![
            Expr::Variable("lambda".to_string()),
            Expr::List(vec![Expr::Variable("x".to_string())]),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ]);

        let combinators = BracketAbstraction::lambda_to_combinators(&lambda_expr).unwrap();
        let expected = CombinatorExpr::App(
            Box::new(CombinatorExpr::K),
            Box::new(CombinatorExpr::Atomic(Expr::Literal(Literal::Number(
                SchemeNumber::Integer(42),
            )))),
        );

        assert_eq!(combinators, expected);
    }

    #[test]
    fn test_free_variables() {
        let expr = Expr::List(vec![
            Expr::Variable("f".to_string()),
            Expr::Variable("x".to_string()),
            Expr::Literal(Literal::Number(SchemeNumber::Integer(42))),
        ]);

        let free_vars = expr.free_variables();
        assert!(free_vars.contains("f"));
        assert!(free_vars.contains("x"));
        assert!(!free_vars.contains("42"));
    }

    #[test]
    fn test_combinator_size() {
        let expr = CombinatorExpr::App(
            Box::new(CombinatorExpr::App(
                Box::new(CombinatorExpr::S),
                Box::new(CombinatorExpr::K),
            )),
            Box::new(CombinatorExpr::I),
        );

        assert_eq!(expr.size(), 5); // Two applications (2) plus base combinators (3)
    }

    #[test]
    fn test_normal_form_detection() {
        // I is already in normal form
        assert!(CombinatorExpr::I.is_normal_form());

        // I x is not in normal form
        let i_app = CombinatorExpr::App(
            Box::new(CombinatorExpr::I),
            Box::new(CombinatorExpr::Atomic(Expr::Variable("x".to_string()))),
        );
        assert!(!i_app.is_normal_form());
    }

    #[test]
    fn test_reduce_combinators() {
        // Test I combinator reduction: I x → x
        let i_app = CombinatorExpr::App(
            Box::new(CombinatorExpr::I),
            Box::new(CombinatorExpr::Atomic(Expr::Variable("x".to_string()))),
        );
        
        let reduced = BracketAbstraction::reduce_combinators(i_app).unwrap();
        assert_eq!(
            reduced,
            CombinatorExpr::Atomic(Expr::Variable("x".to_string()))
        );
    }

    #[test]
    fn test_reduce_combinators_k() {
        // Test K combinator reduction: K x y → x
        let k_app = CombinatorExpr::App(
            Box::new(CombinatorExpr::App(
                Box::new(CombinatorExpr::K),
                Box::new(CombinatorExpr::Atomic(Expr::Variable("x".to_string()))),
            )),
            Box::new(CombinatorExpr::Atomic(Expr::Variable("y".to_string()))),
        );
        
        let reduced = BracketAbstraction::reduce_combinators(k_app).unwrap();
        assert_eq!(
            reduced,
            CombinatorExpr::Atomic(Expr::Variable("x".to_string()))
        );
    }

    #[test]
    fn test_reduce_combinators_with_limit() {
        // Test step limit functionality
        let i_app = CombinatorExpr::App(
            Box::new(CombinatorExpr::I),
            Box::new(CombinatorExpr::Atomic(Expr::Variable("x".to_string()))),
        );
        
        // Should succeed with sufficient limit
        let reduced = BracketAbstraction::reduce_combinators_with_limit(i_app.clone(), 10).unwrap();
        assert_eq!(
            reduced,
            CombinatorExpr::Atomic(Expr::Variable("x".to_string()))
        );
        
        // Should also succeed with limit of 1 (only needs 1 step)
        let reduced_limited = BracketAbstraction::reduce_combinators_with_limit(i_app, 1).unwrap();
        assert_eq!(
            reduced_limited,
            CombinatorExpr::Atomic(Expr::Variable("x".to_string()))
        );
    }

    #[test]
    fn test_reduce_combinators_normal_form() {
        // Already in normal form - should return unchanged
        let atom = CombinatorExpr::Atomic(Expr::Variable("x".to_string()));
        let reduced = BracketAbstraction::reduce_combinators(atom.clone()).unwrap();
        assert_eq!(reduced, atom);
        
        // Base combinator - should return unchanged
        let k_comb = CombinatorExpr::K;
        let reduced_k = BracketAbstraction::reduce_combinators(k_comb.clone()).unwrap();
        assert_eq!(reduced_k, k_comb);
    }
}

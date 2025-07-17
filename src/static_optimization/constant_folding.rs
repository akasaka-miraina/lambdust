//! Compile-time constant folding optimization
//!
//! This module implements constant folding for Scheme expressions,
//! evaluating constant expressions at compile time to reduce runtime overhead.

use crate::ast::{Expr, Literal};
use crate::error::{LambdustError, Result};
use crate::lexer::SchemeNumber;
use std::time::{Duration, Instant};

/// Constant folder for compile-time constant evaluation
#[derive(Debug, Clone)]
pub struct ConstantFolder {
    /// Folding statistics
    pub statistics: FoldingStatistics,
}

/// Statistics for constant folding operations
#[derive(Debug, Clone, Default)]
pub struct FoldingStatistics {
    /// Total folding attempts
    pub total_attempts: usize,
    /// Successful foldings
    pub successful_foldings: usize,
    /// Total time spent folding
    pub total_folding_time: Duration,
    /// Number of arithmetic operations folded
    pub arithmetic_operations_folded: usize,
    /// Number of logical operations folded
    pub logical_operations_folded: usize,
    /// Number of string operations folded
    pub string_operations_folded: usize,
}

/// Result of constant folding operation
#[derive(Debug, Clone)]
pub struct ConstantFoldingResult {
    /// The folded expression
    pub folded_expr: Expr,
    /// Whether any folding was applied
    pub optimization_applied: bool,
    /// Folding time
    pub folding_time: Duration,
    /// Number of constants folded
    pub constants_folded: usize,
}

/// Types of constants that can be folded
#[derive(Debug, Clone, PartialEq)]
pub enum FoldableConstant {
    /// Numeric constant
    Number(SchemeNumber),
    /// Boolean constant
    Boolean(bool),
    /// String constant
    String(String),
    /// Character constant
    Character(char),
}

impl ConstantFolder {
    /// Create a new constant folder
    #[must_use] pub fn new() -> Self {
        Self {
            statistics: FoldingStatistics::default(),
        }
    }

    /// Fold constants in an expression
    pub fn fold(&mut self, expr: &Expr) -> Result<ConstantFoldingResult> {
        let start_time = Instant::now();
        self.statistics.total_attempts += 1;

        let (folded_expr, constants_folded) = self.fold_expression(expr)?;
        let optimization_applied = constants_folded > 0;

        if optimization_applied {
            self.statistics.successful_foldings += 1;
        }

        let folding_time = start_time.elapsed();
        self.statistics.total_folding_time += folding_time;

        Ok(ConstantFoldingResult {
            folded_expr,
            optimization_applied,
            folding_time,
            constants_folded,
        })
    }

    /// Fold constants in an expression recursively
    fn fold_expression(&mut self, expr: &Expr) -> Result<(Expr, usize)> {
        match expr {
            Expr::Literal(_) => Ok((expr.clone(), 0)),
            Expr::Variable(_) => Ok((expr.clone(), 0)),
            Expr::HygienicVariable(_) => Ok((expr.clone(), 0)),
            Expr::Quote(inner) => {
                let (folded_inner, count) = self.fold_expression(inner)?;
                Ok((Expr::Quote(Box::new(folded_inner)), count))
            }
            Expr::Quasiquote(inner) => {
                let (folded_inner, count) = self.fold_expression(inner)?;
                Ok((Expr::Quasiquote(Box::new(folded_inner)), count))
            }
            Expr::Unquote(inner) => {
                let (folded_inner, count) = self.fold_expression(inner)?;
                Ok((Expr::Unquote(Box::new(folded_inner)), count))
            }
            Expr::UnquoteSplicing(inner) => {
                let (folded_inner, count) = self.fold_expression(inner)?;
                Ok((Expr::UnquoteSplicing(Box::new(folded_inner)), count))
            }
            Expr::Vector(_exprs) => {
                // Vector elements are not evaluated
                Ok((expr.clone(), 0))
            }
            Expr::List(exprs) => self.fold_list(exprs),
            Expr::DottedList(_, _) => Ok((expr.clone(), 0)), // Don't fold dotted lists
        }
    }

    /// Fold constants in a list expression
    fn fold_list(&mut self, exprs: &[Expr]) -> Result<(Expr, usize)> {
        if exprs.is_empty() {
            return Ok((Expr::List(vec![]), 0));
        }

        // Try to fold arithmetic operations
        if let Some(result) = self.try_fold_arithmetic(exprs)? {
            self.statistics.arithmetic_operations_folded += 1;
            return Ok((result, 1));
        }

        // Try to fold logical operations
        if let Some(result) = self.try_fold_logical(exprs)? {
            self.statistics.logical_operations_folded += 1;
            return Ok((result, 1));
        }

        // Try to fold string operations
        if let Some(result) = self.try_fold_string(exprs)? {
            self.statistics.string_operations_folded += 1;
            return Ok((result, 1));
        }

        // Recursively fold subexpressions
        let mut folded_exprs = Vec::new();
        let mut total_count = 0;

        for expr in exprs {
            let (folded, count) = self.fold_expression(expr)?;
            folded_exprs.push(folded);
            total_count += count;
        }

        Ok((Expr::List(folded_exprs), total_count))
    }

    /// Try to fold arithmetic operations
    fn try_fold_arithmetic(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() < 3 {
            return Ok(None);
        }

        let operator = match &exprs[0] {
            Expr::Variable(op) => op,
            _ => return Ok(None),
        };

        match operator.as_str() {
            "+" => self.fold_addition(&exprs[1..]),
            "-" => self.fold_subtraction(&exprs[1..]),
            "*" => self.fold_multiplication(&exprs[1..]),
            "/" => self.fold_division(&exprs[1..]),
            "mod" | "modulo" => self.fold_modulo(&exprs[1..]),
            "abs" => self.fold_abs(&exprs[1..]),
            "min" => self.fold_min(&exprs[1..]),
            "max" => self.fold_max(&exprs[1..]),
            _ => Ok(None),
        }
    }

    /// Try to fold logical operations
    fn try_fold_logical(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() < 2 {
            return Ok(None);
        }

        let operator = match &exprs[0] {
            Expr::Variable(op) => op,
            _ => return Ok(None),
        };

        match operator.as_str() {
            "and" => self.fold_and(&exprs[1..]),
            "or" => self.fold_or(&exprs[1..]),
            "not" => self.fold_not(&exprs[1..]),
            "=" => self.fold_equals(&exprs[1..]),
            "<" => self.fold_less_than(&exprs[1..]),
            ">" => self.fold_greater_than(&exprs[1..]),
            "<=" => self.fold_less_equal(&exprs[1..]),
            ">=" => self.fold_greater_equal(&exprs[1..]),
            _ => Ok(None),
        }
    }

    /// Try to fold string operations
    fn try_fold_string(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() < 2 {
            return Ok(None);
        }

        let operator = match &exprs[0] {
            Expr::Variable(op) => op,
            _ => return Ok(None),
        };

        match operator.as_str() {
            "string-append" => self.fold_string_append(&exprs[1..]),
            "string-length" => self.fold_string_length(&exprs[1..]),
            _ => Ok(None),
        }
    }

    /// Fold addition
    fn fold_addition(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        let mut result = SchemeNumber::Integer(0);
        
        for expr in exprs {
            if let Expr::Literal(Literal::Number(n)) = expr {
                result = self.add_numbers(&result, n)?;
            } else {
                return Ok(None); // Not all constants
            }
        }

        Ok(Some(Expr::Literal(Literal::Number(result))))
    }

    /// Fold subtraction
    fn fold_subtraction(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.is_empty() {
            return Ok(None);
        }

        let first = match &exprs[0] {
            Expr::Literal(Literal::Number(n)) => n.clone(),
            _ => return Ok(None),
        };

        if exprs.len() == 1 {
            // Unary minus
            return Ok(Some(Expr::Literal(Literal::Number(self.negate_number(&first)?))));
        }

        let mut result = first;
        for expr in &exprs[1..] {
            if let Expr::Literal(Literal::Number(n)) = expr {
                result = self.subtract_numbers(&result, n)?;
            } else {
                return Ok(None);
            }
        }

        Ok(Some(Expr::Literal(Literal::Number(result))))
    }

    /// Fold multiplication
    fn fold_multiplication(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        let mut result = SchemeNumber::Integer(1);
        
        for expr in exprs {
            if let Expr::Literal(Literal::Number(n)) = expr {
                result = self.multiply_numbers(&result, n)?;
            } else {
                return Ok(None);
            }
        }

        Ok(Some(Expr::Literal(Literal::Number(result))))
    }

    /// Fold division
    fn fold_division(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() < 2 {
            return Ok(None);
        }

        let first = match &exprs[0] {
            Expr::Literal(Literal::Number(n)) => n.clone(),
            _ => return Ok(None),
        };

        let mut result = first;
        for expr in &exprs[1..] {
            if let Expr::Literal(Literal::Number(n)) = expr {
                if self.is_zero(n) {
                    return Err(LambdustError::runtime_error("Division by zero".to_string()));
                }
                result = self.divide_numbers(&result, n)?;
            } else {
                return Ok(None);
            }
        }

        Ok(Some(Expr::Literal(Literal::Number(result))))
    }

    /// Fold modulo operation
    fn fold_modulo(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() != 2 {
            return Ok(None);
        }

        let a = match &exprs[0] {
            Expr::Literal(Literal::Number(n)) => n,
            _ => return Ok(None),
        };

        let b = match &exprs[1] {
            Expr::Literal(Literal::Number(n)) => n,
            _ => return Ok(None),
        };

        if self.is_zero(b) {
            return Err(LambdustError::runtime_error("Modulo by zero".to_string()));
        }

        let result = self.modulo_numbers(a, b)?;
        Ok(Some(Expr::Literal(Literal::Number(result))))
    }

    /// Fold absolute value
    fn fold_abs(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() != 1 {
            return Ok(None);
        }

        match &exprs[0] {
            Expr::Literal(Literal::Number(n)) => {
                let result = self.abs_number(n)?;
                Ok(Some(Expr::Literal(Literal::Number(result))))
            }
            _ => Ok(None),
        }
    }

    /// Fold min operation
    fn fold_min(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.is_empty() {
            return Ok(None);
        }

        let mut min_val = match &exprs[0] {
            Expr::Literal(Literal::Number(n)) => n.clone(),
            _ => return Ok(None),
        };

        for expr in &exprs[1..] {
            if let Expr::Literal(Literal::Number(n)) = expr {
                if self.compare_numbers(n, &min_val)? < 0 {
                    min_val = n.clone();
                }
            } else {
                return Ok(None);
            }
        }

        Ok(Some(Expr::Literal(Literal::Number(min_val))))
    }

    /// Fold max operation
    fn fold_max(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.is_empty() {
            return Ok(None);
        }

        let mut max_val = match &exprs[0] {
            Expr::Literal(Literal::Number(n)) => n.clone(),
            _ => return Ok(None),
        };

        for expr in &exprs[1..] {
            if let Expr::Literal(Literal::Number(n)) = expr {
                if self.compare_numbers(n, &max_val)? > 0 {
                    max_val = n.clone();
                }
            } else {
                return Ok(None);
            }
        }

        Ok(Some(Expr::Literal(Literal::Number(max_val))))
    }

    /// Fold logical AND
    fn fold_and(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        for expr in exprs {
            match expr {
                Expr::Literal(Literal::Boolean(false)) => {
                    return Ok(Some(Expr::Literal(Literal::Boolean(false))));
                }
                Expr::Literal(Literal::Boolean(true)) => continue,
                _ => return Ok(None), // Not all constants
            }
        }
        Ok(Some(Expr::Literal(Literal::Boolean(true))))
    }

    /// Fold logical OR
    fn fold_or(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        for expr in exprs {
            match expr {
                Expr::Literal(Literal::Boolean(true)) => {
                    return Ok(Some(Expr::Literal(Literal::Boolean(true))));
                }
                Expr::Literal(Literal::Boolean(false)) => continue,
                _ => return Ok(None), // Not all constants
            }
        }
        Ok(Some(Expr::Literal(Literal::Boolean(false))))
    }

    /// Fold logical NOT
    fn fold_not(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() != 1 {
            return Ok(None);
        }

        match &exprs[0] {
            Expr::Literal(Literal::Boolean(b)) => {
                Ok(Some(Expr::Literal(Literal::Boolean(!b))))
            }
            _ => Ok(None),
        }
    }

    /// Fold equality comparison
    fn fold_equals(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() < 2 {
            return Ok(None);
        }

        let first = &exprs[0];
        for expr in &exprs[1..] {
            if !self.expressions_equal(first, expr)? {
                return Ok(Some(Expr::Literal(Literal::Boolean(false))));
            }
        }
        Ok(Some(Expr::Literal(Literal::Boolean(true))))
    }

    /// Fold less than comparison
    fn fold_less_than(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() != 2 {
            return Ok(None);
        }

        let a = match &exprs[0] {
            Expr::Literal(Literal::Number(n)) => n,
            _ => return Ok(None),
        };

        let b = match &exprs[1] {
            Expr::Literal(Literal::Number(n)) => n,
            _ => return Ok(None),
        };

        let result = self.compare_numbers(a, b)? < 0;
        Ok(Some(Expr::Literal(Literal::Boolean(result))))
    }

    /// Fold greater than comparison
    fn fold_greater_than(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() != 2 {
            return Ok(None);
        }

        let a = match &exprs[0] {
            Expr::Literal(Literal::Number(n)) => n,
            _ => return Ok(None),
        };

        let b = match &exprs[1] {
            Expr::Literal(Literal::Number(n)) => n,
            _ => return Ok(None),
        };

        let result = self.compare_numbers(a, b)? > 0;
        Ok(Some(Expr::Literal(Literal::Boolean(result))))
    }

    /// Fold less than or equal comparison
    fn fold_less_equal(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() != 2 {
            return Ok(None);
        }

        let a = match &exprs[0] {
            Expr::Literal(Literal::Number(n)) => n,
            _ => return Ok(None),
        };

        let b = match &exprs[1] {
            Expr::Literal(Literal::Number(n)) => n,
            _ => return Ok(None),
        };

        let result = self.compare_numbers(a, b)? <= 0;
        Ok(Some(Expr::Literal(Literal::Boolean(result))))
    }

    /// Fold greater than or equal comparison
    fn fold_greater_equal(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() != 2 {
            return Ok(None);
        }

        let a = match &exprs[0] {
            Expr::Literal(Literal::Number(n)) => n,
            _ => return Ok(None),
        };

        let b = match &exprs[1] {
            Expr::Literal(Literal::Number(n)) => n,
            _ => return Ok(None),
        };

        let result = self.compare_numbers(a, b)? >= 0;
        Ok(Some(Expr::Literal(Literal::Boolean(result))))
    }

    /// Fold string concatenation
    fn fold_string_append(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        let mut result = String::new();
        
        for expr in exprs {
            if let Expr::Literal(Literal::String(s)) = expr {
                result.push_str(s);
            } else {
                return Ok(None);
            }
        }

        Ok(Some(Expr::Literal(Literal::String(result))))
    }

    /// Fold string length
    fn fold_string_length(&self, exprs: &[Expr]) -> Result<Option<Expr>> {
        if exprs.len() != 1 {
            return Ok(None);
        }

        match &exprs[0] {
            Expr::Literal(Literal::String(s)) => {
                let length = s.len() as i64;
                Ok(Some(Expr::Literal(Literal::Number(SchemeNumber::Integer(length)))))
            }
            _ => Ok(None),
        }
    }

    // Helper methods for number operations
    fn add_numbers(&self, a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
        match (a, b) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => Ok(SchemeNumber::Integer(x + y)),
            (SchemeNumber::Rational(x_num, x_den), SchemeNumber::Rational(y_num, y_den)) => {
                // Add rational numbers: (a/b) + (c/d) = (ad + bc) / (bd)
                let numerator = x_num * y_den + y_num * x_den;
                let denominator = x_den * y_den;
                Ok(SchemeNumber::Rational(numerator, denominator))
            },
            (SchemeNumber::Real(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(x + y)),
            _ => Ok(SchemeNumber::Real(self.to_real(a) + self.to_real(b))),
        }
    }

    fn subtract_numbers(&self, a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
        match (a, b) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => Ok(SchemeNumber::Integer(x - y)),
            (SchemeNumber::Rational(x_num, x_den), SchemeNumber::Rational(y_num, y_den)) => {
                // Subtract rational numbers: (a/b) - (c/d) = (ad - bc) / (bd)
                let numerator = x_num * y_den - y_num * x_den;
                let denominator = x_den * y_den;
                Ok(SchemeNumber::Rational(numerator, denominator))
            },
            (SchemeNumber::Real(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(x - y)),
            _ => Ok(SchemeNumber::Real(self.to_real(a) - self.to_real(b))),
        }
    }

    fn multiply_numbers(&self, a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
        match (a, b) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => Ok(SchemeNumber::Integer(x * y)),
            (SchemeNumber::Rational(x_num, x_den), SchemeNumber::Rational(y_num, y_den)) => {
                // Multiply rational numbers: (a/b) * (c/d) = (ac) / (bd)
                let numerator = x_num * y_num;
                let denominator = x_den * y_den;
                Ok(SchemeNumber::Rational(numerator, denominator))
            },
            (SchemeNumber::Real(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(x * y)),
            _ => Ok(SchemeNumber::Real(self.to_real(a) * self.to_real(b))),
        }
    }

    fn divide_numbers(&self, a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
        if self.is_zero(b) {
            return Err(LambdustError::runtime_error("Division by zero".to_string()));
        }
        
        match (a, b) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => {
                if x % y == 0 {
                    Ok(SchemeNumber::Integer(x / y))
                } else {
                    Ok(SchemeNumber::Real(*x as f64 / *y as f64))
                }
            }
            (SchemeNumber::Rational(x_num, x_den), SchemeNumber::Rational(y_num, y_den)) => {
                // Divide rational numbers: (a/b) / (c/d) = (ad) / (bc)
                let numerator = x_num * y_den;
                let denominator = x_den * y_num;
                Ok(SchemeNumber::Rational(numerator, denominator))
            },
            (SchemeNumber::Real(x), SchemeNumber::Real(y)) => Ok(SchemeNumber::Real(x / y)),
            _ => Ok(SchemeNumber::Real(self.to_real(a) / self.to_real(b))),
        }
    }

    fn modulo_numbers(&self, a: &SchemeNumber, b: &SchemeNumber) -> Result<SchemeNumber> {
        match (a, b) {
            (SchemeNumber::Integer(x), SchemeNumber::Integer(y)) => Ok(SchemeNumber::Integer(x % y)),
            _ => Err(LambdustError::runtime_error("Modulo only supported for integers".to_string())),
        }
    }

    fn abs_number(&self, n: &SchemeNumber) -> Result<SchemeNumber> {
        match n {
            SchemeNumber::Integer(x) => Ok(SchemeNumber::Integer(x.abs())),
            SchemeNumber::Rational(num, den) => Ok(SchemeNumber::Rational(num.abs(), *den)),
            SchemeNumber::Real(x) => Ok(SchemeNumber::Real(x.abs())),
            _ => Ok(n.clone()),
        }
    }

    fn negate_number(&self, n: &SchemeNumber) -> Result<SchemeNumber> {
        match n {
            SchemeNumber::Integer(x) => Ok(SchemeNumber::Integer(-x)),
            SchemeNumber::Rational(num, den) => Ok(SchemeNumber::Rational(-num, *den)),
            SchemeNumber::Real(x) => Ok(SchemeNumber::Real(-x)),
            _ => Ok(n.clone()),
        }
    }

    fn compare_numbers(&self, a: &SchemeNumber, b: &SchemeNumber) -> Result<i32> {
        let a_real = self.to_real(a);
        let b_real = self.to_real(b);
        
        if a_real < b_real {
            Ok(-1)
        } else if a_real > b_real {
            Ok(1)
        } else {
            Ok(0)
        }
    }

    fn to_real(&self, n: &SchemeNumber) -> f64 {
        match n {
            SchemeNumber::Integer(x) => *x as f64,
            SchemeNumber::Rational(num, den) => *num as f64 / *den as f64,
            SchemeNumber::Real(x) => *x,
            _ => 0.0,
        }
    }

    fn is_zero(&self, n: &SchemeNumber) -> bool {
        match n {
            SchemeNumber::Integer(x) => *x == 0,
            SchemeNumber::Rational(num, _) => *num == 0,
            SchemeNumber::Real(x) => *x == 0.0,
            _ => false,
        }
    }

    fn expressions_equal(&self, a: &Expr, b: &Expr) -> Result<bool> {
        match (a, b) {
            (Expr::Literal(lit_a), Expr::Literal(lit_b)) => Ok(lit_a == lit_b),
            (Expr::Variable(var_a), Expr::Variable(var_b)) => Ok(var_a == var_b),
            _ => Ok(false),
        }
    }

    /// Get folding statistics
    #[must_use] pub fn get_statistics(&self) -> &FoldingStatistics {
        &self.statistics
    }

    /// Reset statistics
    pub fn reset_statistics(&mut self) {
        self.statistics = FoldingStatistics::default();
    }
}

impl Default for ConstantFolder {
    fn default() -> Self {
        Self::new()
    }
}

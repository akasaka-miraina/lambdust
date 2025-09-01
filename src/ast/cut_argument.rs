//! Cut/Cute argument types for SRFI-26 implementation.
//!
//! This module defines the argument types used in cut and cute expressions,
//! including slot placeholders and rest argument placeholders.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents an argument in a cut or cute expression.
///
/// This enum distinguishes between slot placeholders (`<>` and `<...>`)
/// and regular expressions that should be evaluated according to the
/// cut/cute semantics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CutArgument {
    /// A slot placeholder (`<>`) that will be filled by lambda parameters
    Slot,

    /// A rest slot placeholder (`<...>`) that will receive remaining arguments
    RestSlot,

    /// A regular expression argument
    /// - In `cut`: evaluated when the generated lambda is called (lazy)
    /// - In `cute`: evaluated when the cut/cute form is expanded (eager)
    Expression(Box<Spanned<super::Expr>>),
}

impl fmt::Display for CutArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CutArgument::Slot => write!(f, "<>"),
            CutArgument::RestSlot => write!(f, "<...>"),
            CutArgument::Expression(expr) => write!(f, "{}", expr.inner),
        }
    }
}

impl CutArgument {
    /// Returns true if this argument is a slot placeholder.
    pub fn is_slot(&self) -> bool {
        matches!(self, CutArgument::Slot)
    }

    /// Returns true if this argument is a rest slot placeholder.
    pub fn is_rest_slot(&self) -> bool {
        matches!(self, CutArgument::RestSlot)
    }

    /// Returns true if this argument is a regular expression.
    pub fn is_expression(&self) -> bool {
        matches!(self, CutArgument::Expression(_))
    }

    /// Creates a slot placeholder argument.
    pub fn slot() -> Self {
        CutArgument::Slot
    }

    /// Creates a rest slot placeholder argument.
    pub fn rest_slot() -> Self {
        CutArgument::RestSlot
    }

    /// Creates an expression argument.
    pub fn expression(expr: Spanned<super::Expr>) -> Self {
        CutArgument::Expression(Box::new(expr))
    }

    /// Gets the expression if this argument is an Expression variant.
    pub fn as_expression(&self) -> Option<&Spanned<super::Expr>> {
        match self {
            CutArgument::Expression(expr) => Some(expr),
            _ => None,
        }
    }

    /// Unwraps the expression, panicking if not an Expression variant.
    pub fn unwrap_expression(self) -> Spanned<super::Expr> {
        match self {
            CutArgument::Expression(expr) => *expr,
            _ => panic!("CutArgument is not an Expression"),
        }
    }
}

/// Statistics about cut/cute argument analysis.
#[derive(Debug, Clone, Default)]
pub struct CutArgumentStats {
    /// Number of slot placeholders found
    pub slot_count: usize,
    /// Number of rest slot placeholders found (should be 0 or 1)
    pub rest_slot_count: usize,
    /// Number of regular expressions found
    pub expression_count: usize,
    /// Whether a rest slot was found
    pub has_rest_slot: bool,
}

impl CutArgumentStats {
    /// Analyzes a list of cut arguments and returns statistics.
    pub fn analyze(args: &[CutArgument]) -> Self {
        let mut stats = CutArgumentStats::default();

        for arg in args {
            match arg {
                CutArgument::Slot => stats.slot_count += 1,
                CutArgument::RestSlot => {
                    stats.rest_slot_count += 1;
                    stats.has_rest_slot = true;
                }
                CutArgument::Expression(_) => stats.expression_count += 1,
            }
        }

        stats
    }

    /// Returns the total number of arguments.
    pub fn total_arguments(&self) -> usize {
        self.slot_count + self.rest_slot_count + self.expression_count
    }

    /// Validates the argument list according to SRFI-26 rules.
    /// Returns Ok(()) if valid, or an error message if invalid.
    pub fn validate(&self) -> Result<(), String> {
        // Check for multiple rest slots
        if self.rest_slot_count > 1 {
            return Err(
                "Multiple rest slots (<...>) not allowed in cut/cute expression".to_string(),
            );
        }

        // Rest slot should be the last argument (this check would need context of argument order)
        // This validation would be done during parsing when we have the full argument list

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;
    use crate::diagnostics::{Span, Spanned};

    fn dummy_span() -> Span {
        Span::new(0, 0)
    }

    fn dummy_expr() -> Spanned<Expr> {
        Spanned::new(
            Expr::Literal(crate::ast::Literal::integer(42)),
            dummy_span(),
        )
    }

    #[test]
    fn test_cut_argument_creation() {
        let slot = CutArgument::slot();
        let rest_slot = CutArgument::rest_slot();
        let expr = CutArgument::expression(dummy_expr());

        assert!(slot.is_slot());
        assert!(!slot.is_rest_slot());
        assert!(!slot.is_expression());

        assert!(!rest_slot.is_slot());
        assert!(rest_slot.is_rest_slot());
        assert!(!rest_slot.is_expression());

        assert!(!expr.is_slot());
        assert!(!expr.is_rest_slot());
        assert!(expr.is_expression());
    }

    #[test]
    fn test_cut_argument_stats() {
        let args = vec![
            CutArgument::slot(),
            CutArgument::expression(dummy_expr()),
            CutArgument::slot(),
            CutArgument::rest_slot(),
        ];

        let stats = CutArgumentStats::analyze(&args);

        assert_eq!(stats.slot_count, 2);
        assert_eq!(stats.rest_slot_count, 1);
        assert_eq!(stats.expression_count, 1);
        assert_eq!(stats.total_arguments(), 4);
        assert!(stats.has_rest_slot);
        assert!(stats.validate().is_ok());
    }

    #[test]
    fn test_multiple_rest_slots_validation() {
        let args = vec![
            CutArgument::slot(),
            CutArgument::rest_slot(),
            CutArgument::rest_slot(),
        ];

        let stats = CutArgumentStats::analyze(&args);
        assert!(stats.validate().is_err());
        assert!(
            stats
                .validate()
                .unwrap_err()
                .contains("Multiple rest slots")
        );
    }

    #[test]
    fn test_display_formatting() {
        let slot = CutArgument::slot();
        let rest_slot = CutArgument::rest_slot();
        let expr = CutArgument::expression(dummy_expr());

        assert_eq!(format!("{}", slot), "<>");
        assert_eq!(format!("{}", rest_slot), "<...>");
        // Expression formatting depends on the Expr Display impl
        assert!(format!("{}", expr).contains("42"));
    }
}

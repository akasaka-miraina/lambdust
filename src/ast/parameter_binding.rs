//! Parameter binding for parameterize forms.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

use super::Expr;

/// A parameter binding for parameterize forms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterBinding {
    pub parameter: Spanned<Expr>,
    pub value: Spanned<Expr>,
}
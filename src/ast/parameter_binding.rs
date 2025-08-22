//! Parameter binding for parameterize forms.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};

use super::Expr;

/// A parameter binding for parameterize forms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterBinding {
    /// Parameter expression (typically an identifier or parameter object)
    pub parameter: Spanned<Expr>,
    /// Value expression to bind to the parameter
    pub value: Spanned<Expr>,
}

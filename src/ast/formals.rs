//! Formal parameters for lambda expressions.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::Expr;

/// Formal parameters for lambda expressions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Formals {
    /// Fixed number of parameters: (param1 param2 ...)
    Fixed(Vec<String>),
    /// Variable parameters: param (all arguments collected into param)
    Variable(String),
    /// Mixed parameters: (param1 param2 . rest)
    Mixed {
        fixed: Vec<String>,
        rest: String,
    },
    /// Keyword parameters: (param1 param2 #:key1 default1 ...)
    Keyword {
        fixed: Vec<String>,
        rest: Option<String>,
        keywords: Vec<KeywordParam>,
    },
}

/// A keyword parameter with optional default value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeywordParam {
    pub name: String,
    pub default: Option<Spanned<Expr>>,
}

impl fmt::Display for Formals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Formals::Fixed(params) => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{param}")?;
                }
                write!(f, ")")
            }
            Formals::Variable(param) => write!(f, "{param}"),
            Formals::Mixed { fixed, rest } => {
                write!(f, "(")?;
                for (i, param) in fixed.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{param}")?;
                }
                write!(f, " . {rest})")
            }
            Formals::Keyword { fixed, rest, keywords } => {
                write!(f, "(")?;
                for (i, param) in fixed.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{param}")?;
                }
                if let Some(rest) = rest {
                    if !fixed.is_empty() { write!(f, " ")?; }
                    write!(f, ". {rest}")?;
                }
                for kw in keywords {
                    write!(f, " #{} {}", kw.name, kw.name)?;
                    if let Some(default) = &kw.default {
                        write!(f, " {}", default.inner)?;
                    }
                }
                write!(f, ")")
            }
        }
    }
}
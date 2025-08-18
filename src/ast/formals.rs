//! Formal parameters for lambda expressions.

use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::{Expr, TypeExpr};

/// Formal parameters for lambda expressions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Formals {
    /// Fixed number of parameters: (param1 param2 ...)
    Fixed(Vec<String>),
    /// Variable parameters: param (all arguments collected into param)
    Variable(String),
    /// Mixed parameters: (param1 param2 . rest)
    Mixed {
        /// Fixed positional parameters
        fixed: Vec<String>,
        /// Rest parameter name
        rest: String,
    },
    /// Keyword parameters: (param1 param2 #:key1 default1 ...)
    Keyword {
        /// Fixed positional parameters
        fixed: Vec<String>,
        /// Optional rest parameter
        rest: Option<String>,
        /// Keyword parameters with optional defaults
        keywords: Vec<KeywordParam>,
    },
    /// Typed parameters: ((param1 : Type1) (param2 : Type2) ...)
    Typed(Vec<TypedParam>),
    /// Typed variable parameters: (param : Type) where param collects all arguments
    TypedVariable(TypedParam),
    /// Typed mixed parameters: ((param1 : Type1) (param2 : Type2) . (rest : RestType))
    TypedMixed {
        /// Fixed typed parameters
        fixed: Vec<TypedParam>,
        /// Rest typed parameter
        rest: TypedParam,
    },
}

/// A typed parameter with name and type annotation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedParam {
    /// Parameter name
    pub name: String,
    /// Type annotation
    pub type_annotation: Spanned<TypeExpr>,
}

/// A keyword parameter with optional default value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeywordParam {
    /// Keyword parameter name
    pub name: String,
    /// Optional default value expression
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
            Formals::Typed(params) => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "({} : {})", param.name, param.type_annotation.inner)?;
                }
                write!(f, ")")
            }
            Formals::TypedVariable(param) => {
                write!(f, "({} : {})", param.name, param.type_annotation.inner)
            }
            Formals::TypedMixed { fixed, rest } => {
                write!(f, "(")?;
                for (i, param) in fixed.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "({} : {})", param.name, param.type_annotation.inner)?;
                }
                write!(f, " . ({} : {}))", rest.name, rest.type_annotation.inner)
            }
        }
    }
}

impl TypedParam {
    /// Creates a new typed parameter.
    pub fn new(name: impl Into<String>, type_annotation: Spanned<TypeExpr>) -> Self {
        Self {
            name: name.into(),
            type_annotation,
        }
    }
}

impl KeywordParam {
    /// Creates a new keyword parameter without a default value.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: None,
        }
    }
    
    /// Creates a new keyword parameter with a default value.
    pub fn with_default(name: impl Into<String>, default: Spanned<Expr>) -> Self {
        Self {
            name: name.into(),
            default: Some(default),
        }
    }
}
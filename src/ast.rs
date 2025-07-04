//! Abstract Syntax Tree definitions for Scheme

use crate::lexer::SchemeNumber;
use std::fmt;

/// AST node representing a Scheme expression
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal values
    Literal(Literal),
    /// Variable reference
    Variable(String),
    /// Function call or special form
    List(Vec<Expr>),
    /// Quoted expression
    Quote(Box<Expr>),
    /// Quasiquoted expression
    Quasiquote(Box<Expr>),
    /// Unquoted expression (within quasiquote)
    Unquote(Box<Expr>),
    /// Unquote-splicing (within quasiquote)
    UnquoteSplicing(Box<Expr>),
    /// Vector literal
    Vector(Vec<Expr>),
    /// Dotted pair (improper list)
    DottedList(Vec<Expr>, Box<Expr>),
}

/// Literal values in Scheme
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// Boolean values
    Boolean(bool),
    /// Numeric values
    Number(SchemeNumber),
    /// String values
    String(String),
    /// Character values
    Character(char),
    /// The empty list
    Nil,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{lit}"),
            Expr::Variable(name) => write!(f, "{name}"),
            Expr::List(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{expr}")?;
                }
                write!(f, ")")
            }
            Expr::Quote(expr) => write!(f, "'{expr}"),
            Expr::Quasiquote(expr) => write!(f, "`{expr}"),
            Expr::Unquote(expr) => write!(f, ",{expr}"),
            Expr::UnquoteSplicing(expr) => write!(f, ",@{expr}"),
            Expr::Vector(exprs) => {
                write!(f, "#(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{expr}")?;
                }
                write!(f, ")")
            }
            Expr::DottedList(exprs, tail) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{expr}")?;
                }
                write!(f, " . {tail})")
            }
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Boolean(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            Literal::Number(n) => write!(f, "{n}"),
            Literal::String(s) => write!(f, "\"{s}\""),
            Literal::Character(c) => match c {
                ' ' => write!(f, "#\\space"),
                '\n' => write!(f, "#\\newline"),
                '\t' => write!(f, "#\\tab"),
                _ => write!(f, "#\\{c}"),
            },
            Literal::Nil => write!(f, "()"),
        }
    }
}

impl Expr {
    /// Check if this expression is a literal
    pub fn is_literal(&self) -> bool {
        matches!(self, Expr::Literal(_))
    }

    /// Check if this expression is a variable
    pub fn is_variable(&self) -> bool {
        matches!(self, Expr::Variable(_))
    }

    /// Check if this expression is a list
    pub fn is_list(&self) -> bool {
        matches!(self, Expr::List(_))
    }

    /// Check if this expression is an empty list
    pub fn is_empty_list(&self) -> bool {
        matches!(self, Expr::List(exprs) if exprs.is_empty())
            || matches!(self, Expr::Literal(Literal::Nil))
    }

    /// Get the symbol name if this is a variable
    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Expr::Variable(name) => Some(name),
            _ => None,
        }
    }

    /// Get the list elements if this is a list
    pub fn as_list(&self) -> Option<&[Expr]> {
        match self {
            Expr::List(exprs) => Some(exprs),
            _ => None,
        }
    }

    /// Convert to a proper list if this is a dotted list
    pub fn to_proper_list(&self) -> Option<Vec<Expr>> {
        match self {
            Expr::List(exprs) => Some(exprs.clone()),
            Expr::DottedList(exprs, tail) => {
                if matches!(tail.as_ref(), Expr::Literal(Literal::Nil)) {
                    let mut result = exprs.clone();
                    result.push((**tail).clone());
                    Some(result)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if this is a special form (list starting with a known special form symbol)
    pub fn is_special_form(&self) -> bool {
        match self {
            Expr::List(exprs) if !exprs.is_empty() => match &exprs[0] {
                Expr::Variable(name) => matches!(
                    name.as_str(),
                    "define"
                        | "lambda"
                        | "if"
                        | "cond"
                        | "case"
                        | "and"
                        | "or"
                        | "let"
                        | "let*"
                        | "letrec"
                        | "begin"
                        | "do"
                        | "delay"
                        | "set!"
                        | "quote"
                        | "quasiquote"
                        | "unquote"
                        | "unquote-splicing"
                ),
                _ => false,
            },
            _ => false,
        }
    }

    /// Get the operator of a list expression
    pub fn get_operator(&self) -> Option<&str> {
        match self {
            Expr::List(exprs) if !exprs.is_empty() => exprs[0].as_symbol(),
            _ => None,
        }
    }

    /// Get the operands of a list expression
    pub fn get_operands(&self) -> Option<&[Expr]> {
        match self {
            Expr::List(exprs) if !exprs.is_empty() => Some(&exprs[1..]),
            _ => None,
        }
    }
}

impl Literal {
    /// Check if this is a truthy value (everything except #f is truthy)
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Literal::Boolean(false))
    }

    /// Check if this is a number
    pub fn is_number(&self) -> bool {
        matches!(self, Literal::Number(_))
    }

    /// Get the number value if this is a number
    pub fn as_number(&self) -> Option<&SchemeNumber> {
        match self {
            Literal::Number(n) => Some(n),
            _ => None,
        }
    }

    /// Check if this is a string
    pub fn is_string(&self) -> bool {
        matches!(self, Literal::String(_))
    }

    /// Get the string value if this is a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Literal::String(s) => Some(s),
            _ => None,
        }
    }

    /// Check if this is a character
    pub fn is_character(&self) -> bool {
        matches!(self, Literal::Character(_))
    }

    /// Get the character value if this is a character
    pub fn as_character(&self) -> Option<char> {
        match self {
            Literal::Character(c) => Some(*c),
            _ => None,
        }
    }
}

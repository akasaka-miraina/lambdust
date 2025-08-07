//! Abstract Syntax Tree definitions for the Lambdust language.
//!
//! This module defines the AST nodes for all language constructs in Lambdust,
//! including the 10 special forms and derived forms implemented as macros.

#![allow(missing_docs)]

pub use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

pub mod literal;
pub mod visitor;
pub mod program;
pub mod formals;
pub mod binding;
pub mod parameter_binding;
pub mod cond_clause;
pub mod case_clause;
pub mod guard_clause;
pub mod case_lambda_clause;

pub use literal::*;
pub use visitor::*;
pub use program::*;
pub use formals::*;
pub use binding::*;
pub use parameter_binding::*;
pub use cond_clause::*;
pub use case_clause::*;
pub use guard_clause::*;
pub use case_lambda_clause::*;


/// The main expression type for Lambdust.
///
/// This enum represents all possible expressions in the language,
/// including the 10 special forms and literals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    // ============= LITERALS =============
    
    /// Literal values (numbers, strings, booleans, etc.)
    Literal(Literal),

    /// Identifiers and symbols
    Identifier(String),

    /// Symbols (for compatibility - alias for Identifier)
    Symbol(String),

    /// Keywords (#:key)
    Keyword(String),

    /// Generic list expression (for module syntax)
    List(Vec<Spanned<Expr>>),

    // ============= SPECIAL FORMS =============
    
    /// Quote expression: (quote <datum>) or '<datum>
    Quote(Box<Spanned<Expr>>),

    /// Lambda expression: (lambda <formals> <body>)
    Lambda {
        formals: Formals,
        metadata: HashMap<String, Spanned<Expr>>,
        body: Vec<Spanned<Expr>>,
    },

    /// Conditional: (if <test> <consequent> <alternative>)
    If {
        test: Box<Spanned<Expr>>,
        consequent: Box<Spanned<Expr>>,
        alternative: Option<Box<Spanned<Expr>>>,
    },

    /// Definition: (define <identifier> <expression>) or (define (<identifier> <formals>) <body>)
    Define {
        name: String,
        value: Box<Spanned<Expr>>,
        metadata: HashMap<String, Spanned<Expr>>,
    },

    /// Assignment: (set! <identifier> <expression>)
    Set {
        name: String,
        value: Box<Spanned<Expr>>,
    },

    /// Macro definition: (define-syntax <identifier> <transformer>)
    DefineSyntax {
        name: String,
        transformer: Box<Spanned<Expr>>,
    },

    /// Syntax rules: (syntax-rules (literals ...) (pattern template) ...)
    SyntaxRules {
        literals: Vec<String>,
        rules: Vec<(Spanned<Expr>, Spanned<Expr>)>, // (pattern, template) pairs
    },

    /// Continuation capture: (call-with-current-continuation <procedure>)
    CallCC(Box<Spanned<Expr>>),

    /// Primitive operation: (primitive <symbol> <arguments>*)
    Primitive {
        name: String,
        args: Vec<Spanned<Expr>>,
    },

    /// Type annotation: (:: <expression> <type>)
    TypeAnnotation {
        expr: Box<Spanned<Expr>>,
        type_expr: Box<Spanned<Expr>>,
    },

    /// Parameter binding: (parameterize ((<parameter> <value>) ...) <body>)
    Parameterize {
        bindings: Vec<ParameterBinding>,
        body: Vec<Spanned<Expr>>,
    },

    /// Module import: (import <import-spec>+)
    Import {
        import_specs: Vec<Spanned<Expr>>,
    },

    /// Library definition: (define-library <name> <library-declaration>*)
    DefineLibrary {
        name: Vec<String>, // Library name as a list, e.g., (srfi 41) becomes ["srfi", "41"]
        imports: Vec<Spanned<Expr>>, // import declarations
        exports: Vec<Spanned<Expr>>, // export declarations
        body: Vec<Spanned<Expr>>, // includes, begin blocks, and other declarations
    },

    // ============= COMPOUND EXPRESSIONS =============
    
    /// Function application: (<procedure> <arguments>*)
    Application {
        operator: Box<Spanned<Expr>>,
        operands: Vec<Spanned<Expr>>,
    },

    /// Dotted pair (cons cell): (car . cdr)
    Pair {
        car: Box<Spanned<Expr>>,
        cdr: Box<Spanned<Expr>>,
    },

    // ============= DERIVED FORMS (implemented as macros) =============
    
    /// Begin expression: (begin <expressions>+)
    Begin(Vec<Spanned<Expr>>),

    /// Let binding: (let (<bindings>*) <body>)
    Let {
        bindings: Vec<Binding>,
        body: Vec<Spanned<Expr>>,
    },

    /// Let* binding: (let* (<bindings>*) <body>)
    LetStar {
        bindings: Vec<Binding>,
        body: Vec<Spanned<Expr>>,
    },

    /// Letrec binding: (letrec (<bindings>*) <body>)
    LetRec {
        bindings: Vec<Binding>,
        body: Vec<Spanned<Expr>>,
    },

    /// Conditional with multiple clauses: (cond <clauses>+)
    Cond(Vec<CondClause>),

    /// Case expression: (case <expression> <clauses>+)
    Case {
        expr: Box<Spanned<Expr>>,
        clauses: Vec<CaseClause>,
    },

    /// Logical AND: (and <expressions>*)
    And(Vec<Spanned<Expr>>),

    /// Logical OR: (or <expressions>*)
    Or(Vec<Spanned<Expr>>),

    /// When expression: (when <test> <expressions>+)
    When {
        test: Box<Spanned<Expr>>,
        body: Vec<Spanned<Expr>>,
    },

    /// Unless expression: (unless <test> <expressions>+)
    Unless {
        test: Box<Spanned<Expr>>,
        body: Vec<Spanned<Expr>>,
    },

    /// Guard expression: (guard (<variable> <clauses>*) <body>)
    Guard {
        variable: String,
        clauses: Vec<GuardClause>,
        body: Vec<Spanned<Expr>>,
    },

    /// Case-lambda expression: (case-lambda (<formals1> <body1>...) (<formals2> <body2>...) ...)
    CaseLambda {
        clauses: Vec<CaseLambdaClause>,
        metadata: HashMap<String, Spanned<Expr>>,
    },
}


impl Expr {
    /// Returns true if this expression is a literal.
    pub fn is_literal(&self) -> bool {
        matches!(self, Expr::Literal(_))
    }

    /// Returns true if this expression is an identifier.
    pub fn is_identifier(&self) -> bool {
        matches!(self, Expr::Identifier(_) | Expr::Symbol(_))
    }

    /// Returns true if this expression is a special form.
    pub fn is_special_form(&self) -> bool {
        matches!(
            self,
            Expr::Quote(_)
                | Expr::Lambda { .. }
                | Expr::If { .. }
                | Expr::Define { .. }
                | Expr::Set { .. }
                | Expr::DefineSyntax { .. }
                | Expr::CallCC(_)
                | Expr::Primitive { .. }
                | Expr::TypeAnnotation { .. }
                | Expr::Parameterize { .. }
                | Expr::Import { .. }
                | Expr::DefineLibrary { .. }
                | Expr::CaseLambda { .. }
        )
    }

    /// Returns true if this expression is self-evaluating.
    pub fn is_self_evaluating(&self) -> bool {
        matches!(self, Expr::Literal(_) | Expr::Keyword(_))
    }

    /// Gets the identifier name if this is an identifier expression.
    pub fn as_identifier(&self) -> Option<&str> {
        match self {
            Expr::Identifier(name) | Expr::Symbol(name) => Some(name),
            _ => None,
        }
    }

    /// Gets the literal value if this is a literal expression.
    pub fn as_literal(&self) -> Option<&Literal> {
        match self {
            Expr::Literal(lit) => Some(lit),
            _ => None,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{lit}"),
            Expr::Identifier(name) => write!(f, "{name}"),
            Expr::Symbol(name) => write!(f, "{name}"),
            Expr::Keyword(name) => write!(f, "#{name}"),
            Expr::List(elements) => {
                write!(f, "(")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", element.inner)?;
                }
                write!(f, ")")
            }
            Expr::Quote(expr) => write!(f, "'{}", expr.inner),
            Expr::Lambda { formals, body, .. } => {
                write!(f, "(lambda {formals} ")?;
                for (i, expr) in body.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", expr.inner)?;
                }
                write!(f, ")")
            }
            Expr::If { test, consequent, alternative } => {
                write!(f, "(if {} {}", test.inner, consequent.inner)?;
                if let Some(alt) = alternative {
                    write!(f, " {}", alt.inner)?;
                }
                write!(f, ")")
            }
            Expr::Define { name, value, .. } => {
                write!(f, "(define {} {})", name, value.inner)
            }
            Expr::Set { name, value } => {
                write!(f, "(set! {} {})", name, value.inner)
            }
            Expr::Application { operator, operands } => {
                write!(f, "({}", operator.inner)?;
                for operand in operands {
                    write!(f, " {}", operand.inner)?;
                }
                write!(f, ")")
            }
            Expr::Pair { car, cdr } => {
                write!(f, "({} . {})", car.inner, cdr.inner)
            }
            Expr::CaseLambda { clauses, .. } => {
                write!(f, "(case-lambda")?;
                for clause in clauses {
                    write!(f, " ({} ", clause.formals)?;
                    for (i, expr) in clause.body.iter().enumerate() {
                        if i > 0 { write!(f, " ")?; }
                        write!(f, "{}", expr.inner)?;
                    }
                    write!(f, ")")?;
                }
                write!(f, ")")
            }
            Expr::Import { import_specs } => {
                write!(f, "(import")?;
                for spec in import_specs {
                    write!(f, " {}", spec.inner)?;
                }
                write!(f, ")")
            }
            Expr::DefineLibrary { name, imports, exports, body } => {
                write!(f, "(define-library ({}) ", name.join(" "))?;
                for import in imports {
                    write!(f, " {}", import.inner)?;
                }
                for export in exports {
                    write!(f, " {}", export.inner)?;
                }
                for expr in body {
                    write!(f, " {}", expr.inner)?;
                }
                write!(f, ")")
            }
            // Add more display implementations as needed
            _ => write!(f, "<{self:?}>"),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_expr_type_checks() {
        let literal = Expr::Literal(Literal::Number(42.0));
        let identifier = Expr::Identifier("x".to_string());
        let keyword = Expr::Keyword("type".to_string());

        assert!(literal.is_literal());
        assert!(!literal.is_identifier());
        assert!(literal.is_self_evaluating());

        assert!(!identifier.is_literal());
        assert!(identifier.is_identifier());
        assert!(!identifier.is_self_evaluating());

        assert!(!keyword.is_literal());
        assert!(!keyword.is_identifier());
        assert!(keyword.is_self_evaluating());
    }

    #[test]
    fn test_program_creation() {
        let mut program = Program::new();
        assert!(program.is_empty());

        let span = Span::new(0, 1);
        let expr = Spanned::new(Expr::Identifier("x".to_string()), span);
        program.add_expression(expr);
        
        assert!(!program.is_empty());
        assert_eq!(program.expressions.len(), 1);
    }

    #[test]
    fn test_formals_display() {
        let fixed = Formals::Fixed(vec!["x".to_string(), "y".to_string()]);
        assert_eq!(format!("{fixed}"), "(x y)");

        let variable = Formals::Variable("args".to_string());
        assert_eq!(format!("{variable}"), "args");

        let mixed = Formals::Mixed {
            fixed: vec!["x".to_string()],
            rest: "rest".to_string(),
        };
        assert_eq!(format!("{mixed}"), "(x . rest)");
    }
}
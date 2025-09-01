//! Abstract Syntax Tree definitions for the Lambdust language.
//!
//! This module defines the AST nodes for all language constructs in Lambdust,
//! including the 10 special forms and derived forms implemented as macros.

pub use crate::diagnostics::Spanned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

pub mod and_let_clause;
pub mod binding;
pub mod case_clause;
pub mod case_lambda_clause;
pub mod cond_clause;
pub mod cond_expand_clause;
pub mod cut_argument;
pub mod formals;
pub mod guard_clause;
pub mod literal;
pub mod literal_helpers;
pub mod parameter_binding;
pub mod program;
pub mod type_expr;
pub mod visitor;

pub use and_let_clause::*;
pub use binding::*;
pub use case_clause::*;
pub use case_lambda_clause::*;
pub use cond_clause::*;
pub use cond_expand_clause::*;
pub use cut_argument::*;
pub use formals::*;
pub use guard_clause::*;
pub use literal::*;
pub use parameter_binding::*;
pub use program::*;
pub use type_expr::*;
pub use visitor::*;

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
    /// Quote expression: `(quote datum)` or `'datum`
    Quote(Box<Spanned<Expr>>),

    /// Quasiquote expression: `(quasiquote datum)` or `` `datum``
    Quasiquote(Box<Spanned<Expr>>),

    /// Unquote expression: `(unquote datum)` or `,datum`
    Unquote(Box<Spanned<Expr>>),

    /// Unquote-splicing expression: `(unquote-splicing datum)` or `,@datum`
    UnquoteSplicing(Box<Spanned<Expr>>),

    /// External form: `#,(tag arg ...)` (SRFI-10)
    ExternalForm {
        /// The tag (constructor name)
        tag: String,
        /// Arguments to the constructor
        args: Vec<Spanned<Expr>>,
    },

    /// Lambda expression: `(lambda formals body)`
    Lambda {
        /// Formal parameters for the lambda
        formals: Formals,
        /// Optional return type annotation
        return_type: Option<Spanned<TypeExpr>>,
        /// Metadata and annotations (e.g., #:pure, #:type)
        metadata: HashMap<String, Spanned<Expr>>,
        /// Body expressions of the lambda
        body: Vec<Spanned<Expr>>,
    },

    /// Conditional: `(if test consequent alternative)`
    If {
        /// Test condition expression
        test: Box<Spanned<Expr>>,
        /// Expression to evaluate if test is true
        consequent: Box<Spanned<Expr>>,
        /// Optional expression to evaluate if test is false
        alternative: Option<Box<Spanned<Expr>>>,
    },

    /// Definition: `(define identifier expression)` or `(define (identifier formals) body)`
    Define {
        /// Name of the identifier being defined
        name: String,
        /// Value expression being bound to the name
        value: Box<Spanned<Expr>>,
        /// Optional return type annotation
        return_type: Option<Spanned<TypeExpr>>,
        /// Metadata and annotations
        metadata: HashMap<String, Spanned<Expr>>,
    },

    /// Assignment: `(set! identifier expression)`
    Set {
        /// Name of the identifier being assigned
        name: String,
        /// New value expression
        value: Box<Spanned<Expr>>,
    },

    /// Macro definition: `(define-syntax identifier transformer)`
    DefineSyntax {
        /// Name of the macro being defined
        name: String,
        /// Transformer expression
        transformer: Box<Spanned<Expr>>,
    },

    /// Syntax rules: (syntax-rules (literals ...) (pattern template) ...)
    SyntaxRules {
        /// Literal identifiers that match only themselves
        literals: Vec<String>,
        /// Pattern-template rule pairs
        rules: Vec<(Spanned<Expr>, Spanned<Expr>)>,
    },

    /// Continuation capture: (call-with-current-continuation procedure)
    CallCC(Box<Spanned<Expr>>),

    /// Delayed evaluation: (delay expression)
    /// Creates a promise that can be forced later
    Delay {
        /// Expression to evaluate when forced
        expression: Box<Spanned<Expr>>,
    },
    /// SRFI-45 lazy evaluation: (lazy expression)
    /// Creates a lazy promise that supports iterative lazy algorithms
    Lazy {
        /// Expression to evaluate lazily when forced
        expression: Box<Spanned<Expr>>,
    },
    /// SRFI-45 eager evaluation: (eager expression)
    /// Creates an eager value that can be forced immediately
    Eager {
        /// Expression to evaluate eagerly
        expression: Box<Spanned<Expr>>,
    },

    /// Primitive operation: `(primitive symbol arguments*)`
    Primitive {
        /// Name of the primitive operation
        name: String,
        /// Arguments to the primitive
        args: Vec<Spanned<Expr>>,
    },

    /// Type annotation: `(:: expression type)`
    TypeAnnotation {
        /// Expression being annotated
        expr: Box<Spanned<Expr>>,
        /// Type expression
        type_expr: Box<Spanned<Expr>>,
    },

    /// Parameter binding: (parameterize ((parameter value) ...) body)
    Parameterize {
        /// Parameter bindings
        bindings: Vec<ParameterBinding>,
        /// Body expressions
        body: Vec<Spanned<Expr>>,
    },

    /// Conditional expansion: (cond-expand (feature body ...) ... (else body ...))
    /// SRFI-0 implementation for compile-time feature detection
    CondExpand {
        /// List of feature-condition clauses
        clauses: Vec<CondExpandClause>,
        /// Optional else clause
        else_clause: Option<Vec<Spanned<Expr>>>,
    },

    /// Module import: (import import-spec+)
    Import {
        /// Import specifications
        import_specs: Vec<Spanned<Expr>>,
    },

    /// Library definition: (define-library name library-declaration*)
    DefineLibrary {
        /// Library name as a list (e.g., (srfi 41) becomes ["srfi", "41"])
        name: Vec<String>,
        /// Import declarations
        imports: Vec<Spanned<Expr>>,
        /// Export declarations
        exports: Vec<Spanned<Expr>>,
        /// Body including includes, begin blocks, and other declarations
        body: Vec<Spanned<Expr>>,
    },

    // ============= COMPOUND EXPRESSIONS =============
    /// Function application: `(procedure arguments*)`
    Application {
        /// The procedure/operator being called
        operator: Box<Spanned<Expr>>,
        /// Arguments/operands to the procedure
        operands: Vec<Spanned<Expr>>,
    },

    /// Dotted pair (cons cell): (car . cdr)
    Pair {
        /// First element of the pair
        car: Box<Spanned<Expr>>,
        /// Second element of the pair
        cdr: Box<Spanned<Expr>>,
    },

    // ============= DERIVED FORMS (implemented as macros) =============
    /// Begin expression: (begin expressions+)
    Begin(Vec<Spanned<Expr>>),

    /// Let binding: `(let (bindings*) body)`
    Let {
        /// Variable bindings
        bindings: Vec<Binding>,
        /// Body expressions
        body: Vec<Spanned<Expr>>,
    },

    /// Let* binding: `(let* (bindings*) body)`
    LetStar {
        /// Sequential variable bindings
        bindings: Vec<Binding>,
        /// Body expressions
        body: Vec<Spanned<Expr>>,
    },

    /// Letrec binding: `(letrec (bindings*) body)`
    LetRec {
        /// Recursive variable bindings
        bindings: Vec<Binding>,
        /// Body expressions
        body: Vec<Spanned<Expr>>,
    },

    /// SRFI-2 and-let*: `(and-let* (clauses*) body)`
    /// Conditional binding with short-circuit evaluation
    AndLetStar {
        /// and-let* clauses (bindings and tests)
        clauses: Vec<AndLetClause>,
        /// Body expressions to evaluate if all clauses succeed
        body: Vec<Spanned<Expr>>,
    },

    /// Conditional with multiple clauses: `(cond clauses+)`
    Cond(Vec<CondClause>),

    /// Case expression: `(case expression clauses+)`
    Case {
        /// Expression to match against
        expr: Box<Spanned<Expr>>,
        /// Case clauses with patterns and bodies
        clauses: Vec<CaseClause>,
    },

    /// Logical AND: (and expressions*)
    And(Vec<Spanned<Expr>>),

    /// Logical OR: (or expressions*)
    Or(Vec<Spanned<Expr>>),

    /// When expression: (when test expressions+)
    When {
        /// Test condition
        test: Box<Spanned<Expr>>,
        /// Body expressions to evaluate if test is true
        body: Vec<Spanned<Expr>>,
    },

    /// Unless expression: (unless test expressions+)
    Unless {
        /// Test condition
        test: Box<Spanned<Expr>>,
        /// Body expressions to evaluate if test is false
        body: Vec<Spanned<Expr>>,
    },

    /// Guard expression: `(guard (variable clauses*) body)`
    Guard {
        /// Exception variable name
        variable: String,
        /// Exception handling clauses
        clauses: Vec<GuardClause>,
        /// Body expressions
        body: Vec<Spanned<Expr>>,
    },

    /// Case-lambda expression: (case-lambda (formals1 body1...) (formals2 body2...) ...)
    CaseLambda {
        /// Lambda clauses with different arities
        clauses: Vec<CaseLambdaClause>,
        /// Optional return type annotation
        return_type: Option<Spanned<TypeExpr>>,
        /// Metadata and annotations
        metadata: HashMap<String, Spanned<Expr>>,
    },

    // ============= CONTRACT SYSTEM =============
    /// Contract definition: (define/contract (name formals) contract body ...)
    DefineContract {
        /// Name of the function being defined with contract
        name: String,
        /// Optional formal parameters
        formals: Option<Formals>,
        /// Contract specification
        contract: Box<Spanned<crate::contracts::ast::ContractExpr>>,
        /// Optional return type contract
        return_type: Option<Spanned<crate::contracts::ast::ContractExpr>>,
        /// Function body
        body: Vec<Spanned<Expr>>,
    },

    /// Contract expression: contract literals and combinators
    Contract(Box<Spanned<crate::contracts::ast::ContractExpr>>),

    /// Contract application: (contract expr)
    ContractApplication {
        /// Contract to apply
        contract: Box<Spanned<crate::contracts::ast::ContractExpr>>,
        /// Expression to which the contract applies
        expr: Box<Spanned<Expr>>,
    },

    // ============= SRFI-26: NOTATION FOR SPECIALIZING PARAMETERS =============
    /// Cut expression: (cut <slot-or-expr> <slot-or-expr> ...)
    /// Creates a lambda with some arguments specialized (lazy evaluation)
    Cut {
        /// The procedure expression being specialized
        procedure: Box<Spanned<Expr>>,
        /// Arguments where `<>` represents slots and expressions are evaluated lazily
        arguments: Vec<CutArgument>,
    },

    /// Cute expression: (cute <slot-or-expr> <slot-or-expr> ...)  
    /// Like cut but non-slot expressions are evaluated immediately (eager evaluation)
    Cute {
        /// The procedure expression being specialized
        procedure: Box<Spanned<Expr>>,
        /// Arguments where `<>` represents slots and expressions are evaluated eagerly
        arguments: Vec<CutArgument>,
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
                | Expr::Quasiquote(_)
                | Expr::Unquote(_)
                | Expr::UnquoteSplicing(_)
                | Expr::ExternalForm { .. }
                | Expr::Lambda { .. }
                | Expr::If { .. }
                | Expr::Define { .. }
                | Expr::Set { .. }
                | Expr::DefineSyntax { .. }
                | Expr::CallCC(_)
                | Expr::Delay { .. }
                | Expr::Lazy { .. }
                | Expr::Eager { .. }
                | Expr::Primitive { .. }
                | Expr::TypeAnnotation { .. }
                | Expr::Parameterize { .. }
                | Expr::Import { .. }
                | Expr::DefineLibrary { .. }
                | Expr::CaseLambda { .. }
                | Expr::DefineContract { .. }
                | Expr::Contract(_)
                | Expr::ContractApplication { .. }
                | Expr::Cut { .. }
                | Expr::Cute { .. }
                | Expr::AndLetStar { .. }
        )
    }

    /// Returns true if this expression is a contract-related form.
    pub fn is_contract_form(&self) -> bool {
        matches!(
            self,
            Expr::DefineContract { .. } | Expr::Contract(_) | Expr::ContractApplication { .. }
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
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", element.inner)?;
                }
                write!(f, ")")
            }
            Expr::Quote(expr) => write!(f, "'{}", expr.inner),
            Expr::Quasiquote(expr) => write!(f, "`{}", expr.inner),
            Expr::Unquote(expr) => write!(f, ",{}", expr.inner),
            Expr::UnquoteSplicing(expr) => write!(f, ",@{}", expr.inner),
            Expr::ExternalForm { tag, args } => {
                write!(f, "#,({tag}")?;
                for arg in args {
                    write!(f, " {}", arg.inner)?;
                }
                write!(f, ")")
            }
            Expr::Delay { expression } => write!(f, "(delay {})", expression.inner),
            Expr::Lazy { expression } => write!(f, "(lazy {})", expression.inner),
            Expr::Eager { expression } => write!(f, "(eager {})", expression.inner),
            Expr::Lambda {
                formals,
                return_type,
                body,
                ..
            } => {
                write!(f, "(lambda {formals}")?;
                if let Some(ret_type) = return_type {
                    write!(f, " : {}", ret_type.inner)?;
                }
                write!(f, " ")?;
                for (i, expr) in body.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", expr.inner)?;
                }
                write!(f, ")")
            }
            Expr::If {
                test,
                consequent,
                alternative,
            } => {
                write!(f, "(if {} {}", test.inner, consequent.inner)?;
                if let Some(alt) = alternative {
                    write!(f, " {}", alt.inner)?;
                }
                write!(f, ")")
            }
            Expr::Define {
                name,
                value,
                return_type,
                ..
            } => {
                write!(f, "(define {name}")?;
                if let Some(ret_type) = return_type {
                    write!(f, " : {}", ret_type.inner)?;
                }
                write!(f, " {})", value.inner)
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
            Expr::CaseLambda {
                clauses,
                return_type,
                ..
            } => {
                write!(f, "(case-lambda")?;
                if let Some(ret_type) = return_type {
                    write!(f, " : {}", ret_type.inner)?;
                }
                for clause in clauses {
                    write!(f, " ({} ", clause.formals)?;
                    for (i, expr) in clause.body.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ")?;
                        }
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
            Expr::DefineLibrary {
                name,
                imports,
                exports,
                body,
            } => {
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
            Expr::Cut {
                procedure,
                arguments,
            } => {
                write!(f, "(cut {}", procedure.inner)?;
                for arg in arguments {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")
            }
            Expr::Cute {
                procedure,
                arguments,
            } => {
                write!(f, "(cute {}", procedure.inner)?;
                for arg in arguments {
                    write!(f, " {}", arg)?;
                }
                write!(f, ")")
            }
            Expr::AndLetStar { clauses, body } => {
                write!(f, "(and-let* (")?;
                for (i, clause) in clauses.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    match clause {
                        AndLetClause::Binding { variable, expression } => {
                            write!(f, "({} {})", variable, expression.inner)?;
                        }
                        AndLetClause::Test { expression } => {
                            write!(f, "({})", expression.inner)?;
                        }
                    }
                }
                write!(f, ")")?;
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

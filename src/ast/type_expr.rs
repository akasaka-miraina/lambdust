//! Type expression AST nodes for type annotation syntax.
//!
//! This module provides AST nodes for representing type expressions in the
//! concrete syntax of the language. These are distinct from the Type enum
//! in the type system - TypeExpr represents the syntax while Type represents
//! the semantic meaning after type checking.

use crate::diagnostics::{Span, Spanned};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Type expression AST node representing type annotations in syntax.
///
/// These nodes capture the concrete syntax of type expressions as written
/// by the programmer, before semantic analysis and type checking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExpr {
    // ============= BASIC TYPE NAMES =============
    /// Type identifier (e.g., Number, String, Boolean)
    Identifier(String),

    /// Type variable (e.g., 'a, 'alpha)
    Variable(String),

    // ============= COMPOUND TYPES =============
    /// Function type: (A -> B) or (A B -> C)
    Function {
        /// Parameter types
        params: Vec<Spanned<TypeExpr>>,
        /// Return type
        return_type: Box<Spanned<TypeExpr>>,
    },

    /// Pair type: (Pair A B)
    Pair {
        /// First type of the pair
        first: Box<Spanned<TypeExpr>>,
        /// Second type of the pair
        second: Box<Spanned<TypeExpr>>,
    },

    /// List type: (List A)
    List {
        /// Element type of the list
        element_type: Box<Spanned<TypeExpr>>,
    },

    /// Vector type: (Vector A)
    Vector {
        /// Element type of the vector
        element_type: Box<Spanned<TypeExpr>>,
    },

    // ============= POLYMORPHIC TYPES =============
    /// Universal quantification: (forall (a b) Type) or (∀ (a b) Type)
    Forall {
        /// Type variables being quantified
        vars: Vec<String>,
        /// Type expression body
        body: Box<Spanned<TypeExpr>>,
    },

    /// Existential quantification: (exists (a) Type) or (∃ (a) Type)
    Exists {
        /// Type variables being quantified
        vars: Vec<String>,
        /// Type expression body
        body: Box<Spanned<TypeExpr>>,
    },

    // ============= TYPE CONSTRUCTORS =============
    /// Type application: (F A) - applies type constructor F to argument A
    Application {
        /// Type constructor being applied
        constructor: Box<Spanned<TypeExpr>>,
        /// Argument to the type constructor
        argument: Box<Spanned<TypeExpr>>,
    },

    /// Parametric type: (Maybe A), (Either A B)
    Parametric {
        /// Name of the parametric type constructor
        name: String,
        /// Type arguments
        args: Vec<Spanned<TypeExpr>>,
    },

    // ============= TYPE CONSTRAINTS =============
    /// Constrained type: (Show a => a -> String)
    Constrained {
        /// Type class constraints
        constraints: Vec<TypeConstraint>,
        /// Constrained type expression
        type_expr: Box<Spanned<TypeExpr>>,
    },

    // ============= ADVANCED TYPES =============
    /// Record type: {x : Int, y : String}
    Record {
        /// Record field types
        fields: Vec<(String, Spanned<TypeExpr>)>,
        /// Row variable for row polymorphism: {x : Int | r}
        rest: Option<String>,
    },

    /// Variant type: (| Some A | None)
    Variant {
        /// Variant constructor cases
        cases: Vec<VariantCase>,
    },

    /// Recursive type: (mu t. List t -> t)
    Recursive {
        /// Recursive type variable
        var: String,
        /// Type expression body
        body: Box<Spanned<TypeExpr>>,
    },

    // ============= EFFECT TYPES =============
    /// Effectful type: (a ~> IO b)
    Effectful {
        /// Input type
        input: Box<Spanned<TypeExpr>>,
        /// Effect annotations
        effects: Vec<String>,
        /// Output type
        output: Box<Spanned<TypeExpr>>,
    },

    // ============= GRADUAL TYPING =============
    /// Dynamic type: *
    Dynamic,

    /// Unknown type: ?
    Unknown,

    // ============= SYNTAX HELPERS =============
    /// Parenthesized type expression
    Parenthesized(Box<Spanned<TypeExpr>>),

    /// Type annotation with kind: (A : *)
    Kinded {
        /// Type expression being kinded
        type_expr: Box<Spanned<TypeExpr>>,
        /// Kind annotation
        kind: Box<Spanned<TypeExpr>>,
    },
}

/// Type constraint in constrained types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeConstraint {
    /// Type class name
    pub class: String,
    /// Type being constrained
    pub type_expr: Spanned<TypeExpr>,
}

/// Variant case in variant types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantCase {
    /// Constructor name
    pub constructor: String,
    /// Optional payload type
    pub payload: Option<Spanned<TypeExpr>>,
}

impl TypeExpr {
    /// Creates a simple identifier type expression.
    pub fn identifier(name: impl Into<String>) -> Self {
        TypeExpr::Identifier(name.into())
    }

    /// Creates a type variable expression.
    pub fn variable(name: impl Into<String>) -> Self {
        TypeExpr::Variable(name.into())
    }

    /// Creates a function type expression.
    pub fn function(params: Vec<Spanned<TypeExpr>>, return_type: Spanned<TypeExpr>) -> Self {
        TypeExpr::Function {
            params,
            return_type: Box::new(return_type),
        }
    }

    /// Creates a list type expression.
    pub fn list(element_type: Spanned<TypeExpr>) -> Self {
        TypeExpr::List {
            element_type: Box::new(element_type),
        }
    }

    /// Creates a pair type expression.
    pub fn pair(first: Spanned<TypeExpr>, second: Spanned<TypeExpr>) -> Self {
        TypeExpr::Pair {
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Creates a vector type expression.
    pub fn vector(element_type: Spanned<TypeExpr>) -> Self {
        TypeExpr::Vector {
            element_type: Box::new(element_type),
        }
    }

    /// Creates a parametric type expression.
    pub fn parametric(name: impl Into<String>, args: Vec<Spanned<TypeExpr>>) -> Self {
        TypeExpr::Parametric {
            name: name.into(),
            args,
        }
    }

    /// Creates a universal quantification.
    pub fn forall(vars: Vec<String>, body: Spanned<TypeExpr>) -> Self {
        TypeExpr::Forall {
            vars,
            body: Box::new(body),
        }
    }

    /// Creates a type application.
    pub fn application(constructor: Spanned<TypeExpr>, argument: Spanned<TypeExpr>) -> Self {
        TypeExpr::Application {
            constructor: Box::new(constructor),
            argument: Box::new(argument),
        }
    }

    /// Creates a constrained type.
    pub fn constrained(constraints: Vec<TypeConstraint>, type_expr: Spanned<TypeExpr>) -> Self {
        TypeExpr::Constrained {
            constraints,
            type_expr: Box::new(type_expr),
        }
    }

    /// Creates a record type.
    pub fn record(fields: Vec<(String, Spanned<TypeExpr>)>, rest: Option<String>) -> Self {
        TypeExpr::Record { fields, rest }
    }

    /// Creates a variant type.
    pub fn variant(cases: Vec<VariantCase>) -> Self {
        TypeExpr::Variant { cases }
    }

    /// Creates an effectful type.
    pub fn effectful(
        input: Spanned<TypeExpr>,
        effects: Vec<String>,
        output: Spanned<TypeExpr>,
    ) -> Self {
        TypeExpr::Effectful {
            input: Box::new(input),
            effects,
            output: Box::new(output),
        }
    }

    /// Creates a parenthesized type expression.
    pub fn parenthesized(inner: Spanned<TypeExpr>) -> Self {
        TypeExpr::Parenthesized(Box::new(inner))
    }

    /// Returns true if this type expression is a simple identifier.
    pub fn is_identifier(&self) -> bool {
        matches!(self, TypeExpr::Identifier(_))
    }

    /// Returns true if this type expression is a type variable.
    pub fn is_variable(&self) -> bool {
        matches!(self, TypeExpr::Variable(_))
    }

    /// Returns true if this type expression is a function type.
    pub fn is_function(&self) -> bool {
        matches!(self, TypeExpr::Function { .. })
    }

    /// Returns true if this type expression involves polymorphism.
    pub fn is_polymorphic(&self) -> bool {
        matches!(self, TypeExpr::Forall { .. } | TypeExpr::Exists { .. })
    }

    /// Gets the identifier name if this is an identifier.
    pub fn as_identifier(&self) -> Option<&str> {
        match self {
            TypeExpr::Identifier(name) => Some(name),
            _ => None,
        }
    }

    /// Gets the variable name if this is a type variable.
    pub fn as_variable(&self) -> Option<&str> {
        match self {
            TypeExpr::Variable(name) => Some(name),
            _ => None,
        }
    }
}

impl TypeConstraint {
    /// Creates a new type constraint.
    pub fn new(class: impl Into<String>, type_expr: Spanned<TypeExpr>) -> Self {
        Self {
            class: class.into(),
            type_expr,
        }
    }
}

impl VariantCase {
    /// Creates a new variant case with a payload.
    pub fn with_payload(constructor: impl Into<String>, payload: Spanned<TypeExpr>) -> Self {
        Self {
            constructor: constructor.into(),
            payload: Some(payload),
        }
    }

    /// Creates a new variant case without a payload.
    pub fn without_payload(constructor: impl Into<String>) -> Self {
        Self {
            constructor: constructor.into(),
            payload: None,
        }
    }
}

impl fmt::Display for TypeExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeExpr::Identifier(name) => write!(f, "{name}"),
            TypeExpr::Variable(name) => write!(f, "'{name}"),
            TypeExpr::Function {
                params,
                return_type,
            } => {
                if params.is_empty() {
                    write!(f, "(() -> {})", return_type.inner)
                } else if params.len() == 1 {
                    write!(f, "({} -> {})", params[0].inner, return_type.inner)
                } else {
                    write!(f, "(")?;
                    for (i, param) in params.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ")?;
                        }
                        write!(f, "{}", param.inner)?;
                    }
                    write!(f, " -> {})", return_type.inner)
                }
            }
            TypeExpr::Pair { first, second } => {
                write!(f, "(Pair {} {})", first.inner, second.inner)
            }
            TypeExpr::List { element_type } => {
                write!(f, "(List {})", element_type.inner)
            }
            TypeExpr::Vector { element_type } => {
                write!(f, "(Vector {})", element_type.inner)
            }
            TypeExpr::Forall { vars, body } => {
                write!(f, "(∀ (")?;
                for (i, var) in vars.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{var}")?;
                }
                write!(f, ") {})", body.inner)
            }
            TypeExpr::Exists { vars, body } => {
                write!(f, "(∃ (")?;
                for (i, var) in vars.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{var}")?;
                }
                write!(f, ") {})", body.inner)
            }
            TypeExpr::Application {
                constructor,
                argument,
            } => {
                write!(f, "({} {})", constructor.inner, argument.inner)
            }
            TypeExpr::Parametric { name, args } => {
                write!(f, "({name}")?;
                for arg in args {
                    write!(f, " {}", arg.inner)?;
                }
                write!(f, ")")
            }
            TypeExpr::Constrained {
                constraints,
                type_expr,
            } => {
                write!(f, "(")?;
                for (i, constraint) in constraints.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} {}", constraint.class, constraint.type_expr.inner)?;
                }
                write!(f, " => {})", type_expr.inner)
            }
            TypeExpr::Record { fields, rest } => {
                write!(f, "{{")?;
                for (i, (name, type_expr)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{name} : {}", type_expr.inner)?;
                }
                if let Some(rest_var) = rest {
                    if !fields.is_empty() {
                        write!(f, " | ")?;
                    }
                    write!(f, "{rest_var}")?;
                }
                write!(f, "}}")
            }
            TypeExpr::Variant { cases } => {
                write!(f, "(|")?;
                for (i, case) in cases.iter().enumerate() {
                    if i > 0 {
                        write!(f, " |")?;
                    }
                    write!(f, " {}", case.constructor)?;
                    if let Some(payload) = &case.payload {
                        write!(f, " {}", payload.inner)?;
                    }
                }
                write!(f, ")")
            }
            TypeExpr::Recursive { var, body } => {
                write!(f, "(μ {var}. {})", body.inner)
            }
            TypeExpr::Effectful {
                input,
                effects,
                output,
            } => {
                write!(f, "({}", input.inner)?;
                if !effects.is_empty() {
                    write!(f, " ~>")?;
                    for effect in effects {
                        write!(f, " {effect}")?;
                    }
                } else {
                    write!(f, " ~>")?;
                }
                write!(f, " {})", output.inner)
            }
            TypeExpr::Dynamic => write!(f, "*"),
            TypeExpr::Unknown => write!(f, "?"),
            TypeExpr::Parenthesized(inner) => write!(f, "({})", inner.inner),
            TypeExpr::Kinded { type_expr, kind } => {
                write!(f, "({} : {})", type_expr.inner, kind.inner)
            }
        }
    }
}

impl fmt::Display for TypeConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.class, self.type_expr.inner)
    }
}

impl fmt::Display for VariantCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.constructor)?;
        if let Some(payload) = &self.payload {
            write!(f, " {}", payload.inner)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    fn dummy_span() -> Span {
        Span::new(0, 1)
    }

    fn spanned_type(expr: TypeExpr) -> Spanned<TypeExpr> {
        Spanned::new(expr, dummy_span())
    }

    #[test]
    fn test_basic_type_expressions() {
        let int_type = TypeExpr::identifier("Integer");
        assert!(int_type.is_identifier());
        assert_eq!(int_type.as_identifier(), Some("Integer"));

        let var_type = TypeExpr::variable("a");
        assert!(var_type.is_variable());
        assert_eq!(var_type.as_variable(), Some("a"));
    }

    #[test]
    fn test_function_type_display() {
        let int_type = spanned_type(TypeExpr::identifier("Integer"));
        let string_type = spanned_type(TypeExpr::identifier("String"));

        let func_type = TypeExpr::function(vec![int_type], string_type);
        assert!(func_type.is_function());
        assert_eq!(format!("{func_type}"), "(Integer -> String)");
    }

    #[test]
    fn test_parametric_type_display() {
        let int_type = spanned_type(TypeExpr::identifier("Integer"));
        let list_type = TypeExpr::parametric("List", vec![int_type]);
        assert_eq!(format!("{list_type}"), "(List Integer)");
    }

    #[test]
    fn test_polymorphic_type_display() {
        let var_a = spanned_type(TypeExpr::variable("a"));
        let forall_type = TypeExpr::forall(vec!["a".to_string()], var_a);
        assert!(forall_type.is_polymorphic());
        assert_eq!(format!("{forall_type}"), "(∀ (a) 'a)");
    }

    #[test]
    fn test_record_type_display() {
        let int_type = spanned_type(TypeExpr::identifier("Integer"));
        let string_type = spanned_type(TypeExpr::identifier("String"));

        let record_type = TypeExpr::record(
            vec![("x".to_string(), int_type), ("y".to_string(), string_type)],
            None,
        );
        assert_eq!(format!("{record_type}"), "{x : Integer, y : String}");
    }

    #[test]
    fn test_variant_type_display() {
        let int_type = spanned_type(TypeExpr::identifier("Integer"));
        let cases = vec![
            VariantCase::with_payload("Some", int_type),
            VariantCase::without_payload("None"),
        ];

        let variant_type = TypeExpr::variant(cases);
        assert_eq!(format!("{variant_type}"), "(| Some Integer | None)");
    }

    #[test]
    fn test_constrained_type_display() {
        let var_a = spanned_type(TypeExpr::variable("a"));
        let string_type = spanned_type(TypeExpr::identifier("String"));
        let func_type = spanned_type(TypeExpr::function(vec![var_a.clone()], string_type));

        let constraint = TypeConstraint::new("Show", var_a);
        let constrained_type = TypeExpr::constrained(vec![constraint], func_type);

        assert_eq!(format!("{constrained_type}"), "(Show 'a => ('a -> String))");
    }
}

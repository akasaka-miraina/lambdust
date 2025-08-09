//! Template expansion for macro output.

use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Spanned};
use crate::macro_system::pattern::PatternBindings;

/// A template for generating expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Template {
    /// Substitutes the bound value of a pattern variable
    Variable(String),
    /// Generates a literal value (number, string, boolean, etc.)
    Literal(Literal),
    /// Generates an identifier with the specified name
    Identifier(String),
    /// Generates a keyword with the specified name
    Keyword(String),
    /// Generates the nil/empty list value
    Nil,
    /// Generates a list containing expanded sub-templates
    List(Vec<Template>),
    /// Generates a cons pair (car . cdr)
    Pair {
        /// Template for the first element of the pair
        car: Box<Template>,
        /// Template for the rest/tail of the pair
        cdr: Box<Template>,
    },
    /// Generates repetitive structure using ellipsis (...) patterns
    Ellipsis {
        /// Fixed templates that appear before the ellipsis
        templates: Vec<Template>,
        /// Template that gets repeated for each ellipsis binding
        ellipsis_template: Box<Template>,
        /// Optional templates that appear after the ellipsis
        rest: Option<Box<Template>>,
    },
    /// Applies a transformation function to the argument template
    Transform {
        /// Name of the transformation function to apply
        function: String,
        /// Template argument to be transformed
        argument: Box<Template>,
    },
    /// Conditional template expansion based on a condition
    Conditional {
        /// Template condition to evaluate
        condition: Box<Template>,
        /// Template to expand if condition is true
        then_branch: Box<Template>,
        /// Optional template to expand if condition is false
        else_branch: Option<Box<Template>>,
    },
    /// Splices the bound value directly into the output (unquoted)
    Splice(String),
}

impl Template {
    /// Creates a variable template that substitutes a pattern binding.
    pub fn variable(name: impl Into<String>) -> Self {
        Template::Variable(name.into())
    }
    
    /// Creates a literal template that generates a specific literal value.
    pub fn literal(lit: Literal) -> Self {
        Template::Literal(lit)
    }
    
    /// Creates an identifier template that generates a specific identifier.
    pub fn identifier(name: impl Into<String>) -> Self {
        Template::Identifier(name.into())
    }
    
    /// Creates a keyword template that generates a specific keyword.
    pub fn keyword(name: impl Into<String>) -> Self {
        Template::Keyword(name.into())
    }
    
    /// Creates a list template with the given sub-templates.
    pub fn list(templates: Vec<Template>) -> Self {
        Template::List(templates)
    }
    
    /// Creates a pair template with the given car and cdr templates.
    pub fn pair(car: Template, cdr: Template) -> Self {
        Template::Pair {
            car: Box::new(car),
            cdr: Box::new(cdr),
        }
    }
    
    /// Creates an ellipsis template with fixed templates, repeating template, and optional rest.
    pub fn ellipsis(templates: Vec<Template>, ellipsis_template: Template, rest: Option<Template>) -> Self {
        Template::Ellipsis {
            templates,
            ellipsis_template: Box::new(ellipsis_template),
            rest: rest.map(Box::new),
        }
    }
    
    /// Creates a transform template that applies a function to an argument template.
    pub fn transform(function: impl Into<String>, argument: Template) -> Self {
        Template::Transform {
            function: function.into(),
            argument: Box::new(argument),
        }
    }
    
    /// Creates a conditional template with condition, then, and optional else branches.
    pub fn conditional(condition: Template, then_branch: Template, else_branch: Option<Template>) -> Self {
        Template::Conditional {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }
    
    /// Creates a splice template that directly substitutes a pattern binding.
    pub fn splice(name: impl Into<String>) -> Self {
        Template::Splice(name.into())
    }
    
    /// Expands this template into an expression using the given pattern bindings.
    pub fn expand(&self, bindings: &PatternBindings, span: crate::diagnostics::Span) -> Result<Spanned<Expr>> {
        match self {
            Template::Variable(name) => {
                if let Some(expr) = bindings.get(name) {
                    Ok(expr.clone())
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Unbound template variable: {name}"),
                        span,
                    )))
                }
            }
            
            Template::Literal(lit) => {
                Ok(Spanned::new(Expr::Literal(lit.clone()), span))
            }
            
            Template::Identifier(name) => {
                Ok(Spanned::new(Expr::Identifier(name.clone()), span))
            }
            
            Template::Keyword(name) => {
                Ok(Spanned::new(Expr::Keyword(name.clone()), span))
            }
            
            Template::Nil => {
                Ok(Spanned::new(Expr::Literal(Literal::Nil), span))
            }
            
            Template::List(templates) => {
                let mut expanded_items = Vec::new();
                
                for template in templates {
                    let expanded = template.expand(bindings, span)?;
                    expanded_items.push(expanded);
                }
                
                if expanded_items.is_empty() {
                    Ok(Spanned::new(Expr::Literal(Literal::Nil), span))
                } else {
                    let operator = expanded_items.remove(0);
                    Ok(Spanned::new(
                        Expr::Application {
                            operator: Box::new(operator),
                            operands: expanded_items,
                        },
                        span,
                    ))
                }
            }
            
            Template::Pair { car, cdr } => {
                let car_expanded = car.expand(bindings, span)?;
                let cdr_expanded = cdr.expand(bindings, span)?;
                Ok(Spanned::new(
                    Expr::Pair {
                        car: Box::new(car_expanded),
                        cdr: Box::new(cdr_expanded),
                    },
                    span,
                ))
            }
            
            Template::Ellipsis { templates, ellipsis_template, rest } => {
                Err(Box::new(Error::macro_error(
                    "Ellipsis templates not yet implemented".to_string(),
                    span,
                )))
            }
            
            Template::Transform { function, argument } => {
                // For now, just expand the argument and return it
                // TODO: Implement actual transformation functions
                argument.expand(bindings, span)
            }
            
            Template::Conditional { condition, then_branch, else_branch } => {
                // For now, just return the then_branch
                // TODO: Implement proper conditional logic
                then_branch.expand(bindings, span)
            }
            
            Template::Splice(name) => {
                // Similar to Variable but intended for splicing
                if let Some(expr) = bindings.get(name) {
                    Ok(expr.clone())
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Unbound splice variable: {name}"),
                        span,
                    )))
                }
            }
        }
    }
}
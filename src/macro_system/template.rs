//! Template expansion for macro definitions.
//!
//! This module implements the template expansion system used in Scheme macro
//! definitions. Templates define how to construct the output expression based
//! on the variables bound during pattern matching.

#![allow(missing_docs)]

use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Span, Spanned};
use super::pattern::PatternBindings;
// use std::collections::HashMap;

/// A template that defines how to expand a macro.
#[derive(Debug, Clone, PartialEq)]
pub enum Template {
    /// Insert a bound variable
    Variable(String),
    
    /// Insert a literal value
    Literal(Literal),
    
    /// Insert an identifier
    Identifier(String),
    
    /// Insert a keyword
    Keyword(String),
    
    /// Insert the empty list
    Nil,
    
    /// Create a list from sub-templates
    List(Vec<Template>),
    
    /// Create a list with repeating elements (ellipsis)
    Ellipsis {
        templates: Vec<Template>,
        ellipsis_template: Box<Template>,
        rest: Option<Box<Template>>,
    },
    
    /// Create a pair (dotted list)
    Pair {
        car: Box<Template>,
        cdr: Box<Template>,
    },
    
    /// Conditional template expansion
    Conditional {
        condition: Box<Template>,
        then_template: Box<Template>,
        else_template: Option<Box<Template>>,
    },
    
    /// Apply a template transformation function
    Transform {
        function: String,
        argument: Box<Template>,
    },
    
    /// Splice in a list of expressions (for ellipsis expansion)
    Splice(String),
}

impl Template {
    /// Creates a variable template.
    pub fn variable(name: impl Into<String>) -> Self {
        Template::Variable(name)
    }
    
    /// Creates a literal template.
    pub fn literal(lit: Literal) -> Self {
        Template::Literal(lit)
    }
    
    /// Creates an identifier template.
    pub fn identifier(name: impl Into<String>) -> Self {
        Template::Identifier(name)
    }
    
    /// Creates a list template.
    pub fn list(templates: Vec<Template>) -> Self {
        Template::List(templates)
    }
    
    /// Creates an ellipsis template.
    pub fn ellipsis(
        templates: Vec<Template>,
        ellipsis_template: Template,
        rest: Option<Template>,
    ) -> Self {
        Template::Ellipsis {
            templates,
            ellipsis_template: Box::new(ellipsis_template),
            rest: rest.map(Box::new),
        }
    }
    
    /// Creates a pair template.
    pub fn pair(car: Template, cdr: Template) -> Self {
        Template::Pair {
            car: Box::new(car),
            cdr: Box::new(cdr),
        }
    }
    
    /// Creates a conditional template.
    pub fn conditional(
        condition: Template,
        then_template: Template,
        else_template: Option<Template>,
    ) -> Self {
        Template::Conditional {
            condition: Box::new(condition),
            then_template: Box::new(then_template),
            else_template: else_template.map(Box::new),
        }
    }
    
    /// Expands this template using the given pattern bindings.
    pub fn expand(&self, bindings: &PatternBindings, span: Span) -> Result<Spanned<Expr>> {
        match self {
            Template::Variable(name) => {
                if let Some(expr) = bindings.get(name) {
                    Ok(expr.clone())
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Unbound template variable: {name}"),
                        span,
                    ))
                }
            }
            
            Template::Literal(lit) => {
                Ok(Spanned::new(Expr::Literal(lit.clone()), span))
            }
            
            Template::Identifier(name) => {
                Ok(Spanned::new(Expr::Identifier(name.clone()), span))
            }
            
            Template::Keyword(kw) => {
                Ok(Spanned::new(Expr::Keyword(kw.clone()), span))
            }
            
            Template::Nil => {
                Ok(Spanned::new(Expr::Literal(Literal::Nil), span))
            }
            
            Template::List(templates) => {
                let mut expanded_items = Vec::new();
                for template in templates {
                    expanded_items.push(template.expand(bindings, span)?);
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
            
            Template::Ellipsis { templates, ellipsis_template, rest } => {
                self.expand_ellipsis(templates, ellipsis_template, rest.as_deref(), bindings, span)
            }
            
            Template::Pair { car, cdr } => {
                let car_expr = car.expand(bindings, span)?;
                let cdr_expr = cdr.expand(bindings, span)?;
                Ok(Spanned::new(
                    Expr::Pair {
                        car: Box::new(car_expr),
                        cdr: Box::new(cdr_expr),
                    },
                    span,
                ))
            }
            
            Template::Conditional { condition, then_template, else_template } => {
                let condition_expr = condition.expand(bindings, span)?;
                
                // Evaluate condition (simplified - in full implementation would use evaluator)
                let condition_result = self.evaluate_condition(&condition_expr)?;
                
                if condition_result {
                    then_template.expand(bindings, span)
                } else if let Some(else_tmpl) = else_template {
                    else_tmpl.expand(bindings, span)
                } else {
                    Ok(Spanned::new(Expr::Literal(Literal::Nil), span))
                }
            }
            
            Template::Transform { function, argument } => {
                let arg_expr = argument.expand(bindings, span)?;
                self.apply_transform(function, arg_expr, span)
            }
            
            Template::Splice(name) => {
                if let Some(expr_list) = bindings.get_ellipsis(name) {
                    // Create a begin expression with all the spliced expressions
                    Ok(Spanned::new(Expr::Begin(expr_list.clone()), span))
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Unbound ellipsis variable for splice: {name}"),
                        span,
                    ))
                }
            }
        }
    }
    
    /// Expands an ellipsis template.
    fn expand_ellipsis(
        &self,
        pre_templates: &[Template],
        ellipsis_template: &Template,
        rest_template: Option<&Template>,
        bindings: &PatternBindings,
        span: Span,
    ) -> Result<Spanned<Expr>> {
        let mut result_items = Vec::new();
        
        // Expand pre-templates
        for template in pre_templates {
            result_items.push(template.expand(bindings, span)?);
        }
        
        // Find the length of ellipsis variables
        let ellipsis_length = self.determine_ellipsis_length(ellipsis_template, bindings)?;
        
        // Expand ellipsis template for each repetition
        for i in 0..ellipsis_length {
            let indexed_bindings = self.create_indexed_bindings(bindings, i)?;
            let expanded = ellipsis_template.expand(&indexed_bindings, span)?;
            result_items.push(expanded);
        }
        
        // Expand rest template if present
        if let Some(rest_tmpl) = rest_template {
            result_items.push(rest_tmpl.expand(bindings, span)?);
        }
        
        // Create the result expression
        if result_items.is_empty() {
            Ok(Spanned::new(Expr::Literal(Literal::Nil), span))
        } else {
            let operator = result_items.remove(0);
            Ok(Spanned::new(
                Expr::Application {
                    operator: Box::new(operator),
                    operands: result_items,
                },
                span,
            ))
        }
    }
    
    /// Determines the length of ellipsis expansion based on bound variables.
    fn determine_ellipsis_length(
        &self,
        ellipsis_template: &Template,
        bindings: &PatternBindings,
    ) -> Result<usize> {
        let ellipsis_vars = ellipsis_template.ellipsis_variables();
        
        if ellipsis_vars.is_empty() {
            return Ok(0);
        }
        
        let mut length = None;
        for var in &ellipsis_vars {
            if let Some(expr_list) = bindings.get_ellipsis(var) {
                let var_length = expr_list.len();
                if let Some(existing_length) = length {
                    if existing_length != var_length {
                        return Err(Box::new(Error::macro_error(
                            format!(
                                "Ellipsis variable length mismatch: {var} has length {var_length}, expected {existing_length}"
                            ),
                            crate::diagnostics::Span::new(0, 0),
                        ).boxed());
                    }
                } else {
                    length = Some(var_length);
                }
            } else {
                return Err(Box::new(Error::macro_error(
                    format!("Unbound ellipsis variable: {var}"),
                    crate::diagnostics::Span::new(0, 0),
                ).boxed());
            }
        }
        
        Ok(length.unwrap_or(0))
    }
    
    /// Creates bindings for a specific index in ellipsis expansion.
    fn create_indexed_bindings(
        &self,
        bindings: &PatternBindings,
        index: usize,
    ) -> Result<PatternBindings> {
        let mut indexed_bindings = PatternBindings::new();
        
        // Copy regular bindings
        for (name, expr) in bindings.bindings() {
            indexed_bindings.bind(name.clone()), expr.clone());
        }
        
        // Create indexed bindings for ellipsis variables
        for (name, expr_list) in bindings.ellipsis_bindings() {
            if index < expr_list.len() {
                indexed_bindings.bind(name.clone()), expr_list[index].clone());
            }
        }
        
        Ok(indexed_bindings)
    }
    
    /// Gets all ellipsis variables referenced in this template.
    fn ellipsis_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_ellipsis_variables(&mut vars);
        vars.sort();
        vars.dedup();
        vars
    }
    
    /// Recursively collects ellipsis variables.
    fn collect_ellipsis_variables(&self, vars: &mut Vec<String>) {
        match self {
            Template::Variable(name) => {
                // Variables within ellipsis context are ellipsis variables
                vars.push(name.clone());
            }
            Template::List(templates) => {
                for template in templates {
                    template.collect_ellipsis_variables(vars);
                }
            }
            Template::Ellipsis { templates, ellipsis_template, rest } => {
                for template in templates {
                    template.collect_ellipsis_variables(vars);
                }
                ellipsis_template.collect_ellipsis_variables(vars);
                if let Some(rest_tmpl) = rest {
                    rest_tmpl.collect_ellipsis_variables(vars);
                }
            }
            Template::Pair { car, cdr } => {
                car.collect_ellipsis_variables(vars);
                cdr.collect_ellipsis_variables(vars);
            }
            Template::Conditional { condition, then_template, else_template } => {
                condition.collect_ellipsis_variables(vars);
                then_template.collect_ellipsis_variables(vars);
                if let Some(else_tmpl) = else_template {
                    else_tmpl.collect_ellipsis_variables(vars);
                }
            }
            Template::Transform { argument, .. } => {
                argument.collect_ellipsis_variables(vars);
            }
            Template::Splice(name) => {
                vars.push(name.clone());
            }
            _ => {} // Literals, identifiers, etc. don't contain ellipsis variables
        }
    }
    
    /// Evaluates a condition expression (simplified).
    fn evaluate_condition(&self, expr: &Spanned<Expr>) -> Result<bool> {
        match &expr.inner {
            Expr::Literal(Literal::Boolean(b)) => Ok(*b),
            Expr::Literal(Literal::Nil) => Ok(false),
            _ => Ok(true), // Everything else is truthy
        }
    }
    
    /// Applies a transformation function to an expression.
    fn apply_transform(&self, function: &str, expr: Spanned<Expr>, span: Span) -> Result<Spanned<Expr>> {
        match function {
            "quote" => Ok(Spanned::new(Expr::Quote(Box::new(expr)), span)),
            "unquote" => {
                // Remove one level of quoting
                match expr.inner {
                    Expr::Quote(inner) => Ok(*inner),
                    _ => Ok(expr),
                }
            }
            "syntax" => {
                // Mark as syntax (for hygiene purposes)
                // In full implementation, this would mark identifiers with syntax scope
                Ok(expr)
            }
            "unsyntax" => {
                // Remove syntax marking
                Ok(expr)
            }
            _ => Err(Box::new(Error::macro_error(
                format!("Unknown transformation function: {function}"),
                span,
            )),
        }
    }
    
    /// Gets all variables referenced in this template.
    pub fn variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_variables(&mut vars);
        vars.sort();
        vars.dedup();
        vars
    }
    
    /// Recursively collects all variable names in this template.
    fn collect_variables(&self, vars: &mut Vec<String>) {
        match self {
            Template::Variable(name) => vars.push(name.clone()),
            Template::List(templates) => {
                for template in templates {
                    template.collect_variables(vars);
                }
            }
            Template::Ellipsis { templates, ellipsis_template, rest } => {
                for template in templates {
                    template.collect_variables(vars);
                }
                ellipsis_template.collect_variables(vars);
                if let Some(rest_tmpl) = rest {
                    rest_tmpl.collect_variables(vars);
                }
            }
            Template::Pair { car, cdr } => {
                car.collect_variables(vars);
                cdr.collect_variables(vars);
            }
            Template::Conditional { condition, then_template, else_template } => {
                condition.collect_variables(vars);
                then_template.collect_variables(vars);
                if let Some(else_tmpl) = else_template {
                    else_tmpl.collect_variables(vars);
                }
            }
            Template::Transform { argument, .. } => {
                argument.collect_variables(vars);
            }
            Template::Splice(name) => vars.push(name.clone()),
            _ => {} // Literals, identifiers, etc. don't reference variables
        }
    }
}

/// Builder for constructing complex templates.
#[derive(Debug)]
pub struct TemplateBuilder {
    template: Template,
}

impl TemplateBuilder {
    /// Creates a new template builder.
    pub fn new() -> Self {
        Self {
            template: Template::Nil,
        }
    }
    
    /// Sets the template to a variable.
    pub fn variable(mut self, name: impl Into<String>) -> Self {
        self.template = Template::Variable(name.into())
        self
    }
    
    /// Sets the template to a literal.
    pub fn literal(mut self, lit: Literal) -> Self {
        self.template = Template::Literal(lit);
        self
    }
    
    /// Sets the template to an identifier.
    pub fn identifier(mut self, name: impl Into<String>) -> Self {
        self.template = Template::Identifier(name.into())
        self
    }
    
    /// Sets the template to a list.
    pub fn list(mut self, templates: Vec<Template>) -> Self {
        self.template = Template::List(templates);
        self
    }
    
    /// Builds the template.
    pub fn build(self) -> Template {
        self.template
    }
}

impl Default for TemplateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;
    use super::super::pattern::PatternBindings;
    
    fn make_spanned<T>(value: T) -> Spanned<T> {
        Spanned::new(value, Span::new(0, 1))
    }
    
    #[test]
    fn test_variable_template() {
        let template = Template::variable("x");
        let mut bindings = PatternBindings::new();
        bindings.bind("x".to_string(), make_spanned(Expr::Literal(Literal::Number(42.0))));
        
        let result = template.expand(&bindings, Span::new(0, 1)).unwrap();
        match result.inner {
            Expr::Literal(Literal::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected number literal"),
        }
    }
    
    #[test]
    fn test_literal_template() {
        let template = Template::literal(Literal::String("hello".to_string()));
        let bindings = PatternBindings::new();
        
        let result = template.expand(&bindings, Span::new(0, 1)).unwrap();
        match result.inner {
            Expr::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected string literal"),
        }
    }
    
    #[test]
    fn test_list_template() {
        let template = Template::list(vec![
            Template::identifier("if"),
            Template::variable("test"),
            Template::variable("then"),
            Template::variable("else"),
        ]);
        
        let mut bindings = PatternBindings::new();
        bindings.bind("test".to_string(), make_spanned(Expr::Identifier("condition".to_string())));
        bindings.bind("then".to_string(), make_spanned(Expr::Literal(Literal::Number(1.0))));
        bindings.bind("else".to_string(), make_spanned(Expr::Literal(Literal::Number(2.0))));
        
        let result = template.expand(&bindings, Span::new(0, 1)).unwrap();
        match result.inner {
            Expr::Application { operator, operands } => {
                assert_eq!(operands.len(), 3);
                match &operator.inner {
                    Expr::Identifier(name) => assert_eq!(name, "if"),
                    _ => panic!("Expected if identifier"),
                }
            }
            _ => panic!("Expected application"),
        }
    }
    
    #[test]
    fn test_template_builder() {
        let template = TemplateBuilder::new()
            .identifier("lambda")
            .build();
            
        match template {
            Template::Identifier(name) => assert_eq!(name, "lambda"),
            _ => panic!("Expected identifier template"),
        }
    }
}
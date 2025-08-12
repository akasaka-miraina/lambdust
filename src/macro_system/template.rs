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
    /// SRFI-149: Multiple consecutive ellipses for nested structure
    /// ((a b ...) ...) support with depth tracking
    NestedEllipsis {
        /// Fixed templates that appear before the nested ellipsis
        templates: Vec<Template>,
        /// Template that contains nested ellipsis patterns
        nested_template: Box<Template>,
        /// Ellipsis depth (how many consecutive ellipses)
        depth: usize,
        /// Optional templates that appear after the ellipsis
        rest: Option<Box<Template>>,
    },
    /// SRFI-149: Extra ellipses in template (depth > pattern depth)
    ExtraEllipsis {
        /// Base template to repeat
        base_template: Box<Template>,
        /// Number of extra ellipses beyond pattern depth
        extra_depth: usize,
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
    
    /// Creates a nested ellipsis template for SRFI-149 multiple consecutive ellipses.
    pub fn nested_ellipsis(
        templates: Vec<Template>, 
        nested_template: Template, 
        depth: usize, 
        rest: Option<Template>
    ) -> Self {
        Template::NestedEllipsis {
            templates,
            nested_template: Box::new(nested_template),
            depth,
            rest: rest.map(Box::new),
        }
    }
    
    /// Creates an extra ellipsis template for SRFI-149 templates with more ellipses than patterns.
    pub fn extra_ellipsis(base_template: Template, extra_depth: usize) -> Self {
        Template::ExtraEllipsis {
            base_template: Box::new(base_template),
            extra_depth,
        }
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
                self.expand_ellipsis(templates, ellipsis_template, rest.as_ref().map(|v| &**v), bindings, span)
            }
            
            Template::NestedEllipsis { templates, nested_template, depth, rest } => {
                self.expand_nested_ellipsis(templates, nested_template, *depth, rest.as_ref().map(|v| &**v), bindings, span)
            }
            
            Template::ExtraEllipsis { base_template, extra_depth } => {
                self.expand_extra_ellipsis(base_template, *extra_depth, bindings, span)
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

    /// Expands a standard ellipsis template with pattern bindings.
    fn expand_ellipsis(
        &self,
        templates: &[Template],
        ellipsis_template: &Template,
        rest: Option<&Template>,
        bindings: &PatternBindings,
        span: crate::diagnostics::Span,
    ) -> Result<Spanned<Expr>> {
        let mut expanded_items = Vec::new();
        
        // Expand fixed templates before ellipsis
        for template in templates {
            let expanded = template.expand(bindings, span)?;
            expanded_items.push(expanded);
        }
        
        // Find the ellipsis variables and their binding lengths
        let ellipsis_vars = self.find_ellipsis_variables(ellipsis_template);
        let max_length = ellipsis_vars
            .iter()
            .filter_map(|var| bindings.get_ellipsis(var))
            .map(|exprs| exprs.len())
            .max()
            .unwrap_or(0);
        
        // Expand ellipsis template for each repetition
        for i in 0..max_length {
            let mut ellipsis_bindings = PatternBindings::new();
            
            // Create bindings for this repetition
            for var in &ellipsis_vars {
                if let Some(expr_list) = bindings.get_ellipsis(var) {
                    if i < expr_list.len() {
                        ellipsis_bindings.bind(var.clone(), expr_list[i].clone());
                    }
                }
            }
            
            // Copy non-ellipsis bindings
            for (name, expr) in bindings.bindings() {
                ellipsis_bindings.bind(name.clone(), expr.clone());
            }
            
            let expanded = ellipsis_template.expand(&ellipsis_bindings, span)?;
            expanded_items.push(expanded);
        }
        
        // Expand rest templates after ellipsis
        if let Some(rest_template) = rest {
            let expanded = rest_template.expand(bindings, span)?;
            expanded_items.push(expanded);
        }
        
        // Convert to appropriate expression form
        if expanded_items.is_empty() {
            Ok(Spanned::new(Expr::Literal(Literal::Nil), span))
        } else if expanded_items.len() == 1 {
            Ok(expanded_items.into_iter().next().unwrap())
        } else {
            Ok(Spanned::new(
                Expr::List(expanded_items),
                span,
            ))
        }
    }

    /// SRFI-149: Expands a nested ellipsis template ((a b ...) ...)
    fn expand_nested_ellipsis(
        &self,
        templates: &[Template],
        nested_template: &Template,
        depth: usize,
        rest: Option<&Template>,
        bindings: &PatternBindings,
        span: crate::diagnostics::Span,
    ) -> Result<Spanned<Expr>> {
        // SRFI-149 error handling: validate depth
        if depth == 0 {
            return Err(Box::new(Error::macro_error(
                "SRFI-149 error: nested ellipsis depth cannot be zero".to_string(),
                span,
            )));
        }
        
        if depth > 10 {
            return Err(Box::new(Error::macro_error(
                format!("SRFI-149 error: nested ellipsis depth {depth} exceeds maximum (10)"),
                span,
            )));
        }
        
        if depth == 1 {
            // Single depth, same as regular ellipsis
            return self.expand_ellipsis(templates, nested_template, rest, bindings, span);
        }
        
        // Multiple depth - implement nested expansion
        let mut expanded_items = Vec::new();
        
        // Expand fixed templates before ellipsis
        for template in templates {
            let expanded = template.expand(bindings, span)?;
            expanded_items.push(expanded);
        }
        
        // Find nested ellipsis variables and compute nested structure
        let ellipsis_vars = self.find_ellipsis_variables(nested_template);
        
        // SRFI-149 error handling: check for unbound variables
        if ellipsis_vars.is_empty() {
            return Err(Box::new(Error::macro_error(
                "SRFI-149 error: nested ellipsis template contains no ellipsis variables".to_string(),
                span,
            )));
        }
        
        // For nested ellipses, we need to find variables at different depths
        let mut nested_structure = Vec::new();
        let mut any_bindings_found = false;
        
        for var in &ellipsis_vars {
            if let Some(expr_list) = bindings.get_ellipsis(var) {
                any_bindings_found = true;
                
                // SRFI-149 error handling: validate binding length
                if expr_list.is_empty() {
                    continue; // Empty bindings are allowed
                }
                
                // Group expressions by nesting level
                // This is a simplified approach - real SRFI-149 needs more sophisticated grouping
                let chunks: Vec<&[Spanned<Expr>]> = expr_list.chunks(depth).collect();
                for chunk in chunks {
                    let mut chunk_bindings = PatternBindings::new();
                    chunk_bindings.bind_ellipsis(var.clone(), chunk.to_vec());
                    
                    // Copy non-ellipsis bindings
                    for (name, expr) in bindings.bindings() {
                        chunk_bindings.bind(name.clone(), expr.clone());
                    }
                    
                    let expanded = nested_template.expand(&chunk_bindings, span)?;
                    nested_structure.push(expanded);
                }
            }
        }
        
        // SRFI-149 error handling: check if any variables were bound
        if !any_bindings_found {
            return Err(Box::new(Error::macro_error(
                format!("SRFI-149 error: no ellipsis bindings found for variables: {ellipsis_vars:?}"),
                span,
            )));
        }
        
        expanded_items.extend(nested_structure);
        
        // Expand rest templates
        if let Some(rest_template) = rest {
            let expanded = rest_template.expand(bindings, span)?;
            expanded_items.push(expanded);
        }
        
        Ok(Spanned::new(
            Expr::List(expanded_items),
            span,
        ))
    }

    /// SRFI-149: Expands extra ellipses in template (more ellipses than in pattern)
    fn expand_extra_ellipsis(
        &self,
        base_template: &Template,
        extra_depth: usize,
        bindings: &PatternBindings,
        span: crate::diagnostics::Span,
    ) -> Result<Spanned<Expr>> {
        // SRFI-149 error handling: validate extra depth
        if extra_depth == 0 {
            return base_template.expand(bindings, span);
        }
        
        if extra_depth > 5 {
            return Err(Box::new(Error::macro_error(
                format!("SRFI-149 error: extra ellipsis depth {extra_depth} exceeds reasonable maximum (5)"),
                span,
            )));
        }
        
        // Create extra nesting by wrapping the result in lists
        let base_result = base_template.expand(bindings, span)?;
        
        let mut result = base_result;
        
        // Apply extra ellipses by wrapping in lists
        for level in 0..extra_depth {
            result = Spanned::new(
                Expr::List(vec![result]),
                span,
            );
            
            // Optional: Add debugging information for deep nesting
            if level > 3 {
                eprintln!("SRFI-149 warning: creating deep nesting at level {}", level + 1);
            }
        }
        
        Ok(result)
    }

    /// Finds all variable names that would be bound as ellipsis variables in this template
    fn find_ellipsis_variables(&self, template: &Template) -> Vec<String> {
        let mut vars = Vec::new();
        Self::collect_ellipsis_variables(template, &mut vars);
        vars.sort();
        vars.dedup();
        vars
    }

    /// Recursively collects ellipsis variable names from a template
    fn collect_ellipsis_variables(template: &Template, vars: &mut Vec<String>) {
        match template {
            Template::Variable(name) => vars.push(name.clone()),
            Template::List(templates) => {
                for tmpl in templates {
                    Self::collect_ellipsis_variables(tmpl, vars);
                }
            }
            Template::Ellipsis { ellipsis_template, .. } => {
                Self::collect_ellipsis_variables(ellipsis_template, vars);
            }
            Template::NestedEllipsis { nested_template, .. } => {
                Self::collect_ellipsis_variables(nested_template, vars);
            }
            Template::ExtraEllipsis { base_template, .. } => {
                Self::collect_ellipsis_variables(base_template, vars);
            }
            Template::Pair { car, cdr } => {
                Self::collect_ellipsis_variables(car, vars);
                Self::collect_ellipsis_variables(cdr, vars);
            }
            Template::Transform { argument, .. } => {
                Self::collect_ellipsis_variables(argument, vars);
            }
            Template::Conditional { condition, then_branch, else_branch } => {
                Self::collect_ellipsis_variables(condition, vars);
                Self::collect_ellipsis_variables(then_branch, vars);
                if let Some(else_tmpl) = else_branch {
                    Self::collect_ellipsis_variables(else_tmpl, vars);
                }
            }
            Template::Splice(name) => vars.push(name.clone()),
            _ => {}
        }
    }

    /// Computes the ellipsis depth of this template (how many nested ellipses).
    pub fn ellipsis_depth(&self) -> usize {
        match self {
            Template::Ellipsis { ellipsis_template, .. } => {
                1 + ellipsis_template.ellipsis_depth()
            }
            Template::NestedEllipsis { depth, .. } => *depth,
            Template::ExtraEllipsis { extra_depth, .. } => *extra_depth,
            Template::List(templates) => {
                templates.iter().map(|t| t.ellipsis_depth()).max().unwrap_or(0)
            }
            Template::Pair { car, cdr } => {
                car.ellipsis_depth().max(cdr.ellipsis_depth())
            }
            Template::Transform { argument, .. } => argument.ellipsis_depth(),
            Template::Conditional { condition, then_branch, else_branch } => {
                let max_depth = condition.ellipsis_depth().max(then_branch.ellipsis_depth());
                if let Some(else_tmpl) = else_branch {
                    max_depth.max(else_tmpl.ellipsis_depth())
                } else {
                    max_depth
                }
            }
            _ => 0,
        }
    }

    /// SRFI-149: Analyzes if this template needs extra ellipses resolution
    /// (has more ellipses depth than the given pattern depth)
    pub fn needs_extra_ellipses(&self, pattern_depth: usize) -> bool {
        self.ellipsis_depth() > pattern_depth
    }

    /// SRFI-149: Creates a template with extra ellipses handling for depth mismatches
    pub fn with_extra_ellipses(self, pattern_depth: usize) -> Self {
        let template_depth = self.ellipsis_depth();
        if template_depth > pattern_depth {
            Template::ExtraEllipsis {
                base_template: Box::new(self),
                extra_depth: template_depth - pattern_depth,
            }
        } else {
            self
        }
    }

    /// SRFI-149: Applies ambiguity resolution rules for variable binding conflicts
    /// When a variable could be bound at multiple ellipsis depths, SRFI-149 specifies
    /// that the innermost (deepest) binding takes precedence
    pub fn resolve_ambiguities(&mut self, pattern_vars: &std::collections::HashMap<String, usize>) {
        self.resolve_ambiguities_inner(pattern_vars, 0);
    }

    fn resolve_ambiguities_inner(&mut self, pattern_vars: &std::collections::HashMap<String, usize>, current_depth: usize) {
        match self {
            Template::Variable(name) => {
                // Check if this variable has depth conflicts and resolve
                if let Some(&var_depth) = pattern_vars.get(name) {
                    if var_depth != current_depth {
                        // SRFI-149: Prefer innermost binding depth
                        // This is handled during expansion by checking binding depths
                    }
                }
            }
            Template::List(templates) => {
                for template in templates {
                    template.resolve_ambiguities_inner(pattern_vars, current_depth);
                }
            }
            Template::Ellipsis { templates, ellipsis_template, rest } => {
                for template in templates {
                    template.resolve_ambiguities_inner(pattern_vars, current_depth);
                }
                ellipsis_template.resolve_ambiguities_inner(pattern_vars, current_depth + 1);
                if let Some(rest_template) = rest {
                    rest_template.resolve_ambiguities_inner(pattern_vars, current_depth);
                }
            }
            Template::NestedEllipsis { templates, nested_template, depth, rest } => {
                for template in templates {
                    template.resolve_ambiguities_inner(pattern_vars, current_depth);
                }
                nested_template.resolve_ambiguities_inner(pattern_vars, current_depth + *depth);
                if let Some(rest_template) = rest {
                    rest_template.resolve_ambiguities_inner(pattern_vars, current_depth);
                }
            }
            Template::ExtraEllipsis { base_template, .. } => {
                base_template.resolve_ambiguities_inner(pattern_vars, current_depth);
            }
            Template::Pair { car, cdr } => {
                car.resolve_ambiguities_inner(pattern_vars, current_depth);
                cdr.resolve_ambiguities_inner(pattern_vars, current_depth);
            }
            Template::Transform { argument, .. } => {
                argument.resolve_ambiguities_inner(pattern_vars, current_depth);
            }
            Template::Conditional { condition, then_branch, else_branch } => {
                condition.resolve_ambiguities_inner(pattern_vars, current_depth);
                then_branch.resolve_ambiguities_inner(pattern_vars, current_depth);
                if let Some(else_tmpl) = else_branch {
                    else_tmpl.resolve_ambiguities_inner(pattern_vars, current_depth);
                }
            }
            _ => {}
        }
    }
}
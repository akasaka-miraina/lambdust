//! Syntax-case macro system implementation
//!
//! This module provides the syntax-case macro system, which offers more
//! sophisticated pattern matching capabilities compared to syntax-rules.
//! It includes support for hygienic macro expansion, guard conditions,
//! and runtime pattern matching decisions.

use crate::ast::{Expr, Literal};
use crate::error::{LambdustError, Result};
use super::pattern_matching::{
    Pattern, SyntaxCaseClause, SyntaxCaseBody, MatchResult, BindingValue,
    PatternMatcher,
    // Removed unused imports: SyntaxObject, SourceInfo,
};
use super::hygiene::{ExpansionContext, HygienicEnvironment};
// Removed unused import: HygienicSymbol
use std::collections::HashMap;

/// Syntax-case macro transformer
#[derive(Debug, Clone)]
pub struct SyntaxCaseTransformer {
    /// Literal identifiers
    #[allow(dead_code)]
    literals: Vec<String>,
    /// Pattern matching clauses
    clauses: Vec<SyntaxCaseClause>,
    /// Hygienic environment for macro definition
    definition_environment: HygienicEnvironment,
    /// Pattern matcher
    matcher: PatternMatcher,
}

impl SyntaxCaseTransformer {
    /// Create a new syntax-case transformer
    pub fn new(
        literals: Vec<String>,
        clauses: Vec<SyntaxCaseClause>,
        definition_environment: HygienicEnvironment,
    ) -> Self {
        let matcher = PatternMatcher::new(literals.clone());
        
        Self {
            literals,
            clauses,
            definition_environment,
            matcher,
        }
    }

    /// Transform input expression using syntax-case pattern matching
    ///
    /// This method implements value borrow principles by taking references
    /// and avoiding unnecessary clones until the final result construction.
    pub fn transform(
        &self,
        input: &Expr,
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        // Create expansion context combining definition and usage environments
        let expansion_context = ExpansionContext::new(
            self.definition_environment.clone(),
            usage_environment.clone(),
        );

        // Try each clause until one matches
        for clause in &self.clauses {
            if let Ok(match_result) = self.matcher.match_pattern(
                &clause.pattern,
                input,
                &expansion_context,
            ) {
                if match_result.success {
                    // Check guard condition if present
                    if let Some(guard) = &clause.guard {
                        if !self.evaluate_guard_condition(
                            guard,
                            &match_result,
                            &expansion_context,
                        )? {
                            continue; // Guard failed, try next clause
                        }
                    }

                    // Execute the clause body
                    return self.execute_clause_body(
                        &clause.body,
                        &match_result,
                        &expansion_context,
                    );
                }
            }
        }

        Err(LambdustError::syntax_error(
            "syntax-case: no matching clause found".to_string(),
        ))
    }

    /// Evaluate a guard condition
    fn evaluate_guard_condition(
        &self,
        guard: &Expr,
        match_result: &MatchResult,
        context: &ExpansionContext,
    ) -> Result<bool> {
        // Substitute pattern variables in the guard expression
        let substituted_guard = self.substitute_pattern_variables(
            guard,
            &match_result.bindings,
            context,
        )?;

        // For now, implement a simple guard evaluation
        // In a full implementation, this would evaluate the expression
        // in the appropriate environment
        match substituted_guard {
            Expr::Literal(Literal::Boolean(b)) => Ok(b),
            Expr::Variable(var) if var == "#t" => Ok(true),
            Expr::Variable(var) if var == "#f" => Ok(false),
            _ => {
                // For complex guard expressions, we would need to evaluate them
                // For now, assume true to continue development
                Ok(true)
            }
        }
    }

    /// Execute the body of a matched clause
    fn execute_clause_body(
        &self,
        body: &SyntaxCaseBody,
        match_result: &MatchResult,
        context: &ExpansionContext,
    ) -> Result<Expr> {
        match body {
            SyntaxCaseBody::Template(template) => {
                self.expand_template(template, &match_result.bindings, context)
            }
            SyntaxCaseBody::Expression(expr) => {
                // Substitute pattern variables in the expression
                self.substitute_pattern_variables(expr, &match_result.bindings, context)
            }
        }
    }

    /// Expand a template with pattern variable substitutions
    fn expand_template(
        &self,
        template: &super::pattern_matching::Template,
        bindings: &HashMap<String, BindingValue>,
        context: &ExpansionContext,
    ) -> Result<Expr> {
        use super::pattern_matching::Template;

        match template {
            Template::Literal(lit) => {
                // Parse literal back to expression
                Ok(Expr::Variable(lit.clone()))
            }
            Template::Variable(var) => {
                if let Some(binding) = bindings.get(var) {
                    match binding {
                        BindingValue::Single(expr) => Ok(expr.clone()),
                        BindingValue::List(_exprs) => {
                            // Single variable bound to list - error in this context
                            Err(LambdustError::syntax_error(format!(
                                "template variable {var} bound to list but used as single value"
                            )))
                        }
                        BindingValue::SyntaxObject(obj) => Ok(obj.expression.clone()),
                    }
                } else {
                    // Unbound variable - return as is (might be a literal)
                    Ok(Expr::Variable(var.clone()))
                }
            }
            Template::List(templates) => {
                let mut result_exprs = Vec::new();
                for tmpl in templates {
                    if let Template::Ellipsis(ellipsis_template) = tmpl {
                        // Expand ellipsis template
                        let expanded = self.expand_ellipsis_template(
                            ellipsis_template,
                            bindings,
                            context,
                        )?;
                        result_exprs.extend(expanded);
                    } else {
                        let expanded = self.expand_template(tmpl, bindings, context)?;
                        result_exprs.push(expanded);
                    }
                }
                Ok(Expr::List(result_exprs))
            }
            Template::Vector(templates) => {
                let mut result_exprs = Vec::new();
                for tmpl in templates {
                    let expanded = self.expand_template(tmpl, bindings, context)?;
                    result_exprs.push(expanded);
                }
                Ok(Expr::Vector(result_exprs))
            }
            Template::HygienicVariable(hyg_sym) => {
                // Generate hygienic identifier
                let fresh_name = context.generate_fresh_identifier(&hyg_sym.name)?;
                Ok(Expr::Variable(fresh_name))
            }
            Template::SyntaxObject(inner_template) => {
                // Extract expression from syntax object template
                self.expand_template(inner_template, bindings, context)
            }
            Template::Conditional { condition, then_template, else_template } => {
                // Evaluate condition
                let condition_result = self.evaluate_template_condition(condition, bindings, context)?;
                
                if condition_result {
                    self.expand_template(then_template, bindings, context)
                } else if let Some(else_tmpl) = else_template {
                    self.expand_template(else_tmpl, bindings, context)
                } else {
                    // No else clause, return empty list
                    Ok(Expr::List(vec![]))
                }
            }
            Template::Repeat { template, separator, min_count, max_count } => {
                self.expand_repeat_template(template, separator, *min_count, *max_count, bindings, context)
            }
            Template::Transform { template, function } => {
                self.expand_transform_template(template, function, bindings, context)
            }
            Template::Ellipsis(_) => {
                Err(LambdustError::syntax_error(
                    "Bare ellipsis in template".to_string(),
                ))
            }
            Template::Dotted(_, _) | Template::NestedEllipsis(_, _) => {
                // These would be implemented for full SRFI 46 support
                Err(LambdustError::syntax_error(
                    "Advanced template features not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Expand ellipsis template
    fn expand_ellipsis_template(
        &self,
        template: &super::pattern_matching::Template,
        bindings: &HashMap<String, BindingValue>,
        context: &ExpansionContext,
    ) -> Result<Vec<Expr>> {
        // Find variables in the template that are bound to lists
        let ellipsis_vars = self.find_ellipsis_variables(template, bindings);
        
        if ellipsis_vars.is_empty() {
            return Ok(vec![]);
        }

        // Determine the length of expansion (all ellipsis variables should have same length)
        let expansion_length = ellipsis_vars.iter()
            .filter_map(|var| bindings.get(var))
            .filter_map(|binding| match binding {
                BindingValue::List(exprs) => Some(exprs.len()),
                _ => None,
            })
            .next()
            .unwrap_or(0);

        let mut result = Vec::new();
        for i in 0..expansion_length {
            // Create temporary bindings for this iteration
            let mut iteration_bindings = HashMap::new();
            for var in &ellipsis_vars {
                if let Some(BindingValue::List(exprs)) = bindings.get(var) {
                    if let Some(expr) = exprs.get(i) {
                        iteration_bindings.insert(
                            var.clone(),
                            BindingValue::Single(expr.clone()),
                        );
                    }
                }
            }

            // Add non-ellipsis bindings
            for (var, binding) in bindings {
                if !ellipsis_vars.contains(var) {
                    iteration_bindings.insert(var.clone(), binding.clone());
                }
            }

            // Expand template for this iteration
            let expanded = self.expand_template(template, &iteration_bindings, context)?;
            result.push(expanded);
        }

        Ok(result)
    }

    /// Evaluate condition in template context
    fn evaluate_template_condition(
        &self,
        condition: &Expr,
        bindings: &HashMap<String, BindingValue>,
        context: &ExpansionContext,
    ) -> Result<bool> {
        // Substitute variables in condition
        let substituted = self.substitute_pattern_variables(condition, bindings, context)?;
        
        // Simple evaluation
        match substituted {
            Expr::Literal(Literal::Boolean(b)) => Ok(b),
            Expr::Variable(var) if var == "#t" => Ok(true),
            Expr::Variable(var) if var == "#f" => Ok(false),
            _ => Ok(true), // Default to true for complex expressions
        }
    }

    /// Expand repeat template with separator and count constraints
    fn expand_repeat_template(
        &self,
        template: &super::pattern_matching::Template,
        separator: &Option<String>,
        min_count: usize,
        max_count: Option<usize>,
        bindings: &HashMap<String, BindingValue>,
        context: &ExpansionContext,
    ) -> Result<Expr> {
        // Find ellipsis variables to determine repetition count
        let ellipsis_vars = self.find_ellipsis_variables(template, bindings);
        
        if ellipsis_vars.is_empty() {
            return Err(LambdustError::syntax_error(
                "Repeat template requires ellipsis variables".to_string(),
            ));
        }

        // Determine repetition count
        let repetition_count = ellipsis_vars.iter()
            .filter_map(|var| bindings.get(var))
            .filter_map(|binding| match binding {
                BindingValue::List(exprs) => Some(exprs.len()),
                _ => None,
            })
            .next()
            .unwrap_or(0);

        // Check count constraints
        if repetition_count < min_count {
            return Err(LambdustError::syntax_error(format!(
                "Repeat template requires at least {min_count} repetitions, got {repetition_count}"
            )));
        }

        if let Some(max) = max_count {
            if repetition_count > max {
                return Err(LambdustError::syntax_error(format!(
                    "Repeat template allows at most {max} repetitions, got {repetition_count}"
                )));
            }
        }

        // Expand with separator
        let mut result = Vec::new();
        for i in 0..repetition_count {
            // Create iteration bindings
            let mut iteration_bindings = HashMap::new();
            for var in &ellipsis_vars {
                if let Some(BindingValue::List(exprs)) = bindings.get(var) {
                    if let Some(expr) = exprs.get(i) {
                        iteration_bindings.insert(
                            var.clone(),
                            BindingValue::Single(expr.clone()),
                        );
                    }
                }
            }

            // Add non-ellipsis bindings
            for (var, binding) in bindings {
                if !ellipsis_vars.contains(var) {
                    iteration_bindings.insert(var.clone(), binding.clone());
                }
            }

            // Expand template for this iteration
            let expanded = self.expand_template(template, &iteration_bindings, context)?;
            result.push(expanded);

            // Add separator if not last iteration
            if i < repetition_count - 1 {
                if let Some(sep) = separator {
                    result.push(Expr::Variable(sep.clone()));
                }
            }
        }

        Ok(Expr::List(result))
    }

    /// Expand transform template (apply function to bindings)
    fn expand_transform_template(
        &self,
        template: &super::pattern_matching::Template,
        function: &str,
        bindings: &HashMap<String, BindingValue>,
        context: &ExpansionContext,
    ) -> Result<Expr> {
        // First expand the template normally
        let expanded = self.expand_template(template, bindings, context)?;

        // Apply transformation function
        match function {
            "upcase" => self.apply_upcase_transform(&expanded),
            "downcase" => self.apply_downcase_transform(&expanded),
            "length" => self.apply_length_transform(&expanded),
            "reverse" => self.apply_reverse_transform(&expanded),
            _ => {
                // Unknown function - return as is with warning
                Ok(expanded)
            }
        }
    }

    /// Apply upcase transformation
    fn apply_upcase_transform(&self, expr: &Expr) -> Result<Expr> {
        match expr {
            Expr::Variable(name) => Ok(Expr::Variable(name.to_uppercase())),
            Expr::Literal(Literal::String(s)) => {
                Ok(Expr::Literal(Literal::String(s.to_uppercase())))
            }
            Expr::List(exprs) => {
                let transformed: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.apply_upcase_transform(e))
                    .collect();
                Ok(Expr::List(transformed?))
            }
            _ => Ok(expr.clone()),
        }
    }

    /// Apply downcase transformation
    fn apply_downcase_transform(&self, expr: &Expr) -> Result<Expr> {
        match expr {
            Expr::Variable(name) => Ok(Expr::Variable(name.to_lowercase())),
            Expr::Literal(Literal::String(s)) => {
                Ok(Expr::Literal(Literal::String(s.to_lowercase())))
            }
            Expr::List(exprs) => {
                let transformed: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.apply_downcase_transform(e))
                    .collect();
                Ok(Expr::List(transformed?))
            }
            _ => Ok(expr.clone()),
        }
    }

    /// Apply length transformation
    fn apply_length_transform(&self, expr: &Expr) -> Result<Expr> {
        use crate::lexer::SchemeNumber;
        
        let length = match expr {
            Expr::List(exprs) => exprs.len() as i64,
            Expr::Vector(exprs) => exprs.len() as i64,
            Expr::Literal(Literal::String(s)) => s.len() as i64,
            _ => 0,
        };

        Ok(Expr::Literal(Literal::Number(SchemeNumber::Integer(length))))
    }

    /// Apply reverse transformation
    fn apply_reverse_transform(&self, expr: &Expr) -> Result<Expr> {
        match expr {
            Expr::List(exprs) => {
                let mut reversed = exprs.clone();
                reversed.reverse();
                Ok(Expr::List(reversed))
            }
            Expr::Vector(exprs) => {
                let mut reversed = exprs.clone();
                reversed.reverse();
                Ok(Expr::Vector(reversed))
            }
            _ => Ok(expr.clone()),
        }
    }

    /// Find variables in template that are bound to lists (ellipsis variables)
    fn find_ellipsis_variables(
        &self,
        template: &super::pattern_matching::Template,
        bindings: &HashMap<String, BindingValue>,
    ) -> Vec<String> {
        use super::pattern_matching::Template;
        
        let mut vars = Vec::new();
        
        match template {
            Template::Variable(var) => {
                if let Some(BindingValue::List(_)) = bindings.get(var) {
                    vars.push(var.clone());
                }
            }
            Template::List(templates) | Template::Vector(templates) => {
                for tmpl in templates {
                    vars.extend(self.find_ellipsis_variables(tmpl, bindings));
                }
            }
            Template::Ellipsis(inner) | Template::SyntaxObject(inner) => {
                vars.extend(self.find_ellipsis_variables(inner, bindings));
            }
            _ => {}
        }
        
        vars
    }

    /// Substitute pattern variables in an expression
    fn substitute_pattern_variables(
        &self,
        expr: &Expr,
        bindings: &HashMap<String, BindingValue>,
        context: &ExpansionContext,
    ) -> Result<Expr> {
        match expr {
            Expr::Variable(var) => {
                if let Some(binding) = bindings.get(var) {
                    match binding {
                        BindingValue::Single(substitution) => Ok(substitution.clone()),
                        BindingValue::List(_) => {
                            Err(LambdustError::syntax_error(format!(
                                "Variable {var} bound to list but used as single value"
                            )))
                        }
                        BindingValue::SyntaxObject(obj) => Ok(obj.expression.clone()),
                    }
                } else {
                    Ok(expr.clone())
                }
            }
            Expr::List(exprs) => {
                let mut substituted = Vec::new();
                for sub_expr in exprs {
                    substituted.push(self.substitute_pattern_variables(
                        sub_expr,
                        bindings,
                        context,
                    )?);
                }
                Ok(Expr::List(substituted))
            }
            Expr::Vector(exprs) => {
                let mut substituted = Vec::new();
                for sub_expr in exprs {
                    substituted.push(self.substitute_pattern_variables(
                        sub_expr,
                        bindings,
                        context,
                    )?);
                }
                Ok(Expr::Vector(substituted))
            }
            Expr::DottedList(exprs, tail) => {
                let mut substituted_exprs = Vec::new();
                for sub_expr in exprs {
                    substituted_exprs.push(self.substitute_pattern_variables(
                        sub_expr,
                        bindings,
                        context,
                    )?);
                }
                let substituted_tail = Box::new(self.substitute_pattern_variables(
                    tail,
                    bindings,
                    context,
                )?);
                Ok(Expr::DottedList(substituted_exprs, substituted_tail))
            }
            _ => Ok(expr.clone()),
        }
    }
}

/// Syntax-case macro definition
#[derive(Debug, Clone)]
pub struct SyntaxCaseMacro {
    /// Macro name
    pub name: String,
    /// Transformer
    pub transformer: SyntaxCaseTransformer,
    /// Definition environment
    pub definition_environment: HygienicEnvironment,
}

impl SyntaxCaseMacro {
    /// Create a new syntax-case macro
    pub fn new(
        name: String,
        literals: Vec<String>,
        clauses: Vec<SyntaxCaseClause>,
        definition_environment: HygienicEnvironment,
    ) -> Self {
        let transformer = SyntaxCaseTransformer::new(
            literals,
            clauses,
            definition_environment.clone(),
        );

        Self {
            name,
            transformer,
            definition_environment,
        }
    }

    /// Transform input using this macro
    pub fn transform(
        &self,
        input: &Expr,
        usage_environment: &HygienicEnvironment,
    ) -> Result<Expr> {
        self.transformer.transform(input, usage_environment)
    }
}

/// Helper functions for syntax-case pattern and template construction
/// Parse an expression into a syntax-case pattern
pub fn parse_syntax_case_pattern(expr: &Expr, literals: &[String]) -> Result<Pattern> {
    match expr {
        Expr::Variable(name) => {
            if literals.contains(name) {
                Ok(Pattern::Literal(name.clone()))
            } else if name == "_" {
                Ok(Pattern::Any)
            } else {
                Ok(Pattern::Variable(name.clone()))
            }
        }
        Expr::Literal(lit) => {
            Ok(Pattern::Literal(format!("{lit:?}")))
        }
        Expr::List(exprs) => {
            let mut patterns = Vec::new();
            let mut i = 0;
            
            while i < exprs.len() {
                if let Expr::Variable(name) = &exprs[i] {
                    if name == "..." {
                        if patterns.is_empty() {
                            return Err(LambdustError::syntax_error(
                                "Ellipsis without preceding pattern".to_string(),
                            ));
                        }
                        let last_pattern = patterns.pop().unwrap();
                        patterns.push(Pattern::Ellipsis(Box::new(last_pattern)));
                        i += 1;
                        continue;
                    }
                }
                patterns.push(parse_syntax_case_pattern(&exprs[i], literals)?);
                i += 1;
            }
            
            Ok(Pattern::List(patterns))
        }
        Expr::Vector(exprs) => {
            let mut patterns = Vec::new();
            for expr in exprs {
                patterns.push(parse_syntax_case_pattern(expr, literals)?);
            }
            Ok(Pattern::Vector(patterns))
        }
        _ => {
            Err(LambdustError::syntax_error(
                "Invalid syntax-case pattern".to_string(),
            ))
        }
    }
}

/// Parse an expression into a syntax-case template
pub fn parse_syntax_case_template(expr: &Expr) -> Result<super::pattern_matching::Template> {
    use super::pattern_matching::Template;
    
    match expr {
        Expr::Variable(name) => {
            Ok(Template::Variable(name.clone()))
        }
        Expr::Literal(lit) => {
            Ok(Template::Literal(format!("{lit:?}")))
        }
        Expr::List(exprs) => {
            let mut templates = Vec::new();
            let mut i = 0;
            
            while i < exprs.len() {
                if let Expr::Variable(name) = &exprs[i] {
                    if name == "..." {
                        if templates.is_empty() {
                            return Err(LambdustError::syntax_error(
                                "Ellipsis without preceding template".to_string(),
                            ));
                        }
                        let last_template = templates.pop().unwrap();
                        templates.push(Template::Ellipsis(Box::new(last_template)));
                        i += 1;
                        continue;
                    }
                }
                templates.push(parse_syntax_case_template(&exprs[i])?);
                i += 1;
            }
            
            Ok(Template::List(templates))
        }
        Expr::Vector(exprs) => {
            let mut templates = Vec::new();
            for expr in exprs {
                templates.push(parse_syntax_case_template(expr)?);
            }
            Ok(Template::Vector(templates))
        }
        _ => {
            Ok(Template::Literal(format!("{expr:?}")))
        }
    }
}
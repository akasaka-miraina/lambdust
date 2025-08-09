//! Implementation of syntax-rules for R7RS hygienic macros.
//!
//! This module provides a complete implementation of the syntax-rules macro system
//! as specified in R7RS-small. It includes pattern parsing, template construction,
//! ellipsis handling, and proper hygiene management.

use super::{Pattern, Template, MacroTransformer};
use crate::ast::Expr;
use crate::diagnostics::{Error, Result, Spanned};
use crate::eval::Environment;
use std::collections::HashSet;
use std::rc::Rc;

/// Represents a syntax-rules transformer with multiple pattern-template rules.
#[derive(Debug, Clone)]
pub struct SyntaxRulesTransformer {
    /// List of literal identifiers that should be matched exactly
    pub literals: Vec<String>,
    /// List of pattern-template rules
    pub rules: Vec<SyntaxRule>,
    /// Name of the macro (for debugging)
    pub name: Option<String>,
    /// Environment where the macro was defined (for hygiene)
    pub definition_env: Rc<Environment>,
}

/// A single syntax rule consisting of a pattern and template.
#[derive(Debug, Clone)]
pub struct SyntaxRule {
    /// Pattern to match against input
    pub pattern: Pattern,
    /// Template to expand when pattern matches
    pub template: Template,
}

/// Parses a syntax-rules expression into a transformer.
/// 
/// syntax-rules has the form:
/// (syntax-rules (literal ...) 
///   (pattern template) ...)
/// 
/// Where:
/// - literals are identifiers that must match exactly in patterns
/// - each (pattern template) pair defines a transformation rule
pub fn parse_syntax_rules(
    expr: &Spanned<Expr>,
    definition_env: Rc<Environment>,
) -> Result<SyntaxRulesTransformer> {
    match &expr.inner {
        Expr::Application { operator, operands } => {
            // Check that operator is 'syntax-rules'
            if let Expr::Identifier(name) = &operator.inner {
                if name != "syntax-rules" {
                    return Err(Box::new(Error::macro_error(
                        format!("Expected syntax-rules, got {name}"),
                        operator.span,
                    )));
                }
            } else {
                return Err(Box::new(Error::macro_error(
                    "syntax-rules must be called as a function".to_string(),
                    operator.span,
                )));
            }
            
            if operands.len() < 2 {
                return Err(Box::new(Error::macro_error(
                    "syntax-rules requires at least literals list and one rule".to_string(),
                    expr.span,
                )));
            }
            
            // Parse literals list
            let literals = parse_literals_list(&operands[0])?;
            
            // Parse rules
            let mut rules = Vec::new();
            for rule_expr in &operands[1..] {
                let rule = parse_syntax_rule(rule_expr, &literals)?;
                rules.push(rule);
            }
            
            if rules.is_empty() {
                return Err(Box::new(Error::macro_error(
                    "syntax-rules must have at least one rule".to_string(),
                    expr.span,
                )));
            }
            
            Ok(SyntaxRulesTransformer {
                literals,
                rules,
                name: None,
                definition_env,
            })
        }
        _ => Err(Box::new(Error::macro_error(
            "syntax-rules must be a function application".to_string(),
            expr.span,
        ))),
    }
}

/// Parses the literals list from a syntax-rules expression.
fn parse_literals_list(expr: &Spanned<Expr>) -> Result<Vec<String>> {
    match &expr.inner {
        // Empty list
        Expr::List(elements) if elements.is_empty() => Ok(Vec::new()),
        
        // List of identifiers
        Expr::List(elements) => {
            let mut literals = Vec::new();
            for element in elements {
                match &element.inner {
                    Expr::Identifier(name) => literals.push(name.clone()),
                    _ => return Err(Box::new(Error::macro_error(
                        "Literals must be identifiers".to_string(),
                        element.span,
                    ))),
                }
            }
            Ok(literals)
        }
        
        // Application form: (lit1 lit2 ...)
        Expr::Application { operands, .. } => {
            let mut literals = Vec::new();
            for operand in operands {
                match &operand.inner {
                    Expr::Identifier(name) => literals.push(name.clone()),
                    _ => return Err(Box::new(Error::macro_error(
                        "Literals must be identifiers".to_string(),
                        operand.span,
                    ))),
                }
            }
            Ok(literals)
        }
        
        _ => Err(Box::new(Error::macro_error(
            "Expected list of literal identifiers".to_string(),
            expr.span,
        ))),
    }
}

/// Parses a single syntax rule (pattern template) pair.
fn parse_syntax_rule(
    expr: &Spanned<Expr>,
    literals: &[String],
) -> Result<SyntaxRule> {
    match &expr.inner {
        Expr::List(elements) if elements.len() == 2 => {
            let pattern = parse_pattern(&elements[0], literals)?;
            let template = parse_template(&elements[1])?;
            Ok(SyntaxRule { pattern, template })
        }
        Expr::Application { operands, .. } if operands.len() == 2 => {
            let pattern = parse_pattern(&operands[0], literals)?;
            let template = parse_template(&operands[1])?;
            Ok(SyntaxRule { pattern, template })
        }
        _ => Err(Box::new(Error::macro_error(
            "Syntax rule must be (pattern template)".to_string(),
            expr.span,
        ))),
    }
}

/// Parses a pattern from an expression.
fn parse_pattern(expr: &Spanned<Expr>, literals: &[String]) -> Result<Pattern> {
    match &expr.inner {
        // Identifiers can be variables or literals
        Expr::Identifier(name) => {
            if literals.contains(name) {
                Ok(Pattern::Identifier(name.clone()))
            } else {
                Ok(Pattern::Variable(name.clone()))
            }
        }
        
        // Literals match exactly
        Expr::Literal(lit) => Ok(Pattern::Literal(lit.clone())),
        
        // Keywords match exactly
        Expr::Keyword(kw) => Ok(Pattern::Keyword(kw.clone())),
        
        // Lists can be patterns with ellipsis
        Expr::List(elements) => parse_list_pattern(elements, literals),
        
        // Applications are treated as lists
        Expr::Application { operator, operands } => {
            let mut all_elements = vec![(**operator).clone()];
            all_elements.extend(operands.iter().cloned());
            parse_list_pattern(&all_elements, literals)
        }
        
        // Pairs (dotted lists)
        Expr::Pair { car, cdr } => {
            let car_pattern = parse_pattern(car, literals)?;
            let cdr_pattern = parse_pattern(cdr, literals)?;
            Ok(Pattern::Pair {
                car: Box::new(car_pattern),
                cdr: Box::new(cdr_pattern),
            })
        }
        
        _ => Err(Box::new(Error::macro_error(
            format!("Unsupported pattern type: {:?}", expr.inner),
            expr.span,
        ))),
    }
}

/// Parses a list pattern, handling ellipsis.
fn parse_list_pattern(
    elements: &[Spanned<Expr>],
    literals: &[String],
) -> Result<Pattern> {
    if elements.is_empty() {
        return Ok(Pattern::Nil);
    }
    
    // Look for ellipsis (...)
    let mut patterns = Vec::new();
    let mut i = 0;
    
    while i < elements.len() {
        // Check if next element is ellipsis
        if i + 1 < elements.len() {
            if let Expr::Identifier(name) = &elements[i + 1].inner {
                if name == "..." {
                    // Found ellipsis - create ellipsis pattern
                    let ellipsis_pattern = parse_pattern(&elements[i], literals)?;
                    
                    // Collect remaining patterns after ellipsis
                    let mut rest_patterns = Vec::new();
                    for rest_elem in &elements[i + 2..] {
                        rest_patterns.push(parse_pattern(rest_elem, literals)?);
                    }
                    
                    let rest = if rest_patterns.is_empty() {
                        None
                    } else if rest_patterns.len() == 1 {
                        Some(Box::new(rest_patterns.into_iter().next().unwrap()))
                    } else {
                        Some(Box::new(Pattern::List(rest_patterns)))
                    };
                    
                    return Ok(Pattern::Ellipsis {
                        patterns,
                        ellipsis_pattern: Box::new(ellipsis_pattern),
                        rest,
                    });
                }
            }
        }
        
        // Regular pattern
        patterns.push(parse_pattern(&elements[i], literals)?);
        i += 1;
    }
    
    Ok(Pattern::List(patterns))
}

/// Parses a template from an expression.
fn parse_template(expr: &Spanned<Expr>) -> Result<Template> {
    match &expr.inner {
        // Identifiers become variable references or literals
        Expr::Identifier(name) => Ok(Template::Variable(name.clone())),
        
        // Literals are copied literally
        Expr::Literal(lit) => Ok(Template::Literal(lit.clone())),
        
        // Keywords are copied literally
        Expr::Keyword(kw) => Ok(Template::Keyword(kw.clone())),
        
        // Lists can contain ellipsis expansion
        Expr::List(elements) => parse_list_template(elements),
        
        // Applications are treated as lists
        Expr::Application { operator, operands } => {
            let mut all_elements = vec![(**operator).clone()];
            all_elements.extend(operands.iter().cloned());
            parse_list_template(&all_elements)
        }
        
        // Pairs (dotted lists)
        Expr::Pair { car, cdr } => {
            let car_template = parse_template(car)?;
            let cdr_template = parse_template(cdr)?;
            Ok(Template::Pair {
                car: Box::new(car_template),
                cdr: Box::new(cdr_template),
            })
        }
        
        _ => Err(Box::new(Error::macro_error(
            format!("Unsupported template type: {:?}", expr.inner),
            expr.span,
        ))),
    }
}

/// Parses a list template, handling ellipsis expansion.
fn parse_list_template(elements: &[Spanned<Expr>]) -> Result<Template> {
    if elements.is_empty() {
        return Ok(Template::Nil);
    }
    
    // Look for ellipsis (...)
    let mut templates = Vec::new();
    let mut i = 0;
    
    while i < elements.len() {
        // Check if next element is ellipsis
        if i + 1 < elements.len() {
            if let Expr::Identifier(name) = &elements[i + 1].inner {
                if name == "..." {
                    // Found ellipsis - create ellipsis template
                    let ellipsis_template = parse_template(&elements[i])?;
                    
                    // Collect remaining templates after ellipsis
                    let mut rest_templates = Vec::new();
                    for rest_elem in &elements[i + 2..] {
                        rest_templates.push(parse_template(rest_elem)?);
                    }
                    
                    let rest = if rest_templates.is_empty() {
                        None
                    } else if rest_templates.len() == 1 {
                        Some(Box::new(rest_templates.into_iter().next().unwrap()))
                    } else {
                        Some(Box::new(Template::List(rest_templates)))
                    };
                    
                    return Ok(Template::Ellipsis {
                        templates,
                        ellipsis_template: Box::new(ellipsis_template),
                        rest,
                    });
                }
            }
        }
        
        // Regular template
        templates.push(parse_template(&elements[i])?);
        i += 1;
    }
    
    Ok(Template::List(templates))
}

/// Converts a syntax-rules transformer to a macro transformer.
/// 
/// Since MacroTransformer supports only single pattern/template pairs,
/// we use the first rule and rely on expand_syntax_rules for proper
/// multi-rule handling.
pub fn syntax_rules_to_macro_transformer(
    syntax_rules: SyntaxRulesTransformer,
) -> MacroTransformer {
    let primary_rule = syntax_rules.rules.first().cloned().unwrap_or(SyntaxRule {
        pattern: Pattern::Wildcard,
        template: Template::Nil,
    });
    
    MacroTransformer {
        pattern: primary_rule.pattern,
        template: primary_rule.template,
        definition_env: syntax_rules.definition_env,
        name: syntax_rules.name,
        source: None,
    }
}

/// Expands a macro using syntax-rules semantics.
pub fn expand_syntax_rules(
    transformer: &SyntaxRulesTransformer,
    input: &Spanned<Expr>,
) -> Result<Spanned<Expr>> {
    // Try each rule in order until one matches
    for rule in &transformer.rules {
        if let Ok(bindings) = rule.pattern.match_expr(input) {
            // Pattern matched, expand template
            let expanded = rule.template.expand(&bindings, input.span)?;
            return Ok(expanded);
        }
    }
    
    Err(Box::new(Error::macro_error(
        "No pattern matched in syntax-rules".to_string(),
        input.span,
    )))
}

/// Validates that a pattern is well-formed for syntax-rules.
pub fn validate_pattern(pattern: &Pattern, literals: &[String]) -> Result<()> {
    let literal_set: HashSet<_> = literals.iter().collect();
    validate_pattern_inner(pattern, &literal_set, &mut HashSet::new())
}

/// Internal pattern validation with context.
fn validate_pattern_inner(
    pattern: &Pattern,
    literals: &HashSet<&String>,
    bound_vars: &mut HashSet<String>,
) -> Result<()> {
    match pattern {
        Pattern::Variable(name) => {
            if literals.contains(&name) {
                return Err(Box::new(Error::macro_error(
                    format!("Variable {name} conflicts with literal"),
                    crate::diagnostics::Span::new(0, 0),
                )));
            }
            if bound_vars.contains(name) {
                return Err(Box::new(Error::macro_error(
                    format!("Variable {name} bound multiple times"),
                    crate::diagnostics::Span::new(0, 0),
                )));
            }
            bound_vars.insert(name.clone());
            Ok(())
        }
        Pattern::List(patterns) => {
            for pat in patterns {
                validate_pattern_inner(pat, literals, bound_vars)?;
            }
            Ok(())
        }
        Pattern::Ellipsis { patterns, ellipsis_pattern, rest } => {
            // Pre-patterns
            for pat in patterns {
                validate_pattern_inner(pat, literals, bound_vars)?;
            }
            
            // Ellipsis pattern (in separate scope)
            let mut ellipsis_vars = HashSet::new();
            validate_pattern_inner(ellipsis_pattern, literals, &mut ellipsis_vars)?;
            
            // Check for conflicts between ellipsis and outer scope
            for var in &ellipsis_vars {
                if bound_vars.contains(var) {
                    return Err(Box::new(Error::macro_error(
                        format!("Ellipsis variable {var} conflicts with outer variable"),
                        crate::diagnostics::Span::new(0, 0),
                    )));
                }
            }
            
            // Rest pattern
            if let Some(rest_pat) = rest {
                validate_pattern_inner(rest_pat, literals, bound_vars)?;
            }
            
            Ok(())
        }
        Pattern::Pair { car, cdr } => {
            validate_pattern_inner(car, literals, bound_vars)?;
            validate_pattern_inner(cdr, literals, bound_vars)
        }
        Pattern::Or(alternatives) => {
            // All alternatives must bind the same variables
            let mut first_vars: Option<HashSet<String>> = None;
            for alt in alternatives {
                let mut alt_vars = HashSet::new();
                validate_pattern_inner(alt, literals, &mut alt_vars)?;
                
                if let Some(ref expected_vars) = first_vars {
                    if alt_vars != *expected_vars {
                        return Err(Box::new(Error::macro_error(
                            "Alternative patterns must bind same variables".to_string(),
                            crate::diagnostics::Span::new(0, 0),
                        )));
                    }
                } else {
                    first_vars = Some(alt_vars.clone());
                }
                
                // Add to bound vars
                bound_vars.extend(alt_vars);
            }
            Ok(())
        }
        Pattern::And(conjuncts) => {
            for conj in conjuncts {
                validate_pattern_inner(conj, literals, bound_vars)?;
            }
            Ok(())
        }
        Pattern::Not(sub_pattern) => {
            // Negative patterns can't bind variables
            let mut dummy_vars = HashSet::new();
            validate_pattern_inner(sub_pattern, literals, &mut dummy_vars)?;
            if !dummy_vars.is_empty() {
                return Err(Box::new(Error::macro_error(
                    "Negative patterns cannot bind variables".to_string(),
                    crate::diagnostics::Span::new(0, 0),
                )));
            }
            Ok(())
        }
        _ => Ok(()), // Literals, identifiers, etc. are always valid
    }
}

/// Validates that a template only uses variables bound by the pattern.
pub fn validate_template(
    template: &Template,
    pattern_vars: &HashSet<String>,
    ellipsis_vars: &HashSet<String>,
) -> Result<()> {
    match template {
        Template::Variable(name) => {
            if !pattern_vars.contains(name) && !ellipsis_vars.contains(name) {
                return Err(Box::new(Error::macro_error(
                    format!("Template variable {name} not bound by pattern"),
                    crate::diagnostics::Span::new(0, 0),
                )));
            }
            Ok(())
        }
        Template::List(templates) => {
            for tmpl in templates {
                validate_template(tmpl, pattern_vars, ellipsis_vars)?;
            }
            Ok(())
        }
        Template::Ellipsis { templates, ellipsis_template, rest } => {
            // Pre-templates
            for tmpl in templates {
                validate_template(tmpl, pattern_vars, ellipsis_vars)?;
            }
            
            // Ellipsis template must only use ellipsis variables
            validate_template(ellipsis_template, &HashSet::new(), ellipsis_vars)?;
            
            // Rest template
            if let Some(rest_tmpl) = rest {
                validate_template(rest_tmpl, pattern_vars, ellipsis_vars)?;
            }
            
            Ok(())
        }
        Template::Pair { car, cdr } => {
            validate_template(car, pattern_vars, ellipsis_vars)?;
            validate_template(cdr, pattern_vars, ellipsis_vars)
        }
        Template::Conditional { condition, then_branch, else_branch } => {
            validate_template(condition, pattern_vars, ellipsis_vars)?;
            validate_template(then_branch, pattern_vars, ellipsis_vars)?;
            if let Some(else_tmpl) = else_branch {
                validate_template(else_tmpl, pattern_vars, ellipsis_vars)?;
            }
            Ok(())
        }
        Template::Transform { argument, .. } => {
            validate_template(argument, pattern_vars, ellipsis_vars)
        }
        Template::Splice(name) => {
            if !ellipsis_vars.contains(name) {
                return Err(Box::new(Error::macro_error(
                    format!("Splice variable {name} not bound as ellipsis variable"),
                    crate::diagnostics::Span::new(0, 0),
                )));
            }
            Ok(())
        }
        _ => Ok(()), // Literals, identifiers, etc. are always valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;
    
    fn make_spanned<T>(value: T) -> Spanned<T> {
        Spanned::new(value, Span::new(0, 1))
    }
    
    #[test]
    fn test_parse_literals_list() {
        // Empty list
        let expr = make_spanned(Expr::List(vec![]));
        let literals = parse_literals_list(&expr).unwrap();
        assert!(literals.is_empty());
        
        // List with identifiers
        let expr = make_spanned(Expr::List(vec![
            make_spanned(Expr::Identifier("else".to_string())),
            make_spanned(Expr::Identifier("=>".to_string())),
        ]));
        let literals = parse_literals_list(&expr).unwrap();
        assert_eq!(literals, vec!["else", "=>"]);
    }
    
    #[test]
    fn test_parse_simple_pattern() {
        let literals = vec!["else".to_string()];
        
        // Variable pattern
        let expr = make_spanned(Expr::Identifier("x".to_string()));
        let pattern = parse_pattern(&expr, &literals).unwrap();
        assert!(matches!(pattern, Pattern::Variable(_)));
        
        // Literal pattern
        let expr = make_spanned(Expr::Identifier("else".to_string()));
        let pattern = parse_pattern(&expr, &literals).unwrap();
        assert!(matches!(pattern, Pattern::Identifier(_)));
        
        // Literal value
        let expr = make_spanned(Expr::Literal(crate::ast::Literal::Number(42.0)));
        let pattern = parse_pattern(&expr, &literals).unwrap();
        assert!(matches!(pattern, Pattern::Literal(_)));
    }
    
    #[test]
    fn test_parse_list_pattern() {
        let literals = vec![];
        
        // Simple list pattern
        let elements = vec![
            make_spanned(Expr::Identifier("if".to_string())),
            make_spanned(Expr::Identifier("test".to_string())),
            make_spanned(Expr::Identifier("then".to_string())),
        ];
        let pattern = parse_list_pattern(&elements, &literals).unwrap();
        
        match pattern {
            Pattern::List(patterns) => {
                assert_eq!(patterns.len(), 3);
                assert!(matches!(patterns[0], Pattern::Variable(_)));
            }
            _ => panic!("Expected list pattern"),
        }
    }
    
    #[test]
    fn test_parse_ellipsis_pattern() {
        let literals = vec![];
        
        // Pattern with ellipsis: (x y ...)
        let elements = vec![
            make_spanned(Expr::Identifier("x".to_string())),
            make_spanned(Expr::Identifier("y".to_string())),
            make_spanned(Expr::Identifier("...".to_string())),
        ];
        let pattern = parse_list_pattern(&elements, &literals).unwrap();
        
        match pattern {
            Pattern::Ellipsis { patterns, ellipsis_pattern, rest } => {
                assert_eq!(patterns.len(), 1);
                if let Pattern::Variable(_) = ellipsis_pattern.as_ref() {
                    // Expected pattern type
                } else {
                    panic!("Expected variable pattern");
                }
                assert!(rest.is_none());
            }
            _ => panic!("Expected ellipsis pattern"),
        }
    }
    
    #[test]
    fn test_validate_pattern() {
        let literals = vec!["else".to_string()];
        
        // Valid pattern
        let pattern = Pattern::List(vec![
            Pattern::Variable("x".to_string()),
            Pattern::Identifier("else".to_string()),
        ]);
        assert!(validate_pattern(&pattern, &literals).is_ok());
        
        // Invalid pattern (variable conflicts with literal)
        let pattern = Pattern::Variable("else".to_string());
        assert!(validate_pattern(&pattern, &literals).is_err());
    }
    
    #[test]
    fn test_validate_template() {
        let mut pattern_vars = HashSet::new();
        pattern_vars.insert("x".to_string());
        let ellipsis_vars = HashSet::new();
        
        // Valid template
        let template = Template::List(vec![
            Template::Identifier("if".to_string()),
            Template::Variable("x".to_string()),
        ]);
        assert!(validate_template(&template, &pattern_vars, &ellipsis_vars).is_ok());
        
        // Invalid template (unbound variable)
        let template = Template::Variable("y".to_string());
        assert!(validate_template(&template, &pattern_vars, &ellipsis_vars).is_err());
    }
}

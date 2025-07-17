//! Pattern and template definitions for syntax-rules and syntax-case

use crate::ast::Expr;
use crate::error::{LambdustError, Result};
use super::hygiene::{HygienicSymbol, ExpansionContext};
use std::collections::HashMap;

/// Type pattern for type-based matching
#[derive(Debug, Clone, PartialEq)]
pub enum TypePattern {
    /// Number type
    Number,
    /// String type
    String,
    /// Boolean type
    Boolean,
    /// Character type
    Character,
    /// Symbol type
    Symbol,
    /// List type
    List,
    /// Vector type
    Vector,
    /// Procedure type
    Procedure,
    /// Any type (equivalent to no type constraint)
    Any,
    /// Custom type predicate
    Custom(String),
}

/// Pattern for syntax-rules and syntax-case
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Literal pattern (must match exactly)
    Literal(String),
    /// Variable pattern (binds to any expression)
    Variable(String),
    /// List pattern
    List(Vec<Pattern>),
    /// Ellipsis pattern (matches zero or more)
    Ellipsis(Box<Pattern>),
    /// Dotted pattern
    Dotted(Vec<Pattern>, Box<Pattern>),
    /// Nested ellipsis pattern (SRFI 46 extension)
    NestedEllipsis(Box<Pattern>, usize), // pattern with nesting level
    /// Vector pattern (SRFI 46 extension)
    Vector(Vec<Pattern>),
    /// Hygienic pattern variable (syntax-case specific)
    HygienicVariable(HygienicSymbol),
    /// Syntax object pattern (syntax-case specific)
    SyntaxObject(Box<Pattern>),
    /// Any pattern - matches any syntax object
    Any,
    /// Conditional pattern with guard expression
    Conditional {
        /// The pattern to match
        pattern: Box<Pattern>,
        /// Guard expression that must evaluate to true
        guard: crate::ast::Expr,
    },
    /// Type guard pattern
    TypeGuard {
        /// The pattern to match
        pattern: Box<Pattern>,
        /// Expected type that must match
        expected_type: TypePattern,
    },
    /// AND pattern - all sub-patterns must match
    And(Vec<Pattern>),
    /// OR pattern - at least one sub-pattern must match
    Or(Vec<Pattern>),
    /// NOT pattern - pattern must not match
    Not(Box<Pattern>),
    /// Range pattern for numeric values
    Range {
        /// Start of the range (inclusive if present)
        start: Option<i64>,
        /// End of the range
        end: Option<i64>,
        /// Whether the end is inclusive
        inclusive: bool,
    },
    /// Regex pattern for string matching
    Regex(String),
}

/// Template for syntax-rules and syntax-case
#[derive(Debug, Clone, PartialEq)]
pub enum Template {
    /// Literal template
    Literal(String),
    /// Variable reference
    Variable(String),
    /// List template
    List(Vec<Template>),
    /// Ellipsis template (expands pattern variables)
    Ellipsis(Box<Template>),
    /// Dotted template
    Dotted(Vec<Template>, Box<Template>),
    /// Nested ellipsis template (SRFI 46 extension)
    NestedEllipsis(Box<Template>, usize), // template with nesting level
    /// Vector template (SRFI 46 extension)
    Vector(Vec<Template>),
    /// Hygienic variable reference (syntax-case specific)
    HygienicVariable(HygienicSymbol),
    /// Syntax object construction (syntax-case specific)
    SyntaxObject(Box<Template>),
    /// Conditional template expansion
    Conditional {
        /// Condition expression to evaluate
        condition: Expr,
        /// Template to use if condition is true
        then_template: Box<Template>,
        /// Optional template to use if condition is false
        else_template: Option<Box<Template>>,
    },
    /// Repeated template with separator
    Repeat {
        /// Template to repeat
        template: Box<Template>,
        /// Optional separator between repetitions
        separator: Option<String>,
        /// Minimum number of repetitions
        min_count: usize,
        /// Maximum number of repetitions
        max_count: Option<usize>,
    },
    /// Transform template (apply function to bound values)
    Transform {
        /// Template to transform
        template: Box<Template>,
        /// Function name to apply to bound values
        function: String,
    },
}

/// Syntax rule (pattern -> template)
///
/// Represents a single transformation rule in a syntax-rules macro definition.
/// Each rule consists of a pattern that matches input expressions and a template
/// that specifies how to transform the matched input.
#[derive(Debug, Clone)]
pub struct SyntaxRule {
    /// The pattern to match against input expressions
    pub pattern: Pattern,
    /// The template for generating the output expression
    pub template: Template,
}

/// Syntax-case clause
///
/// Represents a syntax-case pattern matching clause with optional guard condition.
/// This enables more sophisticated pattern matching with runtime conditions.
#[derive(Debug, Clone)]
pub struct SyntaxCaseClause {
    /// The pattern to match against
    pub pattern: Pattern,
    /// Optional guard expression (when condition)
    pub guard: Option<Expr>,
    /// The template or expression to generate
    pub body: SyntaxCaseBody,
}

/// Body of a syntax-case clause
#[derive(Debug, Clone)]
pub enum SyntaxCaseBody {
    /// Template expansion
    Template(Template),
    /// Direct expression
    Expression(Expr),
}

/// Pattern matching result
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct MatchResult {
    /// Variable bindings from pattern matching
    pub bindings: HashMap<String, BindingValue>,
    /// Hygienic symbol bindings
    pub hygienic_bindings: HashMap<HygienicSymbol, SyntaxObject>,
    /// Success flag
    pub success: bool,
}

/// Binding value in pattern matching
#[derive(Debug, Clone)]
pub enum BindingValue {
    /// Single expression
    Single(Expr),
    /// List of expressions (from ellipsis)
    List(Vec<Expr>),
    /// Syntax object
    SyntaxObject(SyntaxObject),
}

/// Syntax object representation
#[derive(Debug, Clone)]
pub struct SyntaxObject {
    /// The expression content
    pub expression: Expr,
    /// Lexical context for hygiene
    pub context: ExpansionContext,
    /// Source location information
    pub source_info: Option<SourceInfo>,
}

/// Source location information for syntax objects
#[derive(Debug, Clone)]
pub struct SourceInfo {
    /// File name
    pub file: String,
    /// Line number
    pub line: u32,
    /// Column number  
    pub column: u32,
}

/// Pattern matcher for syntax-case
#[derive(Debug, Clone)]
pub struct PatternMatcher {
    /// Literal identifiers that should be matched literally
    literals: Vec<String>,
}

impl PatternMatcher {
    /// Create a new pattern matcher
    #[must_use] pub fn new(literals: Vec<String>) -> Self {
        Self { literals }
    }

    /// Create a pattern matcher with advanced features enabled
    #[must_use] pub fn with_advanced_features(literals: Vec<String>) -> Self {
        Self { literals }
    }

    /// Match a pattern against an expression
    ///
    /// This method implements value borrow principles by taking references
    /// instead of moving values, and returns immutable match results.
    pub fn match_pattern(
        &self,
        pattern: &Pattern,
        expr: &Expr,
        context: &ExpansionContext,
    ) -> Result<MatchResult> {
        let mut bindings = HashMap::new();
        let mut hygienic_bindings = HashMap::new();

        let success = self.match_pattern_recursive(
            pattern,
            expr,
            context,
            &mut bindings,
            &mut hygienic_bindings,
        )?;

        Ok(MatchResult {
            bindings,
            hygienic_bindings,
            success,
        })
    }

    /// Recursive pattern matching implementation
    fn match_pattern_recursive(
        &self,
        pattern: &Pattern,
        expr: &Expr,
        context: &ExpansionContext,
        bindings: &mut HashMap<String, BindingValue>,
        hygienic_bindings: &mut HashMap<HygienicSymbol, SyntaxObject>,
    ) -> Result<bool> {
        match (pattern, expr) {
            // Literal patterns must match exactly
            (Pattern::Literal(lit), Expr::Variable(var)) => {
                Ok(lit == var && self.literals.contains(lit))
            }
            (Pattern::Literal(lit), Expr::Literal(expr_lit)) => {
                Ok(lit == &format!("{expr_lit:?}"))
            }

            // Variable patterns bind to any expression
            (Pattern::Variable(var), expr) => {
                if self.literals.contains(var) {
                    // Literal variables must match exactly
                    match expr {
                        Expr::Variable(expr_var) => Ok(var == expr_var),
                        _ => Ok(false),
                    }
                } else {
                    // Pattern variables bind to the expression
                    bindings.insert(
                        var.clone(),
                        BindingValue::Single(expr.clone()),
                    );
                    Ok(true)
                }
            }

            // Hygienic variable patterns
            (Pattern::HygienicVariable(hyg_sym), expr) => {
                let syntax_obj = SyntaxObject {
                    expression: expr.clone(),
                    context: context.clone(),
                    source_info: None,
                };
                hygienic_bindings.insert(hyg_sym.clone(), syntax_obj);
                Ok(true)
            }

            // Any pattern matches anything
            (Pattern::Any, _) => Ok(true),

            // Conditional patterns
            (Pattern::Conditional { pattern, guard }, expr) => {
                // First check if the base pattern matches
                if !self.match_pattern_recursive(
                    pattern,
                    expr,
                    context,
                    bindings,
                    hygienic_bindings,
                )? {
                    return Ok(false);
                }

                // Then evaluate the guard condition
                self.evaluate_guard_expression(guard, bindings)
            }

            // Type guard patterns
            (Pattern::TypeGuard { pattern, expected_type }, expr) => {
                // Check type first
                if !self.matches_type_pattern(expr, expected_type) {
                    return Ok(false);
                }

                // Then check the nested pattern
                self.match_pattern_recursive(
                    pattern,
                    expr,
                    context,
                    bindings,
                    hygienic_bindings,
                )
            }

            // AND patterns - all must match
            (Pattern::And(patterns), expr) => {
                for pattern in patterns {
                    if !self.match_pattern_recursive(
                        pattern,
                        expr,
                        context,
                        bindings,
                        hygienic_bindings,
                    )? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }

            // OR patterns - at least one must match
            (Pattern::Or(patterns), expr) => {
                for pattern in patterns {
                    // Try each pattern with temporary bindings
                    let mut temp_bindings = bindings.clone();
                    let mut temp_hygienic_bindings = hygienic_bindings.clone();
                    
                    if self.match_pattern_recursive(
                        pattern,
                        expr,
                        context,
                        &mut temp_bindings,
                        &mut temp_hygienic_bindings,
                    )? {
                        // Merge successful bindings
                        *bindings = temp_bindings;
                        *hygienic_bindings = temp_hygienic_bindings;
                        return Ok(true);
                    }
                }
                Ok(false)
            }

            // NOT patterns - pattern must not match
            (Pattern::Not(pattern), expr) => {
                let mut temp_bindings = HashMap::new();
                let mut temp_hygienic_bindings = HashMap::new();
                
                let matches = self.match_pattern_recursive(
                    pattern,
                    expr,
                    context,
                    &mut temp_bindings,
                    &mut temp_hygienic_bindings,
                )?;
                
                Ok(!matches)
            }

            // Range patterns for numbers
            (Pattern::Range { start, end, inclusive }, Expr::Literal(crate::ast::Literal::Number(num))) => {
                use crate::lexer::SchemeNumber;
                match num {
                    SchemeNumber::Integer(n) => {
                        self.check_numeric_range(*n, *start, *end, *inclusive)
                    }
                    SchemeNumber::Real(f) => {
                        self.check_numeric_range(*f as i64, *start, *end, *inclusive)
                    }
                    _ => Ok(false),
                }
            }

            // Regex patterns for strings
            (Pattern::Regex(regex_str), Expr::Literal(crate::ast::Literal::String(s))) => {
                self.match_regex(regex_str, s)
            }

            // List patterns
            (Pattern::List(patterns), Expr::List(exprs)) => {
                self.match_list_pattern(patterns, exprs, context, bindings, hygienic_bindings)
            }

            // Vector patterns
            (Pattern::Vector(patterns), Expr::Vector(exprs)) => {
                self.match_list_pattern(patterns, exprs, context, bindings, hygienic_bindings)
            }

            // Syntax object patterns
            (Pattern::SyntaxObject(inner_pattern), expr) => {
                self.match_pattern_recursive(
                    inner_pattern,
                    expr,
                    context,
                    bindings,
                    hygienic_bindings,
                )
            }

            // Ellipsis patterns require special handling
            (Pattern::Ellipsis(_), _) => {
                Err(LambdustError::syntax_error(
                    "Ellipsis patterns cannot be matched directly".to_string(),
                ))
            }

            // Mismatched types
            _ => Ok(false),
        }
    }

    /// Match list patterns with ellipsis support
    fn match_list_pattern(
        &self,
        patterns: &[Pattern],
        exprs: &[Expr],
        context: &ExpansionContext,
        bindings: &mut HashMap<String, BindingValue>,
        hygienic_bindings: &mut HashMap<HygienicSymbol, SyntaxObject>,
    ) -> Result<bool> {
        let mut pattern_idx = 0;
        let mut expr_idx = 0;

        while pattern_idx < patterns.len() && expr_idx < exprs.len() {
            match &patterns[pattern_idx] {
                Pattern::Ellipsis(sub_pattern) => {
                    // Match ellipsis pattern against remaining expressions
                    let mut ellipsis_matches = Vec::new();
                    
                    // Calculate how many expressions to consume
                    let remaining_patterns = patterns.len() - pattern_idx - 1;
                    let remaining_exprs = exprs.len() - expr_idx;
                    
                    if remaining_exprs < remaining_patterns {
                        return Ok(false);
                    }
                    
                    let ellipsis_count = remaining_exprs - remaining_patterns;
                    
                    // Match ellipsis pattern against expressions
                    for _ in 0..ellipsis_count {
                        if expr_idx >= exprs.len() {
                            break;
                        }
                        
                        let mut temp_bindings = HashMap::new();
                        let mut temp_hygienic_bindings = HashMap::new();
                        
                        if self.match_pattern_recursive(
                            sub_pattern,
                            &exprs[expr_idx],
                            context,
                            &mut temp_bindings,
                            &mut temp_hygienic_bindings,
                        )? {
                            ellipsis_matches.push(exprs[expr_idx].clone());
                            // Merge bindings appropriately
                            self.merge_ellipsis_bindings(
                                &temp_bindings,
                                bindings,
                            );
                        } else {
                            return Ok(false);
                        }
                        
                        expr_idx += 1;
                    }
                    
                    pattern_idx += 1;
                }
                pattern => {
                    if !self.match_pattern_recursive(
                        pattern,
                        &exprs[expr_idx],
                        context,
                        bindings,
                        hygienic_bindings,
                    )? {
                        return Ok(false);
                    }
                    pattern_idx += 1;
                    expr_idx += 1;
                }
            }
        }

        // Check if we've consumed all patterns and expressions appropriately
        Ok(pattern_idx == patterns.len() && expr_idx == exprs.len())
    }

    /// Merge ellipsis bindings into the main binding map
    fn merge_ellipsis_bindings(
        &self,
        temp_bindings: &HashMap<String, BindingValue>,
        bindings: &mut HashMap<String, BindingValue>,
    ) {
        for (var, value) in temp_bindings {
            match bindings.get_mut(var) {
                Some(BindingValue::List(ref mut list)) => {
                    match value {
                        BindingValue::Single(expr) => list.push(expr.clone()),
                        BindingValue::List(exprs) => list.extend_from_slice(exprs),
                        BindingValue::SyntaxObject(_) => {
                            // Handle syntax objects in ellipsis context
                            // For now, convert to expression
                        }
                    }
                }
                None => {
                    match value {
                        BindingValue::Single(expr) => {
                            bindings.insert(var.clone(), BindingValue::List(vec![expr.clone()]));
                        }
                        BindingValue::List(exprs) => {
                            bindings.insert(var.clone(), BindingValue::List(exprs.clone()));
                        }
                        BindingValue::SyntaxObject(obj) => {
                            bindings.insert(var.clone(), BindingValue::SyntaxObject(obj.clone()));
                        }
                    }
                }
                Some(_) => {
                    // Convert single binding to list
                    if let Some(existing) = bindings.remove(var) {
                        let mut new_list = match existing {
                            BindingValue::Single(expr) => vec![expr],
                            BindingValue::List(exprs) => exprs,
                            BindingValue::SyntaxObject(_) => vec![], // Handle appropriately
                        };
                        
                        match value {
                            BindingValue::Single(expr) => new_list.push(expr.clone()),
                            BindingValue::List(exprs) => new_list.extend_from_slice(exprs),
                            BindingValue::SyntaxObject(_) => {}
                        }
                        
                        bindings.insert(var.clone(), BindingValue::List(new_list));
                    }
                }
            }
        }
    }

    /// Evaluate guard expression in pattern context
    fn evaluate_guard_expression(
        &self,
        guard: &Expr,
        bindings: &HashMap<String, BindingValue>,
    ) -> Result<bool> {
        // Substitute variables in guard expression
        let substituted = self.substitute_variables_in_expr(guard, bindings)?;

        // Simple evaluation for common cases
        match substituted {
            Expr::Literal(crate::ast::Literal::Boolean(b)) => Ok(b),
            Expr::Variable(var) if var == "#t" => Ok(true),
            Expr::Variable(var) if var == "#f" => Ok(false),
            // For complex expressions, assume true for development
            // In production, this would use a full evaluator
            _ => Ok(true),
        }
    }

    /// Check if expression matches type pattern
    fn matches_type_pattern(&self, expr: &Expr, type_pattern: &TypePattern) -> bool {
        match (expr, type_pattern) {
            (_, TypePattern::Any) => true,
            (Expr::Literal(crate::ast::Literal::Number(_)), TypePattern::Number) => true,
            (Expr::Literal(crate::ast::Literal::String(_)), TypePattern::String) => true,
            (Expr::Literal(crate::ast::Literal::Boolean(_)), TypePattern::Boolean) => true,
            (Expr::Literal(crate::ast::Literal::Character(_)), TypePattern::Character) => true,
            (Expr::Variable(_), TypePattern::Symbol) => true,
            (Expr::List(_), TypePattern::List) => true,
            (Expr::Vector(_), TypePattern::Vector) => true,
            // Custom type predicates would require runtime evaluation
            (_, TypePattern::Custom(_)) => false,
            _ => false,
        }
    }

    /// Check numeric range
    fn check_numeric_range(
        &self,
        value: i64,
        start: Option<i64>,
        end: Option<i64>,
        inclusive: bool,
    ) -> Result<bool> {
        let in_start_range = match start {
            Some(s) => if inclusive { value >= s } else { value > s },
            None => true,
        };

        let in_end_range = match end {
            Some(e) => if inclusive { value <= e } else { value < e },
            None => true,
        };

        Ok(in_start_range && in_end_range)
    }

    /// Match regex pattern
    fn match_regex(&self, regex_str: &str, string: &str) -> Result<bool> {
        // Simplified regex matching - in production would use regex crate
        match regex_str {
            ".*" => Ok(true),
            "^[a-zA-Z]+$" => Ok(string.chars().all(char::is_alphabetic)),
            "^[0-9]+$" => Ok(string.chars().all(|c| c.is_ascii_digit())),
            _ => {
                // Basic literal matching for now
                Ok(string.contains(regex_str))
            }
        }
    }

    /// Substitute variables in expression for guard evaluation
    fn substitute_variables_in_expr(
        &self,
        expr: &Expr,
        bindings: &HashMap<String, BindingValue>,
    ) -> Result<Expr> {
        match expr {
            Expr::Variable(var) => {
                if let Some(binding) = bindings.get(var) {
                    match binding {
                        BindingValue::Single(expr) => Ok(expr.clone()),
                        BindingValue::List(_) => {
                            Err(LambdustError::syntax_error(format!(
                                "Variable {var} bound to list in guard expression"
                            )))
                        }
                        BindingValue::SyntaxObject(obj) => Ok(obj.expression.clone()),
                    }
                } else {
                    Ok(expr.clone())
                }
            }
            Expr::List(exprs) => {
                let substituted: Result<Vec<_>> = exprs
                    .iter()
                    .map(|e| self.substitute_variables_in_expr(e, bindings))
                    .collect();
                Ok(Expr::List(substituted?))
            }
            _ => Ok(expr.clone()),
        }
    }
}

impl SyntaxObject {
    /// Create a new syntax object
    #[must_use] pub fn new(expression: Expr, context: ExpansionContext) -> Self {
        Self {
            expression,
            context,
            source_info: None,
        }
    }

    /// Create a syntax object with source information
    #[must_use] pub fn with_source_info(
        expression: Expr,
        context: ExpansionContext,
        source_info: SourceInfo,
    ) -> Self {
        Self {
            expression,
            context,
            source_info: Some(source_info),
        }
    }
}

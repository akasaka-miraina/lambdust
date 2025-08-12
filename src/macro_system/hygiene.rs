//! Hygienic macro expansion system.
//!
//! This module implements the hygiene preservation mechanism for Lambdust macros,
//! ensuring that macro-introduced identifiers don't accidentally capture or be
//! captured by identifiers in the macro use context. This follows the R7RS
//! standard for hygienic macro expansion.

use crate::ast::{CaseLambdaClause, Expr, Formals, Binding, CondClause, CaseClause, GuardClause, KeywordParam, ParameterBinding};
use crate::diagnostics::{Result, Spanned};
use crate::eval::Environment;
// use crate::utils::{intern_symbol, symbol_name, SymbolId};
use std::collections::{HashMap, HashSet};
// use std::rc::Rc;

/// A unique identifier for tracking macro expansion contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacroContext(u64);

impl MacroContext {
    /// Creates a new macro context.
    pub fn new(id: u64) -> Self {
        MacroContext(id)
    }
    
    /// Gets the context ID.
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Information about an identifier's hygiene properties.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentifierInfo {
    /// The original name of the identifier
    pub original_name: String,
    /// The hygienically renamed name (if any)
    pub renamed_name: Option<String>,
    /// The macro context where this identifier was introduced
    pub context: Option<MacroContext>,
    /// The lexical scope where this identifier was defined
    pub scope_id: Option<u64>,
    /// Whether this identifier is from a syntax template
    pub is_syntax: bool,
}

impl IdentifierInfo {
    /// Creates identifier info for a regular identifier.
    pub fn regular(name: String) -> Self {
        Self {
            original_name: name,
            renamed_name: None,
            context: None,
            scope_id: None,
            is_syntax: false,
        }
    }
    
    /// Creates identifier info for a macro-introduced identifier.
    pub fn macro_introduced(name: String, context: MacroContext, scope_id: u64) -> Self {
        Self {
            original_name: name.clone(),
            renamed_name: Some(format!("{name}#{}", context.id())),
            context: Some(context),
            scope_id: Some(scope_id),
            is_syntax: false,
        }
    }
    
    /// Creates identifier info for a syntax identifier.
    pub fn syntax(name: String, context: MacroContext) -> Self {
        Self {
            original_name: name,
            renamed_name: None,
            context: Some(context),
            scope_id: None,
            is_syntax: true,
        }
    }
    
    /// Gets the effective name of this identifier.
    pub fn effective_name(&self) -> &str {
        self.renamed_name.as_ref().unwrap_or(&self.original_name)
    }
}

/// Tracks hygiene information during macro expansion.
#[derive(Debug, Clone)]
pub struct HygieneContext {
    /// Current macro context
    current_context: Option<MacroContext>,
    /// Mapping from original names to hygiene info
    identifier_info: HashMap<String, IdentifierInfo>,
    /// Set of reserved names that should not be renamed
    reserved_names: HashSet<String>,
    /// Counter for generating unique scope IDs
    scope_counter: u64,
    /// Current scope ID
    current_scope: u64,
}

impl HygieneContext {
    /// Creates a new hygiene context.
    pub fn new() -> Self {
        let mut reserved_names = HashSet::new();
        
        // Add special forms that should never be renamed
        reserved_names.insert("quote".to_string());
        reserved_names.insert("lambda".to_string());
        reserved_names.insert("if".to_string());
        reserved_names.insert("define".to_string());
        reserved_names.insert("set!".to_string());
        reserved_names.insert("define-syntax".to_string());
        reserved_names.insert("call-with-current-continuation".to_string());
        reserved_names.insert("call/cc".to_string());
        reserved_names.insert("primitive".to_string());
        reserved_names.insert("::".to_string());
        
        Self {
            current_context: None,
            identifier_info: HashMap::new(),
            reserved_names,
            scope_counter: 0,
            current_scope: 0,
        }
    }
    
    /// Enters a new macro expansion context.
    pub fn enter_macro_context(&mut self, context: MacroContext) -> MacroContext {
        let old_context = self.current_context;
        self.current_context = Some(context);
        old_context.unwrap_or(context)
    }
    
    /// Exits the current macro expansion context.
    pub fn exit_macro_context(&mut self, previous: MacroContext) {
        self.current_context = Some(previous);
    }
    
    /// Enters a new lexical scope.
    pub fn enter_scope(&mut self) -> u64 {
        self.scope_counter += 1;
        let old_scope = self.current_scope;
        self.current_scope = self.scope_counter;
        old_scope
    }
    
    /// Exits the current lexical scope.
    pub fn exit_scope(&mut self, previous: u64) {
        self.current_scope = previous;
    }
    
    /// Renames identifiers in an expression to preserve hygiene.
    pub fn rename_identifiers(
        &mut self,
        expr: Spanned<Expr>,
        _definition_env: &Environment,
    ) -> Result<Spanned<Expr>> {
        self.rename_expr(expr)
    }
    
    /// Renames identifiers in an expression.
    fn rename_expr(&mut self, expr: Spanned<Expr>) -> Result<Spanned<Expr>> {
        let renamed_inner = match expr.inner {
            Expr::Identifier(name) => {
                Expr::Identifier(self.rename_identifier(&name))
            }
            Expr::Symbol(name) => {
                Expr::Symbol(self.rename_identifier(&name))
            }
            Expr::List(elements) => {
                let renamed_elements = elements.into_iter()
                    .map(|e| self.rename_expr(e))
                    .collect::<Result<Vec<_>>>()?;
                Expr::List(renamed_elements)
            }
            
            Expr::Lambda { formals, metadata, body } => {
                let old_scope = self.enter_scope();
                let renamed_formals = self.rename_formals(formals)?;
                let renamed_metadata = self.rename_metadata(metadata)?;
                let renamed_body = self.rename_body(body)?;
                self.exit_scope(old_scope);
                
                Expr::Lambda {
                    formals: renamed_formals,
                    metadata: renamed_metadata,
                    body: renamed_body,
                }
            }
            
            Expr::CaseLambda { clauses, metadata } => {
                let renamed_metadata = self.rename_metadata(metadata)?;
                let mut renamed_clauses = Vec::new();
                
                for clause in clauses {
                    let old_scope = self.enter_scope();
                    let renamed_formals = self.rename_formals(clause.formals.clone())?;
                    let renamed_body = self.rename_body(clause.body.clone())?;
                    self.exit_scope(old_scope);
                    
                    renamed_clauses.push(CaseLambdaClause {
                        formals: renamed_formals,
                        body: renamed_body,
                    });
                }
                
                Expr::CaseLambda {
                    clauses: renamed_clauses,
                    metadata: renamed_metadata,
                }
            }
            
            Expr::If { test, consequent, alternative } => {
                let renamed_test = self.rename_expr(*test)?;
                let renamed_consequent = self.rename_expr(*consequent)?;
                let renamed_alternative = if let Some(alt) = alternative {
                    Some(Box::new(self.rename_expr(*alt)?))
                } else {
                    None
                };
                
                Expr::If {
                    test: Box::new(renamed_test),
                    consequent: Box::new(renamed_consequent),
                    alternative: renamed_alternative,
                }
            }
            
            Expr::Define { name, value, metadata } => {
                let renamed_name = self.rename_identifier(&name);
                let renamed_value = self.rename_expr(*value)?;
                let renamed_metadata = self.rename_metadata(metadata)?;
                
                Expr::Define {
                    name: renamed_name,
                    value: Box::new(renamed_value),
                    metadata: renamed_metadata,
                }
            }
            
            Expr::Set { name, value } => {
                let renamed_name = self.rename_identifier(&name);
                let renamed_value = self.rename_expr(*value)?;
                
                Expr::Set {
                    name: renamed_name,
                    value: Box::new(renamed_value),
                }
            }
            
            Expr::DefineSyntax { name, transformer } => {
                let renamed_name = self.rename_identifier(&name);
                let renamed_transformer = self.rename_expr(*transformer)?;
                
                Expr::DefineSyntax {
                    name: renamed_name,
                    transformer: Box::new(renamed_transformer),
                }
            }
            
            Expr::CallCC(proc) => {
                let renamed_proc = self.rename_expr(*proc)?;
                Expr::CallCC(Box::new(renamed_proc))
            }
            
            Expr::Primitive { name, args } => {
                let renamed_args = self.rename_args(args)?;
                Expr::Primitive {
                    name, // Primitive names are not renamed
                    args: renamed_args,
                }
            }
            
            Expr::TypeAnnotation { expr: inner_expr, type_expr } => {
                let renamed_expr = self.rename_expr(*inner_expr)?;
                let renamed_type = self.rename_expr(*type_expr)?;
                
                Expr::TypeAnnotation {
                    expr: Box::new(renamed_expr),
                    type_expr: Box::new(renamed_type),
                }
            }
            
            Expr::Application { operator, operands } => {
                let renamed_operator = self.rename_expr(*operator)?;
                let renamed_operands = self.rename_args(operands)?;
                
                Expr::Application {
                    operator: Box::new(renamed_operator),
                    operands: renamed_operands,
                }
            }
            
            Expr::Pair { car, cdr } => {
                let renamed_car = self.rename_expr(*car)?;
                let renamed_cdr = self.rename_expr(*cdr)?;
                
                Expr::Pair {
                    car: Box::new(renamed_car),
                    cdr: Box::new(renamed_cdr),
                }
            }
            
            Expr::Begin(exprs) => {
                let renamed_exprs = self.rename_body(exprs)?;
                Expr::Begin(renamed_exprs)
            }
            
            Expr::Let { bindings, body } => {
                let old_scope = self.enter_scope();
                let renamed_bindings = self.rename_bindings(bindings)?;
                let renamed_body = self.rename_body(body)?;
                self.exit_scope(old_scope);
                
                Expr::Let {
                    bindings: renamed_bindings,
                    body: renamed_body,
                }
            }
            
            Expr::LetStar { bindings, body } => {
                let old_scope = self.enter_scope();
                let renamed_bindings = self.rename_bindings(bindings)?;
                let renamed_body = self.rename_body(body)?;
                self.exit_scope(old_scope);
                
                Expr::LetStar {
                    bindings: renamed_bindings,
                    body: renamed_body,
                }
            }
            
            Expr::LetRec { bindings, body } => {
                let old_scope = self.enter_scope();
                let renamed_bindings = self.rename_bindings(bindings)?;
                let renamed_body = self.rename_body(body)?;
                self.exit_scope(old_scope);
                
                Expr::LetRec {
                    bindings: renamed_bindings,
                    body: renamed_body,
                }
            }
            
            Expr::Cond(clauses) => {
                let renamed_clauses = self.rename_cond_clauses(clauses)?;
                Expr::Cond(renamed_clauses)
            }
            
            Expr::Case { expr: case_expr, clauses } => {
                let renamed_expr = self.rename_expr(*case_expr)?;
                let renamed_clauses = self.rename_case_clauses(clauses)?;
                
                Expr::Case {
                    expr: Box::new(renamed_expr),
                    clauses: renamed_clauses,
                }
            }
            
            Expr::And(exprs) => {
                let renamed_exprs = self.rename_body(exprs)?;
                Expr::And(renamed_exprs)
            }
            
            Expr::Or(exprs) => {
                let renamed_exprs = self.rename_body(exprs)?;
                Expr::Or(renamed_exprs)
            }
            
            Expr::When { test, body } => {
                let renamed_test = self.rename_expr(*test)?;
                let renamed_body = self.rename_body(body)?;
                
                Expr::When {
                    test: Box::new(renamed_test),
                    body: renamed_body,
                }
            }
            
            Expr::Unless { test, body } => {
                let renamed_test = self.rename_expr(*test)?;
                let renamed_body = self.rename_body(body)?;
                
                Expr::Unless {
                    test: Box::new(renamed_test),
                    body: renamed_body,
                }
            }
            
            Expr::Guard { variable, clauses, body } => {
                // Don't rename the exception variable - it's bound in the handler environment
                let renamed_clauses = clauses.into_iter().map(|clause| {
                    let renamed_test = self.rename_expr(clause.test)?;
                    let renamed_body = self.rename_body(clause.body)?;
                    let renamed_arrow = if let Some(arrow) = clause.arrow {
                        Some(self.rename_expr(arrow)?)
                    } else {
                        None
                    };
                    Ok(GuardClause {
                        test: renamed_test,
                        body: renamed_body,
                        arrow: renamed_arrow,
                    })
                }).collect::<Result<Vec<_>>>()?;
                
                let renamed_body = self.rename_body(body)?;
                
                Expr::Guard {
                    variable, // Don't rename the variable
                    clauses: renamed_clauses,
                    body: renamed_body,
                }
            }
            
            Expr::Parameterize { bindings, body } => {
                let renamed_bindings = bindings.into_iter()
                    .map(|b| -> Result<ParameterBinding> {
                        Ok(ParameterBinding {
                            parameter: self.rename_expr(b.parameter)?,
                            value: self.rename_expr(b.value)?,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let renamed_body = body.into_iter()
                    .map(|e| self.rename_expr(e))
                    .collect::<Result<Vec<_>>>()?;
                Expr::Parameterize {
                    bindings: renamed_bindings,
                    body: renamed_body,
                }
            }
            
            Expr::Import { import_specs } => {
                let renamed_specs = import_specs.into_iter()
                    .map(|spec| self.rename_expr(spec))
                    .collect::<Result<Vec<_>>>()?;
                Expr::Import { import_specs: renamed_specs }
            }
            
            Expr::DefineLibrary { name, imports, exports, body } => {
                let renamed_imports = imports.into_iter()
                    .map(|spec| self.rename_expr(spec))
                    .collect::<Result<Vec<_>>>()?;
                let renamed_exports = exports.into_iter()
                    .map(|spec| self.rename_expr(spec))
                    .collect::<Result<Vec<_>>>()?;
                let renamed_body = body.into_iter()
                    .map(|expr| self.rename_expr(expr))
                    .collect::<Result<Vec<_>>>()?;
                Expr::DefineLibrary { 
                    name: name.clone(),
                    imports: renamed_imports, 
                    exports: renamed_exports, 
                    body: renamed_body 
                }
            }
            
            // Syntax rules handling
            Expr::SyntaxRules { literals, rules } => {
                // For syntax-rules, we need to be careful about hygiene
                // Patterns and templates should preserve their structure but rename bound identifiers
                let renamed_rules = rules.iter()
                    .map(|(pattern, template)| {
                        // For now, don't rename within patterns/templates as they have special semantics
                        Ok((pattern.clone(), template.clone()))
                    })
                    .collect::<Result<Vec<_>>>()?;
                
                Expr::SyntaxRules {
                    literals: literals.clone(), // Keep literals unchanged
                    rules: renamed_rules,
                }
            }

            // Handle quasiquote and related forms
            Expr::Quote(inner_expr) => {
                // Don't rename inside quoted expressions
                Expr::Quote(inner_expr)
            }
            
            Expr::Quasiquote(inner_expr) => {
                // Recursively rename in quasiquote, but preserve unquote contexts
                let renamed_inner = self.rename_expr(*inner_expr)?;
                Expr::Quasiquote(Box::new(renamed_inner))
            }
            
            Expr::Unquote(inner_expr) => {
                // Rename inside unquote expressions
                let renamed_inner = self.rename_expr(*inner_expr)?;
                Expr::Unquote(Box::new(renamed_inner))
            }
            
            Expr::UnquoteSplicing(inner_expr) => {
                // Rename inside unquote-splicing expressions
                let renamed_inner = self.rename_expr(*inner_expr)?;
                Expr::UnquoteSplicing(Box::new(renamed_inner))
            }

            // These don't contain identifiers to rename
            Expr::Literal(_) | Expr::Keyword(_) => expr.inner,
        };
        
        Ok(Spanned::new(renamed_inner, expr.span))
    }
    
    /// Renames an identifier according to hygiene rules.
    fn rename_identifier(&mut self, name: &str) -> String {
        // Don't rename reserved names
        if self.reserved_names.contains(name) {
            return name.to_string();
        }
        
        // Check if we already have hygiene info for this identifier
        if let Some(info) = self.identifier_info.get(name) {
            return info.effective_name().to_string();
        }
        
        // If we're in a macro context, create a hygienically renamed identifier
        if let Some(context) = self.current_context {
            let info = IdentifierInfo::macro_introduced(
                name.to_string(),
                context,
                self.current_scope,
            );
            let effective_name = info.effective_name().to_string();
            self.identifier_info.insert(name.to_string(), info);
            effective_name
        } else {
            // Not in a macro context, use the original name
            let info = IdentifierInfo::regular(name.to_string());
            self.identifier_info.insert(name.to_string(), info);
            name.to_string()
        }
    }
    
    /// Renames formal parameters.
    fn rename_formals(&mut self, formals: Formals) -> Result<Formals> {
        match formals {
            Formals::Fixed(params) => {
                let renamed_params = params.into_iter()
                    .map(|p| self.rename_identifier(&p))
                    .collect();
                Ok(Formals::Fixed(renamed_params))
            }
            
            Formals::Variable(param) => {
                let renamed_param = self.rename_identifier(&param);
                Ok(Formals::Variable(renamed_param))
            }
            
            Formals::Mixed { fixed, rest } => {
                let renamed_fixed = fixed.into_iter()
                    .map(|p| self.rename_identifier(&p))
                    .collect();
                let renamed_rest = self.rename_identifier(&rest);
                Ok(Formals::Mixed {
                    fixed: renamed_fixed,
                    rest: renamed_rest,
                })
            }
            
            Formals::Keyword { fixed, rest, keywords } => {
                let renamed_fixed = fixed.into_iter()
                    .map(|p| self.rename_identifier(&p))
                    .collect();
                let renamed_rest = rest.map(|r| self.rename_identifier(&r));
                let renamed_keywords = keywords.into_iter()
                    .map(|kw| self.rename_keyword_param(kw))
                    .collect::<Result<Vec<_>>>()?;
                
                Ok(Formals::Keyword {
                    fixed: renamed_fixed,
                    rest: renamed_rest,
                    keywords: renamed_keywords,
                })
            }
        }
    }
    
    /// Renames a keyword parameter.
    fn rename_keyword_param(&mut self, param: KeywordParam) -> Result<KeywordParam> {
        let renamed_name = self.rename_identifier(&param.name);
        let renamed_default = if let Some(default) = param.default {
            Some(self.rename_expr(default)?)
        } else {
            None
        };
        
        Ok(KeywordParam {
            name: renamed_name,
            default: renamed_default,
        })
    }
    
    /// Renames metadata expressions.
    fn rename_metadata(
        &mut self,
        metadata: HashMap<String, Spanned<Expr>>,
    ) -> Result<HashMap<String, Spanned<Expr>>> {
        let mut renamed = HashMap::new();
        for (key, expr) in metadata {
            renamed.insert(key, self.rename_expr(expr)?);
        }
        Ok(renamed)
    }
    
    /// Renames a sequence of expressions.
    fn rename_body(&mut self, body: Vec<Spanned<Expr>>) -> Result<Vec<Spanned<Expr>>> {
        body.into_iter()
            .map(|expr| self.rename_expr(expr))
            .collect()
    }
    
    /// Renames argument expressions.
    fn rename_args(&mut self, args: Vec<Spanned<Expr>>) -> Result<Vec<Spanned<Expr>>> {
        args.into_iter()
            .map(|arg| self.rename_expr(arg))
            .collect()
    }
    
    /// Renames variable bindings.
    fn rename_bindings(&mut self, bindings: Vec<Binding>) -> Result<Vec<Binding>> {
        bindings.into_iter()
            .map(|binding| {
                let renamed_name = self.rename_identifier(&binding.name);
                let renamed_value = self.rename_expr(binding.value)?;
                Ok(Binding {
                    name: renamed_name,
                    value: renamed_value,
                })
            })
            .collect()
    }
    
    /// Renames cond clauses.
    fn rename_cond_clauses(&mut self, clauses: Vec<CondClause>) -> Result<Vec<CondClause>> {
        clauses.into_iter()
            .map(|clause| {
                let renamed_test = self.rename_expr(clause.test)?;
                let renamed_body = self.rename_body(clause.body)?;
                Ok(CondClause {
                    test: renamed_test,
                    body: renamed_body,
                })
            })
            .collect()
    }
    
    /// Renames case clauses.
    fn rename_case_clauses(&mut self, clauses: Vec<CaseClause>) -> Result<Vec<CaseClause>> {
        clauses.into_iter()
            .map(|clause| {
                let renamed_values = self.rename_body(clause.values)?;
                let renamed_body = self.rename_body(clause.body)?;
                Ok(CaseClause {
                    values: renamed_values,
                    body: renamed_body,
                })
            })
            .collect()
    }
    
    /// Checks if two identifiers are the same after hygiene transformation.
    pub fn identifiers_equal(&self, name1: &str, name2: &str) -> bool {
        // Simple case: if the names are identical, they're equal
        if name1 == name2 {
            return true;
        }
        
        // Check if both names have the same pattern (both renamed or both original)
        let is_renamed1 = name1.contains('#');
        let is_renamed2 = name2.contains('#');
        
        // If one is renamed and the other isn't, they can't be equal
        if is_renamed1 != is_renamed2 {
            return false;
        }
        
        // If both are renamed, check if they have the same original and context
        if is_renamed1 && is_renamed2 {
            let original1 = self.find_original_identifier(name1);
            let original2 = self.find_original_identifier(name2);
            
            if original1 != original2 {
                return false;
            }
            
            let info1 = self.identifier_info.get(&original1);
            let info2 = self.identifier_info.get(&original2);
            
            match (info1, info2) {
                (Some(i1), Some(i2)) => {
                    i1.context == i2.context
                }
                _ => false,
            }
        } else {
            // Both are original names, simple comparison (already handled above)
            false
        }
    }
    
    /// Finds the original identifier for a potentially renamed identifier.
    fn find_original_identifier(&self, name: &str) -> String {
        // Check if this name ends with #<number> pattern
        if let Some(pos) = name.rfind('#') {
            let (potential_original, suffix) = name.split_at(pos);
            // Check if suffix is all digits
            if suffix[1..].chars().all(|c| c.is_ascii_digit()) {
                // Check if we have hygiene info for the potential original
                if self.identifier_info.contains_key(potential_original) {
                    return potential_original.to_string();
                }
            }
        }
        name.to_string()
    }
    
    /// Gets hygiene information for an identifier.
    pub fn get_identifier_info(&self, name: &str) -> Option<&IdentifierInfo> {
        self.identifier_info.get(name)
    }
    
    /// Marks an identifier as syntax (from a syntax template).
    pub fn mark_syntax(&mut self, name: &str) {
        if let Some(context) = self.current_context {
            let info = IdentifierInfo::syntax(name.to_string(), context);
            self.identifier_info.insert(name.to_string(), info);
        }
    }
}

impl Default for HygieneContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;
    use super::super::next_hygiene_id;
    
    fn make_spanned<T>(value: T) -> Spanned<T> {
        Spanned::new(value, Span::new(0, 1))
    }
    
    #[test]
    fn test_hygiene_context_creation() {
        let ctx = HygieneContext::new();
        assert!(ctx.reserved_names.contains("lambda"));
        assert!(ctx.reserved_names.contains("if"));
        assert!(ctx.reserved_names.contains("define"));
    }
    
    #[test]
    fn test_identifier_renaming() {
        let mut ctx = HygieneContext::new();
        let macro_ctx = MacroContext::new(next_hygiene_id());
        
        // Outside macro context
        let name1 = ctx.rename_identifier("foo");
        assert_eq!(name1, "foo");
        
        // Inside macro context
        ctx.enter_macro_context(macro_ctx);
        let name2 = ctx.rename_identifier("bar");
        assert!(name2.starts_with("bar#"));
        assert_ne!(name2, "bar");
    }
    
    #[test]
    fn test_reserved_names_not_renamed() {
        let mut ctx = HygieneContext::new();
        let macro_ctx = MacroContext::new(next_hygiene_id());
        ctx.enter_macro_context(macro_ctx);
        
        let lambda_name = ctx.rename_identifier("lambda");
        assert_eq!(lambda_name, "lambda");
        
        let if_name = ctx.rename_identifier("if");
        assert_eq!(if_name, "if");
    }
    
    #[test]
    fn test_identifier_equality() {
        let mut ctx = HygieneContext::new();
        let macro_ctx = MacroContext::new(next_hygiene_id());
        
        ctx.enter_macro_context(macro_ctx);
        let renamed1 = ctx.rename_identifier("foo");
        let renamed2 = ctx.rename_identifier("foo");
        
        assert!(ctx.identifiers_equal(&renamed1, &renamed2));
        assert!(!ctx.identifiers_equal(&renamed1, "foo"));
    }
    
    #[test]
    fn test_lambda_renaming() {
        let mut ctx = HygieneContext::new();
        let macro_ctx = MacroContext::new(next_hygiene_id());
        ctx.enter_macro_context(macro_ctx);
        
        let lambda_expr = make_spanned(Expr::Lambda {
            formals: Formals::Fixed(vec!["x".to_string(), "y".to_string()]),
            metadata: HashMap::new(),
            body: vec![make_spanned(Expr::Identifier("x".to_string()))],
        });
        
        let renamed = ctx.rename_expr(lambda_expr).unwrap();
        
        match renamed.inner {
            Expr::Lambda { formals, body, .. } => {
                match formals {
                    Formals::Fixed(params) => {
                        assert!(params[0].starts_with("x#"));
                        assert!(params[1].starts_with("y#"));
                    }
                    _ => panic!("Expected fixed formals"),
                }
                
                match &body[0].inner {
                    Expr::Identifier(name) => {
                        assert!(name.starts_with("x#"));
                    }
                    _ => panic!("Expected identifier in body"),
                }
            }
            _ => panic!("Expected lambda expression"),
        }
    }
}
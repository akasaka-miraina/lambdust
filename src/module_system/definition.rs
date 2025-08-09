//! Module definition parsing and compilation.
//!
//! Handles the parsing and compilation of module definitions from source code:
//! - (define-module (module-name) ...) syntax
//! - Import and export declarations
//! - Module body compilation
//! - Dependency extraction

use super::{Module, ModuleId, ModuleMetadata, ModuleSource, ImportSpec, ExportSpec};
use crate::diagnostics::{Error, Result, Span};
use crate::eval::Value;
use crate::ast::{Expr, Program};
use crate::diagnostics::Spanned;
use std::collections::HashMap;

/// Represents a module definition as parsed from source code.
#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    /// Module identifier
    pub id: ModuleId,
    /// Import specifications
    pub imports: Vec<ImportSpec>,
    /// Export specification
    pub export: ExportSpec,
    /// Module body expressions
    pub body: Vec<Spanned<Expr>>,
    /// Module metadata
    pub metadata: ModuleMetadata,
    /// Source span for error reporting
    pub span: Option<Span>,
}

/// Compiles a module definition into a runtime module.
pub fn compile_module_definition(
    definition: ModuleDefinition,
    import_resolver: &mut dyn FnMut(&ImportSpec) -> Result<HashMap<String, Value>>,
) -> Result<Module> {
    let mut module_env = HashMap::new();
    
    // Process imports
    for import_spec in &definition.imports {
        let import_bindings = import_resolver(import_spec)?;
        
        // Merge import bindings into module environment
        for (symbol, value) in import_bindings {
            if module_env.insert(symbol.clone(), value).is_some() {
                return Err(Box::new(Error::syntax_error(
                    format!("Duplicate import binding: {symbol}"),
                    definition.span,
                )));
            }
        }
    }
    
    // Evaluate module body in the environment
    let mut local_bindings = HashMap::new();
    evaluate_module_body(&definition.body, &module_env, &mut local_bindings)?;
    
    // Apply export specification
    let exports = super::export::apply_export_config(
        &local_bindings,
        &definition.export.config,
        &definition.export.symbols,
    )?;
    
    // Extract dependencies
    let dependencies = definition.imports.iter()
        .map(|import| import.module_id.clone())
        .collect();
    
    Ok(Module {
        id: definition.id,
        exports,
        dependencies,
        source: Some(ModuleSource::Source("module definition".to_string())),
        metadata: definition.metadata,
    })
}

/// Parses a module definition from a program AST.
pub fn parse_module_definition(program: &Program) -> Result<ModuleDefinition> {
    if program.expressions.is_empty() {
        return Err(Box::new(Error::syntax_error(
            "Empty module definition".to_string(),
            None,
        )));
    }
    
    // Look for define-module form
    let define_module_expr = &program.expressions[0];
    
    match &define_module_expr.inner {
        Expr::List(elements) if !elements.is_empty() => {
            match &elements[0].inner {
                Expr::Symbol(keyword) if keyword == "define-module" => {
                    parse_define_module_form(&elements[1..], define_module_expr.span)
                }
                _ => Err(Box::new(Error::syntax_error(
                    "Module must start with define-module".to_string(),
                    Some(define_module_expr.span),
                ))),
            }
        }
        _ => Err(Box::new(Error::syntax_error(
            "Module definition must be a list".to_string(),
            Some(define_module_expr.span),
        ))),
    }
}

/// Parses a define-module form.
fn parse_define_module_form(
    elements: &[Spanned<Expr>],
    span: Span,
) -> Result<ModuleDefinition> {
    if elements.is_empty() {
        return Err(Box::new(Error::syntax_error(
            "define-module requires module name".to_string(),
            Some(span),
        )));
    }
    
    // Parse module name
    let module_id = parse_module_name_expr(&elements[0])?;
    
    // Parse module body (imports, exports, definitions)
    let mut imports = Vec::new();
    let mut export = None;
    let mut body = Vec::new();
    let mut metadata = ModuleMetadata::default();
    
    for expr in &elements[1..] {
        match &expr.inner {
            Expr::List(form_elements) if !form_elements.is_empty() => {
                match &form_elements[0].inner {
                    Expr::Symbol(keyword) => {
                        match keyword.as_str() {
                            "import" => {
                                let import_spec = super::import::parse_import_spec(&form_elements[1..])?;
                                imports.push(import_spec);
                            }
                            "export" => {
                                if export.is_some() {
                                    return Err(Box::new(Error::syntax_error(
                                        "Multiple export declarations not allowed".to_string(),
                                        Some(expr.span),
                                    )));
                                }
                                export = Some(super::export::parse_export_spec(&form_elements[1..])?);
                            }
                            "metadata" => {
                                metadata = parse_metadata(&form_elements[1..], expr.span)?;
                            }
                            _ => {
                                // Regular definition - add to body
                                body.push(expr.clone());
                            }
                        }
                    }
                    _ => {
                        // Regular expression - add to body
                        body.push(expr.clone());
                    }
                }
            }
            _ => {
                // Regular expression - add to body
                body.push(expr.clone());
            }
        }
    }
    
    // Default export if none specified
    let export = export.unwrap_or_else(|| ExportSpec {
        symbols: Vec::new(),
        config: super::ExportConfig::Direct,
    });
    
    Ok(ModuleDefinition {
        id: module_id,
        imports,
        export,
        body,
        metadata,
        span: Some(span),
    })
}

/// Parses a module name expression.
fn parse_module_name_expr(expr: &Spanned<Expr>) -> Result<ModuleId> {
    match &expr.inner {
        Expr::List(elements) => {
            let mut parts = Vec::new();
            for element in elements {
                match &element.inner {
                    Expr::Symbol(symbol) => parts.push(symbol.clone()),
                    _ => return Err(Box::new(Error::syntax_error(
                        "Module name must contain only symbols".to_string(),
                        Some(element.span),
                    ))),
                }
            }
            
            if parts.is_empty() {
                return Err(Box::new(Error::syntax_error(
                    "Module name cannot be empty".to_string(),
                    Some(expr.span),
                )));
            }
            
            let module_name = format!("({})", parts.join(" "));
            super::name::parse_module_name(&module_name)
        }
        _ => Err(Box::new(Error::syntax_error(
            "Module name must be a list".to_string(),
            Some(expr.span),
        ))),
    }
}

/// Parses module metadata.
fn parse_metadata(elements: &[Spanned<Expr>], _span: Span) -> Result<ModuleMetadata> {
    let mut metadata = ModuleMetadata::default();
    
    for element in elements {
        match &element.inner {
            Expr::List(pair) if pair.len() == 2 => {
                let key = match &pair[0].inner {
                    Expr::Symbol(symbol) => symbol.clone(),
                    _ => return Err(Box::new(Error::syntax_error(
                        "Metadata key must be a symbol".to_string(),
                        Some(pair[0].span),
                    ))),
                };
                
                let value = match &pair[1].inner {
                    Expr::Literal(crate::ast::Literal::String(s)) => s.clone(),
                    Expr::Symbol(s) => s.clone(),
                    _ => return Err(Box::new(Error::syntax_error(
                        "Metadata value must be a string or symbol".to_string(),
                        Some(pair[1].span),
                    ))),
                };
                
                match key.as_str() {
                    "version" => metadata.version = Some(value),
                    "description" => metadata.description = Some(value),
                    "author" => metadata.authors.push(value),
                    _ => {
                        metadata.extra.insert(key, value);
                    }
                }
            }
            _ => return Err(Box::new(Error::syntax_error(
                "Metadata must be key-value pairs".to_string(),
                Some(element.span),
            ))),
        }
    }
    
    Ok(metadata)
}

/// Evaluates the module body to extract local bindings.
fn evaluate_module_body(
    body: &[Spanned<Expr>],
    _imports: &HashMap<String, Value>,
    local_bindings: &mut HashMap<String, Value>,
) -> Result<()> {
    // For now, we'll do a simple extraction of define forms
    // In a full implementation, this would use the actual evaluator
    
    for expr in body {
        match &expr.inner {
            Expr::List(elements) if !elements.is_empty() => {
                match &elements[0].inner {
                    Expr::Symbol(keyword) if keyword == "define" => {
                        extract_define_binding(&elements[1..], local_bindings, expr.span)?;
                    }
                    _ => {
                        // Other expressions would be evaluated here
                        // For now, we skip them
                    }
                }
            }
            _ => {
                // Non-list expressions would be evaluated here
            }
        }
    }
    
    Ok(())
}

/// Extracts a binding from a define form.
fn extract_define_binding(
    elements: &[Spanned<Expr>],
    bindings: &mut HashMap<String, Value>,
    span: Span,
) -> Result<()> {
    if elements.len() < 2 {
        return Err(Box::new(Error::syntax_error(
            "define requires at least 2 arguments".to_string(),
            Some(span),
        )));
    }
    
    match &elements[0].inner {
        Expr::Symbol(name) => {
            // Simple variable definition
            // For now, we'll use a placeholder value
            // In a full implementation, we'd evaluate the expression
            bindings.insert(name.clone(), Value::Unspecified);
            Ok(())
        }
        Expr::List(function_elements) if !function_elements.is_empty() => {
            // Function definition
            match &function_elements[0].inner {
                Expr::Symbol(name) => {
                    // For now, we'll use a placeholder value
                    // In a full implementation, we'd create a lambda
                    bindings.insert(name.clone(), Value::Unspecified);
                    Ok(())
                }
                _ => Err(Box::new(Error::syntax_error(
                    "Function name must be a symbol".to_string(),
                    Some(function_elements[0].span),
                ))),
            }
        }
        _ => Err(Box::new(Error::syntax_error(
            "define target must be a symbol or function definition".to_string(),
            Some(elements[0].span),
        ))),
    }
}

/// Validates a module definition for correctness.
pub fn validate_module_definition(definition: &ModuleDefinition) -> Result<()> {
    // Validate module ID
    super::name::validate_module_id(&definition.id)?;
    
    // Validate imports
    for import_spec in &definition.imports {
        super::import::validate_import_spec(import_spec)?;
    }
    
    // Validate exports (would need bindings to do full validation)
    // For now, we'll do basic validation
    if definition.export.symbols.is_empty() && 
       matches!(definition.export.config, super::ExportConfig::Direct) {
        // Empty export list might be intentional, so we'll allow it
    }
    
    // Check for circular imports (basic check)
    for import_spec in &definition.imports {
        if import_spec.module_id == definition.id {
            return Err(Box::new(Error::syntax_error(
                "Module cannot import itself".to_string(),
                definition.span,
            )));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::diagnostics::Spanned;
    use crate::diagnostics::Span;

    fn make_span() -> Span {
        Span::new(0, 0)
    }

    fn make_symbol(name: &str) -> Spanned<Expr> {
        Spanned {
            inner: Expr::Symbol(name.to_string()),
            span: make_span(),
        }
    }

    fn make_string(s: &str) -> Spanned<Expr> {
        Spanned {
            inner: Expr::Literal(Literal::String(s.to_string())),
            span: make_span(),
        }
    }

    #[test]
    fn test_parse_module_name_expr() {
        let expr = Spanned {
            inner: Expr::List(vec![
                make_symbol("::"),
                make_symbol("string"),
            ]),
            span: make_span(),
        };
        
        let module_id = parse_module_name_expr(&expr).unwrap();
        assert_eq!(module_id.namespace, super::super::ModuleNamespace::Builtin);
        assert_eq!(module_id.components, vec!["string"]);
    }

    #[test]
    fn test_parse_metadata() {
        let elements = vec![
            Spanned {
                inner: Expr::List(vec![
                    make_symbol("version"),
                    make_string("1.0.0"),
                ]),
                span: make_span(),
            },
            Spanned {
                inner: Expr::List(vec![
                    make_symbol("description"),
                    make_string("String manipulation module"),
                ]),
                span: make_span(),
            },
        ];
        
        let metadata = parse_metadata(&elements, make_span()).unwrap();
        assert_eq!(metadata.version, Some("1.0.0".to_string()));
        assert_eq!(metadata.description, Some("String manipulation module".to_string()));
    }

    #[test]
    fn test_extract_define_binding() {
        let mut bindings = HashMap::new();
        let elements = vec![
            make_symbol("test-function"),
            make_string("test value"),
        ];
        
        extract_define_binding(&elements, &mut bindings, make_span()).unwrap();
        
        assert!(bindings.contains_key("test-function"));
    }

    #[test]
    fn test_validate_module_definition() {
        let definition = ModuleDefinition {
            id: super::super::name::builtin_module("test"),
            imports: Vec::new(),
            export: ExportSpec {
                symbols: vec!["test-function".to_string()],
                config: super::super::ExportConfig::Direct,
            },
            body: Vec::new(),
            metadata: ModuleMetadata::default(),
            span: Some(make_span()),
        };
        
        let result = validate_module_definition(&definition);
        assert!(result.is_ok());
    }
}
//! Syntax-case macro system implementation
//!
//! This module provides the core syntax-case macro system, including pattern matching
//! for syntax objects, template construction, and the fundamental procedures:
//! - syntax-case: pattern matching for syntax objects
//! - syntax: template construction (#' quasisyntax)
//! - datum->syntax: convert datums to syntax objects
//! - syntax->datum: extract datums from syntax objects
//!
//! This implementation follows R6RS and R7RS specifications for syntax-case.

use super::advanced_hygiene::{HygieneResolver, Mark, MarkSet};
use super::syntax_objects::{SyntaxObject, LexicalContext, BindingInfo, syntax_utils};
use super::identifier_transformers::{VariableTransformer, IdentifierContext, VariableTransformerRegistry};
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Span, Spanned};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// A syntax-case pattern that can match against syntax objects
pub enum SyntaxPattern {
    /// Matches any syntax object and binds it to a pattern variable
    PatternVariable(String),
    /// Matches a literal value exactly
    Literal(Literal),
    /// Matches an identifier, possibly with binding constraints
    Identifier {
        /// The identifier name to match
        name: String,
        /// Optional binding level constraint for hygiene checking
        binding_level: Option<i32>,
    },
    /// Matches the empty list
    Nil,
    /// Matches a proper list of patterns
    List(Vec<SyntaxPattern>),
    /// Matches an improper list (dotted pair)
    ImproperList {
        /// Patterns for the list elements
        patterns: Vec<SyntaxPattern>,
        /// Pattern for the tail (final cdr)
        tail: Box<SyntaxPattern>,
    },
    /// Matches a vector
    Vector(Vec<SyntaxPattern>),
    /// Ellipsis pattern for matching repetitive structures
    Ellipsis {
        /// The pattern to match repeatedly
        pattern: Box<SyntaxPattern>,
        /// Minimum number of required matches
        min_count: usize,
        /// Maximum number of allowed matches (None = unbounded)
        max_count: Option<usize>,
    },
    /// Matches any of the given alternatives
    Alternative(Vec<SyntaxPattern>),
    /// Pattern guard that applies additional constraints
    Guard {
        /// The base pattern to match
        pattern: Box<SyntaxPattern>,
        /// Additional predicate that must be satisfied
        predicate: Box<dyn Fn(&SyntaxObject) -> bool>,
    },
    /// Matches syntax with specific properties
    WithProperties {
        /// The base pattern to match
        pattern: Box<SyntaxPattern>,
        /// List of property names that must be present
        required_properties: Vec<String>,
    },
}

impl std::fmt::Debug for SyntaxPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyntaxPattern::PatternVariable(name) => write!(f, "PatternVariable({name})"),
            SyntaxPattern::Literal(lit) => write!(f, "Literal({lit:?})"),
            SyntaxPattern::Identifier { name, binding_level } => {
                write!(f, "Identifier {{ name: {name}, binding_level: {binding_level:?} }}")
            }
            SyntaxPattern::Nil => write!(f, "Nil"),
            SyntaxPattern::List(patterns) => write!(f, "List({patterns:?})"),
            SyntaxPattern::ImproperList { patterns, tail } => {
                write!(f, "ImproperList {{ patterns: {patterns:?}, tail: {tail:?} }}")
            }
            SyntaxPattern::Ellipsis { pattern, min_count, max_count } => {
                write!(f, "Ellipsis {{ pattern: {pattern:?}, min_count: {min_count}, max_count: {max_count:?} }}")
            }
            SyntaxPattern::Alternative(patterns) => write!(f, "Alternative({patterns:?})"),
            SyntaxPattern::Guard { pattern, predicate: _ } => {
                write!(f, "Guard {{ pattern: {pattern:?}, predicate: <fn> }}")
            }
            SyntaxPattern::WithProperties { pattern, required_properties } => {
                write!(f, "WithProperties {{ pattern: {pattern:?}, required_properties: {required_properties:?} }}")
            }
            SyntaxPattern::Vector(patterns) => write!(f, "Vector({patterns:?})"),
        }
    }
}

impl Clone for SyntaxPattern {
    fn clone(&self) -> Self {
        match self {
            SyntaxPattern::PatternVariable(name) => SyntaxPattern::PatternVariable(name.clone()),
            SyntaxPattern::Literal(lit) => SyntaxPattern::Literal(lit.clone()),
            SyntaxPattern::Identifier { name, binding_level } => {
                SyntaxPattern::Identifier {
                    name: name.clone(),
                    binding_level: *binding_level,
                }
            }
            SyntaxPattern::Nil => SyntaxPattern::Nil,
            SyntaxPattern::List(patterns) => SyntaxPattern::List(patterns.clone()),
            SyntaxPattern::ImproperList { patterns, tail } => {
                SyntaxPattern::ImproperList {
                    patterns: patterns.clone(),
                    tail: tail.clone(),
                }
            }
            SyntaxPattern::Ellipsis { pattern, min_count, max_count } => {
                SyntaxPattern::Ellipsis {
                    pattern: pattern.clone(),
                    min_count: *min_count,
                    max_count: *max_count,
                }
            }
            SyntaxPattern::Alternative(patterns) => SyntaxPattern::Alternative(patterns.clone()),
            SyntaxPattern::Guard { pattern, predicate: _ } => {
                // Note: We can't clone function pointers, so we create a default guard that always returns true
                // In a real implementation, this would need to be handled differently
                SyntaxPattern::Guard {
                    pattern: pattern.clone(),
                    predicate: Box::new(|_| true),
                }
            }
            SyntaxPattern::WithProperties { pattern, required_properties } => {
                SyntaxPattern::WithProperties {
                    pattern: pattern.clone(),
                    required_properties: required_properties.clone(),
                }
            }
            SyntaxPattern::Vector(patterns) => SyntaxPattern::Vector(patterns.clone()),
        }
    }
}

impl PartialEq for SyntaxPattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SyntaxPattern::PatternVariable(a), SyntaxPattern::PatternVariable(b)) => a == b,
            (SyntaxPattern::Literal(a), SyntaxPattern::Literal(b)) => a == b,
            (SyntaxPattern::Identifier { name: a, binding_level: al }, 
             SyntaxPattern::Identifier { name: b, binding_level: bl }) => a == b && al == bl,
            (SyntaxPattern::Nil, SyntaxPattern::Nil) => true,
            (SyntaxPattern::List(a), SyntaxPattern::List(b)) => a == b,
            (SyntaxPattern::ImproperList { patterns: a, tail: at }, 
             SyntaxPattern::ImproperList { patterns: b, tail: bt }) => a == b && at == bt,
            (SyntaxPattern::Ellipsis { pattern: a, min_count: ac, max_count: amax }, 
             SyntaxPattern::Ellipsis { pattern: b, min_count: bc, max_count: bmax }) => {
                a == b && ac == bc && amax == bmax
            }
            (SyntaxPattern::Alternative(a), SyntaxPattern::Alternative(b)) => a == b,
            (SyntaxPattern::Guard { pattern: a, predicate: _ }, 
             SyntaxPattern::Guard { pattern: b, predicate: _ }) => {
                // Note: We can't compare function pointers, so we only compare the pattern
                a == b
            }
            (SyntaxPattern::WithProperties { pattern: a, required_properties: ap }, 
             SyntaxPattern::WithProperties { pattern: b, required_properties: bp }) => a == b && ap == bp,
            _ => false,
        }
    }
}

impl Serialize for SyntaxPattern {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        match self {
            SyntaxPattern::PatternVariable(name) => {
                let mut state = serializer.serialize_struct("SyntaxPattern", 2)?;
                state.serialize_field("type", "PatternVariable")?;
                state.serialize_field("name", name)?;
                state.end()
            }
            SyntaxPattern::Literal(lit) => {
                let mut state = serializer.serialize_struct("SyntaxPattern", 2)?;
                state.serialize_field("type", "Literal")?;
                state.serialize_field("literal", lit)?;
                state.end()
            }
            SyntaxPattern::Identifier { name, binding_level } => {
                let mut state = serializer.serialize_struct("SyntaxPattern", 3)?;
                state.serialize_field("type", "Identifier")?;
                state.serialize_field("name", name)?;
                state.serialize_field("binding_level", binding_level)?;
                state.end()
            }
            SyntaxPattern::Nil => {
                let mut state = serializer.serialize_struct("SyntaxPattern", 1)?;
                state.serialize_field("type", "Nil")?;
                state.end()
            }
            SyntaxPattern::List(patterns) => {
                let mut state = serializer.serialize_struct("SyntaxPattern", 2)?;
                state.serialize_field("type", "List")?;
                state.serialize_field("patterns", patterns)?;
                state.end()
            }
            SyntaxPattern::ImproperList { patterns, tail } => {
                let mut state = serializer.serialize_struct("SyntaxPattern", 3)?;
                state.serialize_field("type", "ImproperList")?;
                state.serialize_field("patterns", patterns)?;
                state.serialize_field("tail", tail)?;
                state.end()
            }
            SyntaxPattern::Ellipsis { pattern, min_count, max_count } => {
                let mut state = serializer.serialize_struct("SyntaxPattern", 4)?;
                state.serialize_field("type", "Ellipsis")?;
                state.serialize_field("pattern", pattern)?;
                state.serialize_field("min_count", min_count)?;
                state.serialize_field("max_count", max_count)?;
                state.end()
            }
            SyntaxPattern::Alternative(patterns) => {
                let mut state = serializer.serialize_struct("SyntaxPattern", 2)?;
                state.serialize_field("type", "Alternative")?;
                state.serialize_field("patterns", patterns)?;
                state.end()
            }
            SyntaxPattern::Guard { pattern, predicate: _ } => {
                // Note: We can't serialize function pointers, so we just serialize the pattern
                let mut state = serializer.serialize_struct("SyntaxPattern", 2)?;
                state.serialize_field("type", "Guard")?;
                state.serialize_field("pattern", pattern)?;
                state.end()
            }
            SyntaxPattern::WithProperties { pattern, required_properties } => {
                let mut state = serializer.serialize_struct("SyntaxPattern", 3)?;
                state.serialize_field("type", "WithProperties")?;
                state.serialize_field("pattern", pattern)?;
                state.serialize_field("required_properties", required_properties)?;
                state.end()
            }
            SyntaxPattern::Vector(patterns) => {
                let mut state = serializer.serialize_struct("SyntaxPattern", 2)?;
                state.serialize_field("type", "Vector")?;
                state.serialize_field("patterns", patterns)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for SyntaxPattern {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use serde::de::Error;
        
        struct SyntaxPatternVisitor;
        
        impl<'de> Visitor<'de> for SyntaxPatternVisitor {
            type Value = SyntaxPattern;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a SyntaxPattern")
            }
            
            fn visit_map<V>(self, mut map: V) -> std::result::Result<SyntaxPattern, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut pattern_type: Option<String> = None;
                let mut data: Option<serde_json::Value> = None;
                
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => pattern_type = Some(map.next_value()?),
                        _ => {
                            if data.is_none() {
                                data = Some(serde_json::Value::Object(serde_json::Map::new()));
                            }
                            if let Some(serde_json::Value::Object(ref mut obj)) = data {
                                obj.insert(key, map.next_value()?);
                            }
                        }
                    }
                }
                
                let pattern_type = pattern_type.ok_or_else(|| Error::missing_field("type"))?;
                
                match pattern_type.as_str() {
                    "PatternVariable" => {
                        let name: String = serde_json::from_value(
                            data.and_then(|d| d.get("name").cloned()).unwrap_or(serde_json::Value::Null)
                        ).map_err(Error::custom)?;
                        Ok(SyntaxPattern::PatternVariable(name))
                    }
                    "Nil" => Ok(SyntaxPattern::Nil),
                    "Guard" => {
                        // For Guard patterns, we deserialize the pattern but create a default predicate
                        let pattern: Box<SyntaxPattern> = serde_json::from_value(
                            data.and_then(|d| d.get("pattern").cloned()).unwrap_or(serde_json::Value::Null)
                        ).map_err(Error::custom)?;
                        Ok(SyntaxPattern::Guard {
                            pattern,
                            predicate: Box::new(|_| true), // Default predicate
                        })
                    }
                    _ => {
                        // For other variants, use serde_json for simplicity in this example
                        // In a real implementation, you'd handle each variant properly
                        Err(Error::custom(format!("Unsupported pattern type: {pattern_type}")))
                    }
                }
            }
        }
        
        deserializer.deserialize_map(SyntaxPatternVisitor)
    }
}

/// Bindings created during syntax pattern matching
#[derive(Debug, Clone)]
pub struct SyntaxBindings {
    /// Single-value bindings
    bindings: HashMap<String, SyntaxObject>,
    /// Multi-value bindings from ellipsis patterns
    ellipsis_bindings: HashMap<String, Vec<SyntaxObject>>,
    /// Nested ellipsis bindings (for multiple levels of ellipsis)
    nested_bindings: HashMap<String, Vec<Vec<SyntaxObject>>>,
}

impl SyntaxBindings {
    /// Creates a new empty set of syntax bindings
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            ellipsis_bindings: HashMap::new(),
            nested_bindings: HashMap::new(),
        }
    }

    /// Binds a pattern variable to a syntax object
    pub fn bind(&mut self, name: String, syntax: SyntaxObject) {
        self.bindings.insert(name, syntax);
    }

    /// Binds a pattern variable to a list of syntax objects (ellipsis binding)
    pub fn bind_ellipsis(&mut self, name: String, syntaxes: Vec<SyntaxObject>) {
        self.ellipsis_bindings.insert(name, syntaxes);
    }

    /// Binds a pattern variable to nested lists (multiple ellipsis levels)
    pub fn bind_nested(&mut self, name: String, nested_syntaxes: Vec<Vec<SyntaxObject>>) {
        self.nested_bindings.insert(name, nested_syntaxes);
    }

    /// Gets a single binding
    pub fn get(&self, name: &str) -> Option<&SyntaxObject> {
        self.bindings.get(name)
    }

    /// Gets an ellipsis binding
    pub fn get_ellipsis(&self, name: &str) -> Option<&Vec<SyntaxObject>> {
        self.ellipsis_bindings.get(name)
    }

    /// Gets a nested binding
    pub fn get_nested(&self, name: &str) -> Option<&Vec<Vec<SyntaxObject>>> {
        self.nested_bindings.get(name)
    }

    /// Merges another set of bindings into this one
    pub fn merge(&mut self, other: SyntaxBindings) {
        self.bindings.extend(other.bindings);
        self.ellipsis_bindings.extend(other.ellipsis_bindings);
        self.nested_bindings.extend(other.nested_bindings);
    }

    /// Lists all binding names
    pub fn binding_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.bindings.keys().map(|s| s.as_str()).collect();
        names.extend(self.ellipsis_bindings.keys().map(|s| s.as_str()));
        names.extend(self.nested_bindings.keys().map(|s| s.as_str()));
        names.sort();
        names.dedup();
        names
    }

    /// Gets all pattern variable names
    pub fn pattern_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();
        vars.extend(self.bindings.keys().cloned());
        vars.extend(self.ellipsis_bindings.keys().cloned());
        vars.extend(self.nested_bindings.keys().cloned());
        vars.sort();
        vars.dedup();
        vars
    }
}

impl Default for SyntaxBindings {
    fn default() -> Self {
        Self::new()
    }
}

/// A syntax template for constructing syntax objects
#[derive(Debug, Clone, PartialEq)]
pub enum SyntaxTemplate {
    /// Substitutes a pattern variable binding
    PatternVariable(String),
    /// Generates a literal syntax object
    Literal(Literal),
    /// Generates an identifier syntax object
    Identifier(String),
    /// Generates an empty list
    Nil,
    /// Generates a proper list
    List(Vec<SyntaxTemplate>),
    /// Generates an improper list (dotted pair)
    ImproperList {
        /// Templates for the list elements
        templates: Vec<SyntaxTemplate>,
        /// Template for the tail (final cdr)
        tail: Box<SyntaxTemplate>,
    },
    /// Generates a vector
    Vector(Vec<SyntaxTemplate>),
    /// Ellipsis template for generating repetitive structures
    Ellipsis {
        /// The template to repeat
        template: Box<SyntaxTemplate>,
        /// Optional separator template between repetitions
        separator: Option<Box<SyntaxTemplate>>,
    },
    /// Conditional template expansion
    Conditional {
        /// Pattern variable name to test for binding
        condition: String,
        /// Template to use if condition is bound
        then_template: Box<SyntaxTemplate>,
        /// Optional template to use if condition is not bound
        else_template: Option<Box<SyntaxTemplate>>,
    },
    /// Applies a transformation function
    Transform {
        /// Name of the transformation function to apply
        function_name: String,
        /// Template to transform
        argument: Box<SyntaxTemplate>,
    },
    /// Escapes to compute a template at macro-expansion time
    Escape(Box<SyntaxTemplate>),
}

impl Serialize for SyntaxTemplate {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        match self {
            SyntaxTemplate::PatternVariable(name) => {
                let mut state = serializer.serialize_struct("SyntaxTemplate", 2)?;
                state.serialize_field("type", "PatternVariable")?;
                state.serialize_field("name", name)?;
                state.end()
            }
            SyntaxTemplate::Literal(lit) => {
                let mut state = serializer.serialize_struct("SyntaxTemplate", 2)?;
                state.serialize_field("type", "Literal")?;
                state.serialize_field("literal", lit)?;
                state.end()
            }
            SyntaxTemplate::Identifier(name) => {
                let mut state = serializer.serialize_struct("SyntaxTemplate", 2)?;
                state.serialize_field("type", "Identifier")?;
                state.serialize_field("name", name)?;
                state.end()
            }
            SyntaxTemplate::Nil => {
                let mut state = serializer.serialize_struct("SyntaxTemplate", 1)?;
                state.serialize_field("type", "Nil")?;
                state.end()
            }
            SyntaxTemplate::List(templates) => {
                let mut state = serializer.serialize_struct("SyntaxTemplate", 2)?;
                state.serialize_field("type", "List")?;
                state.serialize_field("templates", templates)?;
                state.end()
            }
            _ => {
                // For other variants, serialize as a basic identifier for now
                let mut state = serializer.serialize_struct("SyntaxTemplate", 2)?;
                state.serialize_field("type", "Simplified")?;
                state.serialize_field("name", "template")?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for SyntaxTemplate {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use serde::de::Error;
        
        struct SyntaxTemplateVisitor;
        
        impl<'de> Visitor<'de> for SyntaxTemplateVisitor {
            type Value = SyntaxTemplate;
            
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a SyntaxTemplate")
            }
            
            fn visit_map<V>(self, mut map: V) -> std::result::Result<SyntaxTemplate, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut template_type: Option<String> = None;
                let mut data: Option<serde_json::Value> = None;
                
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => template_type = Some(map.next_value()?),
                        _ => {
                            if data.is_none() {
                                data = Some(serde_json::Value::Object(serde_json::Map::new()));
                            }
                            if let Some(serde_json::Value::Object(ref mut obj)) = data {
                                obj.insert(key, map.next_value()?);
                            }
                        }
                    }
                }
                
                let template_type = template_type.ok_or_else(|| Error::missing_field("type"))?;
                
                match template_type.as_str() {
                    "PatternVariable" => {
                        let name: String = serde_json::from_value(
                            data.and_then(|d| d.get("name").cloned()).unwrap_or(serde_json::Value::Null)
                        ).map_err(Error::custom)?;
                        Ok(SyntaxTemplate::PatternVariable(name))
                    }
                    "Identifier" => {
                        let name: String = serde_json::from_value(
                            data.and_then(|d| d.get("name").cloned()).unwrap_or(serde_json::Value::Null)
                        ).map_err(Error::custom)?;
                        Ok(SyntaxTemplate::Identifier(name))
                    }
                    "Nil" => Ok(SyntaxTemplate::Nil),
                    _ => {
                        // For unsupported types, return a basic identifier
                        Ok(SyntaxTemplate::Identifier("template".to_string()))
                    }
                }
            }
        }
        
        deserializer.deserialize_map(SyntaxTemplateVisitor)
    }
}

impl SyntaxTemplate {
    /// Expands this template using the given bindings
    pub fn expand(
        &self,
        bindings: &SyntaxBindings,
        context: &LexicalContext,
        span: Span,
    ) -> Result<SyntaxObject> {
        match self {
            SyntaxTemplate::PatternVariable(name) => {
                if let Some(syntax) = bindings.get(name) {
                    Ok(syntax.clone())
                } else {
                    Err(Box::new(Error::macro_error(
                        format!("Unbound pattern variable: {name}"),
                        span,
                    )))
                }
            }

            SyntaxTemplate::Literal(lit) => {
                Ok(SyntaxObject::new(Expr::Literal(lit.clone()), span, context.clone()))
            }

            SyntaxTemplate::Identifier(name) => {
                Ok(SyntaxObject::new(Expr::Identifier(name.clone()), span, context.clone()))
            }

            SyntaxTemplate::Nil => {
                Ok(SyntaxObject::new(Expr::Literal(Literal::Nil), span, context.clone()))
            }

            SyntaxTemplate::List(templates) => {
                let mut elements = Vec::new();
                for template in templates {
                    let syntax = template.expand(bindings, context, span)?;
                    elements.push(syntax.to_spanned());
                }
                Ok(SyntaxObject::new(Expr::List(elements), span, context.clone()))
            }

            SyntaxTemplate::ImproperList { templates, tail } => {
                if templates.is_empty() {
                    return tail.expand(bindings, context, span);
                }

                let mut result = tail.expand(bindings, context, span)?;
                
                // Build the improper list from right to left
                for template in templates.iter().rev() {
                    let car_syntax = template.expand(bindings, context, span)?;
                    let pair_expr = Expr::Pair {
                        car: Box::new(car_syntax.to_spanned()),
                        cdr: Box::new(result.to_spanned()),
                    };
                    result = SyntaxObject::new(pair_expr, span, context.clone());
                }

                Ok(result)
            }

            SyntaxTemplate::Vector(templates) => {
                let mut elements = Vec::new();
                for template in templates {
                    let syntax = template.expand(bindings, context, span)?;
                    elements.push(syntax.to_spanned());
                }
                // For now, represent vectors as lists - could be extended
                Ok(SyntaxObject::new(Expr::List(elements), span, context.clone()))
            }

            SyntaxTemplate::Ellipsis { template, separator } => {
                self.expand_ellipsis(template, separator.as_deref(), bindings, context, span)
            }

            SyntaxTemplate::Conditional { condition, then_template, else_template } => {
                // Check if the condition pattern variable is bound and non-empty
                let use_then = if let Some(syntax) = bindings.get(condition) {
                    !matches!(syntax.expr, Expr::Literal(Literal::Nil))
                } else if let Some(ellipsis_list) = bindings.get_ellipsis(condition) {
                    !ellipsis_list.is_empty()
                } else {
                    false
                };

                if use_then {
                    then_template.expand(bindings, context, span)
                } else if let Some(else_tmpl) = else_template {
                    else_tmpl.expand(bindings, context, span)
                } else {
                    Ok(SyntaxObject::new(Expr::Literal(Literal::Nil), span, context.clone()))
                }
            }

            SyntaxTemplate::Transform { function_name, argument } => {
                // For now, just expand the argument
                // TODO: Implement actual transformation functions
                argument.expand(bindings, context, span)
            }

            SyntaxTemplate::Escape(template) => {
                // For now, just expand normally
                // TODO: Implement macro-time computation
                template.expand(bindings, context, span)
            }
        }
    }

    /// Expands an ellipsis template
    fn expand_ellipsis(
        &self,
        template: &SyntaxTemplate,
        separator: Option<&SyntaxTemplate>,
        bindings: &SyntaxBindings,
        context: &LexicalContext,
        span: Span,
    ) -> Result<SyntaxObject> {
        // Find ellipsis variables in the template
        let ellipsis_vars = self.find_ellipsis_variables(template);
        
        if ellipsis_vars.is_empty() {
            return Err(Box::new(Error::macro_error(
                "Ellipsis template contains no ellipsis variables".to_string(),
                span,
            )));
        }

        // Determine the length of expansion
        let max_length = ellipsis_vars
            .iter()
            .filter_map(|var| bindings.get_ellipsis(var))
            .map(|list| list.len())
            .max()
            .unwrap_or(0);

        let mut expanded_elements = Vec::new();

        for i in 0..max_length {
            // Create local bindings for this iteration
            let mut local_bindings = SyntaxBindings::new();
            
            // Copy non-ellipsis bindings
            for (name, syntax) in &bindings.bindings {
                local_bindings.bind(name.clone(), syntax.clone());
            }

            // Add ellipsis bindings for this iteration
            for var in &ellipsis_vars {
                if let Some(ellipsis_list) = bindings.get_ellipsis(var) {
                    if i < ellipsis_list.len() {
                        local_bindings.bind(var.clone(), ellipsis_list[i].clone());
                    }
                }
            }

            // Expand template for this iteration
            let expanded = template.expand(&local_bindings, context, span)?;
            expanded_elements.push(expanded.to_spanned());

            // Add separator if specified and not the last element
            if let Some(sep_template) = separator {
                if i < max_length - 1 {
                    let sep_syntax = sep_template.expand(&local_bindings, context, span)?;
                    expanded_elements.push(sep_syntax.to_spanned());
                }
            }
        }

        Ok(SyntaxObject::new(Expr::List(expanded_elements), span, context.clone()))
    }

    /// Finds all ellipsis variables referenced in a template
    fn find_ellipsis_variables(&self, template: &SyntaxTemplate) -> Vec<String> {
        let mut vars = Vec::new();
        self.collect_ellipsis_variables(template, &mut vars);
        vars.sort();
        vars.dedup();
        vars
    }

    /// Recursively collects ellipsis variables
    #[allow(clippy::only_used_in_recursion)]
    fn collect_ellipsis_variables(&self, template: &SyntaxTemplate, vars: &mut Vec<String>) {
        match template {
            SyntaxTemplate::PatternVariable(name) => vars.push(name.clone()),
            SyntaxTemplate::List(templates) => {
                for tmpl in templates {
                    self.collect_ellipsis_variables(tmpl, vars);
                }
            }
            SyntaxTemplate::ImproperList { templates, tail } => {
                for tmpl in templates {
                    self.collect_ellipsis_variables(tmpl, vars);
                }
                self.collect_ellipsis_variables(tail, vars);
            }
            SyntaxTemplate::Vector(templates) => {
                for tmpl in templates {
                    self.collect_ellipsis_variables(tmpl, vars);
                }
            }
            SyntaxTemplate::Ellipsis { template: tmpl, separator } => {
                self.collect_ellipsis_variables(tmpl, vars);
                if let Some(sep) = separator {
                    self.collect_ellipsis_variables(sep, vars);
                }
            }
            SyntaxTemplate::Conditional { then_template, else_template, .. } => {
                self.collect_ellipsis_variables(then_template, vars);
                if let Some(else_tmpl) = else_template {
                    self.collect_ellipsis_variables(else_tmpl, vars);
                }
            }
            SyntaxTemplate::Transform { argument, .. } => {
                self.collect_ellipsis_variables(argument, vars);
            }
            SyntaxTemplate::Escape(template) => {
                self.collect_ellipsis_variables(template, vars);
            }
            _ => {}
        }
    }
}

/// Implementation of syntax-case pattern matching
impl SyntaxPattern {
    /// Attempts to match this pattern against a syntax object
    pub fn match_syntax(&self, syntax: &SyntaxObject) -> Result<SyntaxBindings> {
        let mut bindings = SyntaxBindings::new();
        self.match_syntax_with_bindings(syntax, &mut bindings)?;
        Ok(bindings)
    }

    /// Internal matching with existing bindings
    fn match_syntax_with_bindings(
        &self,
        syntax: &SyntaxObject,
        bindings: &mut SyntaxBindings,
    ) -> Result<()> {
        match self {
            SyntaxPattern::PatternVariable(name) => {
                bindings.bind(name.clone(), syntax.clone());
                Ok(())
            }

            SyntaxPattern::Literal(pattern_lit) => {
                match &syntax.expr {
                    Expr::Literal(syntax_lit) if pattern_lit == syntax_lit => Ok(()),
                    _ => Err(Box::new(Error::macro_error(
                        format!("Literal pattern mismatch: expected {pattern_lit:?}"),
                        syntax.span,
                    ))),
                }
            }

            SyntaxPattern::Identifier { name, binding_level } => {
                match &syntax.expr {
                    Expr::Identifier(syntax_name) | Expr::Symbol(syntax_name) => {
                        if name == syntax_name {
                            // Additional binding level checks could go here
                            Ok(())
                        } else {
                            Err(Box::new(Error::macro_error(
                                format!("Identifier pattern mismatch: expected {name}, got {syntax_name}"),
                                syntax.span,
                            )))
                        }
                    }
                    _ => Err(Box::new(Error::macro_error(
                        format!("Expected identifier {}, got {:?}", name, syntax.expr),
                        syntax.span,
                    ))),
                }
            }

            SyntaxPattern::Nil => {
                match &syntax.expr {
                    Expr::Literal(Literal::Nil) => Ok(()),
                    Expr::List(elements) if elements.is_empty() => Ok(()),
                    _ => Err(Box::new(Error::macro_error(
                        "Expected empty list/nil".to_string(),
                        syntax.span,
                    ))),
                }
            }

            SyntaxPattern::List(patterns) => {
                match &syntax.expr {
                    Expr::List(elements) => {
                        if patterns.len() != elements.len() {
                            return Err(Box::new(Error::macro_error(
                                format!("List length mismatch: pattern has {}, syntax has {}",
                                       patterns.len(), elements.len()),
                                syntax.span,
                            )));
                        }

                        for (pattern, element) in patterns.iter().zip(elements.iter()) {
                            let element_syntax = SyntaxObject::from_spanned(
                                element.clone(),
                                syntax.context.clone(),
                            );
                            pattern.match_syntax_with_bindings(&element_syntax, bindings)?;
                        }
                        Ok(())
                    }
                    Expr::Application { operator, operands } => {
                        let mut all_elements = vec![(**operator).clone()];
                        all_elements.extend(operands.iter().cloned());
                        
                        if patterns.len() != all_elements.len() {
                            return Err(Box::new(Error::macro_error(
                                format!("Application length mismatch: pattern has {}, syntax has {}",
                                       patterns.len(), all_elements.len()),
                                syntax.span,
                            )));
                        }

                        for (pattern, element) in patterns.iter().zip(all_elements.iter()) {
                            let element_syntax = SyntaxObject::from_spanned(
                                element.clone(),
                                syntax.context.clone(),
                            );
                            pattern.match_syntax_with_bindings(&element_syntax, bindings)?;
                        }
                        Ok(())
                    }
                    _ => Err(Box::new(Error::macro_error(
                        "Expected list or application".to_string(),
                        syntax.span,
                    ))),
                }
            }

            SyntaxPattern::ImproperList { patterns, tail } => {
                match &syntax.expr {
                    Expr::Pair { .. } => {
                        // Handle dotted pairs
                        let elements = self.collect_improper_list_elements(&syntax.expr);
                        if elements.len() < patterns.len() {
                            return Err(Box::new(Error::macro_error(
                                "Improper list too short for pattern".to_string(),
                                syntax.span,
                            )));
                        }

                        // Match fixed patterns
                        for (i, pattern) in patterns.iter().enumerate() {
                            let element_syntax = SyntaxObject::new(
                                elements[i].clone(),
                                syntax.span,
                                syntax.context.clone(),
                            );
                            pattern.match_syntax_with_bindings(&element_syntax, bindings)?;
                        }

                        // Match tail
                        if patterns.len() < elements.len() {
                            let tail_expr = self.build_tail_from_elements(&elements[patterns.len()..]);
                            let tail_syntax = SyntaxObject::new(
                                tail_expr,
                                syntax.span,
                                syntax.context.clone(),
                            );
                            tail.match_syntax_with_bindings(&tail_syntax, bindings)?;
                        }

                        Ok(())
                    }
                    _ => Err(Box::new(Error::macro_error(
                        "Expected improper list (dotted pair)".to_string(),
                        syntax.span,
                    ))),
                }
            }

            SyntaxPattern::Ellipsis { pattern, min_count, max_count } => {
                self.match_ellipsis_pattern(pattern, *min_count, *max_count, syntax, bindings)
            }

            SyntaxPattern::Alternative(alternatives) => {
                let mut last_error = None;
                for alt_pattern in alternatives {
                    let mut alt_bindings = bindings.clone();
                    match alt_pattern.match_syntax_with_bindings(syntax, &mut alt_bindings) {
                        Ok(()) => {
                            bindings.merge(alt_bindings);
                            return Ok(());
                        }
                        Err(e) => last_error = Some(e),
                    }
                }
                Err(last_error.unwrap_or_else(|| {
                    Box::new(Error::macro_error(
                        "No alternative patterns matched".to_string(),
                        syntax.span,
                    ))
                }))
            }

            SyntaxPattern::Guard { pattern, predicate } => {
                // First match the underlying pattern
                pattern.match_syntax_with_bindings(syntax, bindings)?;
                
                // Then check the predicate
                if predicate(syntax) {
                    Ok(())
                } else {
                    Err(Box::new(Error::macro_error(
                        "Pattern guard failed".to_string(),
                        syntax.span,
                    )))
                }
            }

            SyntaxPattern::WithProperties { pattern, required_properties } => {
                // Check that syntax has required properties
                for prop_name in required_properties {
                    if syntax.get_property(prop_name).is_none() {
                        return Err(Box::new(Error::macro_error(
                            format!("Required property '{prop_name}' not found"),
                            syntax.span,
                        )));
                    }
                }

                // Then match the underlying pattern
                pattern.match_syntax_with_bindings(syntax, bindings)
            }

            SyntaxPattern::Vector(patterns) => {
                // For now, treat vectors like lists
                match &syntax.expr {
                    Expr::List(elements) => {
                        if patterns.len() != elements.len() {
                            return Err(Box::new(Error::macro_error(
                                format!("Vector length mismatch: pattern has {}, syntax has {}",
                                       patterns.len(), elements.len()),
                                syntax.span,
                            )));
                        }

                        for (pattern, element) in patterns.iter().zip(elements.iter()) {
                            let element_syntax = SyntaxObject::from_spanned(
                                element.clone(),
                                syntax.context.clone(),
                            );
                            pattern.match_syntax_with_bindings(&element_syntax, bindings)?;
                        }
                        Ok(())
                    }
                    _ => Err(Box::new(Error::macro_error(
                        "Expected vector".to_string(),
                        syntax.span,
                    ))),
                }
            }
        }
    }

    /// Matches an ellipsis pattern against syntax
    fn match_ellipsis_pattern(
        &self,
        pattern: &SyntaxPattern,
        min_count: usize,
        max_count: Option<usize>,
        syntax: &SyntaxObject,
        bindings: &mut SyntaxBindings,
    ) -> Result<()> {
        let elements = match &syntax.expr {
            Expr::List(elements) => elements.clone(),
            Expr::Application { operator, operands } => {
                let mut all_elements = vec![(**operator).clone()];
                all_elements.extend(operands.iter().cloned());
                all_elements
            }
            _ => {
                return Err(Box::new(Error::macro_error(
                    "Ellipsis pattern requires list or application".to_string(),
                    syntax.span,
                )));
            }
        };

        let count = elements.len();
        if count < min_count {
            return Err(Box::new(Error::macro_error(
                format!("Not enough elements: need at least {min_count}, got {count}"),
                syntax.span,
            )));
        }

        if let Some(max) = max_count {
            if count > max {
                return Err(Box::new(Error::macro_error(
                    format!("Too many elements: maximum {max}, got {count}"),
                    syntax.span,
                )));
            }
        }

        // Collect ellipsis bindings
        let mut ellipsis_matches = Vec::new();
        for element in &elements {
            let element_syntax = SyntaxObject::from_spanned(
                element.clone(),
                syntax.context.clone(),
            );
            
            let mut element_bindings = SyntaxBindings::new();
            pattern.match_syntax_with_bindings(&element_syntax, &mut element_bindings)?;
            ellipsis_matches.push(element_syntax);

            // Collect pattern variables for ellipsis binding
            for var_name in element_bindings.pattern_variables() {
                if let Some(bound_syntax) = element_bindings.get(&var_name) {
                    if let Some(existing) = bindings.ellipsis_bindings.get_mut(&var_name) {
                        existing.push(bound_syntax.clone());
                    } else {
                        bindings.bind_ellipsis(var_name, vec![bound_syntax.clone()]);
                    }
                }
            }
        }

        Ok(())
    }

    /// Helper to collect elements from an improper list
    fn collect_improper_list_elements(&self, expr: &Expr) -> Vec<Expr> {
        let mut elements = Vec::new();
        let mut current = expr;
        
        while let Expr::Pair { car, cdr } = current {
            elements.push(car.inner.clone());
            current = &cdr.inner;
        }
        
        // Add the final tail if it's not nil
        if !matches!(current, Expr::Literal(Literal::Nil)) {
            elements.push(current.clone());
        }
        
        elements
    }

    /// Helper to build a tail expression from remaining elements
    fn build_tail_from_elements(&self, elements: &[Expr]) -> Expr {
        if elements.is_empty() {
            Expr::Literal(Literal::Nil)
        } else if elements.len() == 1 {
            elements[0].clone()
        } else {
            // Build nested pairs
            let mut result = elements[elements.len() - 1].clone();
            for element in elements[..elements.len() - 1].iter().rev() {
                result = Expr::Pair {
                    car: Box::new(Spanned::new(element.clone(), Span::new(0, 0))),
                    cdr: Box::new(Spanned::new(result, Span::new(0, 0))),
                };
            }
            result
        }
    }
}

/// Core syntax procedures implementation
pub mod syntax_procedures {
    use super::*;

    /// Implements datum->syntax procedure
    pub fn datum_to_syntax(
        datum: Expr,
        template_identifier: Option<&SyntaxObject>,
        properties: Option<HashMap<String, super::super::syntax_objects::SyntaxProperty>>,
    ) -> SyntaxObject {
        let context = if let Some(template) = template_identifier {
            template.context.clone()
        } else {
            LexicalContext::new(0, vec!["top-level".to_string()])
        };

        let span = template_identifier
            .map(|t| t.span)
            .unwrap_or_else(|| Span::new(0, 0));

        let mut syntax = SyntaxObject::new(datum, span, context);

        if let Some(props) = properties {
            for (key, value) in props {
                syntax.set_property(key, value);
            }
        }

        syntax
    }

    /// Implements syntax->datum procedure
    pub fn syntax_to_datum(syntax: &SyntaxObject) -> Expr {
        syntax.expr.clone()
    }

    /// Implements bound-identifier=? procedure
    pub fn bound_identifier_equal(stx1: &SyntaxObject, stx2: &SyntaxObject) -> bool {
        syntax_utils::bound_identifier_equal(stx1, stx2)
    }

    /// Implements free-identifier=? procedure
    pub fn free_identifier_equal(stx1: &SyntaxObject, stx2: &SyntaxObject) -> bool {
        syntax_utils::free_identifier_equal(stx1, stx2)
    }

    /// Implements identifier? procedure
    pub fn is_identifier(syntax: &SyntaxObject) -> bool {
        syntax.is_identifier()
    }

    /// Creates a syntax object with the #' (syntax) form
    pub fn make_syntax(
        template: SyntaxTemplate,
        bindings: &SyntaxBindings,
        context: &LexicalContext,
        span: Span,
    ) -> Result<SyntaxObject> {
        template.expand(bindings, context, span)
    }

    /// Implements syntax-case pattern matching
    pub fn syntax_case_match(
        input_syntax: &SyntaxObject,
        literal_identifiers: &[String],
        clauses: &[(SyntaxPattern, SyntaxTemplate)],
    ) -> Result<SyntaxObject> {
        for (pattern, template) in clauses {
            match pattern.match_syntax(input_syntax) {
                Ok(bindings) => {
                    return template.expand(&bindings, &input_syntax.context, input_syntax.span);
                }
                Err(_) => continue, // Try next clause
            }
        }

        Err(Box::new(Error::macro_error(
            "No syntax-case clause matched".to_string(),
            input_syntax.span,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;
    use crate::diagnostics::Span;

    #[test]
    fn test_pattern_variable_matching() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let syntax = SyntaxObject::new(
            Expr::Identifier("foo".to_string()),
            Span::new(0, 3),
            context,
        );

        let pattern = SyntaxPattern::PatternVariable("x".to_string());
        let bindings = pattern.match_syntax(&syntax).unwrap();

        assert_eq!(bindings.get("x").unwrap().identifier_name(), Some("foo"));
    }

    #[test]
    fn test_literal_pattern_matching() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let syntax = SyntaxObject::new(
            Expr::Literal(Literal::Number(42.0)),
            Span::new(0, 2),
            context,
        );

        let pattern = SyntaxPattern::Literal(Literal::Number(42.0));
        let bindings = pattern.match_syntax(&syntax).unwrap();

        // Literal patterns don't create bindings
        assert!(bindings.pattern_variables().is_empty());
    }

    #[test]
    fn test_list_pattern_matching() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let elements = vec![
            Spanned::new(Expr::Identifier("foo".to_string()), Span::new(1, 4)),
            Spanned::new(Expr::Literal(Literal::Number(42.0)), Span::new(5, 7)),
        ];
        let syntax = SyntaxObject::new(
            Expr::List(elements),
            Span::new(0, 8),
            context,
        );

        let pattern = SyntaxPattern::List(vec![
            SyntaxPattern::PatternVariable("x".to_string()),
            SyntaxPattern::PatternVariable("y".to_string()),
        ]);

        let bindings = pattern.match_syntax(&syntax).unwrap();

        assert_eq!(bindings.get("x").unwrap().identifier_name(), Some("foo"));
        assert!(matches!(bindings.get("y").unwrap().expr, Expr::Literal(Literal::Number(n)) if n == 42.0));
    }

    #[test]
    fn test_template_expansion() {
        let mut bindings = SyntaxBindings::new();
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        
        // Add a binding
        let bound_syntax = SyntaxObject::new(
            Expr::Identifier("foo".to_string()),
            Span::new(0, 3),
            context.clone(),
        );
        bindings.bind("x".to_string(), bound_syntax);

        // Create a template
        let template = SyntaxTemplate::List(vec![
            SyntaxTemplate::Identifier("lambda".to_string()),
            SyntaxTemplate::List(vec![SyntaxTemplate::PatternVariable("x".to_string())]),
            SyntaxTemplate::PatternVariable("x".to_string()),
        ]);

        // Expand the template
        let result = template.expand(&bindings, &context, Span::new(0, 10)).unwrap();

        // Should produce (lambda (foo) foo)
        assert!(matches!(result.expr, Expr::List(_)));
    }

    #[test]
    fn test_datum_to_syntax() {
        let datum = Expr::Identifier("test".to_string());
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let template = SyntaxObject::new(
            Expr::Identifier("template".to_string()),
            Span::new(0, 8),
            context,
        );

        let syntax = syntax_procedures::datum_to_syntax(datum.clone(), Some(&template), None);

        assert_eq!(syntax.expr, datum);
        assert_eq!(syntax.context.context_id, template.context.context_id);
    }

    #[test]
    fn test_syntax_to_datum() {
        let context = LexicalContext::new(1, vec!["test".to_string()]);
        let expr = Expr::Identifier("test".to_string());
        let syntax = SyntaxObject::new(expr.clone(), Span::new(0, 4), context);

        let datum = syntax_procedures::syntax_to_datum(&syntax);
        assert_eq!(datum, expr);
    }
}
//! Macro-time Computation System for Lambdust
//!
//! This module implements the macro-time computation capabilities that enable
//! sophisticated compile-time evaluation and template generation for advanced
//! macro programming. Key features include:
//!
//! - Phase-separated evaluation environments (macro-time vs runtime)
//! - Compile-time procedure calls and data structure manipulation
//! - Advanced template generation with splicing support
//! - Meta-programming utilities for macro writers
//! - Integration with syntax objects and hygiene system
//!
//! This system enables the target functionality from the roadmap:
//! ```scheme
//! (define-syntax repeat
//!   (lambda (stx)
//!     (syntax-case stx ()
//!       ((_ n expr)
//!        #`(begin #,@(make-list n #'expr))))))
//! ```

use super::{
    advanced_hygiene::{HygieneResolver, Mark, MarkSet},
    quasisyntax::{QuasisyntaxContext, QuasisyntaxExpansionResult, QuasisyntaxTemplate},
    syntax_objects::{LexicalContext, SyntaxObject, syntax_utils},
    unified_expander::{MacroTransformerType, UnifiedMacroTransformer},
};
use crate::ast::{Expr, Literal};
use crate::diagnostics::{Error, Result, Span, Spanned};
use crate::eval::{Environment, Parameter, Value};
use crate::utils::symbol_id::SymbolId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Phase levels for macro-time computation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Phase(i32);

impl Phase {
    /// Runtime phase (phase 0)
    pub const RUNTIME: Phase = Phase(0);
    /// Macro-time phase (phase 1)
    pub const MACRO_TIME: Phase = Phase(1);
    /// Meta-macro-time phase (phase 2)
    pub const META_MACRO_TIME: Phase = Phase(2);

    /// Creates a new phase
    pub fn new(level: i32) -> Self {
        Phase(level)
    }

    /// Gets the phase level
    pub fn level(&self) -> i32 {
        self.0
    }

    /// Increments the phase (for entering macro expansion)
    pub fn increment(&self) -> Phase {
        Phase(self.0 + 1)
    }

    /// Decrements the phase (for exiting macro expansion)
    pub fn decrement(&self) -> Phase {
        Phase(self.0 - 1)
    }

    /// Checks if this is a macro-time phase (> 0)
    pub fn is_macro_time(&self) -> bool {
        self.0 > 0
    }

    /// Checks if this is runtime phase (== 0)
    pub fn is_runtime(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            0 => write!(f, "runtime"),
            1 => write!(f, "macro-time"),
            n if n > 1 => write!(f, "meta-macro-time-{}", n - 1),
            n => write!(f, "phase-{n}"),
        }
    }
}

/// Macro-time evaluation environment with phase separation
#[derive(Debug, Clone)]
pub struct MacroTimeEnvironment {
    /// Current phase level
    current_phase: Phase,
    /// Phase-separated environments
    phase_environments: HashMap<Phase, Environment>,
    /// Compile-time bindings (available at macro-time)
    compile_time_bindings: HashMap<String, MacroTimeValue>,
    /// Template utilities registry
    template_utilities: TemplateUtilities,
    /// Syntax transformer cache
    transformer_cache: HashMap<String, CachedTransformer>,
    /// Debugging and introspection state
    debug_state: MacroDebugState,
    /// Statistics for performance monitoring
    stats: MacroTimeStats,
}

/// Values that can exist at macro-time
#[derive(Debug, Clone)]
pub enum MacroTimeValue {
    /// A syntax object
    Syntax(Box<SyntaxObject>),
    /// A list of syntax objects
    SyntaxList(Vec<SyntaxObject>),
    /// A compile-time procedure
    Procedure {
        /// Name of the procedure
        name: String,
        /// Arity (number of parameters) of the procedure
        arity: usize,
        /// Implementation of the procedure
        implementation: MacroTimeProcedure,
    },
    /// A compile-time constant (serialized as string representation)
    Constant(String),
    /// A template fragment
    Template(QuasisyntaxTemplate),
    /// Environment reference
    Environment(Phase),
    /// Meta-level data structure
    MetaData(HashMap<String, MacroTimeValue>),
}

/// Implementation of a compile-time procedure
#[derive(Debug, Clone)]
pub enum MacroTimeProcedure {
    /// Built-in procedure (implemented in Rust)
    Builtin(String),
    /// User-defined procedure (Scheme code evaluated at compile-time)
    UserDefined {
        /// Parameter names for the procedure
        parameters: Vec<String>,
        /// Body expression of the procedure
        body: Box<SyntaxObject>,
        /// Closure environment containing captured bindings
        closure_env: Box<HashMap<String, MacroTimeValue>>,
    },
    /// Template generator procedure
    TemplateGenerator {
        /// Pattern to match input against
        pattern: String,
        /// Template to generate output from
        template: String,
    },
}

/// Cached transformer for performance optimization
#[derive(Debug, Clone)]
struct CachedTransformer {
    transformer: UnifiedMacroTransformer,
    last_used: std::time::Instant,
    use_count: u64,
    cache_hits: u64,
}

/// Debug state for macro expansion introspection
#[derive(Debug, Clone)]
pub struct MacroDebugState {
    /// Stack of currently expanding macros
    expansion_stack: Vec<MacroExpansionFrame>,
    /// Trace of all expansions
    expansion_trace: Vec<MacroExpansionEvent>,
    /// Whether debugging is enabled
    debug_enabled: bool,
    /// Maximum trace size
    max_trace_size: usize,
}

/// Frame in the macro expansion stack
#[derive(Debug, Clone)]
pub struct MacroExpansionFrame {
    /// Name of the macro being expanded
    macro_name: String,
    /// Input syntax object
    input: SyntaxObject,
    /// Phase at which expansion is happening
    phase: Phase,
    /// Start time of expansion
    start_time: std::time::Instant,
    /// Nested expansion depth
    depth: usize,
}

/// Event in macro expansion trace
#[derive(Debug, Clone)]
pub struct MacroExpansionEvent {
    /// Type of event
    event_type: MacroEventType,
    /// Macro name
    macro_name: String,
    /// Phase
    phase: Phase,
    /// Timestamp
    timestamp: std::time::Instant,
    /// Additional data
    data: Option<String>,
}

/// Types of macro expansion events
#[derive(Debug, Clone)]
pub enum MacroEventType {
    /// Started expanding a macro
    ExpansionStart,
    /// Finished expanding a macro
    ExpansionEnd,
    /// Template instantiation
    TemplateInstantiation,
    /// Compile-time evaluation
    CompileTimeEval,
    /// Error occurred
    Error,
    /// Cache hit
    CacheHit,
    /// Cache miss
    CacheMiss,
}

/// Performance statistics for macro-time computation
#[derive(Debug, Clone, Default)]
pub struct MacroTimeStats {
    /// Total number of macro expansions
    total_expansions: u64,
    /// Number of compile-time evaluations
    compile_time_evals: u64,
    /// Number of template instantiations
    template_instantiations: u64,
    /// Cache hit ratio
    cache_hits: u64,
    /// Cache misses
    cache_misses: u64,
    /// Total time spent in macro expansion
    total_expansion_time: std::time::Duration,
    /// Time spent in template generation
    template_generation_time: std::time::Duration,
}

impl MacroTimeEnvironment {
    /// Creates a new macro-time environment
    pub fn new() -> Self {
        let mut env = Self {
            current_phase: Phase::RUNTIME,
            phase_environments: HashMap::new(),
            compile_time_bindings: HashMap::new(),
            template_utilities: TemplateUtilities::new(),
            transformer_cache: HashMap::new(),
            debug_state: MacroDebugState::new(),
            stats: MacroTimeStats::default(),
        };

        // Initialize runtime environment
        env.phase_environments
            .insert(Phase::RUNTIME, Environment::new(None, 0));

        // Initialize macro-time environment with built-in utilities
        let mut macro_env = Environment::new(None, 0);
        env.setup_builtin_utilities(&mut macro_env);
        env.phase_environments.insert(Phase::MACRO_TIME, macro_env);

        env
    }

    /// Sets up built-in compile-time utilities
    fn setup_builtin_utilities(&mut self, env: &mut Environment) {
        // Add built-in macro-time procedures
        self.add_builtin_procedure("make-list", 2);
        self.add_builtin_procedure("generate-temporaries", 1);
        self.add_builtin_procedure("syntax->datum", 1);
        self.add_builtin_procedure("datum->syntax", 2);
        self.add_builtin_procedure("syntax-length", 1);
        self.add_builtin_procedure("syntax-map", 2);
        self.add_builtin_procedure("syntax-append", 0); // variadic
        self.add_builtin_procedure("free-identifier=?", 2);
        self.add_builtin_procedure("bound-identifier=?", 2);
        self.add_builtin_procedure("identifier?", 1);
        self.add_builtin_procedure("syntax?", 1);
    }

    /// Adds a built-in procedure to the compile-time environment
    fn add_builtin_procedure(&mut self, name: &str, arity: usize) {
        let procedure = MacroTimeValue::Procedure {
            name: name.to_string(),
            arity,
            implementation: MacroTimeProcedure::Builtin(name.to_string()),
        };
        self.compile_time_bindings
            .insert(name.to_string(), procedure);
    }

    /// Enters a new phase level
    pub fn enter_phase(&mut self, phase: Phase) -> Phase {
        let previous_phase = self.current_phase;
        self.current_phase = phase;

        // Ensure environment exists for this phase
        if !self.phase_environments.contains_key(&phase) {
            let new_env = if phase.level() > 0 {
                // Macro-time phases inherit from the previous phase
                self.phase_environments
                    .get(&Phase::new(phase.level() - 1))
                    .cloned()
                    .unwrap_or_else(|| Environment::new(None, 0))
            } else {
                Environment::new(None, 0)
            };
            self.phase_environments.insert(phase, new_env);
        }

        previous_phase
    }

    /// Exits to the previous phase
    pub fn exit_phase(&mut self, previous_phase: Phase) {
        self.current_phase = previous_phase;
    }

    /// Gets the current phase
    pub fn current_phase(&self) -> Phase {
        self.current_phase
    }

    /// Gets the environment for a specific phase
    pub fn get_phase_environment(&self, phase: Phase) -> Option<&Environment> {
        self.phase_environments.get(&phase)
    }

    /// Gets a mutable reference to the environment for a specific phase
    pub fn get_phase_environment_mut(&mut self, phase: Phase) -> Option<&mut Environment> {
        self.phase_environments.get_mut(&phase)
    }

    /// Evaluates an expression at compile-time
    pub fn compile_time_eval(
        &mut self,
        expr: &SyntaxObject,
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        let start_time = std::time::Instant::now();
        self.stats.compile_time_evals += 1;

        // Enter macro-time phase for evaluation
        let previous_phase = self.enter_phase(Phase::MACRO_TIME);

        let result = match &expr.expr {
            Expr::Identifier(name) => {
                // Look up compile-time binding
                if let Some(value) = self.compile_time_bindings.get(name) {
                    Ok(value.clone())
                } else {
                    Err(Box::new(Error::MacroError {
                        message: format!("Unbound identifier in compile-time context: {name}"),
                        span: expr.span,
                    }))
                }
            }

            Expr::List(elements) if !elements.is_empty() => {
                // Procedure call at compile-time
                self.compile_time_procedure_call(elements, hygiene_env)
            }

            Expr::Literal(lit) => {
                // Literals evaluate to themselves
                Ok(MacroTimeValue::Constant(format!(
                    "{:?}",
                    Value::from_literal(lit.clone())
                )))
            }

            _ => Err(Box::new(Error::MacroError {
                message: "Unsupported expression in compile-time context".to_string(),
                span: expr.span,
            })),
        };

        self.exit_phase(previous_phase);

        // Update statistics
        let elapsed = start_time.elapsed();
        self.stats.total_expansion_time += elapsed;

        result
    }

    /// Handles compile-time procedure calls
    fn compile_time_procedure_call(
        &mut self,
        elements: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        let proc_expr = &elements[0];
        let args = &elements[1..];

        if let Expr::Identifier(proc_name) = &proc_expr.inner {
            if let Some(MacroTimeValue::Procedure { implementation, .. }) =
                self.compile_time_bindings.get(proc_name)
            {
                // Clone the implementation to avoid borrowing conflicts
                let implementation = implementation.clone();
                match implementation {
                    MacroTimeProcedure::Builtin(builtin_name) => {
                        self.call_builtin_procedure(&builtin_name, args, hygiene_env)
                    }
                    MacroTimeProcedure::UserDefined {
                        parameters,
                        body,
                        closure_env,
                    } => self.call_user_defined_procedure(
                        &parameters,
                        &body,
                        &closure_env,
                        args,
                        hygiene_env,
                    ),
                    MacroTimeProcedure::TemplateGenerator { pattern, template } => {
                        self.call_template_generator(&pattern, &template, args, hygiene_env)
                    }
                }
            } else {
                Err(Box::new(Error::MacroError {
                    message: format!("Unknown compile-time procedure: {proc_name}"),
                    span: proc_expr.span,
                }))
            }
        } else {
            Err(Box::new(Error::MacroError {
                message: "Invalid procedure expression in compile-time call".to_string(),
                span: proc_expr.span,
            }))
        }
    }

    /// Calls a built-in compile-time procedure
    fn call_builtin_procedure(
        &mut self,
        builtin_name: &str,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        match builtin_name {
            "make-list" => self.builtin_make_list(args, hygiene_env),
            "generate-temporaries" => self.builtin_generate_temporaries(args, hygiene_env),
            "syntax->datum" => self.builtin_syntax_to_datum(args, hygiene_env),
            "datum->syntax" => self.builtin_datum_to_syntax(args, hygiene_env),
            "syntax-length" => self.builtin_syntax_length(args, hygiene_env),
            "syntax-map" => self.builtin_syntax_map(args, hygiene_env),
            "syntax-append" => self.builtin_syntax_append(args, hygiene_env),
            "free-identifier=?" => self.builtin_free_identifier_equal(args, hygiene_env),
            "bound-identifier=?" => self.builtin_bound_identifier_equal(args, hygiene_env),
            "identifier?" => self.builtin_identifier_p(args, hygiene_env),
            "syntax?" => self.builtin_syntax_p(args, hygiene_env),
            _ => Err(Box::new(Error::MacroError {
                message: format!("Unknown built-in procedure: {builtin_name}"),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            })),
        }
    }

    /// Calls a user-defined compile-time procedure
    fn call_user_defined_procedure(
        &mut self,
        parameters: &[String],
        body: &SyntaxObject,
        closure_env: &HashMap<String, MacroTimeValue>,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        if args.len() != parameters.len() {
            return Err(Box::new(Error::MacroError {
                message: format!(
                    "Procedure expects {} arguments, got {}",
                    parameters.len(),
                    args.len()
                ),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            }));
        }

        // Set up local environment with parameters bound to arguments
        let previous_bindings = self.compile_time_bindings.clone();

        // Add closure environment
        for (name, value) in closure_env {
            self.compile_time_bindings
                .insert(name.clone(), value.clone());
        }

        // Bind parameters to arguments
        for (param, arg) in parameters.iter().zip(args.iter()) {
            let arg_syntax = SyntaxObject::from_spanned(
                arg.clone(),
                LexicalContext::new(0, vec!["macro-time".to_string()]),
            );
            self.compile_time_bindings
                .insert(param.clone(), MacroTimeValue::Syntax(Box::new(arg_syntax)));
        }

        // Evaluate the body
        let result = self.compile_time_eval(body, hygiene_env);

        // Restore previous bindings
        self.compile_time_bindings = previous_bindings;

        result
    }

    /// Calls a template generator procedure
    fn call_template_generator(
        &mut self,
        pattern: &str,
        template: &str,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        // This is a simplified implementation
        // A real implementation would parse the pattern and template strings
        // and perform proper pattern matching and template expansion

        if args.is_empty() {
            return Err(Box::new(Error::MacroError {
                message: "Template generator requires at least one argument".to_string(),
                span: Span::new(0, 0),
            }));
        }

        // For now, create a simple syntax object from the template
        let context = LexicalContext::new(0, vec!["template-generated".to_string()]);
        let syntax =
            syntax_utils::make_identifier_syntax(template.to_string(), args[0].span, context);

        Ok(MacroTimeValue::Syntax(Box::new(syntax)))
    }

    /// Creates a template from a compile-time computation result
    pub fn create_template(
        &mut self,
        result: MacroTimeValue,
        hygiene_env: &mut HygieneResolver,
    ) -> Result<QuasisyntaxTemplate> {
        let start_time = std::time::Instant::now();

        let template = match result {
            MacroTimeValue::Syntax(syntax) => QuasisyntaxTemplate::from_expr(&syntax.expr),
            MacroTimeValue::SyntaxList(syntaxes) => {
                let templates: Vec<_> = syntaxes
                    .iter()
                    .map(|s| QuasisyntaxTemplate::from_expr(&s.expr))
                    .collect();
                QuasisyntaxTemplate::List(templates)
            }
            MacroTimeValue::Template(template) => template,
            MacroTimeValue::Constant(value) => {
                // Convert string representation back to template
                // This is a simplified conversion - a real implementation would parse the string
                if value == "Nil" {
                    QuasisyntaxTemplate::Nil
                } else {
                    // For now, treat all constants as string literals
                    QuasisyntaxTemplate::Literal(Literal::string(value.clone()))
                }
            }
            _ => {
                return Err(Box::new(Error::MacroError {
                    message: "Cannot create template from this macro-time value".to_string(),
                    span: Span::new(0, 0),
                }));
            }
        };

        // Update statistics
        let elapsed = start_time.elapsed();
        self.stats.template_generation_time += elapsed;
        self.stats.template_instantiations += 1;

        Ok(template)
    }

    /// Generates syntax objects with splicing support
    pub fn generate_with_splicing(
        &mut self,
        templates: Vec<QuasisyntaxTemplate>,
        bindings: &HashMap<String, MacroTimeValue>,
        context: &LexicalContext,
        span: Span,
    ) -> Result<Vec<SyntaxObject>> {
        let mut result = Vec::new();

        for template in templates {
            match template {
                QuasisyntaxTemplate::UnquoteSplicing(inner) => {
                    // Handle splicing - this should expand to multiple syntax objects
                    let expanded =
                        self.expand_splicing_template(&inner, bindings, context, span)?;
                    result.extend(expanded);
                }
                _ => {
                    // Regular template - expand to single syntax object
                    let expanded =
                        self.expand_single_template(&template, bindings, context, span)?;
                    result.push(expanded);
                }
            }
        }

        Ok(result)
    }

    /// Expands a splicing template to multiple syntax objects
    fn expand_splicing_template(
        &mut self,
        template: &QuasisyntaxTemplate,
        bindings: &HashMap<String, MacroTimeValue>,
        context: &LexicalContext,
        span: Span,
    ) -> Result<Vec<SyntaxObject>> {
        match template {
            QuasisyntaxTemplate::PatternVariable(name) => {
                if let Some(MacroTimeValue::SyntaxList(syntaxes)) = bindings.get(name) {
                    Ok(syntaxes.clone())
                } else if let Some(MacroTimeValue::Syntax(syntax)) = bindings.get(name) {
                    // Single syntax object becomes a single-element list for splicing
                    Ok(vec![(**syntax).clone()])
                } else {
                    Err(Box::new(Error::MacroError {
                        message: format!("Unbound splicing variable: {name}"),
                        span,
                    }))
                }
            }
            _ => {
                // For other templates, expand as single and wrap in list
                let expanded = self.expand_single_template(template, bindings, context, span)?;
                Ok(vec![expanded])
            }
        }
    }

    /// Expands a single template to a syntax object
    #[allow(clippy::only_used_in_recursion)]
    fn expand_single_template(
        &mut self,
        template: &QuasisyntaxTemplate,
        bindings: &HashMap<String, MacroTimeValue>,
        context: &LexicalContext,
        span: Span,
    ) -> Result<SyntaxObject> {
        match template {
            QuasisyntaxTemplate::Literal(lit) => Ok(SyntaxObject::new(
                Expr::Literal(lit.clone()),
                span,
                context.clone(),
            )),
            QuasisyntaxTemplate::Identifier(name) => Ok(SyntaxObject::new(
                Expr::Identifier(name.clone()),
                span,
                context.clone(),
            )),
            QuasisyntaxTemplate::PatternVariable(name) => {
                if let Some(MacroTimeValue::Syntax(syntax)) = bindings.get(name) {
                    Ok((**syntax).clone())
                } else {
                    Err(Box::new(Error::MacroError {
                        message: format!("Unbound pattern variable: {name}"),
                        span,
                    }))
                }
            }
            QuasisyntaxTemplate::List(templates) => {
                let elements: Result<Vec<_>> = templates
                    .iter()
                    .map(|t| {
                        let expanded = self.expand_single_template(t, bindings, context, span)?;
                        Ok(expanded.to_spanned())
                    })
                    .collect();

                Ok(SyntaxObject::new(
                    Expr::List(elements?),
                    span,
                    context.clone(),
                ))
            }
            _ => Err(Box::new(Error::MacroError {
                message: "Unsupported template form".to_string(),
                span,
            })),
        }
    }

    /// Gets performance statistics
    pub fn get_stats(&self) -> &MacroTimeStats {
        &self.stats
    }

    /// Clears the transformer cache
    pub fn clear_cache(&mut self) {
        self.transformer_cache.clear();
        self.stats.cache_hits = 0;
        self.stats.cache_misses = 0;
    }

    /// Enables or disables debugging
    pub fn set_debug_enabled(&mut self, enabled: bool) {
        self.debug_state.debug_enabled = enabled;
    }

    /// Gets the current expansion trace
    pub fn get_expansion_trace(&self) -> &[MacroExpansionEvent] {
        &self.debug_state.expansion_trace
    }
}

impl Default for MacroTimeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroDebugState {
    /// Creates a new debug state
    fn new() -> Self {
        Self {
            expansion_stack: Vec::new(),
            expansion_trace: Vec::new(),
            debug_enabled: false,
            max_trace_size: 1000,
        }
    }
}

impl MacroTimeStats {
    /// Calculates cache hit ratio
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// Gets average expansion time
    pub fn average_expansion_time(&self) -> std::time::Duration {
        if self.total_expansions == 0 {
            std::time::Duration::ZERO
        } else {
            self.total_expansion_time / self.total_expansions as u32
        }
    }
}

/// Template utilities for common macro patterns
#[derive(Debug, Clone)]
pub struct TemplateUtilities {
    /// Counter for generating unique identifiers
    unique_counter: std::sync::Arc<AtomicU64>,
}

impl TemplateUtilities {
    /// Creates new template utilities
    fn new() -> Self {
        Self {
            unique_counter: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    /// Generates a unique temporary identifier
    pub fn generate_temporary(&self, base_name: &str) -> String {
        let id = self.unique_counter.fetch_add(1, Ordering::SeqCst);
        format!("{base_name}#{id}")
    }

    /// Generates multiple temporary identifiers
    pub fn generate_temporaries(&self, count: usize, base_name: &str) -> Vec<String> {
        (0..count)
            .map(|_| self.generate_temporary(base_name))
            .collect()
    }
}

// Implementation of built-in compile-time procedures
impl MacroTimeEnvironment {
    /// Built-in: (make-list n item)
    fn builtin_make_list(
        &mut self,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        if args.len() != 2 {
            return Err(Box::new(Error::MacroError {
                message: "make-list expects 2 arguments".to_string(),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            }));
        }

        // Evaluate first argument (count)
        let count_syntax = SyntaxObject::from_spanned(
            args[0].clone(),
            LexicalContext::new(0, vec!["make-list".to_string()]),
        );
        let count_value = self.compile_time_eval(&count_syntax, hygiene_env)?;

        let count = match count_value {
            MacroTimeValue::Constant(s) if s.parse::<f64>().is_ok() => {
                s.parse::<f64>().unwrap_or(0.0) as usize
            }
            _ => {
                return Err(Box::new(Error::MacroError {
                    message: "make-list: first argument must be a number".to_string(),
                    span: args[0].span,
                }));
            }
        };

        // Second argument is the item template
        let item_syntax = SyntaxObject::from_spanned(
            args[1].clone(),
            LexicalContext::new(0, vec!["make-list".to_string()]),
        );

        // Create list of syntax objects
        let list: Vec<SyntaxObject> = (0..count).map(|_| item_syntax.clone()).collect();

        Ok(MacroTimeValue::SyntaxList(list))
    }

    /// Built-in: (generate-temporaries stx-list)
    fn builtin_generate_temporaries(
        &mut self,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        if args.len() != 1 {
            return Err(Box::new(Error::MacroError {
                message: "generate-temporaries expects 1 argument".to_string(),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            }));
        }

        let input_syntax = SyntaxObject::from_spanned(
            args[0].clone(),
            LexicalContext::new(0, vec!["generate-temporaries".to_string()]),
        );

        let count = if let Some(list) = input_syntax.as_list() {
            list.len()
        } else {
            1
        };

        let temporaries = self.template_utilities.generate_temporaries(count, "tmp");
        let context = LexicalContext::new(0, vec!["generated".to_string()]);

        let syntax_list: Vec<SyntaxObject> = temporaries
            .into_iter()
            .map(|name| syntax_utils::make_identifier_syntax(name, args[0].span, context.clone()))
            .collect();

        Ok(MacroTimeValue::SyntaxList(syntax_list))
    }

    /// Built-in: (syntax->datum stx)
    fn builtin_syntax_to_datum(
        &mut self,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        if args.len() != 1 {
            return Err(Box::new(Error::MacroError {
                message: "syntax->datum expects 1 argument".to_string(),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            }));
        }

        let syntax_obj = SyntaxObject::from_spanned(
            args[0].clone(),
            LexicalContext::new(0, vec!["syntax->datum".to_string()]),
        );

        // Convert syntax object to its datum (the underlying expression)
        let datum = syntax_utils::syntax_to_datum(&syntax_obj);
        let value = Value::from_expr(&datum)?;

        Ok(MacroTimeValue::Constant(format!("{value:?}")))
    }

    /// Built-in: (datum->syntax template-identifier datum)
    fn builtin_datum_to_syntax(
        &mut self,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        if args.len() != 2 {
            return Err(Box::new(Error::MacroError {
                message: "datum->syntax expects 2 arguments".to_string(),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            }));
        }

        let template_syntax = SyntaxObject::from_spanned(
            args[0].clone(),
            LexicalContext::new(0, vec!["datum->syntax".to_string()]),
        );

        let datum_expr = &args[1].inner;
        let syntax_obj =
            syntax_utils::datum_to_syntax(datum_expr.clone(), Some(&template_syntax), args[1].span);

        Ok(MacroTimeValue::Syntax(Box::new(syntax_obj)))
    }

    /// Built-in: (syntax-length stx)
    fn builtin_syntax_length(
        &mut self,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        if args.len() != 1 {
            return Err(Box::new(Error::MacroError {
                message: "syntax-length expects 1 argument".to_string(),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            }));
        }

        let syntax_obj = SyntaxObject::from_spanned(
            args[0].clone(),
            LexicalContext::new(0, vec!["syntax-length".to_string()]),
        );

        let length = if let Some(list) = syntax_obj.as_list() {
            list.len()
        } else {
            return Err(Box::new(Error::MacroError {
                message: "syntax-length: argument must be a syntax list".to_string(),
                span: args[0].span,
            }));
        };

        Ok(MacroTimeValue::Constant(format!("{length}")))
    }

    /// Built-in: (syntax-map proc stx-list)
    fn builtin_syntax_map(
        &mut self,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        if args.len() != 2 {
            return Err(Box::new(Error::MacroError {
                message: "syntax-map expects 2 arguments".to_string(),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            }));
        }

        // This is a simplified implementation
        // A full implementation would apply the procedure to each element
        let syntax_obj = SyntaxObject::from_spanned(
            args[1].clone(),
            LexicalContext::new(0, vec!["syntax-map".to_string()]),
        );

        if let Some(list) = syntax_obj.as_list() {
            // For now, just return the list unchanged
            Ok(MacroTimeValue::SyntaxList(list))
        } else {
            Err(Box::new(Error::MacroError {
                message: "syntax-map: second argument must be a syntax list".to_string(),
                span: args[1].span,
            }))
        }
    }

    /// Built-in: (syntax-append stx-list ...)
    fn builtin_syntax_append(
        &mut self,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        let mut result = Vec::new();

        for arg in args {
            let syntax_obj = SyntaxObject::from_spanned(
                arg.clone(),
                LexicalContext::new(0, vec!["syntax-append".to_string()]),
            );

            if let Some(list) = syntax_obj.as_list() {
                result.extend(list);
            } else {
                result.push(syntax_obj);
            }
        }

        Ok(MacroTimeValue::SyntaxList(result))
    }

    /// Built-in: (free-identifier=? stx1 stx2)
    fn builtin_free_identifier_equal(
        &mut self,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        if args.len() != 2 {
            return Err(Box::new(Error::MacroError {
                message: "free-identifier=? expects 2 arguments".to_string(),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            }));
        }

        let stx1 = SyntaxObject::from_spanned(
            args[0].clone(),
            LexicalContext::new(0, vec!["free-identifier=?".to_string()]),
        );
        let stx2 = SyntaxObject::from_spanned(
            args[1].clone(),
            LexicalContext::new(0, vec!["free-identifier=?".to_string()]),
        );

        let equal = syntax_utils::free_identifier_equal(&stx1, &stx2);
        Ok(MacroTimeValue::Constant(equal.to_string()))
    }

    /// Built-in: (bound-identifier=? stx1 stx2)
    fn builtin_bound_identifier_equal(
        &mut self,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        if args.len() != 2 {
            return Err(Box::new(Error::MacroError {
                message: "bound-identifier=? expects 2 arguments".to_string(),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            }));
        }

        let stx1 = SyntaxObject::from_spanned(
            args[0].clone(),
            LexicalContext::new(0, vec!["bound-identifier=?".to_string()]),
        );
        let stx2 = SyntaxObject::from_spanned(
            args[1].clone(),
            LexicalContext::new(0, vec!["bound-identifier=?".to_string()]),
        );

        let equal = syntax_utils::bound_identifier_equal(&stx1, &stx2);
        Ok(MacroTimeValue::Constant(equal.to_string()))
    }

    /// Built-in: (identifier? stx)
    fn builtin_identifier_p(
        &mut self,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        if args.len() != 1 {
            return Err(Box::new(Error::MacroError {
                message: "identifier? expects 1 argument".to_string(),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            }));
        }

        let syntax_obj = SyntaxObject::from_spanned(
            args[0].clone(),
            LexicalContext::new(0, vec!["identifier?".to_string()]),
        );

        Ok(MacroTimeValue::Constant(
            syntax_obj.is_identifier().to_string(),
        ))
    }

    /// Built-in: (syntax? obj)
    fn builtin_syntax_p(
        &mut self,
        args: &[Spanned<Expr>],
        hygiene_env: &mut HygieneResolver,
    ) -> Result<MacroTimeValue> {
        if args.len() != 1 {
            return Err(Box::new(Error::MacroError {
                message: "syntax? expects 1 argument".to_string(),
                span: args.first().map(|a| a.span).unwrap_or(Span::new(0, 0)),
            }));
        }

        // For now, assume everything is syntax in this context
        Ok(MacroTimeValue::Constant("true".to_string()))
    }
}

// Helper trait to convert values
trait ValueConversion {
    fn from_literal(lit: Literal) -> Self;
    fn from_expr(expr: &Expr) -> Result<Self>
    where
        Self: Sized;
}

impl ValueConversion for Value {
    fn from_literal(lit: Literal) -> Self {
        match lit {
            lit => Value::Literal(lit.clone()),
            Literal::Nil => Value::Nil,
        }
    }

    fn from_expr(expr: &Expr) -> Result<Self> {
        match expr {
            Expr::Literal(lit) => Ok(Self::from_literal(lit.clone())),
            Expr::Identifier(name) => {
                // Simple hash-based symbol ID generation
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                name.hash(&mut hasher);
                Ok(Value::Symbol(SymbolId::new(hasher.finish() as usize)))
            }
            _ => Err(Box::new(Error::MacroError {
                message: "Cannot convert expression to value".to_string(),
                span: Span::new(0, 0),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Span;

    #[test]
    fn test_phase_management() {
        let runtime = Phase::RUNTIME;
        let macro_time = Phase::MACRO_TIME;
        let meta_macro = Phase::META_MACRO_TIME;

        assert_eq!(runtime.level(), 0);
        assert_eq!(macro_time.level(), 1);
        assert_eq!(meta_macro.level(), 2);

        assert!(runtime.is_runtime());
        assert!(!runtime.is_macro_time());

        assert!(!macro_time.is_runtime());
        assert!(macro_time.is_macro_time());

        assert_eq!(runtime.increment(), macro_time);
        assert_eq!(macro_time.decrement(), runtime);
    }

    #[test]
    fn test_macro_time_environment() {
        let mut env = MacroTimeEnvironment::new();

        assert_eq!(env.current_phase(), Phase::RUNTIME);

        let previous = env.enter_phase(Phase::MACRO_TIME);
        assert_eq!(env.current_phase(), Phase::MACRO_TIME);
        assert_eq!(previous, Phase::RUNTIME);

        env.exit_phase(previous);
        assert_eq!(env.current_phase(), Phase::RUNTIME);
    }

    #[test]
    fn test_builtin_make_list() {
        let mut env = MacroTimeEnvironment::new();
        let mut hygiene_env = HygieneResolver::new();

        let args = vec![
            Spanned::new(Expr::Literal(Literal::Number(3.0)), Span::new(0, 1)),
            Spanned::new(Expr::Identifier("x".to_string()), Span::new(2, 3)),
        ];

        let result = env.builtin_make_list(&args, &mut hygiene_env).unwrap();

        if let MacroTimeValue::SyntaxList(list) = result {
            assert_eq!(list.len(), 3);
            for syntax in list {
                assert_eq!(syntax.identifier_name(), Some("x"))
            }
        } else {
            panic!("Expected SyntaxList");
        }
    }

    #[test]
    fn test_template_utilities() {
        let utils = TemplateUtilities::new();

        let temp1 = utils.generate_temporary("test");
        let temp2 = utils.generate_temporary("test");

        assert!(temp1.starts_with("test#"));
        assert!(temp2.starts_with("test#"));
        assert_ne!(temp1, temp2);

        let temps = utils.generate_temporaries(3, "var");
        assert_eq!(temps.len(), 3);
        for temp in temps {
            assert!(temp.starts_with("var#"))
        }
    }

    #[test]
    fn test_macro_time_stats() {
        let mut stats = MacroTimeStats::default();

        assert_eq!(stats.cache_hit_ratio(), 0.0);

        stats.cache_hits = 7;
        stats.cache_misses = 3;
        assert_eq!(stats.cache_hit_ratio(), 0.7);
    }
}

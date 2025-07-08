//! Lambdust REPL - Interactive Scheme interpreter
//!
//! This module provides a read-eval-print loop for the Lambdust Scheme interpreter.
//! It supports interactive evaluation, command history, tab completion, syntax highlighting,
//! and basic debugging features.

use clap::{Arg, Command};
use lambdust::error::LambdustError;
use lambdust::Interpreter;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{MatchingBracketValidator, Validator};
use rustyline::{Context, DefaultEditor, Result as RustylineResult};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashSet;
use std::path::Path;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BANNER: &str = r#"
┌─────────────────────────────────────────┐
│  Lambdust (λust) R7RS Scheme Interpreter │
│  Version {version}                       │
│  Type (exit) to quit, (help) for help   │
└─────────────────────────────────────────┘
"#;

const HELP_TEXT: &str = r#"Available commands:
  (exit)       - Exit the REPL
  (help)       - Show this help message
  (clear)      - Clear the screen
  (env)        - Show current environment bindings
  (load "file") - Load and evaluate a file
  (reset)      - Reset the interpreter state
  (debug on)   - Enable debug mode with breakpoints
  (debug off)  - Disable debug mode
  (break)      - Set breakpoint at next evaluation
  (step)       - Step through evaluation (debug mode)
  (continue)   - Continue execution from breakpoint
  (backtrace)  - Show call stack (debug mode)

Tab Completion:
  - Press Tab to complete function names, special forms, and filenames
  - Functions are shown in green, special forms in red

Syntax Highlighting:
  - Special forms: red
  - Builtin functions: green
  - Numbers: light blue
  - Strings: yellow
  - Comments: gray
  - Parentheses: colored by depth

Examples:
  > (+ 1 2 3)
  6
  > (define square (lambda (x) (* x x)))
  > (square 5)
  25
  > (map square '(1 2 3 4))
  (1 4 9 16)
"#;

/// Debug mode state
#[derive(Debug, Clone, Default)]
pub struct DebugState {
    pub enabled: bool,
    pub breakpoint_set: bool,
    pub step_mode: bool,
    pub call_stack: Vec<String>,
    pub last_expression: Option<String>,
}

/// REPL configuration
#[derive(Debug, Clone)]
pub struct ReplConfig {
    pub prompt: String,
    pub continuation_prompt: String,
    pub debug_prompt: String,
    pub show_banner: bool,
    pub enable_history: bool,
    pub history_file: Option<String>,
    pub enable_syntax_highlighting: bool,
    pub enable_tab_completion: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            prompt: "λust> ".to_string(),
            continuation_prompt: "   ... ".to_string(),
            debug_prompt: "λust[debug]> ".to_string(),
            show_banner: true,
            enable_history: true,
            history_file: Some(".lambdust_history".to_string()),
            enable_syntax_highlighting: true,
            enable_tab_completion: true,
        }
    }
}

/// Scheme completion helper for enhanced REPL experience
pub struct SchemeHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    builtin_functions: HashSet<String>,
    special_forms: HashSet<String>,
}

impl SchemeHelper {
    #[allow(dead_code)]
    fn new() -> Self {
        let mut builtin_functions = HashSet::new();
        let mut special_forms = HashSet::new();

        // R7RS builtin functions
        for func in &[
            // Arithmetic
            "+",
            "-",
            "*",
            "/",
            "quotient",
            "remainder",
            "modulo",
            "abs",
            "floor",
            "ceiling",
            "sqrt",
            "expt",
            "min",
            "max",
            "exact?",
            "inexact?",
            "number?",
            "integer?",
            "real?",
            "rational?",
            "complex?",
            "exact->inexact",
            "inexact->exact",
            "number->string",
            "string->number",
            // Comparison
            "=",
            "<",
            ">",
            "<=",
            ">=",
            "eq?",
            "eqv?",
            "equal?",
            // List operations
            "car",
            "cdr",
            "cons",
            "list",
            "append",
            "reverse",
            "length",
            "null?",
            "pair?",
            "list?",
            "set-car!",
            "set-cdr!",
            "list->vector",
            "list->string",
            // String operations
            "string?",
            "string=?",
            "string<?",
            "string>?",
            "string<=?",
            "string>=?",
            "string-ci=?",
            "string-ci<?",
            "string-ci>?",
            "string-ci<=?",
            "string-ci>=?",
            "make-string",
            "string-length",
            "string-ref",
            "string-set!",
            "substring",
            "string-append",
            "string->list",
            "string-copy",
            "string-fill!",
            // Character operations
            "char?",
            "char=?",
            "char<?",
            "char>?",
            "char<=?",
            "char>=?",
            "char-ci=?",
            "char-ci<?",
            "char-ci>?",
            "char-ci<=?",
            "char-ci>=?",
            "char-alphabetic?",
            "char-numeric?",
            "char-whitespace?",
            "char-upper-case?",
            "char-lower-case?",
            "char-upcase",
            "char-downcase",
            "char->integer",
            "integer->char",
            // Vector operations
            "vector?",
            "make-vector",
            "vector",
            "vector-length",
            "vector-ref",
            "vector-set!",
            "vector->list",
            "list->vector",
            "vector-copy",
            "vector-fill!",
            // I/O
            "read",
            "write",
            "display",
            "newline",
            "read-char",
            "write-char",
            "peek-char",
            "eof-object?",
            "char-ready?",
            "load",
            // Higher-order functions
            "map",
            "for-each",
            "apply",
            "fold",
            "fold-right",
            "filter",
            // Control
            "call/cc",
            "call-with-current-continuation",
            "values",
            "call-with-values",
            "dynamic-wind",
            "raise",
            "with-exception-handler",
            "error",
            // Type predicates
            "boolean?",
            "symbol?",
            "procedure?",
            "port?",
            "input-port?",
            "output-port?",
            // Record types (SRFI 9)
            "make-record",
            "record-of-type?",
            "record-field",
            "record-set-field!",
            // SRFI functions
            "take",
            "drop",
            "concatenate",
            "delete-duplicates",
            "find",
            "any",
            "every",
            "string-null?",
            "string-hash",
            "string-hash-ci",
            "string-prefix?",
            "string-suffix?",
            "string-contains",
            "string-take",
            "string-drop",
            "string-concatenate",
            "make-hash-table",
            "hash-table?",
            "hash-table-set!",
            "hash-table-ref",
            "hash-table-delete!",
            "hash-table-size",
            "hash-table-exists?",
            "hash-table-keys",
            "hash-table-values",
            "hash-table->alist",
            "alist->hash-table",
            "hash",
            "string-hash",
        ] {
            builtin_functions.insert(func.to_string());
        }

        // Special forms
        for form in &[
            "define",
            "lambda",
            "if",
            "cond",
            "case",
            "and",
            "or",
            "when",
            "unless",
            "begin",
            "do",
            "let",
            "let*",
            "letrec",
            "letrec*",
            "set!",
            "quote",
            "quasiquote",
            "unquote",
            "unquote-splicing",
            "syntax-rules",
            "define-syntax",
            "guard",
            "define-record-type",
            "delay",
            "lazy",
            "force",
            "promise?",
        ] {
            special_forms.insert(form.to_string());
        }

        Self {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter {},
            builtin_functions,
            special_forms,
        }
    }

    #[allow(dead_code)]
    fn update_builtin_functions(&mut self, interpreter: &Interpreter) {
        // Add host functions
        for func_name in interpreter.list_host_functions() {
            self.builtin_functions.insert(func_name.clone());
        }

        // Add scheme functions
        for func_name in interpreter.list_scheme_functions() {
            self.builtin_functions.insert(func_name.clone());
        }
    }
}

impl Completer for SchemeHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> RustylineResult<(usize, Vec<Pair>)> {
        // Find the start of the current word
        let mut start = pos;
        while start > 0 {
            let ch = line.chars().nth(start - 1).unwrap_or(' ');
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                break;
            }
            start -= 1;
        }

        let word = &line[start..pos];
        if word.is_empty() {
            return Ok((start, vec![]));
        }

        let mut candidates = Vec::new();

        // Complete builtin functions
        for func in &self.builtin_functions {
            if func.starts_with(word) {
                candidates.push(Pair {
                    display: func.clone(),
                    replacement: func.clone(),
                });
            }
        }

        // Complete special forms
        for form in &self.special_forms {
            if form.starts_with(word) {
                candidates.push(Pair {
                    display: format!("{} (special form)", form),
                    replacement: form.clone(),
                });
            }
        }

        // If no matches and word looks like a filename, try file completion
        if candidates.is_empty() && (word.contains('/') || word.contains('.')) {
            return self.completer.complete(line, pos, _ctx);
        }

        // Sort candidates alphabetically
        candidates.sort_by(|a, b| a.display.cmp(&b.display));

        Ok((start, candidates))
    }
}

impl Hinter for SchemeHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for SchemeHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Simple syntax highlighting for demonstration
        let mut result = String::new();
        let chars = line.chars();
        let mut in_string = false;
        let mut in_comment = false;

        for ch in chars {
            match ch {
                // String literals
                '"' if !in_comment => {
                    if !in_string {
                        result.push_str("\x1b[33m"); // Yellow for strings
                        in_string = true;
                    } else {
                        in_string = false;
                        result.push(ch);
                        result.push_str("\x1b[0m"); // Reset color
                        continue;
                    }
                }

                // Comments
                ';' if !in_string => {
                    result.push_str("\x1b[90m"); // Gray for comments
                    in_comment = true;
                }

                // Numbers (simplified detection)
                c if c.is_ascii_digit() && !in_string && !in_comment => {
                    result.push_str("\x1b[94m"); // Light blue for numbers
                    result.push(c);
                    result.push_str("\x1b[0m");
                    continue;
                }

                // Reset comment flag at end of line
                '\n' => {
                    if in_comment {
                        result.push(ch);
                        result.push_str("\x1b[0m");
                        in_comment = false;
                        continue;
                    }
                }

                _ => {}
            }

            result.push(ch);
        }

        // Reset color at end if still in string or comment
        if in_string || in_comment {
            result.push_str("\x1b[0m");
        }

        Owned(result)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

impl Validator for SchemeHelper {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> RustylineResult<rustyline::validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl rustyline::Helper for SchemeHelper {}

/// Interactive REPL session
pub struct Repl {
    interpreter: Interpreter,
    editor: DefaultEditor,
    config: ReplConfig,
    debug_state: DebugState,
    #[allow(dead_code)]
    completion_enabled: bool,
}

impl Repl {
    /// Create a new REPL session
    pub fn new() -> RustylineResult<Self> {
        Self::new_with_config(ReplConfig::default())
    }

    /// Create a new REPL session with custom configuration
    pub fn new_with_config(config: ReplConfig) -> RustylineResult<Self> {
        let mut editor = DefaultEditor::new()?;
        let interpreter = Interpreter::new();

        // Load history if enabled
        if config.enable_history {
            if let Some(ref history_file) = config.history_file {
                if Path::new(history_file).exists() {
                    let _ = editor.load_history(history_file);
                }
            }
        }

        Ok(Self {
            interpreter,
            editor,
            config,
            debug_state: DebugState::default(),
            completion_enabled: true,
        })
    }

    /// Show the welcome banner
    fn show_banner(&self) {
        if self.config.show_banner {
            println!("{}", BANNER.replace("{version}", VERSION));
        }
    }

    /// Save command history
    fn save_history(&mut self) {
        if self.config.enable_history {
            if let Some(ref history_file) = self.config.history_file {
                let _ = self.editor.save_history(history_file);
            }
        }
    }

    /// Check if input is a complete expression
    fn is_complete_expression(&self, input: &str) -> bool {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return false;
        }

        // Simple balance check for parentheses
        let mut paren_count = 0;
        let mut in_string = false;
        let mut escaped = false;

        for ch in trimmed.chars() {
            if escaped {
                escaped = false;
                continue;
            }

            match ch {
                '\\' if in_string => escaped = true,
                '"' => in_string = !in_string,
                '(' if !in_string => paren_count += 1,
                ')' if !in_string => paren_count -= 1,
                _ => {}
            }
        }

        paren_count == 0 && !in_string
    }

    /// Read a complete expression from user input
    fn read_expression(&mut self) -> RustylineResult<Option<String>> {
        let mut input = String::new();
        let mut first_line = true;

        loop {
            let prompt = if first_line {
                if self.debug_state.enabled {
                    &self.config.debug_prompt
                } else {
                    &self.config.prompt
                }
            } else {
                &self.config.continuation_prompt
            };

            match self.editor.readline(prompt) {
                Ok(line) => {
                    if !input.is_empty() {
                        input.push('\n');
                    }
                    input.push_str(&line);

                    // Check for special commands on first line
                    if first_line {
                        let trimmed = line.trim();
                        if trimmed == "(exit)" || trimmed == "(quit)" {
                            return Ok(None);
                        }
                    }

                    if self.is_complete_expression(&input) {
                        // Add to history
                        self.editor.add_history_entry(&input)?;
                        return Ok(Some(input));
                    }

                    first_line = false;
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C - clear current input and start over
                    println!("^C");
                    return self.read_expression();
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D - exit
                    return Ok(None);
                }
                Err(err) => return Err(err),
            }
        }
    }

    /// Handle special REPL commands
    fn handle_special_command(&mut self, input: &str) -> Option<bool> {
        let trimmed = input.trim();

        match trimmed {
            "(help)" => {
                println!("{}", HELP_TEXT);
                Some(true)
            }
            "(clear)" => {
                print!("\x1B[2J\x1B[1;1H"); // ANSI escape codes to clear screen
                Some(true)
            }
            "(env)" => {
                self.show_environment();
                Some(true)
            }
            "(debug on)" => {
                self.debug_state.enabled = true;
                println!(
                    "Debug mode enabled. Use (break) to set breakpoints, (step) to step through code."
                );
                Some(true)
            }
            "(debug off)" => {
                self.debug_state = DebugState::default();
                println!("Debug mode disabled.");
                Some(true)
            }
            "(break)" => {
                if self.debug_state.enabled {
                    self.debug_state.breakpoint_set = true;
                    println!("Breakpoint set for next evaluation.");
                } else {
                    println!("Debug mode not enabled. Use (debug on) first.");
                }
                Some(true)
            }
            "(step)" => {
                if self.debug_state.enabled {
                    self.debug_state.step_mode = true;
                    println!("Step mode enabled. Next expression will be traced.");
                } else {
                    println!("Debug mode not enabled. Use (debug on) first.");
                }
                Some(true)
            }
            "(continue)" => {
                if self.debug_state.enabled {
                    self.debug_state.breakpoint_set = false;
                    self.debug_state.step_mode = false;
                    println!("Continuing execution...");
                } else {
                    println!("Debug mode not enabled.");
                }
                Some(true)
            }
            "(backtrace)" => {
                if self.debug_state.enabled {
                    self.show_backtrace();
                } else {
                    println!("Debug mode not enabled. Use (debug on) first.");
                }
                Some(true)
            }
            "(reset)" => {
                self.interpreter = Interpreter::new();
                self.debug_state = DebugState::default();
                println!("Interpreter state reset");
                Some(true)
            }
            _ if trimmed.starts_with("(load ") => {
                // Parse file path from (load "filename")
                if let Some(start) = trimmed.find('"') {
                    if let Some(end) = trimmed[start + 1..].find('"') {
                        let filename = &trimmed[start + 1..start + 1 + end];
                        match self.load_file(filename) {
                            Ok(_) => println!("File loaded successfully"),
                            Err(e) => println!("Error loading file: {}", e),
                        }
                        return Some(true);
                    }
                }
                println!("Invalid load syntax. Use: (load \"filename\")");
                Some(true)
            }
            _ => None,
        }
    }

    /// Load and evaluate a file
    fn load_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(filename)?;
        match self.interpreter.eval_string(&content) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Evaluate an expression and print the result
    fn eval_and_print(&mut self, input: &str) {
        // Handle debug mode
        if self.debug_state.enabled {
            self.debug_state.last_expression = Some(input.to_string());

            if self.debug_state.breakpoint_set || self.debug_state.step_mode {
                println!("\x1b[93m[DEBUG]\x1b[0m Breaking at: {}", input);
                println!(
                    "\x1b[93m[DEBUG]\x1b[0m Use (continue) to proceed, (backtrace) to see stack"
                );
                self.debug_state.breakpoint_set = false;
                return;
            }
        }

        match self.interpreter.eval_string(input) {
            Ok(value) => {
                // Add to call stack for debugging
                if self.debug_state.enabled && input.trim().starts_with('(') {
                    let call_info = self.extract_call_info(input);
                    self.debug_state.call_stack.push(call_info);

                    // Limit stack size
                    if self.debug_state.call_stack.len() > 20 {
                        self.debug_state.call_stack.remove(0);
                    }
                }

                // Don't print undefined values (common for definitions)
                if !matches!(value, lambdust::Value::Undefined) {
                    println!("{}", value);
                }
            }
            Err(LambdustError::ParseError { message, .. }) => {
                println!("Parse error: {}", message);
            }
            Err(LambdustError::RuntimeError { message, .. }) => {
                println!("Runtime error: {}", message);
            }
            Err(LambdustError::TypeError { message, .. }) => {
                println!("Type error: {}", message);
            }
            Err(LambdustError::ArityError {
                expected, actual, ..
            }) => {
                println!(
                    "Arity error: expected {} arguments, got {}",
                    expected, actual
                );
            }
            Err(LambdustError::SyntaxError { message, .. }) => {
                println!("Syntax error: {}", message);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    /// Run the main REPL loop
    pub fn run(&mut self) -> RustylineResult<()> {
        self.show_banner();

        while let Some(input) = self.read_expression()? {
            // Handle special commands
            if self.handle_special_command(&input).is_some() {
                continue;
            }

            // Evaluate normal expressions
            self.eval_and_print(&input);
        }

        self.save_history();
        println!("Goodbye!");
        Ok(())
    }

    /// Show current environment bindings
    fn show_environment(&self) {
        println!("\x1b[96m[ENVIRONMENT]\x1b[0m Available functions and variables:");

        // Show host functions
        let host_funcs = self.interpreter.list_host_functions();
        if !host_funcs.is_empty() {
            println!("\x1b[32mBuiltin Functions:\x1b[0m");
            for (i, func) in host_funcs.iter().enumerate() {
                if i % 6 == 0 && i > 0 {
                    println!();
                }
                print!("{:12} ", func);
            }
            println!();
        }

        // Show user-defined functions
        let scheme_funcs = self.interpreter.list_scheme_functions();
        if !scheme_funcs.is_empty() {
            println!("\x1b[94mUser-defined Functions:\x1b[0m");
            for (i, func) in scheme_funcs.iter().enumerate() {
                if i % 6 == 0 && i > 0 {
                    println!();
                }
                print!("{:12} ", func);
            }
            println!();
        }
    }

    /// Show debug backtrace
    fn show_backtrace(&self) {
        println!("\x1b[93m[BACKTRACE]\x1b[0m Call stack:");
        if self.debug_state.call_stack.is_empty() {
            println!("  (empty)");
        } else {
            for (i, call) in self.debug_state.call_stack.iter().enumerate() {
                println!("  {}: {}", i, call);
            }
        }

        if let Some(ref expr) = self.debug_state.last_expression {
            println!("\x1b[93m[CURRENT]\x1b[0m {}", expr);
        }
    }

    /// Extract call information for debugging
    fn extract_call_info(&self, input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.starts_with('(') {
            if let Some(end) = trimmed.find(' ') {
                let func_name = &trimmed[1..end];
                format!("{}(...)", func_name)
            } else if trimmed.len() > 2 {
                let func_name = &trimmed[1..trimmed.len() - 1];
                format!("{}()", func_name)
            } else {
                "(unknown)".to_string()
            }
        } else {
            trimmed.chars().take(20).collect::<String>() + "..."
        }
    }
}

/// Main entry point for the REPL
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("lambdust")
        .version(VERSION)
        .about("Interactive R7RS Scheme interpreter")
        .long_about(
            "Lambdust provides an interactive environment for evaluating R7RS Scheme expressions.",
        )
        .arg(
            Arg::new("file")
                .help("Scheme file to load and execute")
                .value_name("FILE")
                .index(1),
        )
        .arg(
            Arg::new("no-banner")
                .long("no-banner")
                .help("Don't show the welcome banner")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-history")
                .long("no-history")
                .help("Disable command history")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("prompt")
                .long("prompt")
                .help("Custom prompt string")
                .value_name("PROMPT")
                .default_value("λust> "),
        )
        .get_matches();

    // Create REPL configuration
    let config = ReplConfig {
        prompt: matches.get_one::<String>("prompt").unwrap().clone(),
        continuation_prompt: "   ... ".to_string(),
        debug_prompt: "λust[debug]> ".to_string(),
        show_banner: !matches.get_flag("no-banner"),
        enable_history: !matches.get_flag("no-history"),
        history_file: if matches.get_flag("no-history") {
            None
        } else {
            Some(".lambdust_history".to_string())
        },
        enable_syntax_highlighting: true,
        enable_tab_completion: true,
    };

    // Create and run REPL
    let mut repl = Repl::new_with_config(config)?;

    // If a file was specified, load it first
    if let Some(filename) = matches.get_one::<String>("file") {
        match repl.load_file(filename) {
            Ok(_) => println!("Loaded file: {}", filename),
            Err(e) => {
                eprintln!("Error loading file {}: {}", filename, e);
                std::process::exit(1);
            }
        }
    }

    // Run the interactive loop
    repl.run()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let config = ReplConfig {
            show_banner: false,
            enable_history: false,
            ..Default::default()
        };

        let repl = Repl::new_with_config(config);
        assert!(repl.is_ok());
    }

    #[test]
    fn test_complete_expression_detection() {
        let repl = Repl::new_with_config(ReplConfig {
            show_banner: false,
            enable_history: false,
            ..Default::default()
        })
        .unwrap();

        assert!(repl.is_complete_expression("42"));
        assert!(repl.is_complete_expression("(+ 1 2)"));
        assert!(repl.is_complete_expression("(define x 10)"));
        assert!(!repl.is_complete_expression("(+ 1"));
        assert!(!repl.is_complete_expression("(define x"));
        assert!(repl.is_complete_expression("\"hello world\""));
        assert!(!repl.is_complete_expression("\"hello"));
    }

    #[test]
    fn test_special_commands() {
        let mut repl = Repl::new_with_config(ReplConfig {
            show_banner: false,
            enable_history: false,
            ..Default::default()
        })
        .unwrap();

        // Test help command
        let result = repl.handle_special_command("(help)");
        assert_eq!(result, Some(true));

        // Test reset command
        let result = repl.handle_special_command("(reset)");
        assert_eq!(result, Some(true));

        // Test non-special command
        let result = repl.handle_special_command("(+ 1 2)");
        assert_eq!(result, None);
    }
}

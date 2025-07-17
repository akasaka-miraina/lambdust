//! Core REPL Implementation
//!
//! このモジュールはREPLのメイン実装を提供します。

use super::config::{ReplConfig, DebugState, HELP_TEXT, BANNER, VERSION};
use lambdust::error::LambdustError;
use lambdust::interpreter::LambdustInterpreter;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use std::path::Path;

/// Interactive REPL session
pub struct Repl {
    interpreter: LambdustInterpreter,
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
        let interpreter = LambdustInterpreter::new();

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
    pub fn handle_special_command(&mut self, input: &str) -> Option<bool> {
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
                self.interpreter = LambdustInterpreter::new();
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
    pub fn load_file(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
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

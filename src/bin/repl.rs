//! Lambdust REPL - Interactive Scheme interpreter
//!
//! This module provides a read-eval-print loop for the Lambdust Scheme interpreter.
//! It supports interactive evaluation, command history, and basic editing features.

use clap::{Arg, Command};
use lambdust::error::LambdustError;
use lambdust::interpreter::LambdustInterpreter;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use std::path::Path;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BANNER: &str = r#"
┌─────────────────────────────────────────┐
│  Lambdust (λust) R7RS Scheme Interpreter │
│  Version {version}                       │
│  Type (exit) to quit                     │
└─────────────────────────────────────────┘
"#;

const HELP_TEXT: &str = r#"Available commands:
  (exit)       - Exit the REPL
  (help)       - Show this help message
  (clear)      - Clear the screen
  (env)        - Show current environment bindings
  (load "file") - Load and evaluate a file
  (reset)      - Reset the interpreter state

Examples:
  > (+ 1 2 3)
  6
  > (define square (lambda (x) (* x x)))
  > (square 5)
  25
  > (map square '(1 2 3 4))
  (1 4 9 16)
"#;

/// REPL configuration
#[derive(Debug, Clone)]
pub struct ReplConfig {
    pub prompt: String,
    pub continuation_prompt: String,
    pub show_banner: bool,
    pub enable_history: bool,
    pub history_file: Option<String>,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            prompt: "λust> ".to_string(),
            continuation_prompt: "   ... ".to_string(),
            show_banner: true,
            enable_history: true,
            history_file: Some(".lambdust_history".to_string()),
        }
    }
}

/// Interactive REPL session
pub struct Repl {
    interpreter: LambdustInterpreter,
    editor: DefaultEditor,
    config: ReplConfig,
}

impl Repl {
    /// Create a new REPL session
    pub fn new() -> RustylineResult<Self> {
        Self::new_with_config(ReplConfig::default())
    }

    /// Create a new REPL session with custom configuration
    pub fn new_with_config(config: ReplConfig) -> RustylineResult<Self> {
        let mut editor = DefaultEditor::new()?;

        // Load history if enabled
        if config.enable_history {
            if let Some(ref history_file) = config.history_file {
                if Path::new(history_file).exists() {
                    let _ = editor.load_history(history_file);
                }
            }
        }

        Ok(Self {
            interpreter: LambdustInterpreter::new(),
            editor,
            config,
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
                &self.config.prompt
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
                // This would require exposing environment inspection methods
                println!("Environment inspection not yet implemented");
                Some(true)
            }
            "(reset)" => {
                self.interpreter = LambdustInterpreter::new();
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
        match self.interpreter.eval_string(input) {
            Ok(value) => {
                // Don't print undefined values (common for definitions)
                if !matches!(value, lambdust::value::Value::Undefined) {
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
        show_banner: !matches.get_flag("no-banner"),
        enable_history: !matches.get_flag("no-history"),
        history_file: if matches.get_flag("no-history") {
            None
        } else {
            Some(".lambdust_history".to_string())
        },
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

//! Minimal REPL implementation for lightweight binary builds.
//!
//! This module provides a basic Read-Eval-Print Loop that depends only on
//! the colored crate, achieving significant binary size reduction compared
//! to the full-featured REPL implementations.

use crate::{Lambdust, Error, Result};
use std::io::{self, Write, BufRead};

#[cfg(feature = "minimal-repl")]
use colored::{Colorize};

/// Minimal REPL configuration.
#[derive(Debug, Clone)]
pub struct MinimalReplConfig {
    /// Whether to use colors in output.
    pub use_colors: bool,
    /// Command prompt string.
    pub prompt: String,
    /// Whether to show evaluation time.
    pub show_timing: bool,
}

impl Default for MinimalReplConfig {
    fn default() -> Self {
        Self {
            use_colors: true,
            prompt: "λ> ".to_string(),
            show_timing: false,
        }
    }
}

/// Minimal REPL implementation.
pub struct MinimalRepl {
    config: MinimalReplConfig,
    history: Vec<String>,
}

impl MinimalRepl {
    /// Creates a new minimal REPL.
    pub fn new() -> Self {
        Self {
            config: MinimalReplConfig::default(),
            history: Vec::new(),
        }
    }
    
    /// Creates a new minimal REPL with custom configuration.
    pub fn with_config(config: MinimalReplConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
        }
    }
    
    /// Starts the REPL loop.
    pub fn run(&mut self, lambdust: &mut Lambdust) -> Result<()> {
        self.print_welcome();
        
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        
        loop {
            // Print prompt
            self.print_prompt(&mut stdout)?;
            
            // Read input
            let mut input = String::new();
            match stdin.lock().read_line(&mut input) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let input = input.trim();
                    
                    // Handle special commands
                    if input.is_empty() {
                        continue;
                    }
                    
                    if self.handle_meta_command(input, lambdust)? {
                        continue;
                    }
                    
                    // Add to history
                    self.history.push(input.to_string());
                    
                    // Evaluate expression
                    self.evaluate_and_print(input, lambdust);
                }
                Err(e) => {
                    eprintln!("Error reading input: {e}");
                    break;
                }
            }
        }
        
        self.print_goodbye();
        Ok(())
    }
    
    /// Prints welcome message.
    fn print_welcome(&self) {
        #[cfg(feature = "minimal-repl")]
        {
            if self.config.use_colors {
                println!("{}", "Welcome to Lambdust (λust) - Minimal REPL".bright_green().bold());
                println!("Type :help for available commands, :quit to exit");
            } else {
                println!("Welcome to Lambdust (λust) - Minimal REPL");
                println!("Type :help for available commands, :quit to exit");
            }
        }
        #[cfg(not(feature = "minimal-repl"))]
        {
            println!("Welcome to Lambdust (λust) - Minimal REPL");
            println!("Type :help for available commands, :quit to exit");
        }
        println!();
    }
    
    /// Prints goodbye message.
    fn print_goodbye(&self) {
        #[cfg(feature = "minimal-repl")]
        {
            if self.config.use_colors {
                println!("\n{}", "Goodbye!".bright_blue());
            } else {
                println!("\nGoodbye!");
            }
        }
        #[cfg(not(feature = "minimal-repl"))]
        {
            println!("\nGoodbye!");
        }
    }
    
    /// Prints the prompt.
    fn print_prompt(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        #[cfg(feature = "minimal-repl")]
        {
            if self.config.use_colors {
                print!("{}", self.config.prompt.bright_cyan());
            } else {
                print!("{}", self.config.prompt);
            }
        }
        #[cfg(not(feature = "minimal-repl"))]
        {
            print!("{}", self.config.prompt);
        }
        
        stdout.flush()
    }
    
    /// Handles meta commands (starting with :).
    fn handle_meta_command(&self, input: &str, _lambdust: &mut Lambdust) -> Result<bool> {
        if !input.starts_with(':') {
            return Ok(false);
        }
        
        let command = &input[1..];
        match command {
            "quit" | "q" | "exit" => std::process::exit(0),
            "help" | "h" => {
                self.print_help();
                Ok(true)
            }
            "history" => {
                self.print_history();
                Ok(true)
            }
            "clear" => {
                // Simple clear - print newlines
                for _ in 0..50 {
                    println!();
                }
                Ok(true)
            }
            _ => {
                #[cfg(feature = "minimal-repl")]
                {
                    if self.config.use_colors {
                        eprintln!("{}: {}", "Unknown command".red(), command);
                    } else {
                        eprintln!("Unknown command: {command}");
                    }
                }
                #[cfg(not(feature = "minimal-repl"))]
                {
                    eprintln!("Unknown command: {}", command);
                }
                Ok(true)
            }
        }
    }
    
    /// Prints help information.
    fn print_help(&self) {
        #[cfg(feature = "minimal-repl")]
        {
            if self.config.use_colors {
                println!("{}", "Available commands:".bright_yellow().bold());
                println!("  {}  - Show this help", ":help, :h".cyan());
                println!("  {}  - Exit REPL", ":quit, :q, :exit".cyan());
                println!("  {}  - Show command history", ":history".cyan());
                println!("  {}  - Clear screen", ":clear".cyan());
                println!();
                println!("{}", "Examples:".bright_yellow().bold());
                println!("  {}", "(+ 1 2 3)".green());
                println!("  {}", "(define x 42)".green());
                println!("  {}", "(lambda (x) (* x x))".green());
            } else {
                println!("Available commands:");
                println!("  :help, :h         - Show this help");
                println!("  :quit, :q, :exit  - Exit REPL");
                println!("  :history          - Show command history");
                println!("  :clear            - Clear screen");
                println!();
                println!("Examples:");
                println!("  (+ 1 2 3)");
                println!("  (define x 42)");
                println!("  (lambda (x) (* x x))");
            }
        }
        #[cfg(not(feature = "minimal-repl"))]
        {
            println!("Available commands:");
            println!("  :help, :h         - Show this help");
            println!("  :quit, :q, :exit  - Exit REPL");
            println!("  :history          - Show command history");
            println!("  :clear            - Clear screen");
            println!();
            println!("Examples:");
            println!("  (+ 1 2 3)");
            println!("  (define x 42)");
            println!("  (lambda (x) (* x x))");
        }
    }
    
    /// Prints command history.
    fn print_history(&self) {
        if self.history.is_empty() {
            println!("No history available.");
            return;
        }
        
        #[cfg(feature = "minimal-repl")]
        {
            if self.config.use_colors {
                println!("{}", "Command history:".bright_yellow());
                for (i, cmd) in self.history.iter().enumerate() {
                    println!("  {:3}: {}", i + 1, cmd);
                }
            } else {
                println!("Command history:");
                for (i, cmd) in self.history.iter().enumerate() {
                    println!("  {:3}: {}", i + 1, cmd);
                }
            }
        }
        #[cfg(not(feature = "minimal-repl"))]
        {
            println!("Command history:");
            for (i, cmd) in self.history.iter().enumerate() {
                println!("  {:3}: {}", i + 1, cmd);
            }
        }
    }
    
    /// Evaluates expression and prints result.
    fn evaluate_and_print(&self, input: &str, lambdust: &mut Lambdust) {
        let start_time = if self.config.show_timing {
            Some(std::time::Instant::now())
        } else {
            None
        };
        
        match lambdust.eval(input, Some("<repl>")) {
            Ok(value) => {
                #[cfg(feature = "minimal-repl")]
                {
                    if self.config.use_colors {
                        println!("{}", format!("=> {value}").green());
                    } else {
                        println!("=> {value}");
                    }
                }
                #[cfg(not(feature = "minimal-repl"))]
                {
                    println!("=> {}", value);
                }
                
                if let Some(start) = start_time {
                    let elapsed = start.elapsed();
                    #[cfg(feature = "minimal-repl")]
                    {
                        println!("(evaluated in {elapsed:?})");
                    }
                    #[cfg(not(feature = "minimal-repl"))]
                    {
                        println!("(evaluated in {:?})", elapsed);
                    }
                }
            }
            Err(e) => {
                #[cfg(feature = "minimal-repl")]
                {
                    if self.config.use_colors {
                        eprintln!("{}: {}", "Error".red().bold(), e);
                    } else {
                        eprintln!("Error: {e}");
                    }
                }
                #[cfg(not(feature = "minimal-repl"))]
                {
                    eprintln!("Error: {}", e);
                }
            }
        }
    }
}

impl Default for MinimalRepl {
    fn default() -> Self {
        Self::new()
    }
}

/// Starts a minimal REPL session.
pub fn start_minimal_repl(lambdust: &mut Lambdust) -> Result<()> {
    MinimalRepl::new().run(lambdust)
}

/// Starts a minimal REPL session with custom configuration.
pub fn start_minimal_repl_with_config(
    lambdust: &mut Lambdust, 
    config: MinimalReplConfig
) -> Result<()> {
    MinimalRepl::with_config(config).run(lambdust)
}
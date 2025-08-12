use super::{
    ReplConfig, EnhancedEditor, HistoryManager, Debugger, 
    CompletionProvider, SyntaxHighlighter, CodeInspector, SessionManager
};
use crate::{Lambdust, Result};

#[cfg(feature = "repl")]
use colored::*;

/// The main enhanced REPL interface
pub struct EnhancedRepl {
    config: ReplConfig,
    lambdust: Lambdust,
    editor: EnhancedEditor,
    history: HistoryManager,
    debugger: Debugger,
    completion: CompletionProvider,
    highlighter: SyntaxHighlighter,
    inspector: CodeInspector,
    session: SessionManager,
    line_number: usize,
}

impl EnhancedRepl {
    /// Creates a new enhanced REPL with the given configuration
    pub fn new(lambdust: Lambdust, config: ReplConfig) -> Result<Self> {
        let history = HistoryManager::new(config.max_history)?;
        let debugger = Debugger::new();
        let completion = CompletionProvider::new(&lambdust)?;
        let highlighter = SyntaxHighlighter::new()?;
        let inspector = CodeInspector::new();
        let session = SessionManager::new()?;
        let editor = EnhancedEditor::new(config.clone())?;

        Ok(Self {
            config,
            lambdust,
            editor,
            history,
            debugger,
            completion,
            highlighter,
            inspector,
            session,
            line_number: 1,
        })
    }
    
    /// Creates a new enhanced REPL with default configuration
    pub fn with_defaults(lambdust: Lambdust) -> Result<Self> {
        Self::new(lambdust, ReplConfig::default())
    }
    
    /// Gets the current configuration.
    pub fn config(&self) -> &ReplConfig {
        &self.config
    }
    
    /// Gets a reference to the Lambdust instance.
    pub fn lambdust(&self) -> &Lambdust {
        &self.lambdust
    }
    
    /// Gets a mutable reference to the Lambdust instance.
    pub fn lambdust_mut(&mut self) -> &mut Lambdust {
        &mut self.lambdust
    }
    
    /// Gets the editor.
    pub fn editor(&self) -> &EnhancedEditor {
        &self.editor
    }
    
    /// Gets a mutable reference to the editor.
    pub fn editor_mut(&mut self) -> &mut EnhancedEditor {
        &mut self.editor
    }
    
    /// Gets the history manager.
    pub fn history(&self) -> &HistoryManager {
        &self.history
    }
    
    /// Gets a mutable reference to the history manager.
    pub fn history_mut(&mut self) -> &mut HistoryManager {
        &mut self.history
    }
    
    /// Gets the debugger.
    pub fn debugger(&self) -> &Debugger {
        &self.debugger
    }
    
    /// Gets a mutable reference to the debugger.
    pub fn debugger_mut(&mut self) -> &mut Debugger {
        &mut self.debugger
    }
    
    /// Gets the completion provider.
    pub fn completion(&self) -> &CompletionProvider {
        &self.completion
    }
    
    /// Gets the syntax highlighter.
    pub fn highlighter(&self) -> &SyntaxHighlighter {
        &self.highlighter
    }
    
    /// Gets the code inspector.
    pub fn inspector(&self) -> &CodeInspector {
        &self.inspector
    }
    
    /// Gets the session manager.
    pub fn session(&self) -> &SessionManager {
        &self.session
    }
    
    /// Gets a mutable reference to the session manager.
    pub fn session_mut(&mut self) -> &mut SessionManager {
        &mut self.session
    }
    
    /// Gets the current line number.
    pub fn line_number(&self) -> usize {
        self.line_number
    }
    
    /// Increments the line number.
    pub fn increment_line_number(&mut self) {
        self.line_number += 1;
    }
    
    /// Sets the line number.
    pub fn set_line_number(&mut self, line_number: usize) {
        self.line_number = line_number;
    }

    /// Runs the enhanced REPL main loop.
    pub fn run(&mut self) -> Result<()> {
        #[cfg(feature = "repl")]
        {
            println!("{}", format!("Lambdust {} Enhanced REPL", crate::VERSION).bright_blue().bold());
            println!("{}", "Type :help for available commands or (exit) to quit".dimmed());
        }
        #[cfg(not(feature = "repl"))]
        {
            println!("Lambdust {} Enhanced REPL", crate::VERSION);
            println!("Type :help for available commands or (exit) to quit");
        }
        println!();

        loop {
            let prompt = format!("λust:{line}> ", line = self.line_number);
            
            match self.editor.read_line(&prompt, &mut self.completion, &self.highlighter) {
                Ok(Some(line)) => {
                    let line = line.trim();
                    
                    if line.is_empty() {
                        continue;
                    }

                    // Handle special REPL commands
                    if let Some(result) = self.handle_repl_command(line)? {
                        if result {
                            // Command handled successfully
                            continue;
                        } else {
                            // Exit command
                            break;
                        }
                    }

                    // Add to history
                    self.history.add_entry(line.to_string());
                    
                    // Add to session
                    match self.evaluate_expression(line) {
                        Ok(result) => {
                            // Don't print the result if it's unspecified (e.g., from display, set!, etc.)
                            if !matches!(result, crate::eval::Value::Unspecified) {
                                #[cfg(feature = "repl")]
                                println!("{}", format!("{result}").bright_green());
                                #[cfg(not(feature = "repl"))]
                                println!("{result}");
                                self.session.add_command(line.to_string(), Some(result.to_string()), None)?;
                            } else {
                                self.session.add_command(line.to_string(), None, None)?;
                            }
                        }
                        Err(e) => {
                            #[cfg(feature = "repl")]
                            eprintln!("{}", format!("Error: {e}").bright_red());
                            #[cfg(not(feature = "repl"))]
                            eprintln!("Error: {e}");
                            self.session.add_command(line.to_string(), None, Some(e.to_string()))?;
                        }
                    }

                    self.increment_line_number();
                }
                Ok(None) => {
                    // EOF (Ctrl-D)
                    println!("^D");
                    break;
                }
                Err(e) => {
                    eprintln!("Error: {e:?}");
                    break;
                }
            }
        }

        // Save session if it has unsaved changes
        if self.session.has_unsaved_changes() {
            self.session.save_current_session()?;
        }

        println!("Goodbye!");
        Ok(())
    }

    /// Handles REPL-specific commands.
    fn handle_repl_command(&mut self, line: &str) -> Result<Option<bool>> {
        match line {
            "(exit)" | "(quit)" | ":quit" | ":q" => Ok(Some(false)),
            ":help" | ":h" => {
                self.print_help();
                Ok(Some(true))
            }
            ":version" | ":v" => {
                println!("Lambdust version: {}", crate::VERSION);
                println!("Language version: {}", crate::LANGUAGE_VERSION);
                Ok(Some(true))
            }
            ":session" => {
                self.session.show_current_session()?;
                Ok(Some(true))
            }
            ":sessions" => {
                self.session.list_sessions()?;
                Ok(Some(true))
            }
            line if line.starts_with(":load ") => {
                let session_id = line.strip_prefix(":load ").unwrap().trim();
                self.session.load_session(session_id)?;
                println!("Loaded session: {session_id}");
                Ok(Some(true))
            }
            line if line.starts_with(":save ") => {
                let session_name = line.strip_prefix(":save ").unwrap().trim();
                self.session.save_session(session_name)?;
                Ok(Some(true))
            }
            _ => Ok(None)
        }
    }

    /// Evaluates a Scheme expression using the Lambdust instance.
    fn evaluate_expression(&mut self, source: &str) -> Result<crate::eval::Value> {
        self.lambdust.eval(source, Some("<repl>"))
    }

    /// Prints REPL help information.
    fn print_help(&self) {
        #[cfg(feature = "repl")]
        {
            println!("{}", "Lambdust Enhanced REPL Commands:".bright_blue().bold());
            println!("  {}  - Show this help", ":help, :h".bright_yellow());
            println!("  {}  - Show version information", ":version, :v".bright_yellow());
            println!("  {}  - Show current session info", ":session".bright_yellow());
            println!("  {}  - List all sessions", ":sessions".bright_yellow());
            println!("  {}  - Load a session by ID", ":load <session-id>".bright_yellow());
            println!("  {}  - Save current session with name", ":save <name>".bright_yellow());
            println!("  {}  - Exit the REPL", "(exit), (quit), :quit, :q".bright_yellow());
            println!();
            println!("{}", "Example expressions:".bright_blue().bold());
            println!("  {}  - Basic arithmetic", "(+ 1 2 3)".bright_cyan());
            println!("  {}  - Function definition", "(define (square x) (* x x))".bright_cyan());
            println!("  {}  - Type annotation", "(:: (+ 1 2) Number)".bright_cyan());
            println!("  {}  - Import a library", "(import (scheme base))".bright_cyan());
        }
        #[cfg(not(feature = "repl"))]
        {
            println!("Lambdust Enhanced REPL Commands:");
            println!("  :help, :h  - Show this help");
            println!("  :version, :v  - Show version information");
            println!("  :session  - Show current session info");
            println!("  :sessions  - List all sessions");
            println!("  :load <session-id>  - Load a session by ID");
            println!("  :save <name>  - Save current session with name");
            println!("  (exit), (quit), :quit, :q  - Exit the REPL");
            println!();
            println!("Example expressions:");
            println!("  (+ 1 2 3)  - Basic arithmetic");
            println!("  (define (square x) (* x x))  - Function definition");
            println!("  (:: (+ 1 2) Number)  - Type annotation");
            println!("  (import (scheme base))  - Import a library");
        }
        println!();
    }
}
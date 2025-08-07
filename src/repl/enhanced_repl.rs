use super::{
    ReplConfig, EnhancedEditor, HistoryManager, Debugger, 
    CompletionProvider, SyntaxHighlighter, CodeInspector, SessionManager
};
use crate::{Lambdust, Result};

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
}
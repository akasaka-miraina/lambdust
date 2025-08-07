//! Enhanced line editor for the REPL with advanced features.

#![allow(dead_code, missing_docs)]

use crate::{Result, Error};
use crate::repl::{ReplConfig, CompletionProvider, SyntaxHighlighter};

#[cfg(feature = "enhanced-repl")]
use {
    reedline::{Reedline, Signal, ReedlineEvent, ReedlineMenu, DefaultPrompt, 
               Prompt, PromptHistorySearch, PromptViMode, PromptEditMode,
               MenuBuilder},
    crossterm::event::{KeyCode, KeyModifiers},
    nu_ansi_term::{Color, Style},
};

#[cfg(not(feature = "enhanced-repl"))]
use {
    rustyline::{DefaultEditor, error::ReadlineError},
};

/// Enhanced editor that provides advanced line editing capabilities
pub struct EnhancedEditor {
    #[cfg(feature = "enhanced-repl")]
    editor: Reedline,
    #[cfg(not(feature = "enhanced-repl"))]
    editor: DefaultEditor,
    config: ReplConfig,
    multiline_buffer: String,
    in_multiline: bool,
    paren_depth: isize,
}

impl EnhancedEditor {
    pub fn new(config: ReplConfig) -> Result<Self> {
        #[cfg(feature = "enhanced-repl")]
        {
            let editor = Reedline::create();

            // Configure editor based on settings
            if config.syntax_highlighting {
                // Syntax highlighting will be handled separately
            }

            Ok(Self {
                editor,
                config,
                multiline_buffer: String::new(),
                in_multiline: false,
                paren_depth: 0,
            })
        }
        
        #[cfg(not(feature = "enhanced-repl"))]
        {
            let editor = DefaultEditor::new()
                .map_err(|e| Error::io_error(format!("Failed to create editor: {}", e)))?;
            
            Ok(Self {
                editor,
                config,
                multiline_buffer: String::new(),
                in_multiline: false,
                paren_depth: 0,
            })
        }
    }

    pub fn read_line(
        &mut self, 
        prompt: &str, 
        _completion_provider: &mut CompletionProvider, 
        _highlighter: &SyntaxHighlighter
    ) -> Result<Option<String>> {
        
        #[cfg(feature = "enhanced-repl")]
        {
            self.read_line_enhanced(prompt, _completion_provider, _highlighter)
        }
        
        #[cfg(not(feature = "enhanced-repl"))]
        {
            self.read_line_basic(prompt)
        }
    }

    #[cfg(feature = "enhanced-repl")]
    fn read_line_enhanced(
        &mut self, 
        prompt: &str, 
        _completion_provider: &mut CompletionProvider, 
        _highlighter: &SyntaxHighlighter
    ) -> Result<Option<String>> {
        let prompt = LambdustPrompt::new(prompt.to_string());
        
        loop {
            let sig = self.editor.read_line(&prompt)
                .map_err(|e| Error::io_error(format!("Failed to read line: {}", e)))?;

            match sig {
                Signal::Success(buffer) => {
                    let line = buffer.trim();
                    
                    if line.is_empty() {
                        continue;
                    }

                    // Handle multiline input
                    if self.needs_more_input(line) {
                        self.multiline_buffer.push_str(line);
                        self.multiline_buffer.push('\n');
                        self.in_multiline = true;
                        continue;
                    } else if self.in_multiline {
                        self.multiline_buffer.push_str(line);
                        let complete_input = self.multiline_buffer.clone());
                        self.multiline_buffer.clear();
                        self.in_multiline = false;
                        self.paren_depth = 0;
                        return Ok(Some(complete_input));
                    } else {
                        return Ok(Some(line.to_string()));
                    }
                }
                Signal::CtrlD => {
                    if self.in_multiline {
                        // Cancel multiline input
                        self.multiline_buffer.clear();
                        self.in_multiline = false;
                        self.paren_depth = 0;
                        println!("^D (multiline cancelled)");
                        continue;
                    } else {
                        return Ok(None);
                    }
                }
                Signal::CtrlC => {
                    if self.in_multiline {
                        // Cancel multiline input
                        self.multiline_buffer.clear();
                        self.in_multiline = false;
                        self.paren_depth = 0;
                        println!("^C (multiline cancelled)");
                        continue;
                    } else {
                        println!("^C");
                        continue;
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "enhanced-repl"))]
    fn read_line_basic(&mut self, prompt: &str) -> Result<Option<String>> {
        loop {
            let effective_prompt = if self.in_multiline {
                format!("{}...", prompt)
            } else {
                prompt.to_string()
            };

            match self.editor.readline(&effective_prompt) {
                Ok(line) => {
                    let line = line.trim();
                    
                    if line.is_empty() {
                        continue;
                    }

                    // Add to history
                    let _ = self.editor.add_history_entry(line);

                    // Handle multiline input
                    if self.needs_more_input(line) {
                        self.multiline_buffer.push_str(line);
                        self.multiline_buffer.push('\n');
                        self.in_multiline = true;
                        continue;
                    } else if self.in_multiline {
                        self.multiline_buffer.push_str(line);
                        let complete_input = self.multiline_buffer.clone());
                        self.multiline_buffer.clear();
                        self.in_multiline = false;
                        self.paren_depth = 0;
                        return Ok(Some(complete_input));
                    } else {
                        return Ok(Some(line.to_string()));
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    if self.in_multiline {
                        // Cancel multiline input
                        self.multiline_buffer.clear();
                        self.in_multiline = false;
                        self.paren_depth = 0;
                        println!("^C (multiline cancelled)");
                        continue;
                    } else {
                        println!("^C");
                        continue;
                    }
                }
                Err(ReadlineError::Eof) => {
                    if self.in_multiline {
                        // Cancel multiline input
                        self.multiline_buffer.clear();
                        self.in_multiline = false;
                        self.paren_depth = 0;
                        println!("^D (multiline cancelled)");
                        continue;
                    } else {
                        return Ok(None);
                    }
                }
                Err(err) => {
                    return Err(Box::new(Error::io_error(format!("Readline error: {}", err).boxed()));
                }
            }
        }
    }

    fn needs_more_input(&mut self, line: &str) -> bool {
        // Count parentheses to determine if we need more input
        let mut in_string = false;
        let mut in_comment = false;
        let mut escape_next = false;

        for ch in line.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' if !in_comment => in_string = !in_string,
                ';' if !in_string => in_comment = true,
                '\n' => in_comment = false,
                '(' if !in_string && !in_comment => self.paren_depth += 1,
                ')' if !in_string && !in_comment => self.paren_depth -= 1,
                _ => {}
            }
        }

        // We need more input if:
        // 1. We have unmatched parentheses
        // 2. We're in a string that wasn't closed
        // 3. The line ends with a backslash (line continuation)
        self.paren_depth > 0 || in_string || line.trim_end().ends_with('\\')
    }

    pub fn add_to_history(&mut self, line: &str) -> Result<()> {
        #[cfg(feature = "enhanced-repl")]
        {
            // History is handled automatically by reedline
            Ok(())
        }
        
        #[cfg(not(feature = "enhanced-repl"))]
        {
            let _ = self.editor.add_history_entry(line);
            Ok(())
        }
    }

    pub fn save_history<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        #[cfg(feature = "enhanced-repl")]
        {
            // TODO: Implement history saving for reedline
            Ok(())
        }
        
        #[cfg(not(feature = "enhanced-repl"))]
        {
            self.editor.save_history(path.as_ref())
                .map_err(|e| Error::io_error(format!("Failed to save history: {}", e)))
        }
    }

    pub fn load_history<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        #[cfg(feature = "enhanced-repl")]
        {
            // TODO: Implement history loading for reedline
            Ok(())
        }
        
        #[cfg(not(feature = "enhanced-repl"))]
        {
            let _ = self.editor.load_history(path.as_ref());
            Ok(())
        }
    }

    pub fn is_in_multiline(&self) -> bool {
        self.in_multiline
    }

    pub fn cancel_multiline(&mut self) {
        self.multiline_buffer.clear();
        self.in_multiline = false;
        self.paren_depth = 0;
    }

    pub fn get_multiline_buffer(&self) -> &str {
        &self.multiline_buffer
    }
}

/// Custom prompt implementation for the enhanced REPL
#[cfg(feature = "enhanced-repl")]
pub struct LambdustPrompt {
    prompt_text: String,
}

#[cfg(feature = "enhanced-repl")]
impl LambdustPrompt {
    pub fn new(prompt_text: String) -> Self {
        Self { prompt_text }
    }
}

#[cfg(feature = "enhanced-repl")]
impl Prompt for LambdustPrompt {
    fn render_prompt_left(&self) -> std::borrow::Cow<str> {
        self.prompt_text.as_str().into())
    }

    fn render_prompt_right(&self) -> std::borrow::Cow<str> {
        "".into())
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> std::borrow::Cow<str> {
        " ".into())
    }

    fn render_prompt_multiline_indicator(&self) -> std::borrow::Cow<str> {
        "... ".into())
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        "(search) ".into())
    }
}

/// Bracket matching helper
pub struct BracketMatcher {
    open_brackets: Vec<char>,
    close_brackets: Vec<char>,
}

impl BracketMatcher {
    pub fn new() -> Self {
        Self {
            open_brackets: vec!['(', '[', '{'],
            close_brackets: vec![')', ']', '}'],
        }
    }

    pub fn find_matching_bracket(&self, text: &str, position: usize) -> Option<usize> {
        let chars: Vec<char> = text.chars().collect();
        if position >= chars.len() {
            return None;
        }

        let current_char = chars[position];
        
        // Find if current character is a bracket
        let (open_char, close_char, direction) = if let Some(idx) = self.open_brackets.iter().position(|&c| c == current_char) {
            (self.open_brackets[idx], self.close_brackets[idx], 1isize)
        } else if let Some(idx) = self.close_brackets.iter().position(|&c| c == current_char) {
            (self.open_brackets[idx], self.close_brackets[idx], -1isize)
        } else {
            return None;
        };

        let mut depth = 0;
        let mut i = position as isize;
        let mut in_string = false;
        let mut in_comment = false;

        loop {
            let current_pos = i as usize;
            if current_pos >= chars.len() {
                break;
            }

            let ch = chars[current_pos];

            // Handle string and comment contexts
            match ch {
                '"' if !in_comment => in_string = !in_string,
                ';' if !in_string => in_comment = true,
                '\n' => in_comment = false,
                _ => {}
            }

            // Only count brackets outside of strings and comments
            if !in_string && !in_comment {
                if ch == open_char {
                    depth += direction;
                } else if ch == close_char {
                    depth -= direction;
                }

                if depth == 0 && current_pos != position {
                    return Some(current_pos);
                }
            }

            i += direction;
            if i < 0 {
                break;
            }
        }

        None
    }

    pub fn highlight_matching_brackets(&self, text: &str, cursor_position: usize) -> Vec<(usize, usize)> {
        let mut highlights = Vec::new();
        
        if let Some(matching_pos) = self.find_matching_bracket(text, cursor_position) {
            highlights.push((cursor_position, cursor_position + 1));
            highlights.push((matching_pos, matching_pos + 1));
        }

        highlights
    }
}

impl Default for BracketMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Indentation helper for multi-line input
pub struct IndentationHelper {
    indent_size: usize,
}

impl IndentationHelper {
    pub fn new(indent_size: usize) -> Self {
        Self { indent_size }
    }

    pub fn calculate_indentation(&self, text: &str) -> usize {
        let mut indent_level: usize = 0;
        let mut in_string = false;
        let mut in_comment = false;

        for ch in text.chars() {
            match ch {
                '"' if !in_comment => in_string = !in_string,
                ';' if !in_string => in_comment = true,
                '\n' => in_comment = false,
                '(' if !in_string && !in_comment => indent_level += 1,
                ')' if !in_string && !in_comment => indent_level = indent_level.saturating_sub(1),
                _ => {}
            }
        }

        indent_level * self.indent_size
    }

    pub fn get_indent_string(&self, level: usize) -> String {
        " ".repeat(level)
    }
}

impl Default for IndentationHelper {
    fn default() -> Self {
        Self::new(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bracket_matching() {
        let matcher = BracketMatcher::new();
        
        // Test simple parentheses
        let text = "(+ 1 2)";
        assert_eq!(matcher.find_matching_bracket(text, 0), Some(6));
        assert_eq!(matcher.find_matching_bracket(text, 6), Some(0));
        
        // Test nested parentheses
        let text = "(+ 1 (- 3 2))";
        assert_eq!(matcher.find_matching_bracket(text, 0), Some(12));
        assert_eq!(matcher.find_matching_bracket(text, 5), Some(11));
        
        // Test no matching bracket
        let text = "(+ 1 2";
        assert_eq!(matcher.find_matching_bracket(text, 0), None);
    }

    #[test]
    fn test_indentation_calculation() {
        let helper = IndentationHelper::new(2);
        
        // Simple expression - no extra indentation needed
        assert_eq!(helper.calculate_indentation("(+ 1 2)"), 0);
        
        // One open paren - should indent
        assert_eq!(helper.calculate_indentation("(+"), 2);
        
        // Nested expression
        assert_eq!(helper.calculate_indentation("(let ((x"), 4);
    }

    #[test]
    fn test_multiline_detection() {
        let config = ReplConfig::default();
        let mut editor = EnhancedEditor::new(config).unwrap();
        
        // Complete expression - no more input needed
        assert!(!editor.needs_more_input("(+ 1 2)"));
        
        // Incomplete expression - more input needed
        editor.paren_depth = 0; // Reset
        assert!(editor.needs_more_input("(+ 1"));
        
        // String continuation
        editor.paren_depth = 0; // Reset
        assert!(editor.needs_more_input("\"hello"));
    }
}
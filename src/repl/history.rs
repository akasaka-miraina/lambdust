//! History management system for the enhanced REPL.

#![allow(dead_code, missing_docs)]

use crate::{Result, Error};
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a single history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
}

impl HistoryEntry {
    pub fn new(command: String) -> Self {
        Self {
            command,
            timestamp: Utc::now(),
            session_id: None,
            result: None,
            error: None,
        }
    }

    pub fn with_result(mut self, result: String) -> Self {
        self.result = Some(result);
        self
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }

    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }
}

/// Search functionality for command history
pub struct HistorySearch {
    query: String,
    case_sensitive: bool,
    regex_mode: bool,
}

impl HistorySearch {
    pub fn new(query: String) -> Self {
        Self {
            query,
            case_sensitive: false,
            regex_mode: false,
        }
    }

    pub fn case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }

    pub fn regex_mode(mut self) -> Self {
        self.regex_mode = true;
        self
    }

    pub fn matches(&self, entry: &HistoryEntry) -> bool {
        let command = if self.case_sensitive {
            entry.command.as_str()
        } else {
            &entry.command.to_lowercase()
        };

        let query = if self.case_sensitive {
            self.query.as_str()
        } else {
            &self.query.to_lowercase()
        };

        if self.regex_mode {
            // TODO: Implement regex matching
            command.contains(query)
        } else {
            command.contains(query)
        }
    }
}

/// Manages command history with persistence and search capabilities
pub struct HistoryManager {
    entries: VecDeque<HistoryEntry>,
    max_entries: usize,
    history_file: PathBuf,
    current_session_id: String,
    auto_save: bool,
}

impl HistoryManager {
    pub fn new(max_entries: usize) -> Result<Self> {
        let history_file = Self::default_history_file()?;
        let current_session_id = Self::generate_session_id();
        
        let mut manager = Self {
            entries: VecDeque::new(),
            max_entries,
            history_file,
            current_session_id,
            auto_save: true,
        };

        // Load existing history
        if manager.history_file.exists() {
            if let Err(e) = manager.load_from_file() {
                eprintln!("Warning: Failed to load history: {}", e);
            }
        }

        Ok(manager)
    }

    pub fn with_file<P: AsRef<Path>>(max_entries: usize, history_file: P) -> Result<Self> {
        let history_file = history_file.as_ref().to_path_buf();
        let current_session_id = Self::generate_session_id();
        
        let mut manager = Self {
            entries: VecDeque::new(),
            max_entries,
            history_file,
            current_session_id,
            auto_save: true,
        };

        // Load existing history
        if manager.history_file.exists() {
            manager.load_from_file()?;
        }

        Ok(manager)
    }

    fn default_history_file() -> Result<PathBuf> {
        let mut path = dirs::home_dir()
            .ok_or_else(|| Error::io_error("Could not determine home directory"))?;
        path.push(".lambdust_history");
        Ok(path)
    }

    fn generate_session_id() -> String {
        format!("session_{}", Utc::now().timestamp())
    }

    pub fn add_entry(&mut self, command: String) -> Result<()> {
        if command.trim().is_empty() {
            return Ok(());
        }

        let entry = HistoryEntry::new(command)
            .with_session(self.current_session_id.clone());

        self.entries.push_back(entry);

        // Maintain max entries limit
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }

        if self.auto_save {
            self.save_to_file()?;
        }

        Ok(())
    }

    pub fn add_entry_with_result(&mut self, command: String, result: String) -> Result<()> {
        if command.trim().is_empty() {
            return Ok(());
        }

        let entry = HistoryEntry::new(command)
            .with_session(self.current_session_id.clone())
            .with_result(result);

        self.entries.push_back(entry);

        // Maintain max entries limit
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }

        if self.auto_save {
            self.save_to_file()?;
        }

        Ok(())
    }

    pub fn add_entry_with_error(&mut self, command: String, error: String) -> Result<()> {
        if command.trim().is_empty() {
            return Ok(());
        }

        let entry = HistoryEntry::new(command)
            .with_session(self.current_session_id.clone())
            .with_error(error);

        self.entries.push_back(entry);

        // Maintain max entries limit
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }

        if self.auto_save {
            self.save_to_file()?;
        }

        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<Vec<&HistoryEntry>> {
        let search = HistorySearch::new(query.to_string());
        let matches: Vec<&HistoryEntry> = self.entries
            .iter()
            .filter(|entry| search.matches(entry))
            .collect();

        if matches.is_empty() {
            println!("No matches found for: {}", query);
        } else {
            println!("Found {} matches for: {}", matches.len(), query);
            for (i, entry) in matches.iter().enumerate() {
                println!("  {}: {} ({})", 
                    i + 1, 
                    entry.command, 
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S")
                );
            }
        }

        Ok(matches)
    }

    pub fn reverse_search(&self, query: &str) -> Result<Option<&HistoryEntry>> {
        let search = HistorySearch::new(query.to_string());
        
        // Search from most recent to oldest
        for entry in self.entries.iter().rev() {
            if search.matches(entry) {
                return Ok(Some(entry));
            }
        }

        Ok(None)
    }

    pub fn show_recent(&self, count: usize) -> Result<()> {
        let recent_count = count.min(self.entries.len());
        
        if recent_count == 0 {
            println!("No history entries");
            return Ok(());
        }

        println!("Recent history ({} entries):", recent_count);
        
        let start_index = self.entries.len().saturating_sub(recent_count);
        for (i, entry) in self.entries.iter().skip(start_index).enumerate() {
            let display_index = start_index + i + 1;
            println!("  {}: {} ({})", 
                display_index,
                entry.command,
                entry.timestamp.format("%H:%M:%S")
            );
        }

        Ok(())
    }

    pub fn get_entry(&self, index: usize) -> Option<&HistoryEntry> {
        if index == 0 || index > self.entries.len() {
            None
        } else {
            self.entries.get(index - 1)
        }
    }

    pub fn replay_session(&self, session_id: &str) -> Result<Vec<String>> {
        let commands: Vec<String> = self.entries
            .iter()
            .filter(|entry| entry.session_id.as_deref() == Some(session_id))
            .map(|entry| entry.command.clone())
            .collect();

        if commands.is_empty() {
            println!("No commands found for session: {}", session_id);
        } else {
            println!("Found {} commands for session: {}", commands.len(), session_id);
            for (i, command) in commands.iter().enumerate() {
                println!("  {}: {}", i + 1, command);
            }
        }

        Ok(commands)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        if self.auto_save {
            self.save_to_file()?;
        }
        Ok(())
    }

    pub fn save_to_file(&self) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = self.history_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| Error::io_error(format!("Failed to create history directory: {}", e)))?;
        }

        let mut file = File::create(&self.history_file)
            .map_err(|e| Error::io_error(format!("Failed to create history file: {}", e)))?;

        for entry in &self.entries {
            let json = serde_json::to_string(entry)
                .map_err(|e| Error::io_error(format!("Failed to serialize history entry: {}", e)))?;
            writeln!(file, "{}", json)
                .map_err(|e| Error::io_error(format!("Failed to write history entry: {}", e)))?;
        }

        Ok(())
    }

    pub fn load_from_file(&mut self) -> Result<()> {
        let file = File::open(&self.history_file)
            .map_err(|e| Error::io_error(format!("Failed to open history file: {}", e)))?;

        let reader = BufReader::new(file);
        self.entries.clear();

        for line in reader.lines() {
            let line = line
                .map_err(|e| Error::io_error(format!("Failed to read history line: {}", e)))?;
            
            if line.trim().is_empty() {
                continue;
            }

            let entry: HistoryEntry = serde_json::from_str(&line)
                .map_err(|e| Error::io_error(format!("Failed to parse history entry: {}", e)))?;
            
            self.entries.push_back(entry);
        }

        // Maintain max entries limit
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }

        Ok(())
    }

    pub fn export_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = File::create(path.as_ref())
            .map_err(|e| Error::io_error(format!("Failed to create export file: {}", e)))?;

        writeln!(file, "# Lambdust REPL History Export")?;
        writeln!(file, "# Generated at: {}", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(file, "# Total entries: {}", self.entries.len())?;
        writeln!(file)?;

        for (i, entry) in self.entries.iter().enumerate() {
            writeln!(file, "# Entry {}: {}", i + 1, entry.timestamp.format("%Y-%m-%d %H:%M:%S"))?;
            writeln!(file, "{}", entry.command)?;
            if let Some(ref result) = entry.result {
                writeln!(file, "# Result: {}", result)?;
            }
            if let Some(ref error) = entry.error {
                writeln!(file, "# Error: {}", error)?;
            }
            writeln!(file)?;
        }

        Ok(())
    }

    pub fn get_stats(&self) -> HistoryStats {
        let mut command_counts = std::collections::HashMap::new();
        let mut session_counts = std::collections::HashMap::new();
        let mut error_count = 0;

        for entry in &self.entries {
            // Count command prefixes (first word)
            if let Some(first_word) = entry.command.split_whitespace().next() {
                *command_counts.entry(first_word.to_string()).or_insert(0) += 1;
            }

            // Count sessions
            if let Some(ref session) = entry.session_id {
                *session_counts.entry(session.clone()).or_insert(0) += 1;
            }

            // Count errors
            if entry.error.is_some() {
                error_count += 1;
            }
        }

        HistoryStats {
            total_entries: self.entries.len(),
            error_count,
            session_count: session_counts.len(),
            most_used_commands: {
                let mut commands: Vec<_> = command_counts.into_iter().collect();
                commands.sort_by(|a, b| b.1.cmp(&a.1));
                commands.into_iter().take(10).collect()
            },
        }
    }

    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }

    pub fn current_session_id(&self) -> &str {
        &self.current_session_id
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Statistics about the command history
#[derive(Debug)]
pub struct HistoryStats {
    pub total_entries: usize,
    pub error_count: usize,
    pub session_count: usize,
    pub most_used_commands: Vec<(String, usize)>,
}

impl HistoryStats {
    pub fn print(&self) {
        println!("History Statistics:");
        println!("  Total entries: {}", self.total_entries);
        println!("  Errors: {}", self.error_count);
        println!("  Sessions: {}", self.session_count);
        
        if !self.most_used_commands.is_empty() {
            println!("  Most used commands:");
            for (command, count) in &self.most_used_commands {
                println!("    {}: {} times", command, count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_history_manager_basic() -> Result<()> {
        let dir = tempdir().unwrap();
        let history_file = dir.path().join("test_history");
        
        let mut manager = HistoryManager::with_file(10, &history_file)?;
        
        // Test adding entries
        manager.add_entry("(+ 1 2)".to_string())?;
        manager.add_entry("(define x 42)".to_string())?;
        
        assert_eq!(manager.len(), 2);
        
        // Test search
        let matches = manager.search("+")?;
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].command, "(+ 1 2)");
        
        // Test recent history
        manager.show_recent(5)?;
        
        Ok(())
    }

    #[test]
    fn test_history_entry() {
        let entry = HistoryEntry::new("test command".to_string())
            .with_result("42".to_string())
            .with_session("test_session".to_string());
        
        assert_eq!(entry.command, "test command");
        assert_eq!(entry.result, Some("42".to_string()));
        assert_eq!(entry.session_id, Some("test_session".to_string()));
        assert!(entry.error.is_none());
    }

    #[test]
    fn test_history_search() {
        let entry1 = HistoryEntry::new("(+ 1 2)".to_string());
        let entry2 = HistoryEntry::new("(define x 42)".to_string());
        
        let search = HistorySearch::new("+".to_string());
        assert!(search.matches(&entry1));
        assert!(!search.matches(&entry2));
        
        let search = HistorySearch::new("define".to_string());
        assert!(!search.matches(&entry1));
        assert!(search.matches(&entry2));
    }
}
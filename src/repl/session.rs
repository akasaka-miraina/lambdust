//! Session management for the enhanced REPL.

#![allow(dead_code, missing_docs)]

use crate::{Result, Error};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a saved REPL session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub commands: Vec<SessionCommand>,
    pub metadata: SessionMetadata,
}

/// A command executed in a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCommand {
    pub input: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub execution_time: Option<u64>, // in milliseconds
}

/// Metadata about a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub variables: HashMap<String, String>, // Simplified variable storage
    pub imports: Vec<String>,
    pub working_directory: Option<String>,
    pub lambdust_version: String,
}

/// Current state of the REPL session
#[derive(Debug, Clone)]
pub struct SessionState {
    pub current_session: Session,
    pub unsaved_changes: bool,
    pub auto_save_enabled: bool,
    pub auto_save_interval: std::time::Duration,
    pub last_save_time: Option<std::time::Instant>,
}

/// Manages REPL sessions with save/load/replay functionality
pub struct SessionManager {
    sessions_dir: PathBuf,
    current_state: Option<SessionState>,
    available_sessions: HashMap<String, SessionInfo>,
}

/// Information about an available session
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub file_path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub command_count: usize,
    pub size_bytes: u64,
}

impl SessionManager {
    pub fn new() -> Result<Self> {
        let sessions_dir = Self::default_sessions_dir()?;
        fs::create_dir_all(&sessions_dir)
            .map_err(|e| Error::io_error(format!("Failed to create sessions directory: {e}")))?;

        let mut manager = Self {
            sessions_dir,
            current_state: None,
            available_sessions: HashMap::new(),
        };

        manager.scan_available_sessions()?;
        manager.start_new_session()?;

        Ok(manager)
    }

    pub fn with_sessions_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let sessions_dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&sessions_dir)
            .map_err(|e| Error::io_error(format!("Failed to create sessions directory: {e}")))?;

        let mut manager = Self {
            sessions_dir,
            current_state: None,
            available_sessions: HashMap::new(),
        };

        manager.scan_available_sessions()?;
        manager.start_new_session()?;

        Ok(manager)
    }

    fn default_sessions_dir() -> Result<PathBuf> {
        let mut path = dirs::home_dir()
            .ok_or_else(|| Error::io_error("Could not determine home directory"))?;
        path.push(".lambdust");
        path.push("sessions");
        Ok(path)
    }

    fn generate_session_id() -> String {
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        format!("session_{timestamp}")
    }

    pub fn start_new_session(&mut self) -> Result<()> {
        let session_id = Self::generate_session_id();
        let session = Session {
            id: session_id.clone(),
            name: {
                let formatted_time = Utc::now().format("%Y-%m-%d %H:%M:%S");
                format!("Session {formatted_time}")
            },
            created_at: Utc::now(),
            modified_at: Utc::now(),
            commands: Vec::new(),
            metadata: SessionMetadata {
                description: None,
                tags: Vec::new(),
                variables: HashMap::new(),
                imports: Vec::new(),
                working_directory: std::env::current_dir()
                    .ok()
                    .and_then(|p| p.to_str().map(|s| s.to_string())),
                lambdust_version: crate::VERSION.to_string(),
            },
        };

        let state = SessionState {
            current_session: session,
            unsaved_changes: false,
            auto_save_enabled: true,
            auto_save_interval: std::time::Duration::from_secs(300), // 5 minutes
            last_save_time: None,
        };

        self.current_state = Some(state);
        Ok(())
    }

    pub fn add_command(&mut self, input: String, output: Option<String>, error: Option<String>) -> Result<()> {
        if let Some(ref mut state) = self.current_state {
            let command = SessionCommand {
                input,
                output,
                error,
                timestamp: Utc::now(),
                execution_time: None, // TODO: Add timing information
            };

            state.current_session.commands.push(command);
            state.current_session.modified_at = Utc::now();
            state.unsaved_changes = true;

            // Auto-save if enabled and enough time has passed
            if state.auto_save_enabled {
                let should_auto_save = state.last_save_time
                    .map(|last| last.elapsed() >= state.auto_save_interval)
                    .unwrap_or(true);

                if should_auto_save {
                    self.save_current_session()?;
                }
            }
        }

        Ok(())
    }

    pub fn save_current_session(&mut self) -> Result<()> {
        if let Some(ref mut state) = self.current_state {
            let session_file = self.sessions_dir.join({
                let session_id = &state.current_session.id;
                format!("{session_id}.json")
            });
            
            let json = serde_json::to_string_pretty(&state.current_session)
                .map_err(|e| Error::io_error(format!("Failed to serialize session: {e}")))?;

            fs::write(&session_file, json)
                .map_err(|e| Error::io_error(format!("Failed to write session file: {e}")))?;

            state.unsaved_changes = false;
            state.last_save_time = Some(std::time::Instant::now());

            // Get session info before mutable borrow ends
            let _session_id = state.current_session.id.clone();
            let session_name = state.current_session.name.clone();
            let _session_created = state.current_session.created_at;
            let _session_modified = state.current_session.modified_at;
            let _command_count = state.current_session.commands.len();

            println!("Session saved: {session_name}");
        }

        // Update available sessions info after borrow ends
        let session_file = {
            if let Some(ref state) = self.current_state {
                Some((state.current_session.clone(), self.sessions_dir.join({
                let session_id = &state.current_session.id;
                format!("{session_id}.json")
            })))
            } else {
                None
            }
        };
        
        if let Some((session, file_path)) = session_file {
            self.update_session_info(&session, &file_path)?;
        }

        Ok(())
    }

    pub fn save_session(&mut self, name: &str) -> Result<()> {
        if let Some(ref mut state) = self.current_state {
            state.current_session.name = name.to_string();
            state.current_session.modified_at = Utc::now();
            self.save_current_session()
        } else {
            Err(Box::new(Error::runtime_error("No active session to save", None)))
        }
    }

    pub fn load_session(&mut self, session_id: &str) -> Result<()> {
        // Save current session if it has unsaved changes
        if let Some(ref state) = self.current_state {
            if state.unsaved_changes {
                println!("Saving current session before loading...");
                self.save_current_session()?;
            }
        }

        let session_file = self.sessions_dir.join(format!("{session_id}.json"));
        if !session_file.exists() {
            return Err(Box::new(Error::io_error(format!("Session file not found: {session_id}"))));
        }

        let content = fs::read_to_string(&session_file)
            .map_err(|e| Error::io_error(format!("Failed to read session file: {e}")))?;

        let session: Session = serde_json::from_str(&content)
            .map_err(|e| Error::io_error(format!("Failed to parse session file: {e}")))?;

        let state = SessionState {
            current_session: session,
            unsaved_changes: false,
            auto_save_enabled: true,
            auto_save_interval: std::time::Duration::from_secs(300),
            last_save_time: Some(std::time::Instant::now()),
        };

        self.current_state = Some(state);
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<()> {
        if self.available_sessions.is_empty() {
            println!("No saved sessions found");
            return Ok(());
        }

        println!("Available sessions:");
        println!("{:<20} {:<30} {:<20} {:<10} {:<10}", 
                 "ID", "Name", "Modified", "Commands", "Size");
        println!("{}", "-".repeat(90));

        let mut sessions: Vec<_> = self.available_sessions.values().collect();
        sessions.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

        for session_info in sessions {
            let size_str = if session_info.size_bytes < 1024 {
                let size = session_info.size_bytes;
                format!("{size}B")
            } else if session_info.size_bytes < 1024 * 1024 {
                let size_kb = session_info.size_bytes / 1024;
                format!("{size_kb}KB")
            } else {
                let size_mb = session_info.size_bytes / (1024 * 1024);
                format!("{size_mb}MB")
            };

            println!("{:<20} {:<30} {:<20} {:<10} {:<10}",
                     &session_info.id[..20.min(session_info.id.len())],
                     &session_info.name[..30.min(session_info.name.len())],
                     session_info.modified_at.format("%Y-%m-%d %H:%M:%S"),
                     session_info.command_count,
                     size_str);
        }

        if let Some(ref state) = self.current_state {
            println!("\nCurrent session: {} ({})", 
                     state.current_session.name,
                     if state.unsaved_changes { "unsaved changes" } else { "saved" });
        }

        Ok(())
    }

    pub fn show_current_session(&self) -> Result<()> {
        if let Some(ref state) = self.current_state {
            let session = &state.current_session;
            let session_name = &session.name;
            println!("📁 Current Session: {session_name}");
            let session_id = &session.id;
            println!("   ID: {session_id}");
            let created = session.created_at.format("%Y-%m-%d %H:%M:%S UTC");
            println!("   Created: {created}");
            let modified = session.modified_at.format("%Y-%m-%d %H:%M:%S UTC");
            println!("   Modified: {modified}");
            let command_count = session.commands.len();
            println!("   Commands: {command_count}");
            let status = if state.unsaved_changes { "unsaved changes" } else { "saved" };
            println!("   Status: {status}");
            
            if let Some(ref desc) = session.metadata.description {
                println!("   Description: {desc}");
            }
            
            if !session.metadata.tags.is_empty() {
                let tags = session.metadata.tags.join(", ");
                println!("   Tags: {tags}");
            }
            
            if !session.metadata.imports.is_empty() {
                let imports = session.metadata.imports.join(", ");
                println!("   Imports: {imports}");
            }

            if !session.commands.is_empty() {
                println!("\n   Recent commands:");
                let recent_count = 5.min(session.commands.len());
                for command in session.commands.iter().rev().take(recent_count) {
                    let input = &command.input;
                    let output = command.output.as_deref().unwrap_or("<no output>");
                    println!("     {input} -> {output}");
                }
                if session.commands.len() > recent_count {
                    let more_count = session.commands.len() - recent_count;
                    println!("     ... and {more_count} more commands");
                }
            }
        } else {
            println!("No active session");
        }

        Ok(())
    }

    pub fn replay_session(&self, session_id: &str, start_from: Option<usize>) -> Result<Vec<String>> {
        let session_file = self.sessions_dir.join(format!("{session_id}.json"));
        if !session_file.exists() {
            return Err(Box::new(Error::io_error(format!("Session file not found: {session_id}"))));
        }

        let content = fs::read_to_string(&session_file)
            .map_err(|e| Error::io_error(format!("Failed to read session file: {e}")))?;

        let session: Session = serde_json::from_str(&content)
            .map_err(|e| Error::io_error(format!("Failed to parse session file: {e}")))?;

        let start_index = start_from.unwrap_or(0);
        let commands: Vec<String> = session.commands
            .iter()
            .skip(start_index)
            .map(|cmd| cmd.input.clone())
            .collect();

        if commands.is_empty() {
            let session_name = &session.name;
            println!("No commands to replay from session: {session_name}");
        } else {
            let command_count = commands.len();
            let session_name = &session.name;
            println!("Replaying {command_count} commands from session: {session_name}");
            for (i, command) in commands.iter().enumerate() {
                let command_num = start_index + i + 1;
                println!("  {command_num}: {command}");
            }
        }

        Ok(commands)
    }

    pub fn export_session(&self, session_id: &str, output_path: &Path) -> Result<()> {
        let session_file = self.sessions_dir.join(format!("{session_id}.json"));
        if !session_file.exists() {
            return Err(Box::new(Error::io_error(format!("Session file not found: {session_id}"))));
        }

        let content = fs::read_to_string(&session_file)
            .map_err(|e| Error::io_error(format!("Failed to read session file: {e}")))?;

        let session: Session = serde_json::from_str(&content)
            .map_err(|e| Error::io_error(format!("Failed to parse session file: {e}")))?;

        let mut output_file = File::create(output_path)
            .map_err(|e| Error::io_error(format!("Failed to create output file: {e}")))?;

        // Write header
        writeln!(output_file, ";; Lambdust REPL Session Export")?;
        let session_name = &session.name;
        writeln!(output_file, ";; Session: {session_name}")?;
        let session_id = &session.id;
        writeln!(output_file, ";; ID: {session_id}")?;
        let created = session.created_at.format("%Y-%m-%d %H:%M:%S UTC");
        writeln!(output_file, ";; Created: {created}")?;
        let command_count = session.commands.len();
        writeln!(output_file, ";; Commands: {command_count}")?;
        let version = &session.metadata.lambdust_version;
        writeln!(output_file, ";; Lambdust version: {version}")?;
        writeln!(output_file)?;

        // Write imports
        if !session.metadata.imports.is_empty() {
            writeln!(output_file, ";; Imports:")?;
            for import in &session.metadata.imports {
                writeln!(output_file, "(import {import})")?;
            }
            writeln!(output_file)?;
        }

        // Write commands
        for (i, command) in session.commands.iter().enumerate() {
            let command_num = i + 1;
            let timestamp = command.timestamp.format("%H:%M:%S");
            writeln!(output_file, ";; Command {command_num} - {timestamp}")?;
            let input = &command.input;
            writeln!(output_file, "{input}")?;
            
            if let Some(ref output) = command.output {
                writeln!(output_file, ";; => {output}")?;
            }
            
            if let Some(ref error) = command.error {
                writeln!(output_file, ";; Error: {error}")?;
            }
            
            writeln!(output_file)?;
        }

        let path_display = output_path.display();
        println!("Session exported to: {path_display}");
        Ok(())
    }

    pub fn delete_session(&mut self, session_id: &str) -> Result<()> {
        let session_file = self.sessions_dir.join(format!("{session_id}.json"));
        
        if !session_file.exists() {
            return Err(Box::new(Error::io_error(format!("Session file not found: {session_id}"))));
        }

        // Don't allow deleting the current session
        if let Some(ref state) = self.current_state {
            if state.current_session.id == session_id {
                return Err(Box::new(Error::runtime_error("Cannot delete the current session", None)));
            }
        }

        fs::remove_file(&session_file)
            .map_err(|e| Error::io_error(format!("Failed to delete session file: {e}")))?;

        self.available_sessions.remove(session_id);
        println!("Session deleted: {session_id}");
        Ok(())
    }

    fn scan_available_sessions(&mut self) -> Result<()> {
        self.available_sessions.clear();

        if !self.sessions_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.sessions_dir)
            .map_err(|e| Error::io_error(format!("Failed to read sessions directory: {e}")))?;

        for entry in entries {
            let entry = entry.map_err(|e| Error::io_error(format!("Failed to read directory entry: {e}")))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(session_info) = self.load_session_info(&path) {
                    self.available_sessions.insert(session_info.id.clone(), session_info);
                }
            }
        }

        Ok(())
    }

    fn load_session_info(&self, path: &Path) -> Result<SessionInfo> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::io_error(format!("Failed to read session file: {e}")))?;

        let session: Session = serde_json::from_str(&content)
            .map_err(|e| Error::io_error(format!("Failed to parse session file: {e}")))?;

        let metadata = fs::metadata(path)
            .map_err(|e| Error::io_error(format!("Failed to get file metadata: {e}")))?;

        Ok(SessionInfo {
            id: session.id,
            name: session.name,
            file_path: path.to_path_buf(),
            created_at: session.created_at,
            modified_at: session.modified_at,
            command_count: session.commands.len(),
            size_bytes: metadata.len(),
        })
    }

    fn update_session_info(&mut self, session: &Session, file_path: &Path) -> Result<()> {
        let metadata = fs::metadata(file_path)
            .map_err(|e| Error::io_error(format!("Failed to get file metadata: {e}")))?;

        let session_info = SessionInfo {
            id: session.id.clone(),
            name: session.name.clone(),
            file_path: file_path.to_path_buf(),
            created_at: session.created_at,
            modified_at: session.modified_at,
            command_count: session.commands.len(),
            size_bytes: metadata.len(),
        };

        self.available_sessions.insert(session.id.clone(), session_info);
        Ok(())
    }

    pub fn set_session_description(&mut self, description: String) -> Result<()> {
        if let Some(ref mut state) = self.current_state {
            state.current_session.metadata.description = Some(description);
            state.current_session.modified_at = Utc::now();
            state.unsaved_changes = true;
            Ok(())
        } else {
            Err(Box::new(Error::runtime_error("No active session", None)))
        }
    }

    pub fn add_session_tag(&mut self, tag: String) -> Result<()> {
        if let Some(ref mut state) = self.current_state {
            if !state.current_session.metadata.tags.contains(&tag) {
                state.current_session.metadata.tags.push(tag);
                state.current_session.modified_at = Utc::now();
                state.unsaved_changes = true;
            }
            Ok(())
        } else {
            Err(Box::new(Error::runtime_error("No active session", None)))
        }
    }

    pub fn remove_session_tag(&mut self, tag: &str) -> Result<bool> {
        if let Some(ref mut state) = self.current_state {
            if let Some(pos) = state.current_session.metadata.tags.iter().position(|t| t == tag) {
                state.current_session.metadata.tags.remove(pos);
                state.current_session.modified_at = Utc::now();
                state.unsaved_changes = true;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(Box::new(Error::runtime_error("No active session", None)))
        }
    }

    pub fn current_session_id(&self) -> Option<&str> {
        self.current_state.as_ref().map(|state| state.current_session.id.as_str())
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.current_state.as_ref().map(|state| state.unsaved_changes).unwrap_or(false)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default session manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_session_creation() {
        let dir = tempdir().unwrap();
        let mut manager = SessionManager::with_sessions_dir(dir.path()).unwrap();
        
        // Should start with a new session
        assert!(manager.current_state.is_some());
        assert!(manager.current_session_id().is_some());
    }

    #[test]
    fn test_command_addition() -> Result<()> {
        let dir = tempdir().unwrap();
        let mut manager = SessionManager::with_sessions_dir(dir.path())?;
        
        manager.add_command("(+ 1 2)".to_string(), Some("3".to_string()), None)?;
        
        if let Some(ref state) = manager.current_state {
            assert_eq!(state.current_session.commands.len(), 1);
            assert_eq!(state.current_session.commands[0].input, "(+ 1 2)");
            assert_eq!(state.current_session.commands[0].output, Some("3".to_string()));
        }
        
        Ok(())
    }

    #[test]
    fn test_session_save_load() -> Result<()> {
        let dir = tempdir().unwrap();
        let mut manager = SessionManager::with_sessions_dir(dir.path())?;
        
        manager.add_command("(define x 42)".to_string(), None, None)?;
        manager.add_command("x".to_string(), Some("42".to_string()), None)?;
        
        let session_id = manager.current_session_id().unwrap().to_string();
        manager.save_current_session()?;
        
        // Start a new session
        manager.start_new_session()?;
        assert!(manager.current_session_id() != Some(&session_id));
        
        // Load the saved session
        manager.load_session(&session_id)?;
        assert_eq!(manager.current_session_id(), Some(session_id.as_str()));
        
        if let Some(ref state) = manager.current_state {
            assert_eq!(state.current_session.commands.len(), 2);
        }
        
        Ok(())
    }
}
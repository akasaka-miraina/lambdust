//! Lightweight command-line argument parser to replace clap dependency.
//!
//! This module provides a minimal, high-performance CLI argument parsing
//! system designed specifically for Lambdust's needs, achieving significant
//! binary size reduction while maintaining essential CLI functionality.

use std::collections::HashMap;
use std::fmt;

/// Lightweight command-line argument parser.
#[derive(Debug, Clone)]
pub struct LightweightCli {
    /// Program name.
    pub name: String,
    /// Program version.
    pub version: String,
    /// Program author.
    pub author: String,
    /// Program description.
    pub about: String,
    /// Arguments definition.
    pub args: Vec<ArgDef>,
}

/// Argument definition.
#[derive(Debug, Clone)]
pub struct ArgDef {
    /// Argument name.
    pub name: String,
    /// Short flag (e.g., 'h').
    pub short: Option<char>,
    /// Long flag (e.g., "help").
    pub long: Option<String>,
    /// Help text.
    pub help: String,
    /// Value name for help display.
    pub value_name: Option<String>,
    /// Argument type.
    pub arg_type: ArgType,
    /// Whether this is a positional argument.
    pub positional: bool,
    /// Position index for positional args.
    pub index: Option<usize>,
    /// Allowed values (for validation).
    pub allowed_values: Option<Vec<String>>,
}

/// Argument type specification.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgType {
    /// Boolean flag (present/absent).
    Flag,
    /// Single value.
    Value,
    /// Multiple values.
    MultiValue,
}

/// Parsed command-line arguments.
#[derive(Debug, Clone)]
pub struct ParsedArgs {
    /// Flag arguments (name -> present).
    pub flags: HashMap<String, bool>,
    /// Value arguments (name -> value).  
    pub values: HashMap<String, String>,
    /// Multi-value arguments (name -> values).
    pub multi_values: HashMap<String, Vec<String>>,
    /// Positional arguments.
    pub positional: Vec<String>,
}

/// CLI parsing error.
#[derive(Debug)]
pub enum CliError {
    /// Unknown argument.
    UnknownArg(String),
    /// Missing required value.
    MissingValue(String),
    /// Invalid value for argument.
    InvalidValue { 
        /// Argument name
        arg: String, 
        /// Provided value
        value: String, 
        /// Allowed values
        allowed: Vec<String> 
    },
    /// Help requested.
    HelpRequested,
    /// Version requested.
    VersionRequested,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownArg(arg) => write!(f, "Unknown argument: {arg}"),
            Self::MissingValue(arg) => write!(f, "Missing value for argument: {arg}"),
            Self::InvalidValue { arg, value, allowed } => {
                write!(f, "Invalid value '{}' for argument '{}'. Allowed values: {}", 
                    value, arg, allowed.join(", "))
            }
            Self::HelpRequested => write!(f, "Help requested"),
            Self::VersionRequested => write!(f, "Version requested"),
        }
    }
}

impl std::error::Error for CliError {}

impl LightweightCli {
    /// Creates a new CLI parser.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: String::new(),
            author: String::new(),
            about: String::new(),
            args: Vec::new(),
        }
    }
    
    /// Sets the version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
    
    /// Sets the author.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }
    
    /// Sets the about text.
    pub fn about(mut self, about: impl Into<String>) -> Self {
        self.about = about.into();
        self
    }
    
    /// Adds an argument.
    pub fn arg(mut self, arg: ArgDef) -> Self {
        self.args.push(arg);
        self
    }
    
    /// Parses command-line arguments.
    pub fn parse<I, T>(&self, args: I) -> Result<ParsedArgs, CliError>
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let args: Vec<String> = args.into_iter().map(|s| s.as_ref().to_string()).collect();
        self.parse_from_vec(args)
    }
    
    /// Parses from environment arguments (skipping program name).
    pub fn parse_env(&self) -> Result<ParsedArgs, CliError> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        self.parse_from_vec(args)
    }
    
    /// Internal parsing implementation.
    fn parse_from_vec(&self, args: Vec<String>) -> Result<ParsedArgs, CliError> {
        let mut parsed = ParsedArgs {
            flags: HashMap::new(),
            values: HashMap::new(),
            multi_values: HashMap::new(),
            positional: Vec::new(),
        };
        
        let mut i = 0;
        let mut positional_index = 0;
        
        while i < args.len() {
            let arg = &args[i];
            
            // Check for help/version first
            if arg == "--help" || arg == "-h" {
                return Err(CliError::HelpRequested);
            }
            if arg == "--version" || arg == "-V" {
                return Err(CliError::VersionRequested);
            }
            
            if let Some(name) = arg.strip_prefix("--") {
                // Long flag
                if let Some(arg_def) = self.find_arg_by_long(name) {
                    match arg_def.arg_type {
                        ArgType::Flag => {
                            parsed.flags.insert(arg_def.name.clone(), true);
                        }
                        ArgType::Value => {
                            i += 1;
                            if i >= args.len() {
                                return Err(CliError::MissingValue(arg_def.name.clone()));
                            }
                            let value = &args[i];
                            self.validate_value(arg_def, value)?;
                            parsed.values.insert(arg_def.name.clone(), value.clone());
                        }
                        ArgType::MultiValue => {
                            i += 1;
                            if i >= args.len() {
                                return Err(CliError::MissingValue(arg_def.name.clone()));
                            }
                            let value = &args[i];
                            self.validate_value(arg_def, value)?;
                            parsed.multi_values
                                .entry(arg_def.name.clone())
                                .or_default()
                                .push(value.clone());
                        }
                    }
                } else {
                    return Err(CliError::UnknownArg(arg.clone()));
                }
            } else if arg.starts_with('-') && arg.len() == 2 {
                // Short flag
                let flag = arg.chars().nth(1).unwrap();
                if let Some(arg_def) = self.find_arg_by_short(flag) {
                    match arg_def.arg_type {
                        ArgType::Flag => {
                            parsed.flags.insert(arg_def.name.clone(), true);
                        }
                        ArgType::Value => {
                            i += 1;
                            if i >= args.len() {
                                return Err(CliError::MissingValue(arg_def.name.clone()));
                            }
                            let value = &args[i];
                            self.validate_value(arg_def, value)?;
                            parsed.values.insert(arg_def.name.clone(), value.clone());
                        }
                        ArgType::MultiValue => {
                            i += 1;
                            if i >= args.len() {
                                return Err(CliError::MissingValue(arg_def.name.clone()));
                            }
                            let value = &args[i];
                            self.validate_value(arg_def, value)?;
                            parsed.multi_values
                                .entry(arg_def.name.clone())
                                .or_default()
                                .push(value.clone());
                        }
                    }
                } else {
                    return Err(CliError::UnknownArg(arg.clone()));
                }
            } else {
                // Positional argument
                if let Some(arg_def) = self.find_positional_arg(positional_index) {
                    self.validate_value(arg_def, arg)?;
                    parsed.values.insert(arg_def.name.clone(), arg.clone());
                }
                parsed.positional.push(arg.clone());
                positional_index += 1;
            }
            
            i += 1;
        }
        
        Ok(parsed)
    }
    
    /// Finds argument definition by long flag.
    fn find_arg_by_long(&self, long: &str) -> Option<&ArgDef> {
        self.args.iter().find(|arg| {
            arg.long.as_ref().is_some_and(|l| l == long)
        })
    }
    
    /// Finds argument definition by short flag.
    fn find_arg_by_short(&self, short: char) -> Option<&ArgDef> {
        self.args.iter().find(|arg| {
            arg.short == Some(short)
        })
    }
    
    /// Finds positional argument by index.
    fn find_positional_arg(&self, index: usize) -> Option<&ArgDef> {
        self.args.iter().find(|arg| {
            arg.positional && (arg.index == Some(index))
        })
    }
    
    /// Validates argument value.
    fn validate_value(&self, arg_def: &ArgDef, value: &str) -> Result<(), CliError> {
        if let Some(allowed) = &arg_def.allowed_values {
            if !allowed.contains(&value.to_string()) {
                return Err(CliError::InvalidValue {
                    arg: arg_def.name.clone(),
                    value: value.to_string(),
                    allowed: allowed.clone(),
                });
            }
        }
        Ok(())
    }
    
    /// Generates help text.
    pub fn generate_help(&self) -> String {
        let mut help = String::new();
        
        // Header
        help.push_str(&format!("{} {}\n", self.name, self.version));
        if !self.author.is_empty() {
            help.push_str(&format!("{}\n", self.author));
        }
        if !self.about.is_empty() {
            help.push_str(&format!("{}\n", self.about));
        }
        help.push('\n');
        
        // Usage
        help.push_str("USAGE:\n");
        help.push_str(&format!("    {} [OPTIONS]", self.name));
        
        // Add positional args to usage
        for arg in &self.args {
            if arg.positional {
                if let Some(value_name) = &arg.value_name {
                    help.push_str(&format!(" <{value_name}>"));
                } else {
                    help.push_str(&format!(" <{}>", arg.name.to_uppercase()));
                }
            }
        }
        help.push_str("\n\n");
        
        // Arguments
        if self.args.iter().any(|arg| arg.positional) {
            help.push_str("ARGS:\n");
            for arg in &self.args {
                if arg.positional {
                    let uppercase_name = arg.name.to_uppercase();
                    let value_name = arg.value_name.as_deref()
                        .unwrap_or(&uppercase_name);
                    help.push_str(&format!("    <{}>    {}\n", value_name, arg.help));
                }
            }
            help.push('\n');
        }
        
        // Options
        help.push_str("OPTIONS:\n");
        for arg in &self.args {
            if !arg.positional {
                let mut line = "    ".to_string();
                
                if let Some(short) = arg.short {
                    line.push_str(&format!("-{short}"));
                    if arg.long.is_some() {
                        line.push_str(", ");
                    }
                }
                
                if let Some(long) = &arg.long {
                    line.push_str(&format!("--{long}"));
                }
                
                if arg.arg_type == ArgType::Value {
                    let uppercase_name = arg.name.to_uppercase();
                    let value_name = arg.value_name.as_deref()
                        .unwrap_or(&uppercase_name);
                    line.push_str(&format!(" <{value_name}>"));
                }
                
                // Pad to align help text
                while line.len() < 24 {
                    line.push(' ');
                }
                line.push_str(&arg.help);
                
                if let Some(allowed) = &arg.allowed_values {
                    line.push_str(&format!(" [possible values: {}]", allowed.join(", ")));
                }
                
                help.push_str(&line);
                help.push('\n');
            }
        }
        
        // Built-in options
        help.push_str("    -h, --help       Print help information\n");
        help.push_str("    -V, --version    Print version information\n");
        
        help
    }
    
    /// Generates version text.
    pub fn generate_version(&self) -> String {
        format!("{} {}", self.name, self.version)
    }
    
    /// Prints help text to stdout.
    pub fn print_help(&self) {
        println!("{}", self.generate_help());
    }
    
    /// Prints version text to stdout.
    pub fn print_version(&self) {
        println!("{}", self.generate_version());
    }
}

impl ArgDef {
    /// Creates a new argument definition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            short: None,
            long: None,
            help: String::new(),
            value_name: None,
            arg_type: ArgType::Flag,
            positional: false,
            index: None,
            allowed_values: None,
        }
    }
    
    /// Sets the short flag.
    pub fn short(mut self, short: char) -> Self {
        self.short = Some(short);
        self
    }
    
    /// Sets the long flag.
    pub fn long(mut self, long: impl Into<String>) -> Self {
        self.long = Some(long.into());
        self
    }
    
    /// Sets the help text.
    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = help.into();
        self
    }
    
    /// Sets the value name.
    pub fn value_name(mut self, value_name: impl Into<String>) -> Self {
        self.value_name = Some(value_name.into());
        self
    }
    
    /// Sets the argument type to value.
    pub fn takes_value(mut self) -> Self {
        self.arg_type = ArgType::Value;
        self
    }
    
    /// Sets the argument as positional with index.
    pub fn index(mut self, index: usize) -> Self {
        self.positional = true;
        self.index = Some(index);
        self
    }
    
    /// Sets allowed values for validation.
    pub fn possible_values(mut self, values: &[&str]) -> Self {
        self.allowed_values = Some(values.iter().map(|s| s.to_string()).collect());
        self
    }
}

impl ParsedArgs {
    /// Gets a flag value.
    pub fn get_flag(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(false)
    }
    
    /// Gets a value.
    pub fn get_one<T>(&self, name: &str) -> Option<&str>
    where
        T: std::str::FromStr,
    {
        self.values.get(name).map(|s| s.as_str())
    }
    
    /// Gets the first positional argument.
    pub fn get_positional(&self, index: usize) -> Option<&str> {
        self.positional.get(index).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flag_parsing() {
        let cli = LightweightCli::new("test")
            .arg(ArgDef::new("verbose").short('v').long("verbose"));
        
        let parsed = cli.parse(&["--verbose"]).unwrap();
        assert!(parsed.get_flag("verbose"));
        
        let parsed = cli.parse(&["-v"]).unwrap();
        assert!(parsed.get_flag("verbose"));
        
        let parsed = cli.parse(&[]).unwrap();
        assert!(!parsed.get_flag("verbose"));
    }

    #[test] 
    fn test_value_parsing() {
        let cli = LightweightCli::new("test")
            .arg(ArgDef::new("file").short('f').long("file").takes_value());
        
        let parsed = cli.parse(&["--file", "test.txt"]).unwrap();
        assert_eq!(parsed.get_one::<String>("file"), Some("test.txt"));
        
        let parsed = cli.parse(&["-f", "test.txt"]).unwrap();
        assert_eq!(parsed.get_one::<String>("file"), Some("test.txt"));
    }

    #[test]
    fn test_positional_parsing() {
        let cli = LightweightCli::new("test")
            .arg(ArgDef::new("input").index(0).value_name("FILE"));
        
        let parsed = cli.parse(&["input.txt"]).unwrap();
        assert_eq!(parsed.get_one::<String>("input"), Some("input.txt"));
        assert_eq!(parsed.get_positional(0), Some("input.txt"));
    }

    #[test]
    fn test_help_generation() {
        let cli = LightweightCli::new("test")
            .version("1.0.0")
            .about("Test CLI")
            .arg(ArgDef::new("verbose").short('v').long("verbose").help("Enable verbose output"));
        
        let help = cli.generate_help();
        assert!(help.contains("test 1.0.0"));
        assert!(help.contains("Test CLI"));
        assert!(help.contains("-v, --verbose"));
        assert!(help.contains("Enable verbose output"));
    }

    #[test]
    fn test_value_validation() {
        let cli = LightweightCli::new("test")
            .arg(ArgDef::new("mode").long("mode").takes_value().possible_values(&["full", "minimal"]));
        
        let parsed = cli.parse(&["--mode", "full"]).unwrap();
        assert_eq!(parsed.get_one::<String>("mode"), Some("full"));
        
        let result = cli.parse(&["--mode", "invalid"]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CliError::InvalidValue { .. }));
    }
}
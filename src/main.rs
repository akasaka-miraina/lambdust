//! Lambdust Command-Line Interface
//!
//! This binary provides a REPL and file execution capabilities for the Lambdust language.

use clap::{Arg, ArgAction, Command};
use lambdust::{Lambdust, Error, Result};
use lambdust::runtime::{BootstrapIntegrationConfig, BootstrapMode, LibraryPathResolver, LibraryPathConfig};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[cfg(feature = "repl")]
use {
    colored::*,
    rustyline::{error::ReadlineError, DefaultEditor},
};

#[cfg(feature = "enhanced-repl")]
use lambdust::repl::{EnhancedRepl, ReplConfig};

fn main() -> Result<()> {
    // Capture command line arguments for the system interface
    let args: Vec<String> = std::env::args().collect();
    
    // Initialize the system state with command line arguments
    // This must be done before creating the Lambdust instance so the system functions work
    lambdust::stdlib::system::initialize_system_state(args);
    
    let matches = Command::new("lambdust")
        .version(lambdust::VERSION)
        .author("Lambdust Contributors")
        .about("A Scheme dialect with gradual typing and effect systems")
        .arg(
            Arg::new("file")
                .help("Lambdust source file to execute")
                .value_name("FILE")
                .index(1),
        )
        .arg(
            Arg::new("repl")
                .short('r')
                .long("repl")
                .help("Start interactive REPL")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("eval")
                .short('e')
                .long("eval")
                .help("Evaluate expression")
                .value_name("EXPR")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("type-check")
                .short('t')
                .long("type-check")
                .help("Type check only, don't evaluate")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("bootstrap-mode")
                .long("bootstrap-mode")
                .help("Bootstrap mode: full, minimal, or fallback")
                .value_name("MODE")
                .value_parser(["full", "minimal", "fallback"]),
        )
        .arg(
            Arg::new("lazy-loading")
                .long("lazy-loading")
                .help("Enable lazy loading of Scheme libraries")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("lib-dir")
                .long("lib-dir")
                .help("Override library directory path (alternative to LAMBDUST_LIB_DIR)")
                .value_name("PATH")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("validate-libs")
                .long("validate-libs")
                .help("Validate library setup and show status")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    // Handle library validation if requested
    if matches.get_flag("validate-libs") {
        return validate_library_setup(&matches);
    }

    // Configure bootstrap based on command line arguments
    let bootstrap_start = Instant::now();
    let bootstrap_config = create_bootstrap_config(&matches)?;
    let mut lambdust = create_lambdust_with_bootstrap(bootstrap_config, matches.get_flag("verbose"))?;
    
    if matches.get_flag("verbose") {
        let bootstrap_time = bootstrap_start.elapsed();
        println!("Bootstrap completed in {:?}", bootstrap_time);
    }

    // Handle different execution modes
    if let Some(expr) = matches.get_one::<String>("eval") {
        // Evaluate single expression
        eval_expression(&mut lambdust, expr, matches.get_flag("type-check"))?;
    } else if let Some(filename) = matches.get_one::<String>("file") {
        // Execute file
        execute_file(&mut lambdust, filename, matches.get_flag("type-check"))?;
    } else if matches.get_flag("repl") || matches.get_one::<String>("file").is_none() {
        // Start REPL if no file specified or explicitly requested
        #[cfg(feature = "enhanced-repl")]
        start_enhanced_repl(lambdust)?;
        
        #[cfg(all(feature = "repl", not(feature = "enhanced-repl")))]
        start_repl(&mut lambdust)?;
        
        #[cfg(not(any(feature = "repl", feature = "enhanced-repl")))]
        {
            eprintln!("REPL support not compiled in. Use --eval or provide a file.");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn eval_expression(lambdust: &mut Lambdust, expr: &str, type_check_only: bool) -> Result<()> {
    if type_check_only {
        // Type check only
        let tokens = lambdust.tokenize(expr, Some("<command-line>"))?;
        let ast = lambdust.parse(tokens)?;
        let expanded = lambdust.expand_macros(ast)?;
        let _typed = lambdust.type_check(expanded)?;
        println!("Type check passed");
    } else {
        // Evaluate
        let result = lambdust.eval(expr, Some("<command-line>"))?;
        println!("{result}");
    }
    Ok(())
}

fn execute_file(lambdust: &mut Lambdust, filename: &str, type_check_only: bool) -> Result<()> {
    let path = Path::new(filename);
    if !path.exists() {
        return Err(Box::new(Error::io_error(format!("File not found: {filename}".into().boxed())));
    }

    let source = fs::read_to_string(path)
        .map_err(|e| Error::io_error(format!("Failed to read file {filename}: {e}")))?;

    if type_check_only {
        // Type check only
        let tokens = lambdust.tokenize(&source, Some(filename))?;
        let ast = lambdust.parse(tokens)?;
        let expanded = lambdust.expand_macros(ast)?;
        let _typed = lambdust.type_check(expanded)?;
        println!("Type check passed for {filename}");
    } else {
        // Evaluate
        let result = lambdust.eval(&source, Some(filename))?;
        println!("{result}");
    }
    Ok(())
}

#[cfg(feature = "repl")]
fn start_repl(lambdust: &mut Lambdust) -> Result<()> {
    println!("{}", format!("Lambdust {} REPL", lambdust::VERSION).bright_blue().bold());
    println!("{}", "Type (exit) to quit".dimmed());
    println!();

    let mut rl = DefaultEditor::new()
        .map_err(|e| Error::io_error(format!("Failed to initialize REPL: {e}")))?;

    // Load history if available
    let history_file = dirs::home_dir()
        .map(|mut p| {
            p.push(".lambdust_history");
            p
        });

    if let Some(ref history_path) = history_file {
        let _ = rl.load_history(history_path);
    }

    let mut line_number = 1;

    loop {
        let prompt = format!("λust:{line_number}> ");
        
        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();
                
                if line.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(line);

                // Handle special REPL commands
                match line {
                    "(exit)" | "(quit)" | ":quit" | ":q" => break,
                    ":help" | ":h" => {
                        print_repl_help();
                        continue;
                    }
                    ":version" | ":v" => {
                        println!("Lambdust version: {}", lambdust::VERSION);
                        println!("Language version: {}", lambdust::LANGUAGE_VERSION);
                        continue;
                    }
                    _ => {}
                }

                // Evaluate expression
                match lambdust.eval(line, Some("<repl>")) {
                    Ok(result) => {
                        println!("{}", format!("{result}").bright_green());
                    }
                    Err(e) => {
                        eprintln!("{}", format!("Error: {e}").bright_red());
                    }
                }

                line_number += 1;
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }

    // Save history
    if let Some(history_path) = history_file {
        let _ = rl.save_history(&history_path);
    }

    println!("Goodbye!");
    Ok(())
}

#[cfg(feature = "enhanced-repl")]
fn start_enhanced_repl(lambdust: Lambdust) -> Result<()> {
    let config = ReplConfig::default();
    let mut repl = EnhancedRepl::with_defaults(lambdust)?;
    repl.run()
}

#[cfg(feature = "repl")]
fn print_repl_help() {
    println!("{}", "Lambdust REPL Commands:".bright_blue().bold());
    println!("  {}  - Show this help", ":help, :h".bright_yellow());
    println!("  {}  - Show version information", ":version, :v".bright_yellow());
    println!("  {}  - Exit the REPL", "(exit), (quit), :quit, :q".bright_yellow());
    println!();
    println!("{}", "Example expressions:".bright_blue().bold());
    println!("  {}  - Basic arithmetic", "(+ 1 2 3)".bright_cyan());
    println!("  {}  - Function definition", "(define (square x) (* x x))".bright_cyan());
    println!("  {}  - Type annotation", "(:: (+ 1 2) Number)".bright_cyan());
    println!();
}

/// Creates bootstrap configuration from command line arguments.
fn create_bootstrap_config(matches: &clap::ArgMatches) -> Result<BootstrapIntegrationConfig> {
    // Create library path configuration
    let lib_path_config = LibraryPathConfig {
        include_dev_paths: true,
        additional_paths: Vec::new(),
        enable_caching: true,
        lib_dir_override: matches.get_one::<String>("lib-dir").map(PathBuf::from),
    };

    // Determine bootstrap mode - check library setup first if not explicitly set
    let mode = match matches.get_one::<String>("bootstrap-mode").map(|s| s.as_str()) {
        Some("full") => BootstrapMode::Full,
        Some("minimal") => BootstrapMode::Minimal,
        Some("fallback") => BootstrapMode::Fallback,
        _ => {
            // Auto-detect based on environment
            if std::env::var("LAMBDUST_BOOTSTRAP_MODE").as_deref() == Ok("minimal") {
                BootstrapMode::Minimal
            } else if std::env::var("LAMBDUST_BOOTSTRAP_MODE").as_deref() == Ok("fallback") {
                BootstrapMode::Fallback
            } else {
                // Check if Scheme libraries are available using the lib path config
                determine_bootstrap_mode_with_lib_config(&lib_path_config)?
            }
        }
    };

    Ok(BootstrapIntegrationConfig {
        mode,
        verbose: matches.get_flag("verbose"),
        lazy_loading: matches.get_flag("lazy-loading") || 
                     std::env::var("LAMBDUST_LAZY_LOADING").as_deref() == Ok("true"),
        development_mode: std::env::var("LAMBDUST_DEV_MODE").as_deref() == Ok("true"),
        library_paths: if let Some(lib_dir) = matches.get_one::<String>("lib-dir") {
            vec![PathBuf::from(lib_dir)]
        } else {
            Vec::new()
        },
        ..Default::default()
    })
}

/// Determines bootstrap mode using library path configuration.
fn determine_bootstrap_mode_with_lib_config(lib_config: &LibraryPathConfig) -> Result<BootstrapMode> {
    match LibraryPathResolver::with_config(lib_config.clone()) {
        Ok(resolver) => {
            let validation = resolver.validate_library_setup()?;
            if validation.is_usable() {
                Ok(BootstrapMode::Full)
            } else {
                Ok(BootstrapMode::Minimal)
            }
        }
        Err(_) => Ok(BootstrapMode::Minimal)
    }
}

/// Validates library setup and displays results.
fn validate_library_setup(matches: &clap::ArgMatches) -> Result<()> {
    println!("Validating Lambdust library setup...\n");

    // Check environment variables
    println!("Environment variables:");
    if let Ok(lib_dir) = std::env::var("LAMBDUST_LIB_DIR") {
        println!("  LAMBDUST_LIB_DIR = {}", lib_dir);
    } else {
        println!("  LAMBDUST_LIB_DIR = (not set)");
    }

    if let Some(override_dir) = matches.get_one::<String>("lib-dir") {
        println!("  --lib-dir override = {}", override_dir);
    }
    println!();

    // Create library path configuration
    let lib_config = LibraryPathConfig {
        include_dev_paths: true,
        additional_paths: Vec::new(),
        enable_caching: true,
        lib_dir_override: matches.get_one::<String>("lib-dir").map(PathBuf::from),
    };

    // Try to create library path resolver
    match LibraryPathResolver::with_config(lib_config) {
        Ok(resolver) => {
            println!("Library path resolution:");
            if let Some(primary) = resolver.primary_lib_dir() {
                println!("  Primary library directory: {}", primary.display());
            } else {
                println!("  Primary library directory: (auto-detected)");
            }

            println!("  Search paths:");
            for (i, path) in resolver.search_paths().iter().enumerate() {
                println!("    {}: {}", i + 1, path.display());
            }
            println!();

            // Validate setup
            match resolver.validate_library_setup() {
                Ok(report) => {
                    println!("{}", report.summary());
                    
                    if report.is_usable() {
                        println!("✓ Library setup is usable. Full bootstrap mode available.");
                    } else {
                        println!("⚠ Library setup has issues. May fall back to minimal mode.");
                    }
                }
                Err(e) => {
                    println!("✗ Validation failed: {}", e);
                    return Err(e);
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to create library path resolver: {}", e);
            println!("This indicates a fundamental configuration issue.");
            return Err(e);
        }
    }

    Ok(())
}

/// Creates a Lambdust instance with bootstrap integration.
fn create_lambdust_with_bootstrap(config: BootstrapIntegrationConfig, verbose: bool) -> Result<Lambdust> {
    // Try to create with bootstrap integration
    match lambdust::Runtime::with_bootstrap_config(config.clone()) {
        Ok(runtime) => {
            if verbose {
                println!("Successfully initialized with {:?} bootstrap mode", config.mode);
            }
            Ok(Lambdust::with_runtime(runtime))
        }
        Err(e) => {
            if verbose {
                eprintln!("Bootstrap failed: {}. Using fallback...", e);
            }
            
            // Try fallback mode
            let fallback_config = BootstrapIntegrationConfig {
                mode: BootstrapMode::Fallback,
                verbose,
                ..config
            };
            
            match lambdust::Runtime::with_bootstrap_config(fallback_config) {
                Ok(runtime) => Ok(Lambdust::with_runtime(runtime)),
                Err(fallback_error) => {
                    // If even fallback fails, use the legacy constructor
                    eprintln!("Warning: Bootstrap system failed. Using legacy initialization.");
                    eprintln!("Original error: {}", e);
                    eprintln!("Fallback error: {}", fallback_error);
                    Ok(Lambdust::new())
                }
            }
        }
    }
}
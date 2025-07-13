//! Command Line Interface for REPL
//!
//! このモジュールはREPLのコマンドライン引数処理とメインエントリーポイントを
//! 提供します。

use super::config::{ReplConfig, VERSION};
use super::core::Repl;
use clap::{Arg, Command};

/// Main entry point for the REPL
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        debug_prompt: "λust[debug]> ".to_string(),
        show_banner: !matches.get_flag("no-banner"),
        enable_history: !matches.get_flag("no-history"),
        history_file: if matches.get_flag("no-history") {
            None
        } else {
            Some(".lambdust_history".to_string())
        },
        enable_syntax_highlighting: true,
        enable_tab_completion: true,
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
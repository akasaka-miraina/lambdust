//! REPL Configuration and Constants
//!
//! このモジュールはREPLの設定と定数を定義します。

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BANNER: &str = r#"
┌─────────────────────────────────────────┐
│  Lambdust (λust) R7RS Scheme Interpreter │
│  Version {version}                       │
│  Type (exit) to quit, (help) for help   │
└─────────────────────────────────────────┘
"#;

pub const HELP_TEXT: &str = r#"Available commands:
  (exit)       - Exit the REPL
  (help)       - Show this help message
  (clear)      - Clear the screen
  (env)        - Show current environment bindings
  (load "file") - Load and evaluate a file
  (reset)      - Reset the interpreter state
  (debug on)   - Enable debug mode with breakpoints
  (debug off)  - Disable debug mode
  (break)      - Set breakpoint at next evaluation
  (step)       - Step through evaluation (debug mode)
  (continue)   - Continue execution from breakpoint
  (backtrace)  - Show call stack (debug mode)

Tab Completion:
  - Press Tab to complete function names, special forms, and filenames
  - Functions are shown in green, special forms in red

Syntax Highlighting:
  - Special forms: red
  - Builtin functions: green
  - Numbers: light blue
  - Strings: yellow
  - Comments: gray
  - Parentheses: colored by depth

Examples:
  > (+ 1 2 3)
  6
  > (define square (lambda (x) (* x x)))
  > (square 5)
  25
  > (map square '(1 2 3 4))
  (1 4 9 16)
"#;

/// Debug mode state
#[derive(Debug, Clone, Default)]
pub struct DebugState {
    pub enabled: bool,
    pub breakpoint_set: bool,
    pub step_mode: bool,
    pub call_stack: Vec<String>,
    pub last_expression: Option<String>,
}

/// REPL configuration
#[derive(Debug, Clone)]
pub struct ReplConfig {
    pub prompt: String,
    pub continuation_prompt: String,
    pub debug_prompt: String,
    pub show_banner: bool,
    pub enable_history: bool,
    pub history_file: Option<String>,
    pub enable_syntax_highlighting: bool,
    pub enable_tab_completion: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            prompt: "λust> ".to_string(),
            continuation_prompt: "   ... ".to_string(),
            debug_prompt: "λust[debug]> ".to_string(),
            show_banner: true,
            enable_history: true,
            history_file: Some(".lambdust_history".to_string()),
            enable_syntax_highlighting: true,
            enable_tab_completion: true,
        }
    }
}
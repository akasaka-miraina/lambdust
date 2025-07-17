//! Lambdust REPL Module
//!
//! このモジュールはインタラクティブなScheme REPLの実装を提供します。
//! モジュール構成：
//!
//! - `config`: REPL設定と定数
//! - `completion`: タブ補完とシンタックスハイライト
//! - `core`: メインREPL実装
//! - `cli`: コマンドライン引数処理

pub mod config;
pub mod completion;
pub mod core;
pub mod cli;

// Re-export key types for backward compatibility
pub use config::{DebugState, ReplConfig, BANNER, HELP_TEXT, VERSION};
pub use completion::SchemeHelper;
pub use core::Repl;
pub use cli::main;
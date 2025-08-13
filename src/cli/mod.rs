//! Lightweight command-line interface for Lambdust.
//!
//! This module provides a minimal CLI system to replace the clap dependency,
//! achieving significant binary size reduction while maintaining essential
//! command-line functionality for the Lambdust interpreter.

pub mod lightweight_parser;

pub use lightweight_parser::{LightweightCli, ArgDef, ArgType, ParsedArgs, CliError};
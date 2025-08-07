use super::{RecoveryConfig, Parser};
use crate::diagnostics::SourceMap;
use crate::lexer::Token;
use std::sync::Arc;

#[derive(Debug)]
pub struct ParserBuilder {
    recovery_config: RecoveryConfig,
    source_map: Option<Arc<SourceMap>>,
}

impl ParserBuilder {
    /// Creates a new parser builder.
    pub fn new() -> Self {
        Self {
            recovery_config: RecoveryConfig::default(),
            source_map: None,
        }
    }
    
    /// Sets the recovery configuration.
    pub fn with_recovery_config(mut self, config: RecoveryConfig) -> Self {
        self.recovery_config = config;
        self
    }
    
    /// Sets the source map for enhanced error reporting.
    pub fn with_source_map(mut self, source_map: Arc<SourceMap>) -> Self {
        self.source_map = Some(source_map);
        self
    }
    
    /// Builds a parser with the given tokens.
    pub fn build(self, tokens: Vec<Token>) -> Parser {
        Parser::with_settings(
            tokens,
            self.recovery_config.max_errors,
            self.recovery_config.aggressive_recovery,
        )
    }
    
    /// Gets the recovery configuration.
    pub fn recovery_config(&self) -> &RecoveryConfig {
        &self.recovery_config
    }
    
    /// Gets the source map.
    pub fn source_map(&self) -> Option<&Arc<SourceMap>> {
        self.source_map.as_ref()
    }
}

impl Default for ParserBuilder {
    fn default() -> Self {
        Self::new()
    }
}
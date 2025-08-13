//! Default effect interpreter implementation

use super::interpreter_configuration::InterpreterConfiguration;

/// Default effect interpreter implementation
#[derive(Debug)]
pub struct DefaultEffectInterpreter {
    /// IO context for handling IO effects
    pub io_context: crate::effects::IOContext,
    
    /// Configuration
    pub config: InterpreterConfiguration,
}
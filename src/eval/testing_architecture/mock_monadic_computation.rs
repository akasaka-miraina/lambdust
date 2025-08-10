use crate::eval::Value;

/// Simplified mock monadic computation
#[derive(Debug, Clone)]
pub enum MockMonadicComputation {
    /// Pure computation returning a value
    Pure(Value),
    /// IO computation returning a value
    IO(Value),
    /// Maybe computation with optional value
    Maybe(Option<Value>),
    /// Either computation with result or error
    Either(std::result::Result<Value, String>),
}
use super::AritySpec;

/// Type signature for FFI functions.
#[derive(Debug, Clone)]
pub struct FfiSignature {
    /// Function name
    pub name: String,
    /// Expected argument count
    pub arity: AritySpec,
    /// Parameter type descriptions (for error reporting)
    pub parameter_types: Vec<String>,
    /// Return type description
    pub return_type: String,
    /// Optional documentation
    pub documentation: Option<String>,
}
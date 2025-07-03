//! Port types for I/O operations

/// Port types for I/O
#[derive(Clone)]
pub enum Port {
    /// Input port
    Input,
    /// Output port
    Output,
    /// String port
    String(String),
}

impl std::fmt::Debug for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input => write!(f, "Input"),
            Self::Output => write!(f, "Output"),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
        }
    }
}

impl PartialEq for Port {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
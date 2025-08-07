/// A unique identifier for a module.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId {
    /// The module name components (e.g., ["scheme", "base"] for (scheme base))
    pub components: Vec<String>,
    /// The module namespace (builtin, r7rs, user, file)
    pub namespace: ModuleNamespace,
}

impl ModuleId {
    /// Creates a new ModuleId with the given namespace and components.
    pub fn new(namespace: ModuleNamespace, components: Vec<String>) -> Self {
        Self { namespace, components }
    }
}

/// Module namespace classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ModuleNamespace {
    /// Lambdust built-in modules (lambdust prefix)
    Builtin,
    /// R7RS standard library modules (scheme prefix)
    R7RS,
    /// SRFI modules (srfi prefix)
    SRFI,
    /// User-defined modules (user prefix)
    User,
    /// File-based modules (file prefix)
    File,
}

/// Formats a module ID for display.
pub fn format_module_id(id: &ModuleId) -> String {
    match id.namespace {
        ModuleNamespace::Builtin => format!("(lambdust {})", id.components.join(" ")),
        ModuleNamespace::R7RS => format!("(scheme {})", id.components.join(" ")),
        ModuleNamespace::SRFI => {
            if id.components.len() == 1 {
                format!("(srfi {})", id.components[0])
            } else {
                format!("(srfi ({}))", id.components.join(" "))
            }
        }
        ModuleNamespace::User => format!("(user {})", id.components.join(" ")),
        ModuleNamespace::File => format!("(file \"{}\")", id.components.join("/")),
    }
}
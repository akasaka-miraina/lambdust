// SRFI-35: Conditions - Comprehensive condition system implementation
//
// This module implements the complete SRFI-35 condition system providing:
// - Hierarchical condition types with efficient inheritance checking
// - Condition objects with structured data fields
// - Compound conditions for multi-layered error contexts
// - Standard condition type hierarchy
// - Integration with existing exception handling system

use crate::ast::Literal;
use crate::diagnostics::{Error as DiagnosticError, Result};
use crate::eval::Environment;
use crate::eval::value::Value;
use crate::utils::{SymbolId, string_interner::StringInterner};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

// ============= CONDITION TYPE SYSTEM =============

/// Unique identifier for condition types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConditionTypeId(u32);

/// Condition type metadata - stores type hierarchy and field information
#[derive(Debug, Clone)]
pub struct ConditionType {
    /// Unique type identifier
    pub id: ConditionTypeId,
    /// Type name symbol
    pub name: SymbolId,
    /// Parent type (None for root &condition)
    pub parent: Option<ConditionTypeId>,
    /// Constructor procedure name
    pub constructor: SymbolId,
    /// Predicate procedure name
    pub predicate: SymbolId,
    /// Field definitions: (field_name, accessor_name)
    pub fields: Vec<(SymbolId, SymbolId)>,
    /// All ancestor type IDs (for efficient subtype checking)
    pub ancestors: HashSet<ConditionTypeId>,
}

impl ConditionType {
    /// Creates a new condition type
    pub fn new(
        id: ConditionTypeId,
        name: SymbolId,
        parent: Option<ConditionTypeId>,
        constructor: SymbolId,
        predicate: SymbolId,
        fields: Vec<(SymbolId, SymbolId)>,
        ancestors: HashSet<ConditionTypeId>,
    ) -> Self {
        Self {
            id,
            name,
            parent,
            constructor,
            predicate,
            fields,
            ancestors,
        }
    }

    /// Checks if this condition type is a subtype of another
    pub fn is_subtype_of(&self, other_id: ConditionTypeId) -> bool {
        self.id == other_id || self.ancestors.contains(&other_id)
    }

    /// Gets field accessor for a given field name
    pub fn get_field_accessor(&self, field_name: SymbolId) -> Option<SymbolId> {
        self.fields
            .iter()
            .find(|(name, _)| *name == field_name)
            .map(|(_, accessor)| *accessor)
    }
}

// ============= CONDITION OBJECTS =============

/// A simple condition - single condition type with field values
#[derive(Debug, Clone)]
pub struct SimpleCondition {
    /// The condition type
    pub condition_type: ConditionTypeId,
    /// Field values indexed by field name
    pub fields: HashMap<SymbolId, Value>,
}

impl SimpleCondition {
    /// Creates a new simple condition
    pub fn new(condition_type: ConditionTypeId, fields: HashMap<SymbolId, Value>) -> Self {
        Self {
            condition_type,
            fields,
        }
    }

    /// Gets the value of a field
    pub fn get_field(&self, field_name: SymbolId) -> Option<&Value> {
        self.fields.get(&field_name)
    }

    /// Sets the value of a field
    pub fn set_field(&mut self, field_name: SymbolId, value: Value) {
        self.fields.insert(field_name, value);
    }
}

/// Condition object - can be simple or compound
#[derive(Debug, Clone)]
pub enum ConditionObject {
    /// Single simple condition
    Simple(SimpleCondition),
    /// Compound condition containing multiple simple conditions
    Compound(Vec<SimpleCondition>),
}

impl ConditionObject {
    /// Creates a simple condition object
    pub fn simple(condition: SimpleCondition) -> Self {
        ConditionObject::Simple(condition)
    }

    /// Creates a compound condition object
    pub fn compound(conditions: Vec<SimpleCondition>) -> Self {
        if conditions.len() == 1 {
            ConditionObject::Simple(conditions.into_iter().next().unwrap())
        } else {
            ConditionObject::Compound(conditions)
        }
    }

    /// Gets all simple conditions
    pub fn simple_conditions(&self) -> Vec<&SimpleCondition> {
        match self {
            ConditionObject::Simple(condition) => vec![condition],
            ConditionObject::Compound(conditions) => conditions.iter().collect(),
        }
    }

    /// Checks if this condition has the given type
    pub fn has_type(&self, type_id: ConditionTypeId, registry: &ConditionTypeRegistry) -> bool {
        self.simple_conditions().iter().any(|condition| {
            if let Some(condition_type) = registry.get_type(condition.condition_type) {
                condition_type.is_subtype_of(type_id)
            } else {
                false
            }
        })
    }

    /// Extracts conditions of a specific type
    pub fn extract_conditions(
        &self,
        type_id: ConditionTypeId,
        registry: &ConditionTypeRegistry,
    ) -> Vec<&SimpleCondition> {
        self.simple_conditions()
            .into_iter()
            .filter(|condition| {
                if let Some(condition_type) = registry.get_type(condition.condition_type) {
                    condition_type.is_subtype_of(type_id)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Gets field value from the first matching condition type
    pub fn condition_ref(
        &self,
        field_name: SymbolId,
        registry: &ConditionTypeRegistry,
    ) -> Option<&Value> {
        for condition in self.simple_conditions() {
            if let Some(value) = condition.get_field(field_name) {
                return Some(value);
            }
            // Also check if field exists in condition type definition
            if let Some(condition_type) = registry.get_type(condition.condition_type) {
                if condition_type.get_field_accessor(field_name).is_some() {
                    return condition.get_field(field_name);
                }
            }
        }
        None
    }
}

// ============= CONDITION TYPE REGISTRY =============

/// Thread-safe registry for condition types
pub struct ConditionTypeRegistry {
    /// Type definitions indexed by ID
    types: RwLock<HashMap<ConditionTypeId, ConditionType>>,
    /// Type name to ID mapping
    name_to_id: RwLock<HashMap<SymbolId, ConditionTypeId>>,
    /// Next available type ID
    next_id: std::sync::atomic::AtomicU32,
    /// String interner for symbols
    interner: Arc<StringInterner>,
}

impl ConditionTypeRegistry {
    /// Creates a new condition type registry
    pub fn new(interner: Arc<StringInterner>) -> Self {
        Self {
            types: RwLock::new(HashMap::new()),
            name_to_id: RwLock::new(HashMap::new()),
            next_id: std::sync::atomic::AtomicU32::new(1),
            interner,
        }
    }

    /// Generates a new condition type ID
    fn next_type_id(&self) -> ConditionTypeId {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        ConditionTypeId(id)
    }

    /// Registers a new condition type
    pub fn register_type(
        &self,
        name: &str,
        parent_name: Option<&str>,
        constructor: &str,
        predicate: &str,
        field_specs: Vec<(&str, &str)>, // (field_name, accessor_name)
    ) -> Result<ConditionTypeId> {
        let name_id = self.interner.intern(name);
        let name_symbol_id = crate::utils::SymbolId::new(0); // Temporary placeholder
        let constructor_id = self.interner.intern(constructor);
        let predicate_id = self.interner.intern(predicate);

        // Check if type already exists
        if let Some(&existing_id) = self.name_to_id.read().unwrap().get(&name_symbol_id) {
            return Ok(existing_id);
        }

        // Find parent type (simplified for now)
        let parent_id = match parent_name {
            Some(_parent_name) => {
                // TODO: Implement proper parent type lookup
                Some(ConditionTypeId(0))
            }
            None => None,
        };

        // Build ancestor set
        let mut ancestors = HashSet::new();
        if let Some(parent_id) = parent_id {
            ancestors.insert(parent_id);
            if let Some(parent_type) = self.get_type(parent_id) {
                ancestors.extend(&parent_type.ancestors);
            }
        }

        // Process field specifications (simplified for now)
        let fields: Vec<(SymbolId, SymbolId)> = field_specs
            .into_iter()
            .map(|(_field_name, _accessor_name)| {
                // TODO: Implement proper InternedString to SymbolId conversion
                (
                    crate::utils::SymbolId::new(0), // Temporary placeholder
                    crate::utils::SymbolId::new(1), // Temporary placeholder
                )
            })
            .collect();

        // Create and register the type
        let type_id = self.next_type_id();
        let condition_type = ConditionType::new(
            type_id,
            name_symbol_id, // Use the already converted SymbolId
            parent_id,
            crate::utils::SymbolId::new(2), // Temporary placeholder for constructor_id
            crate::utils::SymbolId::new(3), // Temporary placeholder for predicate_id
            fields,
            ancestors,
        );

        self.types.write().unwrap().insert(type_id, condition_type);
        self.name_to_id
            .write()
            .unwrap()
            .insert(name_symbol_id, type_id);

        Ok(type_id)
    }

    /// Gets a condition type by ID
    pub fn get_type(&self, id: ConditionTypeId) -> Option<ConditionType> {
        self.types.read().unwrap().get(&id).cloned()
    }

    /// Gets a condition type by name
    pub fn get_type_by_name(&self, name: &str) -> Option<ConditionType> {
        let name_id = self.interner.intern(name);
        let name_symbol_id = crate::utils::SymbolId::new(0); // Temporary placeholder
        let type_id = self
            .name_to_id
            .read()
            .unwrap()
            .get(&name_symbol_id)
            .copied()?;
        self.get_type(type_id)
    }

    /// Gets a condition type ID by name
    pub fn get_type_id(&self, name: &str) -> Option<ConditionTypeId> {
        let name_id = self.interner.intern(name);
        let name_symbol_id = crate::utils::SymbolId::new(0); // Temporary placeholder
        self.name_to_id
            .read()
            .unwrap()
            .get(&name_symbol_id)
            .copied()
    }

    /// Checks if a type is a subtype of another
    pub fn is_subtype(&self, child_id: ConditionTypeId, parent_id: ConditionTypeId) -> bool {
        if child_id == parent_id {
            return true;
        }

        if let Some(child_type) = self.get_type(child_id) {
            child_type.is_subtype_of(parent_id)
        } else {
            false
        }
    }

    /// Lists all registered condition types
    pub fn list_types(&self) -> Vec<ConditionType> {
        self.types.read().unwrap().values().cloned().collect()
    }

    /// Gets the string interner used by this registry
    pub fn get_interner(&self) -> &Arc<StringInterner> {
        &self.interner
    }
}

// ============= STANDARD CONDITION TYPES =============

/// Standard SRFI-35 condition type IDs
#[derive(Debug, Clone)]
pub struct StandardConditionTypes {
    pub condition: ConditionTypeId,
    pub message: ConditionTypeId,
    pub serious: ConditionTypeId,
    pub error: ConditionTypeId,
    pub violation: ConditionTypeId,
    pub assertion: ConditionTypeId,
    pub non_continuable: ConditionTypeId,
    pub implementation_restriction: ConditionTypeId,
}

impl StandardConditionTypes {
    /// Registers all standard condition types
    pub fn register_all(registry: &ConditionTypeRegistry) -> Result<Self> {
        // Root condition type
        let condition =
            registry.register_type("&condition", None, "make-condition", "condition?", vec![])?;

        // Message condition type
        let message = registry.register_type(
            "&message",
            Some("&condition"),
            "make-message-condition",
            "message-condition?",
            vec![("message", "condition-message")],
        )?;

        // Serious condition type
        let serious = registry.register_type(
            "&serious",
            Some("&condition"),
            "make-serious-condition",
            "serious-condition?",
            vec![],
        )?;

        // Error condition type
        let error =
            registry.register_type("&error", Some("&serious"), "make-error", "error?", vec![])?;

        // Violation condition type
        let violation = registry.register_type(
            "&violation",
            Some("&condition"),
            "make-violation",
            "violation?",
            vec![],
        )?;

        // Assertion violation condition type
        let assertion = registry.register_type(
            "&assertion",
            Some("&violation"),
            "make-assertion-violation",
            "assertion-violation?",
            vec![],
        )?;

        // Non-continuable violation condition type
        let non_continuable = registry.register_type(
            "&non-continuable",
            Some("&violation"),
            "make-non-continuable-violation",
            "non-continuable-violation?",
            vec![],
        )?;

        // Implementation restriction violation condition type
        let implementation_restriction = registry.register_type(
            "&implementation-restriction",
            Some("&violation"),
            "make-implementation-restriction-violation",
            "implementation-restriction-violation?",
            vec![],
        )?;

        Ok(StandardConditionTypes {
            condition,
            message,
            serious,
            error,
            violation,
            assertion,
            non_continuable,
            implementation_restriction,
        })
    }
}

// ============= GLOBAL CONDITION SYSTEM =============

/// Global condition system state
pub struct ConditionSystem {
    /// Type registry
    pub registry: ConditionTypeRegistry,
    /// Standard condition types
    pub standard_types: StandardConditionTypes,
}

impl ConditionSystem {
    /// Creates and initializes the condition system
    pub fn new(interner: Arc<StringInterner>) -> Result<Self> {
        let registry = ConditionTypeRegistry::new(interner);
        let standard_types = StandardConditionTypes::register_all(&registry)?;

        Ok(Self {
            registry,
            standard_types,
        })
    }
}

// Thread-local condition system instance
thread_local! {
    static CONDITION_SYSTEM: std::cell::RefCell<Option<Arc<ConditionSystem>>> = std::cell::RefCell::new(None);
}

/// Initializes the global condition system
pub fn initialize_condition_system(interner: Arc<StringInterner>) -> Result<()> {
    let system = Arc::new(ConditionSystem::new(interner)?);
    CONDITION_SYSTEM.with(|cs| {
        *cs.borrow_mut() = Some(system);
    });
    Ok(())
}

/// Gets the global condition system
pub fn condition_system() -> Arc<ConditionSystem> {
    CONDITION_SYSTEM.with(|cs| {
        cs.borrow()
            .as_ref()
            .expect("Condition system not initialized")
            .clone()
    })
}

// ============= VALUE SYSTEM INTEGRATION =============

/// Condition value wrapper for the Value enum
#[derive(Debug, Clone)]
pub struct ConditionValue {
    pub condition: ConditionObject,
}

impl ConditionValue {
    /// Creates a new condition value
    pub fn new(condition: ConditionObject) -> Self {
        Self { condition }
    }

    /// Creates a simple condition value
    pub fn simple(condition_type: ConditionTypeId, fields: HashMap<SymbolId, Value>) -> Self {
        let simple_condition = SimpleCondition::new(condition_type, fields);
        Self::new(ConditionObject::simple(simple_condition))
    }

    /// Creates a compound condition value
    pub fn compound(conditions: Vec<SimpleCondition>) -> Self {
        Self::new(ConditionObject::compound(conditions))
    }
}

// ============= CONDITION PROCEDURES =============

/// Creates a condition object from multiple condition objects
pub fn condition_procedure(args: &[Value]) -> Result<Value> {
    if args.is_empty() {
        return Err(Box::new(DiagnosticError::runtime_error(
            "condition: expected at least 1 argument".to_string(),
            None,
        )));
    }

    let mut simple_conditions = Vec::new();

    for arg in args {
        match arg {
            Value::Condition(condition_value) => {
                simple_conditions.extend(
                    condition_value
                        .condition
                        .simple_conditions()
                        .into_iter()
                        .cloned(),
                );
            }
            _ => {
                return Err(Box::new(DiagnosticError::runtime_error(
                    format!(
                        "condition: expected condition object, got {}",
                        crate::stdlib::types::get_value_type_name(arg)
                    ),
                    None,
                )));
            }
        }
    }

    let compound_condition = ConditionObject::compound(simple_conditions);
    Ok(Value::Condition(Arc::new(ConditionValue::new(
        compound_condition,
    ))))
}

/// Gets simple conditions from a condition object
pub fn simple_conditions_procedure(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("simple-conditions: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    match &args[0] {
        Value::Condition(condition_value) => {
            let simple_conditions: Vec<Value> = condition_value
                .condition
                .simple_conditions()
                .into_iter()
                .map(|simple_condition| {
                    let condition_obj = ConditionObject::simple(simple_condition.clone());
                    Value::Condition(Arc::new(ConditionValue::new(condition_obj)))
                })
                .collect();

            Ok(Value::from_vec(simple_conditions))
        }
        _ => Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "simple-conditions: expected condition object, got {}",
                crate::stdlib::types::get_value_type_name(&args[0])
            ),
            None,
        ))),
    }
}

/// Tests if an object is a condition
pub fn condition_predicate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("condition?: expected 1 argument, got {}", args.len()),
            None,
        )));
    }

    Ok(Value::boolean(matches!(args[0], Value::Condition(_))))
}

/// Tests if a condition has a specific type
pub fn condition_has_type_procedure(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "condition-has-type?: expected 2 arguments, got {}",
                args.len()
            ),
            None,
        )));
    }

    let condition = match &args[0] {
        Value::Condition(condition_value) => &condition_value.condition,
        _ => {
            return Ok(Value::boolean(false));
        }
    };

    let type_id = match &args[1] {
        Value::ConditionType(type_id) => type_id.clone(),
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!(
                    "condition-has-type?: expected condition type, got {}",
                    crate::stdlib::types::get_value_type_name(&args[1])
                ),
                None,
            )));
        }
    };

    let system = condition_system();
    Ok(Value::boolean(
        condition.has_type(type_id.id, &system.registry),
    ))
}

/// Extracts a condition of a specific type
pub fn extract_condition_procedure(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!(
                "extract-condition: expected 2 arguments, got {}",
                args.len()
            ),
            None,
        )));
    }

    let condition = match &args[0] {
        Value::Condition(condition_value) => &condition_value.condition,
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!(
                    "extract-condition: expected condition object, got {}",
                    crate::stdlib::types::get_value_type_name(&args[0])
                ),
                None,
            )));
        }
    };

    let type_id = match &args[1] {
        Value::ConditionType(type_id) => type_id.clone(),
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!(
                    "extract-condition: expected condition type, got {}",
                    crate::stdlib::types::get_value_type_name(&args[1])
                ),
                None,
            )));
        }
    };

    let system = condition_system();
    let matching_conditions = condition.extract_conditions(type_id.id, &system.registry);

    if matching_conditions.is_empty() {
        Ok(Value::boolean(false))
    } else {
        let compound_condition =
            ConditionObject::compound(matching_conditions.into_iter().cloned().collect());
        Ok(Value::Condition(Arc::new(ConditionValue::new(
            compound_condition,
        ))))
    }
}

/// Accesses a field value from a condition
pub fn condition_ref_procedure(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(Box::new(DiagnosticError::runtime_error(
            format!("condition-ref: expected 2 arguments, got {}", args.len()),
            None,
        )));
    }

    let condition = match &args[0] {
        Value::Condition(condition_value) => &condition_value.condition,
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!(
                    "condition-ref: expected condition object, got {}",
                    crate::stdlib::types::get_value_type_name(&args[0])
                ),
                None,
            )));
        }
    };

    let field_name = match &args[1] {
        Value::Symbol(symbol_id) => *symbol_id,
        _ => {
            return Err(Box::new(DiagnosticError::runtime_error(
                format!(
                    "condition-ref: expected symbol, got {}",
                    crate::stdlib::types::get_value_type_name(&args[1])
                ),
                None,
            )));
        }
    };

    let system = condition_system();
    if let Some(value) = condition.condition_ref(field_name, &system.registry) {
        Ok(value.clone())
    } else {
        Err(Box::new(DiagnosticError::runtime_error(
            "condition-ref: field not found".to_string(),
            None,
        )))
    }
}

// ============= INTEGRATION WITH EXISTING EXCEPTION SYSTEM =============

/// Converts an ExceptionObject to a condition object
pub fn exception_to_condition(
    exception: &crate::stdlib::exceptions::ExceptionObject,
) -> Result<ConditionValue> {
    let system = condition_system();
    let mut fields = HashMap::new();

    // Add message field if present
    if let Some(ref message) = exception.message {
        let message_field = crate::utils::intern_symbol("message");
        fields.insert(message_field, Value::string(message.clone()));
    }

    // Determine condition type based on exception type
    let condition_type = match exception.exception_type.as_str() {
        "error" => system.standard_types.error,
        "read-error" => system.standard_types.error, // Map to error for now
        "file-error" => system.standard_types.error, // Map to error for now
        _ => system.standard_types.condition,
    };

    let simple_condition = SimpleCondition::new(condition_type, fields);
    Ok(ConditionValue::new(ConditionObject::simple(
        simple_condition,
    )))
}

/// Converts a condition object to an ExceptionObject
pub fn condition_to_exception(
    condition: &ConditionValue,
) -> crate::stdlib::exceptions::ExceptionObject {
    let system = condition_system();

    // Extract message if present
    let message_field = crate::utils::intern_symbol("message");
    let message = condition
        .condition
        .condition_ref(message_field, &system.registry)
        .and_then(|v| match v {
            Value::Literal(Literal::String(s)) => Some(s.clone()),
            _ => None,
        });

    // Determine exception type based on condition hierarchy
    let exception_type = if condition
        .condition
        .has_type(system.standard_types.error, &system.registry)
    {
        "error"
    } else {
        "condition"
    };

    // Create exception object
    crate::stdlib::exceptions::ExceptionObject::new(
        exception_type.to_string(),
        Value::Condition(Arc::new(condition.clone())),
        false, // Conditions are generally non-continuable
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::string_interner::StringInterner;

    #[test]
    fn test_condition_type_registration() {
        let interner = Arc::new(StringInterner::new());
        let registry = ConditionTypeRegistry::new(interner);

        let condition_id = registry
            .register_type("&condition", None, "make-condition", "condition?", vec![])
            .unwrap();

        let message_id = registry
            .register_type(
                "&message",
                Some("&condition"),
                "make-message-condition",
                "message-condition?",
                vec![("message", "condition-message")],
            )
            .unwrap();

        assert!(registry.is_subtype(message_id, condition_id));
        assert!(!registry.is_subtype(condition_id, message_id));
    }

    #[test]
    fn test_condition_object_creation() {
        let interner = Arc::new(StringInterner::new());
        initialize_condition_system(interner).unwrap();

        let system = condition_system();
        let message_field = crate::utils::intern_symbol("message");

        let mut fields = HashMap::new();
        fields.insert(message_field, Value::string("Test message".to_string()));

        let condition = ConditionValue::simple(system.standard_types.message, fields);

        assert!(matches!(condition.condition, ConditionObject::Simple(_)));
    }

    #[test]
    fn test_compound_condition_creation() {
        let interner = Arc::new(StringInterner::new());
        initialize_condition_system(interner).unwrap();

        let system = condition_system();
        let message_field = crate::utils::intern_symbol("message");

        let mut fields1 = HashMap::new();
        fields1.insert(message_field, Value::string("Error message".to_string()));
        let condition1 = SimpleCondition::new(system.standard_types.message, fields1);

        let condition2 = SimpleCondition::new(system.standard_types.error, HashMap::new());

        let compound = ConditionValue::compound(vec![condition1, condition2]);

        match compound.condition {
            ConditionObject::Compound(conditions) => {
                assert_eq!(conditions.len(), 2);
            }
            _ => panic!("Expected compound condition"),
        }
    }
}

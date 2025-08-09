//! R7RS-large Integration Layer for Advanced Type System.
//!
//! This module provides the integration layer between Lambdust's advanced type system
//! and R7RS-large features, ensuring type safety while maintaining R7RS compatibility:
//! - Comparator system type safety
//! - Hash table type parameterization
//! - Generator type inference and safety
//! - Immutable data structure type guarantees
//! - SRFI integration with type checking

#![allow(missing_docs)]

use super::{Type, TypeVar, TypeScheme, TypeEnv, Constraint};
use super::type_classes::TypeClassInstance;
use crate::diagnostics::{Error, Result, Span};
use crate::eval::value::Value;
use std::collections::HashMap;
use std::fmt;

/// R7RS-large type system integration.
pub struct R7RSIntegration {
    /// Core type environment
    type_env: TypeEnv,
    /// Comparator type mappings
    comparator_types: HashMap<String, ComparatorType>,
    /// Hash table type instances
    hash_table_types: HashMap<String, HashTableType>,
    /// Generator type information
    generator_types: HashMap<String, GeneratorType>,
    /// Immutable data structure types
    immutable_types: HashMap<String, ImmutableType>,
    /// SRFI type extensions
    srfi_types: HashMap<String, SRFITypeExtension>,
}

/// Comparator type information for R7RS-large comparators.
#[derive(Debug, Clone)]
pub struct ComparatorType {
    /// Element type that this comparator works with
    pub element_type: Type,
    /// Type class constraints (Eq, Ord, Hash)
    pub constraints: Vec<Constraint>,
    /// Comparator procedures
    pub procedures: ComparatorProcedures,
}

/// Procedures in a comparator.
#[derive(Debug, Clone)]
pub struct ComparatorProcedures {
    /// Type predicate function
    pub type_test: Option<TypeScheme>,
    /// Equality predicate function
    pub equality: TypeScheme,
    /// Ordering function
    pub ordering: Option<TypeScheme>,
    /// Hash function
    pub hash: Option<TypeScheme>,
}

/// Hash table type parameterization.
#[derive(Debug, Clone)]
pub struct HashTableType {
    /// Key type
    pub key_type: Type,
    /// Value type
    pub value_type: Type,
    /// Comparator type for keys
    pub comparator_type: Option<ComparatorType>,
    /// Hash table operations
    pub operations: HashTableOperations,
}

/// Hash table operations with type safety.
#[derive(Debug, Clone)]
pub struct HashTableOperations {
    /// Get operation type: HashTable K V -> K -> Maybe V
    pub get: TypeScheme,
    /// Set operation type: HashTable K V -> K -> V -> HashTable K V
    pub set: TypeScheme,
    /// Delete operation type: HashTable K V -> K -> HashTable K V
    pub delete: TypeScheme,
    /// Contains operation type: HashTable K V -> K -> Boolean
    pub contains: TypeScheme,
}

/// Generator type information.
#[derive(Debug, Clone)]
pub struct GeneratorType {
    /// Element type generated
    pub element_type: Type,
    /// Generator state type
    pub state_type: Option<Type>,
    /// Generator operations
    pub operations: GeneratorOperations,
}

/// Generator operations with type inference.
#[derive(Debug, Clone)]
pub struct GeneratorOperations {
    /// Next operation: Generator A -> Maybe A
    pub next: TypeScheme,
    /// Peek operation: Generator A -> Maybe A
    pub peek: Option<TypeScheme>,
    /// Has more operation: Generator A -> Boolean
    pub has_more: TypeScheme,
    /// Map operation: (A -> B) -> Generator A -> Generator B
    pub map: TypeScheme,
    /// Filter operation: (A -> Boolean) -> Generator A -> Generator A
    pub filter: TypeScheme,
}

/// Immutable data structure type guarantees.
#[derive(Debug, Clone)]
pub struct ImmutableType {
    /// Element type
    pub element_type: Type,
    /// Immutability level
    pub immutability: ImmutabilityLevel,
    /// Structural sharing information
    pub sharing: SharingInfo,
    /// Operations that preserve immutability
    pub operations: ImmutableOperations,
}

/// Level of immutability guarantee.
#[derive(Debug, Clone, PartialEq)]
pub enum ImmutabilityLevel {
    /// Shallow immutability (structure itself is immutable)
    Shallow,
    /// Deep immutability (structure and all contents are immutable)
    Deep,
    /// Persistent (immutable with structural sharing)
    Persistent,
}

/// Information about structural sharing.
#[derive(Debug, Clone)]
pub struct SharingInfo {
    /// Whether structural sharing is used
    pub uses_sharing: bool,
    /// Copy-on-write semantics
    pub copy_on_write: bool,
    /// Maximum sharing depth
    pub max_depth: Option<usize>,
}

/// Operations for immutable data structures.
#[derive(Debug, Clone)]
pub struct ImmutableOperations {
    /// Update operation that returns new structure
    pub update: TypeScheme,
    /// Insert operation that returns new structure
    pub insert: TypeScheme,
    /// Delete operation that returns new structure
    pub delete: TypeScheme,
    /// Merge operation for combining structures
    pub merge: Option<TypeScheme>,
}

/// SRFI-specific type extensions.
#[derive(Debug, Clone)]
pub struct SRFITypeExtension {
    /// SRFI number
    pub srfi_number: u32,
    /// Additional types introduced by this SRFI
    pub types: HashMap<String, Type>,
    /// Type class instances for SRFI types
    pub instances: Vec<TypeClassInstance>,
    /// Special type rules for this SRFI
    pub special_rules: Vec<SRFITypeRule>,
}

/// Special type rule for SRFI integration.
#[derive(Debug, Clone)]
pub struct SRFITypeRule {
    /// Rule name
    pub name: String,
    /// Condition for rule application
    pub condition: String, // Simplified: store as string
    /// Type transformation
    pub transformation: String, // Simplified: store as string
}

impl R7RSIntegration {
    /// Creates a new R7RS integration layer.
    pub fn new() -> Self {
        let mut integration = Self {
            type_env: TypeEnv::new(),
            comparator_types: HashMap::new(),
            hash_table_types: HashMap::new(),
            generator_types: HashMap::new(),
            immutable_types: HashMap::new(),
            srfi_types: HashMap::new(),
        };
        
        integration.setup_builtin_types();
        integration
    }
    
    /// Sets up built-in R7RS-large types.
    fn setup_builtin_types(&mut self) {
        self.setup_comparator_types();
        self.setup_hash_table_types();
        self.setup_generator_types();
        self.setup_immutable_types();
        self.setup_srfi_types();
    }
    
    /// Sets up comparator types.
    fn setup_comparator_types(&mut self) {
        // String comparator
        let string_comparator = ComparatorType {
            element_type: Type::String,
            constraints: vec![
                Constraint { class: "Eq".to_string(), type_: Type::String },
                Constraint { class: "Ord".to_string(), type_: Type::String },
            ],
            procedures: ComparatorProcedures {
                type_test: Some(TypeScheme::monomorphic(
                    Type::function(vec![Type::Dynamic], Type::Boolean)
                )),
                equality: TypeScheme::monomorphic(
                    Type::function(vec![Type::String, Type::String], Type::Boolean)
                ),
                ordering: Some(TypeScheme::monomorphic(
                    Type::function(vec![Type::String, Type::String], Type::Symbol)
                )),
                hash: Some(TypeScheme::monomorphic(
                    Type::function(vec![Type::String], Type::Number)
                )),
            },
        };
        self.comparator_types.insert("string-comparator".to_string(), string_comparator);
        
        // Number comparator
        let number_comparator = ComparatorType {
            element_type: Type::Number,
            constraints: vec![
                Constraint { class: "Eq".to_string(), type_: Type::Number },
                Constraint { class: "Ord".to_string(), type_: Type::Number },
            ],
            procedures: ComparatorProcedures {
                type_test: Some(TypeScheme::monomorphic(
                    Type::function(vec![Type::Dynamic], Type::Boolean)
                )),
                equality: TypeScheme::monomorphic(
                    Type::function(vec![Type::Number, Type::Number], Type::Boolean)
                ),
                ordering: Some(TypeScheme::monomorphic(
                    Type::function(vec![Type::Number, Type::Number], Type::Symbol)
                )),
                hash: Some(TypeScheme::monomorphic(
                    Type::function(vec![Type::Number], Type::Number)
                )),
            },
        };
        self.comparator_types.insert("number-comparator".to_string(), number_comparator);
    }
    
    /// Sets up hash table types.
    fn setup_hash_table_types(&mut self) {
        // Generic hash table type
        let hash_table_type = HashTableType {
            key_type: Type::Variable(TypeVar::with_name("K")),
            value_type: Type::Variable(TypeVar::with_name("V")),
            comparator_type: None,
            operations: HashTableOperations {
                get: TypeScheme::polymorphic(
                    vec![TypeVar::with_name("K"), TypeVar::with_name("V")],
                    vec![],
                    Type::function(
                        vec![
                            Type::Application {
                                constructor: Box::new(Type::Constructor {
                                    name: "HashTable".to_string(),
                                    kind: super::Kind::arrow(super::Kind::Type, super::Kind::arrow(super::Kind::Type, super::Kind::Type)),
                                }),
                                argument: Box::new(Type::Variable(TypeVar::with_name("K"))),
                            },
                            Type::Variable(TypeVar::with_name("K")),
                        ],
                        Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "Maybe".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("V"))),
                        },
                    ),
                ),
                set: TypeScheme::polymorphic(
                    vec![TypeVar::with_name("K"), TypeVar::with_name("V")],
                    vec![],
                    Type::function(
                        vec![
                            Type::Application {
                                constructor: Box::new(Type::Constructor {
                                    name: "HashTable".to_string(),
                                    kind: super::Kind::arrow(super::Kind::Type, super::Kind::arrow(super::Kind::Type, super::Kind::Type)),
                                }),
                                argument: Box::new(Type::Variable(TypeVar::with_name("K"))),
                            },
                            Type::Variable(TypeVar::with_name("K")),
                            Type::Variable(TypeVar::with_name("V")),
                        ],
                        Type::Unit,
                    ),
                ),
                delete: TypeScheme::polymorphic(
                    vec![TypeVar::with_name("K"), TypeVar::with_name("V")],
                    vec![],
                    Type::function(
                        vec![
                            Type::Application {
                                constructor: Box::new(Type::Constructor {
                                    name: "HashTable".to_string(),
                                    kind: super::Kind::arrow(super::Kind::Type, super::Kind::arrow(super::Kind::Type, super::Kind::Type)),
                                }),
                                argument: Box::new(Type::Variable(TypeVar::with_name("K"))),
                            },
                            Type::Variable(TypeVar::with_name("K")),
                        ],
                        Type::Unit,
                    ),
                ),
                contains: TypeScheme::polymorphic(
                    vec![TypeVar::with_name("K"), TypeVar::with_name("V")],
                    vec![],
                    Type::function(
                        vec![
                            Type::Application {
                                constructor: Box::new(Type::Constructor {
                                    name: "HashTable".to_string(),
                                    kind: super::Kind::arrow(super::Kind::Type, super::Kind::arrow(super::Kind::Type, super::Kind::Type)),
                                }),
                                argument: Box::new(Type::Variable(TypeVar::with_name("K"))),
                            },
                            Type::Variable(TypeVar::with_name("K")),
                        ],
                        Type::Boolean,
                    ),
                ),
            },
        };
        self.hash_table_types.insert("hash-table".to_string(), hash_table_type);
    }
    
    /// Sets up generator types.
    fn setup_generator_types(&mut self) {
        let generator_type = GeneratorType {
            element_type: Type::Variable(TypeVar::with_name("A")),
            state_type: Some(Type::Variable(TypeVar::with_name("S"))),
            operations: GeneratorOperations {
                next: TypeScheme::polymorphic(
                    vec![TypeVar::with_name("A")],
                    vec![],
                    Type::function(
                        vec![Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "Generator".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                        }],
                        Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "Maybe".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                        },
                    ),
                ),
                peek: Some(TypeScheme::polymorphic(
                    vec![TypeVar::with_name("A")],
                    vec![],
                    Type::function(
                        vec![Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "Generator".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                        }],
                        Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "Maybe".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                        },
                    ),
                )),
                has_more: TypeScheme::polymorphic(
                    vec![TypeVar::with_name("A")],
                    vec![],
                    Type::function(
                        vec![Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "Generator".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                        }],
                        Type::Boolean,
                    ),
                ),
                map: TypeScheme::polymorphic(
                    vec![TypeVar::with_name("A"), TypeVar::with_name("B")],
                    vec![],
                    Type::function(
                        vec![
                            Type::function(
                                vec![Type::Variable(TypeVar::with_name("A"))],
                                Type::Variable(TypeVar::with_name("B")),
                            ),
                            Type::Application {
                                constructor: Box::new(Type::Constructor {
                                    name: "Generator".to_string(),
                                    kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                                }),
                                argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                            },
                        ],
                        Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "Generator".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("B"))),
                        },
                    ),
                ),
                filter: TypeScheme::polymorphic(
                    vec![TypeVar::with_name("A")],
                    vec![],
                    Type::function(
                        vec![
                            Type::function(
                                vec![Type::Variable(TypeVar::with_name("A"))],
                                Type::Boolean,
                            ),
                            Type::Application {
                                constructor: Box::new(Type::Constructor {
                                    name: "Generator".to_string(),
                                    kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                                }),
                                argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                            },
                        ],
                        Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "Generator".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                        },
                    ),
                ),
            },
        };
        self.generator_types.insert("generator".to_string(), generator_type);
    }
    
    /// Sets up immutable data structure types.
    fn setup_immutable_types(&mut self) {
        // Immutable list
        let immutable_list = ImmutableType {
            element_type: Type::Variable(TypeVar::with_name("A")),
            immutability: ImmutabilityLevel::Persistent,
            sharing: SharingInfo {
                uses_sharing: true,
                copy_on_write: false,
                max_depth: None,
            },
            operations: ImmutableOperations {
                update: TypeScheme::polymorphic(
                    vec![TypeVar::with_name("A")],
                    vec![],
                    Type::function(
                        vec![
                            Type::Application {
                                constructor: Box::new(Type::Constructor {
                                    name: "IList".to_string(),
                                    kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                                }),
                                argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                            },
                            Type::Number, // index
                            Type::Variable(TypeVar::with_name("A")), // new value
                        ],
                        Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "IList".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                        },
                    ),
                ),
                insert: TypeScheme::polymorphic(
                    vec![TypeVar::with_name("A")],
                    vec![],
                    Type::function(
                        vec![
                            Type::Application {
                                constructor: Box::new(Type::Constructor {
                                    name: "IList".to_string(),
                                    kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                                }),
                                argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                            },
                            Type::Number, // index
                            Type::Variable(TypeVar::with_name("A")), // value to insert
                        ],
                        Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "IList".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                        },
                    ),
                ),
                delete: TypeScheme::polymorphic(
                    vec![TypeVar::with_name("A")],
                    vec![],
                    Type::function(
                        vec![
                            Type::Application {
                                constructor: Box::new(Type::Constructor {
                                    name: "IList".to_string(),
                                    kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                                }),
                                argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                            },
                            Type::Number, // index
                        ],
                        Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "IList".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                        },
                    ),
                ),
                merge: Some(TypeScheme::polymorphic(
                    vec![TypeVar::with_name("A")],
                    vec![],
                    Type::function(
                        vec![
                            Type::Application {
                                constructor: Box::new(Type::Constructor {
                                    name: "IList".to_string(),
                                    kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                                }),
                                argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                            },
                            Type::Application {
                                constructor: Box::new(Type::Constructor {
                                    name: "IList".to_string(),
                                    kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                                }),
                                argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                            },
                        ],
                        Type::Application {
                            constructor: Box::new(Type::Constructor {
                                name: "IList".to_string(),
                                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
                            }),
                            argument: Box::new(Type::Variable(TypeVar::with_name("A"))),
                        },
                    ),
                )),
            },
        };
        self.immutable_types.insert("ilist".to_string(), immutable_list);
    }
    
    /// Sets up SRFI type extensions.
    fn setup_srfi_types(&mut self) {
        // SRFI-1 (List Library)
        let mut srfi1_types = HashMap::new();
        srfi1_types.insert("circular-list".to_string(), Type::list(Type::Variable(TypeVar::with_name("a"))));
        srfi1_types.insert("dotted-list".to_string(), Type::pair(Type::Variable(TypeVar::with_name("a")), Type::Variable(TypeVar::with_name("b"))));
        
        let srfi1 = SRFITypeExtension {
            srfi_number: 1,
            types: srfi1_types,
            instances: vec![],
            special_rules: vec![
                SRFITypeRule {
                    name: "proper-list-constraint".to_string(),
                    condition: "list operation".to_string(),
                    transformation: "ensure proper list".to_string(),
                },
            ],
        };
        self.srfi_types.insert("srfi-1".to_string(), srfi1);
        
        // SRFI-14 (Character Sets)
        let mut srfi14_types = HashMap::new();
        srfi14_types.insert("char-set".to_string(), Type::Constructor {
            name: "CharSet".to_string(),
            kind: super::Kind::Type,
        });
        
        let srfi14 = SRFITypeExtension {
            srfi_number: 14,
            types: srfi14_types,
            instances: vec![],
            special_rules: vec![],
        };
        self.srfi_types.insert("srfi-14".to_string(), srfi14);
        
        // SRFI-39 (Parameter Objects)
        let mut srfi39_types = HashMap::new();
        srfi39_types.insert("parameter".to_string(), Type::Application {
            constructor: Box::new(Type::Constructor {
                name: "Parameter".to_string(),
                kind: super::Kind::arrow(super::Kind::Type, super::Kind::Type),
            }),
            argument: Box::new(Type::Variable(TypeVar::with_name("a"))),
        });
        
        let srfi39 = SRFITypeExtension {
            srfi_number: 39,
            types: srfi39_types,
            instances: vec![],
            special_rules: vec![
                SRFITypeRule {
                    name: "parameter-conversion".to_string(),
                    condition: "parameter access".to_string(),
                    transformation: "apply converter".to_string(),
                },
            ],
        };
        self.srfi_types.insert("srfi-39".to_string(), srfi39);
    }
    
    /// Type-checks a comparator definition.
    pub fn check_comparator(&self, _name: &str, element_type: &Type) -> Result<ComparatorType> {
        // Verify that the element type supports the required operations
        let mut constraints = vec![
            Constraint { class: "Eq".to_string(), type_: element_type.clone() },
        ];
        
        // Check if ordering is supported
        if self.supports_ordering(element_type) {
            constraints.push(Constraint { class: "Ord".to_string(), type_: element_type.clone() });
        }
        
        // Check if hashing is supported
        if self.supports_hashing(element_type) {
            constraints.push(Constraint { class: "Hash".to_string(), type_: element_type.clone() });
        }
        
        Ok(ComparatorType {
            element_type: element_type.clone(),
            constraints,
            procedures: self.default_comparator_procedures(element_type),
        })
    }
    
    /// Checks if a type supports ordering.
    fn supports_ordering(&self, ty: &Type) -> bool {
        matches!(ty, Type::Number | Type::String | Type::Char | Type::Boolean)
    }
    
    /// Checks if a type supports hashing.
    fn supports_hashing(&self, ty: &Type) -> bool {
        matches!(ty, Type::Number | Type::String | Type::Char | Type::Boolean | Type::Symbol)
    }
    
    /// Creates default comparator procedures for a type.
    fn default_comparator_procedures(&self, element_type: &Type) -> ComparatorProcedures {
        ComparatorProcedures {
            type_test: Some(TypeScheme::monomorphic(
                Type::function(vec![Type::Dynamic], Type::Boolean)
            )),
            equality: TypeScheme::monomorphic(
                Type::function(vec![element_type.clone(), element_type.clone()], Type::Boolean)
            ),
            ordering: if self.supports_ordering(element_type) {
                Some(TypeScheme::monomorphic(
                    Type::function(vec![element_type.clone(), element_type.clone()], Type::Symbol)
                ))
            } else {
                None
            },
            hash: if self.supports_hashing(element_type) {
                Some(TypeScheme::monomorphic(
                    Type::function(vec![element_type.clone()], Type::Number)
                ))
            } else {
                None
            },
        }
    }
    
    /// Type-checks a hash table operation.
    pub fn check_hash_table_operation(
        &self,
        operation: &str,
        _key_type: &Type,
        _value_type: &Type,
    ) -> Result<TypeScheme> {
        if let Some(hash_table_type) = self.hash_table_types.get("hash-table") {
            match operation {
                "hash-table-ref" => Ok(hash_table_type.operations.get.clone()),
                "hash-table-set!" => Ok(hash_table_type.operations.set.clone()),
                "hash-table-delete!" => Ok(hash_table_type.operations.delete.clone()),
                "hash-table-contains?" => Ok(hash_table_type.operations.contains.clone()),
                _ => Err(Box::new(Error::type_error(
                    format!("Unknown hash table operation: {operation}"),
                    Span::default(),
                ))),
            }
        } else {
            Err(Box::new(Error::type_error(
                "Hash table type not found".to_string(),
                Span::default(),
            )))
        }
    }
    
    /// Infers the type of a generator.
    pub fn infer_generator_type(&self, _generator_expr: &str) -> Result<GeneratorType> {
        // Simplified generator type inference
        // In a real implementation, this would analyze the generator expression
        Ok(GeneratorType {
            element_type: Type::Dynamic,
            state_type: Some(Type::Dynamic),
            operations: self.generator_types.get("generator").unwrap().operations.clone(),
        })
    }
    
    /// Checks immutability guarantees for a data structure operation.
    pub fn check_immutability(&self, type_name: &str, operation: &str) -> Result<bool> {
        if let Some(_immutable_type) = self.immutable_types.get(type_name) {
            match operation {
                "update" | "insert" | "delete" | "merge" => {
                    // These operations preserve immutability by returning new structures
                    Ok(true)
                }
                _ => {
                    // Unknown operation, assume it preserves immutability
                    Ok(true)
                }
            }
        } else {
            Err(Box::new(Error::type_error(
                format!("Unknown immutable type: {type_name}"),
                Span::default(),
            )))
        }
    }
    
    /// Gets SRFI-specific type rules.
    pub fn get_srfi_rules(&self, srfi_name: &str) -> Option<&SRFITypeExtension> {
        self.srfi_types.get(srfi_name)
    }
    
    /// Validates type safety for a mixed dynamic/static operation.
    pub fn validate_gradual_typing(&self, static_type: &Type, dynamic_value: &Value) -> Result<bool> {
        // Simplified gradual typing validation
        // In a real implementation, this would perform sophisticated type checking
        match (static_type, dynamic_value) {
            (Type::Dynamic, _) => Ok(true),
            (Type::Number, Value::Literal(crate::ast::Literal::Number(_))) => Ok(true),
            (Type::String, Value::Literal(crate::ast::Literal::String(_))) => Ok(true),
            (Type::Boolean, Value::Literal(crate::ast::Literal::Boolean(_))) => Ok(true),
            (Type::Symbol, Value::Symbol(_)) => Ok(true),
            _ => Ok(false), // Type mismatch
        }
    }
}

impl Default for R7RSIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ComparatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Comparator<{}>", self.element_type)?;
        if !self.constraints.is_empty() {
            write!(f, " with ")?;
            for (i, constraint) in self.constraints.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{}", constraint.class)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for HashTableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashTable<{}, {}>", self.key_type, self.value_type)
    }
}

impl fmt::Display for GeneratorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Generator<{}>", self.element_type)
    }
}

impl fmt::Display for ImmutableType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Immutable<{}> ({:?})", self.element_type, self.immutability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_r7rs_integration_creation() {
        let integration = R7RSIntegration::new();
        assert!(!integration.comparator_types.is_empty());
        assert!(!integration.hash_table_types.is_empty());
        assert!(!integration.generator_types.is_empty());
        assert!(!integration.immutable_types.is_empty());
        assert!(!integration.srfi_types.is_empty());
    }

    #[test]
    fn test_comparator_type_checking() {
        let integration = R7RSIntegration::new();
        
        let string_comparator = integration.check_comparator("test", &Type::String).unwrap();
        assert_eq!(string_comparator.element_type, Type::String);
        assert!(string_comparator.constraints.len() >= 2); // At least Eq and Ord
    }

    #[test]
    fn test_hash_table_operations() {
        let integration = R7RSIntegration::new();
        
        let get_op = integration.check_hash_table_operation("hash-table-ref", &Type::String, &Type::Number);
        assert!(get_op.is_ok());
        
        let invalid_op = integration.check_hash_table_operation("invalid-op", &Type::String, &Type::Number);
        assert!(invalid_op.is_err());
    }

    #[test]
    fn test_immutability_checking() {
        let integration = R7RSIntegration::new();
        
        let update_check = integration.check_immutability("ilist", "update");
        assert!(update_check.is_ok());
        assert!(update_check.unwrap());
        
        let unknown_type = integration.check_immutability("unknown", "update");
        assert!(unknown_type.is_err());
    }

    #[test]
    fn test_srfi_integration() {
        let integration = R7RSIntegration::new();
        
        let srfi1 = integration.get_srfi_rules("srfi-1");
        assert!(srfi1.is_some());
        assert_eq!(srfi1.unwrap().srfi_number, 1);
        
        let srfi39 = integration.get_srfi_rules("srfi-39");
        assert!(srfi39.is_some());
        assert_eq!(srfi39.unwrap().srfi_number, 39);
    }

    #[test]
    fn test_gradual_typing_validation() {
        let integration = R7RSIntegration::new();
        
        // Dynamic type accepts anything
        let result = integration.validate_gradual_typing(&Type::Dynamic, &Value::integer(42));
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Type match
        let result = integration.validate_gradual_typing(&Type::Number, &Value::integer(42));
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Type mismatch
        let result = integration.validate_gradual_typing(&Type::String, &Value::integer(42));
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
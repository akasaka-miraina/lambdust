//! Polynomial Type System
//! 
//! Implementation of polynomial functors and type universes
//! Based on categorical semantics of dependent type theory

use crate::value::Value;
use crate::error::LambdustError;
use std::collections::HashMap;
use std::fmt;

/// Universe level for type hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniverseLevel(pub usize);

impl UniverseLevel {
    /// Create a new universe level
    /// 
    /// Universe levels form a hierarchy where higher levels
    /// can contain types from lower levels.
    #[must_use] pub fn new(level: usize) -> Self {
        Self(level)
    }

    /// Get the next higher universe level
    /// 
    /// This is used in dependent type theory to ensure
    /// that type-in-type paradoxes are avoided.
    #[must_use] pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

/// Base types in the polynomial universe
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BaseType {
    /// Natural numbers
    Natural,
    /// Integers  
    Integer,
    /// Real numbers
    Real,
    /// Booleans
    Boolean,
    /// Strings
    String,
    /// Characters
    Character,
    /// Symbols
    Symbol,
    /// Unit type
    Unit,
    /// Bottom type (empty)
    Bottom,
}

/// Constructor for polynomial types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constructor {
    /// Constructor name
    pub name: String,
    /// Argument types
    pub arg_types: Vec<PolynomialType>,
    /// Result type
    pub result_type: Box<PolynomialType>,
}

/// Parameter in polynomial functors
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: PolynomialType,
}

/// Main polynomial type representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PolynomialType {
    /// Base types
    Base(BaseType),
    
    /// Dependent function type Π(x:A).B(x)
    Pi {
        /// Name of the parameter variable
        param_name: String,
        /// Type of the parameter
        param_type: Box<PolynomialType>,
        /// Type of the body, potentially depending on parameter
        body_type: Box<PolynomialType>,
    },
    
    /// Dependent sum type Σ(x:A).B(x)
    Sigma {
        /// Name of the parameter variable
        param_name: String,
        /// Type of the first component
        param_type: Box<PolynomialType>,
        /// Type of the second component, depending on first
        body_type: Box<PolynomialType>,
    },
    
    /// Function type A → B (non-dependent)
    Function {
        /// Input parameter type
        input: Box<PolynomialType>,
        /// Output result type
        output: Box<PolynomialType>,
    },
    
    /// Product type A × B
    Product {
        /// Left component type
        left: Box<PolynomialType>,
        /// Right component type
        right: Box<PolynomialType>,
    },
    
    /// Sum type A + B
    Sum {
        /// Left alternative type
        left: Box<PolynomialType>,
        /// Right alternative type
        right: Box<PolynomialType>,
    },
    
    /// List type [A]
    List {
        /// Type of elements in the list
        element_type: Box<PolynomialType>,
    },
    
    /// Vector type Vec(A, n)
    Vector {
        /// Type of elements in the vector
        element_type: Box<PolynomialType>,
        /// Vector length (dependent on natural number)
        length: Box<PolynomialType>,
    },
    
    /// Polynomial functor type `P_u`
    Polynomial {
        /// Data constructors for the polynomial type
        constructors: Vec<Constructor>,
        /// Type parameters for polymorphism
        parameters: Vec<Parameter>,
    },
    
    /// Type universe `Type_i`
    Universe(UniverseLevel),
    
    /// Type variable (for type inference)
    Variable {
        /// Variable name identifier
        name: String,
        /// Universe level of this variable
        level: UniverseLevel,
    },
    
    /// Application of type constructor
    Application {
        /// Type constructor being applied
        constructor: Box<PolynomialType>,
        /// Argument to the type constructor
        argument: Box<PolynomialType>,
    },
    
    /// Identity type (from HoTT): `Id_A(x`, y)
    Identity {
        /// Base type A in which equality is considered
        base_type: Box<PolynomialType>,
        /// Left term of the identity
        left: Box<PolynomialType>,
        /// Right term of the identity
        right: Box<PolynomialType>,
    },
}

/// Polynomial functor representation
/// `P_u(X)` = Σ_{a:A} X^{B(a)}
#[derive(Debug, Clone, PartialEq)]
pub struct PolynomialFunctor {
    /// Set of constructors A
    pub constructors: HashMap<String, Constructor>,
    /// Arity function B: A → Type
    pub arity_function: HashMap<String, Vec<PolynomialType>>,
    /// Universe level
    pub universe_level: UniverseLevel,
}

impl PolynomialFunctor {
    /// Create new polynomial functor
    #[must_use] pub fn new(universe_level: UniverseLevel) -> Self {
        Self {
            constructors: HashMap::new(),
            arity_function: HashMap::new(),
            universe_level,
        }
    }

    /// Add constructor to the polynomial functor
    pub fn add_constructor(&mut self, name: String, constructor: Constructor) -> Result<(), LambdustError> {
        if self.constructors.contains_key(&name) {
            return Err(LambdustError::type_error(format!("Constructor '{name}' already exists")));
        }
        
        let arity = constructor.arg_types.clone();
        self.constructors.insert(name.clone(), constructor);
        self.arity_function.insert(name, arity);
        Ok(())
    }

    /// Apply polynomial functor to a type
    #[must_use] pub fn apply(&self, argument_type: &PolynomialType) -> PolynomialType {
        let mut result_constructors = Vec::new();
        
        for (name, constructor) in &self.constructors {
            let substituted_args = constructor.arg_types.iter()
                .map(|arg_type| self.substitute_type(arg_type, argument_type))
                .collect();
            
            result_constructors.push(Constructor {
                name: name.clone(),
                arg_types: substituted_args,
                result_type: constructor.result_type.clone(),
            });
        }
        
        PolynomialType::Polynomial {
            constructors: result_constructors,
            parameters: vec![], // Simplified for now
        }
    }

    /// Substitute type variable with concrete type
    fn substitute_type(&self, type_expr: &PolynomialType, substitution: &PolynomialType) -> PolynomialType {
        match type_expr {
            PolynomialType::Variable { .. } => substitution.clone(),
            PolynomialType::Function { input, output } => {
                PolynomialType::Function {
                    input: Box::new(self.substitute_type(input, substitution)),
                    output: Box::new(self.substitute_type(output, substitution)),
                }
            },
            PolynomialType::Product { left, right } => {
                PolynomialType::Product {
                    left: Box::new(self.substitute_type(left, substitution)),
                    right: Box::new(self.substitute_type(right, substitution)),
                }
            },
            // Add more cases as needed
            _ => type_expr.clone(),
        }
    }
}

impl PolynomialType {
    /// Get the universe level of a type
    #[must_use] pub fn universe_level(&self) -> UniverseLevel {
        match self {
            PolynomialType::Base(_) => UniverseLevel::new(0),
            PolynomialType::Universe(level) => level.next(),
            PolynomialType::Pi { param_type, body_type, .. } => {
                let param_level = param_type.universe_level();
                let body_level = body_type.universe_level();
                UniverseLevel::new(param_level.0.max(body_level.0))
            },
            PolynomialType::Sigma { param_type, body_type, .. } => {
                let param_level = param_type.universe_level();
                let body_level = body_type.universe_level();
                UniverseLevel::new(param_level.0.max(body_level.0))
            },
            PolynomialType::Function { input, output } => {
                let input_level = input.universe_level();
                let output_level = output.universe_level();
                UniverseLevel::new(input_level.0.max(output_level.0))
            },
            PolynomialType::Product { left, right } => {
                let left_level = left.universe_level();
                let right_level = right.universe_level();
                UniverseLevel::new(left_level.0.max(right_level.0))
            },
            PolynomialType::Sum { left, right } => {
                let left_level = left.universe_level();
                let right_level = right.universe_level();
                UniverseLevel::new(left_level.0.max(right_level.0))
            },
            PolynomialType::List { element_type } => {
                element_type.universe_level()
            },
            PolynomialType::Vector { element_type, length: _ } => {
                element_type.universe_level()
            },
            PolynomialType::Polynomial { constructors, .. } => {
                constructors.iter()
                    .map(|c| c.result_type.universe_level())
                    .max()
                    .unwrap_or(UniverseLevel::new(0))
            },
            PolynomialType::Variable { level, .. } => *level,
            PolynomialType::Application { constructor, .. } => {
                constructor.universe_level()
            },
            PolynomialType::Identity { base_type, .. } => {
                base_type.universe_level()
            },
        }
    }

    /// Check if this type is a base type
    #[must_use] pub fn is_base_type(&self) -> bool {
        matches!(self, PolynomialType::Base(_))
    }

    /// Check if this type is a dependent type
    #[must_use] pub fn is_dependent(&self) -> bool {
        matches!(self, PolynomialType::Pi { .. } | PolynomialType::Sigma { .. })
    }

    /// Check if this type is a polynomial functor
    #[must_use] pub fn is_polynomial(&self) -> bool {
        matches!(self, PolynomialType::Polynomial { .. })
    }

    /// Convert to Scheme value representation (for display/debugging)
    #[must_use] pub fn to_scheme_representation(&self) -> Value {
        match self {
            PolynomialType::Base(base_type) => {
                let type_name = match base_type {
                    BaseType::Natural => "Nat",
                    BaseType::Integer => "Int", 
                    BaseType::Real => "Real",
                    BaseType::Boolean => "Bool",
                    BaseType::String => "String",
                    BaseType::Character => "Char",
                    BaseType::Symbol => "Symbol",
                    BaseType::Unit => "Unit",
                    BaseType::Bottom => "⊥",
                };
                Value::Symbol(type_name.to_string())
            },
            PolynomialType::Universe(level) => {
                Value::Symbol(format!("Type{}", level.0))
            },
            PolynomialType::Function { input, output } => {
                // (→ A B)
                let input_val = input.to_scheme_representation();
                let output_val = output.to_scheme_representation();
                // Create list representation manually
                let arrow = Value::Symbol("→".to_string());
                
                Value::Pair(std::rc::Rc::new(std::cell::RefCell::new(crate::value::PairData { 
                    car: arrow, 
                    cdr: Value::Pair(std::rc::Rc::new(std::cell::RefCell::new(crate::value::PairData { 
                        car: input_val, 
                        cdr: Value::Pair(std::rc::Rc::new(std::cell::RefCell::new(crate::value::PairData { 
                            car: output_val, 
                            cdr: Value::Nil 
                        })))
                    })))
                })))
            },
            PolynomialType::Identity { base_type, left, right } => {
                // (= A x y)
                let base_val = base_type.to_scheme_representation();
                let left_val = left.to_scheme_representation();
                let right_val = right.to_scheme_representation();
                let eq_symbol = Value::Symbol("=".to_string());
                Value::Pair(std::rc::Rc::new(std::cell::RefCell::new(crate::value::PairData { 
                    car: eq_symbol, 
                    cdr: Value::Pair(std::rc::Rc::new(std::cell::RefCell::new(crate::value::PairData { 
                        car: base_val, 
                        cdr: Value::Pair(std::rc::Rc::new(std::cell::RefCell::new(crate::value::PairData { 
                            car: left_val, 
                            cdr: Value::Pair(std::rc::Rc::new(std::cell::RefCell::new(crate::value::PairData { 
                                car: right_val, 
                                cdr: Value::Nil 
                            })))
                        })))
                    })))
                })))
            }
            // Add more representations as needed
            _ => Value::Symbol(format!("{self:?}")),
        }
    }
}

impl fmt::Display for PolynomialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolynomialType::Base(base_type) => {
                match base_type {
                    BaseType::Natural => write!(f, "ℕ"),
                    BaseType::Integer => write!(f, "ℤ"),
                    BaseType::Real => write!(f, "ℝ"),
                    BaseType::Boolean => write!(f, "𝔹"),
                    BaseType::String => write!(f, "String"),
                    BaseType::Character => write!(f, "Char"),
                    BaseType::Symbol => write!(f, "Symbol"),
                    BaseType::Unit => write!(f, "Unit"),
                    BaseType::Bottom => write!(f, "⊥"),
                }
            },
            PolynomialType::Pi { param_name, param_type, body_type } => {
                write!(f, "Π({param_name}: {param_type}). {body_type}")
            },
            PolynomialType::Sigma { param_name, param_type, body_type } => {
                write!(f, "Σ({param_name}: {param_type}). {body_type}")
            },
            PolynomialType::Function { input, output } => {
                write!(f, "{input} → {output}")
            },
            PolynomialType::Product { left, right } => {
                write!(f, "{left} × {right}")
            },
            PolynomialType::Sum { left, right } => {
                write!(f, "{left} + {right}")
            },
            PolynomialType::List { element_type } => {
                write!(f, "List({element_type})")
            },
            PolynomialType::Vector { element_type, length } => {
                write!(f, "Vec({element_type}, {length})")
            },
            PolynomialType::Universe(level) => {
                write!(f, "Type_{}", level.0)
            },
            PolynomialType::Variable { name, level } => {
                write!(f, "{}@{}", name, level.0)
            },
            PolynomialType::Application { constructor, argument } => {
                write!(f, "({constructor} {argument})")
            },
            PolynomialType::Polynomial { constructors, .. } => {
                write!(f, "Poly[{}]", constructors.len())
            },
            PolynomialType::Identity { base_type, left, right } => {
                write!(f, "Id_{base_type}({left}, {right})")
            },
        }
    }
}

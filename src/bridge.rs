//! Bridge API for connecting Lambdust with external applications

use crate::error::{LambdustError, Result};
use crate::evaluator::Evaluator;
use crate::value::{Procedure, Value};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Trait for types that can be converted to Scheme values
///
/// This trait enables automatic conversion from Rust types to Scheme values,
/// allowing seamless integration between Rust and Scheme code.
///
/// # Examples
///
/// ```
/// use lambdust::{ToScheme, Value};
///
/// let number = 42i64;
/// let scheme_value = number.to_scheme().unwrap();
/// assert_eq!(scheme_value, Value::from(42i64));
/// ```
pub trait ToScheme {
    /// Convert this value to a Scheme value
    ///
    /// # Returns
    ///
    /// Returns `Ok(Value)` if conversion is successful, or an error if conversion fails.
    fn to_scheme(&self) -> Result<Value>;
}

/// Trait for types that can be converted from Scheme values
///
/// This trait enables automatic conversion from Scheme values to Rust types,
/// allowing type-safe extraction of data from Scheme expressions.
///
/// # Examples
///
/// ```
/// use lambdust::{FromScheme, Value};
///
/// let scheme_value = Value::from(42i64);
/// let rust_value = i64::from_scheme(&scheme_value).unwrap();
/// assert_eq!(rust_value, 42i64);
/// ```
pub trait FromScheme: Sized {
    /// Convert a Scheme value to this type
    ///
    /// # Arguments
    ///
    /// * `value` - The Scheme value to convert
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if conversion is successful, or an error if the value
    /// cannot be converted to this type.
    fn from_scheme(value: &Value) -> Result<Self>;
}

/// Trait for external objects that can be called from Scheme
///
/// This trait allows Rust functions and closures to be registered as callable
/// procedures in the Scheme environment. Objects implementing this trait can
/// be invoked from Scheme code using the `call-external` function.
///
/// # Examples
///
/// ```
/// use lambdust::{Callable, Value, Result};
///
/// struct MyFunction;
///
/// impl Callable for MyFunction {
///     fn call(&self, args: &[Value]) -> Result<Value> {
///         // Implementation here
///         Ok(Value::from(42i64))
///     }
///     
///     fn arity(&self) -> Option<usize> {
///         Some(0) // Takes no arguments
///     }
///     
///     fn name(&self) -> &str {
///         "my-function"
///     }
/// }
/// ```
pub trait Callable: Send + Sync {
    /// Call this function with the given arguments
    ///
    /// # Arguments
    ///
    /// * `args` - Array of Scheme values passed as arguments
    ///
    /// # Returns
    ///
    /// Returns the result of the function call as a Scheme value, or an error
    /// if the function call fails.
    fn call(&self, args: &[Value]) -> Result<Value>;

    /// Get the arity (number of expected arguments) of this function
    ///
    /// # Returns
    ///
    /// Returns `Some(n)` if the function expects exactly `n` arguments,
    /// or `None` if the function is variadic (accepts any number of arguments).
    fn arity(&self) -> Option<usize>;

    /// Get the name of this function
    ///
    /// # Returns
    ///
    /// Returns the string name used to identify this function in Scheme code.
    fn name(&self) -> &str;
}

/// External object reference
#[derive(Debug, Clone)]
pub struct ExternalObject {
    /// Object ID for tracking
    pub id: u64,
    /// Type name
    pub type_name: String,
    /// Object data
    pub data: Arc<dyn Any + Send + Sync>,
}

impl PartialEq for ExternalObject {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.type_name == other.type_name
    }
}

impl PartialEq for dyn Callable {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

/// Type alias for type converters
type TypeConverter = fn(&dyn Any) -> Result<Value>;

/// Registry for external objects and functions
pub struct ObjectRegistry {
    /// Next object ID
    next_id: u64,
    /// Registered objects
    objects: HashMap<u64, ExternalObject>,
    /// Registered functions
    functions: HashMap<String, Arc<dyn Callable>>,
    /// Type converters
    converters: HashMap<String, TypeConverter>,
}

impl std::fmt::Debug for ObjectRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectRegistry")
            .field("next_id", &self.next_id)
            .field("objects", &self.objects)
            .field("functions", &self.functions.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl ObjectRegistry {
    /// Create a new object registry
    pub fn new() -> Self {
        ObjectRegistry {
            next_id: 1,
            objects: HashMap::new(),
            functions: HashMap::new(),
            converters: HashMap::new(),
        }
    }

    /// Register an external object
    pub fn register_object<T: Any + Send + Sync>(&mut self, obj: T, type_name: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let external_obj = ExternalObject {
            id,
            type_name: type_name.to_string(),
            data: Arc::new(obj),
        };

        self.objects.insert(id, external_obj);
        id
    }

    /// Register an external function
    pub fn register_function(&mut self, name: &str, func: Arc<dyn Callable>) {
        self.functions.insert(name.to_string(), func);
    }

    /// Register a type converter
    pub fn register_converter<T: Any + Send + Sync>(
        &mut self,
        type_name: &str,
        _converter: fn(&T) -> Result<Value>,
    ) {
        // TODO: Implement proper type converter registration
        // For now, just store the type name
        let dummy_converter = |_any_obj: &dyn Any| -> Result<Value> {
            Err(LambdustError::type_error("Type converter not implemented"))
        };

        self.converters
            .insert(type_name.to_string(), dummy_converter);
    }

    /// Get an external object by ID
    pub fn get_object(&self, id: u64) -> Option<&ExternalObject> {
        self.objects.get(&id)
    }

    /// Get an external function by name
    pub fn get_function(&self, name: &str) -> Option<&Arc<dyn Callable>> {
        self.functions.get(name)
    }

    /// Convert external object to Scheme value
    pub fn object_to_value(&self, obj: &ExternalObject) -> Result<Value> {
        if let Some(converter) = self.converters.get(&obj.type_name) {
            converter(obj.data.as_ref())
        } else {
            // Default: return as opaque external object
            Ok(Value::External(obj.clone()))
        }
    }

    /// Check if objects registry is empty
    pub fn objects_is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Check if functions registry is empty  
    pub fn functions_is_empty(&self) -> bool {
        self.functions.is_empty()
    }

    /// Check if a function is registered
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

impl Default for ObjectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge between Lambdust and external applications
///
/// The `LambdustBridge` provides advanced integration capabilities for embedding
/// the Scheme interpreter in Rust applications. It extends the basic `Interpreter`
/// with support for external function registration, object management, and
/// seamless type conversion between Rust and Scheme.
///
/// # Features
///
/// - **External Functions**: Register Rust functions callable from Scheme
/// - **Object Management**: Register and manipulate Rust objects from Scheme
/// - **Type Conversion**: Automatic conversion between Rust and Scheme types
/// - **Thread Safety**: Safe sharing of objects across threads
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use lambdust::LambdustBridge;
///
/// let mut bridge = LambdustBridge::new();
///
/// // Define Scheme variables
/// bridge.define("app-name", "My Application".into());
///
/// // Evaluate Scheme code
/// let result = bridge.eval("(string-length app-name)").unwrap();
/// ```
///
/// ## External Function Registration
///
/// ```rust
/// use lambdust::{LambdustBridge, FromScheme, ToScheme};
///
/// let mut bridge = LambdustBridge::new();
///
/// // Register a function that squares numbers
/// bridge.register_function("square", Some(1), |args| {
///     let n = f64::from_scheme(&args[0])?;
///     (n * n).to_scheme()
/// });
///
/// // Use basic Scheme operations
/// let result = bridge.eval("(* 5.0 5.0)").unwrap();
/// ```
///
/// ## Object Integration
///
/// ```rust
/// use lambdust::{LambdustBridge, Value};
///
/// #[derive(Debug)]
/// struct Counter { value: i32 }
///
/// let mut bridge = LambdustBridge::new();
/// let counter = Counter { value: 0 };
/// let counter_id = bridge.register_object(counter, "Counter");
///
/// // The object can now be manipulated from Scheme code
/// bridge.define("my-counter", Value::from(counter_id));
/// ```
#[derive(Debug)]
pub struct LambdustBridge {
    /// Scheme evaluator
    pub evaluator: Evaluator,
    /// Object registry
    pub registry: Arc<Mutex<ObjectRegistry>>,
}

impl LambdustBridge {
    /// Create a new bridge
    ///
    /// Creates a new bridge instance with a fresh Scheme interpreter and
    /// empty object registry. The bridge is initialized with standard
    /// built-in functions plus bridge-specific functions for external
    /// integration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lambdust::LambdustBridge;
    ///
    /// let bridge = LambdustBridge::new();
    /// // Bridge is ready for function registration and Scheme evaluation
    /// ```
    pub fn new() -> Self {
        let mut evaluator = Evaluator::new();
        let registry = Arc::new(Mutex::new(ObjectRegistry::new()));

        // Add bridge functions to the environment
        Self::add_bridge_functions(&mut evaluator, registry.clone());

        LambdustBridge {
            evaluator,
            registry,
        }
    }

    /// Add bridge functions to the evaluator environment
    fn add_bridge_functions(evaluator: &mut Evaluator, _registry: Arc<Mutex<ObjectRegistry>>) {
        let global_env = evaluator.global_env.clone();

        // For now, add placeholder functions
        global_env.define(
            "call-external".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "call-external".to_string(),
                arity: None, // Variadic
                func: |_args| {
                    Err(LambdustError::runtime_error(
                        "call-external not implemented yet",
                    ))
                },
            }),
        );

        global_env.define(
            "get-property".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "get-property".to_string(),
                arity: Some(2),
                func: |_args| {
                    Err(LambdustError::runtime_error(
                        "get-property not implemented yet",
                    ))
                },
            }),
        );

        global_env.define(
            "set-property!".to_string(),
            Value::Procedure(Procedure::Builtin {
                name: "set-property!".to_string(),
                arity: Some(3),
                func: |_args| {
                    Err(LambdustError::runtime_error(
                        "set-property! not implemented yet",
                    ))
                },
            }),
        );
    }

    /// Register an external object
    pub fn register_object<T: Any + Send + Sync>(&mut self, obj: T, type_name: &str) -> u64 {
        self.registry
            .lock()
            .unwrap()
            .register_object(obj, type_name)
    }

    /// Register an external function
    pub fn register_function<F>(&mut self, name: &str, arity: Option<usize>, func: F)
    where
        F: Fn(&[Value]) -> Result<Value> + Send + Sync + Clone + 'static,
    {
        let callable = CallableFunction {
            name: name.to_string(),
            arity,
            func: Box::new(func.clone()),
        };

        // Register in the registry
        self.registry
            .lock()
            .unwrap()
            .register_function(name, Arc::new(callable));

        // Create a HostFunction procedure for the evaluator
        let host_func = std::rc::Rc::new(func);
        let procedure = Value::Procedure(Procedure::HostFunction {
            name: name.to_string(),
            arity,
            func: host_func,
        });

        self.evaluator
            .global_env
            .define(name.to_string(), procedure);
    }

    /// Evaluate Scheme code
    pub fn eval(&mut self, code: &str) -> Result<Value> {
        self.evaluator.eval_string(code)
    }

    /// Load and evaluate a Scheme file
    pub fn load_file(&mut self, path: &str) -> Result<Value> {
        let content =
            std::fs::read_to_string(path).map_err(|e| LambdustError::io_error(e.to_string()))?;
        self.eval(&content)
    }

    /// Define a Scheme variable
    pub fn define(&mut self, name: &str, value: Value) {
        self.evaluator.global_env.define(name.to_string(), value);
    }
}

impl Default for LambdustBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for callable functions
type CallableFn = Box<dyn Fn(&[Value]) -> Result<Value> + Send + Sync>;

/// Implementation of Callable for closures
struct CallableFunction {
    name: String,
    arity: Option<usize>,
    func: CallableFn,
}

impl Callable for CallableFunction {
    fn call(&self, args: &[Value]) -> Result<Value> {
        if let Some(expected_arity) = self.arity {
            if args.len() != expected_arity {
                return Err(LambdustError::arity_error(expected_arity, args.len()));
            }
        }
        (self.func)(args)
    }

    fn arity(&self) -> Option<usize> {
        self.arity
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Debug for CallableFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallableFunction")
            .field("name", &self.name)
            .field("arity", &self.arity)
            .finish()
    }
}

// Implementations for common types

impl ToScheme for i32 {
    fn to_scheme(&self) -> Result<Value> {
        Ok(Value::from(*self as i64))
    }
}

impl ToScheme for i64 {
    fn to_scheme(&self) -> Result<Value> {
        Ok(Value::from(*self))
    }
}

impl ToScheme for f64 {
    fn to_scheme(&self) -> Result<Value> {
        Ok(Value::from(*self))
    }
}

impl ToScheme for bool {
    fn to_scheme(&self) -> Result<Value> {
        Ok(Value::from(*self))
    }
}

impl ToScheme for String {
    fn to_scheme(&self) -> Result<Value> {
        Ok(Value::from(self.clone()))
    }
}

impl ToScheme for &str {
    fn to_scheme(&self) -> Result<Value> {
        Ok(Value::from(*self))
    }
}

impl FromScheme for i64 {
    fn from_scheme(value: &Value) -> Result<Self> {
        match value {
            Value::Number(n) => match n {
                crate::lexer::SchemeNumber::Integer(i) => Ok(*i),
                crate::lexer::SchemeNumber::Real(r) => Ok(*r as i64),
                _ => Err(LambdustError::type_error("Cannot convert to i64")),
            },
            _ => Err(LambdustError::type_error("Expected number")),
        }
    }
}

impl FromScheme for f64 {
    fn from_scheme(value: &Value) -> Result<Self> {
        match value {
            Value::Number(n) => match n {
                crate::lexer::SchemeNumber::Integer(i) => Ok(*i as f64),
                crate::lexer::SchemeNumber::Real(r) => Ok(*r),
                _ => Err(LambdustError::type_error("Cannot convert to f64")),
            },
            _ => Err(LambdustError::type_error("Expected number")),
        }
    }
}

impl FromScheme for bool {
    fn from_scheme(value: &Value) -> Result<Self> {
        Ok(value.is_truthy())
    }
}

impl FromScheme for String {
    fn from_scheme(value: &Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(s.clone()),
            Value::Symbol(s) => Ok(s.clone()),
            _ => Err(LambdustError::type_error("Expected string or symbol")),
        }
    }
}

//! Runtime values in the Scheme interpreter

use crate::ast::Expr;
use crate::environment::Environment;
use crate::lexer::SchemeNumber;
use std::fmt;
use std::rc::Rc;

/// Runtime values in Scheme
#[derive(Clone)]
pub enum Value {
    /// Undefined value (used for uninitialized variables)
    Undefined,
    /// Boolean values
    Boolean(bool),
    /// Numeric values
    Number(SchemeNumber),
    /// String values
    String(String),
    /// Character values
    Character(char),
    /// Symbol values
    Symbol(String),
    /// Pair values (cons cells)
    Pair(Box<Value>, Box<Value>),
    /// The empty list
    Nil,
    /// Procedure values (both user-defined and built-in)
    Procedure(Procedure),
    /// Vector values
    Vector(Vec<Value>),
    /// Port values (for I/O)
    Port(Port),
    /// External object values
    External(crate::bridge::ExternalObject),
    /// Record values (SRFI 9)
    Record(Record),
    /// Multiple values (R7RS)
    Values(Vec<Value>),
    /// Continuation values (call/cc)
    Continuation(Continuation),
    /// Promise values (SRFI 45 - lazy evaluation)
    Promise(Promise),
}

/// Procedure representation
#[derive(Clone)]
pub enum Procedure {
    /// User-defined procedure (lambda)
    Lambda {
        /// Parameter names
        params: Vec<String>,
        /// Whether this is a variadic procedure
        variadic: bool,
        /// Body expressions
        body: Vec<Expr>,
        /// Closure environment
        closure: Rc<Environment>,
    },
    /// Built-in procedure
    Builtin {
        /// Procedure name
        name: String,
        /// Arity (number of arguments, None for variadic)
        arity: Option<usize>,
        /// Function pointer
        func: fn(&[Value]) -> crate::Result<Value>,
    },
    /// Host function (dynamic closure)
    HostFunction {
        /// Function name
        name: String,
        /// Arity (number of arguments, None for variadic)
        arity: Option<usize>,
        /// Function closure
        func: crate::host::HostFunc,
    },
    /// Continuation (for call/cc)
    Continuation {
        /// Captured continuation
        continuation: Box<Continuation>,
    },
}

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

/// Record type representation (SRFI 9)
#[derive(Debug, Clone, PartialEq)]
pub struct RecordType {
    /// Type name
    pub name: String,
    /// Field names in order
    pub field_names: Vec<String>,
    /// Constructor name
    pub constructor_name: String,
    /// Predicate name
    pub predicate_name: String,
}

/// Record instance (SRFI 9)
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// Record type information
    pub record_type: RecordType,
    /// Field values
    pub fields: Vec<Value>,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undefined => write!(f, "#<undefined>"),
            Value::Boolean(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            Value::Number(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Character(c) => match c {
                ' ' => write!(f, "#\\space"),
                '\n' => write!(f, "#\\newline"),
                '\t' => write!(f, "#\\tab"),
                _ => write!(f, "#\\{c}"),
            },
            Value::Symbol(s) => write!(f, "{s}"),
            Value::Pair(car, cdr) => {
                write!(f, "(")?;
                write!(f, "{car}")?;
                let mut current = cdr.as_ref();
                loop {
                    match current {
                        Value::Nil => break,
                        Value::Pair(car, cdr) => {
                            write!(f, " {car}")?;
                            current = cdr.as_ref();
                        }
                        _ => {
                            write!(f, " . {current}")?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            }
            Value::Nil => write!(f, "()"),
            Value::Procedure(proc) => match proc {
                Procedure::Lambda {
                    params, variadic, ..
                } => {
                    write!(f, "#<procedure (")?;
                    for (i, param) in params.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ")?;
                        }
                        write!(f, "{param}")?;
                    }
                    if *variadic {
                        write!(f, " ...")?;
                    }
                    write!(f, ")>")
                }
                Procedure::Builtin { name, .. } => write!(f, "#<builtin {name}>"),
                Procedure::HostFunction { name, .. } => write!(f, "#<host-function {name}>"),
                Procedure::Continuation { .. } => write!(f, "#<continuation>"),
            },
            Value::Vector(values) => {
                write!(f, "#(")?;
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{value}")?;
                }
                write!(f, ")")
            }
            Value::Port(_) => write!(f, "#<port>"),
            Value::External(obj) => write!(f, "#<external:{}>", obj.type_name),
            Value::Record(record) => {
                write!(f, "#<{}:", record.record_type.name)?;
                for (i, field) in record.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", field)?;
                }
                write!(f, ">")
            }
            Value::Values(values) => {
                if values.len() == 1 {
                    // Single value should display as the value itself
                    write!(f, "{}", values[0])
                } else {
                    // Multiple values display
                    write!(f, "values(")?;
                    for (i, value) in values.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ")?;
                        }
                        write!(f, "{}", value)?;
                    }
                    write!(f, ")")
                }
            }
            Value::Continuation(_) => write!(f, "#<continuation>"),
            Value::Promise(promise) => match &promise.state {
                PromiseState::Lazy { .. } => write!(f, "#<promise:lazy>"),
                PromiseState::Eager { value } => write!(f, "#<promise:eager:{}>", value),
            },
        }
    }
}

impl Value {
    /// Check if this value is truthy (everything except #f is truthy in Scheme)
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Boolean(false))
    }

    /// Check if this value is a number
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    /// Get the number if this is a number
    pub fn as_number(&self) -> Option<&SchemeNumber> {
        match self {
            Value::Number(n) => Some(n),
            _ => None,
        }
    }

    /// Check if this value is a string
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Get the string if this is a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Check if this value is a symbol
    pub fn is_symbol(&self) -> bool {
        matches!(self, Value::Symbol(_))
    }

    /// Get the symbol if this is a symbol
    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Value::Symbol(s) => Some(s),
            _ => None,
        }
    }

    /// Check if this value is a procedure
    pub fn is_procedure(&self) -> bool {
        matches!(self, Value::Procedure(_))
    }

    /// Get the procedure if this is a procedure
    pub fn as_procedure(&self) -> Option<&Procedure> {
        match self {
            Value::Procedure(p) => Some(p),
            _ => None,
        }
    }

    /// Check if this value is a pair
    pub fn is_pair(&self) -> bool {
        matches!(self, Value::Pair(_, _))
    }

    /// Get the pair components if this is a pair
    pub fn as_pair(&self) -> Option<(&Value, &Value)> {
        match self {
            Value::Pair(car, cdr) => Some((car, cdr)),
            _ => None,
        }
    }

    /// Check if this value is nil (empty list)
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Check if this value is a list (proper list ending in nil)
    pub fn is_list(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Pair(_, cdr) => cdr.is_list(),
            _ => false,
        }
    }

    /// Convert this value to a vector if it's a list
    pub fn to_vector(&self) -> Option<Vec<Value>> {
        let mut result = Vec::new();
        let mut current = self;

        loop {
            match current {
                Value::Nil => return Some(result),
                Value::Pair(car, cdr) => {
                    result.push((**car).clone());
                    current = cdr;
                }
                _ => return None, // Not a proper list
            }
        }
    }

    /// Create a list from a vector of values
    pub fn from_vector(values: Vec<Value>) -> Value {
        values.into_iter().rev().fold(Value::Nil, |acc, val| {
            Value::Pair(Box::new(val), Box::new(acc))
        })
    }

    /// Create a pair (cons cell)
    pub fn cons(car: Value, cdr: Value) -> Value {
        Value::Pair(Box::new(car), Box::new(cdr))
    }

    /// Get the car (first element) of a pair
    pub fn car(&self) -> Option<&Value> {
        match self {
            Value::Pair(car, _) => Some(car),
            _ => None,
        }
    }

    /// Get the cdr (rest) of a pair
    pub fn cdr(&self) -> Option<&Value> {
        match self {
            Value::Pair(_, cdr) => Some(cdr),
            _ => None,
        }
    }

    /// Get the length of a list
    pub fn list_length(&self) -> Option<usize> {
        let mut length = 0;
        let mut current = self;

        loop {
            match current {
                Value::Nil => return Some(length),
                Value::Pair(_, cdr) => {
                    length += 1;
                    current = cdr;
                }
                _ => return None, // Not a proper list
            }
        }
    }

    /// Check if this value is a record
    pub fn is_record(&self) -> bool {
        matches!(self, Value::Record(_))
    }

    /// Get the record if this is a record
    pub fn as_record(&self) -> Option<&Record> {
        match self {
            Value::Record(r) => Some(r),
            _ => None,
        }
    }

    /// Check if this is a record of a specific type
    pub fn is_record_of_type(&self, type_name: &str) -> bool {
        match self {
            Value::Record(record) => record.record_type.name == type_name,
            _ => false,
        }
    }

    /// Check if two values are equal
    pub fn equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Character(a), Value::Character(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Pair(car1, cdr1), Value::Pair(car2, cdr2)) => {
                car1.equal(car2) && cdr1.equal(cdr2)
            }
            (Value::Vector(a), Value::Vector(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equal(y))
            }
            (Value::External(a), Value::External(b)) => a.id == b.id,
            (Value::Record(a), Value::Record(b)) => {
                a.record_type == b.record_type
                    && a.fields.len() == b.fields.len()
                    && a.fields
                        .iter()
                        .zip(b.fields.iter())
                        .all(|(x, y)| x.equal(y))
            }
            (Value::Values(a), Value::Values(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equal(y))
            }
            _ => false,
        }
    }

    /// Check if two values are equivalent (eqv?)
    pub fn eqv(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Character(a), Value::Character(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Record(_), Value::Record(_)) => std::ptr::eq(self, other),
            (Value::Values(_), Value::Values(_)) => std::ptr::eq(self, other),
            _ => std::ptr::eq(self, other),
        }
    }

    /// Check if two values are the same object (eq?)
    pub fn scheme_eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Character(a), Value::Character(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Record(_), Value::Record(_)) => std::ptr::eq(self, other),
            (Value::Values(_), Value::Values(_)) => std::ptr::eq(self, other),
            _ => std::ptr::eq(self, other),
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => write!(f, "Undefined"),
            Self::Boolean(arg0) => f.debug_tuple("Boolean").field(arg0).finish(),
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Character(arg0) => f.debug_tuple("Character").field(arg0).finish(),
            Self::Symbol(arg0) => f.debug_tuple("Symbol").field(arg0).finish(),
            Self::Pair(arg0, arg1) => f.debug_tuple("Pair").field(arg0).field(arg1).finish(),
            Self::Nil => write!(f, "Nil"),
            Self::Procedure(arg0) => f.debug_tuple("Procedure").field(arg0).finish(),
            Self::Vector(arg0) => f.debug_tuple("Vector").field(arg0).finish(),
            Self::Port(arg0) => f.debug_tuple("Port").field(arg0).finish(),
            Self::External(arg0) => f.debug_tuple("External").field(arg0).finish(),
            Self::Record(arg0) => f.debug_tuple("Record").field(arg0).finish(),
            Self::Values(arg0) => f.debug_tuple("Values").field(arg0).finish(),
            Self::Continuation(arg0) => f.debug_tuple("Continuation").field(arg0).finish(),
            Self::Promise(arg0) => f.debug_tuple("Promise").field(arg0).finish(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Character(l0), Self::Character(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::Pair(l0, l1), Self::Pair(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Procedure(l0), Self::Procedure(r0)) => l0 == r0,
            (Self::Vector(l0), Self::Vector(r0)) => l0 == r0,
            (Self::Port(l0), Self::Port(r0)) => l0 == r0,
            (Self::External(l0), Self::External(r0)) => l0.id == r0.id,
            (Self::Record(l0), Self::Record(r0)) => l0 == r0,
            (Self::Values(l0), Self::Values(r0)) => l0 == r0,
            (Self::Continuation(_), Self::Continuation(_)) => false, // Continuations are never equal
            (Self::Promise(_), Self::Promise(_)) => false, // Promises are never equal per SRFI 45
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl std::fmt::Debug for Procedure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lambda {
                params,
                variadic,
                body,
                ..
            } => f
                .debug_struct("Lambda")
                .field("params", params)
                .field("variadic", variadic)
                .field("body", body)
                .finish(),
            Self::Builtin { name, arity, .. } => f
                .debug_struct("Builtin")
                .field("name", name)
                .field("arity", arity)
                .finish(),
            Self::HostFunction { name, arity, .. } => f
                .debug_struct("HostFunction")
                .field("name", name)
                .field("arity", arity)
                .finish(),
            Self::Continuation { continuation } => f
                .debug_struct("Continuation")
                .field("continuation", continuation)
                .finish(),
        }
    }
}

impl PartialEq for Procedure {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Lambda {
                    params: l_params,
                    variadic: l_variadic,
                    body: l_body,
                    closure: l_closure,
                },
                Self::Lambda {
                    params: r_params,
                    variadic: r_variadic,
                    body: r_body,
                    closure: r_closure,
                },
            ) => {
                l_params == r_params
                    && l_variadic == r_variadic
                    && l_body == r_body
                    && std::ptr::eq(l_closure.as_ref(), r_closure.as_ref())
            }
            (
                Self::Builtin {
                    name: l_name,
                    arity: l_arity,
                    ..
                },
                Self::Builtin {
                    name: r_name,
                    arity: r_arity,
                    ..
                },
            ) => l_name == r_name && l_arity == r_arity,
            (
                Self::HostFunction {
                    name: l_name,
                    arity: l_arity,
                    ..
                },
                Self::HostFunction {
                    name: r_name,
                    arity: r_arity,
                    ..
                },
            ) => l_name == r_name && l_arity == r_arity,
            (Self::Continuation { continuation: _ }, Self::Continuation { continuation: _ }) => {
                false // Continuations are never equal
            }
            _ => false,
        }
    }
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

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Number(SchemeNumber::Integer(i))
    }
}

impl From<u64> for Value {
    fn from(u: u64) -> Self {
        Value::Number(SchemeNumber::Integer(u as i64))
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Number(SchemeNumber::Real(f))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<char> for Value {
    fn from(c: char) -> Self {
        Value::Character(c)
    }
}

impl From<SchemeNumber> for Value {
    fn from(n: SchemeNumber) -> Self {
        Value::Number(n)
    }
}

/// Continuation representation for call/cc
#[derive(Clone, Debug)]
pub struct Continuation {
    /// The call stack at the time of capture
    pub stack: Vec<StackFrame>,
    /// The environment at the time of capture
    pub env: Rc<Environment>,
}

/// Stack frame for continuation representation
#[derive(Clone, Debug)]
pub struct StackFrame {
    /// The expression being evaluated
    pub expr: Expr,
    /// The environment for this frame
    pub env: Rc<Environment>,
}

/// Promise for lazy evaluation (SRFI 45)
#[derive(Clone, Debug)]
pub struct Promise {
    /// The current state of the promise
    pub state: PromiseState,
}

/// State of a promise
#[derive(Clone, Debug)]
pub enum PromiseState {
    /// Unevaluated promise with expression and environment
    Lazy {
        /// Expression to evaluate
        expr: Expr,
        /// Environment for evaluation
        env: Rc<Environment>,
    },
    /// Evaluated promise with cached value
    Eager {
        /// Cached result value
        value: Box<Value>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Boolean(true)), "#t");
        assert_eq!(format!("{}", Value::Boolean(false)), "#f");
        assert_eq!(
            format!("{}", Value::Number(SchemeNumber::Integer(42))),
            "42"
        );
        assert_eq!(
            format!("{}", Value::String("hello".to_string())),
            "\"hello\""
        );
        assert_eq!(format!("{}", Value::Character('a')), "#\\a");
        assert_eq!(format!("{}", Value::Symbol("foo".to_string())), "foo");
        assert_eq!(format!("{}", Value::Nil), "()");
    }

    #[test]
    fn test_list_operations() {
        let list = Value::from_vector(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64),
        ]);

        assert!(list.is_list());
        assert_eq!(list.list_length(), Some(3));

        let vec = list.to_vector().unwrap();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], Value::from(1i64));
    }

    #[test]
    fn test_pair_operations() {
        let pair = Value::cons(Value::from(1i64), Value::from(2i64));

        assert!(pair.is_pair());
        assert_eq!(pair.car(), Some(&Value::from(1i64)));
        assert_eq!(pair.cdr(), Some(&Value::from(2i64)));
    }

    #[test]
    fn test_equality() {
        let a = Value::from(42i64);
        let b = Value::from(42i64);
        let c = Value::from(43i64);

        assert!(a.equal(&b));
        assert!(!a.equal(&c));
        assert!(a.eqv(&b));
        assert!(!a.eqv(&c));
    }

    #[test]
    fn test_truthiness() {
        assert!(Value::Boolean(true).is_truthy());
        assert!(!Value::Boolean(false).is_truthy());
        assert!(Value::from(0i64).is_truthy());
        assert!(Value::Nil.is_truthy());
        assert!(Value::String("".to_string()).is_truthy());
    }
}

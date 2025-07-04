//! Runtime values in the Scheme interpreter

// Re-export all submodules
pub mod conversions;
pub mod continuation;
pub mod display;
pub mod equality;
pub mod list;
pub mod pair;
pub mod port;
pub mod predicates;
pub mod procedure;
pub mod promise;
pub mod record;

// Re-export key types
pub use continuation::{Continuation, StackFrame};
pub use pair::PairData;
pub use port::Port;
pub use procedure::Procedure;
pub use promise::{Promise, PromiseState};
pub use record::{Record, RecordType};

use crate::lexer::SchemeNumber;

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
    /// Pair values (cons cells) - shared reference for efficient memory management
    Pair(std::rc::Rc<std::cell::RefCell<PairData>>),
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
    /// Hash table values (SRFI 69)
    HashTable(std::rc::Rc<std::cell::RefCell<crate::builtins::srfi_69::HashTable>>),
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
        assert_eq!(pair.car(), Some(Value::from(1i64)));
        assert_eq!(pair.cdr(), Some(Value::from(2i64)));
        
        // Test mutation operations
        pair.set_car(Value::from(10i64)).unwrap();
        assert_eq!(pair.car(), Some(Value::from(10i64)));
        
        pair.set_cdr(Value::from(20i64)).unwrap();
        assert_eq!(pair.cdr(), Some(Value::from(20i64)));
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
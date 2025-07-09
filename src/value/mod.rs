//! Runtime values in the Scheme interpreter

// Re-export all submodules
pub mod continuation;
pub mod conversions;
pub mod display;
pub mod equality;
pub mod lazy_vector;
pub mod list;
pub mod optimized;
pub mod pair;
pub mod port;
pub mod predicates;
pub mod procedure;
pub mod promise;
pub mod record;

// Re-export key types
pub use continuation::{Continuation, StackFrame};
pub use lazy_vector::{MemoryStats, VectorStorage};
pub use optimized::{OptimizationStats, OptimizedValue, ShortStringData, ValueOptimizer};
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
    /// Vector values (traditional immediate allocation)
    Vector(Vec<Value>),
    /// Lazy vector values (memory-efficient for large vectors)
    LazyVector(std::rc::Rc<std::cell::RefCell<lazy_vector::VectorStorage>>),
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
    HashTable(std::rc::Rc<std::cell::RefCell<crate::srfi::srfi_69::HashTable>>),
    /// Box values (SRFI 111)
    Box(crate::srfi::srfi_111::Box),
    /// Comparator values (SRFI 128)
    Comparator(std::rc::Rc<crate::srfi::srfi_128::Comparator>),
    /// String cursor values (SRFI 130)
    StringCursor(std::rc::Rc<crate::srfi::srfi_130::StringCursor>),
    /// Immutable deque values (SRFI 134)
    Ideque(std::rc::Rc<crate::srfi::srfi_134::Ideque>),
    /// Immutable text values (SRFI 135)
    Text(std::rc::Rc<crate::srfi::srfi_135::Text>),
    /// Immutable string values (SRFI 140)
    IString(std::rc::Rc<crate::srfi::srfi_140::IString>),
}

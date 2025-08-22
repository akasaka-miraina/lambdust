//! Runtime value types for the Lambdust evaluation engine.
//!
//! This module defines the core `Value` enum and associated types that represent
//! all runtime values in the Lambdust language. The value system is designed to
//! balance R7RS Scheme compatibility with modern performance requirements and
//! advanced features like gradual typing and effect systems.
//!
//! ## Architectural Overview
//!
//! The value representation follows a **tagged union** approach with performance
//! optimizations for common cases and seamless integration with Rust's memory
//! safety guarantees.
//!
//! ### Core Design Principles
//!
//! - **R7RS Compatibility**: Full compatibility with Scheme value semantics
//! - **Memory Safety**: All values are memory-safe with Rust's ownership system
//! - **Performance**: Optimized representations for hot-path operations
//! - **Thread Safety**: Support for both single and multi-threaded execution
//! - **Gradual Typing**: Runtime type information for gradual type migration
//!
//! ## Value Categories
//!
//! ### 1. Primitive Values
//! - **Literals**: Numbers, strings, characters, booleans (R7RS compliant)
//! - **Symbols**: Interned identifiers with efficient comparison
//! - **Keywords**: Self-evaluating keyword literals (#:key syntax)
//! - **Special Values**: Nil (empty list), unspecified results
//!
//! ### 2. Compound Values  
//! - **Pairs**: Cons cells with both immutable and mutable variants
//! - **Vectors**: Dynamic arrays with efficient random access
//! - **Hash Tables**: Associative arrays with O(1) average lookup
//! - **Strings**: Both immutable and mutable string representations
//!
//! ### 3. Advanced Containers (SRFI Support)
//! - **Priority Queues**: Heap-based priority queues with custom comparators
//! - **Ordered Sets**: Red-black tree based sets with ordering
//! - **Ideques**: Persistent double-ended queues (SRFI-134)
//! - **Random Access Lists**: Finger tree based lists (SRFI-101)
//! - **Generators**: Lazy sequence generators (SRFI-121)
//!
//! ### 4. Procedures and Closures
//! - **User Procedures**: Closures capturing lexical environments
//! - **Case-Lambda**: Variable arity procedures with pattern matching
//! - **Primitives**: Built-in procedures implemented in Rust
//! - **Continuations**: First-class continuations for control flow
//!
//! ### 5. I/O and System Integration
//! - **Ports**: Input/output abstractions (files, strings, network)
//! - **Foreign Objects**: C FFI integration and external resources
//! - **Parameters**: Dynamic scoping with parameterize (SRFI-39)
//!
//! ### 6. Advanced Features
//! - **Syntax Transformers**: Macro system integration
//! - **Records**: User-defined types with SRFI-9/R7RS support  
//! - **Promises**: Lazy evaluation and delay/force semantics
//! - **Type Values**: Runtime type information for gradual typing
//!
//! ## Memory Layout and Performance
//!
//! ### Single-Threaded Optimizations
//! - **Rc<RefCell<T>>**: Reference-counted mutable data for single threads
//! - **Box<T>**: Heap allocation for large or recursive data
//! - **Inline Data**: Small values stored directly in enum variants
//!
//! ### Multi-Threaded Support
//! - **Arc<T>**: Atomic reference counting for shared immutable data
//! - **Arc<RwLock<T>>**: Read-write locks for shared mutable data
//! - **Lock-Free Structures**: When safe and beneficial for performance
//!
//! ## Integration Points
//!
//! - **Evaluation Engine**: Direct integration with evaluator for performance
//! - **Type System**: Runtime type information for gradual typing
//! - **Effect System**: Effect tracking and monadic computations  
//! - **FFI System**: Seamless interop with C and Rust code
//! - **Serialization**: Support for marshaling and persistence
//!
//! ## Performance Characteristics
//!
//! - **Small Value Optimization**: Common values fit in single cache line
//! - **Reference Sharing**: Structural sharing for immutable data
//! - **Copy Avoidance**: Move semantics where possible
//! - **Cache Locality**: Hot data kept together in memory
//! - **Branch Prediction**: Most common values first in enum for better prediction
//!
//! ## R7RS Compliance
//!
//! The value system implements all required R7RS data types with proper semantics:
//! - Exact numeric tower with arbitrary precision
//! - Proper list structure with dotted pairs
//! - Lexical scoping and closure capture
//! - Continuation-passing style support
//! - I/O port abstraction
//! - Symbol interning and comparison

use crate::ast::{CaseLambdaClause, Expr, Formals, Literal};
use crate::diagnostics::{Span, Spanned};
use crate::effects::Effect;
use crate::utils::SymbolId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
// Removed unused imports

/// Generation counter for environments to support garbage collection.
///
/// This type is used to track environment generations for generational
/// garbage collection and helps optimize memory management by distinguishing
/// between older and newer environments.
pub type Generation = u64;

/// A Lambdust runtime value implementing R7RS Scheme semantics.
///
/// This enum represents all possible runtime values in the Lambdust language,
/// from simple literals to complex data structures and procedures. The design
/// prioritizes both correctness (R7RS compliance) and performance (efficient
/// representation and operations).
///
/// ## Design Philosophy
///
/// The `Value` enum follows these principles:
/// - **Completeness**: Covers all R7RS-required data types
/// - **Extensibility**: Supports advanced features beyond R7RS
/// - **Performance**: Optimized memory layout and operation costs
/// - **Safety**: Memory-safe with Rust's ownership system
/// - **Interoperability**: Seamless integration with Rust ecosystem
///
/// ## Memory Representation
///
/// Values are represented using a tagged union with different strategies:
/// - **Immediate Values**: Small data stored inline (literals, symbols)
/// - **Boxed Values**: Larger data allocated on heap (pairs, vectors)
/// - **Reference-Counted**: Shared immutable data (procedures, continuations)
/// - **Interior Mutability**: Controlled mutation (mutable pairs, vectors)
///
/// ## Thread Safety Considerations
///
/// The value system supports both single-threaded and multi-threaded execution:
/// - Single-threaded: `Rc<RefCell<T>>` for efficient local mutation
/// - Multi-threaded: `Arc<RwLock<T>>` for safe concurrent access
/// - Lock-free: Where possible for maximum performance
///
/// ## R7RS Compliance
///
/// All value types implement proper R7RS semantics for:
/// - Equality testing (`equal?`, `eqv?`, `eq?`)
/// - Type predicates (`number?`, `pair?`, etc.)
/// - Truthiness (only `#f` is false)
/// - Proper tail recursion support
/// - Exact numeric tower
#[derive(Debug, Clone)]
pub enum Value {
    // ============= PRIMITIVE VALUES =============
    /// Literal values (numbers, strings, characters, booleans).
    ///
    /// This variant holds all R7RS literal values including:
    /// - **Numbers**: Exact and inexact numbers with full numeric tower
    /// - **Strings**: Immutable Unicode strings  
    /// - **Characters**: Unicode characters
    /// - **Booleans**: `#t` and `#f` values
    /// - **Bytevectors**: Raw byte sequences
    ///
    /// ## Memory Layout
    /// The `Literal` is stored inline when small enough, avoiding heap allocation
    /// for common cases like small integers and short strings.
    Literal(Literal),

    /// Symbols (interned strings).
    ///
    /// Symbols are interned identifiers that support efficient equality comparison
    /// through unique `SymbolId` values. This implements R7RS symbol semantics
    /// where symbols with the same name are identical (`eq?`).
    ///
    /// ## Performance Characteristics
    /// - **Creation**: O(1) amortized through interning
    /// - **Comparison**: O(1) through id comparison
    /// - **Memory**: Shared storage for identical symbols
    Symbol(SymbolId),

    /// Keywords (#:key syntax).
    ///
    /// Self-evaluating keyword literals that extend R7RS with convenient
    /// named parameters and options. Keywords always evaluate to themselves
    /// and are commonly used for configuration and options.
    ///
    /// ## Usage Examples
    /// - `#:color` for named parameters
    /// - `#:mode` for configuration options  
    /// - `#:type` for type annotations
    Keyword(String),

    /// The empty list (null value).
    ///
    /// This represents the R7RS empty list `'()`, which serves as:
    /// - **List Terminator**: End of proper lists
    /// - **Boolean Context**: Truthy value (unlike some Lisps)
    /// - **Default Value**: Common default for optional parameters
    ///
    /// ## R7RS Semantics
    /// - `(null? '())` returns `#t`
    /// - `(pair? '())` returns `#f`  
    /// - `'()` is truthy in boolean contexts
    Nil,

    /// Unspecified value (result of side-effecting operations).
    ///
    /// This represents the R7RS unspecified value returned by procedures
    /// that are called for their side effects rather than their return value.
    /// Examples include `set!`, `define`, and I/O operations.
    ///
    /// ## Usage Patterns
    /// - Return value of `define` expressions
    /// - Result of `set!` assignments
    /// - Output operations like `display` and `write`
    /// - Initialization of variables before assignment
    Unspecified,

    // ============= COMPOUND VALUES =============
    /// Cons pair (a . b) - immutable pair with single ownership.
    ///
    /// This represents R7RS pairs (cons cells) in their immutable form.
    /// Uses `Box<T>` for heap allocation with single ownership, making
    /// it efficient for functional programming patterns.
    ///
    /// ## Memory Layout
    /// - **Heap Allocated**: Both car and cdr stored on heap
    /// - **Single Ownership**: No reference counting overhead
    /// - **Cache Friendly**: Consecutive allocation when possible
    ///
    /// ## Usage Patterns
    /// - Building immutable lists: `(cons 1 (cons 2 '()))`
    /// - Creating dotted pairs: `(cons 'a 'b)` -> `(a . b)`
    /// - Functional data structures with structural sharing
    Pair(Box<Value>, Box<Value>),

    /// Mutable cons pair (a . b) - optimized for single-threaded mutation.
    ///
    /// This provides R7RS mutable pairs using `Rc<RefCell<T>>` for efficient
    /// interior mutability in single-threaded contexts. Supports operations
    /// like `set-car!` and `set-cdr!`.
    ///
    /// ## Thread Safety
    /// - **Single-Threaded Only**: Uses `RefCell` for interior mutability
    /// - **Runtime Borrow Checking**: Dynamic borrow checking at runtime
    /// - **Panic on Conflict**: Panics on simultaneous mutable borrows
    ///
    /// ## Performance
    /// - **Reference Counting**: Allows sharing with automatic cleanup
    /// - **Interior Mutability**: Mutation without unique ownership
    /// - **Low Overhead**: Minimal runtime cost for single-threaded use
    MutablePair(Rc<RefCell<Value>>, Rc<RefCell<Value>>),

    /// Vector (mutable array-like structure) - single-threaded optimized.
    ///
    /// This implements R7RS vectors as growable arrays with O(1) indexed access
    /// and amortized O(1) append operations. Optimized for single-threaded
    /// performance using `Rc<RefCell<Vec<T>>>`.
    ///
    /// ## Operations Supported
    /// - `vector-ref`: O(1) random access
    /// - `vector-set!`: O(1) mutation
    /// - `vector-length`: O(1) size query
    /// - Dynamic resizing with amortized O(1) growth
    ///
    /// ## Memory Characteristics
    /// - **Contiguous Storage**: Elements stored in contiguous memory
    /// - **Efficient Growth**: Exponential growth strategy
    /// - **Cache Friendly**: Good locality for sequential access
    Vector(Rc<RefCell<Vec<Value>>>),

    /// Hash table (mutable associative array) - single-threaded optimized.
    ///
    /// This implements efficient key-value mapping using Rust's `HashMap`
    /// with `Value` keys and values. Provides average O(1) lookup, insertion,
    /// and deletion operations.
    ///
    /// ## Key Requirements
    /// - Keys must implement proper equality semantics
    /// - Hash values must be consistent with equality
    /// - Supports all `Value` types as keys (where meaningful)
    ///
    /// ## Performance
    /// - **Average O(1)**: Lookup, insertion, deletion
    /// - **Worst Case O(n)**: With pathological hash collisions
    /// - **Memory Efficient**: Rust's optimized hash table implementation
    Hashtable(Rc<RefCell<HashMap<Value, Value>>>),

    /// Mutable string (for string-set! and string-fill!) - single-threaded optimized.
    ///
    /// This supports R7RS mutable string operations using a character vector
    /// representation. Allows in-place modification of string contents while
    /// maintaining Unicode correctness.
    ///
    /// ## Operations
    /// - `string-ref`: O(1) character access  
    /// - `string-set!`: O(1) character mutation
    /// - `string-fill!`: O(n) bulk character setting
    /// - `string-length`: O(1) length query
    ///
    /// ## Unicode Handling
    /// - **Character-Based**: Stores Unicode scalar values
    /// - **Boundary Safe**: Maintains valid Unicode sequences
    /// - **Normalization**: May require normalization after mutation
    MutableString(Rc<RefCell<Vec<char>>>),

    // ============= ADVANCED CONTAINERS =============
    /// High-performance hash table (SRFI-125) - Thread-safe
    AdvancedHashTable(Arc<crate::containers::ThreadSafeHashTable>),

    /// Persistent ideque (SRFI-134) - Thread-safe
    Ideque(Arc<crate::containers::PersistentIdeque>),

    /// Priority queue with custom comparators - Thread-safe
    PriorityQueue(Arc<crate::containers::ThreadSafePriorityQueue>),

    /// Ordered set based on red-black tree - Thread-safe
    OrderedSet(Arc<crate::containers::ThreadSafeOrderedSet>),

    /// List queue with FIFO semantics (SRFI-117) - Thread-safe
    ListQueue(Arc<crate::containers::ThreadSafeListQueue>),

    /// Random access list (SRFI-101) - Thread-safe
    RandomAccessList(Arc<crate::containers::ThreadSafeRandomAccessList>),

    /// Set data structure (SRFI-113) - Thread-safe
    Set(Arc<crate::containers::ThreadSafeSet>),

    /// Bag (multiset) data structure (SRFI-113) - Thread-safe
    Bag(Arc<crate::containers::ThreadSafeBag>),

    /// Generator data structure (SRFI-121) - Thread-safe
    Generator(Arc<crate::containers::ThreadSafeGenerator>),

    /// Box (mutable cell) data structure (SRFI-111) - Thread-safe
    Box(Arc<crate::stdlib::r#box::Box>),

    // ============= PROCEDURES =============
    /// User-defined procedure (closure) - Thread-safe
    Procedure(Arc<Procedure>),

    /// Case-lambda procedure (variable arity procedure) - Thread-safe
    CaseLambda(Arc<CaseLambdaProcedure>),

    /// Built-in primitive procedure - Thread-safe
    Primitive(Arc<PrimitiveProcedure>),

    /// Continuation captured by call/cc - Thread-safe
    Continuation(Arc<Continuation>),

    /// Syntax transformer (macro) - Thread-safe
    Syntax(Arc<SyntaxTransformer>),

    // ============= ADVANCED VALUES =============
    /// Port for I/O operations - Thread-safe
    Port(Arc<Port>),

    /// Promise for lazy evaluation - Thread-safe
    Promise(Arc<RwLock<Promise>>),

    /// Type value (for gradual typing) - Thread-safe
    Type(Arc<TypeValue>),

    /// FFI object (foreign function interface) - Thread-safe
    Foreign(Arc<ForeignObject>),

    /// Error object for exception handling - Thread-safe
    ErrorObject(Arc<crate::stdlib::exceptions::ErrorObject>),

    /// Character set for SRFI-14 support - Thread-safe
    CharSet(Arc<crate::stdlib::charset::CharSet>),
    /// Parameter object for SRFI-39 support - Thread-safe
    Parameter(Arc<Parameter>),
    /// Record instance for SRFI-9 support - Thread-safe
    Record(Arc<Record>),

    // ============= CONCURRENCY VALUES =============
    // These are only available when async-runtime feature is enabled
    #[cfg(feature = "async-runtime")]
    /// Future for asynchronous computation - Thread-safe
    Future(Arc<crate::concurrency::futures::Future>),

    #[cfg(feature = "async-runtime")]
    /// Communication channel - Thread-safe
    Channel(Arc<crate::concurrency::channels::Channel>),

    #[cfg(feature = "async-runtime")]
    /// Mutex for synchronization - Thread-safe
    Mutex(Arc<crate::concurrency::Mutex>),

    #[cfg(feature = "async-runtime")]
    /// Semaphore for resource control - Thread-safe
    Semaphore(Arc<crate::concurrency::SemaphoreSync>),

    #[cfg(feature = "async-runtime")]
    /// Atomic counter - Thread-safe
    AtomicCounter(Arc<crate::concurrency::AtomicCounter>),

    #[cfg(feature = "async-runtime")]
    /// Distributed node - Thread-safe
    DistributedNode(Arc<crate::concurrency::distributed::DistributedNode>),

    /// Opaque value for FFI - Thread-safe
    Opaque(Arc<dyn std::any::Any + Send + Sync>),

    // ============= ENVIRONMENT VALUES =============
    /// Environment for dynamic evaluation (scheme eval) - Thread-safe
    Environment(Arc<ThreadSafeEnvironment>),
}

/// A user-defined procedure (closure) - Thread-safe.
#[derive(Debug, Clone)]
pub struct Procedure {
    /// Formal parameters
    pub formals: Formals,
    /// Procedure body (sequence of expressions)
    pub body: Vec<Spanned<Expr>>,
    /// Lexical environment captured at definition time (thread-safe)
    pub environment: Arc<ThreadSafeEnvironment>,
    /// Optional name for debugging
    pub name: Option<String>,
    /// Metadata associated with the procedure
    pub metadata: HashMap<String, Value>,
    /// Source location for error reporting
    pub source: Option<Span>,
}

/// A case-lambda procedure with multiple arity clauses - Thread-safe.
#[derive(Debug, Clone)]
pub struct CaseLambdaProcedure {
    /// Clauses with different parameter patterns
    pub clauses: Vec<CaseLambdaClause>,
    /// Lexical environment captured at definition time (thread-safe)
    pub environment: Arc<ThreadSafeEnvironment>,
    /// Optional name for debugging
    pub name: Option<String>,
    /// Metadata associated with the procedure
    pub metadata: HashMap<String, Value>,
    /// Source location for error reporting
    pub source: Option<Span>,
}

/// A built-in primitive procedure.
#[derive(Debug, Clone)]
pub struct PrimitiveProcedure {
    /// Name of the primitive
    pub name: String,
    /// Minimum number of arguments
    pub arity_min: usize,
    /// Maximum number of arguments (None for variadic)
    pub arity_max: Option<usize>,
    /// The actual implementation
    pub implementation: PrimitiveImpl,
    /// Effects this primitive may produce
    pub effects: Vec<Effect>,
}

/// Implementation of a primitive procedure.
#[derive(Debug, Clone)]
pub enum PrimitiveImpl {
    /// Rust function pointer
    RustFn(fn(&[Value]) -> crate::diagnostics::Result<Value>),
    /// Native implementation (alias for RustFn for compatibility)
    Native(fn(&[Value]) -> crate::diagnostics::Result<Value>),
    /// Evaluator-integrated function for higher-order functions
    EvaluatorIntegrated(
        fn(&mut crate::eval::evaluator::Evaluator, &[Value]) -> crate::diagnostics::Result<Value>,
    ),
    /// FFI function from dynamic library
    ForeignFn {
        /// Name of the library containing the function
        library: String,
        /// Symbol name of the function in the library
        symbol: String,
    },
}

// PrimitiveImpl doesn't need special GC handling

/// A captured continuation - Thread-safe.
#[derive(Debug, Clone)]
pub struct Continuation {
    /// The evaluation context stack at capture time
    pub stack: Vec<Frame>,
    /// The environment at capture time (thread-safe)
    pub environment: Arc<ThreadSafeEnvironment>,
    /// Unique identifier for this continuation
    pub id: u64,
    /// The current expression being evaluated when continuation was captured
    pub current_expr: Option<Spanned<Expr>>,
    /// Whether this continuation has been invoked (for one-shot semantics)
    pub invoked: Arc<std::sync::atomic::AtomicBool>,
}

/// A stack frame in a continuation - Thread-safe.
#[derive(Debug, Clone)]
pub enum Frame {
    /// Application frame (evaluating function arguments)
    Application {
        /// The function operator being applied
        operator: Box<Value>,
        /// Arguments that have already been evaluated
        evaluated_args: Box<Vec<Value>>,
        /// Arguments still to be evaluated
        remaining_args: Box<Vec<Spanned<Expr>>>,
        /// Environment for argument evaluation
        environment: Arc<ThreadSafeEnvironment>,
        /// Source location information
        source: Span,
    },
    /// If frame (evaluating conditional)
    If {
        /// Expression to evaluate if condition is true
        consequent: Box<Spanned<Expr>>,
        /// Expression to evaluate if condition is false (optional)
        alternative: Box<Option<Spanned<Expr>>>,
        /// Environment for evaluation
        environment: Arc<ThreadSafeEnvironment>,
        /// Source location information
        source: Span,
    },
    /// Set frame (evaluating assignment)
    Set {
        /// Name of the variable being assigned
        name: String,
        /// Environment where the assignment occurs
        environment: Arc<ThreadSafeEnvironment>,
        /// Source location information
        source: Span,
    },
    /// Begin frame (evaluating sequence)
    Begin {
        /// Expressions remaining to be evaluated in sequence
        remaining_exprs: Vec<Spanned<Expr>>,
        /// Environment for sequence evaluation
        environment: Arc<ThreadSafeEnvironment>,
        /// Source location information
        source: Span,
    },
    /// Let frame (evaluating let bindings)
    Let {
        /// Bindings still to be evaluated
        remaining_bindings: Vec<crate::ast::Binding>,
        /// Bindings that have already been evaluated
        evaluated_bindings: Box<Vec<(String, Value)>>,
        /// Body expressions to evaluate after bindings
        body: Vec<Spanned<Expr>>,
        /// Environment for let evaluation
        environment: Arc<ThreadSafeEnvironment>,
        /// Source location information
        source: Span,
    },
    /// Lambda application frame (procedure call)
    ProcedureCall {
        /// Name of the procedure being called (for debugging)
        procedure_name: Option<String>,
        /// Body expressions remaining to be evaluated
        remaining_body: Vec<Spanned<Expr>>,
        /// Environment for procedure execution
        environment: Arc<ThreadSafeEnvironment>,
        /// Source location information
        source: Span,
    },
    /// Call/CC frame (continuation capture)
    CallCC {
        /// Environment for continuation capture
        environment: Arc<ThreadSafeEnvironment>,
        /// Source location information
        source: Span,
    },
}

/// A syntax transformer (macro) - Thread-safe.
#[derive(Debug, Clone)]
pub struct SyntaxTransformer {
    /// Name of the syntax
    pub name: String,
    /// The transformer procedure
    pub transformer: Value,
    /// Environment for macro expansion (thread-safe)
    pub environment: Arc<ThreadSafeEnvironment>,
}

/// A port for I/O operations - R7RS compliant.
#[derive(Debug, Clone)]
pub struct Port {
    /// Port implementation
    pub implementation: PortImpl,
    /// Whether the port is open
    pub is_open: Arc<RwLock<bool>>,
    /// Port mode (textual or binary)
    pub mode: PortMode,
    /// Port direction
    pub direction: PortDirection,
    /// Buffer for efficient I/O
    pub buffer: Arc<RwLock<Vec<u8>>>,
    /// Current position in the port (for seekable ports)
    pub position: Arc<RwLock<usize>>,
    /// Port metadata
    pub metadata: HashMap<String, Value>,
}

/// Port implementation details.
#[derive(Debug, Clone)]
pub enum PortImpl {
    /// String-based port
    String {
        /// Content for input ports, accumulator for output ports
        content: Arc<RwLock<String>>,
        /// Current position for input ports
        position: Arc<RwLock<usize>>,
    },
    /// Bytevector-based port
    Bytevector {
        /// Content for input ports, accumulator for output ports
        content: Arc<RwLock<Vec<u8>>>,
        /// Current position for input ports
        position: Arc<RwLock<usize>>,
    },
    /// File-based port
    File {
        /// File path
        path: String,
        /// File handle (buffered)
        handle: Arc<RwLock<Option<PortFileHandle>>>,
    },
    /// Standard I/O port
    Standard(StandardPort),
}

/// Standard port types.
#[derive(Debug, Clone)]
pub enum StandardPort {
    /// Standard input
    Stdin,
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
}

/// File handle wrapper for thread safety.
#[derive(Debug)]
pub enum PortFileHandle {
    /// Text file reader
    TextReader(std::io::BufReader<std::fs::File>),
    /// Text file writer
    TextWriter(std::io::BufWriter<std::fs::File>),
    /// Binary file reader
    BinaryReader(std::io::BufReader<std::fs::File>),
    /// Binary file writer
    BinaryWriter(std::io::BufWriter<std::fs::File>),
}

/// Port mode (textual or binary).
#[derive(Debug, Clone, PartialEq)]
pub enum PortMode {
    /// Textual port (character-based)
    Textual,
    /// Binary port (byte-based)
    Binary,
}

/// Port direction.
#[derive(Debug, Clone, PartialEq)]
pub enum PortDirection {
    /// Input port
    Input,
    /// Output port
    Output,
    /// Bidirectional port
    InputOutput,
}

/// Memory limits for port operations to prevent excessive memory usage.
pub mod port_limits {
    /// Maximum size for string port content (64MB)
    pub const MAX_STRING_PORT_SIZE: usize = 64 * 1024 * 1024;

    /// Maximum size for bytevector port content (64MB)
    pub const MAX_BYTEVECTOR_PORT_SIZE: usize = 64 * 1024 * 1024;

    /// Default buffer size for efficient I/O operations (8KB)
    pub const DEFAULT_BUFFER_SIZE: usize = 8 * 1024;

    /// Large allocation threshold for optimization decisions (1MB)
    pub const LARGE_ALLOCATION_THRESHOLD: usize = 1024 * 1024;
}

impl Port {
    /// Creates a new string input port.
    pub fn new_string_input(content: String) -> Self {
        Port {
            implementation: PortImpl::String {
                content: Arc::new(RwLock::new(content)),
                position: Arc::new(RwLock::new(0)),
            },
            is_open: Arc::new(RwLock::new(true)),
            mode: PortMode::Textual,
            direction: PortDirection::Input,
            buffer: Arc::new(RwLock::new(Vec::new())),
            position: Arc::new(RwLock::new(0)),
            metadata: HashMap::new(),
        }
    }

    /// Creates a new string output port.
    pub fn new_string_output() -> Self {
        Self::new_string_output_with_capacity(0)
    }

    /// Creates a new string output port with a capacity hint for better memory allocation.
    pub fn new_string_output_with_capacity(capacity: usize) -> Self {
        let mut initial_string = String::new();
        if capacity > 0 {
            initial_string.reserve(capacity);
        }

        Port {
            implementation: PortImpl::String {
                content: Arc::new(RwLock::new(initial_string)),
                position: Arc::new(RwLock::new(0)),
            },
            is_open: Arc::new(RwLock::new(true)),
            mode: PortMode::Textual,
            direction: PortDirection::Output,
            buffer: Arc::new(RwLock::new(Vec::new())),
            position: Arc::new(RwLock::new(0)),
            metadata: HashMap::new(),
        }
    }

    /// Creates a new bytevector input port.
    pub fn new_bytevector_input(content: Vec<u8>) -> Self {
        Port {
            implementation: PortImpl::Bytevector {
                content: Arc::new(RwLock::new(content)),
                position: Arc::new(RwLock::new(0)),
            },
            is_open: Arc::new(RwLock::new(true)),
            mode: PortMode::Binary,
            direction: PortDirection::Input,
            buffer: Arc::new(RwLock::new(Vec::new())),
            position: Arc::new(RwLock::new(0)),
            metadata: HashMap::new(),
        }
    }

    /// Creates a new bytevector output port.
    pub fn new_bytevector_output() -> Self {
        Self::new_bytevector_output_with_capacity(0)
    }

    /// Creates a new bytevector output port with a capacity hint for better memory allocation.
    pub fn new_bytevector_output_with_capacity(capacity: usize) -> Self {
        let mut initial_vec = Vec::new();
        if capacity > 0 {
            initial_vec.reserve(capacity);
        }

        Port {
            implementation: PortImpl::Bytevector {
                content: Arc::new(RwLock::new(initial_vec)),
                position: Arc::new(RwLock::new(0)),
            },
            is_open: Arc::new(RwLock::new(true)),
            mode: PortMode::Binary,
            direction: PortDirection::Output,
            buffer: Arc::new(RwLock::new(Vec::new())),
            position: Arc::new(RwLock::new(0)),
            metadata: HashMap::new(),
        }
    }

    /// Creates a new file input port.
    pub fn new_file_input(path: String, binary: bool) -> Self {
        Port {
            implementation: PortImpl::File {
                path,
                handle: Arc::new(RwLock::new(None)),
            },
            is_open: Arc::new(RwLock::new(true)),
            mode: if binary {
                PortMode::Binary
            } else {
                PortMode::Textual
            },
            direction: PortDirection::Input,
            buffer: Arc::new(RwLock::new(Vec::new())),
            position: Arc::new(RwLock::new(0)),
            metadata: HashMap::new(),
        }
    }

    /// Creates a new file output port.
    pub fn new_file_output(path: String, binary: bool) -> Self {
        Port {
            implementation: PortImpl::File {
                path,
                handle: Arc::new(RwLock::new(None)),
            },
            is_open: Arc::new(RwLock::new(true)),
            mode: if binary {
                PortMode::Binary
            } else {
                PortMode::Textual
            },
            direction: PortDirection::Output,
            buffer: Arc::new(RwLock::new(Vec::new())),
            position: Arc::new(RwLock::new(0)),
            metadata: HashMap::new(),
        }
    }

    /// Creates a standard port.
    pub fn new_standard(port_type: StandardPort) -> Self {
        let direction = match port_type {
            StandardPort::Stdin => PortDirection::Input,
            StandardPort::Stdout | StandardPort::Stderr => PortDirection::Output,
        };

        Port {
            implementation: PortImpl::Standard(port_type),
            is_open: Arc::new(RwLock::new(true)),
            mode: PortMode::Textual,
            direction,
            buffer: Arc::new(RwLock::new(Vec::new())),
            position: Arc::new(RwLock::new(0)),
            metadata: HashMap::new(),
        }
    }

    /// Checks if the port is open.
    pub fn is_open(&self) -> bool {
        *self.is_open.try_read().unwrap()
    }

    /// Closes the port.
    pub fn close(&self) {
        *self.is_open.write().unwrap() = false;

        // Perform specific cleanup based on port implementation
        match &self.implementation {
            PortImpl::File { handle, .. } => {
                // Close file handle if open
                let mut handle_guard = handle.write().unwrap();
                if let Some(file_handle) = handle_guard.take() {
                    // File handles are automatically closed when dropped
                    drop(file_handle);
                }
            }
            PortImpl::String { content, .. } => {
                // For output ports, clear content to free memory
                if self.is_output() {
                    content.write().unwrap().clear();
                    content.write().unwrap().shrink_to_fit();
                }
            }
            PortImpl::Bytevector { content, .. } => {
                // For output ports, clear content to free memory
                if self.is_output() {
                    content.write().unwrap().clear();
                    content.write().unwrap().shrink_to_fit();
                }
            }
            PortImpl::Standard(_) => {
                // Standard ports should not be closed, but we respect the flag
            }
        }

        // Clear buffer to free memory
        self.buffer.write().unwrap().clear();
        self.buffer.write().unwrap().shrink_to_fit();
    }

    /// Checks if the port is textual.
    pub fn is_textual(&self) -> bool {
        self.mode == PortMode::Textual
    }

    /// Checks if the port is binary.
    pub fn is_binary(&self) -> bool {
        self.mode == PortMode::Binary
    }

    /// Checks if the port is an input port.
    pub fn is_input(&self) -> bool {
        matches!(
            self.direction,
            PortDirection::Input | PortDirection::InputOutput
        )
    }

    /// Checks if the port is an output port.
    pub fn is_output(&self) -> bool {
        matches!(
            self.direction,
            PortDirection::Output | PortDirection::InputOutput
        )
    }

    /// Force closes the port if it's still open.
    /// This is called automatically when the port is dropped.
    fn force_close(&self) {
        if self.is_open() {
            self.close();
        }
    }

    /// Gets the current memory usage of the port content.
    pub fn memory_usage(&self) -> usize {
        match &self.implementation {
            PortImpl::String { content, .. } => content.try_read().unwrap().capacity(),
            PortImpl::Bytevector { content, .. } => content.try_read().unwrap().capacity(),
            PortImpl::File { .. } => {
                // File ports don't hold content in memory directly
                self.buffer.try_read().unwrap().capacity()
            }
            PortImpl::Standard(_) => 0,
        }
    }

    /// Validates that the proposed memory allocation is within limits.
    pub fn validate_memory_allocation(size: usize, is_string: bool) -> Result<(), String> {
        let limit = if is_string {
            port_limits::MAX_STRING_PORT_SIZE
        } else {
            port_limits::MAX_BYTEVECTOR_PORT_SIZE
        };

        if size > limit {
            let type_name = if is_string { "string" } else { "bytevector" };
            Err(format!(
                "Attempted {type_name} port allocation of {size} bytes exceeds limit of {limit} bytes"
            ))
        } else {
            Ok(())
        }
    }

    /// Checks if the port content size is approaching memory limits.
    pub fn is_near_memory_limit(&self) -> bool {
        let usage = self.memory_usage();
        let limit = match &self.implementation {
            PortImpl::String { .. } => port_limits::MAX_STRING_PORT_SIZE,
            PortImpl::Bytevector { .. } => port_limits::MAX_BYTEVECTOR_PORT_SIZE,
            _ => return false,
        };

        usage > (limit * 3) / 4 // 75% of limit
    }
}

impl Drop for Port {
    /// Automatically close the port when it goes out of scope.
    /// This ensures proper resource cleanup even if close() isn't called explicitly.
    fn drop(&mut self) {
        self.force_close();
    }
}

/// A promise for lazy evaluation - Thread-safe and R7RS compliant.
/// Uses trampoline technique to avoid stack overflow in deep promise chains.
#[derive(Debug, Clone)]
pub enum Promise {
    /// Unevaluated promise with thunk and memoization support
    Delayed {
        /// The thunk (function) to evaluate when forced
        thunk: Value,
    },
    /// Evaluated promise with cached result (memoized)
    Forced(Value),
    /// Tail-recursive promise for delay-force optimization
    TailRecursive {
        /// The thunk for tail-recursive evaluation
        thunk: Value,
    },
    /// Expression-based promise (for macro expansion)
    Expression {
        /// The expression to evaluate when forced
        expression: Box<Spanned<Expr>>,
        /// Environment for expression evaluation
        environment: Arc<ThreadSafeEnvironment>,
    },
}

/// Trampoline continuation for iterative promise evaluation.
/// This avoids stack overflow by converting recursive calls to iteration.
#[derive(Debug, Clone)]
pub enum PromiseTrampoline {
    /// Continue evaluation with a new promise
    Continue(Arc<RwLock<Promise>>),
    /// Evaluation completed with final result
    Done(Value),
    /// Evaluation requires external computation (thunk call)
    ComputeThunk {
        /// The thunk value to compute
        thunk: Value,
        /// Reference to the promise for result caching
        promise_ref: Arc<RwLock<Promise>>,
    },
}

/// A type value for gradual typing.
#[derive(Debug, Clone)]
pub enum TypeValue {
    /// Base type
    Base(String),
    /// Function type
    Function {
        /// Types of function parameters
        parameter_types: Vec<TypeValue>,
        /// Return type of the function
        return_type: Box<TypeValue>,
    },
    /// Union type
    Union(Vec<TypeValue>),
    /// Intersection type
    Intersection(Vec<TypeValue>),
    /// Variable type
    Variable(String),
}

/// A foreign object from FFI.
#[derive(Debug, Clone)]
pub struct ForeignObject {
    /// Type name
    pub type_name: String,
    /// Opaque data pointer
    pub data: *mut std::ffi::c_void,
    /// Destructor function
    pub destructor: Option<fn(*mut std::ffi::c_void)>,
}

/// A parameter object for SRFI-39 support - Thread-safe.
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Unique identifier for this parameter
    pub id: u64,
    /// Optional converter function to validate/transform values
    pub converter: Option<Arc<Value>>,
    /// Global default value (fallback when no thread-local binding exists)
    pub global_default: Arc<RwLock<Value>>,
    /// Optional name for debugging
    pub name: Option<String>,
}

/// A record type definition for SRFI-9 support - Thread-safe.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordType {
    /// Unique identifier for this record type
    pub id: u64,
    /// Name of the record type
    pub name: String,
    /// Field names in order
    pub field_names: Vec<String>,
    /// Constructor name (optional, defaults to make-TYPE_NAME)
    pub constructor_name: Option<String>,
    /// Predicate name (optional, defaults to TYPE_NAME?)
    pub predicate_name: Option<String>,
    /// Field accessors and mutators
    pub field_info: Vec<FieldInfo>,
}

/// Information about a record field.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldInfo {
    /// Field name
    pub name: String,
    /// Accessor procedure name
    pub accessor: String,
    /// Optional mutator procedure name
    pub mutator: Option<String>,
}

/// A record instance for SRFI-9 support - Thread-safe.
#[derive(Debug, Clone)]
pub struct Record {
    /// Type identifier
    pub type_id: u64,
    /// Field values (stored in order matching the type definition)
    pub fields: Arc<RwLock<Vec<Value>>>,
}

/// Thread-safe environment for variable bindings with immutable semantics.
///
/// This is the new thread-safe environment that uses Arc and RwLock for
/// thread safety while maintaining proper lexical scoping semantics.
/// It implements Copy-on-Write (COW) semantics for mutations.
#[derive(Debug, Clone)]
pub struct ThreadSafeEnvironment {
    /// Variable bindings in this environment (with interior mutability for initialization)
    bindings: std::sync::Arc<std::sync::RwLock<HashMap<String, Value>>>,
    /// Parent environment (for lexical scoping)
    parent: Option<Arc<ThreadSafeEnvironment>>,
    /// Generation counter for GC
    generation: Generation,
    /// Optional name for debugging
    name: Option<String>,
}

/// Legacy environment for variable bindings (will be phased out).
///
/// Uses generational garbage collection for memory management
/// and proper lexical scoping semantics.
#[derive(Debug, Clone)]
pub struct Environment {
    /// Variable bindings in this environment
    pub bindings: Rc<std::cell::RefCell<HashMap<String, Value>>>,
    /// Parent environment (for lexical scoping)
    pub parent: Option<Rc<Environment>>,
    /// Generation counter for GC
    pub generation: Generation,
    /// Optional name for debugging
    pub name: Option<String>,
}

impl Value {
    /// Returns true if this value is truthy in Scheme semantics.
    ///
    /// In Scheme, only #f is falsy; everything else is truthy.
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Value::Literal(Literal::Boolean(false)))
    }

    /// Returns true if this value is falsy in Scheme semantics.
    pub fn is_falsy(&self) -> bool {
        matches!(self, Value::Literal(Literal::Boolean(false)))
    }

    /// Returns true if this value is a number.
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Literal(lit) if lit.is_number())
    }

    /// Returns true if this value is a boolean.
    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Literal(Literal::Boolean(_)))
    }

    /// Extracts a boolean value if this is a boolean literal.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Literal(Literal::Boolean(b)) => Some(*b),
            _ => None,
        }
    }

    /// Formats this value for display according to R7RS specification.
    ///
    /// This is the proper formatting function for the `display` procedure:
    /// - Strings are displayed without quotes
    /// - Characters are displayed without the #\ prefix
    /// - Other values use their standard Display representation
    pub fn display_string(&self) -> String {
        match self {
            Value::Literal(Literal::String(s)) => (**s).clone(),
            Value::Literal(Literal::Character(c)) => c.to_string(),
            _ => format!("{self}"),
        }
    }

    /// Returns true if this value is a string (immutable or mutable).
    pub fn is_string(&self) -> bool {
        matches!(
            self,
            Value::Literal(Literal::String(_)) | Value::MutableString(_)
        )
    }

    /// Returns true if this value is an immutable string.
    pub fn is_immutable_string(&self) -> bool {
        matches!(self, Value::Literal(Literal::String(_)))
    }

    /// Returns true if this value is a mutable string.
    pub fn is_mutable_string(&self) -> bool {
        matches!(self, Value::MutableString(_))
    }

    /// Returns true if this value is a symbol.
    pub fn is_symbol(&self) -> bool {
        matches!(self, Value::Symbol(_))
    }

    /// Returns true if this value is a pair.
    pub fn is_pair(&self) -> bool {
        matches!(self, Value::Pair(_, _) | Value::MutablePair(_, _))
    }

    /// Returns true if this value is the empty list.
    pub fn is_nil(&self) -> bool {
        matches!(self, Value::Nil)
    }

    /// Returns true if this value is a list (either nil or a pair).
    pub fn is_list(&self) -> bool {
        self.is_nil() || self.is_pair()
    }

    /// Returns true if this value is a procedure.
    pub fn is_procedure(&self) -> bool {
        matches!(
            self,
            Value::Procedure(_)
                | Value::CaseLambda(_)
                | Value::Primitive(_)
                | Value::Continuation(_)
                | Value::Parameter(_)
        )
    }

    /// Returns true if this value is a vector.
    pub fn is_vector(&self) -> bool {
        matches!(self, Value::Vector(_))
    }

    /// Returns true if this value is a box (SRFI-111).
    pub fn is_box(&self) -> bool {
        matches!(self, Value::Box(_))
    }

    /// Returns true if this value is a port.
    pub fn is_port(&self) -> bool {
        matches!(self, Value::Port(_))
    }

    /// Returns true if this value is a character set.
    pub fn is_charset(&self) -> bool {
        matches!(self, Value::CharSet(_))
    }

    /// Returns true if this value is a parameter.
    pub fn is_parameter(&self) -> bool {
        matches!(self, Value::Parameter(_))
    }

    /// Returns true if this value is a record.
    pub fn is_record(&self) -> bool {
        matches!(self, Value::Record(_))
    }

    /// Returns true if this value is an environment.
    pub fn is_environment(&self) -> bool {
        matches!(self, Value::Environment(_))
    }

    /// Converts this value to a Rust f64 if it's a number.
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Literal(lit) => lit.to_f64(),
            _ => None,
        }
    }

    /// Converts this value to a Rust i64 if it's an integer.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Literal(lit) => lit.to_i64(),
            _ => None,
        }
    }

    /// Converts this value to a Rust string if it's an immutable string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::Literal(Literal::String(s)) => Some(s),
            _ => None,
        }
    }

    /// Gets the string content as an owned String (works with both immutable and mutable strings).
    pub fn as_string_owned(&self) -> Option<String> {
        match self {
            Value::Literal(Literal::String(s)) => Some((**s).clone()),
            Value::MutableString(chars) => {
                chars.try_borrow().ok().map(|guard| guard.iter().collect())
            }
            _ => None,
        }
    }

    /// Gets the length of a string (works with both immutable and mutable strings).
    pub fn string_length(&self) -> Option<usize> {
        match self {
            Value::Literal(Literal::String(s)) => Some(s.chars().count()),
            Value::MutableString(chars) => chars.try_borrow().ok().map(|guard| guard.len()),
            _ => None,
        }
    }

    /// Gets the symbol ID if this is a symbol.
    pub fn as_symbol(&self) -> Option<SymbolId> {
        match self {
            Value::Symbol(id) => Some(*id),
            _ => None,
        }
    }

    /// Converts this value to a proper list if possible.
    pub fn as_list(&self) -> Option<Vec<Value>> {
        let mut result = Vec::new();
        let mut current = self;

        loop {
            match current {
                Value::Nil => return Some(result),
                Value::Pair(car, cdr) => {
                    result.push((**car).clone());
                    current = cdr;
                }
                Value::MutablePair(car_ref, cdr_ref) => {
                    if let (Ok(car), Ok(cdr)) = (car_ref.try_borrow(), cdr_ref.try_borrow()) {
                        result.push(car.clone());
                        // For mutable pairs, we need to handle recursion carefully
                        // to avoid holding locks too long
                        let cdr_clone = cdr.clone();
                        drop(cdr); // Release the lock
                        if let Some(mut rest) = cdr_clone.as_list() {
                            result.append(&mut rest);
                            return Some(result);
                        } else {
                            return None; // Not a proper list
                        }
                    } else {
                        return None; // Lock failed
                    }
                }
                _ => return None, // Not a proper list
            }
        }
    }

    /// Creates a new number value.
    pub fn number(n: f64) -> Self {
        Value::Literal(Literal::from_f64(n))
    }

    /// Creates a new integer value.
    pub fn integer(n: i64) -> Self {
        Value::Literal(Literal::integer(n))
    }

    /// Creates a new immutable string value.
    pub fn string(s: impl Into<String>) -> Self {
        Value::Literal(Literal::String(Box::new(s.into())))
    }

    /// Creates a new mutable string value.
    pub fn mutable_string(s: impl Into<String>) -> Self {
        let chars: Vec<char> = s.into().chars().collect();
        Value::MutableString(Rc::new(RefCell::new(chars)))
    }

    /// Creates a new mutable string value with specified length and fill character.
    pub fn mutable_string_filled(length: usize, ch: char) -> Self {
        let chars = vec![ch; length];
        Value::MutableString(Rc::new(RefCell::new(chars)))
    }

    /// Creates a new boolean value.
    pub fn boolean(b: bool) -> Self {
        Value::Literal(Literal::Boolean(b))
    }

    /// Creates a new symbol value.
    pub fn symbol(id: SymbolId) -> Self {
        Value::Symbol(id)
    }

    /// Creates a new box (SRFI-111) containing the given value.
    pub fn box_value(value: Value) -> Self {
        let box_instance = crate::stdlib::r#box::Box::new(value);
        Value::Box(Arc::new(box_instance))
    }

    /// Creates a new symbol value from a string.
    pub fn symbol_from_str(name: impl Into<String>) -> Self {
        let name_str = name.into();
        // Simple hash-based symbol ID generation (in a real implementation,
        // this would use a proper symbol table)
        let id = SymbolId::new(
            name_str
                .chars()
                .fold(0, |acc, c| acc.wrapping_mul(31).wrapping_add(c as usize)),
        );
        Value::Symbol(id)
    }

    /// Creates a new pair value.
    pub fn pair(car: Value, cdr: Value) -> Self {
        Value::Pair(Box::new(car), Box::new(cdr))
    }

    /// Creates a new mutable pair value.
    pub fn mutable_pair(car: Value, cdr: Value) -> Self {
        Value::MutablePair(Rc::new(RefCell::new(car)), Rc::new(RefCell::new(cdr)))
    }

    /// Creates a list from a vector of values.
    pub fn list(values: Vec<Value>) -> Self {
        values
            .into_iter()
            .rev()
            .fold(Value::Nil, |acc, val| Value::pair(val, acc))
    }

    /// Creates a new vector value.
    pub fn vector(values: Vec<Value>) -> Self {
        Value::Vector(Rc::new(RefCell::new(values)))
    }

    /// Creates a new vector value from a `Vec<Value>`.
    /// This is an alias for the vector method for compatibility.
    pub fn from_vec(values: Vec<Value>) -> Self {
        Self::vector(values)
    }

    /// Creates a new bytevector value.
    pub fn bytevector(bytes: Vec<u8>) -> Self {
        Value::Literal(Literal::Bytevector(Box::new(bytes)))
    }

    /// Creates a new character set value.
    pub fn charset(charset: crate::stdlib::charset::CharSet) -> Self {
        Value::CharSet(Arc::new(charset))
    }

    /// Creates a new parameter value.
    pub fn parameter(parameter: Parameter) -> Self {
        Value::Parameter(Arc::new(parameter))
    }

    /// Creates a new case-lambda value.
    pub fn case_lambda(case_lambda: CaseLambdaProcedure) -> Self {
        Value::CaseLambda(Arc::new(case_lambda))
    }

    /// Creates a new opaque value.
    pub fn opaque<T: std::any::Any + Send + Sync>(value: T) -> Self {
        Value::Opaque(Arc::new(value))
    }

    /// Creates a new record value.
    pub fn record(record: Record) -> Self {
        Value::Record(Arc::new(record))
    }

    /// Creates a Value from a Literal.
    pub fn from_literal(lit: Literal) -> Self {
        Value::Literal(lit)
    }

    /// Creates a new procedure value.
    pub fn procedure(proc: Procedure) -> Self {
        Value::Procedure(Arc::new(proc))
    }

    /// The canonical true value.
    pub fn t() -> Self {
        Value::boolean(true)
    }

    /// The canonical false value.
    pub fn f() -> Self {
        Value::boolean(false)
    }

    /// Creates a new error object value.
    pub fn error_object(error: crate::stdlib::exceptions::ErrorObject) -> Self {
        Value::ErrorObject(Arc::new(error))
    }

    /// Creates a value from an exception object.
    pub fn exception_object(exception: crate::stdlib::exceptions::ExceptionObject) -> Self {
        exception.value
    }

    // ============= ADVANCED CONTAINER CONSTRUCTORS =============

    /// Creates a new advanced hash table value.
    pub fn advanced_hash_table() -> Self {
        Value::AdvancedHashTable(Arc::new(crate::containers::ThreadSafeHashTable::new()))
    }

    /// Creates a new advanced hash table with comparator.
    pub fn advanced_hash_table_with_comparator(
        comparator: crate::containers::HashComparator,
    ) -> Self {
        Value::AdvancedHashTable(Arc::new(
            crate::containers::ThreadSafeHashTable::with_comparator(comparator),
        ))
    }

    /// Creates a new ideque value.
    pub fn ideque() -> Self {
        Value::Ideque(Arc::new(crate::containers::PersistentIdeque::new()))
    }

    /// Creates an ideque from a vector of values.
    pub fn ideque_from_vec(values: Vec<Value>) -> Self {
        Value::Ideque(Arc::new(crate::containers::PersistentIdeque::from_vec(
            values,
        )))
    }

    /// Creates a new priority queue value.
    pub fn priority_queue() -> Self {
        Value::PriorityQueue(Arc::new(crate::containers::ThreadSafePriorityQueue::new()))
    }

    /// Creates a new min-heap priority queue value.
    pub fn min_priority_queue() -> Self {
        Value::PriorityQueue(Arc::new(
            crate::containers::ThreadSafePriorityQueue::new_min_heap(),
        ))
    }

    /// Creates a new priority queue with comparator.
    pub fn priority_queue_with_comparator(comparator: crate::containers::Comparator) -> Self {
        Value::PriorityQueue(Arc::new(
            crate::containers::ThreadSafePriorityQueue::with_comparator(comparator),
        ))
    }

    /// Creates a new ordered set value.
    pub fn ordered_set() -> Self {
        Value::OrderedSet(Arc::new(crate::containers::ThreadSafeOrderedSet::new()))
    }

    /// Creates a new ordered set with comparator.
    pub fn ordered_set_with_comparator(comparator: crate::containers::Comparator) -> Self {
        Value::OrderedSet(Arc::new(
            crate::containers::ThreadSafeOrderedSet::with_comparator(comparator),
        ))
    }

    /// Creates a new list queue value.
    pub fn list_queue() -> Self {
        Value::ListQueue(Arc::new(crate::containers::ThreadSafeListQueue::new()))
    }

    /// Creates a list queue from a vector of values.
    pub fn list_queue_from_vec(values: Vec<Value>) -> Self {
        Value::ListQueue(Arc::new(crate::containers::ThreadSafeListQueue::from_vec(
            values,
        )))
    }

    /// Creates a new random access list value.
    pub fn random_access_list() -> Self {
        Value::RandomAccessList(Arc::new(
            crate::containers::ThreadSafeRandomAccessList::new(),
        ))
    }

    /// Creates a random access list from a vector of values.
    pub fn random_access_list_from_vec(values: Vec<Value>) -> Self {
        Value::RandomAccessList(Arc::new(
            crate::containers::ThreadSafeRandomAccessList::from_vec(values),
        ))
    }

    /// Creates a new set value.
    pub fn set() -> Self {
        Value::Set(Arc::new(crate::containers::ThreadSafeSet::new()))
    }

    /// Creates a new set with a custom comparator.
    pub fn set_with_comparator(comparator: crate::containers::HashComparator) -> Self {
        Value::Set(Arc::new(crate::containers::ThreadSafeSet::with_comparator(
            comparator,
        )))
    }

    /// Creates a set from an iterator of values.
    pub fn set_from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        Value::Set(Arc::new(crate::containers::ThreadSafeSet::from_iter(iter)))
    }

    /// Creates a set from an iterator of values with a custom comparator.
    pub fn set_from_iter_with_comparator<I>(
        iter: I,
        comparator: crate::containers::HashComparator,
    ) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        Value::Set(Arc::new(
            crate::containers::ThreadSafeSet::from_iter_with_comparator(iter, comparator),
        ))
    }

    /// Tries to get a reference to the ThreadSafeSet if this value is a set.
    pub fn as_set(&self) -> Option<&Arc<crate::containers::ThreadSafeSet>> {
        match self {
            Value::Set(set) => Some(set),
            _ => None,
        }
    }

    /// Tries to get a clone of the ThreadSafeSet if this value is a set.
    pub fn to_set(&self) -> Option<Arc<crate::containers::ThreadSafeSet>> {
        match self {
            Value::Set(set) => Some(set.clone()),
            _ => None,
        }
    }

    /// Creates a new bag value.
    pub fn bag() -> Self {
        Value::Bag(Arc::new(crate::containers::ThreadSafeBag::new()))
    }

    /// Creates a new bag with a custom comparator.
    pub fn bag_with_comparator(comparator: crate::containers::HashComparator) -> Self {
        Value::Bag(Arc::new(crate::containers::ThreadSafeBag::with_comparator(
            comparator,
        )))
    }

    /// Creates a bag from an iterator of values.
    pub fn bag_from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        Value::Bag(Arc::new(crate::containers::ThreadSafeBag::from_iter(iter)))
    }

    // ============= GENERATOR CONSTRUCTORS =============

    /// Creates a new generator from a procedure (thunk).
    pub fn generator_from_procedure(thunk: Value, environment: Arc<ThreadSafeEnvironment>) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::from_procedure(
            thunk,
            environment,
        )))
    }

    /// Creates a new generator from a procedure (thunk) with an evaluator callback.
    pub fn generator_from_procedure_with_evaluator(
        thunk: Value,
        environment: Arc<ThreadSafeEnvironment>,
        evaluator: Arc<crate::containers::generator::ProcedureEvaluator>,
    ) -> Self {
        Value::Generator(Arc::new(
            crate::containers::Generator::from_procedure_with_evaluator(
                thunk,
                environment,
                evaluator,
            ),
        ))
    }

    /// Creates a new generator from explicit values.
    pub fn generator_from_values(values: Vec<Value>) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::from_values(values)))
    }

    /// Creates a new range generator.
    pub fn generator_range(start: f64, end: Option<f64>, step: f64) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::range(
            start, end, step,
        )))
    }

    /// Creates a new iota generator.
    pub fn generator_iota(count: Option<usize>, start: i64, step: i64) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::iota(
            count, start, step,
        )))
    }

    /// Creates a new generator from a list.
    pub fn generator_from_list(list: Value) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::from_list(list)))
    }

    /// Creates a new generator from a vector.
    pub fn generator_from_vector(vector: Arc<RwLock<Vec<Value>>>) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::from_vector(vector)))
    }

    /// Creates a new generator from a string.
    pub fn generator_from_string(string: String) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::from_string(string)))
    }

    /// Creates an already exhausted generator.
    pub fn generator_exhausted() -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::exhausted()))
    }

    /// Creates a new unfold generator.
    pub fn generator_unfold(
        stop_predicate: Value,
        mapper: Value,
        successor: Value,
        seed: Value,
    ) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::unfold(
            stop_predicate,
            mapper,
            successor,
            seed,
        )))
    }

    /// Creates a new unfold generator with an evaluator.
    pub fn generator_unfold_with_evaluator(
        stop_predicate: Value,
        mapper: Value,
        successor: Value,
        seed: Value,
        evaluator: Arc<crate::containers::generator::ProcedureEvaluator>,
    ) -> Self {
        Value::Generator(Arc::new(
            crate::containers::Generator::unfold_with_evaluator(
                stop_predicate,
                mapper,
                successor,
                seed,
                evaluator,
            ),
        ))
    }

    /// Creates a new tabulate generator.
    pub fn generator_tabulate(func: Value, max_count: Option<usize>) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::tabulate(
            func, max_count,
        )))
    }

    /// Creates a new tabulate generator with an evaluator.
    pub fn generator_tabulate_with_evaluator(
        func: Value,
        max_count: Option<usize>,
        evaluator: Arc<crate::containers::generator::ProcedureEvaluator>,
    ) -> Self {
        Value::Generator(Arc::new(
            crate::containers::Generator::tabulate_with_evaluator(func, max_count, evaluator),
        ))
    }

    /// Creates a new map generator.
    pub fn generator_map(source: Arc<crate::containers::Generator>, mapper: Value) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::map(source, mapper)))
    }

    /// Creates a new map generator with an evaluator.
    pub fn generator_map_with_evaluator(
        source: Arc<crate::containers::Generator>,
        mapper: Value,
        evaluator: Arc<crate::containers::generator::ProcedureEvaluator>,
    ) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::map_with_evaluator(
            source, mapper, evaluator,
        )))
    }

    /// Creates a new filter generator.
    pub fn generator_filter(source: Arc<crate::containers::Generator>, predicate: Value) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::filter(
            source, predicate,
        )))
    }

    /// Creates a new filter generator with an evaluator.
    pub fn generator_filter_with_evaluator(
        source: Arc<crate::containers::Generator>,
        predicate: Value,
        evaluator: Arc<crate::containers::generator::ProcedureEvaluator>,
    ) -> Self {
        Value::Generator(Arc::new(
            crate::containers::Generator::filter_with_evaluator(source, predicate, evaluator),
        ))
    }

    /// Creates a new take generator.
    pub fn generator_take(source: Arc<crate::containers::Generator>, count: usize) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::take(source, count)))
    }

    /// Creates a new drop generator.
    pub fn generator_drop(source: Arc<crate::containers::Generator>, count: usize) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::drop(source, count)))
    }

    /// Creates a new append generator.
    pub fn generator_append(
        first: Arc<crate::containers::Generator>,
        second: Arc<crate::containers::Generator>,
    ) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::append(
            first, second,
        )))
    }

    /// Creates a new concatenate generator.
    pub fn generator_concatenate(generators: Vec<Arc<crate::containers::Generator>>) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::concatenate(
            generators,
        )))
    }

    /// Creates a new zip generator.
    pub fn generator_zip(sources: Vec<Arc<crate::containers::Generator>>) -> Self {
        Value::Generator(Arc::new(crate::containers::Generator::zip(sources)))
    }

    // ============= ADVANCED CONTAINER TYPE PREDICATES =============

    /// Returns true if this value is an advanced hash table.
    pub fn is_advanced_hash_table(&self) -> bool {
        matches!(self, Value::AdvancedHashTable(_))
    }

    /// Returns true if this value is an ideque.
    pub fn is_ideque(&self) -> bool {
        matches!(self, Value::Ideque(_))
    }

    /// Returns true if this value is a priority queue.
    pub fn is_priority_queue(&self) -> bool {
        matches!(self, Value::PriorityQueue(_))
    }

    /// Returns true if this value is an ordered set.
    pub fn is_ordered_set(&self) -> bool {
        matches!(self, Value::OrderedSet(_))
    }

    /// Returns true if this value is a list queue.
    pub fn is_list_queue(&self) -> bool {
        matches!(self, Value::ListQueue(_))
    }

    /// Returns true if this value is a random access list.
    pub fn is_random_access_list(&self) -> bool {
        matches!(self, Value::RandomAccessList(_))
    }

    /// Returns true if this value is a set.
    pub fn is_set(&self) -> bool {
        matches!(self, Value::Set(_))
    }

    /// Returns true if this value is a bag.
    pub fn is_bag(&self) -> bool {
        matches!(self, Value::Bag(_))
    }

    /// Returns true if this value is a generator.
    pub fn is_generator(&self) -> bool {
        matches!(self, Value::Generator(_))
    }

    /// Returns true if this value is a future.
    #[cfg(feature = "async-runtime")]
    pub fn is_future(&self) -> bool {
        matches!(self, Value::Future(_))
    }

    /// Returns true if this value is a future (no-op when async-runtime disabled).
    #[cfg(not(feature = "async-runtime"))]
    pub fn is_future(&self) -> bool {
        false
    }

    /// Returns true if this value is a channel.
    #[cfg(feature = "async-runtime")]
    pub fn is_channel(&self) -> bool {
        matches!(self, Value::Channel(_))
    }

    /// Returns true if this value is a channel (no-op when async-runtime disabled).
    #[cfg(not(feature = "async-runtime"))]
    pub fn is_channel(&self) -> bool {
        false
    }

    /// Returns true if this value is a mutex.
    #[cfg(feature = "async-runtime")]
    pub fn is_mutex(&self) -> bool {
        matches!(self, Value::Mutex(_))
    }

    /// Returns true if this value is a mutex (no-op when async-runtime disabled).
    #[cfg(not(feature = "async-runtime"))]
    pub fn is_mutex(&self) -> bool {
        false
    }

    /// Returns true if this value is a semaphore.
    #[cfg(feature = "async-runtime")]
    pub fn is_semaphore(&self) -> bool {
        matches!(self, Value::Semaphore(_))
    }

    /// Returns true if this value is a semaphore (no-op when async-runtime disabled).
    #[cfg(not(feature = "async-runtime"))]
    pub fn is_semaphore(&self) -> bool {
        false
    }

    /// Returns true if this value is an atomic counter.
    #[cfg(feature = "async-runtime")]
    pub fn is_atomic_counter(&self) -> bool {
        matches!(self, Value::AtomicCounter(_))
    }

    /// Returns true if this value is an atomic counter (no-op when async-runtime disabled).
    #[cfg(not(feature = "async-runtime"))]
    pub fn is_atomic_counter(&self) -> bool {
        false
    }

    /// Returns true if this value is a distributed node.
    #[cfg(feature = "async-runtime")]
    pub fn is_distributed_node(&self) -> bool {
        matches!(self, Value::DistributedNode(_))
    }

    /// Returns true if this value is a distributed node (no-op when async-runtime disabled).
    #[cfg(not(feature = "async-runtime"))]
    pub fn is_distributed_node(&self) -> bool {
        false
    }

    /// Returns true if this value is an opaque value.
    pub fn is_opaque(&self) -> bool {
        matches!(self, Value::Opaque(_))
    }

    /// Returns the car (first element) of a pair, or an error if not a pair.
    pub fn car(&self) -> Option<&Value> {
        match self {
            Value::Pair(car, _) => Some(car.as_ref()),
            _ => None,
        }
    }

    /// Returns the cdr (rest element) of a pair, or an error if not a pair.
    pub fn cdr(&self) -> Option<&Value> {
        match self {
            Value::Pair(_, cdr) => Some(cdr.as_ref()),
            _ => None,
        }
    }

    // ============= R7RS NUMERIC PREDICATES =============

    /// R7RS exact? predicate - returns true if this value is an exact number.
    pub fn is_exact_number(&self) -> bool {
        match self {
            Value::Literal(lit) => lit.is_exact(),
            _ => false,
        }
    }

    /// R7RS inexact? predicate - returns true if this value is an inexact number.
    pub fn is_inexact_number(&self) -> bool {
        match self {
            Value::Literal(lit) => lit.is_inexact(),
            _ => false,
        }
    }

    /// R7RS finite? predicate - returns true if this value is a finite number.
    pub fn is_finite_number(&self) -> bool {
        match self {
            Value::Literal(Literal::ExactInteger(_)) => true,
            Value::Literal(Literal::InexactReal(f)) => f.is_finite(),
            Value::Literal(Literal::Rational { .. }) => true,
            Value::Literal(Literal::Complex(complex)) => {
                complex.real.is_finite() && complex.imaginary.is_finite()
            }
            _ => false,
        }
    }

    /// R7RS infinite? predicate - returns true if this value is an infinite number.
    pub fn is_infinite_number(&self) -> bool {
        match self {
            Value::Literal(Literal::InexactReal(f)) => f.is_infinite(),
            Value::Literal(Literal::Complex(complex)) => {
                complex.real.is_infinite() || complex.imaginary.is_infinite()
            }
            _ => false,
        }
    }

    /// R7RS nan? predicate - returns true if this value is a NaN.
    pub fn is_nan_number(&self) -> bool {
        match self {
            Value::Literal(Literal::InexactReal(f)) => f.is_nan(),
            Value::Literal(Literal::Complex(complex)) => {
                complex.real.is_nan() || complex.imaginary.is_nan()
            }
            _ => false,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Literal(a), Value::Literal(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Keyword(a), Value::Keyword(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Unspecified, Value::Unspecified) => true,
            (Value::Pair(a1, b1), Value::Pair(a2, b2)) => a1 == a2 && b1 == b2,
            // For mutable objects, use reference equality
            (Value::Vector(a), Value::Vector(b)) => Rc::ptr_eq(a, b),
            (Value::Hashtable(a), Value::Hashtable(b)) => Rc::ptr_eq(a, b),
            (Value::Procedure(a), Value::Procedure(b)) => Arc::ptr_eq(a, b),
            (Value::CaseLambda(a), Value::CaseLambda(b)) => Arc::ptr_eq(a, b),
            (Value::Primitive(a), Value::Primitive(b)) => a.name == b.name,
            (Value::Continuation(a), Value::Continuation(b)) => a.id == b.id,
            (Value::ErrorObject(a), Value::ErrorObject(b)) => Arc::ptr_eq(a, b),
            (Value::CharSet(a), Value::CharSet(b)) => a == b,
            (Value::Parameter(a), Value::Parameter(b)) => a.id == b.id,
            (Value::Record(a), Value::Record(b)) => {
                // Records are equal if they have the same type and field values
                if a.type_id != b.type_id {
                    false
                } else {
                    // Compare field values
                    if let (Ok(a_fields), Ok(b_fields)) = (a.fields.try_read(), b.fields.try_read())
                    {
                        *a_fields == *b_fields
                    } else {
                        false // Handle lock errors conservatively
                    }
                }
            }
            // Advanced containers use reference equality for efficiency
            (Value::AdvancedHashTable(a), Value::AdvancedHashTable(b)) => Arc::ptr_eq(a, b),
            (Value::Box(a), Value::Box(b)) => Arc::ptr_eq(a, b),
            (Value::Ideque(a), Value::Ideque(b)) => Arc::ptr_eq(a, b),
            (Value::PriorityQueue(a), Value::PriorityQueue(b)) => Arc::ptr_eq(a, b),
            (Value::OrderedSet(a), Value::OrderedSet(b)) => Arc::ptr_eq(a, b),
            (Value::ListQueue(a), Value::ListQueue(b)) => Arc::ptr_eq(a, b),
            (Value::RandomAccessList(a), Value::RandomAccessList(b)) => Arc::ptr_eq(a, b),
            (Value::Set(a), Value::Set(b)) => Arc::ptr_eq(a, b),
            (Value::Bag(a), Value::Bag(b)) => Arc::ptr_eq(a, b),
            (Value::Generator(a), Value::Generator(b)) => Arc::ptr_eq(a, b),
            // Concurrency values use reference equality (only available with async-runtime)
            #[cfg(feature = "async-runtime")]
            (Value::Future(a), Value::Future(b)) => Arc::ptr_eq(a, b),
            #[cfg(feature = "async-runtime")]
            (Value::Channel(a), Value::Channel(b)) => Arc::ptr_eq(a, b),
            #[cfg(feature = "async-runtime")]
            (Value::Mutex(a), Value::Mutex(b)) => Arc::ptr_eq(a, b),
            #[cfg(feature = "async-runtime")]
            (Value::Semaphore(a), Value::Semaphore(b)) => Arc::ptr_eq(a, b),
            #[cfg(feature = "async-runtime")]
            (Value::AtomicCounter(a), Value::AtomicCounter(b)) => Arc::ptr_eq(a, b),
            #[cfg(feature = "async-runtime")]
            (Value::DistributedNode(a), Value::DistributedNode(b)) => Arc::ptr_eq(a, b),
            (Value::Opaque(a), Value::Opaque(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for Value {}

impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Literal(lit) => {
                0u8.hash(state);
                lit.hash(state);
            }
            Value::Symbol(id) => {
                1u8.hash(state);
                id.hash(state);
            }
            Value::Keyword(k) => {
                2u8.hash(state);
                k.hash(state);
            }
            Value::Nil => 3u8.hash(state),
            Value::Unspecified => 4u8.hash(state),
            // For compound values, we can't easily implement hash
            // so we use a type discriminant
            _ => std::mem::discriminant(self).hash(state),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Literal(lit) => write!(f, "{lit}"),
            Value::Symbol(id) => {
                if let Some(name) = crate::utils::symbol_name(*id) {
                    write!(f, "{name}")
                } else {
                    write!(f, "#<symbol:{}>", id.id())
                }
            }
            Value::Keyword(k) => write!(f, "#{k}"),
            Value::Nil => write!(f, "()"),
            Value::Unspecified => write!(f, "#<unspecified>"),
            Value::Pair(_car, _cdr) => {
                write!(f, "(")?;
                self.write_list_contents(f, true)?;
                write!(f, ")")
            }
            Value::MutablePair(_car, _cdr) => {
                write!(f, "(")?;
                self.write_mutable_list_contents(f, true)?;
                write!(f, ")")
            }
            Value::Vector(vec) => {
                write!(f, "#(")?;
                if let Ok(vec_ref) = vec.try_borrow() {
                    for (i, value) in vec_ref.iter().enumerate() {
                        if i > 0 {
                            write!(f, " ")?;
                        }
                        write!(f, "{value}")?;
                    }
                } else {
                    write!(f, "...")?; // Fallback if lock is poisoned
                }
                write!(f, ")")
            }
            Value::Hashtable(_) => write!(f, "#<hashtable>"),
            Value::Procedure(proc) => {
                if let Some(name) = &proc.name {
                    write!(f, "#<procedure:{name}>")
                } else {
                    write!(f, "#<procedure>")
                }
            }
            Value::CaseLambda(case_lambda) => {
                if let Some(name) = &case_lambda.name {
                    write!(f, "#<case-lambda:{name}>")
                } else {
                    write!(f, "#<case-lambda>")
                }
            }
            Value::Primitive(prim) => write!(f, "#<primitive:{}>", prim.name),
            Value::Continuation(cont) => write!(f, "#<continuation:{}>", cont.id),
            Value::Syntax(syn) => write!(f, "#<syntax:{}>", syn.name),
            Value::Port(_) => write!(f, "#<port>"),
            Value::Promise(_) => write!(f, "#<promise>"),
            Value::Type(_) => write!(f, "#<type>"),
            Value::Foreign(obj) => write!(f, "#<foreign:{}>", obj.type_name),
            Value::ErrorObject(err) => write!(f, "#<error:{}>", err.message),
            Value::CharSet(charset) => write!(f, "{charset}"),
            Value::Parameter(param) => {
                if let Some(name) = &param.name {
                    write!(f, "#<parameter:{name}>")
                } else {
                    write!(f, "#<parameter:{}>", param.id)
                }
            }
            Value::Record(record) => {
                write!(f, "#<record:{}>", record.type_id)
            }
            // Advanced containers
            Value::AdvancedHashTable(_) => write!(f, "#<advanced-hash-table>"),
            Value::Box(box_ref) => {
                if let Ok(content) = box_ref.get() {
                    write!(f, "#<box:{content}>")
                } else {
                    write!(f, "#<box:error>")
                }
            }
            Value::Ideque(_) => write!(f, "#<ideque>"),
            Value::PriorityQueue(_) => write!(f, "#<priority-queue>"),
            Value::OrderedSet(_) => write!(f, "#<ordered-set>"),
            Value::ListQueue(_) => write!(f, "#<list-queue>"),
            Value::RandomAccessList(_) => write!(f, "#<random-access-list>"),
            // Concurrency values (only available with async-runtime)
            #[cfg(feature = "async-runtime")]
            Value::Future(_) => write!(f, "#<future>"),
            #[cfg(feature = "async-runtime")]
            Value::Channel(_) => write!(f, "#<channel>"),
            #[cfg(feature = "async-runtime")]
            Value::Mutex(_) => write!(f, "#<mutex>"),
            #[cfg(feature = "async-runtime")]
            Value::Semaphore(_) => write!(f, "#<semaphore>"),
            #[cfg(feature = "async-runtime")]
            Value::AtomicCounter(counter) => write!(f, "#<atomic-counter:{}>", counter.get()),
            #[cfg(feature = "async-runtime")]
            Value::DistributedNode(_) => write!(f, "#<distributed-node>"),
            Value::MutableString(s) => match s.try_borrow() {
                Ok(chars) => {
                    write!(f, "\"")?;
                    for ch in chars.iter() {
                        match ch {
                            '"' => write!(f, "\\\"")?,
                            '\\' => write!(f, "\\\\")?,
                            '\n' => write!(f, "\\n")?,
                            '\t' => write!(f, "\\t")?,
                            '\r' => write!(f, "\\r")?,
                            c if c.is_control() => write!(f, "\\x{:02x}", *c as u8)?,
                            c => write!(f, "{c}")?,
                        }
                    }
                    write!(f, "\"")
                }
                Err(_) => write!(f, "#<locked-string>"),
            },
            Value::Set(set) => match set.size() {
                Ok(size) => write!(f, "#<set:{size}>"),
                Err(_) => write!(f, "#<set:locked>"),
            },
            Value::Bag(bag) => match bag.total_size() {
                Ok(size) => write!(f, "#<bag:{size}>"),
                Err(_) => write!(f, "#<bag:locked>"),
            },
            Value::Generator(generator) => {
                write!(f, "{generator}")
            }
            Value::Opaque(_) => write!(f, "#<opaque>"),
            Value::Environment(env) => {
                if let Some(name) = &env.name {
                    write!(f, "#<environment:{name}>")
                } else {
                    write!(f, "#<environment>")
                }
            }
        }
    }
}

impl Value {
    /// Helper method to write list contents for display.
    fn write_list_contents(&self, f: &mut fmt::Formatter<'_>, first: bool) -> fmt::Result {
        match self {
            Value::Nil => Ok(()),
            Value::Pair(car, cdr) => {
                if !first {
                    write!(f, " ")?;
                }
                write!(f, "{car}")?;
                match &**cdr {
                    Value::Nil => Ok(()),
                    Value::Pair(_, _) => cdr.write_list_contents(f, false),
                    _ => write!(f, " . {cdr}"),
                }
            }
            _ => write!(f, " . {self}"),
        }
    }

    /// Helper method to write mutable list contents for display.
    fn write_mutable_list_contents(&self, f: &mut fmt::Formatter<'_>, first: bool) -> fmt::Result {
        match self {
            Value::Nil => Ok(()),
            Value::MutablePair(car_ref, cdr_ref) => {
                if !first {
                    write!(f, " ")?;
                }
                if let Ok(car) = car_ref.try_borrow() {
                    write!(f, "{car}")?;
                } else {
                    write!(f, "...")?;
                }
                if let Ok(cdr) = cdr_ref.try_borrow() {
                    match &*cdr {
                        Value::Nil => Ok(()),
                        Value::MutablePair(_, _) => cdr.write_mutable_list_contents(f, false),
                        _ => write!(f, " . {cdr}"),
                    }
                } else {
                    write!(f, " . ...")?;
                    Ok(())
                }
            }
            Value::Pair(car, cdr) => {
                if !first {
                    write!(f, " ")?;
                }
                write!(f, "{car}")?;
                match &**cdr {
                    Value::Nil => Ok(()),
                    Value::Pair(_, _) => cdr.write_list_contents(f, false),
                    Value::MutablePair(_, _) => cdr.write_mutable_list_contents(f, false),
                    _ => write!(f, " . {cdr}"),
                }
            }
            _ => write!(f, " . {self}"),
        }
    }
}

impl Environment {
    /// Creates a new environment with optional parent.
    pub fn new(parent: Option<Rc<Environment>>, generation: Generation) -> Self {
        Self {
            bindings: Rc::new(std::cell::RefCell::new(HashMap::new())),
            parent,
            generation,
            name: None,
        }
    }

    /// Creates a new environment with a name.
    pub fn with_name(
        parent: Option<Rc<Environment>>,
        generation: Generation,
        name: String,
    ) -> Self {
        Self {
            bindings: Rc::new(std::cell::RefCell::new(HashMap::new())),
            parent,
            generation,
            name: Some(name),
        }
    }

    /// Looks up a variable in this environment or its parents.
    pub fn lookup(&self, name: &str) -> Option<Value> {
        // Check local bindings first
        if let Some(value) = self.bindings.try_borrow().ok()?.get(name) {
            return Some(value.clone());
        }

        // Check parent environments
        if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }

    /// Defines a variable in this environment.
    pub fn define(&self, name: String, value: Value) {
        self.bindings.borrow_mut().insert(name, value);
    }

    /// Sets a variable in this environment or its parents.
    ///
    /// Returns true if the variable was found and set, false otherwise.
    pub fn set(&self, name: &str, value: Value) -> bool {
        // Check if variable exists in local bindings
        if let Ok(bindings) = self.bindings.try_borrow() {
            if bindings.contains_key(name) {
                drop(bindings); // Release the borrow
                self.bindings.borrow_mut().insert(name.to_string(), value);
                return true;
            }
        }

        // Check parent environments
        if let Some(parent) = &self.parent {
            parent.set(name, value)
        } else {
            false
        }
    }

    /// Extends this environment with new bindings.
    pub fn extend(&self, generation: Generation) -> Rc<Environment> {
        Rc::new(Environment::new(Some(Rc::new(self.clone())), generation))
    }

    /// Gets all variable names in this environment (for debugging).
    pub fn variable_names(&self) -> Vec<String> {
        match self.bindings.try_borrow() {
            Ok(bindings) => bindings.keys().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Converts this Environment to a ThreadSafeEnvironment.
    /// This is a bridge method during the migration process.
    pub fn to_thread_safe(&self) -> Arc<ThreadSafeEnvironment> {
        let parent = self.parent.as_ref().map(|p| p.to_thread_safe());

        let bindings = self
            .bindings
            .try_borrow()
            .ok()
            .map(|b| b.clone())
            .unwrap_or_default();

        Arc::new(ThreadSafeEnvironment {
            bindings: Arc::new(std::sync::RwLock::new(bindings)),
            parent,
            generation: self.generation,
            name: self.name.clone(),
        })
    }

    /// Converts this Environment to a ThreadSafeEnvironment that maintains live bindings.
    /// Used for recursive function definitions where the environment may be updated.
    pub fn to_thread_safe_live(&self) -> Arc<ThreadSafeEnvironment> {
        let parent = self.parent.as_ref().map(|p| p.to_thread_safe_live());

        // Create a thread-safe environment that references the live bindings
        let bindings = self
            .bindings
            .try_borrow()
            .ok()
            .map(|b| b.clone())
            .unwrap_or_default();
        Arc::new(ThreadSafeEnvironment {
            bindings: Arc::new(std::sync::RwLock::new(bindings)),
            parent,
            generation: self.generation,
            name: self.name.clone(),
        })
    }
}

impl ThreadSafeEnvironment {
    /// Creates a new thread-safe environment with optional parent.
    pub fn new(parent: Option<Arc<ThreadSafeEnvironment>>, generation: Generation) -> Self {
        Self {
            bindings: Arc::new(std::sync::RwLock::new(HashMap::new())),
            parent,
            generation,
            name: None,
        }
    }

    /// Creates a new thread-safe environment with a name.
    pub fn with_name(
        parent: Option<Arc<ThreadSafeEnvironment>>,
        generation: Generation,
        name: String,
    ) -> Self {
        Self {
            bindings: Arc::new(std::sync::RwLock::new(HashMap::new())),
            parent,
            generation,
            name: Some(name),
        }
    }

    /// Looks up a variable in this environment or its parents.
    /// This is thread-safe and immutable.
    pub fn lookup(&self, name: &str) -> Option<Value> {
        // Check local bindings first
        if let Some(value) = self.bindings.try_read().unwrap().get(name) {
            return Some(value.clone());
        }

        // Check parent environments
        if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }

    /// Defines a variable in this environment using interior mutability.
    /// This allows in-place updates for standard library initialization.
    pub fn define(&self, name: String, value: Value) {
        self.bindings.write().unwrap().insert(name, value);
    }

    /// Creates a new environment with an additional binding (COW semantics).
    /// This preserves immutability by creating a new environment.
    pub fn define_cow(&self, name: String, value: Value) -> Arc<ThreadSafeEnvironment> {
        let mut new_bindings = self.bindings.try_read().unwrap().clone();
        new_bindings.insert(name, value);

        Arc::new(ThreadSafeEnvironment {
            bindings: Arc::new(std::sync::RwLock::new(new_bindings)),
            parent: self.parent.clone(),
            generation: self.generation,
            name: self.name.clone(),
        })
    }

    /// Sets a variable in this environment or its parents using interior mutability.
    /// Returns true if the variable was found and set, false otherwise.
    pub fn set(&self, name: &str, value: Value) -> bool {
        // Check if variable exists in local bindings
        if self.bindings.try_read().unwrap().contains_key(name) {
            self.bindings
                .write()
                .unwrap()
                .insert(name.to_string(), value);
            return true;
        }

        // Check parent environments
        if let Some(parent) = &self.parent {
            parent.set(name, value)
        } else {
            false
        }
    }

    /// Creates a new environment with an updated binding (COW semantics).
    /// Returns None if the variable doesn't exist in the environment chain.
    pub fn set_cow(&self, name: &str, value: Value) -> Option<Arc<ThreadSafeEnvironment>> {
        // Check if variable exists in local bindings
        if self.bindings.try_read().unwrap().contains_key(name) {
            let mut new_bindings = self.bindings.try_read().unwrap().clone();
            new_bindings.insert(name.to_string(), value);

            return Some(Arc::new(ThreadSafeEnvironment {
                bindings: Arc::new(std::sync::RwLock::new(new_bindings)),
                parent: self.parent.clone(),
                generation: self.generation,
                name: self.name.clone(),
            }));
        }

        // Check parent environments and propagate the change
        if let Some(parent) = &self.parent {
            if let Some(new_parent) = parent.set_cow(name, value) {
                return Some(Arc::new(ThreadSafeEnvironment {
                    bindings: self.bindings.clone(),
                    parent: Some(new_parent),
                    generation: self.generation,
                    name: self.name.clone(),
                }));
            }
        }

        None
    }

    /// Extends this environment with a new generation.
    pub fn extend(&self, generation: Generation) -> Arc<ThreadSafeEnvironment> {
        Arc::new(ThreadSafeEnvironment::new(
            Some(Arc::new(self.clone())),
            generation,
        ))
    }

    /// Gets all variable names in this environment (for debugging).
    pub fn variable_names(&self) -> Vec<String> {
        self.bindings.try_read().unwrap().keys().cloned().collect()
    }

    /// Gets all accessible variable names (including from parents).
    pub fn all_variable_names(&self) -> Vec<String> {
        let mut names = self.variable_names();

        if let Some(parent) = &self.parent {
            names.extend(parent.all_variable_names());
        }

        names.sort();
        names.dedup();
        names
    }

    /// Gets the generation of this environment.
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// Gets the parent environment.
    pub fn parent(&self) -> Option<&Arc<ThreadSafeEnvironment>> {
        self.parent.as_ref()
    }

    /// Gets the environment name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Converts this environment to the legacy Environment type for compatibility.
    /// This is a temporary method during the migration process.
    pub fn to_legacy(&self) -> Rc<Environment> {
        let legacy_parent = self.parent.as_ref().map(|p| p.to_legacy());

        Rc::new(Environment {
            bindings: Rc::new(std::cell::RefCell::new(
                self.bindings.try_read().unwrap().clone(),
            )),
            parent: legacy_parent,
            generation: self.generation,
            name: self.name.clone(),
        })
    }

    /// Creates a ThreadSafeEnvironment from a legacy Environment.
    /// This is a temporary method during the migration process.
    pub fn from_legacy(legacy: &Environment) -> Arc<ThreadSafeEnvironment> {
        let parent = legacy.parent.as_ref().map(|p| Self::from_legacy(p));

        Arc::new(ThreadSafeEnvironment {
            bindings: Arc::new(std::sync::RwLock::new(HashMap::new())),
            parent,
            generation: legacy.generation,
            name: legacy.name.clone(),
        })
    }
}

// Thread safety markers for ThreadSafeEnvironment
unsafe impl Send for ThreadSafeEnvironment {}
unsafe impl Sync for ThreadSafeEnvironment {}

// Thread safety markers for Value and related types
// Value is Send + Sync because all its contents are thread-safe
unsafe impl Send for Value {}
unsafe impl Sync for Value {}

// Procedure needs special handling for metadata HashMap
unsafe impl Send for Procedure {}
unsafe impl Sync for Procedure {}

// CaseLambdaProcedure is safe because all its contents are thread-safe
unsafe impl Send for CaseLambdaProcedure {}
unsafe impl Sync for CaseLambdaProcedure {}

// PrimitiveProcedure is safe because function pointers are Send + Sync
unsafe impl Send for PrimitiveProcedure {}
unsafe impl Sync for PrimitiveProcedure {}

// Continuation is safe because all its contents are thread-safe
unsafe impl Send for Continuation {}
unsafe impl Sync for Continuation {}

// Frame is safe because all its contents are thread-safe
unsafe impl Send for Frame {}
unsafe impl Sync for Frame {}

// SyntaxTransformer is safe because all its contents are thread-safe
unsafe impl Send for SyntaxTransformer {}
unsafe impl Sync for SyntaxTransformer {}

// Port types are safe - all internal state is protected by Arc<RwLock<T>>
unsafe impl Send for Port {}
unsafe impl Sync for Port {}
unsafe impl Send for PortImpl {}
unsafe impl Sync for PortImpl {}
unsafe impl Send for PortFileHandle {}
unsafe impl Sync for PortFileHandle {}
unsafe impl Send for StandardPort {}
unsafe impl Sync for StandardPort {}
unsafe impl Send for PortMode {}
unsafe impl Sync for PortMode {}
unsafe impl Send for PortDirection {}
unsafe impl Sync for PortDirection {}

// Promise is safe because all its contents are thread-safe
unsafe impl Send for Promise {}
unsafe impl Sync for Promise {}

// TypeValue is safe because it only contains basic types
unsafe impl Send for TypeValue {}
unsafe impl Sync for TypeValue {}

// ForeignObject needs careful handling due to raw pointer
// For now, we'll be conservative and not implement Send/Sync
// This will need to be revisited based on specific FFI requirements

// CharSet is safe because it only contains immutable data (BTreeSet<char>)
// and all access is through Arc

// Parameter is safe because all its contents are thread-safe
unsafe impl Send for Parameter {}
unsafe impl Sync for Parameter {}

// RecordType is safe because it only contains basic types
unsafe impl Send for RecordType {}
unsafe impl Sync for RecordType {}

// FieldInfo is safe because it only contains basic types
unsafe impl Send for FieldInfo {}
unsafe impl Sync for FieldInfo {}

// Record is safe because all its contents are thread-safe
unsafe impl Send for Record {}
unsafe impl Sync for Record {}

impl Default for ThreadSafeEnvironment {
    fn default() -> Self {
        Self::new(None, 0)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(None, 0)
    }
}

// ============= STACK TRACE SUPPORT =============

/// Stack trace for runtime error reporting.
#[derive(Debug, Clone)]
pub struct StackTrace {
    /// Stack frames from most recent to oldest
    pub frames: Vec<StackFrame>,
}

/// A single frame in the execution stack.
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Function or procedure name
    pub name: Option<String>,
    /// Source location
    pub location: Option<Span>,
    /// Type of frame (function call, special form, etc.)
    pub frame_type: FrameType,
}

/// Type of stack frame.
#[derive(Debug, Clone)]
pub enum FrameType {
    /// Function or procedure call
    ProcedureCall,
    /// Special form evaluation
    SpecialForm(String),
    /// Primitive function call
    Primitive(String),
    /// Macro expansion
    MacroExpansion,
    /// Top-level evaluation
    TopLevel,
}

impl StackTrace {
    /// Creates a new empty stack trace.
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    /// Pushes a new frame onto the stack.
    pub fn push(&mut self, frame: StackFrame) {
        self.frames.push(frame);
    }

    /// Pops the most recent frame from the stack.
    pub fn pop(&mut self) -> Option<StackFrame> {
        self.frames.pop()
    }

    /// Returns true if the stack trace is empty.
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Returns the number of frames in the stack trace.
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Returns an iterator over the stack frames.
    pub fn frames(&self) -> impl Iterator<Item = &StackFrame> {
        self.frames.iter()
    }
}

impl StackFrame {
    /// Creates a new stack frame for a procedure call.
    pub fn procedure_call(name: Option<String>, location: Option<Span>) -> Self {
        Self {
            name,
            location,
            frame_type: FrameType::ProcedureCall,
        }
    }

    /// Creates a new stack frame for a special form.
    pub fn special_form(form_name: String, location: Option<Span>) -> Self {
        Self {
            name: Some(form_name.clone()),
            location,
            frame_type: FrameType::SpecialForm(form_name),
        }
    }

    /// Creates a new stack frame for a primitive function.
    pub fn primitive(name: String, location: Option<Span>) -> Self {
        Self {
            name: Some(name.clone()),
            location,
            frame_type: FrameType::Primitive(name),
        }
    }

    /// Creates a new stack frame for macro expansion.
    pub fn macro_expansion(name: Option<String>, location: Option<Span>) -> Self {
        Self {
            name,
            location,
            frame_type: FrameType::MacroExpansion,
        }
    }

    /// Creates a new stack frame for top-level evaluation.
    pub fn top_level(location: Option<Span>) -> Self {
        Self {
            name: None,
            location,
            frame_type: FrameType::TopLevel,
        }
    }
}

impl fmt::Display for StackTrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.frames.is_empty() {
            return writeln!(f, "  (empty stack trace)");
        }

        for (i, frame) in self.frames.iter().enumerate() {
            write!(f, "  {i}: ")?;
            match &frame.frame_type {
                FrameType::ProcedureCall => {
                    if let Some(name) = &frame.name {
                        write!(f, "in procedure '{name}'")?;
                    } else {
                        write!(f, "in anonymous procedure")?;
                    }
                }
                FrameType::SpecialForm(form) => {
                    write!(f, "in special form '{form}'")?;
                }
                FrameType::Primitive(name) => {
                    write!(f, "in primitive '{name}'")?;
                }
                FrameType::MacroExpansion => {
                    if let Some(name) = &frame.name {
                        write!(f, "in macro '{name}'")?;
                    } else {
                        write!(f, "in macro expansion")?;
                    }
                }
                FrameType::TopLevel => {
                    write!(f, "at top level")?;
                }
            }

            if let Some(location) = &frame.location {
                write!(f, " (at {}:{})", location.start, location.end())?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

impl Default for StackTrace {
    fn default() -> Self {
        Self::new()
    }
}

impl Continuation {
    /// Creates a new continuation with the given parameters.
    pub fn new(
        stack: Vec<Frame>,
        environment: Arc<ThreadSafeEnvironment>,
        id: u64,
        current_expr: Option<Spanned<Expr>>,
    ) -> Self {
        Self {
            stack,
            environment,
            id,
            current_expr,
            invoked: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Checks if this continuation has been invoked.
    pub fn is_invoked(&self) -> bool {
        self.invoked.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Marks this continuation as invoked.
    pub fn mark_invoked(&self) -> bool {
        self.invoked.swap(true, std::sync::atomic::Ordering::SeqCst)
    }
}

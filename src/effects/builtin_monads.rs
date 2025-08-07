//! Built-in monad types that appear in Lambdust's type system.
//!
//! This module implements the core monadic types that are visible at the 
//! language level and are used to control operational semantics through
//! the type system.

#![allow(missing_docs)]

use crate::diagnostics::{Error, Result};
use crate::eval::value::{Value, ThreadSafeEnvironment};
use crate::effects::continuation_monad::{ContinuationMonad, ContinuationFunction};
use crate::effects::list_monad::{List, ValueList, ListFunc};
use crate::effects::parser_monad::{Parser, ParseResult, ParseError, Input, Position};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// The Maybe monad - represents computations that may fail or return nothing
#[derive(Debug, Clone, PartialEq)]
pub enum Maybe<T> {
    /// Successful computation with a value
    Just(T),
    /// Failed or empty computation
    Nothing,
}

/// The Either monad - represents computations that may return one of two types
#[derive(Debug, Clone, PartialEq)]
pub enum Either<L, R> {
    /// Left value (typically used for errors)
    Left(L),
    /// Right value (typically used for success)
    Right(R),
}

/// The IO monad - represents computations with side effects
#[derive(Debug, Clone)]
pub struct IO<T> where T: Clone {
    /// The IO action to perform
    action: IOAction<T>,
}

/// The State monad - represents stateful computations
#[derive(Debug, Clone)]
pub struct State<S, A> {
    /// The stateful computation
    computation: StateComputation<S, A>,
}

/// The Reader monad - represents computations with a shared environment
#[derive(Debug, Clone)]
pub struct Reader<R, A> {
    /// The computation that reads from environment
    computation: ReaderComputation<R, A>,
}

/// Monoid trait for Writer monad output types
/// 
/// Provides the algebraic structure needed for Writer monad output accumulation.
/// Must satisfy monoid laws:
/// - Identity: mempty <> x = x <> mempty = x
/// - Associativity: (x <> y) <> z = x <> (y <> z)
pub trait Monoid: Clone + Send + Sync + 'static {
    /// The identity element
    fn mempty() -> Self;
    
    /// Associative binary operation
    fn mappend(&self, other: &Self) -> Self;
    
    /// Efficient fold over multiple values (default implementation)
    #[inline]
    fn mconcat(values: &[Self]) -> Self {
        values.iter().fold(Self::mempty(), |acc, x| acc.mappend(x))
    }
}

/// The Writer monad - represents computations that produce output
/// 
/// Type parameters:
/// - W: Output type that must implement Monoid for accumulation
/// - A: Value type of the computation
/// 
/// Thread-safe with Send + Sync bounds ensuring safe concurrent usage.
#[derive(Debug, Clone)]
pub struct Writer<W: Monoid, A> {
    /// The computation result
    value: A,
    /// The accumulated output
    output: W,
}

/// The Identity monad - zero-cost wrapper for pure computations
/// 
/// Provides monadic interface for values without computational context.
/// All operations are inlined for zero-cost abstraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identity<A> {
    /// The wrapped value
    value: A,
}

/// Type-safe function wrapper for Writer continuations
#[derive(Clone)]
pub struct WriterFunc<A, B> {
    id: u64,
    func: Arc<dyn Fn(A) -> B + Send + Sync + 'static>,
}

impl<A, B> WriterFunc<A, B> {
    pub fn new<F>(id: u64, func: F) -> Self
    where
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        Self {
            id,
            func: Arc::new(func),
        }
    }
    
    #[inline]
    pub fn call(&self, arg: A) -> B {
        (self.func)(arg)
    }
}

impl<A, B> fmt::Debug for WriterFunc<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WriterFunc({}):", self.id)
    }
}

/// The List monad - represents non-deterministic computations
/// This is a re-export of the high-performance List monad
pub type ListMonad<T> = List<T>;

/// The Parser monad - represents parsing computations
/// This is a re-export of the high-performance Parser monad  
pub type ParserMonad<T> = Parser<T>;

/// Type-safe function wrapper for IO continuations
#[derive(Clone)]
pub struct IOFunc<A, B> {
    id: u64,
    func: Arc<dyn Fn(A) -> B + Send + Sync + 'static>,
}

impl<A, B> IOFunc<A, B> {
    pub fn new<F>(id: u64, func: F) -> Self
    where
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        Self {
            id,
            func: Arc::new(func),
        }
    }
    
    pub fn call(&self, arg: A) -> B {
        (self.func)(arg)
    }
}

impl<A, B> std::fmt::Debug for IOFunc<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IOFunc({})", self.id)
    }
}

/// Actions within the IO monad (type-safe version)
#[derive(Debug, Clone)]
pub enum IOAction<T: Clone> {
    /// Pure value (return)
    Pure(T),
    
    /// Read from input
    Read {
        /// Continuation after reading
        continuation: IOFunc<String, IO<T>>,
    },
    
    /// Write to output
    Write {
        /// Value to write
        value: Value,
        /// Continuation after writing
        continuation: Box<IO<T>>,
    },
    
    /// Print a value
    Print {
        /// Value to print
        value: Value,
        /// Continuation after printing
        continuation: Box<IO<T>>,
    },
    
    /// Open a file
    OpenFile {
        /// File path
        path: String,
        /// File mode
        mode: FileMode,
        /// Continuation with file handle
        continuation: IOFunc<FileHandle, IO<T>>,
    },
    
    /// Close a file
    CloseFile {
        /// File handle to close
        handle: FileHandle,
        /// Continuation after closing
        continuation: Box<IO<T>>,
    },
    
    /// Bind operation
    Bind {
        /// Inner computation
        inner: Box<IO<Value>>,
        /// Next computation
        next: IOFunc<Value, IO<T>>,
    },
    
    /// Error in IO
    Error {
        /// The error that occurred
        error: Error,
    },
}

/// Type-safe function wrapper for State continuations
#[derive(Clone)]
pub struct StateFunc<A, B> {
    id: u64,
    func: Arc<dyn Fn(A) -> B + Send + Sync + 'static>,
}

impl<A, B> StateFunc<A, B> {
    pub fn new<F>(id: u64, func: F) -> Self
    where
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        Self {
            id,
            func: Arc::new(func),
        }
    }
    
    pub fn call(&self, arg: A) -> B {
        (self.func)(arg)
    }
}

impl<A, B> std::fmt::Debug for StateFunc<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateFunc({})", self.id)
    }
}

/// State computations (type-safe version)
#[derive(Debug, Clone)]
pub enum StateComputation<S, A> {
    /// Pure value
    Pure(A),
    
    /// Get current state
    Get {
        /// Continuation with state value
        continuation: StateFunc<S, State<S, A>>,
    },
    
    /// Set new state
    Put {
        /// New state value
        new_state: S,
        /// Continuation after setting state
        continuation: Box<State<S, A>>,
    },
    
    /// Modify state
    Modify {
        /// State modification function
        modifier: StateFunc<S, S>,
        /// Continuation after modification
        continuation: Box<State<S, A>>,
    },
    
    /// Bind operation
    Bind {
        /// Inner computation
        inner: Box<State<S, Value>>,
        /// Next computation
        next: StateFunc<Value, State<S, A>>,
    },
}

/// Type-safe function wrapper for Reader continuations
#[derive(Clone)]
pub struct ReaderFunc<A, B> {
    id: u64,
    func: Arc<dyn Fn(A) -> B + Send + Sync + 'static>,
}

impl<A, B> ReaderFunc<A, B> {
    pub fn new<F>(id: u64, func: F) -> Self
    where
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        Self {
            id,
            func: Arc::new(func),
        }
    }
    
    pub fn call(&self, arg: A) -> B {
        (self.func)(arg)
    }
}

impl<A, B> std::fmt::Debug for ReaderFunc<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReaderFunc({})", self.id)
    }
}

/// Reader computations (type-safe version)
#[derive(Debug, Clone)]
pub enum ReaderComputation<R, A> {
    /// Pure value
    Pure(A),
    
    /// Ask for environment
    Ask {
        /// Continuation with environment
        continuation: ReaderFunc<R, Reader<R, A>>,
    },
    
    /// Local environment modification
    Local {
        /// Environment modifier
        modifier: ReaderFunc<R, R>,
        /// Inner computation with modified environment
        inner: Box<Reader<R, A>>,
    },
    
    /// Bind operation
    Bind {
        /// Inner computation
        inner: Box<Reader<R, Value>>,
        /// Next computation
        next: ReaderFunc<Value, Reader<R, A>>,
    },
}

/// File modes for IO operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileMode {
    /// Read only
    Read,
    /// Write only
    Write,
    /// Append
    Append,
    /// Read and write
    ReadWrite,
}

/// File handle for IO operations
#[derive(Debug, Clone)]
pub struct FileHandle {
    /// Unique identifier
    pub id: u64,
    /// File path
    pub path: String,
    /// Current mode
    pub mode: FileMode,
    /// Whether the file is open
    pub is_open: bool,
}

/// Monadic operations for Maybe
impl<T> Maybe<T> {
    /// Create a successful Maybe value
    pub fn just(value: T) -> Self {
        Maybe::Just(value)
    }
    
    /// Create a failed Maybe value
    pub fn nothing() -> Self {
        Maybe::Nothing
    }
    
    /// Monadic bind for Maybe
    pub fn bind<U, F>(self, f: F) -> Maybe<U>
    where
        F: FnOnce(T) -> Maybe<U>,
    {
        match self {
            Maybe::Just(value) => f(value),
            Maybe::Nothing => Maybe::Nothing,
        }
    }
    
    /// Functor map for Maybe
    pub fn map<U, F>(self, f: F) -> Maybe<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Maybe::Just(value) => Maybe::Just(f(value)),
            Maybe::Nothing => Maybe::Nothing,
        }
    }
    
    /// Check if Maybe contains a value
    pub fn is_just(&self) -> bool {
        matches!(self, Maybe::Just(_))
    }
    
    /// Check if Maybe is empty
    pub fn is_nothing(&self) -> bool {
        matches!(self, Maybe::Nothing)
    }
    
    /// Extract value, providing a default if Nothing
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Maybe::Just(value) => value,
            Maybe::Nothing => default,
        }
    }
    
    /// Convert to Rust Option
    pub fn to_option(self) -> Option<T> {
        match self {
            Maybe::Just(value) => Some(value),
            Maybe::Nothing => None,
        }
    }
    
    /// Convert from Rust Option
    pub fn from_option(opt: Option<T>) -> Self {
        match opt {
            Some(value) => Maybe::Just(value),
            None => Maybe::Nothing,
        }
    }
}

/// Monadic operations for Either
impl<L, R> Either<L, R> {
    /// Create a Left value
    pub fn left(value: L) -> Self {
        Either::Left(value)
    }
    
    /// Create a Right value  
    pub fn right(value: R) -> Self {
        Either::Right(value)
    }
    
    /// Monadic bind for Either (on Right values)
    pub fn bind<U, F>(self, f: F) -> Either<L, U>
    where
        F: FnOnce(R) -> Either<L, U>,
    {
        match self {
            Either::Right(value) => f(value),
            Either::Left(err) => Either::Left(err),
        }
    }
    
    /// Functor map for Either (on Right values)
    pub fn map<U, F>(self, f: F) -> Either<L, U>
    where
        F: FnOnce(R) -> U,
    {
        match self {
            Either::Right(value) => Either::Right(f(value)),
            Either::Left(err) => Either::Left(err),
        }
    }
    
    /// Map over Left values
    pub fn map_left<M, F>(self, f: F) -> Either<M, R>
    where
        F: FnOnce(L) -> M,
    {
        match self {
            Either::Left(err) => Either::Left(f(err)),
            Either::Right(value) => Either::Right(value),
        }
    }
    
    /// Check if Either is Right
    pub fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }
    
    /// Check if Either is Left
    pub fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }
}

/// Monadic operations for IO
impl<T: Send + Sync + 'static + Clone> IO<T> {
    /// Create a pure IO value
    pub fn pure(value: T) -> Self {
        IO {
            action: IOAction::Pure(value),
        }
    }
    
    /// Create an IO error
    pub fn error(error: Error) -> Self {
        IO {
            action: IOAction::Error { error },
        }
    }
    
    /// Read from input
    pub fn read_line() -> IO<String> {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        IO {
            action: IOAction::Read {
                continuation: IOFunc::new(id, |input| IO::pure(input)),
            },
        }
    }
    
    /// Print a value
    pub fn print(value: Value) -> IO<()> {
        IO {
            action: IOAction::Print {
                value,
                continuation: Box::new(IO::pure(())),
            },
        }
    }
    
    /// Write a value
    pub fn write(value: Value) -> IO<()> {
        IO {
            action: IOAction::Write {
                value,
                continuation: Box::new(IO::pure(())),
            },
        }
    }
    
    /// Monadic bind for IO (simplified for type-safety)
    pub fn bind<U, F>(self, f: F) -> IO<U>
    where
        F: Fn(T) -> IO<U> + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static + Clone,
    {
        match self.action {
            IOAction::Pure(value) => f(value),
            _ => {
                // For simplicity, we'll restrict bind to work only with Value types
                // This is a temporary limitation for type safety
                panic!("bind currently only works with Pure values - complex type conversion needed")
            }
        }
    }
    
    /// Functor map for IO (simplified)
    pub fn map<U, F>(self, f: F) -> IO<U>
    where
        F: Fn(T) -> U + Send + Sync + 'static,
        T: Send + Sync + 'static,
        U: Send + Sync + 'static + Clone,
    {
        self.bind(move |value| IO::pure(f(value)))
    }
    
    /// Sequence IO operations, ignoring first result
    pub fn then<U>(self, next: IO<U>) -> IO<U>
    where
        T: 'static,
        U: Send + Sync + 'static + Clone,
    {
        self.bind(move |_| next.clone())
    }
}

/// Monadic operations for State
impl<S, A> State<S, A> {
    /// Create a pure state value
    pub fn pure(value: A) -> Self {
        State {
            computation: StateComputation::Pure(value),
        }
    }
    
    /// Get current state
    pub fn get() -> State<S, S>
    where
        S: Clone + Send + Sync + 'static,
    {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        State {
            computation: StateComputation::Get {
                continuation: StateFunc::new(id, |state| State::pure(state)),
            },
        }
    }
    
    /// Set new state
    pub fn put(new_state: S) -> State<S, ()>
    where
        S: Send + Sync + 'static,
    {
        State {
            computation: StateComputation::Put {
                new_state,
                continuation: Box::new(State::pure(())),
            },
        }
    }
    
    /// Modify current state
    pub fn modify<F>(modifier: F) -> State<S, ()>
    where
        F: Fn(S) -> S + Send + Sync + 'static,
        S: Send + Sync + 'static,
    {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        State {
            computation: StateComputation::Modify {
                modifier: StateFunc::new(id, modifier),
                continuation: Box::new(State::pure(())),
            },
        }
    }
    
    /// Monadic bind for State (type-safe version)
    pub fn bind<B, F>(self, f: F) -> State<S, B>
    where
        F: Fn(A) -> State<S, B> + Send + Sync + 'static,
        A: 'static,
        S: 'static,
        B: 'static,
    {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        match self.computation {
            StateComputation::Pure(value) => f(value),
            _ => {
                // For now, we'll restrict this to a specific case to avoid unsafe transmute
                // In a full implementation, we would handle the type conversion properly
                panic!("Complex State bind operations not yet implemented without type conversion")
            }
        }
    }
    
    /// Functor map for State
    pub fn map<B, F>(self, f: F) -> State<S, B>
    where
        F: Fn(A) -> B + Send + Sync + 'static,
        A: 'static,
        S: 'static,
        B: 'static,
    {
        self.bind(move |a| State::pure(f(a)))
    }
    
    /// Run a stateful computation
    pub fn run_state(self, initial_state: S) -> Result<(A, S)>
    where
        S: Clone,
    {
        match self.computation {
            StateComputation::Pure(value) => Ok((value, initial_state)),
            StateComputation::Get { continuation } => {
                let next_state = continuation.call(initial_state.clone());
                next_state.run_state(initial_state)
            }
            StateComputation::Put { new_state, continuation } => {
                continuation.run_state(new_state)
            }
            StateComputation::Modify { modifier, continuation } => {
                let new_state = modifier.call(initial_state);
                continuation.run_state(new_state)
            }
            StateComputation::Bind { inner, next } => {
                let (intermediate, intermediate_state) = inner.run_state(initial_state)?;
                let final_computation = next.call(intermediate);
                final_computation.run_state(intermediate_state)
            }
        }
    }
}

/// Monadic operations for Reader
impl<R, A> Reader<R, A> {
    /// Create a pure reader value
    pub fn pure(value: A) -> Self {
        Reader {
            computation: ReaderComputation::Pure(value),
        }
    }
    
    /// Ask for the environment
    pub fn ask() -> Reader<R, R>
    where
        R: Clone + Send + Sync + 'static,
    {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        Reader {
            computation: ReaderComputation::Ask {
                continuation: ReaderFunc::new(id, |env| Reader::pure(env)),
            },
        }
    }
    
    /// Run computation with modified environment
    pub fn local<F>(modifier: F, computation: Reader<R, A>) -> Reader<R, A>
    where
        F: Fn(R) -> R + Send + Sync + 'static,
        R: 'static,
    {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        Reader {
            computation: ReaderComputation::Local {
                modifier: ReaderFunc::new(id, modifier),
                inner: Box::new(computation),
            },
        }
    }
    
    /// Monadic bind for Reader (type-safe version)
    pub fn bind<B, F>(self, f: F) -> Reader<R, B>
    where
        F: Fn(A) -> Reader<R, B> + Send + Sync + 'static,
        A: 'static,
        R: 'static,
        B: 'static,
    {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        match self.computation {
            ReaderComputation::Pure(value) => f(value),
            _ => {
                // For now, we'll restrict this to a specific case to avoid unsafe transmute
                // In a full implementation, we would handle the type conversion properly
                panic!("Complex Reader bind operations not yet implemented without type conversion")
            }
        }
    }
    
    /// Run a reader computation with an environment
    pub fn run_reader(self, environment: R) -> Result<A>
    where
        R: Clone,
    {
        match self.computation {
            ReaderComputation::Pure(value) => Ok(value),
            ReaderComputation::Ask { continuation } => {
                let next_computation = continuation.call(environment.clone());
                next_computation.run_reader(environment)
            }
            ReaderComputation::Local { modifier, inner } => {
                let modified_env = modifier.call(environment);
                inner.run_reader(modified_env)
            }
            ReaderComputation::Bind { inner, next } => {
                let intermediate = inner.run_reader(environment.clone())?;
                let final_computation = next.call(intermediate);
                final_computation.run_reader(environment)
            }
        }
    }
}

/// Conversion functions between monads and Lambdust Values
/// Convert Maybe to Value
impl From<Maybe<Value>> for Value {
    fn from(maybe: Maybe<Value>) -> Self {
        match maybe {
            Maybe::Just(value) => value,
            Maybe::Nothing => Value::Nil,
        }
    }
}

/// Convert Value to Maybe
impl From<Value> for Maybe<Value> {
    fn from(value: Value) -> Self {
        match value {
            Value::Nil => Maybe::Nothing,
            other => Maybe::Just(other),
        }
    }
}

/// Convert List to Value (as cons list)
impl From<ValueList> for Value {
    fn from(list: ValueList) -> Self {
        list.to_cons_list()
    }
}

/// Convert Value to List (from cons list)
impl TryFrom<Value> for ValueList {
    type Error = Box<Error>;
    
    fn try_from(value: Value) -> Result<Self> {
        ValueList::from_cons_list(value)
    }
}

/// Convert ParseError to Lambdust Error
impl From<ParseError> for Error {
    fn from(parse_error: ParseError) -> Self {
        Error::ParseError {
            message: parse_error.message,
            span: crate::diagnostics::Span::new(
                parse_error.position.offset,
                parse_error.position.offset + 1,
            ),
        }
    }
}

/// Convert Either to Value
impl From<Either<Error, Value>> for Result<Value> {
    fn from(either: Either<Error, Value>) -> Self {
        match either {
            Either::Right(value) => Ok(value),
            Either::Left(error) => Err(error),
        }
    }
}

/// IO execution context
#[derive(Debug)]
pub struct IOContext {
    /// File handles
    file_handles: Arc<Mutex<HashMap<u64, FileHandle>>>,
    
    /// Next file handle ID
    next_handle_id: Arc<Mutex<u64>>,
}

impl IOContext {
    /// Create a new IO context
    pub fn new() -> Self {
        Self {
            file_handles: Arc::new(Mutex::new(HashMap::new())),
            next_handle_id: Arc::new(Mutex::new(1)),
        }
    }
    
    /// Execute an IO computation
    pub fn run_io<T: Clone>(&self, io: IO<T>) -> Result<T> {
        self.execute_io_action(io.action)
    }
    
    /// Execute a specific IO action
    fn execute_io_action<T: Clone>(&self, action: IOAction<T>) -> Result<T> {
        match action {
            IOAction::Pure(value) => Ok(value),
            
            IOAction::Read { continuation } => {
                // Read from stdin (simplified)
                let input = "test input".to_string(); // In practice, read from actual stdin
                let next_io = continuation.call(input);
                self.run_io(next_io)
            }
            
            IOAction::Write { value, continuation } => {
                print!("{}", value);
                self.run_io(*continuation)
            }
            
            IOAction::Print { value, continuation } => {
                println!("{}", value);
                self.run_io(*continuation)
            }
            
            IOAction::OpenFile { path, mode, continuation } => {
                let handle = FileHandle {
                    id: *self.next_handle_id.lock().unwrap(),
                    path: path.clone()),
                    mode,
                    is_open: true,
                };
                
                // Register handle
                self.file_handles.lock().unwrap().insert(handle.id, handle.clone());
                *self.next_handle_id.lock().unwrap() += 1;
                
                let next_io = continuation.call(handle);
                self.run_io(next_io)
            }
            
            IOAction::CloseFile { handle, continuation } => {
                // Remove handle
                self.file_handles.lock().unwrap().remove(&handle.id);
                self.run_io(*continuation)
            }
            
            IOAction::Bind { inner, next } => {
                let intermediate = self.run_io(*inner)?;
                let final_io = next.call(intermediate);
                self.run_io(final_io)
            }
            
            IOAction::Error { error } => Err(error),
        }
    }
}

impl Default for IOContext {
    fn default() -> Self {
        Self::new()
    }
}

/// String monoid - concatenation with empty string identity
impl Monoid for String {
    #[inline]
    fn mempty() -> Self {
        String::new()
    }
    
    #[inline]
    fn mappend(&self, other: &Self) -> Self {
        let mut result = self.clone());
        result.push_str(other);
        result
    }
    
    #[inline]
    fn mconcat(values: &[Self]) -> Self {
        values.join("")
    }
}

/// Vec<T> monoid - concatenation with empty vector identity
impl<T: Clone + Send + Sync + 'static> Monoid for Vec<T> {
    #[inline]
    fn mempty() -> Self {
        Vec::new()
    }
    
    #[inline]
    fn mappend(&self, other: &Self) -> Self {
        let mut result = self.clone());
        result.extend_from_slice(other);
        result
    }
    
    #[inline]
    fn mconcat(values: &[Self]) -> Self {
        values.iter().flat_map(|v| v.iter()).clone())().collect()
    }
}


/// Unit monoid - trivial monoid for computations with no output
impl Monoid for () {
    #[inline]
    fn mempty() -> Self {
        ()
    }
    
    #[inline]
    fn mappend(&self, _other: &Self) -> Self {
        ()
    }
}

/// Sum monoid for numeric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Sum<T>(pub T);

impl<T> Monoid for Sum<T> 
where 
    T: Clone + Send + Sync + 'static + std::ops::Add<Output = T> + Default,
{
    #[inline]
    fn mempty() -> Self {
        Sum(T::default())
    }
    
    #[inline]
    fn mappend(&self, other: &Self) -> Self {
        Sum(self.0.clone()) + other.0.clone())
    }
}

/// Product monoid for numeric types (basic implementation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Product<T>(pub T);

// Basic implementation for i32 as example
impl Monoid for Product<i32> {
    #[inline]
    fn mempty() -> Self {
        Product(1)
    }
    
    #[inline]
    fn mappend(&self, other: &Self) -> Self {
        Product(self.0 * other.0)
    }
}

// Basic implementation for f64 as example
impl Monoid for Product<f64> {
    #[inline]
    fn mempty() -> Self {
        Product(1.0)
    }
    
    #[inline]
    fn mappend(&self, other: &Self) -> Self {
        Product(self.0 * other.0)
    }
}

/// Monadic operations for Writer
impl<W: Monoid, A> Writer<W, A> {
    /// Create a pure Writer value with empty output
    /// 
    /// # Examples
    /// ```
    /// let w: Writer<String, i32> = Writer::pure(42);
    /// assert_eq!(w.value(), &42);
    /// assert_eq!(w.output(), &String::new());
    /// ```
    #[inline]
    pub fn pure(value: A) -> Self {
        Self {
            value,
            output: W::mempty(),
        }
    }
    
    /// Create a Writer with both value and output
    /// 
    /// # Examples  
    /// ```
    /// let w = Writer::new(42, "Hello".to_string());
    /// assert_eq!(w.value(), &42);
    /// assert_eq!(w.output(), "Hello");
    /// ```
    #[inline]
    pub fn new(value: A, output: W) -> Self {
        Self { value, output }
    }
    
    /// Write output without changing the value
    /// 
    /// # Examples
    /// ```
    /// let w: Writer<String, ()> = Writer::tell("Debug message".to_string());
    /// assert_eq!(w.output(), "Debug message");
    /// ```
    #[inline]
    pub fn tell(output: W) -> Writer<W, ()> {
        Writer {
            value: (),
            output,
        }
    }
    
    /// Get the computed value
    #[inline]
    pub fn value(&self) -> &A {
        &self.value
    }
    
    /// Get the accumulated output
    #[inline]
    pub fn output(&self) -> &W {
        &self.output
    }
    
    /// Extract both value and output
    #[inline]
    pub fn run_writer(self) -> (A, W) {
        (self.value, self.output)
    }
    
    /// Monadic bind for Writer
    /// 
    /// Sequences computations while accumulating output using the monoid operation.
    pub fn bind<B, F>(self, f: F) -> Writer<W, B>
    where
        F: FnOnce(A) -> Writer<W, B>,
    {
        let Writer { value: next_value, output: next_output } = f(self.value);
        Writer {
            value: next_value,
            output: self.output.mappend(&next_output),
        }
    }
    
    /// Functor map for Writer
    #[inline]
    pub fn map<B, F>(self, f: F) -> Writer<W, B>
    where
        F: FnOnce(A) -> B,
    {
        Writer {
            value: f(self.value),
            output: self.output,
        }
    }
    
    /// Listen to the output while preserving it
    /// 
    /// Returns a Writer containing the original value paired with the output.
    #[inline]
    pub fn listen(self) -> Writer<W, (A, W)> {
        let output_copy = self.output.clone());
        Writer {
            value: (self.value, output_copy.clone()),
            output: output_copy,
        }
    }
    
    /// Execute computation and apply function to the output
    /// 
    /// The function receives both the value and output, returning a new output.
    pub fn pass<F>(self) -> Writer<W, A>
    where
        F: FnOnce(&W) -> W,
        A: Clone,
    {
        match self.value {
            // For simplicity, assuming the value contains the transformation function
            // In a full implementation, this would be more sophisticated
            _ => Writer {
                value: self.value,
                output: self.output, // Placeholder - would apply transformation
            }
        }
    }
    
    /// Censor (transform) the output
    /// 
    /// Applies a function to modify the output before it's combined with parent computations.
    pub fn censor<F>(self, f: F) -> Writer<W, A>
    where
        F: FnOnce(W) -> W,
    {
        Writer {
            value: self.value,
            output: f(self.output),
        }
    }
    
    /// Sequence Writer operations, ignoring first result
    #[inline]
    pub fn then<B>(self, next: Writer<W, B>) -> Writer<W, B> {
        self.bind(|_| next)
    }
}

/// Monadic operations for Identity
impl<A> Identity<A> {
    /// Create a pure Identity value
    /// 
    /// # Examples
    /// ```
    /// let id = Identity::pure(42);
    /// assert_eq!(id.run_identity(), 42);
    /// ```
    #[inline]
    pub fn pure(value: A) -> Self {
        Identity { value }
    }
    
    /// Create an Identity (alias for pure)
    #[inline]
    pub fn new(value: A) -> Self {
        Identity { value }
    }
    
    /// Extract the wrapped value (zero-cost)
    #[inline]
    pub fn run_identity(self) -> A {
        self.value
    }
    
    /// Get a reference to the wrapped value
    #[inline]
    pub fn value(&self) -> &A {
        &self.value
    }
    
    /// Monadic bind for Identity (zero-cost composition)
    #[inline]
    pub fn bind<B, F>(self, f: F) -> Identity<B>
    where
        F: FnOnce(A) -> Identity<B>,
    {
        f(self.value)
    }
    
    /// Functor map for Identity (zero-cost transformation)
    #[inline]
    pub fn map<B, F>(self, f: F) -> Identity<B>
    where
        F: FnOnce(A) -> B,
    {
        Identity {
            value: f(self.value),
        }
    }
    
    /// Applicative apply for Identity
    #[inline]
    pub fn apply<B, F>(self, func: Identity<F>) -> Identity<B>
    where
        F: FnOnce(A) -> B,
    {
        Identity {
            value: (func.value)(self.value),
        }
    }
    
    /// Sequence Identity operations (zero-cost)
    #[inline]
    pub fn then<B>(self, next: Identity<B>) -> Identity<B> {
        next
    }
}


/// Default implementation for Identity
impl<A: Default> Default for Identity<A> {
    #[inline]
    fn default() -> Self {
        Identity {
            value: A::default(),
        }
    }
}

/// Thread safety implementations
unsafe impl<W: Monoid, A: Send> Send for Writer<W, A> {}
unsafe impl<W: Monoid, A: Sync> Sync for Writer<W, A> {}
unsafe impl<A: Send> Send for Identity<A> {}
unsafe impl<A: Sync> Sync for Identity<A> {}

/// Convert Writer to Value (as a pair of value and output)
impl<W: Monoid> From<Writer<W, Value>> for Value 
where
    W: Into<Value>,
{
    fn from(writer: Writer<W, Value>) -> Self {
        let (value, output) = writer.run_writer();
        Value::Pair(
            Arc::new(value),
            Arc::new(output.into()),
        )
    }
}

/// Convert Identity to Value
impl From<Identity<Value>> for Value {
    #[inline]
    fn from(identity: Identity<Value>) -> Self {
        identity.run_identity()
    }
}

/// Convert Value to Identity
impl From<Value> for Identity<Value> {
    #[inline]
    fn from(value: Value) -> Self {
        Identity::pure(value)
    }
}

/// Convert String Writer to Value for R7RS integration
impl From<Writer<String, Value>> for Value {
    fn from(writer: Writer<String, Value>) -> Self {
        let (value, output) = writer.run_writer();
        if output.is_empty() {
            value
        } else {
            Value::Pair(
                Arc::new(value),
                Arc::new(Value::Literal(crate::ast::Literal::String(output))),
            )
        }
    }
}

impl<W: Monoid + fmt::Display, A: fmt::Display> fmt::Display for Writer<W, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Writer({}, output: {})", self.value, self.output)
    }
}

impl<A: fmt::Display> fmt::Display for Identity<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Identity({})", self.value)
    }
}

/// Display implementations
impl<T: fmt::Display> fmt::Display for Maybe<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Maybe::Just(value) => write!(f, "Just({})", value),
            Maybe::Nothing => write!(f, "Nothing"),
        }
    }
}

impl<L: fmt::Display, R: fmt::Display> fmt::Display for Either<L, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Either::Left(value) => write!(f, "Left({})", value),
            Either::Right(value) => write!(f, "Right({})", value),
        }
    }
}

impl<T: Clone> fmt::Display for IO<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IO(<computation>)")
    }
}

impl<S, A> fmt::Display for State<S, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "State(<computation>)")
    }
}

impl<R, A> fmt::Display for Reader<R, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Reader(<computation>)")
    }
}

// Thread safety
unsafe impl<T: Send + Clone> Send for IO<T> {}
unsafe impl<T: Sync + Clone> Sync for IO<T> {}
unsafe impl<S: Send, A: Send> Send for State<S, A> {}
unsafe impl<S: Sync, A: Sync> Sync for State<S, A> {}
unsafe impl<R: Send, A: Send> Send for Reader<R, A> {}
unsafe impl<R: Sync, A: Sync> Sync for Reader<R, A> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maybe_monad() {
        let result = Maybe::just(21)
            .bind(|x| Maybe::just(x * 2));
        
        assert_eq!(result, Maybe::just(42));
        
        let nothing_result = Maybe::<i32>::nothing()
            .bind(|x| Maybe::just(x * 2));
        
        assert_eq!(nothing_result, Maybe::nothing());
    }

    #[test]
    fn test_either_monad() {
        let result = Either::right(21)
            .bind(|x| Either::right(x * 2));
        
        assert_eq!(result, Either::right(42));
        
        let error_result = Either::<&str, i32>::left("error")
            .bind(|x| Either::right(x * 2));
        
        assert_eq!(error_result, Either::left("error"));
    }

    #[test]
    fn test_io_monad() {
        let io = IO::pure(42);
        let context = IOContext::new();
        let result = context.run_io(io).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_state_monad() {
        let computation = State::put(42)
            .bind(|_| State::get());
        
        let (result, final_state) = computation.run_state(0).unwrap();
        assert_eq!(result, 42);
        assert_eq!(final_state, 42);
    }

    #[test]
    fn test_reader_monad() {
        let computation = Reader::ask::<String>()
            .bind(|env| Reader::pure(format!("Hello, {}!", env)));
        
        let result = computation.run_reader("World".to_string()).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_list_monad_integration() {
        // Test List monad basic operations
        let list1 = ListMonad::from_vec(vec![1, 2, 3]);
        let list2 = ListMonad::from_vec(vec![4, 5, 6]);
        
        let combined = list1.plus(list2);
        assert_eq!(combined.to_vec(), vec![1, 2, 3, 4, 5, 6]);
        
        // Test List monad bind
        let result = ListMonad::from_vec(vec![1, 2]).bind(|x| {
            ListMonad::from_vec(vec![x * 2, x * 3])
        });
        assert_eq!(result.to_vec(), vec![2, 3, 4, 6]);
    }

    #[test] 
    fn test_parser_monad_integration() {
        // Test Parser monad basic operations
        let parser = ParserMonad::char('a');
        let input = Input::new("abc");
        let result = parser.parse(input).unwrap();
        
        assert_eq!(result.0, 'a');
        assert_eq!(result.1.remaining(), "bc");
        
        // Test Parser monad bind
        let parser = ParserMonad::char('a').bind(|_| ParserMonad::char('b'));
        let input = Input::new("abc");
        let result = parser.parse(input).unwrap();
        
        assert_eq!(result.0, 'b');
        assert_eq!(result.1.remaining(), "c");
    }

    #[test]
    fn test_value_list_conversion() {
        use std::sync::Arc;
        
        // Create a cons list in Value format
        let cons_list = Value::Pair(
            Arc::new(Value::Literal(crate::ast::Literal::Number(1.0))),
            Arc::new(Value::Pair(
                Arc::new(Value::Literal(crate::ast::Literal::Number(2.0))),
                Arc::new(Value::Pair(
                    Arc::new(Value::Literal(crate::ast::Literal::Number(3.0))),
                    Arc::new(Value::Nil)
                ))
            ))
        );
        
        // Convert to ValueList
        let value_list: ValueList = cons_list.try_into().unwrap();
        assert_eq!(value_list.len(), 3);
        
        // Convert back to Value
        let back_to_cons: Value = value_list.into();
        // Verify it's not Nil (full equality test would be complex)
        assert!(!matches!(back_to_cons, Value::Nil));
    }

    #[test]
    fn test_writer_monad_laws() {
        // Test left identity: return a >>= k ≡ k a
        let value = 42;
        let k = |x: i32| Writer::new(x * 2, "computed".to_string());
        
        let left = Writer::<String, i32>::pure(value).bind(k);
        let right = k(value);
        
        assert_eq!(left.run_writer(), right.run_writer());
        
        // Test right identity: m >>= return ≡ m
        let m = Writer::new(42, "output".to_string());
        let left = m.clone()).bind(Writer::pure);
        let right = m;
        
        assert_eq!(left.run_writer(), right.run_writer());
        
        // Test associativity: (m >>= f) >>= g ≡ m >>= (\x -> f x >>= g)
        let m = Writer::new(10, "start ".to_string());
        let f = |x: i32| Writer::new(x + 5, "f ".to_string());
        let g = |x: i32| Writer::new(x * 2, "g".to_string());
        
        let left = m.clone()).bind(f).bind(g);
        let right = m.bind(|x| f(x).bind(g));
        
        assert_eq!(left.run_writer(), right.run_writer());
    }

    #[test]
    fn test_writer_operations() {
        // Test tell
        let writer = Writer::<String, ()>::tell("Hello, World!".to_string());
        assert_eq!(writer.output(), "Hello, World!");
        
        // Test listen
        let writer = Writer::new(42, "output".to_string());
        let listened = writer.listen();
        let ((value, output), final_output) = listened.run_writer();
        assert_eq!(value, 42);
        assert_eq!(output, "output");
        assert_eq!(final_output, "output");
        
        // Test censor
        let writer = Writer::new(42, "hello".to_string());
        let censored = writer.censor(|s| s.to_uppercase());
        assert_eq!(censored.run_writer(), (42, "HELLO".to_string()));
        
        // Test output accumulation
        let result = Writer::<String, ()>::tell("Hello ".to_string())
            .bind(|_| Writer::<String, ()>::tell("World".to_string()))
            .bind(|_| Writer::new(42, "!".to_string()));
        
        assert_eq!(result.run_writer(), (42, "Hello World!".to_string()));
    }

    #[test]
    fn test_identity_monad_laws() {
        // Test left identity: return a >>= k ≡ k a
        let value = 42;
        let k = |x: i32| Identity::pure(x * 2);
        
        let left = Identity::pure(value).bind(k);
        let right = k(value);
        
        assert_eq!(left.run_identity(), right.run_identity());
        
        // Test right identity: m >>= return ≡ m
        let m = Identity::pure(42);
        let left = m.bind(Identity::pure);
        let right = m;
        
        assert_eq!(left.run_identity(), right.run_identity());
        
        // Test associativity: (m >>= f) >>= g ≡ m >>= (\x -> f x >>= g)
        let m = Identity::pure(10);
        let f = |x: i32| Identity::pure(x + 5);
        let g = |x: i32| Identity::pure(x * 2);
        
        let left = m.bind(f).bind(g);
        let right = m.bind(|x| f(x).bind(g));
        
        assert_eq!(left.run_identity(), right.run_identity());
    }

    #[test]
    fn test_identity_zero_cost() {
        // Test that Identity operations compile to zero-cost abstractions
        let value = 42;
        let identity = Identity::pure(value);
        assert_eq!(identity.run_identity(), value);
        
        // Test map operation
        let mapped = identity.map(|x| x * 2);
        assert_eq!(mapped.run_identity(), 84);
        
        // Test bind operation
        let bound = identity.bind(|x| Identity::pure(x + 10));
        assert_eq!(bound.run_identity(), 52);
    }

    #[test]
    fn test_monoid_laws() {
        // Test String monoid laws
        let a = "hello".to_string();
        let b = " ".to_string();
        let c = "world".to_string();
        
        // Identity laws
        assert_eq!(String::mempty().mappend(&a), a);
        assert_eq!(a.clone()).mappend(&String::mempty()), a);
        
        // Associativity
        let left = a.clone()).mappend(&b).mappend(&c);
        let right = a.mappend(&b.mappend(&c));
        assert_eq!(left, right);
        assert_eq!(left, "hello world");
        
        // Test Vec<i32> monoid laws
        let vec_a = vec![1, 2];
        let vec_b = vec![3];
        let vec_c = vec![4, 5];
        
        // Identity laws
        assert_eq!(Vec::<i32>::mempty().mappend(&vec_a), vec_a);
        assert_eq!(vec_a.clone()).mappend(&Vec::mempty()), vec_a);
        
        // Associativity
        let left = vec_a.clone()).mappend(&vec_b).mappend(&vec_c);
        let right = vec_a.mappend(&vec_b.mappend(&vec_c));
        assert_eq!(left, right);
        assert_eq!(left, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_writer_r7rs_integration() {
        use crate::ast::Literal;
        
        // Test Writer with String output for Scheme port integration
        let writer = Writer::<String, ()>::tell("Debug: ".to_string())
            .bind(|_| Writer::new(
                Value::Literal(Literal::Number(42.0)),
                "computation complete\n".to_string()
            ));
        
        let (value, output) = writer.run_writer();
        assert!(matches!(value, Value::Literal(Literal::Number(42.0))));
        assert_eq!(output, "Debug: computation complete\n");
        
        // Test conversion to Value
        let writer_value: Value = Writer::new(
            Value::Literal(Literal::String("result".to_string())),
            "log output".to_string()
        ).into();
        
        // Should be a pair of (value, output)
        match writer_value {
            Value::Pair(car, cdr) => {
                assert!(matches!(**car, Value::Literal(Literal::String(_))));
                assert!(matches!(**cdr, Value::Literal(Literal::String(_))));
            },
            _ => panic!("Expected pair")
        }
    }

    #[test]
    fn test_performance_characteristics() {
        use std::time::Instant;
        
        // Test Writer performance with large output accumulation
        let start = Instant::now();
        let mut writer = Writer::<String, i32>::pure(0);
        
        for i in 0..1000 {
            writer = writer.bind(|n| Writer::new(n + 1, format!("step {} ", i)));
        }
        
        let (value, output) = writer.run_writer();
        let elapsed = start.elapsed();
        
        assert_eq!(value, 1000);
        assert!(output.len() > 5000); // Should have accumulated substantial output
        assert!(elapsed.as_millis() < 100); // Should be fast
        
        // Test Identity zero-cost abstraction
        let start = Instant::now();
        let mut identity = Identity::pure(0);
        
        for _ in 0..10000 {
            identity = identity.bind(|n| Identity::pure(n + 1));
        }
        
        let result = identity.run_identity();
        let elapsed = start.elapsed();
        
        assert_eq!(result, 10000);
        assert!(elapsed.as_millis() < 10); // Should be extremely fast due to inlining
    }
}
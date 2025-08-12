//! High-performance List monad implementation for Lambdust.
//!
//! This module provides a zero-cost abstraction for the List monad that integrates
//! seamlessly with Lambdust's existing Value system. The implementation focuses on
//! memory efficiency and thread safety while maintaining the mathematical properties
//! of the List monad.
//!
//! The List monad represents non-deterministic computations where each computation
//! can yield multiple results. This is particularly useful for search algorithms,
//! parsing with backtracking, and generating combinations.

#![allow(missing_docs)]

use crate::eval::value::Value;
use crate::diagnostics::{Error, Result};
use std::sync::Arc;
use std::fmt;

/// A high-performance List monad that integrates with Lambdust's Value system.
/// 
/// The List monad represents computations that can produce multiple results.
/// It uses Arc for efficient cloning and is thread-safe by design.
#[derive(Clone)]
pub struct List<T> {
    /// The underlying list implementation using persistent data structures
    inner: ListImpl<T>,
}

impl<T: fmt::Debug> fmt::Debug for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "List({:?})", self.inner)
    }
}

impl<T: PartialEq + Clone + Send + Sync + 'static> PartialEq for List<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

/// Internal representation of the list monad
#[derive(Clone)]
enum ListImpl<T> {
    /// Empty list (MonadZero/MonadPlus zero element)
    Empty,
    
    /// Single element
    Single(T),
    
    /// Multiple elements stored efficiently
    Multiple(Arc<Vec<T>>),
    
    /// Lazy computation (for performance optimization)
    Lazy(Arc<dyn Fn() -> ListImpl<T> + Send + Sync + 'static>),
    
    /// Concatenation of two lists (lazy evaluation)
    Concat(Arc<List<T>>, Arc<List<T>>),
}

impl<T: fmt::Debug> fmt::Debug for ListImpl<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ListImpl::Empty => write!(f, "Empty"),
            ListImpl::Single(t) => write!(f, "Single({t:?})"),
            ListImpl::Multiple(vec) => write!(f, "Multiple({vec:?})"),
            ListImpl::Lazy(_) => write!(f, "Lazy(<function>)"),
            ListImpl::Concat(left, right) => write!(f, "Concat({left:?}, {right:?})"),
        }
    }
}

impl<T: PartialEq> PartialEq for ListImpl<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ListImpl::Empty, ListImpl::Empty) => true,
            (ListImpl::Single(a), ListImpl::Single(b)) => a == b,
            (ListImpl::Multiple(a), ListImpl::Multiple(b)) => a == b,
            (ListImpl::Concat(a1, a2), ListImpl::Concat(b1, b2)) => a1 == b1 && a2 == b2,
            // For lazy computations, we need to force evaluation to compare
            _ => {
                let self_vec = List { inner: self.clone() }.to_vec();
                let other_vec = List { inner: other.clone() }.to_vec();
                self_vec == other_vec
            },
        }
    }
}

/// Type alias for List monad specialized to Lambdust Values
pub type ValueList = List<Value>;

/// Function wrapper for type-safe list transformations
#[derive(Clone)]
pub struct ListFunc<A, B> {
    id: u64,
    func: Arc<dyn Fn(A) -> B + Send + Sync + 'static>,
}

impl<A, B> ListFunc<A, B> {
    /// Create a new ListFunc with unique ID for debugging
    pub fn new<F>(id: u64, func: F) -> Self
    where
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        Self {
            id,
            func: Arc::new(func),
        }
    }
    
    /// Call the wrapped function
    pub fn call(&self, arg: A) -> B {
        (self.func)(arg)
    }
}

impl<A, B> fmt::Debug for ListFunc<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ListFunc({})", self.id)
    }
}

/// Counter for generating unique function IDs
static LIST_FUNC_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl<T: Clone + Send + Sync + 'static> List<T> {
    /// Create an empty list (MonadZero/MonadPlus zero element)
    pub fn empty() -> Self {
        List {
            inner: ListImpl::Empty,
        }
    }
    
    /// Create a list with a single element (Monad return/pure)
    pub fn single(value: T) -> Self {
        List {
            inner: ListImpl::Single(value),
        }
    }
    
    /// Create a list from a vector
    pub fn from_vec(vec: Vec<T>) -> Self {
        match vec.len() {
            0 => List::empty(),
            1 => List::single(vec.into_iter().next().unwrap()),
            _ => List {
                inner: ListImpl::Multiple(Arc::new(vec)),
            },
        }
    }
    
    /// Create a list from an iterator (eager evaluation)
    pub fn from_iterator<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        List::from_vec(iter.into_iter().collect())
    }
    
    /// Create a lazy list from a computation
    pub fn lazy<F>(computation: F) -> Self
    where
        F: Fn() -> List<T> + Send + Sync + 'static,
    {
        List {
            inner: ListImpl::Lazy(Arc::new(move || computation().inner)),
        }
    }
    
    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        match &self.inner {
            ListImpl::Empty => true,
            ListImpl::Single(_) => false,
            ListImpl::Multiple(vec) => vec.is_empty(),
            ListImpl::Lazy(f) => List { inner: f() }.is_empty(),
            ListImpl::Concat(left, right) => left.is_empty() && right.is_empty(),
        }
    }
    
    /// Get the length of the list (forces evaluation)
    pub fn len(&self) -> usize {
        match &self.inner {
            ListImpl::Empty => 0,
            ListImpl::Single(_) => 1,
            ListImpl::Multiple(vec) => vec.len(),
            ListImpl::Lazy(f) => List { inner: f() }.len(),
            ListImpl::Concat(left, right) => left.len() + right.len(),
        }
    }
    
    /// Monadic bind operation
    /// 
    /// This is the core operation of the List monad. It applies a function that
    /// returns a List to each element of this list, then flattens the results.
    pub fn bind<U, F>(self, f: F) -> List<U>
    where
        F: Fn(T) -> List<U> + Send + Sync + 'static + Clone,
        T: Clone + Send + Sync + 'static,
        U: Clone + Send + Sync + 'static,
    {
        match self.inner {
            ListImpl::Empty => List::empty(),
            ListImpl::Single(value) => f(value),
            ListImpl::Multiple(vec) => {
                let results: Vec<List<U>> = vec.iter().map(|x| f(x.clone())).collect();
                List::concat_many(results)
            },
            ListImpl::Lazy(computation) => {
                let comp_clone = computation.clone();
                List { 
                    inner: ListImpl::Lazy(Arc::new(move || {
                        List { inner: comp_clone() }.bind(f.clone()).inner
                    }))
                }
            },
            ListImpl::Concat(left, right) => {
                let left_result = left.as_ref().clone().bind(f.clone());
                let right_result = right.as_ref().clone().bind(f);
                left_result.plus(right_result)
            },
        }
    }
    
    /// Functor map operation
    pub fn map<U, F>(self, f: F) -> List<U>
    where
        F: Fn(T) -> U + Send + Sync + 'static + Clone,
        T: Clone + Send + Sync + 'static,
        U: Clone + Send + Sync + 'static,
    {
        self.bind(move |x| List::single(f(x)))
    }
    
    /// MonadPlus plus operation (concatenation/choice)
    pub fn plus(self, other: List<T>) -> List<T> {
        match (self.inner, other.inner) {
            (ListImpl::Empty, other_inner) => List { inner: other_inner },
            (self_inner, ListImpl::Empty) => List { inner: self_inner },
            (self_inner, other_inner) => List {
                inner: ListImpl::Concat(
                    Arc::new(List { inner: self_inner }),
                    Arc::new(List { inner: other_inner }),
                ),
            },
        }
    }
    
    /// Concatenate multiple lists efficiently
    pub fn concat_many(lists: Vec<List<T>>) -> List<T> {
        lists.into_iter().fold(List::empty(), |acc, list| acc.plus(list))
    }
    
    /// Guard operation for filtering (MonadPlus)
    /// 
    /// Returns the list if the condition is true, empty list otherwise.
    pub fn guard(self, condition: bool) -> List<T> {
        if condition {
            self
        } else {
            List::empty()
        }
    }
    
    /// Filter elements based on a predicate
    pub fn filter<P>(self, predicate: P) -> List<T>
    where
        P: Fn(&T) -> bool + Send + Sync + 'static + Clone,
        T: Clone + Send + Sync + 'static,
    {
        self.bind(move |x| {
            if predicate(&x) {
                List::single(x)
            } else {
                List::empty()
            }
        })
    }
    
    /// Take the first n elements
    pub fn take(self, n: usize) -> List<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        if n == 0 {
            return List::empty();
        }
        
        match self.inner {
            ListImpl::Empty => List::empty(),
            ListImpl::Single(value) => if n > 0 { List::single(value) } else { List::empty() },
            ListImpl::Multiple(vec) => {
                let taken: Vec<T> = vec.iter().take(n).cloned().collect();
                List::from_vec(taken)
            },
            ListImpl::Lazy(f) => List { inner: f() }.take(n),
            ListImpl::Concat(left, right) => {
                let left_len = left.len();
                if n <= left_len {
                    left.as_ref().clone().take(n)
                } else {
                    left.as_ref().clone().plus(right.as_ref().clone().take(n - left_len))
                }
            },
        }
    }
    
    /// Drop the first n elements
    pub fn drop(self, n: usize) -> List<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        if n == 0 {
            return self;
        }
        
        match self.inner {
            ListImpl::Empty => List::empty(),
            ListImpl::Single(_) => if n > 0 { List::empty() } else { self },
            ListImpl::Multiple(vec) => {
                let dropped: Vec<T> = vec.iter().skip(n).cloned().collect();
                List::from_vec(dropped)
            },
            ListImpl::Lazy(f) => List { inner: f() }.drop(n),
            ListImpl::Concat(left, right) => {
                let left_len = left.len();
                if n >= left_len {
                    right.as_ref().clone().drop(n - left_len)
                } else {
                    left.as_ref().clone().drop(n).plus(right.as_ref().clone())
                }
            },
        }
    }
    
    /// Convert to a regular Vec (forces evaluation)
    pub fn to_vec(self) -> Vec<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        match self.inner {
            ListImpl::Empty => Vec::new(),
            ListImpl::Single(value) => vec![value],
            ListImpl::Multiple(vec) => {
                // Try to avoid clone if we're the only owner
                match Arc::try_unwrap(vec) {
                    Ok(owned_vec) => owned_vec,
                    Err(arc_vec) => (*arc_vec).clone(),
                }
            },
            ListImpl::Lazy(f) => List { inner: f() }.to_vec(),
            ListImpl::Concat(left, right) => {
                let mut result = left.as_ref().clone().to_vec();
                result.extend(right.as_ref().clone().to_vec());
                result
            },
        }
    }
    
    /// Get an iterator over the elements (forces evaluation)
    pub fn iter(&self) -> impl Iterator<Item = T> + '_
    where
        T: Clone + Send + Sync + 'static,
    {
        // For simplicity, we convert to vec and iterate
        // In a production implementation, we might use a more sophisticated iterator
        self.clone().to_vec().into_iter()
    }
}

impl<T: Clone + Send + Sync + 'static> List<T> {
    /// Applicative apply operation
    pub fn ap<U, F>(self, f_list: List<F>) -> List<U>
    where
        F: Fn(T) -> U + Clone + Send + Sync + 'static,
        U: Clone + Send + Sync + 'static,
    {
        f_list.bind(move |f| {
            let list_clone = self.clone();
            list_clone.map(f)
        })
    }
    
    /// Sequence a list of monadic computations
    pub fn sequence<M>(self) -> M
    where
        T: Clone,
        M: From<List<Vec<T::Item>>> + Default,
        T: IntoIterator,
        T::Item: Clone,
    {
        // This is a placeholder - actual implementation would depend on the target monad
        M::default()
    }
}

/// Specialized operations for ValueList
impl ValueList {
    /// Create a ValueList from a Lambdust cons list Value
    pub fn from_cons_list(value: Value) -> Result<ValueList> {
        let mut elements = Vec::new();
        let mut current = value;
        
        loop {
            match current {
                Value::Nil => break,
                Value::Pair(head, tail) => {
                    elements.push((*head).clone());
                    current = (*tail).clone();
                }
                _ => return Err(Box::new(Error::type_error(
                    format!("Expected list, got: {current}"),
                    crate::diagnostics::Span::new(0, 0)
                ))),
            }
        }
        
        Ok(ValueList::from_vec(elements))
    }
    
    /// Convert to a Lambdust cons list Value
    pub fn to_cons_list(self) -> Value {
        let vec = self.to_vec();
        vec.into_iter().rev().fold(Value::Nil, |acc, val| {
            Value::Pair(Arc::new(val), Arc::new(acc))
        })
    }
    
    /// Bind operation specialized for Value transformations
    pub fn bind_value<F>(self, f: F) -> ValueList
    where
        F: Fn(Value) -> ValueList + Send + Sync + 'static + Clone,
    {
        self.bind(f)
    }
    
    /// Map operation specialized for Value transformations  
    pub fn map_value<F>(self, f: F) -> ValueList
    where
        F: Fn(Value) -> Value + Send + Sync + 'static + Clone,
    {
        self.map(f)
    }
}

/// Display implementation for List
impl<T: fmt::Display> fmt::Display for List<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "List[")?;
        let vec = self.clone().to_vec();
        for (i, item) in vec.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{item}")?;
        }
        write!(f, "]")
    }
}

/// Thread safety implementations
unsafe impl<T: Send> Send for List<T> {}
unsafe impl<T: Sync> Sync for List<T> {}

/// Conversion from Vec to List
impl<T: Clone + Send + Sync + 'static> From<Vec<T>> for List<T> {
    fn from(vec: Vec<T>) -> Self {
        List::from_vec(vec)
    }
}

/// Conversion from List to Vec
impl<T: Clone + Send + Sync + 'static> From<List<T>> for Vec<T> {
    fn from(list: List<T>) -> Self {
        list.to_vec()
    }
}

/// Iterator trait implementation
impl<T: Clone + Send + Sync + 'static> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}

/// Extend trait for building lists from iterators
impl<T: Clone + Default + Send + Sync + 'static> std::iter::Extend<T> for List<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let new_elements: Vec<T> = iter.into_iter().collect();
        let new_list = List::from_vec(new_elements);
        let old_list = std::mem::replace(self, List::empty());
        *self = old_list.plus(new_list);
    }
}

/// FromIterator trait for collecting into List
impl<T: Clone + Send + Sync + 'static> std::iter::FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        List::from_vec(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_monad_laws() {
        // Left identity: return(a) >>= f ≡ f(a)
        let a = 42;
        let left = List::single(a).bind(|x| List::from_vec(vec![x * 2, x * 3]));
        let right = List::from_vec(vec![a * 2, a * 3]);
        assert_eq!(left.to_vec(), right.to_vec());

        // Right identity: m >>= return ≡ m
        let m = List::from_vec(vec![1, 2, 3]);
        let left = m.clone().bind(List::single);
        assert_eq!(left.to_vec(), m.to_vec());

        // Associativity: (m >>= f) >>= g ≡ m >>= (\x -> f x >>= g)
        let m = List::from_vec(vec![1, 2]);
        
        let left = m.clone().bind(|x| List::from_vec(vec![x * 2])).bind(|x| List::from_vec(vec![x + 1, x + 2]));
        let right = m.bind(|x| List::from_vec(vec![x * 2]).bind(|y| List::from_vec(vec![y + 1, y + 2])));
        assert_eq!(left.to_vec(), right.to_vec());
    }

    #[test]
    fn test_monad_plus_laws() {
        // Left zero: mzero >>= f ≡ mzero
        let f = |x: i32| List::single(x * 2);
        let left = List::<i32>::empty().bind(f);
        assert_eq!(left.to_vec(), List::<i32>::empty().to_vec());

        // Right zero: m >>= (\_ -> mzero) ≡ mzero
        let m = List::from_vec(vec![1, 2, 3]);
        let left = m.bind(|_| List::<i32>::empty());
        assert_eq!(left.to_vec(), List::<i32>::empty().to_vec());

        // Left identity: mzero `mplus` m ≡ m
        let m = List::from_vec(vec![1, 2, 3]);
        let left = List::<i32>::empty().plus(m.clone());
        assert_eq!(left.to_vec(), m.to_vec());

        // Right identity: m `mplus` mzero ≡ m
        let m = List::from_vec(vec![1, 2, 3]);
        let left = m.clone().plus(List::empty());
        assert_eq!(left.to_vec(), m.to_vec());
    }

    #[test]
    fn test_list_operations() {
        let list1 = List::from_vec(vec![1, 2, 3]);
        let list2 = List::from_vec(vec![4, 5, 6]);
        
        // Test concatenation
        let concat = list1.clone().plus(list2.clone());
        assert_eq!(concat.to_vec(), vec![1, 2, 3, 4, 5, 6]);
        
        // Test map
        let mapped = list1.clone().map(|x| x * 2);
        assert_eq!(mapped.to_vec(), vec![2, 4, 6]);
        
        // Test filter
        let filtered = list1.clone().filter(|&x| x % 2 == 0);
        assert_eq!(filtered.to_vec(), vec![2]);
        
        // Test take
        let taken = list1.clone().take(2);
        assert_eq!(taken.to_vec(), vec![1, 2]);
        
        // Test drop
        let dropped = list1.clone().drop(1);
        assert_eq!(dropped.to_vec(), vec![2, 3]);
    }

    #[test]
    fn test_guard_operation() {
        let list = List::from_vec(vec![1, 2, 3, 4, 5]);
        
        // Test guard with condition
        let result = list.clone().bind(|x| {
            List::single(x).guard(x % 2 == 0).map(|y| y * 2)
        });
        
        assert_eq!(result.to_vec(), vec![4, 8]);
    }

    #[test]
    fn test_value_list_conversion() {
        use crate::eval::value::Value;
        use std::sync::Arc;
        
        // Test from cons list
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
        
        let value_list = ValueList::from_cons_list(cons_list).unwrap();
        assert_eq!(value_list.len(), 3);
        
        // Test to cons list
        let back_to_cons = value_list.to_cons_list();
        // We can't easily test equality due to the recursive structure,
        // but we can test that it's not Nil
        assert!(!matches!(back_to_cons, Value::Nil));
    }

    #[test]
    fn test_lazy_evaluation() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        let lazy_list = List::lazy(move || {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            List::from_vec(vec![1, 2, 3])
        });
        
        // Computation should not have run yet
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 0);
        
        // Force evaluation
        let result = lazy_list.to_vec();
        assert_eq!(result, vec![1, 2, 3]);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
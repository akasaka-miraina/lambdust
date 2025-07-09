//! Equality operations for Value

use super::Value;

impl Value {
    /// Check if two values are equal
    pub fn equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Character(a), Value::Character(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Pair(pair1_ref), Value::Pair(pair2_ref)) => {
                let pair1 = pair1_ref.borrow();
                let pair2 = pair2_ref.borrow();
                pair1.car.equal(&pair2.car) && pair1.cdr.equal(&pair2.cdr)
            }
            (Value::Vector(a), Value::Vector(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equal(y))
            }
            (Value::LazyVector(a), Value::LazyVector(b)) => {
                let mut a_storage = a.borrow_mut();
                let mut b_storage = b.borrow_mut();

                if a_storage.len() != b_storage.len() {
                    return false;
                }

                // Compare elements lazily without full materialization
                for i in 0..a_storage.len() {
                    let a_val = a_storage.get(i).unwrap_or(Value::Undefined);
                    let b_val = b_storage.get(i).unwrap_or(Value::Undefined);
                    if !a_val.equal(&b_val) {
                        return false;
                    }
                }
                true
            }
            (Value::Vector(vec), Value::LazyVector(lazy))
            | (Value::LazyVector(lazy), Value::Vector(vec)) => {
                let mut lazy_storage = lazy.borrow_mut();

                if vec.len() != lazy_storage.len() {
                    return false;
                }

                // Compare vector elements with lazy vector elements
                for (i, vec_val) in vec.iter().enumerate() {
                    let lazy_val = lazy_storage.get(i).unwrap_or(Value::Undefined);
                    if !vec_val.equal(&lazy_val) {
                        return false;
                    }
                }
                true
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
            (Value::Box(a), Value::Box(b)) => a.unbox().equal(&b.unbox()),
            (Value::Comparator(a), Value::Comparator(b)) => a == b,
            (Value::StringCursor(a), Value::StringCursor(b)) => a == b,
            (Value::Ideque(a), Value::Ideque(b)) => a == b,
            (Value::Text(a), Value::Text(b)) => a.text_equal(b),
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
            (Value::Record(_), Value::Record(_)) | (Value::Values(_), Value::Values(_)) => {
                std::ptr::eq(self, other)
            }
            (Value::Box(a), Value::Box(b)) => {
                a == b // Use Box's PartialEq which checks Rc pointer equality
            }
            (Value::Comparator(a), Value::Comparator(b)) => {
                std::rc::Rc::ptr_eq(a, b) // Comparators are eqv? if same object
            }
            (Value::StringCursor(a), Value::StringCursor(b)) => {
                std::rc::Rc::ptr_eq(a, b) // String cursors are eqv? if same object
            }
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
            (Value::Record(_), Value::Record(_)) | (Value::Values(_), Value::Values(_)) => {
                std::ptr::eq(self, other)
            }
            _ => std::ptr::eq(self, other),
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
            (Self::Pair(l_pair), Self::Pair(r_pair)) => {
                let l_borrow = l_pair.borrow();
                let r_borrow = r_pair.borrow();
                l_borrow.car == r_borrow.car && l_borrow.cdr == r_borrow.cdr
            }
            (Self::Procedure(l0), Self::Procedure(r0)) => l0 == r0,
            (Self::Vector(l0), Self::Vector(r0)) => l0 == r0,
            (Self::LazyVector(l0), Self::LazyVector(r0)) => {
                // For PartialEq, we use pointer equality for lazy vectors
                // to avoid expensive materialization
                std::rc::Rc::ptr_eq(l0, r0)
            }
            (Self::Vector(_), Self::LazyVector(_)) | (Self::LazyVector(_), Self::Vector(_)) => {
                false
            }
            (Self::Port(l0), Self::Port(r0)) => l0 == r0,
            (Self::External(l0), Self::External(r0)) => l0.id == r0.id,
            (Self::Record(l0), Self::Record(r0)) => l0 == r0,
            (Self::Values(l0), Self::Values(r0)) => l0 == r0,
            (Self::Continuation(_), Self::Continuation(_)) => false, // Continuations are never equal
            (Self::Promise(_), Self::Promise(_)) => false, // Promises are never equal per SRFI 45
            (Self::Box(l0), Self::Box(r0)) => l0 == r0,
            (Self::Comparator(l0), Self::Comparator(r0)) => l0 == r0,
            (Self::StringCursor(l0), Self::StringCursor(r0)) => l0 == r0,
            (Self::Ideque(l0), Self::Ideque(r0)) => l0 == r0,
            (Self::Text(l0), Self::Text(r0)) => l0.text_equal(r0),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

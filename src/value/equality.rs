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
            (Value::Record(_), Value::Record(_)) | (Value::Values(_), Value::Values(_)) => {
                std::ptr::eq(self, other)
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

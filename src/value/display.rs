//! Display and Debug implementations for Value

use super::{Procedure, Value};
use std::fmt;

impl Value {
    /// Display a pair value
    fn display_pair(&self, f: &mut fmt::Formatter<'_>, pair_ref: &std::rc::Rc<std::cell::RefCell<crate::value::pair::PairData>>) -> fmt::Result {
        write!(f, "(")?;
        let pair = pair_ref.borrow();
        write!(f, "{}", pair.car)?;

        let mut current = pair.cdr.clone();
        loop {
            match current {
                Value::Nil => break,
                Value::Pair(inner_pair_ref) => {
                    let inner_pair = inner_pair_ref.borrow();
                    write!(f, " {}", inner_pair.car)?;
                    current = inner_pair.cdr.clone();
                }
                _ => {
                    write!(f, " . {current}")?;
                    break;
                }
            }
        }
        write!(f, ")")
    }

    /// Display a procedure value
    fn display_procedure(&self, f: &mut fmt::Formatter<'_>, proc: &Procedure) -> fmt::Result {
        match proc {
            Procedure::Lambda {
                params, variadic, ..
            } => {
                write!(f, "#<procedure (")?;
                self.display_lambda_params(f, params, *variadic)?;
                write!(f, ")>")
            }
            Procedure::Builtin { name, .. } => write!(f, "#<builtin {name}>"),
            Procedure::HostFunction { name, .. } => write!(f, "#<host-function {name}>"),
            Procedure::Continuation { .. } => write!(f, "#<continuation>"),
            Procedure::CapturedContinuation { .. } => write!(f, "#<continuation>"),
            Procedure::ReusableContinuation { reuse_id, .. } => write!(f, "#<reusable-continuation:{}>", reuse_id),
        }
    }

    /// Display lambda parameters
    fn display_lambda_params(&self, f: &mut fmt::Formatter<'_>, params: &[String], variadic: bool) -> fmt::Result {
        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{param}")?;
        }
        if variadic {
            write!(f, " ...")?;
        }
        Ok(())
    }

    /// Display a vector value
    fn display_vector(&self, f: &mut fmt::Formatter<'_>, values: &[Value]) -> fmt::Result {
        write!(f, "#(")?;
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{value}")?;
        }
        write!(f, ")")
    }

    /// Display values (multiple values)
    fn display_values(&self, f: &mut fmt::Formatter<'_>, values: &[Value]) -> fmt::Result {
        if values.len() == 1 {
            write!(f, "{}", values[0])
        } else {
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
            Value::Pair(pair_ref) => self.display_pair(f, pair_ref),
            Value::Nil => write!(f, "()"),
            Value::Procedure(proc) => self.display_procedure(f, proc),
            Value::Vector(values) => self.display_vector(f, values),
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
            Value::Values(values) => self.display_values(f, values),
            Value::Continuation(_) => write!(f, "#<continuation>"),
            Value::Promise(promise) => match &promise.state {
                crate::value::PromiseState::Lazy { .. } => write!(f, "#<promise:lazy>"),
                crate::value::PromiseState::Eager { value } => {
                    write!(f, "#<promise:eager:{}>", value)
                }
            },
            Value::HashTable(ht) => {
                let table = ht.borrow();
                write!(f, "#<hash-table size:{}>", table.size())
            }
            Value::Box(box_val) => {
                write!(f, "#<box:{}>", box_val.unbox())
            }
            Value::Comparator(comp) => {
                write!(f, "#<comparator:{}>", comp.name)
            }
            Value::StringCursor(cursor) => {
                write!(f, "#<string-cursor:{}:{}>", cursor.position(), cursor.string().len())
            }
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
            Self::Pair(pair_ref) => {
                let pair = pair_ref.borrow();
                f.debug_tuple("Pair")
                    .field(&pair.car)
                    .field(&pair.cdr)
                    .finish()
            }
            Self::Nil => write!(f, "Nil"),
            Self::Procedure(arg0) => f.debug_tuple("Procedure").field(arg0).finish(),
            Self::Vector(arg0) => f.debug_tuple("Vector").field(arg0).finish(),
            Self::Port(arg0) => f.debug_tuple("Port").field(arg0).finish(),
            Self::External(arg0) => f.debug_tuple("External").field(arg0).finish(),
            Self::Record(arg0) => f.debug_tuple("Record").field(arg0).finish(),
            Self::Values(arg0) => f.debug_tuple("Values").field(arg0).finish(),
            Self::HashTable(arg0) => f.debug_tuple("HashTable").field(arg0).finish(),
            Self::Continuation(arg0) => f.debug_tuple("Continuation").field(arg0).finish(),
            Self::Promise(arg0) => f.debug_tuple("Promise").field(arg0).finish(),
            Self::Box(arg0) => f.debug_tuple("Box").field(arg0).finish(),
            Self::Comparator(arg0) => f.debug_tuple("Comparator").field(&arg0.name).finish(),
            Self::StringCursor(arg0) => f.debug_tuple("StringCursor").field(&arg0.position()).finish(),
        }
    }
}

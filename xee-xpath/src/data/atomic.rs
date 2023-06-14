use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

use crate::comparison;

use super::error::ValueError;
use crate::stack::Atomic;

type Result<T> = std::result::Result<T, ValueError>;

#[derive(Debug, Clone)]
pub enum OutputAtomic {
    Boolean(bool),
    Integer(i64),
    Float(f32),
    Double(f64),
    Decimal(Decimal),
    String(String),
    Untyped(String),
    // a special marker to note empty sequences after atomization
    // This should be treated as an emtpy sequence.
    Empty,
    // a special marker to indicate an absent context item
    Absent,
}

impl From<OutputAtomic> for Atomic {
    fn from(a: OutputAtomic) -> Self {
        (&a).into()
    }
}

impl From<&OutputAtomic> for Atomic {
    fn from(a: &OutputAtomic) -> Self {
        match a {
            OutputAtomic::Boolean(b) => Atomic::Boolean(*b),
            OutputAtomic::Integer(i) => Atomic::Integer(*i),
            OutputAtomic::Float(f) => Atomic::Float(OrderedFloat(*f)),
            OutputAtomic::Double(d) => Atomic::Double(OrderedFloat(*d)),
            OutputAtomic::Decimal(d) => Atomic::Decimal(*d),
            OutputAtomic::String(s) => Atomic::String(Rc::new(s.clone())),
            OutputAtomic::Untyped(s) => Atomic::Untyped(Rc::new(s.clone())),
            OutputAtomic::Empty => Atomic::Empty,
            OutputAtomic::Absent => Atomic::Absent,
        }
    }
}

impl Display for OutputAtomic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OutputAtomic::Boolean(b) => write!(f, "{}", b),
            OutputAtomic::Integer(i) => write!(f, "{}", i),
            OutputAtomic::Float(n) => write!(f, "{}", n),
            OutputAtomic::Double(d) => write!(f, "{}", d),
            OutputAtomic::Decimal(d) => write!(f, "{}", d),
            OutputAtomic::String(s) => write!(f, "{}", s),
            OutputAtomic::Untyped(s) => write!(f, "{}", s),
            OutputAtomic::Empty => write!(f, "()"),
            OutputAtomic::Absent => write!(f, "absent"),
        }
    }
}

impl OutputAtomic {
    pub(crate) fn to_integer(&self) -> Result<i64> {
        match self {
            OutputAtomic::Integer(i) => Ok(*i),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_decimal(&self) -> Result<Decimal> {
        match self {
            OutputAtomic::Decimal(d) => Ok(*d),
            OutputAtomic::Integer(i) => Ok(Decimal::from(*i)),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_float(&self) -> Result<f32> {
        match self {
            OutputAtomic::Float(f) => Ok(*f),
            OutputAtomic::Decimal(d) => Ok(d.to_f32().ok_or(ValueError::Type)?),
            OutputAtomic::Integer(_) => Ok(self.to_decimal()?.to_f32().ok_or(ValueError::Type)?),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_double(&self) -> Result<f64> {
        match self {
            OutputAtomic::Double(d) => Ok(*d),
            OutputAtomic::Float(f) => Ok(*f as f64),
            OutputAtomic::Decimal(d) => Ok(d.to_f64().ok_or(ValueError::Type)?),
            OutputAtomic::Integer(_) => Ok(self.to_decimal()?.to_f64().ok_or(ValueError::Type)?),
            _ => Err(ValueError::Type),
        }
    }

    pub(crate) fn to_bool(&self) -> Result<bool> {
        match self {
            OutputAtomic::Integer(i) => Ok(*i != 0),
            OutputAtomic::Decimal(d) => Ok(!d.is_zero()),
            OutputAtomic::Float(f) => Ok(!f.is_zero()),
            OutputAtomic::Double(d) => Ok(!d.is_zero()),
            OutputAtomic::Boolean(b) => Ok(*b),
            OutputAtomic::String(s) => Ok(!s.is_empty()),
            OutputAtomic::Untyped(s) => Ok(!s.is_empty()),
            OutputAtomic::Empty => Ok(false),
            OutputAtomic::Absent => Err(ValueError::Absent),
        }
    }

    // XXX is this named right? It's consistent with  to_double, to_bool, etc,
    // but inconsistent with the to_string Rust convention
    pub fn to_str(&self) -> Result<&str> {
        match self {
            OutputAtomic::String(s) => Ok(s),
            _ => Err(ValueError::Type),
        }
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(self.to_str()?.to_string())
    }

    pub fn string_value(&self) -> Result<String> {
        Ok(match self {
            OutputAtomic::String(s) => s.to_string(),
            OutputAtomic::Untyped(s) => s.to_string(),
            OutputAtomic::Boolean(b) => b.to_string(),
            OutputAtomic::Integer(i) => i.to_string(),
            OutputAtomic::Float(f) => f.to_string(),
            OutputAtomic::Double(d) => d.to_string(),
            OutputAtomic::Decimal(d) => d.to_string(),
            OutputAtomic::Empty => "".to_string(),
            OutputAtomic::Absent => Err(ValueError::Absent)?,
        })
    }
}

impl PartialEq for OutputAtomic {
    fn eq(&self, other: &Self) -> bool {
        match comparison::value_eq(&self.into(), &other.into()) {
            Ok(b) => b.to_bool().unwrap(),
            Err(_) => false,
        }
    }
}

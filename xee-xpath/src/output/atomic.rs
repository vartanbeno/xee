use ordered_float::OrderedFloat;
use rust_decimal::prelude::*;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

use crate::comparison;
use crate::stack;

#[derive(Debug, Clone)]
pub enum Atomic {
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

impl From<Atomic> for stack::Atomic {
    fn from(a: Atomic) -> Self {
        (&a).into()
    }
}

impl From<&Atomic> for stack::Atomic {
    fn from(a: &Atomic) -> Self {
        match a {
            Atomic::Boolean(b) => stack::Atomic::Boolean(*b),
            Atomic::Integer(i) => stack::Atomic::Integer(*i),
            Atomic::Float(f) => stack::Atomic::Float(OrderedFloat(*f)),
            Atomic::Double(d) => stack::Atomic::Double(OrderedFloat(*d)),
            Atomic::Decimal(d) => stack::Atomic::Decimal(*d),
            Atomic::String(s) => stack::Atomic::String(Rc::new(s.clone())),
            Atomic::Untyped(s) => stack::Atomic::Untyped(Rc::new(s.clone())),
            Atomic::Empty => stack::Atomic::Empty,
            Atomic::Absent => stack::Atomic::Absent,
        }
    }
}

impl Display for Atomic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Atomic::Boolean(b) => write!(f, "{}", b),
            Atomic::Integer(i) => write!(f, "{}", i),
            Atomic::Float(n) => write!(f, "{}", n),
            Atomic::Double(d) => write!(f, "{}", d),
            Atomic::Decimal(d) => write!(f, "{}", d),
            Atomic::String(s) => write!(f, "{}", s),
            Atomic::Untyped(s) => write!(f, "{}", s),
            Atomic::Empty => write!(f, "()"),
            Atomic::Absent => write!(f, "absent"),
        }
    }
}

impl Atomic {
    pub(crate) fn to_integer(&self) -> stack::ValueResult<i64> {
        match self {
            Atomic::Integer(i) => Ok(*i),
            _ => Err(stack::ValueError::Type),
        }
    }

    pub(crate) fn to_decimal(&self) -> stack::ValueResult<Decimal> {
        match self {
            Atomic::Decimal(d) => Ok(*d),
            Atomic::Integer(i) => Ok(Decimal::from(*i)),
            _ => Err(stack::ValueError::Type),
        }
    }

    pub(crate) fn to_float(&self) -> stack::ValueResult<f32> {
        match self {
            Atomic::Float(f) => Ok(*f),
            Atomic::Decimal(d) => Ok(d.to_f32().ok_or(stack::ValueError::Type)?),
            Atomic::Integer(_) => Ok(self.to_decimal()?.to_f32().ok_or(stack::ValueError::Type)?),
            _ => Err(stack::ValueError::Type),
        }
    }

    pub(crate) fn to_double(&self) -> stack::ValueResult<f64> {
        match self {
            Atomic::Double(d) => Ok(*d),
            Atomic::Float(f) => Ok(*f as f64),
            Atomic::Decimal(d) => Ok(d.to_f64().ok_or(stack::ValueError::Type)?),
            Atomic::Integer(_) => Ok(self.to_decimal()?.to_f64().ok_or(stack::ValueError::Type)?),
            _ => Err(stack::ValueError::Type),
        }
    }

    pub(crate) fn to_bool(&self) -> stack::ValueResult<bool> {
        match self {
            Atomic::Integer(i) => Ok(*i != 0),
            Atomic::Decimal(d) => Ok(!d.is_zero()),
            Atomic::Float(f) => Ok(!f.is_zero()),
            Atomic::Double(d) => Ok(!d.is_zero()),
            Atomic::Boolean(b) => Ok(*b),
            Atomic::String(s) => Ok(!s.is_empty()),
            Atomic::Untyped(s) => Ok(!s.is_empty()),
            Atomic::Empty => Ok(false),
            Atomic::Absent => Err(stack::ValueError::Absent),
        }
    }

    // XXX is this named right? It's consistent with  to_double, to_bool, etc,
    // but inconsistent with the to_string Rust convention
    pub fn to_str(&self) -> stack::ValueResult<&str> {
        match self {
            Atomic::String(s) => Ok(s),
            _ => Err(stack::ValueError::Type),
        }
    }

    pub fn to_string(&self) -> stack::ValueResult<String> {
        Ok(self.to_str()?.to_string())
    }

    pub fn string_value(&self) -> stack::ValueResult<String> {
        Ok(match self {
            Atomic::String(s) => s.to_string(),
            Atomic::Untyped(s) => s.to_string(),
            Atomic::Boolean(b) => b.to_string(),
            Atomic::Integer(i) => i.to_string(),
            Atomic::Float(f) => f.to_string(),
            Atomic::Double(d) => d.to_string(),
            Atomic::Decimal(d) => d.to_string(),
            Atomic::Empty => "".to_string(),
            Atomic::Absent => Err(stack::ValueError::Absent)?,
        })
    }
}

impl PartialEq for Atomic {
    fn eq(&self, other: &Self) -> bool {
        match comparison::value_eq(&self.into(), &other.into()) {
            Ok(b) => b.to_bool().unwrap(),
            Err(_) => false,
        }
    }
}

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
        }
    }
}

impl Atomic {
    pub(crate) fn to_bool(&self) -> stack::Result<bool> {
        match self {
            Atomic::Integer(i) => Ok(*i != 0),
            Atomic::Decimal(d) => Ok(!d.is_zero()),
            Atomic::Float(f) => Ok(!f.is_zero()),
            Atomic::Double(d) => Ok(!d.is_zero()),
            Atomic::Boolean(b) => Ok(*b),
            Atomic::String(s) => Ok(!s.is_empty()),
            Atomic::Untyped(s) => Ok(!s.is_empty()),
        }
    }

    pub fn to_str(&self) -> stack::Result<&str> {
        match self {
            Atomic::String(s) => Ok(s),
            _ => Err(stack::Error::Type),
        }
    }

    pub fn to_string(&self) -> stack::Result<String> {
        Ok(self.to_str()?.to_string())
    }

    pub fn string_value(&self) -> stack::Result<String> {
        Ok(match self {
            Atomic::String(s) => s.to_string(),
            Atomic::Untyped(s) => s.to_string(),
            Atomic::Boolean(b) => b.to_string(),
            Atomic::Integer(i) => i.to_string(),
            Atomic::Float(f) => f.to_string(),
            Atomic::Double(d) => d.to_string(),
            Atomic::Decimal(d) => d.to_string(),
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

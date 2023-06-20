use ordered_float::OrderedFloat;
use rust_decimal::Decimal;
use std::fmt::{self, Display, Formatter};

use crate::error;
use crate::stack;

#[derive(Debug, Clone, PartialEq)]
pub struct Atomic {
    pub(crate) stack_atomic: stack::Atomic,
}

#[derive(Debug)]
pub enum AtomicValue {
    Boolean(bool),
    Integer(i64),
    Float(f32),
    Double(f64),
    Decimal(Decimal),
    String(String),
    Untyped(String),
}

impl Display for Atomic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.stack_atomic {
            stack::Atomic::Boolean(b) => write!(f, "{}", b),
            stack::Atomic::Integer(i) => write!(f, "{}", i),
            stack::Atomic::Float(n) => write!(f, "{}", n),
            stack::Atomic::Double(d) => write!(f, "{}", d),
            stack::Atomic::Decimal(d) => write!(f, "{}", d),
            stack::Atomic::String(s) => write!(f, "{}", s),
            stack::Atomic::Untyped(s) => write!(f, "{}", s),
            _ => unreachable!("Cannot exists in output space"),
        }
    }
}

impl Atomic {
    pub(crate) fn new(stack_atomic: stack::Atomic) -> Self {
        Self { stack_atomic }
    }

    pub fn from_value(value: AtomicValue) -> Self {
        match value {
            AtomicValue::Boolean(b) => Self::new(stack::Atomic::Boolean(b)),
            AtomicValue::Integer(i) => Self::new(stack::Atomic::Integer(i)),
            AtomicValue::Float(n) => Self::new(stack::Atomic::Float(OrderedFloat(n))),
            AtomicValue::Double(d) => Self::new(stack::Atomic::Double(OrderedFloat(d))),
            AtomicValue::Decimal(d) => Self::new(stack::Atomic::Decimal(d)),
            AtomicValue::String(s) => Self::new(stack::Atomic::String(s.into())),
            AtomicValue::Untyped(s) => Self::new(stack::Atomic::Untyped(s.into())),
        }
    }

    pub fn value(&self) -> AtomicValue {
        match &self.stack_atomic {
            stack::Atomic::Boolean(b) => AtomicValue::Boolean(*b),
            stack::Atomic::Integer(i) => AtomicValue::Integer(*i),
            stack::Atomic::Float(OrderedFloat(n)) => AtomicValue::Float(*n),
            stack::Atomic::Double(OrderedFloat(d)) => AtomicValue::Double(*d),
            stack::Atomic::Decimal(d) => AtomicValue::Decimal(*d),
            stack::Atomic::String(s) => AtomicValue::String(s.to_string()),
            stack::Atomic::Untyped(s) => AtomicValue::Untyped(s.to_string()),
            stack::Atomic::Empty => unreachable!("Cannot exists in output space"),
            stack::Atomic::Absent => unreachable!("Cannot exists in output space"),
        }
    }

    pub fn to_bool(&self) -> error::Result<bool> {
        if let stack::Atomic::Boolean(b) = self.stack_atomic {
            Ok(b)
        } else {
            Err(error::Error::XPTY0004A)
        }
    }

    pub fn to_integer(&self) -> error::Result<i64> {
        if let stack::Atomic::Integer(i) = self.stack_atomic {
            Ok(i)
        } else {
            Err(error::Error::XPTY0004A)
        }
    }

    pub fn to_float(&self) -> error::Result<f32> {
        if let stack::Atomic::Float(OrderedFloat(n)) = self.stack_atomic {
            Ok(n)
        } else {
            Err(error::Error::XPTY0004A)
        }
    }

    pub fn to_double(&self) -> error::Result<f64> {
        if let stack::Atomic::Double(OrderedFloat(d)) = self.stack_atomic {
            Ok(d)
        } else {
            Err(error::Error::XPTY0004A)
        }
    }

    pub fn to_decimal(&self) -> error::Result<Decimal> {
        if let stack::Atomic::Decimal(d) = self.stack_atomic {
            Ok(d)
        } else {
            Err(error::Error::XPTY0004A)
        }
    }

    pub fn to_str(&self) -> error::Result<&str> {
        Ok(self.stack_atomic.to_str()?)
    }

    pub fn to_string(&self) -> error::Result<String> {
        Ok(self.to_str()?.to_string())
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self.stack_atomic, stack::Atomic::Boolean(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.stack_atomic, stack::Atomic::Integer(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self.stack_atomic, stack::Atomic::Float(_))
    }

    pub fn is_double(&self) -> bool {
        matches!(self.stack_atomic, stack::Atomic::Double(_))
    }

    pub fn is_decimal(&self) -> bool {
        matches!(
            self.stack_atomic,
            stack::Atomic::Decimal(_) | stack::Atomic::Integer(_)
        )
    }

    pub fn is_string(&self) -> bool {
        matches!(self.stack_atomic, stack::Atomic::String(_))
    }

    pub fn string_value(&self) -> error::Result<String> {
        Ok(self.stack_atomic.string_value()?)
    }
}

impl From<bool> for Atomic {
    fn from(b: bool) -> Self {
        Self::new(stack::Atomic::Boolean(b))
    }
}

impl From<i64> for Atomic {
    fn from(i: i64) -> Self {
        Self::new(stack::Atomic::Integer(i))
    }
}

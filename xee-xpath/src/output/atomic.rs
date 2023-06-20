use ordered_float::OrderedFloat;
use rust_decimal::Decimal;
use std::fmt::{self, Display, Formatter};

use crate::error;
use crate::stack;

#[derive(Debug, Clone, PartialEq)]
pub struct Atomic {
    pub(crate) stack_atomic: stack::Atomic,
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

impl TryFrom<Atomic> for bool {
    type Error = error::Error;
    fn try_from(a: Atomic) -> error::Result<Self> {
        a.to_bool()
    }
}

impl From<bool> for Atomic {
    fn from(b: bool) -> Self {
        Self::new(stack::Atomic::Boolean(b))
    }
}

impl TryFrom<Atomic> for i64 {
    type Error = error::Error;
    fn try_from(a: Atomic) -> error::Result<Self> {
        a.to_integer()
    }
}

impl From<i64> for Atomic {
    fn from(i: i64) -> Self {
        Self::new(stack::Atomic::Integer(i))
    }
}

impl TryFrom<Atomic> for f32 {
    type Error = error::Error;
    fn try_from(a: Atomic) -> error::Result<Self> {
        a.to_float()
    }
}

impl From<f32> for Atomic {
    fn from(n: f32) -> Self {
        Self::new(stack::Atomic::Float(OrderedFloat(n)))
    }
}

impl TryFrom<Atomic> for f64 {
    type Error = error::Error;
    fn try_from(a: Atomic) -> error::Result<Self> {
        a.to_double()
    }
}

impl From<f64> for Atomic {
    fn from(d: f64) -> Self {
        Self::new(stack::Atomic::Double(OrderedFloat(d)))
    }
}

impl TryFrom<Atomic> for Decimal {
    type Error = error::Error;
    fn try_from(a: Atomic) -> error::Result<Self> {
        a.to_decimal()
    }
}

impl From<Decimal> for Atomic {
    fn from(d: Decimal) -> Self {
        Self::new(stack::Atomic::Decimal(d))
    }
}

impl TryFrom<Atomic> for String {
    type Error = error::Error;
    fn try_from(a: Atomic) -> error::Result<Self> {
        a.to_string()
    }
}

impl From<String> for Atomic {
    fn from(s: String) -> Self {
        Self::new(stack::Atomic::String(s.into()))
    }
}

impl From<&str> for Atomic {
    fn from(s: &str) -> Self {
        Self::new(stack::Atomic::String(s.to_string().into()))
    }
}

use ordered_float::OrderedFloat;
use rust_decimal::Decimal;
use std::fmt::{self, Display, Formatter};

use crate::atomic;
use crate::error;

// TODO: output::Atomic isn't pulling its weight and could simply be
// the same as Atomic

#[derive(Debug, Clone, PartialEq)]
pub struct Atomic {
    pub(crate) stack_atomic: atomic::Atomic,
}

impl Display for Atomic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.stack_atomic {
            atomic::Atomic::Boolean(b) => write!(f, "{}", b),
            atomic::Atomic::Integer(i) => write!(f, "{}", i),
            atomic::Atomic::Float(n) => write!(f, "{}", n),
            atomic::Atomic::Double(d) => write!(f, "{}", d),
            atomic::Atomic::Decimal(d) => write!(f, "{}", d),
            atomic::Atomic::String(s) => write!(f, "{}", s),
            atomic::Atomic::Untyped(s) => write!(f, "{}", s),
            _ => unreachable!("Cannot exist in output space"),
        }
    }
}

impl Atomic {
    pub(crate) fn new(stack_atomic: atomic::Atomic) -> Self {
        Self { stack_atomic }
    }

    pub fn to_bool(&self) -> error::Result<bool> {
        if let atomic::Atomic::Boolean(b) = self.stack_atomic {
            Ok(b)
        } else {
            Err(error::Error::XPTY0004A)
        }
    }

    pub fn to_integer(&self) -> error::Result<i64> {
        if let atomic::Atomic::Integer(i) = self.stack_atomic {
            Ok(i)
        } else {
            Err(error::Error::XPTY0004A)
        }
    }

    pub fn to_float(&self) -> error::Result<f32> {
        if let atomic::Atomic::Float(OrderedFloat(n)) = self.stack_atomic {
            Ok(n)
        } else {
            Err(error::Error::XPTY0004A)
        }
    }

    pub fn to_double(&self) -> error::Result<f64> {
        if let atomic::Atomic::Double(OrderedFloat(d)) = self.stack_atomic {
            Ok(d)
        } else {
            Err(error::Error::XPTY0004A)
        }
    }

    pub fn to_decimal(&self) -> error::Result<Decimal> {
        if let atomic::Atomic::Decimal(d) = self.stack_atomic {
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
        matches!(self.stack_atomic, atomic::Atomic::Boolean(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.stack_atomic, atomic::Atomic::Integer(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self.stack_atomic, atomic::Atomic::Float(_))
    }

    pub fn is_double(&self) -> bool {
        matches!(self.stack_atomic, atomic::Atomic::Double(_))
    }

    pub fn is_decimal(&self) -> bool {
        matches!(
            self.stack_atomic,
            atomic::Atomic::Decimal(_) | atomic::Atomic::Integer(_)
        )
    }

    pub fn is_string(&self) -> bool {
        matches!(self.stack_atomic, atomic::Atomic::String(_))
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

impl<T> From<T> for Atomic
where
    T: Into<atomic::Atomic>,
{
    fn from(t: T) -> Self {
        Self::new(t.into())
    }
}

impl TryFrom<Atomic> for i64 {
    type Error = error::Error;
    fn try_from(a: Atomic) -> error::Result<Self> {
        a.to_integer()
    }
}

impl TryFrom<Atomic> for f32 {
    type Error = error::Error;
    fn try_from(a: Atomic) -> error::Result<Self> {
        a.to_float()
    }
}

impl TryFrom<Atomic> for f64 {
    type Error = error::Error;
    fn try_from(a: Atomic) -> error::Result<Self> {
        a.to_double()
    }
}

impl TryFrom<Atomic> for Decimal {
    type Error = error::Error;
    fn try_from(a: Atomic) -> error::Result<Self> {
        a.to_decimal()
    }
}

impl TryFrom<Atomic> for String {
    type Error = error::Error;
    fn try_from(a: Atomic) -> error::Result<Self> {
        a.to_string()
    }
}

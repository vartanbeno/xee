use std::convert::TryFrom;
use std::rc::Rc;
use std::vec::Vec;

use ordered_float::OrderedFloat;
use rust_decimal::Decimal;

use super::{Atomic, Closure, Node, Sequence, Step, Value, ValueError};
use crate::context::DynamicContext;

type Result<T> = std::result::Result<T, ValueError>;

// wrapper should generate:
// Value -> i64
// Value -> &str
// String -> Value

pub(crate) trait ContextFrom<T>: Sized {
    fn context_from(value: T, context: &DynamicContext) -> Self;
}

pub(crate) trait ContextTryFrom<T>: Sized {
    fn context_try_from(value: T, context: &DynamicContext) -> Result<Self>;
}

pub(crate) trait ContextInto<T>: Sized {
    fn context_into(self, context: &DynamicContext) -> T;
}

pub(crate) trait ContextTryInto<T>: Sized {
    fn context_try_into(self, context: &DynamicContext) -> Result<T>;
}

impl<T, U> ContextInto<U> for T
where
    U: ContextFrom<T>,
{
    fn context_into(self, context: &DynamicContext) -> U {
        U::context_from(self, context)
    }
}

impl<T, U> ContextTryInto<U> for T
where
    U: ContextTryFrom<T>,
{
    fn context_try_into(self, context: &DynamicContext) -> Result<U> {
        U::context_try_from(self, context)
    }
}

// Conversions from Value

impl ContextTryFrom<Value> for Atomic {
    fn context_try_from(value: Value, context: &DynamicContext) -> Result<Self> {
        ContextTryFrom::context_try_from(&value, context)
    }
}

impl ContextTryFrom<&Value> for Atomic {
    fn context_try_from(value: &Value, context: &DynamicContext) -> Result<Self> {
        match value {
            Value::Atomic(a) => Ok(a.clone()),
            Value::Sequence(s) => s.borrow().to_atomic(context),
            _ => {
                todo!("don't know how to atomize this yet")
            }
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a Closure {
    type Error = ValueError;

    fn try_from(value: &'a Value) -> Result<&'a Closure> {
        match value {
            Value::Closure(c) => Ok(c),
            _ => Err(ValueError::Type),
        }
    }
}

impl TryFrom<&Value> for Rc<Step> {
    type Error = ValueError;

    fn try_from(value: &Value) -> Result<Rc<Step>> {
        match value {
            Value::Step(s) => Ok(Rc::clone(s)),
            _ => Err(ValueError::Type),
        }
    }
}

impl TryFrom<Value> for Node {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Node> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&Value> for Node {
    type Error = ValueError;

    fn try_from(value: &Value) -> Result<Node> {
        match value {
            Value::Node(n) => Ok(*n),
            Value::Sequence(s) => s.borrow().singleton().and_then(|n| n.to_node()),
            _ => Err(ValueError::Type),
        }
    }
}

// Conversions from Atomic

impl TryFrom<Atomic> for i64 {
    type Error = ValueError;

    fn try_from(atomic: Atomic) -> Result<Self> {
        atomic.to_integer()
    }
}

impl TryFrom<Atomic> for Decimal {
    type Error = ValueError;

    fn try_from(atomic: Atomic) -> Result<Self> {
        atomic.to_decimal()
    }
}

impl TryFrom<Atomic> for OrderedFloat<f32> {
    type Error = ValueError;

    fn try_from(atomic: Atomic) -> Result<Self> {
        atomic.to_float()
    }
}

impl TryFrom<Atomic> for OrderedFloat<f64> {
    type Error = ValueError;

    fn try_from(atomic: Atomic) -> Result<Self> {
        atomic.to_double()
    }
}

impl TryFrom<Atomic> for bool {
    type Error = ValueError;

    fn try_from(atomic: Atomic) -> Result<Self> {
        atomic.to_bool()
    }
}

impl<'a> TryFrom<&'a Atomic> for &'a str {
    type Error = ValueError;

    fn try_from(atomic: &'a Atomic) -> Result<&'a str> {
        atomic.to_str()
    }
}

impl TryFrom<Atomic> for String {
    type Error = ValueError;

    fn try_from(atomic: Atomic) -> Result<Self> {
        atomic.to_string()
    }
}

impl TryFrom<Value> for Sequence {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&Value> for Sequence {
    type Error = ValueError;

    fn try_from(value: &Value) -> Result<Self> {
        match value {
            Value::Sequence(s) => Ok(s.clone()),
            Value::Atomic(a) => Ok(Sequence::from_atomic(a)),
            Value::Node(n) => Ok(Sequence::from_node(*n)),
            _ => Err(ValueError::Type),
        }
    }
}

impl ContextFrom<Sequence> for Vec<Atomic> {
    fn context_from(sequence: Sequence, context: &DynamicContext) -> Self {
        sequence.borrow().to_atoms(context.xot)
    }
}

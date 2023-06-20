use ordered_float::OrderedFloat;
use rust_decimal::Decimal;
use std::convert::TryFrom;
use std::rc::Rc;

use crate::context::{ContextTryFrom, DynamicContext};
use crate::stack;
use crate::xml;

// Conversions from Value

// impl TryFrom<&Value> for Item {
//     type Error = stack::ValueError;

//     fn try_from(value: &Value) -> stack::ValueResult<Item> {
//         match value {
//             Value::stack::Atomic(a) => Ok(Item::stack::Atomic(a.clone())),
//             Value::xml::Node(n) => Ok(Item::xml::Node(*n)),
//             Value::Sequence(s) => s.borrow().singleton(),
//             Value::Closure(c) => Ok(Item::Closure(Rc::clone(c))),
//         }
//     }
// }
// Conversions from Item

impl TryFrom<&stack::Item> for stack::Atomic {
    type Error = stack::Error;

    fn try_from(item: &stack::Item) -> stack::Result<Self> {
        match item {
            stack::Item::Atomic(a) => Ok(a.clone()),
            _ => Err(stack::Error::Type),
        }
    }
}

impl TryFrom<stack::Item> for stack::Atomic {
    type Error = stack::Error;

    fn try_from(item: stack::Item) -> stack::Result<Self> {
        match item {
            stack::Item::Atomic(a) => Ok(a),
            _ => Err(stack::Error::Type),
        }
    }
}

impl TryFrom<stack::Item> for f64 {
    type Error = stack::Error;

    fn try_from(item: stack::Item) -> stack::Result<Self> {
        match item {
            stack::Item::Atomic(a) => a.try_into(),
            _ => Err(stack::Error::Type),
        }
    }
}

// impl<T> ContextTryFrom<Option<Item>> for Option<T>
// where
//     T: TryFrom<T, Error = stack::ValueError>,
// {
//     fn context_try_from(item: Option<Item>, context: &DynamicContext) -> stack::ValueResult<Option<T>> {
//         match item {
//             Some(i) => Ok(Some(i.try_into()?)),
//             None => Ok(None),
//         }
//     }
// }

impl TryFrom<stack::Item> for xml::Node {
    type Error = stack::Error;

    fn try_from(item: stack::Item) -> stack::Result<Self> {
        match item {
            stack::Item::Node(n) => Ok(n),
            _ => Err(stack::Error::Type),
        }
    }
}

// Conversions from stack::Atomic

impl TryFrom<stack::Atomic> for i64 {
    type Error = stack::Error;

    fn try_from(atomic: stack::Atomic) -> stack::Result<Self> {
        atomic.to_integer()
    }
}

impl TryFrom<stack::Atomic> for Decimal {
    type Error = stack::Error;

    fn try_from(atomic: stack::Atomic) -> stack::Result<Self> {
        atomic.to_decimal()
    }
}

impl TryFrom<stack::Atomic> for OrderedFloat<f32> {
    type Error = stack::Error;

    fn try_from(atomic: stack::Atomic) -> stack::Result<Self> {
        atomic.to_float()
    }
}

impl TryFrom<stack::Atomic> for OrderedFloat<f64> {
    type Error = stack::Error;

    fn try_from(atomic: stack::Atomic) -> stack::Result<Self> {
        atomic.to_double()
    }
}

impl TryFrom<stack::Atomic> for f64 {
    type Error = stack::Error;

    fn try_from(atomic: stack::Atomic) -> stack::Result<Self> {
        atomic.to_double().map(|d| d.into())
    }
}

impl TryFrom<stack::Atomic> for bool {
    type Error = stack::Error;

    fn try_from(atomic: stack::Atomic) -> stack::Result<Self> {
        // TODO: is this correct? or should we use a to_bool instead?
        atomic.effective_boolean_value()
    }
}

impl<'a> TryFrom<&'a stack::Atomic> for &'a str {
    type Error = stack::Error;

    fn try_from(atomic: &'a stack::Atomic) -> stack::Result<&'a str> {
        atomic.to_str()
    }
}

impl TryFrom<stack::Atomic> for String {
    type Error = stack::Error;

    fn try_from(atomic: stack::Atomic) -> stack::Result<Self> {
        atomic.to_string()
    }
}

impl TryFrom<stack::Value> for stack::Sequence {
    type Error = stack::Error;

    fn try_from(value: stack::Value) -> stack::Result<Self> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&stack::Value> for stack::Sequence {
    type Error = stack::Error;

    fn try_from(value: &stack::Value) -> stack::Result<Self> {
        match value {
            stack::Value::Sequence(s) => Ok(s.clone()),
            stack::Value::Atomic(a) => Ok(stack::Sequence::from_atomic(a)),
            stack::Value::Node(n) => Ok(stack::Sequence::from_node(*n)),
            _ => Err(stack::Error::Type),
        }
    }
}

// Conversions from Rust values into Value

impl From<String> for stack::Value {
    fn from(s: String) -> stack::Value {
        stack::Value::Atomic(stack::Atomic::String(Rc::new(s)))
    }
}

impl From<f64> for stack::Value {
    fn from(f: f64) -> stack::Value {
        stack::Value::Atomic(stack::Atomic::Double(OrderedFloat(f)))
    }
}

impl From<i64> for stack::Value {
    fn from(i: i64) -> stack::Value {
        stack::Value::Atomic(stack::Atomic::Integer(i))
    }
}

impl From<bool> for stack::Value {
    fn from(b: bool) -> stack::Value {
        stack::Value::Atomic(stack::Atomic::Boolean(b))
    }
}

impl From<xml::Node> for stack::Value {
    fn from(n: xml::Node) -> stack::Value {
        stack::Value::Node(n)
    }
}

impl<T> From<Option<T>> for stack::Value
where
    T: Into<stack::Value>,
{
    fn from(o: Option<T>) -> stack::Value {
        match o {
            Some(v) => v.into(),
            None => stack::Value::Atomic(stack::Atomic::Empty),
        }
    }
}

// impl<T> From<Option<T>> for Value
// where
//     T: Into<Item>,
// {
//     fn from(o: Option<T>) -> Value {
//         match o {
//             Some(v) => Value::from_item(v.into()),
//             None => Value::stack::Atomic(stack::Atomic::Empty),
//         }
//     }
// }

impl From<stack::Item> for stack::Value {
    fn from(item: stack::Item) -> stack::Value {
        stack::Value::from_item(item)
    }
}

use ordered_float::OrderedFloat;
use std::convert::TryFrom;
use std::rc::Rc;

use crate::stack;
use crate::xml;

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

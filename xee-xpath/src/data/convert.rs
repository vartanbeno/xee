use ordered_float::OrderedFloat;
use rust_decimal::Decimal;
use std::convert::TryFrom;
use std::rc::Rc;
use std::vec::Vec;

use crate::context::DynamicContext;

use super::{Closure, Step};
use crate::stack;
use crate::xml;

// wrapper should generate:
// Value -> i64
// Value -> &str
// String -> Value

pub(crate) trait ContextFrom<T>: Sized {
    fn context_from(value: T, context: &DynamicContext) -> Self;
}

pub(crate) trait ContextTryFrom<T>: Sized {
    fn context_try_from(value: T, context: &DynamicContext) -> stack::ValueResult<Self>;
}

pub(crate) trait ContextInto<T>: Sized {
    fn context_into(self, context: &DynamicContext) -> T;
}

pub(crate) trait ContextTryInto<T>: Sized {
    fn context_try_into(self, context: &DynamicContext) -> stack::ValueResult<T>;
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
    fn context_try_into(self, context: &DynamicContext) -> stack::ValueResult<U> {
        U::context_try_from(self, context)
    }
}

// Conversions from Value

impl ContextTryFrom<stack::StackValue> for stack::Atomic {
    fn context_try_from(
        value: stack::StackValue,
        context: &DynamicContext,
    ) -> stack::ValueResult<Self> {
        ContextTryFrom::context_try_from(&value, context)
    }
}

impl ContextTryFrom<&stack::StackValue> for stack::Atomic {
    fn context_try_from(
        value: &stack::StackValue,
        context: &DynamicContext,
    ) -> stack::ValueResult<Self> {
        match value {
            stack::StackValue::Atomic(a) => Ok(a.clone()),
            stack::StackValue::Sequence(s) => s.borrow().to_atomic(context),
            _ => {
                todo!("don't know how to atomize this yet")
            }
        }
    }
}

// impl ContextTryFrom<&Value> for f64 {
//     fn context_try_from(value: &Value, context: &DynamicContext) -> Result<Self> {
//         let atomic: stack::Atomic = value.context_try_into(context)?;
//         atomic.try_into()
//     }
// }

impl<T> ContextTryFrom<&stack::StackValue> for T
where
    T: TryFrom<stack::Atomic, Error = stack::ValueError>,
{
    fn context_try_from(
        value: &stack::StackValue,
        context: &DynamicContext,
    ) -> stack::ValueResult<Self> {
        let atomic: stack::Atomic = value.context_try_into(context)?;
        atomic.try_into()
    }
}

impl ContextTryFrom<&stack::StackValue> for xml::Node {
    fn context_try_from(
        value: &stack::StackValue,
        _context: &DynamicContext,
    ) -> stack::ValueResult<Self> {
        match value.to_one()? {
            stack::StackItem::Node(n) => Ok(n),
            _ => Err(stack::ValueError::Type),
        }
    }
}

// impl ContextTryFrom<&Value> for i64 {
//     fn context_try_from(value: &Value, context: &DynamicContext) -> Result<Self> {
//         let atomic: stack::Atomic = value.context_try_into(context)?;
//         atomic.try_into()
//     }
// }

impl<T> ContextTryFrom<&stack::StackValue> for Option<T>
where
    T: TryFrom<stack::StackItem, Error = stack::ValueError>,
{
    fn context_try_from(
        value: &stack::StackValue,
        _context: &DynamicContext,
    ) -> stack::ValueResult<Self> {
        match value.to_option()? {
            Some(v) => Ok(Some(v.try_into()?)),
            None => Ok(None),
        }
    }
}

impl<'a> TryFrom<&'a stack::StackValue> for &'a Closure {
    type Error = stack::ValueError;

    fn try_from(value: &'a stack::StackValue) -> stack::ValueResult<&'a Closure> {
        match value {
            stack::StackValue::Closure(c) => Ok(c),
            _ => Err(stack::ValueError::Type),
        }
    }
}

impl TryFrom<&stack::StackValue> for Rc<Step> {
    type Error = stack::ValueError;

    fn try_from(value: &stack::StackValue) -> stack::ValueResult<Rc<Step>> {
        match value {
            stack::StackValue::Step(s) => Ok(Rc::clone(s)),
            _ => Err(stack::ValueError::Type),
        }
    }
}

impl TryFrom<stack::StackValue> for xml::Node {
    type Error = stack::ValueError;

    fn try_from(value: stack::StackValue) -> stack::ValueResult<xml::Node> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&stack::StackValue> for xml::Node {
    type Error = stack::ValueError;

    fn try_from(value: &stack::StackValue) -> stack::ValueResult<xml::Node> {
        match value {
            stack::StackValue::Node(n) => Ok(*n),
            stack::StackValue::Sequence(s) => s.borrow().singleton().and_then(|n| n.to_node()),
            _ => Err(stack::ValueError::Type),
        }
    }
}

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

impl TryFrom<&stack::StackItem> for stack::Atomic {
    type Error = stack::ValueError;

    fn try_from(item: &stack::StackItem) -> stack::ValueResult<Self> {
        match item {
            stack::StackItem::Atomic(a) => Ok(a.clone()),
            _ => Err(stack::ValueError::Type),
        }
    }
}

impl TryFrom<stack::StackItem> for stack::Atomic {
    type Error = stack::ValueError;

    fn try_from(item: stack::StackItem) -> stack::ValueResult<Self> {
        match item {
            stack::StackItem::Atomic(a) => Ok(a),
            _ => Err(stack::ValueError::Type),
        }
    }
}

impl TryFrom<stack::StackItem> for f64 {
    type Error = stack::ValueError;

    fn try_from(item: stack::StackItem) -> stack::ValueResult<Self> {
        match item {
            stack::StackItem::Atomic(a) => a.try_into(),
            _ => Err(stack::ValueError::Type),
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

impl TryFrom<stack::StackItem> for xml::Node {
    type Error = stack::ValueError;

    fn try_from(item: stack::StackItem) -> stack::ValueResult<Self> {
        match item {
            stack::StackItem::Node(n) => Ok(n),
            _ => Err(stack::ValueError::Type),
        }
    }
}

// Conversions from stack::Atomic

impl TryFrom<stack::Atomic> for i64 {
    type Error = stack::ValueError;

    fn try_from(atomic: stack::Atomic) -> stack::ValueResult<Self> {
        atomic.to_integer()
    }
}

impl TryFrom<stack::Atomic> for Decimal {
    type Error = stack::ValueError;

    fn try_from(atomic: stack::Atomic) -> stack::ValueResult<Self> {
        atomic.to_decimal()
    }
}

impl TryFrom<stack::Atomic> for OrderedFloat<f32> {
    type Error = stack::ValueError;

    fn try_from(atomic: stack::Atomic) -> stack::ValueResult<Self> {
        atomic.to_float()
    }
}

impl TryFrom<stack::Atomic> for OrderedFloat<f64> {
    type Error = stack::ValueError;

    fn try_from(atomic: stack::Atomic) -> stack::ValueResult<Self> {
        atomic.to_double()
    }
}

impl TryFrom<stack::Atomic> for f64 {
    type Error = stack::ValueError;

    fn try_from(atomic: stack::Atomic) -> stack::ValueResult<Self> {
        atomic.to_double().map(|d| d.into())
    }
}

impl TryFrom<stack::Atomic> for bool {
    type Error = stack::ValueError;

    fn try_from(atomic: stack::Atomic) -> stack::ValueResult<Self> {
        atomic.to_bool()
    }
}

impl<'a> TryFrom<&'a stack::Atomic> for &'a str {
    type Error = stack::ValueError;

    fn try_from(atomic: &'a stack::Atomic) -> stack::ValueResult<&'a str> {
        atomic.to_str()
    }
}

impl TryFrom<stack::Atomic> for String {
    type Error = stack::ValueError;

    fn try_from(atomic: stack::Atomic) -> stack::ValueResult<Self> {
        atomic.to_string()
    }
}

impl TryFrom<stack::StackValue> for stack::StackSequence {
    type Error = stack::ValueError;

    fn try_from(value: stack::StackValue) -> stack::ValueResult<Self> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&stack::StackValue> for stack::StackSequence {
    type Error = stack::ValueError;

    fn try_from(value: &stack::StackValue) -> stack::ValueResult<Self> {
        match value {
            stack::StackValue::Sequence(s) => Ok(s.clone()),
            stack::StackValue::Atomic(a) => Ok(stack::StackSequence::from_atomic(a)),
            stack::StackValue::Node(n) => Ok(stack::StackSequence::from_node(*n)),
            _ => Err(stack::ValueError::Type),
        }
    }
}

impl ContextFrom<stack::StackSequence> for Vec<stack::Atomic> {
    fn context_from(sequence: stack::StackSequence, context: &DynamicContext) -> Self {
        sequence.borrow().to_atoms(context.xot)
    }
}

// Conversions from Rust values into Value

impl From<String> for stack::StackValue {
    fn from(s: String) -> stack::StackValue {
        stack::StackValue::Atomic(stack::Atomic::String(Rc::new(s)))
    }
}

impl From<f64> for stack::StackValue {
    fn from(f: f64) -> stack::StackValue {
        stack::StackValue::Atomic(stack::Atomic::Double(OrderedFloat(f)))
    }
}

impl From<i64> for stack::StackValue {
    fn from(i: i64) -> stack::StackValue {
        stack::StackValue::Atomic(stack::Atomic::Integer(i))
    }
}

impl From<bool> for stack::StackValue {
    fn from(b: bool) -> stack::StackValue {
        stack::StackValue::Atomic(stack::Atomic::Boolean(b))
    }
}

impl From<xml::Node> for stack::StackValue {
    fn from(n: xml::Node) -> stack::StackValue {
        stack::StackValue::Node(n)
    }
}

impl<T> From<Option<T>> for stack::StackValue
where
    T: Into<stack::StackValue>,
{
    fn from(o: Option<T>) -> stack::StackValue {
        match o {
            Some(v) => v.into(),
            None => stack::StackValue::Atomic(stack::Atomic::Empty),
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

impl From<stack::StackItem> for stack::StackValue {
    fn from(item: stack::StackItem) -> stack::StackValue {
        stack::StackValue::from_item(item)
    }
}

use ordered_float::OrderedFloat;
use rust_decimal::Decimal;
use std::convert::TryFrom;
use std::rc::Rc;
use std::vec::Vec;

use crate::context::DynamicContext;

use super::{Atomic, Closure, Item, Node, StackSequence, StackValue, Step, ValueError};

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

impl ContextTryFrom<StackValue> for Atomic {
    fn context_try_from(value: StackValue, context: &DynamicContext) -> Result<Self> {
        ContextTryFrom::context_try_from(&value, context)
    }
}

impl ContextTryFrom<&StackValue> for Atomic {
    fn context_try_from(value: &StackValue, context: &DynamicContext) -> Result<Self> {
        match value {
            StackValue::Atomic(a) => Ok(a.clone()),
            StackValue::Sequence(s) => s.borrow().to_atomic(context),
            _ => {
                todo!("don't know how to atomize this yet")
            }
        }
    }
}

// impl ContextTryFrom<&Value> for f64 {
//     fn context_try_from(value: &Value, context: &DynamicContext) -> Result<Self> {
//         let atomic: Atomic = value.context_try_into(context)?;
//         atomic.try_into()
//     }
// }

impl<T> ContextTryFrom<&StackValue> for T
where
    T: TryFrom<Atomic, Error = ValueError>,
{
    fn context_try_from(value: &StackValue, context: &DynamicContext) -> Result<Self> {
        let atomic: Atomic = value.context_try_into(context)?;
        atomic.try_into()
    }
}

impl ContextTryFrom<&StackValue> for Node {
    fn context_try_from(value: &StackValue, _context: &DynamicContext) -> Result<Self> {
        match value.to_one()? {
            Item::Node(n) => Ok(n),
            _ => Err(ValueError::Type),
        }
    }
}

// impl ContextTryFrom<&Value> for i64 {
//     fn context_try_from(value: &Value, context: &DynamicContext) -> Result<Self> {
//         let atomic: Atomic = value.context_try_into(context)?;
//         atomic.try_into()
//     }
// }

impl<T> ContextTryFrom<&StackValue> for Option<T>
where
    T: TryFrom<Item, Error = ValueError>,
{
    fn context_try_from(value: &StackValue, _context: &DynamicContext) -> Result<Self> {
        match value.to_option()? {
            Some(v) => Ok(Some(v.try_into()?)),
            None => Ok(None),
        }
    }
}

impl<'a> TryFrom<&'a StackValue> for &'a Closure {
    type Error = ValueError;

    fn try_from(value: &'a StackValue) -> Result<&'a Closure> {
        match value {
            StackValue::Closure(c) => Ok(c),
            _ => Err(ValueError::Type),
        }
    }
}

impl TryFrom<&StackValue> for Rc<Step> {
    type Error = ValueError;

    fn try_from(value: &StackValue) -> Result<Rc<Step>> {
        match value {
            StackValue::Step(s) => Ok(Rc::clone(s)),
            _ => Err(ValueError::Type),
        }
    }
}

impl TryFrom<StackValue> for Node {
    type Error = ValueError;

    fn try_from(value: StackValue) -> Result<Node> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&StackValue> for Node {
    type Error = ValueError;

    fn try_from(value: &StackValue) -> Result<Node> {
        match value {
            StackValue::Node(n) => Ok(*n),
            StackValue::Sequence(s) => s.borrow().singleton().and_then(|n| n.to_node()),
            _ => Err(ValueError::Type),
        }
    }
}

// impl TryFrom<&Value> for Item {
//     type Error = ValueError;

//     fn try_from(value: &Value) -> Result<Item> {
//         match value {
//             Value::Atomic(a) => Ok(Item::Atomic(a.clone())),
//             Value::Node(n) => Ok(Item::Node(*n)),
//             Value::Sequence(s) => s.borrow().singleton(),
//             Value::Closure(c) => Ok(Item::Closure(Rc::clone(c))),
//         }
//     }
// }
// Conversions from Item

impl TryFrom<&Item> for Atomic {
    type Error = ValueError;

    fn try_from(item: &Item) -> Result<Self> {
        match item {
            Item::Atomic(a) => Ok(a.clone()),
            _ => Err(ValueError::Type),
        }
    }
}

impl TryFrom<Item> for Atomic {
    type Error = ValueError;

    fn try_from(item: Item) -> Result<Self> {
        match item {
            Item::Atomic(a) => Ok(a),
            _ => Err(ValueError::Type),
        }
    }
}

impl TryFrom<Item> for f64 {
    type Error = ValueError;

    fn try_from(item: Item) -> Result<Self> {
        match item {
            Item::Atomic(a) => a.try_into(),
            _ => Err(ValueError::Type),
        }
    }
}

// impl<T> ContextTryFrom<Option<Item>> for Option<T>
// where
//     T: TryFrom<T, Error = ValueError>,
// {
//     fn context_try_from(item: Option<Item>, context: &DynamicContext) -> Result<Option<T>> {
//         match item {
//             Some(i) => Ok(Some(i.try_into()?)),
//             None => Ok(None),
//         }
//     }
// }

impl TryFrom<Item> for Node {
    type Error = ValueError;

    fn try_from(item: Item) -> Result<Self> {
        match item {
            Item::Node(n) => Ok(n),
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

impl TryFrom<Atomic> for f64 {
    type Error = ValueError;

    fn try_from(atomic: Atomic) -> Result<Self> {
        atomic.to_double().map(|d| d.into())
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

impl TryFrom<StackValue> for StackSequence {
    type Error = ValueError;

    fn try_from(value: StackValue) -> Result<Self> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&StackValue> for StackSequence {
    type Error = ValueError;

    fn try_from(value: &StackValue) -> Result<Self> {
        match value {
            StackValue::Sequence(s) => Ok(s.clone()),
            StackValue::Atomic(a) => Ok(StackSequence::from_atomic(a)),
            StackValue::Node(n) => Ok(StackSequence::from_node(*n)),
            _ => Err(ValueError::Type),
        }
    }
}

impl ContextFrom<StackSequence> for Vec<Atomic> {
    fn context_from(sequence: StackSequence, context: &DynamicContext) -> Self {
        sequence.borrow().to_atoms(context.xot)
    }
}

// Conversions from Rust values into Value

impl From<String> for StackValue {
    fn from(s: String) -> StackValue {
        StackValue::Atomic(Atomic::String(Rc::new(s)))
    }
}

impl From<f64> for StackValue {
    fn from(f: f64) -> StackValue {
        StackValue::Atomic(Atomic::Double(OrderedFloat(f)))
    }
}

impl From<i64> for StackValue {
    fn from(i: i64) -> StackValue {
        StackValue::Atomic(Atomic::Integer(i))
    }
}

impl From<bool> for StackValue {
    fn from(b: bool) -> StackValue {
        StackValue::Atomic(Atomic::Boolean(b))
    }
}

impl From<Node> for StackValue {
    fn from(n: Node) -> StackValue {
        StackValue::Node(n)
    }
}

impl<T> From<Option<T>> for StackValue
where
    T: Into<StackValue>,
{
    fn from(o: Option<T>) -> StackValue {
        match o {
            Some(v) => v.into(),
            None => StackValue::Atomic(Atomic::Empty),
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
//             None => Value::Atomic(Atomic::Empty),
//         }
//     }
// }

impl From<Item> for StackValue {
    fn from(item: Item) -> StackValue {
        StackValue::from_item(item)
    }
}

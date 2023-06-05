use std::convert::TryFrom;
use std::vec::Vec;

use ordered_float::OrderedFloat;
use rust_decimal::Decimal;

use crate::context::DynamicContext;
use crate::data::{Atomic, Sequence, Value, ValueError};

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

// impl TryFrom<Value> for Sequence {
//     fn try_from(value: Value) -> Result<Self> {
//         match self {
//             Value::Sequence(s) => Ok(s.clone()),
//             Value::Atomic(a) => Ok(Sequence::from_atomic(a)),
//             Value::Node(n) => Ok(Sequence::from_node(*n)),
//             _ => Err(ValueError::Type),
//         }
//         // match value {
//         //     Value::Sequence(s) => Ok(s),
//         //     _ => todo!("don't know how to atomize this yet"),
//         // }
//     }
// }

// impl<T> ContextTryFrom<Value> for T
// where
//     T: TryFrom<Atomic, Error = ValueError>,
// {
//     fn context_try_from(value: Value, context: &DynamicContext) -> Result<Self> {
//         let atomic: Atomic = ContextTryInto::context_try_into(value, context)?;
//         TryInto::try_into(atomic)
//     }
// }

// impl<T> ContextTryFrom<Value> for T
// where
//     T: ContextFrom<Rc<RefCell<Sequence>>>,
// {
//     fn context_try_from(value: Value, context: &DynamicContext) -> Result<Self> {
//         let sequence: Rc<RefCell<Sequence>> = ContextTryInto::context_try_into(value, context)?;
//         Ok(ContextFrom::context_from(sequence, context))
//     }
// }

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

// impl TryFrom<Value> for Node {
//     type Error = ValueError;

//     fn try_from(value: Value) -> Result<Self> {
//         value.to_node()
//     }
// }

// impl TryFrom<Value> for Closure {
//     type Error = ValueError;

//     fn try_from(value: Value) -> Result<Self, Self::Error> {
//         value.to_closure()
//     }
// }

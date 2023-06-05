use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;

use crate::context::DynamicContext;
use crate::data::{Atomic, Node, Sequence, Value, ValueError};

// wrapper should generate:
// Value -> i64
// Value -> &str
// String -> Value

pub(crate) trait TryFromValue: Sized {
    fn try_from(value: Value, context: &DynamicContext) -> Result<Self, ValueError>;
}

impl TryFromValue for Atomic {
    fn try_from(value: Value, context: &DynamicContext) -> Result<Atomic, ValueError> {
        value.to_atomic(context)
    }
}

impl TryFrom<Value> for bool {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        // is this correct? this gives back the effective boolean value
        value.effective_boolean_value()
    }
}

impl TryFrom<Value> for Rc<RefCell<Sequence>> {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.to_sequence()
    }
}

impl TryFrom<Value> for Node {
    type Error = ValueError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.to_node()
    }
}

// impl TryFrom<Value> for Closure {
//     type Error = ValueError;

//     fn try_from(value: Value) -> Result<Self, Self::Error> {
//         value.to_closure()
//     }
// }

// the stack::Value abstraction is a sequence partitioned into special cases:
// empty sequence, sequence with a single item, and sequence with multiple
// items. This partitioning makes it easier to optimize various common cases
// and keeps the code cleaner.
use std::rc::Rc;

use ahash::{HashSet, HashSetExt};
use xot::Xot;

use crate::atomic;
use crate::atomic::AtomicCompare;
use crate::context;
use crate::error;
use crate::function;
use crate::occurrence;
use crate::sequence;
use crate::stack;
use crate::xml;

use super::comparison;

#[derive(Debug, Clone)]
pub enum Value {
    Absent,
    Sequence(sequence::Sequence),
}

impl TryFrom<Value> for sequence::Sequence {
    type Error = error::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Absent => Err(error::Error::XPDY0002),
            Value::Sequence(sequence) => Ok(sequence),
        }
    }
}

impl TryFrom<&Value> for sequence::Sequence {
    type Error = error::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Absent => Err(error::Error::XPDY0002),
            Value::Sequence(sequence) => Ok(sequence.clone()),
        }
    }
}

// impl From<sequence::Sequence> for Value {
//     fn from(sequence: sequence::Sequence) -> Self {
//         Value::Sequence(sequence)
//     }
// }

impl<T> From<T> for Value
where
    T: Into<sequence::Sequence>,
{
    fn from(t: T) -> Self {
        let sequence: sequence::Sequence = t.into();
        Value::Sequence(sequence)
    }
}

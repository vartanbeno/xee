use xot::Xot;

use crate::error;
use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Empty,
    Item(stack::Item),
    Sequence(stack::Sequence),
    Absent,
}

impl Value {
    pub(crate) fn into_output(self) -> output::Sequence {
        output::Sequence::new(self)
    }

    pub(crate) fn to_sequence(&self) -> error::Result<stack::Sequence> {
        match self {
            Value::Sequence(s) => Ok(s.clone()),
            _ => Ok(stack::Sequence::from(
                self.items().collect::<error::Result<Vec<_>>>()?,
            )),
        }
    }

    pub(crate) fn items(&self) -> ValueIter {
        ValueIter::new(self.clone())
    }

    pub(crate) fn atomized<'a>(&self, xot: &'a Xot) -> stack::AtomizedIter<'a> {
        stack::AtomizedIter::new(self.clone(), xot)
    }

    pub(crate) fn effective_boolean_value(&self) -> error::Result<bool> {
        match self {
            Value::Empty => Ok(false),
            Value::Item(item) => item.effective_boolean_value(),
            Value::Sequence(s) => {
                let s = s.borrow();
                // If its operand is an empty sequence, fn:boolean returns false.
                if s.is_empty() {
                    return Ok(false);
                }
                // If its operand is a sequence whose first item is a node, fn:boolean returns true.
                if matches!(s.items[0], stack::Item::Node(_)) {
                    return Ok(true);
                }
                // If its operand is a singleton value
                let singleton = s.singleton()?;
                singleton.effective_boolean_value()
            }
            Value::Absent => Err(error::Error::ComponentAbsentInDynamicContext),
        }
    }

    pub(crate) fn is_empty_sequence(&self) -> bool {
        match self {
            Value::Empty => true,
            Value::Sequence(s) => s.is_empty(),
            _ => false,
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> error::Result<String> {
        let value = match self {
            Value::Empty => "".to_string(),
            Value::Item(item) => item.string_value(xot)?,
            Value::Sequence(sequence) => {
                let sequence = sequence.borrow();
                let len = sequence.len();
                match len {
                    0 => "".to_string(),
                    1 => Value::from(sequence.items[0].clone()).string_value(xot)?,
                    _ => Err(error::Error::Type)?,
                }
            }
            Value::Absent => Err(error::Error::ComponentAbsentInDynamicContext)?,
        };
        Ok(value)
    }
}

impl<T> From<T> for Value
where
    T: Into<stack::Item>,
{
    fn from(item: T) -> Self {
        Value::Item(item.into())
    }
}

impl From<Vec<stack::Item>> for Value {
    fn from(items: Vec<stack::Item>) -> Self {
        if items.is_empty() {
            Value::Empty
        } else if items.len() == 1 {
            Value::from(items[0].clone())
        } else {
            Value::Sequence(stack::Sequence::from(items))
        }
    }
}

impl<'a> TryFrom<&'a stack::Value> for &'a stack::Closure {
    type Error = error::Error;

    fn try_from(value: &'a stack::Value) -> error::Result<&'a stack::Closure> {
        match value {
            stack::Value::Item(stack::Item::Function(c)) => Ok(c),
            // TODO: not handling this correctly yet
            // stack::Value::Sequence(s) => s.borrow().singleton().and_then(|n| n.to_function()),
            _ => Err(error::Error::Type),
        }
    }
}

impl TryFrom<stack::Value> for xml::Node {
    type Error = error::Error;

    fn try_from(value: stack::Value) -> error::Result<xml::Node> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&stack::Value> for xml::Node {
    type Error = error::Error;

    fn try_from(value: &stack::Value) -> error::Result<xml::Node> {
        match value {
            stack::Value::Item(stack::Item::Node(n)) => Ok(*n),
            stack::Value::Sequence(s) => s.borrow().singleton().and_then(|n| n.to_node()),
            _ => Err(error::Error::Type),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        // comparisons between values are tricky, as a value may be a single
        // item or a sequence of items. If they are single items, the
        // comparison is easy, if one half is a sequence and the other half is
        // not, then we convert the value into a sequence first before
        // comparing
        match (self, other) {
            (Value::Empty, Value::Empty) => true,
            (Value::Item(a), Value::Item(b)) => a == b,
            (Value::Sequence(a), Value::Sequence(b)) => a == b,
            (Value::Empty, Value::Sequence(b)) => b.is_empty(),
            (Value::Sequence(a), Value::Empty) => a.is_empty(),
            (Value::Item(a), Value::Sequence(b)) => {
                if b.len() != 1 {
                    return false;
                }
                let a: stack::Sequence = a.clone().into();
                &a == b
            }
            (Value::Sequence(a), Value::Item(b)) => {
                if a.len() != 1 {
                    return false;
                }
                let b: stack::Sequence = b.clone().into();
                a == &b
            }
            (Value::Empty, Value::Item(_)) => false,
            (Value::Item(_), Value::Empty) => false,
            (Value::Absent, _) => false,
            (_, Value::Absent) => false,
        }
    }
}

pub(crate) enum ValueIter {
    Empty,
    AbsentIter(std::iter::Once<error::Result<stack::Item>>),
    ItemIter(std::iter::Once<stack::Item>),
    SequenceIter(stack::SequenceIter),
}

impl ValueIter {
    fn new(value: Value) -> Self {
        match value {
            Value::Empty => ValueIter::Empty,
            Value::Item(item) => ValueIter::ItemIter(std::iter::once(item)),
            Value::Sequence(sequence) => ValueIter::SequenceIter(sequence.items()),
            Value::Absent => ValueIter::AbsentIter(std::iter::once(Err(
                error::Error::ComponentAbsentInDynamicContext,
            ))),
        }
    }
}

impl Iterator for ValueIter {
    type Item = error::Result<stack::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ValueIter::Empty => None,
            ValueIter::ItemIter(iter) => iter.next().map(Ok),
            ValueIter::SequenceIter(iter) => iter.next().map(Ok),
            ValueIter::AbsentIter(iter) => iter.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::atomic;

    #[test]
    fn test_integer_compares_with_decimal() {
        let a = atomic::Atomic::Integer(1);
        let b = atomic::Atomic::Decimal(Decimal::from(1));
        assert_eq!(a, b);
    }
}

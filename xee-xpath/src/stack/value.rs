use xot::Xot;

use crate::occurrence;
use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Empty,
    Item(stack::Item),
    Sequence(stack::Sequence),
}

impl Value {
    pub(crate) fn into_output(self) -> output::Sequence {
        output::Sequence::new(self)
    }

    pub(crate) fn to_sequence(&self) -> stack::Sequence {
        match self {
            Value::Sequence(s) => s.clone(),
            _ => stack::Sequence::from(self.items().collect::<Vec<_>>()),
        }
    }

    pub(crate) fn items(&self) -> ValueIter {
        ValueIter::new(self.clone())
    }

    pub(crate) fn atomized<'a>(&self, xot: &'a Xot) -> stack::AtomizedIter<'a> {
        stack::AtomizedIter::new(self.clone(), xot)
    }

    pub(crate) fn effective_boolean_value(&self) -> stack::Result<bool> {
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
        }
    }

    pub(crate) fn is_empty_sequence(&self) -> bool {
        match self {
            Value::Empty => true,
            Value::Sequence(s) => s.is_empty(),
            _ => false,
        }
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> stack::Result<String> {
        let value = match self {
            Value::Empty => "".to_string(),
            Value::Item(item) => item.string_value(xot)?,
            Value::Sequence(sequence) => {
                let sequence = sequence.borrow();
                let len = sequence.len();
                match len {
                    0 => "".to_string(),
                    1 => Value::from(sequence.items[0].clone()).string_value(xot)?,
                    _ => Err(stack::Error::Type)?,
                }
            }
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
    type Error = stack::Error;

    fn try_from(value: &'a stack::Value) -> stack::Result<&'a stack::Closure> {
        match value {
            stack::Value::Item(stack::Item::Function(c)) => Ok(c),
            // TODO: not handling this correctly yet
            // stack::Value::Sequence(s) => s.borrow().singleton().and_then(|n| n.to_function()),
            _ => Err(stack::Error::Type),
        }
    }
}

impl TryFrom<stack::Value> for xml::Node {
    type Error = stack::Error;

    fn try_from(value: stack::Value) -> stack::Result<xml::Node> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&stack::Value> for xml::Node {
    type Error = stack::Error;

    fn try_from(value: &stack::Value) -> stack::Result<xml::Node> {
        match value {
            stack::Value::Item(stack::Item::Node(n)) => Ok(*n),
            stack::Value::Sequence(s) => s.borrow().singleton().and_then(|n| n.to_node()),
            _ => Err(stack::Error::Type),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        // comparisons between values are tricky, as a value
        // may be a single item or a sequence of items.
        // If they are single items, the comparison is easy,
        // if one half is a sequence (or an empty atomic) and
        // the other half is not, then we convert the value into a sequence first
        // before comparing
        match (self, other) {
            (Value::Empty, Value::Empty) => true,
            (Value::Item(a), Value::Item(b)) => a == b,
            (Value::Sequence(a), Value::Sequence(b)) => a == b,
            (Value::Empty, Value::Sequence(b)) => b.is_empty(),
            (Value::Sequence(a), Value::Empty) => a.is_empty(),
            (_, Value::Sequence(b)) => (&self.to_sequence()) == b,
            (Value::Sequence(a), _) => a == &other.to_sequence(),
            _ => false,
        }
    }
}

pub(crate) struct ValueIter {
    stack_value: Value,
    index: usize,
}

impl ValueIter {
    fn new(stack_value: Value) -> Self {
        Self {
            stack_value,
            index: 0,
        }
    }
}

impl Iterator for ValueIter {
    type Item = stack::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.stack_value {
            stack::Value::Empty => None,
            stack::Value::Item(item) => match item {
                stack::Item::Atomic(a) => {
                    if self.index == 0 {
                        self.index += 1;
                        Some(stack::Item::Atomic(a.clone()))
                    } else {
                        None
                    }
                }
                stack::Item::Node(node) => {
                    if self.index == 0 {
                        self.index += 1;
                        Some(stack::Item::Node(*node))
                    } else {
                        None
                    }
                }
                stack::Item::Function(closure) => {
                    if self.index == 0 {
                        self.index += 1;
                        Some(stack::Item::Function(closure.clone()))
                    } else {
                        None
                    }
                }
            },
            stack::Value::Sequence(sequence) => {
                let sequence = sequence.borrow();
                if self.index < sequence.len() {
                    let item = sequence.items[self.index].clone();
                    self.index += 1;
                    Some(item)
                } else {
                    None
                }
            }
        }
    }
}

impl occurrence::Occurrence<stack::Item, stack::Error> for ValueIter {
    fn error(&self) -> stack::Error {
        stack::Error::Type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_integer_compares_with_decimal() {
        let a = stack::Atomic::Integer(1);
        let b = stack::Atomic::Decimal(Decimal::from(1));
        assert_eq!(a, b);
    }
}

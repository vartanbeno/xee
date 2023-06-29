use ahash::{HashSet, HashSetExt};
use std::rc::Rc;
use xot::Xot;

use crate::error;
use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Empty,
    Item(stack::Item),
    Many(Rc<Vec<stack::Item>>),
    Absent,
    Build(stack::BuildSequence),
}

impl Value {
    pub(crate) fn into_output(self) -> output::Sequence {
        output::Sequence::new(self)
    }

    pub(crate) fn len(self) -> usize {
        match self {
            Value::Empty => 0,
            Value::Item(_) => 1,
            Value::Many(items) => items.len(),
            Value::Absent => panic!("Don't know how to handle absent"),
            Value::Build(_) => unreachable!(),
        }
    }

    pub(crate) fn index(self, index: usize) -> error::Result<stack::Item> {
        match self {
            Value::Empty => Err(error::Error::Type),
            Value::Item(item) => {
                if index == 0 {
                    Ok(item)
                } else {
                    Err(error::Error::Type)
                }
            }
            Value::Many(items) => items
                .get(index)
                .ok_or(error::Error::Type)
                .map(|item| item.clone()),
            Value::Absent => Err(error::Error::ComponentAbsentInDynamicContext),
            Value::Build(_) => unreachable!(),
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
            Value::Many(items) => {
                // handle the case where the first item is a node
                // it has to be a singleton otherwise
                if matches!(items[0], stack::Item::Node(_)) {
                    Ok(true)
                } else {
                    Err(error::Error::Type)
                }
            }
            Value::Absent => Err(error::Error::ComponentAbsentInDynamicContext),
            Value::Build(_) => unreachable!(),
        }
    }

    pub(crate) fn is_empty_sequence(&self) -> bool {
        matches!(self, Value::Empty)
    }

    pub(crate) fn string_value(&self, xot: &Xot) -> error::Result<String> {
        match self {
            Value::Empty => Ok("".to_string()),
            Value::Item(item) => item.string_value(xot),
            Value::Many(_) => Err(error::Error::Type),
            Value::Absent => Err(error::Error::ComponentAbsentInDynamicContext),
            Value::Build(_) => unreachable!(),
        }
    }

    pub(crate) fn concat(self, other: stack::Value) -> stack::Value {
        match (self, other) {
            (Value::Empty, Value::Empty) => Value::Empty,
            (Value::Empty, Value::Item(item)) => Value::Item(item),
            (Value::Item(item), Value::Empty) => Value::Item(item),
            (Value::Empty, Value::Many(items)) => Value::Many(items),
            (Value::Many(items), Value::Empty) => Value::Many(items),
            (Value::Item(item1), Value::Item(item2)) => Value::Many(Rc::new(vec![item1, item2])),
            (Value::Item(item), Value::Many(items)) => {
                let mut many = vec![item];
                many.extend(Rc::as_ref(&items).clone());
                Value::Many(Rc::new(many))
            }
            (Value::Many(items), Value::Item(item)) => {
                let mut many = Rc::as_ref(&items).clone();
                many.push(item);
                Value::Many(Rc::new(many))
            }
            (Value::Many(items1), Value::Many(items2)) => {
                let mut many = Rc::as_ref(&items1).clone();
                many.extend(Rc::as_ref(&items2).clone());
                Value::Many(Rc::new(many))
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn union(
        self,
        other: stack::Value,
        annotations: &xml::Annotations,
    ) -> error::Result<stack::Value> {
        let mut s = HashSet::new();
        for item in self.items() {
            let node = match item? {
                stack::Item::Node(node) => node,
                stack::Item::Atomic(..) => return Err(error::Error::Type),
                stack::Item::Function(..) => return Err(error::Error::Type),
            };
            s.insert(node);
        }
        for item in other.items() {
            let node = match item? {
                stack::Item::Node(node) => node,
                stack::Item::Atomic(..) => return Err(error::Error::Type),
                stack::Item::Function(..) => return Err(error::Error::Type),
            };
            s.insert(node);
        }

        // sort nodes by document order
        let mut nodes = s.into_iter().collect::<Vec<_>>();
        nodes.sort_by_key(|n| annotations.document_order(*n));

        let items = nodes.into_iter().map(stack::Item::Node).collect::<Vec<_>>();
        Ok(items.into())
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
    fn from(mut items: Vec<stack::Item>) -> Self {
        if items.is_empty() {
            Value::Empty
        } else if items.len() == 1 {
            Value::Item(items.pop().unwrap())
        } else {
            Value::Many(Rc::new(items))
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
            _ => Err(error::Error::Type),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Empty, Value::Empty) => true,
            (Value::Item(a), Value::Item(b)) => a == b,
            (Value::Many(a), Value::Many(b)) => a == b,
            _ => false,
        }
    }
}

pub(crate) enum ValueIter {
    Empty,
    ItemIter(std::iter::Once<stack::Item>),
    ManyIter(std::vec::IntoIter<stack::Item>),
    AbsentIter(std::iter::Once<error::Result<stack::Item>>),
}

impl ValueIter {
    fn new(value: Value) -> Self {
        match value {
            Value::Empty => ValueIter::Empty,
            Value::Item(item) => ValueIter::ItemIter(std::iter::once(item)),
            Value::Many(items) => ValueIter::ManyIter(Rc::as_ref(&items).clone().into_iter()),
            Value::Absent => ValueIter::AbsentIter(std::iter::once(Err(
                error::Error::ComponentAbsentInDynamicContext,
            ))),
            Value::Build(_) => unreachable!(),
        }
    }
}

impl Iterator for ValueIter {
    type Item = error::Result<stack::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ValueIter::Empty => None,
            ValueIter::ItemIter(iter) => iter.next().map(Ok),
            ValueIter::ManyIter(iter) => iter.next().map(Ok),
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

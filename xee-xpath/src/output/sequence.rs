use xot::Xot;

use crate::error;
use crate::occurrence;
use crate::output;
use crate::stack;
use crate::xml;

#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    stack_value: stack::Value,
}

impl Sequence {
    pub(crate) fn new(stack_value: stack::Value) -> Self {
        Self { stack_value }
    }

    pub fn empty() -> Self {
        Self {
            stack_value: stack::Value::Atomic(stack::Atomic::Empty),
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.stack_value {
            stack::Value::Atomic(stack::Atomic::Empty) => true,
            stack::Value::Sequence(sequence) => sequence.borrow().is_empty(),
            _ => false,
        }
    }

    pub fn len(&self) -> usize {
        match &self.stack_value {
            stack::Value::Atomic(stack::Atomic::Empty) => 0,
            stack::Value::Atomic(_) => 1,
            stack::Value::Sequence(sequence) => sequence.borrow().len(),
            stack::Value::Node(_) => 1,
            stack::Value::Closure(_) => 1,
            stack::Value::Step(_) => 1,
        }
    }

    pub fn is_absent(&self) -> bool {
        matches!(
            &self.stack_value,
            stack::Value::Atomic(stack::Atomic::Absent)
        )
    }

    pub fn ensure_empty(&self) -> error::Result<&Self> {
        if self.is_empty() {
            Ok(self)
        } else {
            Err(error::Error::XPTY0004A)
        }
    }

    pub fn items(&self) -> ItemIter {
        ItemIter {
            value_iter: self.stack_value.items(),
        }
    }

    pub fn nodes(&self) -> NodeIter {
        NodeIter {
            value_iter: self.stack_value.items(),
        }
    }

    pub fn atomized<'a>(&self, xot: &'a Xot) -> AtomizedIter<'a> {
        AtomizedIter {
            atomized_iter: self.stack_value.atomized(xot),
            xot,
        }
    }

    pub fn atomized_sequence(&self, xot: &Xot) -> error::Result<Sequence> {
        // TODO: conceivably we don't consume the iterator here,
        // but this would require the Sequence to be aware of an atomized
        // iterator.
        let items = self
            .atomized(xot)
            .map(|a| {
                a.map(output::Item::from)
                    .map_err(|_| error::Error::XPTY0004A)
            })
            .collect::<error::Result<Vec<_>>>()?;
        Ok(Sequence::from(items))
    }

    pub fn unboxed_atomized<'a, T>(
        &self,
        xot: &'a Xot,
        extract: impl Fn(&output::Atomic) -> error::Result<T>,
    ) -> UnboxedAtomizedIter<'a, impl Fn(&output::Atomic) -> error::Result<T>> {
        UnboxedAtomizedIter {
            atomized_iter: self.atomized(xot),
            extract,
        }
    }

    pub fn effective_boolean_value(&self) -> error::Result<bool> {
        // TODO: error conversion is a bit blunt
        self.stack_value
            .effective_boolean_value()
            .map_err(|_| error::Error::FORG0006)
    }
}

impl From<stack::Value> for Sequence {
    fn from(stack_value: stack::Value) -> Self {
        Self { stack_value }
    }
}

impl From<&stack::Value> for Sequence {
    fn from(stack_value: &stack::Value) -> Self {
        stack_value.clone().into()
    }
}

impl From<Vec<output::Item>> for Sequence {
    fn from(items: Vec<output::Item>) -> Self {
        if items.is_empty() {
            return Self {
                stack_value: stack::Value::Atomic(stack::Atomic::Empty),
            };
        }
        if items.len() == 1 {
            return Self {
                stack_value: items[0].clone().into(),
            };
        }
        let stack_items = items.iter().map(|item| item.into()).collect::<Vec<_>>();
        Self {
            stack_value: stack::Value::Sequence(stack::Sequence::from_items(&stack_items)),
        }
    }
}

impl From<Vec<xml::Node>> for Sequence {
    fn from(items: Vec<xml::Node>) -> Self {
        let items = items
            .into_iter()
            .map(output::Item::from)
            .collect::<Vec<_>>();
        Self::from(items)
    }
}

impl<T> From<Option<T>> for Sequence
where
    T: Into<output::Item>,
{
    fn from(item: Option<T>) -> Self {
        match item {
            Some(item) => Self::from(vec![item.into()]),
            None => Sequence::empty(),
        }
    }
}

impl From<output::Sequence> for stack::Value {
    fn from(sequence: output::Sequence) -> Self {
        sequence.stack_value
    }
}

impl<T> From<Vec<T>> for Sequence
where
    T: Into<output::Atomic>,
{
    fn from(items: Vec<T>) -> Self {
        let items = items
            .into_iter()
            .map(|i| output::Item::from(i.into()))
            .collect::<Vec<_>>();
        Self::from(items)
    }
}

impl<T> From<T> for Sequence
where
    T: Into<output::Atomic>,
{
    fn from(item: T) -> Self {
        Self::from(vec![output::Item::from(item.into())])
    }
}

pub struct ItemIter {
    value_iter: stack::ItemIter,
}

impl Iterator for ItemIter {
    type Item = output::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.value_iter.next().map(|v| output::Item::from(v))
    }
}

pub struct NodeIter {
    value_iter: stack::ItemIter,
}

impl Iterator for NodeIter {
    type Item = error::Result<xml::Node>;

    fn next(&mut self) -> Option<Self::Item> {
        self.value_iter
            .next()
            .map(|v| v.to_node().map_err(|e| error::Error::from(e)))
    }
}

pub struct AtomizedIter<'a> {
    atomized_iter: stack::AtomizedIter<'a>,
    xot: &'a Xot,
}

impl<'a> Iterator for AtomizedIter<'a> {
    type Item = error::Result<output::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        self.atomized_iter.next().map(|a| {
            a.map(output::Atomic::new)
                .map_err(|_| error::Error::XPTY0004A)
        })
    }
}

pub struct UnboxedAtomizedIter<'a, F> {
    atomized_iter: output::AtomizedIter<'a>,
    extract: F,
}

impl<'a, T, F> Iterator for UnboxedAtomizedIter<'a, F>
where
    F: Fn(&output::Atomic) -> error::Result<T>,
{
    type Item = error::Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.atomized_iter.next().map(|a| (self.extract)(&(a?)))
    }
}

impl occurrence::Occurrence<output::Item, error::Error> for ItemIter {
    fn error(&self) -> error::Error {
        error::Error::XPTY0004A
    }
}

impl<V, U> occurrence::ResultOccurrence<V, error::Error> for U
where
    U: Iterator<Item = error::Result<V>>,
{
    fn error(&self) -> error::Error {
        error::Error::XPTY0004A
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::occurrence::Occurrence;

    #[test]
    fn test_one() {
        let item = output::Item::from(output::Atomic::from(true));
        let sequence = output::Sequence::from(vec![item.clone()]);
        assert_eq!(sequence.items().one().unwrap(), item);
    }
}

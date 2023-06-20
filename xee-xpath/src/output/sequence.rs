use xot::Xot;

use crate::error;
use crate::occurrence;
use crate::output;
use crate::output::item::StackItem;
use crate::stack;

#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    stack_value: stack::Value,
}

impl Sequence {
    pub(crate) fn new(stack_value: stack::Value) -> Self {
        Self { stack_value }
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

    pub fn items(&self) -> ItemIter {
        ItemIter {
            value_iter: stack::ValueIter::new(self.stack_value.clone()),
        }
    }

    pub fn atomized<'a>(&self, xot: &'a Xot) -> AtomizedIter<'a> {
        AtomizedIter {
            atomized_iter: self.stack_value.atomized(xot),
            xot,
        }
    }

    pub fn atomized_sequence(&self, xot: &Xot) -> error::Result<Sequence> {
        // TODO: conceivably we don't consume the iterator here
        let items = self
            .atomized(xot)
            .map(|a| {
                a.map(output::Item::from)
                    .map_err(|_| error::Error::XPTY0004A)
            })
            .collect::<error::Result<Vec<_>>>()?;
        Ok(Sequence::from(&items))
    }

    pub fn unboxed_atomized<'a, T>(
        &self,
        xot: &'a Xot,
        extract: impl Fn(&output::Atomic) -> Option<T>,
    ) -> UnboxedAtomizedIter<'a, impl Fn(&output::Atomic) -> Option<T>> {
        UnboxedAtomizedIter {
            atomized_iter: self.atomized(xot),
            extract,
        }
    }

    pub fn effective_boolean_value(&self) -> error::Result<bool> {
        match self.stack_value.effective_boolean_value() {
            Ok(b) => Ok(b),
            // TODO should handle errors better here
            Err(_) => Err(crate::Error::FORG0006),
        }
    }
}

impl From<stack::Value> for Sequence {
    fn from(stack_value: stack::Value) -> Self {
        Self { stack_value }
    }
}

impl<T> From<T> for Sequence
where
    T: AsRef<[output::Item]>,
{
    fn from(items: T) -> Self {
        let items = items.as_ref();
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

pub struct ItemIter {
    value_iter: stack::ValueIter,
}

impl Iterator for ItemIter {
    type Item = output::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.value_iter
            .next()
            .map(|v| output::Item::StackItem(StackItem(v)))
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
        let sequence = output::Sequence::from(&[item.clone()]);
        assert_eq!(sequence.items().one().unwrap(), item);
    }
}

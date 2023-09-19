use std::cmp::Ordering;

// This contains a sequence abstraction that is useful
// in interfacing with external APIs. It's a layer over the
// stack::Value abstraction
use xot::Xot;

use crate::atomic;
use crate::error;
use crate::occurrence;
use crate::sequence;
use crate::stack;
use crate::string::Collation;
use crate::xml;
use crate::Item;

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
            stack_value: stack::Value::Empty,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack_value.is_empty_sequence()
    }

    pub fn len(&self) -> usize {
        match &self.stack_value {
            stack::Value::Empty => 0,
            stack::Value::One(_) => 1,
            stack::Value::Many(items) => items.len(),
            stack::Value::Absent => panic!("Don't know how to handle absent"),
        }
    }

    pub(crate) fn to_array(&self) -> error::Result<stack::Array> {
        let mut array = Vec::new();
        for item in self.items() {
            let item = item?;
            array.push(item.into());
        }
        Ok(array.into())
    }

    pub fn is_absent(&self) -> bool {
        matches!(&self.stack_value, stack::Value::Absent)
    }

    pub fn ensure_empty(&self) -> error::Result<&Self> {
        if self.is_empty() {
            Ok(self)
        } else {
            Err(error::Error::Type)
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

    pub fn map_iter(&self) -> impl Iterator<Item = error::Result<stack::Map>> {
        self.items().map(|item| item?.to_map())
    }

    pub fn array_iter(&self) -> impl Iterator<Item = error::Result<stack::Array>> {
        self.items().map(|item| item?.to_array())
    }

    pub fn elements<'a>(
        &self,
        xot: &'a Xot,
    ) -> impl Iterator<Item = error::Result<xml::Node>> + 'a {
        self.nodes().map(|n| match n {
            Ok(n) => {
                if n.is_element(xot) {
                    Ok(n)
                } else {
                    Err(error::Error::Type)
                }
            }
            Err(n) => Err(n),
        })
    }

    pub fn atomized<'a>(&self, xot: &'a Xot) -> stack::AtomizedIter<'a> {
        self.stack_value.atomized(xot)
    }

    pub fn unboxed_atomized<'a, T>(
        &self,
        xot: &'a Xot,
        extract: impl Fn(atomic::Atomic) -> error::Result<T>,
    ) -> UnboxedAtomizedIter<'a, impl Fn(atomic::Atomic) -> error::Result<T>> {
        UnboxedAtomizedIter {
            atomized_iter: self.atomized(xot),
            extract,
        }
    }

    pub fn items_atomic(&self) -> ItemAtomicIter {
        ItemAtomicIter {
            value_iter: self.stack_value.items(),
        }
    }

    pub fn effective_boolean_value(&self) -> error::Result<bool> {
        // TODO: error conversion is a bit blunt
        self.stack_value
            .effective_boolean_value()
            .map_err(|_| error::Error::FORG0006)
    }

    pub fn deep_equal(
        &self,
        other: &Sequence,
        collation: &Collation,
        default_offset: chrono::FixedOffset,
        xot: &Xot,
    ) -> error::Result<bool> {
        // https://www.w3.org/TR/xpath-functions-31/#func-deep-equal
        if self.is_empty() && other.is_empty() {
            return Ok(true);
        }
        if self.len() != other.len() {
            return Ok(false);
        }
        for (a, b) in self.items().zip(other.items()) {
            let a = a?;
            let b = b?;
            match (a, b) {
                (Item::Atomic(a), Item::Atomic(b)) => {
                    if !a.deep_equal(&b, collation, default_offset) {
                        return Ok(false);
                    }
                }
                (Item::Node(a), Item::Node(b)) => {
                    if !a.deep_equal(&b, collation, xot) {
                        return Ok(false);
                    }
                }
                (Item::Function(a), Item::Function(b)) => match (a.as_ref(), b.as_ref()) {
                    (stack::Closure::Array(a), stack::Closure::Array(b)) => {
                        if !a.deep_equal(b.clone(), collation, default_offset, xot)? {
                            return Ok(false);
                        }
                    }
                    (stack::Closure::Map(a), stack::Closure::Map(b)) => {
                        if !a.deep_equal(b.clone(), collation, default_offset, xot)? {
                            return Ok(false);
                        }
                    }
                    (stack::Closure::Map(_), stack::Closure::Array(_)) => return Ok(false),
                    (stack::Closure::Array(_), stack::Closure::Map(_)) => return Ok(false),
                    _ => return Err(error::Error::FOTY0015),
                },
                _ => {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    pub(crate) fn fallible_compare(
        &self,
        other: &Sequence,
        collation: &Collation,
        implicit_offset: chrono::FixedOffset,
    ) -> error::Result<Ordering> {
        // we get atoms not by atomizing, but by trying to turn each
        // item into an atom. If it's not an atom, it's not comparable
        // by eq, lt, gt, etc.
        let a_atoms = self.items_atomic();
        let mut b_atoms = other.items_atomic();
        for a_atom in a_atoms {
            let b_atom = b_atoms.next();
            let a_atom = a_atom?;
            if let Some(b_atom) = b_atom {
                let b_atom = b_atom?;
                let ordering = a_atom.fallible_compare(&b_atom, collation, implicit_offset)?;
                if !ordering.is_eq() {
                    return Ok(ordering);
                }
            } else {
                return Ok(Ordering::Greater);
            }
        }
        if b_atoms.next().is_some() {
            Ok(Ordering::Less)
        } else {
            Ok(Ordering::Equal)
        }
    }

    /// For use in sorting. If the comparison fails, it's always Ordering::Less
    /// Another pass is required to determine whether the sequence is in order
    /// or whether the comparison failed.
    pub(crate) fn compare(
        &self,
        other: &Sequence,
        collation: &Collation,
        implicit_offset: chrono::FixedOffset,
    ) -> Ordering {
        self.fallible_compare(other, collation, implicit_offset)
            .unwrap_or(Ordering::Less)
    }
}

impl From<stack::Value> for Sequence {
    fn from(stack_value: stack::Value) -> Self {
        Self { stack_value }
    }
}

impl From<Sequence> for stack::Value {
    fn from(sequence: Sequence) -> Self {
        sequence.stack_value
    }
}

impl From<&stack::Value> for Sequence {
    fn from(stack_value: &stack::Value) -> Self {
        stack_value.clone().into()
    }
}

impl From<sequence::Item> for Sequence {
    fn from(item: sequence::Item) -> Self {
        Self {
            stack_value: item.into(),
        }
    }
}

impl From<Vec<sequence::Item>> for Sequence {
    fn from(items: Vec<sequence::Item>) -> Self {
        Self {
            stack_value: items.into(),
        }
    }
}

impl From<Vec<xml::Node>> for Sequence {
    fn from(items: Vec<xml::Node>) -> Self {
        let items = items
            .into_iter()
            .map(sequence::Item::from)
            .collect::<Vec<_>>();
        items.into()
    }
}

impl From<stack::Array> for Sequence {
    fn from(array: stack::Array) -> Self {
        Self {
            stack_value: array.into(),
        }
    }
}

impl From<stack::Map> for Sequence {
    fn from(map: stack::Map) -> Self {
        Self {
            stack_value: map.into(),
        }
    }
}

impl<T> From<Option<T>> for Sequence
where
    T: Into<sequence::Item>,
{
    fn from(item: Option<T>) -> Self {
        match item {
            Some(item) => Self::from(vec![item.into()]),
            None => Sequence::empty(),
        }
    }
}

impl<T> From<Vec<T>> for Sequence
where
    T: Into<atomic::Atomic>,
{
    fn from(items: Vec<T>) -> Self {
        let items = items
            .into_iter()
            .map(|i| sequence::Item::from(i.into()))
            .collect::<Vec<_>>();
        Self::from(items)
    }
}

impl<T> From<T> for Sequence
where
    T: Into<atomic::Atomic>,
{
    fn from(item: T) -> Self {
        Self::from(vec![sequence::Item::from(item.into())])
    }
}

pub struct ItemIter {
    value_iter: stack::ValueIter,
}

impl Iterator for ItemIter {
    type Item = error::Result<sequence::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.value_iter.next().map(|r| r.map(sequence::Item::from))
    }
}

pub struct NodeIter {
    value_iter: stack::ValueIter,
}

impl Iterator for NodeIter {
    type Item = error::Result<xml::Node>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.value_iter.next();
        match next {
            None => None,
            Some(Err(e)) => Some(Err(e)),
            Some(Ok(v)) => Some(v.to_node()),
        }
    }
}

pub struct UnboxedAtomizedIter<'a, F> {
    atomized_iter: stack::AtomizedIter<'a>,
    extract: F,
}

impl<'a, T, F> Iterator for UnboxedAtomizedIter<'a, F>
where
    F: Fn(atomic::Atomic) -> error::Result<T>,
{
    type Item = error::Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.atomized_iter.next().map(|a| (self.extract)(a?))
    }
}

impl<V, U> occurrence::Occurrence<V, error::Error> for U
where
    V: std::fmt::Debug,
    U: Iterator<Item = error::Result<V>>,
{
    fn error(&self) -> error::Error {
        error::Error::Type
    }
}

pub struct ItemAtomicIter {
    value_iter: stack::ValueIter,
}

impl Iterator for ItemAtomicIter {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        self.value_iter.next().map(|r| r?.to_atomic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::occurrence::Occurrence;

    #[test]
    fn test_one() {
        let item = sequence::Item::from(atomic::Atomic::from(true));
        let sequence = Sequence::from(vec![item.clone()]);
        assert_eq!(sequence.items().one().unwrap(), item);
    }
}

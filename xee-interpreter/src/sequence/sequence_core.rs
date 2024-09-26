use std::cmp::Ordering;

// This contains a sequence abstraction that is useful
// in interfacing with external APIs. It's a layer over the
// stack::Value abstraction
use xot::Xot;

use crate::atomic;
use crate::context;
use crate::error;
use crate::function;
use crate::occurrence;
use crate::sequence;
use crate::sequence::Item;
use crate::stack;
use crate::string::Collation;

/// A XPath sequence of items.
///
/// <https://www.w3.org/TR/xpath-datamodel-31/#sequences>
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    stack_value: stack::Value,
}

impl Sequence {
    pub(crate) fn new(stack_value: stack::Value) -> Self {
        Self { stack_value }
    }

    /// Construct an empty sequence
    pub fn empty() -> Self {
        Self {
            stack_value: stack::Value::Empty,
        }
    }

    /// Check whether the sequence is empty
    pub fn is_empty(&self) -> bool {
        self.stack_value.is_empty_sequence()
    }

    /// Get the length of the sequence
    pub fn len(&self) -> usize {
        match &self.stack_value {
            stack::Value::Empty => 0,
            stack::Value::One(_) => 1,
            stack::Value::Many(items) => items.len(),
            stack::Value::Absent => panic!("Don't know how to handle absent"),
        }
    }

    pub(crate) fn to_array(&self) -> error::Result<function::Array> {
        let mut array = Vec::new();
        for item in self.items()? {
            array.push(item.into());
        }
        Ok(array.into())
    }

    /// Check whether this sequence is the absent value
    pub fn is_absent(&self) -> bool {
        matches!(&self.stack_value, stack::Value::Absent)
    }

    /// Ensure the sequence is empty, and if not, return XPTY0004 error.
    pub fn ensure_empty(&self) -> error::Result<&Self> {
        if self.is_empty() {
            Ok(self)
        } else {
            Err(error::Error::XPTY0004)
        }
    }

    /// Access an iterator over the items in the sequence
    ///
    /// This is fallible, as an Absent value is not iterable.
    pub fn items(&self) -> error::Result<ItemIter> {
        Ok(ItemIter {
            value_iter: self.stack_value.items()?,
        })
    }

    /// Access an iterator over the nodes in the sequence
    ///
    /// This is fallible as an Absent value is not iterable.
    ///
    /// An error is returned for items that are not a node.
    pub fn nodes(&self) -> error::Result<NodeIter> {
        Ok(NodeIter {
            value_iter: self.stack_value.items()?,
        })
    }

    /// Access an iterator over the XPath maps in the sequence
    ///
    /// This is fallible as an Absent value is not iterable.
    ///
    /// An error is returned for items that are not a map.
    pub fn map_iter(&self) -> error::Result<impl Iterator<Item = error::Result<function::Map>>> {
        Ok(self.items()?.map(|item| item.to_map()))
    }

    /// Access an iterator over the XPath arrays in the sequence
    ///
    /// This is fallible as an Absent value is not iterable.
    ///
    /// An error is returned for items that are not an array.
    pub fn array_iter(
        &self,
    ) -> error::Result<impl Iterator<Item = error::Result<function::Array>>> {
        Ok(self.items()?.map(|item| item.to_array()))
    }

    /// Access an iterator over elements nodes in the sequence
    ///
    /// This is fallible as an Absent value is not iterable.
    ///
    /// An error is returned for items that are not an element.
    pub fn elements<'a>(
        &self,
        xot: &'a Xot,
    ) -> error::Result<impl Iterator<Item = error::Result<xot::Node>> + 'a> {
        Ok(self.nodes()?.map(|n| match n {
            Ok(n) => {
                if xot.is_element(n) {
                    Ok(n)
                } else {
                    Err(error::Error::XPTY0004)
                }
            }
            Err(n) => Err(n),
        }))
    }

    /// Access an iterator over the atomized values in the sequence
    ///
    /// <https://www.w3.org/TR/xpath-31/#id-atomization>
    pub fn atomized<'a>(&self, xot: &'a Xot) -> stack::AtomizedIter<'a> {
        self.stack_value.atomized(xot)
    }

    /// Is used internally by the library macro.
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

    pub fn items_atomic(&self) -> error::Result<ItemAtomicIter> {
        Ok(ItemAtomicIter {
            value_iter: self.stack_value.items()?,
        })
    }

    /// Get the effective boolean value of the sequence
    ///
    /// <https://www.w3.org/TR/xpath-31/#id-ebv>
    pub fn effective_boolean_value(&self) -> error::Result<bool> {
        // TODO: error conversion is a bit blunt
        self.stack_value
            .effective_boolean_value()
            .map_err(|_| error::Error::FORG0006)
    }

    /// Concatenate two sequences producing a new sequence.
    pub fn concat(&self, other: &sequence::Sequence) -> Self {
        let a: stack::Value = self.clone().into();
        let b: stack::Value = other.clone().into();
        a.concat(b).into()
    }

    /// Compare two sequences using XPath deep equal rules.
    ///
    /// <https://www.w3.org/TR/xpath-functions-31/#func-deep-equal>
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
        for (a, b) in self.items()?.zip(other.items()?) {
            match (a, b) {
                (Item::Atomic(a), Item::Atomic(b)) => {
                    if !a.deep_equal(&b, collation, default_offset) {
                        return Ok(false);
                    }
                }
                (Item::Node(a), Item::Node(b)) => {
                    if !xot.deep_equal_xpath(a, b, |a, b| collation.compare(a, b).is_eq()) {
                        return Ok(false);
                    }
                }
                (Item::Function(a), Item::Function(b)) => match (a.as_ref(), b.as_ref()) {
                    (function::Function::Array(a), function::Function::Array(b)) => {
                        if !a.deep_equal(b.clone(), collation, default_offset, xot)? {
                            return Ok(false);
                        }
                    }
                    (function::Function::Map(a), function::Function::Map(b)) => {
                        if !a.deep_equal(b.clone(), collation, default_offset, xot)? {
                            return Ok(false);
                        }
                    }
                    (function::Function::Map(_), function::Function::Array(_)) => return Ok(false),
                    (function::Function::Array(_), function::Function::Map(_)) => return Ok(false),
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
        let a_atoms = self.items_atomic()?;
        let mut b_atoms = other.items_atomic()?;
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

    pub fn sorted(
        &self,
        context: &context::DynamicContext,
        collation: &str,
        xot: &Xot,
    ) -> error::Result<Self> {
        self.sorted_by_key(context, collation, |item| {
            // the equivalent of fn:data()
            let seq: sequence::Sequence = item.clone().into();
            let atoms = seq.atomized(xot).collect::<error::Result<Vec<_>>>()?;
            Ok(atoms.into())
        })
    }

    pub fn sorted_by_key<F>(
        &self,
        context: &context::DynamicContext,
        collation: &str,
        mut get: F,
    ) -> error::Result<Self>
    where
        F: FnMut(&sequence::Item) -> error::Result<sequence::Sequence>,
    {
        // see also sort_by_sequence in array.rs. The signatures are
        // sufficiently different we don't want to try to unify them.

        let collation = context.static_context.collation(collation)?;
        let items = self.items()?.collect::<Vec<_>>();
        let keys = self
            .items()?
            .map(|key| get(&key))
            .collect::<error::Result<Vec<_>>>()?;

        let mut keys_and_items = keys.into_iter().zip(items).collect::<Vec<_>>();
        // sort by key. unfortunately sort_by requires the compare function
        // to be infallible. It's not in reality, so we make any failures
        // sort less, so they appear early on in the sequence.
        keys_and_items.sort_by(|(a_key, _), (b_key, _)| {
            a_key.compare(b_key, &collation, context.implicit_timezone())
        });
        // a pass to detect any errors; if sorting between two items is
        // impossible we want to raise a type error
        for ((a_key, _), (b_key, _)) in keys_and_items.iter().zip(keys_and_items.iter().skip(1)) {
            a_key.fallible_compare(b_key, &collation, context.implicit_timezone())?;
        }
        // now pick up items again
        let items = keys_and_items
            .into_iter()
            .map(|(_, item)| item)
            .collect::<Vec<_>>();
        Ok(items.into())
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

impl From<Vec<xot::Node>> for Sequence {
    fn from(items: Vec<xot::Node>) -> Self {
        let items = items
            .into_iter()
            .map(sequence::Item::from)
            .collect::<Vec<_>>();
        items.into()
    }
}

impl From<function::Array> for Sequence {
    fn from(array: function::Array) -> Self {
        Self {
            stack_value: array.into(),
        }
    }
}

impl From<function::Map> for Sequence {
    fn from(map: function::Map) -> Self {
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

/// An iterator over the items in a sequence.
pub struct ItemIter {
    value_iter: stack::ValueIter,
}

impl occurrence::Occurrence<Item, error::Error> for ItemIter {
    fn one(&mut self) -> Result<Item, error::Error> {
        if let Some(one) = self.next() {
            if self.next().is_none() {
                Ok(one)
            } else {
                Err(self.error())
            }
        } else {
            Err(self.error())
        }
    }

    fn option(&mut self) -> Result<Option<Item>, error::Error> {
        if let Some(one) = self.next() {
            if self.next().is_none() {
                Ok(Some(one))
            } else {
                Err(self.error())
            }
        } else {
            Ok(None)
        }
    }

    fn many(&mut self) -> Result<Vec<Item>, error::Error> {
        Ok(self.collect::<Vec<_>>())
    }

    fn error(&self) -> error::Error {
        error::Error::XPTY0004
    }
}

impl Iterator for ItemIter {
    type Item = sequence::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.value_iter.next()
    }
}

/// An iterator over the nodes in a sequence.
pub struct NodeIter {
    value_iter: stack::ValueIter,
}

impl Iterator for NodeIter {
    type Item = error::Result<xot::Node>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.value_iter.next();
        next.map(|v| v.to_node())
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
    fn one(&mut self) -> Result<V, error::Error> {
        if let Some(one) = self.next() {
            if self.next().is_none() {
                Ok(one?)
            } else {
                Err(self.error())
            }
        } else {
            Err(self.error())
        }
    }

    fn option(&mut self) -> Result<Option<V>, error::Error> {
        if let Some(one) = self.next() {
            if self.next().is_none() {
                Ok(Some(one?))
            } else {
                Err(self.error())
            }
        } else {
            Ok(None)
        }
    }

    fn many(&mut self) -> Result<Vec<V>, error::Error> {
        self.collect::<Result<Vec<_>, _>>()
    }

    fn error(&self) -> error::Error {
        error::Error::XPTY0004
    }
}

pub struct ItemAtomicIter {
    value_iter: stack::ValueIter,
}

impl Iterator for ItemAtomicIter {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        self.value_iter.next().map(|r| r.to_atomic())
    }
}

#[cfg(test)]
mod tests {
    use occurrence::Occurrence;

    use super::*;

    #[test]
    fn test_one() {
        let item = sequence::Item::from(atomic::Atomic::from(true));
        let sequence = Sequence::from(vec![item.clone()]);
        assert_eq!(sequence.items().unwrap().one().unwrap(), item);
    }
}
